use hooksmith::{
    determine_state, get_worktree_status, get_worktrees, log_error, log_header, log_info,
    log_success, log_warning, run_git_command, run_git_command_in_dir,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
enum WorktreeState {
    Created,
    Developing,
    Conflicted,
    Resolving,
    Resolved,
    Ready,
    PrCreated,
    Merged,
    Cleanup,
    Removed,
}

impl WorktreeState {
    fn to_string(&self) -> &'static str {
        match self {
            WorktreeState::Created => "CREATED",
            WorktreeState::Developing => "DEVELOPING",
            WorktreeState::Conflicted => "CONFLICTED",
            WorktreeState::Resolving => "RESOLVING",
            WorktreeState::Resolved => "RESOLVED",
            WorktreeState::Ready => "READY",
            WorktreeState::PrCreated => "PR_CREATED",
            WorktreeState::Merged => "MERGED",
            WorktreeState::Cleanup => "CLEANUP",
            WorktreeState::Removed => "REMOVED",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            WorktreeState::Created => "Worktree created",
            WorktreeState::Developing => "Worktree has uncommitted changes",
            WorktreeState::Conflicted => "Worktree has rebase conflicts",
            WorktreeState::Resolving => "Resolving conflicts",
            WorktreeState::Resolved => "Conflicts resolved",
            WorktreeState::Ready => "Worktree ready for PR",
            WorktreeState::PrCreated => "PR created",
            WorktreeState::Merged => "PR merged",
            WorktreeState::Cleanup => "Cleaning up worktree",
            WorktreeState::Removed => "Worktree removed",
        }
    }
}

struct WorktreeStateMachine {
    worktrees: Vec<String>,
}

impl WorktreeStateMachine {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees().map_err(|e| e.to_string())?;
        Ok(Self { worktrees })
    }

    fn get_worktree_state(
        &self,
        worktree_path: &str,
    ) -> Result<WorktreeState, Box<dyn std::error::Error>> {
        // Get worktree status using the library function
        let status = get_worktree_status(worktree_path).map_err(|e| e.to_string())?;

        // Map the library's WorktreeState to our internal WorktreeState
        match determine_state(&status) {
            hooksmith::WorktreeState::Merged => Ok(WorktreeState::Merged),
            hooksmith::WorktreeState::Conflicted => Ok(WorktreeState::Conflicted),
            hooksmith::WorktreeState::Developing => Ok(WorktreeState::Developing),
            hooksmith::WorktreeState::Ready => Ok(WorktreeState::Ready),
            hooksmith::WorktreeState::Outdated => Ok(WorktreeState::Resolving),
            hooksmith::WorktreeState::Unknown => Ok(WorktreeState::Created),
        }
    }

    fn transition_state(
        &self,
        worktree_path: &str,
        branch_name: &str,
        current_state: &WorktreeState,
        target_state: &WorktreeState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!(
            "Transitioning {}: {} → {}",
            branch_name,
            current_state.to_string(),
            target_state.to_string()
        ));

        match target_state {
            WorktreeState::Resolving => {
                let _result = run_git_command_in_dir(&["rebase", "main"], worktree_path)
                    .map_err(|e| e.to_string())?;
                log_success("Rebase successful");
                Ok(())
            }
            WorktreeState::Ready => {
                let result =
                    run_git_command_in_dir(&["push", "origin", branch_name], worktree_path);
                match result {
                    Ok(_) => {
                        log_success("Branch pushed successfully");
                        Ok(())
                    }
                    Err(_) => {
                        log_warning("Push failed");
                        Err("Push failed".into())
                    }
                }
            }
            WorktreeState::PrCreated => {
                let pr_url = self.generate_pr_url(branch_name);
                log_info(&format!("PR URL: {}", pr_url));
                Ok(())
            }
            WorktreeState::Cleanup => {
                // Remove worktree
                let result = run_git_command(&["worktree", "remove", worktree_path, "--force"]);
                if result.is_ok() {
                    // Try to delete remote branch
                    let _ = run_git_command(&["push", "origin", "--delete", branch_name]);
                    log_success("Worktree cleaned up");
                    Ok(())
                } else {
                    log_warning("Failed to remove worktree");
                    Err("Failed to remove worktree".into())
                }
            }
            _ => {
                log_warning(&format!(
                    "Unknown target state: {}",
                    target_state.to_string()
                ));
                Err("Unknown target state".into())
            }
        }
    }

    fn generate_pr_url(&self, branch_name: &str) -> String {
        // Get repository URL
        let output = run_git_command(&["config", "--get", "remote.origin.url"]);
        match output {
            Ok(repo_url) => {
                let repo_url = repo_url.trim().replace(".git", "");
                if repo_url.contains("github.com") {
                    format!("{}/compare/main...{}", repo_url, branch_name)
                } else {
                    "Unknown repository URL".to_string()
                }
            }
            Err(_) => "Unknown repository URL".to_string(),
        }
    }

    fn process_worktree(&mut self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = std::path::Path::new(worktree_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        log_info(&format!(
            "Processing worktree: {} (branch: {})",
            worktree_path, branch_name
        ));

        // Get current state
        let current_state = self.get_worktree_state(worktree_path)?;
        log_info(&format!("Current state: {}", current_state.to_string()));

        // Determine next state
        let next_state = match current_state {
            WorktreeState::Created => Some(WorktreeState::Developing),
            WorktreeState::Developing => {
                // Check if worktree is clean
                let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path);
                match status {
                    Ok(output) if output.trim().is_empty() => Some(WorktreeState::Resolving),
                    _ => None,
                }
            }
            WorktreeState::Conflicted => Some(WorktreeState::Resolving),
            WorktreeState::Resolving => Some(WorktreeState::Ready),
            WorktreeState::Ready => Some(WorktreeState::PrCreated),
            WorktreeState::PrCreated => Some(WorktreeState::Merged),
            WorktreeState::Merged => Some(WorktreeState::Cleanup),
            WorktreeState::Cleanup => Some(WorktreeState::Removed),
            _ => None,
        };

        if let Some(next_state) = next_state {
            match self.transition_state(worktree_path, &branch_name, &current_state, &next_state) {
                Ok(_) => {
                    log_success(&format!(
                        "Successfully transitioned to {}",
                        next_state.to_string()
                    ));
                    Ok(())
                }
                Err(e) => {
                    log_warning(&format!(
                        "Failed to transition to {}: {}",
                        next_state.to_string(),
                        e
                    ));
                    Err(e)
                }
            }
        } else {
            log_info("No transition needed");
            Ok(())
        }
    }

    fn print_diagram(&self) {
        log_header("WORKTREE STATE MACHINE DIAGRAM");
        println!();
        println!(
            "CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED"
        );
        println!("    ↓         ↓");
        println!("CONFLICTED → RESOLVING");
        println!();
        println!("State Descriptions:");
        println!("  CREATED: {}", WorktreeState::Created.description());
        println!("  DEVELOPING: {}", WorktreeState::Developing.description());
        println!("  CONFLICTED: {}", WorktreeState::Conflicted.description());
        println!("  RESOLVING: {}", WorktreeState::Resolving.description());
        println!("  RESOLVED: {}", WorktreeState::Resolved.description());
        println!("  READY: {}", WorktreeState::Ready.description());
        println!("  PR_CREATED: {}", WorktreeState::PrCreated.description());
        println!("  MERGED: {}", WorktreeState::Merged.description());
        println!("  CLEANUP: {}", WorktreeState::Cleanup.description());
        println!("  REMOVED: {}", WorktreeState::Removed.description());
    }

    fn process_all_worktrees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("PROCESSING ALL WORKTREES");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut processed_count = 0;
        let mut success_count = 0;

        let worktree_paths: Vec<String> = self.worktrees.clone();
        for worktree_path in &worktree_paths {
            // Skip main worktree
            let branch_name = std::path::Path::new(worktree_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if branch_name == "hooksmith" {
                continue;
            }

            processed_count += 1;

            match self.process_worktree(worktree_path) {
                Ok(_) => success_count += 1,
                Err(_) => {}
            }

            println!("---");
        }

        log_success(&format!(
            "Processed {} worktree(s), {} successful",
            processed_count, success_count
        ));
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("process");

    let mut state_machine = WorktreeStateMachine::new()?;

    match command {
        "diagram" => {
            state_machine.print_diagram();
        }
        "process" => {
            state_machine.process_all_worktrees()?;
        }
        "status" => {
            // Run worktree status report
            let output = Command::new("cargo")
                .args(&["run", "--bin", "worktree-status-report"])
                .output()?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        _ => {
            println!("Usage: {} [diagram|process|status]", args[0]);
            println!("  diagram: Show state machine diagram");
            println!("  process: Process all worktrees through state machine");
            println!("  status: Show current worktree status");
            std::process::exit(1);
        }
    }

    Ok(())
}
