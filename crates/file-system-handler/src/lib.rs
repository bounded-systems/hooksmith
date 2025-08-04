//! File System Handler for Hooksmith
//!
//! This crate provides native Rust handlers for file system operations
//! that are triggered via the event bus. It handles file I/O operations
//! that cannot be performed by WIT components due to sandboxing restrictions.
//!
//! The handler processes SystemEvent variants related to file operations:
//! - FileRead: Read files from the filesystem
//! - FileWrite: Write content to files
//! - FileDelete: Delete files or directories

use anyhow::{Context, Result};
use event_types::{
    Event, EventHandler, EventPriority, HooksmithEvent, SystemEvent, HandlerRegistration,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// File system handler for processing file-related system events
pub struct FileSystemHandler {
    /// Base directory for file operations
    base_dir: PathBuf,
    /// Configuration options
    config: FileSystemConfig,
}

/// Configuration for the file system handler
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    /// Whether to allow absolute paths
    pub allow_absolute_paths: bool,
    /// Maximum file size to read (in bytes)
    pub max_file_size: usize,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
    /// Whether to create parent directories automatically
    pub auto_create_dirs: bool,
    /// Whether to preserve file permissions
    pub preserve_permissions: bool,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            allow_absolute_paths: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![
                "json".to_string(),
                "jsonc".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "md".to_string(),
                "txt".to_string(),
                "rs".to_string(),
                "wit".to_string(),
            ],
            auto_create_dirs: true,
            preserve_permissions: false,
        }
    }
}

impl FileSystemHandler {
    /// Create a new file system handler
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            config: FileSystemConfig::default(),
        }
    }

    /// Create a new file system handler with custom configuration
    pub fn with_config(base_dir: PathBuf, config: FileSystemConfig) -> Self {
        Self { base_dir, config }
    }

    /// Get the handler registration
    pub fn registration() -> HandlerRegistration {
        HandlerRegistration::new(
            "file-system-handler".to_string(),
            vec![
                "FileRead".to_string(),
                "FileWrite".to_string(),
                "FileDelete".to_string(),
            ],
            100, // High priority for file operations
        )
    }

    /// Validate a file path for security
    fn validate_path(&self, path: &str) -> Result<PathBuf> {
        let path = Path::new(path);
        
        // Check for absolute paths if not allowed
        if !self.config.allow_absolute_paths && path.is_absolute() {
            anyhow::bail!("Absolute paths are not allowed: {}", path.display());
        }

        // Resolve the path relative to base directory
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        };

        // Check for path traversal attacks
        if !resolved_path.starts_with(&self.base_dir) {
            anyhow::bail!("Path traversal detected: {}", path.display());
        }

        Ok(resolved_path)
    }

    /// Check if a file extension is allowed
    fn is_extension_allowed(&self, path: &Path) -> bool {
        if self.config.allowed_extensions.is_empty() {
            return true; // No restrictions
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.config.allowed_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }

    /// Handle file read operation
    async fn handle_file_read(&self, path: &str, binary: Option<bool>) -> Result<String> {
        let resolved_path = self.validate_path(path)?;
        
        if !resolved_path.exists() {
            anyhow::bail!("File does not exist: {}", resolved_path.display());
        }

        if !resolved_path.is_file() {
            anyhow::bail!("Path is not a file: {}", resolved_path.display());
        }

        if !self.is_extension_allowed(&resolved_path) {
            anyhow::bail!("File extension not allowed: {}", resolved_path.display());
        }

        // Check file size
        let metadata = fs::metadata(&resolved_path).await?;
        if metadata.len() as usize > self.config.max_file_size {
            anyhow::bail!(
                "File too large: {} bytes (max: {})",
                metadata.len(),
                self.config.max_file_size
            );
        }

        // Read file content
        let content = if binary.unwrap_or(false) {
            let bytes = fs::read(&resolved_path).await?;
            base64::encode(&bytes)
        } else {
            fs::read_to_string(&resolved_path).await?
        };

        info!("Successfully read file: {} ({} bytes)", resolved_path.display(), content.len());
        Ok(content)
    }

    /// Handle file write operation
    async fn handle_file_write(&self, path: &str, content: &str, create_dirs: Option<bool>) -> Result<()> {
        let resolved_path = self.validate_path(path)?;
        
        if !self.is_extension_allowed(&resolved_path) {
            anyhow::bail!("File extension not allowed: {}", resolved_path.display());
        }

        // Create parent directories if requested
        if create_dirs.unwrap_or(self.config.auto_create_dirs) {
            if let Some(parent) = resolved_path.parent() {
                fs::create_dir_all(parent).await?;
            }
        }

        // Write file content
        fs::write(&resolved_path, content).await?;

        info!("Successfully wrote file: {} ({} bytes)", resolved_path.display(), content.len());
        Ok(())
    }

    /// Handle file delete operation
    async fn handle_file_delete(&self, path: &str, recursive: Option<bool>) -> Result<()> {
        let resolved_path = self.validate_path(path)?;
        
        if !resolved_path.exists() {
            anyhow::bail!("File does not exist: {}", resolved_path.display());
        }

        // Delete file or directory
        if resolved_path.is_file() {
            fs::remove_file(&resolved_path).await?;
            info!("Successfully deleted file: {}", resolved_path.display());
        } else if resolved_path.is_dir() {
            if recursive.unwrap_or(false) {
                fs::remove_dir_all(&resolved_path).await?;
                info!("Successfully deleted directory recursively: {}", resolved_path.display());
            } else {
                fs::remove_dir(&resolved_path).await?;
                info!("Successfully deleted directory: {}", resolved_path.display());
            }
        } else {
            anyhow::bail!("Path is neither a file nor directory: {}", resolved_path.display());
        }

        Ok(())
    }

    /// Emit a file operation result event
    async fn emit_result(&self, event: &Event, result: Result<Value, String>) -> Result<()> {
        // In a real implementation, this would emit an event back to the bus
        match result {
            Ok(data) => {
                info!("File operation successful: {:?}", data);
            }
            Err(error) => {
                error!("File operation failed: {}", error);
            }
        }
        Ok(())
    }
}

impl EventHandler for FileSystemHandler {
    fn handle(&self, event: &Event) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        debug!("Processing file system event: {:?}", event.metadata.id);

        let result = match &event.payload {
            HooksmithEvent::System(SystemEvent::FileRead { path, binary }) => {
                let content = tokio::runtime::Handle::current()
                    .block_on(self.handle_file_read(path, *binary))?;
                
                serde_json::json!({
                    "success": true,
                    "content": content,
                    "path": path,
                    "binary": binary.unwrap_or(false)
                })
            }

            HooksmithEvent::System(SystemEvent::FileWrite { path, content, create_dirs }) => {
                tokio::runtime::Handle::current()
                    .block_on(self.handle_file_write(path, content, *create_dirs))?;
                
                serde_json::json!({
                    "success": true,
                    "path": path,
                    "bytes_written": content.len(),
                    "create_dirs": create_dirs.unwrap_or(self.config.auto_create_dirs)
                })
            }

            HooksmithEvent::System(SystemEvent::FileDelete { path, recursive }) => {
                tokio::runtime::Handle::current()
                    .block_on(self.handle_file_delete(path, *recursive))?;
                
                serde_json::json!({
                    "success": true,
                    "path": path,
                    "recursive": recursive.unwrap_or(false)
                })
            }

            _ => {
                warn!("Unsupported event type for file system handler");
                return Ok(());
            }
        };

        let duration = start_time.elapsed();
        info!(
            "File system event processed in {:?}: {}",
            duration,
            event.metadata.id
        );

        // Emit result event
        tokio::runtime::Handle::current()
            .block_on(self.emit_result(event, Ok(result)))?;

        Ok(())
    }

    fn supported_events(&self) -> Vec<String> {
        vec![
            "FileRead".to_string(),
            "FileWrite".to_string(),
            "FileDelete".to_string(),
        ]
    }

    fn name(&self) -> &str {
        "file-system-handler"
    }
}

/// File system operations result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileOperationResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Operation result data
    pub data: Option<Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Operation duration in milliseconds
    pub duration_ms: u64,
}

/// File metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    /// File path
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// File modification time
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    /// File permissions
    pub permissions: Option<String>,
    /// File extension
    pub extension: Option<String>,
}

impl FileSystemHandler {
    /// Get file metadata
    pub async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let resolved_path = self.validate_path(path)?;
        
        if !resolved_path.exists() {
            anyhow::bail!("File does not exist: {}", resolved_path.display());
        }

        let metadata = fs::metadata(&resolved_path).await?;
        
        Ok(FileMetadata {
            path: resolved_path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .ok()
                .map(|time| chrono::DateTime::from(time)),
            permissions: Some(format!("{:?}", metadata.permissions())),
            extension: resolved_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_string()),
        })
    }

    /// List files in a directory
    pub async fn list_files(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let resolved_path = self.validate_path(path)?;
        
        if !resolved_path.exists() {
            anyhow::bail!("Directory does not exist: {}", resolved_path.display());
        }

        if !resolved_path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", resolved_path.display());
        }

        let mut files = Vec::new();
        let mut read_dir = fs::read_dir(&resolved_path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            
            // Skip hidden files
            if path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            // Check extension if restrictions apply
            if !self.config.allowed_extensions.is_empty() && !self.is_extension_allowed(&path) {
                continue;
            }

            let metadata = entry.metadata().await?;
            files.push(FileMetadata {
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .map(|time| chrono::DateTime::from(time)),
                permissions: Some(format!("{:?}", metadata.permissions())),
                extension: path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_string()),
            });
        }

        Ok(files)
    }

    /// Check if a file exists
    pub async fn file_exists(&self, path: &str) -> Result<bool> {
        let resolved_path = self.validate_path(path)?;
        Ok(resolved_path.exists())
    }

    /// Get file size
    pub async fn get_file_size(&self, path: &str) -> Result<u64> {
        let resolved_path = self.validate_path(path)?;
        
        if !resolved_path.exists() {
            anyhow::bail!("File does not exist: {}", resolved_path.display());
        }

        let metadata = fs::metadata(&resolved_path).await?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FileSystemHandler::new(temp_dir.path().to_path_buf());

        // Test file write
        let content = r#"{"test": "data"}"#;
        handler
            .handle_file_write("test.json", content, Some(true))
            .await
            .unwrap();

        // Test file read
        let read_content = handler
            .handle_file_read("test.json", Some(false))
            .await
            .unwrap();

        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_file_delete() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FileSystemHandler::new(temp_dir.path().to_path_buf());

        // Create a test file
        let content = "test content";
        handler
            .handle_file_write("test.txt", content, Some(true))
            .await
            .unwrap();

        // Verify file exists
        assert!(handler.file_exists("test.txt").await.unwrap());

        // Delete file
        handler.handle_file_delete("test.txt", Some(false)).await.unwrap();

        // Verify file is deleted
        assert!(!handler.file_exists("test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FileSystemHandler::new(temp_dir.path().to_path_buf());

        // Test absolute path (should fail by default)
        let result = handler.validate_path("/etc/passwd");
        assert!(result.is_err());

        // Test relative path (should succeed)
        let result = handler.validate_path("test.json");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extension_validation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = FileSystemConfig::default();
        config.allowed_extensions = vec!["json".to_string(), "txt".to_string()];
        let handler = FileSystemHandler::with_config(temp_dir.path().to_path_buf(), config);

        // Test allowed extension
        assert!(handler.is_extension_allowed(Path::new("test.json")));

        // Test disallowed extension
        assert!(!handler.is_extension_allowed(Path::new("test.exe")));
    }
} 
