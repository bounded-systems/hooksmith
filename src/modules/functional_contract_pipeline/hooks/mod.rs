use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, HookEvent};

/// Hook logic for pre-commit
pub fn pre_commit() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Index,
        ConcernSymbol::AttrLineEndingNormalization,
        ConcernSymbol::AttrDiffStrategy,
    ]
}

/// Hook logic for pre-push
pub fn pre_push() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Ref,
        ConcernSymbol::Branch,
        ConcernSymbol::Remote,
        ConcernSymbol::TreeExecutable,
        ConcernSymbol::AttrLineEndingNormalization,
    ]
}

/// Hook logic for pre-receive
pub fn pre_receive() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Ref,
        ConcernSymbol::Branch,
        ConcernSymbol::Remote,
    ]
}

/// Hook logic for post-receive
pub fn post_receive() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Ref,
        ConcernSymbol::Branch,
        ConcernSymbol::Remote,
    ]
}

/// Hook logic for update
pub fn update() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::Ref, ConcernSymbol::Branch]
}

/// Hook logic for post-update
pub fn post_update() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::Ref, ConcernSymbol::Branch]
}

/// Hook logic for pre-auto-gc
pub fn pre_auto_gc() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::ConfigGc]
}

/// Hook logic for post-merge
pub fn post_merge() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Commit,
        ConcernSymbol::Tree,
        ConcernSymbol::Ref,
    ]
}

/// Hook logic for pre-rebase
pub fn pre_rebase() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Ref,
        ConcernSymbol::Branch,
        ConcernSymbol::ConfigRebase,
    ]
}

/// Hook logic for post-checkout
pub fn post_checkout() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Head,
        ConcernSymbol::Ref,
        ConcernSymbol::Index,
    ]
}

/// Hook logic for post-commit
pub fn post_commit() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Commit,
        ConcernSymbol::Ref,
        ConcernSymbol::Head,
    ]
}

/// Hook logic for pre-apply-patch
pub fn pre_apply_patch() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::Index, ConcernSymbol::AttrDiffStrategy]
}

/// Hook logic for post-apply-patch
pub fn post_apply_patch() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::Index, ConcernSymbol::Commit]
}

/// Hook logic for post-rebase
pub fn post_rebase() -> Vec<ConcernSymbol> {
    vec![
        ConcernSymbol::Ref,
        ConcernSymbol::Branch,
        ConcernSymbol::Commit,
    ]
}

/// Hook logic for pre-commit-msg
pub fn pre_commit_msg() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::ConfigCommit, ConcernSymbol::ConfigUser]
}

/// Hook logic for commit-msg
pub fn commit_msg() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::ConfigCommit, ConcernSymbol::ConfigUser]
}

/// Hook logic for post-commit-msg
pub fn post_commit_msg() -> Vec<ConcernSymbol> {
    vec![ConcernSymbol::Commit, ConcernSymbol::Ref]
}

/// Get concerns for a hook event
pub fn get_concerns(hook: &HookEvent) -> Vec<ConcernSymbol> {
    match hook {
        HookEvent::PreCommit => pre_commit(),
        HookEvent::PrePush => pre_push(),
        HookEvent::PreReceive => pre_receive(),
        HookEvent::PostReceive => post_receive(),
        HookEvent::Update => update(),
        HookEvent::PostUpdate => post_update(),
        HookEvent::PreAutoGc => pre_auto_gc(),
        HookEvent::PostMerge => post_merge(),
        HookEvent::PreRebase => pre_rebase(),
        HookEvent::PostCheckout => post_checkout(),
        HookEvent::PostCommit => post_commit(),
        HookEvent::PreApplyPatch => pre_apply_patch(),
        HookEvent::PostApplyPatch => post_apply_patch(),
        HookEvent::PostRebase => post_rebase(),
        HookEvent::PreCommitMsg => pre_commit_msg(),
        HookEvent::CommitMsg => commit_msg(),
        HookEvent::PostCommitMsg => post_commit_msg(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_commit_concerns() {
        let concerns = pre_commit();
        assert_eq!(concerns.len(), 3);
        assert!(concerns.contains(&ConcernSymbol::Index));
        assert!(concerns.contains(&ConcernSymbol::AttrLineEndingNormalization));
        assert!(concerns.contains(&ConcernSymbol::AttrDiffStrategy));
    }

    #[test]
    fn test_pre_push_concerns() {
        let concerns = pre_push();
        assert_eq!(concerns.len(), 5);
        assert!(concerns.contains(&ConcernSymbol::Ref));
        assert!(concerns.contains(&ConcernSymbol::Branch));
        assert!(concerns.contains(&ConcernSymbol::Remote));
        assert!(concerns.contains(&ConcernSymbol::TreeExecutable));
        assert!(concerns.contains(&ConcernSymbol::AttrLineEndingNormalization));
    }

    #[test]
    fn test_get_concerns_for_hook() {
        let concerns = get_concerns(&HookEvent::PreCommit);
        assert_eq!(concerns.len(), 3);
        assert!(concerns.contains(&ConcernSymbol::Index));
    }
}
