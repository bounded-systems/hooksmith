use hooksmith::{
    get_worktree_paths, log_error, log_info, log_success, log_warning, run_git_command,
};
use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("🔄 Updating all worktrees to latest origin/main...");

    // Get the latest from origin
    log_info("📥 Fetching latest from origin...");
    run_git_command(&["fetch", "origin"])?;

    let current_dir = env::current_dir()?;
    let worktrees = get_worktree_paths()?;
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for worktree_path in worktrees {
        // Skip the main worktree
        if worktree_path == current_dir {
            continue;
        }

        if worktree_path.exists() {
            println!();
            log_info(&format!(
                "🔄 Updating worktree: {}",
                worktree_path.file_name().unwrap().to_string_lossy()
            ));
            log_info(&format!("   Path: {}", worktree_path.display()));

            // Get the branch name for this worktree
            let branch = get_current_branch(&worktree_path)?;
            log_info(&format!("   Branch: {}", branch));

            // Check if there are uncommitted changes
            let uncommitted = get_uncommitted_changes(&worktree_path)?;
            if !uncommitted.is_empty() {
                log_warning("   ⚠️  WARNING: Uncommitted changes detected!");
                log_info("   📝 Changes:");
                for change in &uncommitted {
                    log_info(&format!("   {}", change));
                }
                log_info("   💡 Consider committing or stashing changes before updating");
                log_info("   ⏭️  Skipping this worktree...");
                skipped_count += 1;
                continue;
            }

            // Get current commit
            let current_commit = get_current_commit(&worktree_path)?;
            log_info(&format!("   Current commit: {}", current_commit));

            // Check how many commits behind origin/main
            let behind_count = get_behind_count(&worktree_path)?;
            log_info(&format!(
                "   Behind origin/main by: {} commits",
                behind_count
            ));

            if behind_count == 0 {
                log_success("   ✅ Already up to date!");
                continue;
            }

            // Rebase to origin/main
            log_info("   🔄 Rebasing to origin/main...");
            match rebase_to_origin_main(&worktree_path) {
                Ok(_) => {
                    log_success("   ✅ Successfully updated!");
                    let new_commit = get_current_commit(&worktree_path)?;
                    log_info(&format!("   New commit: {}", new_commit));
                    updated_count += 1;
                }
                Err(e) => {
                    log_error(&format!(
                        "   ❌ Rebase failed! Manual intervention may be needed. Error: {}",
                        e
                    ));
                    log_info("   💡 You can:");
                    log_info(&format!("      - cd {}", worktree_path.display()));
                    log_info("      - git rebase --abort (to cancel)");
                    log_info("      - git rebase --continue (after resolving conflicts)");
                    failed_count += 1;
                }
            }
        }
    }

    println!();
    log_success("🎉 Worktree update process completed!");
    println!();
    log_info("📊 Summary:");
    let main_commit = get_current_commit(&current_dir)?;
    log_info(&format!("   - Main worktree: {}", main_commit));
    let origin_main_commit = get_origin_main_commit()?;
    log_info(&format!("   - Origin/main: {}", origin_main_commit));
    log_info(&format!(
        "   - Updated: {}, Skipped: {}, Failed: {}",
        updated_count, skipped_count, failed_count
    ));
    println!();
    log_info("💡 Next steps:");
    log_info("   - Review any worktrees that had conflicts");
    log_info("   - Test your changes in updated worktrees");
    log_info("   - Create PRs for worktrees that are ready");

    Ok(())
}

fn get_current_branch(
    worktree_path: &std::path::Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(worktree_path)
        .output()?;

    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(branch)
}

fn get_uncommitted_changes(
    worktree_path: &std::path::Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    let changes = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(changes)
}

fn get_current_commit(
    worktree_path: &std::path::Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(worktree_path)
        .output()?;

    let commit = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(commit)
}

fn get_behind_count(worktree_path: &std::path::Path) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-list", "--count", "HEAD..origin/main"])
        .current_dir(worktree_path)
        .output()?;

    let count_str = String::from_utf8(output.stdout)?.trim().to_string();
    let count: u32 = count_str.parse()?;
    Ok(count)
}

fn rebase_to_origin_main(
    worktree_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rebase", "origin/main"])
        .current_dir(worktree_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(format!("Rebase failed: {}", stderr).into());
    }

    Ok(())
}

fn get_origin_main_commit() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "origin/main", "-1"])
        .output()?;

    let commit = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(commit)
}
