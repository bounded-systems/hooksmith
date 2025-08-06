use std::process::Command;
use std::env;
use hooksmith::{log_info, log_warning, log_error, log_success, run_git_command};
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("🔍 Checking if main is ahead of origin/main...");

    // Check if we're on main branch
    let current_branch = get_current_branch()?;
    if current_branch != "main" {
        log_warning(&format!("⚠️  Not on main branch (currently on: {}). Skipping cleanup.", current_branch));
        return Ok(());
    }

    // Check if main is ahead of origin/main
    let ahead_commits = get_ahead_commits()?;
    if ahead_commits.is_empty() {
        log_success("✅ Main is clean and up to date with origin/main");
        return Ok(());
    }

    log_warning(&format!("⚠️  Main is ahead of origin/main by {} commit(s):", ahead_commits.len()));
    for commit in &ahead_commits {
        log_info(&format!("   - {}", commit));
    }

    // Check for uncommitted changes
    let uncommitted = get_uncommitted_changes()?;
    if !uncommitted.is_empty() {
        log_warning("⚠️  Found uncommitted changes:");
        for change in &uncommitted {
            log_info(&format!("   - {}", change));
        }
    }

    // Create timestamp for worktree name
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let worktree_name = format!("fix/main-cleanup-{}", timestamp);
    let branch_name = format!("fix/main-cleanup-{}", timestamp);

    log_info(&format!("🔄 Creating worktree: {} -> {}", branch_name, worktree_name));

    // Create worktree
    create_worktree(&branch_name, &worktree_name)?;

    // Switch to worktree
    log_info("🔄 Switching to worktree...");
    switch_to_worktree(&worktree_name)?;

    // Commit all changes in worktree
    if !uncommitted.is_empty() {
        log_info("🔄 Committing changes in worktree...");
        commit_changes(&branch_name)?;
    }

    // Switch back to main
    log_info("🔄 Switching back to main...");
    switch_to_main()?;

    // Reset main to match origin/main
    log_info("🔄 Resetting main to match origin/main...");
    reset_main()?;

    log_success("✅ Main cleanup completed!");
    log_info(&format!("📁 Changes moved to worktree: {}", worktree_name));
    log_info(&format!("🌿 Branch: {}", branch_name));

    Ok(())
}

fn get_current_branch() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;

    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(branch)
}

fn get_ahead_commits() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "origin/main..HEAD"])
        .output()?;

    let commits = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(commits)
}

fn get_uncommitted_changes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;

    let changes = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(changes)
}

fn create_worktree(branch_name: &str, worktree_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // First create the branch
    run_git_command(&["checkout", "-b", branch_name])?;

    // Create worktree using the project's worktree tools
    let output = Command::new("cargo")
        .args(&["xtask", "worktree", "create", "--branch", branch_name, "--switch"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(format!("Failed to create worktree: {}", stderr).into());
    }

    Ok(())
}

fn switch_to_worktree(worktree_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let worktree_path = format!("worktree-{}", worktree_name);
    if !std::path::Path::new(&worktree_path).exists() {
        return Err(format!("Worktree directory not found: {}", worktree_path).into());
    }

    // Change to worktree directory
    env::set_current_dir(&worktree_path)?;

    Ok(())
}

fn commit_changes(branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Add all changes
    run_git_command(&["add", "."])?;

    // Commit changes
    let commit_message = format!("feat: Move main changes to worktree {}", branch_name);
    run_git_command(&["commit", "-m", &commit_message])?;

    Ok(())
}

fn switch_to_main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the repository root from the current directory
    let repo_root = get_repo_root()?;
    
    // Change back to main directory
    env::set_current_dir(&repo_root)?;

    // Switch to main branch
    run_git_command(&["checkout", "main"])?;

    Ok(())
}

fn reset_main() -> Result<(), Box<dyn std::error::Error>> {
    run_git_command(&["reset", "--hard", "origin/main"])?;
    Ok(())
}

fn get_repo_root() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    let repo_root = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(std::path::PathBuf::from(repo_root))
}
