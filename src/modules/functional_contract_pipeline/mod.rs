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

/// Concern snapshot implementations
pub mod concerns;
/// Contract templates for reusable contract definitions
pub mod contract_templates;
/// Contract definitions and mapping
pub mod contracts;
/// Hashed store for content-addressable caching
pub mod hashed_store;
/// High-performance diffing strategies
pub mod high_performance_diff;
/// Hook event handlers and concern mapping
pub mod hooks;
pub mod repair_core;
/// Repair planning system with Triage Officer, Investigator, Dispatcher, and Fixers
pub mod repair_planning;
/// SARIF merge utilities for parallel processing
pub mod sarif_merge;
/// SARIF-first validation roles and architecture
pub mod sarif_roles;
/// Contract to expectation specification
pub mod specifier;
/// Symbols and enums for the functional contract validation pipeline
pub mod symbols;
/// Types and data structures for the functional contract validation pipeline
pub mod types;
/// Validation and verification logic
pub mod verifier;

use crate::modules::functional_contract_pipeline::repair_core::{
    Fixer, LintIgnoreOrderFixer, MermaidExporter, PlanValidator,
    RepairPlan as CoreRepairPlan, RepairResult, ReplaceRootStarFixer, RootCause as CoreRootCause,
    Violation as CoreViolation,
};
use crate::modules::functional_contract_pipeline::repair_planning::{
    RepairPlan, TriageOfficer, Violation,
};
use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, HookEvent};
use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Main functional contract validation pipeline
pub struct FunctionalContractPipeline {
    /// Repository path
    #[allow(dead_code)]
    repo_path: String,
    /// Hook event to concern mapping
    #[allow(dead_code)]
    hook_concerns: HashMap<HookEvent, Vec<ConcernSymbol>>,
    /// Triage Officer for repair planning
    triage_officer: TriageOfficer,
}

impl FunctionalContractPipeline {
    /// Create a new pipeline for the given repository
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_string_lossy().to_string(),
            hook_concerns: HashMap::new(),
            triage_officer: TriageOfficer::new(),
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
    pub fn run_hook_with_diff(&self, hook: HookEvent) -> Result<types::DiffSet> {
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

        // 5. Verify snapshots against expectations and return diff
        Ok(verifier::verify_with_diffs(&snapshots, &expectations))
    }

    /// Run the complete pipeline with repair planning
    pub fn run_hook_with_repair(&mut self, hook: HookEvent) -> Result<Vec<RepairPlan>> {
        // 1. Run the standard validation pipeline
        let diff_set = self.run_hook_with_diff(hook)?;

        // 2. Convert violations to repair plans
        let mut repair_plans = Vec::new();

        for diff in &diff_set.diffs {
            // Convert diff to violation
            let violation = Violation {
                concern: diff.concern.clone(),
                contract: "unknown".to_string(), // Would be determined from contract mapping
                message: diff.description.clone(),
                location: None, // Would be extracted from diff
                severity: diff.severity.clone(),
                details: diff.metadata.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // Find corresponding snapshot
            let snapshot = self.find_snapshot_for_concern(&diff.concern)?;

            // Create repair plan
            let plan = self.triage_officer.create_plan(&violation, &snapshot)?;
            repair_plans.push(plan);
        }

        Ok(repair_plans)
    }

    /// Creates a repair plan using the core repair system
    pub fn create_core_repair_plan(
        &self,
        concern: &ConcernSymbol,
        contract: &str,
        violation_msg: &str,
    ) -> RepairResult<CoreRepairPlan> {
        let violation = CoreViolation {
            concern: concern.clone(),
            contract: contract.to_string(),
            message: violation_msg.to_string(),
            location: format!("{:?}", concern),
            severity: repair_core::ViolationSeverity::Error,
        };

        let root_cause = CoreRootCause {
            primary_cause: "Contract violation detected".to_string(),
            factors: vec!["Automated analysis".to_string()],
            fix_categories: vec![repair_core::FixCategory::Formatting],
            confidence: 0.9,
        };

        let mut actions = Vec::new();

        // Add example fixers
        let fixers: Vec<Box<dyn Fixer>> = vec![
            Box::new(ReplaceRootStarFixer),
            Box::new(LintIgnoreOrderFixer),
        ];

        for fixer in fixers {
            if let Ok(Some(action)) = fixer.plan(&violation, &root_cause) {
                actions.push(action);
            }
        }

        let plan = CoreRepairPlan {
            id: format!("plan-{:?}-{}", concern, contract)
                .replace(":", "-")
                .replace("\"", ""),
            concern: concern.clone(),
            contract: contract.to_string(),
            violation,
            root_cause,
            dispatcher: "core-dispatcher".to_string(),
            actions,
            is_complete: true,
            metadata: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Validate the plan
        PlanValidator::validate(&plan)?;

        Ok(plan)
    }

    /// Exports a repair plan as a Mermaid diagram
    pub fn export_repair_plan_mermaid(&self, plan: &CoreRepairPlan) -> String {
        MermaidExporter::export_plan(plan)
    }

    /// Find snapshot for a specific concern
    fn find_snapshot_for_concern(&self, concern: &ConcernSymbol) -> Result<ConcernSnapshot> {
        // This would typically look up the snapshot from the concern
        // For now, create a placeholder snapshot
        Ok(ConcernSnapshot::new(
            concern.clone(),
            serde_json::json!({}),
            HashMap::new(),
        ))
    }
}

/// Extended pipeline with repair planning capabilities
pub struct RepairPlanningPipeline {
    /// Base functional contract pipeline
    base_pipeline: FunctionalContractPipeline,
    /// Triage Officer for repair planning
    triage_officer: TriageOfficer,
}

impl RepairPlanningPipeline {
    /// Create a new repair planning pipeline
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            base_pipeline: FunctionalContractPipeline::new(repo_path),
            triage_officer: TriageOfficer::new(),
        }
    }

    /// Run the complete pipeline with repair planning
    pub fn run_with_repair(&mut self, hook: HookEvent) -> Result<Vec<RepairPlan>> {
        self.base_pipeline.run_hook_with_repair(hook)
    }

    /// Get the triage officer for direct access
    pub fn triage_officer(&mut self) -> &mut TriageOfficer {
        &mut self.triage_officer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::symbols::HookEvent;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = FunctionalContractPipeline::new(".");
        assert_eq!(pipeline.repo_path, ".");
    }

    #[test]
    fn test_repair_planning_pipeline_creation() {
        let mut pipeline = RepairPlanningPipeline::new(".");
        // Test that the triage officer can be accessed
        let _triage_officer = pipeline.triage_officer();
    }
}
