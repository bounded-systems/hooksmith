use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "git-agreement")]
#[command(about = "Git plugin for agreement management")]
struct Cli {
    #[command(subcommand)]
    command: AgreementCommands,
}

#[derive(Subcommand)]
enum AgreementCommands {
    /// List all agreements
    List {
        /// Show only active agreements
        #[arg(long)]
        active_only: bool,
        /// Show only fulfilled agreements
        #[arg(long)]
        fulfilled_only: bool,
        /// Show detailed output
        #[arg(long)]
        verbose: bool,
    },
    /// Show details of a specific agreement
    Show {
        /// Scope SHA of the agreement to show
        scope: String,
    },
    /// Validate an agreement
    Validate {
        /// Scope SHA of the agreement to validate
        scope: String,
        /// Exit with error if validation fails
        #[arg(long)]
        strict: bool,
        /// Check origin/main reachability
        #[arg(long)]
        check_main: bool,
    },
    /// Resolve contract content from an agreement
    Resolve {
        /// Scope SHA of the agreement to resolve
        scope: String,
        /// Output format (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    /// Show agreement history/log
    Log {
        /// Scope SHA of the agreement to show history for (optional)
        scope: Option<String>,
    },
    /// Create a new agreement
    Create {
        /// Tree SHA - the filesystem layout this agreement applies to
        #[arg(long)]
        scope: String,
        /// Blob SHA - the contract that defines expectations or validation
        #[arg(long)]
        contract: String,
        /// Optional description of the agreement
        #[arg(long)]
        description: Option<String>,
    },
    /// Create agreement from current tree and contract file
    CreateFromFile {
        /// Path to the contract file
        #[arg(long)]
        contract_file: String,
        /// Optional description of the agreement
        #[arg(long)]
        description: Option<String>,
        /// Optional scope tree SHA (defaults to current HEAD tree)
        #[arg(long)]
        scope: Option<String>,
    },
    /// Honor an agreement by creating a remote branch and worktree
    Honor {
        /// Scope SHA of the agreement to honor
        scope: String,
        /// Base branch to create the new branch from (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
        /// Whether to create a worktree after creating the branch
        #[arg(long, default_value = "true")]
        create_worktree: bool,
        /// Worktree directory name (defaults to scope SHA)
        #[arg(long)]
        worktree_name: Option<String>,
        /// Whether to open the worktree in Cursor
        #[arg(long, default_value = "true")]
        open_in_cursor: bool,
    },
    /// Show the current agreement based on the current branch
    Current,
    /// List agreements with enhanced resolution details
    ListWithResolution,
    /// Verify agreement with detailed resolution
    Verify {
        /// Scope SHA of the agreement to verify
        scope: String,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Audit all agreements for trust decay
    Audit,
    /// List local agreements (agreements with worktrees)
    Local,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Convert our CLI args to xtask format
    let mut args = vec!["agreement".to_string()];

    match cli.command {
        AgreementCommands::List {
            active_only,
            fulfilled_only,
            verbose,
        } => {
            args.push("list".to_string());
            if active_only {
                args.push("--active-only".to_string());
            }
            if fulfilled_only {
                args.push("--fulfilled-only".to_string());
            }
            if verbose {
                args.push("--verbose".to_string());
            }
        }
        AgreementCommands::Show { scope } => {
            args.push("show".to_string());
            args.push(scope);
        }
        AgreementCommands::Validate {
            scope,
            strict,
            check_main,
        } => {
            args.push("validate".to_string());
            args.push(scope);
            if strict {
                args.push("--strict".to_string());
            }
            if check_main {
                args.push("--check-main".to_string());
            }
        }
        AgreementCommands::Resolve { scope, format } => {
            args.push("resolve".to_string());
            args.push(scope);
            args.push("--format".to_string());
            args.push(format);
        }
        AgreementCommands::Log { scope } => {
            args.push("log".to_string());
            if let Some(s) = scope {
                args.push(s);
            }
        }
        AgreementCommands::Create {
            scope,
            contract,
            description,
        } => {
            args.push("create".to_string());
            args.push("--scope".to_string());
            args.push(scope);
            args.push("--contract".to_string());
            args.push(contract);
            if let Some(desc) = description {
                args.push("--description".to_string());
                args.push(desc);
            }
        }
        AgreementCommands::CreateFromFile {
            contract_file,
            description,
            scope,
        } => {
            args.push("create-from-file".to_string());
            args.push("--contract-file".to_string());
            args.push(contract_file);
            if let Some(desc) = description {
                args.push("--description".to_string());
                args.push(desc);
            }
            if let Some(s) = scope {
                args.push("--scope".to_string());
                args.push(s);
            }
        }
        AgreementCommands::Honor {
            scope,
            base_branch,
            create_worktree,
            worktree_name,
            open_in_cursor,
        } => {
            args.push("honor".to_string());
            args.push(scope);
            args.push("--base-branch".to_string());
            args.push(base_branch);
            if create_worktree {
                args.push("--create-worktree".to_string());
            }
            if let Some(name) = worktree_name {
                args.push("--worktree-name".to_string());
                args.push(name);
            }
            if open_in_cursor {
                args.push("--open-in-cursor".to_string());
            }
        }
        AgreementCommands::Current => {
            args.push("current".to_string());
        }
        AgreementCommands::ListWithResolution => {
            args.push("list-with-resolution".to_string());
        }
        AgreementCommands::Verify { scope, verbose } => {
            args.push("verify".to_string());
            args.push(scope);
            if verbose {
                args.push("--verbose".to_string());
            }
        }
        AgreementCommands::Audit => {
            args.push("audit".to_string());
        }
        AgreementCommands::Local => {
            args.push("local".to_string());
        }
    }

    // Call xtask with the converted arguments
    let status = Command::new("cargo")
        .args(["run", "-p", "xtask", "--"])
        .args(&args)
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}
