use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, ContractSymbol};
use crate::modules::functional_contract_pipeline::types::ExpectedSnapshot;
use std::collections::HashMap;

/// Build expectation from contract
pub fn build_expectation(contract: &ContractSymbol) -> ExpectedSnapshot {
    match contract.name() {
        "must-exist" => ExpectedSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"exists": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "no-unstaged-changes" => ExpectedSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"unstaged_files": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "no-untracked-files" => ExpectedSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"untracked_files": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "text-files-normalized" => ExpectedSnapshot::new(
            ConcernSymbol::AttrLineEndingNormalization,
            serde_json::json!({"text_files": [], "line_ending_rules": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "binary-files-identified" => ExpectedSnapshot::new(
            ConcernSymbol::AttrLineEndingNormalization,
            serde_json::json!({"binary_files": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "diff-rules-defined" => ExpectedSnapshot::new(
            ConcernSymbol::AttrDiffStrategy,
            serde_json::json!({"diff_rules": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "binary-files-handled" => ExpectedSnapshot::new(
            ConcernSymbol::AttrDiffStrategy,
            serde_json::json!({"binary_files": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "must-match-origin" => ExpectedSnapshot::new(
            ConcernSymbol::Ref,
            serde_json::json!({"ref_eq": "origin/main"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "ref-naming-convention" => ExpectedSnapshot::new(
            ConcernSymbol::Ref,
            serde_json::json!({"naming_convention": "valid"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "branch-naming-convention" => ExpectedSnapshot::new(
            ConcernSymbol::Branch,
            serde_json::json!({"naming_convention": "valid"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "protected-branches" => ExpectedSnapshot::new(
            ConcernSymbol::Branch,
            serde_json::json!({"protected_branches": ["main", "develop"]}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "remote-url-validation" => ExpectedSnapshot::new(
            ConcernSymbol::Remote,
            serde_json::json!({"url_validation": "valid"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "remote-authentication" => ExpectedSnapshot::new(
            ConcernSymbol::Remote,
            serde_json::json!({"authentication": "valid"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "no-new-executables" => ExpectedSnapshot::new(
            ConcernSymbol::TreeExecutable,
            serde_json::json!({"executable_files": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "executable-safety-check" => ExpectedSnapshot::new(
            ConcernSymbol::TreeExecutable,
            serde_json::json!({"safety_check": "passed"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "commit-message-format" => ExpectedSnapshot::new(
            ConcernSymbol::Commit,
            serde_json::json!({"message_format": "conventional"}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "commit-author-valid" => ExpectedSnapshot::new(
            ConcernSymbol::Commit,
            serde_json::json!({"author_valid": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "tree-structure-valid" => ExpectedSnapshot::new(
            ConcernSymbol::Tree,
            serde_json::json!({"structure_valid": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "no-orphaned-trees" => ExpectedSnapshot::new(
            ConcernSymbol::Tree,
            serde_json::json!({"orphaned_trees": []}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "head-points-to-branch" => ExpectedSnapshot::new(
            ConcernSymbol::Head,
            serde_json::json!({"points_to_branch": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "head-detached-allowed" => ExpectedSnapshot::new(
            ConcernSymbol::Head,
            serde_json::json!({"detached_allowed": false}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "commit-template-defined" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigCommit,
            serde_json::json!({"template_defined": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "commit-gpgsign-enabled" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigCommit,
            serde_json::json!({"gpgsign_enabled": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "user-name-defined" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigUser,
            serde_json::json!({"name_defined": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "user-email-defined" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigUser,
            serde_json::json!({"email_defined": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "gc-auto-enabled" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigGc,
            serde_json::json!({"auto_enabled": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "gc-prune-expire-set" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigGc,
            serde_json::json!({"prune_expire_set": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "rebase-strategy-defined" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigRebase,
            serde_json::json!({"strategy_defined": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        "rebase-auto-stash-enabled" => ExpectedSnapshot::new(
            ConcernSymbol::ConfigRebase,
            serde_json::json!({"auto_stash_enabled": true}),
            contract.name().to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
        _ => {
            // Default expectation for unknown contracts
            let mut metadata = HashMap::new();
            metadata.insert("status".to_string(), serde_json::json!("unknown-contract"));
            ExpectedSnapshot::new(
                ConcernSymbol::Index, // Default concern
                serde_json::json!({ "error": "Unknown contract" }),
                contract.name().to_string(),
                "1.0".to_string(),
                metadata,
            )
        }
    }
}

/// Build expectations for multiple contracts
pub fn build_expectations(contracts: &[ContractSymbol]) -> Vec<ExpectedSnapshot> {
    contracts.iter().map(build_expectation).collect()
}

/// Build expectations for contracts grouped by concern
pub fn build_expectations_by_concern(
    contracts: &[ContractSymbol],
) -> std::collections::HashMap<ConcernSymbol, Vec<ExpectedSnapshot>> {
    let mut grouped = std::collections::HashMap::new();

    for contract in contracts {
        let expectation = build_expectation(contract);
        grouped
            .entry(expectation.symbol.clone())
            .or_insert_with(Vec::new)
            .push(expectation);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_expectation_must_exist() {
        let contract = ContractSymbol::new("must-exist");
        let expectation = build_expectation(&contract);

        assert_eq!(expectation.symbol, ConcernSymbol::Index);
        assert_eq!(
            expectation.expectation.get("exists"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(expectation.contract, "must-exist");
    }

    #[test]
    fn test_build_expectation_no_unstaged_changes() {
        let contract = ContractSymbol::new("no-unstaged-changes");
        let expectation = build_expectation(&contract);

        assert_eq!(expectation.symbol, ConcernSymbol::Index);
        assert_eq!(
            expectation.expectation.get("unstaged_files"),
            Some(&serde_json::json!([]))
        );
    }

    #[test]
    fn test_build_expectation_text_files_normalized() {
        let contract = ContractSymbol::new("text-files-normalized");
        let expectation = build_expectation(&contract);

        assert_eq!(
            expectation.symbol,
            ConcernSymbol::AttrLineEndingNormalization
        );
        assert!(expectation.expectation.get("text_files").is_some());
        assert!(expectation.expectation.get("line_ending_rules").is_some());
    }

    #[test]
    fn test_build_expectation_no_new_executables() {
        let contract = ContractSymbol::new("no-new-executables");
        let expectation = build_expectation(&contract);

        assert_eq!(expectation.symbol, ConcernSymbol::TreeExecutable);
        assert_eq!(
            expectation.expectation.get("executable_files"),
            Some(&serde_json::json!([]))
        );
    }

    #[test]
    fn test_build_expectations_multiple() {
        let contracts = vec![
            ContractSymbol::new("must-exist"),
            ContractSymbol::new("no-unstaged-changes"),
        ];
        let expectations = build_expectations(&contracts);

        assert_eq!(expectations.len(), 2);
        assert_eq!(expectations[0].symbol, ConcernSymbol::Index);
        assert_eq!(expectations[1].symbol, ConcernSymbol::Index);
    }

    #[test]
    fn test_build_expectations_by_concern() {
        let contracts = vec![
            ContractSymbol::new("must-exist"),
            ContractSymbol::new("no-new-executables"),
        ];
        let grouped = build_expectations_by_concern(&contracts);

        assert_eq!(grouped.len(), 2);
        assert!(grouped.contains_key(&ConcernSymbol::Index));
        assert!(grouped.contains_key(&ConcernSymbol::TreeExecutable));
        assert_eq!(grouped[&ConcernSymbol::Index].len(), 1);
        assert_eq!(grouped[&ConcernSymbol::TreeExecutable].len(), 1);
    }

    #[test]
    fn test_unknown_contract() {
        let contract = ContractSymbol::new("unknown-contract");
        let expectation = build_expectation(&contract);

        assert_eq!(expectation.symbol, ConcernSymbol::Index); // Default concern
        assert!(expectation.expectation.get("error").is_some());
        assert_eq!(
            expectation.metadata.get("status").unwrap(),
            &serde_json::json!("unknown-contract")
        );
    }
}
