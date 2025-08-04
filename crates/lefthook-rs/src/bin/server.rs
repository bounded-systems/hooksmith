//! Lefthook integration RPC server binary
//!
//! This binary starts a Warp-based RPC server that exposes the lefthook-rs
//! API with JSON schema generation.

use clap::Parser;
use lefthook_rs::schema;

#[derive(Parser)]
#[command(name = "lefthook-rs-server")]
#[command(about = "Lefthook integration RPC server for Hooksmith")]
struct Args {
    /// Port to bind the server to
    #[arg(short, long, default_value = "3032")]
    port: u16,
    
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🔧 Lefthook Integration RPC Server");
    println!("==================================");
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
