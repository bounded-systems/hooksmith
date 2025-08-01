use clap::{Parser, Subcommand};
use console::style;
use anyhow::Result;

mod commands;
mod modules;

use commands::*;
use modules::*;

#[derive(Parser)]
#[command(name = "hooksmith")]
#[command(about = "Build Rust binaries into Lefthook hooks")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test command
    Test {
        /// Test message
        #[arg(long, default_value = "Hello from Hooksmith")]
        message: String,
    },
    /// Build Rust binaries for hooks
    Build {
        /// Hook name to build
        hook_name: String,
        /// Output directory
        #[arg(long, default_value = "target/hooks")]
        output: String,
    },
    /// Generate Lefthook configuration
    Generate {
        /// Output file
        #[arg(long, default_value = "lefthook.yml")]
        output: String,
    },
    /// Install hooks
    Install {
        /// Hook names to install (comma-separated)
        #[arg(long)]
        hooks: Option<String>,
    },
    /// List available hooks
    List,
}

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
            // TODO: Implement hook building
        }
        Commands::Generate { output } => {
            println!("{} {} {}", style("📝").blue(), style("Generating Lefthook config:").blue(), style(output).yellow());
            // TODO: Implement config generation
        }
        Commands::Install { hooks } => {
            let hook_list = hooks.unwrap_or_else(|| "all".to_string());
            println!("{} {} {}", style("🔧").blue(), style("Installing hooks:").blue(), style(hook_list).yellow());
            // TODO: Implement hook installation
        }
        Commands::List => {
            println!("{} {}", style("📋").blue(), style("Available hooks:").blue());
            // TODO: Implement hook listing
        }
    }

    Ok(())
}
