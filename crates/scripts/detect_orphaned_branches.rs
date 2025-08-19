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

// Function to show usage
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
        env::args().next().unwrap_or_else(|| "script".to_string())
    );
}

// Function to check dependencies
fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    if Command::new("git").arg("--version").output().is_err() {
        log_error("Git is required but not installed");
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
            if !branch_name.is_empty() {
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

// Function to get all local branches
fn get_local_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "--list"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();

    for line in output_str.lines() {
        // Clean the line and extract branch name
        let clean_line = line.trim();

        // Remove asterisk for current branch
        let branch_name = if clean_line.starts_with("* ") {
            &clean_line[2..]
        } else {
            clean_line
        };

        // Skip empty branch names and malformed lines
        if !branch_name.is_empty() && !branch_name.starts_with('+') {
            branches.push(branch_name.to_string());
        }
    }

    Ok(branches)
}

// Function to find orphaned branches
fn find_orphaned_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let worktree_branches = get_worktree_branches()?;
    let local_branches = get_local_branches()?;
    let mut orphaned = Vec::new();

    for branch in local_branches {
        // Skip main branch
        if branch == "main" {
            continue;
        }

        // Check if branch is in worktrees
        if !worktree_branches.contains(&branch) {
            orphaned.push(branch);
        }
    }

    Ok(orphaned)
}

// Function to create worktree for orphaned branch
fn create_worktree_for_branch(branch_name: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing orphaned branch: {}", branch_name));

    let worktree_path = format!(".wt/{}", branch_name.replace('/', "\\/"));

    if dry_run {
        log_info(&format!("DRY RUN: Would create worktree for {}", branch_name));
        return Ok(true);
    }

    // Check if worktree already exists
    let worktree_list_output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let worktree_list = String::from_utf8(worktree_list_output.stdout)?;
    if worktree_list.contains(&worktree_path) {
        log_warning(&format!("Worktree already exists at: {}", worktree_path));
        return Ok(false);
    }

    // Create the worktree
    log_info(&format!("Creating worktree for branch: {}", branch_name));
    log_info(&format!("Worktree path: {}", worktree_path));

    let status = Command::new("git")
        .args(&["worktree", "add", &worktree_path, branch_name])
        .status()?;

    if status.success() {
        log_success(&format!("Successfully created worktree for branch: {}", branch_name));
        Ok(true)
    } else {
        log_error(&format!("Failed to create worktree for branch: {}", branch_name));
        Ok(false)
    }
}

// Function to delete orphaned branch
fn delete_orphaned_branch(branch_name: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing orphaned branch for deletion: {}", branch_name));

    if dry_run {
        log_info(&format!("DRY RUN: Would delete branch {}", branch_name));
        return Ok(true);
    }

    // Check if branch is merged
    let merged_output = Command::new("git")
        .args(&["branch", "--merged", "main"])
        .output()?;

    let merged_branches = String::from_utf8(merged_output.stdout)?;
    let is_merged = merged_branches.lines().any(|line| line.trim() == branch_name);

    if is_merged {
        log_info(&format!("Branch {} is merged, deleting...", branch_name));
        let status = Command::new("git")
            .args(&["branch", "-d", branch_name])
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
            .args(&["branch", "-D", branch_name])
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

// Function to handle orphaned branches
fn handle_orphaned_branches(create_worktrees: bool, delete_branches: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
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

// Main execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut create_worktrees = false;
    let mut delete_branches = false;
    let mut dry_run = false;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--create-worktrees" => {
                create_worktrees = true;
            }
            "--delete-branches" => {
                delete_branches = true;
            }
            "--dry-run" => {
                dry_run = true;
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

    // Handle orphaned branches
    handle_orphaned_branches(create_worktrees, delete_branches, dry_run)?;

    Ok(())
}

