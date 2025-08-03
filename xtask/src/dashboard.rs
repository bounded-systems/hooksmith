use chrono::{DateTime, Utc};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::error_deduplication;

/// Error statistics for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Error hash
    pub hash: String,
    /// Error type (clippy, fmt, validation, etc.)
    pub error_type: String,
    /// Error message (normalized)
    pub message: String,
    /// Number of times this error has been seen
    pub count: u32,
    /// When this error was first seen
    pub first_seen: DateTime<Utc>,
    /// When this error was last seen
    pub last_seen: DateTime<Utc>,
    /// Whether this error is currently active
    pub is_active: bool,
}

/// Dashboard state
#[derive(Debug)]
pub struct Dashboard {
    /// Error statistics by hash
    errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Whether the dashboard is running
    running: Arc<Mutex<bool>>,
    /// Last update time
    last_update: Arc<Mutex<Instant>>,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Update interval in seconds
    pub update_interval: u64,
    /// Whether to show TUI dashboard
    pub show_dashboard: bool,
    /// Whether to log to JSONL
    pub log_to_jsonl: bool,
    /// JSONL file path
    pub jsonl_path: Option<String>,
    /// Auto-push configuration
    pub auto_push_config: AutoPushConfig,
}

/// Auto-push configuration
#[derive(Debug, Clone)]
pub struct AutoPushConfig {
    /// Whether to enable auto-push
    pub enabled: bool,
    /// Commit message template
    pub commit_message: Option<String>,
    /// Whether to skip validation
    pub skip_validation: bool,
    /// Force push
    pub force: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: 30,
            show_dashboard: false, // Default to headless mode
            log_to_jsonl: true,
            jsonl_path: Some("hooksmith-events.jsonl".to_string()),
            auto_push_config: AutoPushConfig {
                enabled: true,
                commit_message: None,
                skip_validation: false,
                force: false,
            },
        }
    }
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            errors: Arc::new(Mutex::new(HashMap::new())),
            config,
            running: Arc::new(Mutex::new(false)),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Start the dashboard
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting Hooksmith Dashboard...");
        println!(
            "   📊 Update interval: {} seconds",
            self.config.update_interval
        );
        println!(
            "   🖥️  Dashboard mode: {}",
            if self.config.show_dashboard {
                "TUI"
            } else {
                "Headless"
            }
        );
        println!(
            "   📝 JSONL logging: {}",
            if self.config.log_to_jsonl {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "   🔄 Auto-push: {}",
            if self.config.auto_push_config.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "   🔍 Skip validation: {}",
            if self.config.auto_push_config.skip_validation {
                "Yes"
            } else {
                "No"
            }
        );
        println!("   Press 'q' to quit, 'c' to clear errors, 'r' to refresh");
        println!("");

        let running = Arc::clone(&self.running);
        let mut running_guard = running.lock().unwrap();
        *running_guard = true;
        drop(running_guard);

        println!("✅ Dashboard state initialized");

        if self.config.show_dashboard {
            println!("🖥️  Setting up TUI terminal...");
            self.setup_terminal()?;
            println!("✅ TUI terminal setup complete");
        } else {
            println!("📊 Running in headless mode");
        }
        println!(
            "   📊 Update interval: {} seconds",
            self.config.update_interval
        );
        println!(
            "   🖥️  Dashboard mode: {}",
            if self.config.show_dashboard {
                "TUI"
            } else {
                "Headless"
            }
        );
        println!(
            "   📝 JSONL logging: {}",
            if self.config.log_to_jsonl {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "   🔄 Auto-push: {}",
            if self.config.auto_push_config.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!("   Press 'q' to quit, 'c' to clear errors, 'r' to refresh");
        println!("");

        let dashboard_handle = {
            let errors = Arc::clone(&self.errors);
            let config = self.config.clone();
            let running = Arc::clone(&self.running);
            let last_update = Arc::clone(&self.last_update);

            tokio::spawn(async move {
                println!("🔄 Starting dashboard loop...");
                let result = Self::dashboard_loop(errors, config, running, last_update).await;
                println!("🔄 Dashboard loop finished: {:?}", result);
                result
            })
        };

        let watchdog_handle = {
            let errors = Arc::clone(&self.errors);
            let config = self.config.clone();
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                println!("🔄 Starting watchdog loop...");
                let result = Self::watchdog_loop(errors, config, running).await;
                println!("🔄 Watchdog loop finished: {:?}", result);
                result
            })
        };

        // Wait for either task to complete
        tokio::select! {
            result = dashboard_handle => {
                println!("Dashboard task completed: {:?}", result);
            },
            result = watchdog_handle => {
                println!("Watchdog task completed: {:?}", result);
            },
        }

        self.cleanup()?;
        Ok(())
    }

    /// Main dashboard loop
    async fn dashboard_loop(
        errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: DashboardConfig,
        running: Arc<Mutex<bool>>,
        last_update: Arc<Mutex<Instant>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            if !*running.lock().unwrap() {
                break;
            }

            // Only render dashboard if TUI is enabled
            if config.show_dashboard {
                Self::render_dashboard(&errors, &config, &last_update)?;

                // Handle user input
                if event::poll(Duration::from_millis(100))? {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                let mut running_guard = running.lock().unwrap();
                                *running_guard = false;
                                break;
                            }
                            KeyCode::Char('c') => {
                                Self::clear_errors(&errors)?;
                            }
                            KeyCode::Char('r') => {
                                let mut last_update_guard = last_update.lock().unwrap();
                                *last_update_guard = Instant::now();
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                // In headless mode, just sleep and check for quit
                sleep(Duration::from_millis(1000)).await;
            }

            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    /// Main watchdog loop
    async fn watchdog_loop(
        errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: DashboardConfig,
        running: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            if !*running.lock().unwrap() {
                break;
            }

            // Run validation and collect errors (unless skipped)
            if !config.auto_push_config.skip_validation {
                Self::run_validation_cycle(&errors, &config).await?;
            }

            // Run auto-push if enabled
            if config.auto_push_config.enabled {
                Self::run_auto_push_cycle(&config.auto_push_config).await?;
            }

            sleep(Duration::from_secs(config.update_interval)).await;
        }
        Ok(())
    }

    /// Run a single validation cycle
    async fn run_validation_cycle(
        errors: &Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: &DashboardConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut new_errors = Vec::new();

        // Run cargo fix
        let fix_output = std::process::Command::new("cargo")
            .args(["fix", "--allow-dirty", "--allow-staged"])
            .output()?;

        if !fix_output.status.success() {
            new_errors.push(("cargo fix".to_string(), fix_output));
        }

        // Run cargo fmt
        let fmt_output = std::process::Command::new("cargo")
            .args(["fmt", "--all"])
            .output()?;

        if !fmt_output.status.success() {
            new_errors.push(("cargo fmt".to_string(), fmt_output));
        }

        // Run cargo clippy
        let clippy_output = std::process::Command::new("cargo")
            .args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .output()?;

        if !clippy_output.status.success() {
            new_errors.push(("cargo clippy".to_string(), clippy_output));
        }

        // Process new errors
        for (command_name, output) in new_errors {
            Self::process_validation_output(&errors, &command_name, &output).await?;
        }

        Ok(())
    }

    /// Process validation output and update error stats
    async fn process_validation_output(
        errors: &Arc<Mutex<HashMap<String, ErrorStats>>>,
        command_name: &str,
        output: &std::process::Output,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut error_texts = Vec::new();
        if !stderr.is_empty() {
            error_texts.push(stderr.to_string());
        }
        if !stdout.is_empty() {
            error_texts.push(stdout.to_string());
        }

        for error_text in error_texts {
            let normalized = error_deduplication::normalize_error(&error_text);
            if !normalized.is_empty() {
                let hash = error_deduplication::hash_error(&normalized);
                let now = Utc::now();

                let mut errors_guard = errors.lock().unwrap();
                let error_stats = errors_guard
                    .entry(hash.clone())
                    .or_insert_with(|| ErrorStats {
                        hash: hash.clone(),
                        error_type: command_name.to_string(),
                        message: normalized.clone(),
                        count: 0,
                        first_seen: now,
                        last_seen: now,
                        is_active: true,
                    });

                error_stats.count += 1;
                error_stats.last_seen = now;
                error_stats.is_active = true;
            }
        }

        Ok(())
    }

    /// Run auto-push cycle
    async fn run_auto_push_cycle(
        config: &AutoPushConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !config.enabled {
            return Ok(());
        }

        println!("🔄 Running auto-push cycle...");

        // Check if there are any changes to commit
        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| format!("Failed to check git status: {}", e))?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        if status.trim().is_empty() {
            println!("   ✅ No changes to commit");
            return Ok(());
        }

        // Add all changes
        println!("   📦 Adding all changes...");
        let add_status = std::process::Command::new("git")
            .args(["add", "."])
            .status()
            .map_err(|e| format!("Failed to add changes: {}", e))?;

        if !add_status.success() {
            return Err("git add failed".into());
        }

        // Generate commit message
        let commit_message = if let Some(template) = &config.commit_message {
            template.clone()
        } else {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            format!("chore: auto-update at {}", timestamp)
        };

        // Commit changes
        println!("   📝 Committing with message: {}", commit_message);
        let commit_status = std::process::Command::new("git")
            .args(["commit", "-m", &commit_message])
            .status()
            .map_err(|e| format!("Failed to commit: {}", e))?;

        if !commit_status.success() {
            return Err("git commit failed".into());
        }

        // Push changes
        println!("   🚀 Pushing to remote...");
        let push_args = if config.force {
            vec!["push", "--force"]
        } else {
            vec!["push"]
        };

        let push_status = std::process::Command::new("git")
            .args(&push_args)
            .status()
            .map_err(|e| format!("Failed to push: {}", e))?;

        if !push_status.success() {
            return Err("git push failed".into());
        }

        println!("   ✅ Auto-push completed successfully!");
        Ok(())
    }

    /// Render the dashboard
    fn render_dashboard(
        errors: &Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: &DashboardConfig,
        last_update: &Arc<Mutex<Instant>>,
    ) -> std::io::Result<()> {
        execute!(stdout(), Clear(ClearType::All))?;

        let errors_guard = errors.lock().unwrap();
        let last_update_guard = last_update.lock().unwrap();
        let uptime = last_update_guard.elapsed();

        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│                           🚀 Hooksmith Dashboard                           │");
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│ Status: {} | Uptime: {:?} | Errors: {} | Auto-push: {}",
            if config.auto_push_config.enabled {
                "🟢 Active"
            } else {
                "🔴 Disabled"
            },
            uptime,
            errors_guard.len(),
            if config.auto_push_config.enabled {
                "🟢 On"
            } else {
                "🔴 Off"
            }
        );
        println!("├─────────────────────────────────────────────────────────────────────────────┤");

        if errors_guard.is_empty() {
            println!(
                "│ ✅ No errors detected - all systems operational!                              │"
            );
        } else {
            println!(
                "│ Type           │ Count │ Hash (first 8) │ Last Seen                        │"
            );
            println!(
                "├────────────────┼───────┼────────────────┼──────────────────────────────────┤"
            );

            for error in errors_guard.values() {
                let hash_short = &error.hash[..8.min(error.hash.len())];
                let last_seen = error.last_seen.format("%H:%M:%S");
                println!(
                    "│ {:<14} │ {:<5} │ {:<14} │ {:<32} │",
                    error.error_type, error.count, hash_short, last_seen
                );
            }
        }

        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!("│ Controls: [q] Quit | [c] Clear Errors | [r] Refresh | [s] Toggle Auto-push │");
        println!("└─────────────────────────────────────────────────────────────────────────────┘");

        Ok(())
    }

    /// Clear all errors
    fn clear_errors(errors: &Arc<Mutex<HashMap<String, ErrorStats>>>) -> std::io::Result<()> {
        let mut errors_guard = errors.lock().unwrap();
        errors_guard.clear();
        Ok(())
    }

    /// Setup terminal for TUI
    fn setup_terminal(&self) -> std::io::Result<()> {
        // Try to set up the terminal, but don't fail if it doesn't work
        match execute!(stdout(), EnterAlternateScreen, Hide) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("⚠️  Warning: Could not set up TUI terminal: {}", e);
                eprintln!("📊 Falling back to console mode");
                Ok(())
            }
        }
    }

    /// Cleanup terminal
    fn cleanup(&self) -> std::io::Result<()> {
        if self.config.show_dashboard {
            execute!(stdout(), Show, LeaveAlternateScreen)?;
        }
        Ok(())
    }
}
