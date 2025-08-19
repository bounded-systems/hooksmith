#!/usr/bin/env rust-script

use std::process::{Command, Stdio};
use std::env;
use std::path::Path;
use std::collections::HashMap;
use regex::Regex;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

#[derive(Debug, Clone)]
enum Decision {
    Remove,
    Cleanup,
    Keep,
}

fn print_status(state: &str, message: &str) {
    match state {
        "ERROR" => println!("{}❌ {}{}", RED, message, NC),
        "SUCCESS" => println!("{}✅ {}{}", GREEN, message, NC),
        "WARNING" => println!("{}⚠️  {}{}", YELLOW, message, NC),
        "INFO" => println!("{}ℹ️  {}{}", BLUE, message, NC),
        "DECISION" => println!("{}🤔 {}{}", PURPLE, message, NC),
        _ => println!("📝 {}", message),
    }
}

// Function to analyze worktree and make decision
fn analyze_worktree(worktree_path: &str) -> Result<Decision, Box<dyn std::error::Error>> {
    let worktree_name = Path::new(worktree_path).file_name().unwrap().to_str().unwrap();

    println!("=== ANALYZING: {} ===", worktree_name);

    if !Path::new(worktree_path).exists() {
        print_status("ERROR", &format!("Worktree {} does not exist", worktree_name));
        return Ok(Decision::Remove);
    }

    // Get branch name
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_path)
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    print_status("INFO", &format!("Branch: {}", branch));

    // Check commit history
    let commit_output = Command::new("git")
        .args(&["log", "--oneline", "--since=1 week ago"])
        .current_dir(worktree_path)
        .output()?;

    let commit_count = String::from_utf8(commit_output.stdout)?.lines().count();

    let last_commit_output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(worktree_path)
        .output()?;

    let last_commit = if last_commit_output.status.success() {
        String::from_utf8(last_commit_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    print_status("INFO", &format!("Recent commits: {}", commit_count));
    print_status("INFO", &format!("Last commit: {}", last_commit));

    // Check for conflicts
    let conflicts_output = Command::new("git")
        .args(&["diff", "--name-only", "--diff-filter=U"])
        .current_dir(worktree_path)
        .output();

    let rebase_output = Command::new("git")
        .args(&["status"])
        .current_dir(worktree_path)
        .output()?;

    let has_conflicts = conflicts_output.is_ok() && !String::from_utf8(conflicts_output.unwrap().stdout)?.trim().is_empty();
    let in_rebase = String::from_utf8(rebase_output.stdout)?.contains("rebase");

    if has_conflicts || in_rebase {
        print_status("WARNING", "Worktree has conflicts or is in rebase state");

        // Check if this worktree is from old development
        let date_regex = Regex::new(r"202508[0-9][0-9]")?;
        if date_regex.is_match(worktree_name) {
            print_status("DECISION", "This appears to be an old worktree from 202508xx");
            print_status("DECISION", "Recommendation: REMOVE (likely obsolete)");
            return Ok(Decision::Remove);
        }
    }

    // Check if branch is behind main
    let behind_output = Command::new("git")
        .args(&["rev-list", "--count", "HEAD..origin/main"])
        .current_dir(worktree_path)
        .output();

    let behind_count = if behind_output.is_ok() {
        String::from_utf8(behind_output.unwrap().stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if behind_count > 5 {
        print_status("WARNING", &format!("Branch is {} commits behind main", behind_count));
        print_status("DECISION", "Recommendation: REMOVE (too far behind)");
        return Ok(Decision::Remove);
    }

    // Check if branch exists on origin
    let remote_output = Command::new("git")
        .args(&["ls-remote", "--heads", "origin", &branch])
        .current_dir(worktree_path)
        .output()?;

    let exists_on_origin = remote_output.status.success() && !String::from_utf8(remote_output.stdout)?.trim().is_empty();

    if exists_on_origin {
        print_status("INFO", "Branch exists on origin");

        // Check if merged
        let merged_output = Command::new("git")
            .args(&["branch", "--merged", "origin/main"])
            .current_dir(worktree_path)
            .output()?;

        let is_merged = String::from_utf8(merged_output.stdout)?.lines().any(|line| line.trim() == branch);

        if is_merged {
            print_status("SUCCESS", "Branch is merged");
            print_status("DECISION", "Recommendation: CLEANUP (merged)");
            return Ok(Decision::Cleanup);
        } else {
            print_status("INFO", "Branch not merged");
            print_status("DECISION", "Recommendation: KEEP (active development)");
            return Ok(Decision::Keep);
        }
    } else {
        print_status("WARNING", "Branch does not exist on origin");
        print_status("DECISION", "Recommendation: REMOVE (no remote branch)");
        return Ok(Decision::Remove);
    }
}

// Function to execute decision
fn execute_decision(worktree_path: &str, decision: Decision) -> Result<(), Box<dyn std::error::Error>> {
    let worktree_name = Path::new(worktree_path).file_name().unwrap().to_str().unwrap();

    match decision {
        Decision::Remove => {
            print_status("INFO", &format!("Removing worktree {}", worktree_name));

            // Get branch name
            let branch_output = Command::new("git")
                .args(&["branch", "--show-current"])
                .current_dir(worktree_path)
                .output()?;

            let branch = if branch_output.status.success() {
                String::from_utf8(branch_output.stdout)?.trim().to_string()
            } else {
                String::new()
            };

            // Abort any ongoing operations
            let _ = Command::new("git")
                .args(&["rebase", "--abort"])
                .current_dir(worktree_path)
                .output();

            // Remove worktree
            let remove_result = Command::new("git")
                .args(&["worktree", "remove", worktree_name, "--force"])
                .output();

            if remove_result.is_err() || !remove_result.unwrap().status.success() {
                print_status("WARNING", "Could not remove worktree, trying to delete directory");
                std::fs::remove_dir_all(worktree_path)?;
            }

            // Remove branch if it exists
            if !branch.is_empty() {
                let _ = Command::new("git")
                    .args(&["branch", "-D", &branch])
                    .output();
            }

            print_status("SUCCESS", &format!("Removed worktree {}", worktree_name));
        }
        Decision::Cleanup => {
            print_status("INFO", &format!("Cleaning up merged worktree {}", worktree_name));

            let branch_output = Command::new("git")
                .args(&["branch", "--show-current"])
                .current_dir(worktree_path)
                .output()?;

            let branch = if branch_output.status.success() {
                String::from_utf8(branch_output.stdout)?.trim().to_string()
            } else {
                String::new()
            };

            if !branch.is_empty() {
                let _ = Command::new("git")
                    .args(&["worktree", "remove", worktree_name, "--force"])
                    .output();

                let _ = Command::new("git")
                    .args(&["branch", "-d", &branch])
                    .output();

                print_status("SUCCESS", &format!("Cleaned up worktree {}", worktree_name));
            }
        }
        Decision::Keep => {
            print_status("INFO", &format!("Keeping worktree {}", worktree_name));
        }
    }

    Ok(())
}

// Function to create PR for ready worktrees
fn create_prs_for_ready() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("🚀 CREATING PRS FOR READY WORKTREES");
    println!("===================================");

    // Get list of worktrees
    let worktrees_output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    let worktrees_str = String::from_utf8(worktrees_output.stdout)?;
    let current_dir = std::env::current_dir()?.to_str().unwrap();

    for line in worktrees_str.lines() {
        if line.starts_with("worktree") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];
                if worktree_path != current_dir && Path::new(worktree_path).exists() {
                    // Check if ready for PR
                    let behind_output = Command::new("git")
                        .args(&["rev-list", "--count", "HEAD..origin/main"])
                        .current_dir(worktree_path)
                        .output();

                    let behind_count = if behind_output.is_ok() {
                        String::from_utf8(behind_output.unwrap().stdout)?.trim().parse::<i32>().unwrap_or(0)
                    } else {
                        0
                    };

                    let uncommitted_output = Command::new("git")
                        .args(&["status", "--porcelain"])
                        .current_dir(worktree_path)
                        .output()?;

                    let uncommitted = String::from_utf8(uncommitted_output.stdout)?.trim().to_string();

                    let branch_output = Command::new("git")
                        .args(&["branch", "--show-current"])
                        .current_dir(worktree_path)
                        .output()?;

                    let branch = if branch_output.status.success() {
                        String::from_utf8(branch_output.stdout)?.trim().to_string()
                    } else {
                        String::new()
                    };

                    if behind_count == 0 && uncommitted.is_empty() && !branch.is_empty() {
                        let remote_output = Command::new("git")
                            .args(&["ls-remote", "--heads", "origin", &branch])
                            .current_dir(worktree_path)
                            .output()?;

                        if remote_output.status.success() && !String::from_utf8(remote_output.stdout)?.trim().is_empty() {
                            print_status("SUCCESS", &format!("Creating PR for {}", worktree_path));

                            let url_output = Command::new("git")
                                .args(&["config", "--get", "remote.origin.url"])
                                .current_dir(worktree_path)
                                .output()?;

                            if url_output.status.success() {
                                let repo_url = String::from_utf8(url_output.stdout)?.trim().replace(".git", "");
                                if repo_url.contains("github.com") {
                                    let pr_url = format!("{}/compare/main...{}", repo_url, branch);
                                    print_status("INFO", &format!("Create PR at: {}", pr_url));
                                }
                            }
                        }
                    }
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
    println!("🧠 INTELLIGENT WORKTREE CLEANUP");
    println!("===============================");
    println!();

    println!("🔍 Analyzing all worktrees...");
    println!();

    // Get list of worktrees
    let worktrees_output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    let worktrees_str = String::from_utf8(worktrees_output.stdout)?;
    let current_dir = std::env::current_dir()?.to_str().unwrap();
    let mut decisions = Vec::new();

    for line in worktrees_str.lines() {
        if line.starts_with("worktree") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];
                if worktree_path != current_dir && Path::new(worktree_path).exists() {
                    let decision = analyze_worktree(worktree_path)?;
                    decisions.push((worktree_path.to_string(), decision));
                    println!();
                }
            }
        }
    }

    println!("📋 DECISIONS SUMMARY");
    println!("====================");
    for (worktree_path, decision) in &decisions {
        let worktree_name = Path::new(worktree_path).file_name().unwrap().to_str().unwrap();
        print_status("INFO", &format!("{}: {:?}", worktree_name, decision));
    }

    println!();
    println!("🚀 EXECUTING DECISIONS");
    println!("======================");

    for (worktree_path, decision) in &decisions {
        execute_decision(worktree_path, decision.clone())?;
    }

    // Create PRs for ready worktrees
    create_prs_for_ready()?;

    println!();
    println!("🎉 Intelligent cleanup completed!");
    println!();
    println!("📊 Final Status:");
    run_worktree_status_report()?;

    Ok(())
}
