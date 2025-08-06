#!/usr/bin/env rust-script
//! Comprehensive Worktree Conflict Resolution Script
//! This script handles worktree conflicts, rebases, and lifecycle management

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

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
    let status_output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    let status = String::from_utf8(status_output.stdout)?;
    let has_conflicts = status.lines().any(|line| {
        line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")
    });

    if has_conflicts {
        std::env::set_current_dir(current_dir)?;
        return Ok(true);
    }

    // Check for rebase in progress message
    let git_status = Command::new("git")
        .arg("status")
        .output()?;

    let git_status_str = String::from_utf8(git_status.stdout)?;
    let rebase_in_progress = git_status_str.contains("rebase in progress");

    std::env::set_current_dir(current_dir)?;
    Ok(rebase_in_progress)
}

/// Safely abort rebase
fn abort_rebase(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Aborting rebase in {}", worktree_path));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    if is_rebasing(worktree_path)? {
        let status = Command::new("git")
            .arg("rebase")
            .arg("--abort")
            .status()?;

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
    let diff_output = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .status();

    let has_changes = diff_output.is_err() || !diff_output.unwrap().success();

    if has_changes {
        let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now());
        let status = Command::new("git")
            .arg("stash")
            .arg("push")
            .arg("-m")
            .arg(&stash_message)
            .status()?;

        if status.success() {
            log_success("Changes stashed");
        } else {
            log_warning("Failed to stash changes");
        }
    } else {
        log_info("No changes to stash");
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Resolve conflicts in a worktree
fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    if !std::path::Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check current status
    let status_output = Command::new("git")
        .arg("status")
        .arg("--short")
        .output()?;

    let status = String::from_utf8(status_output.stdout)?;
    log_info(&format!("Current status: {}", status.trim()));

    // Check if in rebase state
    if is_rebasing(worktree_path)? {
        log_warning("Rebase in progress - aborting to preserve state");
        let abort_status = Command::new("git")
            .arg("rebase")
            .arg("--abort")
            .status()?;

        if abort_status.success() {
            log_success("Rebase aborted");
        } else {
            log_warning("Failed to abort rebase");
        }
    }

    // Stash any uncommitted changes
    let diff_quiet = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .status();

    if diff_quiet.is_err() || !diff_quiet.unwrap().success() {
        stash_changes(worktree_path)?;
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    let rebase_status = Command::new("git")
        .arg("rebase")
        .arg("main")
        .status();

    match rebase_status {
        Ok(status) if status.success() => {
            log_success("Rebase successful");
        }
        _ => {
            log_warning("Rebase failed - preserving worktree state");
            let _ = Command::new("git")
                .arg("rebase")
                .arg("--abort")
                .status();
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
        .arg("push")
        .arg("origin")
        .arg(branch_name)
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
        .arg("branch")
        .arg("--merged")
        .arg("main")
        .output()?;

    let merged_str = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_str.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Go back to main directory for worktree operations
        std::env::set_current_dir(&current_dir)?;

        // Remove worktree
        let remove_status = Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg(worktree_path)
            .arg("--force")
            .status();

        if remove_status.is_ok() && remove_status.unwrap().success() {
            log_success("Worktree removed successfully");
        } else {
            log_warning("Failed to remove worktree");
        }

        // Delete branch from origin
        let _ = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("--delete")
            .arg(branch_name)
            .status();

        log_success("Merged worktree cleaned up");
    } else {
        log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
    }

    std::env::set_current_dir(current_dir)?;
    Ok(())
}

/// Get worktrees
fn get_worktrees() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut worktrees = Vec::new();

    for line in output_str.lines() {
        if line.starts_with("worktree ") {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let worktree_path = parts[1].to_string();
                let branch_name = std::path::Path::new(&worktree_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                worktrees.push((worktree_path, branch_name));
            }
        }
    }

    Ok(worktrees)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Starting comprehensive worktree conflict resolution");

    // Get list of worktrees
    let worktrees = get_worktrees()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    // Process each worktree
    for (worktree_path, branch_name) in &worktrees {
        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        resolve_worktree_conflicts(worktree_path, branch_name)?;

        // Push branch
        push_worktree_branch(worktree_path, branch_name)?;

        // Check if merged and cleanup if needed
        cleanup_merged_worktree(worktree_path, branch_name)?;

        println!("---");
    }

    log_success("Worktree conflict resolution completed");

    Ok(())
}
