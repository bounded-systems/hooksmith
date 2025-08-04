//! Git operations RPC server binary
//!
//! This binary starts a Warp-based RPC server that exposes the git-operations
//! API with JSON schema generation.

use clap::Parser;
use git_operations::schema;

#[derive(Parser)]
#[command(name = "git-operations-server")]
#[command(about = "Git operations RPC server for Hooksmith")]
struct Args {
    /// Port to bind the server to
    #[arg(short, long, default_value = "3031")]
    port: u16,
    
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🔧 Git Operations RPC Server");
    println!("============================");
    println!("Starting server on {}:{}", args.host, args.port);
    println!();
    println!("Available endpoints:");
    println!("  GET  /schema  - JSON schema for all API types");
    println!("  GET  /health  - Health check endpoint");
    println!("  GET  /info    - API information");
    println!();
    
    // Start the server
    schema::start_server(args.port).await?;
    
    Ok(())
} 
