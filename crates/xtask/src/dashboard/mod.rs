pub mod state;
pub mod ui;

pub use state::StateManager;
pub use ui::render_dashboard;

// Re-export the main Dashboard struct and configs
use crate::event_bus::HooksmithEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

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
            },
            file_watch_mode: false,
            heartbeat_interval: 30,
        }
    }
}

/// Event-driven dashboard that subscribes to the event bus
pub struct Dashboard {
    /// State manager for the dashboard
    state_manager: StateManager,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Whether the dashboard is running
    running: Arc<Mutex<bool>>,
    /// Event bus subscription
    event_receiver: Option<broadcast::Receiver<HooksmithEvent>>,
    /// Terminal instance for TUI
    terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
}

impl Dashboard {
    /// Create a new event-driven dashboard
    pub fn new(config: DashboardConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let state_manager = StateManager::new();
        let running = Arc::new(Mutex::new(true));

        // Subscribe to the event bus
        let event_receiver =
            crate::event_bus::get_event_bus().map(|event_bus| event_bus.subscribe());

        // Initialize terminal if dashboard is enabled
        let terminal = if config.show_dashboard {
            let backend = CrosstermBackend::new(stdout());
            let terminal = Terminal::new(backend)?;
            Some(terminal)
        } else {
            None
        };

        Ok(Self {
            state_manager,
            config,
            running,
            event_receiver,
            terminal,
        })
    }

    /// Start the event-driven dashboard
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting event-driven Hooksmith Dashboard...");
        println!("📡 Subscribing to event bus...");

        // Update system status
        self.state_manager.update_system_status(
            self.config.auto_push_config.enabled,
            self.config.file_watch_mode,
            self.config.heartbeat_interval,
        );

        if self.config.show_dashboard {
            self.setup_terminal()?;
        }

        // Start the main event processing loop
        let event_loop_handle = {
            let state_manager = self.state_manager.clone();
            let config = self.config.clone();
            let running = self.running.clone();
            let event_receiver = self.event_receiver.take();

            tokio::spawn(async move {
                Self::event_processing_loop(state_manager, config, running, event_receiver).await
            })
        };

        // Start the UI update loop (if dashboard is enabled)
        let ui_loop_handle = if self.config.show_dashboard {
            let state_manager = self.state_manager.clone();
            let running = self.running.clone();
            let terminal = self.terminal.take();

            Some(tokio::spawn(async move {
                Self::ui_update_loop(state_manager, running, terminal).await
            }))
        } else {
            None
        };

        // Start heartbeat events for periodic updates
        let heartbeat_handle = {
            let config = self.config.clone();
            let state_manager = self.state_manager.clone();
            tokio::spawn(async move { Self::heartbeat_loop(config, state_manager).await })
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

    /// Main event processing loop with tokio::select!
    async fn event_processing_loop(
        state_manager: StateManager,
        config: DashboardConfig,
        running: Arc<Mutex<bool>>,
        mut event_receiver: Option<broadcast::Receiver<HooksmithEvent>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref mut receiver) = event_receiver {
            println!("📡 Event processing loop started");

            while *running.lock().unwrap() {
                // Use tokio::select! for reactive event handling
                tokio::select! {
                    // Handle incoming events
                    event_result = receiver.recv() => {
                        match event_result {
                            Ok(event) => {
                                // Update state from event
                                state_manager.update_from_event(&event);

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
                                println!("⚠️  Lagged {n} events");
                            }
                        }
                    }
                    // Periodic heartbeat
                    _ = sleep(Duration::from_secs(config.heartbeat_interval)) => {
                        state_manager.update_heartbeat();

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
                            eprintln!("Failed to emit heartbeat event: {e}");
                        }
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

    /// UI update loop with keyboard navigation
    async fn ui_update_loop(
        state_manager: StateManager,
        running: Arc<Mutex<bool>>,
        mut terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🎨 UI update loop started");

        while *running.lock().unwrap() {
            // Handle keyboard events
            if let Some(ref mut term) = terminal {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Left => {
                                let current_tab = state_manager.get_state().selected_tab;
                                let new_tab = if current_tab == 0 { 3 } else { current_tab - 1 };
                                state_manager.set_selected_tab(new_tab);
                            }
                            KeyCode::Right => {
                                let current_tab = state_manager.get_state().selected_tab;
                                let new_tab = (current_tab + 1) % 4;
                                state_manager.set_selected_tab(new_tab);
                            }
                            KeyCode::Char('q') | KeyCode::Esc => {
                                *running.lock().unwrap() = false;
                                break;
                            }
                            KeyCode::Char('1') => state_manager.set_selected_tab(0),
                            KeyCode::Char('2') => state_manager.set_selected_tab(1),
                            KeyCode::Char('3') => state_manager.set_selected_tab(2),
                            KeyCode::Char('4') => state_manager.set_selected_tab(3),
                            _ => {}
                        }
                    }
                }

                // Render the dashboard
                let state = state_manager.get_state();
                term.draw(|frame| {
                    render_dashboard(frame, &state);
                })?;
            }

            // Wait for next update
            sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }

    /// Heartbeat loop for periodic events
    async fn heartbeat_loop(
        config: DashboardConfig,
        state_manager: StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "💓 Heartbeat loop started ({}s interval)",
            config.heartbeat_interval
        );

        loop {
            sleep(Duration::from_secs(config.heartbeat_interval)).await;

            // Update heartbeat in state
            state_manager.update_heartbeat();

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
                eprintln!("Failed to emit heartbeat event: {e}");
            }
        }
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
                    println!("🚫 Git push failed: {error}");
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
            .map_err(|e| format!("Failed to check git status: {e}"))?;

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
            .map_err(|e| format!("Failed to add changes: {e}"))?;
        if !add_status.success() {
            return Err("Git add failed".into());
        }

        // Generate commit message
        let commit_message = if let Some(template) = &config.commit_message {
            template.clone()
        } else {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            format!("chore: auto-update at {timestamp}")
        };

        // Commit changes
        println!("📝 Committing changes with message: \"{commit_message}\"");
        let commit_status = std::process::Command::new("git")
            .args(["commit", "-m", &commit_message])
            .status()
            .map_err(|e| format!("Failed to commit changes: {e}"))?;
        if !commit_status.success() {
            return Err("Git commit failed".into());
        }

        // Push changes (never force push)
        println!("📤 Pushing changes...");
        let push_output = std::process::Command::new("git")
            .args(["push", "--porcelain"])
            .output()
            .map_err(|e| format!("Failed to push changes: {e}"))?;

        if !push_output.status.success() {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            let stdout = String::from_utf8_lossy(&push_output.stdout);

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

            return Err(format!("Git push failed: {error_message}").into());
        }

        println!("✅ Auto-push cycle completed successfully!");
        Ok(())
    }

    /// Setup terminal for TUI
    fn setup_terminal(&self) -> std::io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), Clear(ClearType::All), EnableMouseCapture)?;
        Ok(())
    }

    /// Cleanup terminal
    fn cleanup(&self) -> std::io::Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), Clear(ClearType::All), DisableMouseCapture)?;
        Ok(())
    }
}
