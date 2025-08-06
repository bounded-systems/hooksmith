use std::process::Command;
use std::path::Path;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header, get_worktrees, run_git_command_in_dir, print_status};

fn is_rebasing(worktree_path: &str) -> Result<bool, String> {
    let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path)?;
    let has_conflicts = status.lines().any(|line| line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD"));

    if has_conflicts {
        return Ok(true);
    }

    let status_output = run_git_command_in_dir(&["status"], worktree_path)?;
    Ok(status_output.contains("rebase in progress"))
}

fn abort_rebase(worktree_path: &str) -> Result<bool, String> {
    log_info(&format!("Aborting rebase in {}", worktree_path));

    if is_rebasing(worktree_path)? {
        run_git_command_in_dir(&["rebase", "--abort"], worktree_path)?;
        log_success("Rebase aborted successfully");
        Ok(true)
    } else {
        log_info("No rebase in progress");
        Ok(false)
    }
}

fn stash_changes(worktree_path: &str) -> Result<bool, String> {
    log_info(&format!("Stashing changes in {}", worktree_path));

    // Check if there are uncommitted changes
    let diff_output = run_git_command_in_dir(&["diff", "--quiet"], worktree_path);
    if diff_output.is_ok() {
        log_info("No changes to stash");
        return Ok(false);
    }

    // Stash changes
    let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    run_git_command_in_dir(&["stash", "push", "-m", &stash_message], worktree_path)?;
    log_success("Changes stashed");
    Ok(true)
}

fn resolve_worktree_conflicts(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    if !Path::new(worktree_path).exists() {
        log_error(&format!("Worktree directory does not exist: {}", worktree_path));
        return Err("Worktree does not exist".to_string());
    }

    // Check current status
    let status = run_git_command_in_dir(&["status", "--short"], worktree_path)?;
    log_info(&format!("Current status: {}", status.trim()));

    // Check if in rebase state
    if is_rebasing(worktree_path)? {
        log_warning("Rebase in progress - aborting to preserve state");
        abort_rebase(worktree_path)?;
    }

    // Stash any uncommitted changes
    stash_changes(worktree_path)?;

    // Try to rebase onto main
    log_info("Attempting to rebase onto main");
    match run_git_command_in_dir(&["rebase", "main"], worktree_path) {
        Ok(_) => {
            log_success("Rebase successful");
            Ok(true)
        }
        Err(e) => {
            log_warning(&format!("Rebase failed: {}", e));
            // Abort the failed rebase
            let _ = run_git_command_in_dir(&["rebase", "--abort"], worktree_path);
            Ok(false)
        }
    }
}

fn push_worktree_branch(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Pushing branch {}", branch_name));

    match run_git_command_in_dir(&["push", "origin", branch_name], worktree_path) {
        Ok(_) => {
            log_success("Branch pushed successfully");
            Ok(true)
        }
        Err(e) => {
            log_warning(&format!("Push failed: {} - branch may already be up to date", e));
            Ok(false)
        }
    }
}

fn cleanup_merged_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Checking if worktree {} is merged", branch_name));

    // Check if branch is merged into main
    let merged_branches = run_git_command_in_dir(&["branch", "--merged", "main"], worktree_path)?;
    if merged_branches.lines().any(|line| line.trim() == format!("* {}", branch_name)) {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Remove worktree
        let output = Command::new("git")
            .args(&["worktree", "remove", "--force", worktree_path])
            .output()
            .map_err(|e| format!("Failed to remove worktree: {}", e))?;

        if output.status.success() {
            log_success(&format!("Removed worktree: {}", branch_name));
        } else {
            log_warning(&format!("Failed to remove worktree: {}", branch_name));
        }

        // Delete branch from origin
        let _ = Command::new("git")
            .args(&["push", "origin", "--delete", branch_name])
            .output();

        log_success("Merged worktree cleaned up");
        Ok(true)
    } else {
        log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
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
    let mut resolved_count = 0;
    let mut cleaned_count = 0;

    // Process each worktree
    for worktree_path in &worktrees {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        log_info(&format!("Processing worktree: {}", worktree_path));

        // Resolve conflicts
        if resolve_worktree_conflicts(worktree_path, &branch_name)? {
            resolved_count += 1;
        }

        // Push branch
        push_worktree_branch(worktree_path, &branch_name)?;

        // Check if merged and cleanup if needed
        if cleanup_merged_worktree(worktree_path, &branch_name)? {
            cleaned_count += 1;
        }

        processed_count += 1;
        println!("---");
    }

    // Summary
    log_header("SUMMARY");
    println!();
    log_success(&format!("Processed {} worktree(s)", processed_count));
    log_info(&format!("Resolved conflicts in {} worktree(s)", resolved_count));
    log_info(&format!("Cleaned up {} merged worktree(s)", cleaned_count));

    Ok(())
}
