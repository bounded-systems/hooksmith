#!/usr/bin/env rust-script
//! Comprehensive Worktree Conflict Resolution Script
//! This script handles worktree conflicts, rebases, and lifecycle management

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{self, Write};

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
    eprintln!("{}[ERROR]{} {}", RED, NC, message);
}

// Function to check if we're in a rebase state
fn is_rebasing(worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status_output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(worktree_path)
        .output()?;

    let status_str = String::from_utf8(status_output.stdout)?;

    // Check for conflict markers
    if status_str.lines().any(|line| line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")) {
        return Ok(true);
    }

    // Check for rebase in progress
    let status_full_output = Command::new("git")
        .arg("status")
        .current_dir(worktree_path)
        .output()?;

    let status_full_str = String::from_utf8(status_full_output.stdout)?;
    Ok(status_full_str.contains("rebase in progress"))
}

// Function to safely abort rebase
fn abort_rebase(worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Aborting rebase in {}", worktree_path));

    if is_rebasing(worktree_path)? {
        let status = Command::new("git")
            .args(["rebase", "--abort"])
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
        .arg("diff")
        .arg("--quiet")
        .current_dir(worktree_path)
        .output()?;

    if !diff_output.status.success() {
        // There are changes to stash
        let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now());
        let status = Command::new("git")
            .args(["stash", "push", "-m", &stash_message])
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
        return Err("Worktree directory does not exist".into());
    }

    // Check current status
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    let status_str = String::from_utf8(status_output.stdout)?;
    let is_rebase_state = is_rebasing(worktree_path)?;

    log_info(&format!("Current status: {}", status_str.trim()));

    if is_rebase_state {
        log_warning("Rebase in progress - aborting to preserve state");
        abort_rebase(worktree_path)?;
    }

    // Stash any uncommitted changes
    let diff_output = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .current_dir(worktree_path)
        .output()?;

    if !diff_output.status.success() {
        stash_changes(worktree_path)?;
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    let rebase_status = Command::new("git")
        .args(["rebase", "main"])
        .current_dir(worktree_path)
        .status()?;

    if rebase_status.success() {
        log_success("Rebase successful");
    } else {
        log_warning("Rebase failed - preserving worktree state");
        Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(worktree_path)
            .status()?;
    }

    Ok(())
}

// Function to push worktree branch
fn push_worktree_branch(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info(&format!("Pushing branch {}", branch_name));

    let status = Command::new("git")
        .args(["push", "origin", branch_name])
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
        .args(["branch", "--merged", "main"])
        .current_dir(worktree_path)
        .output()?;

    let merged_str = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_str.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Remove worktree
        let remove_status = Command::new("git")
            .args(["worktree", "remove", worktree_path, "--force"])
            .status()?;

        if remove_status.success() {
            // Delete branch from origin
            Command::new("git")
                .args(["push", "origin", "--delete", branch_name])
                .output()
                .ok(); // Ignore errors for remote deletion

            log_success("Merged worktree cleaned up");
        } else {
            log_warning("Failed to remove worktree");
        }
    } else {
        log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
    }

    Ok(())
}

// Function to get worktree list
fn get_worktree_list() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
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
                    .unwrap_or_default()
                    .to_string_lossy()
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
    let worktrees = get_worktree_list()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    // Process each worktree
    for (worktree_path, branch_name) in &worktrees {
        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        if let Err(e) = resolve_worktree_conflicts(worktree_path, branch_name) {
            log_error(&format!("Failed to resolve conflicts: {}", e));
            continue;
        }

        // Push branch
        if let Err(e) = push_worktree_branch(worktree_path, branch_name) {
            log_error(&format!("Failed to push branch: {}", e));
            continue;
        }

        // Check if merged and cleanup if needed
        if let Err(e) = cleanup_merged_worktree(worktree_path, branch_name) {
            log_error(&format!("Failed to cleanup worktree: {}", e));
            continue;
        }

        println!("---");
    }

    log_success("Worktree conflict resolution completed");

    Ok(())
}
