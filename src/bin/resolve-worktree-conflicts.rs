use hooksmith::{
    cleanup_merged_worktree, get_worktrees, is_rebasing, log_error, log_header, log_info,
    log_success, log_warning, push_branch, run_git_command_in_dir, stash_changes,
};
use std::path::Path;
use std::process::Command;

fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!(
        "Processing worktree: {} (branch: {})",
        worktree_path, branch_name
    ));

    if !Path::new(worktree_path).exists() {
        log_error(&format!(
            "Worktree directory does not exist: {}",
            worktree_path
        ));
        return Err("Worktree directory does not exist".to_string());
    }

    // Check current status
    let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path)?;
    let is_rebase_state = is_rebasing(worktree_path)?;

    log_info(&format!("Current status: {}", status.trim()));

    if is_rebase_state {
        log_warning("Rebase in progress - aborting to preserve state");
        let output = Command::new("git")
            .args(&["rebase", "--abort"])
            .current_dir(worktree_path)
            .output()
            .map_err(|e| format!("Failed to abort rebase: {}", e))?;

        if output.status.success() {
            log_success("Rebase aborted");
        } else {
            log_warning("Failed to abort rebase");
        }
    }

    // Stash any uncommitted changes
    if !run_git_command_in_dir(&["diff", "--quiet"], worktree_path).is_ok() {
        stash_changes(worktree_path)?;
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    let output = Command::new("git")
        .args(&["rebase", "main"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to rebase: {}", e))?;

    if output.status.success() {
        log_success("Rebase successful");
        Ok(true)
    } else {
        log_warning("Rebase failed - preserving worktree state");
        let abort_output = Command::new("git")
            .args(&["rebase", "--abort"])
            .current_dir(worktree_path)
            .output()
            .map_err(|e| format!("Failed to abort rebase: {}", e))?;

        if abort_output.status.success() {
            log_info("Rebase aborted successfully");
        }
        Ok(false)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("COMPREHENSIVE WORKTREE CONFLICT RESOLUTION");
    println!();

    let worktrees = get_worktrees()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    let mut processed_count = 0;
    let mut successful_count = 0;
    let mut conflicted_count = 0;

    // Process each worktree
    for worktree_path in &worktrees {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        match resolve_worktree_conflicts(worktree_path, &branch_name) {
            Ok(success) => {
                if success {
                    successful_count += 1;
                } else {
                    conflicted_count += 1;
                }
                processed_count += 1;
            }
            Err(e) => {
                log_error(&format!("Failed to resolve conflicts: {}", e));
            }
        }

        // Push branch
        if let Err(e) = push_branch(worktree_path, &branch_name) {
            log_warning(&format!("Failed to push branch {}: {}", branch_name, e));
        }

        // Check if merged and cleanup if needed
        if let Err(e) = cleanup_merged_worktree(worktree_path, &branch_name) {
            log_warning(&format!("Failed to cleanup merged worktree: {}", e));
        }

        println!("---");
    }

    // Summary
    log_header("SUMMARY");
    println!();
    log_success(&format!("Processed {} worktree(s)", processed_count));
    log_info(&format!("Successfully resolved: {}", successful_count));
    log_warning(&format!("Remaining conflicts: {}", conflicted_count));

    Ok(())
}
