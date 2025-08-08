use crate::modules::functional_contract_pipeline::symbols::ConcernSymbol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot of a concern's current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcernSnapshot {
    /// The concern symbol
    pub symbol: ConcernSymbol,
    /// Snapshot data as JSON
    pub data: serde_json::Value,
    /// Timestamp of snapshot
    pub timestamp: String,
    /// Hash of the snapshot data
    pub hash: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Expected snapshot based on contract rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSnapshot {
    /// The concern symbol
    pub symbol: ConcernSymbol,
    /// Expected data as JSON
    pub expectation: serde_json::Value,
    /// Contract that generated this expectation
    pub contract: String,
    /// Contract version
    pub contract_version: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Validation difference between observed and expected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDiff {
    /// Concern that has a difference
    pub concern: ConcernSymbol,
    /// Type of difference
    pub diff_type: DiffType,
    /// Description of the difference
    pub description: String,
    /// Observed value
    pub observed: Option<serde_json::Value>,
    /// Expected value
    pub expected: Option<serde_json::Value>,
    /// Severity of the difference
    pub severity: crate::modules::functional_contract_pipeline::symbols::RuleSeverity,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of validation differences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffType {
    /// Missing expected concern
    Missing,
    /// Unexpected observed concern
    Unexpected,
    /// Value mismatch
    Mismatch,
    /// Schema validation failure
    SchemaViolation,
    /// Rule violation
    RuleViolation,
}

impl DiffType {
    /// Get the name of the diff type
    pub fn name(&self) -> &'static str {
        match self {
            DiffType::Missing => "missing",
            DiffType::Unexpected => "unexpected",
            DiffType::Mismatch => "mismatch",
            DiffType::SchemaViolation => "schema_violation",
            DiffType::RuleViolation => "rule_violation",
        }
    }
}

/// Set of validation differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSet {
    /// List of differences
    pub diffs: Vec<ValidationDiff>,
    /// Whether validation passed
    pub is_valid: bool,
    /// Summary of validation
    pub summary: String,
}

/// Set of observed concern snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedConcernSet {
    /// Map of concern symbols to their snapshots
    pub snapshots: HashMap<ConcernSymbol, ConcernSnapshot>,
    /// Hook event that triggered this observation
    pub hook_event: crate::modules::functional_contract_pipeline::symbols::HookEvent,
    /// Timestamp of the observation
    pub timestamp: String,
}

/// Set of expected concern snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedConcernSet {
    /// Map of concern symbols to their expected snapshots
    pub snapshots: HashMap<ConcernSymbol, ExpectedSnapshot>,
    /// Hook event that triggered this expectation
    pub hook_event: crate::modules::functional_contract_pipeline::symbols::HookEvent,
    /// Timestamp of the expectation
    pub timestamp: String,
}

impl ConcernSnapshot {
    /// Create a new concern snapshot
    pub fn new(
        symbol: ConcernSymbol,
        data: serde_json::Value,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        let hash = Self::compute_hash(&data);
        Self {
            symbol,
            data,
            timestamp: chrono::Utc::now().to_rfc3339(),
            hash,
            metadata,
        }
    }

    /// Compute hash of data
    fn compute_hash(data: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};
        let json_string = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl ExpectedSnapshot {
    /// Create a new expected snapshot
    pub fn new(
        symbol: ConcernSymbol,
        expectation: serde_json::Value,
        contract: String,
        contract_version: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            symbol,
            expectation,
            contract,
            contract_version,
            metadata,
        }
    }
}

impl ValidationDiff {
    /// Create a new validation difference
    pub fn new(
        concern: ConcernSymbol,
        diff_type: DiffType,
        description: String,
        observed: Option<serde_json::Value>,
        expected: Option<serde_json::Value>,
        severity: crate::modules::functional_contract_pipeline::symbols::RuleSeverity,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            concern,
            diff_type,
            description,
            observed,
            expected,
            severity,
            metadata,
        }
    }
}

impl DiffSet {
    /// Create a new diff set
    pub fn new(diffs: Vec<ValidationDiff>) -> Self {
        let is_valid = diffs.iter().all(|diff| {
            matches!(
                diff.severity,
                crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Info
                    | crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Warning
            )
        });

        let summary = if is_valid {
            "Validation passed".to_string()
        } else {
            format!("Validation failed with {} differences", diffs.len())
        };

        Self {
            diffs,
            is_valid,
            summary,
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get the number of differences
    pub fn diff_count(&self) -> usize {
        self.diffs.len()
    }

    /// Get errors only
    pub fn errors(&self) -> Vec<&ValidationDiff> {
        self.diffs
            .iter()
            .filter(|diff| {
                matches!(
                    diff.severity,
                    crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Error
                        | crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Critical
                )
            })
            .collect()
    }

    /// Get warnings only
    pub fn warnings(&self) -> Vec<&ValidationDiff> {
        self.diffs
            .iter()
            .filter(|diff| {
                matches!(
                    diff.severity,
                    crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Warning
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, RuleSeverity};

    #[test]
    fn test_concern_snapshot_creation() {
        let data = serde_json::json!({"exists": true});
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::json!("git-index"));

        let snapshot = ConcernSnapshot::new(ConcernSymbol::TreeFile, data.clone(), metadata);
        
        assert_eq!(snapshot.symbol, ConcernSymbol::TreeFile);
        assert_eq!(snapshot.data, data);
        assert!(!snapshot.hash.is_empty());
        assert!(!snapshot.timestamp.is_empty());
    }

    #[test]
    fn test_expected_snapshot_creation() {
        let expectation = serde_json::json!({"exists": true});
        let mut metadata = HashMap::new();
        metadata.insert("rule".to_string(), serde_json::json!("must-exist"));

        let snapshot = ExpectedSnapshot::new(
            ConcernSymbol::TreeFile,
            expectation.clone(),
            "test-contract".to_string(),
            "1.0".to_string(),
            metadata,
        );
        
        assert_eq!(snapshot.symbol, ConcernSymbol::TreeFile);
        assert_eq!(snapshot.expectation, expectation);
        assert_eq!(snapshot.contract, "test-contract");
        assert_eq!(snapshot.contract_version, "1.0");
    }

    #[test]
    fn test_validation_diff_creation() {
        let diff = ValidationDiff::new(
            ConcernSymbol::TreeFile,
            DiffType::Mismatch,
            "File does not exist".to_string(),
            Some(serde_json::json!(false)),
            Some(serde_json::json!(true)),
            RuleSeverity::Error,
            HashMap::new(),
        );
        
        assert_eq!(diff.concern, ConcernSymbol::TreeFile);
        assert_eq!(diff.diff_type, DiffType::Mismatch);
        assert_eq!(diff.description, "File does not exist");
        assert_eq!(diff.severity, RuleSeverity::Error);
    }

    #[test]
    fn test_diff_set_creation() {
        let diffs = vec![
            ValidationDiff::new(
                ConcernSymbol::TreeFile,
                DiffType::Mismatch,
                "File does not exist".to_string(),
                Some(serde_json::json!(false)),
                Some(serde_json::json!(true)),
                RuleSeverity::Error,
                HashMap::new(),
            ),
        ];

        let diff_set = DiffSet::new(diffs);
        
        assert!(!diff_set.is_valid());
        assert_eq!(diff_set.diff_count(), 1);
        assert_eq!(diff_set.errors().len(), 1);
        assert_eq!(diff_set.warnings().len(), 0);
    }

    #[test]
    fn test_diff_set_with_warnings() {
        let diffs = vec![
            ValidationDiff::new(
                ConcernSymbol::TreeFile,
                DiffType::Unexpected,
                "Unexpected file".to_string(),
                Some(serde_json::json!(true)),
                None,
                RuleSeverity::Warning,
                HashMap::new(),
            ),
        ];

        let diff_set = DiffSet::new(diffs);
        
        assert!(diff_set.is_valid());
        assert_eq!(diff_set.diff_count(), 1);
        assert_eq!(diff_set.errors().len(), 0);
        assert_eq!(diff_set.warnings().len(), 1);
    }
}
