//! Hooksmith Orchestrator
//!
//! The orchestrator is the central coordination layer that manages WASM components
//! and provides a unified interface for the CLI. It handles component lifecycle,
//! communication, and configuration management.

pub mod components;
pub mod config;
pub mod router;
pub mod runtime;
pub mod event_bus;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::{Engine, Linker, Store, Component};

use self::components::ComponentHandle;
use self::config::OrchestratorConfig;
use self::router::CommandRouter;
use self::runtime::WasmRuntime;
use self::event_bus::{EventBusManager, EventBusManagerBuilder};

// Import types from components for now (in a real implementation, these would come from WIT)
/// Metadata about a build operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Rust version used for the build
    pub rust_version: String,
    /// Cargo version used for the build
    pub cargo_version: String,
    /// Target triple used for the build
    pub target_triple: String,
    /// Build timestamp
    pub timestamp: String,
    /// Build hash for caching
    pub build_hash: String,
}

/// Main orchestrator for Hooksmith
///
/// This orchestrator manages all WASM components and provides a unified
/// interface for the CLI to interact with them.
pub struct HooksmithOrchestrator {
    /// WASM runtime for component execution
    runtime: WasmRuntime,
    /// Command router for CLI command handling
    router: CommandRouter,
    /// Configuration management
    config: OrchestratorConfig,
    /// Loaded component handles
    components: HashMap<String, ComponentHandle>,
    /// Event bus manager for event-driven communication
    event_bus: EventBusManager,
    /// Linked components for direct communication (fast path)
    linked_components: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    /// Component linker for direct linking
    linker: Option<Linker<()>>,
}

impl HooksmithOrchestrator {
    /// Create a new orchestrator with default configuration
    pub async fn new() -> Result<Self> {
        let config = OrchestratorConfig::default();
        let runtime = WasmRuntime::new(&config.runtime_config).await?;
        let router = CommandRouter::new();
        let event_bus = EventBusManagerBuilder::default().build()?;

        Ok(Self {
            runtime,
            router,
            config,
            components: HashMap::new(),
            event_bus,
            linked_components: HashMap::new(),
            linker: None,
        })
    }

    /// Create a new orchestrator with custom configuration
    pub async fn with_config(config: OrchestratorConfig) -> Result<Self> {
        let runtime = WasmRuntime::new(&config.runtime_config).await?;
        let router = CommandRouter::new();
        let event_bus = EventBusManagerBuilder::default().build()?;

        Ok(Self {
            runtime,
            router,
            config,
            components: HashMap::new(),
            event_bus,
            linked_components: HashMap::new(),
            linker: None,
        })
    }

    /// Load a component from a WASM file
    pub async fn load_component(&mut self, name: &str, wasm_path: PathBuf) -> Result<()> {
        let handle = self.runtime.load_component(name, wasm_path).await?;
        self.components.insert(name.to_string(), handle);
        Ok(())
    }

    /// Get a component handle by name
    pub fn get_component(&self, name: &str) -> Result<&ComponentHandle> {
        self.components
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Component '{}' not loaded", name))
    }

    /// Execute a command through the router
    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<CommandResult> {
        self.router.execute(command, args, &self.components).await
    }

    /// Build a hook using the hook-builder component
    pub async fn build_hook(&self, config: BuildConfig) -> Result<BuildResult> {
        let hook_builder = self.get_component("hook-builder")?;
        let result = hook_builder.call("build-hook", config).await?;
        // TODO: Deserialize the result into BuildResult
        Ok(BuildResult {
            success: result.success,
            binary_path: None,
            artifacts: vec![],
            build_logs: result.return_value.unwrap_or_default(),
            error: result.error,
            duration_ms: result.duration_ms,
            binary_size: None,
            metadata: BuildMetadata {
                rust_version: "unknown".to_string(),
                cargo_version: "unknown".to_string(),
                target_triple: "unknown".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                build_hash: "unknown".to_string(),
            },
        })
    }

    /// Generate Lefthook configuration using the lefthook-generator component
    pub async fn generate_lefthook_config(&self, config: LefthookConfig) -> Result<LefthookResult> {
        let generator = self.get_component("lefthook-generator")?;
        let result = generator.call("generate-config", config).await?;
        // TODO: Deserialize the result into LefthookResult
        Ok(LefthookResult {
            success: result.success,
            config_content: result.return_value,
            output_path: None,
            error: result.error,
        })
    }

    /// Manage worktrees using the worktree-manager component
    pub async fn manage_worktree(&self, operation: WorktreeOperation) -> Result<WorktreeResult> {
        let manager = self.get_component("worktree-manager")?;
        let result = manager.call("execute-operation", operation).await?;
        // TODO: Deserialize the result into WorktreeResult
        Ok(WorktreeResult {
            success: result.success,
            output: result.return_value.unwrap_or_default(),
            error: result.error,
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Validate configuration using the validation component
    pub async fn validate_config(&self, config: ValidationConfig) -> Result<ValidationResult> {
        let validator = self.get_component("validation")?;
        let result = validator.call("validate", config).await?;
        // TODO: Deserialize the result into ValidationResult
        Ok(ValidationResult {
            success: result.success,
            errors: vec![],
            warnings: vec![],
            details: result.return_value,
        })
    }

    /// Get orchestrator configuration
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Update orchestrator configuration
    pub fn update_config(&mut self, config: OrchestratorConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// List all loaded components
    pub fn list_components(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }

    /// Check if a component is loaded
    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// Route an event through the event bus
    pub async fn route_event(&self, event: crate::xtask::event_bus::HooksmithEvent) -> Result<()> {
        self.event_bus.route_event(event).await
    }

    /// Register a component with the event bus
    pub async fn register_component_with_event_bus(&mut self, name: String, component: ComponentHandle) {
        self.event_bus.register_component(name.clone(), component.clone()).await;
        self.components.insert(name, component);
    }

    /// Register a native handler with the event bus
    pub async fn register_native_handler(&mut self, name: String, handler: Box<dyn crate::xtask::event_bus::EventHandler>) {
        self.event_bus.register_native_handler(name, handler).await;
    }

    /// Get event bus statistics
    pub async fn get_event_bus_statistics(&self) -> Result<event_bus::EventBusStatistics> {
        Ok(self.event_bus.get_statistics().await)
    }

    /// Validate a contract using event-driven approach
    pub async fn validate_contract_via_events(
        &self,
        contract_name: &str,
        file_path: &str,
        content: &str,
        strict: bool,
        store_proof: bool,
    ) -> Result<ValidationResult> {
        use crate::xtask::event_bus::HooksmithEvent;
        use serde_json::json;
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        let session_id = self.config.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create validation request event
        let validation_event = HooksmithEvent::new(
            "orchestrator".to_string(),
            "validation_request".to_string(),
            json!({
                "request_id": request_id,
                "contract_name": contract_name,
                "file_path": file_path,
                "content": content,
                "validation_config": {
                    "strict": strict,
                    "store_proof": store_proof,
                    "rules": ["json_schema", "content_validation"]
                },
                "metadata": {
                    "session_id": session_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "component": "validation-handler"
                }
            }),
        )
        .with_session_id(session_id);

        // Route the event
        self.route_event(validation_event).await?;

        // TODO: Wait for validation result event
        // In a real implementation, this would subscribe to the result event
        // and wait for the response

        // For now, return a mock result
        Ok(ValidationResult {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            details: Some("Mock validation result".to_string()),
        })
    }

    /// Read file content using event-driven approach
    pub async fn read_file_via_events(&self, file_path: &str) -> Result<String> {
        use crate::xtask::event_bus::HooksmithEvent;
        use serde_json::json;
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        let session_id = self.config.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create file read request event
        let read_event = HooksmithEvent::new(
            "orchestrator".to_string(),
            "file_read_request".to_string(),
            json!({
                "request_id": request_id,
                "path": file_path,
                "encoding": "utf8",
                "metadata": {
                    "session_id": session_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "working_directory": std::env::current_dir()?.to_string_lossy()
                }
            }),
        )
        .with_session_id(session_id);

        // Route the event
        self.route_event(read_event).await?;

        // TODO: Wait for file read result event
        // In a real implementation, this would subscribe to the result event
        // and wait for the response

        // For now, return mock content
        Ok(format!("Mock content for file: {}", file_path))
    }

    /// Store validation proof using event-driven approach
    pub async fn store_proof_via_events(&self, file_path: &str, validation_result: &ValidationResult) -> Result<()> {
        use crate::xtask::event_bus::HooksmithEvent;
        use serde_json::json;
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        let session_id = self.config.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create Git note add request event
        let note_event = HooksmithEvent::new(
            "orchestrator".to_string(),
            "git_note_add_request".to_string(),
            json!({
                "request_id": request_id,
                "note": {
                    "object": "HEAD",
                    "message": format!("Validation proof for {}", file_path),
                    "file": "validation_proof.json"
                },
                "metadata": {
                    "session_id": session_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "repository": std::env::current_dir()?.to_string_lossy()
                }
            }),
        )
        .with_session_id(session_id);

        // Route the event
        self.route_event(note_event).await?;

        Ok(())
    }
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether the command was successful
    pub success: bool,
    /// Output from the command
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

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
}

/// Result of a hook build operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Whether the build was successful
    pub success: bool,
    /// Path to the built binary
    pub binary_path: Option<String>,
    /// Build artifacts
    pub artifacts: Vec<String>,
    /// Build logs
    pub build_logs: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Build duration in milliseconds
    pub duration_ms: u64,
    /// Binary size in bytes
    pub binary_size: Option<u64>,
    /// Build metadata
    pub metadata: BuildMetadata,
}

/// Configuration for Lefthook generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookConfig {
    /// Output path for the configuration file
    pub output_path: String,
    /// Hook configurations to include
    pub hooks: Vec<HookConfig>,
    /// Whether to validate against schema
    pub validate_schema: bool,
}

/// Hook configuration for Lefthook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Hook name
    pub name: String,
    /// Hook type (pre-commit, pre-push, etc.)
    pub hook_type: String,
    /// Command to execute
    pub command: String,
    /// Whether the hook is enabled
    pub enabled: bool,
}

/// Result of Lefthook configuration generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookResult {
    /// Whether generation was successful
    pub success: bool,
    /// Generated configuration content
    pub config_content: Option<String>,
    /// Output file path
    pub output_path: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Worktree operation to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorktreeOperation {
    /// Create a new worktree
    Create {
        /// Branch name for the new worktree
        branch_name: String,
        /// Base path for the worktree
        base_path: Option<String>,
        /// Tool to use for creation
        tool: Option<String>,
    },
    /// List all worktrees
    List {
        /// Tool to use for listing
        tool: Option<String>,
    },
    /// Switch to a worktree
    Switch {
        /// Name of the worktree to switch to
        worktree_name: String,
        /// Tool to use for switching
        tool: Option<String>,
    },
    /// Remove a worktree
    Remove {
        /// Name of the worktree to remove
        worktree_name: String,
        /// Whether to also remove the branch
        with_branch: bool,
        /// Tool to use for removal
        tool: Option<String>,
    },
}

/// Result of a worktree operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Output from the operation
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Worktree path if created
    pub worktree_path: Option<String>,
    /// Branch name if created
    pub branch_name: Option<String>,
}

/// Configuration for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Type of validation to perform
    pub validation_type: ValidationType,
    /// Data to validate
    pub data: String,
    /// Schema to validate against (if applicable)
    pub schema: Option<String>,
}

/// Type of validation to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// Validate Lefthook configuration
    LefthookConfig,
    /// Validate hook configuration
    HookConfig,
    /// Validate WIT interface
    WitInterface,
    /// Validate build configuration
    BuildConfig,
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation was successful
    pub success: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation details
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = HooksmithOrchestrator::new().await;
        assert!(orchestrator.is_ok());
    }

    #[tokio::test]
    async fn test_orchestrator_config() {
        let config = OrchestratorConfig::default();
        let orchestrator = HooksmithOrchestrator::with_config(config).await;
        assert!(orchestrator.is_ok());
    }
}
