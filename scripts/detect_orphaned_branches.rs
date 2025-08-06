#!/usr/bin/env rust-script
//! Detect Orphaned Branches
//! Finds branches that exist locally but aren't in worktrees (except main)

use std::collections::HashSet;
use std::env;
use std::process::{Command, ExitCode};
use std::str::FromStr;

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

/// Show usage information
fn show_usage() {
    println!(
        r#"Detect Orphaned Branches

Usage: {} [options]

Options:
  --create-worktrees    Create worktrees for orphaned branches
  --delete-branches     Delete orphaned branches (use with caution)
  --dry-run            Show what would be done without making changes
  --help               Show this usage information

Examples:
  {}                    # Show orphaned branches
  {} --dry-run         # Show what would be done
  {} --create-worktrees # Create worktrees for orphaned branches
  {} --delete-branches  # Delete orphaned branches

This script will:
1. Find local branches that aren't in worktrees
2. Exclude main branch from orphaned list
3. Provide options to create worktrees or delete branches
4. Show summary of actions taken"#,
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
        env::args().next().unwrap_or_else(|| "script".to_string()),
    );
}

/// Check if git is available
fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git").arg("--version").output()?;
    if !output.status.success() {
        log_error("Git is required but not installed");
        return Err("Git not found".into());
    }
    Ok(())
}

/// Get worktree branches
fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["worktree", "list"]).output()?;

    if !output.status.success() {
        return Err("Failed to list worktrees".into());
    }

    let output_str = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();

    for line in output_str.lines() {
        // Extract branch name from worktree list using regex-like parsing
        if let Some(branch_start) = line.find('[') {
            if let Some(branch_end) = line[branch_start..].find(']') {
                let branch_name = &line[branch_start + 1..branch_start + branch_end];
                if !branch_name.is_empty() {
                    branches.push(branch_name.to_string());
                }
            }
        }
    }

    Ok(branches)
}

/// Get all local branches
fn get_local_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["branch", "--list"]).output()?;

    if !output.status.success() {
        return Err("Failed to list local branches".into());
    }

    let output_str = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();

    for line in output_str.lines() {
        let clean_line = line.trim();

        // Skip empty lines
        if clean_line.is_empty() {
            continue;
        }

        // Remove asterisk for current branch
        let branch_name = if clean_line.starts_with("* ") {
            &clean_line[2..]
        } else {
            clean_line
        };

        // Skip malformed lines
        if branch_name.is_empty() || branch_name.starts_with('+') {
            continue;
        }

        branches.push(branch_name.to_string());
    }

    Ok(branches)
}

/// Find orphaned branches
fn find_orphaned_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let worktree_branches = get_worktree_branches()?;
    let local_branches = get_local_branches()?;

    let worktree_set: HashSet<&str> = worktree_branches.iter().map(|s| s.as_str()).collect();
    let mut orphaned = Vec::new();

    for branch in &local_branches {
        // Skip main branch
        if branch == "main" {
            continue;
        }

        // Check if branch is in worktrees
        if !worktree_set.contains(branch.as_str()) {
            orphaned.push(branch.clone());
        }
    }

    Ok(orphaned)
}

/// Create worktree for orphaned branch
fn create_worktree_for_branch(branch_name: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing orphaned branch: {}", branch_name));

    let worktree_path = format!(".wt/{}", branch_name.replace('/', "/"));

    if dry_run {
        log_info(&format!("DRY RUN: Would create worktree for {}", branch_name));
        return Ok(true);
    }

    // Check if worktree already exists
    let output = Command::new("git").args(["worktree", "list"]).output()?;
    let output_str = String::from_utf8(output.stdout)?;

    if output_str.contains(&worktree_path) {
        log_warning(&format!("Worktree already exists at: {}", worktree_path));
        return Ok(false);
    }

    // Create the worktree
    log_info(&format!("Creating worktree for branch: {}", branch_name));
    log_info(&format!("Worktree path: {}", worktree_path));

    let status = Command::new("git")
        .args(["worktree", "add", &worktree_path, branch_name])
        .status()?;

    if status.success() {
        log_success(&format!("Successfully created worktree for branch: {}", branch_name));
        Ok(true)
    } else {
        log_error(&format!("Failed to create worktree for branch: {}", branch_name));
        Ok(false)
    }
}

/// Delete orphaned branch
fn delete_orphaned_branch(branch_name: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing orphaned branch for deletion: {}", branch_name));

    if dry_run {
        log_info(&format!("DRY RUN: Would delete branch {}", branch_name));
        return Ok(true);
    }

    // Check if branch is merged
    let merged_output = Command::new("git")
        .args(["branch", "--merged", "main"])
        .output()?;

    let merged_str = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_str.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged, deleting...", branch_name));
        let status = Command::new("git")
            .args(["branch", "-d", branch_name])
            .status()?;

        if status.success() {
            log_success(&format!("Successfully deleted merged branch: {}", branch_name));
            Ok(true)
        } else {
            log_error(&format!("Failed to delete merged branch: {}", branch_name));
            Ok(false)
        }
    } else {
        log_warning(&format!("Branch {} is not merged, use -D to force delete", branch_name));
        let status = Command::new("git")
            .args(["branch", "-D", branch_name])
            .status()?;

        if status.success() {
            log_success(&format!("Successfully force deleted branch: {}", branch_name));
            Ok(true)
        } else {
            log_error(&format!("Failed to force delete branch: {}", branch_name));
            Ok(false)
        }
    }
}

/// Handle orphaned branches
fn handle_orphaned_branches(
    create_worktrees: bool,
    delete_branches: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    log_header("DETECTING ORPHANED BRANCHES");

    // Find orphaned branches
    let orphaned = find_orphaned_branches()?;

    if orphaned.is_empty() {
        log_success("No orphaned branches found! All branches (except main) are properly managed as worktrees.");
        return Ok(());
    }

    log_warning(&format!("Found {} orphaned branches:", orphaned.len()));
    for branch in &orphaned {
        log_warning(&format!("  - {}", branch));
    }

    if create_worktrees {
        log_header("CREATING WORKTREES FOR ORPHANED BRANCHES");

        let mut created_count = 0;
        let mut failed_count = 0;

        for branch in &orphaned {
            if create_worktree_for_branch(branch, dry_run)? {
                created_count += 1;
            } else {
                failed_count += 1;
            }
        }

        log_header("WORKTREE CREATION SUMMARY");
        log_success(&format!("Created: {}", created_count));
        log_error(&format!("Failed: {}", failed_count));

    } else if delete_branches {
        log_header("DELETING ORPHANED BRANCHES");

        let mut deleted_count = 0;
        let mut failed_count = 0;

        for branch in &orphaned {
            if delete_orphaned_branch(branch, dry_run)? {
                deleted_count += 1;
            } else {
                failed_count += 1;
            }
        }

        log_header("BRANCH DELETION SUMMARY");
        log_success(&format!("Deleted: {}", deleted_count));
        log_error(&format!("Failed: {}", failed_count));

    } else {
        log_header("ORPHANED BRANCHES DETECTED");
        log_info("Use --create-worktrees to create worktrees for these branches");
        log_info("Use --delete-branches to delete these branches (use with caution)");
        log_info("Use --dry-run to see what would be done");
    }

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let mut create_worktrees = false;
    let mut delete_branches = false;
    let mut dry_run = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "--create-worktrees" => create_worktrees = true,
            "--delete-branches" => delete_branches = true,
            "--dry-run" => dry_run = true,
            "--help" => {
                show_usage();
                return ExitCode::SUCCESS;
            }
            _ => {
                log_error(&format!("Unknown option: {}", arg));
                show_usage();
                return ExitCode::FAILURE;
            }
        }
    }

    // Check dependencies
    if let Err(e) = check_dependencies() {
        log_error(&format!("Dependency check failed: {}", e));
        return ExitCode::FAILURE;
    }

    // Handle orphaned branches
    if let Err(e) = handle_orphaned_branches(create_worktrees, delete_branches, dry_run) {
        log_error(&format!("Failed to handle orphaned branches: {}", e));
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
