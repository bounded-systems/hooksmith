use hooksmith::{
    create_pr_with_gh, generate_pr_url, get_worktrees, is_ready_for_pr, log_error, log_header,
    log_info, log_success, log_warning, push_branch,
};
use std::path::Path;
use std::process::Command;

struct PrCreator {
    worktrees: Vec<String>,
}

impl PrCreator {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let worktrees = get_worktrees().map_err(|e| e.to_string())?;
        Ok(Self { worktrees })
    }

    fn is_ready_for_pr(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(is_ready_for_pr(worktree_path).map_err(|e| e.to_string())?)
    }

    fn push_branch(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(push_branch(worktree_path, branch_name).map_err(|e| e.to_string())?)
    }

    fn create_pr_with_gh(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(create_pr_with_gh(worktree_path, branch_name).map_err(|e| e.to_string())?)
    }

    fn generate_pr_url(&self, branch_name: &str) -> String {
        generate_pr_url(branch_name)
    }

    fn process_ready_worktree(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info(&format!(
            "Processing ready worktree: {} (branch: {})",
            worktree_path, branch_name
        ));

        // Push branch
        if self.push_branch(worktree_path, branch_name).is_ok() {
            // Try to create PR with GitHub CLI
            if self.create_pr_with_gh(worktree_path, branch_name).is_ok() {
                log_success(&format!("PR created successfully for {}", branch_name));
                return Ok(());
            }

            // Fallback: generate PR URL
            let pr_url = self.generate_pr_url(branch_name);
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

            // Check if ready for PR
            if self.is_ready_for_pr(worktree_path, &branch_name)? {
                ready_worktrees.push((worktree_path.clone(), branch_name));
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

        log_info(&format!(
            "Found {} worktree(s) ready for PR creation",
            ready_worktrees.len()
        ));
        println!();

        let mut processed_count = 0;

        for (worktree_path, branch_name) in ready_worktrees {
            if self
                .process_ready_worktree(&worktree_path, &branch_name)
                .is_ok()
            {
                processed_count += 1;
            }

            println!("---");
        }

        log_success(&format!("Processed {} worktree(s)", processed_count));
        Ok(())
    }

    fn process_single_worktree(
        &self,
        worktree_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = Path::new(worktree_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        log_info(&format!(
            "Processing single worktree: {} (branch: {})",
            worktree_path, branch_name
        ));

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

        log_info(&format!(
            "Found {} worktree(s) ready for PR creation:",
            ready_worktrees.len()
        ));
        println!();

        for (worktree_path, branch_name) in ready_worktrees {
            log_info(&format!("  {} (branch: {})", worktree_path, branch_name));

            // Show PR URL
            let pr_url = self.generate_pr_url(&branch_name);
            log_info(&format!("    PR URL: {}", pr_url));
            println!();
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let worktree_path = args.get(2);

    let pr_creator = PrCreator::new()?;

    match command {
        "all" => {
            pr_creator.process_all_ready_worktrees()?;
        }
        "single" => {
            if let Some(path) = worktree_path {
                pr_creator.process_single_worktree(path)?;
            } else {
                println!("Usage: {} single <worktree_path>", args[0]);
                std::process::exit(1);
            }
        }
        "status" => {
            pr_creator.show_ready_worktrees()?;
        }
        "check" => {
            // Check GitHub CLI availability
            let output = Command::new("gh").arg("--version").output();
            match output {
                Ok(_) => {
                    log_success("GitHub CLI is available");
                }
                Err(_) => {
                    log_warning(
                        "GitHub CLI is not available - will use fallback PR URL generation",
                    );
                }
            }
        }
        _ => {
            println!(
                "Usage: {} [all|single|status|check] [worktree_path]",
                args[0]
            );
            println!("  all: Process all ready worktrees");
            println!("  single <path>: Process single worktree");
            println!("  status: Show ready worktrees for PR creation");
            println!("  check: Check GitHub CLI availability");
            std::process::exit(1);
        }
    }

    Ok(())
}
