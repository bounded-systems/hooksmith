use clap::{Parser, Subcommand};
use hooksmith_core::git_snapshot::{format_snapshot_line_based, GitSnapshotCollector};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "git-snapshot")]
#[command(about = "Create comprehensive line-based snapshots of Git repository state")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a complete Git snapshot
    Create {
        /// Include unreachable objects in the snapshot
        #[arg(long)]
        include_unreachable: bool,

        /// Output format (line, json)
        #[arg(long, default_value = "line")]
        format: String,

        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<String>,
    },
    /// Show available Git object types
    Types,
    /// Show snapshot statistics
    Stats {
        /// Include unreachable objects in statistics
        #[arg(long)]
        include_unreachable: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            include_unreachable,
            format,
            output,
        } => {
            println!("🔍 Creating comprehensive Git snapshot...");
            if include_unreachable {
                println!("📦 Including unreachable objects...");
            }

            let snapshot = GitSnapshotCollector::create_snapshot(include_unreachable)?;

            let output_content = match format.as_str() {
                "line" => format_snapshot_line_based(&snapshot),
                "json" => serde_json::to_string_pretty(&snapshot)?,
                _ => {
                    eprintln!("❌ Unknown format: {}", format);
                    eprintln!("Available formats: line, json");
                    std::process::exit(1);
                }
            };

            if let Some(output_path) = output {
                fs::write(&output_path, output_content)?;
                println!("✅ Snapshot written to: {}", output_path);
            } else {
                println!("{}", output_content);
            }

            println!("📊 Snapshot summary:");
            println!("  Total objects: {}", snapshot.summary.total_objects);
            println!("  Commits: {}", snapshot.summary.commits);
            println!("  Trees: {}", snapshot.summary.trees);
            println!("  Blobs: {}", snapshot.summary.blobs);
            println!("  Tags: {}", snapshot.summary.tags);
            println!("  Stashes: {}", snapshot.summary.stashes);
            println!("  Worktrees: {}", snapshot.summary.worktrees);
            println!("  Index entries: {}", snapshot.summary.index_entries);
            println!("  Reflog entries: {}", snapshot.summary.reflog_entries);
            println!("  Remotes: {}", snapshot.summary.remotes);
            println!("  Config entries: {}", snapshot.summary.config_entries);
            if snapshot.summary.unreachable > 0 {
                println!("  Unreachable: {}", snapshot.summary.unreachable);
            }
        }
        Commands::Types => {
            println!("📋 Available Git object types:");
            println!("  Commit    - Git commit objects");
            println!("  Tree      - Git tree objects (directories)");
            println!("  Blob      - Git blob objects (files)");
            println!("  Tag       - Git tag objects");
            println!("  Stash     - Git stash entries");
            println!("  Worktree  - Git worktree information");
            println!("  Index     - Git index entries (staged files)");
            println!("  Reflog    - Git reflog entries");
            println!("  Remote    - Git remote configurations");
            println!("  Config    - Git configuration entries");
            println!("  Unreachable - Unreachable Git objects");
        }
        Commands::Stats {
            include_unreachable,
        } => {
            println!("📊 Collecting Git snapshot statistics...");
            if include_unreachable {
                println!("📦 Including unreachable objects...");
            }

            let snapshot = GitSnapshotCollector::create_snapshot(include_unreachable)?;

            println!("📈 Git Repository Statistics");
            println!("==========================");
            println!("Total objects: {}", snapshot.summary.total_objects);
            println!();
            println!("Git Objects:");
            println!("  Commits: {}", snapshot.summary.commits);
            println!("  Trees: {}", snapshot.summary.trees);
            println!("  Blobs: {}", snapshot.summary.blobs);
            println!("  Tags: {}", snapshot.summary.tags);
            println!();
            println!("Repository State:");
            println!("  Stashes: {}", snapshot.summary.stashes);
            println!("  Worktrees: {}", snapshot.summary.worktrees);
            println!("  Index entries: {}", snapshot.summary.index_entries);
            println!("  Reflog entries: {}", snapshot.summary.reflog_entries);
            println!("  Remotes: {}", snapshot.summary.remotes);
            println!("  Config entries: {}", snapshot.summary.config_entries);
            if snapshot.summary.unreachable > 0 {
                println!("  Unreachable: {}", snapshot.summary.unreachable);
            }
        }
    }

    Ok(())
}
