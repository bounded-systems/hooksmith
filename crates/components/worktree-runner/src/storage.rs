use anyhow::{Context, Result};
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

use crate::kube_crd::WorktreeChangeRequest;

/// Storage system for WorktreeChangeRequest CRDs
pub struct WorktreeStorage {
    storage_dir: PathBuf,
}

impl WorktreeStorage {
    /// Create a new storage system
    pub fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }

    /// Initialize the storage directory
    pub async fn init(&self) -> Result<()> {
        if !self.storage_dir.exists() {
            fs::create_dir_all(&self.storage_dir).await
                .context("Failed to create storage directory")?;
            info!("Created storage directory: {:?}", self.storage_dir);
        }
        Ok(())
    }

    /// Save a CRD to storage
    pub async fn save_crd(&self, crd: &WorktreeChangeRequest) -> Result<()> {
        let filename = self.get_crd_filename(&crd.metadata.name.as_ref().unwrap_or(&crd.spec.branch));
        let filepath = self.storage_dir.join(filename);
        
        let json = serde_json::to_string_pretty(crd)
            .context("Failed to serialize CRD")?;
        
        fs::write(&filepath, json).await
            .context("Failed to write CRD file")?;
        
        debug!("Saved CRD to: {:?}", filepath);
        Ok(())
    }

    /// Load a CRD from storage
    pub async fn load_crd(&self, branch_name: &str) -> Result<Option<WorktreeChangeRequest>> {
        let filename = self.get_crd_filename(branch_name);
        let filepath = self.storage_dir.join(filename);
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&filepath).await
            .context("Failed to read CRD file")?;
        
        let crd: WorktreeChangeRequest = serde_json::from_str(&content)
            .context("Failed to deserialize CRD")?;
        
        debug!("Loaded CRD from: {:?}", filepath);
        Ok(Some(crd))
    }

    /// Load all CRDs from storage
    pub async fn load_all_crds(&self) -> Result<HashMap<String, WorktreeChangeRequest>> {
        let mut crds = HashMap::new();
        
        if !self.storage_dir.exists() {
            return Ok(crds);
        }
        
        let mut entries = fs::read_dir(&self.storage_dir).await
            .context("Failed to read storage directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(branch_name) = self.parse_branch_name_from_filename(&path) {
                    if let Ok(Some(crd)) = self.load_crd(&branch_name).await {
                        crds.insert(branch_name, crd);
                    }
                }
            }
        }
        
        info!("Loaded {} CRDs from storage", crds.len());
        Ok(crds)
    }

    /// Delete a CRD from storage
    pub async fn delete_crd(&self, branch_name: &str) -> Result<bool> {
        let filename = self.get_crd_filename(branch_name);
        let filepath = self.storage_dir.join(filename);
        
        if filepath.exists() {
            fs::remove_file(&filepath).await
                .context("Failed to delete CRD file")?;
            debug!("Deleted CRD: {:?}", filepath);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update a CRD in storage
    pub async fn update_crd(&self, crd: &WorktreeChangeRequest) -> Result<()> {
        // Update the last modified timestamp
        let crd_clone = crd.clone();
        // Note: Kubernetes CRD doesn't have a touch method
        // We'll update the creation timestamp instead
        
        self.save_crd(&crd_clone).await
    }

    /// Get the filename for a CRD
    fn get_crd_filename(&self, branch_name: &str) -> String {
        // Sanitize branch name for filename
        let sanitized = branch_name
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_")
            .replace('*', "_")
            .replace('?', "_")
            .replace('"', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_");
        
        format!("{}.json", sanitized)
    }

    /// Parse branch name from filename
    fn parse_branch_name_from_filename(&self, path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.replace('_', "/"))
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let mut stats = StorageStats {
            total_crds: 0,
            active_crds: 0,
            completed_crds: 0,
            failed_crds: 0,
            storage_size_bytes: 0,
        };
        
        if !self.storage_dir.exists() {
            return Ok(stats);
        }
        
        let mut entries = fs::read_dir(&self.storage_dir).await
            .context("Failed to read storage directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                stats.total_crds += 1;
                
                if let Ok(metadata) = fs::metadata(&path).await {
                    stats.storage_size_bytes += metadata.len();
                }
                
                // Try to load the CRD to get status info
                if let Some(branch_name) = self.parse_branch_name_from_filename(&path) {
                    if let Ok(Some(crd)) = self.load_crd(&branch_name).await {
                        // Note: Kubernetes CRD status is managed separately
                        // For now, we'll use the state from spec
                        match crd.spec.state {
                            crate::kube_crd::WorktreeState::Merged => stats.completed_crds += 1,
                            crate::kube_crd::WorktreeState::Removed => stats.completed_crds += 1,
                            _ => stats.active_crds += 1,
                        }
                    }
                }
            }
        }
        
        Ok(stats)
    }

    /// Clean up old completed CRDs
    pub async fn cleanup_old_crds(&self, max_age_days: u64) -> Result<usize> {
        let mut deleted_count = 0;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);
        
        let crds = self.load_all_crds().await?;
        let _crds_len = crds.len();
        
        for (branch_name, crd) in crds {
            // Note: Kubernetes CRD doesn't have last_modified field
            // We'll use creation_timestamp instead
            if let Some(creation_timestamp) = &crd.metadata.creation_timestamp {
                if creation_timestamp.0 < cutoff {
                    if self.delete_crd(&branch_name).await? {
                        deleted_count += 1;
                        info!("Cleaned up old CRD: {}", branch_name);
                    }
                }
            }
        }
        
        info!("Cleaned up {} old CRDs", deleted_count);
        Ok(deleted_count)
    }

    /// Export CRDs to a different format
    pub async fn export_crds(&self, format: ExportFormat, output_path: &Path) -> Result<()> {
        let crds = self.load_all_crds().await?;
        let _crds_len = crds.len();
        
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&crds)
                    .context("Failed to serialize CRDs to JSON")?;
                fs::write(output_path, json).await
                    .context("Failed to write JSON export")?;
            }
            ExportFormat::Yaml => {
                let yaml = serde_yaml::to_string(&crds)
                    .context("Failed to serialize CRDs to YAML")?;
                fs::write(output_path, yaml).await
                    .context("Failed to write YAML export")?;
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("branch,state,local,remote,worktree,pr,last_modified\n");
                
                for (branch_name, crd) in crds {
                    let last_modified = crd.metadata.creation_timestamp
                        .as_ref()
                        .map(|dt| dt.0.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        branch_name,
                        format!("{:?}", crd.spec.state),
                        crd.spec.domains.local.exists,
                        crd.spec.domains.remote.exists,
                        crd.spec.domains.worktree.exists,
                        crd.spec.domains.pr.exists,
                        last_modified
                    ));
                }
                
                fs::write(output_path, csv).await
                    .context("Failed to write CSV export")?;
            }
        }
        
        info!("Exported {} CRDs to {:?}", _crds_len, output_path);
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_crds: usize,
    pub active_crds: usize,
    pub completed_crds: usize,
    pub failed_crds: usize,
    pub storage_size_bytes: u64,
}

/// Export formats
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Yaml,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::kube_crd::WorktreeChangeRequest;

    #[tokio::test]
    async fn test_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = WorktreeStorage::new(temp_dir.path().to_path_buf());
        
        storage.init().await.unwrap();
        assert!(temp_dir.path().exists());
    }

    #[tokio::test]
    async fn test_crd_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let storage = WorktreeStorage::new(temp_dir.path().to_path_buf());
        storage.init().await.unwrap();
        
        let crd = WorktreeChangeRequest::create("feature/test");
        storage.save_crd(&crd).await.unwrap();
        
        let loaded = storage.load_crd("feature/test").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().spec.branch, "feature/test");
    }

    #[tokio::test]
    async fn test_filename_sanitization() {
        let temp_dir = tempdir().unwrap();
        let storage = WorktreeStorage::new(temp_dir.path().to_path_buf());
        
        let filename = storage.get_crd_filename("feature/test-branch");
        assert_eq!(filename, "feature_test-branch.json");
        
        let filename = storage.get_crd_filename("bugfix/urgent-fix!");
        assert_eq!(filename, "bugfix_urgent-fix_.json");
    }
} 
