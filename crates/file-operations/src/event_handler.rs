//! Event handler integration for file operations

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

impl HooksmithEvent {
    pub fn new(id: String, event: String, context: serde_json::Value) -> Self {
        Self {
            id,
            event,
            context,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        // Add session_id to context
        if let Some(obj) = self.context.as_object_mut() {
            obj.insert("session_id".to_string(), serde_json::Value::String(session_id));
        }
        self
    }
}

/// File operations event handler
pub struct FileOperationsEventHandler {
    handler: FileOperationsHandler,
}

impl FileOperationsEventHandler {
    /// Create a new file operations event handler
    pub fn new(working_directory: PathBuf, session_id: String) -> Self {
        Self {
            handler: FileOperationsHandler::new(working_directory, session_id),
        }
    }

    /// Convert a HooksmithEvent to a FileOperationEvent
    fn parse_event(&self, event: &HooksmithEvent) -> Result<FileOperationEvent> {
        let event_type = event.event.as_str();
        let context = &event.context;
        
        match event_type {
            "file_read_request" => {
                let request = FileReadRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in file_read_request"))?
                        .to_string(),
                    encoding: context.get("encoding")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileReadRequest(request))
            }
            "file_write_request" => {
                let request = FileWriteRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in file_write_request"))?
                        .to_string(),
                    content: context.get("content")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing content in file_write_request"))?
                        .to_string(),
                    encoding: context.get("encoding")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    create_parents: context.get("create_parents")
                        .and_then(|v| v.as_bool()),
                    overwrite: context.get("overwrite")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileWriteRequest(request))
            }
            "file_delete_request" => {
                let request = FileDeleteRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in file_delete_request"))?
                        .to_string(),
                    recursive: context.get("recursive")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileDeleteRequest(request))
            }
            "file_exists_request" => {
                let request = FileExistsRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in file_exists_request"))?
                        .to_string(),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileExistsRequest(request))
            }
            "file_copy_request" => {
                let request = FileCopyRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    source: context.get("source")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing source in file_copy_request"))?
                        .to_string(),
                    destination: context.get("destination")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing destination in file_copy_request"))?
                        .to_string(),
                    overwrite: context.get("overwrite")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileCopyRequest(request))
            }
            "file_move_request" => {
                let request = FileMoveRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    source: context.get("source")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing source in file_move_request"))?
                        .to_string(),
                    destination: context.get("destination")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing destination in file_move_request"))?
                        .to_string(),
                    overwrite: context.get("overwrite")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileMoveRequest(request))
            }
            "directory_create_request" => {
                let request = DirectoryCreateRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in directory_create_request"))?
                        .to_string(),
                    create_parents: context.get("create_parents")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::DirectoryCreateRequest(request))
            }
            "directory_list_request" => {
                let request = DirectoryListRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in directory_list_request"))?
                        .to_string(),
                    recursive: context.get("recursive")
                        .and_then(|v| v.as_bool()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::DirectoryListRequest(request))
            }
            "file_checksum_request" => {
                let request = FileChecksumRequest {
                    request_id: context.get("request_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&event.id)
                        .to_string(),
                    path: context.get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing path in file_checksum_request"))?
                        .to_string(),
                    algorithm: context.get("algorithm")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    metadata: self.parse_metadata(context),
                };
                Ok(FileOperationEvent::FileChecksumRequest(request))
            }
            _ => Err(anyhow::anyhow!("Unsupported event type: {}", event_type)),
        }
    }

    /// Convert a FileOperationEvent to a HooksmithEvent
    fn create_result_event(&self, file_event: FileOperationEvent, original_event: &HooksmithEvent) -> HooksmithEvent {
        let (event_type, context) = match file_event {
            FileOperationEvent::FileReadResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(content) = result.content {
                    context["content"] = json!(content);
                }
                if let Some(size) = result.size {
                    context["size"] = json!(size);
                }
                if let Some(encoding) = result.encoding {
                    context["encoding"] = json!(encoding);
                }
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_read_result", context)
            }
            FileOperationEvent::FileWriteResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(size) = result.size {
                    context["size"] = json!(size);
                }
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_write_result", context)
            }
            FileOperationEvent::FileDeleteResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_delete_result", context)
            }
            FileOperationEvent::FileExistsResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                    "exists": result.exists,
                });
                
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_exists_result", context)
            }
            FileOperationEvent::FileCopyResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(size) = result.size {
                    context["size"] = json!(size);
                }
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_copy_result", context)
            }
            FileOperationEvent::FileMoveResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_move_result", context)
            }
            FileOperationEvent::DirectoryCreateResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(metadata) = result.metadata {
                    context["metadata"] = json!(metadata);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("directory_create_result", context)
            }
            FileOperationEvent::DirectoryListResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(files) = result.files {
                    context["files"] = json!(files);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("directory_list_result", context)
            }
            FileOperationEvent::FileChecksumResult(result) => {
                let mut context = json!({
                    "request_id": result.request_id,
                    "success": result.success,
                });
                
                if let Some(checksum) = result.checksum {
                    context["checksum"] = json!(checksum);
                }
                if let Some(algorithm) = result.algorithm {
                    context["algorithm"] = json!(algorithm);
                }
                if let Some(error) = result.error {
                    context["error"] = json!(error);
                }
                if let Some(duration_ms) = result.duration_ms {
                    context["duration_ms"] = json!(duration_ms);
                }
                
                ("file_checksum_result", context)
            }
            _ => {
                // This shouldn't happen for result events
                return HooksmithEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    event: "unknown_result".to_string(),
                    context: json!({"error": "Unknown result event type"}),
                    timestamp: chrono::Utc::now(),
                };
            }
        };

        HooksmithEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event: event_type.to_string(),
            context,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Parse metadata from event context
    fn parse_metadata(&self, context: &serde_json::Value) -> Option<EventMetadata> {
        let metadata = context.get("metadata")?;
        
        Some(EventMetadata {
            working_directory: metadata.get("working_directory")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timestamp: metadata.get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            session_id: metadata.get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

impl EventHandler for FileOperationsEventHandler {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()> {
        // Only handle file operation request events
        if !self.should_handle(event) {
            return Ok(());
        }

        // Parse the event
        let file_event = self.parse_event(event)?;
        
        // Handle the file operation
        let runtime = tokio::runtime::Runtime::new()?;
        let result_event = runtime.block_on(self.handler.handle_event(file_event))?;
        
        // Convert result back to HooksmithEvent and emit it
        let _result_hooksmith_event = self.create_result_event(result_event, event);
        // TODO: Implement event emission when xtask integration is complete
        // crate::xtask::event_bus::emit_event(result_hooksmith_event)?;
        
        Ok(())
    }

    fn name(&self) -> &str {
        "file-operations-handler"
    }

    fn should_handle(&self, event: &HooksmithEvent) -> bool {
        matches!(
            event.event.as_str(),
            "file_read_request" |
            "file_write_request" |
            "file_delete_request" |
            "file_exists_request" |
            "file_copy_request" |
            "file_move_request" |
            "directory_create_request" |
            "directory_list_request" |
            "file_checksum_request"
        )
    }
} 
