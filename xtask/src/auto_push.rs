use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

/// Clean auto-push workflow with porcelain output and comprehensive logging
pub struct CleanAutoPush {
    /// Whether to enable detailed logging
    pub verbose: bool,
    /// Whether to log to file
    pub log_to_file: bool,
    /// Log file path
    pub log_file: Option<String>,
}

impl Default for CleanAutoPush {
    fn default() -> Self {
        Self {
            verbose: false,
            log_to_file: true,
            log_file: Some(".hooksmith/logs/auto-push.log".to_string()),
        }
    }
}

impl CleanAutoPush {
    /// Run a clean auto-push workflow
    pub async fn run(
        &self,
        message: Option<String>,
        allow_empty_message: bool,
        skip_validation: bool,
        force: bool,
        args: Vec<String>,
    ) -> Result<()> {
        let start_time = SystemTime::now();

        // Ensure log directory exists
        if self.log_to_file {
            if let Some(log_path) = &self.log_file {
                if let Some(parent) = std::path::Path::new(log_path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }

        self.log("🚀 Starting clean auto-push workflow...");

        // Step 1: Run validation checks (unless skipped)
        if !skip_validation {
            self.log("🔍 Running validation checks...");
            self.run_validation().await?;
            self.log("✅ All validation checks passed!");
        } else {
            self.log("⚠️  Skipping validation checks");
        }

        // Step 2: Check for changes
        self.log("📊 Checking for changes...");
        let has_changes = self.check_for_changes().await?;

        if !has_changes {
            self.log("✅ No changes to commit");
            return Ok(());
        }

        // Step 3: Add changes
        self.log("📦 Adding changes...");
        self.add_changes().await?;

        // Step 4: Commit changes
        self.log("💾 Committing changes...");
        let commit_hash = self
            .commit_changes(&message, allow_empty_message, &args)
            .await?;

        // Step 5: Push changes
        self.log("🚀 Pushing changes...");
        let push_result = self.push_changes(force).await?;

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        // Print clean summary
        println!("📦 Auto-push completed successfully!");
        println!("   ⏱️  Duration: {:?}", duration);
        println!("   📝 Commit: {}", commit_hash);
        println!("   🚀 {}", push_result);

        self.log(&format!("✅ Auto-push completed in {:?}", duration));
        Ok(())
    }

    /// Run validation checks
    async fn run_validation(&self) -> Result<()> {
        let checks = vec![
            ("cargo fix", vec!["fix", "--allow-dirty", "--allow-staged"]),
            ("cargo fmt", vec!["fmt", "--all"]),
            (
                "cargo clippy",
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
                "contract validation",
                vec!["run", "-p", "xtask", "--", "contract-check", "--strict"],
            ),
            (
                "generated file validation",
                vec!["run", "-p", "xtask", "--", "validate-generated", "--strict"],
            ),
        ];

        for (name, args) in checks {
            self.log(&format!("   🔧 Running {}...", name));

            let output = Command::new("cargo")
                .args(args)
                .output()
                .context(format!("Failed to run {}", name))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                self.log(&format!("❌ {} failed: {}", name, error));
                anyhow::bail!("{} failed", name);
            }
        }

        Ok(())
    }

    /// Check if there are any changes to commit
    async fn check_for_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to check git status")?;

        let status = String::from_utf8_lossy(&output.stdout);
        let has_changes = !status.trim().is_empty();

        if has_changes {
            self.log("📝 Found changes to commit:");
            for line in status.lines() {
                if !line.trim().is_empty() {
                    self.log(&format!("   {}", line));
                }
            }
        }

        Ok(has_changes)
    }

    /// Add all changes
    async fn add_changes(&self) -> Result<()> {
        let status = Command::new("git")
            .args(["add", "."])
            .status()
            .context("Failed to add changes")?;

        if !status.success() {
            anyhow::bail!("git add failed");
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

        let mut commit_args = vec!["commit"];

        if allow_empty_message || commit_message.is_empty() {
            commit_args.extend_from_slice(&["--allow-empty-message", "-m", ""]);
        } else {
            commit_args.extend_from_slice(&["-m", &commit_message]);
        }

        // Add any additional arguments
        for arg in args {
            commit_args.push(arg);
        }

        let status = Command::new("git")
            .args(&commit_args)
            .status()
            .context("Failed to commit changes")?;

        if !status.success() {
            anyhow::bail!("git commit failed");
        }

        // Get commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .context("Failed to get commit hash")?;

        let hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();

        self.log(&format!(
            "   📝 Committed with message: {}",
            if commit_message.is_empty() {
                "(empty)"
            } else {
                &commit_message
            }
        ));

        Ok(hash)
    }

    /// Push changes with porcelain output
    async fn push_changes(&self, force: bool) -> Result<String> {
        let mut push_args = vec!["push", "--porcelain"];

        if force {
            push_args.push("--force");
            self.log("⚠️  Force pushing (use with caution!)");
        }

        let output = Command::new("git")
            .args(&push_args)
            .output()
            .context("Failed to push changes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Parse porcelain output for cleaner error messages
            let error_message = if !stdout.is_empty() {
                // Parse porcelain format: <ref> <status> <summary>
                let lines: Vec<&str> = stdout.lines().collect();
                if let Some(first_line) = lines.first() {
                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        match parts[1] {
                            "rejected" => {
                                "Push rejected (non-fast-forward, requires pull/rebase)".to_string()
                            }
                            "up to date" => "Already up to date".to_string(),
                            "forced update" => "Force update required".to_string(),
                            _ => format!("Push failed: {}", parts[1]),
                        }
                    } else {
                        "Push failed: unknown error".to_string()
                    }
                } else {
                    "Push failed: no output".to_string()
                }
            } else if !stderr.is_empty() {
                // Fallback to stderr if no porcelain output
                stderr.trim().to_string()
            } else {
                "Push failed: no error details available".to_string()
            };

            self.log(&format!("❌ Push failed: {}", error_message));
            anyhow::bail!("git push failed: {}", error_message);
        }

        // Parse successful porcelain output for clean status message
        let stdout = String::from_utf8_lossy(&output.stdout);
        let push_status = if !stdout.is_empty() {
            let lines: Vec<&str> = stdout.lines().collect();
            if let Some(first_line) = lines.first() {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    match parts[1] {
                        "ok" => "Successfully pushed".to_string(),
                        "up to date" => "Already up to date".to_string(),
                        _ => format!("Push completed: {}", parts[1]),
                    }
                } else {
                    "Push completed successfully".to_string()
                }
            } else {
                "Push completed successfully".to_string()
            }
        } else {
            "Push completed successfully".to_string()
        };

        self.log(&format!("✅ {}", push_status));
        Ok(push_status)
    }

    /// Log message with optional file output
    fn log(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }

        if self.log_to_file {
            if let Some(log_path) = &self.log_file {
                let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let log_entry = format!("[{}] {}\n", timestamp, message);
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)
                    .and_then(|mut file| {
                        std::io::Write::write_all(&mut file, log_entry.as_bytes())
                    });
            }
        }
    }

    /// Run in watchdog mode
    pub async fn run_watchdog(
        &self,
        message: Option<String>,
        allow_empty_message: bool,
        skip_validation: bool,
        force: bool,
        args: Vec<String>,
        interval: u64,
    ) -> Result<()> {
        self.log(&format!(
            "🔄 Starting watchdog mode with {}s interval...",
            interval
        ));
        self.log("   Press Ctrl+C to stop");

        loop {
            match self
                .run(
                    message.clone(),
                    allow_empty_message,
                    skip_validation,
                    force,
                    args.clone(),
                )
                .await
            {
                Ok(_) => {
                    self.log("✅ Watchdog cycle completed successfully");
                }
                Err(e) => {
                    self.log(&format!("❌ Watchdog cycle failed: {}", e));
                    if !skip_validation {
                        self.log("   Validation errors detected - skipping commit/push");
                    }
                }
            }

            self.log(&format!(
                "⏰ Waiting {} seconds before next cycle...",
                interval
            ));
            sleep(Duration::from_secs(interval)).await;
        }
    }
}
