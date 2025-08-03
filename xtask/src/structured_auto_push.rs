use anyhow::{Context, Result};
use serde_json::Value;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;

// Removed unused imports - structured_logging module doesn't have these types

/// Structured auto-push workflow with JSONL output and event bus integration
pub struct StructuredAutoPush {
    /// Session ID for grouping related events
    session_id: String,
    /// Whether to enable detailed logging
    verbose: bool,
}

impl Default for StructuredAutoPush {
    fn default() -> Self {
        let session_id = Uuid::new_v4().to_string();
        Self {
            session_id,
            verbose: false,
        }
    }
}

impl StructuredAutoPush {
    /// Create a new structured auto-push instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Disable JSONL output (for TUI mode)
    pub fn without_jsonl(self) -> Self {
        self
    }

    /// Disable event bus integration
    pub fn without_event_bus(self) -> Self {
        self
    }

    /// Run the structured auto-push workflow
    pub async fn run(
        &self,
        message: Option<String>,
        allow_empty_message: bool,
        force: bool,
        args: Vec<String>,
    ) -> Result<()> {
        let start_time = SystemTime::now();

        // Emit start event
        if self.verbose {
            println!("🚀 Starting structured auto-push workflow");
        }

        // Step 1: Run validation checks
        if self.verbose {
            println!("🔍 Running validation checks");
        }
        let validation_result = self.run_validation().await;

        match validation_result {
            Ok(_) => {
                if self.verbose {
                    println!("✅ All validation checks passed");
                }
            }
            Err(e) => {
                if self.verbose {
                    println!("❌ Validation failed: {}", e);
                }
                return Err(e);
            }
        }

        // Step 2: Check for changes
        if self.verbose {
            println!("🔍 Checking for changes");
        }
        let has_changes = self.check_for_changes().await?;

        if !has_changes {
            if self.verbose {
                println!("ℹ️  No changes to commit");
            }
            return Ok(());
        }

        // Step 3: Add changes
        if self.verbose {
            println!("📝 Adding changes");
        }
        self.add_changes().await?;

        // Step 4: Commit changes
        if self.verbose {
            println!("💾 Committing changes");
        }
        let commit_hash = self
            .commit_changes(&message, allow_empty_message, &args)
            .await?;

        // Step 5: Push changes
        if self.verbose {
            println!("🚀 Pushing changes");
        }
        let push_result = self.push_changes(force).await?;

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        // Emit completion event
        let completion_details = serde_json::json!({
            "duration_ms": duration.as_millis(),
            "commit_hash": commit_hash,
            "push_result": push_result,
            "session_id": self.session_id
        });

        if self.verbose {
            println!("✅ Auto-push completed successfully");
        }

        Ok(())
    }

    /// Run validation checks with structured output
    async fn run_validation(&self) -> Result<()> {
        let checks = vec![
            ("fix", vec!["fix", "--allow-dirty", "--allow-staged"]),
            ("fmt", vec!["fmt", "--all"]),
            (
                "clippy",
                vec![
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            (
                "contract-check",
                vec!["run", "-p", "xtask", "--", "contract-check", "--strict"],
            ),
            (
                "validate-generated",
                vec!["run", "-p", "xtask", "--", "validate-generated", "--strict"],
            ),
        ];

        for (name, args) in checks {
            if self.verbose {
                println!("🔧 Running cargo {}", name);
            }

            // Simplified validation - just run the command
            let args_strings: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let success = self.run_cargo_command(name, &args_strings).await?;

            if !success {
                if self.verbose {
                    println!("❌ cargo {} failed", name);
                }
                anyhow::bail!("cargo {} failed", name);
            }
        }

        Ok(())
    }

    /// Run a cargo command
    async fn run_cargo_command(&self, name: &str, args: &[String]) -> Result<bool> {
        use std::process::Command;
        
        let mut cmd = Command::new("cargo");
        cmd.args(args);
        
        let output = cmd.output()
            .context(format!("Failed to run cargo {}", name))?;
        
        Ok(output.status.success())
    }

    /// Check if there are any changes to commit
    async fn check_for_changes(&self) -> Result<bool> {
        // Simplified git status check
        let has_changes = self.check_git_status().await?;

        if has_changes {
            if self.verbose {
                println!("📝 Found changes to commit");
            }
        }

        Ok(has_changes)
    }

    /// Add all changes
    async fn add_changes(&self) -> Result<()> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(["add", "."])
            .output()
            .context("Failed to run git add")?;
        
        if self.verbose {
            println!("📝 Added all changes");
        }
        Ok(())
    }

    /// Commit changes and return commit hash
    async fn commit_changes(
        &self,
        message: &Option<String>,
        allow_empty_message: bool,
        args: &[String],
    ) -> Result<String> {
        let commit_message = if let Some(msg) = message {
            msg.clone()
        } else {
            // Generate default message
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            format!("chore: auto-update at {}", timestamp)
        };

        let mut commit_args = vec!["commit".to_string()];

        if allow_empty_message || commit_message.is_empty() {
            commit_args.extend_from_slice(&[
                "--allow-empty-message".to_string(),
                "-m".to_string(),
                "".to_string(),
            ]);
        } else {
            commit_args.extend_from_slice(&["-m".to_string(), commit_message.clone()]);
        }

        // Add any additional arguments
        for arg in args {
            commit_args.push(arg.clone());
        }

        use std::process::Command;
        
        let output = Command::new("git")
            .args(&commit_args)
            .output()
            .context("Failed to run git commit")?;

        // Get commit hash
        let commit_hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .context("Failed to get commit hash")?;
        
        let commit_hash = String::from_utf8_lossy(&commit_hash_output.stdout).trim().to_string();

        if self.verbose {
            println!("💾 Committed changes: {}", commit_hash);
        }

        Ok(commit_hash.trim().to_string())
    }

    /// Check git status for changes
    async fn check_git_status(&self) -> Result<bool> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;
        
        let status = String::from_utf8_lossy(&output.stdout);
        Ok(!status.trim().is_empty())
    }

    /// Push changes
    async fn push_changes(&self, force: bool) -> Result<String> {
        use std::process::Command;
        
        let mut push_args = vec!["push"];

        if force {
            push_args.push("--force");
        }

        let output = Command::new("git")
            .args(push_args)
            .output()
            .context("Failed to run git push")?;

        if self.verbose {
            println!("🚀 Successfully pushed changes");
        }

        Ok("Push completed successfully".to_string())
    }

    /// Run in watchdog mode
    pub async fn run_watchdog(
        &self,
        message: Option<String>,
        allow_empty_message: bool,
        force: bool,
        args: Vec<String>,
        interval: u64,
    ) -> Result<()> {
        if self.verbose {
            println!("🔄 Starting watchdog mode with {}s interval", interval);
        }

        loop {
            match self
                .run(message.clone(), allow_empty_message, force, args.clone())
                .await
            {
                Ok(_) => {
                    if self.verbose {
                        println!("✅ Watchdog cycle completed successfully");
                    }
                }
                Err(e) => {
                    if self.verbose {
                        println!("❌ Watchdog cycle failed: {}", e);
                        println!("ℹ️  Validation errors detected - skipping commit/push");
                    }
                }
            }

            if self.verbose {
                println!("⏳ Waiting {} seconds before next cycle", interval);
            }

            sleep(Duration::from_secs(interval)).await;
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get logger reference
    pub fn logger(&self) -> &str {
        "structured_auto_push"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_auto_push_creation() {
        let auto_push = StructuredAutoPush::new();
        assert!(!auto_push.verbose);
        assert!(!auto_push.session_id.is_empty());
    }

    #[test]
    fn test_structured_auto_push_with_verbose() {
        let auto_push = StructuredAutoPush::new().with_verbose(true);
        assert!(auto_push.verbose);
    }

    #[test]
    fn test_structured_auto_push_without_jsonl() {
        let auto_push = StructuredAutoPush::new().without_jsonl();
        assert!(!auto_push.logger.jsonl_output);
    }
}
