use std::process::Command;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

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

fn main() -> Result<()> {
    // Default rules for a typical Rust project
    let mut allowed_extensions_by_dir = HashMap::new();
    allowed_extensions_by_dir.insert("src".to_string(), vec![".rs".to_string()]);
    allowed_extensions_by_dir.insert("docs".to_string(), vec![".md".to_string(), ".txt".to_string()]);
    allowed_extensions_by_dir.insert("examples".to_string(), vec![".rs".to_string(), ".md".to_string()]);
    allowed_extensions_by_dir.insert("tests".to_string(), vec![".rs".to_string()]);
    allowed_extensions_by_dir.insert("scripts".to_string(), vec![".rs".to_string(), ".sh".to_string()]);
    allowed_extensions_by_dir.insert("config".to_string(), vec![".toml".to_string(), ".yml".to_string(), ".yaml".to_string(), ".json".to_string(), ".jsonc".to_string()]);
    allowed_extensions_by_dir.insert("schemas".to_string(), vec![".json".to_string(), ".jsonc".to_string()]);

    let rules = FileRuleSet {
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
    };

    match check_ls_files(&rules) {
        Ok(violations) => {
            if violations.is_empty() {
                println!("✅ All file structure rules passed");
                std::process::exit(0);
            } else {
                eprintln!("❌ Found {} file structure violations:", violations.len());
                for violation in violations {
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
        Err(e) => {
            eprintln!("❌ Failed to check file structure: {}", e);
            std::process::exit(1);
        }
    }
}
