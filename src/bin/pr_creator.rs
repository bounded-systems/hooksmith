use std::process::Command;
use std::path::Path;
use hooksmith::{
    log_info, log_success, log_warning, log_error, log_header,
    get_worktrees, run_git_command, get_git_status, get_commit_counts
};

struct PrCreator {
    worktrees: Vec<hooksmith::Worktree>,
}

impl PrCreator {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees()?;
        Ok(Self { worktrees })
    }

    fn is_ready_for_pr(&self, worktree_path: &str, branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Check if clean
        let status = get_git_status()?;
        if !status.is_empty() {
            std::env::set_current_dir(original_dir)?;
            return Ok(false);
        }

        // Check if up to date with main
        let (ahead, behind) = get_commit_counts("main")?;
        if behind > 0 {
            std::env::set_current_dir(original_dir)?;
            return Ok(false);
        }

        // Check if has commits ahead of main
        if ahead == 0 {
            std::env::set_current_dir(original_dir)?;
            return Ok(false);
        }

        std::env::set_current_dir(original_dir)?;
        Ok(true)
    }

    fn push_branch(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Pushing branch {}", branch_name));

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        match run_git_command(&["push", "origin", branch_name]) {
            Ok(_) => {
                log_success("Branch pushed successfully");
                std::env::set_current_dir(original_dir)?;
                Ok(())
            }
            Err(_) => {
                log_warning("Push failed - branch may already be up to date");
                std::env::set_current_dir(original_dir)?;
                Err("Push failed".into())
            }
        }
    }

    fn create_pr_with_gh(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Creating PR for branch {} using GitHub CLI", branch_name));

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(worktree_path)?;

        // Get commit message for PR title
        let commit_msg = run_git_command(&["log", "--oneline", "-1"])?;
        let pr_title = commit_msg
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");

        // Get PR body from commit messages
        let pr_body = run_git_command(&["log", "--oneline", "main..HEAD"])?;
        let pr_body_lines: Vec<String> = pr_body
            .lines()
            .take(5)
            .map(|line| format!("- {}", line))
            .collect();
        let pr_body_text = pr_body_lines.join("\n");

        // Check if GitHub CLI is available
        let gh_available = Command::new("gh")
            .arg("--version")
            .output()
            .is_ok();

        if gh_available {
            let output = Command::new("gh")
                .args(&[
                    "pr", "create",
                    "--title", &pr_title,
                    "--body", &pr_body_text,
                    "--base", "main",
                    "--head", branch_name
                ])
                .output()?;

            if output.status.success() {
                log_success("PR created successfully");
                std::env::set_current_dir(original_dir)?;
                Ok(())
            } else {
                log_warning("Failed to create PR with GitHub CLI");
                std::env::set_current_dir(original_dir)?;
                Err("GitHub CLI PR creation failed".into())
            }
        } else {
            log_warning("GitHub CLI not available");
            std::env::set_current_dir(original_dir)?;
            Err("GitHub CLI not available".into())
        }
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

    fn process_ready_worktree(&self, worktree_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!("Processing ready worktree: {} (branch: {})", worktree_path, branch_name));

        // Push branch
        if self.push_branch(worktree_path, branch_name).is_ok() {
            // Try to create PR with GitHub CLI
            if self.create_pr_with_gh(worktree_path, branch_name).is_ok() {
                log_success(&format!("PR created successfully for {}", branch_name));
                return Ok(());
            }

            // Fallback: generate PR URL
            let pr_url = self.generate_pr_url(branch_name)?;
            log_info(&format!("PR URL generated: {}", pr_url));
            log_warning("Please create PR manually using the URL above");
            return Ok(());
        } else {
            log_error(&format!("Failed to push branch {}", branch_name));
            return Err("Failed to push branch".into());
        }
    }

    fn find_ready_worktrees(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut ready_worktrees = Vec::new();

        for worktree in &self.worktrees {
            let worktree_path = &worktree.path;
            let branch_name = &worktree.branch;

            // Skip main worktree
            if branch_name == "hooksmith" {
                continue;
            }

            // Check if ready for PR
            if self.is_ready_for_pr(worktree_path, branch_name)? {
                ready_worktrees.push((worktree_path.clone(), branch_name.clone()));
            }
        }

        Ok(ready_worktrees)
    }

    fn process_all_ready_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("CREATE WORKTREE PRs");
        println!();

        let ready_worktrees = self.find_ready_worktrees()?;

        if ready_worktrees.is_empty() {
            log_info("No worktrees ready for PR creation");
            return Ok(());
        }

        log_info(&format!("Found {} worktree(s) ready for PR creation", ready_worktrees.len()));
        println!();

        let mut processed_count = 0;

        for (worktree_path, branch_name) in ready_worktrees {
            if self.process_ready_worktree(&worktree_path, &branch_name).is_ok() {
                processed_count += 1;
            }

            println!("---");
        }

        log_success(&format!("Processed {} worktree(s)", processed_count));
        Ok(())
    }

    fn process_single_worktree(&self, worktree_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        log_info(&format!("Processing single worktree: {} (branch: {})", worktree_path, branch_name));

        // Check if ready for PR
        if !self.is_ready_for_pr(worktree_path, &branch_name)? {
            log_warning("Worktree is not ready for PR creation");
            return Ok(());
        }

        self.process_ready_worktree(worktree_path, &branch_name)?;
        log_success("Single worktree PR creation completed");
        Ok(())
    }

    fn show_ready_worktrees(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_header("READY WORKTREES FOR PR");
        println!();

        let ready_worktrees = self.find_ready_worktrees()?;

        if ready_worktrees.is_empty() {
            log_info("No worktrees ready for PR creation");
            return Ok(());
        }

        log_info(&format!("Found {} worktree(s) ready for PR creation:", ready_worktrees.len()));
        println!();

        for (worktree_path, branch_name) in ready_worktrees {
            log_info(&format!("  {} (branch: {})", worktree_path, branch_name));

            // Show commit info
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(&worktree_path)?;

            let (ahead, _) = get_commit_counts("main")?;
            log_info(&format!("    Commits ahead of main: {}", ahead));

            let commit_msg = run_git_command(&["log", "--oneline", "-1"])?;
            log_info(&format!("    Latest commit: {}", commit_msg.trim()));

            std::env::set_current_dir(original_dir)?;
            println!();
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("pr_creator");
    println!();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).unwrap_or(&"process".to_string());
    let target_path = args.get(2);

    let creator = PrCreator::new()?;

    match command.as_str() {
        "process" => {
            if let Some(path) = target_path {
                creator.process_single_worktree(path)?;
            } else {
                creator.process_all_ready_worktrees()?;
            }
        }
        "status" => {
            creator.show_ready_worktrees()?;
        }
        "push" => {
            if let Some(path) = target_path {
                let branch_name = Path::new(path)
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                    .to_string_lossy()
                    .to_string();
                log_info(&format!("Pushing branch {}", branch_name));
                creator.push_branch(path, &branch_name)?;
            } else {
                log_error("Please specify a worktree path for push command");
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
                let pr_url = creator.generate_pr_url(&branch_name)?;
                log_info(&format!("PR URL for {}: {}", branch_name, pr_url));
            } else {
                log_error("Please specify a worktree path for url command");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            println!("Usage: cargo run --bin pr_creator [process|status|push|url|help] [worktree_path]");
            println!("  process [path]: Process all ready worktrees or specific worktree for PR creation");
            println!("  status: Show worktrees ready for PR creation");
            println!("  push [path]: Push branch for specific worktree");
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
