#!/usr/bin/env rust-script
//! Create PRs for Ready Worktrees
//! Automatically creates pull requests for worktrees that are ready

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;
use std::collections::HashMap;

// ANSI color codes for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
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

fn log_header(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

struct WorktreeInfo {
    path: String,
    branch: String,
}

fn is_ready_for_pr(worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Check if clean
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    let status = String::from_utf8(status_output.stdout)?;
    if !status.trim().is_empty() {
        return Ok(false);
    }
    
    // Check if up to date with main
    let behind_output = Command::new("git")
        .args(&["rev-list", "--count", "HEAD..main"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;
    
    let behind_count = String::from_utf8_lossy(&behind_output.stdout).trim();
    if behind_count != "0" {
        return Ok(false);
    }
    
    // Check if has commits ahead of main
    let ahead_output = Command::new("git")
        .args(&["rev-list", "--count", "main..HEAD"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;
    
    let ahead_count = String::from_utf8_lossy(&ahead_output.stdout).trim();
    if ahead_count == "0" {
        return Ok(false);
    }
    
    Ok(true)
}

fn push_branch(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Pushing branch {}", branch_name));
    
    let status = Command::new("git")
        .args(&["push", "origin", branch_name])
        .current_dir(worktree_path)
        .status()?;
    
    if status.success() {
        log_success("Branch pushed successfully");
        Ok(true)
    } else {
        log_warning("Push failed - branch may already be up to date");
        Ok(false)
    }
}

fn create_pr_with_gh(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Creating PR for branch {} using GitHub CLI", branch_name));
    
    // Get commit message for PR title
    let commit_output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    let commit_msg = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();
    let pr_title = if let Some(rest) = commit_msg.splitn(2, ' ').nth(1) {
        rest.to_string()
    } else {
        format!("Update from {}", branch_name)
    };
    
    // Get PR body from commit messages
    let body_output = Command::new("git")
        .args(&["log", "--oneline", "main..HEAD"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;
    
    let body_lines: Vec<String> = String::from_utf8(body_output.stdout)?
        .lines()
        .take(5)
        .map(|line| format!("- {}", line))
        .collect();
    
    let pr_body = body_lines.join("\n");
    
    // Check if gh CLI is available
    let gh_check = Command::new("gh")
        .args(&["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if gh_check.is_err() {
        log_warning("GitHub CLI not available");
        return Ok(false);
    }
    
    let status = Command::new("gh")
        .args(&["pr", "create", "--title", &pr_title, "--body", &pr_body, "--base", "main", "--head", branch_name])
        .current_dir(worktree_path)
        .status()?;
    
    if status.success() {
        log_success("PR created successfully");
        Ok(true)
    } else {
        log_warning("Failed to create PR with GitHub CLI");
        Ok(false)
    }
}

fn generate_pr_url(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let repo_output = Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .stdout(Stdio::piped())
        .output()?;
    
    let repo_url = String::from_utf8(repo_output.stdout)?
        .trim()
        .replace(".git", "");
    
    if repo_url.contains("github.com") {
        Ok(format!("{}/compare/main...{}", repo_url, branch_name))
    } else {
        Ok("Unknown repository URL".to_string())
    }
}

fn process_ready_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing ready worktree: {} (branch: {})", worktree_path, branch_name));
    
    // Push branch
    if push_branch(worktree_path, branch_name)? {
        // Try to create PR with GitHub CLI
        if create_pr_with_gh(worktree_path, branch_name)? {
            log_success(&format!("PR created successfully for {}", branch_name));
            return Ok(true);
        }
        
        // Fallback: generate PR URL
        let pr_url = generate_pr_url(branch_name)?;
        log_info(&format!("PR URL generated: {}", pr_url));
        log_warning("Please create PR manually using the URL above");
        return Ok(true);
    } else {
        log_error(&format!("Failed to push branch {}", branch_name));
        return Ok(false);
    }
}

fn get_worktrees() -> Result<Vec<WorktreeInfo>, Box<dyn std::error::Error>> {
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
                    worktrees.push(WorktreeInfo {
                        path: worktree_path.to_string(),
                        branch: branch_name,
                    });
                }
            }
        }
    }
    
    Ok(worktrees)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CREATE WORKTREE PRs");
    println!();
    
    // Get list of worktrees
    let worktrees = get_worktrees()?;
    
    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }
    
    let mut ready_worktrees = Vec::new();
    
    // Find ready worktrees
    for worktree in &worktrees {
        if is_ready_for_pr(&worktree.path)? {
            ready_worktrees.push(worktree.clone());
        }
    }
    
    // Process ready worktrees
    if ready_worktrees.is_empty() {
        log_info("No worktrees ready for PR creation");
        return Ok(());
    }
    
    log_info(&format!("Found {} worktree(s) ready for PR creation", ready_worktrees.len()));
    println!();
    
    let mut processed_count = 0;
    
    for worktree in &ready_worktrees {
        if process_ready_worktree(&worktree.path, &worktree.branch)? {
            processed_count += 1;
        }
        
        println!("---");
    }
    
    log_success(&format!("Processed {} worktree(s)", processed_count));
    
    Ok(())
} 
