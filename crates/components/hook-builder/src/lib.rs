//! Hook Builder WASM Component
//!
//! This component provides WASM interface for building Rust hooks into binary executables.
//! It handles Rust compilation, binary optimization, and hook metadata generation.

use anyhow::Result;

mod builder;
mod compiler;
mod optimizer;
mod validator;

use builder::{BuildConfig, BuildMetadata, BuildResult, HookBuilder};
use optimizer::{BinaryOptimizer, OptimizationConfig, OptimizationResult};
use validator::{SourceValidator, ValidationConfig, ValidationResult};

/// Hook Builder component
pub struct HookBuilderComponent {
    builder: HookBuilder,
    validator: SourceValidator,
    optimizer: BinaryOptimizer,
}

impl HookBuilderComponent {
    /// Create a new hook builder component
    pub fn new() -> Self {
        Self {
            builder: HookBuilder::new(),
            validator: SourceValidator::new(),
            optimizer: BinaryOptimizer::new(),
        }
    }

    /// Build a hook from source
    pub async fn build_hook(&self, config: BuildConfig) -> Result<BuildResult> {
        self.builder.build_hook(config).await
    }

    /// Validate hook source code
    pub async fn validate_source(&self, config: ValidationConfig) -> Result<ValidationResult> {
        self.validator.validate_source(config).await
    }

    /// Optimize a built binary
    pub async fn optimize_binary(&self, config: OptimizationConfig) -> Result<OptimizationResult> {
        self.optimizer.optimize_binary(config).await
    }

    /// Get build information
    pub fn get_build_info(&self) -> Result<BuildMetadata> {
        self.builder.get_build_info()
    }

    /// Clean build artifacts
    pub fn clean_build(&self, build_path: &str) -> Result<()> {
        self.builder.clean_build(build_path)
    }

    /// Get available compilation targets
    pub fn get_available_targets(&self) -> Result<Vec<String>> {
        self.builder.get_available_targets()
    }

    /// Check if a target is supported
    pub fn is_target_supported(&self, target: &str) -> Result<bool> {
        self.builder.is_target_supported(target)
    }
}

impl Default for HookBuilderComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize panic hook for better error reporting
pub fn init_panic_hook() {
    // In a real WASM environment, this would set up panic handling
    // For now, we'll leave it empty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_builder_creation() {
        let component = HookBuilderComponent::new();
        assert!(component.get_build_info().is_ok());
    }

    #[test]
    fn test_build_info() {
        let component = HookBuilderComponent::new();
        let info = component.get_build_info().unwrap();
        assert!(!info.rust_version.is_empty());
        assert!(!info.cargo_version.is_empty());
    }
}
