use crate::event_bus::{emit_event, HooksmithEvent};
use chrono::Utc;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Hook name
    pub name: String,
    /// Execution status
    pub status: HookStatus,
    /// Execution duration
    pub duration: Duration,
    /// Error count (if applicable)
    pub error_count: Option<usize>,
    /// Warning count (if applicable)
    pub warning_count: Option<usize>,
    /// Files changed (if applicable)
    pub files_changed: Option<usize>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Log file path
    pub log_file: PathBuf,
}

/// Hook execution status
#[derive(Debug, Clone, PartialEq)]
pub enum HookStatus {
    Passed,
    Failed,
    Warning,
}

impl HookStatus {
    fn to_emoji(&self) -> &'static str {
        match self {
            HookStatus::Passed => "✅",
            HookStatus::Failed => "❌",
            HookStatus::Warning => "⚠️",
        }
    }

    fn to_color(&self) -> Color {
        match self {
            HookStatus::Passed => Color::Green,
            HookStatus::Failed => Color::Red,
            HookStatus::Warning => Color::Yellow,
        }
    }
}

/// Hook runner configuration
#[derive(Debug, Clone)]
pub struct HookRunnerConfig {
    /// Whether to show detailed output
    pub verbose: bool,
    /// Whether to save logs
    pub save_logs: bool,
    /// Log directory
    pub log_dir: PathBuf,
    /// Whether to emit events
    pub emit_events: bool,
    /// Hook timeout in seconds
    pub timeout: Option<u64>,
}

impl Default for HookRunnerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            save_logs: true,
            log_dir: PathBuf::from(".hooksmith/logs"),
            emit_events: true,
            timeout: Some(300), // 5 minutes
        }
    }
}

/// Hook runner for clean, summarized output
pub struct HookRunner {
    config: HookRunnerConfig,
}

impl HookRunner {
    /// Create a new hook runner
    pub fn new(config: HookRunnerConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure log directory exists
        if config.save_logs {
            fs::create_dir_all(&config.log_dir)?;
        }

        Ok(Self { config })
    }

    /// Run a specific hook with clean output
    pub fn run_hook(
        &self,
        hook_name: &str,
    ) -> Result<HookResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let log_file = self
            .config
            .log_dir
            .join(format!("{hook_name}-{timestamp}.log"));

        println!("🔧 Running {hook_name}...");

        // Run the hook command
        let result = self.execute_hook(hook_name, &log_file)?;
        let duration = start_time.elapsed();

        // Parse the result
        let hook_result = self.parse_hook_result(hook_name, result, duration, log_file)?;

        // Print summary
        self.print_hook_summary(&hook_result);

        // Emit event if enabled
        if self.config.emit_events {
            self.emit_hook_event(&hook_result)?;
        }

        Ok(hook_result)
    }

    /// Run multiple hooks and provide a summary
    pub fn run_hooks(
        &self,
        hook_names: &[&str],
    ) -> Result<Vec<HookResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let start_time = Instant::now();

        println!("╭──────────────────────────────────────╮");
        println!("│ 🥊 Running {} hooks", hook_names.len());
        println!("╰──────────────────────────────────────╯");

        for hook_name in hook_names {
            match self.run_hook(hook_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("❌ Failed to run {hook_name}: {e}");
                    // Create a failed result
                    let failed_result = HookResult {
                        name: hook_name.to_string(),
                        status: HookStatus::Failed,
                        duration: Duration::from_secs(0),
                        error_count: None,
                        warning_count: None,
                        files_changed: None,
                        error_message: Some(e.to_string()),
                        log_file: PathBuf::new(),
                    };
                    results.push(failed_result);
                }
            }
        }

        // Print final summary
        self.print_final_summary(&results, start_time.elapsed());

        Ok(results)
    }

    /// Execute a single hook command
    fn execute_hook(
        &self,
        hook_name: &str,
        log_file: &Path,
    ) -> Result<ExitStatus, Box<dyn std::error::Error + Send + Sync>> {
        let mut command = Command::new("lefthook");
        command.args(["run", hook_name]);

        if !self.config.verbose {
            command.arg("--no-colors");
        }

        // Set up output capture
        let output = if self.config.save_logs {
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        } else {
            command.status()?;
            return Ok(ExitStatus::from_raw(0));
        };

        // Save output to log file
        if self.config.save_logs {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(log_file)?;

            writeln!(file, "=== {hook_name} Hook Execution ===")?;
            writeln!(file, "Timestamp: {}", Utc::now())?;
            writeln!(file, "Command: lefthook run {hook_name}")?;
            writeln!(file, "Exit Code: {}", output.status)?;
            writeln!(file, "\n=== STDOUT ===")?;
            file.write_all(&output.stdout)?;
            writeln!(file, "\n=== STDERR ===")?;
            file.write_all(&output.stderr)?;
        }

        Ok(output.status)
    }

    /// Parse hook execution result
    fn parse_hook_result(
        &self,
        hook_name: &str,
        exit_status: ExitStatus,
        duration: Duration,
        log_file: PathBuf,
    ) -> Result<HookResult, Box<dyn std::error::Error + Send + Sync>> {
        let success = exit_status.success();
        let status = if success {
            HookStatus::Passed
        } else {
            HookStatus::Failed
        };

        // Parse specific hook results
        let (error_count, warning_count, files_changed, error_message) = match hook_name {
            "fmt" => self.parse_fmt_result(&log_file)?,
            "clippy" => self.parse_clippy_result(&log_file)?,
            "validate-extensions" => self.parse_validate_extensions_result(&log_file)?,
            _ => (None, None, None, None),
        };

        Ok(HookResult {
            name: hook_name.to_string(),
            status,
            duration,
            error_count,
            warning_count,
            files_changed,
            error_message,
            log_file,
        })
    }

    /// Parse cargo fmt results
    fn parse_fmt_result(
        &self,
        log_file: &Path,
    ) -> Result<(Option<usize>, Option<usize>, Option<usize>, Option<String>), io::Error> {
        if !log_file.exists() {
            return Ok((None, None, None, None));
        }

        let content = fs::read_to_string(log_file)?;

        // Look for diff count in fmt output
        if content.contains("Diff in") {
            let diff_count = content
                .lines()
                .filter(|line| line.contains("Diff in"))
                .count();
            Ok((Some(diff_count), None, Some(diff_count), None))
        } else {
            Ok((None, None, None, None))
        }
    }

    /// Parse cargo clippy results
    fn parse_clippy_result(
        &self,
        log_file: &Path,
    ) -> Result<(Option<usize>, Option<usize>, Option<usize>, Option<String>), io::Error> {
        if !log_file.exists() {
            return Ok((None, None, None, None));
        }

        let content = fs::read_to_string(log_file)?;

        let error_count = content
            .lines()
            .filter(|line| line.contains("error[E"))
            .count();
        let warning_count = content
            .lines()
            .filter(|line| line.contains("warning:"))
            .count();

        let error_message = if error_count > 0 {
            Some(format!("{error_count} errors, {warning_count} warnings"))
        } else {
            None
        };

        Ok((Some(error_count), Some(warning_count), None, error_message))
    }

    /// Parse validate-extensions results
    fn parse_validate_extensions_result(
        &self,
        log_file: &Path,
    ) -> Result<(Option<usize>, Option<usize>, Option<usize>, Option<String>), io::Error> {
        if !log_file.exists() {
            return Ok((None, None, None, None));
        }

        let content = fs::read_to_string(log_file)?;

        // Look for validation results
        if content.contains("✅ All extension validations passed") {
            Ok((None, None, None, None))
        } else {
            let error_count = content.lines().filter(|line| line.contains("❌")).count();
            Ok((
                Some(error_count),
                None,
                None,
                Some("Extension validation failed".to_string()),
            ))
        }
    }

    /// Print hook summary
    fn print_hook_summary(&self, result: &HookResult) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        // Print status with color
        stdout
            .set_color(ColorSpec::new().set_fg(Some(result.status.to_color())))
            .unwrap();
        print!("{}", result.status.to_emoji());
        stdout.reset().unwrap();

        print!(" {} ", result.name);

        // Print duration
        print!("({:.2}s)", result.duration.as_secs_f64());

        // Print additional info
        if let Some(error_count) = result.error_count {
            if error_count > 0 {
                print!(" – {error_count} errors");
            }
        }

        if let Some(warning_count) = result.warning_count {
            if warning_count > 0 {
                print!(", {warning_count} warnings");
            }
        }

        if let Some(files_changed) = result.files_changed {
            if files_changed > 0 {
                print!(" – {files_changed} files");
            }
        }

        println!();
    }

    /// Print final summary
    fn print_final_summary(&self, results: &[HookResult], total_duration: Duration) {
        let passed = results
            .iter()
            .filter(|r| r.status == HookStatus::Passed)
            .count();
        let failed = results
            .iter()
            .filter(|r| r.status == HookStatus::Failed)
            .count();
        let warnings = results
            .iter()
            .filter(|r| r.status == HookStatus::Warning)
            .count();

        let overall_status = if failed > 0 {
            HookStatus::Failed
        } else if warnings > 0 {
            HookStatus::Warning
        } else {
            HookStatus::Passed
        };

        println!();
        println!("────────────────────────────");
        println!(
            "Hook Results: {} {}",
            overall_status.to_emoji(),
            if failed > 0 { "FAILED" } else { "PASSED" }
        );
        println!("Summary:");

        for result in results {
            let status_emoji = result.status.to_emoji();
            let status_text = match result.status {
                HookStatus::Passed => "Passed",
                HookStatus::Failed => "Failed",
                HookStatus::Warning => "Warning",
            };

            print!("- {}: {}", result.name, status_emoji);

            if let Some(error_count) = result.error_count {
                if error_count > 0 {
                    print!(" ({error_count} errors)");
                }
            }

            println!(" {status_text}");
        }

        println!();
        println!("Total time: {:.2}s", total_duration.as_secs_f64());

        if self.config.save_logs {
            println!("See logs:");
            for result in results {
                if result.log_file.exists() {
                    println!("  {}", result.log_file.display());
                }
            }
        }

        println!("────────────────────────────");
    }

    /// Emit hook event to event bus
    fn emit_hook_event(
        &self,
        result: &HookResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = HooksmithEvent::new(
            "hook-runner".to_string(),
            "hook_completed".to_string(),
            json!({
                "hook": result.name,
                "status": match result.status {
                    HookStatus::Passed => "passed",
                    HookStatus::Failed => "failed",
                    HookStatus::Warning => "warning",
                },
                "duration_ms": result.duration.as_millis(),
                "error_count": result.error_count,
                "warning_count": result.warning_count,
                "files_changed": result.files_changed,
                "error_message": result.error_message,
                "log_file": result.log_file.to_string_lossy(),
                "timestamp": Utc::now(),
            }),
        );

        emit_event(event)?;
        Ok(())
    }
}

/// Predefined hook configurations
pub struct HookConfigs;

impl HookConfigs {
    /// Get pre-commit hook configuration
    pub fn pre_commit() -> Vec<&'static str> {
        vec!["fmt", "validate-extensions", "clippy"]
    }

    /// Get pre-push hook configuration
    pub fn pre_push() -> Vec<&'static str> {
        vec!["validate-extensions", "clippy"]
    }

    /// Get all available hooks
    pub fn all() -> Vec<&'static str> {
        vec!["fmt", "validate-extensions", "clippy", "test"]
    }
}
