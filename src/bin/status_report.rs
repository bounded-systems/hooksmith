use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, get_git_status, get_commit_counts,
    get_current_branch, is_rebasing, is_merged_into_main
};

#[derive(Debug, Clone, PartialEq)]
enum WorktreeState {
    Ready,
    Conflicted,
    Merged,
    Developing,
    Outdated,
    Unknown,
}

impl WorktreeState {
    fn to_string(&self) -> &'static str {
        match self {
            WorktreeState::Ready => "READY",
            WorktreeState::Conflicted => "CONFLICTED",
            WorktreeState::Merged => "MERGED",
            WorktreeState::Developing => "DEVELOPING",
            WorktreeState::Outdated => "OUTDATED",
            WorktreeState::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug)]
struct WorktreeStatus {
    current_branch: String,
    is_clean: bool,
    is_rebasing: bool,
    remote_exists: bool,
    is_merged: bool,
    ahead_count: i32,
    behind_count: i32,
    state: WorktreeState,
}

struct StatusReporter {
    worktrees: Vec<hooksmith::Worktree>,
}

impl StatusReporter {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees()?;
        Ok(Self { worktrees })
    }

    fn get_worktree_status(&self, worktree_path: &str, branch_name: &str) -> Result<WorktreeStatus, Box<dyn std::error::Error>> {
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Get current branch
        let current_branch = get_current_branch()?;

        // Get status
        let status = get_git_status()?;
        let is_clean = status.is_empty();

        // Check if rebasing
        let is_rebasing = is_rebasing()?;

        // Check if branch exists on origin
        let remote_exists = run_git_command(&["ls-remote", "--heads", "origin", &current_branch])
            .map(|output| !output.trim().is_empty())
            .unwrap_or(false);

        // Check if merged into main
        let is_merged = is_merged_into_main(&current_branch)?;

        // Get commit counts
        let (ahead_count, behind_count) = get_commit_counts("main")?;

        // Determine state
        let state = self.determine_state(is_clean, is_rebasing, remote_exists, is_merged, ahead_count, behind_count);

        std::env::set_current_dir(original_dir)?;

        Ok(WorktreeStatus {
            current_branch,
            is_clean,
            is_rebasing,
            remote_exists,
            is_merged,
            ahead_count,
            behind_count,
            state,
        })
    }

    fn determine_state(&self, is_clean: bool, is_rebasing: bool, remote_exists: bool, is_merged: bool, ahead_count: i32, behind_count: i32) -> WorktreeState {
        if is_merged {
            WorktreeState::Merged
        } else if is_rebasing {
            WorktreeState::Conflicted
        } else if !is_clean {
            WorktreeState::Developing
        } else if ahead_count > 0 && behind_count == 0 {
            WorktreeState::Ready
        } else if behind_count > 0 {
            WorktreeState::Outdated
        } else {
            WorktreeState::Unknown
        }
    }

    fn print_worktree_status(&self, worktree_path: &str, branch_name: &str, status: &WorktreeStatus) {
        println!("📁 Worktree: {}", worktree_path);
        println!("   Branch: {}", status.current_branch);
        println!("   State: {}", status.state.to_string());
        println!("   Status: {}", if status.is_clean { "clean" } else { "dirty" });
        println!("   Rebasing: {}", status.is_rebasing);
        println!("   Remote: {}", status.remote_exists);
        println!("   Merged: {}", status.is_merged);
        println!("   Commits: +{} -{}", status.ahead_count, status.behind_count);
        println!();
    }

    fn generate_pr_url(&self, branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let repo_url = run_git_command(&["config", "--get", "remote.origin.url"])?
            .replace(".git", "");

        if repo_url.contains("github.com") {
            Ok(format!("{}/compare/main...{}", repo_url, branch_name))
        } else {
            Ok("Unknown repository URL".to_string())
        }
    }

    fn generate_report(&self) -> Result<(), Box<dyn std::error::Error>> {
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
        for worktree in &self.worktrees {
            let worktree_path = &worktree.path;
            let branch_name = &worktree.branch;

            // Get status
            let status = self.get_worktree_status(worktree_path, branch_name)?;

            // Print status
            self.print_worktree_status(worktree_path, branch_name, &status);

            // Categorize worktree
            match status.state {
                WorktreeState::Ready => {
                    ready_worktrees.push(status.current_branch.clone());
                }
                WorktreeState::Conflicted => {
                    conflicted_worktrees.push(status.current_branch.clone());
                }
                WorktreeState::Merged => {
                    merged_worktrees.push(status.current_branch.clone());
                }
                WorktreeState::Developing => {
                    developing_worktrees.push(status.current_branch.clone());
                }
                _ => {}
            }
        }

        // Summary
        log_header("SUMMARY");
        println!();

        if !ready_worktrees.is_empty() {
            log_success(&format!("Ready for PR: {}", ready_worktrees.join(", ")));
            for branch in &ready_worktrees {
                let pr_url = self.generate_pr_url(branch)?;
                println!("   PR URL: {}", pr_url);
            }
            println!();
        }

        if !conflicted_worktrees.is_empty() {
            log_warning(&format!("Conflicted: {}", conflicted_worktrees.join(", ")));
            println!();
        }

        if !merged_worktrees.is_empty() {
            log_info(&format!("Merged (ready for cleanup): {}", merged_worktrees.join(", ")));
            println!();
        }

        if !developing_worktrees.is_empty() {
            log_info(&format!("Developing: {}", developing_worktrees.join(", ")));
            println!();
        }

        if ready_worktrees.is_empty() && conflicted_worktrees.is_empty() && merged_worktrees.is_empty() && developing_worktrees.is_empty() {
            log_info("No worktrees to process");
        }

        Ok(())
    }

    fn show_single_worktree(&self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        log_header(&format!("WORKTREE STATUS: {}", branch_name));
        println!();

        let status = self.get_worktree_status(worktree_path, &branch_name)?;
        self.print_worktree_status(worktree_path, &branch_name, &status);

        // Show additional details
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Show recent commits
        let recent_commits = run_git_command(&["log", "--oneline", "-5"])?;
        if !recent_commits.trim().is_empty() {
            println!("Recent commits:");
            for line in recent_commits.lines() {
                println!("  {}", line);
            }
            println!();
        }

        // Show uncommitted changes
        let status_output = get_git_status()?;
        if !status_output.trim().is_empty() {
            println!("Uncommitted changes:");
            for line in status_output.lines() {
                println!("  {}", line);
            }
            println!();
        }

        std::env::set_current_dir(original_dir)?;

        // Generate PR URL if ready
        if status.state == WorktreeState::Ready {
            let pr_url = self.generate_pr_url(&status.current_branch)?;
            log_info(&format!("PR URL: {}", pr_url));
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

        let mut state_counts: HashMap<WorktreeState, i32> = HashMap::new();

        for worktree in &self.worktrees {
            let worktree_path = &worktree.path;
            let branch_name = &worktree.branch;

            if let Ok(status) = self.get_worktree_status(worktree_path, branch_name) {
                *state_counts.entry(status.state.clone()).or_insert(0) += 1;
            }
        }

        let total = self.worktrees.len();
        log_info(&format!("Total worktrees: {}", total));

        for (state, count) in state_counts {
            let percentage = (count as f64 / total as f64 * 100.0) as i32;
            match state {
                WorktreeState::Ready => {
                    log_success(&format!("Ready: {} ({}%)", count, percentage));
                }
                WorktreeState::Conflicted => {
                    log_warning(&format!("Conflicted: {} ({}%)", count, percentage));
                }
                WorktreeState::Merged => {
                    log_info(&format!("Merged: {} ({}%)", count, percentage));
                }
                WorktreeState::Developing => {
                    log_info(&format!("Developing: {} ({}%)", count, percentage));
                }
                WorktreeState::Outdated => {
                    log_warning(&format!("Outdated: {} ({}%)", count, percentage));
                }
                WorktreeState::Unknown => {
                    log_warning(&format!("Unknown: {} ({}%)", count, percentage));
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("status_report");
    println!();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).unwrap_or(&"report".to_string());
    let target_path = args.get(2);

    let reporter = StatusReporter::new()?;

    match command.as_str() {
        "report" => {
            reporter.generate_report()?;
        }
        "summary" => {
            reporter.show_summary()?;
        }
        "worktree" => {
            if let Some(path) = target_path {
                reporter.show_single_worktree(path)?;
            } else {
                log_error("Please specify a worktree path for worktree command");
                std::process::exit(1);
            }
        }
        "url" => {
            if let Some(path) = target_path {
                let branch_name = Path::new(path)
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                    .to_string_lossy()
                    .to_string();
                let pr_url = reporter.generate_pr_url(&branch_name)?;
                log_info(&format!("PR URL for {}: {}", branch_name, pr_url));
            } else {
                log_error("Please specify a worktree path for url command");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            println!("Usage: cargo run --bin status_report [report|summary|worktree|url|help] [worktree_path]");
            println!("  report: Generate comprehensive worktree status report");
            println!("  summary: Show worktree summary with counts");
            println!("  worktree [path]: Show detailed status for specific worktree");
            println!("  url [path]: Generate PR URL for specific worktree");
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
