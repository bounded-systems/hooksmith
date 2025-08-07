//! Functional Contract Validation Pipeline
//! 
//! This module implements a deterministic, stateless, and parallelizable system
//! for validating Git operations against declarative contracts.
//! 
//! ## Architecture
//! 
//! ```rust
//! Hook Event → Identify Concerns → Archive Snapshots → Map Contracts → Specify Expectations → Verify → Result
//! ```
//! 
//! ## Core Principles
//! 
//! 1. **Stateless**: Each step operates independently without shared state
//! 2. **Deterministic**: Same inputs always produce same outputs  
//! 3. **Parallelizable**: Steps can be executed concurrently
//! 4. **Declarative**: Contracts defined as data, not imperative code
//! 
//! ## Usage
//! 
//! ```rust
//! use hooksmith::modules::functional_contract_pipeline::{
//!     FunctionalContractPipeline, HookEvent
//! };
//! 
//! let mut pipeline = FunctionalContractPipeline::new(".");
//! let result = pipeline.run_hook(HookEvent::PreCommit);
//! 
//! match result {
//!     Ok(()) => println!("✅ Validation passed"),
//!     Err(errors) => {
//!         eprintln!("❌ Validation failed:");
//!         for error in errors {
//!             eprintln!("  - {}", error);
//!         }
//!     }
//! }
//! ```

/// Symbols and enums for the functional contract validation pipeline
pub mod symbols;
/// Types and data structures for the functional contract validation pipeline
pub mod types;
/// Hook event handlers and concern mapping
pub mod hooks;
/// Concern snapshot implementations
pub mod concerns;
/// Contract definitions and mapping
pub mod contracts;
/// Contract to expectation specification
pub mod specifier;
/// Validation and verification logic
pub mod verifier;
/// High-performance diffing strategies
pub mod high_performance_diff;
/// SARIF-first validation roles and architecture
pub mod sarif_roles;

use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, HookEvent};
use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Main functional contract validation pipeline
pub struct FunctionalContractPipeline {
    /// Repository path
    repo_path: String,
    /// Hook event to concern mapping
    hook_concerns: HashMap<HookEvent, Vec<ConcernSymbol>>,
}

impl FunctionalContractPipeline {
    /// Create a new pipeline for the given repository
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_string_lossy().to_string(),
            hook_concerns: HashMap::new(),
        }
    }

    /// Run the pipeline for a specific hook event
    pub fn run_hook(&self, hook: HookEvent) -> Result<()> {
        // 1. Identify concerns for the hook
        let concerns = hooks::get_concerns(&hook);
        
        // 2. Archive snapshots of concerns
        let snapshots: Vec<ConcernSnapshot> = concerns
            .iter()
            .map(|concern| concerns::snapshot_concern(concern))
            .collect();
        
        // 3. Get contracts for concerns
        let contracts = contracts::get_all_contracts(&concerns);
        
        // 4. Build expectations from contracts
        let expectations: Vec<ExpectedSnapshot> = contracts
            .iter()
            .map(|contract| specifier::build_expectation(contract))
            .collect();
        
        // 5. Verify snapshots against expectations
        verifier::verify(&snapshots, &expectations)
            .map_err(|errors| anyhow::anyhow!("Validation failed: {:?}", errors))
    }

    /// Run the pipeline with detailed diff information
    pub fn run_hook_with_diffs(&self, hook: HookEvent) -> crate::modules::functional_contract_pipeline::types::DiffSet {
        // 1. Identify concerns for the hook
        let concerns = hooks::get_concerns(&hook);
        
        // 2. Archive snapshots of concerns
        let snapshots: Vec<ConcernSnapshot> = concerns
            .iter()
            .map(|concern| concerns::snapshot_concern(concern))
            .collect();
        
        // 3. Get contracts for concerns
        let contracts = contracts::get_all_contracts(&concerns);
        
        // 4. Build expectations from contracts
        let expectations: Vec<ExpectedSnapshot> = contracts
            .iter()
            .map(|contract| specifier::build_expectation(contract))
            .collect();
        
        // 5. Verify with detailed diffs
        verifier::verify_with_diffs(&snapshots, &expectations)
    }

    /// Run the pipeline with custom severity mapping
    pub fn run_hook_with_severity(
        &self,
        hook: HookEvent,
        severity_map: &HashMap<ConcernSymbol, crate::modules::functional_contract_pipeline::symbols::RuleSeverity>,
    ) -> crate::modules::functional_contract_pipeline::types::DiffSet {
        // 1. Identify concerns for the hook
        let concerns = hooks::get_concerns(&hook);
        
        // 2. Archive snapshots of concerns
        let snapshots: Vec<ConcernSnapshot> = concerns
            .iter()
            .map(|concern| concerns::snapshot_concern(concern))
            .collect();
        
        // 3. Get contracts for concerns
        let contracts = contracts::get_all_contracts(&concerns);
        
        // 4. Build expectations from contracts
        let expectations: Vec<ExpectedSnapshot> = contracts
            .iter()
            .map(|contract| specifier::build_expectation(contract))
            .collect();
        
        // 5. Verify with custom severity
        verifier::verify_with_severity(&snapshots, &expectations, severity_map)
    }

    /// Run the pipeline with JSON Patch diff generation
    pub fn run_hook_with_json_patch(
        &self,
        hook: HookEvent,
    ) -> crate::modules::functional_contract_pipeline::types::DiffSet {
        // 1. Identify concerns for the hook
        let concerns = hooks::get_concerns(&hook);
        
        // 2. Archive snapshots of concerns
        let snapshots: Vec<ConcernSnapshot> = concerns
            .iter()
            .map(|concern| concerns::snapshot_concern(concern))
            .collect();
        
        // 3. Get contracts for concerns
        let contracts = contracts::get_all_contracts(&concerns);
        
        // 4. Build expectations from contracts
        let expectations: Vec<ExpectedSnapshot> = contracts
            .iter()
            .map(|contract| specifier::build_expectation(contract))
            .collect();
        
        // 5. Verify with JSON Patch
        verifier::verify_with_json_patch(&snapshots, &expectations)
    }

    /// Generate JSON Patch for the entire pipeline
pub fn generate_pipeline_patch(
    &self,
    hook: HookEvent,
) -> json_patch::Patch {
    // 1. Identify concerns for the hook
    let concerns = hooks::get_concerns(&hook);
    
    // 2. Archive snapshots of concerns
    let snapshots: Vec<ConcernSnapshot> = concerns
        .iter()
        .map(|concern| concerns::snapshot_concern(concern))
        .collect();
    
    // 3. Get contracts for concerns
    let contracts = contracts::get_all_contracts(&concerns);
    
    // 4. Build expectations from contracts
    let expectations: Vec<ExpectedSnapshot> = contracts
        .iter()
        .map(|contract| specifier::build_expectation(contract))
        .collect();
    
    // 5. Generate JSON Patch
    verifier::generate_json_patch(&snapshots, &expectations)
}

/// Run the pipeline with high-performance diffing and automatic strategy selection
pub fn run_hook_with_high_performance_diff(
    &self,
    hook: HookEvent,
) -> (crate::modules::functional_contract_pipeline::types::DiffSet, crate::modules::functional_contract_pipeline::high_performance_diff::DiffMetrics) {
    // 1. Identify concerns for the hook
    let concerns = hooks::get_concerns(&hook);
    
    // 2. Archive snapshots of concerns
    let snapshots: Vec<ConcernSnapshot> = concerns
        .iter()
        .map(|concern| concerns::snapshot_concern(concern))
        .collect();
    
    // 3. Get contracts for concerns
    let contracts = contracts::get_all_contracts(&concerns);
    
    // 4. Build expectations from contracts
    let expectations: Vec<ExpectedSnapshot> = contracts
        .iter()
        .map(|contract| specifier::build_expectation(contract))
        .collect();
    
    // 5. Use high-performance diffing with automatic strategy selection
    high_performance_diff::convenience::auto_diff(&snapshots, &expectations)
}

/// Run the pipeline with specified high-performance diffing strategy
pub fn run_hook_with_diff_strategy(
    &self,
    hook: HookEvent,
    strategy: crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy,
) -> (crate::modules::functional_contract_pipeline::types::DiffSet, crate::modules::functional_contract_pipeline::high_performance_diff::DiffMetrics) {
    // 1. Identify concerns for the hook
    let concerns = hooks::get_concerns(&hook);
    
    // 2. Archive snapshots of concerns
    let snapshots: Vec<ConcernSnapshot> = concerns
        .iter()
        .map(|concern| concerns::snapshot_concern(concern))
        .collect();
    
    // 3. Get contracts for concerns
    let contracts = contracts::get_all_contracts(&concerns);
    
    // 4. Build expectations from contracts
    let expectations: Vec<ExpectedSnapshot> = contracts
        .iter()
        .map(|contract| specifier::build_expectation(contract))
        .collect();
    
    // 5. Use specified high-performance diffing strategy
    high_performance_diff::convenience::diff_with_strategy(strategy, &snapshots, &expectations)
}

/// Benchmark all diffing strategies for a hook
pub fn benchmark_hook_strategies(
    &self,
    hook: HookEvent,
) -> String {
    // 1. Identify concerns for the hook
    let concerns = hooks::get_concerns(&hook);
    
    // 2. Archive snapshots of concerns
    let snapshots: Vec<ConcernSnapshot> = concerns
        .iter()
        .map(|concern| concerns::snapshot_concern(concern))
        .collect();
    
    // 3. Get contracts for concerns
    let contracts = contracts::get_all_contracts(&concerns);
    
    // 4. Build expectations from contracts
    let expectations: Vec<ExpectedSnapshot> = contracts
        .iter()
        .map(|contract| specifier::build_expectation(contract))
        .collect();
    
    // 5. Generate benchmark report
    high_performance_diff::convenience::benchmark_report(&snapshots, &expectations)
}

    /// Get repository path
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    /// Set repository path
    pub fn set_repo_path<P: AsRef<Path>>(&mut self, repo_path: P) {
        self.repo_path = repo_path.as_ref().to_string_lossy().to_string();
    }
}

/// Convenience function to run a hook validation
pub fn run_hook(hook: HookEvent, repo_path: &str) -> Result<()> {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.run_hook(hook)
}

/// Convenience function to run a hook validation with diffs
pub fn run_hook_with_diffs(hook: HookEvent, repo_path: &str) -> crate::modules::functional_contract_pipeline::types::DiffSet {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.run_hook_with_diffs(hook)
}

/// Convenience function to run a hook validation with JSON Patch
pub fn run_hook_with_json_patch(hook: HookEvent, repo_path: &str) -> crate::modules::functional_contract_pipeline::types::DiffSet {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.run_hook_with_json_patch(hook)
}

/// Convenience function to generate JSON Patch for a hook
pub fn generate_hook_patch(hook: HookEvent, repo_path: &str) -> json_patch::Patch {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.generate_pipeline_patch(hook)
}

/// Convenience function to run hook with high-performance diffing
pub fn run_hook_with_high_performance_diff(hook: HookEvent, repo_path: &str) -> (crate::modules::functional_contract_pipeline::types::DiffSet, crate::modules::functional_contract_pipeline::high_performance_diff::DiffMetrics) {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.run_hook_with_high_performance_diff(hook)
}

/// Convenience function to run hook with specified diff strategy
pub fn run_hook_with_diff_strategy(
    hook: HookEvent, 
    repo_path: &str,
    strategy: crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy,
) -> (crate::modules::functional_contract_pipeline::types::DiffSet, crate::modules::functional_contract_pipeline::high_performance_diff::DiffMetrics) {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.run_hook_with_diff_strategy(hook, strategy)
}

/// Convenience function to benchmark hook strategies
pub fn benchmark_hook_strategies(hook: HookEvent, repo_path: &str) -> String {
    let pipeline = FunctionalContractPipeline::new(repo_path);
    pipeline.benchmark_hook_strategies(hook)
}

/// Convenience function to run SARIF-first pipeline
pub fn run_sarif_first_pipeline(
    hook_event: HookEvent,
    commit_hash: String,
    tree_hash: String,
) -> (serde_sarif::sarif::SarifLog, crate::modules::functional_contract_pipeline::sarif_roles::AuditResult) {
    let mut pipeline = crate::modules::functional_contract_pipeline::sarif_roles::SarifFirstPipeline::new();
    let git_metadata = crate::modules::functional_contract_pipeline::sarif_roles::GitMetadata::new(
        commit_hash,
        tree_hash,
        hook_event,
    );
    pipeline.run_pipeline(hook_event, git_metadata)
}

/// Convenience function to create audit policy
pub fn create_audit_policy(
    name: String,
    description: String,
    concern: Option<String>,
    severity: Option<crate::modules::functional_contract_pipeline::symbols::RuleSeverity>,
    hook_event: Option<String>,
    action: crate::modules::functional_contract_pipeline::sarif_roles::AuditAction,
) -> crate::modules::functional_contract_pipeline::sarif_roles::AuditPolicy {
    let mut criteria = crate::modules::functional_contract_pipeline::sarif_roles::QueryCriteria::new();
    
    if let Some(concern) = concern {
        criteria = criteria.with_concern(concern);
    }
    
    if let Some(severity) = severity {
        criteria = criteria.with_severity(severity);
    }
    
    if let Some(hook_event) = hook_event {
        criteria = criteria.with_hook_event(hook_event);
    }
    
    crate::modules::functional_contract_pipeline::sarif_roles::AuditPolicy::new(
        name,
        description,
        criteria,
        action,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = FunctionalContractPipeline::new(".");
        assert_eq!(pipeline.repo_path(), ".");
    }

    #[test]
    fn test_pipeline_repo_path_setter() {
        let mut pipeline = FunctionalContractPipeline::new(".");
        pipeline.set_repo_path("/tmp/repo");
        assert_eq!(pipeline.repo_path(), "/tmp/repo");
    }

    #[test]
    fn test_run_hook_convenience() {
        // This test would require a real Git repository
        // For now, we just test that the function compiles
        let _result = run_hook(HookEvent::PreCommit, ".");
    }

    #[test]
    fn test_run_hook_with_diffs_convenience() {
        // This test would require a real Git repository
        // For now, we just test that the function compiles
        let _result = run_hook_with_diffs(HookEvent::PreCommit, ".");
    }
}
