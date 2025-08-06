#!/usr/bin/env rust-script
//! Auto Merge All PRs
//! Automatically merges all PRs for worktrees using gh pr merge --delete-branch

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

// ANSI color codes for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

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

fn show_usage() {
    println!("Auto Merge All PRs");
    println!();
    println!("Usage: auto_merge_all_prs [options]");
    println!();
    println!("Options:");
    println!("  --dry-run           Show what would be done without making changes");
    println!("  --skip-main         Skip main branch (always skipped by default)");
    println!("  --force             Force merge even if checks are failing");
    println!("  --help              Show this usage information");
    println!();
    println!("Examples:");
    println!("  auto_merge_all_prs                    # Merge all PRs for worktrees");
    println!("  auto_merge_all_prs --dry-run         # Show what would be merged");
    println!("  auto_merge_all_prs --force           # Force merge even with failing checks");
    println!();
    println!("This script will:");
    println!("1. Find all worktrees with open PRs");
    println!("2. Merge them using gh pr merge --delete-branch");
    println!("3. Skip main branch automatically");
    println!("4. Show summary of merged PRs");
}

fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Check if git is available
    let git_check = Command::new("git")
        .args(&["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if git_check.is_err() {
        log_error("Git is required but not installed");
        std::process::exit(1);
    }
    
    // Check if gh CLI is available
    let gh_check = Command::new("gh")
        .args(&["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if gh_check.is_err() {
        log_error("GitHub CLI (gh) is required but not installed");
        std::process::exit(1);
    }
    
    Ok(())
}

fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .stdout(Stdio::piped())
        .output()?;
    
    let worktree_list = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();
    
    for line in worktree_list.lines() {
        // Extract branch name from worktree list using regex-like parsing
        if let Some(branch_start) = line.find('[') {
            if let Some(branch_end) = line.find(']') {
                if branch_end > branch_start {
                    let branch_name = &line[branch_start + 1..branch_end];
                    
                    // Skip main branch
                    if branch_name != "main" && !branch_name.is_empty() {
                        branches.push(branch_name.to_string());
                    }
                }
            }
        }
    }
    
    Ok(branches)
}

fn get_open_prs_for_branch(branch_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args(&["pr", "list", "--head", branch_name, "--json", "number,title", "--jq", ".[].number"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;
    
    if output.status.success() {
        let pr_list = String::from_utf8(output.stdout)?;
        let prs: Vec<String> = pr_list.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();
        Ok(prs)
    } else {
        Ok(Vec::new())
    }
}

fn merge_pr(pr_number: &str, force: bool, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    if dry_run {
        log_info(&format!("DRY RUN: Would merge PR #{}", pr_number));
        return Ok(true);
    }
    
    let mut args = vec!["pr", "merge", pr_number, "--delete-branch"];
    
    if force {
        args.push("--force");
    }
    
    let status = Command::new("gh")
        .args(&args)
        .status()?;
    
    if status.success() {
        log_success(&format!("Successfully merged PR #{}", pr_number));
        Ok(true)
    } else {
        log_warning(&format!("Failed to merge PR #{}", pr_number));
        Ok(false)
    }
}

fn process_branch_prs(branch_name: &str, force: bool, dry_run: bool) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    log_info(&format!("Checking PRs for branch: {}", branch_name));
    
    let prs = get_open_prs_for_branch(branch_name)?;
    
    if prs.is_empty() {
        log_info(&format!("No open PRs found for branch: {}", branch_name));
        return Ok((0, 0));
    }
    
    log_info(&format!("Found {} open PR(s) for branch: {}", prs.len(), branch_name));
    
    let mut merged_count = 0;
    let mut failed_count = 0;
    
    for pr_number in &prs {
        if merge_pr(pr_number, force, dry_run)? {
            merged_count += 1;
        } else {
            failed_count += 1;
        }
    }
    
    Ok((merged_count, failed_count))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut dry_run = false;
    let mut force = false;
    let mut skip_main = false;
    
    for arg in &args[1..] {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--force" => force = true,
            "--skip-main" => skip_main = true,
            "--help" | "-h" => {
                show_usage();
                return Ok(());
            }
            _ => {
                log_error(&format!("Unknown option: {}", arg));
                show_usage();
                std::process::exit(1);
            }
        }
    }
    
    // Check dependencies
    check_dependencies()?;
    
    log_header("AUTO MERGE ALL PRs");
    println!();
    
    if dry_run {
        log_info("DRY RUN MODE - No changes will be made");
        println!();
    }
    
    if force {
        log_warning("FORCE MODE - Will merge even with failing checks");
        println!();
    }
    
    // Get worktree branches
    let branches = get_worktree_branches()?;
    
    if branches.is_empty() {
        log_info("No worktree branches found");
        return Ok(());
    }
    
    log_info(&format!("Found {} worktree branch(es) to check", branches.len()));
    println!();
    
    let mut total_merged = 0;
    let mut total_failed = 0;
    let mut processed_branches = 0;
    
    // Process each branch
    for branch in &branches {
        let (merged, failed) = process_branch_prs(branch, force, dry_run)?;
        
        if merged > 0 || failed > 0 {
            processed_branches += 1;
            total_merged += merged;
            total_failed += failed;
        }
        
        println!("---");
    }
    
    // Show summary
    log_header("MERGE SUMMARY");
    log_info(&format!("Branches processed: {}", processed_branches));
    log_success(&format!("Successfully merged: {}", total_merged));
    
    if total_failed > 0 {
        log_error(&format!("Failed to merge: {}", total_failed));
    }
    
    if dry_run {
        log_info("DRY RUN: No actual changes were made");
    }
    
    Ok(())
} 
