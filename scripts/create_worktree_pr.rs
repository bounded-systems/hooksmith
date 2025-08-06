#!/usr/bin/env rust-script

use std::process::{Command, Stdio};
use std::env;
use std::path::Path;

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

// Function to check if worktree is ready for PR
fn is_ready_for_pr(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Check if clean
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    let status = String::from_utf8(status_output.stdout)?;
    if !status.trim().is_empty() {
        return Ok(false);
    }

    // Check if up to date with main
    let behind_output = Command::new("git")
        .args(&["rev-list", "--count", "HEAD..main"])
        .current_dir(worktree_path)
        .output();

    let behind_count = if behind_output.is_ok() {
        String::from_utf8(behind_output.unwrap().stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if behind_count > 0 {
        return Ok(false);
    }

    // Check if has commits ahead of main
    let ahead_output = Command::new("git")
        .args(&["rev-list", "--count", "main..HEAD"])
        .current_dir(worktree_path)
        .output();

    let ahead_count = if ahead_output.is_ok() {
        String::from_utf8(ahead_output.unwrap().stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if ahead_count == 0 {
        return Ok(false);
    }

    Ok(true)
}

// Function to push branch
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

// Function to create PR using GitHub CLI
fn create_pr_with_gh(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Creating PR for branch {} using GitHub CLI", branch_name));

    // Get commit message for PR title
    let commit_output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(worktree_path)
        .output()?;

    let commit_msg = String::from_utf8(commit_output.stdout)?;
    let parts: Vec<&str> = commit_msg.trim().splitn(2, ' ').collect();
    let pr_title = if parts.len() > 1 { parts[1] } else { "Update" };

    // Get PR body from commit messages
    let body_output = Command::new("git")
        .args(&["log", "--oneline", "main..HEAD"])
        .current_dir(worktree_path)
        .output()?;

    let body_lines: Vec<String> = String::from_utf8(body_output.stdout)?
        .lines()
        .take(5)
        .map(|line| format!("- {}", line))
        .collect();

    let pr_body = body_lines.join("\n");

    let status = Command::new("gh")
        .args(&["pr", "create", "--title", pr_title, "--body", &pr_body, "--base", "main", "--head", branch_name])
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

// Function to generate PR URL
fn generate_pr_url(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url_output = Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()?;

    let repo_url = String::from_utf8(url_output.stdout)?.trim().replace(".git", "");

    if repo_url.contains("github.com") {
        Ok(format!("{}/compare/main...{}", repo_url, branch_name))
    } else {
        Ok("Unknown repository URL".to_string())
    }
}

// Function to process ready worktree
fn process_ready_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing ready worktree: {} (branch: {})", worktree_path, branch_name));

    // Push branch
    if push_branch(worktree_path, branch_name)? {
        // Try to create PR with GitHub CLI
        if Command::new("gh").arg("--version").output().is_ok() {
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
        return Ok(false);
    }
}

// Main execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CREATE WORKTREE PRs");
    println!();

    // Get list of worktrees
    let worktrees_output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    let worktrees_str = String::from_utf8(worktrees_output.stdout)?;
    let mut ready_worktrees = Vec::new();

    // Find ready worktrees
    for line in worktrees_str.lines() {
        if line.starts_with("worktree") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];

                // Get branch name from worktree path
                let branch_name = Path::new(worktree_path).file_name().unwrap().to_str().unwrap();

                // Skip main worktree
                if branch_name == "hooksmith" {
                    continue;
                }

                // Check if ready for PR
                if is_ready_for_pr(worktree_path, branch_name)? {
                    ready_worktrees.push((worktree_path.to_string(), branch_name.to_string()));
                }
            }
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

    for (worktree_path, branch_name) in &ready_worktrees {
        if process_ready_worktree(worktree_path, branch_name)? {
            processed_count += 1;
        }

        println!("---");
    }

    log_success(&format!("Processed {} worktree(s)", processed_count));

    Ok(())
}
