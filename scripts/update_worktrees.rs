#!/usr/bin/env rustc
//! Update all worktrees to latest origin/main
//! This script will rebase each worktree's base branch to origin/main

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Updating all worktrees to latest origin/main...");

    // Get the latest from origin
    println!("📥 Fetching latest from origin...");
    let fetch_status = Command::new("git")
        .args(["fetch", "origin"])
        .status()?;

    if !fetch_status.success() {
        eprintln!("❌ Failed to fetch from origin");
        std::process::exit(1);
    }

    // Get list of worktrees (excluding the main worktree)
    let worktree_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(worktree_output.stdout)?;
    let current_dir = env::current_dir()?.to_string_lossy().to_string();

    let worktrees: Vec<String> = worktree_list
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
        .filter(|path| path != &current_dir)
        .collect();

    for worktree in worktrees {
        if Path::new(&worktree).exists() {
            println!();
            println!("🔄 Updating worktree: {}", Path::new(&worktree).file_name().unwrap().to_string_lossy());
            println!("   Path: {}", worktree);

            // Get the branch name for this worktree
            let branch_output = Command::new("git")
                .args(["branch", "--show-current"])
                .current_dir(&worktree)
                .stdout(Stdio::piped())
                .output()?;

            let branch = String::from_utf8(branch_output.stdout)?.trim().to_string();
            println!("   Branch: {}", branch);

            // Check if there are uncommitted changes
            let status_output = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(&worktree)
                .stdout(Stdio::piped())
                .output()?;

            let uncommitted = String::from_utf8(status_output.stdout)?;
            if !uncommitted.trim().is_empty() {
                println!("   ⚠️  WARNING: Uncommitted changes detected!");
                println!("   📝 Changes:");

                let status_short_output = Command::new("git")
                    .args(["status", "--short"])
                    .current_dir(&worktree)
                    .stdout(Stdio::piped())
                    .output()?;

                let status_short = String::from_utf8(status_short_output.stdout)?;
                for line in status_short.lines() {
                    println!("   {}", line);
                }

                println!("   💡 Consider committing or stashing changes before updating");
                println!("   ⏭️  Skipping this worktree...");
                continue;
            }

            // Get current commit
            let current_commit_output = Command::new("git")
                .args(["log", "--oneline", "-1"])
                .current_dir(&worktree)
                .stdout(Stdio::piped())
                .output()?;

            let current_commit = String::from_utf8(current_commit_output.stdout)?.trim().to_string();
            println!("   Current commit: {}", current_commit);

            // Check how many commits behind origin/main
            let behind_count_output = Command::new("git")
                .args(["rev-list", "--count", "HEAD..origin/main"])
                .current_dir(&worktree)
                .stdout(Stdio::piped())
                .output()?;

            let behind_count = String::from_utf8(behind_count_output.stdout)?.trim().parse::<i32>().unwrap_or(0);
            println!("   Behind origin/main by: {} commits", behind_count);

            if behind_count == 0 {
                println!("   ✅ Already up to date!");
                continue;
            }

            // Rebase to origin/main
            println!("   🔄 Rebasing to origin/main...");
            let rebase_status = Command::new("git")
                .args(["rebase", "origin/main"])
                .current_dir(&worktree)
                .status()?;

            if rebase_status.success() {
                println!("   ✅ Successfully updated!");

                let new_commit_output = Command::new("git")
                    .args(["log", "--oneline", "-1"])
                    .current_dir(&worktree)
                    .stdout(Stdio::piped())
                    .output()?;

                let new_commit = String::from_utf8(new_commit_output.stdout)?.trim().to_string();
                println!("   New commit: {}", new_commit);
            } else {
                println!("   ❌ Rebase failed! Manual intervention may be needed.");
                println!("   💡 You can:");
                println!("      - cd {}", worktree);
                println!("      - git rebase --abort (to cancel)");
                println!("      - git rebase --continue (after resolving conflicts)");
            }
        }
    }

    println!();
    println!("🎉 Worktree update process completed!");
    println!();
    println!("📊 Summary:");

    // Get main worktree info
    let main_commit_output = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .output()?;

    let main_commit = String::from_utf8(main_commit_output.stdout)?.trim().to_string();
    println!("   - Main worktree: {}", main_commit);

    // Get origin/main info
    let origin_main_output = Command::new("git")
        .args(["log", "--oneline", "origin/main", "-1"])
        .output()?;

    let origin_main = String::from_utf8(origin_main_output.stdout)?.trim().to_string();
    println!("   - Origin/main: {}", origin_main);
    println!();
    println!("💡 Next steps:");
    println!("   - Review any worktrees that had conflicts");
    println!("   - Test your changes in updated worktrees");
    println!("   - Create PRs for worktrees that are ready");

    Ok(())
}
