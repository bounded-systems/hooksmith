use hooksmith::{
    get_worktrees, log_error, log_info, log_success, log_warning, run_git_command,
};
use std::env;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("🔍 Checking worktrees for uncommitted changes...");

    let current_dir = env::current_dir()?;
    let worktrees = get_worktrees()?;
    let mut removed_count = 0;
    let mut skipped_count = 0;

    for worktree_path in worktrees {
        // Skip the main worktree
        if worktree_path == current_dir {
            log_info(&format!(
                "⏭️  Skipping main worktree: {}",
                worktree_path.display()
            ));
            continue;
        }

        log_info(&format!(
            "📁 Checking worktree: {}",
            worktree_path.display()
        ));

        // Check if worktree directory exists
        if !worktree_path.exists() {
            log_warning(&format!(
                "🗑️  Removing non-existent worktree: {}",
                worktree_path.display()
            ));
            match run_git_command(&[
                "worktree",
                "remove",
                worktree_path.to_str().unwrap(),
                "--force",
            ]) {
                Ok(_) => {
                    log_success(&format!(
                        "✅ Removed non-existent worktree: {}",
                        worktree_path.display()
                    ));
                    removed_count += 1;
                }
                Err(e) => {
                    log_error(&format!(
                        "❌ Failed to remove non-existent worktree {}: {}",
                        worktree_path.display(),
                        e
                    ));
                }
            }
            continue;
        }

        // Check for uncommitted changes
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&worktree_path)
            .output()?;

        let status = String::from_utf8(status_output.stdout)?;

        if !status.trim().is_empty() {
            log_warning(&format!(
                "⚠️  WARNING: Uncommitted changes found in {}",
                worktree_path.display()
            ));
            log_info("   Changes:");
            for line in status.lines() {
                if !line.trim().is_empty() {
                    log_info(&format!("   {}", line));
                }
            }
            log_info("   Please commit or stash changes before removing this worktree");
            skipped_count += 1;
        } else {
            log_success(&format!(
                "✅ No uncommitted changes found in {}",
                worktree_path.display()
            ));
            log_info(&format!(
                "🗑️  Removing worktree: {}",
                worktree_path.display()
            ));

            match run_git_command(&[
                "worktree",
                "remove",
                worktree_path.to_str().unwrap(),
                "--force",
            ]) {
                Ok(_) => {
                    log_success(&format!(
                        "✅ Successfully removed worktree: {}",
                        worktree_path.display()
                    ));
                    removed_count += 1;
                }
                Err(e) => {
                    log_error(&format!(
                        "❌ Failed to remove worktree {}: {}",
                        worktree_path.display(),
                        e
                    ));
                }
            }
        }
    }

    log_success(&format!(
        "🎉 Worktree cleanup completed! Removed: {}, Skipped: {}",
        removed_count, skipped_count
    ));
    Ok(())
}
