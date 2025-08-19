use hooksmith::{get_worktrees, log_error, log_header, log_info, log_success, log_warning};
use std::env;
use std::path::Path;
use std::process::Command;

fn check_dependencies() -> Result<(), String> {
    // Check if git is available
    if Command::new("git").arg("--version").output().is_err() {
        return Err("Git is required but not installed".to_string());
    }

    // Check if GitHub CLI is available
    if Command::new("gh").arg("--version").output().is_err() {
        return Err("GitHub CLI (gh) is required but not installed".to_string());
    }

    Ok(())
}

fn get_worktree_branches() -> Result<Vec<String>, String> {
    let worktrees = get_worktrees()?;
    let mut branches = Vec::new();

    for worktree_path in &worktrees {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        // Skip main branch and empty names
        if !branch_name.is_empty() && branch_name != "main" {
            branches.push(branch_name);
        }
    }

    Ok(branches)
}

fn branch_has_open_pr(branch_name: &str) -> Result<bool, String> {
    let output = Command::new("gh")
        .args(&[
            "pr",
            "list",
            "--head",
            branch_name,
            "--json",
            "number",
            "--jq",
            "length",
        ])
        .output()
        .map_err(|e| format!("Failed to check PR status: {}", e))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result != "0")
    } else {
        Ok(false)
    }
}

fn get_pr_number(branch_name: &str) -> Result<String, String> {
    let output = Command::new("gh")
        .args(&[
            "pr",
            "list",
            "--head",
            branch_name,
            "--json",
            "number",
            "--jq",
            ".[0].number",
        ])
        .output()
        .map_err(|e| format!("Failed to get PR number: {}", e))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result != "null" {
            Ok(result)
        } else {
            Err("No PR found for branch".to_string())
        }
    } else {
        Err("Failed to get PR number".to_string())
    }
}

fn merge_pr(pr_number: &str, branch_name: &str, force: bool) -> Result<bool, String> {
    log_info(&format!(
        "Merging PR #{} for branch: {}",
        pr_number, branch_name
    ));

    let mut args = vec!["pr", "merge", pr_number, "--delete-branch"];
    if force {
        args.push("--force");
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to merge PR: {}", e))?;

    if output.status.success() {
        log_success(&format!(
            "Successfully merged PR #{} for branch: {}",
            pr_number, branch_name
        ));
        Ok(true)
    } else {
        log_error(&format!(
            "Failed to merge PR #{} for branch: {}",
            pr_number, branch_name
        ));
        Ok(false)
    }
}

fn auto_merge_all_prs(dry_run: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    log_header("AUTO MERGING ALL PRS");

    // Get list of worktree branches
    let branches = get_worktree_branches()?;

    log_info(&format!(
        "Found {} worktree branches to check",
        branches.len()
    ));

    let mut merged_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    // Process each branch
    for branch in &branches {
        log_info(&format!("Checking branch: {}", branch));

        if branch_has_open_pr(branch)? {
            match get_pr_number(branch) {
                Ok(pr_number) => {
                    if dry_run {
                        log_info(&format!(
                            "DRY RUN: Would merge PR #{} for branch: {}",
                            pr_number, branch
                        ));
                        merged_count += 1;
                    } else {
                        if merge_pr(&pr_number, branch, force)? {
                            merged_count += 1;
                        } else {
                            failed_count += 1;
                        }
                    }
                }
                Err(e) => {
                    log_warning(&format!("Failed to get PR number for {}: {}", branch, e));
                    skipped_count += 1;
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

fn show_usage() {
    println!("Auto Merge All PRs");
    println!();
    println!("Usage: auto-merge-all-prs [options]");
    println!();
    println!("Options:");
    println!("  --dry-run           Show what would be done without making changes");
    println!("  --skip-main         Skip main branch (always skipped by default)");
    println!("  --force             Force merge even if checks are failing");
    println!("  --help              Show this usage information");
    println!();
    println!("Examples:");
    println!("  auto-merge-all-prs                    # Merge all PRs for worktrees");
    println!("  auto-merge-all-prs --dry-run         # Show what would be merged");
    println!("  auto-merge-all-prs --force           # Force merge even with failing checks");
    println!();
    println!("This script will:");
    println!("1. Find all worktrees with open PRs");
    println!("2. Merge them using gh pr merge --delete-branch");
    println!("3. Skip main branch automatically");
    println!("4. Show summary of merged PRs");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut dry_run = false;
    let mut force = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--force" => force = true,
            "--help" => {
                show_usage();
                return Ok(());
            }
            _ => {
                log_error(&format!("Unknown option: {}", arg));
                show_usage();
                return Ok(());
            }
        }
    }

    // Check dependencies
    check_dependencies()?;

    // Run the auto merge
    auto_merge_all_prs(dry_run, force)?;

    Ok(())
}
