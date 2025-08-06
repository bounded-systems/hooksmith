use std::process::Command;
use std::path::Path;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header, get_worktrees, run_git_command_in_dir, is_rebasing, stash_changes, push_branch, cleanup_merged_worktree};

fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    if !Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Err("Worktree directory does not exist".to_string());
    }

    // Check current status
    let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path)?;
    let is_rebase_state = is_rebasing(worktree_path)?;

    log_info(&format!("Current status: {}", status));

    if is_rebase_state {
        log_warning("Rebase in progress - aborting to preserve state");
        run_git_command_in_dir(&["rebase", "--abort"], worktree_path)?;
        log_success("Rebase aborted");
    }

    // Stash any uncommitted changes
    let has_changes = !run_git_command_in_dir(&["diff", "--quiet"], worktree_path).is_ok();
    if has_changes {
        stash_changes(worktree_path)?;
    }

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    match run_git_command_in_dir(&["rebase", "main"], worktree_path) {
        Ok(_) => {
            log_success("Rebase successful");
            Ok(true)
        }
        Err(_) => {
            log_warning("Rebase failed - preserving worktree state");
            run_git_command_in_dir(&["rebase", "--abort"], worktree_path)?;
            Ok(false)
        }
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
    let mut resolved_count = 0;
    let mut failed_count = 0;

    // Process each worktree
    for worktree_path in &worktrees {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        // Skip main worktree
        if branch_name == "hooksmith" {
            continue;
        }

        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        match resolve_worktree_conflicts(worktree_path, &branch_name) {
            Ok(success) => {
                if success {
                    resolved_count += 1;
                } else {
                    failed_count += 1;
                }
                processed_count += 1;
            }
            Err(e) => {
                log_error(&format!("Failed to resolve conflicts: {}", e));
                failed_count += 1;
            }
        }

        // Push branch
        if push_branch(worktree_path, &branch_name)? {
            log_success(&format!("Branch {} pushed successfully", branch_name));
        } else {
            log_warning(&format!("Failed to push branch {}", branch_name));
        }

        // Check if merged and cleanup if needed
        if cleanup_merged_worktree(worktree_path, &branch_name)? {
            log_success(&format!("Merged worktree {} cleaned up", branch_name));
        }

        println!("---");
    }

    // Summary
    log_header("SUMMARY");
    println!();
    log_success(&format!("Processed {} worktree(s)", processed_count));
    log_info(&format!("Resolved: {}", resolved_count));
    log_warning(&format!("Failed: {}", failed_count));

    Ok(())
}
