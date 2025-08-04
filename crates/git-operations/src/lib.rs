//! Git Operations Handler for Hooksmith Hybrid Architecture
//!
//! This crate provides native Git repository operations as event handlers
//! for the hybrid WIT + native Rust architecture. It handles all Git operations
//! that require system privileges.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;

pub mod operations;
pub mod event_handler;
pub mod schema;

/// Git operation event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitOperationEvent {
    GitCommitRequest(GitCommitRequest),
    GitCommitResult(GitCommitResult),
    GitPushRequest(GitPushRequest),
    GitPushResult(GitPushResult),
    GitPullRequest(GitPullRequest),
    GitPullResult(GitPullResult),
    GitStatusRequest(GitStatusRequest),
    GitStatusResult(GitStatusResult),
    GitAddRequest(GitAddRequest),
    GitAddResult(GitAddResult),
    GitNoteAddRequest(GitNoteAddRequest),
    GitNoteAddResult(GitNoteAddResult),
    GitNoteGetRequest(GitNoteGetRequest),
    GitNoteGetResult(GitNoteGetResult),
}

/// Git commit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitRequest {
    pub request_id: String,
    pub message: String,
    pub files: Option<Vec<String>>,
    pub author: Option<GitAuthor>,
    pub allow_empty: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// Git commit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitResult {
    pub request_id: String,
    pub success: bool,
    pub commit_hash: Option<String>,
    pub files_changed: Option<Vec<String>>,
    pub branch: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git push request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPushRequest {
    pub request_id: String,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub force: Option<bool>,
    pub tags: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// Git push result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPushResult {
    pub request_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPullRequest {
    pub request_id: String,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub rebase: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// Git pull result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPullResult {
    pub request_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git status request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusRequest {
    pub request_id: String,
    pub metadata: Option<EventMetadata>,
}

/// Git status result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub request_id: String,
    pub success: bool,
    pub status: Option<GitStatus>,
    pub branch: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git add request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAddRequest {
    pub request_id: String,
    pub files: Vec<String>,
    pub metadata: Option<EventMetadata>,
}

/// Git add result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAddResult {
    pub request_id: String,
    pub success: bool,
    pub files_added: Option<Vec<String>>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git note add request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitNoteAddRequest {
    pub request_id: String,
    pub object: String,
    pub message: Option<String>,
    pub file: Option<String>,
    pub metadata: Option<EventMetadata>,
}

/// Git note add result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitNoteAddResult {
    pub request_id: String,
    pub success: bool,
    pub note_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git note get request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitNoteGetRequest {
    pub request_id: String,
    pub object: String,
    pub metadata: Option<EventMetadata>,
}

/// Git note get result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitNoteGetResult {
    pub request_id: String,
    pub success: bool,
    pub note_content: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<GitOperationError>,
}

/// Git author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAuthor {
    pub name: String,
    pub email: String,
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub staged: Vec<String>,
    pub unstaged: Vec<String>,
    pub untracked: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub renamed: Vec<String>,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub working_directory: Option<String>,
    pub repository: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub session_id: Option<String>,
}

/// Git operation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
}

/// Git operations handler
pub struct GitOperationsHandler {
    working_directory: PathBuf,
    session_id: String,
    repository: Option<Repository>,
}

impl GitOperationsHandler {
    /// Create a new Git operations handler
    pub fn new(working_directory: PathBuf, session_id: String) -> Self {
        Self {
            working_directory,
            session_id,
            repository: None,
        }
    }

    /// Handle a Git operation event
    pub async fn handle_event(&mut self, event: GitOperationEvent) -> Result<GitOperationEvent> {
        let start_time = Instant::now();
        
        // Ensure repository is initialized
        if self.repository.is_none() {
            self.repository = Some(self.open_repository()?);
        }
        
        let result = match event {
            GitOperationEvent::GitCommitRequest(req) => {
                self.handle_git_commit(req).await?
            }
            GitOperationEvent::GitPushRequest(req) => {
                self.handle_git_push(req).await?
            }
            GitOperationEvent::GitPullRequest(req) => {
                self.handle_git_pull(req).await?
            }
            GitOperationEvent::GitStatusRequest(req) => {
                self.handle_git_status(req).await?
            }
            GitOperationEvent::GitAddRequest(req) => {
                self.handle_git_add(req).await?
            }
            GitOperationEvent::GitNoteAddRequest(req) => {
                self.handle_git_note_add(req).await?
            }
            GitOperationEvent::GitNoteGetRequest(req) => {
                self.handle_git_note_get(req).await?
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported event type"));
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Add duration to result
        let result = match result {
            GitOperationEvent::GitCommitResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitCommitResult(res)
            }
            GitOperationEvent::GitPushResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitPushResult(res)
            }
            GitOperationEvent::GitPullResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitPullResult(res)
            }
            GitOperationEvent::GitStatusResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitStatusResult(res)
            }
            GitOperationEvent::GitAddResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitAddResult(res)
            }
            GitOperationEvent::GitNoteAddResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitNoteAddResult(res)
            }
            GitOperationEvent::GitNoteGetResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                GitOperationEvent::GitNoteGetResult(res)
            }
            _ => result,
        };

        Ok(result)
    }

    /// Open or initialize Git repository
    fn open_repository(&self) -> Result<Repository> {
        // Try to open existing repository
        match Repository::open(&self.working_directory) {
            Ok(repo) => Ok(repo),
            Err(_) => {
                // Initialize new repository if it doesn't exist
                Repository::init(&self.working_directory)
                    .with_context(|| format!("Failed to initialize Git repository in: {}", self.working_directory.display()))
            }
        }
    }

    /// Get current branch name
    fn get_current_branch(&self) -> Result<String> {
        let repo = self.repository.as_ref().ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        let head = repo.head()?;
        let branch_name = head.shorthand()
            .ok_or_else(|| anyhow::anyhow!("Could not get branch name"))?;
        
        Ok(branch_name.to_string())
    }

    /// Get default signature
    fn get_default_signature(&self) -> Result<Signature<'static>> {
        let repo = self.repository.as_ref().ok_or_else(|| anyhow::anyhow!("Repository not initialized"))?;
        
        repo.signature()
            .or_else(|_| {
                // Fallback to default signature
                Signature::now("Hooksmith", "hooksmith@example.com")
            })
            .with_context(|| "Failed to get Git signature")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_git_operations_handler() {
        let temp_dir = TempDir::new().unwrap();
        let mut handler = GitOperationsHandler::new(
            temp_dir.path().to_path_buf(),
            "test-session".to_string(),
        );

        // Test Git status
        let status_req = GitStatusRequest {
            request_id: Uuid::new_v4().to_string(),
            metadata: None,
        };

        let status_result = handler.handle_event(GitOperationEvent::GitStatusRequest(status_req)).await.unwrap();
        assert!(matches!(status_result, GitOperationEvent::GitStatusResult(_)));
    }
} 
