use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, run_git_command_in_dir, get_worktree_status, determine_state
};

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

    fn get_worktree_state(&self, worktree_path: &str) -> Result<WorktreeState, Box<dyn std::error::Error>> {
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

    fn transition_state(&self, worktree_path: &str, branch_name: &str, current_state: &WorktreeState, target_state: &WorktreeState) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Transitioning {}: {} → {}", branch_name, current_state.to_string(), target_state.to_string()));

        match target_state {
            WorktreeState::Resolving => {
                let result = run_git_command_in_dir(&["rebase", "main"], worktree_path).map_err(|e| e.to_string())?;
                log_success("Rebase successful");
                Ok(())
            }
            WorktreeState::Ready => {
                let result = run_git_command_in_dir(&["push", "origin", branch_name], worktree_path);
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
                let _ = run_git_command(&["worktree", "remove", worktree_path, "--force"]);

                // Delete remote branch
                let _ = run_git_command(&["push", "origin", "--delete", branch_name]);

                log_success("Worktree cleaned up");
                Ok(())
            }
            _ => {
                log_warning(&format!("Unknown target state: {}", target_state.to_string()));
                Err("Unknown target state".into())
            }
        }
    }

    fn generate_pr_url(&self, branch_name: &str) -> String {
        let repo_url = run_git_command(&["config", "--get", "remote.origin.url"])
            .unwrap_or_else(|_| "".to_string())
            .replace(".git", "");

        if repo_url.contains("github.com") {
            format!("{}/compare/main...{}", repo_url, branch_name)
        } else {
            "Unknown repository URL".to_string()
        }
    }

    fn process_worktree(&mut self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

        // Get current state
        let current_state = self.get_worktree_state(worktree_path)?;
        log_info(&format!("Current state: {}", current_state.to_string()));

        // Determine next state
        let next_state = match current_state {
            WorktreeState::Created => Some(WorktreeState::Developing),
            WorktreeState::Developing => {
                // Check if ready to move to next state
                let status = run_git_command_in_dir(&["status", "--porcelain"], worktree_path).map_err(|e| e.to_string())?;
                
                if status.is_empty() {
                    Some(WorktreeState::Resolving)
                } else {
                    None
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
            if self.transition_state(worktree_path, &branch_name, &current_state, &next_state).is_ok() {
                log_success(&format!("Successfully transitioned to {}", next_state.to_string()));
                Ok(())
            } else {
                log_warning(&format!("Failed to transition to {}", next_state.to_string()));
                Err("Transition failed".into())
            }
        } else {
            log_info("No transition needed");
            Ok(())
        }
    }

    fn print_diagram(&self) {
        log_header("WORKTREE STATE MACHINE DIAGRAM");
        println!();
        println!("CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED");
        println!("    ↓         ↓");
        println!("CONFLICTED → RESOLVING");
        println!();
        println!("State Descriptions:");
        for state in [
            WorktreeState::Created,
            WorktreeState::Developing,
            WorktreeState::Conflicted,
            WorktreeState::Resolving,
            WorktreeState::Resolved,
            WorktreeState::Ready,
            WorktreeState::PrCreated,
            WorktreeState::Merged,
            WorktreeState::Cleanup,
            WorktreeState::Removed,
        ] {
            println!("  {}: {}", state.to_string(), state.description());
        }
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
            let branch_name = Path::new(worktree_path)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                .to_string_lossy()
                .to_string();

            if branch_name == "hooksmith" {
                continue;
            }

            processed_count += 1;

            if self.process_worktree(worktree_path).is_ok() {
                success_count += 1;
            }

            println!("---");
        }

        log_success(&format!("Processed {} worktree(s), {} successful", processed_count, success_count));
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("state_machine");
    println!();

    let args: Vec<String> = std::env::args().collect();
    let default_command = "process".to_string();
    let command = args.get(1).unwrap_or(&default_command);

    let mut state_machine = WorktreeStateMachine::new()?;

    match command.as_str() {
        "diagram" => {
            state_machine.print_diagram();
        }
        "process" => {
            state_machine.process_all_worktrees()?;
        }
        "status" => {
            log_info("Running worktree status report...");
            let status_output = Command::new("cargo")
                .args(&["run", "--bin", "worktree-status-report"])
                .output()?;

            if status_output.status.success() {
                println!("{}", String::from_utf8_lossy(&status_output.stdout));
            } else {
                log_error("Failed to run worktree status report");
            }
        }
        "help" | "--help" | "-h" => {
            println!("Usage: cargo run --bin state_machine [diagram|process|status|help]");
            println!("  diagram: Show state machine diagram");
            println!("  process: Process all worktrees through state machine");
            println!("  status: Show current worktree status");
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
