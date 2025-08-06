#!/usr/bin/env rustc
//! Update Worktrees to Main
//! Systematically updates all worktrees to be based on current main

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(|s| s.as_str()).unwrap_or("help");

    match command {
        "update" => {
            process_all_worktrees()?;
            println!();
            show_status()?;
        },
        "status" => {
            show_status()?;
        },
        "help" | _ => {
            show_usage();
        }
    }

    Ok(())
}

fn log_info(message: &str) {
    println!("\x1b[0;34m[INFO]\x1b[0m {}", message);
}

fn log_success(message: &str) {
    println!("\x1b[0;32m[SUCCESS]\x1b[0m {}", message);
}

fn log_warning(message: &str) {
    println!("\x1b[1;33m[WARNING]\x1b[0m {}", message);
}

fn log_error(message: &str) {
    println!("\x1b[0;31m[ERROR]\x1b[0m {}", message);
}

fn log_header(message: &str) {
    println!("\x1b[0;35m=== {} ===\x1b[0m", message);
}

fn update_worktree_to_main(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Updating worktree: {}", branch_name));

    // Check if worktree exists
    if !Path::new(worktree_path).exists() {
        log_warning(&format!("Worktree does not exist: {}", worktree_path));
        return Ok(false);
    }

    // Check if already up to date
    let commits_behind_output = Command::new("git")
        .args(["rev-list", "--count", "HEAD..origin/main"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let commits_behind = match commits_behind_output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)?.trim().parse::<i32>().unwrap_or(0)
            } else {
                0
            }
        },
        Err(_) => 0
    };

    if commits_behind == 0 {
        log_info(&format!("Worktree {} is already up to date", branch_name));
        return Ok(true);
    }

    // Check if branch is merged
    let merged_branches_output = Command::new("git")
        .args(["branch", "--merged", "origin/main"])
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .output()?;

    let merged_branches = String::from_utf8(merged_branches_output.stdout)?;
    if merged_branches.lines().any(|line| line.trim() == branch_name) {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Remove worktree and branch
        let _ = Command::new("git")
            .args(["worktree", "remove", worktree_path])
            .output();

        let _ = Command::new("git")
            .args(["branch", "-D", branch_name])
            .output();

        return Ok(true);
    }

    // Try to rebase onto main
    log_info(&format!("Attempting to rebase {} onto main", branch_name));
    let rebase_status = Command::new("git")
        .args(["rebase", "origin/main"])
        .current_dir(worktree_path)
        .status()?;

    if rebase_status.success() {
        log_success(&format!("Successfully rebased {} onto main", branch_name));
        return Ok(true);
    } else {
        log_warning(&format!("Rebase failed for {} - creating fresh branch", branch_name));

        // Remove old worktree and create fresh one
        let _ = Command::new("git")
            .args(["worktree", "remove", worktree_path])
            .output();

        let _ = Command::new("git")
            .args(["branch", "-D", branch_name])
            .output();

        // Create new worktree based on main
        let create_status = Command::new("git")
            .args(["worktree", "add", worktree_path, "-b", branch_name])
            .status()?;

        if create_status.success() {
            log_success(&format!("Created fresh worktree for {} based on main", branch_name));
            return Ok(true);
        } else {
            log_error(&format!("Failed to create fresh worktree for {}", branch_name));
            return Ok(false);
        }
    }
}

fn process_all_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    log_header("UPDATING ALL WORKTREES TO MAIN");
    println!();

    // Get list of worktrees
    let worktree_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(worktree_output.stdout)?;
    let current_dir = env::current_dir()?.to_string_lossy().to_string();

    let worktrees: Vec<String> = worktree_list
        .lines()
        .filter(|line| line.starts_with("workdir "))
        .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
        .filter(|path| path != &current_dir)
        .collect();

    let mut updated_count = 0;
    let total_count = worktrees.len();

    for worktree_path in worktrees {
        // Get branch name from worktree path
        let branch_name = Path::new(&worktree_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        if update_worktree_to_main(&worktree_path, &branch_name)? {
            updated_count += 1;
        }
    }

    println!();
    log_success(&format!("Updated {} out of {} worktrees", updated_count, total_count));

    Ok(())
}

fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    log_header("WORKTREE STATUS AFTER UPDATE");
    println!();

    // Try to use worktree-lifecycle script if available
    if Path::new("./worktree-lifecycle/bin/worktree-lifecycle.sh").exists() {
        let output = Command::new("./worktree-lifecycle/bin/worktree-lifecycle.sh")
            .arg("status")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)?;
                    println!("{}", stdout);
                } else {
                    // Fallback to git worktree list
                    let git_output = Command::new("git")
                        .args(["worktree", "list"])
                        .output()?;

                    let git_stdout = String::from_utf8(git_output.stdout)?;
                    println!("{}", git_stdout);
                }
            },
            Err(_) => {
                // Fallback to git worktree list
                let git_output = Command::new("git")
                    .args(["worktree", "list"])
                    .output()?;

                let git_stdout = String::from_utf8(git_output.stdout)?;
                println!("{}", git_stdout);
            }
        }
    } else {
        // Use git worktree list
        let git_output = Command::new("git")
            .args(["worktree", "list"])
            .output()?;

        let git_stdout = String::from_utf8(git_output.stdout)?;
        println!("{}", git_stdout);
    }

    Ok(())
}

fn show_usage() {
    println!("Usage: {} [update|status|help]", env::args().next().unwrap());
    println!();
    println!("Commands:");
    println!("  update  - Update all worktrees to be based on current main");
    println!("  status  - Show worktree status");
    println!("  help    - Show this usage information");
    println!();
    println!("Examples:");
    println!("  {} update  # Update all worktrees to main", env::args().next().unwrap());
    println!("  {} status  # Show current status", env::args().next().unwrap());
    println!();
}