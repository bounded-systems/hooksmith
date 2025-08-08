use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, ContractSymbol};

/// Get contracts for a concern
pub fn get_contracts(concern: &ConcernSymbol) -> Vec<ContractSymbol> {
    match concern {
        ConcernSymbol::Index => vec![
            ContractSymbol::new("must-exist"),
            ContractSymbol::new("no-unstaged-changes"),
            ContractSymbol::new("no-untracked-files"),
        ],
        ConcernSymbol::AttrLineEndingNormalization => vec![
            ContractSymbol::new("text-files-normalized"),
            ContractSymbol::new("binary-files-identified"),
        ],
        ConcernSymbol::AttrDiffStrategy => vec![
            ContractSymbol::new("diff-rules-defined"),
            ContractSymbol::new("binary-files-handled"),
        ],
        ConcernSymbol::Ref => vec![
            ContractSymbol::new("must-match-origin"),
            ContractSymbol::new("ref-naming-convention"),
        ],
        ConcernSymbol::Branch => vec![
            ContractSymbol::new("branch-naming-convention"),
            ContractSymbol::new("protected-branches"),
        ],
        ConcernSymbol::Remote => vec![
            ContractSymbol::new("remote-url-validation"),
            ContractSymbol::new("remote-authentication"),
        ],
        ConcernSymbol::TreeExecutable => vec![
            ContractSymbol::new("no-new-executables"),
            ContractSymbol::new("executable-safety-check"),
        ],
        ConcernSymbol::Commit => vec![
            ContractSymbol::new("commit-message-format"),
            ContractSymbol::new("commit-author-valid"),
        ],
        ConcernSymbol::Tree => vec![
            ContractSymbol::new("tree-structure-valid"),
            ContractSymbol::new("no-orphaned-trees"),
        ],
        ConcernSymbol::Head => vec![
            ContractSymbol::new("head-points-to-branch"),
            ContractSymbol::new("head-detached-allowed"),
        ],
        ConcernSymbol::ConfigCommit => vec![
            ContractSymbol::new("commit-template-defined"),
            ContractSymbol::new("commit-gpgsign-enabled"),
        ],
        ConcernSymbol::ConfigUser => vec![
            ContractSymbol::new("user-name-defined"),
            ContractSymbol::new("user-email-defined"),
        ],
        ConcernSymbol::ConfigGc => vec![
            ContractSymbol::new("gc-auto-enabled"),
            ContractSymbol::new("gc-prune-expire-set"),
        ],
        ConcernSymbol::ConfigRebase => vec![
            ContractSymbol::new("rebase-strategy-defined"),
            ContractSymbol::new("rebase-auto-stash-enabled"),
        ],
        _ => vec![], // No contracts for unimplemented concerns
    }
}

/// Get all contracts for a list of concerns
pub fn get_all_contracts(concerns: &[ConcernSymbol]) -> Vec<ContractSymbol> {
    concerns
        .iter()
        .flat_map(get_contracts)
        .collect()
}

/// Check if a contract exists for a concern
pub fn has_contracts(concern: &ConcernSymbol) -> bool {
    !get_contracts(concern).is_empty()
}

/// Get contract names for a concern
pub fn get_contract_names(concern: &ConcernSymbol) -> Vec<String> {
    get_contracts(concern)
        .iter()
        .map(|c| c.name().to_string())
        .collect()
}

/// Get concerns that have contracts
pub fn get_concerns_with_contracts(concerns: &[ConcernSymbol]) -> Vec<ConcernSymbol> {
    concerns
        .iter()
        .filter(|c| has_contracts(c))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_contracts_for_index() {
        let contracts = get_contracts(&ConcernSymbol::Index);
        assert_eq!(contracts.len(), 3);
        assert!(contracts.iter().any(|c| c.name() == "must-exist"));
        assert!(contracts.iter().any(|c| c.name() == "no-unstaged-changes"));
        assert!(contracts.iter().any(|c| c.name() == "no-untracked-files"));
    }

    #[test]
    fn test_get_contracts_for_tree_executable() {
        let contracts = get_contracts(&ConcernSymbol::TreeExecutable);
        assert_eq!(contracts.len(), 2);
        assert!(contracts.iter().any(|c| c.name() == "no-new-executables"));
        assert!(contracts.iter().any(|c| c.name() == "executable-safety-check"));
    }

    #[test]
    fn test_get_all_contracts() {
        let concerns = vec![ConcernSymbol::Index, ConcernSymbol::TreeExecutable];
        let contracts = get_all_contracts(&concerns);
        assert_eq!(contracts.len(), 5); // 3 + 2
    }

    #[test]
    fn test_has_contracts() {
        assert!(has_contracts(&ConcernSymbol::Index));
        assert!(has_contracts(&ConcernSymbol::TreeExecutable));
        assert!(!has_contracts(&ConcernSymbol::Blob)); // Unimplemented
    }

    #[test]
    fn test_get_contract_names() {
        let names = get_contract_names(&ConcernSymbol::Index);
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"must-exist".to_string()));
        assert!(names.contains(&"no-unstaged-changes".to_string()));
        assert!(names.contains(&"no-untracked-files".to_string()));
    }

    #[test]
    fn test_get_concerns_with_contracts() {
        let concerns = vec![ConcernSymbol::Index, ConcernSymbol::Blob, ConcernSymbol::TreeExecutable];
        let with_contracts = get_concerns_with_contracts(&concerns);
        assert_eq!(with_contracts.len(), 2); // Index and TreeExecutable
        assert!(with_contracts.contains(&ConcernSymbol::Index));
        assert!(with_contracts.contains(&ConcernSymbol::TreeExecutable));
        assert!(!with_contracts.contains(&ConcernSymbol::Blob));
    }
}
