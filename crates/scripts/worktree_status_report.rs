#!/usr/bin/env rust-script
//! Comprehensive Worktree Status Report
//! Shows detailed status of all worktrees and their branches

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

/// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const CYAN: &str = "\x1b[0;36m";
const NC: &str = "\x1b[0m";

#[derive(Debug)]
struct WorktreeStatus {
    current_branch: String,
    is_clean: bool,
    is_rebasing: bool,
    remote_exists: bool,
    is_merged: bool,
    ahead_behind: i32,
    behind_ahead: i32,
}

#[derive(Debug)]
enum WorktreeState {
    Ready,
    Conflicted,
    Merged,
    Developing,
    Outdated,
    Unknown,
}

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

fn run_command(cmd: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

fn get_worktree_status(worktree_path: &str) -> Result<WorktreeStatus, Box<dyn std::error::Error>> {
    let worktree_path = Path::new(worktree_path);
    
    // Get current branch
    let current_branch = run_command(&mut Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_path))?
        .trim()
        .to_string();
    
    // Get status
    let status = run_command(&mut Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path))?;
    let is_clean = status.trim().is_empty();
    
    // Check if rebasing
    let git_status = run_command(&mut Command::new("git")
        .args(&["status"])
        .current_dir(worktree_path))?;
    let is_rebasing = git_status.contains("rebase");
    
    // Check if branch exists on origin
    let remote_check = run_command(&mut Command::new("git")
        .args(&["ls-remote", "--heads", "origin", &current_branch])
        .current_dir(worktree_path))
        .unwrap_or_default();
    let remote_exists = remote_check.contains(&current_branch);
    
    // Check if merged into main
    let merged_branches = run_command(&mut Command::new("git")
        .args(&["branch", "--merged", "main"])
        .current_dir(worktree_path))
        .unwrap_or_default();
    let is_merged = merged_branches.contains(&current_branch);
    
    // Get commit count ahead/behind main
    let ahead_behind = run_command(&mut Command::new("git")
        .args(&["rev-list", "--count", "main..HEAD"])
        .current_dir(worktree_path))
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);
    
    let behind_ahead = run_command(&mut Command::new("git")
        .args(&["rev-list", "--count", "HEAD..main"])
        .current_dir(worktree_path))
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);
    
    Ok(WorktreeStatus {
        current_branch,
        is_clean,
        is_rebasing,
        remote_exists,
        is_merged,
        ahead_behind,
        behind_ahead,
    })
}

fn determine_state(status: &WorktreeStatus) -> WorktreeState {
    if status.is_merged {
        WorktreeState::Merged
    } else if status.is_rebasing {
        WorktreeState::Conflicted
    } else if !status.is_clean {
        WorktreeState::Developing
    } else if status.ahead_behind > 0 && status.behind_ahead == 0 {
        WorktreeState::Ready
    } else if status.behind_ahead > 0 {
        WorktreeState::Outdated
    } else {
        WorktreeState::Unknown
    }
}

fn print_worktree_status(worktree_path: &str, status: &WorktreeStatus) {
    let state = determine_state(status);
    let state_str = match state {
        WorktreeState::Ready => "READY",
        WorktreeState::Conflicted => "CONFLICTED",
        WorktreeState::Merged => "MERGED",
        WorktreeState::Developing => "DEVELOPING",
        WorktreeState::Outdated => "OUTDATED",
        WorktreeState::Unknown => "UNKNOWN",
    };
    
    println!("{}📁 Worktree:{} {}", CYAN, NC, worktree_path);
    println!("   {}Branch:{} {}", BLUE, NC, status.current_branch);
    println!("   {}State:{} {}", BLUE, NC, state_str);
    println!("   {}Status:{} {}", BLUE, NC, if status.is_clean { "clean" } else { "dirty" });
    println!("   {}Rebasing:{} {}", BLUE, NC, status.is_rebasing);
    println!("   {}Remote:{} {}", BLUE, NC, status.remote_exists);
    println!("   {}Merged:{} {}", BLUE, NC, status.is_merged);
    println!("   {}Commits:{} +{} -{}", BLUE, NC, status.ahead_behind, status.behind_ahead);
    println!();
}

fn generate_pr_url(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let repo_url = run_command(&mut Command::new("git")
        .args(&["config", "--get", "remote.origin.url"]))?
        .trim()
        .replace(".git", "");
    
    if repo_url.contains("github.com") {
        Ok(format!("{}/compare/main...{}", repo_url, branch_name))
    } else {
        Ok("Unknown repository URL".to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("WORKTREE STATUS REPORT");
    println!();
    
    // Get list of worktrees
    let worktrees_output = run_command(&mut Command::new("git").args(&["worktree", "list", "--porcelain"]))?;
    let worktrees: Vec<String> = worktrees_output
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
        .filter(|path| !path.is_empty())
        .collect();
    
    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }
    
    let mut ready_worktrees = Vec::new();
    let mut conflicted_worktrees = Vec::new();
    let mut merged_worktrees = Vec::new();
    let mut developing_worktrees = Vec::new();
    
    // Process each worktree
    for worktree_path in &worktrees {
        let status = match get_worktree_status(worktree_path) {
            Ok(status) => status,
            Err(e) => {
                log_error(&format!("Failed to get status for {}: {}", worktree_path, e));
                continue;
            }
        };
        
        // Print status
        print_worktree_status(worktree_path, &status);
        
        // Categorize worktree
        match determine_state(&status) {
            WorktreeState::Ready => ready_worktrees.push(status.current_branch.clone()),
            WorktreeState::Conflicted => conflicted_worktrees.push(status.current_branch.clone()),
            WorktreeState::Merged => merged_worktrees.push(status.current_branch.clone()),
            WorktreeState::Developing => developing_worktrees.push(status.current_branch.clone()),
            _ => {}
        }
    }
    
    // Summary
    log_header("SUMMARY");
    println!();
    
    if !ready_worktrees.is_empty() {
        log_success(&format!("Ready for PR: {}", ready_worktrees.join(", ")));
        for branch in &ready_worktrees {
            match generate_pr_url(branch) {
                Ok(pr_url) => println!("   PR URL: {}", pr_url),
                Err(_) => println!("   PR URL: Could not generate"),
            }
        }
        println!();
    }
    
    if !conflicted_worktrees.is_empty() {
        log_warning(&format!("Conflicted: {}", conflicted_worktrees.join(", ")));
        println!();
    }
    
    if !merged_worktrees.is_empty() {
        log_info(&format!("Merged (ready for cleanup): {}", merged_worktrees.join(", ")));
        println!();
    }
    
    if !developing_worktrees.is_empty() {
        log_info(&format!("Developing: {}", developing_worktrees.join(", ")));
        println!();
    }
    
    if ready_worktrees.is_empty() && conflicted_worktrees.is_empty() && 
       merged_worktrees.is_empty() && developing_worktrees.is_empty() {
        log_info("No worktrees to process");
    }
    
    Ok(())
} 
