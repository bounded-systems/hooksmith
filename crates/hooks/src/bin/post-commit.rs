use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
    println!("🎉 Post-commit actions...");
    
    // Get the current commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;
    
    let commit_hash = String::from_utf8(output.stdout)?.trim().to_string();
    let short_hash = &commit_hash[..8];
    
    // Get commit message
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .output()?;
    
    let commit_msg = String::from_utf8(output.stdout)?;
    
    println!("✅ Commit {} created: {}", short_hash, commit_msg);
    
    // Show repository status
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()?;
    
    let status = String::from_utf8(output.stdout)?;
    if !status.trim().is_empty() {
        println!("📁 Working directory status:");
        for line in status.lines() {
            if !line.trim().is_empty() {
                println!("  {}", line);
            }
        }
    } else {
        println!("📁 Working directory is clean");
    }
    
    // Show recent commits
    let output = Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output()?;
    
    let recent_commits = String::from_utf8(output.stdout)?;
    println!("📜 Recent commits:");
    for line in recent_commits.lines() {
        println!("  {}", line);
    }
    
    println!("✅ Post-commit actions completed");
    Ok(())
}
