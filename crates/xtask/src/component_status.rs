//! Component Status Module
//!
//! This module provides comprehensive status checking for all WIT components
//! and native crates in the Hooksmith workspace. It includes build status,
//! version information, schema validation, and checksum verification.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Component registry structure
#[derive(Debug, Deserialize)]
pub struct ComponentRegistry {
    pub metadata: RegistryMetadata,
    pub wit_components: Vec<RegistryItem>,
    pub native_crates: Vec<RegistryItem>,
    pub categories: HashMap<String, String>,
    pub targets: HashMap<String, String>,
    pub status_definitions: HashMap<String, String>,
}

/// Registry metadata
#[derive(Debug, Deserialize)]
pub struct RegistryMetadata {
    pub version: String,
    pub description: String,
    pub last_updated: String,
    pub schema_version: String,
}

/// Registry item for components and crates
#[derive(Debug, Deserialize, Clone)]
pub struct RegistryItem {
    pub name: String,
    pub path: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub targets: Vec<String>,
    pub features: Vec<String>,
    pub wit: Option<String>,
}

/// Component status information
#[derive(Debug, Serialize, Clone)]
pub struct ComponentStatus {
    pub name: String,
    pub path: String,
    pub category: String,
    pub status: String,
    pub build_status: BuildStatus,
    pub version: Option<String>,
    pub checksum: Option<String>,
    pub schema_valid: Option<bool>,
    pub last_build: Option<String>,
    pub build_duration: Option<Duration>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Build status enumeration
#[derive(Debug, Serialize, Clone)]
pub enum BuildStatus {
    Success,
    Failed,
    NotBuilt,
    Building,
}

/// Overall status summary
#[derive(Debug, Serialize)]
pub struct StatusSummary {
    pub total_components: usize,
    pub successful_builds: usize,
    pub failed_builds: usize,
    pub not_built: usize,
    pub wit_components: Vec<ComponentStatus>,
    pub native_crates: Vec<ComponentStatus>,
    pub build_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Component status checker
pub struct ComponentStatusChecker {
    registry: ComponentRegistry,
    workspace_root: PathBuf,
    verbose: bool,
    check_checksums: bool,
    check_schemas: bool,
}

impl ComponentStatusChecker {
    /// Create a new component status checker
    pub async fn new(workspace_root: PathBuf, verbose: bool) -> Result<Self> {
        let registry_path = workspace_root.join("config/component-registry.jsonc");
        let registry_content = fs::read_to_string(&registry_path)
            .await
            .context("Failed to read component registry")?;

        let registry: ComponentRegistry = serde_json::from_str(&registry_content)
            .context("Failed to parse component registry")?;

        Ok(Self {
            registry,
            workspace_root,
            verbose,
            check_checksums: true,
            check_schemas: true,
        })
    }

    /// Set whether to check checksums
    pub fn with_checksums(mut self, check: bool) -> Self {
        self.check_checksums = check;
        self
    }

    /// Set whether to check schemas
    pub fn with_schemas(mut self, check: bool) -> Self {
        self.check_schemas = check;
        self
    }

    /// Check status of all components and crates
    pub async fn check_all_status(&self) -> Result<StatusSummary> {
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        info!("Starting component status check...");

        // Check WIT components
        let mut wit_statuses = Vec::new();
        for component in &self.registry.wit_components {
            let status = self.check_wit_component(component).await?;
            wit_statuses.push(status);
        }

        // Check native crates
        let mut native_statuses = Vec::new();
        for crate_item in &self.registry.native_crates {
            let status = self.check_native_crate(crate_item).await?;
            native_statuses.push(status);
        }

        let build_time = start_time.elapsed();
        let all_statuses: Vec<&ComponentStatus> = wit_statuses.iter().chain(native_statuses.iter()).collect();

        let summary = StatusSummary {
            total_components: all_statuses.len(),
            successful_builds: all_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Success)).count(),
            failed_builds: all_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Failed)).count(),
            not_built: all_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::NotBuilt)).count(),
            wit_components: wit_statuses,
            native_crates: native_statuses,
            build_time,
            timestamp,
        };

        info!("Component status check completed in {:?}", build_time);
        Ok(summary)
    }

    /// Check status of a WIT component
    async fn check_wit_component(&self, component: &RegistryItem) -> Result<ComponentStatus> {
        let start_time = Instant::now();
        let manifest_path = self.workspace_root.join(&component.path).join("Cargo.toml");

        if self.verbose {
            println!("Checking WIT component: {}", component.name);
        }

        // Check if manifest exists
        if !manifest_path.exists() {
            return Ok(ComponentStatus {
                name: component.name.clone(),
                path: component.path.clone(),
                category: component.category.clone(),
                status: component.status.clone(),
                build_status: BuildStatus::NotBuilt,
                version: None,
                checksum: None,
                schema_valid: None,
                last_build: None,
                build_duration: None,
                errors: vec!["Manifest file not found".to_string()],
                warnings: vec![],
            });
        }

        // Try to build the component
        let build_result = self.build_wit_component(&component.path).await;
        let build_status = match &build_result {
            Ok(_) => BuildStatus::Success,
            Err(_) => BuildStatus::Failed,
        };

        // Get version information
        let version = self.get_crate_version(&manifest_path).await.ok();

        // Check checksum if enabled
        let checksum = if self.check_checksums {
            self.get_component_checksum(&component.name).await.ok()
        } else {
            None
        };

        // Check schema validation if enabled
        let schema_valid = if self.check_schemas {
            if let Some(wit_path) = &component.wit {
                let full_wit_path = self.workspace_root.join(wit_path);
                Some(self.validate_wit_schema(&full_wit_path).await.is_ok())
            } else {
                None
            }
        } else {
            None
        };

        let build_duration = start_time.elapsed();
        let errors = if let Err(e) = build_result {
            vec![e.to_string()]
        } else {
            vec![]
        };

        Ok(ComponentStatus {
            name: component.name.clone(),
            path: component.path.clone(),
            category: component.category.clone(),
            status: component.status.clone(),
            build_status,
            version,
            checksum,
            schema_valid,
            last_build: Some(chrono::Utc::now().to_rfc3339()),
            build_duration: Some(build_duration),
            errors,
            warnings: vec![],
        })
    }

    /// Check status of a native crate
    async fn check_native_crate(&self, crate_item: &RegistryItem) -> Result<ComponentStatus> {
        let start_time = Instant::now();
        let manifest_path = self.workspace_root.join(&crate_item.path).join("Cargo.toml");

        if self.verbose {
            println!("Checking native crate: {}", crate_item.name);
        }

        // Check if manifest exists
        if !manifest_path.exists() {
            return Ok(ComponentStatus {
                name: crate_item.name.clone(),
                path: crate_item.path.clone(),
                category: crate_item.category.clone(),
                status: crate_item.status.clone(),
                build_status: BuildStatus::NotBuilt,
                version: None,
                checksum: None,
                schema_valid: None,
                last_build: None,
                build_duration: None,
                errors: vec!["Manifest file not found".to_string()],
                warnings: vec![],
            });
        }

        // Try to check the crate
        let check_result = self.check_native_crate_build(&crate_item.path).await;
        let build_status = match &check_result {
            Ok(_) => BuildStatus::Success,
            Err(_) => BuildStatus::Failed,
        };

        // Get version information
        let version = self.get_crate_version(&manifest_path).await.ok();

        // Check checksum if enabled
        let checksum = if self.check_checksums {
            self.get_crate_checksum(&crate_item.name).await.ok()
        } else {
            None
        };

        let build_duration = start_time.elapsed();
        let errors = if let Err(e) = check_result {
            vec![e.to_string()]
        } else {
            vec![]
        };

        Ok(ComponentStatus {
            name: crate_item.name.clone(),
            path: crate_item.path.clone(),
            category: crate_item.category.clone(),
            status: crate_item.status.clone(),
            build_status,
            version,
            checksum,
            schema_valid: None, // Native crates don't have WIT schemas
            last_build: Some(chrono::Utc::now().to_rfc3339()),
            build_duration: Some(build_duration),
            errors,
            warnings: vec![],
        })
    }

    /// Build a WIT component
    async fn build_wit_component(&self, component_path: &str) -> Result<()> {
        let output = Command::new("cargo")
            .args([
                "component",
                "build",
                "--manifest-path",
                &format!("{}/Cargo.toml", component_path),
                "--target",
                "wasm32-wasip2",
                "--release",
            ])
            .current_dir(&self.workspace_root)
            .stdout(if self.verbose { Stdio::inherit() } else { Stdio::piped() })
            .stderr(if self.verbose { Stdio::inherit() } else { Stdio::piped() })
            .output()
            .context("Failed to execute cargo component build")?;

        if !output.status.success() {
            let error_output = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Component build failed: {}", error_output);
        }

        Ok(())
    }

    /// Check a native crate build
    async fn check_native_crate_build(&self, crate_path: &str) -> Result<()> {
        let output = Command::new("cargo")
            .args([
                "check",
                "--manifest-path",
                &format!("{}/Cargo.toml", crate_path),
            ])
            .current_dir(&self.workspace_root)
            .stdout(if self.verbose { Stdio::inherit() } else { Stdio::piped() })
            .stderr(if self.verbose { Stdio::inherit() } else { Stdio::piped() })
            .output()
            .context("Failed to execute cargo check")?;

        if !output.status.success() {
            let error_output = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Crate check failed: {}", error_output);
        }

        Ok(())
    }

    /// Get crate version from Cargo.toml
    async fn get_crate_version(&self, manifest_path: &Path) -> Result<String> {
        let output = Command::new("cargo")
            .args([
                "metadata",
                "--format-version",
                "1",
                "--manifest-path",
                manifest_path.to_str().unwrap(),
                "--no-deps",
            ])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo metadata")?;

        if !output.status.success() {
            anyhow::bail!("Failed to get crate metadata");
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse cargo metadata")?;

        let version = metadata["packages"][0]["version"]
            .as_str()
            .context("Version not found in metadata")?;

        Ok(version.to_string())
    }

    /// Get component checksum
    async fn get_component_checksum(&self, component_name: &str) -> Result<String> {
        let wasm_path = self.workspace_root
            .join("target/wasm32-wasip2/release")
            .join(format!("{}.component.wasm", component_name.replace('-', "_")));

        if !wasm_path.exists() {
            anyhow::bail!("Component WASM file not found");
        }

        let content = fs::read(&wasm_path).await?;
        let checksum = sha2::Sha256::digest(&content);
        Ok(format!("{:x}", checksum))
    }

    /// Get crate checksum
    async fn get_crate_checksum(&self, crate_name: &str) -> Result<String> {
        // For native crates, we'll checksum the source directory
        let crate_path = self.workspace_root.join(crate_name);
        if !crate_path.exists() {
            anyhow::bail!("Crate directory not found");
        }

        // Simple directory checksum (in production, you might want a more sophisticated approach)
        let mut hasher = sha2::Sha256::new();
        self.hash_directory(&crate_path, &mut hasher).await?;
        let checksum = hasher.finalize();
        Ok(format!("{:x}", checksum))
    }

    /// Hash a directory recursively
    async fn hash_directory(&self, dir: &Path, hasher: &mut sha2::Sha256) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                let content = fs::read(&path).await?;
                hasher.update(&content);
            } else if metadata.is_dir() {
                self.hash_directory(&path, hasher).await?;
            }
        }
        Ok(())
    }

    /// Validate WIT schema
    async fn validate_wit_schema(&self, wit_path: &Path) -> Result<()> {
        if !wit_path.exists() {
            anyhow::bail!("WIT file not found: {}", wit_path.display());
        }

        // Basic WIT validation (in production, you might use wit-parser)
        let content = fs::read_to_string(wit_path).await?;
        
        // Check for basic WIT syntax
        if !content.contains("package") || !content.contains("interface") {
            anyhow::bail!("Invalid WIT file format");
        }

        Ok(())
    }

    /// Print status summary in a formatted table
    pub fn print_status_summary(&self, summary: &StatusSummary) {
        println!("\n🔍 Hooksmith Component Status Report");
        println!("=====================================");
        println!("Generated: {}", summary.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Build time: {:?}", summary.build_time);
        println!();

        // Print WIT Components
        if !summary.wit_components.is_empty() {
            println!("📦 WIT Components");
            println!("----------------");
            self.print_component_table(&summary.wit_components);
            println!();
        }

        // Print Native Crates
        if !summary.native_crates.is_empty() {
            println!("🦀 Native Crates");
            println!("---------------");
            self.print_component_table(&summary.native_crates);
            println!();
        }

        // Print summary statistics
        println!("📊 Summary");
        println!("----------");
        println!("Total components: {}", summary.total_components);
        println!("✅ Successful builds: {}", summary.successful_builds);
        println!("❌ Failed builds: {}", summary.failed_builds);
        println!("⏸️  Not built: {}", summary.not_built);
        
        let success_rate = if summary.total_components > 0 {
            (summary.successful_builds as f64 / summary.total_components as f64) * 100.0
        } else {
            0.0
        };
        println!("📈 Success rate: {:.1}%", success_rate);
    }

    /// Print component table
    fn print_component_table(&self, components: &[ComponentStatus]) {
        println!("{:<20} {:<12} {:<10} {:<8} {:<8} {:<8}", 
                 "Name", "Category", "Status", "Build", "Version", "Schema");
        println!("{:-<80}", "");

        for component in components {
            let status_icon = match component.build_status {
                BuildStatus::Success => "✅",
                BuildStatus::Failed => "❌",
                BuildStatus::NotBuilt => "⏸️",
                BuildStatus::Building => "🔄",
            };

            let schema_icon = match component.schema_valid {
                Some(true) => "✅",
                Some(false) => "❌",
                None => "—",
            };

            let version = component.version.as_deref().unwrap_or("—");
            
            println!("{:<20} {:<12} {:<10} {:<8} {:<8} {:<8}",
                     component.name,
                     component.category,
                     component.status,
                     status_icon,
                     version,
                     schema_icon);
        }
    }

    /// Export status to JSON
    pub fn export_status_json(&self, summary: &StatusSummary) -> Result<String> {
        serde_json::to_string_pretty(summary)
            .context("Failed to serialize status summary")
    }

    /// Export status to CSV
    pub fn export_status_csv(&self, summary: &StatusSummary) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("Name,Category,Status,Build Status,Version,Schema Valid,Errors\n");

        for component in summary.wit_components.iter().chain(summary.native_crates.iter()) {
            let build_status = match component.build_status {
                BuildStatus::Success => "Success",
                BuildStatus::Failed => "Failed",
                BuildStatus::NotBuilt => "Not Built",
                BuildStatus::Building => "Building",
            };

            let schema_valid = match component.schema_valid {
                Some(true) => "Yes",
                Some(false) => "No",
                None => "N/A",
            };

            let errors = component.errors.join("; ");
            
            csv.push_str(&format!("{},{},{},{},{},{},{}\n",
                                 component.name,
                                 component.category,
                                 component.status,
                                 build_status,
                                 component.version.as_deref().unwrap_or(""),
                                 schema_valid,
                                 errors));
        }

        Ok(csv)
    }
}

/// CLI command handler for component status
pub async fn show_component_status(verbose: bool, format: Option<&str>) -> Result<()> {
    let workspace_root = std::env::current_dir()?;
    let checker = ComponentStatusChecker::new(workspace_root, verbose).await?;

    let summary = checker.check_all_status().await?;

    match format {
        Some("json") => {
            let json = checker.export_status_json(&summary)?;
            println!("{}", json);
        }
        Some("csv") => {
            let csv = checker.export_status_csv(&summary)?;
            println!("{}", csv);
        }
        _ => {
            checker.print_status_summary(&summary);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_component_status_checker_creation() {
        let temp_dir = TempDir::new().unwrap();
        let registry_content = r#"{
            "metadata": {
                "version": "1.0.0",
                "description": "Test Registry",
                "last_updated": "2024-01-01T00:00:00Z",
                "schema_version": "1.0"
            },
            "wit_components": [],
            "native_crates": [],
            "categories": {},
            "targets": {},
            "status_definitions": {}
        }"#;

        let registry_path = temp_dir.path().join("config");
        std::fs::create_dir_all(&registry_path).unwrap();
        std::fs::write(registry_path.join("component-registry.jsonc"), registry_content).unwrap();

        let checker = ComponentStatusChecker::new(temp_dir.path().to_path_buf(), false).await;
        assert!(checker.is_ok());
    }

    #[test]
    fn test_status_summary_creation() {
        let summary = StatusSummary {
            total_components: 4,
            successful_builds: 3,
            failed_builds: 1,
            not_built: 0,
            wit_components: vec![],
            native_crates: vec![],
            build_time: Duration::from_secs(5),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(summary.total_components, 4);
        assert_eq!(summary.successful_builds, 3);
        assert_eq!(summary.failed_builds, 1);
    }
} 
