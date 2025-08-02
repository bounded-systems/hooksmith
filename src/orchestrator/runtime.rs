//! WASM Runtime for Hooksmith
//!
//! This module provides the WASM runtime environment for executing
//! WASM components. It handles component loading, instantiation,
//! and function calls.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

use super::components::ComponentHandle;

/// Configuration for the WASM runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Whether to enable WASI support
    pub enable_wasi: bool,
    /// Maximum memory size in MB
    pub max_memory_mb: u32,
    /// Whether to enable debug logging
    pub debug_logging: bool,
    /// Component search paths
    pub component_paths: Vec<PathBuf>,
    /// Preloaded components
    pub preloaded_components: HashMap<String, PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enable_wasi: true,
            max_memory_mb: 512,
            debug_logging: false,
            component_paths: vec![PathBuf::from("components")],
            preloaded_components: HashMap::new(),
        }
    }
}

/// WASM runtime for managing components
pub struct WasmRuntime {
    /// WASM engine
    engine: Engine,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Loaded modules cache
    modules: Arc<RwLock<HashMap<String, Module>>>,
    /// Active instances
    instances: Arc<RwLock<HashMap<String, Instance>>>,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub async fn new(config: &RuntimeConfig) -> Result<Self> {
        let engine = Engine::default();

        Ok(Self {
            engine,
            config: config.clone(),
            modules: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load a component from a WASM file
    pub async fn load_component(&self, name: &str, wasm_path: PathBuf) -> Result<ComponentHandle> {
        // Check if module is already loaded
        {
            let modules = self.modules.read().await;
            if modules.contains_key(name) {
                return self.create_component_handle(name).await;
            }
        }

        // Load the WASM module
        let wasm_bytes = tokio::fs::read(&wasm_path).await?;
        let module = Module::new(&self.engine, wasm_bytes)?;

        // Cache the module
        {
            let mut modules = self.modules.write().await;
            modules.insert(name.to_string(), module);
        }

        // Create component handle
        self.create_component_handle(name).await
    }

    /// Create a component handle for an existing module
    async fn create_component_handle(&self, name: &str) -> Result<ComponentHandle> {
        let modules = self.modules.read().await;
        let module = modules
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", name))?;

        if self.config.enable_wasi {
            // Create WASI-enabled linker
            let mut linker = Linker::new(&self.engine);
            let wasi_ctx = WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_args()?
                .build();

            wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

            // Create store with WASI context
            let mut store = Store::new(&self.engine, wasi_ctx);

            // Instantiate the module
            let instance = linker.instantiate(&mut store, module)?;

            // Cache the instance
            {
                let mut instances = self.instances.write().await;
                instances.insert(name.to_string(), instance);
            }

            Ok(ComponentHandle::new(name.to_string(), self.engine.clone()))
        } else {
            // Create basic linker without WASI
            let linker = Linker::new(&self.engine);

            // Create store without WASI context
            let mut store = Store::new(&self.engine, ());

            // Instantiate the module
            let instance = linker.instantiate(&mut store, module)?;

            // Cache the instance
            {
                let mut instances = self.instances.write().await;
                instances.insert(name.to_string(), instance);
            }

            Ok(ComponentHandle::new(name.to_string(), self.engine.clone()))
        }
    }

    /// Get a component handle by name
    pub async fn get_component(&self, name: &str) -> Result<ComponentHandle> {
        let instances = self.instances.read().await;
        if instances.contains_key(name) {
            Ok(ComponentHandle::new(name.to_string(), self.engine.clone()))
        } else {
            Err(anyhow::anyhow!("Component '{}' not loaded", name))
        }
    }

    /// Unload a component
    pub async fn unload_component(&self, name: &str) -> Result<()> {
        // Remove from instances
        {
            let mut instances = self.instances.write().await;
            instances.remove(name);
        }

        // Remove from modules
        {
            let mut modules = self.modules.write().await;
            modules.remove(name);
        }

        Ok(())
    }

    /// List loaded components
    pub async fn list_components(&self) -> Vec<String> {
        let instances = self.instances.read().await;
        instances.keys().cloned().collect()
    }

    /// Check if a component is loaded
    pub async fn has_component(&self, name: &str) -> bool {
        let instances = self.instances.read().await;
        instances.contains_key(name)
    }

    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Update runtime configuration
    pub fn update_config(&mut self, config: RuntimeConfig) {
        self.config = config;
    }

    /// Get engine reference
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

/// WASM function call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCallResult {
    /// Whether the call was successful
    pub success: bool,
    /// Return value (serialized)
    pub return_value: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// WASM function call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCallRequest {
    /// Function name to call
    pub function_name: String,
    /// Arguments (serialized)
    pub arguments: Vec<String>,
    /// Expected return type
    pub return_type: Option<String>,
}

impl WasmRuntime {
    /// Call a function on a component
    pub async fn call_function(
        &self,
        component_name: &str,
        request: WasmCallRequest,
    ) -> Result<WasmCallResult> {
        let start_time = std::time::Instant::now();

        // Get the component instance
        let instances = self.instances.read().await;
        let instance = instances
            .get(component_name)
            .ok_or_else(|| anyhow::anyhow!("Component '{}' not found", component_name))?;

        // Get the function
        let mut store = Store::new(&self.engine, ());
        let function = instance
            .get_func(&mut store, &request.function_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", request.function_name))?;

        // Call the function
        let result = match function.call(&mut store, &[], &mut []) {
            Ok(_) => WasmCallResult {
                success: true,
                return_value: None, // TODO: Handle return values properly
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
            },
            Err(e) => WasmCallResult {
                success: false,
                return_value: None,
                error: Some(e.to_string()),
                duration_ms: start_time.elapsed().as_millis() as u64,
            },
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = WasmRuntime::new(&config).await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_runtime_config() {
        let mut config = RuntimeConfig::default();
        config.max_memory_mb = 1024;
        config.debug_logging = true;

        let runtime = WasmRuntime::new(&config).await;
        assert!(runtime.is_ok());

        let runtime = runtime.unwrap();
        assert_eq!(runtime.config().max_memory_mb, 1024);
        assert!(runtime.config().debug_logging);
    }
}
