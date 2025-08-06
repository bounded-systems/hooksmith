use std::process::Command;
use std::path::Path;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, get_git_status, is_rebasing
};

struct ConflictResolver {
    worktrees: Vec<hooksmith::Worktree>,
}

impl ConflictResolver {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees()?;
        Ok(Self { worktrees })
    }

    fn is_rebasing(&self, worktree_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        let status = get_git_status()?;
        let has_conflicts = status.lines().any(|line| line.starts_with("UU ") || line.starts_with("AA ") || line.starts_with("DD "));

        let rebase_in_progress = is_rebasing()?;

        std::env::set_current_dir(original_dir)?;

        Ok(has_conflicts || rebase_in_progress)
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

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Check if there are uncommitted changes
        let status = get_git_status()?;
        if !status.is_empty() {
            let stash_message = format!("Auto-stash during conflict resolution {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
            run_git_command(&["stash", "push", "-m", &stash_message])?;
            log_success("Changes stashed");
        } else {
            log_info("No changes to stash");
        }

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    fn resolve_worktree_conflicts(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

        if !Path::new(worktree_path).exists() {
            log_error(&format!("Worktree directory does not exist: {}", worktree_path));
            return Err("Worktree directory does not exist".into());
        }

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Check current status
        let status = get_git_status()?;
        let is_rebase_state = self.is_rebasing(worktree_path)?;

        log_info(&format!("Current status: {}", status));

        if is_rebase_state {
            log_warning("Rebase in progress - aborting to preserve state");
            self.abort_rebase(worktree_path)?;
        }

        // Stash any uncommitted changes
        if !status.is_empty() {
            self.stash_changes(worktree_path)?;
        }

        // Try to rebase onto main
        log_info("Attempting to rebase onto main");
        match run_git_command(&["rebase", "main"]) {
            Ok(_) => {
                log_success("Rebase successful");
            }
            Err(_) => {
                log_warning("Rebase failed - preserving worktree state");
                let _ = run_git_command(&["rebase", "--abort"]);
            }
        }

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    fn push_worktree_branch(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Pushing branch {}", branch_name));

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        match run_git_command(&["push", "origin", branch_name]) {
            Ok(_) => {
                log_success("Branch pushed successfully");
            }
            Err(_) => {
                log_warning("Push failed - branch may already be up to date");
            }
        }

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    fn cleanup_merged_worktree(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Checking if worktree {} is merged", branch_name));

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Check if branch is merged into main
        let merged_branches = run_git_command(&["branch", "--merged", "main"])?;
        let is_merged = merged_branches.lines().any(|line| line.trim() == branch_name);

        if is_merged {
            log_info(&format!("Branch {} is merged - cleaning up", branch_name));

            // Remove worktree
            std::env::set_current_dir(original_dir)?;
            let _ = run_git_command(&["worktree", "remove", worktree_path, "--force"]);

            // Delete branch from origin
            let _ = run_git_command(&["push", "origin", "--delete", branch_name]);

            log_success("Merged worktree cleaned up");
        } else {
            log_info(&format!("Branch {} is not merged - keeping worktree", branch_name));
        }

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    fn process_all_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Starting comprehensive worktree conflict resolution");

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut processed_count = 0;
        let mut success_count = 0;

        for worktree in &self.worktrees {
            let worktree_path = &worktree.path;
            let branch_name = &worktree.branch;

            log_info(&format!("Processing worktree: {}", worktree_path));

            processed_count += 1;

            // Resolve conflicts
            if self.resolve_worktree_conflicts(worktree_path, branch_name).is_ok() {
                // Push branch
                if self.push_worktree_branch(worktree_path, branch_name).is_ok() {
                    // Check if merged and cleanup if needed
                    if self.cleanup_merged_worktree(worktree_path, branch_name).is_ok() {
                        success_count += 1;
                    }
                }
            }

            println!("---");
        }

        log_success(&format!("Worktree conflict resolution completed. Processed: {}, Successful: {}", processed_count, success_count));
        Ok(())
    }

    fn process_single_worktree(&self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        log_info(&format!("Processing single worktree: {} (branch: {})", worktree_path, branch_name));

        // Resolve conflicts
        self.resolve_worktree_conflicts(worktree_path, &branch_name)?;

        // Push branch
        self.push_worktree_branch(worktree_path, &branch_name)?;

        // Check if merged and cleanup if needed
        self.cleanup_merged_worktree(worktree_path, &branch_name)?;

        log_success("Single worktree conflict resolution completed");
        Ok(())
    }

    fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("WORKTREE CONFLICT STATUS");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        for worktree in &self.worktrees {
            let worktree_path = &worktree.path;
            let branch_name = &worktree.branch;

            log_info(&format!("Worktree: {} (branch: {})", worktree_path, branch_name));

            // Check if worktree has conflicts
            match self.is_rebasing(worktree_path) {
                Ok(true) => {
                    log_warning("  Status: Has conflicts or rebase in progress");
                }
                Ok(false) => {
                    log_success("  Status: Clean");
                }
                Err(_) => {
                    log_error("  Status: Error checking status");
                }
            }

            // Check if merged
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(worktree_path)?;

            match run_git_command(&["branch", "--merged", "main"]) {
                Ok(merged_branches) => {
                    let is_merged = merged_branches.lines().any(|line| line.trim() == branch_name);
                    if is_merged {
                        log_info("  Merged: Yes");
                    } else {
                        log_info("  Merged: No");
                    }
                }
                Err(_) => {
                    log_warning("  Merged: Unknown");
                }
            }

            std::env::set_current_dir(original_dir)?;
            println!();
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("conflict_resolver");
    println!();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).unwrap_or(&"process".to_string());
    let target_path = args.get(2);

    let resolver = ConflictResolver::new()?;

    match command.as_str() {
        "process" => {
            if let Some(path) = target_path {
                resolver.process_single_worktree(path)?;
            } else {
                resolver.process_all_worktrees()?;
            }
        }
        "status" => {
            resolver.show_status()?;
        }
        "abort" => {
            if let Some(path) = target_path {
                log_info(&format!("Aborting rebase in {}", path));
                resolver.abort_rebase(path)?;
            } else {
                log_error("Please specify a worktree path for abort command");
                std::process::exit(1);
            }
        }
        "stash" => {
            if let Some(path) = target_path {
                log_info(&format!("Stashing changes in {}", path));
                resolver.stash_changes(path)?;
            } else {
                log_error("Please specify a worktree path for stash command");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            println!("Usage: cargo run --bin conflict_resolver [process|status|abort|stash|help] [worktree_path]");
            println!("  process [path]: Process all worktrees or specific worktree for conflict resolution");
            println!("  status: Show conflict status of all worktrees");
            println!("  abort [path]: Abort rebase in specific worktree");
            println!("  stash [path]: Stash changes in specific worktree");
            println!("  help: Show this help message");
        }
        _ => {
            log_error(&format!("Unknown command: {}", command));
            println!("Use 'help' for usage information");
            std::process::exit(1);
        }
    }

    log_success("Script execution completed");
    Ok(())
}
