use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json;
use std::path::PathBuf;
use tracing::info;

use worktree_runner::WorktreeRunner;

#[derive(Parser)]
#[command(name = "worktree-crd")]
#[command(about = "Worktree CRD Lifecycle Management System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Repository path
    #[arg(long, default_value = ".")]
    repo_path: PathBuf,

    /// Worktree base directory
    #[arg(long, default_value = "worktrees")]
    worktree_base: PathBuf,

    /// Storage directory for CRDs
    #[arg(long, default_value = ".worktree-state")]
    storage_dir: PathBuf,

    /// GitHub token for PR operations
    #[arg(long)]
    github_token: Option<String>,

    /// Output format
    #[arg(long, default_value = "json")]
    output_format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the CRD system
    Init,

    /// Run a complete reconciliation cycle
    Reconcile,

    /// Get status of all worktrees
    Status {
        /// Branch name to get status for
        #[arg(long)]
        branch: Option<String>,
    },

    /// Export CRDs
    Export {
        /// Output file path
        #[arg(long)]
        output: PathBuf,

        /// Export format (json, yaml, csv)
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Clean up old CRDs
    Cleanup {
        /// Maximum age in days
        #[arg(long, default_value = "30")]
        max_age_days: u64,
    },

    /// Get storage statistics
    Stats,

    /// Demo the complete workflow
    Demo,

    /// Tool integration commands
    Tools {
        #[command(subcommand)]
        tool_command: ToolCommands,
    },

    /// Bulk operations using integrated tools
    Bulk {
        #[command(subcommand)]
        bulk_command: BulkCommands,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let mut runner = WorktreeRunner::new();

    match cli.command {
        Commands::Init => {
            info!("Initializing CRD system...");
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;
            println!("✅ CRD system initialized successfully");
        }

        Commands::Reconcile => {
            info!("Running reconciliation cycle...");
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            let crds = runner.reconcile().await?;
            println!("✅ Reconciliation completed. Processed {} CRDs", crds.len());

            if cli.output_format == "json" {
                println!("{}", serde_json::to_string_pretty(&crds)?);
            } else {
                for crd in crds {
                    println!("{}", crd.get_summary());
                }
            }
        }

        Commands::Status { branch } => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            if let Some(branch_name) = branch {
                if let Some(crd) = runner.get_branch_status(&branch_name).await? {
                    if cli.output_format == "json" {
                        println!("{}", serde_json::to_string_pretty(&crd)?);
                    } else {
                        println!("{}", crd.get_summary());
                    }
                } else {
                    println!("❌ No CRD found for branch: {}", branch_name);
                }
            } else {
                let crds = runner.get_status().await?;
                println!("📊 Status of {} worktrees:", crds.len());

                if cli.output_format == "json" {
                    println!("{}", serde_json::to_string_pretty(&crds)?);
                } else {
                    for crd in crds {
                        println!("  {}", crd.get_summary());
                    }
                }
            }
        }

        Commands::Export { output, format } => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            let storage = runner.get_storage().unwrap();
            let export_format = match format.as_str() {
                "json" => worktree_runner::storage::ExportFormat::Json,
                "yaml" => worktree_runner::storage::ExportFormat::Yaml,
                "csv" => worktree_runner::storage::ExportFormat::Csv,
                _ => {
                    eprintln!("❌ Invalid export format: {}", format);
                    std::process::exit(1);
                }
            };

            storage.export_crds(export_format, &output).await?;
            println!("✅ Exported CRDs to: {:?}", output);
        }

        Commands::Cleanup { max_age_days } => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            let storage = runner.get_storage().unwrap();
            let deleted = storage.cleanup_old_crds(max_age_days).await?;
            println!("✅ Cleaned up {} old CRDs", deleted);
        }

        Commands::Stats => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            let storage = runner.get_storage().unwrap();
            let stats = storage.get_stats().await?;

            println!("📊 Storage Statistics:");
            println!("  Total CRDs: {}", stats.total_crds);
            println!("  Active CRDs: {}", stats.active_crds);
            println!("  Completed CRDs: {}", stats.completed_crds);
            println!("  Failed CRDs: {}", stats.failed_crds);
            println!("  Storage size: {} bytes", stats.storage_size_bytes);
        }

        Commands::Demo => {
            println!("🎭 Worktree CRD Lifecycle Demo");
            println!("================================");

            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            // Run initial reconciliation
            println!("1. Running initial reconciliation...");
            let crds = runner.reconcile().await?;
            println!("   Found {} worktrees", crds.len());

            // Show status
            println!("2. Current status:");
            for crd in &crds {
                println!("   {}", crd.get_summary());
            }

            // Show storage stats
            println!("3. Storage statistics:");
            let storage = runner.get_storage().unwrap();
            let stats = storage.get_stats().await?;
            println!("   Total CRDs: {}", stats.total_crds);
            println!("   Active CRDs: {}", stats.active_crds);

            println!("✅ Demo completed successfully!");
        }

        Commands::Tools { tool_command } => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            match tool_command {
                ToolCommands::Status => {
                    let tool_status = runner.get_tool_status()?;
                    println!("🔧 Available Tools:");
                    for tool in tool_status {
                        let status = if tool.available { "✅" } else { "❌" };
                        let preferred = if tool.preferred { " (preferred)" } else { "" };
                        println!("  {} {}{}", status, tool.name, preferred);
                        if let Some(version) = tool.version {
                            println!("    Version: {}", version);
                        }
                    }
                }

                ToolCommands::Create { branch } => {
                    println!("Creating worktree for branch: {}", branch);
                    let result = runner.create_worktree_with_setup(&branch).await?;
                    if result.success {
                        println!(
                            "✅ Worktree created successfully using {}",
                            result.tool_used
                        );
                        println!("Output: {}", result.output);
                    } else {
                        println!("❌ Failed to create worktree");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                    }
                }

                ToolCommands::Setup { branch } => {
                    println!("Setting up environment for branch: {}", branch);
                    let enhanced_ops = runner.get_enhanced_ops()?;
                    let result = enhanced_ops
                        .tool_manager
                        .execute_operation(
                            worktree_runner::tools::ToolOperation::SetupEnvironment,
                            &[&branch],
                        )
                        .await?;

                    if result.success {
                        println!("✅ Environment setup completed using {}", result.tool_used);
                    } else {
                        println!("❌ Failed to setup environment");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                    }
                }

                ToolCommands::Devspace { devspace_command } => match devspace_command {
                    DevspaceCommands::Switch { context } => {
                        println!("Switching to context: {}", context);
                        let result = runner.switch_context(&context).await?;
                        if result.success {
                            println!("✅ Switched to context using {}", result.tool_used);
                            println!("Output: {}", result.output);
                        } else {
                            println!("❌ Failed to switch context");
                            if let Some(error) = result.error {
                                println!("Error: {}", error);
                            }
                        }
                    }

                    DevspaceCommands::List => {
                        println!("Listing available contexts/worktrees...");
                        let result = runner.list_devspace_worktrees().await?;
                        if result.success {
                            println!("✅ Contexts listed using {}", result.tool_used);
                            println!("Output: {}", result.output);
                        } else {
                            println!("❌ Failed to list contexts");
                            if let Some(error) = result.error {
                                println!("Error: {}", error);
                            }
                        }
                    }

                    DevspaceCommands::Create { branch } => {
                        println!("Creating devspace context for branch: {}", branch);
                        let result = runner.create_worktree_with_setup(&branch).await?;
                        if result.success {
                            println!("✅ Context created using {}", result.tool_used);
                            println!("Output: {}", result.output);
                        } else {
                            println!("❌ Failed to create context");
                            if let Some(error) = result.error {
                                println!("Error: {}", error);
                            }
                        }
                    }
                },
            }
        }

        Commands::Bulk { bulk_command } => {
            runner
                .init_crd_system(
                    cli.repo_path,
                    cli.worktree_base,
                    cli.storage_dir,
                    cli.github_token,
                )
                .await?;

            match bulk_command {
                BulkCommands::Pull => {
                    println!("🔄 Pulling all worktrees...");
                    let result = runner.bulk_pull_all().await?;
                    if result.success {
                        println!("✅ Bulk pull completed using {}", result.tool_used);
                        println!("Output: {}", result.output);
                    } else {
                        println!("❌ Bulk pull failed");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                    }
                }

                BulkCommands::Prune { force } => {
                    println!("🧹 Pruning stale worktrees...");
                    let result = runner.prune_worktrees(force).await?;
                    if result.success {
                        println!("✅ Prune completed using {}", result.tool_used);
                        println!("Output: {}", result.output);
                    } else {
                        println!("❌ Prune failed");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                    }
                }

                BulkCommands::Status => {
                    println!("📊 Getting comprehensive status...");
                    let result = runner.get_enhanced_ops()?.get_status().await?;
                    if result.success {
                        println!("✅ Status retrieved using {}", result.tool_used);
                        println!("Output: {}", result.output);
                    } else {
                        println!("❌ Failed to get status");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Subcommand)]
enum ToolCommands {
    /// Show status of available tools
    Status,

    /// Create worktree with automatic setup
    Create {
        /// Branch name
        #[arg(long)]
        branch: String,
    },

    /// Setup environment for existing worktree
    Setup {
        /// Branch name
        #[arg(long)]
        branch: String,
    },

    /// Devspace-specific commands
    Devspace {
        #[command(subcommand)]
        devspace_command: DevspaceCommands,
    },
}

#[derive(Subcommand)]
enum BulkCommands {
    /// Pull all worktrees
    Pull,

    /// Prune stale worktrees
    Prune {
        /// Force removal
        #[arg(long)]
        force: bool,
    },

    /// Get comprehensive status
    Status,
}

#[derive(Subcommand)]
enum DevspaceCommands {
    /// Switch to a different context/worktree
    Switch {
        /// Context name
        #[arg(long)]
        context: String,
    },

    /// List all available contexts/worktrees
    List,

    /// Create a new context/worktree
    Create {
        /// Branch name
        #[arg(long)]
        branch: String,
    },
}
