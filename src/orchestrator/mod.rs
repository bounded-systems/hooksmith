//! Hooksmith Orchestrator
//!
//! The orchestrator is the central coordination layer that manages WASM components
//! and provides a unified interface for the CLI. It handles component lifecycle,
//! communication, and configuration management.

pub mod components;
pub mod config;
pub mod event_bus;
pub mod router;
pub mod runtime;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::{component::Component, Linker};

use self::components::ComponentHandle;
use self::config::OrchestratorConfig;
use self::event_bus::{EventBusManager, EventBusManagerBuilder};
use self::router::CommandRouter;
use self::runtime::WasmRuntime;

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
    pub async fn route_event(&self, event: event_types::HooksmithEvent) -> Result<()> {
        self.event_bus.route_event(event).await
    }

    /// Register a component with the event bus
    pub async fn register_component_with_event_bus(
        &mut self,
        name: String,
        component: ComponentHandle,
    ) {
        self.event_bus
            .register_component(name.clone(), component.clone())
            .await;
        self.components.insert(name, component);
    }

    /// Register a native handler with the event bus
    pub async fn register_native_handler(
        &mut self,
        name: String,
        handler: Box<dyn event_types::EventHandler>,
    ) {
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
        _file_path: &str,
        content: &str,
        strict: bool,
        store_proof: bool,
    ) -> Result<ValidationResult> {
        use event_types::{ComputationEvent, Event, ValidationConfig as EventValidationConfig};
        use uuid::Uuid;

        let _request_id = Uuid::new_v4().to_string();
        let session_id = Some(Uuid::new_v4().to_string());

        // Create validation request event
        let validation_event = Event::computation(
            "orchestrator".to_string(),
            ComputationEvent::ValidationRequest {
                contract_name: contract_name.to_string(),
                content: content.to_string(),
                config: Some(EventValidationConfig {
                    strict: Some(strict),
                    store_proof: Some(store_proof),
                    max_errors: Some(10),
                    custom_rules: None,
                }),
            },
            session_id,
        );

        // Route the event
        self.route_event(validation_event.payload).await?;

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
        use event_types::{Event, SystemEvent};
        use uuid::Uuid;

        let _request_id = Uuid::new_v4().to_string();
        let session_id = Some(Uuid::new_v4().to_string());

        // Create file read request event
        let read_event = Event::system(
            "orchestrator".to_string(),
            SystemEvent::FileRead {
                path: file_path.to_string(),
                binary: Some(false),
            },
            session_id,
        );

        // Route the event
        self.route_event(read_event.payload).await?;

        // TODO: Wait for file read result event
        // In a real implementation, this would subscribe to the result event
        // and wait for the response

        // For now, return mock content
        Ok(format!("Mock content for file: {}", file_path))
    }

    /// Store validation proof using event-driven approach
    pub async fn store_proof_via_events(
        &self,
        _file_path: &str,
        validation_result: &ValidationResult,
    ) -> Result<()> {
        use event_types::{Event, SystemEvent};
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        let session_id = Some(Uuid::new_v4().to_string());

        // Create Git note add request event (simplified for now)
        let note_event = Event::system(
            "orchestrator".to_string(),
            SystemEvent::FileWrite {
                path: format!("validation_proof_{}.json", request_id),
                content: serde_json::to_string(validation_result)?,
                create_dirs: Some(true),
            },
            session_id,
        );

        // Route the event
        self.route_event(note_event.payload).await?;

        Ok(())
    }

    /// Initialize component linker for direct communication
    pub async fn init_component_linker(&mut self) -> Result<()> {
        let engine = &self.runtime.engine();
        let linker = Linker::new(engine);
        self.linker = Some(linker);
        Ok(())
    }

    /// Load and link components for direct communication
    pub async fn load_linked_components(&mut self) -> Result<()> {
        // Initialize linker if not already done
        if self.linker.is_none() {
            self.init_component_linker().await?;
        }

        let engine = &self.runtime.engine();
        let _linker = self.linker.as_mut().unwrap();

        // Load validation-handler component (exports validation functions)
        if let Ok(_validation_component) =
            Component::from_file(engine, "validation-handler.component.wasm")
        {
            // TODO: Fix component instantiation
            tracing::info!("Loaded validation component");
        }

        // Load contract-checker component (imports from validation-handler)
        if let Ok(checker_component) =
            Component::from_file(engine, "contract-checker.component.wasm")
        {
            // TODO: Create proper typed interface for contract-checker
            // For now, store as generic component
            self.linked_components.insert(
                "contract-checker".to_string(),
                Box::new(checker_component) as Box<dyn std::any::Any + Send + Sync>,
            );
        }

        Ok(())
    }

    /// Direct component call (fast path) for validation
    pub async fn validate_contract_direct(&self, contract_data: &str) -> Result<ValidationResult> {
        // Try direct linking first
        if self.linked_components.contains_key("contract-checker") {
            // TODO: Implement proper typed interface call
            // For now, fall back to event-driven approach
            return self
                .validate_contract_via_events("contract", "data", contract_data, true, false)
                .await;
        }

        // Fallback to event-driven approach
        self.validate_contract_via_events("contract", "data", contract_data, true, false)
            .await
    }

    /// Check if direct linking is available for a component
    pub fn has_linked_component(&self, name: &str) -> bool {
        self.linked_components.contains_key(name)
    }

    /// Get linked component count
    pub fn linked_component_count(&self) -> usize {
        self.linked_components.len()
    }

    /// List linked components
    pub fn list_linked_components(&self) -> Vec<String> {
        self.linked_components.keys().cloned().collect()
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
