//! Binary entry point for lefthook-rs
//!
//! This binary provides a command-line interface for the lefthook-rs crate.

use lefthook_rs::cli;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Run the CLI
    if let Err(e) = cli::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
