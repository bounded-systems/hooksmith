use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, get_current_branch, get_git_status,
    is_rebasing, is_merged_into_main, get_commit_counts
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
    worktrees: HashMap<String, WorktreeState>,
}

impl WorktreeStateMachine {
    fn new() -> Self {
        Self {
            worktrees: HashMap::new(),
        }
    }

    fn get_worktree_state(&self, worktree_path: &str, branch_name: &str) -> WorktreeState {
        // Change to worktree directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(worktree_path).unwrap();

        // Get current branch
        let current_branch = get_current_branch().unwrap_or_else(|_| "unknown".to_string());
        
        // Get status
        let status = get_git_status().unwrap_or_else(|_| "".to_string());
        let is_clean = status.is_empty();
        
        // Check if rebasing
        let is_rebasing = is_rebasing().unwrap_or(false);
        
        // Check if merged into main
        let is_merged = is_merged_into_main(&current_branch).unwrap_or(false);
        
        // Get commit counts
        let (ahead, behind) = get_commit_counts("main").unwrap_or((0, 0));
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        // Determine state
        if is_merged {
            WorktreeState::Merged
        } else if is_rebasing {
            WorktreeState::Conflicted
        } else if !is_clean {
            WorktreeState::Developing
        } else if ahead > 0 && behind == 0 {
            WorktreeState::Ready
        } else if behind > 0 {
            WorktreeState::Resolving
        } else {
            WorktreeState::Created
        }
    }

    fn transition_state(&self, worktree_path: &str, branch_name: &str, current_state: &WorktreeState, target_state: &WorktreeState) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Transitioning {}: {} → {}", branch_name, current_state.to_string(), target_state.to_string()));
        
        match target_state {
            WorktreeState::Resolving => {
                let original_dir = std::env::current_dir()?;
                std::env::set_current_dir(worktree_path)?;
                
                let result = run_git_command(&["rebase", "main"]);
                std::env::set_current_dir(original_dir)?;
                
                match result {
                    Ok(_) => {
                        log_success("Rebase successful");
                        Ok(())
                    }
                    Err(_) => {
                        log_warning("Rebase failed - aborting");
                        let _ = run_git_command(&["rebase", "--abort"]);
                        Err("Rebase failed".into())
                    }
                }
            }
            WorktreeState::Ready => {
                let original_dir = std::env::current_dir()?;
                std::env::set_current_dir(worktree_path)?;
                
                let result = run_git_command(&["push", "origin", branch_name]);
                std::env::set_current_dir(original_dir)?;
                
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
                let original_dir = std::env::current_dir()?;
                std::env::set_current_dir(worktree_path)?;
                std::env::set_current_dir(original_dir)?;
                
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

    fn process_worktree(&mut self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));
        
        // Get current state
        let current_state = self.get_worktree_state(worktree_path, branch_name);
        log_info(&format!("Current state: {}", current_state.to_string()));
        
        // Determine next state
        let next_state = match current_state {
            WorktreeState::Created => Some(WorktreeState::Developing),
            WorktreeState::Developing => {
                // Check if ready to move to next state
                let original_dir = std::env::current_dir()?;
                std::env::set_current_dir(worktree_path)?;
                let status = get_git_status()?;
                std::env::set_current_dir(original_dir)?;
                
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
            if self.transition_state(worktree_path, branch_name, &current_state, &next_state).is_ok() {
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
        
        let worktrees = get_worktrees()?;
        
        if worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }
        
        let mut processed_count = 0;
        let mut success_count = 0;
        
        for worktree in worktrees {
            let worktree_path = worktree.path;
            let branch_name = worktree.branch;
            
            // Skip main worktree
            if branch_name == "hooksmith" {
                continue;
            }
            
            processed_count += 1;
            
            if self.process_worktree(&worktree_path, &branch_name).is_ok() {
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
    let command = args.get(1).unwrap_or(&"process".to_string());
    
    let mut state_machine = WorktreeStateMachine::new();
    
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
