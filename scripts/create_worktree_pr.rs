#!/usr/bin/env rust-script
//! Create PRs for Ready Worktrees
//! Automatically creates pull requests for worktrees that are ready

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
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

fn log_header(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

/// Check if worktree is ready for PR
fn is_ready_for_pr(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Check if clean
    let status_output = Command::new("git").args(["status", "--porcelain"]).output()?;
    let status = String::from_utf8(status_output.stdout)?;
    if !status.trim().is_empty() {
        std::env::set_current_dir(current_dir)?;
        return Ok(false);
    }

    // Check if up to date with main
    let behind_output = Command::new("git")
        .args(["rev-list", "--count", "HEAD..main"])
        .output();

    let behind_count = if let Ok(output) = behind_output {
        String::from_utf8(output.stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if behind_count > 0 {
        std::env::set_current_dir(current_dir)?;
        return Ok(false);
    }

    // Check if has commits ahead of main
    let ahead_output = Command::new("git")
        .args(["rev-list", "--count", "main..HEAD"])
        .output();

    let ahead_count = if let Ok(output) = ahead_output {
        String::from_utf8(output.stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if ahead_count == 0 {
        std::env::set_current_dir(current_dir)?;
        return Ok(false);
    }

    std::env::set_current_dir(current_dir)?;
    Ok(true)
}

/// Push branch
fn push_branch(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Pushing branch {}", branch_name));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    let status = Command::new("git")
        .args(["push", "origin", branch_name])
        .status()?;

    std::env::set_current_dir(current_dir)?;

    if status.success() {
        log_success("Branch pushed successfully");
        Ok(true)
    } else {
        log_warning("Push failed - branch may already be up to date");
        Ok(false)
    }
}

/// Create PR using GitHub CLI
fn create_pr_with_gh(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Creating PR for branch {} using GitHub CLI", branch_name));

    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;

    // Get commit message for PR title
    let commit_output = Command::new("git").args(["log", "--oneline", "-1"]).output()?;
    let commit_msg = String::from_utf8(commit_output.stdout)?;
    let pr_title = if let Some(space_pos) = commit_msg.find(' ') {
        &commit_msg[space_pos + 1..].trim()
    } else {
        &commit_msg
    };

    // Get PR body from commit messages
    let body_output = Command::new("git")
        .args(["log", "--oneline", "main..HEAD"])
        .output()?;
    let body_lines = String::from_utf8(body_output.stdout)?;
    let pr_body = body_lines
        .lines()
        .take(5)
        .map(|line| format!("- {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let status = Command::new("gh")
        .args([
            "pr", "create",
            "--title", pr_title,
            "--body", &pr_body,
            "--base", "main",
            "--head", branch_name,
        ])
        .status();

    std::env::set_current_dir(current_dir)?;

    match status {
        Ok(exit_status) if exit_status.success() => {
            log_success("PR created successfully");
            Ok(true)
        }
        _ => {
            log_warning("Failed to create PR with GitHub CLI");
            Ok(false)
        }
    }
}

/// Generate PR URL
fn generate_pr_url(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()?;

    let repo_url = String::from_utf8(output.stdout)?.trim().replace(".git", "");

    if repo_url.contains("github.com") {
        Ok(format!("{}/compare/main...{}", repo_url, branch_name))
    } else {
        Ok("Unknown repository URL".to_string())
    }
}

/// Process ready worktree
fn process_ready_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing ready worktree: {} (branch: {})", worktree_path, branch_name));

    // Push branch
    if push_branch(worktree_path, branch_name)? {
        // Try to create PR with GitHub CLI
        let gh_available = Command::new("gh").arg("--version").output().is_ok();

        if gh_available {
            if create_pr_with_gh(worktree_path, branch_name)? {
                log_success(&format!("PR created successfully for {}", branch_name));
                return Ok(true);
            }
        }

        // Fallback: generate PR URL
        let pr_url = generate_pr_url(branch_name)?;
        log_info(&format!("PR URL generated: {}", pr_url));
        log_warning("Please create PR manually using the URL above");
        return Ok(true);
    } else {
        log_error(&format!("Failed to push branch {}", branch_name));
        Ok(false)
    }
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
    log_header("CREATE WORKTREE PRs");
    println!();

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

    let mut ready_worktrees = Vec::new();

    // Find ready worktrees
    for (worktree_path, branch_name) in &worktrees {
        // Skip main worktree
        if branch_name == "refs/heads/main" || worktree_path.ends_with("/hooksmith") {
            continue;
        }

        // Extract branch name from ref
        let branch = if branch_name.starts_with("refs/heads/") {
            &branch_name[11..]
        } else {
            branch_name
        };

        // Check if ready for PR
        if let Ok(ready) = is_ready_for_pr(worktree_path, branch) {
            if ready {
                ready_worktrees.push((worktree_path.clone(), branch.to_string()));
            }
        }
    }

    // Process ready worktrees
    if ready_worktrees.is_empty() {
        log_info("No worktrees ready for PR creation");
        return ExitCode::SUCCESS;
    }

    log_info(&format!("Found {} worktree(s) ready for PR creation", ready_worktrees.len()));
    println!();

    let mut processed_count = 0;

    for (worktree_path, branch_name) in ready_worktrees {
        if process_ready_worktree(&worktree_path, &branch_name).unwrap_or(false) {
            processed_count += 1;
        }

        println!("---");
    }

    log_success(&format!("Processed {} worktree(s)", processed_count));
    ExitCode::SUCCESS
}
