#!/usr/bin/env rust-script
//! Migrate Worktrees to .wt Directory
//! This script moves existing worktrees to the .wt directory structure

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;

// Colors for output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

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

fn is_worktree(dir: &str) -> bool {
    let path = Path::new(dir);
    path.is_dir() && path.join(".git").exists()
}

fn get_worktree_branch(worktree_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try to get branch from git worktree list
    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = worktree_list.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("worktree ") && line.contains(worktree_path) {
            if i + 1 < lines.len() {
                let next_line = lines[i + 1];
                if next_line.starts_with("branch ") {
                    let branch = next_line.replace("branch ", "").replace("refs/heads/", "");
                    return Ok(branch);
                }
            }
        }
    }
    
    // Fallback: try to get branch from .git file content
    let git_file = Path::new(worktree_path).join(".git");
    if git_file.exists() {
        let content = fs::read_to_string(&git_file)?;
        if let Some(refs_line) = content.lines().find(|line| line.contains("refs/heads/")) {
            let branch = refs_line.split_whitespace()
                .find(|part| part.contains("refs/heads/"))
                .unwrap_or("")
                .replace("refs/heads/", "");
            if !branch.is_empty() {
                return Ok(branch);
            }
        }
    }
    
    Err("Could not determine branch name".into())
}

fn move_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Create .wt directory if it doesn't exist
    fs::create_dir_all(".wt")?;
    
    // Determine new path in .wt directory
    let new_name = branch_name.replace("/", "-");
    let new_path = format!(".wt/{}", new_name);
    
    log_info(&format!("Moving worktree from {} to {}", worktree_path, new_path));
    
    // Check if destination already exists
    if Path::new(&new_path).exists() {
        log_warning(&format!("Destination {} already exists. Skipping.", new_path));
        return Ok(false);
    }
    
    // Move the worktree
    fs::rename(worktree_path, &new_path)?;
    log_success(&format!("Successfully moved worktree to {}", new_path));
    
    // Update the worktree in git
    let _ = Command::new("git")
        .args(&["worktree", "remove", worktree_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    // Add the new worktree location
    let add_status = Command::new("git")
        .args(&["worktree", "add", &new_path, branch_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    match add_status {
        Ok(status) if status.success() => {
            log_success("Added new worktree reference");
        }
        _ => {
            log_warning("Could not add new worktree reference, but files are moved");
        }
    }
    
    Ok(true)
}

fn handle_external_worktree(worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_warning(&format!("Found external worktree: {}", worktree_path));
    log_info("This worktree is outside the repository and cannot be moved automatically.");
    log_info("You may want to manually move it to .wt/{}", branch_name);
    
    // Create a placeholder in .wt directory
    let new_name = branch_name.replace("/", "-");
    let new_path = format!(".wt/{}", new_name);
    
    if !Path::new(&new_path).exists() {
        log_info(&format!("Creating placeholder directory: {}", new_path));
        fs::create_dir_all(&new_path)?;
        
        let placeholder_content = format!(
            "# Placeholder for external worktree: {}\n# Original location: {}\n# Please move this worktree manually if needed\n",
            branch_name, worktree_path
        );
        fs::write(format!("{}/README.md", new_path), placeholder_content)?;
    }
    
    Ok(())
}

fn migrate_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Starting worktree migration to .wt directory");
    
    // Get all worktrees
    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    let current_dir = env::current_dir()?.to_string_lossy().to_string();
    
    let worktrees: Vec<String> = worktree_list
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
        .filter(|path| !path.is_empty())
        .collect();
    
    let mut migrated_count = 0;
    let mut skipped_count = 0;
    
    for worktree_path in worktrees {
        // Skip the main worktree
        if worktree_path == current_dir {
            log_info(&format!("Skipping main worktree: {}", worktree_path));
            continue;
        }
        
        // Get branch name
        let branch_name = match get_worktree_branch(&worktree_path) {
            Ok(name) => name,
            Err(_) => {
                log_warning(&format!("Could not determine branch name for worktree: {}", worktree_path));
                skipped_count += 1;
                continue;
            }
        };
        
        // Check if worktree is in the repository
        let repo_root = Command::new("git")
            .args(&["rev-parse", "--show-toplevel"])
            .stdout(Stdio::piped())
            .output()?;
        
        let repo_root = String::from_utf8(repo_root.stdout)?.trim().to_string();
        
        if worktree_path.starts_with(&repo_root) {
            // Worktree is in repository, move it
            if move_worktree(&worktree_path, &branch_name)? {
                migrated_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            // Worktree is external
            handle_external_worktree(&worktree_path, &branch_name)?;
            skipped_count += 1;
        }
    }
    
    log_success("Migration complete!");
    log_info(&format!("Migrated: {} worktrees", migrated_count));
    log_info(&format!("Skipped: {} worktrees", skipped_count));
    
    Ok(())
}

fn cleanup_old_directories() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Checking for old worktree directories...");
    
    let old_dirs = vec![
        "worktree-cleanup-remote-branches",
        "worktree-feat-systems-diagrams",
        "worktree-lifecycle",
    ];
    
    for dir in old_dirs {
        if Path::new(dir).exists() && is_worktree(dir) {
            log_warning(&format!("Found old worktree directory: {}", dir));
            log_info("This should have been moved by the migration script.");
            log_info("You can safely remove it if it's no longer needed.");
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Worktree Migration Script");
    log_info("This script will move existing worktrees to the .wt directory");
    println!();
    
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if git_check.is_err() || !git_check.unwrap().success() {
        log_error("Not in a git repository");
        std::process::exit(1);
    }
    
    // Create .wt directory if it doesn't exist
    fs::create_dir_all(".wt")?;
    
    // Run migration
    migrate_worktrees()?;
    
    println!();
    cleanup_old_directories()?;
    
    log_success("Migration script completed!");
    log_info("All worktrees should now be in the .wt directory");
    
    Ok(())
} 
