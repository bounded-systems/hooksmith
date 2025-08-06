use hooksmith::{
    determine_state, generate_pr_url, get_worktree_status, get_worktrees, log_error, log_header,
    log_info, log_success, log_warning,
};
use std::path::Path;

struct StatusReport {
    worktrees: Vec<String>,
}

impl StatusReport {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees().map_err(|e| e.to_string())?;
        Ok(Self { worktrees })
    }

    fn get_worktree_status_info(
        &self,
        worktree_path: &str,
    ) -> Result<hooksmith::WorktreeStatus, Box<dyn std::error::Error>> {
        Ok(get_worktree_status(worktree_path).map_err(|e| e.to_string())?)
    }

    fn determine_worktree_state(
        &self,
        status: &hooksmith::WorktreeStatus,
    ) -> hooksmith::WorktreeState {
        determine_state(status)
    }

    fn print_worktree_status(
        &self,
        worktree_path: &str,
        branch_name: &str,
        status: &hooksmith::WorktreeStatus,
    ) {
        let state = self.determine_worktree_state(status);

        println!("📁 Worktree: {}", worktree_path);
        println!("   Branch: {}", status.current_branch);
        println!("   State: {:?}", state);
        println!(
            "   Status: {}",
            if status.is_clean { "clean" } else { "dirty" }
        );
        println!("   Rebasing: {}", status.is_rebasing);
        println!("   Remote: {}", status.remote_exists);
        println!("   Merged: {}", status.is_merged);
        println!(
            "   Commits: +{} -{}",
            status.ahead_behind, status.behind_ahead
        );
        println!();
    }

    fn generate_pr_url(&self, branch_name: &str) -> String {
        generate_pr_url(branch_name)
    }

    fn process_all_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("WORKTREE STATUS REPORT");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut ready_worktrees = Vec::new();
        let mut conflicted_worktrees = Vec::new();
        let mut merged_worktrees = Vec::new();
        let mut developing_worktrees = Vec::new();

        // Process each worktree
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

            // Get status
            match self.get_worktree_status_info(worktree_path) {
                Ok(status) => {
                    let state = self.determine_worktree_state(&status);

                    // Print status
                    self.print_worktree_status(worktree_path, &branch_name, &status);

                    // Categorize worktree
                    match state {
                        hooksmith::WorktreeState::Ready => {
                            ready_worktrees.push(status.current_branch.clone());
                        }
                        hooksmith::WorktreeState::Conflicted => {
                            conflicted_worktrees.push(status.current_branch.clone());
                        }
                        hooksmith::WorktreeState::Merged => {
                            merged_worktrees.push(status.current_branch.clone());
                        }
                        hooksmith::WorktreeState::Developing => {
                            developing_worktrees.push(status.current_branch.clone());
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    log_error(&format!(
                        "Failed to get status for {}: {}",
                        worktree_path, e
                    ));
                }
            }
        }

        // Summary
        log_header("SUMMARY");
        println!();

        if !ready_worktrees.is_empty() {
            log_success(&format!("Ready for PR: {}", ready_worktrees.join(", ")));
            for branch in &ready_worktrees {
                let pr_url = self.generate_pr_url(branch);
                println!("   PR URL: {}", pr_url);
            }
            println!();
        }

        if !conflicted_worktrees.is_empty() {
            log_warning(&format!("Conflicted: {}", conflicted_worktrees.join(", ")));
            println!();
        }

        if !merged_worktrees.is_empty() {
            log_info(&format!(
                "Merged (ready for cleanup): {}",
                merged_worktrees.join(", ")
            ));
            println!();
        }

        if !developing_worktrees.is_empty() {
            log_info(&format!("Developing: {}", developing_worktrees.join(", ")));
            println!();
        }

        if ready_worktrees.is_empty()
            && conflicted_worktrees.is_empty()
            && merged_worktrees.is_empty()
            && developing_worktrees.is_empty()
        {
            log_info("No worktrees to process");
        }

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

        log_header(&format!("WORKTREE STATUS: {}", worktree_path));
        println!();

        match self.get_worktree_status_info(worktree_path) {
            Ok(status) => {
                self.print_worktree_status(worktree_path, &branch_name, &status);

                let state = self.determine_worktree_state(&status);
                match state {
                    hooksmith::WorktreeState::Ready => {
                        log_success("Worktree is ready for PR creation");
                        let pr_url = self.generate_pr_url(&status.current_branch);
                        println!("PR URL: {}", pr_url);
                    }
                    hooksmith::WorktreeState::Conflicted => {
                        log_warning("Worktree has conflicts that need resolution");
                    }
                    hooksmith::WorktreeState::Merged => {
                        log_info("Worktree is merged and ready for cleanup");
                    }
                    hooksmith::WorktreeState::Developing => {
                        log_info("Worktree is in development");
                    }
                    hooksmith::WorktreeState::Outdated => {
                        log_warning("Worktree is outdated and needs updating");
                    }
                    hooksmith::WorktreeState::Unknown => {
                        log_warning("Worktree state is unknown");
                    }
                }
            }
            Err(e) => {
                log_error(&format!("Failed to get status: {}", e));
            }
        }

        Ok(())
    }

    fn show_summary(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("WORKTREE SUMMARY");
        println!();

        if self.worktrees.is_empty() {
            log_info("No worktrees found");
            return Ok(());
        }

        let mut ready_count = 0;
        let mut conflicted_count = 0;
        let mut merged_count = 0;
        let mut developing_count = 0;
        let mut total_count = 0;

        for worktree_path in &self.worktrees {
            let branch_name = std::path::Path::new(worktree_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if branch_name == "hooksmith" {
                continue;
            }

            total_count += 1;

            match self.get_worktree_status_info(worktree_path) {
                Ok(status) => {
                    let state = self.determine_worktree_state(&status);
                    match state {
                        hooksmith::WorktreeState::Ready => ready_count += 1,
                        hooksmith::WorktreeState::Conflicted => conflicted_count += 1,
                        hooksmith::WorktreeState::Merged => merged_count += 1,
                        hooksmith::WorktreeState::Developing => developing_count += 1,
                        _ => {}
                    }
                }
                Err(_) => {}
            }
        }

        log_info(&format!("Total worktrees: {}", total_count));
        log_success(&format!("Ready for PR: {}", ready_count));
        log_warning(&format!("Conflicted: {}", conflicted_count));
        log_info(&format!("Merged: {}", merged_count));
        log_info(&format!("Developing: {}", developing_count));

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let worktree_path = args.get(2);

    let status_report = StatusReport::new()?;

    match command {
        "all" => {
            status_report.process_all_worktrees()?;
        }
        "single" => {
            if let Some(path) = worktree_path {
                status_report.process_single_worktree(path)?;
            } else {
                println!("Usage: {} single <worktree_path>", args[0]);
                std::process::exit(1);
            }
        }
        "summary" => {
            status_report.show_summary()?;
        }
        _ => {
            println!("Usage: {} [all|single|summary] [worktree_path]", args[0]);
            println!("  all: Show status of all worktrees");
            println!("  single <path>: Show status of single worktree");
            println!("  summary: Show summary statistics");
            std::process::exit(1);
        }
    }

    Ok(())
}
