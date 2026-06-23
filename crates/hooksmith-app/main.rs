//! hooksmith — stream-driven hook engine (issue #82 rebuild)
//!
//! A hook is a capability that reacts to a git event stream.
//! Events flow: git → event → policy → reaction.
//!
//! Status: scaffold only. The event/stream contract (WIT interface) and
//! policy engine are being designed under issue #82.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hooksmith")]
#[command(about = "Stream-driven git hook engine (under construction — see issue #82)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version and status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            println!("hooksmith {}", env!("CARGO_PKG_VERSION"));
            println!("Stream-driven hook engine — under reconstruction (issue #82).");
            println!("See: https://github.com/bounded-systems/hooksmith/issues/82");
        }
    }
    Ok(())
}
