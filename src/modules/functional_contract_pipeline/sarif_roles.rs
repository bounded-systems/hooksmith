use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, ContractSymbol, HookEvent, RuleSeverity};
use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot, ValidationDiff};
use serde_sarif::sarif::{ArtifactLocation, Location, Message, PhysicalLocation, Result as SarifResult, Run, SarifLog, Tool, ToolComponent};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// SARIF-first validation roles as defined in the slot-based schema
pub mod roles {
    use super::*;

    /// Hook role: Triggers concern collection (never fails)
    pub struct Hook {
        pub event: HookEvent,
        pub concerns: Vec<ConcernSymbol>,
    }

    impl Hook {
        /// Create a new hook role
        pub fn new(event: HookEvent) -> Self {
            Self {
                event,
                concerns: Vec::new(),
            }
        }

        /// Collect concerns for the hook (stateless, never fails)
        pub fn collect_concerns(&self) -> Vec<ConcernSymbol> {
            // This would use the existing hooks module
            crate::modules::functional_contract_pipeline::hooks::get_concerns(&self.event)
        }
    }

    /// Concern role: Extracts Git object snapshots (never fails)
    pub struct Concern {
        pub symbol: ConcernSymbol,
        pub snapshot: Option<ConcernSnapshot>,
    }

    impl Concern {
        /// Create a new concern role
        pub fn new(symbol: ConcernSymbol) -> Self {
            Self {
                symbol,
                snapshot: None,
            }
        }

        /// Take a snapshot of the concern (stateless, never fails)
        pub fn take_snapshot(&self) -> ConcernSnapshot {
            crate::modules::functional_contract_pipeline::concerns::snapshot_concern(&self.symbol)
        }
    }

    /// Specifier role: Generates contract expectations (never fails)
    pub struct Specifier {
        pub contracts: Vec<ContractSymbol>,
    }

    impl Specifier {
        /// Create a new specifier role
        pub fn new() -> Self {
            Self {
                contracts: Vec::new(),
            }
        }

        /// Generate expectations from contracts (stateless, never fails)
        pub fn generate_expectations(&self, contracts: &[ContractSymbol]) -> Vec<ExpectedSnapshot> {
            contracts
                .iter()
                .map(|contract| crate::modules::functional_contract_pipeline::specifier::build_expectation(contract))
                .collect()
        }
    }

    /// Verifier role: Compares snapshots to contracts, emits SARIF (never fails)
    pub struct Verifier {
        pub strategy: crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy,
    }

    impl Verifier {
        /// Create a new verifier role
        pub fn new(strategy: crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy) -> Self {
            Self { strategy }
        }

        /// Compare snapshots to expectations and emit SARIF (stateless, never fails)
        pub fn verify_and_emit_sarif(
            &self,
            observed: &[ConcernSnapshot],
            expected: &[ExpectedSnapshot],
            hook_event: &HookEvent,
        ) -> SarifLog {
            let (diff_set, _metrics) = crate::modules::functional_contract_pipeline::high_performance_diff::convenience::diff_with_strategy(
                self.strategy,
                observed,
                expected,
            );

            self.convert_diffs_to_sarif(&diff_set, hook_event)
        }

        /// Convert validation diffs to SARIF format
        fn convert_diffs_to_sarif(&self, diff_set: &crate::modules::functional_contract_pipeline::types::DiffSet, hook_event: &HookEvent) -> SarifLog {
            let mut results = Vec::new();

            for diff in &diff_set.diffs {
                let result = SarifResult::builder()
                    .rule_id(format!("{}-{}", diff.concern.name(), diff.diff_type.name()))
                    .level(match diff.severity {
                        RuleSeverity::Error => "error",
                        RuleSeverity::Warning => "warning",
                        RuleSeverity::Info => "note",
                    })
                    .message(Message::builder()
                        .text(diff.description.clone())
                        .build())
                    .locations(vec![Location::builder()
                        .physical_location(PhysicalLocation::builder()
                            .artifact_location(ArtifactLocation::builder()
                                .uri(format!("git://concern/{}", diff.concern.name()))
                                .build())
                            .build())
                        .build()])
                    .properties(HashMap::from([
                        ("concern".to_string(), serde_json::Value::String(diff.concern.name().to_string())),
                        ("diff_type".to_string(), serde_json::Value::String(diff.diff_type.name().to_string())),
                        ("origin".to_string(), serde_json::Value::String(format!("hook/{:?}", hook_event))),
                        ("severity".to_string(), serde_json::Value::String(format!("{:?}", diff.severity))),
                    ]))
                    .build();

                results.push(result);
            }

            let tool = Tool::builder()
                .driver(ToolComponent::builder()
                    .name("Hooksmith Contract Validator")
                    .version("1.0.0")
                    .build())
                .build();

            let run = Run::builder()
                .tool(tool)
                .results(results)
                .build();

            SarifLog::builder()
                .version("2.1.0")
                .runs(vec![run])
                .build()
        }
    }

    /// Stegrapher role: Logs and indexes SARIF entries with provenance (never fails)
    pub struct Stegrapher {
        pub indexed_entries: HashMap<String, SarifResult>,
        pub provenance_map: HashMap<String, HashMap<String, String>>,
    }

    impl Stegrapher {
        /// Create a new stegrapher role
        pub fn new() -> Self {
            Self {
                indexed_entries: HashMap::new(),
                provenance_map: HashMap::new(),
            }
        }

        /// Index SARIF entries and add provenance (stateless, never fails)
        pub fn index_and_tag_sarif(&mut self, sarif_log: &SarifLog, git_metadata: &GitMetadata) -> SarifLog {
            let mut indexed_results = Vec::new();

            for run in &sarif_log.runs {
                for result in &run.results {
                    let entry_id = Uuid::new_v4().to_string();
                    
                    // Index the entry
                    self.indexed_entries.insert(entry_id.clone(), result.clone());
                    
                    // Add provenance metadata
                    let mut provenance = HashMap::new();
                    provenance.insert("entry_id".to_string(), entry_id);
                    provenance.insert("commit_hash".to_string(), git_metadata.commit_hash.clone());
                    provenance.insert("tree_hash".to_string(), git_metadata.tree_hash.clone());
                    provenance.insert("timestamp".to_string(), git_metadata.timestamp.to_string());
                    provenance.insert("hook_event".to_string(), git_metadata.hook_event.clone());
                    
                    self.provenance_map.insert(entry_id.clone(), provenance.clone());
                    
                    // Create indexed result with provenance
                    let mut indexed_result = result.clone();
                    if let Some(props) = indexed_result.properties.as_mut() {
                        for (key, value) in provenance {
                            props.insert(key, value);
                        }
                    }
                    
                    indexed_results.push(indexed_result);
                }
            }

            // Create new SARIF log with indexed entries
            let tool = Tool::builder()
                .driver(ToolComponent::builder()
                    .name("Hooksmith Stegrapher")
                    .version("1.0.0")
                    .build())
                .build();

            let run = Run::builder()
                .tool(tool)
                .results(indexed_results)
                .build();

            SarifLog::builder()
                .version("2.1.0")
                .runs(vec![run])
                .build()
        }

        /// Query indexed entries by criteria
        pub fn query_entries(&self, criteria: &QueryCriteria) -> Vec<&SarifResult> {
            self.indexed_entries
                .values()
                .filter(|result| self.matches_criteria(result, criteria))
                .collect()
        }

        /// Check if a result matches query criteria
        fn matches_criteria(&self, result: &SarifResult, criteria: &QueryCriteria) -> bool {
            if let Some(props) = &result.properties {
                if let Some(concern) = &criteria.concern {
                    if let Some(result_concern) = props.get("concern") {
                        if result_concern.as_str() != Some(concern) {
                            return false;
                        }
                    }
                }
                
                if let Some(severity) = &criteria.severity {
                    if let Some(result_severity) = props.get("severity") {
                        if result_severity.as_str() != Some(&severity.to_string()) {
                            return false;
                        }
                    }
                }
                
                if let Some(hook_event) = &criteria.hook_event {
                    if let Some(result_hook) = props.get("hook_event") {
                        if result_hook.as_str() != Some(hook_event) {
                            return false;
                        }
                    }
                }
            }
            
            true
        }
    }

    /// Auditor role: Queries SARIF logs and determines pass/fail (can fail)
    pub struct Auditor {
        pub policies: Vec<AuditPolicy>,
    }

    impl Auditor {
        /// Create a new auditor role
        pub fn new() -> Self {
            Self {
                policies: Vec::new(),
            }
        }

        /// Add an audit policy
        pub fn add_policy(&mut self, policy: AuditPolicy) {
            self.policies.push(policy);
        }

        /// Audit SARIF logs and determine pass/fail (can fail)
        pub fn audit_sarif(&self, sarif_log: &SarifLog) -> AuditResult {
            let mut violations = Vec::new();
            let mut warnings = Vec::new();

            for run in &sarif_log.runs {
                for result in &run.results {
                    for policy in &self.policies {
                        if policy.matches(result) {
                            match policy.action {
                                AuditAction::Fail => violations.push(result.clone()),
                                AuditAction::Warn => warnings.push(result.clone()),
                                AuditAction::Info => {}, // Just log
                            }
                        }
                    }
                }
            }

            if !violations.is_empty() {
                AuditResult::Fail {
                    violations,
                    warnings,
                }
            } else {
                AuditResult::Pass {
                    warnings,
                }
            }
        }

        /// Query SARIF logs with custom criteria
        pub fn query_sarif(&self, sarif_log: &SarifLog, criteria: &QueryCriteria) -> Vec<&SarifResult> {
            sarif_log.runs
                .iter()
                .flat_map(|run| &run.results)
                .filter(|result| self.matches_criteria(result, criteria))
                .collect()
        }

        /// Check if a result matches query criteria
        fn matches_criteria(&self, result: &SarifResult, criteria: &QueryCriteria) -> bool {
            if let Some(props) = &result.properties {
                if let Some(concern) = &criteria.concern {
                    if let Some(result_concern) = props.get("concern") {
                        if result_concern.as_str() != Some(concern) {
                            return false;
                        }
                    }
                }
                
                if let Some(severity) = &criteria.severity {
                    if let Some(result_severity) = props.get("severity") {
                        if result_severity.as_str() != Some(&severity.to_string()) {
                            return false;
                        }
                    }
                }
                
                if let Some(hook_event) = &criteria.hook_event {
                    if let Some(result_hook) = props.get("hook_event") {
                        if result_hook.as_str() != Some(hook_event) {
                            return false;
                        }
                    }
                }
            }
            
            true
        }
    }
}

/// Git metadata for provenance tracking
#[derive(Debug, Clone)]
pub struct GitMetadata {
    pub commit_hash: String,
    pub tree_hash: String,
    pub timestamp: u64,
    pub hook_event: String,
    pub repository: String,
}

impl GitMetadata {
    /// Create new Git metadata
    pub fn new(commit_hash: String, tree_hash: String, hook_event: HookEvent) -> Self {
        Self {
            commit_hash,
            tree_hash,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            hook_event: format!("{:?}", hook_event),
            repository: ".".to_string(),
        }
    }
}

/// Query criteria for filtering SARIF results
#[derive(Debug, Clone)]
pub struct QueryCriteria {
    pub concern: Option<String>,
    pub severity: Option<RuleSeverity>,
    pub hook_event: Option<String>,
    pub diff_type: Option<String>,
}

impl QueryCriteria {
    /// Create new query criteria
    pub fn new() -> Self {
        Self {
            concern: None,
            severity: None,
            hook_event: None,
            diff_type: None,
        }
    }

    /// Set concern filter
    pub fn with_concern(mut self, concern: String) -> Self {
        self.concern = Some(concern);
        self
    }

    /// Set severity filter
    pub fn with_severity(mut self, severity: RuleSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set hook event filter
    pub fn with_hook_event(mut self, hook_event: String) -> Self {
        self.hook_event = Some(hook_event);
        self
    }

    /// Set diff type filter
    pub fn with_diff_type(mut self, diff_type: String) -> Self {
        self.diff_type = Some(diff_type);
        self
    }
}

/// Audit policy for the Auditor role
#[derive(Debug, Clone)]
pub struct AuditPolicy {
    pub name: String,
    pub description: String,
    pub criteria: QueryCriteria,
    pub action: AuditAction,
}

impl AuditPolicy {
    /// Create a new audit policy
    pub fn new(name: String, description: String, criteria: QueryCriteria, action: AuditAction) -> Self {
        Self {
            name,
            description,
            criteria,
            action,
        }
    }

    /// Check if a SARIF result matches this policy
    pub fn matches(&self, result: &SarifResult) -> bool {
        if let Some(props) = &result.properties {
            if let Some(concern) = &self.criteria.concern {
                if let Some(result_concern) = props.get("concern") {
                    if result_concern.as_str() != Some(concern) {
                        return false;
                    }
                }
            }
            
            if let Some(severity) = &self.criteria.severity {
                if let Some(result_severity) = props.get("severity") {
                    if result_severity.as_str() != Some(&severity.to_string()) {
                        return false;
                    }
                }
            }
            
            if let Some(hook_event) = &self.criteria.hook_event {
                if let Some(result_hook) = props.get("hook_event") {
                    if result_hook.as_str() != Some(hook_event) {
                        return false;
                    }
                }
            }
        }
        
        true
    }
}

/// Audit action types
#[derive(Debug, Clone, PartialEq)]
pub enum AuditAction {
    Fail,
    Warn,
    Info,
}

/// Audit result from the Auditor role
#[derive(Debug, Clone)]
pub enum AuditResult {
    Pass {
        warnings: Vec<SarifResult>,
    },
    Fail {
        violations: Vec<SarifResult>,
        warnings: Vec<SarifResult>,
    },
}

impl AuditResult {
    /// Check if the audit passed
    pub fn is_pass(&self) -> bool {
        matches!(self, AuditResult::Pass { .. })
    }

    /// Get all violations (empty if passed)
    pub fn violations(&self) -> &[SarifResult] {
        match self {
            AuditResult::Pass { .. } => &[],
            AuditResult::Fail { violations, .. } => violations,
        }
    }

    /// Get all warnings
    pub fn warnings(&self) -> &[SarifResult] {
        match self {
            AuditResult::Pass { warnings } => warnings,
            AuditResult::Fail { warnings, .. } => warnings,
        }
    }
}

/// Enhanced pipeline with SARIF-first architecture
pub struct SarifFirstPipeline {
    pub hook: roles::Hook,
    pub concern: roles::Concern,
    pub specifier: roles::Specifier,
    pub verifier: roles::Verifier,
    pub stegrapher: roles::Stegrapher,
    pub auditor: roles::Auditor,
}

impl SarifFirstPipeline {
    /// Create a new SARIF-first pipeline
    pub fn new() -> Self {
        Self {
            hook: roles::Hook::new(HookEvent::PreCommit),
            concern: roles::Concern::new(ConcernSymbol::Index),
            specifier: roles::Specifier::new(),
            verifier: roles::Verifier::new(crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy::JsonPatch),
            stegrapher: roles::Stegrapher::new(),
            auditor: roles::Auditor::new(),
        }
    }

    /// Run the complete SARIF-first pipeline
    pub fn run_pipeline(&mut self, hook_event: HookEvent, git_metadata: GitMetadata) -> (SarifLog, AuditResult) {
        // 1. Hook: Collect concerns (never fails)
        self.hook.event = hook_event;
        let concerns = self.hook.collect_concerns();

        // 2. Concern: Take snapshots (never fails)
        let snapshots: Vec<ConcernSnapshot> = concerns
            .iter()
            .map(|concern| {
                self.concern.symbol = concern.clone();
                self.concern.take_snapshot()
            })
            .collect();

        // 3. Specifier: Generate expectations (never fails)
        let contracts = crate::modules::functional_contract_pipeline::contracts::get_all_contracts(&concerns);
        let expectations = self.specifier.generate_expectations(&contracts);

        // 4. Verifier: Compare and emit SARIF (never fails)
        let sarif_log = self.verifier.verify_and_emit_sarif(&snapshots, &expectations, &hook_event);

        // 5. Stegrapher: Index and tag SARIF (never fails)
        let indexed_sarif = self.stegrapher.index_and_tag_sarif(&sarif_log, &git_metadata);

        // 6. Auditor: Query and determine pass/fail (can fail)
        let audit_result = self.auditor.audit_sarif(&indexed_sarif);

        (indexed_sarif, audit_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sarif_first_pipeline() {
        let mut pipeline = SarifFirstPipeline::new();
        let git_metadata = GitMetadata::new(
            "abc123".to_string(),
            "def456".to_string(),
            HookEvent::PreCommit,
        );

        let (sarif_log, audit_result) = pipeline.run_pipeline(HookEvent::PreCommit, git_metadata);

        // Verify SARIF log was generated
        assert!(!sarif_log.runs.is_empty());
        
        // Verify audit result (should pass by default with no policies)
        assert!(audit_result.is_pass());
    }

    #[test]
    fn test_audit_policy() {
        let mut auditor = roles::Auditor::new();
        
        let policy = AuditPolicy::new(
            "no-executable-files".to_string(),
            "Prevent executable files".to_string(),
            QueryCriteria::new().with_concern("TreeExecutable".to_string()),
            AuditAction::Fail,
        );
        
        auditor.add_policy(policy);
        
        // Test with empty SARIF log
        let empty_sarif = SarifLog::builder()
            .version("2.1.0")
            .runs(vec![])
            .build();
        
        let result = auditor.audit_sarif(&empty_sarif);
        assert!(result.is_pass());
    }

    #[test]
    fn test_query_criteria() {
        let criteria = QueryCriteria::new()
            .with_concern("Index".to_string())
            .with_severity(RuleSeverity::Error)
            .with_hook_event("PreCommit".to_string());
        
        assert_eq!(criteria.concern, Some("Index".to_string()));
        assert_eq!(criteria.severity, Some(RuleSeverity::Error));
        assert_eq!(criteria.hook_event, Some("PreCommit".to_string()));
    }
}
