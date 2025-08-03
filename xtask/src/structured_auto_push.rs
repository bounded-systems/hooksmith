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
        self.logger.info("hooksmith", "git", "Adding changes")?;
        self.add_changes().await?;

        // Step 4: Commit changes
        self.logger.info("hooksmith", "git", "Committing changes")?;
        let commit_hash = self
            .commit_changes(&message, allow_empty_message, &args)
            .await?;

        // Step 5: Push changes
        self.logger.info("hooksmith", "git", "Pushing changes")?;
        let push_result = self.push_changes(force).await?;

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        // Emit completion event
        let completion_details = serde_json::json!({
            "duration_ms": duration.as_millis(),
            "commit_hash": commit_hash,
            "push_result": push_result,
            "session_id": self.session_id
        });

        let completion_event = StructuredEvent::new(
            "info",
            "hooksmith",
            "completion",
            "Auto-push completed successfully",
        )
        .with_details(completion_details);

        self.logger.log(completion_event)?;

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
            self.logger
                .info("cargo", name, &format!("Running cargo {}", name))?;

            let success = self.logger.run_cargo_command(name, &args).await?;

            if !success {
                self.logger
                    .error("cargo", name, &format!("cargo {} failed", name))?;
                anyhow::bail!("cargo {} failed", name);
            }
        }

        Ok(())
    }

    /// Check if there are any changes to commit
    async fn check_for_changes(&self) -> Result<bool> {
        let status = self.logger.git_status().await?;

        if let Some(porcelain) = status.get("porcelain") {
            if let Some(porcelain_str) = porcelain.as_str() {
                let has_changes = !porcelain_str.trim().is_empty();

                if has_changes {
                    self.logger
                        .info("git", "status", "Found changes to commit")?;

                    // Log each changed file
                    for line in porcelain_str.lines() {
                        if !line.trim().is_empty() {
                            let details = serde_json::json!({
                                "status_line": line,
                                "session_id": self.session_id
                            });

                            let event = StructuredEvent::new(
                                "info",
                                "git",
                                "status",
                                &format!("Changed: {}", line),
                            )
                            .with_details(details);
                            self.logger.log(event)?;
                        }
                    }
                }

                return Ok(has_changes);
            }
        }

        // Fallback: check if there are any changes
        let porcelain_output = self
            .logger
            .run_git_command("status", &["--porcelain".to_string()])
            .await?;
        let has_changes = !porcelain_output.trim().is_empty();

        if has_changes {
            self.logger
                .info("git", "status", "Found changes to commit")?;
        }

        Ok(has_changes)
    }

    /// Add all changes
    async fn add_changes(&self) -> Result<()> {
        let output = self
            .logger
            .run_git_command("add", &[".".to_string()])
            .await?;
        self.logger
            .info("git", "add", "Successfully added all changes")?;
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

        let output = self.logger.run_git_command("commit", &commit_args).await?;

        // Get commit hash
        let commit_hash = self
            .logger
            .run_git_command("rev-parse", &["HEAD".to_string()])
            .await?;

        let commit_details = serde_json::json!({
            "commit_hash": commit_hash.trim(),
            "commit_message": commit_message,
            "session_id": self.session_id
        });

        let commit_event = StructuredEvent::new(
            "info",
            "git",
            "commit",
            &format!("Committed changes: {}", commit_hash.trim()),
        )
        .with_details(commit_details);

        self.logger.log(commit_event)?;

        Ok(commit_hash.trim().to_string())
    }

    /// Push changes
    async fn push_changes(&self, force: bool) -> Result<String> {
        let mut push_args = vec!["push".to_string()];

        if force {
            push_args.push("--force".to_string());
        }

        let output = self.logger.run_git_command("push", &push_args).await?;

        let push_details = serde_json::json!({
            "force": force,
            "output": output,
            "session_id": self.session_id
        });

        let push_event = StructuredEvent::new("info", "git", "push", "Successfully pushed changes")
            .with_details(push_details);

        self.logger.log(push_event)?;

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
        self.logger.info(
            "hooksmith",
            "watchdog",
            &format!("Starting watchdog mode with {}s interval", interval),
        )?;

        loop {
            match self
                .run(message.clone(), allow_empty_message, force, args.clone())
                .await
            {
                Ok(_) => {
                    self.logger.info(
                        "hooksmith",
                        "watchdog",
                        "Watchdog cycle completed successfully",
                    )?;
                }
                Err(e) => {
                    self.logger.error(
                        "hooksmith",
                        "watchdog",
                        &format!("Watchdog cycle failed: {}", e),
                    )?;
                    self.logger.info(
                        "hooksmith",
                        "watchdog",
                        "Validation errors detected - skipping commit/push",
                    )?;
                }
            }

            self.logger.info(
                "hooksmith",
                "watchdog",
                &format!("Waiting {} seconds before next cycle", interval),
            )?;

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
