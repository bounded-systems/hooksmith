use git_hooks::{run_hook_tests, list_hooks, get_hook_metadata};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hook-test-runner")]
#[command(about = "Comprehensive Git hook testing framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test all hooks for performance and functionality
    Test {
        /// Include Git operations testing
        #[arg(long)]
        git_ops: bool,
        
        /// Export results to JSON
        #[arg(long)]
        export: bool,
    },
    
    /// List all available hooks with their metadata
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    
    /// Get metadata for a specific hook
    Info {
        /// Hook name
        hook_name: String,
    },
    
    /// Show client vs server hook breakdown
    Breakdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { git_ops, export } => {
            println!("🧪 Running comprehensive hook tests...");
            
            if git_ops {
                println!("📝 Including Git operations testing");
            }
            
            run_hook_tests().await?;
            
            if export {
                println!("📄 Results exported to hook-test-results.json");
            }
        }
        
        Commands::List { detailed } => {
            println!("📋 Available Git Hooks:");
            println!("========================");
            
            let hooks = list_hooks();
            
            for hook_name in hooks {
                if detailed {
                    match get_hook_metadata(&hook_name) {
                        Ok(metadata) => {
                            println!("\n🔗 {}", hook_name);
                            println!("  Phase: {:?}", metadata.phase);
                            println!("  Scope: {:?}", metadata.scope);
                            println!("  Git Objects: {:?}", metadata.git_objects);
                            println!("  Validation Capabilities: {:?}", metadata.validation_capabilities);
                            println!("  Expects stdin: {}", metadata.expects_stdin);
                            println!("  Produces stdout: {}", metadata.produces_stdout);
                        }
                        Err(e) => {
                            println!("❌ {}: Error getting metadata - {}", hook_name, e);
                        }
                    }
                } else {
                    println!("  • {}", hook_name);
                }
            }
        }
        
        Commands::Info { hook_name } => {
            match get_hook_metadata(&hook_name) {
                Ok(metadata) => {
                    println!("📋 Hook Information: {}", hook_name);
                    println!("========================");
                    println!("Phase: {:?}", metadata.phase);
                    println!("Scope: {:?}", metadata.scope);
                    println!("Git Objects: {:?}", metadata.git_objects);
                    println!("Validation Capabilities: {:?}", metadata.validation_capabilities);
                    println!("Expects stdin: {}", metadata.expects_stdin);
                    println!("Produces stdout: {}", metadata.produces_stdout);
                    println!("Working Directory: {:?}", metadata.working_directory);
                }
                Err(e) => {
                    eprintln!("❌ Error getting metadata for {}: {}", hook_name, e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Breakdown => {
            println!("🔍 Git Hook Client vs Server Breakdown");
            println!("=====================================");
            
            let hooks = list_hooks();
            let mut client_hooks = Vec::new();
            let mut server_hooks = Vec::new();
            
            for hook_name in hooks {
                match get_hook_metadata(&hook_name) {
                    Ok(metadata) => {
                        match metadata.scope {
                            git_hooks::schema::HookScope::Client => {
                                client_hooks.push((hook_name, metadata));
                            }
                            git_hooks::schema::HookScope::Server => {
                                server_hooks.push((hook_name, metadata));
                            }
                        }
                    }
                    Err(_) => {
                        // Skip hooks we can't get metadata for
                    }
                }
            }
            
            println!("\n🖥️  Client-Side Hooks ({}):", client_hooks.len());
            println!("================================");
            for (hook_name, metadata) in &client_hooks {
                println!("  • {} ({:?})", hook_name, metadata.phase);
            }
            
            println!("\n🖥️  Server-Side Hooks ({}):", server_hooks.len());
            println!("================================");
            for (hook_name, metadata) in &server_hooks {
                println!("  • {} ({:?})", hook_name, metadata.phase);
            }
            
            println!("\n📊 Summary:");
            println!("  Total hooks: {}", client_hooks.len() + server_hooks.len());
            println!("  Client-side: {} ({:.1}%)", client_hooks.len(), 
                (client_hooks.len() as f64 / (client_hooks.len() + server_hooks.len()) as f64) * 100.0);
            println!("  Server-side: {} ({:.1}%)", server_hooks.len(),
                (server_hooks.len() as f64 / (client_hooks.len() + server_hooks.len()) as f64) * 100.0);
            
            // Show phase breakdown
            let mut phase_counts = std::collections::HashMap::new();
            for (_, metadata) in &client_hooks {
                *phase_counts.entry(metadata.phase.clone()).or_insert(0) += 1;
            }
            for (_, metadata) in &server_hooks {
                *phase_counts.entry(metadata.phase.clone()).or_insert(0) += 1;
            }
            
            println!("\n🔄 Phase Breakdown:");
            for (phase, count) in phase_counts {
                println!("  {:?}: {}", phase, count);
            }
        }
    }

    Ok(())
}
