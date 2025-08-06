use std::process::Command;
use std::path::Path;
use std::env;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header, get_worktrees, run_git_command_in_dir, get_worktree_status, determine_state, WorktreeState};

#[derive(Debug, Clone, PartialEq)]
enum State {
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

impl State {
    fn to_string(&self) -> &'static str {
        match self {
            State::Created => "CREATED",
            State::Developing => "DEVELOPING",
            State::Conflicted => "CONFLICTED",
            State::Resolving => "RESOLVING",
            State::Resolved => "RESOLVED",
            State::Ready => "READY",
            State::PrCreated => "PR_CREATED",
            State::Merged => "MERGED",
            State::Cleanup => "CLEANUP",
            State::Removed => "REMOVED",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            State::Created => "Worktree created",
            State::Developing => "Worktree has uncommitted changes",
            State::Conflicted => "Worktree has rebase conflicts",
            State::Resolving => "Resolving conflicts",
            State::Resolved => "Conflicts resolved",
            State::Ready => "Worktree ready for PR",
            State::PrCreated => "PR created",
            State::Merged => "PR merged",
            State::Cleanup => "Cleaning up worktree",
            State::Removed => "Worktree removed",
        }
    }
}

fn get_worktree_state(worktree_path: &str, branch_name: &str) -> Result<State, String> {
    // Get worktree status using shared library
    let status = get_worktree_status(worktree_path)?;
    let state = determine_state(&status);

    // Convert to our state machine states
    match state {
        WorktreeState::Merged => Ok(State::Merged),
        WorktreeState::Conflicted => Ok(State::Conflicted),
        WorktreeState::Developing => Ok(State::Developing),
        WorktreeState::Ready => Ok(State::Ready),
        WorktreeState::Outdated => Ok(State::Resolving),
        WorktreeState::Unknown => Ok(State::Created),
    }
}

fn transition_state(worktree_path: &str, branch_name: &str, current_state: &State, target_state: &State) -> Result<bool, String> {
    log_info(&format!("Transitioning {}: {} → {}", branch_name, current_state.to_string(), target_state.to_string()));

    match target_state {
        State::Resolving => {
            let output = Command::new("git")
                .args(&["rebase", "main"])
                .current_dir(worktree_path)
                .output()
                .map_err(|e| format!("Failed to rebase: {}", e))?;

            if output.status.success() {
                log_success("Rebase successful");
                Ok(true)
            } else {
                log_warning("Rebase failed - aborting");
                let _ = Command::new("git")
                    .args(&["rebase", "--abort"])
                    .current_dir(worktree_path)
                    .output();
                Ok(false)
            }
        }
        State::Ready => {
            let output = Command::new("git")
                .args(&["push", "origin", branch_name])
                .current_dir(worktree_path)
                .output()
                .map_err(|e| format!("Failed to push: {}", e))?;

            if output.status.success() {
                log_success("Branch pushed successfully");
                Ok(true)
            } else {
                log_warning("Push failed");
                Ok(false)
            }
        }
        State::PrCreated => {
            // Generate PR URL
            let pr_url = format!("git@github.com:bdelanghe/hooksmith/compare/main...{}", branch_name);
            log_info(&format!("PR URL: {}", pr_url));
            Ok(true)
        }
        State::Cleanup => {
            // Remove worktree
            let output = Command::new("git")
                .args(&["worktree", "remove", "--force", worktree_path])
                .output()
                .map_err(|e| format!("Failed to remove worktree: {}", e))?;

            if output.status.success() {
                // Delete branch from origin
                let _ = Command::new("git")
                    .args(&["push", "origin", "--delete", branch_name])
                    .output();
                log_success("Worktree cleaned up");
                Ok(true)
            } else {
                log_error("Failed to clean up worktree");
                Ok(false)
            }
        }
        _ => {
            log_warning(&format!("Unknown target state: {}", target_state.to_string()));
            Ok(false)
        }
    }
}

fn get_next_state(current_state: &State) -> Option<State> {
    match current_state {
        State::Created => Some(State::Developing),
        State::Developing => Some(State::Resolving),
        State::Conflicted => Some(State::Resolving),
        State::Resolving => Some(State::Ready),
        State::Resolved => Some(State::Ready),
        State::Ready => Some(State::PrCreated),
        State::PrCreated => Some(State::Merged),
        State::Merged => Some(State::Cleanup),
        State::Cleanup => Some(State::Removed),
        State::Removed => None,
    }
}

fn process_worktree(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Processing worktree: {} (branch: {})", worktree_path, branch_name));

    // Get current state
    let current_state = get_worktree_state(worktree_path, branch_name)?;
    log_info(&format!("Current state: {}", current_state.to_string()));

    // Determine next state
    if let Some(next_state) = get_next_state(&current_state) {
        if transition_state(worktree_path, branch_name, &current_state, &next_state)? {
            log_success(&format!("Successfully transitioned to {}", next_state.to_string()));
            Ok(true)
        } else {
            log_warning(&format!("Failed to transition to {}", next_state.to_string()));
            Ok(false)
        }
    } else {
        log_info("No transition needed");
        Ok(true)
    }
}

fn print_diagram() {
    log_header("WORKTREE STATE MACHINE DIAGRAM");
    println!();
    println!("CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED");
    println!("    ↓         ↓");
    println!("CONFLICTED → RESOLVING");
    println!();
    println!("State Descriptions:");
    println!("  CREATED: {}", State::Created.description());
    println!("  DEVELOPING: {}", State::Developing.description());
    println!("  CONFLICTED: {}", State::Conflicted.description());
    println!("  RESOLVING: {}", State::Resolving.description());
    println!("  RESOLVED: {}", State::Resolved.description());
    println!("  READY: {}", State::Ready.description());
    println!("  PR_CREATED: {}", State::PrCreated.description());
    println!("  MERGED: {}", State::Merged.description());
    println!("  CLEANUP: {}", State::Cleanup.description());
    println!("  REMOVED: {}", State::Removed.description());
}

fn process_all_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    log_header("PROCESSING ALL WORKTREES");
    println!();

    let worktrees = get_worktrees()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    let mut processed_count = 0;
    let mut success_count = 0;

    // Process each worktree
    for worktree_path in &worktrees {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        // Skip main worktree
        if branch_name == "hooksmith" {
            continue;
        }

        processed_count += 1;

        if process_worktree(worktree_path, &branch_name)? {
            success_count += 1;
        }

        println!("---");
    }

    log_success(&format!("Processed {} worktree(s), {} successful", processed_count, success_count));

    Ok(())
}

fn show_usage() {
    println!("Usage: worktree-state-machine [diagram|process|status]");
    println!("  diagram: Show state machine diagram");
    println!("  process: Process all worktrees through state machine");
    println!("  status: Show current worktree status");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("process");

    match command {
        "diagram" => {
            print_diagram();
        }
        "process" => {
            process_all_worktrees()?;
        }
        "status" => {
            // Use our worktree-status-report binary
            let output = Command::new("cargo")
                .args(&["run", "--bin", "worktree-status-report"])
                .output()
                .map_err(|e| format!("Failed to run status report: {}", e))?;

            if output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                log_error("Failed to get worktree status");
            }
        }
        _ => {
            show_usage();
        }
    }

    Ok(())
}
