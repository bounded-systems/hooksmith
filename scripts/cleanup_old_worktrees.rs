#!/usr/bin/env rust-script
//! Clean up old worktrees that are no longer needed
//! Direct approach to remove obsolete worktrees

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

/// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

/// List of worktrees to remove (old conflicted ones)
const OLD_WORKTREES: &[&str] = &[
    "worktree-fix-main-cleanup-20250804-211403",
    "worktree-fix-workspace-config",
    "worktree-fix-workspace-dependencies",
    "worktree-fix-xtask-cleanup",
];

fn print_status(state: &str, message: &str) {
    let color = match state {
        "ERROR" => RED,
        "SUCCESS" => GREEN,
        "WARNING" => YELLOW,
        "INFO" => BLUE,
        _ => "",
    };
    println!("{}{}{}", color, message, NC);
}

fn run_command(cmd: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

fn run_command_silent(cmd: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

fn remove_worktree(worktree_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(worktree_name).exists() {
        print_status("WARNING", &format!("Worktree {} does not exist", worktree_name));
        return Ok(());
    }
    
    print_status("INFO", &format!("Removing worktree: {}", worktree_name));
    
    // Abort any ongoing operations first
    let worktree_path = Path::new(worktree_name);
    if worktree_path.exists() {
        // Check if there's a rebase in progress
        let status_cmd = Command::new("git")
            .args(&["status"])
            .current_dir(worktree_path);
        
        if let Ok(status_output) = run_command(&mut Command::new("git").args(&["status"]).current_dir(worktree_path)) {
            if status_output.contains("rebase") {
                print_status("INFO", &format!("Aborting rebase in {}", worktree_name));
                let _ = run_command_silent(&mut Command::new("git").args(&["rebase", "--abort"]).current_dir(worktree_path));
            }
        }
        
        // Get branch name before removal
        let branch = run_command(&mut Command::new("git").args(&["branch", "--show-current"]).current_dir(worktree_path))
            .ok()
            .map(|s| s.trim().to_string());
        
        // Remove worktree
        print_status("INFO", "Removing worktree directory");
        if let Err(_) = run_command_silent(&mut Command::new("git").args(&["worktree", "remove", worktree_name, "--force"])) {
            print_status("WARNING", "Could not remove worktree, trying to delete directory");
            std::fs::remove_dir_all(worktree_name)?;
        }
        
        // Remove branch if it exists
        if let Some(branch_name) = branch {
            if !branch_name.is_empty() {
                print_status("INFO", &format!("Removing branch: {}", branch_name));
                let _ = run_command_silent(&mut Command::new("git").args(&["branch", "-D", &branch_name]));
            }
        }
        
        print_status("SUCCESS", &format!("Removed worktree {}", worktree_name));
        println!();
    }
    
    Ok(())
}

fn create_pr_for_ready() -> Result<(), Box<dyn std::error::Error>> {
    let worktree_name = "worktree-management-improvements";
    
    if !Path::new(worktree_name).exists() {
        print_status("WARNING", &format!("Ready worktree {} does not exist", worktree_name));
        return Ok(());
    }
    
    print_status("INFO", &format!("Creating PR for ready worktree: {}", worktree_name));
    
    let worktree_path = Path::new(worktree_name);
    let branch = run_command(&mut Command::new("git").args(&["branch", "--show-current"]).current_dir(worktree_path))
        .ok()
        .map(|s| s.trim().to_string());
    
    if let Some(branch_name) = branch {
        if !branch_name.is_empty() {
            // Check if branch exists on origin
            let ls_remote_output = run_command(&mut Command::new("git").args(&["ls-remote", "--heads", "origin", &branch_name]))?;
            
            if ls_remote_output.contains(&branch_name) {
                let repo_url = run_command(&mut Command::new("git").args(&["config", "--get", "remote.origin.url"]))?;
                let repo_url = repo_url.trim().replace(".git", "");
                
                if repo_url.contains("github.com") {
                    let pr_url = format!("{}/compare/main...{}", repo_url, branch_name);
                    print_status("SUCCESS", &format!("Create PR at: {}", pr_url));
                }
            }
        }
    }
    
    Ok(())
}

fn run_worktree_status_report() -> Result<(), Box<dyn std::error::Error>> {
    let status_script = "./scripts/worktree-status-report.sh";
    if Path::new(status_script).exists() {
        let _ = Command::new(status_script).status();
    }
    Ok(())
}

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
