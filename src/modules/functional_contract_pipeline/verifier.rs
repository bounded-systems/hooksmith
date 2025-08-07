use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, RuleSeverity};
use crate::modules::functional_contract_pipeline::types::{DiffSet, DiffType, ValidationDiff};
use std::collections::HashMap;

/// Verify observed snapshots against expected snapshots
pub fn verify(
    observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
    expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
) -> Result<(), Vec<String>> {
    let mut errors = vec![];

    for ex in expected {
        if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
            if obs.data != ex.expectation {
                errors.push(format!(
                    "Mismatch on {:?}: observed {:?}, expected {:?}",
                    obs.symbol, obs.data, ex.expectation
                ));
            }
        } else {
            errors.push(format!("Missing observed snapshot for {:?}", ex.symbol));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Verify with detailed diff information
pub fn verify_with_diffs(
    observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
    expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
) -> DiffSet {
    let mut diffs = Vec::new();

    // Check for missing expected concerns
    for ex in expected {
        if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
            if obs.data != ex.expectation {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Mismatch,
                    format!("Data mismatch for concern: {}", ex.symbol.name()),
                    Some(obs.data.clone()),
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        } else {
            diffs.push(ValidationDiff::new(
                ex.symbol.clone(),
                DiffType::Missing,
                format!("Missing expected concern: {}", ex.symbol.name()),
                None,
                Some(ex.expectation.clone()),
                RuleSeverity::Error,
                HashMap::new(),
            ));
        }
    }

    // Check for unexpected observed concerns
    for obs in observed {
        if !expected.iter().any(|ex| ex.symbol == obs.symbol) {
            diffs.push(ValidationDiff::new(
                obs.symbol.clone(),
                DiffType::Unexpected,
                format!("Unexpected observed concern: {}", obs.symbol.name()),
                Some(obs.data.clone()),
                None,
                RuleSeverity::Warning,
                HashMap::new(),
            ));
        }
    }

    DiffSet::new(diffs)
}

/// Verify with custom severity mapping
pub fn verify_with_severity(
    observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
    expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    severity_map: &HashMap<ConcernSymbol, RuleSeverity>,
) -> DiffSet {
    let mut diffs = Vec::new();

    for ex in expected {
        let severity = severity_map.get(&ex.symbol).unwrap_or(&RuleSeverity::Error);
        
        if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
            if obs.data != ex.expectation {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Mismatch,
                    format!("Data mismatch for concern: {}", ex.symbol.name()),
                    Some(obs.data.clone()),
                    Some(ex.expectation.clone()),
                    severity.clone(),
                    HashMap::new(),
                ));
            }
        } else {
            diffs.push(ValidationDiff::new(
                ex.symbol.clone(),
                DiffType::Missing,
                format!("Missing expected concern: {}", ex.symbol.name()),
                None,
                Some(ex.expectation.clone()),
                                    severity.clone(),
                HashMap::new(),
            ));
        }
    }

    DiffSet::new(diffs)
}

/// Verify with tolerance for specific fields
pub fn verify_with_tolerance(
    observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
    expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    tolerance_fields: &HashMap<ConcernSymbol, Vec<String>>,
) -> DiffSet {
    let mut diffs = Vec::new();

    for ex in expected {
        if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
            let empty_vec = Vec::new();
            let tolerance = tolerance_fields.get(&ex.symbol).unwrap_or(&empty_vec);
            
            if !compare_with_tolerance(&obs.data, &ex.expectation, tolerance) {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Mismatch,
                    format!("Data mismatch for concern: {} (with tolerance)", ex.symbol.name()),
                    Some(obs.data.clone()),
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        } else {
            diffs.push(ValidationDiff::new(
                ex.symbol.clone(),
                DiffType::Missing,
                format!("Missing expected concern: {}", ex.symbol.name()),
                None,
                Some(ex.expectation.clone()),
                RuleSeverity::Error,
                HashMap::new(),
            ));
        }
    }

    DiffSet::new(diffs)
}

/// Compare JSON values with tolerance for specific fields
fn compare_with_tolerance(
    observed: &serde_json::Value,
    expected: &serde_json::Value,
    tolerance_fields: &[String],
) -> bool {
    match (observed, expected) {
        (serde_json::Value::Object(obs), serde_json::Value::Object(exp)) => {
            for (key, exp_val) in exp {
                if tolerance_fields.contains(key) {
                    // Skip tolerance fields
                    continue;
                }
                
                if let Some(obs_val) = obs.get(key) {
                    if !compare_with_tolerance(obs_val, exp_val, tolerance_fields) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
        (serde_json::Value::Array(obs), serde_json::Value::Array(exp)) => {
            if obs.len() != exp.len() {
                return false;
            }
            for (obs_val, exp_val) in obs.iter().zip(exp.iter()) {
                if !compare_with_tolerance(obs_val, exp_val, tolerance_fields) {
                    return false;
                }
            }
            true
        }
        _ => observed == expected,
    }
}

/// Verify with custom comparison function
pub fn verify_with_custom_comparison<F>(
    observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
    expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    compare_fn: F,
) -> DiffSet
where
    F: Fn(&serde_json::Value, &serde_json::Value) -> bool,
{
    let mut diffs = Vec::new();

    for ex in expected {
        if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
            if !compare_fn(&obs.data, &ex.expectation) {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Mismatch,
                    format!("Data mismatch for concern: {} (custom comparison)", ex.symbol.name()),
                    Some(obs.data.clone()),
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        } else {
            diffs.push(ValidationDiff::new(
                ex.symbol.clone(),
                DiffType::Missing,
                format!("Missing expected concern: {}", ex.symbol.name()),
                None,
                Some(ex.expectation.clone()),
                RuleSeverity::Error,
                HashMap::new(),
            ));
        }
    }

    DiffSet::new(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};

    #[test]
    fn test_verify_success() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        let result = verify(&observed, &expected);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_failure() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        let result = verify(&observed, &expected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Mismatch"));
    }

    #[test]
    fn test_verify_missing_concern() {
        let observed = vec![];
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        let result = verify(&observed, &expected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Missing observed snapshot"));
    }

    #[test]
    fn test_verify_with_diffs() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        let diff_set = verify_with_diffs(&observed, &expected);
        assert!(!diff_set.is_valid());
        assert_eq!(diff_set.diff_count(), 1);
        assert_eq!(diff_set.errors().len(), 1);
    }

    #[test]
    fn test_verify_with_tolerance() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({
                    "exists": true,
                    "timestamp": "2023-01-01T00:00:00Z",
                    "files": ["a.txt", "b.txt"]
                }),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({
                    "exists": true,
                    "timestamp": "2023-01-02T00:00:00Z",
                    "files": ["a.txt", "b.txt"]
                }),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        let mut tolerance_fields = HashMap::new();
        tolerance_fields.insert(ConcernSymbol::Index, vec!["timestamp".to_string()]);
        
        let diff_set = verify_with_tolerance(&observed, &expected, &tolerance_fields);
        assert!(diff_set.is_valid()); // Should pass because timestamp is tolerated
    }

    #[test]
    fn test_verify_with_custom_comparison() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"count": 5}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"count": 10}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];
        
        // Custom comparison: check if count is greater than 0
        let diff_set = verify_with_custom_comparison(&observed, &expected, |obs, exp| {
            if let (Some(obs_count), Some(exp_count)) = (obs.get("count"), exp.get("count")) {
                if let (Some(obs_val), Some(exp_val)) = (obs_count.as_u64(), exp_count.as_u64()) {
                    return obs_val > 0 && exp_val > 0; // Both should be > 0
                }
            }
            false
        });
        
        assert!(diff_set.is_valid()); // Should pass with custom comparison
    }
}
