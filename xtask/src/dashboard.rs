use crate::event_bus::{EventBus, HooksmithEvent};
use chrono::{DateTime, Utc};
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;

/// Error statistics for deduplication
#[derive(Debug, Clone)]
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

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Whether to show TUI dashboard
    pub show_dashboard: bool,
    /// Whether to log to JSONL
    pub log_to_jsonl: bool,
    /// JSONL file path
    pub jsonl_path: Option<String>,
    /// Auto-push configuration
    pub auto_push_config: AutoPushConfig,
    /// Whether to run in file-watch mode
    pub file_watch_mode: bool,
    /// Update interval for heartbeat events (seconds)
    pub heartbeat_interval: u64,
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
            show_dashboard: false,
            log_to_jsonl: true,
            jsonl_path: Some("hooksmith-events.jsonl".to_string()),
            auto_push_config: AutoPushConfig {
                enabled: true,
                commit_message: None,
                skip_validation: false,
                force: false,
            },
            file_watch_mode: false,
            heartbeat_interval: 30,
        }
    }
}

/// Event-driven dashboard that subscribes to the event bus
pub struct Dashboard {
    /// Error statistics by hash
    errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Whether the dashboard is running
    running: Arc<Mutex<bool>>,
    /// Last update time
    last_update: Arc<Mutex<Instant>>,
    /// Event bus subscription
    event_receiver: Option<broadcast::Receiver<HooksmithEvent>>,
    /// Event statistics
    stats: Arc<Mutex<DashboardStats>>,
}

/// Dashboard statistics
#[derive(Debug, Default)]
pub struct DashboardStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_actor: HashMap<String, u64>,
    pub errors_count: u64,
    pub warnings_count: u64,
    pub info_count: u64,
    pub last_event_time: Option<DateTime<Utc>>,
}

impl Dashboard {
    /// Create a new event-driven dashboard
    pub fn new(config: DashboardConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let errors = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(Mutex::new(true));
        let last_update = Arc::new(Mutex::new(Instant::now()));
        let stats = Arc::new(Mutex::new(DashboardStats::default()));

        // Subscribe to the event bus
        let event_receiver = if let Some(event_bus) = crate::event_bus::get_event_bus() {
            Some(event_bus.subscribe())
        } else {
            None
        };

        Ok(Self {
            errors,
            config,
            running,
            last_update,
            event_receiver,
            stats,
        })
    }

    /// Start the event-driven dashboard
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting event-driven Hooksmith Dashboard...");
        println!("📡 Subscribing to event bus...");

        if self.config.show_dashboard {
            self.setup_terminal()?;
        }

        // Start the main event processing loop
        let event_loop_handle = {
            let errors = self.errors.clone();
            let config = self.config.clone();
            let running = self.running.clone();
            let last_update = self.last_update.clone();
            let stats = self.stats.clone();
            let event_receiver = self.event_receiver.take();

            tokio::spawn(async move {
                Self::event_processing_loop(
                    errors,
                    config,
                    running,
                    last_update,
                    stats,
                    event_receiver,
                )
                .await
            })
        };

        // Start the UI update loop (if dashboard is enabled)
        let ui_loop_handle = if self.config.show_dashboard {
            let errors = self.errors.clone();
            let config = self.config.clone();
            let running = self.running.clone();
            let last_update = self.last_update.clone();

            Some(tokio::spawn(async move {
                Self::ui_update_loop(errors, config, running, last_update).await
            }))
        } else {
            None
        };

        // Start heartbeat events for periodic updates
        let heartbeat_handle = {
            let config = self.config.clone();
            tokio::spawn(async move { Self::heartbeat_loop(config).await })
        };

        // Wait for the event loop to complete
        event_loop_handle.await??;

        // Clean up
        if let Some(ui_handle) = ui_loop_handle {
            let _ = ui_handle.await;
        }

        if self.config.show_dashboard {
            self.cleanup()?;
        }

        println!("🛑 Dashboard stopped");
        Ok(())
    }

    /// Main event processing loop
    async fn event_processing_loop(
        errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: DashboardConfig,
        running: Arc<Mutex<bool>>,
        last_update: Arc<Mutex<Instant>>,
        stats: Arc<Mutex<DashboardStats>>,
        mut event_receiver: Option<broadcast::Receiver<HooksmithEvent>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref mut receiver) = event_receiver {
            println!("📡 Event processing loop started");

            while *running.lock().unwrap() {
                match receiver.recv().await {
                    Ok(event) => {
                        // Update last update time
                        *last_update.lock().unwrap() = Instant::now();

                        // Process the event
                        Self::process_event(&errors, &stats, &event).await?;

                        // Update statistics
                        Self::update_stats(&stats, &event).await?;

                        // Handle auto-push events
                        if config.auto_push_config.enabled {
                            Self::handle_auto_push_event(&config.auto_push_config, &event).await?;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("📡 Event bus closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        println!("⚠️  Lagged {} events", n);
                    }
                }
            }
        } else {
            println!("⚠️  No event bus available, running in polling mode");
            // Fallback to polling mode if no event bus
            while *running.lock().unwrap() {
                sleep(Duration::from_secs(config.heartbeat_interval)).await;
            }
        }

        Ok(())
    }

    /// UI update loop for TUI dashboard
    async fn ui_update_loop(
        errors: Arc<Mutex<HashMap<String, ErrorStats>>>,
        config: DashboardConfig,
        running: Arc<Mutex<bool>>,
        last_update: Arc<Mutex<Instant>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🎨 UI update loop started");

        while *running.lock().unwrap() {
            // Render the dashboard
            Self::render_dashboard(&errors, &config, &last_update)?;

            // Wait for next update
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Heartbeat loop for periodic events
    async fn heartbeat_loop(
        config: DashboardConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "💓 Heartbeat loop started ({}s interval)",
            config.heartbeat_interval
        );

        loop {
            sleep(Duration::from_secs(config.heartbeat_interval)).await;

            // Emit heartbeat event
            let heartbeat_event = HooksmithEvent::new(
                "dashboard".to_string(),
                "heartbeat".to_string(),
                serde_json::json!({
                    "timestamp": chrono::Utc::now(),
                    "interval": config.heartbeat_interval
                }),
            );

            if let Err(e) = crate::event_bus::emit_event(heartbeat_event) {
                eprintln!("Failed to emit heartbeat event: {}", e);
            }
        }
    }

    /// Process a single event
    async fn process_event(
        errors: &Arc<Mutex<HashMap<String, ErrorStats>>>,
        stats: &Arc<Mutex<DashboardStats>>,
        event: &HooksmithEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Handle error events
        if let Some(error) = &event.error {
            let error_message = error.as_str().unwrap_or("Unknown error");
            let error_type = event.event.clone();

            // Normalize and hash the error
            let normalized = crate::error_deduplication::normalize_error(error_message);
            let hash = crate::error_deduplication::hash_error(&normalized);

            // Update error statistics
            let mut errors_guard = errors.lock().unwrap();
            let error_stats = errors_guard
                .entry(hash.clone())
                .or_insert_with(|| ErrorStats {
                    hash: hash.clone(),
                    error_type: error_type.clone(),
                    message: normalized.clone(),
                    count: 0,
                    first_seen: event.ts,
                    last_seen: event.ts,
                    is_active: true,
                });

            error_stats.count += 1;
            error_stats.last_seen = event.ts;
            error_stats.is_active = true;
        }

        // Handle validation events
        if event.event == "validation_failed" || event.event == "validation_passed" {
            // Clear errors if validation passed
            if event.event == "validation_passed" {
                let mut errors_guard = errors.lock().unwrap();
                for error in errors_guard.values_mut() {
                    error.is_active = false;
                }
            }
        }

        Ok(())
    }

    /// Update dashboard statistics
    async fn update_stats(
        stats: &Arc<Mutex<DashboardStats>>,
        event: &HooksmithEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stats_guard = stats.lock().unwrap();

        stats_guard.total_events += 1;
        stats_guard.last_event_time = Some(event.ts);

        // Count by event type
        *stats_guard
            .events_by_type
            .entry(event.event.clone())
            .or_insert(0) += 1;

        // Count by actor
        *stats_guard
            .events_by_actor
            .entry(event.actor.clone())
            .or_insert(0) += 1;

        // Count by severity (if available in context)
        if let Some(error) = &event.error {
            stats_guard.errors_count += 1;
        } else {
            stats_guard.info_count += 1;
        }

        Ok(())
    }

    /// Handle auto-push related events
    async fn handle_auto_push_event(
        config: &AutoPushConfig,
        event: &HooksmithEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.event.as_str() {
            "validation_passed" => {
                if config.enabled && !config.skip_validation {
                    println!("✅ Validation passed, triggering auto-push...");
                    Self::run_auto_push_cycle(config).await?;
                }
            }
            "validation_failed" => {
                println!("❌ Validation failed, skipping auto-push");
            }
            "git_push_failed" => {
                if let Some(error) = &event.error {
                    println!("🚫 Git push failed: {}", error);
                }
            }
            "git_push_succeeded" => {
                println!("✅ Git push succeeded");
            }
            _ => {}
        }

        Ok(())
    }

    /// Run auto-push cycle
    pub async fn run_auto_push_cycle(
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
            println!("✅ No changes to commit.");
            return Ok(());
        }

        // Add all changes
        println!("➕ Adding all changes...");
        let add_status = std::process::Command::new("git")
            .args(["add", "."])
            .status()
            .map_err(|e| format!("Failed to add changes: {}", e))?;
        if !add_status.success() {
            return Err("Git add failed".into());
        }

        // Generate commit message
        let commit_message = if let Some(template) = &config.commit_message {
            template.clone()
        } else {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            format!("chore: auto-update at {}", timestamp)
        };

        // Commit changes
        println!("📝 Committing changes with message: \"{}\"", commit_message);
        let commit_status = std::process::Command::new("git")
            .args(["commit", "-m", &commit_message])
            .status()
            .map_err(|e| format!("Failed to commit changes: {}", e))?;
        if !commit_status.success() {
            return Err("Git commit failed".into());
        }

        // Push changes
        println!("📤 Pushing changes...");
        let mut push_command = std::process::Command::new("git");
        push_command.arg("push");
        if config.force {
            push_command.arg("--force");
        }
        let push_status = push_command
            .status()
            .map_err(|e| format!("Failed to push changes: {}", e))?;

        if !push_status.success() {
            return Err("Git push failed".into());
        }

        println!("✅ Auto-push cycle completed successfully!");
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
        println!("│                    🚀 Hooksmith Event-Driven Dashboard                    │");
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│ Status: {} | Uptime: {:?} | Errors: {} | Auto-push: {}",
            "🟢 Active",
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
            println!("│ ✅ No errors detected                                                          │");
        } else {
            println!("│ 🔴 Active Errors:                                                              │");
            for (i, (hash, error)) in errors_guard.iter().take(10).enumerate() {
                println!(
                    "│ {}. {} ({}x) - {}",
                    i + 1,
                    error.error_type,
                    error.count,
                    if error.message.len() > 50 {
                        format!("{}...", &error.message[..47])
                    } else {
                        error.message.clone()
                    }
                );
            }
            if errors_guard.len() > 10 {
                println!("│ ... and {} more errors", errors_guard.len() - 10);
            }
        }

        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│ 💡 Press Ctrl+C to stop                                                          │"
        );
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
        match execute!(stdout(), Clear(ClearType::All)) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "⚠️  Failed to setup terminal: {}. Running in console mode.",
                    e
                );
                Ok(())
            }
        }
    }

    /// Cleanup terminal
    fn cleanup(&self) -> std::io::Result<()> {
        execute!(stdout(), Clear(ClearType::All))?;
        execute!(stdout(), cursor::Show)?;
        Ok(())
    }
}
