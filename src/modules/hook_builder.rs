//! Hook Builder Module
//! 
//! This module provides functionality for:
//! - Compiling Rust source code into binary executables
//! - Optimizing binaries for hook execution
//! - Managing hook metadata and dependencies
//! - Integrating with WASM components

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::Command;
use tokio::fs;
use which::which;

/// Configuration for hook building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookBuildConfig {
    /// Hook name
    pub hook_name: String,
    /// Source directory containing Rust code
    pub source_dir: PathBuf,
    /// Output directory for built binaries
    pub output_dir: PathBuf,
    /// Whether to enable optimizations
    pub optimize: bool,
    /// Optimization level (0-3)
    pub optimization_level: u8,
    /// Whether to include debug symbols
    pub debug_symbols: bool,
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target: Option<String>,
    /// Additional Cargo features to enable
    pub features: Vec<String>,
    /// Whether to link with WASM components
    pub link_wasm: bool,
    /// WASM component paths to link
    pub wasm_components: Vec<PathBuf>,
    /// Environment variables for build
    pub env_vars: HashMap<String, String>,
}

impl Default for HookBuildConfig {
    fn default() -> Self {
        Self {
            hook_name: "hook".to_string(),
            source_dir: PathBuf::from("src"),
            output_dir: PathBuf::from("target/hooks"),
            optimize: true,
            optimization_level: 2,
            debug_symbols: false,
            target: None,
            features: Vec::new(),
            link_wasm: false,
            wasm_components: Vec::new(),
            env_vars: HashMap::new(),
        }
    }
}

/// Result of hook building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookBuildResult {
    /// Whether the build was successful
    pub success: bool,
    /// Path to the built binary
    pub binary_path: Option<PathBuf>,
    /// Build output messages
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Build metadata
    pub metadata: HashMap<String, String>,
    /// Build time in milliseconds
    pub build_time_ms: u64,
}

/// Hook metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    /// Hook name
    pub name: String,
    /// Hook description
    pub description: Option<String>,
    /// Hook version
    pub version: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Supported Git hooks (pre-commit, pre-push, etc.)
    pub supported_hooks: Vec<String>,
    /// Whether the hook requires WASM components
    pub requires_wasm: bool,
    /// WASM component dependencies
    pub wasm_dependencies: Vec<String>,
    /// Build timestamp
    pub build_timestamp: String,
}

/// Hook Builder
pub struct HookBuilder {
    config: HookBuildConfig,
}

impl HookBuilder {
    /// Create a new hook builder with default configuration
    pub fn new() -> Self {
        Self {
            config: HookBuildConfig::default(),
        }
    }
    
    /// Create a new hook builder with custom configuration
    pub fn with_config(config: HookBuildConfig) -> Self {
        Self { config }
    }
    
    /// Build a hook binary
    pub async fn build_hook(&self, config: Option<HookBuildConfig>) -> Result<HookBuildResult> {
        let config = config.unwrap_or_else(|| self.config.clone());
        let start_time = std::time::Instant::now();
        
        // Ensure output directory exists
        fs::create_dir_all(&config.output_dir).await
            .context("Failed to create output directory")?;
        
        // Check if Cargo is available
        self.check_cargo_available()?;
        
        // Build the hook using Cargo
        let build_output = self.run_cargo_build(&config).await?;
        
        // Copy binary to output directory
        let binary_path = self.copy_binary_to_output(&config).await?;
        
        // Generate metadata
        let _metadata = self.generate_metadata(&config).await?;
        
        // Optimize binary if requested
        if config.optimize {
            self.optimize_binary(&binary_path).await?;
        }
        
        let build_time = start_time.elapsed();
        
        let binary_size = fs::metadata(&binary_path).await?.len();

        Ok(HookBuildResult {
            success: true,
            binary_path: Some(binary_path),
            output: build_output,
            error: None,
            metadata: HashMap::from([
                ("build_time_ms".to_string(), build_time.as_millis().to_string()),
                ("binary_size".to_string(), binary_size.to_string()),
            ]),
            build_time_ms: build_time.as_millis() as u64,
        })
    }
    
    /// Build multiple hooks
    pub async fn build_hooks(&self, hook_configs: Vec<HookBuildConfig>) -> Result<Vec<HookBuildResult>> {
        let mut results = Vec::new();
        
        for config in hook_configs {
            let result = self.build_hook(Some(config)).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Clean build artifacts
    pub async fn clean_build(&self, build_path: &Path) -> Result<()> {
        if build_path.exists() {
            fs::remove_dir_all(build_path).await
                .context("Failed to remove build directory")?;
        }
        
        // Also clean Cargo target directory
        let cargo_target = PathBuf::from("target");
        if cargo_target.exists() {
            let status = Command::new("cargo")
                .arg("clean")
                .status()
                .context("Failed to run cargo clean")?;
            
            if !status.success() {
                anyhow::bail!("Cargo clean failed with status: {}", status);
            }
        }
        
        Ok(())
    }
    
    /// Get available targets
    pub fn get_available_targets(&self) -> Result<Vec<String>> {
        let output = Command::new("rustup")
            .arg("target")
            .arg("list")
            .output()
            .context("Failed to run rustup target list")?;
        
        if !output.status.success() {
            anyhow::bail!("rustup target list failed");
        }
        
        let targets: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .filter(|line| !line.contains("(default)"))
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .filter(|target| !target.is_empty())
            .collect();
        
        Ok(targets)
    }
    
    /// Check if target is supported
    pub fn is_target_supported(&self, target: &str) -> Result<bool> {
        let targets = self.get_available_targets()?;
        Ok(targets.contains(&target.to_string()))
    }
    
    /// Get build information
    pub fn get_build_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        
        // Get Rust version
        if let Ok(output) = Command::new("rustc").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8(output.stdout)?;
                info.insert("rust_version".to_string(), version.trim().to_string());
            }
        }
        
        // Get Cargo version
        if let Ok(output) = Command::new("cargo").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8(output.stdout)?;
                info.insert("cargo_version".to_string(), version.trim().to_string());
            }
        }
        
        // Get target triple
        if let Ok(output) = Command::new("rustc").arg("--print").arg("target-triple").output() {
            if output.status.success() {
                let target = String::from_utf8(output.stdout)?;
                info.insert("target_triple".to_string(), target.trim().to_string());
            }
        }
        
        Ok(info)
    }
    
    // Private helper methods
    
    fn check_cargo_available(&self) -> Result<()> {
        which("cargo").context("Cargo is not available in PATH")?;
        Ok(())
    }
    
    async fn run_cargo_build(&self, config: &HookBuildConfig) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        // Set optimization level
        if config.optimize {
            cmd.arg("--release");
        }
        
        // Set target if specified
        if let Some(target) = &config.target {
            cmd.args(&["--target", target]);
        }
        
        // Add features
        if !config.features.is_empty() {
            cmd.args(&["--features", &config.features.join(",")]);
        }
        
        // Set working directory
        cmd.current_dir(&config.source_dir);
        
        // Add environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }
        
        let output = cmd.output()
            .context("Failed to run cargo build")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Cargo build failed: {}", error);
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    async fn copy_binary_to_output(&self, config: &HookBuildConfig) -> Result<PathBuf> {
        // Determine source binary path
        let target_dir = if config.optimize { "release" } else { "debug" };
        let source_binary = if let Some(target) = &config.target {
            PathBuf::from("target").join(target).join(target_dir).join(&config.hook_name)
        } else {
            PathBuf::from("target").join(target_dir).join(&config.hook_name)
        };
        
        // Add .exe extension on Windows
        let source_binary = if cfg!(target_os = "windows") && !source_binary.extension().is_some() {
            source_binary.with_extension("exe")
        } else {
            source_binary
        };
        
        // Copy to output directory
        let output_binary = config.output_dir.join(&config.hook_name);
        let output_binary = if cfg!(target_os = "windows") && !output_binary.extension().is_some() {
            output_binary.with_extension("exe")
        } else {
            output_binary
        };
        
        fs::copy(&source_binary, &output_binary).await
            .context("Failed to copy binary to output directory")?;
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_binary).await?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_binary, perms).await?;
        }
        
        Ok(output_binary)
    }
    
    async fn generate_metadata(&self, config: &HookBuildConfig) -> Result<HookMetadata> {
        let metadata = HookMetadata {
            name: config.hook_name.clone(),
            description: None,
            version: Some("0.1.0".to_string()),
            author: None,
            dependencies: Vec::new(),
            supported_hooks: vec!["pre-commit".to_string(), "pre-push".to_string()],
            requires_wasm: config.link_wasm,
            wasm_dependencies: config.wasm_components
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect(),
            build_timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Write metadata to file
        let metadata_path = config.output_dir.join(format!("{}.json", config.hook_name));
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, metadata_json).await?;
        
        Ok(metadata)
    }
    
    async fn optimize_binary(&self, binary_path: &Path) -> Result<()> {
        // This is a placeholder for binary optimization
        // In a real implementation, you might:
        // - Strip debug symbols
        // - Compress the binary
        // - Apply additional optimizations
        
        println!("Optimizing binary: {:?}", binary_path);
        Ok(())
    }
}

/// Install hooks into Git repository
pub async fn install_hooks(hooks_dir: &Path, hook_binaries: &[PathBuf]) -> Result<()> {
    // Ensure hooks directory exists
    fs::create_dir_all(hooks_dir).await
        .context("Failed to create hooks directory")?;
    
    for binary in hook_binaries {
        let hook_name = binary.file_name()
            .and_then(|n| n.to_str())
            .context("Invalid hook binary name")?;
        
        let hook_path = hooks_dir.join(hook_name);
        
        // Copy binary to hooks directory
        fs::copy(binary, &hook_path).await
            .context(format!("Failed to copy hook binary: {:?}", binary))?;
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path).await?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).await?;
        }
        
        println!("Installed hook: {}", hook_name);
    }
    
    Ok(())
}

/// List available hooks
pub async fn list_hooks(hooks_dir: &Path) -> Result<Vec<HookMetadata>> {
    let mut hooks = Vec::new();
    
    if !hooks_dir.exists() {
        return Ok(hooks);
    }
    
    let mut entries = fs::read_dir(hooks_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_file() {
            // Try to read metadata file
            let metadata_path = path.with_extension("json");
            if let Ok(metadata_content) = fs::read_to_string(&metadata_path).await {
                if let Ok(metadata) = serde_json::from_str::<HookMetadata>(&metadata_content) {
                    hooks.push(metadata);
                }
            } else {
                // Create basic metadata from file info
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let metadata = HookMetadata {
                    name,
                    description: None,
                    version: None,
                    author: None,
                    dependencies: Vec::new(),
                    supported_hooks: Vec::new(),
                    requires_wasm: false,
                    wasm_dependencies: Vec::new(),
                    build_timestamp: chrono::Utc::now().to_rfc3339(),
                };
                
                hooks.push(metadata);
            }
        }
    }
    
    Ok(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_hook_builder_creation() {
        let builder = HookBuilder::new();
        assert!(builder.get_build_info().is_ok());
    }
    
    #[tokio::test]
    async fn test_available_targets() {
        let builder = HookBuilder::new();
        let targets = builder.get_available_targets();
        // This might fail if rustup is not available, but that's okay
        if let Ok(targets) = targets {
            assert!(!targets.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_list_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = list_hooks(temp_dir.path()).await;
        assert!(hooks.is_ok());
        assert!(hooks.unwrap().is_empty());
    }
}
