//! Git proxy server implementation
//!
//! This module provides HTTP and SSH server implementations for the Git proxy,
//! handling incoming Git protocol requests and forwarding them to upstream.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use warp::{Filter, Rejection, Reply};

use crate::{
    hooks::{HookContext, ServerHookHandler, ServerHookType},
    ClientInfo, GitFetchRequest, GitFetchResult, GitProtocol, GitProxyConfig, GitProxyEvent,
    GitPullRequest, GitPullResult, GitPushRequest, GitPushResult, ServerStatus,
};

/// HTTP server for Git proxy
pub struct HttpServer {
    config: GitProxyConfig,
    event_sender: mpsc::Sender<GitProxyEvent>,
    status: ServerStatus,
    hook_handler: ServerHookHandler,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(config: GitProxyConfig, event_sender: mpsc::Sender<GitProxyEvent>) -> Self {
        Self {
            config: config.clone(),
            event_sender,
            status: ServerStatus {
                running: false,
                http_enabled: true,
                ssh_enabled: false,
                connections: 0,
                uptime_seconds: 0,
                last_operation: None,
            },
            hook_handler: ServerHookHandler::new(config),
        }
    }

    /// Start the HTTP server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!(
            "{}:{}",
            self.config.server.http_host, self.config.server.http_port
        )
        .parse::<SocketAddr>()
        .with_context(|| "Invalid server address")?;

        info!("Starting HTTP server on {}", addr);

        // Create routes
        let routes = self.create_routes();

        // Start the server
        warp::serve(routes).run(addr).await;

        self.status.running = true;
        Ok(())
    }

    /// Create HTTP routes
    fn create_routes(&self) -> impl Filter<Extract = impl Reply> + Clone {
        let event_sender = Arc::new(self.event_sender.clone());

        // Health check endpoint
        let health = warp::path("health").and(warp::get()).map(|| {
            warp::reply::json(&HealthResponse {
                status: "healthy".to_string(),
                timestamp: Utc::now(),
            })
        });

        // Git info/refs endpoint
        let info_refs = warp::path!("info" / "refs")
            .and(warp::get())
            .and(warp::query::<InfoRefsQuery>())
            .and(with_event_sender(Arc::clone(&event_sender)))
            .and_then(handle_info_refs);

        // Git upload-pack endpoint
        let upload_pack = warp::path!("git-upload-pack")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_event_sender(Arc::clone(&event_sender)))
            .and_then(handle_upload_pack);

        // Git receive-pack endpoint
        let receive_pack = warp::path!("git-receive-pack")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_event_sender(Arc::clone(&event_sender)))
            .and_then(handle_receive_pack);

        // Status endpoint
        let status = warp::path("status")
            .and(warp::get())
            .and(with_event_sender(Arc::clone(&event_sender)))
            .and_then(handle_status);

        // Combine routes
        health
            .or(info_refs)
            .or(upload_pack)
            .or(receive_pack)
            .or(status)
            .with(warp::cors().allow_any_origin())
    }

    /// Get server status
    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        self.status.running = false;
        info!("HTTP server stopped");
        Ok(())
    }

    /// Execute server-side hooks
    async fn execute_server_hooks(
        &self,
        hook_type: ServerHookType,
        args: Vec<String>,
        stdin_data: Option<String>,
    ) -> Result<()> {
        let context = HookContext {
            hook_type,
            repo_path: self.config.proxy_repo_path.clone(),
            work_dir: None,
            env_vars: std::env::vars().collect(),
            args,
            stdin_data,
            client_info: None,
            timestamp: Utc::now(),
        };

        let result = self.hook_handler.execute_hook(context).await?;

        if !result.success {
            warn!("Server hook failed: {:?}", result.error);
        } else {
            debug!("Server hook completed successfully");
        }

        Ok(())
    }
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: DateTime<Utc>,
}

/// Info/refs query parameters
#[derive(Debug, Deserialize)]
struct InfoRefsQuery {
    service: Option<String>,
}

/// Helper function to inject event sender into handlers
fn with_event_sender(
    sender: Arc<mpsc::Sender<GitProxyEvent>>,
) -> impl Filter<Extract = (Arc<mpsc::Sender<GitProxyEvent>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || Arc::clone(&sender))
}

/// Handle info/refs request
async fn handle_info_refs(
    query: InfoRefsQuery,
    sender: Arc<mpsc::Sender<GitProxyEvent>>,
) -> Result<impl Reply, Rejection> {
    debug!("Handling info/refs request: {:?}", query);

    // Create fetch request
    let fetch_request = GitFetchRequest {
        request_id: format!("fetch-{}", Utc::now().timestamp()),
        refs: vec!["refs/heads/*".to_string(), "refs/tags/*".to_string()],
        client_info: ClientInfo {
            user_agent: "git-http-client".to_string(),
            client_ip: "127.0.0.1".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    // Send event
    if let Err(e) = sender
        .send(GitProxyEvent::FetchRequest(fetch_request))
        .await
    {
        error!("Failed to send fetch request: {}", e);
        return Err(warp::reject::custom(ServerError::EventSendFailed));
    }

    // Return Git protocol response
    let response = format!(
        "# service=git-upload-pack\n\
         {}\n\
         0000",
        "0000" // Empty capabilities for now
    );

    Ok(warp::reply::with_header(
        response,
        "Content-Type",
        "application/x-git-upload-pack-advertisement",
    ))
}

/// Handle upload-pack request
async fn handle_upload_pack(
    body: bytes::Bytes,
    sender: Arc<mpsc::Sender<GitProxyEvent>>,
) -> Result<impl Reply, Rejection> {
    debug!("Handling upload-pack request");

    // Parse the request body to extract refs
    let body_str = String::from_utf8_lossy(&body);
    let refs = parse_upload_pack_request(&body_str);

    // Create fetch request
    let fetch_request = GitFetchRequest {
        request_id: format!("fetch-{}", Utc::now().timestamp()),
        refs,
        client_info: ClientInfo {
            user_agent: "git-http-client".to_string(),
            client_ip: "127.0.0.1".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["multi_ack".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    // Send event
    if let Err(e) = sender
        .send(GitProxyEvent::FetchRequest(fetch_request))
        .await
    {
        error!("Failed to send fetch request: {}", e);
        return Err(warp::reject::custom(ServerError::EventSendFailed));
    }

    // Return empty response for now (would contain pack data)
    Ok(warp::reply::with_header(
        "0000",
        "Content-Type",
        "application/x-git-upload-pack-result",
    ))
}

/// Handle receive-pack request
async fn handle_receive_pack(
    body: bytes::Bytes,
    sender: Arc<mpsc::Sender<GitProxyEvent>>,
) -> Result<impl Reply, Rejection> {
    debug!("Handling receive-pack request");

    // Parse the request body to extract refs
    let body_str = String::from_utf8_lossy(&body);
    let refs = parse_receive_pack_request(&body_str);

    // Create push request
    let push_request = GitPushRequest {
        request_id: format!("push-{}", Utc::now().timestamp()),
        refs,
        force: false, // Would be determined from request
        client_info: ClientInfo {
            user_agent: "git-http-client".to_string(),
            client_ip: "127.0.0.1".to_string(),
            protocol: GitProtocol::Https,
            capabilities: vec!["report-status".to_string(), "side-band-64k".to_string()],
        },
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    // Send event
    if let Err(e) = sender.send(GitProxyEvent::PushRequest(push_request)).await {
        error!("Failed to send push request: {}", e);
        return Err(warp::reject::custom(ServerError::EventSendFailed));
    }

    // Return success response
    Ok(warp::reply::with_header(
        "0000",
        "Content-Type",
        "application/x-git-receive-pack-result",
    ))
}

/// Handle status request
async fn handle_status(sender: Arc<mpsc::Sender<GitProxyEvent>>) -> Result<impl Reply, Rejection> {
    debug!("Handling status request");

    let status = ServerStatus {
        running: true,
        http_enabled: true,
        ssh_enabled: false,
        connections: 0,
        uptime_seconds: 0,
        last_operation: Some(Utc::now()),
    };

    Ok(warp::reply::json(&status))
}

/// Parse upload-pack request to extract refs
fn parse_upload_pack_request(body: &str) -> Vec<String> {
    let mut refs = Vec::new();

    for line in body.lines() {
        if line.starts_with("want ") {
            // Extract commit hash
            if let Some(hash) = line.split_whitespace().nth(1) {
                refs.push(format!("refs/heads/{}", hash));
            }
        }
    }

    refs
}

/// Parse receive-pack request to extract refs
fn parse_receive_pack_request(body: &str) -> Vec<String> {
    let mut refs = Vec::new();

    for line in body.lines() {
        if line.starts_with("0000") {
            // End of capabilities
            break;
        }
        if line.starts_with("want ") {
            // Extract commit hash
            if let Some(hash) = line.split_whitespace().nth(1) {
                refs.push(format!("refs/heads/{}", hash));
            }
        }
    }

    refs
}

/// Server error types
#[derive(Debug)]
pub enum ServerError {
    EventSendFailed,
    InvalidRequest,
    AuthenticationFailed,
}

impl warp::reject::Reject for ServerError {}

/// SSH server for Git proxy
pub struct SshServer {
    config: GitProxyConfig,
    event_sender: mpsc::Sender<GitProxyEvent>,
    status: ServerStatus,
    hook_handler: ServerHookHandler,
}

impl SshServer {
    /// Create a new SSH server
    pub fn new(config: GitProxyConfig, event_sender: mpsc::Sender<GitProxyEvent>) -> Self {
        Self {
            config: config.clone(),
            event_sender,
            status: ServerStatus {
                running: false,
                http_enabled: false,
                ssh_enabled: true,
                connections: 0,
                uptime_seconds: 0,
                last_operation: None,
            },
            hook_handler: ServerHookHandler::new(config),
        }
    }

    /// Start the SSH server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!(
            "{}:{}",
            self.config.server.ssh_host, self.config.server.ssh_port
        )
        .parse::<SocketAddr>()
        .with_context(|| "Invalid SSH server address")?;

        info!("Starting SSH server on {}", addr);

        // For now, we'll use a simple TCP listener
        // In a real implementation, you'd use an SSH library like ssh2
        let listener = tokio::net::TcpListener::bind(addr).await?;

        self.status.running = true;

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("SSH connection from {}", addr);
                    self.handle_ssh_connection(socket).await?;
                }
                Err(e) => {
                    error!("Failed to accept SSH connection: {}", e);
                }
            }
        }
    }

    /// Handle SSH connection
    async fn handle_ssh_connection(&self, _socket: tokio::net::TcpStream) -> Result<()> {
        // This is a placeholder implementation
        // In a real implementation, you'd:
        // 1. Perform SSH handshake
        // 2. Authenticate the user
        // 3. Handle Git protocol over SSH
        // 4. Forward requests to upstream

        info!("Handling SSH connection (placeholder)");
        Ok(())
    }

    /// Execute server-side hooks
    async fn execute_server_hooks(
        &self,
        hook_type: ServerHookType,
        args: Vec<String>,
        stdin_data: Option<String>,
    ) -> Result<()> {
        let context = HookContext {
            hook_type,
            repo_path: self.config.proxy_repo_path.clone(),
            work_dir: None,
            env_vars: std::env::vars().collect(),
            args,
            stdin_data,
            client_info: None,
            timestamp: Utc::now(),
        };

        let result = self.hook_handler.execute_hook(context).await?;

        if !result.success {
            warn!("Server hook failed: {:?}", result.error);
        } else {
            debug!("Server hook completed successfully");
        }

        Ok(())
    }

    /// Get server status
    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        self.status.running = false;
        info!("SSH server stopped");
        Ok(())
    }
}

/// Combined server that manages both HTTP and SSH servers
pub struct CombinedServer {
    http_server: Option<HttpServer>,
    ssh_server: Option<SshServer>,
    config: GitProxyConfig,
    event_sender: mpsc::Sender<GitProxyEvent>,
}

impl CombinedServer {
    /// Create a new combined server
    pub fn new(config: GitProxyConfig, event_sender: mpsc::Sender<GitProxyEvent>) -> Self {
        Self {
            http_server: None,
            ssh_server: None,
            config,
            event_sender,
        }
    }

    /// Start the servers
    pub async fn start(&mut self) -> Result<()> {
        // Start HTTP server if enabled
        if self.config.server.enable_http {
            let mut http_server = HttpServer::new(self.config.clone(), self.event_sender.clone());

            // Spawn HTTP server in background
            let http_config = self.config.clone();
            let http_event_sender = self.event_sender.clone();
            tokio::spawn(async move {
                let mut server = HttpServer::new(http_config, http_event_sender);
                if let Err(e) = server.start().await {
                    error!("HTTP server failed: {}", e);
                }
            });

            self.http_server = Some(http_server);
        }

        // Start SSH server if enabled
        if self.config.server.enable_ssh {
            let mut ssh_server = SshServer::new(self.config.clone(), self.event_sender.clone());

            // Spawn SSH server in background
            let ssh_config = self.config.clone();
            let ssh_event_sender = self.event_sender.clone();
            tokio::spawn(async move {
                let mut server = SshServer::new(ssh_config, ssh_event_sender);
                if let Err(e) = server.start().await {
                    error!("SSH server failed: {}", e);
                }
            });

            self.ssh_server = Some(ssh_server);
        }

        info!("Combined server started");
        Ok(())
    }

    /// Stop all servers
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ref mut http_server) = self.http_server {
            http_server.stop().await?;
        }

        if let Some(ref mut ssh_server) = self.ssh_server {
            ssh_server.stop().await?;
        }

        info!("Combined server stopped");
        Ok(())
    }

    /// Get combined server status
    pub fn status(&self) -> ServerStatus {
        let http_status = self
            .http_server
            .as_ref()
            .map(|s| s.status())
            .unwrap_or(&ServerStatus {
                running: false,
                http_enabled: false,
                ssh_enabled: false,
                connections: 0,
                uptime_seconds: 0,
                last_operation: None,
            });

        let ssh_status = self
            .ssh_server
            .as_ref()
            .map(|s| s.status())
            .unwrap_or(&ServerStatus {
                running: false,
                http_enabled: false,
                ssh_enabled: false,
                connections: 0,
                uptime_seconds: 0,
                last_operation: None,
            });

        ServerStatus {
            running: http_status.running || ssh_status.running,
            http_enabled: http_status.http_enabled,
            ssh_enabled: ssh_status.ssh_enabled,
            connections: http_status.connections + ssh_status.connections,
            uptime_seconds: std::cmp::max(http_status.uptime_seconds, ssh_status.uptime_seconds),
            last_operation: http_status.last_operation.or(ssh_status.last_operation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_upload_pack_request() {
        let body = "want abc123\nhave def456\n0000";
        let refs = parse_upload_pack_request(body);
        assert_eq!(refs, vec!["refs/heads/abc123"]);
    }

    #[test]
    fn test_parse_receive_pack_request() {
        let body = "want abc123\n0000";
        let refs = parse_receive_pack_request(body);
        assert_eq!(refs, vec!["refs/heads/abc123"]);
    }

    #[tokio::test]
    async fn test_http_server_creation() {
        let temp_dir = tempdir().unwrap();
        let mut config = GitProxyConfig::default();
        config.proxy_repo_path = temp_dir.path().join("repo.git");

        let (event_sender, _event_receiver) = mpsc::channel(100);
        let http_server = HttpServer::new(config, event_sender);

        assert!(!http_server.status().running);
        assert!(http_server.status().http_enabled);
        assert!(!http_server.status().ssh_enabled);
    }

    #[tokio::test]
    async fn test_ssh_server_creation() {
        let temp_dir = tempdir().unwrap();
        let mut config = GitProxyConfig::default();
        config.proxy_repo_path = temp_dir.path().join("repo.git");

        let (event_sender, _event_receiver) = mpsc::channel(100);
        let ssh_server = SshServer::new(config, event_sender);

        assert!(!ssh_server.status().running);
        assert!(!ssh_server.status().http_enabled);
        assert!(ssh_server.status().ssh_enabled);
    }
}
