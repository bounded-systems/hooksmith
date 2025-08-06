use std::process::Command;
use std::path::Path;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, run_git_command_in_dir, get_worktree_status, determine_state
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LifecycleStage {
    Created,
    Developing,
    Conflicted,
    Resolving,
    Ready,
    PrCreated,
    Merged,
    Cleanup,
    Removed,
}

impl LifecycleStage {
    fn to_string(&self) -> &'static str {
        match self {
            LifecycleStage::Created => "CREATED",
            LifecycleStage::Developing => "DEVELOPING",
            LifecycleStage::Conflicted => "CONFLICTED",
            LifecycleStage::Resolving => "RESOLVING",
            LifecycleStage::Ready => "READY",
            LifecycleStage::PrCreated => "PR_CREATED",
            LifecycleStage::Merged => "MERGED",
            LifecycleStage::Cleanup => "CLEANUP",
            LifecycleStage::Removed => "REMOVED",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            LifecycleStage::Created => "Worktree created",
            LifecycleStage::Developing => "Worktree has uncommitted changes",
            LifecycleStage::Conflicted => "Worktree has rebase conflicts",
            LifecycleStage::Resolving => "Resolving conflicts",
            LifecycleStage::Ready => "Worktree ready for PR",
            LifecycleStage::PrCreated => "PR created",
            LifecycleStage::Merged => "PR merged",
            LifecycleStage::Cleanup => "Cleaning up worktree",
            LifecycleStage::Removed => "Worktree removed",
        }
    }
}

struct WorktreeLifecycle {
    worktrees: Vec<String>,
}

impl WorktreeLifecycle {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees().map_err(|e| e.to_string())?;
        Ok(Self { worktrees })
    }

    fn get_worktree_stage(&self, worktree_path: &str) -> Result<LifecycleStage, Box<dyn std::error::Error>> {
        // Get worktree status using the library function
        let status = get_worktree_status(worktree_path).map_err(|e| e.to_string())?;

        // Map the library's WorktreeState to our internal LifecycleStage
        match determine_state(&status) {
            hooksmith::WorktreeState::Merged => Ok(LifecycleStage::Merged),
            hooksmith::WorktreeState::Conflicted => Ok(LifecycleStage::Conflicted),
            hooksmith::WorktreeState::Developing => Ok(LifecycleStage::Developing),
            hooksmith::WorktreeState::Ready => Ok(LifecycleStage::Ready),
            hooksmith::WorktreeState::Outdated => Ok(LifecycleStage::Resolving),
            hooksmith::WorktreeState::Unknown => Ok(LifecycleStage::Created),
        }
    }

    fn show_lifecycle_diagram(&self) {
        log_header("WORKTREE LIFECYCLE DIAGRAM");
        println!();
        println!("CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED");
        println!("    ↓         ↓");
        println!("CONFLICTED → RESOLVING");
        println!();
        println!("Stage Descriptions:");
        for stage in [
            LifecycleStage::Created,
            LifecycleStage::Developing,
            LifecycleStage::Conflicted,
            LifecycleStage::Resolving,
            LifecycleStage::Ready,
            LifecycleStage::PrCreated,
            LifecycleStage::Merged,
            LifecycleStage::Cleanup,
            LifecycleStage::Removed,
        ] {
            println!("  {}: {}", stage.to_string(), stage.description());
        }
    }

    fn show_worktree_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("WORKTREE LIFECYCLE STATUS");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut stage_counts: std::collections::HashMap<LifecycleStage, Vec<String>> = std::collections::HashMap::new();

        for worktree_path in &self.worktrees {
            let branch_name = Path::new(worktree_path)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                .to_string_lossy()
                .to_string();

            // Skip main worktree
            if branch_name == "hooksmith" {
                continue;
            }

            let stage = self.get_worktree_stage(worktree_path)?;
            stage_counts.entry(stage.clone()).or_insert_with(Vec::new).push(branch_name.clone());

            println!("📁 {} (branch: {}) - Stage: {}", worktree_path, branch_name, stage.to_string());
        }

        println!();
        log_header("SUMMARY BY STAGE");
        println!();

        for (stage, branches) in stage_counts {
            match stage {
                LifecycleStage::Ready => {
                    log_success(&format!("Ready ({}): {}", branches.len(), branches.join(", ")));
                }
                LifecycleStage::Conflicted => {
                    log_warning(&format!("Conflicted ({}): {}", branches.len(), branches.join(", ")));
                }
                LifecycleStage::Merged => {
                    log_info(&format!("Merged ({}): {}", branches.len(), branches.join(", ")));
                }
                LifecycleStage::Developing => {
                    log_info(&format!("Developing ({}): {}", branches.len(), branches.join(", ")));
                }
                LifecycleStage::Resolving => {
                    log_warning(&format!("Resolving ({}): {}", branches.len(), branches.join(", ")));
                }
                LifecycleStage::Created => {
                    log_info(&format!("Created ({}): {}", branches.len(), branches.join(", ")));
                }
                _ => {
                    log_info(&format!("{} ({}): {}", stage.to_string(), branches.len(), branches.join(", ")));
                }
            }
        }

        Ok(())
    }

    fn process_worktree_lifecycle(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Processing worktree lifecycle: {} (branch: {})", worktree_path, branch_name));

        let current_stage = self.get_worktree_stage(worktree_path)?;
        log_info(&format!("Current stage: {}", current_stage.to_string()));

        match current_stage {
            LifecycleStage::Conflicted => {
                log_info("Resolving conflicts...");
                let _ = Command::new("cargo")
                    .args(&["run", "--bin", "conflict_resolver", "process", worktree_path])
                    .output();
            }
            LifecycleStage::Ready => {
                log_info("Creating PR...");
                let _ = Command::new("cargo")
                    .args(&["run", "--bin", "pr_creator", "process", worktree_path])
                    .output();
            }
            LifecycleStage::Merged => {
                log_info("Cleaning up merged worktree...");
                let _ = Command::new("cargo")
                    .args(&["run", "--bin", "safe-worktree-cleanup", worktree_path])
                    .output();
            }
            _ => {
                log_info(&format!("No action needed for stage: {}", current_stage.to_string()));
            }
        }

        Ok(())
    }

    fn process_all_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("PROCESSING ALL WORKTREE LIFECYCLES");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut processed_count = 0;
        let mut success_count = 0;

        let worktree_paths: Vec<String> = self.worktrees.clone();
        for worktree_path in &worktree_paths {
            let branch_name = Path::new(worktree_path)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                .to_string_lossy()
                .to_string();

            // Skip main worktree
            if branch_name == "hooksmith" {
                continue;
            }

            processed_count += 1;

            if self.process_worktree_lifecycle(worktree_path, &branch_name).is_ok() {
                success_count += 1;
            }

            println!("---");
        }

        log_success(&format!("Processed {} worktree(s), {} successful", processed_count, success_count));
        Ok(())
    }

    fn run_state_machine(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Running worktree state machine...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "state_machine", "process"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            log_success("State machine completed successfully");
        } else {
            log_error("State machine failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn run_status_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Running worktree status report...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "status_report", "report"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            log_success("Status report completed successfully");
        } else {
            log_error("Status report failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn run_conflict_resolution(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Running conflict resolution...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "conflict_resolver", "process"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            log_success("Conflict resolution completed successfully");
        } else {
            log_error("Conflict resolution failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn run_pr_creation(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Running PR creation...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "pr_creator", "process"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            log_success("PR creation completed successfully");
        } else {
            log_error("PR creation failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn run_cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_info("Running worktree cleanup...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "safe-worktree-cleanup"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            log_success("Cleanup completed successfully");
        } else {
            log_error("Cleanup failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn run_complete_workflow(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("RUNNING COMPLETE WORKTREE LIFECYCLE WORKFLOW");
        println!();

        // Step 1: Status Report
        log_info("Step 1: Generating status report");
        self.run_status_report()?;
        println!();

        // Step 2: Conflict Resolution
        log_info("Step 2: Resolving conflicts");
        self.run_conflict_resolution()?;
        println!();

        // Step 3: State Machine
        log_info("Step 3: Running state machine");
        self.run_state_machine()?;
        println!();

        // Step 4: PR Creation
        log_info("Step 4: Creating PRs");
        self.run_pr_creation()?;
        println!();

        // Step 5: Cleanup
        log_info("Step 5: Cleaning up");
        self.run_cleanup()?;
        println!();

        // Step 6: Final Status Report
        log_info("Step 6: Final status report");
        self.run_status_report()?;

        log_success("Complete workflow finished successfully");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("worktree-lifecycle");
    println!();

    let args: Vec<String> = std::env::args().collect();
    let default_command = "status".to_string();
    let command = args.get(1).unwrap_or(&default_command);
    let target_path = args.get(2);

    let lifecycle = WorktreeLifecycle::new()?;

    match command.as_str() {
        "status" => {
            lifecycle.show_worktree_status()?;
        }
        "diagram" => {
            lifecycle.show_lifecycle_diagram();
        }
        "process" => {
            if let Some(path) = target_path {
                let branch_name = Path::new(path)
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                    .to_string_lossy()
                    .to_string();
                lifecycle.process_worktree_lifecycle(path, &branch_name)?;
            } else {
                lifecycle.process_all_worktrees()?;
            }
        }
        "workflow" => {
            lifecycle.run_complete_workflow()?;
        }
        "state-machine" => {
            lifecycle.run_state_machine()?;
        }
        "status-report" => {
            lifecycle.run_status_report()?;
        }
        "conflict-resolution" => {
            lifecycle.run_conflict_resolution()?;
        }
        "pr-creation" => {
            lifecycle.run_pr_creation()?;
        }
        "cleanup" => {
            lifecycle.run_cleanup()?;
        }
        "help" | "--help" | "-h" => {
            println!("Usage: cargo run --bin worktree-lifecycle [status|diagram|process|workflow|state-machine|status-report|conflict-resolution|pr-creation|cleanup|help] [worktree_path]");
            println!("  status: Show worktree lifecycle status");
            println!("  diagram: Show lifecycle diagram");
            println!("  process [path]: Process worktree lifecycle (all or specific)");
            println!("  workflow: Run complete workflow (status → conflicts → state → PRs → cleanup)");
            println!("  state-machine: Run worktree state machine");
            println!("  status-report: Run worktree status report");
            println!("  conflict-resolution: Run conflict resolution");
            println!("  pr-creation: Run PR creation");
            println!("  cleanup: Run worktree cleanup");
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
