use dircheck_core::{TreeRuleSet, validate_tree_commit};
use std::process::{Command, exit};
use anyhow::Result;

fn main() -> Result<()> {
    println!("🔍 Running pre-commit validation...");
    
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ git ls-tree failed: {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }
    
    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout.lines().map(str::to_string).collect();
    
    let rules = TreeRuleSet::default();
    let violations = validate_tree_commit(&paths, &rules);
    
    if !violations.is_empty() {
        eprintln!("❌ Found {} directory structure violations:", violations.len());
        for violation in violations {
            eprintln!("  - {}: {}", violation.path, violation.message);
        }
        eprintln!("❌ Pre-commit validation failed");
        exit(1);
    }
    
    println!("✅ Pre-commit validation passed");
    Ok(())
}
