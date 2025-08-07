//! Git Proxy Demo
//!
//! This example demonstrates how to use the Git proxy component programmatically.

use anyhow::Result;
use git_proxy::{
    config::ConfigManager,
    sync::{SyncConfig, SyncManager},
    server::CombinedServer,
    validation::ValidationEngine,
    GitProxyConfig, GitProxyEvent, ValidationRequest, ValidationOperationType, ClientInfo, GitProtocol,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("git_proxy=info")
        .init();
    
    info!("🚀 Git Proxy Demo");
    info!("================");
    
    // Create a demo configuration
    let config = create_demo_config();
    
    // Create event channel
    let (event_sender, mut event_receiver) = mpsc::channel::<GitProxyEvent>(100);
    
    // Initialize components
    let mut validation_engine = ValidationEngine::new(config.clone());
    let mut sync_manager = create_sync_manager(&config, event_sender.clone());
    let mut server = CombinedServer::new(config.clone(), event_sender.clone());
    
    info!("Configuration:");
    info!("  Upstream URL: {}", config.upstream_url);
    info!("  Proxy Repo Path: {}", config.proxy_repo_path.display());
    info!("  HTTP Server: {}:{}", config.server.http_host, config.server.http_port);
    info!("  Validation Rules: {}", config.validation.required_patterns.len());
    
    // Start sync manager (in a real scenario)
    // sync_manager.initialize().await?;
    
    // Start server (in a real scenario)
    // server.start().await?;
    
    // Demo validation
    demo_validation(&mut validation_engine).await?;
    
    // Demo sync operations
    demo_sync_operations(&mut sync_manager).await?;
    
    // Event processing loop
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                GitProxyEvent::ValidationRequest(request) => {
                    info!("Received validation request: {}", request.request_id);
                }
                GitProxyEvent::PushRequest(request) => {
                    info!("Received push request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                GitProxyEvent::PullRequest(request) => {
                    info!("Received pull request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                GitProxyEvent::FetchRequest(request) => {
                    info!("Received fetch request: {} ({} refs)", 
                          request.request_id, request.refs.len());
                }
                _ => {
                    info!("Received event: {:?}", event);
                }
            }
        }
    });
    
    // Simulate some operations
    simulate_git_operations(&event_sender).await?;
    
    info!("✅ Demo completed successfully");
    
    Ok(())
}

fn create_demo_config() -> GitProxyConfig {
    GitProxyConfig {
        upstream_url: "https://github.com/user/demo-repo.git".to_string(),
        proxy_repo_path: PathBuf::from("./demo-proxy.git"),
        auth: git_proxy::ProxyAuthConfig {
            github_token: Some("demo_token".to_string()),
            ssh_key_path: None,
            username: Some("demo_user".to_string()),
        },
        validation: git_proxy::ValidationConfig {
            enable_pre_push: true,
            enable_commit_validation: true,
            enable_file_size_validation: true,
            max_file_size: Some(1024 * 1024), // 1MB
            blocked_patterns: vec![
                "*.key".to_string(),
                "*.pem".to_string(),
                "*.p12".to_string(),
                "*.pfx".to_string(),
                "id_rsa".to_string(),
                "id_ed25519".to_string(),
            ],
            required_patterns: vec![
                "feat:".to_string(),
                "fix:".to_string(),
                "docs:".to_string(),
                "style:".to_string(),
                "refactor:".to_string(),
                "test:".to_string(),
                "chore:".to_string(),
            ],
        },
        server: git_proxy::ServerConfig {
            http_port: 8080,
            http_host: "127.0.0.1".to_string(),
            ssh_port: 2222,
            ssh_host: "127.0.0.1".to_string(),
            enable_http: true,
            enable_ssh: false,
        },
        logging: git_proxy::LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            structured: true,
        },
    }
}

fn create_sync_manager(
    config: &GitProxyConfig,
    event_sender: mpsc::Sender<GitProxyEvent>,
) -> SyncManager {
    let sync_config = SyncConfig {
        interval_seconds: 300,
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
    sync_manager.set_event_sender(event_sender);
    sync_manager
}

async fn demo_validation(validation_engine: &mut ValidationEngine) -> Result<()> {
    info!("🧪 Demo: Validation Engine");
    info!("=========================");
    
    // Create a validation request
    let request = ValidationRequest {
        request_id: "demo-validation-001".to_string(),
        operation_type: ValidationOperationType::PrePush,
        refs: vec!["refs/heads/main".to_string()],
        commit_hashes: vec!["abc123".to_string()],
        client_info: ClientInfo {
            user_agent: "git-proxy-demo".to_string(),
            client_ip: "127.0.0.1".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    // Validate the request
    match validation_engine.validate_operation(request).await {
        Ok(result) => {
            if result.valid {
                info!("✅ Validation passed");
            } else {
                warn!("❌ Validation failed: {:?}", result.errors);
            }
            info!("Validation duration: {}ms", result.duration_ms);
        }
        Err(e) => {
            warn!("❌ Validation error: {}", e);
        }
    }
    
    info!("");
    Ok(())
}

async fn demo_sync_operations(sync_manager: &mut SyncManager) -> Result<()> {
    info!("🔄 Demo: Sync Operations");
    info!("=======================");
    
    // Get sync status
    let status = sync_manager.get_sync_status();
    info!("Sync Status:");
    info!("  Auto Sync: {}", status.auto_sync_enabled);
    info!("  Interval: {} seconds", status.sync_interval_seconds);
    info!("  Total Refs: {}", status.total_refs);
    info!("  Repository: {}", status.repository_url);
    
    // In a real scenario, you would call:
    // let sync_result = sync_manager.force_sync().await?;
    // info!("Force sync result: {:?}", sync_result);
    
    info!("");
    Ok(())
}

async fn simulate_git_operations(event_sender: &mpsc::Sender<GitProxyEvent>) -> Result<()> {
    info!("📝 Demo: Simulating Git Operations");
    info!("=================================");
    
    // Simulate a push request
    let push_request = git_proxy::GitPushRequest {
        request_id: "demo-push-001".to_string(),
        refs: vec!["refs/heads/feature/demo".to_string()],
        force: false,
        client_info: ClientInfo {
            user_agent: "git/2.30.1".to_string(),
            client_ip: "192.168.1.100".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Err(e) = event_sender.send(GitProxyEvent::PushRequest(push_request)).await {
        warn!("Failed to send push request: {}", e);
    }
    
    // Simulate a pull request
    let pull_request = git_proxy::GitPullRequest {
        request_id: "demo-pull-001".to_string(),
        refs: vec!["refs/heads/main".to_string()],
        client_info: ClientInfo {
            user_agent: "git/2.30.1".to_string(),
            client_ip: "192.168.1.100".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Err(e) = event_sender.send(GitProxyEvent::PullRequest(pull_request)).await {
        warn!("Failed to send pull request: {}", e);
    }
    
    // Simulate a fetch request
    let fetch_request = git_proxy::GitFetchRequest {
        request_id: "demo-fetch-001".to_string(),
        refs: vec!["refs/heads/*".to_string(), "refs/tags/*".to_string()],
        client_info: ClientInfo {
            user_agent: "git/2.30.1".to_string(),
            client_ip: "192.168.1.100".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Err(e) = event_sender.send(GitProxyEvent::FetchRequest(fetch_request)).await {
        warn!("Failed to send fetch request: {}", e);
    }
    
    info!("✅ Simulated Git operations sent");
    info!("");
    
    Ok(())
}
