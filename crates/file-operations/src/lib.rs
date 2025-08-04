//! File Operations Handler for Hooksmith Hybrid Architecture
//!
//! This crate provides native file system operations as event handlers
//! for the hybrid WIT + native Rust architecture. It handles all file I/O
//! operations that require system privileges.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use base64::Engine;
use std::path::{Path, PathBuf};
use std::time::Instant;

// Newtype wrapper for DateTime<Utc> to implement JsonSchema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DateTimeUtc(String);

impl From<DateTime<Utc>> for DateTimeUtc {
    fn from(dt: DateTime<Utc>) -> Self {
        DateTimeUtc(dt.to_rfc3339())
    }
}

impl From<DateTimeUtc> for DateTime<Utc> {
    fn from(dt: DateTimeUtc) -> Self {
        DateTime::parse_from_rfc3339(&dt.0)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc)
    }
}

pub mod operations;
pub mod event_handler;
pub mod schema;

/// File operation event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FileOperationEvent {
    FileReadRequest(FileReadRequest),
    FileReadResult(FileReadResult),
    FileWriteRequest(FileWriteRequest),
    FileWriteResult(FileWriteResult),
    FileDeleteRequest(FileDeleteRequest),
    FileDeleteResult(FileDeleteResult),
    FileExistsRequest(FileExistsRequest),
    FileExistsResult(FileExistsResult),
    FileCopyRequest(FileCopyRequest),
    FileCopyResult(FileCopyResult),
    FileMoveRequest(FileMoveRequest),
    FileMoveResult(FileMoveResult),
    DirectoryCreateRequest(DirectoryCreateRequest),
    DirectoryCreateResult(DirectoryCreateResult),
    DirectoryListRequest(DirectoryListRequest),
    DirectoryListResult(DirectoryListResult),
    FileChecksumRequest(FileChecksumRequest),
    FileChecksumResult(FileChecksumResult),
}

/// File read request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileReadRequest {
    pub request_id: String,
    pub path: String,
    pub encoding: Option<String>,
    pub metadata: Option<EventMetadata>,
}

/// File read result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileReadResult {
    pub request_id: String,
    pub success: bool,
    pub content: Option<String>,
    pub size: Option<u64>,
    pub encoding: Option<String>,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File write request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileWriteRequest {
    pub request_id: String,
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
    pub create_parents: Option<bool>,
    pub overwrite: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// File write result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileWriteResult {
    pub request_id: String,
    pub success: bool,
    pub size: Option<u64>,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File delete request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileDeleteRequest {
    pub request_id: String,
    pub path: String,
    pub recursive: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// File delete result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileDeleteResult {
    pub request_id: String,
    pub success: bool,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File exists request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileExistsRequest {
    pub request_id: String,
    pub path: String,
    pub metadata: Option<EventMetadata>,
}

/// File exists result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileExistsResult {
    pub request_id: String,
    pub success: bool,
    pub exists: bool,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File copy request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileCopyRequest {
    pub request_id: String,
    pub source: String,
    pub destination: String,
    pub overwrite: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// File copy result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileCopyResult {
    pub request_id: String,
    pub success: bool,
    pub size: Option<u64>,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File move request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMoveRequest {
    pub request_id: String,
    pub source: String,
    pub destination: String,
    pub overwrite: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// File move result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMoveResult {
    pub request_id: String,
    pub success: bool,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// Directory create request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryCreateRequest {
    pub request_id: String,
    pub path: String,
    pub create_parents: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// Directory create result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryCreateResult {
    pub request_id: String,
    pub success: bool,
    pub metadata: Option<FileMetadata>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// Directory list request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryListRequest {
    pub request_id: String,
    pub path: String,
    pub recursive: Option<bool>,
    pub metadata: Option<EventMetadata>,
}

/// Directory list result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryListResult {
    pub request_id: String,
    pub success: bool,
    pub files: Option<Vec<FileInfo>>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File checksum request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChecksumRequest {
    pub request_id: String,
    pub path: String,
    pub algorithm: Option<String>,
    pub metadata: Option<EventMetadata>,
}

/// File checksum result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChecksumResult {
    pub request_id: String,
    pub success: bool,
    pub checksum: Option<String>,
    pub algorithm: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<FileOperationError>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMetadata {
    pub created: Option<DateTimeUtc>,
    pub modified: Option<DateTimeUtc>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub size: Option<u64>,
    pub is_directory: bool,
}

/// File information for directory listings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<DateTimeUtc>,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventMetadata {
    pub working_directory: Option<String>,
    pub timestamp: Option<DateTimeUtc>,
    pub session_id: Option<String>,
}

/// File operation error
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileOperationError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub io_error: Option<String>,
}

/// File operations handler
pub struct FileOperationsHandler {
    working_directory: PathBuf,
    session_id: String,
}

impl FileOperationsHandler {
    /// Create a new file operations handler
    pub fn new(working_directory: PathBuf, session_id: String) -> Self {
        Self {
            working_directory,
            session_id,
        }
    }

    /// Handle a file operation event
    pub async fn handle_event(&self, event: FileOperationEvent) -> Result<FileOperationEvent> {
        let start_time = Instant::now();
        
        let result = match event {
            FileOperationEvent::FileReadRequest(req) => {
                self.handle_file_read(req).await?
            }
            FileOperationEvent::FileWriteRequest(req) => {
                self.handle_file_write(req).await?
            }
            FileOperationEvent::FileDeleteRequest(req) => {
                self.handle_file_delete(req).await?
            }
            FileOperationEvent::FileExistsRequest(req) => {
                self.handle_file_exists(req).await?
            }
            FileOperationEvent::FileCopyRequest(req) => {
                self.handle_file_copy(req).await?
            }
            FileOperationEvent::FileMoveRequest(req) => {
                self.handle_file_move(req).await?
            }
            FileOperationEvent::DirectoryCreateRequest(req) => {
                self.handle_directory_create(req).await?
            }
            FileOperationEvent::DirectoryListRequest(req) => {
                self.handle_directory_list(req).await?
            }
            FileOperationEvent::FileChecksumRequest(req) => {
                self.handle_file_checksum(req).await?
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported event type"));
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Add duration to result
        let result = match result {
            FileOperationEvent::FileReadResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileReadResult(res)
            }
            FileOperationEvent::FileWriteResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileWriteResult(res)
            }
            FileOperationEvent::FileDeleteResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileDeleteResult(res)
            }
            FileOperationEvent::FileExistsResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileExistsResult(res)
            }
            FileOperationEvent::FileCopyResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileCopyResult(res)
            }
            FileOperationEvent::FileMoveResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileMoveResult(res)
            }
            FileOperationEvent::DirectoryCreateResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::DirectoryCreateResult(res)
            }
            FileOperationEvent::DirectoryListResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::DirectoryListResult(res)
            }
            FileOperationEvent::FileChecksumResult(mut res) => {
                res.duration_ms = Some(duration_ms);
                FileOperationEvent::FileChecksumResult(res)
            }
            _ => result,
        };

        Ok(result)
    }

    /// Resolve a path relative to working directory
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_directory.join(path)
        }
    }

    /// Get file metadata
    fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = fs::metadata(path)?;
        
        Ok(FileMetadata {
            created: None, // Not available on all platforms
            modified: Some(DateTimeUtc::from(DateTime::from(metadata.modified()?))),
            permissions: Some(format!("{:?}", metadata.permissions())),
            owner: None, // Would need platform-specific code
            size: Some(metadata.len()),
            is_directory: metadata.is_dir(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_operations_handler() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FileOperationsHandler::new(
            temp_dir.path().to_path_buf(),
            "test-session".to_string(),
        );

        // Test file write and read
        let write_req = FileWriteRequest {
            request_id: Uuid::new_v4().to_string(),
            path: "test.txt".to_string(),
            content: "Hello, World!".to_string(),
            encoding: Some("utf8".to_string()),
            create_parents: Some(false),
            overwrite: Some(true),
            metadata: None,
        };

        let write_result = handler.handle_event(FileOperationEvent::FileWriteRequest(write_req)).await.unwrap();
        assert!(matches!(write_result, FileOperationEvent::FileWriteResult(_)));

        let read_req = FileReadRequest {
            request_id: Uuid::new_v4().to_string(),
            path: "test.txt".to_string(),
            encoding: Some("utf8".to_string()),
            metadata: None,
        };

        let read_result = handler.handle_event(FileOperationEvent::FileReadRequest(read_req)).await.unwrap();
        assert!(matches!(read_result, FileOperationEvent::FileReadResult(_)));
    }
} 
