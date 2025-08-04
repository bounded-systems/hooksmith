//! Event handler integration for Git operations

use super::*;
use anyhow::Result;
use serde_json::json;

// TODO: Define these types locally until xtask integration is complete
pub trait EventHandler {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()>;
    fn name(&self) -> &str;
    fn should_handle(&self, event: &HooksmithEvent) -> bool;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HooksmithEvent {
    pub id: String,
    pub event: String,
    pub context: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Git operations event handler
pub struct GitOperationsEventHandler {
    handler: GitOperationsHandler,
}

impl GitOperationsEventHandler {
    /// Create a new Git operations event handler
    pub fn new(working_directory: PathBuf, session_id: String) -> Self {
        Self {
            handler: GitOperationsHandler::new(working_directory, session_id),
        }
    }

    /// Convert a HooksmithEvent to a GitOperationEvent
    fn parse_event(&self, event: &HooksmithEvent) -> Result<GitOperationEvent> {
        let event_type = event.event.as_str();
        let context = &event.context;
        
        match event_type {
            "git_commit_request" => {
                let request = GitCommitRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    message: context.get("message")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing message in git_commit_request"))?
                        .to_string(),
                    files: context.get("files")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()),
                    author: context.get("author")
                        .and_then(|v| v.as_object())
                        .map(|obj| GitAuthor {
                            name: obj.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string(),
                            email: obj.get("email")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown@example.com")
                                .to_string(),
                        }),
                    allow_empty: context.get("allow_empty")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitCommitRequest(request))
            }
            "git_push_request" => {
                let request = GitPushRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    remote: context.get("remote")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    branch: context.get("branch")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    force: context.get("force")
                        .and_then(|v| v.as_bool()),
                    tags: context.get("tags")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitPushRequest(request))
            }
            "git_pull_request" => {
                let request = GitPullRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    remote: context.get("remote")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    branch: context.get("branch")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    rebase: context.get("rebase")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitPullRequest(request))
            }
            "git_status_request" => {
                let request = GitStatusRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitStatusRequest(request))
            }
            "git_add_request" => {
                let files = context.get("files")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing files in git_add_request"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                let request = GitAddRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    files,
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitAddRequest(request))
            }
            "git_note_add_request" => {
                let request = GitNoteAddRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    object: context.get("object")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing object in git_note_add_request"))?
                        .to_string(),
                    message: context.get("message")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    file: context.get("file")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitNoteAddRequest(request))
            }
            "git_note_get_request" => {
                let request = GitNoteGetRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    object: context.get("object")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing object in git_note_get_request"))?
                        .to_string(),
                    metadata: self.parse_metadata(context),
                };
                Ok(GitOperationEvent::GitNoteGetRequest(request))
            }
            _ => Err(anyhow::anyhow!("Unknown Git operation event type: {}", event_type)),
        }
    }

    /// Convert a GitOperationEvent back to HooksmithEvent
    fn create_result_event(&self, git_event: GitOperationEvent, original_event: &HooksmithEvent) -> HooksmithEvent {
        let (event_type, context) = match git_event {
            GitOperationEvent::GitCommitResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(commit_hash) = result.commit_hash {
                    context["commit_hash"] = json!(commit_hash);
                }
                if let Some(files_changed) = result.files_changed {
                    context["files_changed"] = json!(files_changed);
                }
                if let Some(branch) = result.branch {
                    context["branch"] = json!(branch);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_commit_result", context)
            }
            GitOperationEvent::GitPushResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(output) = result.output {
                    context["output"] = json!(output);
                }
                if let Some(branch) = result.branch {
                    context["branch"] = json!(branch);
                }
                if let Some(remote) = result.remote {
                    context["remote"] = json!(remote);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_push_result", context)
            }
            GitOperationEvent::GitPullResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(output) = result.output {
                    context["output"] = json!(output);
                }
                if let Some(branch) = result.branch {
                    context["branch"] = json!(branch);
                }
                if let Some(remote) = result.remote {
                    context["remote"] = json!(remote);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_pull_result", context)
            }
            GitOperationEvent::GitStatusResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(status) = result.status {
                    context["status"] = json!({
                        "staged": status.staged,
                        "unstaged": status.unstaged,
                        "untracked": status.untracked,
                        "modified": status.modified,
                        "deleted": status.deleted,
                        "renamed": status.renamed,
                    });
                }
                if let Some(branch) = result.branch {
                    context["branch"] = json!(branch);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_status_result", context)
            }
            GitOperationEvent::GitAddResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(files_added) = result.files_added {
                    context["files_added"] = json!(files_added);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_add_result", context)
            }
            GitOperationEvent::GitNoteAddResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(note_id) = result.note_id {
                    context["note_id"] = json!(note_id);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_note_add_result", context)
            }
            GitOperationEvent::GitNoteGetResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(note_content) = result.note_content {
                    context["note_content"] = json!(note_content);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                if let Some(error) = result.error {
                    context["error"] = json!({
                        "code": error.code,
                        "message": error.message,
                        "details": error.details,
                    });
                }
                
                ("git_note_get_result", context)
            }
            _ => {
                // Handle request events by returning the original event
                return original_event.clone();
            }
        };
        
        HooksmithEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event: event_type.to_string(),
            context,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Parse metadata from context
    fn parse_metadata(&self, context: &serde_json::Value) -> Option<EventMetadata> {
        let working_directory = context.get("working_directory")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let repository = context.get("repository")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let timestamp = context.get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let session_id = context.get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        if working_directory.is_some() || repository.is_some() || timestamp.is_some() || session_id.is_some() {
            Some(EventMetadata {
                working_directory,
                repository,
                timestamp,
                session_id,
            })
        } else {
            None
        }
    }
}

impl EventHandler for GitOperationsEventHandler {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()> {
        let runtime = tokio::runtime::Runtime::new()?;
        if !self.should_handle(event) {
            return Ok(());
        }
        
        // Parse the event
        let git_event = self.parse_event(event)?;
        
        // Handle the Git operation
        let result_event = match git_event {
            GitOperationEvent::GitCommitRequest(req) => {
                self.handler.handle_git_commit(req).await?
            }
            GitOperationEvent::GitPushRequest(req) => {
                self.handler.handle_git_push(req).await?
            }
            GitOperationEvent::GitPullRequest(req) => {
                self.handler.handle_git_pull(req).await?
            }
            GitOperationEvent::GitStatusRequest(req) => {
                self.handler.handle_git_status(req).await?
            }
            GitOperationEvent::GitAddRequest(req) => {
                self.handler.handle_git_add(req).await?
            }
            GitOperationEvent::GitNoteAddRequest(req) => {
                self.handler.handle_git_note_add(req).await?
            }
            GitOperationEvent::GitNoteGetRequest(req) => {
                self.handler.handle_git_note_get(req).await?
            }
            _ => return Ok(()), // Skip result events
        };
        
        // Convert result back to HooksmithEvent and emit it
        let result_hooksmith_event = self.create_result_event(result_event, event);
        // TODO: Implement event emission when xtask integration is complete
        // crate::xtask::event_bus::emit_event(result_hooksmith_event)?;
        
        Ok(())
    }

    fn name(&self) -> &str {
        "git-operations"
    }

    fn should_handle(&self, event: &HooksmithEvent) -> bool {
        matches!(event.event.as_str(), 
            "git_commit_request" | 
            "git_push_request" | 
            "git_pull_request" | 
            "git_status_request" | 
            "git_add_request" | 
            "git_note_add_request" | 
            "git_note_get_request"
        )
    }
} 
