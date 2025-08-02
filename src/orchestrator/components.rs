//! Component Handle for Hooksmith
//!
//! This module provides a unified interface for interacting with WASM components.
//! It abstracts away the complexity of WASM function calls and provides a
//! type-safe interface for component communication.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmtime::Engine;

use super::runtime::{WasmCallRequest, WasmCallResult};

/// Handle for interacting with a WASM component
#[derive(Clone)]
pub struct ComponentHandle {
    /// Component name
    name: String,
    /// WASM engine reference
    #[allow(dead_code)]
    engine: Engine,
    /// Component metadata
    metadata: ComponentMetadata,
}

/// Metadata about a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Available functions
    pub functions: Vec<FunctionMetadata>,
    /// Component capabilities
    pub capabilities: Vec<String>,
    /// Component dependencies
    pub dependencies: Vec<String>,
}

/// Metadata about a component function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Parameter types
    pub parameters: Vec<ParameterMetadata>,
    /// Return type
    pub return_type: Option<String>,
    /// Whether the function is async
    pub is_async: bool,
}

/// Metadata about a function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMetadata {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub parameter_type: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter description
    pub description: String,
}

impl ComponentHandle {
    /// Create a new component handle
    pub fn new(name: String, engine: Engine) -> Self {
        let name_clone = name.clone();
        Self {
            name,
            engine,
            metadata: ComponentMetadata {
                name: name_clone,
                version: "0.1.0".to_string(),
                functions: vec![],
                capabilities: vec![],
                dependencies: vec![],
            },
        }
    }

    /// Get component name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get component metadata
    pub fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    /// Call a function on the component
    pub async fn call<T>(&self, function_name: &str, args: T) -> Result<WasmCallResult>
    where
        T: Serialize,
    {
        let args_json = serde_json::to_string(&args)?;
        let _request = WasmCallRequest {
            function_name: function_name.to_string(),
            arguments: vec![args_json],
            return_type: None,
        };

        // TODO: Implement actual WASM function call
        // For now, return a mock result
        Ok(WasmCallResult {
            success: true,
            return_value: Some(format!("Mock result from {}:{}", self.name, function_name)),
            error: None,
            duration_ms: 0,
        })
    }

    /// Call a function with multiple arguments
    pub async fn call_with_args(
        &self,
        function_name: &str,
        args: Vec<String>,
    ) -> Result<WasmCallResult> {
        let _request = WasmCallRequest {
            function_name: function_name.to_string(),
            arguments: args,
            return_type: None,
        };

        // TODO: Implement actual WASM function call
        Ok(WasmCallResult {
            success: true,
            return_value: Some(format!("Mock result from {}:{}", self.name, function_name)),
            error: None,
            duration_ms: 0,
        })
    }

    /// Get available functions
    pub fn available_functions(&self) -> Vec<String> {
        self.metadata
            .functions
            .iter()
            .map(|f| f.name.clone())
            .collect()
    }

    /// Check if a function is available
    pub fn has_function(&self, function_name: &str) -> bool {
        self.metadata
            .functions
            .iter()
            .any(|f| f.name == function_name)
    }

    /// Get function metadata
    pub fn get_function_metadata(&self, function_name: &str) -> Option<&FunctionMetadata> {
        self.metadata
            .functions
            .iter()
            .find(|f| f.name == function_name)
    }

    /// Update component metadata
    pub fn update_metadata(&mut self, metadata: ComponentMetadata) {
        self.metadata = metadata;
    }
}

/// Component registry for managing multiple components
pub struct ComponentRegistry {
    /// Registered components
    components: HashMap<String, ComponentHandle>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, component: ComponentHandle) {
        let name = component.name().to_string();
        self.components.insert(name, component);
    }

    /// Get a component by name
    pub fn get(&self, name: &str) -> Option<&ComponentHandle> {
        self.components.get(name)
    }

    /// Get a mutable reference to a component
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ComponentHandle> {
        self.components.get_mut(name)
    }

    /// Remove a component
    pub fn remove(&mut self, name: &str) -> Option<ComponentHandle> {
        self.components.remove(name)
    }

    /// List all registered components
    pub fn list(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }

    /// Check if a component is registered
    pub fn has(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// Get component count
    pub fn count(&self) -> usize {
        self.components.len()
    }
}

/// Component factory for creating components
pub struct ComponentFactory {
    /// Engine for creating components
    engine: Engine,
}

impl ComponentFactory {
    /// Create a new component factory
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }

    /// Create a hook builder component
    pub fn create_hook_builder(&self) -> ComponentHandle {
        let mut handle = ComponentHandle::new("hook-builder".to_string(), self.engine.clone());

        let metadata = ComponentMetadata {
            name: "hook-builder".to_string(),
            version: "0.1.0".to_string(),
            functions: vec![
                FunctionMetadata {
                    name: "build-hook".to_string(),
                    description: "Build a hook from source".to_string(),
                    parameters: vec![ParameterMetadata {
                        name: "config".to_string(),
                        parameter_type: "BuildConfig".to_string(),
                        required: true,
                        description: "Build configuration".to_string(),
                    }],
                    return_type: Some("BuildResult".to_string()),
                    is_async: true,
                },
                FunctionMetadata {
                    name: "validate-source".to_string(),
                    description: "Validate hook source code".to_string(),
                    parameters: vec![ParameterMetadata {
                        name: "source-path".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                        description: "Path to source code".to_string(),
                    }],
                    return_type: Some("ValidationResult".to_string()),
                    is_async: false,
                },
            ],
            capabilities: vec![
                "rust-compilation".to_string(),
                "binary-optimization".to_string(),
            ],
            dependencies: vec![],
        };

        handle.update_metadata(metadata);
        handle
    }

    /// Create a worktree manager component
    pub fn create_worktree_manager(&self) -> ComponentHandle {
        let mut handle = ComponentHandle::new("worktree-manager".to_string(), self.engine.clone());

        let metadata = ComponentMetadata {
            name: "worktree-manager".to_string(),
            version: "0.1.0".to_string(),
            functions: vec![
                FunctionMetadata {
                    name: "execute-operation".to_string(),
                    description: "Execute a worktree operation".to_string(),
                    parameters: vec![ParameterMetadata {
                        name: "operation".to_string(),
                        parameter_type: "WorktreeOperation".to_string(),
                        required: true,
                        description: "Operation to execute".to_string(),
                    }],
                    return_type: Some("WorktreeResult".to_string()),
                    is_async: true,
                },
                FunctionMetadata {
                    name: "list-tools".to_string(),
                    description: "List available worktree tools".to_string(),
                    parameters: vec![],
                    return_type: Some("list<string>".to_string()),
                    is_async: false,
                },
            ],
            capabilities: vec!["git-worktree".to_string(), "tool-detection".to_string()],
            dependencies: vec![],
        };

        handle.update_metadata(metadata);
        handle
    }

    /// Create a lefthook generator component
    pub fn create_lefthook_generator(&self) -> ComponentHandle {
        let mut handle =
            ComponentHandle::new("lefthook-generator".to_string(), self.engine.clone());

        let metadata = ComponentMetadata {
            name: "lefthook-generator".to_string(),
            version: "0.1.0".to_string(),
            functions: vec![
                FunctionMetadata {
                    name: "generate-config".to_string(),
                    description: "Generate Lefthook configuration".to_string(),
                    parameters: vec![ParameterMetadata {
                        name: "config".to_string(),
                        parameter_type: "LefthookConfig".to_string(),
                        required: true,
                        description: "Configuration for generation".to_string(),
                    }],
                    return_type: Some("LefthookResult".to_string()),
                    is_async: false,
                },
                FunctionMetadata {
                    name: "validate-config".to_string(),
                    description: "Validate Lefthook configuration".to_string(),
                    parameters: vec![ParameterMetadata {
                        name: "config-content".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                        description: "Configuration content to validate".to_string(),
                    }],
                    return_type: Some("ValidationResult".to_string()),
                    is_async: false,
                },
            ],
            capabilities: vec![
                "yaml-generation".to_string(),
                "schema-validation".to_string(),
            ],
            dependencies: vec![],
        };

        handle.update_metadata(metadata);
        handle
    }

    /// Create a validation component
    pub fn create_validation(&self) -> ComponentHandle {
        let mut handle = ComponentHandle::new("validation".to_string(), self.engine.clone());

        let metadata = ComponentMetadata {
            name: "validation".to_string(),
            version: "0.1.0".to_string(),
            functions: vec![FunctionMetadata {
                name: "validate".to_string(),
                description: "Validate configuration or data".to_string(),
                parameters: vec![ParameterMetadata {
                    name: "config".to_string(),
                    parameter_type: "ValidationConfig".to_string(),
                    required: true,
                    description: "Validation configuration".to_string(),
                }],
                return_type: Some("ValidationResult".to_string()),
                is_async: false,
            }],
            capabilities: vec![
                "schema-validation".to_string(),
                "json-validation".to_string(),
            ],
            dependencies: vec![],
        };

        handle.update_metadata(metadata);
        handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Engine;

    #[test]
    fn test_component_handle() {
        let engine = Engine::default();
        let handle = ComponentHandle::new("test-component".to_string(), engine);

        assert_eq!(handle.name(), "test-component");
        assert_eq!(handle.metadata().name, "test-component");
        assert_eq!(handle.available_functions().len(), 0);
    }

    #[test]
    fn test_component_registry() {
        let mut registry = ComponentRegistry::new();
        let engine = Engine::default();

        let component = ComponentHandle::new("test-component".to_string(), engine);
        registry.register(component);

        assert_eq!(registry.count(), 1);
        assert!(registry.has("test-component"));
        assert!(registry.get("test-component").is_some());
    }

    #[test]
    fn test_component_factory() {
        let engine = Engine::default();
        let factory = ComponentFactory::new(engine);

        let hook_builder = factory.create_hook_builder();
        assert_eq!(hook_builder.name(), "hook-builder");
        assert!(hook_builder.has_function("build-hook"));

        let worktree_manager = factory.create_worktree_manager();
        assert_eq!(worktree_manager.name(), "worktree-manager");
        assert!(worktree_manager.has_function("execute-operation"));
    }
}
