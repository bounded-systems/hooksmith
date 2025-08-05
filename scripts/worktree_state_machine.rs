#!/usr/bin/env -S rustc --edition=2021 -o /tmp/worktree-state-machine && /tmp/worktree-state-machine

use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let diagram = args.iter().any(|arg| arg == "--diagram");
    let process = args.iter().any(|arg| arg == "--process");
    let status = args.iter().any(|arg| arg == "--status");
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    
    if diagram {
        print_diagram();
    } else if process {
        process_all_worktrees(verbose, dry_run)?;
    } else if status {
        run_status_report()?;
    } else {
        println!("🔄 Worktree State Machine");
        println!("=========================");
        println!();
        println!("Usage:");
        println!("  --diagram     Show state machine diagram");
        println!("  --process     Process all worktrees through state machine");
        println!("  --status      Show current worktree status");
        println!("  --verbose     Show detailed output");
        println!("  --dry-run     Show what would be done without making changes");
    }
    
    Ok(())
}

fn print_diagram() {
    println!("�� WORKTREE STATE MACHINE DIAGRAM");
    println!("================================");
    println!();
    println!("CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED");
    println!("    ↓         ↓");
    println!("CONFLICTED → RESOLVING");
    println!();
    println!("State Descriptions:");
    println!("  CREATED: Worktree created");
    println!("  DEVELOPING: Worktree has uncommitted changes");
    println!("  CONFLICTED: Worktree has rebase conflicts");
    println!("  RESOLVING: Resolving conflicts");
    println!("  RESOLVED: Conflicts resolved");
    println!("  READY: Worktree ready for PR");
    println!("  PR_CREATED: PR created");
    println!("  MERGED: PR merged");
    println!("  CLEANUP: Cleaning up worktree");
    println!("  REMOVED: Worktree removed");
}

fn process_all_worktrees(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 PROCESSING ALL WORKTREES");
    println!();
    
    // Get list of worktrees
    let worktrees_output = run_git_command(Path::new("."), &["worktree", "list", "--porcelain"])?;
    let worktrees: Vec<String> = worktrees_output
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .collect();
    
    if worktrees.is_empty() {
        println!("ℹ️  No worktrees found");
        return Ok(());
    }
    
    let mut processed_count = 0;
    let mut success_count = 0;
    
    for worktree_path in worktrees {
        // Get branch name from worktree path
        let branch_name = std::path::Path::new(&worktree_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Skip main worktree
        if branch_name == "hooksmith" {
            continue;
        }
        
        processed_count += 1;
        
        if verbose {
            println!("📁 Processing worktree: {} (branch: {})", worktree_path, branch_name);
        }
        
        // Get current state
        let current_state = get_worktree_state(&worktree_path, &branch_name)?;
        if verbose {
            println!("   Current state: {}", current_state);
        }
        
        // Simple state transition logic
        let next_state = match current_state.as_str() {
            "CREATED" => Some("DEVELOPING"),
            "DEVELOPING" => {
                // Check if clean
                let status = run_git_command(Path::new(&worktree_path), &["status", "--porcelain"])?;
                if status.trim().is_empty() {
                    Some("RESOLVING")
                } else {
                    None
                }
            }
            "CONFLICTED" => Some("RESOLVING"),
            "RESOLVING" => Some("READY"),
            "READY" => Some("PR_CREATED"),
            "PR_CREATED" => Some("MERGED"),
            "MERGED" => Some("CLEANUP"),
            "CLEANUP" => Some("REMOVED"),
            _ => None,
        };
        
        if let Some(next_state) = next_state {
            if verbose {
                println!("   Transitioning: {} → {}", current_state, next_state);
            }
            
            if transition_state(&worktree_path, &branch_name, &current_state, next_state, dry_run)? {
                success_count += 1;
                if verbose {
                    println!("   ✅ Successfully transitioned to {}", next_state);
                }
            } else {
                if verbose {
                    println!("   ⚠️  Failed to transition to {}", next_state);
                }
            }
        } else {
            if verbose {
                println!("   ℹ️  No transition needed");
            }
        }
        
        println!("---");
    }
    
    println!("✅ Processed {} worktree(s), {} successful", processed_count, success_count);
    Ok(())
}

fn get_worktree_state(worktree_path: &str, branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Get current branch
    let current_branch = run_git_command(Path::new(worktree_path), &["branch", "--show-current"])?;
    let current_branch = current_branch.trim();
    
    // Get status
    let status = run_git_command(Path::new(worktree_path), &["status", "--porcelain"])?;
    let is_clean = status.trim().is_empty();
    
    // Check if rebasing
    let git_status = run_git_command(Path::new(worktree_path), &["status"])?;
    let is_rebasing = git_status.contains("rebase");
    
    // Check if merged into main
    let merged_branches = run_git_command(Path::new(worktree_path), &["branch", "--merged", "main"])?;
    let is_merged = merged_branches.contains(current_branch);
    
    // Get commit count ahead/behind main
    let ahead_count = run_git_command(Path::new(worktree_path), &["rev-list", "--count", "main..HEAD"])
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    
    let behind_count = run_git_command(Path::new(worktree_path), &["rev-list", "--count", "HEAD..main"])
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    
    // Determine state
    let state = if is_merged {
        "MERGED"
    } else if is_rebasing {
        "CONFLICTED"
    } else if !is_clean {
        "DEVELOPING"
    } else if ahead_count > 0 && behind_count == 0 {
        "READY"
    } else if behind_count > 0 {
        "RESOLVING"
    } else {
        "CREATED"
    };
    
    Ok(state.to_string())
}

fn transition_state(worktree_path: &str, branch_name: &str, current_state: &str, target_state: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    if dry_run {
        println!("   DRY RUN: Would transition {}: {} → {}", branch_name, current_state, target_state);
        return Ok(true);
    }
    
    match target_state {
        "RESOLVING" => {
            let result = run_git_command(Path::new(worktree_path), &["rebase", "main"]);
            match result {
                Ok(_) => {
                    println!("   ✅ Rebase successful");
                    Ok(true)
                }
                Err(_) => {
                    println!("   ⚠️  Rebase failed - aborting");
                    let _ = run_git_command(Path::new(worktree_path), &["rebase", "--abort"]);
                    Ok(false)
                }
            }
        }
        "READY" => {
            let result = run_git_command(Path::new(worktree_path), &["push", "origin", branch_name]);
            match result {
                Ok(_) => {
                    println!("   ✅ Branch pushed successfully");
                    Ok(true)
                }
                Err(_) => {
                    println!("   ⚠️  Push failed");
                    Ok(false)
                }
            }
        }
        "PR_CREATED" => {
            let pr_url = generate_pr_url(branch_name)?;
            println!("   ℹ️  PR URL: {}", pr_url);
            Ok(true)
        }
        "CLEANUP" => {
            // Remove worktree
            let _ = run_git_command(Path::new("."), &["worktree", "remove", worktree_path, "--force"]);
            
            // Delete remote branch
            let _ = run_git_command(Path::new("."), &["push", "origin", "--delete", branch_name]);
            
            println!("   ✅ Worktree cleaned up");
            Ok(true)
        }
        _ => {
            println!("   ⚠️  Unknown target state: {}", target_state);
            Ok(false)
        }
    }
}

fn generate_pr_url(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let repo_url = run_git_command(Path::new("."), &["config", "--get", "remote.origin.url"])?;
    let repo_url = repo_url.trim().replace(".git", "");
    
    if repo_url.contains("github.com") {
        Ok(format!("{}/compare/main...{}", repo_url, branch_name))
    } else {
        Ok("Unknown repository URL".to_string())
    }
}

fn run_git_command(worktree_path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Git command failed: {}", stderr).into())
    }
}

fn run_status_report() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("./scripts/worktree-status-report.sh")
        .output()?;
    
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}
