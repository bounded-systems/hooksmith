#!/usr/bin/env rust-script
//! Verify Worktree 1:1 Sync with Remote Branches
//! Checks if worktrees match remote branches exactly

use std::process::{Command, Stdio};
use std::collections::HashSet;
use std::env;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

// Logging functions
fn log_info(message: &str) {
    println!("{}[INFO]{} {}", BLUE, NC, message);
}

fn log_success(message: &str) {
    println!("{}[SUCCESS]{} {}", GREEN, NC, message);
}

fn log_warning(message: &str) {
    println!("{}[WARNING]{} {}", YELLOW, NC, message);
}

fn log_error(message: &str) {
    println!("{}[ERROR]{} {}", RED, NC, message);
}

fn log_header(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

// Function to get remote branches (excluding main)
fn get_remote_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "-r"])
        .stdout(Stdio::piped())
        .output()?;

    let branches = String::from_utf8(output.stdout)?;
    let remote_branches: Vec<String> = branches
        .lines()
        .filter(|line| line.contains("origin/") && !line.contains("origin/main") && !line.contains("origin/HEAD"))
        .map(|line| line.replace("origin/", "").trim().to_string())
        .collect();

    Ok(remote_branches)
}

// Function to get worktree branches
fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    let worktree_branches: Vec<String> = worktree_list
        .lines()
        .filter(|line| !line.contains("main"))
        .filter_map(|line| {
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    if end > start {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
            None
        })
        .collect();

    Ok(worktree_branches)
}

// Function to check if worktree exists for a branch
fn worktree_exists(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    Ok(worktree_list.contains(&format!("[{}]", branch_name)))
}

// Function to check if remote branch exists
fn remote_branch_exists(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-remote", "--heads", "origin", branch_name])
        .stdout(Stdio::piped())
        .output()?;

    let remote_list = String::from_utf8(output.stdout)?;
    Ok(remote_list.contains(branch_name))
}

// Function to verify 1:1 mapping
fn verify_worktree_sync() -> Result<bool, Box<dyn std::error::Error>> {
    log_header("VERIFYING WORKTREE 1:1 SYNC");
    
    // Fetch latest remote branches
    log_info("Fetching remote branches...");
    Command::new("git")
        .args(&["fetch", "--all", "--prune"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    
    // Get current state
    let remote_branches = get_remote_branches()?;
    let worktree_branches = get_worktree_branches()?;
    
    log_info(&format!("Remote branches ({}): {}", remote_branches.len(), remote_branches.join(" ")));
    log_info(&format!("Worktree branches ({}): {}", worktree_branches.len(), worktree_branches.join(" ")));
    
    // Find missing worktrees (remote branches without worktrees)
    let mut missing_worktrees = Vec::new();
    for branch in &remote_branches {
        if !worktree_exists(branch)? {
            missing_worktrees.push(branch.clone());
        }
    }
    
    // Find orphaned worktrees (worktrees without remote branches)
    let mut orphaned_worktrees = Vec::new();
    for branch in &worktree_branches {
        if !remote_branch_exists(branch)? {
            orphaned_worktrees.push(branch.clone());
        }
    }
    
    // Report results
    println!();
    log_header("VERIFICATION RESULTS");
    
    if missing_worktrees.is_empty() && orphaned_worktrees.is_empty() {
        log_success("✅ PERFECT SYNC: All worktrees have corresponding remote branches");
        log_success("✅ All remote branches have corresponding worktrees");
        Ok(true)
    } else {
        if !missing_worktrees.is_empty() {
            log_warning(&format!("⚠️  MISSING WORKTREES ({}): {}", missing_worktrees.len(), missing_worktrees.join(" ")));
        }
        
        if !orphaned_worktrees.is_empty() {
            log_warning(&format!("⚠️  ORPHANED WORKTREES ({}): {}", orphaned_worktrees.len(), orphaned_worktrees.join(" ")));
        }
        
        log_info("💡 Run './scripts/simple-worktree-sync.sh' to fix the sync");
        Ok(false)
    }
}

// Function to show current status
fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CURRENT STATUS");
    
    println!("Worktrees:");
    let worktree_output = Command::new("git")
        .args(&["worktree", "list"])
        .stdout(Stdio::piped())
        .output()?;
    
    println!("{}", String::from_utf8(worktree_output.stdout)?);
    
    println!("Remote branches:");
    let remote_output = Command::new("git")
        .args(&["branch", "-r"])
        .stdout(Stdio::piped())
        .output()?;
    
    let remote_branches = String::from_utf8(remote_output.stdout)?;
    for line in remote_branches.lines() {
        if line.contains("origin/") && !line.contains("origin/main") && !line.contains("origin/HEAD") {
            println!("{}", line);
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("verify");
    
    match command {
        "verify" => {
            let success = verify_worktree_sync()?;
            if !success {
                std::process::exit(1);
            }
        }
        "status" => {
            show_status()?;
        }
        "help" | "--help" | "-h" => {
            println!("Usage: {} [verify|status|help]", args[0]);
            println!();
            println!("Commands:");
            println!("  verify  - Check if worktrees are in 1:1 sync with remote branches (default)");
            println!("  status  - Show current worktree and remote branch status");
            println!("  help    - Show this help message");
        }
        _ => {
            log_error(&format!("Unknown command: {}", command));
            println!("Use '{} help' for usage information", args[0]);
            std::process::exit(1);
        }
    }
    
    Ok(())
} 
