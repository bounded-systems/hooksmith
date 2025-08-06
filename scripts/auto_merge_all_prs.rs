#!/usr/bin/env rust-script

use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::env;

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

// Function to show usage
fn show_usage() {
    println!(
        r#"Auto Merge All PRs

Usage: {} [options]

Options:
  --dry-run           Show what would be done without making changes
  --skip-main         Skip main branch (always skipped by default)
  --force             Force merge even if checks are failing
  --help              Show this usage information

Examples:
  {}                    # Merge all PRs for worktrees
  {} --dry-run         # Show what would be merged
  {} --force           # Force merge even with failing checks

This script will:
1. Find all worktrees with open PRs
2. Merge them using gh pr merge --delete-branch
3. Skip main branch automatically
4. Show summary of merged PRs"#,
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string())
    );
}

// Function to check dependencies
fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let mut missing_deps = Vec::new();

    // Check for git
    if Command::new("git").arg("--version").output().is_err() {
        missing_deps.push("git");
    }

    // Check for gh
    if Command::new("gh").arg("--version").output().is_err() {
        log_error("GitHub CLI (gh) is required but not installed");
        std::process::exit(1);
    }

    if !missing_deps.is_empty() {
        log_error(&format!("Missing required dependencies: {}", missing_deps.join(", ")));
        std::process::exit(1);
    }

    Ok(())
}

// Function to get worktree branches
fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();

    for line in output_str.lines() {
        // Extract branch name from worktree list using regex
        if let Some(branch_name) = extract_branch_name(line) {
            // Skip main branch
            if branch_name != "main" && !branch_name.is_empty() {
                branches.push(branch_name);
            }
        }
    }

    Ok(branches)
}

// Function to extract branch name from worktree list line
fn extract_branch_name(line: &str) -> Option<String> {
    // Look for pattern like [branch-name] in the line
    if let Some(start) = line.find('[') {
        if let Some(end) = line[start..].find(']') {
            let branch_name = line[start + 1..start + end].trim();
            if !branch_name.is_empty() {
                return Some(branch_name.to_string());
            }
        }
    }
    None
}

// Function to check if branch has open PR
fn branch_has_open_pr(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args(&["pr", "list", "--head", branch_name, "--json", "number", "--jq", "length"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let count = output_str.trim().parse::<i32>().unwrap_or(0);

    Ok(count > 0)
}

// Function to get PR number for branch
fn get_pr_number(branch_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args(&["pr", "list", "--head", branch_name, "--json", "number", "--jq", ".[0].number"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let pr_number = output_str.trim();

    if pr_number.is_empty() || pr_number == "null" {
        Ok(None)
    } else {
        Ok(Some(pr_number.to_string()))
    }
}

// Function to merge PR
fn merge_pr(pr_number: &str, branch_name: &str, force: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Merging PR #{} for branch: {}", pr_number, branch_name));

    let mut args = vec!["pr", "merge", pr_number, "--delete-branch"];
    if force {
        args.push("--force");
    }

    let status = Command::new("gh")
        .args(&args)
        .status()?;

    if status.success() {
        log_success(&format!("Successfully merged PR #{} for branch: {}", pr_number, branch_name));
        Ok(true)
    } else {
        log_error(&format!("Failed to merge PR #{} for branch: {}", pr_number, branch_name));
        Ok(false)
    }
}

// Function to auto merge all PRs
fn auto_merge_all_prs(dry_run: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    log_header("AUTO MERGING ALL PRS");

    // Get list of worktree branches
    let branches = get_worktree_branches()?;

    log_info(&format!("Found {} worktree branches to check", branches.len()));

    let mut merged_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    // Process each branch
    for branch in &branches {
        log_info(&format!("Checking branch: {}", branch));

        if branch_has_open_pr(branch)? {
            if let Some(pr_number) = get_pr_number(branch)? {
                if dry_run {
                    log_info(&format!("DRY RUN: Would merge PR #{} for branch: {}", pr_number, branch));
                    merged_count += 1;
                } else {
                    if merge_pr(&pr_number, branch, force)? {
                        merged_count += 1;
                    } else {
                        failed_count += 1;
                    }
                }
            }
        } else {
            log_info(&format!("No open PR found for branch: {}", branch));
            skipped_count += 1;
        }
    }

    // Show summary
    log_header("AUTO MERGE SUMMARY");
    log_info(&format!("Branches checked: {}", branches.len()));
    log_success(&format!("Merged: {}", merged_count));
    log_warning(&format!("Skipped: {}", skipped_count));
    log_error(&format!("Failed: {}", failed_count));

    if !dry_run && merged_count > 0 {
        log_info("Use './worktree-lifecycle/bin/worktree-lifecycle.sh cleanup' to clean up merged worktrees");
    }

    Ok(())
}

// Main execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut dry_run = false;
    let mut force = false;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => {
                dry_run = true;
            }
            "--force" => {
                force = true;
            }
            "--help" => {
                show_usage();
                return Ok(());
            }
            _ => {
                log_error(&format!("Unknown option: {}", args[i]));
                show_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Check dependencies
    check_dependencies()?;

    // Run the auto merge
    auto_merge_all_prs(dry_run, force)?;

    Ok(())
}

