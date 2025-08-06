#!/usr/bin/env rustx

use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

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
        .args(&["-C", worktree_name, "rebase", "--abort"])
        .stderr(Stdio::null())
        .status();

    // Get branch name before removal
    let branch_output = Command::new("git")
        .args(&["-C", worktree_name, "branch", "--show-current"])
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    // Remove worktree
    print_status("INFO", "Removing worktree directory");
    let worktree_remove = Command::new("git")
        .args(&["worktree", "remove", worktree_name, "--force"])
        .stderr(Stdio::null())
        .status();

    if worktree_remove.is_err() || !worktree_remove.unwrap().success() {
        print_status("WARNING", "Could not remove worktree, trying to delete directory");
        let _ = Command::new("rm").args(&["-rf", worktree_name]).status();
    }

    // Remove branch if it exists
    if !branch.is_empty() {
        print_status("INFO", &format!("Removing branch: {}", branch));
        let _ = Command::new("git")
            .args(&["branch", "-D", &branch])
            .stderr(Stdio::null())
            .status();
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
        .args(&["-C", worktree_name, "branch", "--show-current"])
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    if !branch.is_empty() {
        // Check if branch exists on origin
        let remote_check = Command::new("git")
            .args(&["-C", worktree_name, "ls-remote", "--heads", "origin", &branch])
            .output()?;

        if remote_check.status.success() {
            let remote_output = String::from_utf8(remote_check.stdout)?;
            if remote_output.contains(&branch) {
                // Get remote URL
                let url_output = Command::new("git")
                    .args(&["-C", worktree_name, "config", "--get", "remote.origin.url"])
                    .output()?;

                if url_output.status.success() {
                    let repo_url = String::from_utf8(url_output.stdout)?
                        .trim()
                        .replace(".git", "");

                    if repo_url.contains("github.com") {
                        let pr_url = format!("{}/compare/main...{}", repo_url, branch);
                        print_status("SUCCESS", &format!("Create PR at: {}", pr_url));
                    }
                }
            }
        }
    }

    Ok(())
}

// Function to run worktree status report
fn run_worktree_status_report() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("./scripts/worktree-status-report.sh").status()?;
    if !status.success() {
        print_status("WARNING", "Failed to run worktree status report");
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
