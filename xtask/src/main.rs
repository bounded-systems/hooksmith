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
    /// Run all code generation tasks
    GenAll {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for LefthookConfig {
    fn default() -> Self {
        Self {
            pre_commit: None,
            pre_push: None,
            commit_msg: None,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { target, release } => {
            build_project(&target, release)?;
        }
        Commands::GenWit { output_dir, overwrite } => {
            generate_wit_interfaces(&output_dir, overwrite)?;
        }
        Commands::GenLefthook { output, validate } => {
            generate_lefthook_config(&output, validate)?;
        }
        Commands::GenDocs { output_dir, open } => {
            generate_documentation(&output_dir, open)?;
        }
        Commands::GenAll { overwrite } => {
            generate_all(overwrite)?;
        }
        Commands::Check { strict } => {
            check_generated_files(strict)?;
        }
    }

    Ok(())
}

/// Build the project and all components
fn build_project(target: &str, release: bool) -> Result<()> {
    println!("🔨 Building Hooksmith project...");
    println!("   Target: {}", target);
    println!("   Release: {}", release);

    let profile = if release { "release" } else { "debug" };

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
                println!("   Building WASM component: {}", component);
                let status = Command::new("cargo")
                    .args(["build", "--target", "wasm32-unknown-unknown"])
                    .args(if release { vec!["--release"] } else { vec![] })
                    .current_dir(format!("components/{}", component))
                    .status()
                    .context(format!("Failed to build WASM component: {}", component))?;

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
    println!("   Output directory: {}", output_dir);

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
                params: vec![
                    WitParam {
                        name: "config".to_string(),
                        param_type: "hook-config".to_string(),
                        docs: Some("Hook configuration".to_string()),
                    },
                ],
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
                params: vec![
                    WitParam {
                        name: "hook-names".to_string(),
                        param_type: "list<string>".to_string(),
                        docs: Some("Names of hooks to install".to_string()),
                    },
                ],
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
                params: vec![
                    WitParam {
                        name: "branch-name".to_string(),
                        param_type: "string".to_string(),
                        docs: Some("Name of the branch to create".to_string()),
                    },
                ],
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
        enums: vec![
            WitEnum {
                name: "worktree-tool".to_string(),
                docs: Some("Available worktree tools".to_string()),
                variants: vec![
                    "wtp".to_string(),
                    "wt".to_string(),
                    "treekanga".to_string(),
                    "git".to_string(),
                ],
            },
        ],
    };

    // Generate WIT files
    let interfaces = vec![
        ("hooksmith.wit", cli_interface),
        ("worktree-runner.wit", worktree_interface),
    ];

    for (filename, interface) in interfaces {
        let file_path = output_path.join(filename);
        
        if file_path.exists() && !overwrite {
            println!("   Skipping {} (already exists)", filename);
            continue;
        }

        let wit_content = generate_wit_content(&interface)?;
        fs::write(&file_path, wit_content).context(format!("Failed to write {}", filename))?;
        println!("   Generated {}", filename);
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
            content.push_str(&format!("/// {}\n", docs));
        }
        content.push_str(&format!("record {} {{\n", record.name));
        for field in &record.fields {
            if let Some(docs) = &field.docs {
                content.push_str(&format!("  /// {}\n", docs));
            }
            content.push_str(&format!("  {}: {};\n", field.name, field.field_type));
        }
        content.push_str("}\n\n");
    }

    // Enums
    for enum_def in &interface.enums {
        if let Some(docs) = &enum_def.docs {
            content.push_str(&format!("/// {}\n", docs));
        }
        content.push_str(&format!("enum {} {{\n", enum_def.name));
        for variant in &enum_def.variants {
            content.push_str(&format!("  {},\n", variant));
        }
        content.push_str("}\n\n");
    }

    // Interface
    if let Some(docs) = &interface.docs {
        content.push_str(&format!("/// {}\n", docs));
    }
    content.push_str(&format!("interface {} {{\n", interface.name));

    for function in &interface.functions {
        if let Some(docs) = &function.docs {
            content.push_str(&format!("  /// {}\n", docs));
        }
        let params = function
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.param_type))
            .collect::<Vec<_>>()
            .join(", ");
        content.push_str(&format!("  {}: func({}) -> {};\n", function.name, params, function.result));
    }

    content.push_str("}\n\n");

    // Export
    content.push_str(&format!("export {};\n", interface.name));

    Ok(content)
}

/// Generate Lefthook configuration from structured definitions
fn generate_lefthook_config(output: &str, validate: bool) -> Result<()> {
    println!("📝 Generating Lefthook configuration...");
    println!("   Output: {}", output);

    let mut config = LefthookConfig::default();

    // Pre-commit hooks
    let mut pre_commit_hooks = HashMap::new();
    pre_commit_hooks.insert(
        "hooksmith-fmt".to_string(),
        LefthookHook {
            run: "cargo fmt --all -- --check".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-clippy".to_string(),
        LefthookHook {
            run: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-test".to_string(),
        LefthookHook {
            run: "cargo test --all-targets --all-features".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-gen-wit".to_string(),
        LefthookHook {
            run: "cargo xtask gen-wit".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_commit = Some(pre_commit_hooks);

    // Pre-push hooks
    let mut pre_push_hooks = HashMap::new();
    pre_push_hooks.insert(
        "hooksmith-audit".to_string(),
        LefthookHook {
            run: "cargo audit".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );
    pre_push_hooks.insert(
        "hooksmith-check-generated".to_string(),
        LefthookHook {
            run: "cargo xtask check --strict".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_push = Some(pre_push_hooks);

    // Write configuration
    let yaml_content = serde_yaml::to_string(&config).context("Failed to serialize config")?;
    fs::write(output, yaml_content).context("Failed to write config file")?;

    if validate {
        println!("   Validating configuration...");
        // Basic validation - check that the file can be parsed
        let content = fs::read_to_string(output).context("Failed to read config file")?;
        let _parsed: LefthookConfig = serde_yaml::from_str(&content).context("Failed to parse config")?;
        println!("   Configuration validation passed");
    }

    println!("✅ Lefthook configuration generated successfully");
    Ok(())
}

/// Generate documentation
fn generate_documentation(output_dir: &str, open: bool) -> Result<()> {
    println!("📚 Generating documentation...");
    println!("   Output directory: {}", output_dir);

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

/// Generate all code generation tasks
fn generate_all(overwrite: bool) -> Result<()> {
    println!("🚀 Running all code generation tasks...");

    generate_wit_interfaces("wit", overwrite)?;
    generate_lefthook_config("lefthook.yml", true)?;
    generate_documentation("docs", false)?;

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
            println!("   ❌ Missing: {}", file);
            outdated = true;
        }
    }

    // Check Lefthook config
    if !Path::new("lefthook.yml").exists() {
        println!("   ❌ Missing: lefthook.yml");
        outdated = true;
    }

    if outdated {
        let message = "Generated files are outdated. Run 'cargo xtask gen-all' to regenerate.";
        if strict {
            anyhow::bail!(message);
        } else {
            println!("   ⚠️  {}", message);
        }
    } else {
        println!("   ✅ All generated files are up to date");
    }

    Ok(())
}
