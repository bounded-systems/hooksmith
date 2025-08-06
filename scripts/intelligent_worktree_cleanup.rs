#!/usr/bin/env rustx

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

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

#[derive(Debug, Clone)]
enum Decision {
    Remove,
    Cleanup,
    Keep,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Remove => write!(f, "REMOVE"),
            Decision::Cleanup => write!(f, "CLEANUP"),
            Decision::Keep => write!(f, "KEEP"),
        }
    }
}

// Function to analyze worktree and make decision
fn analyze_worktree(worktree_path: &str) -> Result<Decision, Box<dyn std::error::Error>> {
    let worktree_name = Path::new(worktree_path).file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    println!("=== ANALYZING: {} ===", worktree_name);

    if !Path::new(worktree_path).exists() {
        print_status("ERROR", &format!("Worktree {} does not exist", worktree_name));
        return Ok(Decision::Remove);
    }

    // Get branch name
    let branch_output = Command::new("git")
        .args(&["-C", worktree_path, "branch", "--show-current"])
        .output()?;

    let branch = if branch_output.status.success() {
        String::from_utf8(branch_output.stdout)?.trim().to_string()
    } else {
        String::new()
    };

    print_status("INFO", &format!("Branch: {}", branch));

    // Check commit history
    let commit_count_output = Command::new("git")
        .args(&["-C", worktree_path, "log", "--oneline", "--since=1 week ago"])
        .output()?;

    let commit_count = if commit_count_output.status.success() {
        String::from_utf8(commit_count_output.stdout)?.lines().count()
    } else {
        0
    };

    let last_commit_output = Command::new("git")
        .args(&["-C", worktree_path, "log", "--oneline", "-1"])
        .output()?;

    let last_commit = if last_commit_output.status.success() {
        String::from_utf8(last_commit_output.stdout)?.trim().to_string()
    } else {
        "No commits".to_string()
    };

    print_status("INFO", &format!("Recent commits: {}", commit_count));
    print_status("INFO", &format!("Last commit: {}", last_commit));

    // Check for conflicts
    let conflicts_output = Command::new("git")
        .args(&["-C", worktree_path, "diff", "--name-only", "--diff-filter=U"])
        .stderr(Stdio::null())
        .output()?;

    let has_conflicts = conflicts_output.status.success() && !String::from_utf8(conflicts_output.stdout)?.trim().is_empty();

    let rebase_status_output = Command::new("git")
        .args(&["-C", worktree_path, "status"])
        .output()?;

    let has_rebase = if rebase_status_output.status.success() {
        String::from_utf8(rebase_status_output.stdout)?.contains("rebase")
    } else {
        false
    };

    if has_conflicts || has_rebase {
        print_status("WARNING", "Worktree has conflicts or is in rebase state");

        // Check if this worktree is from old development
        if worktree_name.contains("202508") {
            print_status("DECISION", "This appears to be an old worktree from 202508");
            print_status("DECISION", "Recommendation: REMOVE (likely obsolete)");
            return Ok(Decision::Remove);
        }
    }

    // Check if branch is behind main
    let behind_count_output = Command::new("git")
        .args(&["-C", worktree_path, "rev-list", "--count", "HEAD..origin/main"])
        .stderr(Stdio::null())
        .output()?;

    let behind_count = if behind_count_output.status.success() {
        String::from_utf8(behind_count_output.stdout)?.trim().parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    if behind_count > 5 {
        print_status("WARNING", &format!("Branch is {} commits behind main", behind_count));
        print_status("DECISION", "Recommendation: REMOVE (too far behind)");
        return Ok(Decision::Remove);
    }

    // Check if branch exists on origin
    let remote_check = Command::new("git")
        .args(&["-C", worktree_path, "ls-remote", "--heads", "origin", &branch])
        .output()?;

    if remote_check.status.success() {
        let remote_output = String::from_utf8(remote_check.stdout)?;
        if remote_output.contains(&branch) {
            print_status("INFO", "Branch exists on origin");

            // Check if merged
            let merged_check = Command::new("git")
                .args(&["-C", worktree_path, "branch", "--merged", "origin/main"])
                .output()?;

            if merged_check.status.success() {
                let merged_output = String::from_utf8(merged_check.stdout)?;
                if merged_output.contains(&branch) {
                    print_status("SUCCESS", "Branch is merged");
                    print_status("DECISION", "Recommendation: CLEANUP (merged)");
                    return Ok(Decision::Cleanup);
                } else {
                    print_status("INFO", "Branch not merged");
                    print_status("DECISION", "Recommendation: KEEP (active development)");
                    return Ok(Decision::Keep);
                }
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
    } else {
        print_status("WARNING", "Branch does not exist on origin");
        print_status("DECISION", "Recommendation: REMOVE (no remote branch)");
        return Ok(Decision::Remove);
    }
}

// Function to execute decision
fn execute_decision(worktree_path: &str, decision: &Decision) -> Result<(), Box<dyn std::error::Error>> {
    let worktree_name = Path::new(worktree_path).file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    match decision {
        Decision::Remove => {
            print_status("INFO", &format!("Removing worktree {}", worktree_name));

            // Get branch name
            let branch_output = Command::new("git")
                .args(&["-C", worktree_path, "branch", "--show-current"])
                .output()?;

            let branch = if branch_output.status.success() {
                String::from_utf8(branch_output.stdout)?.trim().to_string()
            } else {
                String::new()
            };

            // Abort any ongoing operations
            let _ = Command::new("git")
                .args(&["-C", worktree_path, "rebase", "--abort"])
                .stderr(Stdio::null())
                .status();

            // Remove worktree
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
                let _ = Command::new("git")
                    .args(&["branch", "-D", &branch])
                    .stderr(Stdio::null())
                    .status();
            }

            print_status("SUCCESS", &format!("Removed worktree {}", worktree_name));
        }
        Decision::Cleanup => {
            print_status("INFO", &format!("Cleaning up merged worktree {}", worktree_name));

            let branch_output = Command::new("git")
                .args(&["-C", worktree_path, "branch", "--show-current"])
                .output()?;

            let branch = if branch_output.status.success() {
                String::from_utf8(branch_output.stdout)?.trim().to_string()
            } else {
                String::new()
            };

            if !branch.is_empty() {
                let _ = Command::new("git")
                    .args(&["worktree", "remove", worktree_name, "--force"])
                    .status();

                let _ = Command::new("git")
                    .args(&["branch", "-d", &branch])
                    .stderr(Stdio::null())
                    .status();

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
    let current_dir = env::current_dir()?.to_string_lossy().to_string();

    for line in worktrees_str.lines() {
        if line.starts_with("worktree") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let worktree_path = parts[1];
                if worktree_path != current_dir && Path::new(worktree_path).exists() {
                    // Check if ready for PR
                    let behind_count_output = Command::new("git")
                        .args(&["-C", worktree_path, "rev-list", "--count", "HEAD..origin/main"])
                        .stderr(Stdio::null())
                        .output()?;

                    let behind_count = if behind_count_output.status.success() {
                        String::from_utf8(behind_count_output.stdout)?.trim().parse::<i32>().unwrap_or(0)
                    } else {
                        0
                    };

                    let uncommitted_output = Command::new("git")
                        .args(&["-C", worktree_path, "status", "--porcelain"])
                        .output()?;

                    let has_uncommitted = if uncommitted_output.status.success() {
                        !String::from_utf8(uncommitted_output.stdout)?.trim().is_empty()
                    } else {
                        true
                    };

                    let branch_output = Command::new("git")
                        .args(&["-C", worktree_path, "branch", "--show-current"])
                        .output()?;

                    let branch = if branch_output.status.success() {
                        String::from_utf8(branch_output.stdout)?.trim().to_string()
                    } else {
                        String::new()
                    };

                    if behind_count == 0 && !has_uncommitted && !branch.is_empty() {
                        let remote_check = Command::new("git")
                            .args(&["-C", worktree_path, "ls-remote", "--heads", "origin", &branch])
                            .output()?;

                        if remote_check.status.success() {
                            let remote_output = String::from_utf8(remote_check.stdout)?;
                            if remote_output.contains(&branch) {
                                print_status("SUCCESS", &format!("Creating PR for {}", worktree_path));

                                let url_output = Command::new("git")
                                    .args(&["-C", worktree_path, "config", "--get", "remote.origin.url"])
                                    .output()?;

                                if url_output.status.success() {
                                    let repo_url = String::from_utf8(url_output.stdout)?
                                        .trim()
                                        .replace(".git", "");

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
    let current_dir = env::current_dir()?.to_string_lossy().to_string();
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
        let worktree_name = Path::new(worktree_path).file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        print_status("INFO", &format!("{}: {}", worktree_name, decision));
    }

    println!();
    println!("🚀 EXECUTING DECISIONS");
    println!("======================");

    for (worktree_path, decision) in &decisions {
        execute_decision(worktree_path, decision)?;
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
