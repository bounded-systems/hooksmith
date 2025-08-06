use hooksmith::{
    get_worktree_status, get_worktrees, log_error, log_header, log_info, log_success, log_warning,
    run_git_command, run_git_command_in_dir,
};
use std::path::Path;
use std::process::Command;

struct ConflictResolver {
    worktrees: Vec<String>,
}

impl ConflictResolver {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees().map_err(|e| e.to_string())?;
        Ok(Self { worktrees })
    }

    fn is_rebasing(&self, worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let status = get_worktree_status(worktree_path).map_err(|e| e.to_string())?;
        Ok(status.is_rebasing)
    }

    fn abort_rebase(&self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Aborting rebase in {}", worktree_path));

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        if self.is_rebasing(worktree_path)? {
            run_git_command(&["rebase", "--abort"])?;
            log_success("Rebase aborted successfully");
        } else {
            log_info("No rebase in progress");
        }

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    fn stash_changes(&self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Stashing changes in {}", worktree_path));

        // Check if there are uncommitted changes
        let status = get_worktree_status(worktree_path).map_err(|e| e.to_string())?;
        if !status.is_clean {
            let stash_message = format!(
                "Auto-stash during conflict resolution {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );
            run_git_command_in_dir(&["stash", "push", "-m", &stash_message], worktree_path)?;
            log_success("Changes stashed");
        } else {
            log_info("No changes to stash");
        }

        Ok(())
    }

    fn resolve_worktree_conflicts(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!(
            "Processing worktree: {} (branch: {})",
            worktree_path, branch_name
        ));

        if !Path::new(worktree_path).exists() {
            log_error(&format!(
                "Worktree directory does not exist: {}",
                worktree_path
            ));
            return Err("Worktree directory does not exist".into());
        }

        // Check current status
        let status = get_worktree_status(worktree_path).map_err(|e| e.to_string())?;
        let is_rebase_state = self.is_rebasing(worktree_path)?;

        log_info(&format!(
            "Current status: clean={}, rebasing={}",
            status.is_clean, status.is_rebasing
        ));

        if is_rebase_state {
            log_warning("Rebase in progress - aborting to preserve state");
            run_git_command_in_dir(&["rebase", "--abort"], worktree_path)?;
            log_success("Rebase aborted");
        }

        // Stash any uncommitted changes
        if !status.is_clean {
            self.stash_changes(worktree_path)?;
        }

        // Try to rebase onto main
        log_info("Attempting to rebase onto main");
        match run_git_command_in_dir(&["rebase", "main"], worktree_path) {
            Ok(_) => {
                log_success("Rebase successful");
            }
            Err(_) => {
                log_warning("Rebase failed - preserving worktree state");
                run_git_command_in_dir(&["rebase", "--abort"], worktree_path)?;
            }
        }
        Ok(())
    }

    fn push_worktree_branch(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Pushing branch {}", branch_name));

        match run_git_command_in_dir(&["push", "origin", branch_name], worktree_path) {
            Ok(_) => {
                log_success("Branch pushed successfully");
            }
            Err(_) => {
                log_warning("Push failed - branch may already be up to date");
            }
        }

        Ok(())
    }

    fn cleanup_merged_worktree(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Checking if worktree {} is merged", branch_name));

        // Check if branch is merged into main
        let merged_branches =
            run_git_command_in_dir(&["branch", "--merged", "main"], worktree_path)?;
        if merged_branches
            .lines()
            .any(|line| line.trim() == branch_name)
        {
            log_info(&format!("Branch {} is merged - cleaning up", branch_name));

            // Remove worktree
            let _ = run_git_command(&["worktree", "remove", worktree_path, "--force"]);

            // Delete branch from origin
            let _ = run_git_command(&["push", "origin", "--delete", branch_name]);

            log_success("Merged worktree cleaned up");
        } else {
            log_info(&format!(
                "Branch {} is not merged - keeping worktree",
                branch_name
            ));
        }
        Ok(())
    }

    fn process_all_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("COMPREHENSIVE WORKTREE CONFLICT RESOLUTION");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut processed_count = 0;
        let mut success_count = 0;

        for worktree_path in &self.worktrees {
            // Get branch name from worktree path
            let branch_name = std::path::Path::new(worktree_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip main worktree
            if branch_name == "hooksmith" {
                continue;
            }

            processed_count += 1;
            log_info(&format!("Processing worktree: {}", worktree_path));

            // Resolve conflicts
            match self.resolve_worktree_conflicts(worktree_path, &branch_name) {
                Ok(_) => {
                    // Push branch
                    let _ = self.push_worktree_branch(worktree_path, &branch_name);

                    // Check if merged and cleanup if needed
                    let _ = self.cleanup_merged_worktree(worktree_path, &branch_name);

                    success_count += 1;
                }
                Err(e) => {
                    log_error(&format!(
                        "Failed to process worktree {}: {}",
                        worktree_path, e
                    ));
                }
            }

            println!("---");
        }

        log_success(&format!(
            "Processed {} worktree(s), {} successful",
            processed_count, success_count
        ));
        Ok(())
    }

    fn process_single_worktree(
        &self,
        worktree_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = std::path::Path::new(worktree_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        log_info(&format!(
            "Processing single worktree: {} (branch: {})",
            worktree_path, branch_name
        ));

        // Resolve conflicts
        self.resolve_worktree_conflicts(worktree_path, &branch_name)?;

        // Push branch
        self.push_worktree_branch(worktree_path, &branch_name)?;

        // Check if merged and cleanup if needed
        self.cleanup_merged_worktree(worktree_path, &branch_name)?;

        log_success("Single worktree processing completed");
        Ok(())
    }

    fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("WORKTREE CONFLICT STATUS");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut conflicted_count = 0;
        let mut clean_count = 0;

        for worktree_path in &self.worktrees {
            let branch_name = std::path::Path::new(worktree_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if branch_name == "hooksmith" {
                continue;
            }

            match self.is_rebasing(worktree_path) {
                Ok(true) => {
                    log_warning(&format!(
                        "{} (branch: {}) - HAS CONFLICTS",
                        worktree_path, branch_name
                    ));
                    conflicted_count += 1;
                }
                Ok(false) => {
                    log_success(&format!(
                        "{} (branch: {}) - CLEAN",
                        worktree_path, branch_name
                    ));
                    clean_count += 1;
                }
                Err(e) => {
                    log_error(&format!(
                        "{} (branch: {}) - ERROR: {}",
                        worktree_path, branch_name, e
                    ));
                }
            }
        }

        println!();
        log_info(&format!(
            "Summary: {} clean, {} conflicted",
            clean_count, conflicted_count
        ));
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let worktree_path = args.get(2);

    let conflict_resolver = ConflictResolver::new()?;

    match command {
        "all" => {
            conflict_resolver.process_all_worktrees()?;
        }
        "single" => {
            if let Some(path) = worktree_path {
                conflict_resolver.process_single_worktree(path)?;
            } else {
                println!("Usage: {} single <worktree_path>", args[0]);
                std::process::exit(1);
            }
        }
        "status" => {
            conflict_resolver.show_status()?;
        }
        "abort" => {
            if let Some(path) = worktree_path {
                conflict_resolver.abort_rebase(path)?;
            } else {
                println!("Usage: {} abort <worktree_path>", args[0]);
                std::process::exit(1);
            }
        }
        "stash" => {
            if let Some(path) = worktree_path {
                conflict_resolver.stash_changes(path)?;
            } else {
                println!("Usage: {} stash <worktree_path>", args[0]);
                std::process::exit(1);
            }
        }
        _ => {
            println!(
                "Usage: {} [all|single|status|abort|stash] [worktree_path]",
                args[0]
            );
            println!("  all: Process all worktrees");
            println!("  single <path>: Process single worktree");
            println!("  status: Show conflict status of all worktrees");
            println!("  abort <path>: Abort rebase in worktree");
            println!("  stash <path>: Stash changes in worktree");
            std::process::exit(1);
        }
    }

    Ok(())
}
