#!/usr/bin/env rust-script

use std::process::{Command, Stdio};
use std::env;
use std::path::Path;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
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

// Function to check if we're in a rebase state
fn is_rebasing(worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
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
        .output()?;

    let git_status = String::from_utf8(git_status_output.stdout)?;
    Ok(git_status.contains("rebase in progress"))
}

// Function to safely abort rebase
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
            log_warning("Failed to abort rebase");
        }
    } else {
        log_info("No rebase in progress");
    }

    Ok(())
}

// Function to stash changes
fn stash_changes(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Stashing changes in {}", worktree_path));

    // Check if there are changes to stash
    let diff_output = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(worktree_path)
        .output();

    if diff_output.is_err() || !diff_output.unwrap().status.success() {
        // There are changes to stash
        let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let status = Command::new("git")
            .args(&["stash", "push", "-m", &stash_message])
            .current_dir(worktree_path)
            .status()?;

        if status.success() {
            log_success("Changes stashed");
        } else {
            log_warning("Failed to stash changes");
        }
    } else {
        log_info("No changes to stash");
    }

    Ok(())
}

// Function to resolve conflicts in a worktree
fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    if !Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Ok(());
    }

    // Check current status
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    let status = String::from_utf8(status_output.stdout)?;
    let is_rebase_state = is_rebasing(worktree_path)?;

    let short_status_output = Command::new("git")
        .args(&["status", "--short"])
        .current_dir(worktree_path)
        .output()?;

    let short_status = String::from_utf8(short_status_output.stdout)?;
    log_info(&format!("Current status: {}", short_status.trim()));

    if is_rebase_state {
        log_warning("Rebase in progress - aborting to preserve state");
        let _ = Command::new("git")
            .args(&["rebase", "--abort"])
            .current_dir(worktree_path)
            .status();
        log_success("Rebase aborted");
    }

    // Stash any uncommitted changes
    let diff_quiet_output = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(worktree_path)
        .output();

    if diff_quiet_output.is_err() || !diff_quiet_output.unwrap().status.success() {
        stash_changes(worktree_path)?;
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    let rebase_status = Command::new("git")
        .args(&["rebase", "main"])
        .current_dir(worktree_path)
        .status();

    match rebase_status {
        Ok(status) if status.success() => {
            log_success("Rebase successful");
        }
        _ => {
            log_warning("Rebase failed - preserving worktree state");
            let _ = Command::new("git")
                .args(&["rebase", "--abort"])
                .current_dir(worktree_path)
                .status();
        }
    }

    Ok(())
}

// Function to push worktree branch
fn push_worktree_branch(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Pushing branch {}", branch_name));

    let status = Command::new("git")
        .args(&["push", "origin", branch_name])
        .current_dir(worktree_path)
        .status()?;

    if status.success() {
        log_success("Branch pushed successfully");
    } else {
        log_warning("Push failed - branch may already be up to date");
    }

    Ok(())
}

// Function to clean up merged worktrees
fn cleanup_merged_worktree(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Checking if worktree {} is merged", branch_name));

    // Check if branch is merged into main
    let merged_output = Command::new("git")
        .args(&["branch", "--merged", "main"])
        .current_dir(worktree_path)
        .output()?;

    let merged_branches = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_branches.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Remove worktree
        let worktree_remove_status = Command::new("git")
            .args(&["worktree", "remove", worktree_path, "--force"])
            .status();

        if worktree_remove_status.is_ok() && worktree_remove_status.unwrap().success() {
            // Delete branch from origin
            let _ = Command::new("git")
                .args(&["push", "origin", "--delete", branch_name])
                .output();

            log_success("Merged worktree cleaned up");
        } else {
            log_warning("Failed to remove worktree");
        }
    } else {
        log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
    }

    Ok(())
}

// Main execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Starting comprehensive worktree conflict resolution");

    // Get list of worktrees
    let worktrees_output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    let worktrees_str = String::from_utf8(worktrees_output.stdout)?;

    if worktrees_str.trim().is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    // Process each worktree
    for line in worktrees_str.lines() {
        if line.starts_with("worktree") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];

                // Get branch name from worktree path
                let branch_name = Path::new(worktree_path).file_name().unwrap().to_str().unwrap();

                log_info(&format!("Processing worktree: {}", worktree_path));

                // Resolve conflicts
                resolve_worktree_conflicts(worktree_path, branch_name)?;

                // Push branch
                push_worktree_branch(worktree_path, branch_name)?;

                // Check if merged and cleanup if needed
                cleanup_merged_worktree(worktree_path, branch_name)?;

                println!("---");
            }
        }
    }

    log_success("Worktree conflict resolution completed");

    Ok(())
}
