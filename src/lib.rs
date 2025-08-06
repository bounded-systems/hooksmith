#![deny(missing_docs)]

//! Hooksmith CLI Library
//!
//! This library provides core functionality for building Rust binaries into Lefthook hooks with WASM components.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// ANSI color codes for terminal output
pub const RED: &str = "\x1b[0;31m";
/// ANSI color codes for terminal output
pub const GREEN: &str = "\x1b[0;32m";
/// ANSI color codes for terminal output
pub const YELLOW: &str = "\x1b[1;33m";
/// ANSI color codes for terminal output
pub const BLUE: &str = "\x1b[0;34m";
/// ANSI color codes for terminal output
pub const PURPLE: &str = "\x1b[0;35m";
/// ANSI color codes for terminal output
pub const CYAN: &str = "\x1b[0;36m";
/// ANSI color codes for terminal output
pub const NC: &str = "\x1b[0m"; // No Color

/// Represents the status of a git worktree
#[derive(Debug, Clone)]
pub struct WorktreeStatus {
    /// The current branch name
    pub current_branch: String,
    /// Whether the worktree has uncommitted changes
    pub is_clean: bool,
    /// Whether the worktree is currently in a rebase state
    pub is_rebasing: bool,
    /// Whether the branch exists on the remote
    pub remote_exists: bool,
    /// Whether the branch has been merged into main
    pub is_merged: bool,
    /// Number of commits ahead of main
    pub ahead_behind: i32,
    /// Number of commits behind main
    pub behind_ahead: i32,
}

/// Represents the overall state of a worktree
#[derive(Debug)]
pub enum WorktreeState {
    /// Worktree has been merged and can be cleaned up
    Merged,
    /// Worktree has conflicts that need resolution
    Conflicted,
    /// Worktree has uncommitted changes
    Developing,
    /// Worktree is ready for PR creation
    Ready,
    /// Worktree is behind main and needs updating
    Outdated,
    /// Worktree state is unknown
    Unknown,
}

/// Represents the cleanup decision for a worktree
#[derive(Debug, Clone)]
pub enum CleanupDecision {
    /// Remove the worktree entirely
    Remove,
    /// Clean up the worktree (delete branch and remove worktree)
    Cleanup,
    /// Keep the worktree as is
    Keep,
}

/// Log an informational message with blue color
pub fn log_info(message: &str) {
    println!("{}[INFO]{} {}", BLUE, NC, message);
}

/// Log a success message with green color
pub fn log_success(message: &str) {
    println!("{}[SUCCESS]{} {}", GREEN, NC, message);
}

/// Log a warning message with yellow color
pub fn log_warning(message: &str) {
    println!("{}[WARNING]{} {}", YELLOW, NC, message);
}

/// Log an error message with red color
pub fn log_error(message: &str) {
    println!("{}[ERROR]{} {}", RED, NC, message);
}

/// Log a header message with purple color
pub fn log_header(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

/// Print a status message with appropriate emoji and color
pub fn print_status(state: &str, message: &str) {
    match state {
        "ERROR" => println!("{}❌ {}{}", RED, message, NC),
        "SUCCESS" => println!("{}✅ {}{}", GREEN, message, NC),
        "WARNING" => println!("{}⚠️  {}{}", YELLOW, message, NC),
        "INFO" => println!("{}ℹ️  {}{}", BLUE, message, NC),
        "DECISION" => println!("{}🤔 {}{}", PURPLE, message, NC),
        _ => println!("📝 {}", message),
    }
}

/// Run a git command and return the output
pub fn run_git_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Run a git command in a specific directory and return the output
pub fn run_git_command_in_dir(args: &[&str], worktree_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git command in {}: {}", worktree_path, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Get the status of a specific worktree
pub fn get_worktree_status(worktree_path: &str) -> Result<WorktreeStatus, String> {
    // Get current branch
    let current_branch = run_git_command_in_dir(&["branch", "--show-current"], worktree_path)?;

    // Get status
    let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path)?;
    let is_clean = status.is_empty();

    // Check if rebasing
    let git_status = run_git_command_in_dir(&["status"], worktree_path)?;
    let is_rebasing = git_status.contains("rebase");

    // Check if branch exists on origin
    let remote_check = run_git_command_in_dir(
        &["ls-remote", "--heads", "origin", &current_branch],
        worktree_path,
    );
    let remote_exists = remote_check.is_ok() && !remote_check.unwrap().is_empty();

    // Check if merged into main
    let merged_branches = run_git_command_in_dir(&["branch", "--merged", "main"], worktree_path)?;
    let is_merged = merged_branches
        .lines()
        .any(|line| line.trim() == format!("* {}", current_branch));

    // Get commit count ahead/behind main
    let ahead_behind =
        run_git_command_in_dir(&["rev-list", "--count", "main..HEAD"], worktree_path)
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .unwrap_or(0);

    let behind_ahead =
        run_git_command_in_dir(&["rev-list", "--count", "HEAD..main"], worktree_path)
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .unwrap_or(0);

    Ok(WorktreeStatus {
        current_branch,
        is_clean,
        is_rebasing,
        remote_exists,
        is_merged,
        ahead_behind,
        behind_ahead,
    })
}

/// Determine the overall state of a worktree based on its status
pub fn determine_state(status: &WorktreeStatus) -> WorktreeState {
    if status.is_merged {
        WorktreeState::Merged
    } else if status.is_rebasing {
        WorktreeState::Conflicted
    } else if !status.is_clean {
        WorktreeState::Developing
    } else if status.ahead_behind > 0 && status.behind_ahead == 0 {
        WorktreeState::Ready
    } else if status.behind_ahead > 0 {
        WorktreeState::Outdated
    } else {
        WorktreeState::Unknown
    }
}

/// Get a list of all worktree paths
pub fn get_worktrees() -> Result<Vec<String>, String> {
    let output = run_git_command(&["worktree", "list", "--porcelain"])?;

    let worktrees: Vec<String> = output
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .map(|line| line.splitn(2, ' ').nth(1).unwrap_or("").to_string())
        .filter(|path| !path.is_empty())
        .collect();

    Ok(worktrees)
}

/// Generate a PR URL for a given branch name
pub fn generate_pr_url(branch_name: &str) -> String {
    let repo_url = run_git_command(&["config", "--get", "remote.origin.url"])
        .unwrap_or_else(|_| "".to_string())
        .replace(".git", "");

    if repo_url.contains("github.com") {
        format!("{}/compare/main...{}", repo_url, branch_name)
    } else {
        "Unknown repository URL".to_string()
    }
}

/// Check if a worktree is ready for PR creation
pub fn is_ready_for_pr(worktree_path: &str) -> Result<bool, String> {
    // Check if clean
    let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path)?;
    if !status.is_empty() {
        return Ok(false);
    }

    // Check if up to date with main
    let behind_count =
        run_git_command_in_dir(&["rev-list", "--count", "HEAD..main"], worktree_path)
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .unwrap_or(0);
    if behind_count > 0 {
        return Ok(false);
    }

    // Check if has commits ahead of main
    let ahead_count = run_git_command_in_dir(&["rev-list", "--count", "main..HEAD"], worktree_path)
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    if ahead_count == 0 {
        return Ok(false);
    }

    Ok(true)
}

/// Push a branch to the remote repository
pub fn push_branch(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Pushing branch {}", branch_name));

    let output = Command::new("git")
        .args(&["push", "origin", branch_name])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to push branch: {}", e))?;

    if output.status.success() {
        log_success("Branch pushed successfully");
        Ok(true)
    } else {
        log_warning("Push failed - branch may already be up to date");
        Ok(false)
    }
}

/// Create a pull request using GitHub CLI
pub fn create_pr_with_gh(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!(
        "Creating PR for branch {} using GitHub CLI",
        branch_name
    ));

    // Get commit message for PR title
    let commit_msg = run_git_command_in_dir(&["log", "--oneline", "-1"], worktree_path)?;
    let pr_title = commit_msg.splitn(2, ' ').nth(1).unwrap_or("").to_string();

    // Get PR body from commit messages
    let pr_body = run_git_command_in_dir(&["log", "--oneline", "main..HEAD"], worktree_path)
        .unwrap_or_else(|_| "".to_string());
    let pr_body = pr_body
        .lines()
        .take(5)
        .map(|line| format!("- {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let output = Command::new("gh")
        .args(&[
            "pr",
            "create",
            "--title",
            &pr_title,
            "--body",
            &pr_body,
            "--base",
            "main",
            "--head",
            branch_name,
        ])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to create PR with GitHub CLI: {}", e))?;

    if output.status.success() {
        log_success("PR created successfully");
        Ok(true)
    } else {
        log_warning("Failed to create PR with GitHub CLI");
        Ok(false)
    }
}

/// Check if a worktree is currently in a rebase state
pub fn is_rebasing(worktree_path: &str) -> Result<bool, String> {
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .output();

    match status_output {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout);

            // Check for conflict markers
            if status.lines().any(|line| line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")) {
                return Ok(true);
            }

            // Check for rebase in progress
            let git_status_output = Command::new("git")
                .args(&["status"])
                .current_dir(worktree_path)
                .output();

            match git_status_output {
                Ok(git_output) => {
                    let git_status = String::from_utf8_lossy(&git_output.stdout);
                    Ok(git_status.contains("rebase in progress"))
                }
                Err(_) => Ok(false)
            }
        }
        Err(_) => Ok(false)
    }
}

/// Stash uncommitted changes in a worktree
pub fn stash_changes(worktree_path: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .args(&["stash", "push", "-m", "Auto-stashed by conflict resolver"])
        .current_dir(worktree_path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                log_success("Stashed uncommitted changes");
                Ok(true)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                log_warning(&format!("Failed to stash changes: {}", error));
                Ok(false)
            }
        }
        Err(e) => {
            log_error(&format!("Failed to execute stash command: {}", e));
            Ok(false)
        }
    }
}

/// Clean up a merged worktree
pub fn cleanup_merged_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    // Check if branch is merged into main
    let merge_check = Command::new("git")
        .args(&["branch", "--merged", "main"])
        .current_dir(worktree_path)
        .output();

    match merge_check {
        Ok(output) => {
            let merged_branches = String::from_utf8_lossy(&output.stdout);
            if merged_branches.lines().any(|line| line.trim() == branch_name) {
                log_info(&format!("Branch {} is merged, cleaning up worktree", branch_name));

                // Remove the worktree
                let remove_output = Command::new("git")
                    .args(&["worktree", "remove", worktree_path])
                    .output();

                match remove_output {
                    Ok(remove_output) => {
                        if remove_output.status.success() {
                            log_success(&format!("Removed merged worktree: {}", worktree_path));
                            Ok(true)
                        } else {
                            let error = String::from_utf8_lossy(&remove_output.stderr);
                            log_warning(&format!("Failed to remove worktree: {}", error));
                            Ok(false)
                        }
                    }
                    Err(e) => {
                        log_error(&format!("Failed to execute worktree remove: {}", e));
                        Ok(false)
                    }
                }
            } else {
                log_info(&format!("Branch {} is not merged, keeping worktree", branch_name));
                Ok(false)
            }
        }
        Err(e) => {
            log_error(&format!("Failed to check merged branches: {}", e));
            Ok(false)
        }
    }
}

/// Command implementations for the CLI
pub mod commands;
/// Core modules for CLI functionality
pub mod modules;
/// Orchestrator for WASM component management
pub mod orchestrator;

// Re-export main types
pub use orchestrator::{
    BuildConfig, BuildResult, CommandResult, HooksmithOrchestrator, LefthookConfig, LefthookResult,
    ValidationConfig, ValidationResult, WorktreeOperation, WorktreeResult,
};

/// Result type for CLI operations
pub type CliResult<T> = anyhow::Result<T>;

/// Configuration for CLI operations
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Directory containing hook scripts
    pub hooks_dir: String,
    /// Output directory for built binaries
    pub output_dir: String,
    /// Whether to perform a dry run (no actual changes)
    pub dry_run: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            hooks_dir: "hooks".to_string(),
            output_dir: "target/hooks".to_string(),
            dry_run: false,
        }
    }
}
