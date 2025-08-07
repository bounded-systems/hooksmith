//! Git proxy synchronization module
//!
//! This module handles periodic synchronization with upstream repositories,
//! monitoring for changes, and maintaining consistency between the proxy
//! and the upstream repository.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Remote, FetchOptions, Cred, Direction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::{interval, Interval};
use tracing::{debug, error, info, warn};

use crate::{GitProxyConfig, GitProxyEvent, ValidationRequest, ValidationResult, ValidationOperationType, ClientInfo};

/// Remote reference state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRef {
    /// Reference name (e.g., "refs/heads/main")
    pub name: String,
    /// Current commit hash
    pub commit_hash: String,
    /// Previous commit hash (for detecting changes)
    pub previous_hash: Option<String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Reference type
    pub ref_type: RefType,
    /// Whether this ref was force-pushed
    pub force_pushed: bool,
    /// Whether this ref was deleted
    pub deleted: bool,
}

/// Reference type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefType {
    Branch,
    Tag,
    PullRequest,
    Other,
}

/// Sync operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether the sync was successful
    pub success: bool,
    /// New references found
    pub new_refs: Vec<RemoteRef>,
    /// Updated references
    pub updated_refs: Vec<RemoteRef>,
    /// Deleted references
    pub deleted_refs: Vec<RemoteRef>,
    /// Force-pushed references
    pub force_pushed_refs: Vec<RemoteRef>,
    /// Error message if sync failed
    pub error: Option<String>,
    /// Duration of the sync operation
    pub duration_ms: u64,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync interval in seconds
    pub interval_seconds: u64,
    /// Enable automatic sync
    pub auto_sync: bool,
    /// Maximum number of retries on failure
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Enable force push detection
    pub detect_force_pushes: bool,
    /// Enable branch deletion detection
    pub detect_deletions: bool,
    /// Enable PR branch tracking
    pub track_pr_branches: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Maximum number of refs to track
    pub max_refs: Option<usize>,
}

/// Remote state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// All remote references
    pub refs: HashMap<String, RemoteRef>,
    /// Repository URL
    pub repository_url: String,
    /// Last fetch timestamp
    pub last_fetch: DateTime<Utc>,
    /// Fetch duration in milliseconds
    pub fetch_duration_ms: u64,
}

/// Synchronization manager
pub struct SyncManager {
    config: SyncConfig,
    proxy_config: GitProxyConfig,
    repository: Option<Repository>,
    last_snapshot: Arc<Mutex<Option<RemoteSnapshot>>>,
    sync_interval: Option<Interval>,
    event_sender: Option<tokio::sync::mpsc::Sender<GitProxyEvent>>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(config: SyncConfig, proxy_config: GitProxyConfig) -> Self {
        Self {
            config,
            proxy_config,
            repository: None,
            last_snapshot: Arc::new(Mutex::new(None)),
            sync_interval: None,
            event_sender: None,
        }
    }
    
    /// Set event sender for notifications
    pub fn set_event_sender(&mut self, sender: tokio::sync::mpsc::Sender<GitProxyEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Initialize the sync manager
    pub async fn initialize(&mut self) -> Result<()> {
        // Open or clone the repository
        self.repository = Some(self.open_or_clone_repository().await?);
        
        // Perform initial sync
        let _ = self.sync_with_upstream().await?;
        
        // Start periodic sync if enabled
        if self.config.auto_sync {
            self.start_periodic_sync();
        }
        
        Ok(())
    }
    
    /// Open or clone the repository
    async fn open_or_clone_repository(&self) -> Result<Repository> {
        let repo_path = &self.proxy_config.proxy_repo_path;
        
        if repo_path.exists() {
            // Open existing repository
            Repository::open(repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))
        } else {
            // Clone from upstream
            self.clone_repository().await
        }
    }
    
    /// Clone repository from upstream
    async fn clone_repository(&self) -> Result<Repository> {
        let repo_path = &self.proxy_config.proxy_repo_path;
        
        // Create parent directory
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Clone options
        let mut fetch_options = FetchOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.proxy_config.auth.github_token {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    token,
                )
            });
        } else if let Some(ssh_key_path) = &self.proxy_config.auth.ssh_key_path {
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
        Repository::clone(&self.proxy_config.upstream_url, repo_path)
            .with_context(|| format!("Failed to clone repository from {}", self.proxy_config.upstream_url))
    }
    
    /// Start periodic synchronization
    fn start_periodic_sync(&mut self) {
        let interval_duration = Duration::from_secs(self.config.interval_seconds);
        let mut interval = interval(interval_duration);
        
        let config = self.config.clone();
        let proxy_config = self.proxy_config.clone();
        let last_snapshot = Arc::clone(&self.last_snapshot);
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                let mut sync_manager = SyncManager {
                    config: config.clone(),
                    proxy_config: proxy_config.clone(),
                    repository: None,
                    last_snapshot: Arc::clone(&last_snapshot),
                    sync_interval: None,
                    event_sender: event_sender.clone(),
                };
                
                if let Err(e) = sync_manager.sync_with_upstream().await {
                    error!("Periodic sync failed: {}", e);
                }
            }
        });
        
        self.sync_interval = Some(interval);
    }
    
    /// Synchronize with upstream repository
    pub async fn sync_with_upstream(&mut self) -> Result<SyncResult> {
        let start_time = Instant::now();
        let timestamp = Utc::now();
        
        // Open repository if not already open
        if self.repository.is_none() {
            self.repository = Some(self.open_or_clone_repository().await?);
        }
        
        let repo = self.repository.as_ref().unwrap();
        
        // Fetch from upstream
        let fetch_result = self.fetch_from_upstream(repo).await?;
        
        // Get current remote state
        let current_snapshot = self.create_remote_snapshot(repo).await?;
        
        // Compare with previous snapshot
        let sync_result = self.compare_snapshots(&current_snapshot).await?;
        
        // Update last snapshot
        {
            let mut last_snapshot = self.last_snapshot.lock().unwrap();
            *last_snapshot = Some(current_snapshot);
        }
        
        // Send events for significant changes
        self.send_sync_events(&sync_result).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(SyncResult {
            timestamp,
            success: sync_result.success,
            new_refs: sync_result.new_refs,
            updated_refs: sync_result.updated_refs,
            deleted_refs: sync_result.deleted_refs,
            force_pushed_refs: sync_result.force_pushed_refs,
            error: sync_result.error,
            duration_ms,
        })
    }
    
    /// Fetch from upstream repository
    async fn fetch_from_upstream(&self, repo: &Repository) -> Result<()> {
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", &self.proxy_config.upstream_url))?;
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        
        // Set up authentication
        if let Some(token) = &self.proxy_config.auth.github_token {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    token,
                )
            });
        } else if let Some(ssh_key_path) = &self.proxy_config.auth.ssh_key_path {
            fetch_options.credentials(|_url, username_from_url, _allowed_types| {
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    ssh_key_path,
                    None,
                )
            });
        }
        
        // Fetch from remote
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)?;
        
        info!("Successfully fetched from upstream");
        Ok(())
    }
    
    /// Create a snapshot of remote references
    async fn create_remote_snapshot(&self, repo: &Repository) -> Result<RemoteSnapshot> {
        let start_time = Instant::now();
        let timestamp = Utc::now();
        
        let mut refs = HashMap::new();
        
        // Get all remote references
        let references = repo.references()?;
        
        for reference in references {
            let reference = reference?;
            let name = reference.name().unwrap_or("").to_string();
            
            // Only track remote references
            if name.starts_with("refs/remotes/origin/") {
                let ref_type = if name.contains("refs/remotes/origin/refs/pull/") {
                    RefType::PullRequest
                } else if name.ends_with("/tags/") {
                    RefType::Tag
                } else {
                    RefType::Branch
                };
                
                let commit_hash = reference.target().unwrap().to_string();
                
                refs.insert(name.clone(), RemoteRef {
                    name,
                    commit_hash,
                    previous_hash: None, // Will be set during comparison
                    last_updated: timestamp,
                    ref_type,
                    force_pushed: false,
                    deleted: false,
                });
            }
        }
        
        let fetch_duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(RemoteSnapshot {
            timestamp,
            refs,
            repository_url: self.proxy_config.upstream_url.clone(),
            last_fetch: timestamp,
            fetch_duration_ms,
        })
    }
    
    /// Compare current snapshot with previous snapshot
    async fn compare_snapshots(&self, current: &RemoteSnapshot) -> Result<SyncResult> {
        let mut new_refs = Vec::new();
        let mut updated_refs = Vec::new();
        let mut deleted_refs = Vec::new();
        let mut force_pushed_refs = Vec::new();
        
        let previous_snapshot = {
            let last_snapshot = self.last_snapshot.lock().unwrap();
            last_snapshot.clone()
        };
        
        if let Some(previous) = previous_snapshot {
            // Compare current refs with previous refs
            for (name, current_ref) in &current.refs {
                if let Some(previous_ref) = previous.refs.get(name) {
                    if current_ref.commit_hash != previous_ref.commit_hash {
                        // Reference was updated
                        let mut updated_ref = current_ref.clone();
                        updated_ref.previous_hash = Some(previous_ref.commit_hash.clone());
                        
                        // Check if this was a force push
                        if self.config.detect_force_pushes {
                            // A force push typically means the new commit is not a descendant of the old one
                            if let Ok(repo) = self.repository.as_ref().unwrap().clone() {
                                if let Ok(merge_base) = repo.merge_base(
                                    &repo.find_commit(git2::Oid::from_str(&current_ref.commit_hash)?)?,
                                    &repo.find_commit(git2::Oid::from_str(&previous_ref.commit_hash)?)?
                                ) {
                                    if merge_base.to_string() != previous_ref.commit_hash {
                                        updated_ref.force_pushed = true;
                                        force_pushed_refs.push(updated_ref.clone());
                                    }
                                }
                            }
                        }
                        
                        updated_refs.push(updated_ref);
                    }
                } else {
                    // New reference
                    new_refs.push(current_ref.clone());
                }
            }
            
            // Check for deleted references
            if self.config.detect_deletions {
                for (name, _) in &previous.refs {
                    if !current.refs.contains_key(name) {
                        deleted_refs.push(RemoteRef {
                            name: name.clone(),
                            commit_hash: "".to_string(),
                            previous_hash: None,
                            last_updated: current.timestamp,
                            ref_type: RefType::Branch,
                            force_pushed: false,
                            deleted: true,
                        });
                    }
                }
            }
        } else {
            // First sync - all refs are new
            new_refs.extend(current.refs.values().cloned());
        }
        
        let success = true;
        let error = None;
        
        Ok(SyncResult {
            timestamp: current.timestamp,
            success,
            new_refs,
            updated_refs,
            deleted_refs,
            force_pushed_refs,
            error,
            duration_ms: 0,
        })
    }
    
    /// Send sync events for significant changes
    async fn send_sync_events(&self, sync_result: &SyncResult) -> Result<()> {
        if let Some(sender) = &self.event_sender {
            // Send validation events for force pushes
            for force_pushed_ref in &sync_result.force_pushed_refs {
                let validation_request = ValidationRequest {
                    request_id: format!("sync-force-push-{}", Utc::now().timestamp()),
                    operation_type: ValidationOperationType::PrePush,
                    refs: vec![force_pushed_ref.name.clone()],
                    commit_hashes: vec![force_pushed_ref.commit_hash.clone()],
                    client_info: ClientInfo {
                        user_agent: "git-proxy-sync".to_string(),
                        client_ip: "127.0.0.1".to_string(),
                        protocol: crate::GitProtocol::Https,
                        capabilities: vec!["force-push-detection".to_string()],
                    },
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                };
                
                let _ = sender.send(GitProxyEvent::ValidationRequest(validation_request)).await;
            }
            
            // Log significant changes
            if !sync_result.new_refs.is_empty() {
                info!("Sync detected {} new references", sync_result.new_refs.len());
            }
            
            if !sync_result.force_pushed_refs.is_empty() {
                warn!("Sync detected {} force-pushed references", sync_result.force_pushed_refs.len());
            }
            
            if !sync_result.deleted_refs.is_empty() {
                warn!("Sync detected {} deleted references", sync_result.deleted_refs.len());
            }
        }
        
        Ok(())
    }
    
    /// Get the current sync status
    pub fn get_sync_status(&self) -> SyncStatus {
        let last_snapshot = {
            let snapshot = self.last_snapshot.lock().unwrap();
            snapshot.clone()
        };
        
        SyncStatus {
            last_sync: last_snapshot.map(|s| s.timestamp),
            auto_sync_enabled: self.config.auto_sync,
            sync_interval_seconds: self.config.interval_seconds,
            total_refs: last_snapshot.map(|s| s.refs.len()).unwrap_or(0),
            repository_url: self.proxy_config.upstream_url.clone(),
        }
    }
    
    /// Force a manual sync
    pub async fn force_sync(&mut self) -> Result<SyncResult> {
        self.sync_with_upstream().await
    }
    
    /// Stop periodic synchronization
    pub fn stop_periodic_sync(&mut self) {
        self.sync_interval = None;
    }
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Whether auto-sync is enabled
    pub auto_sync_enabled: bool,
    /// Sync interval in seconds
    pub sync_interval_seconds: u64,
    /// Total number of tracked references
    pub total_refs: usize,
    /// Repository URL
    pub repository_url: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 300, // 5 minutes
            auto_sync: true,
            max_retries: 3,
            retry_delay_seconds: 60,
            detect_force_pushes: true,
            detect_deletions: true,
            track_pr_branches: true,
            audit_logging: true,
            max_refs: Some(1000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.interval_seconds, 300);
        assert!(config.auto_sync);
        assert!(config.detect_force_pushes);
        assert!(config.detect_deletions);
    }
    
    #[tokio::test]
    async fn test_sync_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let proxy_config = GitProxyConfig::default();
        let sync_config = SyncConfig::default();
        
        let sync_manager = SyncManager::new(sync_config, proxy_config);
        assert!(sync_manager.repository.is_none());
        assert!(sync_manager.sync_interval.is_none());
    }
    
    #[test]
    fn test_remote_ref_creation() {
        let ref_info = RemoteRef {
            name: "refs/remotes/origin/main".to_string(),
            commit_hash: "abc123".to_string(),
            previous_hash: None,
            last_updated: Utc::now(),
            ref_type: RefType::Branch,
            force_pushed: false,
            deleted: false,
        };
        
        assert_eq!(ref_info.name, "refs/remotes/origin/main");
        assert_eq!(ref_info.commit_hash, "abc123");
        assert!(!ref_info.force_pushed);
    }
}
