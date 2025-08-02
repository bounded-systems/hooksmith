//! Hook Builder WASM Component
//!
//! This component provides WASM interface for building Rust hooks into binary executables.
//! It handles Rust compilation, binary optimization, and hook metadata generation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

mod builder;
mod compiler;
mod optimizer;
mod validator;

use builder::{BuildConfig, BuildMetadata, BuildResult, HookBuilder};
use optimizer::{BinaryOptimizer, OptimizationConfig, OptimizationResult};
use validator::{SourceValidator, ValidationConfig, ValidationResult};

/// Hook Builder component
#[wasm_bindgen]
pub struct HookBuilderComponent {
    builder: HookBuilder,
    validator: SourceValidator,
    optimizer: BinaryOptimizer,
}

#[wasm_bindgen]
impl HookBuilderComponent {
    /// Create a new hook builder component
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            builder: HookBuilder::new(),
            validator: SourceValidator::new(),
            optimizer: BinaryOptimizer::new(),
        }
    }

    /// Build a hook from source
    #[cfg(target_arch = "wasm32")]
    pub async fn build_hook(&self, config: JsValue) -> Result<JsValue, JsValue> {
        let config: BuildConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        let result = self
            .builder
            .build_hook(config)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Validate hook source code
    #[cfg(target_arch = "wasm32")]
    pub async fn validate_source(&self, config: JsValue) -> Result<JsValue, JsValue> {
        let config: ValidationConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        let result = self
            .validator
            .validate_source(config)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Optimize a built binary
    #[cfg(target_arch = "wasm32")]
    pub async fn optimize_binary(&self, config: JsValue) -> Result<JsValue, JsValue> {
        let config: OptimizationConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        let result = self
            .optimizer
            .optimize_binary(config)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get build information
    pub fn get_build_info(&self) -> Result<JsValue, JsValue> {
        let info = self
            .builder
            .get_build_info()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&info)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Clean build artifacts
    pub fn clean_build(&self, build_path: &str) -> Result<(), JsValue> {
        self.builder
            .clean_build(build_path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get available targets
    pub fn get_available_targets(&self) -> Result<JsValue, JsValue> {
        let targets = self
            .builder
            .get_available_targets()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&targets)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Check if target is supported
    pub fn is_target_supported(&self, target: &str) -> Result<JsValue, JsValue> {
        let supported = self
            .builder
            .is_target_supported(target)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&supported)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

impl HookBuilderComponent {
    /// Internal method to build a hook
    async fn build_hook_internal(&self, config: BuildConfig) -> Result<BuildResult> {
        self.builder.build_hook(config).await
    }

    /// Internal method to validate source
    async fn validate_source_internal(&self, config: ValidationConfig) -> Result<ValidationResult> {
        self.validator.validate_source(config).await
    }

    /// Internal method to optimize binary
    async fn optimize_binary_internal(
        &self,
        config: OptimizationConfig,
    ) -> Result<OptimizationResult> {
        self.optimizer.optimize_binary(config).await
    }
}

// WASM bindings for JavaScript interop
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_builder_creation() {
        let component = HookBuilderComponent::new();
        // Test that we can create a component
        assert!(true);
    }

    #[test]
    fn test_build_info() {
        let component = HookBuilderComponent::new();
        let info = component.get_build_info();
        // Test that we can get build info
        assert!(info.is_ok());
    }
}
