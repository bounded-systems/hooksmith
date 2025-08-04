//! File operations implementation for the file operations handler

use super::*;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use walkdir::WalkDir;

impl FileOperationsHandler {
    /// Handle file read request
    pub async fn handle_file_read(&self, req: FileReadRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.read_file(&path, req.encoding.as_deref()).await {
            Ok((content, size, metadata)) => {
                Ok(FileOperationEvent::FileReadResult(FileReadResult {
                    request_id: req.request_id,
                    success: true,
                    content: Some(content),
                    size: Some(size),
                    encoding: req.encoding,
                    metadata: Some(metadata),
                    duration_ms: None, // Set by caller
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileReadResult(FileReadResult {
                    request_id: req.request_id,
                    success: false,
                    content: None,
                    size: None,
                    encoding: req.encoding,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "READ_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file write request
    pub async fn handle_file_write(&self, req: FileWriteRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.write_file(&path, &req.content, req.encoding.as_deref(), req.create_parents.unwrap_or(false), req.overwrite.unwrap_or(false)).await {
            Ok((size, metadata)) => {
                Ok(FileOperationEvent::FileWriteResult(FileWriteResult {
                    request_id: req.request_id,
                    success: true,
                    size: Some(size),
                    metadata: Some(metadata),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileWriteResult(FileWriteResult {
                    request_id: req.request_id,
                    success: false,
                    size: None,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "WRITE_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file delete request
    pub async fn handle_file_delete(&self, req: FileDeleteRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.delete_file(&path, req.recursive.unwrap_or(false)).await {
            Ok(()) => {
                Ok(FileOperationEvent::FileDeleteResult(FileDeleteResult {
                    request_id: req.request_id,
                    success: true,
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileDeleteResult(FileDeleteResult {
                    request_id: req.request_id,
                    success: false,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "DELETE_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file exists request
    pub async fn handle_file_exists(&self, req: FileExistsRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.file_exists(&path).await {
            Ok((exists, metadata)) => {
                Ok(FileOperationEvent::FileExistsResult(FileExistsResult {
                    request_id: req.request_id,
                    success: true,
                    exists,
                    metadata: Some(metadata),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileExistsResult(FileExistsResult {
                    request_id: req.request_id,
                    success: false,
                    exists: false,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "EXISTS_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file copy request
    pub async fn handle_file_copy(&self, req: FileCopyRequest) -> Result<FileOperationEvent> {
        let source = self.resolve_path(&req.source);
        let destination = self.resolve_path(&req.destination);
        
        match self.copy_file(&source, &destination, req.overwrite.unwrap_or(false)).await {
            Ok((size, metadata)) => {
                Ok(FileOperationEvent::FileCopyResult(FileCopyResult {
                    request_id: req.request_id,
                    success: true,
                    size: Some(size),
                    metadata: Some(metadata),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileCopyResult(FileCopyResult {
                    request_id: req.request_id,
                    success: false,
                    size: None,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "COPY_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file move request
    pub async fn handle_file_move(&self, req: FileMoveRequest) -> Result<FileOperationEvent> {
        let source = self.resolve_path(&req.source);
        let destination = self.resolve_path(&req.destination);
        
        match self.move_file(&source, &destination, req.overwrite.unwrap_or(false)).await {
            Ok(metadata) => {
                Ok(FileOperationEvent::FileMoveResult(FileMoveResult {
                    request_id: req.request_id,
                    success: true,
                    metadata: Some(metadata),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileMoveResult(FileMoveResult {
                    request_id: req.request_id,
                    success: false,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "MOVE_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle directory create request
    pub async fn handle_directory_create(&self, req: DirectoryCreateRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.create_directory(&path, req.create_parents.unwrap_or(false)).await {
            Ok(metadata) => {
                Ok(FileOperationEvent::DirectoryCreateResult(DirectoryCreateResult {
                    request_id: req.request_id,
                    success: true,
                    metadata: Some(metadata),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::DirectoryCreateResult(DirectoryCreateResult {
                    request_id: req.request_id,
                    success: false,
                    metadata: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "CREATE_DIR_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle directory list request
    pub async fn handle_directory_list(&self, req: DirectoryListRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        
        match self.list_directory(&path, req.recursive.unwrap_or(false)).await {
            Ok(files) => {
                Ok(FileOperationEvent::DirectoryListResult(DirectoryListResult {
                    request_id: req.request_id,
                    success: true,
                    files: Some(files),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::DirectoryListResult(DirectoryListResult {
                    request_id: req.request_id,
                    success: false,
                    files: None,
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "LIST_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Handle file checksum request
    pub async fn handle_file_checksum(&self, req: FileChecksumRequest) -> Result<FileOperationEvent> {
        let path = self.resolve_path(&req.path);
        let algorithm = req.algorithm.unwrap_or_else(|| "sha256".to_string());
        
        match self.calculate_checksum(&path, &algorithm).await {
            Ok(checksum) => {
                Ok(FileOperationEvent::FileChecksumResult(FileChecksumResult {
                    request_id: req.request_id,
                    success: true,
                    checksum: Some(checksum),
                    algorithm: Some(algorithm),
                    duration_ms: None,
                    error: None,
                }))
            }
            Err(e) => {
                Ok(FileOperationEvent::FileChecksumResult(FileChecksumResult {
                    request_id: req.request_id,
                    success: false,
                    checksum: None,
                    algorithm: Some(algorithm),
                    duration_ms: None,
                    error: Some(FileOperationError {
                        code: "CHECKSUM_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        io_error: Some(e.to_string()),
                    }),
                }))
            }
        }
    }

    /// Read a file
    async fn read_file(&self, path: &Path, encoding: Option<&str>) -> Result<(String, u64, FileMetadata)> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let content_str = match encoding {
            Some("base64") => base64::engine::general_purpose::STANDARD.encode(&content),
            Some("binary") => format!("{:?}", content),
            _ => String::from_utf8(content)
                .with_context(|| format!("Failed to decode UTF-8 content from: {}", path.display()))?,
        };
        
        let file_metadata = self.get_file_metadata(path)?;
        
        Ok((content_str, size, file_metadata))
    }

    /// Write a file
    async fn write_file(&self, path: &Path, content: &str, encoding: Option<&str>, create_parents: bool, overwrite: bool) -> Result<(u64, FileMetadata)> {
        if create_parents {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directories for: {}", path.display()))?;
            }
        }
        
        if path.exists() && !overwrite {
            return Err(anyhow::anyhow!("File already exists and overwrite is false: {}", path.display()));
        }
        
        let content_bytes = match encoding {
            Some("base64") => base64::engine::general_purpose::STANDARD.decode(content)
                .with_context(|| "Failed to decode base64 content")?,
            Some("binary") => {
                // Parse binary string like "[1, 2, 3]"
                let content = content.trim_matches(&['[', ']'] as &[_]);
                content.split(',')
                    .map(|s| s.trim().parse::<u8>())
                    .collect::<Result<Vec<u8>, _>>()
                    .with_context(|| "Failed to parse binary content")?
            }
            _ => content.as_bytes().to_vec(),
        };
        
        let mut file = File::create(path)
            .with_context(|| format!("Failed to create file: {}", path.display()))?;
        
        file.write_all(&content_bytes)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
        
        let size = content_bytes.len() as u64;
        let file_metadata = self.get_file_metadata(path)?;
        
        Ok((size, file_metadata))
    }

    /// Delete a file or directory
    async fn delete_file(&self, path: &Path, recursive: bool) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        
        let metadata = fs::metadata(path)?;
        
        if metadata.is_dir() {
            if recursive {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            } else {
                fs::remove_dir(path)
                    .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            }
        } else {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        }
        
        Ok(())
    }

    /// Check if file exists
    async fn file_exists(&self, path: &Path) -> Result<(bool, FileMetadata)> {
        if path.exists() {
            let metadata = self.get_file_metadata(path)?;
            Ok((true, metadata))
        } else {
            Ok((false, FileMetadata {
                created: None,
                modified: None,
                permissions: None,
                owner: None,
                size: None,
                is_directory: false,
            }))
        }
    }

    /// Copy a file
    async fn copy_file(&self, source: &Path, destination: &Path, overwrite: bool) -> Result<(u64, FileMetadata)> {
        if !source.exists() {
            return Err(anyhow::anyhow!("Source file does not exist: {}", source.display()));
        }
        
        if destination.exists() && !overwrite {
            return Err(anyhow::anyhow!("Destination file already exists and overwrite is false: {}", destination.display()));
        }
        
        // Create parent directories if needed
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for: {}", destination.display()))?;
        }
        
        fs::copy(source, destination)
            .with_context(|| format!("Failed to copy {} to {}", source.display(), destination.display()))?;
        
        let metadata = self.get_file_metadata(destination)?;
        let size = metadata.size.unwrap_or(0);
        
        Ok((size, metadata))
    }

    /// Move a file
    async fn move_file(&self, source: &Path, destination: &Path, overwrite: bool) -> Result<FileMetadata> {
        if !source.exists() {
            return Err(anyhow::anyhow!("Source file does not exist: {}", source.display()));
        }
        
        if destination.exists() && !overwrite {
            return Err(anyhow::anyhow!("Destination file already exists and overwrite is false: {}", destination.display()));
        }
        
        // Create parent directories if needed
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for: {}", destination.display()))?;
        }
        
        fs::rename(source, destination)
            .with_context(|| format!("Failed to move {} to {}", source.display(), destination.display()))?;
        
        let metadata = self.get_file_metadata(destination)?;
        
        Ok(metadata)
    }

    /// Create a directory
    async fn create_directory(&self, path: &Path, create_parents: bool) -> Result<FileMetadata> {
        if path.exists() {
            let metadata = self.get_file_metadata(path)?;
            if metadata.is_directory {
                return Ok(metadata);
            } else {
                return Err(anyhow::anyhow!("Path exists but is not a directory: {}", path.display()));
            }
        }
        
        if create_parents {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        } else {
            fs::create_dir(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        
        let metadata = self.get_file_metadata(path)?;
        
        Ok(metadata)
    }

    /// List directory contents
    async fn list_directory(&self, path: &Path, recursive: bool) -> Result<Vec<FileInfo>> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Directory does not exist: {}", path.display()));
        }
        
        let metadata = fs::metadata(path)?;
        if !metadata.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {}", path.display()));
        }
        
        let mut files = Vec::new();
        
        if recursive {
            for entry in WalkDir::new(path).min_depth(1) {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let file_info = FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_directory: path.is_dir(),
                    size: if path.is_file() { Some(fs::metadata(path)?.len()) } else { None },
                    modified: fs::metadata(path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| DateTime::from(t)),
                };
                
                files.push(file_info);
            }
        } else {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let file_info = FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_directory: path.is_dir(),
                    size: if path.is_file() { Some(fs::metadata(&path)?.len()) } else { None },
                    modified: fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| DateTime::from(t)),
                };
                
                files.push(file_info);
            }
        }
        
        Ok(files)
    }

    /// Calculate file checksum
    async fn calculate_checksum(&self, path: &Path, algorithm: &str) -> Result<String> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
        }
        
        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            return Err(anyhow::anyhow!("Cannot calculate checksum for directory: {}", path.display()));
        }
        
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file for checksum: {}", path.display()))?;
        
        let mut hasher = match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let mut hasher = Sha256::new();
                let mut buffer = [0; 4096];
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 { break; }
                    hasher.update(&buffer[..n]);
                }
                format!("{:x}", hasher.finalize())
            }
            "sha1" => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                let mut buffer = [0; 4096];
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 { break; }
                    hasher.update(&buffer[..n]);
                }
                format!("{:x}", hasher.finalize())
            }
            "md5" => {
                use md5::{Md5, Digest};
                let mut hasher = Md5::new();
                let mut buffer = [0; 4096];
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 { break; }
                    hasher.update(&buffer[..n]);
                }
                format!("{:x}", hasher.finalize())
            }
            _ => return Err(anyhow::anyhow!("Unsupported checksum algorithm: {}", algorithm)),
        };
        
        Ok(hasher)
    }
} 
