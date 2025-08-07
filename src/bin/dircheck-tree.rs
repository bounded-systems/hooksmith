use std::process::Command;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
struct TreeRuleSet {
    allowed_root_dirs: Vec<String>,
    forbidden_dirs: Vec<String>,
    required_dirs: Vec<String>,
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

fn main() -> Result<()> {
    // Default rules for a typical Rust project
    let rules = TreeRuleSet {
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
    };

    match check_ls_tree(&rules) {
        Ok(violations) => {
            if violations.is_empty() {
                println!("✅ All directory structure rules passed");
                std::process::exit(0);
            } else {
                eprintln!("❌ Found {} directory structure violations:", violations.len());
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
            eprintln!("❌ Failed to check directory structure: {}", e);
            std::process::exit(1);
        }
    }
}
