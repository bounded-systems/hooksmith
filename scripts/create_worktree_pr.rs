#!/usr/bin/env rust-script
//! Create PRs for Ready Worktrees
//! Automatically creates pull requests for worktrees that are ready

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
    eprintln!("{}[ERROR]{} {}", RED, NC, message);
}

fn log_header(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

// Function to check if worktree is ready for PR
fn is_ready_for_pr(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Check if clean
    let status_output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(worktree_path)
        .output()?;

    if !status_output.stdout.is_empty() {
        return Ok(false);
    }

    // Check if up to date with main
    let behind_count_output = Command::new("git")
        .args(["rev-list", "--count", "HEAD..main"])
        .current_dir(worktree_path)
        .output();

    let behind_count = if let Ok(output) = behind_count_output {
        if output.status.success() {
            String::from_utf8(output.stdout)?.trim().parse::<i32>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    if behind_count > 0 {
        return Ok(false);
    }

    // Check if has commits ahead of main
    let ahead_count_output = Command::new("git")
        .args(["rev-list", "--count", "main..HEAD"])
        .current_dir(worktree_path)
        .output();

    let ahead_count = if let Ok(output) = ahead_count_output {
        if output.status.success() {
            String::from_utf8(output.stdout)?.trim().parse::<i32>().unwrap_or(0)
        } else {
            0
        }
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
        .args(["push", "origin", branch_name])
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
        .args(["log", "--oneline", "-1"])
        .current_dir(worktree_path)
        .output()?;

    let commit_msg = String::from_utf8(commit_output.stdout)?;
    let pr_title = if let Some(space_pos) = commit_msg.find(' ') {
        commit_msg[space_pos + 1..].trim()
    } else {
        &commit_msg
    };

    // Get PR body from commit messages
    let body_output = Command::new("git")
        .args(["log", "--oneline", "main..HEAD"])
        .current_dir(worktree_path)
        .output()?;

    let body_lines: Vec<String> = String::from_utf8(body_output.stdout)?
        .lines()
        .take(5)
        .map(|line| format!("- {}", line))
        .collect();

    let pr_body = body_lines.join("\n");

    let status = Command::new("gh")
        .args([
            "pr", "create",
            "--title", pr_title,
            "--body", &pr_body,
            "--base", "main",
            "--head", branch_name
        ])
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

                // Skip main worktree
                if branch_name != "hooksmith" {
                    worktrees.push((worktree_path, branch_name));
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
    let worktrees = get_worktree_list()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    let mut ready_worktrees = Vec::new();

    // Find ready worktrees
    for (worktree_path, branch_name) in &worktrees {
        if is_ready_for_pr(worktree_path, branch_name)? {
            ready_worktrees.push((worktree_path.clone(), branch_name.clone()));
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
