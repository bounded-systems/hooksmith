#!/usr/bin/env rust-script
//! Comprehensive Worktree Conflict Resolution Script
//! This script handles worktree conflicts, rebases, and lifecycle management

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

/// Logging functions
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
    eprintln!("{}[ERROR]{} {}", RED, NC, message);
}

/// Check if we're in a rebase state
fn is_rebasing(worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check for rebase state
    let status_output = Command::new("git").args(["status", "--porcelain"]).output()?;
    let status = String::from_utf8(status_output.stdout)?;

    let has_conflicts = status.lines().any(|line| {
        line.starts_with("UU ") || line.starts_with("AA ") || line.starts_with("DD ")
    });

    if has_conflicts {
        std::env::set_current_dir(current_dir)?;
        return Ok(true);
    }

    // Check for rebase in progress message
    let git_status = Command::new("git").arg("status").output()?;
    let status_text = String::from_utf8(git_status.stdout)?;

    let rebase_in_progress = status_text.contains("rebase in progress");

    std::env::set_current_dir(current_dir)?;
    Ok(rebase_in_progress)
}

/// Safely abort rebase
fn abort_rebase(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Aborting rebase in {}", worktree_path));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    if is_rebasing(worktree_path)? {
        let status = Command::new("git").args(["rebase", "--abort"]).status()?;
        if status.success() {
            log_success("Rebase aborted successfully");
        } else {
            log_warning("Failed to abort rebase");
        }
    } else {
        log_info("No rebase in progress");
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Stash changes
fn stash_changes(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Stashing changes in {}", worktree_path));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check if there are changes to stash
    let diff_output = Command::new("git").args(["diff", "--quiet"]).status();

    if let Ok(exit_status) = diff_output {
        if !exit_status.success() {
            // There are changes to stash
            let stash_message = format!("Auto-stash during conflict resolution {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

            let status = Command::new("git")
                .args(["stash", "push", "-m", &stash_message])
                .status()?;

            if status.success() {
                log_success("Changes stashed");
            } else {
                log_warning("Failed to stash changes");
            }
        } else {
            log_info("No changes to stash");
        }
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Resolve conflicts in a worktree
fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    if !Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Err("Worktree directory not found".into());
    }

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check current status
    let status_output = Command::new("git").args(["status", "--porcelain"]).output()?;
    let status = String::from_utf8(status_output.stdout)?;

    log_info(&format!("Current status: {}", status.trim()));

    // Check if in rebase state
    if is_rebasing(worktree_path)? {
        log_warning("Rebase in progress - aborting to preserve state");
        let status = Command::new("git").args(["rebase", "--abort"]).status()?;
        if status.success() {
            log_success("Rebase aborted");
        }
    }

    // Stash any uncommitted changes
    let diff_status = Command::new("git").args(["diff", "--quiet"]).status();
    if let Ok(exit_status) = diff_status {
        if !exit_status.success() {
            stash_changes(worktree_path)?;
        }
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    let rebase_status = Command::new("git").args(["rebase", "main"]).status();

    match rebase_status {
        Ok(exit_status) if exit_status.success() => {
            log_success("Rebase successful");
        }
        _ => {
            log_warning("Rebase failed - preserving worktree state");
            let _ = Command::new("git").args(["rebase", "--abort"]).status();
        }
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Push worktree branch
fn push_worktree_branch(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Pushing branch {}", branch_name));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    let status = Command::new("git")
        .args(["push", "origin", branch_name])
        .status()?;

    if status.success() {
        log_success("Branch pushed successfully");
    } else {
        log_warning("Push failed - branch may already be up to date");
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Clean up merged worktrees
fn cleanup_merged_worktree(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Checking if worktree {} is merged", branch_name));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check if branch is merged into main
    let merged_output = Command::new("git")
        .args(["branch", "--merged", "main"])
        .output()?;

    let merged_branches = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_branches.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        std::env::set_current_dir(current_dir)?;

        // Remove worktree
        let remove_status = Command::new("git")
            .args(["worktree", "remove", worktree_path, "--force"])
            .status();

        if let Ok(exit_status) = remove_status {
            if exit_status.success() {
                log_success("Worktree removed successfully");
            }
        }

        // Delete branch from origin
        let _ = Command::new("git")
            .args(["push", "origin", "--delete", branch_name])
            .status();

        log_success("Merged worktree cleaned up");
    } else {
        log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Get worktree information
fn get_worktrees() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["worktree", "list", "--porcelain"]).output()?;
    let output_str = String::from_utf8(output.stdout)?;

    let mut worktrees = Vec::new();
    let mut current_worktree = None;
    let mut current_branch = None;

    for line in output_str.lines() {
        if line.starts_with("worktree ") {
            if let Some((worktree, branch)) = current_worktree.take().zip(current_branch.take()) {
                worktrees.push((worktree, branch));
            }
            current_worktree = Some(line[9..].to_string());
        } else if line.starts_with("branch ") {
            current_branch = Some(line[8..].to_string());
        }
    }

    // Add the last worktree
    if let Some((worktree, branch)) = current_worktree.zip(current_branch) {
        worktrees.push((worktree, branch));
    }

    Ok(worktrees)
}

/// Main execution
fn main() -> ExitCode {
    log_info("Starting comprehensive worktree conflict resolution");

    // Get list of worktrees
    let worktrees = match get_worktrees() {
        Ok(wt) => wt,
        Err(e) => {
            log_error(&format!("Failed to get worktrees: {}", e));
            return ExitCode::FAILURE;
        }
    };

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return ExitCode::SUCCESS;
    }

    // Process each worktree
    for (worktree_path, branch_name) in worktrees {
        // Extract branch name from ref
        let branch = if branch_name.starts_with("refs/heads/") {
            &branch_name[11..]
        } else {
            &branch_name
        };

        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        if let Err(e) = resolve_worktree_conflicts(&worktree_path, branch) {
            log_error(&format!("Failed to resolve conflicts: {}", e));
        }

        // Push branch
        if let Err(e) = push_worktree_branch(&worktree_path, branch) {
            log_error(&format!("Failed to push branch: {}", e));
        }

        // Check if merged and cleanup if needed
        if let Err(e) = cleanup_merged_worktree(&worktree_path, branch) {
            log_error(&format!("Failed to cleanup worktree: {}", e));
        }

        println!("---");
    }

    log_success("Worktree conflict resolution completed");
    ExitCode::SUCCESS
}
