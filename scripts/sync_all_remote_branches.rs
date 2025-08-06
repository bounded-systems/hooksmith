#!/usr/bin/env -S rustc --edition=2021 -o /tmp/sync-all-remote-branches && /tmp/sync-all-remote-branches

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;
use std::collections::HashSet;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

#[derive(Debug)]
struct SyncOptions {
    dry_run: bool,
    skip_main: bool,
    force: bool,
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let options = parse_options(&args);
    
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        show_usage();
        return Ok(());
    }
    
    log_header("Sync All Remote Branches to Worktrees");
    
    // Check dependencies
    check_dependencies()?;
    
    // Fetch remote branches
    fetch_remote_branches(&options)?;
    
    // Get remote branches
    let remote_branches = get_remote_branches(&options)?;
    
    // Sync all branches
    sync_all_branches(&remote_branches, &options)?;
    
    log_success("Remote branch sync completed!");
    Ok(())
}

fn parse_options(args: &[String]) -> SyncOptions {
    SyncOptions {
        dry_run: args.iter().any(|arg| arg == "--dry-run"),
        skip_main: args.iter().any(|arg| arg == "--skip-main"),
        force: args.iter().any(|arg| arg == "--force"),
        verbose: args.iter().any(|arg| arg == "--verbose" || arg == "-v"),
    }
}

fn show_usage() {
    println!("Sync All Remote Branches to Worktrees");
    println!();
    println!("Usage: sync_all_remote_branches [options]");
    println!();
    println!("Options:");
    println!("  --dry-run           Show what would be done without making changes");
    println!("  --skip-main         Skip creating worktree for main branch");
    println!("  --force             Force recreation of existing worktrees");
    println!("  --verbose, -v       Show detailed output");
    println!("  --help, -h          Show this usage information");
    println!();
    println!("Examples:");
    println!("  sync_all_remote_branches --dry-run");
    println!("  sync_all_remote_branches --skip-main --verbose");
    println!("  sync_all_remote_branches --force");
}

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

fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Checking dependencies...");
    
    let required_commands = ["git", "gh"];
    
    for cmd in &required_commands {
        let output = Command::new("which")
            .arg(cmd)
            .output();
        
        match output {
            Ok(_) => log_success(&format!("✓ {} found", cmd)),
            Err(_) => {
                log_error(&format!("✗ {} not found", cmd));
                return Err(format!("Required command '{}' not found", cmd).into());
            }
        }
    }
    
    Ok(())
}

fn fetch_remote_branches(options: &SyncOptions) -> Result<(), Box<dyn std::error::Error>> {
    log_info("Fetching remote branches...");
    
    if options.dry_run {
        log_info("DRY RUN: Would fetch remote branches");
        return Ok(());
    }
    
    let output = Command::new("git")
        .args(["fetch", "--all"])
        .output()?;
    
    if output.status.success() {
        log_success("Remote branches fetched successfully");
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        log_error(&format!("Failed to fetch remote branches: {}", error));
        return Err("Git fetch failed".into());
    }
    
    Ok(())
}

fn get_remote_branches(options: &SyncOptions) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    log_info("Getting remote branches...");
    
    let output = Command::new("git")
        .args(["branch", "-r", "--format=%(refname:short)"])
        .output()?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        log_error(&format!("Failed to get remote branches: {}", error));
        return Err("Git branch command failed".into());
    }
    
    let branches_str = String::from_utf8_lossy(&output.stdout);
    let mut branches: Vec<String> = branches_str
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    
    // Filter out HEAD and main if skip_main is set
    branches.retain(|branch| {
        if options.skip_main && (branch == "origin/main" || branch == "main") {
            log_info(&format!("Skipping main branch: {}", branch));
            false
        } else {
            true
        }
    });
    
    if options.verbose {
        log_info(&format!("Found {} remote branches", branches.len()));
        for branch in &branches {
            log_info(&format!("  - {}", branch));
        }
    }
    
    Ok(branches)
}

fn worktree_exists(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()?;
    
    if !output.status.success() {
        return Err("Git worktree list failed".into());
    }
    
    let worktrees_str = String::from_utf8_lossy(&output.stdout);
    let branch_without_origin = branch_name.replace("origin/", "");
    
    for line in worktrees_str.lines() {
        if line.contains(&branch_without_origin) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

fn create_worktree(branch_name: &str, options: &SyncOptions) -> Result<(), Box<dyn std::error::Error>> {
    let branch_without_origin = branch_name.replace("origin/", "");
    let worktree_path = format!("worktree-{}", branch_without_origin);
    
    if worktree_exists(branch_name)? {
        if options.force {
            log_warning(&format!("Worktree for {} already exists, forcing recreation", branch_name));
            // TODO: Implement worktree removal and recreation
        } else {
            log_info(&format!("Worktree for {} already exists, skipping", branch_name));
            return Ok(());
        }
    }
    
    if options.dry_run {
        log_info(&format!("DRY RUN: Would create worktree for {} at {}", branch_name, worktree_path));
        return Ok(());
    }
    
    log_info(&format!("Creating worktree for {} at {}", branch_name, worktree_path));
    
    let output = Command::new("git")
        .args(["worktree", "add", &worktree_path, &branch_without_origin])
        .output()?;
    
    if output.status.success() {
        log_success(&format!("Worktree created for {}", branch_name));
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        log_error(&format!("Failed to create worktree for {}: {}", branch_name, error));
        return Err(format!("Worktree creation failed for {}", branch_name).into());
    }
    
    Ok(())
}

fn sync_main_branch(options: &SyncOptions) -> Result<(), Box<dyn std::error::Error>> {
    if options.skip_main {
        log_info("Skipping main branch sync (--skip-main specified)");
        return Ok(());
    }
    
    log_info("Syncing main branch...");
    
    if options.dry_run {
        log_info("DRY RUN: Would sync main branch");
        return Ok(());
    }
    
    // Checkout main and pull latest changes
    let output = Command::new("git")
        .args(["checkout", "main"])
        .output()?;
    
    if !output.status.success() {
        log_error("Failed to checkout main branch");
        return Err("Main branch checkout failed".into());
    }
    
    let pull_output = Command::new("git")
        .args(["pull", "origin", "main"])
        .output()?;
    
    if pull_output.status.success() {
        log_success("Main branch synced successfully");
    } else {
        let error = String::from_utf8_lossy(&pull_output.stderr);
        log_warning(&format!("Failed to pull main branch: {}", error));
    }
    
    Ok(())
}

fn sync_all_branches(branches: &[String], options: &SyncOptions) -> Result<(), Box<dyn std::error::Error>> {
    log_header("Syncing All Remote Branches");
    
    let mut created_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;
    
    for branch in branches {
        match create_worktree(branch, options) {
            Ok(_) => {
                if !options.dry_run {
                    created_count += 1;
                }
            }
            Err(e) => {
                log_error(&format!("Failed to create worktree for {}: {}", branch, e));
                error_count += 1;
            }
        }
    }
    
    // Sync main branch
    if let Err(e) = sync_main_branch(options) {
        log_error(&format!("Failed to sync main branch: {}", e));
        error_count += 1;
    }
    
    // Summary
    log_header("Sync Summary");
    if options.dry_run {
        log_info("DRY RUN: No actual changes made");
    } else {
        log_success(&format!("Created {} worktrees", created_count));
        if skipped_count > 0 {
            log_info(&format!("Skipped {} existing worktrees", skipped_count));
        }
        if error_count > 0 {
            log_warning(&format!("{} errors occurred", error_count));
        }
    }
    
    Ok(())
}
