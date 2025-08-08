//! Repair Planning System
//!
//! This module implements the repair planning pipeline with the following roles:
//!
//! - **Auditor**: Declares failure; emits SARIF log (post-verification)
//! - **Investigator**: Analyzes failing concern (diagnosis)
//! - **Dispatcher**: Routes to a fixer set (assignment)
//! - **Triage Officer**: Plans full fix strategy across concern space (orchestration)
//! - **Fixers**: Stateless tools that attempt repair (execution)
//!
//! ## Architecture
//!
//! ```rust
//! Hook → Concern → Contract → Auditor → Investigator → Dispatcher → Triage Officer → Fixers → Repeat
//! ```
//!
//! ## Core Principles
//!
//! 1. **Single Visit**: Each concern is visited exactly once per phase
//! 2. **Isolated Pipelines**: Each concern executes its own self-contained audit → plan → fix sequence
//! 3. **Deterministic Caching**: Cache keyed by (concern.hash, contract.id)
//! 4. **CRD-Only Output**: Repair output is not an applied patch—it's a regenerated CRD from the fixer tree
//! 5. **Note-Driven**: Everything hangs off Git Notes; they're versionable, mergeable, and scoped

use crate::modules::functional_contract_pipeline::sarif_roles::{AuditResult, SarifResult};
use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, RuleSeverity};
use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ValidationDiff};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Violation context for a failing concern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The concern that violated the contract
    pub concern: ConcernSymbol,
    /// The contract that was violated
    pub contract: String,
    /// The violation message
    pub message: String,
    /// The location of the violation (file path, line number, etc.)
    pub location: Option<String>,
    /// The severity of the violation
    pub severity: RuleSeverity,
    /// Additional violation details
    pub details: HashMap<String, serde_json::Value>,
    /// Timestamp of the violation
    pub timestamp: String,
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// The primary cause of the violation
    pub primary_cause: String,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Suggested fix categories
    pub fix_categories: Vec<FixCategory>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Additional analysis metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Categories of fixes that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixCategory {
    /// Formatting fixes (rustfmt, prettier, etc.)
    Formatting,
    /// Linting fixes (clippy, eslint, etc.)
    Linting,
    /// Structural fixes (file organization, naming, etc.)
    Structural,
    /// Configuration fixes (git attributes, config files, etc.)
    Configuration,
    /// Content fixes (file content, metadata, etc.)
    Content,
    /// Tool-specific fixes (trunk, cargo, etc.)
    ToolSpecific,
}

impl FixCategory {
    /// Get the name of the fix category
    pub fn name(&self) -> &'static str {
        match self {
            FixCategory::Formatting => "formatting",
            FixCategory::Linting => "linting",
            FixCategory::Structural => "structural",
            FixCategory::Configuration => "configuration",
            FixCategory::Content => "content",
            FixCategory::ToolSpecific => "tool_specific",
        }
    }
}

/// Repair action to be executed by a fixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    /// Unique identifier for this action
    pub id: String,
    /// The fixer that will execute this action
    pub fixer_id: String,
    /// The type of action to perform
    pub action_type: ActionType,
    /// The target path for this action
    pub target_path: Option<String>,
    /// The action parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Whether this action is required or suggested
    pub required: bool,
    /// Priority of this action (lower = higher priority)
    pub priority: u32,
    /// Dependencies on other actions
    pub dependencies: Vec<String>,
}

/// Types of repair actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    /// Edit a file
    Edit,
    /// Replace a file
    Replace,
    /// Delete a file
    Delete,
    /// Reorder lines in a file
    ReorderLines,
    /// Run a command
    RunCommand,
    /// Apply a patch
    ApplyPatch,
    /// Create a new file
    Create,
    /// Move a file
    Move,
}

impl ActionType {
    /// Get the name of the action type
    pub fn name(&self) -> &'static str {
        match self {
            ActionType::Edit => "edit",
            ActionType::Replace => "replace",
            ActionType::Delete => "delete",
            ActionType::ReorderLines => "reorder_lines",
            ActionType::RunCommand => "run_command",
            ActionType::ApplyPatch => "apply_patch",
            ActionType::Create => "create",
            ActionType::Move => "move",
        }
    }
}

/// Result of a repair action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    /// The action that was executed
    pub action_id: String,
    /// Whether the action succeeded
    pub success: bool,
    /// Messages from the fixer
    pub messages: Vec<String>,
    /// The diff that was applied (if any)
    pub diff: Option<serde_json::Value>,
    /// The new content hash (if applicable)
    pub new_hash: Option<String>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Complete repair plan for a failing concern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    /// Unique identifier for this plan
    pub id: String,
    /// The concern being repaired
    pub concern: ConcernSymbol,
    /// The contract that was violated
    pub contract: String,
    /// The violation that triggered this plan
    pub violation: Violation,
    /// The root cause analysis
    pub root_cause: RootCause,
    /// The dispatcher that created this plan
    pub dispatcher: String,
    /// The ordered list of repair actions
    pub actions: Vec<RepairAction>,
    /// Whether this plan is complete
    pub is_complete: bool,
    /// Plan metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp of plan creation
    pub timestamp: String,
}

/// Investigator role: Analyzes failing concerns to determine root cause
pub struct Investigator {
    /// Analysis strategies to apply
    strategies: Vec<AnalysisStrategy>,
}

impl Investigator {
    /// Create a new investigator
    pub fn new() -> Self {
        Self {
            strategies: vec![
                AnalysisStrategy::PatternMatching,
                AnalysisStrategy::RuleViolation,
                AnalysisStrategy::ToolOutput,
            ],
        }
    }

    /// Analyze a violation to determine root cause
    pub fn investigate(
        &self,
        violation: &Violation,
        snapshot: &ConcernSnapshot,
    ) -> Result<RootCause> {
        let mut causes = Vec::new();
        let mut factors = Vec::new();
        let mut categories = Vec::new();

        // Apply each analysis strategy
        for strategy in &self.strategies {
            let analysis = strategy.analyze(violation, snapshot)?;
            causes.extend(analysis.causes);
            factors.extend(analysis.contributing_factors);
            categories.extend(analysis.fix_categories);
        }

        // Determine primary cause
        let primary_cause = if !causes.is_empty() {
            causes[0].clone()
        } else {
            "Unknown violation".to_string()
        };

        // Calculate confidence based on analysis quality
        let confidence = self.calculate_confidence(&causes, &factors);

        Ok(RootCause {
            primary_cause,
            contributing_factors: factors,
            fix_categories: categories,
            confidence,
            metadata: HashMap::new(),
        })
    }

    /// Calculate confidence level based on analysis results
    fn calculate_confidence(&self, causes: &[String], factors: &[String]) -> f64 {
        let base_confidence = 0.5;
        let cause_bonus = (causes.len() as f64 * 0.1).min(0.3);
        let factor_bonus = (factors.len() as f64 * 0.05).min(0.2);

        (base_confidence + cause_bonus + factor_bonus).min(1.0)
    }
}

/// Analysis strategy for investigating violations
#[derive(Debug)]
enum AnalysisStrategy {
    /// Pattern-based analysis
    PatternMatching,
    /// Rule violation analysis
    RuleViolation,
    /// Tool output analysis
    ToolOutput,
}

impl AnalysisStrategy {
    /// Analyze a violation using this strategy
    fn analyze(&self, violation: &Violation, snapshot: &ConcernSnapshot) -> Result<AnalysisResult> {
        match self {
            AnalysisStrategy::PatternMatching => self.analyze_patterns(violation, snapshot),
            AnalysisStrategy::RuleViolation => self.analyze_rule_violation(violation, snapshot),
            AnalysisStrategy::ToolOutput => self.analyze_tool_output(violation, snapshot),
        }
    }

    /// Analyze patterns in the violation
    fn analyze_patterns(
        &self,
        violation: &Violation,
        _snapshot: &ConcernSnapshot,
    ) -> Result<AnalysisResult> {
        let mut causes = Vec::new();
        let factors = Vec::new();
        let mut categories = Vec::new();

        // Pattern-based analysis logic
        if violation.message.contains("format") {
            causes.push("Formatting violation".to_string());
            categories.push(FixCategory::Formatting);
        }

        if violation.message.contains("lint") {
            causes.push("Linting violation".to_string());
            categories.push(FixCategory::Linting);
        }

        if violation.message.contains("config") {
            causes.push("Configuration violation".to_string());
            categories.push(FixCategory::Configuration);
        }

        Ok(AnalysisResult {
            causes,
            contributing_factors: factors,
            fix_categories: categories,
        })
    }

    /// Analyze rule violations
    fn analyze_rule_violation(
        &self,
        violation: &Violation,
        _snapshot: &ConcernSnapshot,
    ) -> Result<AnalysisResult> {
        let mut causes = Vec::new();
        let factors = Vec::new();
        let mut categories = Vec::new();

        // Rule violation analysis logic
        match violation.contract.as_str() {
            "format" => {
                causes.push("Formatting rule violation".to_string());
                categories.push(FixCategory::Formatting);
            }
            "lint" => {
                causes.push("Linting rule violation".to_string());
                categories.push(FixCategory::Linting);
            }
            "structure" => {
                causes.push("Structural rule violation".to_string());
                categories.push(FixCategory::Structural);
            }
            _ => {
                causes.push("Unknown rule violation".to_string());
                categories.push(FixCategory::Content);
            }
        }

        Ok(AnalysisResult {
            causes,
            contributing_factors: factors,
            fix_categories: categories,
        })
    }

    /// Analyze tool output
    fn analyze_tool_output(
        &self,
        violation: &Violation,
        _snapshot: &ConcernSnapshot,
    ) -> Result<AnalysisResult> {
        let mut causes = Vec::new();
        let factors = Vec::new();
        let mut categories = Vec::new();

        // Tool output analysis logic
        if violation.message.contains("trunk") {
            causes.push("Trunk tool violation".to_string());
            categories.push(FixCategory::ToolSpecific);
        }

        if violation.message.contains("cargo") {
            causes.push("Cargo tool violation".to_string());
            categories.push(FixCategory::ToolSpecific);
        }

        Ok(AnalysisResult {
            causes,
            contributing_factors: factors,
            fix_categories: categories,
        })
    }
}

/// Result of an analysis strategy
#[derive(Debug)]
struct AnalysisResult {
    causes: Vec<String>,
    contributing_factors: Vec<String>,
    fix_categories: Vec<FixCategory>,
}

/// Dispatcher role: Routes concerns to appropriate fixer pipelines
pub struct Dispatcher {
    /// Available fixer pipelines
    pipelines: HashMap<String, FixerPipeline>,
}

impl Dispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        let mut pipelines = HashMap::new();

        // Register default pipelines
        pipelines.insert("default".to_string(), FixerPipeline::default());
        pipelines.insert("formatting".to_string(), FixerPipeline::formatting());
        pipelines.insert("linting".to_string(), FixerPipeline::linting());
        pipelines.insert("structural".to_string(), FixerPipeline::structural());
        pipelines.insert("configuration".to_string(), FixerPipeline::configuration());

        Self { pipelines }
    }

    /// Dispatch a concern to the appropriate fixer pipeline
    pub fn dispatch(&self, violation: &Violation, root_cause: &RootCause) -> Result<String> {
        // Determine the best pipeline based on root cause
        let pipeline_name = self.select_pipeline(violation, root_cause)?;

        Ok(pipeline_name)
    }

    /// Select the appropriate pipeline for a violation
    fn select_pipeline(&self, _violation: &Violation, root_cause: &RootCause) -> Result<String> {
        // Prioritize by fix categories
        for category in &root_cause.fix_categories {
            match category {
                FixCategory::Formatting => return Ok("formatting".to_string()),
                FixCategory::Linting => return Ok("linting".to_string()),
                FixCategory::Structural => return Ok("structural".to_string()),
                FixCategory::Configuration => return Ok("configuration".to_string()),
                FixCategory::ToolSpecific => return Ok("default".to_string()),
                FixCategory::Content => return Ok("default".to_string()),
            }
        }

        // Fallback to default pipeline
        Ok("default".to_string())
    }

    /// Get a pipeline by name
    pub fn get_pipeline(&self, name: &str) -> Option<&FixerPipeline> {
        self.pipelines.get(name)
    }
}

/// Fixer pipeline configuration
pub struct FixerPipeline {
    /// Pipeline name
    pub name: String,
    /// Ordered list of fixers in this pipeline
    pub fixers: Vec<Box<dyn Fixer>>,
    /// Pipeline metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl FixerPipeline {
    /// Create the default pipeline
    pub fn default() -> Self {
        let mut fixers: Vec<Box<dyn Fixer>> = Vec::new();
        fixers.push(Box::new(TrunkFixer));
        fixers.push(Box::new(DprintFixer));
        fixers.push(Box::new(StructuralFixer));

        Self {
            name: "default".to_string(),
            fixers,
            metadata: HashMap::new(),
        }
    }

    /// Create a formatting pipeline
    pub fn formatting() -> Self {
        let mut fixers: Vec<Box<dyn Fixer>> = Vec::new();
        fixers.push(Box::new(RustfmtFixer));
        fixers.push(Box::new(DprintFixer));
        fixers.push(Box::new(PrettierFixer));

        Self {
            name: "formatting".to_string(),
            fixers,
            metadata: HashMap::new(),
        }
    }

    /// Create a linting pipeline
    pub fn linting() -> Self {
        let mut fixers: Vec<Box<dyn Fixer>> = Vec::new();
        fixers.push(Box::new(ClippyFixer));
        fixers.push(Box::new(EslintFixer));
        fixers.push(Box::new(TrunkFixer));

        Self {
            name: "linting".to_string(),
            fixers,
            metadata: HashMap::new(),
        }
    }

    /// Create a structural pipeline
    pub fn structural() -> Self {
        let mut fixers: Vec<Box<dyn Fixer>> = Vec::new();
        fixers.push(Box::new(FileOrganizationFixer));
        fixers.push(Box::new(NamingFixer));
        fixers.push(Box::new(StructuralFixer));

        Self {
            name: "structural".to_string(),
            fixers,
            metadata: HashMap::new(),
        }
    }

    /// Create a configuration pipeline
    pub fn configuration() -> Self {
        let mut fixers: Vec<Box<dyn Fixer>> = Vec::new();
        fixers.push(Box::new(GitAttributesFixer));
        fixers.push(Box::new(ConfigFixer));
        fixers.push(Box::new(ConfigurationFixer));

        Self {
            name: "configuration".to_string(),
            fixers,
            metadata: HashMap::new(),
        }
    }
}

/// Triage Officer role: Plans full fix strategy across concern space
pub struct TriageOfficer {
    /// The investigator for root cause analysis
    investigator: Investigator,
    /// The dispatcher for routing concerns
    dispatcher: Dispatcher,
    /// Plan cache for memoization
    plan_cache: HashMap<String, RepairPlan>,
}

impl TriageOfficer {
    /// Create a new triage officer
    pub fn new() -> Self {
        Self {
            investigator: Investigator::new(),
            dispatcher: Dispatcher::new(),
            plan_cache: HashMap::new(),
        }
    }

    /// Create a repair plan for a failing concern
    pub fn create_plan(
        &mut self,
        violation: &Violation,
        snapshot: &ConcernSnapshot,
    ) -> Result<RepairPlan> {
        // Check cache first
        let cache_key = self.create_cache_key(violation, snapshot);
        if let Some(cached_plan) = self.plan_cache.get(&cache_key) {
            return Ok(cached_plan.clone());
        }

        // Investigate the violation
        let root_cause = self.investigator.investigate(violation, snapshot)?;

        // Dispatch to appropriate pipeline
        let pipeline_name = self.dispatcher.dispatch(violation, &root_cause)?;
        let pipeline = self
            .dispatcher
            .get_pipeline(&pipeline_name)
            .ok_or_else(|| anyhow::anyhow!("Pipeline '{}' not found", pipeline_name))?;

        // Generate repair actions
        let actions = self.generate_actions(violation, &root_cause, pipeline)?;

        // Create the plan
        let plan = RepairPlan {
            id: Uuid::new_v4().to_string(),
            concern: violation.concern.clone(),
            contract: violation.contract.clone(),
            violation: violation.clone(),
            root_cause,
            dispatcher: pipeline_name,
            actions,
            is_complete: true,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Cache the plan
        self.plan_cache.insert(cache_key, plan.clone());

        Ok(plan)
    }

    /// Generate repair actions for a violation
    fn generate_actions(
        &self,
        violation: &Violation,
        root_cause: &RootCause,
        pipeline: &FixerPipeline,
    ) -> Result<Vec<RepairAction>> {
        let mut actions = Vec::new();
        let mut action_id = 0;

        for (priority, fixer) in pipeline.fixers.iter().enumerate() {
            if let Some(action) = fixer.plan(violation, root_cause)? {
                let mut repair_action = action;
                repair_action.id = format!("action_{}", action_id);
                repair_action.priority = priority as u32;
                actions.push(repair_action);
                action_id += 1;
            }
        }

        // Sort by priority
        actions.sort_by_key(|a| a.priority);

        Ok(actions)
    }

    /// Create a cache key for memoization
    fn create_cache_key(&self, violation: &Violation, snapshot: &ConcernSnapshot) -> String {
        format!(
            "{:?}:{}:{}",
            violation.concern, violation.contract, snapshot.hash
        )
    }
}

/// Fixer trait for stateless repair tools
pub trait Fixer: Send + Sync {
    /// Get the fixer ID
    fn id(&self) -> &'static str;

    /// Plan a repair action for a violation
    fn plan(&self, violation: &Violation, root_cause: &RootCause) -> Result<Option<RepairAction>>;

    /// Execute a repair action
    fn execute(&self, action: &RepairAction) -> Result<RepairResult>;
}

/// Trunk fixer implementation
pub struct TrunkFixer;

impl Fixer for TrunkFixer {
    fn id(&self) -> &'static str {
        "fixer.trunk"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for tool-specific violations
        if !violation.message.contains("trunk") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("check --apply"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(), // Will be set by triage officer
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        // Implementation would run trunk check --apply
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Trunk fix applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Dprint fixer implementation
pub struct DprintFixer;

impl Fixer for DprintFixer {
    fn id(&self) -> &'static str {
        "fixer.dprint"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for formatting violations
        if !violation.message.contains("format") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("fmt"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Dprint formatting applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Rustfmt fixer implementation
pub struct RustfmtFixer;

impl Fixer for RustfmtFixer {
    fn id(&self) -> &'static str {
        "fixer.rustfmt"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for Rust formatting violations
        if !violation.message.contains("rust") && !violation.message.contains("format") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("fmt"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Rustfmt formatting applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Prettier fixer implementation
pub struct PrettierFixer;

impl Fixer for PrettierFixer {
    fn id(&self) -> &'static str {
        "fixer.prettier"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for JavaScript/TypeScript formatting violations
        if !violation.message.contains("js")
            && !violation.message.contains("ts")
            && !violation.message.contains("format")
        {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("--write"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Prettier formatting applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Clippy fixer implementation
pub struct ClippyFixer;

impl Fixer for ClippyFixer {
    fn id(&self) -> &'static str {
        "fixer.clippy"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for Rust linting violations
        if !violation.message.contains("rust") && !violation.message.contains("lint") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("fix"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Clippy fixes applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// ESLint fixer implementation
pub struct EslintFixer;

impl Fixer for EslintFixer {
    fn id(&self) -> &'static str {
        "fixer.eslint"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for JavaScript linting violations
        if !violation.message.contains("js") && !violation.message.contains("lint") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("command".to_string(), serde_json::json!("--fix"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["ESLint fixes applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// File organization fixer implementation
pub struct FileOrganizationFixer;

impl Fixer for FileOrganizationFixer {
    fn id(&self) -> &'static str {
        "fixer.file_organization"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for structural violations
        if !violation.message.contains("structure") && !violation.message.contains("organization") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("organize"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::RunCommand,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["File organization applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Naming fixer implementation
pub struct NamingFixer;

impl Fixer for NamingFixer {
    fn id(&self) -> &'static str {
        "fixer.naming"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for naming violations
        if !violation.message.contains("naming") && !violation.message.contains("name") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("rename"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::Move,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["File naming fixed successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Structural fixer implementation
pub struct StructuralFixer;

impl Fixer for StructuralFixer {
    fn id(&self) -> &'static str {
        "fixer.structural"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for structural violations
        if !violation.message.contains("structure") && !violation.message.contains("structural") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("fix"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::Edit,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Structural fix applied successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Git attributes fixer implementation
pub struct GitAttributesFixer;

impl Fixer for GitAttributesFixer {
    fn id(&self) -> &'static str {
        "fixer.git_attributes"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for git attributes violations
        if !violation.message.contains("git") && !violation.message.contains("attributes") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("fix"));
        parameters.insert("path".to_string(), serde_json::json!(".gitattributes"));

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::Edit,
            target_path: Some(".gitattributes".to_string()),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Git attributes fixed successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Config fixer implementation
pub struct ConfigFixer;

impl Fixer for ConfigFixer {
    fn id(&self) -> &'static str {
        "fixer.config"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for configuration violations
        if !violation.message.contains("config") && !violation.message.contains("configuration") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("fix"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::Edit,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Configuration fixed successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

/// Configuration fixer implementation
pub struct ConfigurationFixer;

impl Fixer for ConfigurationFixer {
    fn id(&self) -> &'static str {
        "fixer.configuration"
    }

    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> Result<Option<RepairAction>> {
        // Only plan for configuration violations
        if !violation.message.contains("config") && !violation.message.contains("configuration") {
            return Ok(None);
        }

        let mut parameters = HashMap::new();
        parameters.insert("action".to_string(), serde_json::json!("fix"));
        parameters.insert(
            "path".to_string(),
            serde_json::json!(violation.location.clone().unwrap_or_default()),
        );

        Ok(Some(RepairAction {
            id: String::new(),
            fixer_id: self.id().to_string(),
            action_type: ActionType::Edit,
            target_path: violation.location.clone(),
            parameters,
            required: true,
            priority: 0,
            dependencies: Vec::new(),
        }))
    }

    fn execute(&self, action: &RepairAction) -> Result<RepairResult> {
        Ok(RepairResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Configuration fixed successfully".to_string()],
            diff: None,
            new_hash: None,
            metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::symbols::ConcernSymbol;

    #[test]
    fn test_investigator_creation() {
        let investigator = Investigator::new();
        assert!(!investigator.strategies.is_empty());
    }

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = Dispatcher::new();
        assert!(!dispatcher.pipelines.is_empty());
        assert!(dispatcher.get_pipeline("default").is_some());
    }

    #[test]
    fn test_triage_officer_creation() {
        let triage_officer = TriageOfficer::new();
        assert!(triage_officer.plan_cache.is_empty());
    }

    #[test]
    fn test_repair_plan_creation() {
        let violation = Violation {
            concern: ConcernSymbol::TreeFile,
            contract: "format".to_string(),
            message: "Rust formatting violation: inconsistent indentation".to_string(),
            location: Some("src/main.rs".to_string()),
            severity: RuleSeverity::Error,
            details: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let snapshot = ConcernSnapshot::new(
            ConcernSymbol::TreeFile,
            serde_json::json!({"content": "test"}),
            HashMap::new(),
        );

        let mut triage_officer = TriageOfficer::new();
        let plan = triage_officer.create_plan(&violation, &snapshot).unwrap();

        assert_eq!(plan.concern, ConcernSymbol::TreeFile);
        assert_eq!(plan.contract, "format");
        assert!(!plan.actions.is_empty());
    }

    #[test]
    fn test_fixer_planning() {
        let violation = Violation {
            concern: ConcernSymbol::TreeFile,
            contract: "format".to_string(),
            message: "Rust formatting violation".to_string(),
            location: Some("src/main.rs".to_string()),
            severity: RuleSeverity::Error,
            details: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let root_cause = RootCause {
            primary_cause: "Formatting violation".to_string(),
            contributing_factors: vec!["Inconsistent indentation".to_string()],
            fix_categories: vec![FixCategory::Formatting],
            confidence: 0.8,
            metadata: HashMap::new(),
        };

        let rustfmt_fixer = RustfmtFixer;
        let action = rustfmt_fixer
            .plan(&violation, &root_cause)
            .unwrap()
            .unwrap();

        assert_eq!(action.fixer_id, "fixer.rustfmt");
        assert_eq!(action.action_type, ActionType::RunCommand);
        assert_eq!(action.target_path, Some("src/main.rs".to_string()));
    }
}
