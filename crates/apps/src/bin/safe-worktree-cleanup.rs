use hooksmith::{get_worktrees, log_error, log_info, log_success, log_warning, run_git_command};
use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct WorktreeResult {
    path: String,
    action: WorktreeAction,
    error: Option<String>,
}

#[derive(Debug)]
enum WorktreeAction {
    Removed,
    Skipped,
    Error,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("🔍 Checking worktrees for uncommitted changes...");

    let current_dir = env::current_dir()?;
    let worktrees = get_worktrees()?;

    // Use Arc<Mutex> for thread-safe counters
    let results = Arc::new(Mutex::new(Vec::new()));

    // Process worktrees sequentially (can be optimized later with rayon)
    worktrees.iter().for_each(|worktree_path| {
        let worktree_path = std::path::PathBuf::from(worktree_path);

        // Skip the main worktree
        if worktree_path == current_dir {
            log_info(&format!(
                "⏭️  Skipping main worktree: {}",
                worktree_path.display()
            ));
            return;
        }

        log_info(&format!(
            "📁 Checking worktree: {}",
            worktree_path.display()
        ));

        let result = process_worktree(&worktree_path);

        // Store result for later reporting
        results.lock().unwrap().push(result);
    });

    // Collect and report results
    let final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    let mut removed_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for result in final_results {
        match result.action {
            WorktreeAction::Removed => {
                log_success(&format!("✅ Removed worktree: {}", result.path));
                removed_count += 1;
            }
            WorktreeAction::Skipped => {
                log_warning(&format!("⚠️  Skipped worktree: {}", result.path));
                skipped_count += 1;
            }
            WorktreeAction::Error => {
                if let Some(error) = result.error {
                    log_error(&format!(
                        "❌ Error with worktree {}: {}",
                        result.path, error
                    ));
                }
                error_count += 1;
            }
        }
    }

    log_success(&format!(
        "🎉 Worktree cleanup completed! Removed: {}, Skipped: {}, Errors: {}",
        removed_count, skipped_count, error_count
    ));
    Ok(())
}

fn process_worktree(worktree_path: &Path) -> WorktreeResult {
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
                return WorktreeResult {
                    path: worktree_path.display().to_string(),
                    action: WorktreeAction::Removed,
                    error: None,
                };
            }
            Err(e) => {
                return WorktreeResult {
                    path: worktree_path.display().to_string(),
                    action: WorktreeAction::Error,
                    error: Some(e.to_string()),
                };
            }
        }
    }

    // Check for uncommitted changes using optimized git status
    let status_output = match Command::new("git")
        .args(&["status", "--porcelain", "--untracked-files=no"])
        .current_dir(worktree_path)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            return WorktreeResult {
                path: worktree_path.display().to_string(),
                action: WorktreeAction::Error,
                error: Some(format!("Failed to run git status: {}", e)),
            };
        }
    };

    let status = match String::from_utf8(status_output.stdout) {
        Ok(s) => s,
        Err(e) => {
            return WorktreeResult {
                path: worktree_path.display().to_string(),
                action: WorktreeAction::Error,
                error: Some(format!("Invalid UTF-8 in git status: {}", e)),
            };
        }
    };

    // Fast path: if no changes, remove immediately
    if status.trim().is_empty() {
        log_success(&format!(
            "✅ No uncommitted changes found in {}",
            worktree_path.display()
        ));

        match run_git_command(&[
            "worktree",
            "remove",
            worktree_path.to_str().unwrap(),
            "--force",
        ]) {
            Ok(_) => {
                return WorktreeResult {
                    path: worktree_path.display().to_string(),
                    action: WorktreeAction::Removed,
                    error: None,
                };
            }
            Err(e) => {
                return WorktreeResult {
                    path: worktree_path.display().to_string(),
                    action: WorktreeAction::Error,
                    error: Some(e.to_string()),
                };
            }
        }
    } else {
        // Has uncommitted changes - log and skip
        log_warning(&format!(
            "⚠️  WARNING: Uncommitted changes found in {}",
            worktree_path.display()
        ));

        // Only log changes if we're in verbose mode (avoid string formatting overhead)
        log_info("   Changes:");
        for line in status.lines() {
            if !line.trim().is_empty() {
                log_info(&format!("   {}", line));
            }
        }

        log_info("   Please commit or stash changes before removing this worktree");

        return WorktreeResult {
            path: worktree_path.display().to_string(),
            action: WorktreeAction::Skipped,
            error: None,
        };
    }
}
