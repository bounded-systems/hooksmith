//! Git protocol handling for the proxy
//!
//! This module handles Git protocol operations including HTTP Smart Protocol
//! and SSH protocol support for the Git proxy.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Remote, FetchOptions, PushOptions, Cred, Direction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

use crate::{
    GitProxyConfig, GitProxyEvent, GitPushRequest, GitPushResult, GitPullRequest, GitPullResult,
    GitFetchRequest, GitFetchResult, ClientInfo, GitProtocol,
};

/// Git protocol handler trait
pub trait GitProtocolHandler {
    /// Handle a push operation
    async fn handle_push(&self, request: GitPushRequest) -> Result<GitPushResult>;
    
    /// Handle a pull operation
    async fn handle_pull(&self, request: GitPullRequest) -> Result<GitPullResult>;
    
    /// Handle a fetch operation
    async fn handle_fetch(&self, request: GitFetchRequest) -> Result<GitFetchResult>;
    
    /// Get protocol information
    fn protocol_info(&self) -> ProtocolInfo;
}

/// Protocol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    /// Protocol name
    pub name: String,
    /// Protocol version
    pub version: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Authentication methods
    pub auth_methods: Vec<String>,
}

/// HTTP Smart Protocol handler
pub struct HttpSmartProtocol {
    config: GitProxyConfig,
    repository: Option<Repository>,
}

impl HttpSmartProtocol {
    /// Create a new HTTP Smart Protocol handler
    pub fn new(config: GitProxyConfig) -> Self {
        Self {
            config,
            repository: None,
        }
    }
    
    /// Initialize the repository
    pub async fn initialize(&mut self) -> Result<()> {
        self.repository = Some(self.open_repository().await?);
        Ok(())
    }
    
    /// Open the repository
    async fn open_repository(&self) -> Result<Repository> {
        let repo_path = &self.config.proxy_repo_path;
        
        if repo_path.exists() {
            Repository::open(repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))
        } else {
            self.clone_repository().await
        }
    }
    
    /// Clone repository from upstream
    async fn clone_repository(&self) -> Result<Repository> {
        let repo_path = &self.config.proxy_repo_path;
        
        // Create parent directory
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Clone options
        let mut fetch_options = FetchOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.config.auth.github_token {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    token,
                )
            });
        }
        
        // Clone the repository
        Repository::clone(&self.config.upstream_url, repo_path)
            .with_context(|| format!("Failed to clone repository from {}", self.config.upstream_url))
    }
    
    /// Forward push to upstream
    async fn forward_push_to_upstream(&self, request: &GitPushRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up push options
        let mut push_options = PushOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.config.auth.github_token {
            push_options.remote_callbacks(|callbacks| {
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    Cred::userpass_plaintext(
                        username_from_url.unwrap_or("git"),
                        token,
                    )
                });
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Push refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.push(&refspecs, Some(&mut push_options))?;
        
        info!("Successfully pushed {} refs to upstream", request.refs.len());
        Ok(true)
    }
    
    /// Forward pull from upstream
    async fn forward_pull_from_upstream(&self, request: &GitPullRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.config.auth.github_token {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    token,
                )
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Fetch refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.fetch(&refspecs, Some(&mut fetch_options), None)?;
        
        info!("Successfully pulled {} refs from upstream", request.refs.len());
        Ok(true)
    }
    
    /// Forward fetch from upstream
    async fn forward_fetch_from_upstream(&self, request: &GitFetchRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.config.auth.github_token {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    token,
                )
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Fetch refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.fetch(&refspecs, Some(&mut fetch_options), None)?;
        
        info!("Successfully fetched {} refs from upstream", request.refs.len());
        Ok(true)
    }
}

impl GitProtocolHandler for HttpSmartProtocol {
    async fn handle_push(&self, request: GitPushRequest) -> Result<GitPushResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_push_to_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitPushResult {
            request_id: request.request_id,
            success,
            refs_pushed: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_pull(&self, request: GitPullRequest) -> Result<GitPullResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_pull_from_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitPullResult {
            request_id: request.request_id,
            success,
            refs_pulled: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_fetch(&self, request: GitFetchRequest) -> Result<GitFetchResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_fetch_from_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitFetchResult {
            request_id: request.request_id,
            success,
            refs_fetched: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    fn protocol_info(&self) -> ProtocolInfo {
        ProtocolInfo {
            name: "HTTP Smart Protocol".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![
                "multi_ack".to_string(),
                "side-band-64k".to_string(),
                "thin-pack".to_string(),
                "ofs-delta".to_string(),
            ],
            auth_methods: vec![
                "basic".to_string(),
                "token".to_string(),
            ],
        }
    }
}

/// SSH Protocol handler
pub struct SshProtocol {
    config: GitProxyConfig,
    repository: Option<Repository>,
}

impl SshProtocol {
    /// Create a new SSH Protocol handler
    pub fn new(config: GitProxyConfig) -> Self {
        Self {
            config,
            repository: None,
        }
    }
    
    /// Initialize the repository
    pub async fn initialize(&mut self) -> Result<()> {
        self.repository = Some(self.open_repository().await?);
        Ok(())
    }
    
    /// Open the repository
    async fn open_repository(&self) -> Result<Repository> {
        let repo_path = &self.config.proxy_repo_path;
        
        if repo_path.exists() {
            Repository::open(repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))
        } else {
            self.clone_repository().await
        }
    }
    
    /// Clone repository from upstream
    async fn clone_repository(&self) -> Result<Repository> {
        let repo_path = &self.config.proxy_repo_path;
        
        // Create parent directory
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Clone options
        let mut fetch_options = FetchOptions::new();
        
        // Set up SSH authentication
        if let Some(ssh_key_path) = &self.config.auth.ssh_key_path {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    ssh_key_path,
                    None,
                )
            });
        }
        
        // Clone the repository
        Repository::clone(&self.config.upstream_url, repo_path)
            .with_context(|| format!("Failed to clone repository from {}", self.config.upstream_url))
    }
    
    /// Forward push to upstream using SSH
    async fn forward_push_to_upstream(&self, request: &GitPushRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up push options
        let mut push_options = PushOptions::new();
        
        // Set up SSH authentication
        if let Some(ssh_key_path) = &self.config.auth.ssh_key_path {
            push_options.remote_callbacks(|callbacks| {
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    Cred::ssh_key(
                        username_from_url.unwrap_or("git"),
                        None,
                        ssh_key_path,
                        None,
                    )
                });
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Push refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.push(&refspecs, Some(&mut push_options))?;
        
        info!("Successfully pushed {} refs to upstream via SSH", request.refs.len());
        Ok(true)
    }
    
    /// Forward pull from upstream using SSH
    async fn forward_pull_from_upstream(&self, request: &GitPullRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        
        // Set up SSH authentication
        if let Some(ssh_key_path) = &self.config.auth.ssh_key_path {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    ssh_key_path,
                    None,
                )
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Fetch refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.fetch(&refspecs, Some(&mut fetch_options), None)?;
        
        info!("Successfully pulled {} refs from upstream via SSH", request.refs.len());
        Ok(true)
    }
    
    /// Forward fetch from upstream using SSH
    async fn forward_fetch_from_upstream(&self, request: &GitFetchRequest) -> Result<bool> {
        let repo = self.repository.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        
        // Set up SSH authentication
        if let Some(ssh_key_path) = &self.config.auth.ssh_key_path {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    ssh_key_path,
                    None,
                )
            });
        }
        
        // Get remote
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.config.upstream_url))?;
        
        // Fetch refs
        let refspecs: Vec<&str> = request.refs.iter().map(|r| r.as_str()).collect();
        remote.fetch(&refspecs, Some(&mut fetch_options), None)?;
        
        info!("Successfully fetched {} refs from upstream via SSH", request.refs.len());
        Ok(true)
    }
}

impl GitProtocolHandler for SshProtocol {
    async fn handle_push(&self, request: GitPushRequest) -> Result<GitPushResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_push_to_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitPushResult {
            request_id: request.request_id,
            success,
            refs_pushed: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_pull(&self, request: GitPullRequest) -> Result<GitPullResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_pull_from_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitPullResult {
            request_id: request.request_id,
            success,
            refs_pulled: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_fetch(&self, request: GitFetchRequest) -> Result<GitFetchResult> {
        let start_time = std::time::Instant::now();
        
        // Forward to upstream
        let success = self.forward_fetch_from_upstream(&request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(GitFetchResult {
            request_id: request.request_id,
            success,
            refs_fetched: if success { request.refs } else { vec![] },
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        })
    }
    
    fn protocol_info(&self) -> ProtocolInfo {
        ProtocolInfo {
            name: "SSH Protocol".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![
                "multi_ack".to_string(),
                "side-band-64k".to_string(),
                "thin-pack".to_string(),
                "ofs-delta".to_string(),
            ],
            auth_methods: vec![
                "ssh-key".to_string(),
                "ssh-agent".to_string(),
            ],
        }
    }
}

/// Protocol factory
pub struct ProtocolFactory;

impl ProtocolFactory {
    /// Create a protocol handler based on configuration
    pub fn create_handler(config: GitProxyConfig) -> Box<dyn GitProtocolHandler> {
        let upstream_url = &config.upstream_url;
        
        if upstream_url.starts_with("ssh://") || upstream_url.starts_with("git@") {
            Box::new(SshProtocol::new(config))
        } else {
            Box::new(HttpSmartProtocol::new(config))
        }
    }
    
    /// Detect protocol from URL
    pub fn detect_protocol(url: &str) -> GitProtocol {
        if url.starts_with("ssh://") || url.starts_with("git@") {
            GitProtocol::Ssh
        } else if url.starts_with("https://") {
            GitProtocol::Https
        } else if url.starts_with("http://") {
            GitProtocol::Http
        } else {
            GitProtocol::Git
        }
    }
}

/// Git command wrapper for protocol operations
pub struct GitCommandWrapper {
    config: GitProxyConfig,
}

impl GitCommandWrapper {
    /// Create a new Git command wrapper
    pub fn new(config: GitProxyConfig) -> Self {
        Self { config }
    }
    
    /// Execute a Git command
    pub async fn execute_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.config.proxy_repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute git command: {:?}", args))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Git command failed: {}", stderr))
        }
    }
    
    /// Push to upstream using git command
    pub async fn push_to_upstream(&self, refs: &[String]) -> Result<bool> {
        let mut args = vec!["push", "origin"];
        args.extend(refs.iter().map(|r| r.as_str()));
        
        self.execute_git_command(&args).await?;
        Ok(true)
    }
    
    /// Pull from upstream using git command
    pub async fn pull_from_upstream(&self, refs: &[String]) -> Result<bool> {
        let mut args = vec!["pull", "origin"];
        args.extend(refs.iter().map(|r| r.as_str()));
        
        self.execute_git_command(&args).await?;
        Ok(true)
    }
    
    /// Fetch from upstream using git command
    pub async fn fetch_from_upstream(&self, refs: &[String]) -> Result<bool> {
        let mut args = vec!["fetch", "origin"];
        args.extend(refs.iter().map(|r| r.as_str()));
        
        self.execute_git_command(&args).await?;
        Ok(true)
    }
    
    /// Get remote information
    pub async fn get_remote_info(&self) -> Result<RemoteInfo> {
        let output = self.execute_git_command(&["remote", "-v"]).await?;
        
        let mut remotes = HashMap::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                remotes.insert(name, url);
            }
        }
        
        Ok(RemoteInfo {
            remotes,
            upstream_url: self.config.upstream_url.clone(),
        })
    }
}

/// Remote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    /// Remote name to URL mapping
    pub remotes: HashMap<String, String>,
    /// Upstream URL
    pub upstream_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_protocol_detection() {
        assert_eq!(ProtocolFactory::detect_protocol("ssh://git@github.com/user/repo.git"), GitProtocol::Ssh);
        assert_eq!(ProtocolFactory::detect_protocol("git@github.com:user/repo.git"), GitProtocol::Ssh);
        assert_eq!(ProtocolFactory::detect_protocol("https://github.com/user/repo.git"), GitProtocol::Https);
        assert_eq!(ProtocolFactory::detect_protocol("http://github.com/user/repo.git"), GitProtocol::Http);
    }
    
    #[tokio::test]
    async fn test_http_protocol_handler() {
        let temp_dir = tempdir().unwrap();
        let mut config = GitProxyConfig::default();
        config.proxy_repo_path = temp_dir.path().join("repo.git");
        config.upstream_url = "https://github.com/user/repo.git".to_string();
        
        let mut handler = HttpSmartProtocol::new(config);
        assert!(handler.initialize().await.is_ok());
        
        let info = handler.protocol_info();
        assert_eq!(info.name, "HTTP Smart Protocol");
        assert!(info.capabilities.contains(&"multi_ack".to_string()));
    }
    
    #[tokio::test]
    async fn test_ssh_protocol_handler() {
        let temp_dir = tempdir().unwrap();
        let mut config = GitProxyConfig::default();
        config.proxy_repo_path = temp_dir.path().join("repo.git");
        config.upstream_url = "ssh://git@github.com/user/repo.git".to_string();
        
        let mut handler = SshProtocol::new(config);
        assert!(handler.initialize().await.is_ok());
        
        let info = handler.protocol_info();
        assert_eq!(info.name, "SSH Protocol");
        assert!(info.auth_methods.contains(&"ssh-key".to_string()));
    }
}
