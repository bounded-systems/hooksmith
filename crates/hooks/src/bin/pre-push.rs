use dircheck_core::{TreeRuleSet, FileRuleSet, validate_tree_commit, validate_files_index};
use std::process::{Command, exit};
use anyhow::Result;

fn main() -> Result<()> {
    println!("🔍 Running pre-push validation...");
    
    // Run tree validation
    println!("📋 Validating directory structure...");
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ git ls-tree failed: {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }
    
    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout.lines().map(str::to_string).collect();
    
    let tree_rules = TreeRuleSet::default();
    let tree_violations = validate_tree_commit(&paths, &tree_rules);
    
    if !tree_violations.is_empty() {
        eprintln!("❌ Found {} directory structure violations:", tree_violations.len());
        for violation in tree_violations {
            eprintln!("  - {}: {}", violation.path, violation.message);
        }
        eprintln!("❌ Directory structure validation failed");
        exit(1);
    }
    
    // Run file validation
    println!("📋 Validating file structure...");
    let output = Command::new("git")
        .args(["ls-files"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ git ls-files failed: {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }
    
    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout.lines().map(str::to_string).collect();
    
    let file_rules = FileRuleSet::default();
    let file_violations = validate_files_index(&paths, &file_rules);
    
    if !file_violations.is_empty() {
        eprintln!("❌ Found {} file structure violations:", file_violations.len());
        for violation in file_violations {
            eprintln!("  - {}: {}", violation.path, violation.message);
        }
        eprintln!("❌ File structure validation failed");
        exit(1);
    }
    
    println!("✅ Pre-push validation passed");
    Ok(())
}
