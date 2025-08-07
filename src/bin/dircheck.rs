use std::process::Command;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dircheck")]
#[command(about = "Validate directory and file structure in Git repositories")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate directory structure using git ls-tree
    Tree,
    /// Validate file structure using git ls-files
    Files,
    /// Run both tree and file validation
    All,
}

#[derive(Debug, Serialize, Deserialize)]
struct TreeRuleSet {
    allowed_root_dirs: Vec<String>,
    forbidden_dirs: Vec<String>,
    required_dirs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileRuleSet {
    forbidden_root_extensions: Vec<String>,
    allowed_extensions_by_dir: HashMap<String, Vec<String>>,
    forbidden_files: Vec<String>,
    required_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Violation {
    rule: String,
    path: String,
    message: String,
    suggestion: Option<String>,
}

fn run_git_ls_tree() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .output()
        .context("Failed to run git ls-tree")?;

    if !output.status.success() {
        anyhow::bail!("git ls-tree failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(paths)
}

fn run_git_ls_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to run git ls-files")?;

    if !output.status.success() {
        anyhow::bail!("git ls-files failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(paths)
}

fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| format!(".{}", s))
}

fn get_directory(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| if s.is_empty() { ".".to_string() } else { s.to_string() })
}

fn check_ls_tree(rules: &TreeRuleSet) -> Result<Vec<Violation>> {
    let tree_paths = run_git_ls_tree()?;
    let mut violations = Vec::new();

    // Get root directories from paths
    let mut root_dirs = std::collections::HashSet::new();
    for path in &tree_paths {
        if let Some(first_component) = path.split('/').next() {
            root_dirs.insert(first_component.to_string());
        }
    }

    // Check allowed root directories
    for root_dir in &root_dirs {
        if !rules.allowed_root_dirs.contains(root_dir) {
            violations.push(Violation {
                rule: "allowed_root_dirs".to_string(),
                path: root_dir.clone(),
                message: format!("Root directory '{}' is not in allowed list", root_dir),
                suggestion: Some(format!("Add '{}' to allowed_root_dirs or remove it", root_dir)),
            });
        }
    }

    // Check forbidden directories
    for forbidden_dir in &rules.forbidden_dirs {
        if root_dirs.contains(forbidden_dir) {
            violations.push(Violation {
                rule: "forbidden_dirs".to_string(),
                path: forbidden_dir.clone(),
                message: format!("Forbidden directory '{}' found in root", forbidden_dir),
                suggestion: Some(format!("Remove '{}' directory", forbidden_dir)),
            });
        }
    }

    // Check required directories
    for required_dir in &rules.required_dirs {
        if !root_dirs.contains(required_dir) {
            violations.push(Violation {
                rule: "required_dirs".to_string(),
                path: required_dir.clone(),
                message: format!("Required directory '{}' not found", required_dir),
                suggestion: Some(format!("Create '{}' directory", required_dir)),
            });
        }
    }

    Ok(violations)
}

fn check_ls_files(rules: &FileRuleSet) -> Result<Vec<Violation>> {
    let file_paths = run_git_ls_files()?;
    let mut violations = Vec::new();

    for path in &file_paths {
        // Check forbidden root extensions
        if let Some(ext) = get_file_extension(path) {
            if rules.forbidden_root_extensions.contains(&ext) {
                // Check if file is in root directory
                if get_directory(path).map_or(true, |dir| dir == ".") {
                    violations.push(Violation {
                        rule: "forbidden_root_extensions".to_string(),
                        path: path.clone(),
                        message: format!("File with forbidden extension '{}' found in root", ext),
                        suggestion: Some(format!("Move '{}' to appropriate subdirectory", path)),
                    });
                }
            }
        }

        // Check allowed extensions by directory
        if let Some(dir) = get_directory(path) {
            if let Some(allowed_extensions) = rules.allowed_extensions_by_dir.get(&dir) {
                if let Some(ext) = get_file_extension(path) {
                    if !allowed_extensions.contains(&ext) {
                        violations.push(Violation {
                            rule: "allowed_extensions_by_dir".to_string(),
                            path: path.clone(),
                            message: format!("File with extension '{}' not allowed in directory '{}'", ext, dir),
                            suggestion: Some(format!("Move '{}' to directory that allows '{}' extension", path, ext)),
                        });
                    }
                }
            }
        }

        // Check forbidden files
        if rules.forbidden_files.contains(path) {
            violations.push(Violation {
                rule: "forbidden_files".to_string(),
                path: path.clone(),
                message: format!("Forbidden file '{}' found", path),
                suggestion: Some(format!("Remove '{}' file", path)),
            });
        }
    }

    // Check required files
    for required_file in &rules.required_files {
        if !file_paths.contains(required_file) {
            violations.push(Violation {
                rule: "required_files".to_string(),
                path: required_file.clone(),
                message: format!("Required file '{}' not found", required_file),
                suggestion: Some(format!("Create '{}' file", required_file)),
            });
        }
    }

    Ok(violations)
}

fn get_default_tree_rules() -> TreeRuleSet {
    TreeRuleSet {
        allowed_root_dirs: vec![
            "src".to_string(),
            "docs".to_string(),
            "crates".to_string(),
            "examples".to_string(),
            "tests".to_string(),
            "scripts".to_string(),
            "config".to_string(),
            "schemas".to_string(),
            "hooks".to_string(),
            "target".to_string(),
            ".github".to_string(),
            ".git".to_string(),
        ],
        forbidden_dirs: vec![
            "node_modules".to_string(),
            "vendor".to_string(),
            "tmp".to_string(),
            "temp".to_string(),
        ],
        required_dirs: vec![
            "src".to_string(),
        ],
    }
}

fn get_default_file_rules() -> FileRuleSet {
    let mut allowed_extensions_by_dir = HashMap::new();
    allowed_extensions_by_dir.insert("src".to_string(), vec![".rs".to_string()]);
    allowed_extensions_by_dir.insert("docs".to_string(), vec![".md".to_string(), ".txt".to_string()]);
    allowed_extensions_by_dir.insert("examples".to_string(), vec![".rs".to_string(), ".md".to_string()]);
    allowed_extensions_by_dir.insert("tests".to_string(), vec![".rs".to_string()]);
    allowed_extensions_by_dir.insert("scripts".to_string(), vec![".rs".to_string(), ".sh".to_string()]);
    allowed_extensions_by_dir.insert("config".to_string(), vec![".toml".to_string(), ".yml".to_string(), ".yaml".to_string(), ".json".to_string(), ".jsonc".to_string()]);
    allowed_extensions_by_dir.insert("schemas".to_string(), vec![".json".to_string(), ".jsonc".to_string()]);

    FileRuleSet {
        forbidden_root_extensions: vec![
            ".md".to_string(),
            ".toml".to_string(),
            ".rs".to_string(),
        ],
        allowed_extensions_by_dir,
        forbidden_files: vec![
            "Cargo.lock".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
        ],
        required_files: vec![
            "Cargo.toml".to_string(),
            "README.md".to_string(),
        ],
    }
}

fn print_violations(violations: &[Violation], check_type: &str) {
    if violations.is_empty() {
        println!("✅ All {} rules passed", check_type);
    } else {
        eprintln!("❌ Found {} {} violations:", violations.len(), check_type);
        for violation in violations {
            eprintln!("  Rule: {}", violation.rule);
            eprintln!("  Path: {}", violation.path);
            eprintln!("  Error: {}", violation.message);
            if let Some(suggestion) = &violation.suggestion {
                eprintln!("  Suggestion: {}", suggestion);
            }
            eprintln!();
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tree => {
            let rules = get_default_tree_rules();
            match check_ls_tree(&rules) {
                Ok(violations) => {
                    print_violations(&violations, "directory structure");
                    if violations.is_empty() {
                        std::process::exit(0);
                    } else {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to check directory structure: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Files => {
            let rules = get_default_file_rules();
            match check_ls_files(&rules) {
                Ok(violations) => {
                    print_violations(&violations, "file structure");
                    if violations.is_empty() {
                        std::process::exit(0);
                    } else {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to check file structure: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::All => {
            let tree_rules = get_default_tree_rules();
            let file_rules = get_default_file_rules();
            
            let mut all_violations = Vec::new();
            let mut has_errors = false;

            // Check tree structure
            match check_ls_tree(&tree_rules) {
                Ok(violations) => {
                    if !violations.is_empty() {
                        has_errors = true;
                        all_violations.extend(violations);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to check directory structure: {}", e);
                    std::process::exit(1);
                }
            }

            // Check file structure
            match check_ls_files(&file_rules) {
                Ok(violations) => {
                    if !violations.is_empty() {
                        has_errors = true;
                        all_violations.extend(violations);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to check file structure: {}", e);
                    std::process::exit(1);
                }
            }

            if all_violations.is_empty() {
                println!("✅ All directory and file structure rules passed");
                std::process::exit(0);
            } else {
                eprintln!("❌ Found {} total violations:", all_violations.len());
                for violation in all_violations {
                    eprintln!("  Rule: {}", violation.rule);
                    eprintln!("  Path: {}", violation.path);
                    eprintln!("  Error: {}", violation.message);
                    if let Some(suggestion) = violation.suggestion {
                        eprintln!("  Suggestion: {}", suggestion);
                    }
                    eprintln!();
                }
                std::process::exit(1);
            }
        }
    }
}
