use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use tokio::process::Command as TokioCommand;

/// Standard event structure for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredEvent {
    /// Event timestamp in RFC3339 format
    pub timestamp: String,
    /// Event level: "info", "warn", "error"
    pub level: String,
    /// Tool that generated the event: "cargo", "git", "hooksmith"
    pub tool: String,
    /// Action being performed: "validation", "commit", "push", "check", "clippy", "fmt"
    pub action: String,
    /// Human-readable message
    pub message: String,
    /// Optional details (JSON object)
    pub details: Option<Value>,
    /// Optional error code (for cargo/rustc diagnostics)
    pub code: Option<String>,
    /// Optional file path (for diagnostics)
    pub file: Option<String>,
    /// Optional line number (for diagnostics)
    pub line: Option<u32>,
    /// Optional column number (for diagnostics)
    pub column: Option<u32>,
    /// Optional session ID for grouping related events
    pub session_id: Option<String>,
}

impl StructuredEvent {
    /// Create a new structured event
    pub fn new(level: &str, tool: &str, action: &str, message: &str) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            tool: tool.to_string(),
            action: action.to_string(),
            message: message.to_string(),
            details: None,
            code: None,
            file: None,
            line: None,
            column: None,
            session_id: None,
        }
    }

    /// Set details
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Set diagnostic information
    pub fn with_diagnostic(mut self, code: &str, file: &str, line: u32, column: u32) -> Self {
        self.code = Some(code.to_string());
        self.file = Some(file.to_string());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Convert to JSONL format (single line)
    pub fn to_jsonl(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Print to stdout in JSONL format
    pub fn print_jsonl(&self) -> Result<()> {
        println!("{}", self.to_jsonl()?);
        Ok(())
    }
}

/// Structured logging manager
pub struct StructuredLogger {
    /// Whether to emit JSONL to stdout
    pub jsonl_output: bool,
    /// Whether to integrate with event bus
    pub event_bus_integration: bool,
    /// Session ID for grouping events
    pub session_id: Option<String>,
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self {
            jsonl_output: true,
            event_bus_integration: true,
            session_id: None,
        }
    }
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new() -> Self {
        Self::default()
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Disable JSONL output
    pub fn without_jsonl(mut self) -> Self {
        self.jsonl_output = false;
        self
    }

    /// Disable event bus integration
    pub fn without_event_bus(mut self) -> Self {
        self.event_bus_integration = false;
        self
    }

    /// Log an event
    pub fn log(&self, mut event: StructuredEvent) -> Result<()> {
        // Set session ID if available
        if let Some(session_id) = &self.session_id {
            event.session_id = Some(session_id.clone());
        }

        // Emit JSONL to stdout
        if self.jsonl_output {
            event.print_jsonl()?;
        }

        // Integrate with event bus if enabled
        if self.event_bus_integration {
            if let Some(event_bus) = crate::event_bus::get_event_bus() {
                let hooksmith_event = crate::event_bus::HooksmithEvent::new(
                    event.tool.clone(),
                    event.action.clone(),
                    serde_json::to_value(&event)?,
                )
                .with_state(event.action.clone())
                .with_session_id(event.session_id.clone().unwrap_or_default());

                if event.level == "error" {
                    let error_context = serde_json::json!({
                        "message": event.message,
                        "code": event.code,
                        "file": event.file,
                        "line": event.line,
                        "column": event.column,
                        "details": event.details
                    });
                    let hooksmith_event = hooksmith_event.with_error(error_context);
                    crate::event_bus::emit_event(hooksmith_event)?;
                } else {
                    crate::event_bus::emit_event(hooksmith_event)?;
                }
            }
        }

        Ok(())
    }

    /// Log info event
    pub fn info(&self, tool: &str, action: &str, message: &str) -> Result<()> {
        let event = StructuredEvent::new("info", tool, action, message);
        self.log(event)
    }

    /// Log warning event
    pub fn warn(&self, tool: &str, action: &str, message: &str) -> Result<()> {
        let event = StructuredEvent::new("warn", tool, action, message);
        self.log(event)
    }

    /// Log error event
    pub fn error(&self, tool: &str, action: &str, message: &str) -> Result<()> {
        let event = StructuredEvent::new("error", tool, action, message);
        self.log(event)
    }

    /// Log diagnostic event (from cargo/rustc JSON output)
    pub fn diagnostic(&self, diagnostic: &serde_json::Value) -> Result<()> {
        if let Some(message) = diagnostic.get("message") {
            let level = message.get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("info");
            
            let tool = "cargo";
            let action = "diagnostic";
            let msg = message.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown diagnostic");

            let mut event = StructuredEvent::new(level, tool, action, msg);

            // Extract diagnostic details
            if let Some(code) = message.get("code").and_then(|c| c.get("code")).and_then(|c| c.as_str()) {
                event.code = Some(code.to_string());
            }

            if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                if let Some(first_span) = spans.first() {
                    if let Some(file_name) = first_span.get("file_name").and_then(|f| f.as_str()) {
                        event.file = Some(file_name.to_string());
                    }
                    if let Some(line_start) = first_span.get("line_start").and_then(|l| l.as_u64()) {
                        event.line = Some(line_start as u32);
                    }
                    if let Some(column_start) = first_span.get("column_start").and_then(|c| c.as_u64()) {
                        event.column = Some(column_start as u32);
                    }
                }
            }

            // Add full diagnostic as details
            event.details = Some(diagnostic.clone());

            self.log(event)
        } else {
            Ok(())
        }
    }

    /// Run cargo command with structured output
    pub async fn run_cargo_command(&self, subcommand: &str, args: &[&str]) -> Result<bool> {
        let mut command_args = vec![subcommand];
        command_args.extend_from_slice(args);

        // Add JSON output format for supported commands
        match subcommand {
            "check" | "build" | "clippy" => {
                command_args.push("--message-format=json");
            }
            "test" => {
                command_args.extend_from_slice(&["--", "-Z", "unstable-options", "--format", "json"]);
            }
            "fmt" => {
                command_args.extend_from_slice(&["--", "--emit=files", "--check"]);
            }
            _ => {}
        }

        self.info("cargo", subcommand, &format!("Running cargo {}", subcommand))?;

        let output = TokioCommand::new("cargo")
            .args(&command_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn cargo {}", subcommand))?;

        let output = output.wait_with_output().await
            .context(format!("Failed to run cargo {}", subcommand))?;

        if output.status.success() {
            self.info("cargo", subcommand, &format!("cargo {} completed successfully", subcommand))?;
            
            // Parse JSON output for diagnostics
            if subcommand == "check" || subcommand == "build" || subcommand == "clippy" {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Ok(diagnostic) = serde_json::from_str::<serde_json::Value>(line) {
                        if diagnostic.get("reason") == Some(&serde_json::Value::String("compiler-message".to_string())) {
                            self.diagnostic(&diagnostic)?;
                        }
                    }
                }
            }
            
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.error("cargo", subcommand, &format!("cargo {} failed: {}", subcommand, stderr))?;
            Ok(false)
        }
    }

    /// Run git command with structured output
    pub async fn run_git_command(&self, subcommand: &str, args: &[String]) -> Result<String> {
        let mut command_args = vec![subcommand.to_string()];
        command_args.extend_from_slice(args);

        // Add JSON output format for supported commands
        match subcommand {
            "status" => {
                command_args.push("--json".to_string());
            }
            "log" => {
                command_args.push("--format=json".to_string());
            }
            "for-each-ref" => {
                command_args.push("--format=%(objectname:json)".to_string());
            }
            _ => {}
        }

        self.info("git", subcommand, &format!("Running git {}", subcommand))?;

        let output = TokioCommand::new("git")
            .args(&command_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn git {}", subcommand))?;

        let output = output.wait_with_output().await
            .context(format!("Failed to run git {}", subcommand))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.info("git", subcommand, &format!("git {} completed successfully", subcommand))?;
            
            // Parse JSON output for structured events
            if subcommand == "status" || subcommand == "log" {
                for line in stdout.lines() {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
                        let details = serde_json::json!({
                            "command": subcommand,
                            "output": json_value
                        });
                        let event = StructuredEvent::new("info", "git", subcommand, &format!("git {} output", subcommand))
                            .with_details(details);
                        self.log(event)?;
                    }
                }
            }
            
            Ok(stdout.trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.error("git", subcommand, &format!("git {} failed: {}", subcommand, stderr))?;
            anyhow::bail!("git {} failed: {}", subcommand, stderr);
        }
    }

    /// Check git status with structured output
    pub async fn git_status(&self) -> Result<HashMap<String, Value>> {
        let status_output = self.run_git_command("status", &["--json".to_string()]).await?;
        
        if let Ok(status_json) = serde_json::from_str::<serde_json::Value>(&status_output) {
            if let Some(status_obj) = status_json.as_object() {
                let mut status_map = HashMap::new();
                for (key, value) in status_obj {
                    status_map.insert(key.clone(), value.clone());
                }
                return Ok(status_map);
            }
        }
        
        // Fallback to parsing porcelain output
        let porcelain_output = self.run_git_command("status", &["--porcelain".to_string()]).await?;
        let mut status_map = HashMap::new();
        status_map.insert("porcelain".to_string(), serde_json::Value::String(porcelain_output));
        Ok(status_map)
    }

    /// Get git log with structured output
    pub async fn git_log(&self, count: Option<usize>) -> Result<Vec<Value>> {
        let mut args = vec!["--format=json".to_string()];
        if let Some(n) = count {
            args.push("-n".to_string());
            args.push(n.to_string());
        }
        
        let log_output = self.run_git_command("log", &args).await?;
        let mut commits = Vec::new();
        
        for line in log_output.lines() {
            if let Ok(commit_json) = serde_json::from_str::<serde_json::Value>(line) {
                commits.push(commit_json);
            }
        }
        
        Ok(commits)
    }
}

/// Global structured logger instance
static GLOBAL_LOGGER: Lazy<Arc<Mutex<StructuredLogger>>> = Lazy::new(|| {
    Arc::new(Mutex::new(StructuredLogger::new()))
});

/// Initialize the global structured logger
pub fn init_global_logger(jsonl_output: bool, event_bus_integration: bool, session_id: Option<String>) {
    let mut logger = GLOBAL_LOGGER.lock().unwrap();
    logger.jsonl_output = jsonl_output;
    logger.event_bus_integration = event_bus_integration;
    logger.session_id = session_id;
}

/// Get the global structured logger
pub fn get_global_logger() -> Arc<Mutex<StructuredLogger>> {
    GLOBAL_LOGGER.clone()
}

/// Log an info message using the global logger
pub fn log_info(tool: &str, action: &str, message: &str) -> Result<()> {
    let logger = get_global_logger();
    let logger = logger.lock().unwrap();
    logger.info(tool, action, message)
}

/// Log a warning message using the global logger
pub fn log_warn(tool: &str, action: &str, message: &str) -> Result<()> {
    let logger = get_global_logger();
    let logger = logger.lock().unwrap();
    logger.warn(tool, action, message)
}

/// Log an error message using the global logger
pub fn log_error(tool: &str, action: &str, message: &str) -> Result<()> {
    let logger = get_global_logger();
    let logger = logger.lock().unwrap();
    logger.error(tool, action, message)
}

/// Log a diagnostic message using the global logger
pub fn log_diagnostic(diagnostic: &serde_json::Value) -> Result<()> {
    let logger = get_global_logger();
    let logger = logger.lock().unwrap();
    logger.diagnostic(diagnostic)
}

/// Macro for structured logging - replaces println!
#[macro_export]
macro_rules! log_info {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_info($tool, $action, &format!($($arg)*))
    };
}

/// Macro for structured logging - replaces eprintln! for warnings
#[macro_export]
macro_rules! log_warn {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_warn($tool, $action, &format!($($arg)*))
    };
}

/// Macro for structured logging - replaces eprintln! for errors
#[macro_export]
macro_rules! log_error {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_error($tool, $action, &format!($($arg)*))
    };
}

/// Macro for structured logging - replaces println! with success styling
#[macro_export]
macro_rules! log_success {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_info($tool, $action, &format!("✅ {}", format!($($arg)*)))
    };
}

/// Macro for structured logging - replaces println! with failure styling
#[macro_export]
macro_rules! log_failure {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_error($tool, $action, &format!("❌ {}", format!($($arg)*)))
    };
}

/// Macro for structured logging - replaces println! with warning styling
#[macro_export]
macro_rules! log_warning {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::structured_logging::log_warn($tool, $action, &format!("⚠️ {}", format!($($arg)*)))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_event_creation() {
        let event = StructuredEvent::new("info", "cargo", "check", "Running cargo check");
        assert_eq!(event.level, "info");
        assert_eq!(event.tool, "cargo");
        assert_eq!(event.action, "check");
        assert_eq!(event.message, "Running cargo check");
    }

    #[test]
    fn test_structured_event_jsonl() {
        let event = StructuredEvent::new("error", "cargo", "clippy", "Clippy found issues");
        let jsonl = event.to_jsonl().unwrap();
        assert!(jsonl.contains("error"));
        assert!(jsonl.contains("cargo"));
        assert!(jsonl.contains("clippy"));
    }

    #[test]
    fn test_logger_creation() {
        let logger = StructuredLogger::new();
        assert!(logger.jsonl_output);
        assert!(logger.event_bus_integration);
    }
} 
