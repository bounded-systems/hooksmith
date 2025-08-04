//! Hook Builder Module
//!
//! This module provides the core functionality for building Rust hooks into binary executables.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use which::which;

/// Configuration for building a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Source path for the hook
    pub source_path: String,
    /// Output path for the built binary
    pub output_path: String,
    /// Target triple for compilation
    pub target_triple: Option<String>,
    /// Optimization level (0-3)
    pub optimization_level: u8,
    /// Whether to include debug symbols
    pub debug_symbols: bool,
    /// Additional cargo features to enable
    pub features: Vec<String>,
    /// Whether to enable all features
    pub all_features: bool,
    /// Additional environment variables
    pub env_vars: Vec<EnvVar>,
    /// Whether to run tests after building
    pub run_tests: bool,
    /// Whether to run clippy checks
    pub run_clippy: bool,
}

/// Environment variable for build process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
}

/// Result of a hook build operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Whether the build was successful
    pub success: bool,
    /// Path to the built binary
    pub binary_path: Option<String>,
    /// Build artifacts (additional files)
    pub artifacts: Vec<String>,
    /// Build logs
    pub build_logs: String,
    /// Error message if build failed
    pub error: Option<String>,
    /// Build duration in milliseconds
    pub duration_ms: u64,
    /// Binary size in bytes
    pub binary_size: Option<u64>,
    /// Build metadata
    pub metadata: BuildMetadata,
}

/// Metadata about the build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Rust version used
    pub rust_version: String,
    /// Cargo version used
    pub cargo_version: String,
    /// Target triple used
    pub target_triple: String,
    /// Build timestamp
    pub timestamp: String,
    /// Build hash for caching
    pub build_hash: String,
}

/// Hook Builder
pub struct HookBuilder {
    /// Build cache
    cache: HashMap<String, BuildResult>,
    /// Build configuration
    config: BuilderConfig,
}

/// Builder configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BuilderConfig {
    /// Whether to enable caching
    pub enable_caching: bool,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Whether to enable parallel builds
    pub parallel_builds: bool,
    /// Maximum parallel builds
    pub max_parallel_builds: usize,
    /// Whether to enable verbose output
    pub verbose: bool,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_dir: PathBuf::from(".hooksmith/cache"),
            parallel_builds: true,
            max_parallel_builds: 4,
            verbose: false,
        }
    }
}

impl HookBuilder {
    /// Create a new hook builder
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            config: BuilderConfig::default(),
        }
    }

    /// Create a new hook builder with custom configuration
    #[allow(dead_code)]
    pub fn with_config(config: BuilderConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
        }
    }

    /// Build a hook from source
    pub async fn build_hook(&self, config: BuildConfig) -> Result<BuildResult> {
        let start_time = std::time::Instant::now();

        // Check cache if enabled
        if self.config.enable_caching {
            if let Some(cached_result) = self.cache.get(&config.source_path) {
                return Ok(cached_result.clone());
            }
        }

        // Validate source path
        if !PathBuf::from(&config.source_path).exists() {
            return Ok(BuildResult {
                success: false,
                binary_path: None,
                artifacts: vec![],
                build_logs: "".to_string(),
                error: Some(format!(
                    "Source path does not exist: {}",
                    config.source_path
                )),
                duration_ms: 0,
                binary_size: None,
                metadata: self.get_default_metadata(),
            });
        }

        // Create output directory
        let output_path = PathBuf::from(&config.output_path);
        let output_dir = output_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid output path"))?;
        fs::create_dir_all(output_dir)?;

        // Build the hook
        let result = self.execute_build(&config).await?;

        // Calculate duration
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Get binary size if build was successful
        let binary_size = if result.success {
            if let Some(ref binary_path) = result.binary_path {
                fs::metadata(binary_path).map(|m| m.len()).ok()
            } else {
                None
            }
        } else {
            None
        };

        let final_result = BuildResult {
            duration_ms,
            binary_size,
            ..result
        };

        // Cache the result if enabled
        if self.config.enable_caching {
            // Note: In a real implementation, you'd want to store this in a persistent cache
            // For now, we'll just keep it in memory
        }

        Ok(final_result)
    }

    /// Execute the actual build process
    async fn execute_build(&self, config: &BuildConfig) -> Result<BuildResult> {
        let mut build_logs = String::new();

        // Check if cargo is available
        if which("cargo").is_err() {
            return Ok(BuildResult {
                success: false,
                binary_path: None,
                artifacts: vec![],
                build_logs: "".to_string(),
                error: Some("Cargo is not available on the system".to_string()),
                duration_ms: 0,
                binary_size: None,
                metadata: self.get_default_metadata(),
            });
        }

        // Build cargo command
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.arg("build");

        // Set target if specified
        if let Some(ref target) = config.target_triple {
            cargo_cmd.args(["--target", target]);
        }

        // Set optimization level
        let profile = match config.optimization_level {
            0 => "dev",
            1 => "release",
            2 => "release",
            3 => "release",
            _ => "release",
        };
        cargo_cmd.arg(format!("--{profile}"));

        // Set features
        if config.all_features {
            cargo_cmd.arg("--all-features");
        } else if !config.features.is_empty() {
            cargo_cmd.arg("--features");
            cargo_cmd.arg(config.features.join(","));
        }

        // Set environment variables
        for env_var in &config.env_vars {
            cargo_cmd.env(&env_var.name, &env_var.value);
        }

        // Set working directory to source path
        cargo_cmd.current_dir(&config.source_path);

        // Execute the build
        let output = cargo_cmd.output()?;

        // Capture build logs
        build_logs.push_str(&String::from_utf8_lossy(&output.stdout));
        build_logs.push_str(&String::from_utf8_lossy(&output.stderr));

        if output.status.success() {
            // Build successful
            let binary_path = self.find_built_binary(config).await?;
            let artifacts = self.collect_artifacts(config).await?;

            Ok(BuildResult {
                success: true,
                binary_path,
                artifacts,
                build_logs,
                error: None,
                duration_ms: 0,
                binary_size: None,
                metadata: self.get_build_metadata(config).await?,
            })
        } else {
            // Build failed
            Ok(BuildResult {
                success: false,
                binary_path: None,
                artifacts: vec![],
                build_logs,
                error: Some("Cargo build failed".to_string()),
                duration_ms: 0,
                binary_size: None,
                metadata: self.get_default_metadata(),
            })
        }
    }

    /// Find the built binary
    async fn find_built_binary(&self, config: &BuildConfig) -> Result<Option<String>> {
        // This is a simplified implementation
        // In a real implementation, you'd parse Cargo.toml and find the actual binary path
        let target_dir = if let Some(ref target) = config.target_triple {
            format!("target/{target}/release")
        } else {
            "target/release".to_string()
        };

        let source_dir = PathBuf::from(&config.source_path);
        let binary_name = source_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("hook");

        let binary_path = source_dir.join(&target_dir).join(binary_name);

        if binary_path.exists() {
            Ok(Some(binary_path.to_string_lossy().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Collect build artifacts
    async fn collect_artifacts(&self, _config: &BuildConfig) -> Result<Vec<String>> {
        // This is a simplified implementation
        // In a real implementation, you'd collect all relevant artifacts
        Ok(vec![])
    }

    /// Get build metadata
    async fn get_build_metadata(&self, config: &BuildConfig) -> Result<BuildMetadata> {
        let rust_version = self.get_rust_version().await?;
        let cargo_version = self.get_cargo_version().await?;
        let target_triple = config
            .target_triple
            .clone()
            .unwrap_or_else(|| "x86_64-unknown-linux-gnu".to_string());
        let timestamp = chrono::Utc::now().to_rfc3339();
        let build_hash = self.calculate_build_hash(config).await?;

        Ok(BuildMetadata {
            rust_version,
            cargo_version,
            target_triple,
            timestamp,
            build_hash,
        })
    }

    /// Get default metadata
    fn get_default_metadata(&self) -> BuildMetadata {
        BuildMetadata {
            rust_version: "unknown".to_string(),
            cargo_version: "unknown".to_string(),
            target_triple: "unknown".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            build_hash: "unknown".to_string(),
        }
    }

    /// Get Rust version
    async fn get_rust_version(&self) -> Result<String> {
        let output = Command::new("rustc").arg("--version").output()?;
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Get Cargo version
    async fn get_cargo_version(&self) -> Result<String> {
        let output = Command::new("cargo").arg("--version").output()?;
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Calculate build hash
    async fn calculate_build_hash(&self, config: &BuildConfig) -> Result<String> {
        // This is a simplified implementation
        // In a real implementation, you'd hash the source files and configuration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        config.source_path.hash(&mut hasher);
        config.optimization_level.hash(&mut hasher);
        config.debug_symbols.hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Get build information
    pub fn get_build_info(&self) -> Result<BuildMetadata> {
        Ok(self.get_default_metadata())
    }

    /// Clean build artifacts
    pub fn clean_build(&self, _build_path: &str) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you'd clean the build artifacts
        Ok(())
    }

    /// Get available targets
    pub fn get_available_targets(&self) -> Result<Vec<String>> {
        // This is a simplified implementation
        // In a real implementation, you'd query rustup for available targets
        Ok(vec![
            "x86_64-unknown-linux-gnu".to_string(),
            "x86_64-apple-darwin".to_string(),
            "x86_64-pc-windows-msvc".to_string(),
        ])
    }

    /// Check if target is supported
    pub fn is_target_supported(&self, target: &str) -> Result<bool> {
        let available_targets = self.get_available_targets()?;
        Ok(available_targets.contains(&target.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_builder_creation() {
        let builder = HookBuilder::new();
        assert!(builder.cache.is_empty());
    }

    #[tokio::test]
    async fn test_build_config() {
        let config = BuildConfig {
            source_path: "test/path".to_string(),
            output_path: "test/output".to_string(),
            target_triple: None,
            optimization_level: 2,
            debug_symbols: false,
            features: vec![],
            all_features: false,
            env_vars: vec![],
            run_tests: false,
            run_clippy: false,
        };

        assert_eq!(config.source_path, "test/path");
        assert_eq!(config.optimization_level, 2);
    }

    #[tokio::test]
    async fn test_get_available_targets() {
        let builder = HookBuilder::new();
        let targets = builder.get_available_targets().unwrap();
        assert!(!targets.is_empty());
    }
}
