//! # Hooksmith CLI
//!
//! A CLI tool for building Rust binaries into Lefthook hooks with WASM components.
//!
//! ## Overview
//!
//! Hooksmith bridges the gap between:
//! - **Lefthook**: Git hooks management system
//! - **WASM Components**: WebAssembly modules for cross-language functionality
//! - **WIT**: WebAssembly Interface Types for stable interfaces
//! - **Rust Binaries**: High-performance hook implementations
//!
//! ## Architecture
//!
//! ```
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Lefthook      │    │   Hooksmith     │    │   WASM          │
//! │   (Git Hooks)   │◄──►│   (CLI Tool)    │◄──►│   Components    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!                                │
//!                                ▼
//!                       ┌─────────────────┐
//!                       │   Rust          │
//!                       │   Binaries      │
//!                       └─────────────────┘
//! ```
//!
//! ## Usage Examples
//!
//! ```bash
//! # Build a hook binary
//! hooksmith build my-hook
//!
//! # Generate Lefthook configuration
//! hooksmith generate --output lefthook.yml
//!
//! # Build WASM component from WIT
//! hooksmith wasm build interface.wit
//!
//! # Run WASM component
//! hooksmith wasm run component.wasm --function validate
//! ```
//!
//! ## Commands
//!
//! - `test`: Test the CLI functionality
//! - `build`: Build Rust binaries for hooks
//! - `generate`: Generate Lefthook configuration
//! - `install`: Install hooks
//! - `list`: List available hooks
//! - `wasm`: WASM component management

use clap::{Parser, Subcommand};
use console::style;
use anyhow::Result;

// All functionality is implemented directly in main.rs

/// Main CLI application for Hooksmith
///
/// This CLI provides tools for building Rust binaries into Lefthook hooks
/// with WASM components. It serves as a bridge between Git hooks management
/// and WebAssembly-based functionality.
#[derive(Parser)]
#[command(name = "hooksmith")]
#[command(about = "Build Rust binaries into Lefthook hooks with WASM components")]
#[command(version = "0.1.0")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Test command to verify CLI functionality
    Test {
        /// Custom test message
        #[arg(long, default_value = "Hello from Hooksmith")]
        message: String,
    },
    /// Build Rust binaries for Git hooks
    ///
    /// This command compiles Rust code into binary executables that can be
    /// used as Lefthook hooks. The binaries are optimized for performance
    /// and can integrate with WASM components.
    Build {
        /// Name of the hook to build
        hook_name: String,
        /// Output directory for built binaries
        #[arg(long, default_value = "target/hooks")]
        output: String,
    },
    /// Generate Lefthook configuration
    ///
    /// Creates a lefthook.yml configuration file that integrates the built
    /// hooks with Git workflow. This enables automatic hook execution on
    /// Git events like pre-commit, pre-push, etc.
    Generate {
        /// Output file path for Lefthook configuration
        #[arg(long, default_value = "lefthook.yml")]
        output: String,
        /// Whether to validate against the official Lefthook schema
        #[arg(long, default_value = "true")]
        validate_schema: bool,
    },
    /// Generate comprehensive Lefthook configuration
    ///
    /// Creates a template lefthook.yml file with all available Git hooks
    /// for documentation or as a starting point.
    GenerateComprehensive {
        /// Output file path for Lefthook configuration
        #[arg(long, default_value = "lefthook-comprehensive.yml")]
        output: String,
        /// Whether to validate against the official Lefthook schema
        #[arg(long, default_value = "true")]
        validate_schema: bool,
    },
    /// Generate structured code and documentation
    ///
    /// Uses structured code generation with WIT schemas to create
    /// documentation, WIT interfaces, and other generated files.
    /// Replaces shell scripts and raw echo statements with type-safe generation.
    GenerateCode {
        /// Type of generation to perform
        #[arg(long, default_value = "all")]
        type_: String,
        /// Output directory for generated files
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
    /// Install hooks into Git repository
    ///
    /// Installs the built hooks into the current Git repository's hooks
    /// directory, making them available for Lefthook to execute.
    Install {
        /// Comma-separated list of hook names to install
        #[arg(long)]
        hooks: Option<String>,
    },
    /// List available hooks
    ///
    /// Displays all available hooks that can be built or installed.
    List,
    /// Validate Lefthook configuration
    ///
    /// Validates an existing lefthook.yml file against the official
    /// Lefthook JSON schema to ensure compliance.
    Validate {
        /// Path to the lefthook.yml file to validate
        #[arg(long, default_value = "lefthook.yml")]
        config_path: String,
    },
    /// WASM component management
    ///
    /// Commands for building, running, and managing WebAssembly components
    /// that can be integrated with the hooks.
    Wasm {
        #[command(subcommand)]
        wasm: WasmCommands,
    },
    /// Worktree management
    ///
    /// Commands for managing Git worktrees using WASM components that wrap
    /// existing tools like wtp, wt, and Treekanga.
    Worktree {
        #[command(subcommand)]
        worktree: WorktreeCommands,
    },
}

/// Worktree management commands
#[derive(Subcommand)]
enum WorktreeCommands {
    /// Create a new worktree
    ///
    /// Creates a new Git worktree using the best available tool (wtp, wt, treekanga, or git).
    /// The WASM component automatically detects and uses the most appropriate tool.
    Create {
        /// Branch name for the new worktree
        branch_name: String,
        /// Preferred tool to use (wtp, wt, treekanga, git)
        #[arg(long)]
        tool: Option<String>,
        /// Base directory for worktrees
        #[arg(long, default_value = "../worktrees")]
        base: String,
    },
    /// List all worktrees
    ///
    /// Lists all available worktrees using the configured tool.
    List {
        /// Preferred tool to use for listing
        #[arg(long)]
        tool: Option<String>,
    },
    /// Switch to a worktree
    ///
    /// Switches to the specified worktree using the configured tool.
    Switch {
        /// Name of the worktree to switch to
        worktree_name: String,
        /// Preferred tool to use for switching
        #[arg(long)]
        tool: Option<String>,
    },
    /// Remove a worktree
    ///
    /// Removes the specified worktree and optionally its branch.
    Remove {
        /// Name of the worktree to remove
        worktree_name: String,
        /// Also remove the branch
        #[arg(long)]
        with_branch: bool,
        /// Preferred tool to use for removal
        #[arg(long)]
        tool: Option<String>,
    },
    /// Show available tools
    ///
    /// Lists all available worktree management tools on the system.
    Tools,
}

/// WASM component management commands
#[derive(Subcommand)]
enum WasmCommands {
    /// Build WASM component from WIT interface
    ///
    /// Compiles WIT (WebAssembly Interface Types) definitions into WASM
    /// components that can be used by the hooks for cross-language
    /// functionality.
    Build {
        /// WIT interface file path
        wit_file: String,
        /// Output directory for WASM files
        #[arg(long, default_value = "target/hooks")]
        output: String,
    },
    /// Run WASM component
    ///
    /// Executes a WASM component with specified function and arguments.
    /// Useful for testing WASM components before integration.
    Run {
        /// WASM file to execute
        wasm_file: String,
        /// Function name to call within the WASM component
        #[arg(long)]
        function: String,
        /// Arguments to pass to the WASM function
        #[arg(long)]
        args: Vec<String>,
    },
    /// Generate bindings from WIT
    ///
    /// Creates language bindings from WIT interface definitions, enabling
    /// the use of WASM components in different programming languages.
    Bindings {
        /// WIT interface file path
        wit_file: String,
        /// Output directory for generated bindings
        #[arg(long, default_value = "target/bindings")]
        output: String,
    },
}

/// Main application entry point
///
/// Parses CLI arguments and executes the appropriate command based on
/// user input. All commands are currently placeholder implementations
/// that demonstrate the intended functionality.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Test { message } => {
            println!("{} {}", style("✅").green(), style(format!("Test successful: {}", message)).green());
        }
        Commands::Build { hook_name, output } => {
            println!("{} {} {}", style("🔨").blue(), style("Building hook:").blue(), style(hook_name).yellow());
            println!("{} {}", style("📁").blue(), style(format!("Output: {}", output)).blue());

            use hooksmith::modules::hook_builder::{HookBuilder, HookBuildConfig};
            use std::path::PathBuf;

            let config = HookBuildConfig {
                hook_name: hook_name.clone(),
                output_dir: PathBuf::from(output),
                ..Default::default()
            };

            let builder = HookBuilder::with_config(config);

            match builder.build_hook(None).await {
                Ok(result) => {
                    if result.success {
                        println!("{} {}", style("✅").green(), style("Hook built successfully").green());
                        if let Some(binary_path) = result.binary_path {
                            println!("{} {}", style("📦").blue(), style(format!("Binary: {:?}", binary_path)).blue());
                        }
                        println!("{} {}ms", style("⏱️").blue(), style(result.build_time_ms).blue());
                    } else {
                        eprintln!("{} {}", style("❌").red(), style("Hook build failed").red());
                        if let Some(error) = result.error {
                            eprintln!("{} {}", style("💥").red(), style(error).red());
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Build error: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::Generate { output, validate_schema } => {
            println!("{} {} {}", style("📝").blue(), style("Generating Lefthook config:").blue(), style(&output).yellow());
            if validate_schema {
                println!("{} {}", style("🔍").blue(), style("Schema validation enabled").blue());
            }
            
            // Import the lefthook module
            use hooksmith::modules::lefthook;
            
            // Generate configuration with schema validation
            match lefthook::generate_lefthook_config(
                std::path::Path::new(&output),
                "target/hooks",
                Some(vec!["components/worktree-runner".to_string()]),
                validate_schema,
            ).await {
                Ok(()) => println!("{} {}", style("✅").green(), style("Configuration generated successfully").green()),
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Failed to generate configuration: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::GenerateComprehensive { output, validate_schema } => {
            println!("{} {} {}", style("📋").blue(), style("Generating comprehensive Lefthook config:").blue(), style(&output).yellow());
            if validate_schema {
                println!("{} {}", style("🔍").blue(), style("Schema validation enabled").blue());
            }
            
            // Import the lefthook module
            use hooksmith::modules::lefthook;
            
            // Generate comprehensive configuration with all hooks
            match lefthook::generate_comprehensive_config(
                std::path::Path::new(&output),
                validate_schema,
            ).await {
                Ok(()) => println!("{} {}", style("✅").green(), style("Comprehensive configuration generated successfully").green()),
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Failed to generate comprehensive configuration: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::GenerateCode { type_, output_dir } => {
            println!("{} {} {}", style("🔧").blue(), style("Generating structured code:").blue(), style(&type_).yellow());
            println!("{} {} {}", style("📁").blue(), style("Output directory:").blue(), style(&output_dir).yellow());
            
            use hooksmith::modules::generator::{CodeGenerator, GeneratorConfig};
            use std::path::PathBuf;
            
            let config = GeneratorConfig {
                output_dir: PathBuf::from(output_dir),
                ..Default::default()
            };
            
            let generator = CodeGenerator::with_config(config);
            
            match type_.as_str() {
                "structure" => {
                    let result = generator.generate_structure_docs()?;
                    generator.write_files(&result)?;
                    println!("{} Generated {} structure files", style("✅").green(), result.files.len());
                }
                "wit" => {
                    let result = generator.generate_wit_interfaces()?;
                    generator.write_files(&result)?;
                    println!("{} Generated {} WIT interface files", style("✅").green(), result.files.len());
                }
                "docs" => {
                    let result = generator.generate_documentation()?;
                    generator.write_files(&result)?;
                    println!("{} Generated {} documentation files", style("✅").green(), result.files.len());
                }
                "all" => {
                    let structure_result = generator.generate_structure_docs()?;
                    generator.write_files(&structure_result)?;
                    
                    let wit_result = generator.generate_wit_interfaces()?;
                    generator.write_files(&wit_result)?;
                    
                    let docs_result = generator.generate_documentation()?;
                    generator.write_files(&docs_result)?;
                    
                    let total_files = structure_result.files.len() + wit_result.files.len() + docs_result.files.len();
                    println!("{} Generated {} total files", style("✅").green(), total_files);
                }
                _ => {
                    println!("{} Unknown generation type: {}", style("❌").red(), type_);
                    println!("Available types: structure, wit, docs, all");
                }
            }
        }
        Commands::Install { hooks } => {
            let hook_list = hooks.unwrap_or_else(|| "all".to_string());
            println!("{} {} {}", style("🔧").blue(), style("Installing hooks:").blue(), style(hook_list).yellow());

            use hooksmith::modules::hook_builder::install_hooks;
            use std::path::PathBuf;

            // Determine hooks directory
            let hooks_dir = PathBuf::from(".git/hooks");
            let hooks_source_dir = PathBuf::from("target/hooks");

            if !hooks_source_dir.exists() {
                eprintln!("{} {}", style("❌").red(), style("No hooks found in target/hooks directory").red());
                eprintln!("{} {}", style("💡").yellow(), style("Run 'hooksmith build <hook-name>' first").yellow());
                std::process::exit(1);
            }

            // Collect hook binaries
            let mut hook_binaries = Vec::new();
            if hook_list == "all" {
                // Install all hooks
                if let Ok(mut entries) = tokio::fs::read_dir(&hooks_source_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file() {
                            hook_binaries.push(path);
                        }
                    }
                }
            } else {
                // Install specific hooks
                for hook_name in hook_list.split(',') {
                    let hook_path = hooks_source_dir.join(hook_name.trim());
                    if hook_path.exists() {
                        hook_binaries.push(hook_path);
                    } else {
                        eprintln!("{} {} {}", style("⚠️").yellow(), style("Hook not found:").yellow(), style(hook_name.trim()).yellow());
                    }
                }
            }

            if hook_binaries.is_empty() {
                eprintln!("{} {}", style("❌").red(), style("No hooks to install").red());
                std::process::exit(1);
            }

            match install_hooks(&hooks_dir, &hook_binaries).await {
                Ok(()) => {
                    println!("{} {}", style("✅").green(), style("Hooks installed successfully").green());
                    println!("{} {} hooks installed", style("📦").blue(), style(hook_binaries.len()).blue());
                }
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Failed to install hooks: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::List => {
            println!("{} {}", style("📋").blue(), style("Available hooks:").blue());

            use hooksmith::modules::hook_builder::list_hooks;
            use std::path::PathBuf;

            let hooks_dir = PathBuf::from("target/hooks");

            match list_hooks(&hooks_dir).await {
                Ok(hooks) => {
                    if hooks.is_empty() {
                        println!("{} {}", style("ℹ️").blue(), style("No hooks found in target/hooks directory").blue());
                        println!("{} {}", style("💡").yellow(), style("Run 'hooksmith build <hook-name>' to create hooks").yellow());
                    } else {
                        println!("{} {} hooks found:", style("📦").blue(), style(hooks.len()).blue());
                        println!();

                        for hook in hooks {
                            println!("  {} {}", style("🔧").green(), style(&hook.name).yellow());
                            if let Some(description) = hook.description {
                                println!("     Description: {}", description);
                            }
                            if let Some(version) = hook.version {
                                println!("     Version: {}", version);
                            }
                            if !hook.supported_hooks.is_empty() {
                                println!("     Supported hooks: {}", hook.supported_hooks.join(", "));
                            }
                            if hook.requires_wasm {
                                println!("     {} Requires WASM components", style("🔗").blue());
                            }
                            if !hook.wasm_dependencies.is_empty() {
                                println!("     WASM dependencies: {}", hook.wasm_dependencies.join(", "));
                            }
                            println!("     Built: {}", hook.build_timestamp);
                            println!();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Failed to list hooks: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate { config_path } => {
            println!("{} {} {}", style("🔍").blue(), style("Validating Lefthook config:").blue(), style(&config_path).yellow());
            
            // Import the lefthook module
            use hooksmith::modules::lefthook;
            
            // Validate existing configuration against schema
            match lefthook::validate_existing_config(std::path::Path::new(&config_path)).await {
                Ok(()) => println!("{} {}", style("✅").green(), style("Configuration is valid").green()),
                Err(e) => {
                    eprintln!("{} {}", style("❌").red(), style(format!("Configuration validation failed: {}", e)).red());
                    std::process::exit(1);
                }
            }
        }
        Commands::Wasm { wasm } => {
            match wasm {
                WasmCommands::Build { wit_file, output } => {
                    println!("{} {} {}", style("🔨").blue(), style("Building WASM from WIT:").blue(), style(wit_file).yellow());
                    println!("{} {}", style("📁").blue(), style(format!("Output: {}", output)).blue());

                    use hooksmith::modules::wasm::{WasmManager, WasmBuildConfig};
                    use std::path::PathBuf;

                    let config = WasmBuildConfig {
                        wit_file: PathBuf::from(wit_file),
                        output_dir: PathBuf::from(output),
                        ..Default::default()
                    };

                    let manager = WasmManager::new()
                        .map_err(|e| {
                            eprintln!("{} {}", style("❌").red(), style(format!("Failed to create WASM manager: {}", e)).red());
                            std::process::exit(1);
                        })?;

                    match manager.build_component(config).await {
                        Ok(result) => {
                            if result.success {
                                println!("{} {}", style("✅").green(), style("WASM component built successfully").green());
                                if let Some(wasm_file) = result.wasm_file {
                                    println!("{} {}", style("📦").blue(), style(format!("WASM: {:?}", wasm_file)).blue());
                                }
                                if let Some(bindings_file) = result.bindings_file {
                                    println!("{} {}", style("🔗").blue(), style(format!("Bindings: {:?}", bindings_file)).blue());
                                }
                                println!("{} {}", style("⏱️").blue(), style(result.metadata.get("execution_time_ms").unwrap_or(&"0".to_string())).blue());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("WASM build failed").red());
                                if let Some(error) = result.error {
                                    eprintln!("{} {}", style("💥").red(), style(error).red());
                                }
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", style("❌").red(), style(format!("WASM build error: {}", e)).red());
                            std::process::exit(1);
                        }
                    }
                }
                WasmCommands::Run { wasm_file, function, args } => {
                    println!("{} {} {}", style("⚡").blue(), style("Running WASM:").blue(), style(wasm_file).yellow());
                    println!("{} {} {}", style("🔧").blue(), style("Function:").blue(), style(function).yellow());
                    println!("{} {} {:?}", style("📝").blue(), style("Args:").blue(), args);

                    use hooksmith::modules::wasm::{WasmManager, WasmRunConfig};
                    use std::path::PathBuf;
                    use std::collections::HashMap;

                    let config = WasmRunConfig {
                        wasm_file: PathBuf::from(wasm_file),
                        function: function.clone(),
                        args: args.clone(),
                        enable_wasi: true,
                        env_vars: HashMap::new(),
                        working_dir: None,
                    };

                    let mut manager = WasmManager::new()
                        .map_err(|e| {
                            eprintln!("{} {}", style("❌").red(), style(format!("Failed to create WASM manager: {}", e)).red());
                            std::process::exit(1);
                        })?;

                    match manager.run_component(config).await {
                        Ok(result) => {
                            if result.success {
                                println!("{} {}", style("✅").green(), style("WASM execution successful").green());
                                if let Some(return_value) = result.return_value {
                                    println!("{} {} {}", style("📤").blue(), style("Return:").blue(), style(return_value).yellow());
                                }
                                if !result.stdout.is_empty() {
                                    println!("{} {}", style("📄").blue(), style("STDOUT:").blue());
                                    println!("{}", result.stdout);
                                }
                                if !result.stderr.is_empty() {
                                    println!("{} {}", style("⚠️").yellow(), style("STDERR:").yellow());
                                    println!("{}", result.stderr);
                                }
                                println!("{} {}ms", style("⏱️").blue(), style(result.execution_time_ms).blue());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("WASM execution failed").red());
                                if let Some(error) = result.error {
                                    eprintln!("{} {}", style("💥").red(), style(error).red());
                                }
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", style("❌").red(), style(format!("WASM execution error: {}", e)).red());
                            std::process::exit(1);
                        }
                    }
                }
                WasmCommands::Bindings { wit_file, output } => {
                    println!("{} {} {}", style("🔗").blue(), style("Generating bindings from WIT:").blue(), style(wit_file).yellow());
                    println!("{} {}", style("📁").blue(), style(format!("Output: {}", output)).blue());

                    use hooksmith::modules::wasm::{WasmManager, WasmBuildConfig};
                    use std::path::PathBuf;

                    let config = WasmBuildConfig {
                        wit_file: PathBuf::from(wit_file),
                        output_dir: PathBuf::from(output),
                        generate_bindings: true,
                        ..Default::default()
                    };

                    let manager = WasmManager::new()
                        .map_err(|e| {
                            eprintln!("{} {}", style("❌").red(), style(format!("Failed to create WASM manager: {}", e)).red());
                            std::process::exit(1);
                        })?;

                    match manager.generate_bindings(config).await {
                        Ok(result) => {
                            if result.success {
                                println!("{} {}", style("✅").green(), style("Bindings generated successfully").green());
                                if let Some(bindings_file) = result.bindings_file {
                                    println!("{} {}", style("📦").blue(), style(format!("Bindings: {:?}", bindings_file)).blue());
                                }
                                println!("{} {}", style("⏱️").blue(), style(result.metadata.get("execution_time_ms").unwrap_or(&"0".to_string())).blue());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("Bindings generation failed").red());
                                if let Some(error) = result.error {
                                    eprintln!("{} {}", style("💥").red(), style(error).red());
                                }
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", style("❌").red(), style(format!("Bindings generation error: {}", e)).red());
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Worktree { worktree } => {
            match worktree {
                WorktreeCommands::Create { branch_name, tool, base } => {
                    println!("{} {} {}", style("🌳").blue(), style("Creating worktree:").blue(), style(branch_name).yellow());
                    if let Some(tool) = tool {
                        println!("{} {} {}", style("🔧").blue(), style("Using tool:").blue(), style(tool).yellow());
                    }
                    println!("{} {} {}", style("📁").blue(), style("Base directory:").blue(), style(base).yellow());

                    // For now, we'll use a simple implementation that calls the worktree tools directly
                    // In the future, this will use the WASM component
                    use std::process::Command;
                    use std::collections::HashMap;

                    let tool_name = tool.as_deref().unwrap_or("git");
                    let mut cmd = Command::new(tool_name);

                    match tool_name {
                        "wtp" => {
                            cmd.arg("add").arg(&branch_name);
                        }
                        "wt" => {
                            cmd.arg("add").arg(&branch_name);
                        }
                        "treekanga" => {
                            cmd.arg("create").arg(&branch_name);
                        }
                        "git" => {
                            cmd.args(&["worktree", "add", &format!("{}/{}", base, branch_name), &branch_name]);
                        }
                        _ => {
                            eprintln!("{} {} {}", style("❌").red(), style("Unknown tool:").red(), style(tool_name).yellow());
                            std::process::exit(1);
                        }
                    }

                    match cmd.status() {
                        Ok(status) => {
                            if status.success() {
                                println!("{} {}", style("✅").green(), style("Worktree created successfully").green());
                                println!("{} {} {}", style("📁").blue(), style("Location:").blue(), style(format!("{}/{}", base, branch_name)).yellow());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("Failed to create worktree").red());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {} {}", style("❌").red(), style("Failed to execute tool:").red(), style(e).yellow());
                            std::process::exit(1);
                        }
                    }
                }
                WorktreeCommands::List { tool } => {
                    println!("{} {}", style("📋").blue(), style("Listing worktrees:").blue());
                    if let Some(tool) = tool {
                        println!("{} {} {}", style("🔧").blue(), style("Using tool:").blue(), style(tool).yellow());
                    }

                    use std::process::Command;

                    let tool_name = tool.as_deref().unwrap_or("git");
                    let mut cmd = Command::new(tool_name);

                    match tool_name {
                        "wtp" => {
                            cmd.arg("list");
                        }
                        "wt" => {
                            cmd.arg("list");
                        }
                        "treekanga" => {
                            cmd.arg("list");
                        }
                        "git" => {
                            cmd.args(&["worktree", "list"]);
                        }
                        _ => {
                            eprintln!("{} {} {}", style("❌").red(), style("Unknown tool:").red(), style(tool_name).yellow());
                            std::process::exit(1);
                        }
                    }

                    match cmd.output() {
                        Ok(output) => {
                            if output.status.success() {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                println!("{} {}", style("✅").green(), style("Worktrees:").green());
                                println!("{}", stdout);
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                eprintln!("{} {}", style("❌").red(), style("Failed to list worktrees").red());
                                eprintln!("{}", stderr);
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {} {}", style("❌").red(), style("Failed to execute tool:").red(), style(e).yellow());
                            std::process::exit(1);
                        }
                    }
                }
                WorktreeCommands::Switch { worktree_name, tool } => {
                    println!("{} {} {}", style("🔄").blue(), style("Switching to worktree:").blue(), style(worktree_name).yellow());
                    if let Some(tool) = tool {
                        println!("{} {} {}", style("🔧").blue(), style("Using tool:").blue(), style(tool).yellow());
                    }

                    use std::process::Command;

                    let tool_name = tool.as_deref().unwrap_or("git");
                    let mut cmd = Command::new(tool_name);

                    match tool_name {
                        "wtp" => {
                            cmd.arg("switch").arg(&worktree_name);
                        }
                        "wt" => {
                            cmd.arg("switch").arg(&worktree_name);
                        }
                        "treekanga" => {
                            cmd.arg("switch").arg(&worktree_name);
                        }
                        "git" => {
                            // For git, we need to find the worktree path first
                            let list_cmd = Command::new("git")
                                .args(&["worktree", "list"])
                                .output();

                            if let Ok(output) = list_cmd {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                // Parse the worktree list to find the path
                                // This is a simplified implementation
                                eprintln!("{} {}", style("⚠️").yellow(), style("Git worktree switching requires manual navigation").yellow());
                                eprintln!("{} {}", style("💡").blue(), style("Use 'cd <worktree-path>' to switch").blue());
                                return Ok(());
                            }
                        }
                        _ => {
                            eprintln!("{} {} {}", style("❌").red(), style("Unknown tool:").red(), style(tool_name).yellow());
                            std::process::exit(1);
                        }
                    }

                    match cmd.status() {
                        Ok(status) => {
                            if status.success() {
                                println!("{} {}", style("✅").green(), style("Switched to worktree successfully").green());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("Failed to switch worktree").red());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {} {}", style("❌").red(), style("Failed to execute tool:").red(), style(e).yellow());
                            std::process::exit(1);
                        }
                    }
                }
                WorktreeCommands::Remove { worktree_name, with_branch, tool } => {
                    println!("{} {} {}", style("🗑️").blue(), style("Removing worktree:").blue(), style(worktree_name).yellow());
                    if with_branch {
                        println!("{} {}", style("🌿").blue(), style("Also removing branch").blue());
                    }
                    if let Some(tool) = tool {
                        println!("{} {} {}", style("🔧").blue(), style("Using tool:").blue(), style(tool).yellow());
                    }

                    use std::process::Command;

                    let tool_name = tool.as_deref().unwrap_or("git");
                    let mut cmd = Command::new(tool_name);

                    match tool_name {
                        "wtp" => {
                            if with_branch {
                                cmd.arg("remove").arg("--with-branch").arg(&worktree_name);
                            } else {
                                cmd.arg("remove").arg(&worktree_name);
                            }
                        }
                        "wt" => {
                            cmd.arg("remove").arg(&worktree_name);
                        }
                        "treekanga" => {
                            cmd.arg("remove").arg(&worktree_name);
                        }
                        "git" => {
                            if with_branch {
                                cmd.args(&["worktree", "remove", "--force", &worktree_name]);
                                // Also remove the branch
                                let branch_cmd = Command::new("git")
                                    .args(&["branch", "-D", &worktree_name])
                                    .status();
                                if let Ok(status) = branch_cmd {
                                    if status.success() {
                                        println!("{} {}", style("🌿").green(), style("Branch removed").green());
                                    }
                                }
                            } else {
                                cmd.args(&["worktree", "remove", &worktree_name]);
                            }
                        }
                        _ => {
                            eprintln!("{} {} {}", style("❌").red(), style("Unknown tool:").red(), style(tool_name).yellow());
                            std::process::exit(1);
                        }
                    }

                    match cmd.status() {
                        Ok(status) => {
                            if status.success() {
                                println!("{} {}", style("✅").green(), style("Worktree removed successfully").green());
                            } else {
                                eprintln!("{} {}", style("❌").red(), style("Failed to remove worktree").red());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {} {}", style("❌").red(), style("Failed to execute tool:").red(), style(e).yellow());
                            std::process::exit(1);
                        }
                    }
                }
                WorktreeCommands::Tools => {
                    println!("{} {}", style("🔧").blue(), style("Available worktree tools:").blue());

                    use std::process::Command;
                    use which::which;

                    let tools = vec![
                        ("wtp", "Smart Git worktree CLI with branch-only commands"),
                        ("wt", "Git worktree switcher for quick navigation"),
                        ("treekanga", "Community CLI for worktree management"),
                        ("git", "Native Git worktree commands"),
                    ];

                    for (tool_name, description) in tools {
                        let available = which(tool_name).is_ok();
                        let status_icon = if available { "✅" } else { "❌" };
                        let status_text = if available { "Available" } else { "Not found" };

                        println!("  {} {} {} ({})",
                            style(status_icon).green(),
                            style(tool_name).yellow(),
                            style(description).blue(),
                            style(status_text).dim()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
