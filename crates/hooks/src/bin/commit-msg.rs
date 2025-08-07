use anyhow::Result;
use std::env;
use std::fs;

fn main() -> Result<()> {
    println!("🔍 Validating commit message...");
    
    // Get the commit message file path from command line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("❌ No commit message file specified");
        std::process::exit(1);
    }
    
    let msg_file = &args[1];
    let commit_msg = fs::read_to_string(msg_file)?;
    
    // Basic commit message validation
    let mut issues = Vec::new();
    
    // Check if message is not empty
    if commit_msg.trim().is_empty() {
        issues.push("Commit message cannot be empty".to_string());
    }
    
    // Check message length
    if commit_msg.len() > 500 {
        issues.push("Commit message is too long (>500 characters)".to_string());
    }
    
    // Check for common patterns
    let lines: Vec<&str> = commit_msg.lines().collect();
    if !lines.is_empty() {
        let first_line = lines[0];
        
        // Check first line length (should be <= 72 chars)
        if first_line.len() > 72 {
            issues.push("First line of commit message is too long (>72 characters)".to_string());
        }
        
        // Check for common prefixes (optional but recommended)
        let prefixes = ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "chore:"];
        let has_prefix = prefixes.iter().any(|prefix| first_line.starts_with(prefix));
        
        if !has_prefix && !first_line.starts_with("Merge") && !first_line.starts_with("Revert") {
            issues.push("Consider using conventional commit prefixes (feat:, fix:, docs:, etc.)".to_string());
        }
    }
    
    // Check for common issues
    if commit_msg.contains("TODO") || commit_msg.contains("FIXME") {
        issues.push("Commit message contains TODO/FIXME - consider addressing these first".to_string());
    }
    
    if !issues.is_empty() {
        eprintln!("❌ Commit message validation failed:");
        for issue in issues {
            eprintln!("  {}", issue);
        }
        eprintln!("💡 Please fix the commit message and try again");
        std::process::exit(1);
    }
    
    println!("✅ Commit message validation passed");
    Ok(())
}
