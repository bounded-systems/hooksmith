#!/usr/bin/env cargo
[package]
name = "ensure-clean-main"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Checking if main is ahead of origin/main...");
    
    // Check if we're on main branch
    let current_branch = get_current_branch()?;
    if current_branch != "main" {
        println!("⚠️  Not on main branch (currently on: {}). Skipping cleanup.", current_branch);
        return Ok(());
    }
    
    // Check if main is ahead of origin/main
    let ahead_commits = get_ahead_commits()?;
    if ahead_commits.is_empty() {
        println!("✅ Main is clean and up to date with origin/main");
        return Ok(());
    }
    
    println!("⚠️  Main is ahead of origin/main by {} commit(s):", ahead_commits.len());
    for commit in &ahead_commits {
        println!("   - {}", commit);
    }
    
    // Check for uncommitted changes
    let uncommitted = get_uncommitted_changes()?;
    if !uncommitted.is_empty() {
        println!("⚠️  Found uncommitted changes:");
        for change in &uncommitted {
            println!("   - {}", change);
        }
    }
    
    // Create timestamp for worktree name
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let worktree_name = format!("fix/main-cleanup-{}", timestamp);
    let branch_name = format!("fix/main-cleanup-{}", timestamp);
    
    println!("🔄 Creating worktree: {} -> {}", branch_name, worktree_name);
    
    // Create worktree
    create_worktree(&branch_name, &worktree_name)?;
    
    // Switch to worktree
    println!("🔄 Switching to worktree...");
    switch_to_worktree(&worktree_name)?;
    
    // Commit all changes in worktree
    if !uncommitted.is_empty() {
        println!("🔄 Committing changes in worktree...");
        commit_changes(&branch_name)?;
    }
    
    // Switch back to main
    println!("🔄 Switching back to main...");
    switch_to_main()?;
    
    // Reset main to match origin/main
    println!("🔄 Resetting main to match origin/main...");
    reset_main()?;
    
    println!("✅ Main cleanup completed!");
    println!("📁 Changes moved to worktree: {}", worktree_name);
    println!("🌿 Branch: {}", branch_name);
    
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;
    
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn get_ahead_commits() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["log", "--oneline", "origin/main..HEAD"])
        .output()
        .context("Failed to get ahead commits")?;
    
    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    Ok(commits)
}

fn get_uncommitted_changes() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to get uncommitted changes")?;
    
    let changes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    Ok(changes)
}

fn create_worktree(branch_name: &str, worktree_name: &str) -> Result<()> {
    // First create the branch
    Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .context("Failed to create branch")?;
    
    // Create worktree
    let output = Command::new("cargo")
        .args(["xtask", "worktree", "create", "--branch", branch_name, "--switch"])
        .output()
        .context("Failed to create worktree")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree: {}", stderr);
    }
    
    Ok(())
}

fn switch_to_worktree(worktree_name: &str) -> Result<()> {
    let worktree_path = format!("worktree-{}", worktree_name);
    if !Path::new(&worktree_path).exists() {
        anyhow::bail!("Worktree directory not found: {}", worktree_path);
    }
    
    // Change to worktree directory
    std::env::set_current_dir(&worktree_path)
        .context(format!("Failed to change to worktree directory: {}", worktree_path))?;
    
    Ok(())
}

fn commit_changes(branch_name: &str) -> Result<()> {
    // Add all changes
    Command::new("git")
        .args(["add", "."])
        .output()
        .context("Failed to add changes")?;
    
    // Commit changes
    let commit_message = format!("feat: Move main changes to worktree {}", branch_name);
    Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
        .context("Failed to commit changes")?;
    
    Ok(())
}

fn switch_to_main() -> Result<()> {
    // Change back to main directory
    std::env::set_current_dir("/Users/bobby/dev/repos/hooksmith")
        .context("Failed to change back to main directory")?;
    
    // Switch to main branch
    Command::new("git")
        .args(["checkout", "main"])
        .output()
        .context("Failed to switch to main")?;
    
    Ok(())
}

fn reset_main() -> Result<()> {
    Command::new("git")
        .args(["reset", "--hard", "origin/main"])
        .output()
        .context("Failed to reset main")?;
    
    Ok(())
} 
