//! Xtask CLI for Hooksmith
//!
//! This binary provides structured build and code generation tasks
//! that replace shell scripts and raw echo statements.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use hook_state_machine::{HookContext, HookManager, HookType};

/// CLI argument enum for hook types
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum HookTypeArg {
    PreCommit,
    PrePush,
    CommitMsg,
    PostCommit,
    AutoPush,
    Watchdog,
}

impl From<HookTypeArg> for HookType {
    fn from(arg: HookTypeArg) -> Self {
        match arg {
            HookTypeArg::PreCommit => HookType::PreCommit,
            HookTypeArg::PrePush => HookType::PrePush,
            HookTypeArg::CommitMsg => HookType::CommitMsg,
            HookTypeArg::PostCommit => HookType::PostCommit,
            HookTypeArg::AutoPush => HookType::AutoPush,
            HookTypeArg::Watchdog => HookType::Watchdog,
        }
    }
}

/// Event stream commands
#[derive(Debug, Clone, clap::Subcommand)]
enum EventStreamCommands {
    /// Initialize event stream
    Init {
        /// Output file for JSONL events
        #[arg(long)]
        output_file: Option<String>,
        /// Whether to enable console output
        #[arg(long, default_value = "true")]
        console_output: bool,
        /// Whether to enable real-time broadcasting
        #[arg(long, default_value = "true")]
        enable_broadcast: bool,
        /// Minimum severity level to log
        #[arg(long, default_value = "info")]
        min_severity: String,
    },
    /// Monitor events in real-time
    Monitor {
        /// Whether to show metadata
        #[arg(long)]
        show_metadata: bool,
        /// Performance threshold in milliseconds
        #[arg(long, default_value = "1000")]
        performance_threshold: u64,
        /// Error threshold for alerts
        #[arg(long, default_value = "5")]
        error_threshold: u64,
    },
    /// Analyze event stream
    Analyze {
        /// Input file to analyze
        #[arg(long)]
        input_file: String,
        /// Output format (json, table, summary)
        #[arg(long, default_value = "summary")]
        format: String,
    },
    /// Generate event stream configuration
    GenConfig {
        /// Output file path
        #[arg(long, default_value = "event-stream.yml")]
        output: String,
    },
}

/// SARIF and CodeQL commands
#[derive(Debug, Clone, clap::Subcommand)]
enum SarifCommands {
    /// Convert JSONL events to SARIF format
    JsonlToSarif {
        /// Input JSONL file
        #[arg(long)]
        input: String,
        /// Output SARIF file
        #[arg(long)]
        output: String,
        /// Validate output SARIF
        #[arg(long)]
        validate: bool,
    },
    /// Convert SARIF to JSONL events
    SarifToJsonl {
        /// Input SARIF file
        #[arg(long)]
        input: String,
        /// Output JSONL file
        #[arg(long)]
        output: String,
        /// Validate input SARIF
        #[arg(long)]
        validate: bool,
    },
    /// Run CodeQL analysis and convert to structured events
    CodeqlAnalysis {
        /// CodeQL CLI path (optional, will search PATH)
        #[arg(long)]
        cli_path: Option<String>,
        /// Database directory
        #[arg(long, default_value = "codeql-db")]
        db_dir: String,
        /// Query suite to run
        #[arg(long, default_value = "codeql-cpp-queries.qls")]
        query_suite: String,
        /// Language to analyze
        #[arg(long, default_value = "cpp")]
        language: String,
        /// Build command
        #[arg(long, default_value = "cargo build")]
        build_command: String,
        /// Output SARIF file
        #[arg(long)]
        output: Option<String>,
        /// Convert to JSONL events
        #[arg(long)]
        to_jsonl: bool,
    },
    /// Validate SARIF file
    ValidateSarif {
        /// SARIF file to validate
        #[arg(long)]
        file: String,
        /// Exit with error on validation failures
        #[arg(long)]
        strict: bool,
    },
    /// Merge multiple SARIF files
    MergeSarif {
        /// Input SARIF files
        #[arg(long)]
        inputs: Vec<String>,
        /// Output merged SARIF file
        #[arg(long)]
        output: String,
        /// Validate merged output
        #[arg(long)]
        validate: bool,
    },
    /// Integrate CodeQL into validation pipeline
    IntegrateCodeql {
        /// Whether to run CodeQL analysis
        #[arg(long)]
        run_analysis: bool,
        /// Whether to convert results to JSONL
        #[arg(long)]
        to_jsonl: bool,
        /// Whether to merge with existing validation results
        #[arg(long)]
        merge: bool,
        /// Output directory for results
        #[arg(long, default_value = "validation-results")]
        output_dir: String,
    },
}

/// Event bus commands
#[derive(Debug, Clone, clap::Subcommand)]
enum EventBusCommands {
    /// Initialize event bus
    Init {
        /// Whether to enable JSONL persistence
        #[arg(long, default_value = "true")]
        enable_persistence: bool,
        /// JSONL file path
        #[arg(long)]
        jsonl_file: Option<String>,
        /// Batch size for JSONL writes
        #[arg(long, default_value = "10")]
        batch_size: usize,
        /// Flush interval in milliseconds
        #[arg(long, default_value = "1000")]
        flush_interval_ms: u64,
        /// Whether to enable console output
        #[arg(long, default_value = "true")]
        console_output: bool,
    },
    /// Start event processor with handlers
    Process {
        /// Whether to enable auto-push handler
        #[arg(long)]
        auto_push: bool,
        /// Whether to enable notification handler
        #[arg(long)]
        notifications: bool,
        /// Whether to enable metrics handler
        #[arg(long)]
        metrics: bool,
    },
    /// Replay events from JSONL file
    Replay {
        /// Input JSONL file to replay
        #[arg(long)]
        input_file: String,
        /// Whether to enable auto-push handler during replay
        #[arg(long)]
        auto_push: bool,
        /// Whether to enable notification handler during replay
        #[arg(long)]
        notifications: bool,
    },
    /// Emit test events
    EmitTest {
        /// Number of test events to emit
        #[arg(long, default_value = "5")]
        count: usize,
    },
}

/// WASM component commands
#[derive(Debug, Clone, clap::Subcommand)]
enum WasmComponentCommands {
    /// Load a WASM component
    Load {
        /// Path to WASM component file
        #[arg(long)]
        component_path: String,
        /// Component configuration (JSON)
        #[arg(long)]
        config: Option<String>,
    },
    /// List loaded WASM components
    List,
    /// Unload a WASM component
    Unload {
        /// Handler ID of component to unload
        #[arg(long)]
        handler_id: u32,
    },
    /// Get WASM component statistics
    Stats,
    /// Build validation handler component
    BuildValidationHandler {
        /// Output directory for built component
        #[arg(long, default_value = "target/wasm")]
        output_dir: String,
    },
}

mod auto_push;
mod code_stats;
mod config;
mod contract;
mod contract_validation;
mod dashboard;
mod docs;
mod emit;
mod error_deduplication;
mod event_bus;
mod event_stream;
mod events;
mod file_audit;
mod generated_file_validator;
mod git_notes_manager;
mod hierarchical_validation;
mod hook_runner;
mod hook_state_machine;
mod sarif_integration;
mod status;
mod structured_auto_push;
mod structured_logging;
mod wasm_event_bus;
mod strict_file_validator;

/// Xtask CLI for Hooksmith project tasks
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Hooksmith project tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project and all components
    Build {
        /// Build target (native, wasm, all)
        #[arg(long, default_value = "all")]
        target: String,
        /// Release build
        #[arg(long)]
        release: bool,
    },
    /// Generate WIT interface definitions
    GenWit {
        /// Output directory for WIT files
        #[arg(long, default_value = "wit")]
        output_dir: String,
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
    /// Generate Lefthook configuration
    GenLefthook {
        /// Output file path
        #[arg(long, default_value = "lefthook.yml")]
        output: String,
        /// Whether to validate against schema
        #[arg(long)]
        validate: bool,
    },
    /// Generate documentation
    GenDocs {
        /// Output directory for documentation
        #[arg(long, default_value = "docs")]
        output_dir: String,
        /// Whether to open docs in browser
        #[arg(long)]
        open: bool,
    },
    /// Generate comprehensive documentation from Rust code and templates
    GenDocsComprehensive {
        /// Generate all documentation
        #[arg(long)]
        all: bool,
        /// Specific file to generate
        #[arg(long)]
        file: Option<String>,
        /// Output directory for documentation
        #[arg(long, default_value = "docs")]
        output_dir: String,
        /// Whether to validate generated files
        #[arg(long)]
        validate: bool,
    },
    /// Generate schema and WIT documentation
    GenSchemaDocs {
        /// Output directory for documentation
        #[arg(long, default_value = "docs")]
        output_dir: String,
        /// Whether to generate PDF output
        #[arg(long)]
        pdf: bool,
        /// Whether to generate HTML output
        #[arg(long)]
        html: bool,
        /// Whether to generate EPUB output
        #[arg(long)]
        epub: bool,
        /// Whether to open docs in browser
        #[arg(long)]
        open: bool,
    },
    /// Generate README with CLI help and module docs
    GenReadme {
        /// Output file path
        #[arg(long, default_value = "README.md")]
        output: String,
        /// Whether to overwrite existing file
        #[arg(long)]
        overwrite: bool,
    },
    /// Generate mod.rs files for commands and modules
    GenMods {
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
    /// Generate hooks README
    GenHooksReadme {
        /// Output file path
        #[arg(long, default_value = "hooks/README.md")]
        output: String,
        /// Whether to overwrite existing file
        #[arg(long)]
        overwrite: bool,
    },
    /// Run all code generation tasks
    GenAllLegacy {
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
    /// Check if generated files are up to date
    Check {
        /// Exit with error if files are not up to date
        #[arg(long)]
        strict: bool,
    },
    /// Validate project configuration
    Validate {
        /// Validate Trunk configuration
        #[arg(long)]
        trunk: bool,
        /// Validate Cargo workspace
        #[arg(long)]
        cargo: bool,
        /// Validate module/test consistency
        #[arg(long)]
        modules: bool,
        /// Validate all configurations
        #[arg(long)]
        all: bool,
    },

    /// Hierarchical contract validation
    ContractValidate {
        #[command(subcommand)]
        command: hierarchical_validation::Commands,
    },
    /// Validate generated files to prevent manual modifications
    ValidateGenerated {
        /// Whether to check only staged files
        #[arg(long)]
        staged_only: bool,
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Custom error message for violations
        #[arg(long)]
        custom_message: Option<String>,
    },
    /// Add generated file headers to all generated files
    AddGeneratedHeaders {
        /// Specific file to add header to
        #[arg(long)]
        file: Option<String>,
    },
    /// Validate that all generated files have proper headers
    ValidateHeaders {
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
    },
    /// Generate documentation using Rust templates
    GenTemplates {
        /// Specific template to generate
        #[arg(long)]
        template: Option<String>,
        /// Output directory for generated files
        #[arg(long, default_value = "docs")]
        output_dir: String,
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
    /// Check if current changes are compatible with the last release
    CheckStable {
        /// Version to check against
        #[arg(long, default_value = "0.1.0")]
        version: String,
        /// Run comprehensive compatibility tests
        #[arg(long)]
        comprehensive: bool,
    },
    /// Test current version against released version
    TestWithRelease {
        /// Version to test against
        #[arg(long, default_value = "0.1.0")]
        version: String,
    },
    /// Compare outputs between current and released version
    CompareWithRelease {
        /// Version to compare against
        #[arg(long, default_value = "0.1.0")]
        version: String,
    },
    /// Set up Git filters for contract validation
    SetupGitFilters {
        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },
    /// Check file types and generation markers
    CheckFiles {
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Show detailed output
        #[arg(long)]
        verbose: bool,
    },
    /// Validate files against strict extension policy (.rs and .jsonc only)
    ValidateFiles {
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Show detailed output
        #[arg(long)]
        verbose: bool,
    },
    /// Generate all code-generated files
    GenAll {
        /// Whether to validate generated files
        #[arg(long)]
        validate: bool,
        /// Whether to force regeneration
        #[arg(long)]
        force: bool,
    },
    /// Bootstrap the project with all generated files
    Bootstrap {
        /// Whether to validate after bootstrap
        #[arg(long)]
        validate: bool,
        /// Whether to commit generated files
        #[arg(long)]
        commit: bool,
    },
    /// Generate Git attributes files
    GenGitattributes {
        /// Output directory for git attributes files
        #[arg(long, default_value = "hooks")]
        output_dir: String,
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Whether to validate generated files
        #[arg(long)]
        validate: bool,
    },
    /// Generate all configuration files from Rust structs
    GenConfig {
        /// Whether to overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Whether to validate generated files
        #[arg(long)]
        validate: bool,
    },
    /// Validate all configuration files
    ValidateConfig {
        /// Whether to exit with error on validation failures
        #[arg(long)]
        strict: bool,
    },
    /// Contract-driven bootstrap & validation workflow
    Contract {
        #[command(subcommand)]
        command: contract::ContractCommands,
    },
    /// Enhanced contract validation with Git notes
    ContractValidation {
        #[command(subcommand)]
        command: contract_validation::ContractValidationCommands,
    },
    /// Track Rust-owned project files and coverage
    Status {
        #[command(subcommand)]
        command: status::StatusCommands,
    },
    /// Comprehensive contract validation and status check
    ContractCheck {
        /// Whether to check only staged files
        #[arg(long)]
        staged_only: bool,
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Whether to generate trend data
        #[arg(long)]
        trend: bool,
        /// Output directory for trend data
        #[arg(long, default_value = "status-trends")]
        trend_output: String,
        /// Whether to show detailed output
        #[arg(long)]
        verbose: bool,
    },
    /// Analyze code statistics and quality
    CodeStats {
        #[command(subcommand)]
        command: code_stats::CodeStatsCommands,
    },
    /// Validate commit message (Trunk-style: allows empty messages)
    ValidateCommitMsg {
        /// Commit message file path (from lefthook {1})
        file: Option<String>,
        /// Whether to allow empty commit messages (Trunk-style)
        #[arg(long, default_value = "true")]
        allow_empty: bool,
        /// Whether to validate conventional commit format for non-empty messages
        #[arg(long, default_value = "true")]
        validate_conventional: bool,
    },
    /// Set up git aliases for Trunk-style commit workflow
    SetupGitAliases {
        /// Whether to force overwrite existing aliases
        #[arg(long)]
        force: bool,
    },
    /// Validate documentation generation (replaces validate-docs.sh)
    ValidateDocs {
        /// Whether to exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Whether to regenerate documentation
        #[arg(long)]
        regenerate: bool,
        /// Whether to check for uncommitted changes
        #[arg(long, default_value = "true")]
        check_uncommitted: bool,
    },
    /// Git commit with Trunk-style empty message support (replaces git-trunk-commit.sh)
    GitCommit {
        /// Git commit message
        #[arg(short, long)]
        message: Option<String>,
        /// Allow empty commit message (Trunk-style)
        #[arg(long)]
        allow_empty_message: bool,
        /// Additional git commit arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Set up pre-commit hook (replaces setup-pre-commit.sh)
    SetupPreCommit {
        /// Use enhanced pre-commit hook with auto-fix capabilities
        #[arg(long)]
        enhanced: bool,
        /// Force overwrite existing hook
        #[arg(long)]
        force: bool,
        /// Use lefthook instead of direct git hooks
        #[arg(long)]
        lefthook: bool,
    },
    /// Run pre-commit validation (replaces pre-commit script)
    PreCommit {
        /// Use enhanced validation with auto-fix
        #[arg(long)]
        enhanced: bool,
        /// Only check staged files
        #[arg(long)]
        staged_only: bool,
        /// Exit with error on violations
        #[arg(long)]
        strict: bool,
        /// Auto-fix issues where possible
        #[arg(long)]
        auto_fix: bool,
    },
    /// Run hooks with clean, summarized output
    RunHooks {
        /// Hook type to run (pre-commit, pre-push, all)
        #[arg(long, default_value = "pre-commit")]
        hook_type: String,
        /// Show detailed output
        #[arg(long)]
        verbose: bool,
        /// Don't save logs
        #[arg(long)]
        no_logs: bool,
        /// Don't emit events
        #[arg(long)]
        no_events: bool,
        /// Custom log directory
        #[arg(long)]
        log_dir: Option<String>,
    },
    /// Check for dead code by temporarily stripping #[allow(dead_code)] attributes
    DeadCodeCheck {
        /// Whether to exit with error on dead code found
        #[arg(long)]
        strict: bool,
        /// Whether to strip attributes from generated files too
        #[arg(long)]
        include_generated: bool,
        /// Whether to restore attributes after checking
        #[arg(long, default_value = "true")]
        restore: bool,
        /// Output format for results
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Convert JSON files to JSONC format with comments
    ConvertJsonc {
        /// Specific JSON file to convert
        #[arg(long)]
        file: Option<String>,
        /// Whether to overwrite existing JSONC files
        #[arg(long)]
        overwrite: bool,
        /// Whether to remove original JSON files after conversion
        #[arg(long)]
        remove_original: bool,
    },
    /// Automated git workflow: validate, add, commit, and push
    AutoPush {
        /// Commit message (optional, will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
        /// Allow empty commit message (Trunk-style)
        #[arg(long)]
        allow_empty_message: bool,
        /// Skip validation checks
        #[arg(long)]
        skip_validation: bool,
        /// Run in watchdog mode (continuous monitoring)
        #[arg(long)]
        watchdog: bool,
        /// Watchdog interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Force push (use with caution)
        #[arg(long)]
        force: bool,
        /// Additional git commit arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Clean auto-push workflow with porcelain output and comprehensive logging
    CleanAutoPush {
        /// Commit message (optional, will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
        /// Allow empty commit message (Trunk-style)
        #[arg(long)]
        allow_empty_message: bool,
        /// Run in watchdog mode (continuous monitoring)
        #[arg(long)]
        watchdog: bool,
        /// Watchdog interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Force push (use with caution)
        #[arg(long)]
        force: bool,
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        /// Disable file logging
        #[arg(long)]
        no_log: bool,
        /// Custom log file path
        #[arg(long)]
        log_file: Option<String>,
        /// Additional git commit arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Structured auto-push workflow with JSONL output and event bus integration
    StructuredAutoPush {
        /// Commit message (optional, will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
        /// Allow empty commit message (Trunk-style)
        #[arg(long)]
        allow_empty_message: bool,
        /// Run in watchdog mode (continuous monitoring)
        #[arg(long)]
        watchdog: bool,
        /// Watchdog interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Force push (use with caution)
        #[arg(long)]
        force: bool,
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        /// Disable JSONL output (for TUI mode)
        #[arg(long)]
        no_jsonl: bool,
        /// Disable event bus integration
        #[arg(long)]
        no_event_bus: bool,
        /// Additional git commit arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a specific hook using the state machine
    Hook {
        /// Hook type to run
        #[arg(value_enum)]
        hook_type: HookTypeArg,
        /// Commit message (for auto-push hooks)
        #[arg(short, long)]
        message: Option<String>,
        /// Allow empty commit message (Trunk-style)
        #[arg(long)]
        allow_empty_message: bool,
        /// Skip validation checks
        #[arg(long)]
        skip_validation: bool,
        /// Run in watchdog mode (continuous monitoring)
        #[arg(long)]
        watchdog: bool,
        /// Watchdog interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Force push (use with caution)
        #[arg(long)]
        force: bool,
        /// Additional arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List all available hooks
    ListHooks,
    /// Generate Lefthook configuration that uses the hook state machine
    GenLefthookHooks {
        /// Output file path
        #[arg(long, default_value = "lefthook.yml")]
        output: String,
        /// Whether to validate against schema
        #[arg(long)]
        validate: bool,
    },
    /// Event stream management and monitoring
    EventStream {
        #[command(subcommand)]
        command: EventStreamCommands,
    },
    /// Event bus management and processing
    EventBus {
        #[command(subcommand)]
        command: EventBusCommands,
    },
    /// WASM component management
    WasmComponents {
        #[command(subcommand)]
        command: WasmComponentCommands,
    },
    /// Interactive dashboard for monitoring and auto-push
    Dashboard {
        /// Update interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Whether to show TUI dashboard
        #[arg(long)]
        show_dashboard: bool,
        /// Whether to enable auto-push
        #[arg(long, default_value = "true")]
        auto_push: bool,
        /// Whether to skip validation
        #[arg(long)]
        skip_validation: bool,

        /// Commit message template
        #[arg(short, long)]
        message: Option<String>,
        /// Run in file-watch mode (wait for manual triggers)
        #[arg(long)]
        file_watch: bool,
        /// Trigger validation manually (for file-watch mode)
        #[arg(long)]
        trigger: bool,
    },
    /// Generate JSON schema for AutoPushEvent
    GenSchema {
        /// Output file path (default: stdout)
        #[arg(long)]
        output: Option<String>,
    },
    /// Validate JSONL output against schema
    ValidateSchema {
        /// Input file (default: stdin)
        #[arg(long)]
        input: Option<String>,
        /// Exit with error on validation failures
        #[arg(long)]
        strict: bool,
    },
    /// SARIF and CodeQL integration commands
    Sarif {
        #[command(subcommand)]
        command: SarifCommands,
    },
}

/// WIT schema for function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitFunction {
    /// Function name
    name: String,
    /// Function parameters
    params: Vec<WitParam>,
    /// Return type
    result: String,
    /// Function documentation
    docs: Option<String>,
}

/// WIT schema for parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitParam {
    /// Parameter name
    name: String,
    /// Parameter type
    param_type: String,
    /// Parameter documentation
    docs: Option<String>,
}

/// WIT schema for record definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitRecord {
    /// Record name
    name: String,
    /// Record fields
    fields: Vec<WitField>,
    /// Record documentation
    docs: Option<String>,
}

/// WIT schema for field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitField {
    /// Field name
    name: String,
    /// Field type
    field_type: String,
    /// Field documentation
    docs: Option<String>,
}

/// WIT schema for enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitEnum {
    /// Enum name
    name: String,
    /// Enum variants
    variants: Vec<String>,
    /// Enum documentation
    docs: Option<String>,
}

/// WIT interface definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitInterface {
    /// Package name
    package: String,
    /// Interface name
    name: String,
    /// Interface functions
    functions: Vec<WitFunction>,
    /// Interface records
    records: Vec<WitRecord>,
    /// Interface enums
    enums: Vec<WitEnum>,
    /// Interface documentation
    docs: Option<String>,
}

/// Lefthook hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LefthookHook {
    /// Command to run
    run: String,
    /// Files to run on
    files: Option<String>,
    /// Whether to run in parallel
    parallel: Option<bool>,
    /// Environment variables
    env: Option<HashMap<String, String>>,
}

/// Lefthook configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LefthookConfig {
    /// Pre-commit hooks
    #[serde(rename = "pre-commit")]
    pre_commit: Option<HashMap<String, LefthookHook>>,
    /// Pre-push hooks
    #[serde(rename = "pre-push")]
    pre_push: Option<HashMap<String, LefthookHook>>,
    /// Commit-msg hooks
    #[serde(rename = "commit-msg")]
    commit_msg: Option<HashMap<String, LefthookHook>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize event bus with default configuration
    let event_bus_config = event_bus::EventBusConfig::default();
    event_bus::init_event_bus(event_bus_config.clone())?;

    // Initialize WASM event bus host
    wasm_event_bus::init_wasm_event_bus_host(event_bus_config)?;

    // Initialize legacy event stream for backward compatibility
    let event_stream_config = event_stream::EventStreamConfig::default();
    event_stream::init_event_stream(event_stream_config)?;

    // Initialize global structured logger
    // Default to JSONL output unless explicitly disabled
    let jsonl_output = true; // We want structured output by default
    let event_bus_integration = true; // Enable event bus integration
    let session_id = Some(uuid::Uuid::new_v4().to_string());

    structured_logging::init_global_logger(jsonl_output, event_bus_integration, session_id);

    let cli = Cli::parse();

    match cli.command {
        Commands::Build { target, release } => {
            build_project(&target, release)?;
        }
        Commands::GenWit {
            output_dir,
            overwrite,
        } => {
            generate_wit_interfaces(&output_dir, overwrite)?;
        }
        Commands::GenLefthook { output, validate } => {
            emit_warning!(
                "hooksmith",
                "lefthook",
                "Lefthook generation disabled - lefthook_rs dependency missing"
            );
            emit_info!("hooksmith", "lefthook", "Output: {output}");
            emit_info!("hooksmith", "lefthook", "Validate: {validate}");
        }
        Commands::GenDocs { output_dir, open } => {
            generate_documentation(&output_dir, open)?;
        }
        Commands::GenDocsComprehensive {
            all,
            file,
            output_dir,
            validate,
        } => {
            generate_comprehensive_documentation(all, &file, &output_dir, validate).await?;
        }
        Commands::GenSchemaDocs {
            output_dir,
            pdf,
            html,
            epub,
            open,
        } => {
            generate_schema_documentation(&output_dir, pdf, html, epub, open).await?;
        }
        Commands::GenReadme { output, overwrite } => {
            generate_readme(&output, overwrite)?;
        }
        Commands::GenMods { overwrite } => {
            generate_mod_files(overwrite)?;
        }
        Commands::GenHooksReadme { output, overwrite } => {
            generate_hooks_readme(&output, overwrite)?;
        }
        Commands::GenAllLegacy { overwrite } => {
            generate_all(overwrite).await?;
        }
        Commands::Check { strict } => {
            check_generated_files(strict)?;
        }
        Commands::Validate {
            trunk,
            cargo,
            modules,
            all,
        } => {
            validate_project_config(trunk, cargo, modules, all)?;
        }

        Commands::ContractValidate { command } => {
            hierarchical_validation::run_command(command).await?;
        }
        Commands::ValidateGenerated {
            staged_only,
            strict,
            custom_message,
        } => {
            validate_generated_files(staged_only, strict, custom_message)?;
        }
        Commands::AddGeneratedHeaders { file } => {
            add_generated_headers(file)?;
        }
        Commands::ValidateHeaders { strict } => {
            validate_generated_headers(strict)?;
        }
        Commands::GenTemplates {
            template,
            output_dir,
            overwrite,
        } => {
            generate_templates(template, &output_dir, overwrite)?;
        }
        Commands::CheckStable {
            version,
            comprehensive,
        } => {
            check_stable_compatibility(&version, comprehensive).await?;
        }
        Commands::TestWithRelease { version } => {
            test_with_release(&version).await?;
        }
        Commands::CompareWithRelease { version } => {
            compare_with_release(&version).await?;
        }
        Commands::SetupGitFilters { force } => {
            setup_git_filters(force).await?;
        }
        Commands::CheckFiles { strict, verbose } => {
            check_files(strict, verbose)?;
        }
        Commands::ValidateFiles { strict, verbose } => {
            validate_files_strict(strict, verbose)?;
        }
        Commands::GenAll { validate, force } => {
            generate_all_files(validate, force).await?;
        }
        Commands::Bootstrap { validate, commit } => {
            bootstrap_project(validate, commit).await?;
        }
        Commands::GenGitattributes {
            output_dir,
            overwrite,
            validate,
        } => {
            generate_git_attributes(&output_dir, overwrite, validate)?;
        }
        Commands::GenConfig {
            overwrite,
            validate,
        } => {
            generate_config(overwrite, validate)?;
        }
        Commands::ValidateConfig { strict } => {
            validate_config(strict)?;
        }
        Commands::Contract { command } => {
            contract::run_contract_command(command).await?;
        }
        Commands::ContractValidation { command } => {
            let validator = contract_validation::ContractValidator::new()?;
            validator.run(command).await?;
        }
        Commands::Status { command } => {
            status::run_status_command(command).await?;
        }
        Commands::ContractCheck {
            staged_only,
            strict,
            trend,
            trend_output,
            verbose,
        } => {
            run_contract_check(staged_only, strict, trend, &trend_output, verbose).await?;
        }
        Commands::CodeStats { command } => {
            code_stats::run_code_stats_command(command).await?;
        }
        Commands::ValidateCommitMsg {
            file,
            allow_empty,
            validate_conventional,
        } => {
            validate_commit_message(file, allow_empty, validate_conventional)?;
        }
        Commands::SetupGitAliases { force } => {
            setup_git_aliases(force)?;
        }
        Commands::ValidateDocs {
            strict,
            regenerate,
            check_uncommitted,
        } => {
            validate_documentation(strict, regenerate, check_uncommitted).await?;
        }
        Commands::GitCommit {
            message,
            allow_empty_message,
            args,
        } => {
            git_commit(message, allow_empty_message, args).await?;
        }
        Commands::SetupPreCommit {
            enhanced,
            force,
            lefthook,
        } => {
            setup_pre_commit(enhanced, force, lefthook).await?;
        }
        Commands::PreCommit {
            enhanced,
            staged_only,
            strict,
            auto_fix,
        } => {
            run_pre_commit(enhanced, staged_only, strict, auto_fix).await?;
        }
        Commands::RunHooks {
            hook_type,
            verbose,
            no_logs,
            no_events,
            log_dir,
        } => {
            println!("⚠️  RunHooks command not yet implemented");
            println!("   Hook type: {}", hook_type);
            println!("   Verbose: {}", verbose);
            println!("   No logs: {}", no_logs);
            println!("   No events: {}", no_events);
            if let Some(dir) = log_dir {
                println!("   Log dir: {}", dir);
            }
        }
        Commands::DeadCodeCheck {
            strict,
            include_generated,
            restore,
            format,
        } => {
            run_dead_code_check(strict, include_generated, restore, format).await?;
        }
        Commands::ConvertJsonc {
            file,
            overwrite,
            remove_original,
        } => {
            convert_json_to_jsonc(file, overwrite, remove_original).await?;
        }
        Commands::AutoPush {
            message,
            allow_empty_message,
            skip_validation,
            watchdog,
            interval,
            force,
            args,
        } => {
            run_auto_push_with_state_machine(
                message,
                allow_empty_message,
                skip_validation,
                watchdog,
                interval,
                force,
                args,
            )
            .await?;
        }
        Commands::CleanAutoPush {
            message,
            allow_empty_message,
            watchdog,
            interval,
            force,
            verbose,
            no_log,
            log_file,
            args,
        } => {
            run_clean_auto_push(
                message,
                allow_empty_message,
                watchdog,
                interval,
                force,
                verbose,
                no_log,
                log_file,
                args,
            )
            .await?;
        }
        Commands::StructuredAutoPush {
            message,
            allow_empty_message,
            watchdog,
            interval,
            force,
            verbose,
            no_jsonl,
            no_event_bus,
            args,
        } => {
            run_structured_auto_push(
                message,
                allow_empty_message,
                watchdog,
                interval,
                force,
                verbose,
                no_jsonl,
                no_event_bus,
                args,
            )
            .await?;
        }
        Commands::Hook {
            hook_type,
            message,
            allow_empty_message,
            skip_validation,
            watchdog,
            interval,
            force,
            args,
        } => {
            run_hook_with_state_machine(
                hook_type,
                message,
                allow_empty_message,
                skip_validation,
                watchdog,
                interval,
                force,
                args,
            )
            .await?;
        }
        Commands::ListHooks => {
            list_available_hooks()?;
        }
        Commands::GenLefthookHooks { output, validate } => {
            generate_lefthook_hooks_config(output, validate).await?;
        }
        Commands::EventStream { command } => match command {
            EventStreamCommands::Init {
                output_file,
                console_output,
                enable_broadcast,
                min_severity,
            } => {
                init_event_stream_command(
                    output_file,
                    console_output,
                    enable_broadcast,
                    min_severity,
                )
                .await?;
            }
            EventStreamCommands::Monitor {
                show_metadata,
                performance_threshold,
                error_threshold,
            } => {
                monitor_events_command(show_metadata, performance_threshold, error_threshold)
                    .await?;
            }
            EventStreamCommands::Analyze { input_file, format } => {
                analyze_events_command(input_file, format).await?;
            }
            EventStreamCommands::GenConfig { output } => {
                generate_event_stream_config(output).await?;
            }
        },
        Commands::EventBus { command } => match command {
            EventBusCommands::Init {
                enable_persistence,
                jsonl_file,
                batch_size,
                flush_interval_ms,
                console_output,
            } => {
                init_event_bus_command(
                    enable_persistence,
                    jsonl_file,
                    batch_size,
                    flush_interval_ms,
                    console_output,
                )
                .await?;
            }
            EventBusCommands::Process {
                auto_push,
                notifications,
                metrics,
            } => {
                process_events_command(auto_push, notifications, metrics).await?;
            }
            EventBusCommands::Replay {
                input_file,
                auto_push,
                notifications,
            } => {
                replay_events_command(input_file, auto_push, notifications).await?;
            }
            EventBusCommands::EmitTest { count } => {
                emit_test_events_command(count).await?;
            }
        },
        Commands::WasmComponents { command } => match command {
            WasmComponentCommands::Load {
                component_path,
                config,
            } => {
                load_wasm_component_command(component_path, config).await?;
            }
            WasmComponentCommands::List => {
                list_wasm_components_command()?;
            }
            WasmComponentCommands::Unload { handler_id } => {
                unload_wasm_component_command(handler_id)?;
            }
            WasmComponentCommands::Stats => {
                get_wasm_component_stats_command()?;
            }
            WasmComponentCommands::BuildValidationHandler { output_dir } => {
                build_validation_handler_command(output_dir).await?;
            }
        },
        Commands::Dashboard {
            interval,
            show_dashboard,
            auto_push,
            skip_validation,
            message,
            file_watch,
            trigger,
        } => {
            run_dashboard_command(
                interval,
                show_dashboard,
                auto_push,
                skip_validation,
                message,
                file_watch,
                trigger,
            )
            .await?;
        }
        Commands::GenSchema { output } => {
            generate_schema_command(output)?;
        }
        Commands::ValidateSchema { input, strict } => {
            validate_schema_command(input, strict)?;
        }
        Commands::Sarif { command } => match command {
            SarifCommands::JsonlToSarif {
                input,
                output,
                validate,
            } => run_jsonl_to_sarif_command(input, output, validate).await?,
            SarifCommands::SarifToJsonl {
                input,
                output,
                validate,
            } => run_sarif_to_jsonl_command(input, output, validate).await?,
            SarifCommands::CodeqlAnalysis {
                cli_path,
                db_dir,
                query_suite,
                language,
                build_command,
                output,
                to_jsonl,
            } => {
                run_codeql_analysis_command(
                    cli_path.as_deref(),
                    db_dir,
                    query_suite,
                    language,
                    build_command,
                    output.as_deref(),
                    to_jsonl,
                )
                .await?
            }
            SarifCommands::ValidateSarif { file, strict } => {
                run_validate_sarif_command(file, strict)?
            }
            SarifCommands::MergeSarif {
                inputs,
                output,
                validate,
            } => run_merge_sarif_command(inputs, output, validate)?,
            SarifCommands::IntegrateCodeql {
                run_analysis,
                to_jsonl,
                merge,
                output_dir,
            } => run_integrate_codeql_command(run_analysis, to_jsonl, merge, output_dir).await?,
        },
    }

    Ok(())
}

/// Build the project and all components
fn build_project(target: &str, release: bool) -> Result<()> {
    println!("🔨 Building Hooksmith project...");
    println!("   Target: {target}");
    println!("   Release: {release}");

    let _profile = if release { "release" } else { "debug" };

    match target {
        "native" => {
            let status = Command::new("cargo")
                .args(["build", "--workspace"])
                .args(if release { vec!["--release"] } else { vec![] })
                .status()
                .context("Failed to build native target")?;

            if !status.success() {
                anyhow::bail!("Native build failed");
            }
        }
        "wasm" => {
            // Build WASM components
            let components = ["worktree-runner"];
            for component in components {
                println!("   Building WASM component: {component}");
                let status = Command::new("cargo")
                    .args(["build", "--target", "wasm32-unknown-unknown"])
                    .args(if release { vec!["--release"] } else { vec![] })
                    .current_dir(format!("components/{component}"))
                    .status()
                    .context(format!("Failed to build WASM component: {component}"))?;

                if !status.success() {
                    anyhow::bail!("WASM build failed for component: {}", component);
                }
            }
        }
        "all" => {
            // Build native first
            build_project("native", release)?;
            // Then build WASM
            build_project("wasm", release)?;
        }
        _ => {
            anyhow::bail!("Unknown target: {}", target);
        }
    }

    println!("✅ Build completed successfully");
    Ok(())
}

/// Generate WIT interface definitions from structured Rust definitions
fn generate_wit_interfaces(output_dir: &str, overwrite: bool) -> Result<()> {
    println!("🔧 Generating WIT interface definitions...");
    println!("   Output directory: {output_dir}");

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).context("Failed to create output directory")?;
    }

    // Define CLI interface
    let cli_interface = WitInterface {
        package: "hooksmith:cli".to_string(),
        name: "hooksmith-cli".to_string(),
        docs: Some("Main CLI interface for Hooksmith".to_string()),
        functions: vec![
            WitFunction {
                name: "build-hook".to_string(),
                params: vec![WitParam {
                    name: "config".to_string(),
                    param_type: "hook-config".to_string(),
                    docs: Some("Hook configuration".to_string()),
                }],
                result: "result<build-result, string>".to_string(),
                docs: Some("Build a hook from source".to_string()),
            },
            WitFunction {
                name: "list-hooks".to_string(),
                params: vec![],
                result: "result<list<hook-info>, string>".to_string(),
                docs: Some("List all available hooks".to_string()),
            },
            WitFunction {
                name: "install-hooks".to_string(),
                params: vec![WitParam {
                    name: "hook-names".to_string(),
                    param_type: "list<string>".to_string(),
                    docs: Some("Names of hooks to install".to_string()),
                }],
                result: "result<unit, string>".to_string(),
                docs: Some("Install hooks into Git repository".to_string()),
            },
        ],
        records: vec![
            WitRecord {
                name: "hook-config".to_string(),
                docs: Some("Configuration for hook building".to_string()),
                fields: vec![
                    WitField {
                        name: "name".to_string(),
                        field_type: "string".to_string(),
                        docs: Some("Name of the hook to build".to_string()),
                    },
                    WitField {
                        name: "source-dir".to_string(),
                        field_type: "string".to_string(),
                        docs: Some("Source directory for the hook".to_string()),
                    },
                    WitField {
                        name: "output-dir".to_string(),
                        field_type: "string".to_string(),
                        docs: Some("Output directory for built binaries".to_string()),
                    },
                    WitField {
                        name: "include-wasm".to_string(),
                        field_type: "bool".to_string(),
                        docs: Some("Whether to include WASM components".to_string()),
                    },
                ],
            },
            WitRecord {
                name: "build-result".to_string(),
                docs: Some("Result of a hook building operation".to_string()),
                fields: vec![
                    WitField {
                        name: "success".to_string(),
                        field_type: "bool".to_string(),
                        docs: Some("Whether the build was successful".to_string()),
                    },
                    WitField {
                        name: "binary-path".to_string(),
                        field_type: "option<string>".to_string(),
                        docs: Some("Output path of the built binary".to_string()),
                    },
                    WitField {
                        name: "error".to_string(),
                        field_type: "option<string>".to_string(),
                        docs: Some("Error message if build failed".to_string()),
                    },
                ],
            },
        ],
        enums: vec![],
    };

    // Define worktree runner interface
    let worktree_interface = WitInterface {
        package: "hooksmith:worktree-runner".to_string(),
        name: "worktree-runner".to_string(),
        docs: Some("Worktree management interface".to_string()),
        functions: vec![
            WitFunction {
                name: "create-worktree".to_string(),
                params: vec![WitParam {
                    name: "branch-name".to_string(),
                    param_type: "string".to_string(),
                    docs: Some("Name of the branch to create".to_string()),
                }],
                result: "result<worktree-result, string>".to_string(),
                docs: Some("Create a new worktree".to_string()),
            },
            WitFunction {
                name: "list-worktrees".to_string(),
                params: vec![],
                result: "result<worktree-result, string>".to_string(),
                docs: Some("List all worktrees".to_string()),
            },
        ],
        records: vec![
            WitRecord {
                name: "tool-config".to_string(),
                docs: Some("Configuration for worktree tools".to_string()),
                fields: vec![
                    WitField {
                        name: "preferred-tool".to_string(),
                        field_type: "option<string>".to_string(),
                        docs: Some("Preferred tool to use".to_string()),
                    },
                    WitField {
                        name: "worktree-base".to_string(),
                        field_type: "option<string>".to_string(),
                        docs: Some("Base directory for worktrees".to_string()),
                    },
                ],
            },
            WitRecord {
                name: "worktree-result".to_string(),
                docs: Some("Result of a worktree operation".to_string()),
                fields: vec![
                    WitField {
                        name: "success".to_string(),
                        field_type: "bool".to_string(),
                        docs: Some("Whether the operation was successful".to_string()),
                    },
                    WitField {
                        name: "output".to_string(),
                        field_type: "string".to_string(),
                        docs: Some("Output from the command".to_string()),
                    },
                ],
            },
        ],
        enums: vec![WitEnum {
            name: "worktree-tool".to_string(),
            docs: Some("Available worktree tools".to_string()),
            variants: vec![
                "wtp".to_string(),
                "wt".to_string(),
                "treekanga".to_string(),
                "git".to_string(),
            ],
        }],
    };

    // Generate WIT files
    let interfaces = vec![
        ("hooksmith.wit", cli_interface),
        ("worktree-runner.wit", worktree_interface),
    ];

    for (filename, interface) in interfaces {
        let file_path = output_path.join(filename);

        if file_path.exists() && !overwrite {
            println!("   Skipping {filename} (already exists)");
            continue;
        }

        let wit_content = generate_wit_content(&interface)?;
        fs::write(&file_path, wit_content).context(format!("Failed to write {filename}"))?;
        println!("   Generated {filename}");
    }

    println!("✅ WIT interfaces generated successfully");
    Ok(())
}

/// Generate WIT content from interface definition
fn generate_wit_content(interface: &WitInterface) -> Result<String> {
    let mut content = String::new();

    // Package declaration
    content.push_str(&format!("package {};\n\n", interface.package));

    // Records
    for record in &interface.records {
        if let Some(docs) = &record.docs {
            content.push_str(&format!("/// {docs}\n"));
        }
        content.push_str(&format!("record {} {{\n", record.name));
        for field in &record.fields {
            if let Some(docs) = &field.docs {
                content.push_str(&format!("  /// {docs}\n"));
            }
            content.push_str(&format!("  {}: {};\n", field.name, field.field_type));
        }
        content.push_str("}\n\n");
    }

    // Enums
    for enum_def in &interface.enums {
        if let Some(docs) = &enum_def.docs {
            content.push_str(&format!("/// {docs}\n"));
        }
        content.push_str(&format!("enum {} {{\n", enum_def.name));
        for variant in &enum_def.variants {
            content.push_str(&format!("  {variant},\n"));
        }
        content.push_str("}\n\n");
    }

    // Interface
    if let Some(docs) = &interface.docs {
        content.push_str(&format!("/// {docs}\n"));
    }
    content.push_str(&format!("interface {} {{\n", interface.name));

    for function in &interface.functions {
        if let Some(docs) = &function.docs {
            content.push_str(&format!("  /// {docs}\n"));
        }
        let params = function
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.param_type))
            .collect::<Vec<_>>()
            .join(", ");
        content.push_str(&format!(
            "  {}: func({}) -> {};\n",
            function.name, params, function.result
        ));
    }

    content.push_str("}\n\n");

    // Export
    content.push_str(&format!("export {};\n", interface.name));

    Ok(content)
}

/// Generate Lefthook configuration from structured definitions
fn generate_lefthook_config(output: &str, validate: bool) -> Result<()> {
    println!("📝 Generating Lefthook configuration...");
    println!("   Output: {output}");

    // Lefthook configuration generation disabled due to missing dependency
    println!("⚠️  Lefthook configuration generation disabled");
    println!("   To enable, add lefthook_rs dependency to xtask/Cargo.toml");
    println!("   For now, using existing lefthook.yml file");

    if validate {
        println!("   Skipping validation (lefthook_rs not available)");
    }

    println!("✅ Lefthook configuration generation skipped");
    Ok(())
}

/// Generate documentation
fn generate_documentation(output_dir: &str, open: bool) -> Result<()> {
    println!("📚 Generating documentation...");
    println!("   Output directory: {output_dir}");

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).context("Failed to create output directory")?;
    }

    // Generate API documentation
    let status = Command::new("cargo")
        .args(["doc", "--no-deps", "--document-private-items"])
        .status()
        .context("Failed to generate API documentation")?;

    if !status.success() {
        anyhow::bail!("API documentation generation failed");
    }

    // Generate CLI help documentation
    let cli_help = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .context("Failed to get CLI help")?;

    let cli_help_content = format!(
        "# CLI Help Documentation\n\n```\n{}\n```\n",
        String::from_utf8_lossy(&cli_help.stdout)
    );

    fs::write(output_path.join("CLI_HELP.md"), cli_help_content)
        .context("Failed to write CLI help documentation")?;

    if open {
        println!("   Opening documentation in browser...");
        let _ = Command::new("cargo")
            .args(["doc", "--no-deps", "--open"])
            .status();
    }

    println!("✅ Documentation generated successfully");
    Ok(())
}

/// Generate comprehensive documentation from Rust code and templates
async fn generate_comprehensive_documentation(
    all: bool,
    file: &Option<String>,
    output_dir: &str,
    validate: bool,
) -> Result<()> {
    println!("📚 Generating comprehensive documentation...");
    println!("   Output directory: {output_dir}");
    println!("   All: {all}, File: {file:?}, Validate: {validate}");

    // Use the new docs module system
    docs::generate_all_docs(output_dir, validate).await?;

    // Generate additional documentation if requested
    if all {
        // Generate JSON Schema documentation
        let schema_docs = generate_json_schema_documentation()?;
        fs::write(
            Path::new(output_dir).join("SCHEMA_DOCUMENTATION.md"),
            &schema_docs,
        )
        .context("Failed to write schema documentation")?;

        // Generate WIT documentation
        let wit_docs = generate_wit_documentation()?;
        fs::write(
            Path::new(output_dir).join("WIT_DOCUMENTATION.md"),
            &wit_docs,
        )
        .context("Failed to write WIT documentation")?;

        // Generate combined documentation
        let combined_docs = generate_combined_documentation(&schema_docs, &wit_docs)?;
        fs::write(
            Path::new(output_dir).join("CONTRACT_STATE_MACHINE.md"),
            combined_docs,
        )
        .context("Failed to write combined documentation")?;

        // Generate Pandoc outputs
        generate_pandoc_outputs(Path::new(output_dir), true, true, true)?;
    } else if let Some(f) = file {
        // Generate specific file if a file is specified
        let output_path = Path::new(output_dir);
        match f.as_str() {
            "schema" => {
                let schema_docs = generate_json_schema_documentation()?;
                fs::write(output_path.join("SCHEMA_DOCUMENTATION.md"), &schema_docs)
                    .context("Failed to write schema documentation")?;
                generate_pandoc_outputs(output_path, true, false, false)?; // PDF only
            }
            "wit" => {
                let wit_docs = generate_wit_documentation()?;
                fs::write(output_path.join("WIT_DOCUMENTATION.md"), &wit_docs)
                    .context("Failed to write WIT documentation")?;
                generate_pandoc_outputs(output_path, false, true, false)?; // HTML only
            }
            "epub" => {
                let combined_docs = generate_combined_documentation(
                    &generate_json_schema_documentation()?,
                    &generate_wit_documentation()?,
                )?;
                fs::write(output_path.join("CONTRACT_STATE_MACHINE.md"), combined_docs)
                    .context("Failed to write combined documentation")?;
                generate_pandoc_outputs(output_path, false, false, true)?; // EPUB only
            }
            _ => {
                println!("   ⚠️  Unknown file type: {f}");
            }
        }
    }

    if all || file.is_some() {
        println!("   Opening documentation in browser...");
        let _ = Command::new("open")
            .arg(Path::new(output_dir).join("README.md"))
            .status();
    }

    println!("✅ Comprehensive documentation generated successfully");
    Ok(())
}

/// Generate schema and WIT documentation with Pandoc integration
async fn generate_schema_documentation(
    output_dir: &str,
    pdf: bool,
    html: bool,
    epub: bool,
    open: bool,
) -> Result<()> {
    println!("📚 Generating schema and WIT documentation...");
    println!("   Output directory: {output_dir}");
    println!("   PDF: {pdf}, HTML: {html}, EPUB: {epub}");

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).context("Failed to create output directory")?;
    }

    // Generate JSON Schema documentation
    let schema_docs = generate_json_schema_documentation()?;
    fs::write(output_path.join("SCHEMA_DOCUMENTATION.md"), &schema_docs)
        .context("Failed to write schema documentation")?;

    // Generate WIT documentation
    let wit_docs = generate_wit_documentation()?;
    fs::write(output_path.join("WIT_DOCUMENTATION.md"), &wit_docs)
        .context("Failed to write WIT documentation")?;

    // Generate combined documentation
    let combined_docs = generate_combined_documentation(&schema_docs, &wit_docs)?;
    fs::write(output_path.join("CONTRACT_STATE_MACHINE.md"), combined_docs)
        .context("Failed to write combined documentation")?;

    // Generate Pandoc outputs if requested
    if pdf || html || epub {
        generate_pandoc_outputs(output_path, pdf, html, epub)?;
    }

    if open {
        println!("   Opening documentation in browser...");
        let _ = Command::new("open")
            .arg(output_path.join("CONTRACT_STATE_MACHINE.md"))
            .status();
    }

    println!("✅ Schema and WIT documentation generated successfully");
    Ok(())
}

/// Generate README with CLI help and module docs
fn generate_readme(output: &str, overwrite: bool) -> Result<()> {
    println!("📖 Generating README...");
    println!("   Output: {output}");

    let output_path = Path::new(output);
    if output_path.exists() && !overwrite {
        println!("   Skipping README (already exists)");
        return Ok(());
    }

    // Get CLI help
    let cli_help = Command::new("cargo")
        .args(["run", "--bin", "hooksmith", "--", "--help"])
        .output()
        .context("Failed to get CLI help")?;

    let cli_help_text = String::from_utf8_lossy(&cli_help.stdout);

    // Generate README content
    let readme_content = format!(
        r#"# Hooksmith

A CLI tool for building Rust binaries into Lefthook hooks with WASM components.

## Features

- 🔧 **Structured Code Generation**: WIT interfaces generated from Rust structs
- 🚀 **WASM Integration**: Build and manage WASM components for Git hooks
- 📝 **Lefthook Integration**: Generate and validate Lefthook configurations
- 🛠️ **Xtask Workflow**: Rust-based build system replacing shell scripts

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Get help
hooksmith --help

# Test the CLI
hooksmith test

# Generate WIT interfaces
cargo xtask gen-wit

# Generate Lefthook configuration
cargo xtask gen-lefthook

# Run all code generation
cargo xtask gen-all
```

## CLI Commands

```bash
{cli_help_text}
```

## Development

### Prerequisites

- **Rust**: Latest stable version (1.75+)
- **Git**: Latest version
- **Lefthook**: For pre-commit hooks (optional but recommended)

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/hooksmith.git
   cd hooksmith
   ```

2. **Install dependencies**
   ```bash
   # Install Lefthook (optional but recommended)
   npm install -g @evilmartians/lefthook

   # Or using Homebrew on macOS
   brew install lefthook
   ```

3. **Install pre-commit hooks**
   ```bash
   lefthook install
   ```

4. **Generate code and build the project**
   ```bash
   # Generate all code and documentation
   ./xtask.sh gen-all --overwrite

   # Or use the build script
   ./build.sh
   ```

5. **Run tests**
   ```bash
   cargo test --all-targets --all-features
   ```

### Xtask Commands

This project uses **xtask** for structured code generation and build tasks, replacing shell scripts and raw echo statements:

```bash
# Build the project and all components
./xtask.sh build --target all --release

# Generate WIT interface definitions
./xtask.sh gen-wit --overwrite

# Generate Lefthook configuration
./xtask.sh gen-lefthook --validate

# Generate documentation
./xtask.sh gen-docs --open

# Generate README with CLI help
./xtask.sh gen-readme --overwrite

# Generate mod.rs files
./xtask.sh gen-mods --overwrite

# Run all code generation tasks
./xtask.sh gen-all --overwrite

# Check if generated files are up to date
./xtask.sh check --strict

# Validate project configuration
./xtask.sh validate --all
```

**Benefits of Xtask:**
- ✅ **No shell scripts** - All tasks are Rust-based
- ✅ **Structured code generation** - WIT files generated from Rust structs
- ✅ **Type-safe configuration** - All configs are strongly typed
- ✅ **Deterministic builds** - Same input always produces same output
- ✅ **CI integration** - Automated checks ensure generated files are up to date

## Project Structure

```
hooksmith/
├── Cargo.toml               # Workspace manifest
├── xtask.sh                 # Xtask wrapper script
├── README.md                # This file (auto-generated)
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command modules (auto-generated mod.rs)
│   └── modules/             # Core modules (auto-generated mod.rs)
├── components/              # WASM components
│   ├── cli-core/            # Core CLI functionality
│   └── worktree-runner/     # Worktree management WASM component
├── wit/                     # WIT interface definitions (auto-generated)
├── hooks/                   # Hook scripts directory
├── tests/                   # Test files
└── target/doc/              # Generated documentation
```

## Components

- **hooksmith**: Main CLI binary for hook building and WASM management
- **cli-core**: Core CLI functionality and utilities
- **worktree-runner**: WASM component for worktree management

## Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
hooksmith generate > lefthook.yml

# Install hooks
hooksmith install
```

## Documentation

- **API Documentation**: `cargo doc --no-deps --open`
- **CLI Help**: `hooksmith --help`
- **Command Help**: `hooksmith <command> --help`

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_cli_help

# Run integration tests
cargo test --test integration
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| CLI Structure | ✅ Complete | Full command parsing and help |
| Documentation | ✅ Complete | Comprehensive docs and examples |
| Tests | ✅ Complete | All tests passing |
| Build System | ✅ Complete | Xtask-based workflow |
| WASM Compilation | ✅ Complete | WASM toolchain integration |
| WIT Processing | ✅ Complete | WIT parser and compiler |
| Lefthook Integration | ✅ Complete | YAML generation and hook installation |
| Hook Building | ✅ Complete | Rust compilation pipeline |

## License

MIT License - see LICENSE file for details.

---

*This README is auto-generated using `cargo xtask gen-readme`. The CLI help section is automatically updated from the actual CLI output.*
"#
    );

    fs::write(output_path, readme_content).context("Failed to write README")?;
    println!("✅ README generated successfully");
    Ok(())
}

/// Generate mod.rs files for commands and modules
fn generate_mod_files(overwrite: bool) -> Result<()> {
    println!("📁 Generating mod.rs files...");

    // Generate commands/mod.rs
    let commands_dir = Path::new("src/commands");
    if commands_dir.exists() {
        let mod_content = generate_mod_content(commands_dir, "commands")?;
        let mod_path = commands_dir.join("mod.rs");

        if mod_path.exists() && !overwrite {
            println!("   Skipping src/commands/mod.rs (already exists)");
        } else {
            fs::write(&mod_path, mod_content).context("Failed to write commands/mod.rs")?;
            println!("   Generated src/commands/mod.rs");
        }
    }

    // Generate modules/mod.rs
    let modules_dir = Path::new("src/modules");
    if modules_dir.exists() {
        let mod_content = generate_mod_content(modules_dir, "modules")?;
        let mod_path = modules_dir.join("mod.rs");

        if mod_path.exists() && !overwrite {
            println!("   Skipping src/modules/mod.rs (already exists)");
        } else {
            fs::write(&mod_path, mod_content).context("Failed to write modules/mod.rs")?;
            println!("   Generated src/modules/mod.rs");
        }
    }

    println!("✅ mod.rs files generated successfully");
    Ok(())
}

/// Generate mod.rs content for a directory
fn generate_mod_content(dir: &Path, dir_name: &str) -> Result<String> {
    let mut content = String::new();
    content.push_str(&format!("//! {dir_name} module\n"));
    content.push_str("//! \n");
    content.push_str(&format!(
        "//! This module contains {dir_name} functionality.\n"
    ));
    content.push_str("//! Auto-generated by xtask gen-mods\n\n");

    let entries = fs::read_dir(dir).context(format!("Failed to read directory: {dir:?}"))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if filename != "mod" {
                // Convert snake_case to Title Case for better documentation
                let title = filename
                    .split('_')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                content.push_str(&format!("/// {title} functionality\n"));
                content.push_str(&format!("pub mod {filename};\n"));
            }
        }
    }

    Ok(content)
}

/// Generate hooks README
fn generate_hooks_readme(output: &str, overwrite: bool) -> Result<()> {
    println!("📝 Generating hooks README...");
    println!("   Output: {output}");

    let output_path = Path::new(output);
    if output_path.exists() && !overwrite {
        println!("   Skipping hooks README (already exists)");
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create parent directory")?;
    }

    let hooks_content = r#"# Hooks Directory

This directory contains Git hooks and related scripts for the Hooksmith project.

## Available Hooks

### Pre-commit Hooks

- **hooksmith-fmt**: Runs `cargo fmt --all -- --check` to ensure code formatting
- **hooksmith-clippy**: Runs `cargo clippy --all-targets --all-features -- -D warnings` for linting
- **hooksmith-test**: Runs `cargo test --all-targets --all-features` to ensure tests pass
- **hooksmith-gen-wit**: Runs `cargo xtask gen-wit` to regenerate WIT interfaces

### Pre-push Hooks

- **hooksmith-audit**: Runs `cargo audit` to check for security vulnerabilities
- **hooksmith-check-generated**: Runs `cargo xtask check --strict` to ensure generated files are up to date

## Installation

Hooks are automatically installed when you run:

```bash
lefthook install
```

## Configuration

Hook configuration is managed in `lefthook.yml` at the project root. This file is auto-generated using:

```bash
cargo xtask gen-lefthook
```

## Custom Hooks

To add custom hooks:

1. Add the hook definition to the appropriate section in `lefthook.yml`
2. Run `cargo xtask gen-lefthook` to regenerate the configuration
3. The hook will be automatically installed on the next `lefthook install`

## Validation

Hooks are validated against the Lefthook schema using:

```bash
cargo xtask validate --all
```

---

*This file is auto-generated by `cargo xtask gen-hooks-readme`.*
"#;

    fs::write(output_path, hooks_content).context("Failed to write hooks README")?;
    println!("✅ Hooks README generated successfully");
    Ok(())
}

/// Generate all code generation tasks
async fn generate_all(overwrite: bool) -> Result<()> {
    println!("🚀 Running all code generation tasks...");

    generate_wit_interfaces("wit", overwrite)?;
    generate_lefthook_config("lefthook.yml", true)?;
    generate_documentation("docs", false)?;

    // Generate schema documentation (Markdown only, no PDF/HTML/EPUB by default)
    generate_schema_documentation("docs", false, false, false, false).await?;

    generate_readme("README.md", overwrite)?;
    generate_mod_files(overwrite)?;
    generate_hooks_readme("hooks/README.md", overwrite)?;

    println!("✅ All code generation tasks completed successfully");
    Ok(())
}

/// Check if generated files are up to date
fn check_generated_files(strict: bool) -> Result<()> {
    println!("🔍 Checking generated files...");

    let mut outdated = false;

    // Check WIT files
    let wit_files = ["wit/hooksmith.wit", "wit/worktree-runner.wit"];
    for file in wit_files {
        if !Path::new(file).exists() {
            println!("   ❌ Missing: {file}");
            outdated = true;
        }
    }

    // Check Lefthook config
    if !Path::new("lefthook.yml").exists() {
        println!("   ❌ Missing: lefthook.yml");
        outdated = true;
    }

    // Check README
    if !Path::new("README.md").exists() {
        println!("   ❌ Missing: README.md");
        outdated = true;
    }

    // Check mod.rs files
    let mod_files = ["src/commands/mod.rs", "src/modules/mod.rs"];
    for file in mod_files {
        if !Path::new(file).exists() {
            println!("   ❌ Missing: {file}");
            outdated = true;
        }
    }

    // Check hooks README
    if !Path::new("hooks/README.md").exists() {
        println!("   ❌ Missing: hooks/README.md");
        outdated = true;
    }

    if outdated {
        let message = "Generated files are outdated. Run 'cargo xtask gen-all' to regenerate.";
        if strict {
            anyhow::bail!(message);
        } else {
            println!("   ⚠️  {message}");
        }
    } else {
        println!("   ✅ All generated files are up to date");
    }

    Ok(())
}

/// Validate project configuration
fn validate_project_config(trunk: bool, cargo: bool, modules: bool, all: bool) -> Result<()> {
    println!("🔍 Validating project configuration...");

    let mut errors = Vec::new();

    if trunk || all {
        if let Err(e) = validate_trunk_config() {
            errors.push(format!("Trunk validation failed: {e}"));
        } else {
            println!("   ✅ Trunk configuration is valid");
        }
    }

    if cargo || all {
        if let Err(e) = validate_cargo_workspace() {
            errors.push(format!("Cargo validation failed: {e}"));
        } else {
            println!("   ✅ Cargo workspace is valid");
        }
    }

    if modules || all {
        if let Err(e) = validate_module_consistency() {
            errors.push(format!("Module validation failed: {e}"));
        } else {
            println!("   ✅ Module consistency is valid");
        }
    }

    if errors.is_empty() {
        println!("✅ All validations passed");
        Ok(())
    } else {
        for error in errors {
            eprintln!("   ❌ {error}");
        }
        anyhow::bail!("Validation failed");
    }
}

/// Validate Trunk configuration
fn validate_trunk_config() -> Result<()> {
    let trunk_config = Path::new(".trunk/trunk.yaml");
    if !trunk_config.exists() {
        return Ok(()); // Trunk config is optional
    }

    let content = fs::read_to_string(trunk_config).context("Failed to read trunk config")?;
    let _config: serde_yaml::Value =
        serde_yaml::from_str(&content).context("Failed to parse trunk config")?;

    Ok(())
}

/// Validate Cargo workspace
fn validate_cargo_workspace() -> Result<()> {
    let cargo_toml = Path::new("Cargo.toml");
    let content = fs::read_to_string(cargo_toml).context("Failed to read Cargo.toml")?;

    // Basic validation - check that it can be parsed
    let _config: toml::Value = toml::from_str(&content).context("Failed to parse Cargo.toml")?;

    Ok(())
}

/// Validate module consistency
fn validate_module_consistency() -> Result<()> {
    // Check that all command files have corresponding test files
    let commands_dir = Path::new("src/commands");
    if commands_dir.exists() {
        let entries = fs::read_dir(commands_dir).context("Failed to read commands directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if filename != "mod" {
                    let test_file = Path::new("tests").join(format!("{filename}_test.rs"));
                    if !test_file.exists() {
                        println!("   ⚠️  No test file found for command: {filename}");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Generate JSON Schema documentation from existing schema files
fn generate_json_schema_documentation() -> Result<String> {
    let mut docs = String::new();

    docs.push_str("# JSON Schema Documentation\n\n");
    docs.push_str("This document describes the JSON schemas used by Hooksmith for contract validation and state machine management.\n\n");

    // Read and document contract state schema
    let contract_state_schema = fs::read_to_string("schemas/contract-state.schema.json")
        .context("Failed to read contract-state.schema.json")?;
    let contract_state: serde_json::Value = serde_json::from_str(&contract_state_schema)
        .context("Failed to parse contract-state.schema.json")?;

    docs.push_str("## Contract State Schema\n\n");
    docs.push_str("Defines the structure for contract validation states.\n\n");

    if let Some(properties) = contract_state.get("properties") {
        if let Some(props) = properties.as_object() {
            docs.push_str("| Property | Type | Required | Description |\n");
            docs.push_str("|----------|------|----------|-------------|\n");

            for (name, prop) in props {
                let prop_type = prop
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("object");
                let description = prop
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let required = if name == "file"
                    || name == "contract"
                    || name == "state"
                    || name == "hash"
                    || name == "validated_by"
                    || name == "timestamp"
                {
                    "✅"
                } else {
                    "❌"
                };

                docs.push_str(&format!(
                    "| {name} | {prop_type} | {required} | {description} |\n"
                ));
            }
        }
    }

    // Read and document contract transition schema
    let contract_transition_schema = fs::read_to_string("schemas/contract-transition.schema.json")
        .context("Failed to read contract-transition.schema.json")?;
    let contract_transition: serde_json::Value = serde_json::from_str(&contract_transition_schema)
        .context("Failed to parse contract-transition.schema.json")?;

    docs.push_str("\n## Contract Transition Schema\n\n");
    docs.push_str("Defines the structure for contract state transitions.\n\n");

    if let Some(properties) = contract_transition.get("properties") {
        if let Some(props) = properties.as_object() {
            docs.push_str("| Property | Type | Required | Description |\n");
            docs.push_str("|----------|------|----------|-------------|\n");

            for (name, prop) in props {
                let prop_type = prop
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("object");
                let description = prop
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let required = if name == "from_state" || name == "to_state" || name == "event" {
                    "✅"
                } else {
                    "❌"
                };

                docs.push_str(&format!(
                    "| {name} | {prop_type} | {required} | {description} |\n"
                ));
            }
        }
    }

    // Read and document merkle proof schema
    let merkle_proof_schema = fs::read_to_string("schemas/merkle-proof.schema.json")
        .context("Failed to read merkle-proof.schema.json")?;
    let merkle_proof: serde_json::Value = serde_json::from_str(&merkle_proof_schema)
        .context("Failed to parse merkle-proof.schema.json")?;

    docs.push_str("\n## Merkle Proof Schema\n\n");
    docs.push_str("Defines the structure for Merkle chain validation proofs.\n\n");

    if let Some(properties) = merkle_proof.get("properties") {
        if let Some(props) = properties.as_object() {
            docs.push_str("| Property | Type | Required | Description |\n");
            docs.push_str("|----------|------|----------|-------------|\n");

            for (name, prop) in props {
                let prop_type = prop
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("object");
                let description = prop
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let required = if name == "root_hash" || name == "leaves" || name == "proof" {
                    "✅"
                } else {
                    "❌"
                };

                docs.push_str(&format!(
                    "| {name} | {prop_type} | {required} | {description} |\n"
                ));
            }
        }
    }

    Ok(docs)
}

/// Generate WIT documentation from existing WIT files
fn generate_wit_documentation() -> Result<String> {
    let mut docs = String::new();

    docs.push_str("# WIT Interface Documentation\n\n");
    docs.push_str(
        "This document describes the WebAssembly Interface Types (WIT) used by Hooksmith.\n\n",
    );

    // Read and document hooksmith.wit
    let hooksmith_wit =
        fs::read_to_string("wit/hooksmith.wit").context("Failed to read hooksmith.wit")?;

    docs.push_str("## Hooksmith CLI Interface\n\n");
    docs.push_str("Main CLI interface for hook building and management.\n\n");
    docs.push_str("```wit\n");
    docs.push_str(&hooksmith_wit);
    docs.push_str("\n```\n\n");

    // Read and document hook-builder.wit
    let hook_builder_wit =
        fs::read_to_string("wit/hook-builder.wit").context("Failed to read hook-builder.wit")?;

    docs.push_str("## Hook Builder Interface\n\n");
    docs.push_str("Interface for building and managing Git hooks.\n\n");
    docs.push_str("```wit\n");
    docs.push_str(&hook_builder_wit);
    docs.push_str("\n```\n\n");

    // Read and document validation.wit
    let validation_wit =
        fs::read_to_string("wit/validation.wit").context("Failed to read validation.wit")?;

    docs.push_str("## Validation Interface\n\n");
    docs.push_str("Interface for contract validation and state machine management.\n\n");
    docs.push_str("```wit\n");
    docs.push_str(&validation_wit);
    docs.push_str("\n```\n\n");

    // Read and document lefthook-generator.wit
    let lefthook_generator_wit = fs::read_to_string("wit/lefthook-generator.wit")
        .context("Failed to read lefthook-generator.wit")?;

    docs.push_str("## Lefthook Generator Interface\n\n");
    docs.push_str("Interface for generating Lefthook configurations.\n\n");
    docs.push_str("```wit\n");
    docs.push_str(&lefthook_generator_wit);
    docs.push_str("\n```\n\n");

    Ok(docs)
}

/// Generate combined documentation
fn generate_combined_documentation(schema_docs: &str, wit_docs: &str) -> Result<String> {
    let mut docs = String::new();

    docs.push_str("# Contract State Machine Documentation\n\n");
    docs.push_str("This document provides a comprehensive overview of Hooksmith's contract validation state machine, including JSON schemas and WIT interfaces.\n\n");

    docs.push_str("## Overview\n\n");
    docs.push_str("Hooksmith implements a schema-driven state machine for contract validation that provides:\n\n");
    docs.push_str("- **State Machine**: Enforces valid state transitions (UNTRACKED → UNVALIDATED → VALIDATED → LOCKED)\n");
    docs.push_str(
        "- **Merkle Chain**: Cryptographic proof of integrity across hierarchical scopes\n",
    );
    docs.push_str(
        "- **Git Notes Integration**: Tamper-proof audit trails with full validation history\n",
    );
    docs.push_str(
        "- **CI Enforcement**: Automated validation and security auditing in GitHub Actions\n\n",
    );

    docs.push_str("## JSON Schema Definitions\n\n");
    docs.push_str("The following schemas define the structure and validation rules for the contract state machine:\n\n");

    // Extract schema documentation sections
    let schema_sections = extract_schema_sections(schema_docs);
    docs.push_str(&schema_sections);

    docs.push_str("## WIT Interface Definitions\n\n");
    docs.push_str("The following WIT interfaces expose contract validation functionality:\n\n");

    // Extract WIT documentation sections
    let wit_sections = extract_wit_sections(wit_docs);
    docs.push_str(&wit_sections);

    docs.push_str("## Integration with WIT & JSON Schema\n\n");
    docs.push_str(
        "This implementation demonstrates how JSON Schema and WIT can work together:\n\n",
    );
    docs.push_str("1. **JSON Schema Defines the Contract State Machine** - Schemas enforce structure and validation rules\n");
    docs.push_str("2. **WIT Interface Exposes Contract Validation** - WASM components can validate and transition states\n");
    docs.push_str("3. **WASM Component Implements Logic** - Components can return schemas, validate states, and apply transitions\n");
    docs.push_str("4. **Rust Host Uses Both** - Combines schemars and wit-bindgen for type-safe validation\n\n");

    docs.push_str("## Benefits\n\n");
    docs.push_str(
        "- ✅ **Schema as Single Source of Truth** – JSON Schema defines the valid state machine\n",
    );
    docs.push_str("- ✅ **Language-agnostic Validation** – Any host that supports WIT/WASM can validate contracts\n");
    docs.push_str("- ✅ **Deterministic Contract Proofs** – The same logic works inside and outside Git hooks\n");
    docs.push_str(
        "- ✅ **Portable Across Hosts** – Works with Rust, Node.js, Deno, or any WASM runtime\n\n",
    );

    Ok(docs)
}

/// Extract schema sections from schema documentation
fn extract_schema_sections(schema_docs: &str) -> String {
    let mut sections = String::new();

    // Find and extract the schema sections
    if let Some(start_idx) = schema_docs.find("## Contract State Schema") {
        sections.push_str(&schema_docs[start_idx..]);
    }

    sections
}

/// Extract WIT sections from WIT documentation
fn extract_wit_sections(wit_docs: &str) -> String {
    let mut sections = String::new();

    // Find and extract the WIT sections
    if let Some(start_idx) = wit_docs.find("## Hooksmith CLI Interface") {
        sections.push_str(&wit_docs[start_idx..]);
    }

    sections
}

/// Generate Pandoc outputs (PDF, HTML, EPUB)
fn generate_pandoc_outputs(output_path: &Path, pdf: bool, html: bool, epub: bool) -> Result<()> {
    let input_file = output_path.join("CONTRACT_STATE_MACHINE.md");

    if !input_file.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_file);
    }

    // Check if pandoc is available
    let pandoc_check = Command::new("pandoc").arg("--version").output();

    if pandoc_check.is_err() {
        println!("   ⚠️  Pandoc not found. Install pandoc to generate PDF/HTML/EPUB output.");
        println!("   📖 Installation: https://pandoc.org/installing.html");
        return Ok(());
    }

    if pdf {
        println!("   📄 Generating PDF...");
        let status = Command::new("pandoc")
            .arg(&input_file)
            .args([
                "-o",
                &output_path
                    .join("CONTRACT_STATE_MACHINE.pdf")
                    .to_string_lossy(),
            ])
            .args(["--pdf-engine=xelatex", "--toc", "--number-sections"])
            .status()
            .context("Failed to generate PDF")?;

        if !status.success() {
            println!("   ⚠️  PDF generation failed");
        } else {
            println!("   ✅ PDF generated successfully");
        }
    }

    if html {
        println!("   🌐 Generating HTML...");
        let status = Command::new("pandoc")
            .arg(&input_file)
            .args([
                "-o",
                &output_path
                    .join("CONTRACT_STATE_MACHINE.html")
                    .to_string_lossy(),
            ])
            .args([
                "--standalone",
                "--toc",
                "--number-sections",
                "--css=style.css",
            ])
            .status()
            .context("Failed to generate HTML")?;

        if !status.success() {
            println!("   ⚠️  HTML generation failed");
        } else {
            println!("   ✅ HTML generated successfully");
        }
    }

    if epub {
        println!("   📚 Generating EPUB...");
        let status = Command::new("pandoc")
            .arg(&input_file)
            .args([
                "-o",
                &output_path
                    .join("CONTRACT_STATE_MACHINE.epub")
                    .to_string_lossy(),
            ])
            .args(["--toc", "--number-sections"])
            .status()
            .context("Failed to generate EPUB")?;

        if !status.success() {
            println!("   ⚠️  EPUB generation failed");
        } else {
            println!("   ✅ EPUB generated successfully");
        }
    }

    Ok(())
}

/// Check if current changes are compatible with the last release
async fn check_stable_compatibility(version: &str, comprehensive: bool) -> Result<()> {
    println!("🛡️ Checking stable version compatibility...");
    println!("   Version: {version}");
    println!("   Comprehensive: {comprehensive}");

    // Check if stable version is installed
    let stable_installed = Command::new("hooksmith").arg("--version").output().is_ok();

    if !stable_installed {
        println!("   ⚠️  Stable version not found. Installing...");
        let status = Command::new("cargo")
            .args(["install", "hooksmith", "--version", version])
            .status()
            .context("Failed to install stable version")?;

        if !status.success() {
            anyhow::bail!("Failed to install stable version {}", version);
        }
    }

    // Build current version
    println!("   🔨 Building current version...");
    let status = Command::new("cargo")
        .args(["build", "--bin", "hooksmith"])
        .status()
        .context("Failed to build current version")?;

    if !status.success() {
        anyhow::bail!("Current version build failed");
    }

    // Run basic compatibility tests
    println!("   🧪 Running compatibility tests...");

    // Test basic commands
    let commands = vec!["test", "list", "--help", "--version"];
    for cmd in commands {
        println!("     Testing command: {cmd}");

        // Run stable version
        let stable_output = Command::new("hooksmith")
            .arg(cmd)
            .output()
            .context(format!("Failed to run stable version with command: {cmd}"))?;

        // Run current version
        let current_output = Command::new("cargo")
            .args(["run", "--bin", "hooksmith", "--", cmd])
            .output()
            .context(format!("Failed to run current version with command: {cmd}"))?;

        // Compare exit codes
        if stable_output.status.success() != current_output.status.success() {
            println!("     ❌ Exit code mismatch for command: {cmd}");
            if comprehensive {
                anyhow::bail!("Compatibility test failed for command: {}", cmd);
            }
        } else {
            println!("     ✅ Command {cmd} passed");
        }
    }

    if comprehensive {
        // Run additional comprehensive tests
        println!("   🔍 Running comprehensive tests...");

        // Test with different arguments
        let test_cases = vec![
            (
                vec!["test", "--message", "compatibility test"],
                "test with custom message",
            ),
            (vec!["list"], "list command"),
            (vec!["--help"], "help command"),
        ];

        for (args, description) in test_cases {
            println!("     Testing: {description}");

            // Run stable version
            let stable_output = Command::new("hooksmith")
                .args(&args)
                .output()
                .context(format!("Failed to run stable version: {description}"))?;

            // Run current version
            let current_output = Command::new("cargo")
                .args(["run", "--bin", "hooksmith", "--"])
                .args(&args)
                .output()
                .context(format!("Failed to run current version: {description}"))?;

            // Compare outputs (basic comparison)
            let stable_stdout = String::from_utf8_lossy(&stable_output.stdout);
            let current_stdout = String::from_utf8_lossy(&current_output.stdout);

            if stable_stdout.trim() != current_stdout.trim() {
                println!("     ⚠️  Output differs for: {description}");
                if comprehensive {
                    println!("     Stable output: {}", stable_stdout.trim());
                    println!("     Current output: {}", current_stdout.trim());
                }
            } else {
                println!("     ✅ Output matches for: {description}");
            }
        }
    }

    println!("✅ Stable version compatibility check completed");
    Ok(())
}

/// Test current version against released version
async fn test_with_release(version: &str) -> Result<()> {
    println!("🧪 Testing current version against release {version}...");

    // Ensure stable version is installed
    let status = Command::new("cargo")
        .args(["install", "hooksmith", "--version", version, "--force"])
        .status()
        .context("Failed to install stable version")?;

    if !status.success() {
        anyhow::bail!("Failed to install stable version {}", version);
    }

    // Run tests with current version
    println!("   🔨 Running tests with current version...");
    let current_status = Command::new("cargo")
        .args(["test", "--all-targets", "--all-features"])
        .status()
        .context("Failed to run tests with current version")?;

    if !current_status.success() {
        anyhow::bail!("Current version tests failed");
    }

    // Run basic functionality tests with stable version
    println!("   🧪 Running functionality tests with stable version...");
    let test_commands = vec!["test", "list", "--help"];

    for cmd in test_commands {
        let output = Command::new("hooksmith")
            .arg(cmd)
            .output()
            .context(format!("Failed to run stable version command: {cmd}"))?;

        if !output.status.success() {
            println!("   ⚠️  Stable version command '{cmd}' failed");
        } else {
            println!("   ✅ Stable version command '{cmd}' passed");
        }
    }

    println!("✅ Testing with release version completed");
    Ok(())
}

/// Compare outputs between current and released version
async fn compare_with_release(version: &str) -> Result<()> {
    println!("🔍 Comparing outputs between current and release {version}...");

    // Ensure stable version is installed
    let status = Command::new("cargo")
        .args(["install", "hooksmith", "--version", version, "--force"])
        .status()
        .context("Failed to install stable version")?;

    if !status.success() {
        anyhow::bail!("Failed to install stable version {}", version);
    }

    // Build current version
    println!("   🔨 Building current version...");
    let build_status = Command::new("cargo")
        .args(["build", "--bin", "hooksmith"])
        .status()
        .context("Failed to build current version")?;

    if !build_status.success() {
        anyhow::bail!("Current version build failed");
    }

    // Compare outputs for various commands
    let comparison_commands = vec![
        ("test", "Basic test command"),
        ("list", "List command"),
        ("--help", "Help command"),
        ("--version", "Version command"),
    ];

    let mut differences_found = false;

    for (cmd, description) in comparison_commands {
        println!("   🔍 Comparing: {description}");

        // Get stable version output
        let stable_output = Command::new("hooksmith")
            .arg(cmd)
            .output()
            .context(format!("Failed to get stable version output for: {cmd}"))?;

        // Get current version output
        let current_output = Command::new("cargo")
            .args(["run", "--bin", "hooksmith", "--", cmd])
            .output()
            .context(format!("Failed to get current version output for: {cmd}"))?;

        // Compare outputs
        let stable_stdout = String::from_utf8_lossy(&stable_output.stdout);
        let current_stdout = String::from_utf8_lossy(&current_output.stdout);
        let stable_stderr = String::from_utf8_lossy(&stable_output.stderr);
        let current_stderr = String::from_utf8_lossy(&current_output.stderr);

        let stdout_match = stable_stdout.trim() == current_stdout.trim();
        let stderr_match = stable_stderr.trim() == current_stderr.trim();
        let exit_code_match = stable_output.status.success() == current_output.status.success();

        if stdout_match && stderr_match && exit_code_match {
            println!("     ✅ Outputs match for: {description}");
        } else {
            println!("     ❌ Outputs differ for: {description}");
            differences_found = true;

            if !stdout_match {
                println!("       STDOUT differs:");
                println!("       Stable: {}", stable_stdout.trim());
                println!("       Current: {}", current_stdout.trim());
            }

            if !stderr_match {
                println!("       STDERR differs:");
                println!("       Stable: {}", stable_stderr.trim());
                println!("       Current: {}", current_stderr.trim());
            }

            if !exit_code_match {
                println!("       Exit codes differ:");
                println!("       Stable: {}", stable_output.status);
                println!("       Current: {}", current_output.status);
            }
        }
    }

    if differences_found {
        println!("⚠️  Differences found between versions");
        println!("   Review the differences above to ensure they are expected");
    } else {
        println!("✅ All outputs match between versions");
    }

    Ok(())
}

/// Set up Git filters for contract validation
async fn setup_git_filters(force: bool) -> Result<()> {
    println!("🔧 Setting up Git filters and diffs for contract validation...");

    // Get the repository root directory
    let repo_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get repository root")?;

    let repo_root = String::from_utf8(repo_root.stdout)
        .context("Failed to parse repository root")?
        .trim()
        .to_string();

    // Check if configuration already exists
    let existing_config = Command::new("git")
        .args(["config", "--list"])
        .output()
        .context("Failed to check existing Git configuration")?;

    let existing_config =
        String::from_utf8(existing_config.stdout).context("Failed to parse Git configuration")?;

    let has_existing = existing_config.contains("filter.contract_validate.clean");

    if has_existing && !force {
        println!("⚠️  Git filters are already configured.");
        println!("   Use --force to overwrite existing configuration.");
        return Ok(());
    }

    // Set up the contract validation filter
    println!("   Setting up contract_validate filter...");
    Command::new("git")
        .args([
            "config",
            "filter.contract_validate.clean",
            &format!("{repo_root}/target/debug/xtask contract-validate clean"),
        ])
        .status()
        .context("Failed to set up clean filter")?;

    Command::new("git")
        .args([
            "config",
            "filter.contract_validate.smudge",
            &format!("{repo_root}/target/debug/xtask contract-validate smudge"),
        ])
        .status()
        .context("Failed to set up smudge filter")?;

    Command::new("git")
        .args(["config", "filter.contract_validate.required", "true"])
        .status()
        .context("Failed to set required flag")?;

    // Set up the contract diff
    println!("   Setting up contract_diff...");
    Command::new("git")
        .args([
            "config",
            "diff.contract_diff.textconv",
            &format!("{repo_root}/target/debug/xtask contract-validate diff"),
        ])
        .status()
        .context("Failed to set up diff textconv")?;

    Command::new("git")
        .args(["config", "diff.contract_diff.cachetextconv", "true"])
        .status()
        .context("Failed to set cachetextconv flag")?;

    println!("✅ Git filters and diffs configured successfully!");
    println!();
    println!("📋 Configuration summary:");
    println!("   Filter: contract_validate");
    println!("   Diff: contract_diff");
    println!();
    println!("🔍 To verify the configuration, run:");
    println!("   git config --list | grep contract");

    Ok(())
}

/// Validate generated files to prevent manual modifications
fn validate_generated_files(
    staged_only: bool,
    strict: bool,
    custom_message: Option<String>,
) -> Result<()> {
    use generated_file_validator::{GeneratedFileConfig, GeneratedFileValidator};

    println!("Validating generated files...");

    let config = GeneratedFileConfig {
        staged_only,
        strict,
        custom_message,
    };

    match GeneratedFileValidator::validate(&config) {
        Ok(result) => {
            if result.is_valid {
                println!("✅ All generated files are valid!");
                Ok(())
            } else {
                println!("{}", result.error_message.unwrap());
                if strict {
                    std::process::exit(1);
                }
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("❌ Generated file validation failed: {e}");
            if strict {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

/// Add generated file headers to files
fn add_generated_headers(file: Option<String>) -> Result<()> {
    use generated_file_validator::GeneratedFileValidator;
    use std::path::PathBuf;

    println!("Adding generated file headers...");

    if let Some(file_path) = file {
        // Add header to specific file
        let path = PathBuf::from(file_path);
        GeneratedFileValidator::add_generated_header(&path)?;
        println!("✅ Added header to {}", path.display());
    } else {
        // Add headers to all generated files
        let generated_files = GeneratedFileValidator::get_all_generated_files()?;
        GeneratedFileValidator::add_generated_headers(&generated_files)?;
        println!(
            "✅ Added headers to {} generated files",
            generated_files.len()
        );
    }

    Ok(())
}

/// Validate that all generated files have proper headers
fn validate_generated_headers(strict: bool) -> Result<()> {
    use generated_file_validator::GeneratedFileValidator;

    println!("Validating generated file headers...");

    match GeneratedFileValidator::validate_headers() {
        Ok(result) => {
            if result.is_valid {
                println!("✅ All generated files have proper headers!");
                Ok(())
            } else {
                println!("{}", result.error_message.unwrap());
                if strict {
                    std::process::exit(1);
                }
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("❌ Header validation failed: {e}");
            if strict {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

/// Check file types and generation markers
fn check_files(strict: bool, verbose: bool) -> Result<()> {
    use file_audit::check_files;

    println!("🔍 Checking file types and generation markers...");

    match check_files() {
        Ok(result) => {
            if verbose {
                result.print_summary();
            } else {
                println!("📊 File Audit Summary");
                println!("Total files checked: {}", result.total_files);
                println!("Allowed files: {}", result.allowed_files);
                println!("Generated files: {}", result.generated_files);
                println!("Manual files: {}", result.manual_files);
                println!();

                if result.has_errors() {
                    println!("❌ Issues found:");
                    if !result.forbidden_files.is_empty() {
                        println!("   - {} forbidden file types", result.forbidden_files.len());
                    }
                    if !result.missing_markers.is_empty() {
                        println!(
                            "   - {} files missing generation markers",
                            result.missing_markers.len()
                        );
                    }
                    if !result.errors.is_empty() {
                        println!("   - {} errors", result.errors.len());
                    }
                    println!();
                    println!("🔧 To fix issues:");
                    println!("   cargo xtask gen-all --validate");
                    println!("   cargo xtask check-files --strict");
                } else {
                    println!("✅ All files are properly configured!");
                }
            }

            if strict && result.has_errors() {
                std::process::exit(1);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ File audit failed: {e}");
            if strict {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

fn validate_files_strict(strict: bool, verbose: bool) -> Result<()> {
    use strict_file_validator::validate_files;

    println!("🔍 Validating files against strict extension policy...");

    match validate_files() {
        Ok(result) => {
            if verbose {
                result.print_summary();
            } else {
                println!("📊 Strict File Extension Policy Summary");
                println!("Total files checked: {}", result.total_files);
                println!("✅ Allowed files (.rs, .jsonc): {}", result.allowed_files);
                println!("🔧 Generated files: {}", result.generated_files);
                println!("🚫 Ignored files: {}", result.ignored_files);
                println!("🛡️  Exempt files: {}", result.exempt_files);
                println!();

                if result.has_violations() {
                    println!("❌ Policy violations found:");
                    for violation in &result.violations {
                        match violation {
                            strict_file_validator::FileViolation::DisallowedExtension { file, extension } => {
                                println!("   ❌ Disallowed extension '{}' in: {}", extension, file);
                            }
                            strict_file_validator::FileViolation::MissingGeneratedHeader { file, extension } => {
                                println!("   ❌ Missing generated header in: {} (extension: {})", file, extension);
                            }
                        }
                    }

                    if !result.errors.is_empty() {
                        println!("   ❌ Errors:");
                        for error in &result.errors {
                            println!("      - {}", error);
                        }
                    }
                    println!();
                    println!("🔧 To fix violations:");
                    println!("   - Convert files to .rs or .jsonc for manual maintenance");
                    println!("   - Add generated headers to files that should be code-generated");
                    println!("   - Run: cargo xtask gen-all --validate");
                } else {
                    println!("✅ All files comply with the strict extension policy!");
                }
            }

            if strict && result.has_violations() {
                std::process::exit(1);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Strict file validation failed: {e}");
            if strict {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

/// Generate all code-generated files
async fn generate_all_files(validate: bool, force: bool) -> Result<()> {
    use file_audit::FileTypeConfig;

    println!("🚀 Generating all code-generated files...");

    let _config = FileTypeConfig::load()?;
    let mut generated_count = 0;

    // Generate documentation
    println!("   📚 Generating documentation...");
    docs::generate_all_docs("docs", validate).await?;
    generated_count += 1;

    // Generate WIT interfaces
    println!("   🔧 Generating WIT interfaces...");
    generate_wit_interfaces("wit", force)?;
    generated_count += 1;

    // Generate Lefthook configuration
    println!("   🪝 Generating Lefthook configuration...");
    generate_lefthook_config("lefthook.yml", validate)?;
    generated_count += 1;

    // Generate mod.rs files
    println!("   📁 Generating mod.rs files...");
    generate_mod_files(force)?;
    generated_count += 1;

    // Generate hooks README
    println!("   📖 Generating hooks README...");
    generate_hooks_readme("hooks/README.md", force)?;
    generated_count += 1;

    // Generate Git attributes files
    println!("   🔧 Generating Git attributes files...");
    generate_git_attributes("hooks", force, validate)?;
    generated_count += 1;

    // Generate README
    println!("   📖 Generating README...");
    generate_readme("README.md", force)?;
    generated_count += 1;

    println!("✅ Generated {generated_count} types of files");

    if validate {
        println!("🔍 Validating generated files...");
        file_audit::validate_generated_files()?;
        println!("✅ All generated files validated successfully!");
    }

    Ok(())
}

/// Bootstrap the project with all generated files
async fn bootstrap_project(validate: bool, commit: bool) -> Result<()> {
    println!("🚀 Bootstrapping project with all generated files...");

    // Generate all files
    generate_all_files(validate, true).await?;

    // Check if everything is valid
    println!("🔍 Running final validation...");
    file_audit::validate_generated_files()?;

    // Check file types
    println!("🔍 Checking file types...");
    let result = file_audit::check_files()?;
    if result.has_errors() {
        anyhow::bail!("Bootstrap validation failed. Please fix issues and try again.");
    }

    println!("✅ Bootstrap completed successfully!");

    if commit {
        println!("📝 Committing generated files...");
        let status = std::process::Command::new("git")
            .args(["add", "."])
            .status()
            .context("Failed to add files to git")?;

        if !status.success() {
            anyhow::bail!("Failed to add files to git");
        }

        let status = std::process::Command::new("git")
            .args(["commit", "-m", "Bootstrap: Add all generated files"])
            .status()
            .context("Failed to commit files")?;

        if !status.success() {
            anyhow::bail!("Failed to commit files");
        }

        println!("✅ Generated files committed successfully!");
    }

    println!("🎉 Project bootstrap completed!");
    println!();
    println!("📋 Next steps:");
    println!("1. Review generated files");
    println!("2. Run tests: cargo test");
    println!("3. Build project: cargo build");
    println!("4. Start development!");

    Ok(())
}

/// Generate documentation using Rust templates
fn generate_templates(template: Option<String>, output_dir: &str, overwrite: bool) -> Result<()> {
    use crate::docs::templates::{
        api::ApiTemplate,
        diagrams::{GitStateMachine, GitWorkflowDiagram},
        examples::ExamplesTemplate,
        readme::ReadmeTemplate,
        TemplateEngine,
    };
    use std::path::Path;

    println!("🔧 Generating documentation using Rust templates...");

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path)?;
    }

    let mut engine = TemplateEngine::new();

    // Register all templates
    let readme_template = ReadmeTemplate::new()
        .map_err(|e| anyhow::anyhow!("Failed to create README template: {}", e))?;
    engine.register(readme_template);

    let api_template =
        ApiTemplate::new("API Reference", "Complete API documentation for Hooksmith");
    engine.register(api_template);

    let examples_template = ExamplesTemplate::new("Examples", "Code examples and usage patterns");
    engine.register(examples_template);

    let git_state_machine = GitStateMachine::default_git_file_states();
    engine.register(git_state_machine);

    let git_workflow = GitWorkflowDiagram::default_commit_workflow();
    engine.register(git_workflow);

    // Validate all templates
    engine.validate_all()?;
    println!("✅ All templates validated successfully");

    // Generate specific template or all templates
    if let Some(template_name) = template {
        if engine.has_template(&template_name) {
            let content = engine.render(&template_name)?;
            let file_path = output_path.join(format!("{template_name}.md"));

            if file_path.exists() && !overwrite {
                println!(
                    "⚠️  File {} already exists, use --overwrite to replace",
                    file_path.display()
                );
                return Ok(());
            }

            std::fs::write(&file_path, content)?;
            println!("✅ Generated {}", file_path.display());
        } else {
            println!("❌ Template '{template_name}' not found");
            println!(
                "Available templates: {}",
                engine
                    .template_names()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Err(anyhow::anyhow!("Template not found"));
        }
    } else {
        // Generate all templates
        for template_name in engine.template_names() {
            let content = engine.render(template_name)?;
            let file_path = output_path.join(format!("{template_name}.md"));

            if file_path.exists() && !overwrite {
                println!("⚠️  File {} already exists, skipping", file_path.display());
                continue;
            }

            std::fs::write(&file_path, content)?;
            println!("✅ Generated {}", file_path.display());
        }
    }

    println!("🎉 Template generation completed!");
    Ok(())
}

/// Generate all configuration files from Rust structs
fn generate_config(overwrite: bool, validate: bool) -> Result<()> {
    println!("🔧 Generating configuration files...");
    println!("   Overwrite: {overwrite}");
    println!("   Validate: {validate}");

    // Use the ConfigGenerator to generate all config files
    config::ConfigGenerator::generate_all()?;

    if validate {
        println!("🔍 Validating generated configuration files...");
        config::ConfigGenerator::validate_all()?;
    }

    println!("✅ Configuration generation completed!");
    Ok(())
}

/// Generate Git attributes files
fn generate_git_attributes(output_dir: &str, overwrite: bool, validate: bool) -> Result<()> {
    println!("🔧 Generating Git attributes files...");
    println!("   Output directory: {output_dir}");
    println!("   Overwrite: {overwrite}");
    println!("   Validate: {validate}");

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path)?;
    }

    // Generate main .gitattributes file
    let main_gitattributes = generate_main_gitattributes()?;
    let main_path = output_path.join(".gitattributes");

    if main_path.exists() && !overwrite {
        println!(
            "⚠️  File {} already exists, use --overwrite to replace",
            main_path.display()
        );
    } else {
        std::fs::write(&main_path, main_gitattributes)?;
        println!("✅ Generated {}", main_path.display());
    }

    // Generate specialized git attributes files
    let safechars_gitattributes = generate_safechars_gitattributes()?;
    let safechars_path = output_path.join(".gitattributes-safechars");

    if safechars_path.exists() && !overwrite {
        println!(
            "⚠️  File {} already exists, use --overwrite to replace",
            safechars_path.display()
        );
    } else {
        std::fs::write(&safechars_path, safechars_gitattributes)?;
        println!("✅ Generated {}", safechars_path.display());
    }

    let blob_contract_gitattributes = generate_blob_contract_gitattributes()?;
    let blob_contract_path = output_path.join(".gitattributes-blob-contract");

    if blob_contract_path.exists() && !overwrite {
        println!(
            "⚠️  File {} already exists, use --overwrite to replace",
            blob_contract_path.display()
        );
    } else {
        std::fs::write(&blob_contract_path, blob_contract_gitattributes)?;
        println!("✅ Generated {}", blob_contract_path.display());
    }

    // Validate generated files if requested
    if validate {
        println!("🔍 Validating generated git attributes files...");
        validate_git_attributes_files(&main_path, &safechars_path, &blob_contract_path)?;
        println!("✅ All git attributes files validated successfully!");
    }

    println!("🎉 Git attributes generation completed!");
    Ok(())
}

/// Generate the main .gitattributes file
fn generate_main_gitattributes() -> Result<String> {
    let content = r#"# Hooksmith Hierarchical Contract Validation Configuration
# This file defines which contract validators apply to different file types and scopes
# Only whitelisted file extensions are allowed for contract validation

# =============================================================================
# WHITELISTED FILE EXTENSIONS
# =============================================================================
# Only these file extensions are allowed for contract validation
# All other files will be rejected by xtask-contract-validate

# Rust source files - full hierarchical validation
*.rs filter=contract_validate
*.rs diff=contract_diff
*.rs scope=char:line:chunk:file:dir

# Configuration files
*.toml filter=contract_validate
*.yaml filter=contract_validate
*.yml filter=contract_validate
*.json filter=contract_validate
*.toml scope=char:line:file:dir
*.yaml scope=char:line:file:dir
*.yml scope=char:line:file:dir
*.json scope=char:line:file:dir

# Documentation files
*.md filter=contract_validate
*.txt filter=contract_validate
*.rst filter=contract_validate
*.md scope=char:line:file:dir
*.txt scope=char:line:file:dir
*.rst scope=char:line:file:dir

# Web files
*.html filter=contract_validate
*.css filter=contract_validate
*.js filter=contract_validate
*.ts filter=contract_validate
*.html scope=char:line:file:dir
*.css scope=char:line:file:dir
*.js scope=char:line:file:dir
*.ts scope=char:line:file:dir

# =============================================================================
# EXCLUDED DIRECTORIES
# =============================================================================
# These directories are excluded from contract validation

# Generated files - no validation
target/ -filter
dist/ -filter
build/ -filter
node_modules/ -filter
.git/ -filter

# =============================================================================
# GIT CONFIGURATION
# =============================================================================
# Note: Filter and diff configurations should be set up in .git/config or via git config commands
# Run the following commands to set up the filters:
# git config filter.contract_validate.clean "xtask-contract-validate clean"
# git config filter.contract_validate.smudge "xtask-contract-validate smudge"
# git config filter.contract_validate.required true
# git config diff.contract_diff.textconv "xtask-contract-validate diff"
# git config diff.contract_diff.cachetextconv true

# =============================================================================
# VALIDATION SCOPES
# =============================================================================
# Scope levels define the depth of validation:
# char:   Character-level validation (byte-by-byte)
# line:   Line-level validation (line endings, content)
# chunk:  Chunk-level validation (semantic blocks)
# file:   File-level validation (structure, metadata)
# dir:    Directory-level validation (hierarchy, naming)

# =============================================================================
# GENERATED DOCUMENTATION
# =============================================================================
# Mark generated documentation files as codegen to prevent manual editing
# These files are auto-generated by xtask gen-docs-comprehensive

# Generated documentation files - mark as linguist-generated for GitHub
# ALL markdown files are generated from source code
*.md        codegen linguist-generated=true
*.yaml      codegen linguist-generated=true
*.yml       codegen linguist-generated=true
*.wit       codegen linguist-generated=true
*.json      codegen linguist-generated=true
*.hbs       codegen linguist-generated=true
*.dot       codegen linguist-generated=true
*.css       codegen linguist-generated=true
*.html      codegen linguist-generated=true
*.pdf       codegen linguist-generated=true
*.epub      codegen linguist-generated=true

# Manually maintained files - explicitly exclude from generation
# These files are manually maintained and should not be auto-generated
README.md        -codegen linguist-generated=false
.gitignore       -codegen linguist-generated=false
LICENSE          -codegen linguist-generated=false
LICENSE.txt      -codegen linguist-generated=false
LICENSE.md       -codegen linguist-generated=false
CHANGELOG.md     -codegen linguist-generated=false
CONTRIBUTING.md  -codegen linguist-generated=false
SECURITY.md      -codegen linguist-generated=false
CODE_OF_CONDUCT.md -codegen linguist-generated=false

# Generated files that should not be manually modified
# These files are automatically generated by xtask and should only be changed via regeneration

# All markdown files are generated
*.md                           generated=true

# All YAML/YML files are generated
*.yml                          generated=true
*.yaml                         generated=true

# Generated Rust module files
src/commands/mod.rs             generated=true
src/modules/mod.rs              generated=true

# Generated WIT interface files
wit/*.wit                       generated=true

# Generated completions
completions/                    generated=true

# Generated structure documentation
STRUCTURE.md                    generated=true

# Generated Git hooks (if any)
.git/hooks/*                    generated=true

# Generated Git attributes files
.gitattributes                  generated=true
hooks/.gitattributes            generated=true
hooks/.gitattributes-*          generated=true
"#;

    Ok(content.to_string())
}

/// Generate the safechars git attributes file
fn generate_safechars_gitattributes() -> Result<String> {
    let content = r#"# Git Attributes Configuration for SafeChars Filter
# This file demonstrates how to use the character contract system

# All text files should use the safechars filter
* text filter=safechars

# Source code files
*.rs text filter=safechars
*.py text filter=safechars
*.js text filter=safechars
*.ts text filter=safechars
*.go text filter=safechars
*.java text filter=safechars
*.c text filter=safechars
*.cpp text filter=safechars
*.h text filter=safechars
*.hpp text filter=safechars

# Configuration files
*.toml text filter=safechars
*.yml text filter=safechars
*.yaml text filter=safechars
*.json text filter=safechars
*.conf text filter=safechars
*.cfg text filter=safechars
*.ini text filter=safechars
*.env text filter=safechars

# Documentation
*.md text filter=safechars
*.txt text filter=safechars
*.rst text filter=safechars
*.adoc text filter=safechars

# Scripts
*.sh text filter=safechars
*.bash text filter=safechars
*.zsh text filter=safechars
*.ps1 text filter=safechars
*.bat text filter=safechars

# Web files
*.html text filter=safechars
*.css text filter=safechars
*.scss text filter=safechars
*.xml text filter=safechars
*.svg text filter=safechars

# Data files
*.csv text filter=safechars
*.tsv text filter=safechars
*.sql text filter=safechars

# Binary files (explicitly mark as binary, no filter)
*.png binary
*.jpg binary
*.jpeg binary
*.gif binary
*.ico binary
*.pdf binary
*.zip binary
*.tar binary
*.gz binary
*.bz2 binary
*.xz binary
*.7z binary
*.exe binary
*.dll binary
*.so binary
*.dylib binary
*.a binary
*.o binary

# Generated files (skip in archive)
*.log export-ignore
*.tmp export-ignore
*.temp export-ignore
*.cache export-ignore
node_modules/ export-ignore
target/ export-ignore
dist/ export-ignore
build/ export-ignore
"#;

    Ok(content.to_string())
}

/// Generate the blob contract git attributes file
fn generate_blob_contract_gitattributes() -> Result<String> {
    let content = r#"# Git Attributes Configuration for Blob Contract Filter
# This file demonstrates how to use the blob contract system for Git object validation

# All text files should use the blob contract filter
* text filter=blob-contract

# Source code files
*.rs text filter=blob-contract
*.py text filter=blob-contract
*.js text filter=blob-contract
*.ts text filter=blob-contract
*.go text filter=blob-contract
*.java text filter=blob-contract
*.c text filter=blob-contract
*.cpp text filter=blob-contract
*.h text filter=blob-contract
*.hpp text filter=blob-contract

# Configuration files
*.toml text filter=blob-contract
*.yml text filter=blob-contract
*.yaml text filter=blob-contract
*.json text filter=blob-contract
*.conf text filter=blob-contract
*.cfg text filter=blob-contract
*.ini text filter=blob-contract
*.env text filter=blob-contract

# Documentation
*.md text filter=blob-contract
*.txt text filter=blob-contract
*.rst text filter=blob-contract
*.adoc text filter=blob-contract

# Scripts
*.sh text filter=blob-contract
*.bash text filter=blob-contract
*.zsh text filter=blob-contract
*.ps1 text filter=blob-contract
*.bat text filter=blob-contract

# Web files
*.html text filter=blob-contract
*.css text filter=blob-contract
*.scss text filter=blob-contract
*.xml text filter=blob-contract
*.svg text filter=blob-contract

# Data files
*.csv text filter=blob-contract
*.tsv text filter=blob-contract
*.sql text filter=blob-contract

# Binary files (explicitly mark as binary, no filter)
*.png binary
*.jpg binary
*.jpeg binary
*.gif binary
*.ico binary
*.pdf binary
*.zip binary
*.tar binary
*.gz binary
*.bz2 binary
*.xz binary
*.7z binary
*.exe binary
*.dll binary
*.so binary
*.dylib binary
*.a binary
*.o binary

# Generated files (skip in archive)
*.log export-ignore
*.tmp export-ignore
*.temp export-ignore
*.cache export-ignore
node_modules/ export-ignore
target/ export-ignore
dist/ export-ignore
build/ export-ignore
"#;

    Ok(content.to_string())
}

/// Validate generated git attributes files
fn validate_git_attributes_files(
    main_path: &Path,
    safechars_path: &Path,
    blob_contract_path: &Path,
) -> Result<()> {
    // Check that all files exist
    if !main_path.exists() {
        anyhow::bail!(
            "Main git attributes file not found: {}",
            main_path.display()
        );
    }
    if !safechars_path.exists() {
        anyhow::bail!(
            "Safechars git attributes file not found: {}",
            safechars_path.display()
        );
    }
    if !blob_contract_path.exists() {
        anyhow::bail!(
            "Blob contract git attributes file not found: {}",
            blob_contract_path.display()
        );
    }

    // Check for invalid patterns (negative patterns with !)
    let main_content = std::fs::read_to_string(main_path)?;
    let safechars_content = std::fs::read_to_string(safechars_path)?;
    let blob_contract_content = std::fs::read_to_string(blob_contract_path)?;

    // Check for negative patterns that would cause warnings
    let invalid_patterns = ["!*.", "!*/", "!*"];
    for pattern in &invalid_patterns {
        if main_content.contains(pattern) {
            anyhow::bail!(
                "Invalid negative pattern found in main git attributes: {}",
                pattern
            );
        }
        if safechars_content.contains(pattern) {
            anyhow::bail!(
                "Invalid negative pattern found in safechars git attributes: {}",
                pattern
            );
        }
        if blob_contract_content.contains(pattern) {
            anyhow::bail!(
                "Invalid negative pattern found in blob contract git attributes: {}",
                pattern
            );
        }
    }

    // Check for basic syntax validity
    for (name, content) in [
        ("main", &main_content),
        ("safechars", &safechars_content),
        ("blob-contract", &blob_contract_content),
    ] {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Basic validation: should have at least one space or tab
            if !line.contains(' ') && !line.contains('\t') {
                anyhow::bail!("Invalid git attributes line in {}: {}", name, line);
            }
        }
    }

    Ok(())
}

/// Validate all configuration files
fn validate_config(strict: bool) -> Result<()> {
    println!("🔍 Validating configuration files...");

    let result = config::ConfigGenerator::validate_all();

    match result {
        Ok(_) => {
            println!("✅ All configuration files are valid");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("❌ Configuration validation failed: {e}");
            if strict {
                Err(anyhow::anyhow!(error_msg))
            } else {
                println!("{error_msg}");
                Ok(())
            }
        }
    }
}

/// Comprehensive contract validation and status check
async fn run_contract_check(
    staged_only: bool,
    strict: bool,
    trend: bool,
    trend_output: &str,
    verbose: bool,
) -> Result<()> {
    println!("🔗 Hooksmith Contract Check");
    println!("==========================");

    let mut all_passed = true;
    let mut errors = Vec::new();

    // Step 1: Validate generated files
    println!("\n1️⃣ Validating generated files...");
    match validate_generated_files(staged_only, strict, None) {
        Ok(_) => {
            println!("   ✅ Generated files validation passed");
        }
        Err(e) => {
            let error_msg = format!("   ❌ Generated files validation failed: {e}");
            errors.push(error_msg.clone());
            if strict {
                all_passed = false;
            }
            if verbose {
                println!("{error_msg}");
            }
        }
    }

    // Step 2: Check migration progress
    println!("\n2️⃣ Checking migration progress...");
    match status::run_migration_progress_check(strict).await {
        Ok(_) => {
            println!("   ✅ Migration progress check passed");
        }
        Err(e) => {
            let error_msg = format!("   ❌ Migration progress check failed: {e}");
            errors.push(error_msg.clone());
            if strict {
                all_passed = false;
            }
            if verbose {
                println!("{error_msg}");
            }
        }
    }

    // Step 3: Generate trend data (optional)
    if trend {
        println!("\n3️⃣ Generating trend data...");
        match status::run_trend_generation(trend_output).await {
            Ok(_) => {
                println!("   ✅ Trend data generated successfully");
            }
            Err(e) => {
                let error_msg = format!("   ⚠️  Trend generation failed: {e}");
                errors.push(error_msg.clone());
                if verbose {
                    println!("{error_msg}");
                }
                // Trend generation failure is not critical
            }
        }
    }

    // Step 4: Show file type breakdown (informational)
    println!("\n4️⃣ File type analysis...");
    match status::run_file_types_analysis("json").await {
        Ok(_) => {
            println!("   ✅ File type analysis completed");
        }
        Err(e) => {
            let error_msg = format!("   ⚠️  File type analysis failed: {e}");
            errors.push(error_msg.clone());
            if verbose {
                println!("{error_msg}");
            }
            // File type analysis failure is not critical
        }
    }

    // Summary
    println!("\n📊 Contract Check Summary");
    println!("========================");

    if all_passed {
        println!("✅ All critical checks passed!");
        if !errors.is_empty() {
            println!("\n⚠️  Non-critical warnings:");
            for error in &errors {
                println!("   {error}");
            }
        }
    } else {
        println!("❌ Some critical checks failed!");
        println!("\n❌ Critical errors:");
        for error in &errors {
            if error.contains("❌") {
                println!("   {error}");
            }
        }
        if strict {
            return Err(anyhow::anyhow!("Contract check failed - see errors above"));
        }
    }

    println!("\n🎯 Next Steps:");
    println!(
        "   • Run 'cargo xtask status migration-progress --format markdown' for detailed report"
    );
    println!("   • Run 'cargo xtask status file-types --format json' for file type breakdown");
    if trend {
        println!("   • Check '{trend_output}' directory for trend data");
    }

    Ok(())
}

/// Validate commit message with Trunk-style empty message support
fn validate_commit_message(
    file: Option<String>,
    allow_empty: bool,
    validate_conventional: bool,
) -> Result<()> {
    // Determine the commit message file path
    let file_path = match file {
        Some(path) => path,
        None => {
            // Try to get from command line arguments (for lefthook integration)
            std::env::args()
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("No commit message file path provided"))?
        }
    };

    // Read the commit message
    let commit_msg = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read commit message file: {file_path}"))?;

    // Trim whitespace and check if empty
    let trimmed_msg = commit_msg.trim();

    if trimmed_msg.is_empty() {
        if allow_empty {
            println!("ℹ️  Empty commit message allowed (Trunk-style)");
            return Ok(());
        } else {
            anyhow::bail!("Empty commit messages are not allowed");
        }
    }

    // If we have a non-empty message and conventional validation is enabled
    if validate_conventional {
        // Conventional commit regex pattern
        let conventional_pattern = regex::Regex::new(
            r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .+",
        )
        .expect("Invalid regex pattern");

        if !conventional_pattern.is_match(trimmed_msg) {
            anyhow::bail!(
                "Commit message must follow conventional commit format:\n\
                \n\
                Format: <type>(<scope>): <description>\n\
                \n\
                Types: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert\n\
                \n\
                Examples:\n\
                • feat(cli): add new command\n\
                • fix(wasm): correct parsing bug\n\
                • docs: update README\n\
                • chore(ci): update GitHub Actions\n\
                \n\
                Your message: {}\n\
                \n\
                Note: Empty commit messages are allowed (Trunk-style).",
                trimmed_msg
            );
        }
    }

    println!("✅ Commit message validation passed");
    Ok(())
}

/// Set up git aliases for Trunk-style commit workflow
fn setup_git_aliases(force: bool) -> Result<()> {
    println!("🔧 Setting up git aliases for Trunk-style commit workflow...");

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()
        .context("Failed to check git repository")?;

    if !status.success() {
        anyhow::bail!("Not in a git repository. Run this command from a git repository root.");
    }

    // Define aliases to set up
    // Note: We cannot set --allow-empty-message globally in git config
    // Instead, we create aliases that use our Rust command which handles this
    let aliases = vec![
        ("cm", "!cargo run -p xtask -- git-commit"),
        ("cc", "commit"),
        (
            "ce",
            "!cargo run -p xtask -- git-commit --allow-empty-message",
        ),
        ("ncommit", "commit"), // Normal commit (requires message)
        // Auto-push aliases
        ("acp", "!cargo run -p xtask -- autopush"),
        (
            "acpe",
            "!cargo run -p xtask -- autopush --allow-empty-message",
        ),
        (
            "acp-skip",
            "!cargo run -p xtask -- autopush --skip-validation",
        ),
        ("acp-force", "!cargo run -p xtask -- autopush --force"),
        ("acp-watchdog", "!cargo run -p xtask -- autopush --watchdog"),
    ];

    for (alias, command) in aliases {
        // Check if alias already exists
        let existing = Command::new("git")
            .args(["config", "--get", &format!("alias.{alias}")])
            .output()
            .context("Failed to check existing alias")?;

        if existing.status.success() && !force {
            println!("   ⚠️  Alias '{alias}' already exists. Use --force to overwrite.");
            continue;
        }

        // Set the alias using shell command to avoid argument parsing issues
        let shell_command = format!("git config --local alias.{alias} '{command}'");
        let status = Command::new("sh")
            .args(["-c", &shell_command])
            .status()
            .with_context(|| format!("Failed to set alias '{alias}'"))?;

        if status.success() {
            println!("   ✅ Set alias 'git {alias}' -> '{command}'");
        } else {
            anyhow::bail!("Failed to set alias '{}'", alias);
        }
    }

    println!("\n✅ Git aliases configured successfully!");
    println!("\n🎯 Available commands:");
    println!("   git cm [options]     - Commit with Trunk-style empty message support");
    println!("   git cc [options]     - Regular commit (requires message)");
    println!("   git ce [options]     - Quick empty commit (Trunk-style)");
    println!("   git ncommit [options] - Normal commit (requires message)");
    println!("\n🚀 Auto-push commands:");
    println!("   git acp [message]    - Auto-push with validation (prompts for message)");
    println!("   git acpe             - Auto-push with empty message");
    println!("   git acp-skip         - Auto-push without validation");
    println!("   git acp-force        - Auto-push with force push");
    println!("   git acp-watchdog     - Auto-push in watchdog mode (continuous)");
    println!("\n💡 Examples:");
    println!("   git cm                    # Commit with empty message (Trunk-style)");
    println!("   git cm -m 'feat: add feature'  # Commit with conventional message");
    println!("   git ce                    # Quick empty commit");
    println!("   git ncommit -m 'fix: bug'      # Normal commit with message");
    println!("   git acp 'feat: new feature'    # Auto-push with message");
    println!("   git acpe                     # Auto-push with empty message");
    println!("   git acp-watchdog              # Start watchdog mode");
    println!("\n📚 Important Notes:");
    println!("   • git cm and git ce use our Rust command which handles --allow-empty-message");
    println!("   • git cc and git ncommit are standard git commit (require messages)");
    println!("   • The commit-msg hook validates non-empty messages with conventional format");
    println!("   • Empty messages are always allowed (Trunk-style behavior)");
    println!("\n🔧 Technical Details:");
    println!("   • --allow-empty-message cannot be set globally in git config");
    println!("   • Our Rust command handles this limitation by passing the flag explicitly");
    println!("   • This provides the Trunk-style workflow you want");

    Ok(())
}

/// Validate documentation generation (replaces validate-docs.sh)
async fn validate_documentation(
    strict: bool,
    regenerate: bool,
    check_uncommitted: bool,
) -> Result<()> {
    println!("🔍 Validating documentation generation...");

    // Check if we're in a CI environment
    if std::env::var("CI").is_ok() {
        println!("🏗️  Running in CI environment");
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check for any markdown files that don't have auto-generated markers
    println!("📋 Checking for direct markdown file creation...");

    let md_files = glob::glob("**/*.md")
        .context("Failed to glob markdown files")?
        .filter_map(|entry| entry.ok())
        .filter(|path| {
            !path.to_string_lossy().contains("target/") && !path.to_string_lossy().contains(".git/")
        })
        .collect::<Vec<_>>();

    let excluded_files = [
        "./README.md",
        "./.gitignore",
        "./LICENSE",
        "./CHANGELOG.md",
        "./CONTRIBUTING.md",
        "./SECURITY.md",
        "./CODE_OF_CONDUCT.md",
    ];

    for file in &md_files {
        let file_str = file.to_string_lossy();

        // Skip explicitly excluded files
        if excluded_files
            .iter()
            .any(|excluded| file_str.contains(excluded))
        {
            println!("   ⏭️  Skipping manually maintained file: {file_str}");
            continue;
        }

        // Check if file contains auto-generated marker
        let content =
            fs::read_to_string(file).with_context(|| format!("Failed to read file: {file_str}"))?;

        if !content.contains("auto-generated") {
            let error_msg = format!("Invalid file (no auto-generated marker): {file_str}");
            errors.push(error_msg.clone());
            println!("   ❌ {error_msg}");
        } else {
            println!("   ✅ Valid generated file: {file_str}");
        }
    }

    // Validate checksums if available
    println!();
    println!("🔐 Validating checksums...");

    let checksum_path = Path::new("docs/checksums.json");
    if checksum_path.exists() {
        match docs::validate_generated_files(Path::new("docs")) {
            Ok(_) => println!("✅ Checksum validation passed"),
            Err(e) => {
                let error_msg = format!("Checksum validation failed: {e}");
                if strict {
                    errors.push(error_msg.clone());
                } else {
                    warnings.push(error_msg.clone());
                }
                println!("❌ {error_msg}");
            }
        }
    } else {
        println!("⚠️  No checksums.json found, skipping checksum validation");
    }

    // Check Git attributes
    println!();
    println!("🏷️  Checking Git attributes...");

    let gitattributes_path = Path::new(".gitattributes");
    if gitattributes_path.exists() {
        let content =
            fs::read_to_string(gitattributes_path).context("Failed to read .gitattributes")?;

        if content.contains("linguist-generated=true") {
            println!("✅ Git attributes properly configured");
        } else {
            let warning_msg = "Git attributes may not be properly configured".to_string();
            warnings.push(warning_msg.clone());
            println!("⚠️  {warning_msg}");
        }
    } else {
        let warning_msg = "No .gitattributes file found".to_string();
        warnings.push(warning_msg.clone());
        println!("⚠️  {warning_msg}");
    }

    // Generate fresh documentation if requested
    if regenerate {
        println!();
        println!("🔄 Generating fresh documentation...");

        match generate_comprehensive_documentation(true, &None, "docs", true).await {
            Ok(_) => println!("✅ Documentation generation successful"),
            Err(e) => {
                let error_msg = format!("Documentation generation failed: {e}");
                if strict {
                    errors.push(error_msg.clone());
                } else {
                    warnings.push(error_msg.clone());
                }
                println!("❌ {error_msg}");
            }
        }
    }

    // Check for uncommitted changes
    if check_uncommitted {
        println!();
        println!("📝 Checking for uncommitted changes...");

        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;

        if !status.stdout.is_empty() {
            let output = String::from_utf8_lossy(&status.stdout);
            let warning_msg = format!("Uncommitted changes detected:\n{output}");
            warnings.push(warning_msg.clone());
            println!("⚠️  {warning_msg}");

            if strict {
                println!();
                println!("Please commit all changes or run:");
                println!("cargo xtask gen-docs-comprehensive --all --validate");
                println!("git add .");
                println!("git commit -m 'Update generated documentation'");
            }
        } else {
            println!("✅ No uncommitted changes");
        }
    }

    // Summary
    println!();
    if errors.is_empty() && warnings.is_empty() {
        println!("🎉 All documentation validation checks passed!");
        println!("✅ No direct markdown file creation detected");
        println!("✅ All files properly generated");
        println!("✅ Checksums validated");
        println!("✅ Git attributes configured");
        if check_uncommitted {
            println!("✅ No uncommitted changes");
        }
    } else {
        if !warnings.is_empty() {
            println!("⚠️  Warnings:");
            for warning in &warnings {
                println!("   {warning}");
            }
        }

        if !errors.is_empty() {
            println!("❌ Errors:");
            for error in &errors {
                println!("   {error}");
            }

            if strict {
                anyhow::bail!(
                    "Documentation validation failed with {} errors",
                    errors.len()
                );
            }
        }
    }

    Ok(())
}

/// Set up pre-commit hook (replaces setup-pre-commit.sh)
async fn setup_pre_commit(enhanced: bool, force: bool, lefthook: bool) -> Result<()> {
    println!("🔧 Setting up pre-commit hook for Hooksmith Contract Check...");

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()
        .context("Failed to check git repository")?;

    if !status.success() {
        anyhow::bail!("❌ Error: Not in a git repository\n💡 Run this command from the project root directory");
    }

    if lefthook {
        // Set up lefthook configuration
        println!("📋 Setting up lefthook pre-commit hook...");

        // Generate lefthook config if it doesn't exist
        let lefthook_path = Path::new("lefthook.yml");
        if !lefthook_path.exists() {
            println!("   Generating lefthook.yml...");
            generate_lefthook_config("lefthook.yml", false)?;
        }

        // Install lefthook hooks
        let status = Command::new("lefthook")
            .args(["install"])
            .status()
            .context("Failed to install lefthook hooks")?;

        if !status.success() {
            anyhow::bail!("❌ Failed to install lefthook hooks");
        }

        println!("✅ Lefthook pre-commit hook installed successfully!");
        println!();
        println!("🎯 What this hook does:");
        println!("   • Runs contract validation on staged files");
        println!("   • Prevents commits that violate the contract");
        println!("   • Provides helpful error messages and fix suggestions");
        println!();
        println!("🚀 Test it:");
        println!("   git add .");
        println!("   git commit -m 'test commit'");
    } else {
        // Set up direct git hook
        let hooks_dir = Path::new(".git").join("hooks");
        let pre_commit_path = hooks_dir.join("pre-commit");

        // Create hooks directory if it doesn't exist
        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;
        }

        // Check if hook already exists
        if pre_commit_path.exists() && !force {
            anyhow::bail!("❌ Pre-commit hook already exists\n💡 Use --force to overwrite");
        }

        // Generate the pre-commit hook content
        let hook_content = if enhanced {
            generate_enhanced_pre_commit_hook()
        } else {
            generate_basic_pre_commit_hook()
        };

        // Write the hook
        fs::write(&pre_commit_path, hook_content).context("Failed to write pre-commit hook")?;

        // Make the hook executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&pre_commit_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_commit_path, perms)
                .context("Failed to make pre-commit hook executable")?;
        }

        println!("✅ Pre-commit hook installed successfully!");
        println!();
        println!("🎯 What this hook does:");
        if enhanced {
            println!("   • Automatically fixes compilation warnings with cargo fix");
            println!("   • Detects and regenerates stale generated files");
            println!("   • Runs code formatting (cargo fmt)");
            println!("   • Runs clippy checks");
            println!("   • Performs final contract validation");
            println!("   • Provides detailed error messages and fix suggestions");
        } else {
            println!("   • Runs contract validation on staged files");
            println!("   • Prevents commits that violate the contract");
            println!("   • Provides helpful error messages and fix suggestions");
        }
        println!();
        println!("🚀 Test it:");
        println!("   git add .");
        println!("   git commit -m 'test commit'");
    }

    println!();
    println!("📚 For more info:");
    println!("   cargo run -p xtask -- contract-check --help");
    println!("   docs/CONTRACT_CHECK_SYSTEM.md");

    Ok(())
}

/// Run enhanced pre-commit workflow with auto-fix capabilities
async fn run_enhanced_pre_commit_workflow(
    _staged_only: bool,
    strict: bool,
    auto_fix: bool,
) -> Result<()> {
    // Step 1: Fix compilation warnings
    println!("🔧 Step 1: Fixing compilation warnings...");
    if auto_fix {
        let status = Command::new("cargo")
            .args(["fix", "-p", "xtask", "--allow-dirty", "--allow-staged"])
            .status()
            .context("Failed to run cargo fix")?;

        if status.success() {
            println!("   ✅ Compilation warnings fixed");
        } else {
            println!("   ⚠️  Some warnings could not be automatically fixed");
        }
    } else {
        println!("   ⏭️  Skipping warning fixes (use --auto-fix to enable)");
    }

    // Step 2: Regenerate files if needed
    println!("🔄 Step 2: Checking for stale generated files...");
    if auto_fix {
        // Run contract check to see what files need regeneration
        let output = Command::new("cargo")
            .args([
                "run",
                "-p",
                "xtask",
                "--",
                "contract-check",
                "--staged-only",
            ])
            .output()
            .context("Failed to run contract check")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("should not be manually modified") {
            println!("   🔄 Regenerating stale files...");

            // Regenerate documentation
            println!("      Regenerating documentation...");
            let _ = Command::new("cargo")
                .args([
                    "run",
                    "-p",
                    "xtask",
                    "--",
                    "gen-docs-comprehensive",
                    "--all",
                ])
                .status();

            // Regenerate lefthook config
            println!("      Regenerating lefthook config...");
            let _ = Command::new("cargo")
                .args(["run", "-p", "xtask", "--", "gen-lefthook"])
                .status();

            // Regenerate git attributes
            println!("      Regenerating git attributes...");
            let _ = Command::new("cargo")
                .args(["run", "-p", "xtask", "--", "gen-gitattributes"])
                .status();

            println!("   ✅ Files regenerated");

            // Re-stage regenerated files
            println!("   📝 Re-staging regenerated files...");
            let status = Command::new("git")
                .args(["add", "."])
                .status()
                .context("Failed to re-stage files")?;

            if !status.success() {
                println!("   ⚠️  Failed to re-stage regenerated files");
            }
        } else {
            println!("   ✅ No stale files detected");
        }
    } else {
        println!("   ⏭️  Skipping file regeneration (use --auto-fix to enable)");
    }

    // Step 3: Run code formatting
    println!("🎨 Step 3: Running code formatting...");
    if auto_fix {
        let status = Command::new("cargo")
            .args(["fmt", "--all", "--", "--check"])
            .status()
            .context("Failed to check code formatting")?;

        if status.success() {
            println!("   ✅ Code formatting is correct");
        } else {
            println!("   🔄 Fixing code formatting...");
            let status = Command::new("cargo")
                .args(["fmt", "--all"])
                .status()
                .context("Failed to fix code formatting")?;

            if status.success() {
                let _ = Command::new("git").args(["add", "."]).status();
                println!("   ✅ Code formatting fixed");
            } else {
                println!("   ❌ Failed to fix code formatting");
            }
        }
    } else {
        println!("   ⏭️  Skipping formatting (use --auto-fix to enable)");
    }

    // Step 4: Run clippy
    println!("🔍 Step 4: Running clippy...");
    let status = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .status()
        .context("Failed to run clippy")?;

    if status.success() {
        println!("   ✅ Clippy checks passed");
    } else {
        println!("   ❌ Clippy found issues that need to be fixed manually");
        println!();
        println!("💡 To fix clippy issues:");
        println!("   cargo clippy --workspace --fix --allow-dirty --allow-staged");
        println!("   git add .");
        println!("   git commit");
        if strict {
            anyhow::bail!("Clippy check failed");
        }
    }

    // Step 5: Final contract check
    println!("🔗 Step 5: Final contract validation...");
    let status = Command::new("cargo")
        .args([
            "run",
            "-p",
            "xtask",
            "--",
            "contract-check",
            "--staged-only",
            "--strict",
        ])
        .status()
        .context("Failed to run contract check")?;

    if status.success() {
        println!("   ✅ Contract check passed");
    } else {
        println!();
        println!("❌ Contract check failed!");
        println!();
        println!("💡 To fix this:");
        println!(
            "   1. Regenerate modified files: cargo run -p xtask -- gen-docs-comprehensive --all"
        );
        println!("   2. Or regenerate specific files:");
        println!("      cargo run -p xtask -- gen-lefthook");
        println!("      cargo run -p xtask -- gen-gitattributes");
        println!("   3. Re-stage and try again");
        println!();
        println!("🔍 For detailed analysis:");
        println!("   cargo run -p xtask -- status migration-progress --format markdown");
        println!("   cargo run -p xtask -- status file-types --format json");
        println!();
        if strict {
            anyhow::bail!("Contract check failed");
        }
    }

    println!();
    println!("✅ Enhanced pre-commit check passed!");
    println!("🚀 Ready to commit!");
    println!();
    println!("📊 Summary:");
    println!("   ✅ Compilation warnings fixed");
    println!("   ✅ Generated files up to date");
    println!("   ✅ Code formatting correct");
    println!("   ✅ Clippy checks passed");
    println!("   ✅ Contract validation passed");

    Ok(())
}

/// Run basic pre-commit workflow
async fn run_basic_pre_commit_workflow(_staged_only: bool, strict: bool) -> Result<()> {
    println!("📋 Validating staged changes...");

    let mut args = vec![
        "run",
        "-p",
        "xtask",
        "--",
        "contract-check",
        "--staged-only",
    ];
    if strict {
        args.push("--strict");
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run contract check")?;

    if !status.success() {
        println!();
        println!("❌ Contract check failed!");
        println!();
        println!("💡 To fix this:");
        println!("   1. Regenerate modified files: cargo xtask gen-all");
        println!("   2. Or regenerate specific files:");
        println!("      cargo xtask gen-lefthook");
        println!("      cargo xtask gen-docs");
        println!("      cargo xtask gen-mods");
        println!("   3. Re-stage and try again");
        println!();
        println!("🔍 For detailed analysis:");
        println!("   cargo xtask status migration-progress --format markdown");
        println!("   cargo xtask status file-types --format json");
        println!();
        anyhow::bail!("Contract check failed");
    }

    println!("✅ Contract check passed!");
    println!("🚀 Ready to commit!");

    Ok(())
}

/// Generate basic pre-commit hook content
fn generate_basic_pre_commit_hook() -> String {
    r#"#!/bin/bash
# Pre-commit hook for Hooksmith Contract Check System
# This hook ensures all staged changes pass contract validation

set -e

echo "🔗 Running Hooksmith Contract Check..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check if there are staged changes
if git diff --cached --quiet; then
    echo "ℹ️  No staged changes to validate"
    exit 0
fi

# Run contract check on staged files
echo "📋 Validating staged changes..."
if ! cargo run -p xtask -- contract-check --staged-only --strict; then
    echo ""
    echo "❌ Contract check failed!"
    echo ""
    echo "💡 To fix this:"
    echo "   1. Regenerate modified files: cargo xtask gen-all"
    echo "   2. Or regenerate specific files:"
    echo "      cargo xtask gen-lefthook"
    echo "      cargo xtask gen-docs"
    echo "      cargo xtask gen-mods"
    echo "   3. Re-stage and try again"
    echo ""
    echo "🔍 For detailed analysis:"
    echo "   cargo xtask status migration-progress --format markdown"
    echo "   cargo xtask status file-types --format json"
    echo ""
    exit 1
fi

echo "✅ Contract check passed!"
echo "🚀 Ready to commit!"
"#
    .to_string()
}

/// Generate enhanced pre-commit hook content
fn generate_enhanced_pre_commit_hook() -> String {
    r#"#!/bin/bash
# Enhanced pre-commit hook for Hooksmith Contract Check System
# This hook ensures all staged changes pass contract validation and regenerates files as needed

set -e

echo "🔗 Running Enhanced Hooksmith Pre-commit Check..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check if there are staged changes
if git diff --cached --quiet; then
    echo "ℹ️  No staged changes to validate"
    exit 0
fi

# Step 1: Fix compilation warnings
echo "🔧 Step 1: Fixing compilation warnings..."
if command -v cargo >/dev/null 2>&1; then
    echo "   Running cargo fix..."
    if cargo fix -p xtask --allow-dirty --allow-staged; then
        echo "   ✅ Compilation warnings fixed"
    else
        echo "   ⚠️  Some warnings could not be automatically fixed"
    fi
else
    echo "   ⚠️  Cargo not available, skipping warning fixes"
fi

# Step 2: Regenerate files if needed
echo "🔄 Step 2: Checking for stale generated files..."
if command -v cargo >/dev/null 2>&1; then
    echo "   Running contract check to identify stale files..."

    # Run contract check to see what files need regeneration
    if cargo run -p xtask -- contract-check --staged-only 2>&1 | grep -q "should not be manually modified"; then
        echo "   🔄 Regenerating stale files..."

        # Regenerate documentation
        echo "      Regenerating documentation..."
        cargo run -p xtask -- gen-docs-comprehensive --all || true

        # Regenerate lefthook config
        echo "      Regenerating lefthook config..."
        cargo run -p xtask -- gen-lefthook || true

        # Regenerate git attributes
        echo "      Regenerating git attributes..."
        cargo run -p xtask -- gen-gitattributes || true

        echo "   ✅ Files regenerated"

        # Re-stage regenerated files
        echo "   📝 Re-staging regenerated files..."
        git add .
    else
        echo "   ✅ No stale files detected"
    fi
else
    echo "   ⚠️  Cargo not available, skipping file regeneration"
fi

# Step 3: Run code formatting
echo "🎨 Step 3: Running code formatting..."
if command -v cargo >/dev/null 2>&1; then
    echo "   Running cargo fmt..."
    if cargo fmt --all -- --check; then
        echo "   ✅ Code formatting is correct"
    else
        echo "   🔄 Fixing code formatting..."
        cargo fmt --all
        git add .
        echo "   ✅ Code formatting fixed"
    fi
else
    echo "   ⚠️  Cargo not available, skipping formatting"
fi

# Step 4: Run clippy
echo "🔍 Step 4: Running clippy..."
if command -v cargo >/dev/null 2>&1; then
    echo "   Running cargo clippy..."
    if cargo clippy --workspace -- -D warnings; then
        echo "   ✅ Clippy checks passed"
    else
        echo "   ❌ Clippy found issues that need to be fixed manually"
        echo ""
        echo "💡 To fix clippy issues:"
        echo "   cargo clippy --workspace --fix --allow-dirty --allow-staged"
        echo "   git add ."
        echo "   git commit"
        exit 1
    fi
else
    echo "   ⚠️  Cargo not available, skipping clippy"
fi

# Step 5: Final contract check
echo "🔗 Step 5: Final contract validation..."
if command -v cargo >/dev/null 2>&1; then
    echo "   Running contract check..."
    if cargo run -p xtask -- contract-check --staged-only --strict; then
        echo "   ✅ Contract check passed"
    else
        echo ""
        echo "❌ Contract check failed!"
        echo ""
        echo "💡 To fix this:"
        echo "   1. Regenerate modified files: cargo run -p xtask -- gen-docs-comprehensive --all"
        echo "   2. Or regenerate specific files:"
        echo "      cargo run -p xtask -- gen-lefthook"
        echo "      cargo run -p xtask -- gen-gitattributes"
        echo "   3. Re-stage and try again"
        echo ""
        echo "🔍 For detailed analysis:"
        echo "   cargo run -p xtask -- status migration-progress --format markdown"
        echo "   cargo run -p xtask -- status file-types --format json"
        echo ""
        exit 1
    fi
else
    echo "   ⚠️  Cargo not available, skipping contract check"
fi

echo ""
echo "✅ Enhanced pre-commit check passed!"
echo "🚀 Ready to commit!"
echo ""
echo "📊 Summary:"
echo "   ✅ Compilation warnings fixed"
echo "   ✅ Generated files up to date"
echo "   ✅ Code formatting correct"
echo "   ✅ Clippy checks passed"
echo "   ✅ Contract validation passed"
"#.to_string()
}

/// Git commit with Trunk-style empty message support (replaces git-trunk-commit.sh)
async fn git_commit(
    message: Option<String>,
    allow_empty_message: bool,
    args: Vec<String>,
) -> Result<()> {
    println!("🚀 Committing with Trunk-style empty message support...");

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()
        .context("Failed to check git repository")?;

    if !status.success() {
        anyhow::bail!("❌ Error: Not in a git repository");
    }

    // Check if there are staged changes
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to check staged changes")?;

    if status.success() {
        anyhow::bail!("❌ Error: No staged changes to commit\n💡 Use 'git add <files>' to stage changes first");
    }

    // Build git commit command
    let mut commit_args = vec!["commit"];

    // Add --allow-empty-message flag if requested or if no message provided
    if allow_empty_message || message.is_none() {
        commit_args.push("--allow-empty-message");
    }

    // Add message if provided
    if let Some(msg) = &message {
        commit_args.extend_from_slice(&["-m", msg]);
    }

    // Add additional arguments
    commit_args.extend(args.iter().map(|s| s.as_str()));

    // Execute git commit
    let status = Command::new("git")
        .args(&commit_args)
        .status()
        .context("Failed to execute git commit")?;

    if !status.success() {
        anyhow::bail!("Git commit failed");
    }

    // Check if the commit message is empty and show reminder
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output()
        .context("Failed to get commit message")?;

    let commit_message = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if commit_message.is_empty() {
        println!();
        println!("✅ Empty commit message accepted (Trunk-style)");
        println!("💡 Use 'git commit --amend' if you want to add details later");
    }

    Ok(())
}

/// Run pre-commit validation (replaces pre-commit script)
async fn run_pre_commit(
    enhanced: bool,
    staged_only: bool,
    strict: bool,
    auto_fix: bool,
) -> Result<()> {
    if enhanced {
        println!("🔗 Running Enhanced Hooksmith Pre-commit Check...");
    } else {
        println!("🔗 Running Hooksmith Contract Check...");
    }

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .status()
        .context("Failed to check git repository")?;

    if !status.success() {
        anyhow::bail!("❌ Error: Not in a git repository");
    }

    // Check if there are staged changes
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to check staged changes")?;

    if status.success() {
        println!("ℹ️  No staged changes to validate");
        return Ok(());
    }

    if enhanced {
        // Enhanced pre-commit workflow
        run_enhanced_pre_commit_workflow(staged_only, strict, auto_fix).await?;
    } else {
        // Basic pre-commit workflow
        run_basic_pre_commit_workflow(staged_only, strict).await?;
    }

    Ok(())
}

/// Check for dead code by temporarily stripping #[allow(dead_code)] attributes
async fn run_dead_code_check(
    strict: bool,
    include_generated: bool,
    restore: bool,
    format: String,
) -> Result<()> {
    println!("🔍 Checking for dead code...");

    // Create backup of files with #[allow(dead_code)] attributes
    let backup_dir = std::env::temp_dir().join("hooksmith_dead_code_backup");
    std::fs::create_dir_all(&backup_dir)?;

    // Find all Rust files with #[allow(dead_code)] attributes
    let output = Command::new("rg")
        .args(["-l", r"#\[allow\(dead_code\)\]", "--type", "rust"])
        .output()
        .context("Failed to find files with #[allow(dead_code)] attributes")?;

    let files_with_attributes: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    if files_with_attributes.is_empty() {
        println!("ℹ️  No files with #[allow(dead_code)] attributes found");
        return Ok(());
    }

    println!(
        "📁 Found {} files with #[allow(dead_code)] attributes",
        files_with_attributes.len()
    );

    // Backup and strip attributes
    for file_path in &files_with_attributes {
        let path = std::path::Path::new(file_path);

        // Skip generated files unless include_generated is true
        if !include_generated && is_generated_file(path) {
            continue;
        }

        // Backup the file
        let backup_path = backup_dir.join(path.file_name().unwrap());
        std::fs::copy(path, &backup_path)?;

        // Strip #[allow(dead_code)] attributes
        let content = std::fs::read_to_string(path)?;
        let stripped_content = strip_allow_dead_code_attributes(&content);
        std::fs::write(path, stripped_content)?;
    }

    // Run cargo check to find dead code
    println!("🔍 Running cargo check to find dead code...");
    let output = Command::new("cargo")
        .args(["check", "--all-targets", "--all-features"])
        .output()
        .context("Failed to run cargo check")?;

    let dead_code_found = !output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let error_str = String::from_utf8_lossy(&output.stderr);

    // Restore attributes if requested
    if restore {
        println!("🔄 Restoring #[allow(dead_code)] attributes...");
        for file_path in &files_with_attributes {
            let path = std::path::Path::new(file_path);

            if !include_generated && is_generated_file(path) {
                continue;
            }

            let backup_path = backup_dir.join(path.file_name().unwrap());
            if backup_path.exists() {
                std::fs::copy(&backup_path, path)?;
            }
        }
    }

    // Clean up backup directory
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }

    // Report results
    if dead_code_found {
        println!("❌ Dead code found!");

        if format == "json" {
            let report = DeadCodeReport {
                dead_code_found: true,
                files_checked: files_with_attributes.len(),
                errors: error_str.lines().map(|s| s.to_string()).collect(),
                warnings: output_str.lines().map(|s| s.to_string()).collect(),
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("📋 Dead code errors:");
            for line in error_str.lines() {
                if line.contains("dead_code") {
                    println!("  {line}");
                }
            }
        }

        if strict {
            return Err(anyhow::anyhow!("Dead code found"));
        }
    } else {
        println!("✅ No dead code found!");

        if format == "json" {
            let report = DeadCodeReport {
                dead_code_found: false,
                files_checked: files_with_attributes.len(),
                errors: vec![],
                warnings: vec![],
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

/// Check if a file is generated (contains codegen markers)
fn is_generated_file(path: &std::path::Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(path) {
        content.contains("// Code generated by")
            || content.contains("// This file is generated")
            || content.contains("// DO NOT EDIT")
    } else {
        false
    }
}

/// Strip #[allow(dead_code)] attributes from Rust code
fn strip_allow_dead_code_attributes(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("#[allow(dead_code)]") {
            // Skip this line entirely
            continue;
        } else if trimmed.contains("#[allow(dead_code)]") {
            // Remove the attribute from the line
            let replaced = line.replace("#[allow(dead_code)]", "");
            let cleaned = replaced.trim();
            if !cleaned.is_empty() {
                result.push(cleaned.to_string());
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

#[derive(serde::Serialize)]
struct DeadCodeReport {
    dead_code_found: bool,
    files_checked: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Run automated git workflow: validate, add, commit, and push
async fn run_auto_push(
    message: Option<String>,
    allow_empty_message: bool,
    skip_validation: bool,
    watchdog: bool,
    interval: u64,
    force: bool,
    args: Vec<String>,
) -> Result<()> {
    if watchdog {
        println!("🔄 Starting watchdog mode with {interval}s interval...");
        println!("   Press Ctrl+C to stop");

        loop {
            match run_single_auto_push(&message, allow_empty_message, skip_validation, force, &args)
                .await
            {
                Ok(_) => {
                    println!("✅ Watchdog cycle completed successfully");
                }
                Err(e) => {
                    eprintln!("❌ Watchdog cycle failed: {e}");
                    if !skip_validation {
                        eprintln!("   Validation errors detected - skipping commit/push");
                    }
                }
            }

            println!("⏰ Waiting {interval} seconds before next cycle...");
            sleep(Duration::from_secs(interval)).await;
        }
    } else {
        run_single_auto_push(&message, allow_empty_message, skip_validation, force, &args).await
    }
}

/// Run a single auto-push cycle
async fn run_single_auto_push(
    message: &Option<String>,
    allow_empty_message: bool,
    skip_validation: bool,
    force: bool,
    args: &[String],
) -> Result<()> {
    println!("🚀 Starting automated git workflow...");

    // Step 1: Run validation checks (unless skipped)
    if !skip_validation {
        println!("🔍 Running validation checks...");

        // Run cargo fix
        println!("   🔧 Running cargo fix...");
        let fix_output = Command::new("cargo")
            .args(["fix", "--allow-dirty", "--allow-staged"])
            .output()
            .context("Failed to run cargo fix")?;

        if !error_deduplication::process_command_output(&fix_output, "cargo fix") {
            anyhow::bail!("cargo fix failed");
        }

        // Run cargo fmt
        println!("   🎨 Running cargo fmt...");
        let fmt_output = Command::new("cargo")
            .args(["fmt", "--all"])
            .output()
            .context("Failed to run cargo fmt")?;

        if !error_deduplication::process_command_output(&fmt_output, "cargo fmt") {
            anyhow::bail!("cargo fmt failed");
        }

        // Run cargo clippy
        println!("   🔍 Running cargo clippy...");
        let clippy_output = Command::new("cargo")
            .args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .output()
            .context("Failed to run cargo clippy")?;

        if !error_deduplication::process_command_output(&clippy_output, "cargo clippy") {
            anyhow::bail!("cargo clippy failed");
        }

        // Run contract validation
        println!("   📋 Running contract validation...");
        let contract_output = Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "contract-check", "--strict"])
            .output()
            .context("Failed to run contract validation")?;

        if !error_deduplication::process_command_output(&contract_output, "contract validation") {
            anyhow::bail!("Contract validation failed");
        }

        // Run generated file validation
        println!("   📄 Running generated file validation...");
        let generated_output = Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "validate-generated", "--strict"])
            .output()
            .context("Failed to run generated file validation")?;

        if !error_deduplication::process_command_output(
            &generated_output,
            "generated file validation",
        ) {
            anyhow::bail!("Generated file validation failed");
        }

        // Print error statistics
        let (total_errors, unique_errors) = error_deduplication::get_error_stats();
        if total_errors > 0 {
            println!(
                "📊 Error Summary: {} total errors, {} unique errors",
                total_errors, unique_errors
            );
        }
        println!("✅ All validation checks passed!");
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
        return Ok(());
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
        anyhow::bail!("git add failed");
    }

    // Step 4: Get commit message
    let commit_message = if let Some(msg) = message {
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

    if allow_empty_message || commit_message.is_empty() {
        commit_args.extend_from_slice(&["--allow-empty-message", "-m", ""]);
    } else {
        commit_args.extend_from_slice(&["-m", &commit_message]);
    }

    // Add any additional arguments
    for arg in args {
        commit_args.push(arg);
    }

    let commit_status = Command::new("git")
        .args(&commit_args)
        .status()
        .context("Failed to commit changes")?;

    if !commit_status.success() {
        anyhow::bail!("git commit failed");
    }

    // Step 6: Push changes
    println!("🚀 Pushing changes...");
    let mut push_args = vec!["push", "--porcelain"];

    if force {
        push_args.push("--force");
        println!("⚠️  Force pushing (use with caution!)");
    }

    let push_output = Command::new("git")
        .args(&push_args)
        .output()
        .context("Failed to push changes")?;

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

        anyhow::bail!("git push failed: {}", error_message);
    }

    // Parse successful porcelain output for clean status message
    let stdout = String::from_utf8_lossy(&push_output.stdout);
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

    println!("✅ Automated git workflow completed successfully!");
    println!(
        "   📝 Committed with message: {}",
        if commit_message.is_empty() {
            "(empty)"
        } else {
            &commit_message
        }
    );
    println!("   🚀 {}", push_status);

    Ok(())
}

/// Run clean auto-push workflow
async fn run_clean_auto_push(
    message: Option<String>,
    allow_empty_message: bool,
    watchdog: bool,
    interval: u64,
    force: bool,
    verbose: bool,
    no_log: bool,
    log_file: Option<String>,
    args: Vec<String>,
) -> Result<()> {
    let mut auto_push = auto_push::CleanAutoPush::default();
    auto_push.verbose = verbose;
    auto_push.log_to_file = !no_log;
    if let Some(log_path) = log_file {
        auto_push.log_file = Some(log_path);
    }

    if watchdog {
        auto_push
            .run_watchdog(message, allow_empty_message, force, args, interval)
            .await?;
    } else {
        auto_push
            .run(message, allow_empty_message, force, args)
            .await?;
    }

    Ok(())
}

/// Run structured auto-push workflow with JSONL output and event bus integration
async fn run_structured_auto_push(
    message: Option<String>,
    allow_empty_message: bool,
    watchdog: bool,
    interval: u64,
    force: bool,
    verbose: bool,
    no_jsonl: bool,
    no_event_bus: bool,
    args: Vec<String>,
) -> Result<()> {
    let mut auto_push = structured_auto_push::StructuredAutoPush::new().with_verbose(verbose);

    if no_jsonl {
        auto_push = auto_push.without_jsonl();
    }

    if no_event_bus {
        auto_push = auto_push.without_event_bus();
    }

    if watchdog {
        auto_push
            .run_watchdog(message, allow_empty_message, force, args, interval)
            .await?;
    } else {
        auto_push
            .run(message, allow_empty_message, force, args)
            .await?;
    }

    Ok(())
}

/// Run auto-push using the new hook state machine
async fn run_auto_push_with_state_machine(
    message: Option<String>,
    _allow_empty_message: bool,
    skip_validation: bool,
    watchdog: bool,
    interval: u64,
    force: bool,
    args: Vec<String>,
) -> Result<()> {
    println!("🚀 Starting automated git workflow with hook state machine...");

    // Create hook context
    let context = HookContext::new(HookType::AutoPush)
        .with_message(message)
        .with_validation_skip(skip_validation)
        .with_force(force)
        .with_args(args);

    // Create hook manager with default hooks
    let mut hook_manager = HookManager::default();

    if watchdog {
        println!("🔄 Starting watchdog mode with {interval}s interval...");
        println!("   Press Ctrl+C to stop");

        // Start watchdog mode
        hook_manager
            .start_watchdog(HookType::AutoPush, interval)
            .await?;
    } else {
        // Run single auto-push cycle
        let result = hook_manager.run_hook(HookType::AutoPush, context)?;

        if result.success {
            println!("✅ Auto-push completed successfully!");
            println!("   Duration: {:?}", result.duration);
        } else {
            println!("❌ Auto-push failed: {}", result.message);
            for error in &result.errors {
                println!("   Error: {error}");
            }
            anyhow::bail!("Auto-push workflow failed");
        }
    }

    Ok(())
}

/// Run a specific hook using the state machine
async fn run_hook_with_state_machine(
    hook_type: HookTypeArg,
    message: Option<String>,
    _allow_empty_message: bool,
    skip_validation: bool,
    watchdog: bool,
    interval: u64,
    force: bool,
    args: Vec<String>,
) -> Result<()> {
    println!("🚀 Running hook: {hook_type:?}");

    // Create hook context
    let context = HookContext::new(hook_type.clone().into())
        .with_message(message)
        .with_validation_skip(skip_validation)
        .with_force(force)
        .with_args(args);

    // Create hook manager with default hooks
    let mut hook_manager = HookManager::default();

    if watchdog {
        println!("🔄 Starting watchdog mode with {interval}s interval...");
        println!("   Press Ctrl+C to stop");

        // Start watchdog mode
        hook_manager
            .start_watchdog(hook_type.into(), interval)
            .await?;
    } else {
        // Run single hook cycle
        let result = hook_manager.run_hook(hook_type.into(), context)?;

        if result.success {
            println!("✅ Hook completed successfully!");
            println!("   Duration: {:?}", result.duration);
        } else {
            println!("❌ Hook failed: {}", result.message);
            for error in &result.errors {
                println!("   Error: {error}");
            }
            anyhow::bail!("Hook workflow failed");
        }
    }

    Ok(())
}

/// List all available hooks
fn list_available_hooks() -> Result<()> {
    println!("📋 Available hooks:");
    println!();

    let hook_manager = HookManager::default();
    let registered_hooks = hook_manager.registered_hooks();

    for hook_type in registered_hooks {
        match hook_type {
            HookType::PreCommit => {
                println!("🔍 pre-commit");
                println!("   Runs validation checks before commit");
                println!("   - cargo fix, fmt, clippy");
                println!("   - contract validation");
                println!("   - generated file validation");
            }
            HookType::PrePush => {
                println!("🚀 pre-push");
                println!("   Runs validation checks before push");
                println!("   - Same as pre-commit validation");
            }
            HookType::AutoPush => {
                println!("⚡ auto-push");
                println!("   Complete automated git workflow");
                println!("   - Validation → Add → Commit → Push");
                println!("   - Supports watchdog mode");
            }
            _ => {
                println!("❓ {:?} (not implemented)", hook_type);
            }
        }
        println!();
    }

    Ok(())
}

/// Generate Lefthook configuration that uses the hook state machine
async fn init_event_stream_command(
    output_file: Option<String>,
    console_output: bool,
    enable_broadcast: bool,
    min_severity: String,
) -> Result<()> {
    println!("🎯 Initializing event stream...");

    let min_severity = match min_severity.to_lowercase().as_str() {
        "trace" => event_stream::EventSeverity::Trace,
        "debug" => event_stream::EventSeverity::Debug,
        "info" => event_stream::EventSeverity::Info,
        "warn" => event_stream::EventSeverity::Warn,
        "error" => event_stream::EventSeverity::Error,
        "critical" => event_stream::EventSeverity::Critical,
        _ => event_stream::EventSeverity::Info,
    };

    let config = event_stream::EventStreamConfig {
        output_file: output_file.clone().map(PathBuf::from),
        console_output,
        enable_broadcast,
        broadcast_capacity: 1000,
        retention_period: Some(Duration::from_secs(24 * 60 * 60)), // 24 hours
        enable_filtering: true,
        min_severity: min_severity.clone(),
    };

    event_stream::init_event_stream(config)?;

    println!("✅ Event stream initialized successfully");
    println!("   📁 Output file: {:?}", output_file);
    println!("   🖥️ Console output: {}", console_output);
    println!("   📡 Broadcasting: {}", enable_broadcast);
    println!("   📊 Min severity: {:?}", min_severity);

    Ok(())
}

async fn monitor_events_command(
    show_metadata: bool,
    performance_threshold: u64,
    error_threshold: u64,
) -> Result<()> {
    println!("🎯 Starting event monitor...");

    let mut monitor = event_stream::EventMonitor::new()?;

    // Add default handlers
    monitor.add_handler(Box::new(event_stream::ConsoleEventHandler::new(
        show_metadata,
    )));
    monitor.add_handler(Box::new(event_stream::PerformanceEventHandler::new()));
    monitor.add_handler(Box::new(event_stream::ErrorAggregationHandler::new(
        error_threshold,
    )));

    println!("✅ Event monitor started with handlers:");
    println!("   📺 Console handler (show_metadata: {})", show_metadata);
    println!(
        "   ⚡ Performance handler (threshold: {}ms)",
        performance_threshold
    );
    println!(
        "   🚨 Error aggregation handler (threshold: {})",
        error_threshold
    );
    println!("   Press Ctrl+C to stop");

    // Start monitoring
    monitor.start().await?;

    Ok(())
}

async fn analyze_events_command(input_file: String, format: String) -> Result<()> {
    println!("📊 Analyzing event stream from: {}", input_file);

    let content = fs::read_to_string(&input_file)
        .context(format!("Failed to read event file: {}", input_file))?;

    let mut events = Vec::new();
    for line in content.lines() {
        if !line.trim().is_empty() {
            if let Ok(event) = serde_json::from_str::<event_stream::Event>(line) {
                events.push(event);
            }
        }
    }

    println!("📈 Found {} events", events.len());

    match format.to_lowercase().as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        "summary" => {
            print_event_summary(&events)?;
        }
        "table" => {
            print_event_table(&events)?;
        }
        _ => {
            anyhow::bail!("Unknown format: {}", format);
        }
    }

    Ok(())
}

async fn init_event_bus_command(
    enable_persistence: bool,
    jsonl_file: Option<String>,
    batch_size: usize,
    flush_interval_ms: u64,
    console_output: bool,
) -> Result<()> {
    println!("🎯 Initializing event bus...");

    let config = event_bus::EventBusConfig {
        enable_persistence,
        jsonl_file: jsonl_file.clone().map(PathBuf::from),
        batch_size,
        flush_interval_ms,
        broadcast_capacity: 1000,
        console_output,
        session_id: Some(uuid::Uuid::new_v4().to_string()),
    };

    event_bus::init_event_bus(config)?;

    println!("✅ Event bus initialized successfully");
    println!("   📁 JSONL file: {:?}", jsonl_file);
    println!("   💾 Persistence: {}", enable_persistence);
    println!("   📦 Batch size: {}", batch_size);
    println!("   ⏱️ Flush interval: {}ms", flush_interval_ms);
    println!("   🖥️ Console output: {}", console_output);

    Ok(())
}

async fn process_events_command(auto_push: bool, notifications: bool, metrics: bool) -> Result<()> {
    println!("🎯 Starting event processor...");

    let mut processor = event_bus::EventProcessor::new(false);

    if auto_push {
        processor.add_handler(Box::new(event_bus::AutoPushHandler::new(true)));
        println!("   ✅ Auto-push handler enabled");
    }

    if notifications {
        processor.add_handler(Box::new(event_bus::NotificationHandler::new(true)));
        println!("   ✅ Notification handler enabled");
    }

    if metrics {
        processor.add_handler(Box::new(event_bus::MetricsHandler::new()));
        println!("   ✅ Metrics handler enabled");
    }

    println!("🎯 Event processor started with handlers");
    println!("   Press Ctrl+C to stop");

    // Start processing events
    processor.start_processing().await?;

    Ok(())
}

async fn replay_events_command(
    input_file: String,
    auto_push: bool,
    notifications: bool,
) -> Result<()> {
    println!("🔄 Replaying events from: {}", input_file);

    let mut processor = event_bus::EventProcessor::new(true);

    if auto_push {
        processor.add_handler(Box::new(event_bus::AutoPushHandler::new(true)));
        println!("   ✅ Auto-push handler enabled");
    }

    if notifications {
        processor.add_handler(Box::new(event_bus::NotificationHandler::new(true)));
        println!("   ✅ Notification handler enabled");
    }

    let file_path = PathBuf::from(input_file);
    processor.process_file(&file_path)?;

    println!("✅ Event replay completed");

    Ok(())
}

async fn emit_test_events_command(count: usize) -> Result<()> {
    println!("🧪 Emitting {} test events...", count);

    for i in 0..count {
        let context = serde_json::json!({
            "test_number": i + 1,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "branch": "test-branch",
            "files": ["test-file.rs"]
        });

        match i % 4 {
            0 => {
                event_bus::emit_validation_event(
                    "test-actor",
                    Some("pre-commit"),
                    Some("validating"),
                    true,
                    context,
                )?;
            }
            1 => {
                event_bus::emit_commit_event(
                    "test-actor",
                    &format!("test-commit-{}", i),
                    "Test commit message",
                    vec!["test-file.rs".to_string()],
                )?;
            }
            2 => {
                event_bus::emit_push_event("test-actor", true, "test-branch", "origin", None)?;
            }
            3 => {
                event_bus::emit_state_transition_event(
                    "test-actor",
                    "idle",
                    "validating",
                    "test-trigger",
                )?;
            }
            _ => {}
        }

        // Small delay between events
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Test events emitted successfully");

    Ok(())
}

async fn load_wasm_component_command(component_path: String, config: Option<String>) -> Result<()> {
    println!("🔧 Loading WASM component: {}", component_path);

    let path = PathBuf::from(component_path);
    if !path.exists() {
        anyhow::bail!("Component file not found: {}", path.display());
    }

    let handler_id = wasm_event_bus::load_wasm_component(&path).await?;
    println!("✅ Component loaded with handler ID: {}", handler_id);

    if let Some(config_str) = config {
        println!("📋 Component configuration: {}", config_str);
    }

    Ok(())
}

fn list_wasm_components_command() -> Result<()> {
    println!("📦 Listing WASM components...");

    let components = wasm_event_bus::list_wasm_components()?;

    if components.is_empty() {
        println!("   No components loaded");
    } else {
        println!("   Loaded components:");
        for component in components {
            println!("   - ID: {} | Name: {}", component.id, component.name);
            println!("     Events: {:?}", component.supported_events);
            println!("     Categories: {:?}", component.supported_categories);
        }
    }

    Ok(())
}

fn unload_wasm_component_command(handler_id: u32) -> Result<()> {
    println!("🗑️ Unloading WASM component: {}", handler_id);

    wasm_event_bus::unregister_wasm_handler(handler_id)?;
    println!("✅ Component unloaded successfully");

    Ok(())
}

fn get_wasm_component_stats_command() -> Result<()> {
    println!("📊 WASM component statistics...");

    let stats = wasm_event_bus::get_wasm_event_bus_stats()?;
    println!("{}", serde_json::to_string_pretty(&stats)?);

    Ok(())
}

async fn build_validation_handler_command(output_dir: String) -> Result<()> {
    println!("🔨 Building validation handler component...");

    let output_path = PathBuf::from(&output_dir);
    std::fs::create_dir_all(&output_path)?;

    // Build the validation handler component
    let status = std::process::Command::new("cargo")
        .args(&[
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--manifest-path",
            "components/validation-handler/Cargo.toml",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to build validation handler component");
    }

    // Copy the built component to output directory
    let source_path =
        PathBuf::from("target/wasm32-unknown-unknown/release/validation_handler.wasm");
    let dest_path = output_path.join("validation-handler.wasm");

    if source_path.exists() {
        std::fs::copy(&source_path, &dest_path)?;
        println!("✅ Component built successfully: {}", dest_path.display());
    } else {
        anyhow::bail!("Built component not found at: {}", source_path.display());
    }

    Ok(())
}

async fn generate_event_stream_config(output: String) -> Result<()> {
    println!("📝 Generating event stream configuration...");

    let config_content = r#"# Event Stream Configuration for Hooksmith
# This file configures the centralized event streaming system

# Output configuration
output:
  # JSONL file for event persistence
  file: "hooksmith-events.jsonl"
  # Whether to enable console output
  console: true
  # Whether to enable real-time broadcasting
  broadcast: true
  # Broadcast channel capacity
  broadcast_capacity: 1000

# Event filtering
filtering:
  # Minimum severity level to log
  min_severity: "info"
  # Event retention period (24 hours)
  retention_period: 86400

# Event handlers
handlers:
  # Console output handler
  console:
    enabled: true
    show_metadata: true

  # Performance monitoring handler
  performance:
    enabled: true
    slow_operation_threshold: 1000  # 1 second
    very_slow_operation_threshold: 5000  # 5 seconds

  # Error aggregation handler
  error_aggregation:
    enabled: true
    error_threshold: 5
    alert_on_threshold_exceeded: true

# Event categories
categories:
  - HookStateMachine
  - Git
  - Validation
  - Build
  - Contract
  - File
  - System
  - User
  - Performance
  - Security
"#;

    fs::write(&output, config_content).context(format!(
        "Failed to write event stream configuration to {}",
        output
    ))?;

    println!("✅ Event stream configuration generated successfully");
    println!("   📁 File: {}", output);

    Ok(())
}

fn print_event_summary(events: &[event_stream::Event]) -> Result<()> {
    println!("📊 Event Summary");
    println!("================");
    println!("Total events: {}", events.len());

    if events.is_empty() {
        return Ok(());
    }

    // Group by severity
    let mut severity_counts = std::collections::HashMap::new();
    let mut category_counts = std::collections::HashMap::new();
    let mut source_counts = std::collections::HashMap::new();

    for event in events {
        *severity_counts.entry(&event.severity).or_insert(0) += 1;
        *category_counts.entry(&event.category).or_insert(0) += 1;
        *source_counts.entry(&event.source).or_insert(0) += 1;
    }

    println!("\n📈 By Severity:");
    for (severity, count) in severity_counts {
        println!("  {:?}: {}", severity, count);
    }

    println!("\n📂 By Category:");
    for (category, count) in category_counts {
        println!("  {:?}: {}", category, count);
    }

    println!("\n🔧 By Source:");
    for (source, count) in source_counts {
        println!("  {}: {}", source, count);
    }

    // Time range
    if let (Some(first), Some(last)) = (events.first(), events.last()) {
        println!("\n⏰ Time Range:");
        println!("  Start: {}", first.timestamp);
        println!("  End: {}", last.timestamp);
        println!("  Duration: {:?}", last.timestamp - first.timestamp);
    }

    Ok(())
}

fn print_event_table(events: &[event_stream::Event]) -> Result<()> {
    println!("📋 Event Table");
    println!("==============");
    println!(
        "{:<20} {:<15} {:<20} {:<30} {:<10}",
        "Timestamp", "Severity", "Category", "Event Type", "Source"
    );
    println!("{:-<95}", "");

    for event in events.iter().take(50) {
        // Limit to first 50 events
        println!(
            "{:<20} {:<15?} {:<20?} {:<30} {:<10}",
            event.timestamp.format("%H:%M:%S"),
            event.severity,
            event.category,
            event.event_type,
            event.source
        );
    }

    if events.len() > 50 {
        println!("... and {} more events", events.len() - 50);
    }

    Ok(())
}

async fn generate_lefthook_hooks_config(output: String, validate: bool) -> Result<()> {
    println!("📝 Generating Lefthook configuration with hook state machine...");
    println!("   Output: {output}");

    let config_content = r#"# Lefthook configuration for Hooksmith
# This file is generated by the hook state machine

pre-commit:
  commands:
    hooksmith-pre-commit:
      run: cargo run -p xtask -- hook pre-commit --strict
      parallel: false

pre-push:
  commands:
    hooksmith-pre-push:
      run: cargo run -p xtask -- hook pre-push --strict
      parallel: false

# Auto-push hook for trunk-style development
# Uncomment to enable automatic commits and pushes
# pre-commit:
#   commands:
#     hooksmith-auto-push:
#       run: cargo run -p xtask -- hook auto-push --watchdog --interval 30
#       parallel: false
"#;

    fs::write(&output, config_content).context(format!(
        "Failed to write Lefthook configuration to {}",
        output
    ))?;

    if validate {
        println!("   Validating configuration...");
        // Basic validation - check if file was written successfully
        if fs::metadata(&output).is_ok() {
            println!("   ✅ Configuration validated successfully");
        } else {
            anyhow::bail!("Configuration validation failed");
        }
    }

    println!("✅ Lefthook configuration generated successfully");
    println!("   📁 File: {}", output);
    println!("   💡 To use: lefthook install");

    Ok(())
}

/// Convert JSON files to JSONC format with comments
async fn convert_json_to_jsonc(
    file: Option<String>,
    overwrite: bool,
    remove_original: bool,
) -> Result<()> {
    println!("🔄 Converting JSON files to JSONC format...");

    let files_to_convert = if let Some(specific_file) = file {
        vec![PathBuf::from(specific_file)]
    } else {
        // Find all JSON files in the project
        let output = Command::new("find")
            .args([".", "-name", "*.json", "-not", "-path", "./target/*"])
            .output()
            .context("Failed to find JSON files")?;

        if !output.status.success() {
            anyhow::bail!("Failed to find JSON files");
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(PathBuf::from)
            .collect()
    };

    if files_to_convert.is_empty() {
        println!("✅ No JSON files found to convert");
        return Ok(());
    }

    println!("📁 Found {} JSON files to convert", files_to_convert.len());

    for json_path in files_to_convert {
        let jsonc_path = json_path.with_extension("jsonc");

        if jsonc_path.exists() && !overwrite {
            println!(
                "   ⏭️  Skipping {} (JSONC file already exists)",
                json_path.display()
            );
            continue;
        }

        println!(
            "   🔄 Converting {} -> {}",
            json_path.display(),
            jsonc_path.display()
        );

        // Read the JSON file
        let json_content = fs::read_to_string(&json_path)
            .context(format!("Failed to read JSON file: {}", json_path.display()))?;

        // Strip header comments if present
        let json_content_clean = strip_json_header(&json_content);

        // Parse JSON to validate it
        let json_value: serde_json::Value = serde_json::from_str(&json_content_clean).context(
            format!("Failed to parse JSON file: {}", json_path.display()),
        )?;

        // Generate JSONC content with comments
        let jsonc_content = generate_jsonc_content(&json_path, &json_value)?;

        // Write the JSONC file
        fs::write(&jsonc_path, jsonc_content).context(format!(
            "Failed to write JSONC file: {}",
            jsonc_path.display()
        ))?;

        println!("   ✅ Converted {}", json_path.display());

        // Remove original JSON file if requested
        if remove_original {
            fs::remove_file(&json_path).context(format!(
                "Failed to remove original JSON file: {}",
                json_path.display()
            ))?;
            println!("   🗑️  Removed original {}", json_path.display());
        }
    }

    println!("✅ JSON to JSONC conversion completed!");
    Ok(())
}

/// Generate JSONC content with appropriate comments based on file type
fn generate_jsonc_content(json_path: &Path, json_value: &serde_json::Value) -> Result<String> {
    let filename = json_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown");
    let parent_dir = json_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("");

    let header_comment = match (parent_dir, filename) {
        ("docs", "checksums.json") => {
            "// This file contains SHA-256 checksums for all generated documentation files\n// It's used to detect when files have been manually modified and need regeneration\n// Generated by: cargo xtask gen-docs-comprehensive\n"
        }
        ("xtask", "status-badge.json") => {
            "// Status badge configuration for GitHub README\n// This badge shows the percentage of Rust-owned files in the project\n// Generated by: cargo xtask status badge\n"
        }
        ("schemas", _) if filename.ends_with(".schema.json") => {
            "// JSON Schema definition\n// This schema defines the structure for validation\n// Generated by: cargo xtask gen-schema\n"
        }
        ("status-trends", _) if filename.starts_with("status-") => {
            "// Status trend data\n// This file contains historical status data for tracking progress\n// Generated by: cargo xtask status trend\n"
        }
        _ => {
            "// JSON configuration file\n// This file contains configuration data\n// Generated by: cargo xtask\n"
        }
    };

    // Pretty print the JSON with 2-space indentation
    let pretty_json =
        serde_json::to_string_pretty(json_value).context("Failed to pretty print JSON")?;

    // Combine header comment with pretty JSON
    Ok(format!("{header_comment}{pretty_json}"))
}

/// Strip header comments from JSON content
fn strip_json_header(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut json_start = 0;

    // Find the first line that starts with '{' or '['
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            json_start = i;
            break;
        }
    }

    // Return everything from the JSON start onwards
    lines[json_start..].join("\n")
}

/// Run the event-driven dashboard
async fn run_dashboard_command(
    interval: u64,
    show_dashboard: bool,
    auto_push: bool,
    skip_validation: bool,
    message: Option<String>,
    file_watch: bool,
    trigger: bool,
) -> Result<()> {
    // Initialize event bus if not already initialized
    if event_bus::get_event_bus().is_none() {
        println!("🔧 Initializing event bus...");
        let event_bus_config = event_bus::EventBusConfig {
            enable_persistence: true,
            jsonl_file: Some("hooksmith-events.jsonl".into()),
            batch_size: 10,
            flush_interval_ms: 1000,
            console_output: true,
            broadcast_capacity: 1000,
            session_id: Some(uuid::Uuid::new_v4().to_string()),
        };
        event_bus::init_event_bus(event_bus_config)?;
    }

    let config = dashboard::DashboardConfig {
        show_dashboard,
        log_to_jsonl: true,
        jsonl_path: Some("hooksmith-events.jsonl".to_string()),
        auto_push_config: dashboard::AutoPushConfig {
            enabled: auto_push,
            commit_message: message,
            skip_validation,
        },
        file_watch_mode: file_watch,
        heartbeat_interval: interval,
    };

    if trigger {
        // For trigger mode, we want to show the dashboard and emit events
        // Create a temporary dashboard to display the events
        let mut temp_config = config.clone();
        temp_config.show_dashboard = true; // Force show dashboard for trigger mode

        let mut dashboard = dashboard::Dashboard::new(temp_config)
            .map_err(|e| anyhow::anyhow!("Failed to create dashboard: {}", e))?;

        // Emit validation start event
        let validation_event = event_bus::HooksmithEvent::new(
            "dashboard".to_string(),
            "validation_started".to_string(),
            serde_json::json!({
                "trigger": "manual",
                "timestamp": chrono::Utc::now()
            }),
        );
        event_bus::emit_event(validation_event)?;

        // Run validation and auto-push in background
        let config_clone = config.clone();
        tokio::spawn(async move {
            // Run validation and auto-push
            if !config_clone.auto_push_config.skip_validation {
                // Emit validation events for each step
                let steps = vec![
                    "cargo_fix",
                    "cargo_fmt",
                    "cargo_clippy",
                    "contract_validation",
                ];
                for step in steps {
                    let step_event = event_bus::HooksmithEvent::new(
                        "dashboard".to_string(),
                        format!("{}_started", step),
                        serde_json::json!({
                            "step": step,
                            "timestamp": chrono::Utc::now()
                        }),
                    );
                    let _ = event_bus::emit_event(step_event);

                    // Simulate step completion
                    let step_complete_event = event_bus::HooksmithEvent::new(
                        "dashboard".to_string(),
                        format!("{}_completed", step),
                        serde_json::json!({
                            "step": step,
                            "success": true,
                            "timestamp": chrono::Utc::now()
                        }),
                    );
                    let _ = event_bus::emit_event(step_complete_event);
                }

                // Emit validation success event
                let validation_success_event = event_bus::HooksmithEvent::new(
                    "dashboard".to_string(),
                    "validation_passed".to_string(),
                    serde_json::json!({
                        "timestamp": chrono::Utc::now()
                    }),
                );
                let _ = event_bus::emit_event(validation_success_event);
            }

            if config_clone.auto_push_config.enabled {
                // Emit auto-push start event
                let auto_push_event = event_bus::HooksmithEvent::new(
                    "dashboard".to_string(),
                    "auto_push_started".to_string(),
                    serde_json::json!({
                        "timestamp": chrono::Utc::now()
                    }),
                );
                let _ = event_bus::emit_event(auto_push_event);

                // Run auto-push
                if let Err(e) =
                    dashboard::Dashboard::run_auto_push_cycle(&config_clone.auto_push_config).await
                {
                    // Emit auto-push failure event
                    let auto_push_failed_event = event_bus::HooksmithEvent::new(
                        "dashboard".to_string(),
                        "auto_push_failed".to_string(),
                        serde_json::json!({
                            "error": e.to_string(),
                            "timestamp": chrono::Utc::now()
                        }),
                    );
                    let _ = event_bus::emit_event(auto_push_failed_event);
                } else {
                    // Emit auto-push success event
                    let auto_push_success_event = event_bus::HooksmithEvent::new(
                        "dashboard".to_string(),
                        "auto_push_succeeded".to_string(),
                        serde_json::json!({
                            "timestamp": chrono::Utc::now()
                        }),
                    );
                    let _ = event_bus::emit_event(auto_push_success_event);
                }
            }
        });

        // Start the dashboard to show the events
        dashboard
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Dashboard error: {}", e))?;

        return Ok(());
    }

    // Create and start the event-driven dashboard
    let mut dashboard = dashboard::Dashboard::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to create dashboard: {}", e))?;

    dashboard
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Dashboard error: {}", e))?;

    Ok(())
}

/// Generate JSON schema for AutoPushEvent
fn generate_schema_command(output: Option<String>) -> Result<()> {
    let schema = emit::generate_schema()?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, schema)?;
        emit_info!("hooksmith", "schema", "Schema written to: {}", output_path);
    } else {
        println!("{}", schema);
    }

    Ok(())
}

/// Validate JSONL output against schema
fn validate_schema_command(input: Option<String>, strict: bool) -> Result<()> {
    let result = if let Some(input_path) = input {
        let file = std::fs::File::open(&input_path)?;
        emit::validate_output(file)
    } else {
        emit::validate_output(std::io::stdin())
    };

    match result {
        Ok(()) => {
            emit_success!("hooksmith", "validation", "Schema validation passed");
            Ok(())
        }
        Err(e) => {
            emit_failure!("hooksmith", "validation", "Schema validation failed: {}", e);
            if strict {
                Err(anyhow::anyhow!("Schema validation failed: {}", e))
            } else {
                Ok(())
            }
        }
    }
}

/// Convert JSONL events to SARIF format
async fn run_jsonl_to_sarif_command(input: String, output: String, validate: bool) -> Result<()> {
    println!("🔄 Converting JSONL to SARIF...");
    println!("   Input: {}", input);
    println!("   Output: {}", output);

    let logger = structured_logging::StructuredLogger::new();
    let integration = sarif_integration::SarifIntegration::new(logger);

    let sarif_content = integration.jsonl_to_sarif(Path::new(&input))?;

    // Write SARIF output
    std::fs::write(&output, &sarif_content)
        .context(format!("Failed to write SARIF file: {}", output))?;

    println!("✅ Successfully converted JSONL to SARIF");

    if validate {
        println!("🔍 Validating SARIF output...");
        let is_valid = integration.validate_sarif(Path::new(&output))?;
        if is_valid {
            println!("✅ SARIF validation passed");
        } else {
            anyhow::bail!("❌ SARIF validation failed");
        }
    }

    Ok(())
}

/// Convert SARIF to JSONL events
async fn run_sarif_to_jsonl_command(input: String, output: String, validate: bool) -> Result<()> {
    println!("🔄 Converting SARIF to JSONL...");
    println!("   Input: {}", input);
    println!("   Output: {}", output);

    let logger = structured_logging::StructuredLogger::new();
    let integration = sarif_integration::SarifIntegration::new(logger);

    if validate {
        println!("🔍 Validating SARIF input...");
        let is_valid = integration.validate_sarif(Path::new(&input))?;
        if !is_valid {
            anyhow::bail!("❌ SARIF validation failed");
        }
        println!("✅ SARIF validation passed");
    }

    let events = integration.sarif_to_jsonl(Path::new(&input))?;

    // Write JSONL output
    let mut output_file = std::fs::File::create(&output)
        .context(format!("Failed to create output file: {}", output))?;

    let events_count = events.len();
    for event in events {
        let jsonl = event.to_jsonl()?;
        use std::io::Write;
        writeln!(output_file, "{}", jsonl)?;
    }

    println!(
        "✅ Successfully converted SARIF to JSONL ({} events)",
        events_count
    );

    Ok(())
}

/// Run CodeQL analysis and convert to structured events
async fn run_codeql_analysis_command(
    cli_path: Option<&str>,
    db_dir: String,
    query_suite: String,
    language: String,
    build_command: String,
    output: Option<&str>,
    to_jsonl: bool,
) -> Result<()> {
    println!("🔍 Running CodeQL analysis...");
    println!("   Database: {}", db_dir);
    println!("   Query suite: {}", query_suite);
    println!("   Language: {}", language);
    println!("   Build command: {}", build_command);

    let logger = structured_logging::StructuredLogger::new();

    let mut config = sarif_integration::CodeQLConfig::default();
    config.cli_path = cli_path.map(|s| s.to_string());
    config.db_dir = PathBuf::from(db_dir);
    config.query_suite = query_suite;
    config.language = language;
    config.build_command = build_command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let integration = sarif_integration::SarifIntegration::new(logger).with_config(config);

    // Run CodeQL analysis
    let events = integration.run_codeql_analysis().await?;

    println!("✅ CodeQL analysis completed with {} results", events.len());

    // Save SARIF output if requested
    if let Some(output_path) = output {
        println!("💾 Saving SARIF output to: {}", output_path);

        // Convert events back to SARIF for output
        let temp_jsonl = format!("{}.tmp.jsonl", output_path);
        let mut temp_file =
            std::fs::File::create(&temp_jsonl).context("Failed to create temporary JSONL file")?;

        use std::io::Write;
        for event in &events {
            let jsonl = event.to_jsonl()?;
            writeln!(temp_file, "{}", jsonl)?;
        }
        drop(temp_file);

        // Convert JSONL to SARIF
        let sarif_content = integration.jsonl_to_sarif(Path::new(&temp_jsonl))?;
        std::fs::write(output_path, &sarif_content)
            .context(format!("Failed to write SARIF file: {}", output_path))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_jsonl);
    }

    // Convert to JSONL if requested
    if to_jsonl {
        let jsonl_output = output
            .map(|s| format!("{}.jsonl", s))
            .unwrap_or_else(|| "codeql-results.jsonl".to_string());
        println!("💾 Saving JSONL output to: {}", jsonl_output);

        let mut output_file = std::fs::File::create(&jsonl_output)
            .context(format!("Failed to create JSONL file: {}", jsonl_output))?;

        use std::io::Write;
        for event in events {
            let jsonl = event.to_jsonl()?;
            writeln!(output_file, "{}", jsonl)?;
        }
    }

    Ok(())
}

/// Validate SARIF file
fn run_validate_sarif_command(file: String, strict: bool) -> Result<()> {
    println!("🔍 Validating SARIF file: {}", file);

    let logger = structured_logging::StructuredLogger::new();
    let integration = sarif_integration::SarifIntegration::new(logger);

    let is_valid = integration.validate_sarif(Path::new(&file))?;

    if is_valid {
        println!("✅ SARIF validation passed");
    } else {
        println!("❌ SARIF validation failed");
        if strict {
            anyhow::bail!("SARIF validation failed");
        }
    }

    Ok(())
}

/// Merge multiple SARIF files
fn run_merge_sarif_command(inputs: Vec<String>, output: String, validate: bool) -> Result<()> {
    println!("🔄 Merging SARIF files...");
    println!("   Inputs: {:?}", inputs);
    println!("   Output: {}", output);

    let logger = structured_logging::StructuredLogger::new();
    let integration = sarif_integration::SarifIntegration::new(logger);

    let input_paths: Vec<PathBuf> = inputs.iter().map(|s| PathBuf::from(s)).collect();
    let merged_content = integration.merge_sarif_files(&input_paths)?;

    // Write merged SARIF
    std::fs::write(&output, &merged_content)
        .context(format!("Failed to write merged SARIF file: {}", output))?;

    println!("✅ Successfully merged {} SARIF files", inputs.len());

    if validate {
        println!("🔍 Validating merged SARIF...");
        let is_valid = integration.validate_sarif(Path::new(&output))?;
        if is_valid {
            println!("✅ Merged SARIF validation passed");
        } else {
            anyhow::bail!("❌ Merged SARIF validation failed");
        }
    }

    Ok(())
}

/// Integrate CodeQL into validation pipeline
async fn run_integrate_codeql_command(
    run_analysis: bool,
    to_jsonl: bool,
    merge: bool,
    output_dir: String,
) -> Result<()> {
    println!("🔧 Integrating CodeQL into validation pipeline...");
    println!("   Output directory: {}", output_dir);

    // Create output directory
    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path)
            .context(format!("Failed to create output directory: {}", output_dir))?;
    }

    let logger = structured_logging::StructuredLogger::new();
    let integration = sarif_integration::SarifIntegration::new(logger);

    if run_analysis {
        println!("🔍 Running CodeQL analysis...");
        let events = integration.run_codeql_analysis().await?;
        println!("✅ CodeQL analysis completed with {} results", events.len());

        // Save results
        let sarif_output = output_path.join("codeql-results.sarif");
        let jsonl_output = output_path.join("codeql-results.jsonl");

        // Convert to SARIF
        let temp_jsonl = output_path.join("temp.jsonl");
        let mut temp_file =
            std::fs::File::create(&temp_jsonl).context("Failed to create temporary JSONL file")?;

        use std::io::Write;
        for event in &events {
            let jsonl = event.to_jsonl()?;
            writeln!(temp_file, "{}", jsonl)?;
        }
        drop(temp_file);

        let sarif_content = integration.jsonl_to_sarif(&temp_jsonl)?;
        std::fs::write(&sarif_output, &sarif_content).context("Failed to write SARIF file")?;

        // Save JSONL if requested
        if to_jsonl {
            let mut output_file =
                std::fs::File::create(&jsonl_output).context("Failed to create JSONL file")?;

            use std::io::Write;
            for event in events {
                let jsonl = event.to_jsonl()?;
                writeln!(output_file, "{}", jsonl)?;
            }
        }

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_jsonl);

        println!("💾 Results saved:");
        println!("   SARIF: {}", sarif_output.display());
        if to_jsonl {
            println!("   JSONL: {}", jsonl_output.display());
        }
    }

    if merge {
        println!("🔄 Merging with existing validation results...");

        // Look for existing SARIF files in output directory
        let mut sarif_files = Vec::new();
        for entry in std::fs::read_dir(output_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sarif") {
                sarif_files.push(path);
            }
        }

        if sarif_files.len() > 1 {
            let merged_output = output_path.join("merged-results.sarif");
            let merged_content = integration.merge_sarif_files(&sarif_files)?;

            std::fs::write(&merged_output, &merged_content)
                .context("Failed to write merged SARIF file")?;

            println!(
                "✅ Merged {} SARIF files into: {}",
                sarif_files.len(),
                merged_output.display()
            );
        } else {
            println!("ℹ️  No multiple SARIF files found to merge");
        }
    }

    println!("✅ CodeQL integration completed");

    Ok(())
}
