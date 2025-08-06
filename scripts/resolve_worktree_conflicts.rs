#!/usr/bin/env rust-script
//! Comprehensive Worktree Conflict Resolution Script
//! This script handles worktree conflicts, rebases, and lifecycle management

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

// ANSI color codes for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m"; // No Color

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

fn is_rebasing(worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    let status = String::from_utf8(status_output.stdout)?;
    
    // Check for conflict markers
    if status.lines().any(|line| line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")) {
        return Ok(true);
    }
    
    // Check for rebase in progress
    let git_status_output = Command::new("git")
        .args(&["status"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    let git_status = String::from_utf8(git_status_output.stdout)?;
    Ok(git_status.contains("rebase in progress"))
}

fn abort_rebase(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Aborting rebase in {}", worktree_path));
    
    if is_rebasing(worktree_path)? {
        let status = Command::new("git")
            .args(&["rebase", "--abort"])
            .current_dir(worktree_path)
            .status()?;
        
        if status.success() {
            log_success("Rebase aborted successfully");
        } else {
            log_error("Failed to abort rebase");
        }
    } else {
        log_info("No rebase in progress");
    }
    
    Ok(())
}

fn stash_changes(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Stashing changes in {}", worktree_path));
    
    // Check if there are changes to stash
    let diff_output = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(worktree_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    
    if !diff_output.success() {
        // There are changes to stash
        let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
        
        let status = Command::new("git")
            .args(&["stash", "push", "-m", &stash_message])
            .current_dir(worktree_path)
            .status()?;
        
        if status.success() {
            log_success("Changes stashed");
        } else {
            log_error("Failed to stash changes");
        }
    } else {
        log_info("No changes to stash");
    }
    
    Ok(())
}

fn get_worktree_status(worktree_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["status", "--short"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    Ok(String::from_utf8(output.stdout)?)
}

fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));
    
    if !Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Ok(false);
    }
    
    // Check current status
    let status = get_worktree_status(worktree_path)?;
    let is_rebase_state = is_rebasing(worktree_path)?;
    
    log_info(&format!("Current status: {}", status.trim()));
    
    if is_rebase_state {
        log_warning("Rebase in progress - aborting to preserve state");
        abort_rebase(worktree_path)?;
    }
    
    // Stash any uncommitted changes
    let diff_output = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(worktree_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    
    if !diff_output.success() {
        stash_changes(worktree_path)?;
    }
    
    // Try to rebase onto main
    log_info("Attempting to rebase onto main...");
    let rebase_status = Command::new("git")
        .args(&["rebase", "main"])
        .current_dir(worktree_path)
        .status()?;
    
    if rebase_status.success() {
        log_success("Successfully rebased onto main");
        Ok(true)
    } else {
        log_warning("Rebase failed - conflicts need manual resolution");
        log_info("Please resolve conflicts manually and then run:");
        log_info("  git add .");
        log_info("  git rebase --continue");
        Ok(false)
    }
}

fn get_worktrees() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;
    
    let worktree_list = String::from_utf8(output.stdout)?;
    let mut worktrees = Vec::new();
    
    for line in worktree_list.lines() {
        if line.starts_with("worktree ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];
                let branch_name = Path::new(worktree_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                // Skip main worktree
                if branch_name != "hooksmith" {
                    worktrees.push((worktree_path.to_string(), branch_name));
                }
            }
        }
    }
    
    Ok(worktrees)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut specific_worktree = None;
    let mut dry_run = false;
    
    for i in 1..args.len() {
        match args[i].as_str() {
            "--worktree" => {
                if i + 1 < args.len() {
                    specific_worktree = Some(args[i + 1].clone());
                }
            }
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                println!("Comprehensive Worktree Conflict Resolution Script");
                println!();
                println!("Usage: resolve_worktree_conflicts [options]");
                println!();
                println!("Options:");
                println!("  --worktree <path>  Resolve conflicts for specific worktree");
                println!("  --dry-run          Show what would be done without making changes");
                println!("  --help             Show this usage information");
                println!();
                println!("Examples:");
                println!("  resolve_worktree_conflicts                    # Resolve all worktrees");
                println!("  resolve_worktree_conflicts --worktree .wt/feature-branch");
                println!("  resolve_worktree_conflicts --dry-run         # Show what would be done");
                return Ok(());
            }
            _ => {
                if !args[i].starts_with("--") {
                    specific_worktree = Some(args[i].clone());
                }
            }
        }
    }
    
    if dry_run {
        log_info("DRY RUN MODE - No changes will be made");
        println!();
    }
    
    if let Some(worktree_path) = specific_worktree {
        // Process specific worktree
        let branch_name = Path::new(&worktree_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        if dry_run {
            log_info(&format!("DRY RUN: Would resolve conflicts for worktree: {} (branch: {})", worktree_path, branch_name));
        } else {
            resolve_worktree_conflicts(&worktree_path, &branch_name)?;
        }
    } else {
        // Process all worktrees
        let worktrees = get_worktrees()?;
        
        if worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }
        
        log_info(&format!("Found {} worktree(s) to process", worktrees.len()));
        println!();
        
        let mut resolved_count = 0;
        let mut failed_count = 0;
        
        for (worktree_path, branch_name) in &worktrees {
            if dry_run {
                log_info(&format!("DRY RUN: Would resolve conflicts for worktree: {} (branch: {})", worktree_path, branch_name));
            } else {
                if resolve_worktree_conflicts(worktree_path, branch_name)? {
                    resolved_count += 1;
                } else {
                    failed_count += 1;
                }
            }
            
            println!("---");
        }
        
        if !dry_run {
            log_info(&format!("Resolved: {}", resolved_count));
            if failed_count > 0 {
                log_error(&format!("Failed: {}", failed_count));
            }
        }
    }
    
    Ok(())
} 
