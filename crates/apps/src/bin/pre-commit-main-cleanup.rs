use hooksmith::{log_error, log_header, log_info, log_success, log_warning, run_git_command};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("PRE-COMMIT MAIN CLEANUP");
    println!();

    // Get the current branch
    let current_branch = run_git_command(&["branch", "--show-current"])?;
    let current_branch = current_branch.trim();

    log_info(&format!("Current branch: {}", current_branch));

    // Only run on main branch
    if current_branch == "main" {
        log_info("🔍 Pre-commit: Checking main branch status...");

        // Check if main is ahead of origin/main
        let ahead_commits_output = run_git_command(&["log", "--oneline", "origin/main..HEAD"])?;
        let ahead_commits = ahead_commits_output.lines().count();

        if ahead_commits > 0 {
            log_warning(&format!(
                "⚠️  Pre-commit: Main is ahead of origin/main by {} commit(s)",
                ahead_commits
            ));
            log_info("🔄 Running main cleanup workflow...");

            // Run the cleanup script
            let cleanup_result = Command::new("./scripts/ensure-clean-main.sh").status();

            match cleanup_result {
                Ok(status) if status.success() => {
                    log_success("✅ Pre-commit: Main cleanup completed");
                }
                Ok(status) => {
                    log_error(&format!(
                        "❌ Pre-commit: Main cleanup failed with exit code: {}",
                        status
                    ));
                    std::process::exit(1);
                }
                Err(e) => {
                    log_error(&format!(
                        "❌ Pre-commit: Failed to run cleanup script: {}",
                        e
                    ));
                    std::process::exit(1);
                }
            }
        } else {
            log_success("✅ Pre-commit: Main is clean");
        }
    } else {
        log_info(&format!(
            "Skipping pre-commit check - not on main branch (current: {})",
            current_branch
        ));
    }

    Ok(())
}
