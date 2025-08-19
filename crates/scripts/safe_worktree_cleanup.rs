#!/usr/bin/env rust-script
//! Safe worktree cleanup script
//! Checks for uncommitted changes before removing worktrees

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking worktrees for uncommitted changes...");

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
                println!("⏭️  Skipping main worktree: {}", worktree_path);
                continue;
            }
            
            println!("📁 Checking worktree: {}", worktree_path);
            
            // Check if worktree directory exists
            if !Path::new(worktree_path).exists() {
                println!("🗑️  Removing non-existent worktree: {}", worktree_path);
                let remove_status = Command::new("git")
                    .args(&["worktree", "remove", worktree_path, "--force"])
                    .status()?;
                
                if !remove_status.success() {
                    eprintln!("❌ Failed to remove worktree: {}", worktree_path);
                }
                continue;
            }
            
            // Check for uncommitted changes
            let status_output = Command::new("git")
                .args(&["status", "--porcelain"])
                .current_dir(worktree_path)
                .stdout(Stdio::piped())
                .output()?;
            
            let status = String::from_utf8(status_output.stdout)?;
            
            if !status.trim().is_empty() {
                println!("⚠️  WARNING: Uncommitted changes found in {}", worktree_path);
                println!("   Changes:");
                for line in status.lines() {
                    if !line.trim().is_empty() {
                        println!("   {}", line);
                    }
                }
                println!("   Please commit or stash changes before removing this worktree");
                println!("");
            } else {
                println!("✅ No uncommitted changes found in {}", worktree_path);
                println!("🗑️  Removing worktree: {}", worktree_path);
                
                let remove_status = Command::new("git")
                    .args(&["worktree", "remove", worktree_path, "--force"])
                    .status()?;
                
                if !remove_status.success() {
                    eprintln!("❌ Failed to remove worktree: {}", worktree_path);
                }
            }
        }
    }

    println!("🎉 Worktree cleanup completed!");
    Ok(())
} 
