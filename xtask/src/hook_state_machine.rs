use crate::event_stream::{
    emit_error, emit_event, emit_info, emit_warn, Event, EventCategory, EventSeverity,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Hook state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookState {
    Idle,
    Validating,
    Committing,
    Pushing,
    Success,
    Error,
}

impl HookState {
    /// Convert state to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            HookState::Idle => "idle",
            HookState::Validating => "validating",
            HookState::Committing => "committing",
            HookState::Pushing => "pushing",
            HookState::Success => "success",
            HookState::Error => "error",
        }
    }

    /// Parse state from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(HookState::Idle),
            "validating" => Some(HookState::Validating),
            "committing" => Some(HookState::Committing),
            "pushing" => Some(HookState::Pushing),
            "success" => Some(HookState::Success),
            "error" => Some(HookState::Error),
            _ => None,
        }
    }
}

/// Hook event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEvent {
    RunValidation,
    Commit,
    Push,
    Retry,
    FileChanged,
    ManualTrigger,
}

/// Hook types that correspond to Git hooks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    PreCommit,
    PrePush,
    CommitMsg,
    PostCommit,
    AutoPush,
    Watchdog,
}

/// Hook context containing runtime information
#[derive(Debug, Clone)]
pub struct HookContext {
    pub hook_type: HookType,
    pub message: Option<String>,
    pub allow_empty_message: bool,
    pub skip_validation: bool,
    pub force: bool,
    pub args: Vec<String>,
    pub staged_files: Vec<PathBuf>,
    pub changed_files: Vec<PathBuf>,
    pub working_directory: PathBuf,
    pub timestamp: SystemTime,
}

impl HookContext {
    pub fn new(hook_type: HookType) -> Self {
        Self {
            hook_type,
            message: None,
            allow_empty_message: false,
            skip_validation: false,
            force: false,
            args: Vec::new(),
            staged_files: Vec::new(),
            changed_files: Vec::new(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            timestamp: SystemTime::now(),
        }
    }

    pub fn with_message(mut self, message: Option<String>) -> Self {
        self.message = message;
        self
    }

    pub fn with_validation_skip(mut self, skip: bool) -> Self {
        self.skip_validation = skip;
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

/// Hook result containing success/failure information
#[derive(Debug, Clone)]
pub struct HookResult {
    pub success: bool,
    pub state: HookState,
    pub message: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration: Duration,
}

impl HookResult {
    pub fn success(state: HookState, message: String) -> Self {
        Self {
            success: true,
            state,
            message,
            errors: Vec::new(),
            warnings: Vec::new(),
            duration: Duration::from_secs(0),
        }
    }

    pub fn error(state: HookState, message: String, errors: Vec<String>) -> Self {
        Self {
            success: false,
            state,
            message,
            errors,
            warnings: Vec::new(),
            duration: Duration::from_secs(0),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// HooksmithHook trait for implementing different hook types
pub trait HooksmithHook: Send + Sync {
    fn name(&self) -> &'static str;
    fn hook_type(&self) -> HookType;
    fn run(&self, ctx: &HookContext) -> Result<HookResult>;
    fn can_handle_event(&self, event: &HookEvent) -> bool;
}

/// Pre-commit hook implementation
pub struct PreCommitHook;

impl HooksmithHook for PreCommitHook {
    fn name(&self) -> &'static str {
        "pre-commit"
    }

    fn hook_type(&self) -> HookType {
        HookType::PreCommit
    }

    fn can_handle_event(&self, event: &HookEvent) -> bool {
        matches!(event, HookEvent::RunValidation)
    }

    fn run(&self, ctx: &HookContext) -> Result<HookResult> {
        let start_time = SystemTime::now();
        println!("🔍 Running pre-commit validation...");

        if ctx.skip_validation {
            println!("⚠️  Skipping validation checks");
            return Ok(HookResult::success(
                HookState::Success,
                "Pre-commit validation skipped".to_string(),
            ));
        }

        // Run cargo fix
        println!("   🔧 Running cargo fix...");
        let fix_status = Command::new("cargo")
            .args(["fix", "--allow-dirty", "--allow-staged"])
            .status()
            .context("Failed to run cargo fix")?;

        if !fix_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "cargo fix failed".to_string(),
                vec!["cargo fix failed".to_string()],
            ));
        }

        // Run cargo fmt
        println!("   🎨 Running cargo fmt...");
        let fmt_status = Command::new("cargo")
            .args(["fmt", "--all"])
            .status()
            .context("Failed to run cargo fmt")?;

        if !fmt_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "cargo fmt failed".to_string(),
                vec!["cargo fmt failed".to_string()],
            ));
        }

        // Run cargo clippy
        println!("   🔍 Running cargo clippy...");
        let clippy_status = Command::new("cargo")
            .args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .status()
            .context("Failed to run cargo clippy")?;

        if !clippy_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "cargo clippy failed".to_string(),
                vec!["cargo clippy failed".to_string()],
            ));
        }

        // Run contract validation
        println!("   📋 Running contract validation...");
        let contract_status = Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "contract-check", "--strict"])
            .status()
            .context("Failed to run contract validation")?;

        if !contract_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "Contract validation failed".to_string(),
                vec!["Contract validation failed".to_string()],
            ));
        }

        // Run generated file validation
        println!("   📄 Running generated file validation...");
        let generated_status = Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "validate-generated", "--strict"])
            .status()
            .context("Failed to run generated file validation")?;

        if !generated_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "Generated file validation failed".to_string(),
                vec!["Generated file validation failed".to_string()],
            ));
        }

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        println!("✅ Pre-commit validation completed successfully!");

        Ok(HookResult::success(
            HookState::Success,
            "Pre-commit validation passed".to_string(),
        )
        .with_duration(duration))
    }
}

/// Pre-push hook implementation
pub struct PrePushHook;

impl HooksmithHook for PrePushHook {
    fn name(&self) -> &'static str {
        "pre-push"
    }

    fn hook_type(&self) -> HookType {
        HookType::PrePush
    }

    fn can_handle_event(&self, event: &HookEvent) -> bool {
        matches!(event, HookEvent::Push)
    }

    fn run(&self, ctx: &HookContext) -> Result<HookResult> {
        let start_time = SystemTime::now();
        println!("🚀 Running pre-push validation...");

        // For pre-push, we run the same validation as pre-commit
        let pre_commit = PreCommitHook;
        let result = pre_commit.run(ctx)?;

        if result.success {
            let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
            println!("✅ Pre-push validation completed successfully!");
            Ok(result.with_duration(duration))
        } else {
            Ok(result)
        }
    }
}

/// Auto-push hook implementation that handles the full workflow
pub struct AutoPushHook;

impl HooksmithHook for AutoPushHook {
    fn name(&self) -> &'static str {
        "auto-push"
    }

    fn hook_type(&self) -> HookType {
        HookType::AutoPush
    }

    fn can_handle_event(&self, event: &HookEvent) -> bool {
        matches!(
            event,
            HookEvent::RunValidation | HookEvent::Commit | HookEvent::Push
        )
    }

    fn run(&self, ctx: &HookContext) -> Result<HookResult> {
        let start_time = SystemTime::now();
        println!("🚀 Starting automated git workflow...");

        // Step 1: Run validation checks (unless skipped)
        if !ctx.skip_validation {
            println!("🔍 Running validation checks...");
            let pre_commit = PreCommitHook;
            let validation_result = pre_commit.run(ctx)?;

            if !validation_result.success {
                return Ok(validation_result);
            }
        } else {
            println!("⚠️  Skipping validation checks");
        }

        // Step 2: Check if there are any changes to commit
        println!("📊 Checking for changes...");
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to check git status")?;

        let status_text = String::from_utf8_lossy(&status_output.stdout);
        if status_text.trim().is_empty() {
            println!("✅ No changes to commit");
            return Ok(HookResult::success(
                HookState::Success,
                "No changes to commit".to_string(),
            ));
        }

        println!("📝 Found changes to commit:");
        for line in status_text.lines() {
            if !line.trim().is_empty() {
                println!("   {line}");
            }
        }

        // Step 3: Add all changes
        println!("📦 Adding changes...");
        let add_status = Command::new("git")
            .args(["add", "."])
            .status()
            .context("Failed to add changes")?;

        if !add_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "git add failed".to_string(),
                vec!["git add failed".to_string()],
            ));
        }

        // Step 4: Get commit message
        let commit_message = if let Some(msg) = &ctx.message {
            msg.clone()
        } else {
            // Prompt for commit message
            println!("💬 Enter commit message (or press Enter for empty message):");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        };

        // Step 5: Commit changes
        println!("💾 Committing changes...");
        let mut commit_args = vec!["commit"];

        if ctx.allow_empty_message || commit_message.is_empty() {
            commit_args.extend_from_slice(&["--allow-empty-message", "-m", ""]);
        } else {
            commit_args.extend_from_slice(&["-m", &commit_message]);
        }

        // Add any additional arguments
        for arg in &ctx.args {
            commit_args.push(arg);
        }

        let commit_status = Command::new("git")
            .args(&commit_args)
            .status()
            .context("Failed to commit changes")?;

        if !commit_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "git commit failed".to_string(),
                vec!["git commit failed".to_string()],
            ));
        }

        // Step 6: Push changes
        println!("🚀 Pushing changes...");
        let mut push_args = vec!["push"];

        if ctx.force {
            push_args.push("--force");
            println!("⚠️  Force pushing (use with caution!)");
        }

        let push_status = Command::new("git")
            .args(&push_args)
            .status()
            .context("Failed to push changes")?;

        if !push_status.success() {
            return Ok(HookResult::error(
                HookState::Error,
                "git push failed".to_string(),
                vec!["git push failed".to_string()],
            ));
        }

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        println!("✅ Automated git workflow completed successfully!");
        println!(
            "   📝 Committed with message: {}",
            if commit_message.is_empty() {
                "(empty)"
            } else {
                &commit_message
            }
        );
        println!("   🚀 Pushed to remote repository");

        Ok(HookResult::success(
            HookState::Success,
            "Automated git workflow completed successfully".to_string(),
        )
        .with_duration(duration))
    }
}

/// Hook state machine that manages hook execution
pub struct HookStateMachine {
    hooks: Vec<Box<dyn HooksmithHook>>,
    current_state: HookState,
    context: HookContext,
}

impl HookStateMachine {
    /// Create a new hook state machine
    pub fn new(hook_type: HookType) -> Self {
        let mut hooks: Vec<Box<dyn HooksmithHook>> = Vec::new();

        // Add default hooks based on type
        match hook_type {
            HookType::PreCommit => {
                hooks.push(Box::new(PreCommitHook));
            }
            HookType::PrePush => {
                hooks.push(Box::new(PrePushHook));
            }
            HookType::AutoPush => {
                hooks.push(Box::new(AutoPushHook));
            }
            _ => {}
        }

        Self {
            hooks,
            current_state: HookState::Idle,
            context: HookContext::new(hook_type),
        }
    }

    /// Add a custom hook to the state machine
    pub fn add_hook(&mut self, hook: Box<dyn HooksmithHook>) {
        self.hooks.push(hook);
    }

    /// Set the hook context
    pub fn with_context(mut self, context: HookContext) -> Self {
        self.context = context;
        self
    }

    /// Set the hook context (mutable reference version)
    pub fn set_context(&mut self, context: HookContext) {
        self.context = context;
    }

    /// Get the current state
    pub fn current_state(&self) -> &HookState {
        &self.current_state
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: HookState) {
        println!(
            "🔄 State transition: {} → {}",
            self.current_state.to_string(),
            new_state.to_string()
        );
        self.current_state = new_state;
    }

    /// Handle a hook event
    pub fn handle_event(&mut self, event: HookEvent) -> Result<HookResult> {
        let start_time = std::time::Instant::now();

        // Emit event received
        emit_info(
            EventCategory::HookStateMachine,
            "event_received",
            &format!("Received event: {:?}", event),
            "hook_state_machine",
        )?;

        println!("📡 Handling event: {:?}", event);

        let result = match event {
            HookEvent::RunValidation => {
                self.transition_to(HookState::Validating);
                self.run_validation()
            }
            HookEvent::Commit => {
                self.transition_to(HookState::Committing);
                self.run_commit()
            }
            HookEvent::Push => {
                self.transition_to(HookState::Pushing);
                self.run_push()
            }
            HookEvent::Retry => {
                self.transition_to(HookState::Idle);
                Ok(HookResult::success(
                    HookState::Idle,
                    "Reset to idle state for retry".to_string(),
                ))
            }
            HookEvent::FileChanged => {
                // For watchdog mode, restart validation
                if self.current_state == HookState::Idle {
                    self.handle_event(HookEvent::RunValidation)
                } else {
                    Ok(HookResult::success(
                        self.current_state.clone(),
                        "File change detected, but already processing".to_string(),
                    ))
                }
            }
            HookEvent::ManualTrigger => {
                // For auto-push, run the full workflow
                match self.context.hook_type {
                    HookType::AutoPush => {
                        self.transition_to(HookState::Validating);
                        self.run_full_workflow()
                    }
                    _ => self.handle_event(HookEvent::RunValidation),
                }
            }
        };

        // Emit event completion
        let duration = start_time.elapsed();
        if result.is_ok() {
            emit_info(
                EventCategory::HookStateMachine,
                "event_completed",
                &format!("Event {:?} completed successfully", event),
                "hook_state_machine",
            )?;
        } else {
            emit_error(
                EventCategory::HookStateMachine,
                "event_failed",
                &format!("Event {:?} failed", event),
                "hook_state_machine",
            )?;
        }

        result
    }

    /// Run validation phase
    fn run_validation(&self) -> Result<HookResult> {
        for hook in &self.hooks {
            if hook.can_handle_event(&HookEvent::RunValidation) {
                return hook.run(&self.context);
            }
        }

        Ok(HookResult::error(
            HookState::Error,
            "No validation hook found".to_string(),
            vec!["No validation hook found".to_string()],
        ))
    }

    /// Run commit phase
    fn run_commit(&self) -> Result<HookResult> {
        // This would be implemented for commit-specific hooks
        Ok(HookResult::success(
            HookState::Success,
            "Commit phase completed".to_string(),
        ))
    }

    /// Run push phase
    fn run_push(&self) -> Result<HookResult> {
        for hook in &self.hooks {
            if hook.can_handle_event(&HookEvent::Push) {
                return hook.run(&self.context);
            }
        }

        Ok(HookResult::error(
            HookState::Error,
            "No push hook found".to_string(),
            vec!["No push hook found".to_string()],
        ))
    }

    /// Run the full auto-push workflow
    fn run_full_workflow(&self) -> Result<HookResult> {
        for hook in &self.hooks {
            if hook.hook_type() == HookType::AutoPush {
                return hook.run(&self.context);
            }
        }

        Ok(HookResult::error(
            HookState::Error,
            "No auto-push hook found".to_string(),
            vec!["No auto-push hook found".to_string()],
        ))
    }

    /// Run in watchdog mode
    pub async fn run_watchdog(&mut self, interval: u64) -> Result<()> {
        println!("🔄 Starting watchdog mode with {interval}s interval...");
        println!("   Press Ctrl+C to stop");

        loop {
            match self.handle_event(HookEvent::ManualTrigger) {
                Ok(result) => {
                    if result.success {
                        println!("✅ Watchdog cycle completed successfully");
                        self.transition_to(HookState::Success);
                    } else {
                        println!("❌ Watchdog cycle failed: {}", result.message);
                        self.transition_to(HookState::Error);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Watchdog cycle error: {e}");
                    self.transition_to(HookState::Error);
                }
            }

            // Reset to idle for next cycle
            self.transition_to(HookState::Idle);

            println!("⏰ Waiting {interval} seconds before next cycle...");
            sleep(Duration::from_secs(interval)).await;
        }
    }
}

/// Hook manager that coordinates multiple hooks
pub struct HookManager {
    state_machines: HashMap<HookType, HookStateMachine>,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        Self {
            state_machines: HashMap::new(),
        }
    }

    /// Register a hook state machine
    pub fn register_hook(&mut self, hook_type: HookType, state_machine: HookStateMachine) {
        self.state_machines.insert(hook_type, state_machine);
    }

    /// Run a specific hook type
    pub fn run_hook(&mut self, hook_type: HookType, context: HookContext) -> Result<HookResult> {
        if let Some(state_machine) = self.state_machines.get_mut(&hook_type) {
            state_machine.set_context(context);
            state_machine.handle_event(HookEvent::ManualTrigger)
        } else {
            Ok(HookResult::error(
                HookState::Error,
                format!("No hook registered for type: {:?}", hook_type),
                vec![format!("No hook registered for type: {:?}", hook_type)],
            ))
        }
    }

    /// Start watchdog mode for a specific hook type
    pub async fn start_watchdog(&mut self, hook_type: HookType, interval: u64) -> Result<()> {
        if let Some(state_machine) = self.state_machines.get_mut(&hook_type) {
            state_machine.run_watchdog(interval).await
        } else {
            anyhow::bail!("No hook registered for type: {:?}", hook_type);
        }
    }

    /// Get all registered hook types
    pub fn registered_hooks(&self) -> Vec<HookType> {
        self.state_machines.keys().cloned().collect()
    }
}

impl Default for HookManager {
    fn default() -> Self {
        let mut manager = Self::new();

        // Register default hooks
        manager.register_hook(
            HookType::PreCommit,
            HookStateMachine::new(HookType::PreCommit),
        );
        manager.register_hook(HookType::PrePush, HookStateMachine::new(HookType::PrePush));
        manager.register_hook(
            HookType::AutoPush,
            HookStateMachine::new(HookType::AutoPush),
        );

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_state_creation() {
        let state = HookState::Idle;
        assert_eq!(state.to_string(), "idle");
        assert_eq!(HookState::from_string("idle"), Some(HookState::Idle));
    }

    #[test]
    fn test_hook_context_creation() {
        let ctx = HookContext::new(HookType::PreCommit);
        assert_eq!(ctx.hook_type, HookType::PreCommit);
        assert_eq!(ctx.skip_validation, false);
    }

    #[test]
    fn test_hook_result_creation() {
        let success_result = HookResult::success(HookState::Success, "Test success".to_string());
        assert!(success_result.success);

        let error_result = HookResult::error(
            HookState::Error,
            "Test error".to_string(),
            vec!["Error 1".to_string()],
        );
        assert!(!error_result.success);
        assert_eq!(error_result.errors.len(), 1);
    }

    #[test]
    fn test_pre_commit_hook() {
        let hook = PreCommitHook;
        assert_eq!(hook.name(), "pre-commit");
        assert_eq!(hook.hook_type(), HookType::PreCommit);
        assert!(hook.can_handle_event(&HookEvent::RunValidation));
        assert!(!hook.can_handle_event(&HookEvent::Push));
    }

    #[test]
    fn test_hook_state_machine_creation() {
        let mut state_machine = HookStateMachine::new(HookType::PreCommit);
        assert_eq!(*state_machine.current_state(), HookState::Idle);

        state_machine.transition_to(HookState::Validating);
        assert_eq!(*state_machine.current_state(), HookState::Validating);
    }

    #[test]
    fn test_hook_manager_creation() {
        let manager = HookManager::default();
        let registered_hooks = manager.registered_hooks();
        assert!(registered_hooks.contains(&HookType::PreCommit));
        assert!(registered_hooks.contains(&HookType::PrePush));
        assert!(registered_hooks.contains(&HookType::AutoPush));
    }
}
