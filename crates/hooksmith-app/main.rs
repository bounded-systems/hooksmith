//! hooksmith — stream-driven hook engine (issue #82 rebuild)
//!
//! A hook is a capability that reacts to a git event stream.
//! Events flow: git → hook-event → policy engine → hook-result → exit.
//!
//! The policy engine lives in `crates/components/hook-engine` as a pure
//! WASM component. This CLI is the host that supplies context and exits
//! with the right code.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hooksmith")]
#[command(about = "Stream-driven git hook engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version and status.
    Status,

    /// Evaluate a hook event and exit with the policy verdict.
    ///
    /// Hook binaries call this as a thin wrapper:
    ///   hooksmith evaluate --hook pre-commit
    ///
    /// Stdin is forwarded to the policy engine for hooks that use it
    /// (pre-receive, post-receive, pre-push).
    Evaluate {
        /// Which git hook fired (e.g. pre-commit, commit-msg, pre-push).
        #[arg(long)]
        hook: String,

        /// Arguments git passed to the hook.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            println!("hooksmith {}", env!("CARGO_PKG_VERSION"));
            println!("Stream-driven hook engine — event → policy → reaction.");
        }
        Commands::Evaluate { hook, args } => {
            use std::io::Read;

            // Read stdin if present (pre-receive, pre-push, etc.)
            let stdin_content = {
                let mut buf = String::new();
                let _ = std::io::stdin().read_to_string(&mut buf);
                if buf.is_empty() { None } else { Some(buf) }
            };

            // Collect relevant env vars
            let env_vars: Vec<(String, String)> = std::env::vars()
                .filter(|(k, _)| {
                    k.starts_with("GIT_")
                        || k == "HOME"
                        || k == "HOOKSMITH_TREE_ENTRIES"
                })
                .collect();

            let repo_root = std::env::var("GIT_WORK_TREE")
                .or_else(|_| std::env::var("PWD"))
                .unwrap_or_else(|_| ".".to_string());

            // Invoke the pure policy engine (native host build for now;
            // WASM via wasmtime once the component is compiled for wasm32-wasip2).
            let result = hook_engine::evaluate_impl_raw(
                &hook,
                &args,
                stdin_content.as_deref(),
                &env_vars,
                &repo_root,
            );

            // Print findings
            for f in &result.findings {
                let prefix = match f.level {
                    hook_engine::FindingLevel::Error => "error",
                    hook_engine::FindingLevel::Warn  => "warn ",
                    hook_engine::FindingLevel::Info  => "info ",
                };
                eprintln!("[{}] {} — {}", prefix, f.rule, f.message);
                if let Some(s) = &f.suggestion {
                    eprintln!("       hint: {}", s);
                }
            }

            if !result.allow {
                eprintln!("\nhooksmith: {}", result.summary);
            }

            std::process::exit(result.exit_code as i32);
        }
    }
    Ok(())
}
