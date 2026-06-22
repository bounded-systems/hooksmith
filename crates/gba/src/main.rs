use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "gba")]
#[command(about = "Git Blob Analysis Tools - Repository analysis and hygiene")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Repository size analysis and limits
    #[command(name = "repo-audit")]
    RepoAudit {
        /// Fail if findings exceed this number
        #[arg(long, value_name = "N")]
        fail_on: Option<usize>,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to repository_size_auditor
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Analyze Rust blob sizes and hotspots
    #[command(name = "rust-blob")]
    RustBlob {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to rust_blob_analyzer
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Analyze Git delta compression opportunities
    #[command(name = "delta")]
    Delta {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to git_delta_analyzer
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Repository hygiene and LFS recommendations
    #[command(name = "hygiene")]
    Hygiene {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to git_hygiene_reporter
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// LFS analysis and auto-tracking
    #[command(name = "lfs")]
    Lfs {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to git_lfs_auto_tracker
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Packfile delta analysis
    #[command(name = "packfile")]
    Packfile {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to packfile_delta_analyzer
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// File churn analysis
    #[command(name = "churn")]
    Churn {
        /// Time period to analyze (e.g., "6 months ago")
        #[arg(default_value = "6 months ago")]
        since: String,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to file_churn_analyzer
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Tree object stability analysis
    #[command(name = "tree-stability")]
    TreeStability {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Extra arguments passed to tree_object_stability_auditor
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Extract tree to separate repository
    #[command(name = "extract")]
    Extract {
        /// Source directory to extract
        source: String,
        /// Target repository path
        target: String,
        /// Extra arguments passed to tree_to_repo_extractor
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Md,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Md => write!(f, "md"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RepoAudit {
            fail_on,
            format,
            args,
        } => run_tool("repository_size_auditor", fail_on, format, args),
        Commands::RustBlob { format, args } => run_tool("rust_blob_analyzer", None, format, args),
        Commands::Delta { format, args } => run_tool("git_delta_analyzer", None, format, args),
        Commands::Hygiene { format, args } => run_tool("git_hygiene_reporter", None, format, args),
        Commands::Lfs { format, args } => run_tool("git_lfs_auto_tracker", None, format, args),
        Commands::Packfile { format, args } => {
            run_tool("packfile_delta_analyzer", None, format, args)
        }
        Commands::Churn {
            since,
            format,
            args,
        } => {
            let mut all_args = vec![since];
            all_args.extend(args);
            run_tool("file_churn_analyzer", None, format, all_args)
        }
        Commands::TreeStability { format, args } => {
            run_tool("tree_object_stability_auditor", None, format, args)
        }
        Commands::Extract {
            source,
            target,
            args,
        } => {
            let mut all_args = vec![source, target];
            all_args.extend(args);
            run_tool(
                "tree_to_repo_extractor",
                None,
                OutputFormat::Table,
                all_args,
            )
        }
    }
}

fn run_tool(
    tool_name: &str,
    fail_on: Option<usize>,
    format: OutputFormat,
    mut args: Vec<String>,
) -> anyhow::Result<()> {
    // Add format argument
    args.extend_from_slice(&["--format".to_string(), format.to_string()]);

    // Add fail-on argument if provided
    if let Some(n) = fail_on {
        args.extend_from_slice(&["--fail-on".to_string(), n.to_string()]);
    }

    // Set environment variables for consistent behavior
    std::env::set_var("GBA_OUTPUT", format.to_string());
    if let Some(n) = fail_on {
        std::env::set_var("GBA_FAIL_ON", n.to_string());
    }

    let status = Command::new(format!("cargo"))
        .args(&["run", "--bin", tool_name, "--"])
        .args(&args)
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
