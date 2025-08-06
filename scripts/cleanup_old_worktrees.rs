#!/usr/bin/env rust-script

use std::process::{Command, Stdio};
use std::env;
use std::path::Path;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m"; // No Color

fn print_status(state: &str, message: &str) {
    match state {
        "ERROR" => println!("{}❌ {}{}", RED, message, NC),
        "SUCCESS" => println!("{}✅ {}{}", GREEN, message, NC),
        "WARNING" => println!("{}⚠️  {}{}", YELLOW, message, NC),
        "INFO" => println!("{}ℹ️  {}{}", BLUE, message, NC),
        _ => println!("📝 {}", message),
    }
}

// List of worktrees to remove (old conflicted ones)
const OLD_WORKTREES: &[&str] = &[
    "worktree-fix-main-cleanup-20250804-211403",
    "worktree-fix-workspace-config",
    "worktree-fix-workspace-dependencies",
    "worktree-fix-xtask-cleanup",
];

// Function to remove a worktree
fn remove_worktree(worktree_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(worktree_name).exists() {
        print_status("WARNING", &format!("Worktree {} does not exist", worktree_name));
        return Ok(());
    }

    print_status("INFO", &format!("Removing worktree: {}", worktree_name));

    // Abort any ongoing operations first
    let _ = Command::new("git")
        .args(&["rebase", "--abort"])
        .current_dir(worktree_name)
        .output();

    // Get branch name before removal
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_name)
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    // Remove worktree
    print_status("INFO", "Removing worktree directory");
    let remove_result = Command::new("git")
        .args(&["worktree", "remove", worktree_name, "--force"])
        .output();

    if remove_result.is_err() || !remove_result.unwrap().status.success() {
        print_status("WARNING", "Could not remove worktree, trying to delete directory");
        std::fs::remove_dir_all(worktree_name)?;
    }

    // Remove branch if it exists
    if !branch.is_empty() {
        print_status("INFO", &format!("Removing branch: {}", branch));
        let _ = Command::new("git")
            .args(&["branch", "-D", &branch])
            .output();
    }

    print_status("SUCCESS", &format!("Removed worktree {}", worktree_name));
    println!();

    Ok(())
}

// Function to create PR for ready worktree
fn create_pr_for_ready() -> Result<(), Box<dyn std::error::Error>> {
    let worktree_name = "worktree-management-improvements";

    if !Path::new(worktree_name).exists() {
        print_status("WARNING", &format!("Ready worktree {} does not exist", worktree_name));
        return Ok(());
    }

    print_status("INFO", &format!("Creating PR for ready worktree: {}", worktree_name));

    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_name)
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    if !branch.is_empty() {
        // Check if branch exists on origin
        let remote_output = Command::new("git")
            .args(&["ls-remote", "--heads", "origin", &branch])
            .current_dir(worktree_name)
            .output()?;

        if remote_output.status.success() && !String::from_utf8(remote_output.stdout)?.trim().is_empty() {
            // Get repo URL
            let url_output = Command::new("git")
                .args(&["config", "--get", "remote.origin.url"])
                .current_dir(worktree_name)
                .output()?;

            if url_output.status.success() {
                let repo_url = String::from_utf8(url_output.stdout)?.trim().replace(".git", "");
                if repo_url.contains("github.com") {
                    let pr_url = format!("{}/compare/main...{}", repo_url, branch);
                    print_status("SUCCESS", &format!("Create PR at: {}", pr_url));
                }
            }
        }
    }

    Ok(())
}

// Function to run worktree status report
fn run_worktree_status_report() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("./scripts/worktree-status-report.sh")
        .status()?;

    if !status.success() {
        print_status("WARNING", "Worktree status report failed");
    }

    Ok(())
}

// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 CLEANING UP OLD WORKTREES");
    println!("============================");
    println!();

    println!("🗑️  Removing old conflicted worktrees...");
    println!();

    for worktree in OLD_WORKTREES {
        remove_worktree(worktree)?;
    }

    println!("🚀 Creating PR for ready worktree...");
    println!();
    create_pr_for_ready()?;

    println!("🎉 Cleanup completed!");
    println!();
    println!("📊 Final Status:");
    run_worktree_status_report()?;

    Ok(())
}
