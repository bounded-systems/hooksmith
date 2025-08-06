#!/usr/bin/env rust-script
//! Update all worktrees to latest origin/main
//! This script will rebase each worktree's base branch to origin/main

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Updating all worktrees to latest origin/main...");

    // Get the latest from origin
    println!("📥 Fetching latest from origin...");
    let fetch_status = Command::new("git")
        .args(&["fetch", "origin"])
        .status()?;

    if !fetch_status.success() {
        eprintln!("❌ Failed to fetch from origin");
        std::process::exit(1);
    }

    // Get current directory
    let current_dir = env::current_dir()?;
    
    // Get list of worktrees
    let worktree_output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(worktree_output.stdout)?;
    
    for line in worktree_list.lines() {
        if line.starts_with("worktree ") {
            let worktree_path = line.split_whitespace().nth(1).unwrap_or("");
            
            // Skip the main worktree
            if worktree_path == current_dir.to_string_lossy() {
                continue;
            }
            
            if Path::new(worktree_path).exists() {
                println!();
                println!("🔄 Updating worktree: {}", Path::new(worktree_path).file_name().unwrap_or_default().to_string_lossy());
                println!("   Path: {}", worktree_path);
                
                // Get the branch name for this worktree
                let branch_output = Command::new("git")
                    .args(&["branch", "--show-current"])
                    .current_dir(worktree_path)
                    .stdout(Stdio::piped())
                    .output()?;
                
                let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
                println!("   Branch: {}", branch);
                
                // Check if there are uncommitted changes
                let status_output = Command::new("git")
                    .args(&["status", "--porcelain"])
                    .current_dir(worktree_path)
                    .stdout(Stdio::piped())
                    .output()?;
                
                let status = String::from_utf8(status_output.stdout)?;
                
                if !status.trim().is_empty() {
                    println!("   ⚠️  WARNING: Uncommitted changes detected!");
                    println!("   📝 Changes:");
                    for line in status.lines() {
                        if !line.trim().is_empty() {
                            println!("      {}", line);
                        }
                    }
                    println!("   💡 Consider committing or stashing changes before updating");
                    println!("   ⏭️  Skipping this worktree...");
                    continue;
                }
                
                // Get current commit
                let commit_output = Command::new("git")
                    .args(&["log", "--oneline", "-1"])
                    .current_dir(worktree_path)
                    .stdout(Stdio::piped())
                    .output()?;
                
                let current_commit = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();
                println!("   Current commit: {}", current_commit);
                
                // Check how many commits behind origin/main
                let behind_output = Command::new("git")
                    .args(&["rev-list", "--count", "HEAD..origin/main"])
                    .current_dir(worktree_path)
                    .stdout(Stdio::piped())
                    .output()?;
                
                let behind_count = String::from_utf8_lossy(&behind_output.stdout).trim().to_string();
                println!("   Behind origin/main by: {} commits", behind_count);
                
                if behind_count == "0" {
                    println!("   ✅ Already up to date!");
                    continue;
                }
                
                // Rebase to origin/main
                println!("   🔄 Rebasing to origin/main...");
                let rebase_status = Command::new("git")
                    .args(&["rebase", "origin/main"])
                    .current_dir(worktree_path)
                    .status()?;
                
                if rebase_status.success() {
                    println!("   ✅ Successfully updated!");
                    
                    // Get new commit
                    let new_commit_output = Command::new("git")
                        .args(&["log", "--oneline", "-1"])
                        .current_dir(worktree_path)
                        .stdout(Stdio::piped())
                        .output()?;
                    
                    let new_commit = String::from_utf8_lossy(&new_commit_output.stdout).trim().to_string();
                    println!("   New commit: {}", new_commit);
                } else {
                    println!("   ❌ Rebase failed! Manual intervention may be needed.");
                    println!("   💡 You can:");
                    println!("      - cd {}", worktree_path);
                    println!("      - git rebase --abort (to cancel)");
                    println!("      - git rebase --continue (after resolving conflicts)");
                }
            }
        }
    }

    println!();
    println!("🎉 Worktree update process completed!");
    println!();
    println!("📊 Summary:");
    
    // Get main worktree info
    let main_commit_output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .stdout(Stdio::piped())
        .output()?;
    
    let main_commit = String::from_utf8_lossy(&main_commit_output.stdout).trim().to_string();
    println!("   - Main worktree: {}", main_commit);
    
    // Get origin/main info
    let origin_main_output = Command::new("git")
        .args(&["log", "--oneline", "origin/main", "-1"])
        .stdout(Stdio::piped())
        .output()?;
    
    let origin_main = String::from_utf8_lossy(&origin_main_output.stdout).trim().to_string();
    println!("   - Origin/main: {}", origin_main);
    println!();
    println!("💡 Next steps:");
    println!("   - Review any worktrees that had conflicts");
    println!("   - Test your changes in updated worktrees");
    println!("   - Create PRs for worktrees that are ready");

    Ok(())
} 
