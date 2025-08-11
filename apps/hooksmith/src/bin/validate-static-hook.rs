use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "validate-static-hook")]
#[command(about = "Validate static hook definitions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a single static hook file
    Validate {
        /// Path to the static hook file
        #[arg(long)]
        file: PathBuf,
    },
    /// Validate all static hooks in a directory
    ValidateDir {
        /// Directory containing static hook files
        #[arg(long)]
        dir: PathBuf,
    },
    /// Discover and validate all static hooks
    Discover {
        /// Root directory to search for static hooks
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
            validate_single_hook(&file)?;
        }
        Commands::ValidateDir { dir } => {
            validate_directory(&dir)?;
        }
        Commands::Discover { root } => {
            discover_hooks(&root)?;
        }
    }

    Ok(())
}

fn validate_single_hook(file: &PathBuf) -> Result<()> {
    println!("🔍 Validating static hook: {}", file.display());

    match hooksmith::modules::static_hook::load_static_hook(file) {
        Ok(hook) => {
            println!("✅ Hook '{}' is valid", hook.name);
            println!("   Scope: {}", hook.scope_str());
            println!("   Concerns: {:?}", hook.concerns);
            println!("   Binary: {}", hook.bin);
        }
        Err(e) => {
            eprintln!("❌ Hook validation failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn validate_directory(dir: &PathBuf) -> Result<()> {
    println!("🔍 Validating static hooks in directory: {}", dir.display());

    let hooks = hooksmith::modules::static_hook::validate_static_hooks(dir)?;

    if hooks.is_empty() {
        println!("⚠️  No static hook files found in {}", dir.display());
    } else {
        println!("✅ Found {} valid static hooks:", hooks.len());
        for hook in hooks {
            println!(
                "   - {} (scope: {}, concerns: {:?})",
                hook.name,
                hook.scope_str(),
                hook.concerns
            );
        }
    }

    Ok(())
}

fn discover_hooks(root: &PathBuf) -> Result<()> {
    println!("🔍 Discovering static hooks in: {}", root.display());

    let mut total_hooks = 0;
    let mut valid_hooks = 0;

    // Look for .hooksmith/hooks/ directory structure
    let hooksmith_dir = root.join(".hooksmith").join("hooks");
    if hooksmith_dir.exists() {
        for scope_dir in std::fs::read_dir(&hooksmith_dir)? {
            let scope_dir = scope_dir?;
            let scope_path = scope_dir.path();

            if scope_path.is_dir() {
                let scope_name = scope_path.file_name().unwrap().to_string_lossy();
                println!("📁 Scope: {}", scope_name);

                let hooks = hooksmith::modules::static_hook::validate_static_hooks(&scope_path)?;
                total_hooks += hooks.len();
                valid_hooks += hooks.len();

                for hook in hooks {
                    println!("   ✅ {} (concerns: {:?})", hook.name, hook.concerns);
                }
            }
        }
    }

    println!("\n📊 Summary:");
    println!("   Total hooks found: {}", total_hooks);
    println!("   Valid hooks: {}", valid_hooks);
    println!("   Invalid hooks: {}", total_hooks - valid_hooks);

    if total_hooks > valid_hooks {
        std::process::exit(1);
    }

    Ok(())
}
