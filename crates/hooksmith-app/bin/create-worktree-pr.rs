use hooksmith::{
    create_pr_with_gh, generate_pr_url, get_worktrees, is_ready_for_pr, log_error, log_header,
    log_info, log_success, log_warning, push_branch,
};
use std::path::Path;
use std::process::Command;

fn process_ready_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!(
        "Processing ready worktree: {} (branch: {})",
        worktree_path, branch_name
    ));

    // Push branch
    if push_branch(worktree_path, branch_name)? {
        // Try to create PR with GitHub CLI
        if Command::new("gh").arg("--version").output().is_ok() {
            if create_pr_with_gh(worktree_path, branch_name)? {
                log_success(&format!("PR created successfully for {}", branch_name));
                return Ok(true);
            }
        }

        // Fallback: generate PR URL
        let pr_url = generate_pr_url(branch_name);
        log_info(&format!("PR URL generated: {}", pr_url));
        log_warning("Please create PR manually using the URL above");
        return Ok(true);
    } else {
        log_error(&format!("Failed to push branch {}", branch_name));
        return Ok(false);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CREATE WORKTREE PRs");
    println!();

    let worktrees = get_worktrees()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    let mut ready_worktrees = Vec::new();
    let mut processed_count = 0;

    // Find ready worktrees
    for worktree_path in &worktrees {
        let branch_name = std::path::Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        // Skip main worktree
        if branch_name == "hooksmith" {
            continue;
        }

        // Check if ready for PR
        if is_ready_for_pr(worktree_path)? {
            ready_worktrees.push((worktree_path.clone(), branch_name));
        }
    }

    // Process ready worktrees
    if ready_worktrees.is_empty() {
        log_info("No worktrees ready for PR creation");
        return Ok(());
    }

    log_info(&format!(
        "Found {} worktree(s) ready for PR creation",
        ready_worktrees.len()
    ));
    println!();

    for (worktree_path, branch_name) in &ready_worktrees {
        if process_ready_worktree(worktree_path, branch_name)? {
            processed_count += 1;
        }

        println!("---");
    }

    log_success(&format!("Processed {} worktree(s)", processed_count));

    Ok(())
}
