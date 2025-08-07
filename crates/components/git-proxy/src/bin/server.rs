//! Git proxy server binary
//!
//! This binary starts a Git proxy server that can intercept, validate, and forward
//! Git operations to upstream repositories like GitHub.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use git_proxy::{
    config::ConfigManager,
    sync::{SyncConfig, SyncManager},
    server::CombinedServer,
    validation::ValidationEngine,
    GitProxyConfig, GitProxyEvent,
};

#[derive(Parser)]
#[command(name = "git-proxy-server")]
#[command(about = "Git proxy server for intercepting and forwarding Git operations")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "git-proxy.toml")]
    config: PathBuf,
    
    /// Upstream repository URL
    #[arg(long)]
    upstream_url: Option<String>,
    
    /// Proxy repository path
    #[arg(long)]
    proxy_repo_path: Option<PathBuf>,
    
    /// GitHub personal access token
    #[arg(long)]
    github_token: Option<String>,
    
    /// SSH key path
    #[arg(long)]
    ssh_key_path: Option<PathBuf>,
    
    /// HTTP server port
    #[arg(long, default_value = "8080")]
    http_port: u16,
    
    /// HTTP server host
    #[arg(long, default_value = "127.0.0.1")]
    http_host: String,
    
    /// SSH server port
    #[arg(long, default_value = "2222")]
    ssh_port: u16,
    
    /// SSH server host
    #[arg(long, default_value = "127.0.0.1")]
    ssh_host: String,
    
    /// Enable HTTP server
    #[arg(long, default_value = "true")]
    enable_http: bool,
    
    /// Enable SSH server
    #[arg(long, default_value = "false")]
    enable_ssh: bool,
    
    /// Sync interval in seconds
    #[arg(long, default_value = "300")]
    sync_interval: u64,
    
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Dry run mode
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(format!("git_proxy={}", args.log_level))
        .init();
    
    info!("🚀 Starting Git Proxy Server");
    info!("============================");
    
    // Load or create configuration
    let config_manager = ConfigManager::new(args.config.clone());
    let mut config = if args.config.exists() {
        config_manager.load()?
    } else {
        info!("Creating default configuration");
        config_manager.create_default()?
    };
    
    // Override config with command line arguments
    if let Some(upstream_url) = args.upstream_url {
        config.upstream_url = upstream_url;
    }
    
    if let Some(proxy_repo_path) = args.proxy_repo_path {
        config.proxy_repo_path = proxy_repo_path;
    }
    
    if let Some(github_token) = args.github_token {
        config.auth.github_token = Some(github_token);
    }
    
    if let Some(ssh_key_path) = args.ssh_key_path {
        config.auth.ssh_key_path = Some(ssh_key_path);
    }
    
    config.server.http_port = args.http_port;
    config.server.http_host = args.http_host;
    config.server.ssh_port = args.ssh_port;
    config.server.ssh_host = args.ssh_host;
    config.server.enable_http = args.enable_http;
    config.server.enable_ssh = args.enable_ssh;
    
    config.logging.level = args.log_level;
    
    // Validate configuration
    config.validate()?;
    
    // Save updated configuration
    config_manager.save(&config)?;
    
    info!("Configuration:");
    info!("  Upstream URL: {}", config.upstream_url);
    info!("  Proxy Repo Path: {}", config.proxy_repo_path.display());
    info!("  HTTP Server: {}:{} (enabled: {})", 
          config.server.http_host, config.server.http_port, config.server.enable_http);
    info!("  SSH Server: {}:{} (enabled: {})", 
          config.server.ssh_host, config.server.ssh_port, config.server.enable_ssh);
    info!("  Sync Interval: {} seconds", args.sync_interval);
    info!("  Dry Run: {}", args.dry_run);
    
    if args.dry_run {
        info!("Running in dry-run mode - no actual operations will be performed");
        return Ok(());
    }
    
    // Create event channel
    let (event_sender, mut event_receiver) = mpsc::channel::<GitProxyEvent>(100);
    
    // Initialize validation engine
    let mut validation_engine = ValidationEngine::new(config.clone());
    
    // Initialize sync manager
    let sync_config = SyncConfig {
        interval_seconds: args.sync_interval,
        auto_sync: true,
        max_retries: 3,
        retry_delay_seconds: 60,
        detect_force_pushes: true,
        detect_deletions: true,
        track_pr_branches: true,
        audit_logging: true,
        max_refs: Some(1000),
    };
    
    let mut sync_manager = SyncManager::new(sync_config, config.clone());
    sync_manager.set_event_sender(event_sender.clone());
    
    // Initialize server
    let mut server = CombinedServer::new(config.clone(), event_sender.clone());
    
    // Start sync manager
    info!("Initializing sync manager...");
    sync_manager.initialize().await?;
    
    // Start server
    info!("Starting server...");
    server.start().await?;
    
    info!("✅ Git proxy server started successfully");
    info!("");
    info!("Available endpoints:");
    if config.server.enable_http {
        info!("  HTTP: http://{}:{}", config.server.http_host, config.server.http_port);
        info!("    - GET  /health     - Health check");
        info!("    - GET  /status     - Server status");
        info!("    - GET  /info/refs  - Git info/refs");
        info!("    - POST /git-upload-pack   - Git upload-pack");
        info!("    - POST /git-receive-pack  - Git receive-pack");
    }
    if config.server.enable_ssh {
        info!("  SSH: ssh://{}:{}", config.server.ssh_host, config.server.ssh_port);
    }
    info!("");
    info!("Press Ctrl+C to stop the server");
    
    // Event processing loop
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                GitProxyEvent::ValidationRequest(request) => {
                    info!("Processing validation request: {}", request.request_id);
                    match validation_engine.validate_operation(request).await {
                        Ok(result) => {
                            if result.valid {
                                info!("Validation passed for request: {}", result.request_id);
                            } else {
                                warn!("Validation failed for request: {}: {:?}", 
                                      result.request_id, result.errors);
                            }
                        }
                        Err(e) => {
                            error!("Validation error for request: {}: {}", 
                                   request.request_id, e);
                        }
                    }
                }
                GitProxyEvent::PushRequest(request) => {
                    info!("Processing push request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                GitProxyEvent::PullRequest(request) => {
                    info!("Processing pull request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                GitProxyEvent::FetchRequest(request) => {
                    info!("Processing fetch request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                _ => {
                    debug!("Received event: {:?}", event);
                }
            }
        }
    });
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down Git proxy server...");
    
    // Stop server
    server.stop().await?;
    
    // Stop sync manager
    sync_manager.stop_periodic_sync();
    
    info!("✅ Git proxy server stopped");
    
    Ok(())
}
