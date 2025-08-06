use std::process::Command;
use std::path::Path;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header, print_status};

fn remove_worktree(worktree_name: &str) -> Result<bool, String> {
    if !Path::new(worktree_name).exists() {
        log_warning(&format!("Worktree {} does not exist", worktree_name));
        return Ok(false);
    }

    log_info(&format!("Removing worktree: {}", worktree_name));

    // Abort any ongoing operations first
    let output = Command::new("git")
        .args(&["status"])
        .current_dir(worktree_name)
        .output()
        .map_err(|e| format!("Failed to check status: {}", e))?;

    if String::from_utf8_lossy(&output.stdout).contains("rebase") {
        log_info(&format!("Aborting rebase in {}", worktree_name));
        let _ = Command::new("git")
            .args(&["rebase", "--abort"])
            .current_dir(worktree_name)
            .output();
    }

    // Get branch name before removal
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_name)
        .output()
        .map_err(|e| format!("Failed to get branch name: {}", e))?;

    let branch_name = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    // Remove worktree
    log_info("Removing worktree directory");
    let output = Command::new("git")
        .args(&["worktree", "remove", "--force", worktree_name])
        .output()
        .map_err(|e| format!("Failed to remove worktree: {}", e))?;

    if !output.status.success() {
        log_warning("Could not remove worktree, trying to delete directory");
        std::fs::remove_dir_all(worktree_name)
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    }

    // Remove branch if it exists
    if !branch_name.is_empty() {
        log_info(&format!("Removing branch: {}", branch_name));
        let _ = Command::new("git")
            .args(&["branch", "-D", &branch_name])
            .output();
    }

    log_success(&format!("Removed worktree {}", worktree_name));
    Ok(true)
}

fn create_pr_for_ready() -> Result<bool, String> {
    let worktree_name = "worktree-management-improvements";

    if !Path::new(worktree_name).exists() {
        log_warning(&format!("Ready worktree {} does not exist", worktree_name));
        return Ok(false);
    }

    log_info(&format!("Creating PR for ready worktree: {}", worktree_name));

    // Get branch name
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_name)
        .output()
        .map_err(|e| format!("Failed to get branch name: {}", e))?;

    let branch_name = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
    } else {
        log_error("Failed to get branch name");
        return Ok(false);
    };

    if !branch_name.is_empty() {
        // Check if branch exists on origin
        let remote_output = Command::new("git")
            .args(&["ls-remote", "--heads", "origin", &branch_name])
            .current_dir(worktree_name)
            .output()
            .map_err(|e| format!("Failed to check remote: {}", e))?;

        if remote_output.status.success() && !String::from_utf8_lossy(&remote_output.stdout).is_empty() {
            // Get repo URL
            let url_output = Command::new("git")
                .args(&["config", "--get", "remote.origin.url"])
                .current_dir(worktree_name)
                .output()
                .map_err(|e| format!("Failed to get repo URL: {}", e))?;

            if url_output.status.success() {
                let repo_url = String::from_utf8_lossy(&url_output.stdout).trim().replace(".git", "");
                if repo_url.contains("github.com") {
                    let pr_url = format!("{}/compare/main...{}", repo_url, branch_name);
                    log_success(&format!("Create PR at: {}", pr_url));
                    return Ok(true);
                }
            }
        }
    }

    log_warning("Could not create PR URL");
    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CLEANING UP OLD WORKTREES");
    println!();

    // List of worktrees to remove (old conflicted ones)
    let old_worktrees = vec![
        "worktree-fix-main-cleanup-20250804-211403",
        "worktree-fix-workspace-config",
        "worktree-fix-workspace-dependencies",
        "worktree-fix-xtask-cleanup",
    ];

    println!("🗑️  Removing old conflicted worktrees...");
    println!();

    let mut removed_count = 0;
    for worktree in &old_worktrees {
        if remove_worktree(worktree)? {
            removed_count += 1;
        }
        println!();
    }

    println!("🚀 Creating PR for ready worktree...");
    println!();
    create_pr_for_ready()?;

    println!("🎉 Cleanup completed!");
    println!();
    println!("📊 Final Status:");
    
    // Run worktree status report
    let output = Command::new("cargo")
        .args(&["run", "--bin", "worktree-status-report"])
        .output()
        .map_err(|e| format!("Failed to run status report: {}", e))?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    log_success(&format!("Removed {} old worktrees", removed_count));

    Ok(())
}
