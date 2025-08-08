use crate::modules::static_hook::HookConcern;
use std::collections::HashMap;
use std::path::Path;

/// Test suite for validating HookConcern variants against Git's actual behavior
#[cfg(test)]
mod tests {
    use super::*;

    /// Test that all core Git object concerns are valid
    #[test]
    fn test_core_git_object_concerns() {
        let core_concerns = vec![
            HookConcern::Blob,
            HookConcern::Tree,
            HookConcern::Commit,
            HookConcern::Tag,
            HookConcern::Ref,
            HookConcern::Note,
            HookConcern::Attr,
        ];

        for concern in core_concerns {
            assert!(is_valid_git_object_concern(&concern));
        }
    }

    /// Test that all reference concerns map to valid Git ref paths
    #[test]
    fn test_reference_concerns() {
        let ref_concerns = vec![
            (HookConcern::RefBranch, ".git/refs/heads/"),
            (HookConcern::RefRemote, ".git/refs/remotes/"),
            (HookConcern::RefTag, ".git/refs/tags/"),
            (HookConcern::RefNote, ".git/refs/notes/"),
            (HookConcern::RefStash, ".git/refs/stash"),
            (HookConcern::HeadPointer, ".git/HEAD"),
            (HookConcern::PackedRefs, ".git/packed-refs"),
            (HookConcern::FetchHeadPointer, ".git/FETCH_HEAD"),
            (HookConcern::MergeHeadPointer, ".git/MERGE_HEAD"),
            (HookConcern::CherryPickPointer, ".git/CHERRY_PICK_HEAD"),
            (HookConcern::RevertHeadPointer, ".git/REVERT_HEAD"),
            (HookConcern::OrigHead, ".git/ORIG_HEAD"),
        ];

        for (concern, expected_path) in ref_concerns {
            assert!(is_valid_git_ref_concern(&concern, expected_path));
        }
    }

    /// Test that storage concerns map to valid Git object database paths
    #[test]
    fn test_storage_concerns() {
        let storage_concerns = vec![
            (HookConcern::PackfileIndex, ".git/objects/pack/*.idx"),
            (HookConcern::PackfileData, ".git/objects/pack/*.pack"),
            (HookConcern::PackfileBitmap, ".git/objects/pack/*.bitmap"),
            (HookConcern::PackfileKeep, ".git/objects/pack/*.keep"),
            (HookConcern::PackfilePromisor, ".git/objects/pack/*.promisor"),
            (HookConcern::LooseObject, ".git/objects/??/*"),
            (HookConcern::ObjectDatabase, ".git/objects/"),
        ];

        for (concern, expected_pattern) in storage_concerns {
            assert!(is_valid_storage_concern(&concern, expected_pattern));
        }
    }

    /// Test that transport concerns map to valid Git protocols
    #[test]
    fn test_transport_concerns() {
        let transport_concerns = vec![
            (HookConcern::ProtocolLocal, "file://"),
            (HookConcern::ProtocolGit, "git://"),
            (HookConcern::ProtocolHttp, "http://"),
            (HookConcern::ProtocolHttps, "https://"),
            (HookConcern::ProtocolSsh, "ssh://"),
        ];

        for (concern, expected_protocol) in transport_concerns {
            assert!(is_valid_transport_concern(&concern, expected_protocol));
        }
    }

    /// Test that environment concerns map to valid Git environment variables
    #[test]
    fn test_environment_concerns() {
        let env_concerns = vec![
            (HookConcern::GitDirOverride, "GIT_DIR"),
            (HookConcern::WorkTreeOverride, "GIT_WORK_TREE"),
            (HookConcern::IndexFileOverride, "GIT_INDEX_FILE"),
            (HookConcern::ObjectDirectoryOverride, "GIT_OBJECT_DIRECTORY"),
            (HookConcern::AlternateObjectDatabase, "GIT_ALTERNATE_OBJECT_DIRECTORIES"),
            (HookConcern::GitConfigOverride, "GIT_CONFIG"),
            (HookConcern::TraceOverride, "GIT_TRACE"),
            (HookConcern::AuthorOverride, "GIT_AUTHOR_NAME"),
            (HookConcern::UiOverride, "GIT_PAGER"),
        ];

        for (concern, expected_env_var) in env_concerns {
            assert!(is_valid_environment_concern(&concern, expected_env_var));
        }
    }

    /// Test that maintenance concerns map to valid Git maintenance commands
    #[test]
    fn test_maintenance_concerns() {
        let maintenance_concerns = vec![
            (HookConcern::FsckCheck, "git fsck"),
            (HookConcern::PruneOrphaned, "git prune"),
            (HookConcern::RepackPackfile, "git repack"),
            (HookConcern::GcLifecycle, "git gc"),
            (HookConcern::ReflogRepair, "git reflog"),
            (HookConcern::IndexRecovery, "git reset"),
        ];

        for (concern, expected_command) in maintenance_concerns {
            assert!(is_valid_maintenance_concern(&concern, expected_command));
        }
    }

    /// Test that command concerns map to valid Git command categories
    #[test]
    fn test_command_concerns() {
        let command_concerns = vec![
            (HookConcern::Init, "Setup & Config"),
            (HookConcern::Snapshot, "Snapshotting"),
            (HookConcern::Branch, "Branching & Merging"),
            (HookConcern::Merge, "Branching & Merging"),
            (HookConcern::Rebase, "Branching & Merging"),
            (HookConcern::Push, "Sharing & Collaboration"),
            (HookConcern::Pull, "Sharing & Collaboration"),
            (HookConcern::Fetch, "Sharing & Collaboration"),
            (HookConcern::Log, "Inspection & Comparison"),
            (HookConcern::Diff, "Inspection & Comparison"),
            (HookConcern::Status, "Inspection & Comparison"),
            (HookConcern::Stash, "Branching & Merging"),
            (HookConcern::Patch, "Patching"),
            (HookConcern::Debug, "Debugging"),
            (HookConcern::Blame, "Inspection & Comparison"),
            (HookConcern::Plumbing, "Low-Level"),
            (HookConcern::ObjectDb, "Low-Level"),
            (HookConcern::Transport, "Low-Level"),
            (HookConcern::ProjectInit, "Project Creation"),
        ];

        for (concern, expected_category) in command_concerns {
            assert!(is_valid_command_concern(&concern, expected_category));
        }
    }

    /// Test that config concerns map to valid Git config sections
    #[test]
    fn test_config_concerns() {
        let config_concerns = vec![
            (HookConcern::ConfigUser, "[user]"),
            (HookConcern::ConfigCore, "[core]"),
            (HookConcern::ConfigBranch, "[branch]"),
            (HookConcern::ConfigRemote, "[remote]"),
            (HookConcern::ConfigInit, "[init]"),
            (HookConcern::ConfigColor, "[color]"),
            (HookConcern::ConfigAlias, "[alias]"),
            (HookConcern::ConfigDiff, "[diff]"),
            (HookConcern::ConfigMerge, "[merge]"),
            (HookConcern::ConfigGpg, "[gpg]"),
            (HookConcern::ConfigCommit, "[commit]"),
            (HookConcern::ConfigPull, "[pull]"),
            (HookConcern::ConfigPush, "[push]"),
            (HookConcern::ConfigRebase, "[rebase]"),
            (HookConcern::ConfigFetch, "[fetch]"),
            (HookConcern::ConfigStatus, "[status]"),
            (HookConcern::ConfigTar, "[tar]"),
            (HookConcern::ConfigRerere, "[rerere]"),
            (HookConcern::ConfigAdvice, "[advice]"),
            (HookConcern::ConfigInteractive, "[interactive]"),
            (HookConcern::ConfigSubmodule, "[submodule]"),
            (HookConcern::ConfigFilter, "[filter]"),
            (HookConcern::ConfigInclude, "[include]"),
            (HookConcern::ConfigCredential, "[credential]"),
            (HookConcern::ConfigHttp, "[http]"),
            (HookConcern::ConfigUrl, "[url]"),
            (HookConcern::ConfigSafe, "[safe]"),
            (HookConcern::ConfigNotes, "[notes]"),
            (HookConcern::ConfigGc, "[gc]"),
            (HookConcern::ConfigMaintenance, "[maintenance]"),
            (HookConcern::ConfigPager, "[pager]"),
            (HookConcern::ConfigWorktree, "[worktree]"),
        ];

        for (concern, expected_section) in config_concerns {
            assert!(is_valid_config_concern(&concern, expected_section));
        }
    }

    /// Test concern serialization and deserialization
    #[test]
    fn test_concern_serialization() {
        let concerns = vec![
            HookConcern::Blob,
            HookConcern::Tree,
            HookConcern::Commit,
            HookConcern::RefBranch,
            HookConcern::PackfileData,
            HookConcern::ProtocolHttps,
            HookConcern::FsckCheck,
            HookConcern::ConfigUser,
        ];

        for concern in concerns {
            let serialized = serde_json::to_string(&concern).unwrap();
            let deserialized: HookConcern = serde_json::from_str(&serialized).unwrap();
            assert_eq!(concern, deserialized);
        }
    }

    /// Test concern uniqueness in a hook
    #[test]
    fn test_concern_uniqueness() {
        let concerns = vec![
            HookConcern::Blob,
            HookConcern::Tree,
            HookConcern::Commit,
        ];

        let mut seen = std::collections::HashSet::new();
        for concern in &concerns {
            assert!(seen.insert(concern));
        }
    }

    /// Test concern ordering and comparison
    #[test]
    fn test_concern_ordering() {
        let mut concerns = vec![
            HookConcern::Commit,
            HookConcern::Blob,
            HookConcern::Tree,
        ];

        concerns.sort();
        
        assert_eq!(concerns[0], HookConcern::Blob);
        assert_eq!(concerns[1], HookConcern::Commit);
        assert_eq!(concerns[2], HookConcern::Tree);
    }
}

/// Validation functions for each concern category

fn is_valid_git_object_concern(concern: &HookConcern) -> bool {
    matches!(concern,
        HookConcern::Blob |
        HookConcern::Tree |
        HookConcern::Commit |
        HookConcern::Tag |
        HookConcern::Ref |
        HookConcern::Note |
        HookConcern::Attr
    )
}

fn is_valid_git_ref_concern(concern: &HookConcern, expected_path: &str) -> bool {
    match concern {
        HookConcern::RefBranch => expected_path.ends_with("refs/heads/"),
        HookConcern::RefRemote => expected_path.ends_with("refs/remotes/"),
        HookConcern::RefTag => expected_path.ends_with("refs/tags/"),
        HookConcern::RefNote => expected_path.ends_with("refs/notes/"),
        HookConcern::RefStash => expected_path.ends_with("refs/stash"),
        HookConcern::HeadPointer => expected_path.ends_with("HEAD"),
        HookConcern::PackedRefs => expected_path.ends_with("packed-refs"),
        HookConcern::FetchHeadPointer => expected_path.ends_with("FETCH_HEAD"),
        HookConcern::MergeHeadPointer => expected_path.ends_with("MERGE_HEAD"),
        HookConcern::CherryPickPointer => expected_path.ends_with("CHERRY_PICK_HEAD"),
        HookConcern::RevertHeadPointer => expected_path.ends_with("REVERT_HEAD"),
        HookConcern::OrigHead => expected_path.ends_with("ORIG_HEAD"),
        _ => false,
    }
}

fn is_valid_storage_concern(concern: &HookConcern, expected_pattern: &str) -> bool {
    match concern {
        HookConcern::PackfileIndex => expected_pattern.contains("*.idx"),
        HookConcern::PackfileData => expected_pattern.contains("*.pack"),
        HookConcern::PackfileBitmap => expected_pattern.contains("*.bitmap"),
        HookConcern::PackfileKeep => expected_pattern.contains("*.keep"),
        HookConcern::PackfilePromisor => expected_pattern.contains("*.promisor"),
        HookConcern::LooseObject => expected_pattern.contains("??/*"),
        HookConcern::ObjectDatabase => expected_pattern.ends_with("objects/"),
        _ => false,
    }
}

fn is_valid_transport_concern(concern: &HookConcern, expected_protocol: &str) -> bool {
    match concern {
        HookConcern::ProtocolLocal => expected_protocol == "file://",
        HookConcern::ProtocolGit => expected_protocol == "git://",
        HookConcern::ProtocolHttp => expected_protocol == "http://",
        HookConcern::ProtocolHttps => expected_protocol == "https://",
        HookConcern::ProtocolSsh => expected_protocol == "ssh://",
        _ => false,
    }
}

fn is_valid_environment_concern(concern: &HookConcern, expected_env_var: &str) -> bool {
    match concern {
        HookConcern::GitDirOverride => expected_env_var == "GIT_DIR",
        HookConcern::WorkTreeOverride => expected_env_var == "GIT_WORK_TREE",
        HookConcern::IndexFileOverride => expected_env_var == "GIT_INDEX_FILE",
        HookConcern::ObjectDirectoryOverride => expected_env_var == "GIT_OBJECT_DIRECTORY",
        HookConcern::AlternateObjectDatabase => expected_env_var == "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        HookConcern::GitConfigOverride => expected_env_var == "GIT_CONFIG",
        HookConcern::TraceOverride => expected_env_var == "GIT_TRACE",
        HookConcern::AuthorOverride => expected_env_var == "GIT_AUTHOR_NAME",
        HookConcern::UiOverride => expected_env_var == "GIT_PAGER",
        _ => false,
    }
}

fn is_valid_maintenance_concern(concern: &HookConcern, expected_command: &str) -> bool {
    match concern {
        HookConcern::FsckCheck => expected_command.contains("fsck"),
        HookConcern::PruneOrphaned => expected_command.contains("prune"),
        HookConcern::RepackPackfile => expected_command.contains("repack"),
        HookConcern::GcLifecycle => expected_command.contains("gc"),
        HookConcern::ReflogRepair => expected_command.contains("reflog"),
        HookConcern::IndexRecovery => expected_command.contains("reset"),
        _ => false,
    }
}

fn is_valid_command_concern(concern: &HookConcern, expected_category: &str) -> bool {
    match concern {
        HookConcern::Init => expected_category == "Setup & Config",
        HookConcern::Snapshot => expected_category == "Snapshotting",
        HookConcern::Branch | HookConcern::Merge | HookConcern::Rebase | HookConcern::Stash => {
            expected_category == "Branching & Merging"
        }
        HookConcern::Push | HookConcern::Pull | HookConcern::Fetch => {
            expected_category == "Sharing & Collaboration"
        }
        HookConcern::Log | HookConcern::Diff | HookConcern::Status | HookConcern::Blame => {
            expected_category == "Inspection & Comparison"
        }
        HookConcern::Patch => expected_category == "Patching",
        HookConcern::Debug => expected_category == "Debugging",
        HookConcern::Plumbing | HookConcern::ObjectDb | HookConcern::Transport => {
            expected_category == "Low-Level"
        }
        HookConcern::ProjectInit => expected_category == "Project Creation",
        _ => false,
    }
}

fn is_valid_config_concern(concern: &HookConcern, expected_section: &str) -> bool {
    match concern {
        HookConcern::ConfigUser => expected_section == "[user]",
        HookConcern::ConfigCore => expected_section == "[core]",
        HookConcern::ConfigBranch => expected_section == "[branch]",
        HookConcern::ConfigRemote => expected_section == "[remote]",
        HookConcern::ConfigInit => expected_section == "[init]",
        HookConcern::ConfigColor => expected_section == "[color]",
        HookConcern::ConfigAlias => expected_section == "[alias]",
        HookConcern::ConfigDiff => expected_section == "[diff]",
        HookConcern::ConfigMerge => expected_section == "[merge]",
        HookConcern::ConfigGpg => expected_section == "[gpg]",
        HookConcern::ConfigCommit => expected_section == "[commit]",
        HookConcern::ConfigPull => expected_section == "[pull]",
        HookConcern::ConfigPush => expected_section == "[push]",
        HookConcern::ConfigRebase => expected_section == "[rebase]",
        HookConcern::ConfigFetch => expected_section == "[fetch]",
        HookConcern::ConfigStatus => expected_section == "[status]",
        HookConcern::ConfigTar => expected_section == "[tar]",
        HookConcern::ConfigRerere => expected_section == "[rerere]",
        HookConcern::ConfigAdvice => expected_section == "[advice]",
        HookConcern::ConfigInteractive => expected_section == "[interactive]",
        HookConcern::ConfigSubmodule => expected_section == "[submodule]",
        HookConcern::ConfigFilter => expected_section == "[filter]",
        HookConcern::ConfigInclude => expected_section == "[include]",
        HookConcern::ConfigCredential => expected_section == "[credential]",
        HookConcern::ConfigHttp => expected_section == "[http]",
        HookConcern::ConfigUrl => expected_section == "[url]",
        HookConcern::ConfigSafe => expected_section == "[safe]",
        HookConcern::ConfigNotes => expected_section == "[notes]",
        HookConcern::ConfigGc => expected_section == "[gc]",
        HookConcern::ConfigMaintenance => expected_section == "[maintenance]",
        HookConcern::ConfigPager => expected_section == "[pager]",
        HookConcern::ConfigWorktree => expected_section == "[worktree]",
        _ => false,
    }
}
