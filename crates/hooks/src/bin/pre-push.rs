use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
    println!("🔍 Running pre-push validation...");
    
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()?;
    
    if !status.success() {
        eprintln!("❌ Not in a git repository");
        std::process::exit(1);
    }
    
    // Check for uncommitted changes
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    
    let status_output = String::from_utf8(output.stdout)?;
    let uncommitted_files: Vec<&str> = status_output
        .lines()
        .filter(|line| !line.starts_with("??")) // Ignore untracked files
        .collect();
    
    if !uncommitted_files.is_empty() {
        eprintln!("❌ Found uncommitted changes:");
        for file in uncommitted_files {
            eprintln!("  {}", file);
        }
        eprintln!("💡 Please commit or stash changes before pushing");
        std::process::exit(1);
    }
    
    // Check if we have commits to push
    let output = Command::new("git")
        .args(["log", "--oneline", "@{u}..HEAD"])
        .output()?;
    
    let commits_to_push = String::from_utf8(output.stdout)?;
    if commits_to_push.trim().is_empty() {
        println!("✅ No commits to push");
        return Ok(());
    }
    
    let commit_count = commits_to_push.lines().count();
    println!("📤 Found {} commit(s) to push", commit_count);
    
    // Basic validation: check commit messages
    let output = Command::new("git")
        .args(["log", "--oneline", "@{u}..HEAD"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    let mut issues = Vec::new();
    
    for line in commits.lines() {
        if let Some(message) = line.splitn(2, ' ').nth(1) {
            if message.len() < 10 {
                issues.push(format!("⚠️  Short commit message: {}", message));
            }
            if message.contains("WIP") || message.contains("wip") {
                issues.push(format!("⚠️  WIP commit detected: {}", message));
            }
        }
    }
    
    if !issues.is_empty() {
        eprintln!("❌ Pre-push validation found issues:");
        for issue in issues {
            eprintln!("  {}", issue);
        }
        eprintln!("💡 Please review commit messages before pushing");
        std::process::exit(1);
    }
    
    println!("✅ Pre-push validation passed");
    Ok(())
}
