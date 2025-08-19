//! Configuration Management for Hooksmith Orchestrator
//!
//! This module provides configuration management for the orchestrator
//! and its components. It handles loading, validation, and updating
//! of configuration files.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use super::runtime::RuntimeConfig;

/// Main orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestratorConfig {
    /// Runtime configuration
    pub runtime_config: RuntimeConfig,
    /// Component configurations
    pub components: HashMap<String, ComponentConfig>,
    /// Global settings
    pub settings: GlobalSettings,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Configuration for individual components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// WASM file path
    pub wasm_path: PathBuf,
    /// Whether the component is enabled
    pub enabled: bool,
    /// Component-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Dependencies on other components
    pub dependencies: Vec<String>,
}

/// Type of component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    /// Hook builder component
    HookBuilder,
    /// Worktree manager component
    WorktreeManager,
    /// Git filter component
    GitFilter,
    /// Lefthook generator component
    LefthookGenerator,
    /// Validation component
    Validation,
    /// Schema manager component
    SchemaManager,
    /// Custom component
    Custom(String),
}

/// Global settings for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Default output directory
    pub output_dir: PathBuf,
    /// Whether to enable parallel execution
    pub parallel_execution: bool,
    /// Maximum parallel tasks
    pub max_parallel_tasks: usize,
    /// Whether to enable caching
    pub enable_caching: bool,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Whether to enable telemetry
    pub enable_telemetry: bool,
    /// Telemetry endpoint
    pub telemetry_endpoint: Option<String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("target/hooksmith"),
            parallel_execution: true,
            max_parallel_tasks: 4,
            enable_caching: true,
            cache_dir: PathBuf::from(".hooksmith/cache"),
            enable_telemetry: false,
            telemetry_endpoint: None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    /// Log format
    pub format: LogFormat,
    /// Log file path (if file logging is enabled)
    pub log_file: Option<PathBuf>,
    /// Whether to enable console logging
    pub console_logging: bool,
    /// Whether to enable structured logging
    pub structured_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Text,
            log_file: None,
            console_logging: true,
            structured_logging: false,
        }
    }
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    /// Trace level logging
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging
    Error,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// Text format logging
    Text,
    /// JSON format logging
    Json,
}

/// Configuration manager
pub struct ConfigManager {
    /// Current configuration
    config: OrchestratorConfig,
    /// Configuration file path
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config: OrchestratorConfig::default(),
            config_path,
        }
    }

    /// Load configuration from file
    pub async fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).await?;
            self.config = serde_yaml::from_str(&content)?;
        }
        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_yaml::to_string(&self.config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut OrchestratorConfig {
        &mut self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: OrchestratorConfig) {
        self.config = config;
    }

    /// Add component configuration
    pub fn add_component(&mut self, component: ComponentConfig) {
        self.config
            .components
            .insert(component.name.clone(), component);
    }

    /// Remove component configuration
    pub fn remove_component(&mut self, name: &str) -> Option<ComponentConfig> {
        self.config.components.remove(name)
    }

    /// Get component configuration
    pub fn get_component(&self, name: &str) -> Option<&ComponentConfig> {
        self.config.components.get(name)
    }

    /// List all component names
    pub fn list_components(&self) -> Vec<String> {
        self.config.components.keys().cloned().collect()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Validate component configurations
        for (name, component) in &self.config.components {
            if !component.wasm_path.exists() {
                errors.push(format!(
                    "Component '{}': WASM file not found at {}",
                    name,
                    component.wasm_path.display()
                ));
            }

            // Check dependencies
            for dep in &component.dependencies {
                if !self.config.components.contains_key(dep) {
                    errors.push(format!("Component '{name}': Missing dependency '{dep}'"));
                }
            }
        }

        // Validate global settings
        if self.config.settings.max_parallel_tasks == 0 {
            errors.push("max_parallel_tasks must be greater than 0".to_string());
        }

        // Validate runtime configuration
        if self.config.runtime_config.max_memory_mb == 0 {
            errors.push("max_memory_mb must be greater than 0".to_string());
        }

        Ok(errors)
    }

    /// Create default configuration with common components
    pub fn create_default_config() -> OrchestratorConfig {
        let mut config = OrchestratorConfig::default();

        // Add hook-builder component
        config.components.insert("hook-builder".to_string(), ComponentConfig {
            name: "hook-builder".to_string(),
            component_type: ComponentType::HookBuilder,
            wasm_path: PathBuf::from("components/hook-builder/target/wasm32-unknown-unknown/release/hook_builder.wasm"),
            enabled: true,
            settings: HashMap::new(),
            dependencies: vec![],
        });

        // Add worktree-manager component
        config.components.insert("worktree-manager".to_string(), ComponentConfig {
            name: "worktree-manager".to_string(),
            component_type: ComponentType::WorktreeManager,
            wasm_path: PathBuf::from("components/worktree-manager/target/wasm32-unknown-unknown/release/worktree_manager.wasm"),
            enabled: true,
            settings: HashMap::new(),
            dependencies: vec![],
        });

        // Add lefthook-generator component
        config.components.insert("lefthook-generator".to_string(), ComponentConfig {
            name: "lefthook-generator".to_string(),
            component_type: ComponentType::LefthookGenerator,
            wasm_path: PathBuf::from("components/lefthook-generator/target/wasm32-unknown-unknown/release/lefthook_generator.wasm"),
            enabled: true,
            settings: HashMap::new(),
            dependencies: vec![],
        });

        // Add validation component
        config.components.insert(
            "validation".to_string(),
            ComponentConfig {
                name: "validation".to_string(),
                component_type: ComponentType::Validation,
                wasm_path: PathBuf::from(
                    "components/validation/target/wasm32-unknown-unknown/release/validation.wasm",
                ),
                enabled: true,
                settings: HashMap::new(),
                dependencies: vec![],
            },
        );

        config
    }
}

/// Configuration builder for creating configurations programmatically
pub struct ConfigBuilder {
    config: OrchestratorConfig,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
        }
    }

    /// Set runtime configuration
    pub fn runtime_config(mut self, runtime_config: RuntimeConfig) -> Self {
        self.config.runtime_config = runtime_config;
        self
    }

    /// Add component configuration
    pub fn component(mut self, component: ComponentConfig) -> Self {
        self.config
            .components
            .insert(component.name.clone(), component);
        self
    }

    /// Set global settings
    pub fn settings(mut self, settings: GlobalSettings) -> Self {
        self.config.settings = settings;
        self
    }

    /// Set logging configuration
    pub fn logging(mut self, logging: LoggingConfig) -> Self {
        self.config.logging = logging;
        self
    }

    /// Build the configuration
    pub fn build(self) -> OrchestratorConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut manager = ConfigManager::new(config_path.clone());

        // Test default configuration
        assert_eq!(manager.config().components.len(), 0);

        // Test adding component
        let component = ComponentConfig {
            name: "test-component".to_string(),
            component_type: ComponentType::Custom("test".to_string()),
            wasm_path: PathBuf::from("test.wasm"),
            enabled: true,
            settings: HashMap::new(),
            dependencies: vec![],
        };

        manager.add_component(component);
        assert_eq!(manager.config().components.len(), 1);
        assert!(manager.get_component("test-component").is_some());

        // Test saving and loading
        manager.save().await.unwrap();

        let mut new_manager = ConfigManager::new(config_path);
        new_manager.load().await.unwrap();
        assert_eq!(new_manager.config().components.len(), 1);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .settings(GlobalSettings {
                output_dir: PathBuf::from("custom/output"),
                ..Default::default()
            })
            .logging(LoggingConfig {
                level: LogLevel::Debug,
                ..Default::default()
            })
            .build();

        assert_eq!(config.settings.output_dir, PathBuf::from("custom/output"));
        assert!(matches!(config.logging.level, LogLevel::Debug));
    }

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.components.len(), 0); // Default config has no components
        assert!(config
            .settings
            .output_dir
            .to_string_lossy()
            .contains("hooksmith"));
        assert!(config.logging.level == LogLevel::Info);
    }
}
