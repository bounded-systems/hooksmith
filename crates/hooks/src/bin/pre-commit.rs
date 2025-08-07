use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
    println!("🔍 Running pre-commit validation...");
    
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()?;
    
    if !status.success() {
        eprintln!("❌ Not in a git repository");
        std::process::exit(1);
    }
    
    // Check for staged files
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()?;
    
    if output.stdout.is_empty() {
        println!("✅ No staged files to validate");
        return Ok(());
    }
    
    let staged_files = String::from_utf8(output.stdout)?;
    let file_count = staged_files.lines().count();
    
    println!("📁 Found {} staged file(s)", file_count);
    
    // Basic validation: check for common issues
    let mut issues = Vec::new();
    
    for line in staged_files.lines() {
        let file = line.trim();
        if file.is_empty() {
            continue;
        }
        
        // Check for common problematic patterns
        if file.contains("TODO") || file.contains("FIXME") {
            issues.push(format!("⚠️  {} contains TODO/FIXME", file));
        }
        
        // Check for large files (over 1MB)
        if let Ok(metadata) = std::fs::metadata(file) {
            if metadata.len() > 1_048_576 { // 1MB
                issues.push(format!("⚠️  {} is large (>1MB)", file));
            }
        }
    }
    
    if !issues.is_empty() {
        eprintln!("❌ Pre-commit validation found issues:");
        for issue in issues {
            eprintln!("  {}", issue);
        }
        eprintln!("💡 Please review and fix these issues before committing");
        std::process::exit(1);
    }
    
    println!("✅ Pre-commit validation passed");
    Ok(())
}
