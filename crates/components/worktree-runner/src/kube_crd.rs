use kube::{CustomResource, CustomResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// WorktreeChangeRequest CRD using kube-derive
#[derive(CustomResource, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[kube(
    group = "hooksmith.dev",
    version = "v1",
    kind = "WorktreeChangeRequest",
    namespaced
)]
pub struct WorktreeChangeRequestSpec {
    /// Git branch name
    pub branch: String,

    /// State across the four domains
    pub domains: WorktreeDomains,

    /// Current lifecycle state
    pub state: WorktreeState,

    /// Next action to perform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<WorktreeAction>,

    /// Action priority (1=highest, 10=lowest)
    #[serde(default = "default_priority")]
    pub priority: i32,

    /// Number of times this action has been retried
    #[serde(default)]
    pub retry_count: i32,

    /// Maximum number of retries for this action
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
}

/// Status for the WorktreeChangeRequest
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct WorktreeChangeRequestStatus {
    /// Current phase of the action
    pub phase: Phase,

    /// Human-readable status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Timestamp of last state transition
    pub last_transition_time: String,

    /// Conditions for status tracking
    #[serde(default)]
    pub conditions: Vec<Condition>,

    /// History of state transitions
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

fn default_priority() -> i32 {
    5
}

fn default_max_retries() -> i32 {
    3
}

/// All four domains combined
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct WorktreeDomains {
    /// Local Git branch state
    pub local: LocalDomain,

    /// Remote Git branch state
    pub remote: RemoteDomain,

    /// Worktree directory state
    pub worktree: WorktreeDomain,

    /// GitHub Pull Request state
    pub pr: PullRequestDomain,
}

/// Domain state for local Git branch
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct LocalDomain {
    /// Whether local branch exists in .git/refs/heads/
    pub exists: bool,

    /// Whether this is the currently checked out branch
    #[serde(default)]
    pub current: bool,

    /// SHA of the last commit on this branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<String>,

    /// Number of commits ahead of main
    #[serde(default)]
    pub ahead: i32,

    /// Number of commits behind main
    #[serde(default)]
    pub behind: i32,

    /// Whether branch has uncommitted changes
    #[serde(default)]
    pub dirty: bool,

    /// Whether branch is stale (no activity for >30 days)
    #[serde(default)]
    pub stale: bool,

    /// Last activity timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<String>,

    /// Branch protection status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection_status: Option<BranchProtection>,
}

/// Domain state for remote Git branch
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct RemoteDomain {
    /// Whether remote branch exists in origin/
    pub exists: bool,

    /// SHA of the last commit on remote branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<String>,

    /// Upstream branch name (e.g., 'origin/feature/xyz')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
}

/// Domain state for worktree directory
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct WorktreeDomain {
    /// Whether worktree directory exists
    pub exists: bool,

    /// Path to the worktree directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Whether worktree has uncommitted changes
    #[serde(default)]
    pub dirty: bool,

    /// Whether worktree has merge/rebase conflicts
    #[serde(default)]
    pub conflicted: bool,

    /// Whether a rebase is in progress
    #[serde(default)]
    pub rebase_in_progress: bool,

    /// Whether worktree is orphaned (no corresponding branch)
    #[serde(default)]
    pub orphaned: bool,

    /// Size of worktree directory in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,

    /// Last access time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_access: Option<String>,

    /// Whether worktree has stashed changes
    #[serde(default)]
    pub has_stash: bool,

    /// Number of stashed changes
    #[serde(default)]
    pub stash_count: i32,
}

/// Domain state for GitHub Pull Request
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct PullRequestDomain {
    /// Whether GitHub PR exists
    pub exists: bool,

    /// GitHub PR number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    /// GitHub PR URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// PR state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<PrState>,

    /// PR title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// PR labels
    #[serde(default)]
    pub labels: Vec<String>,
}

/// GitHub PR state
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum PrState {
    Draft,
    Open,
    Closed,
    Merged,
    Approved,
    ChangesRequested,
    Blocked,
    AutoMergeEnabled,
}

/// Branch protection status
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct BranchProtection {
    /// Whether branch is protected
    pub protected: bool,

    /// Required status checks
    #[serde(default)]
    pub required_status_checks: Vec<String>,

    /// Required reviewers
    #[serde(default)]
    pub required_reviewers: Vec<String>,

    /// Whether force push is allowed
    #[serde(default)]
    pub allow_force_push: bool,

    /// Whether deletion is allowed
    #[serde(default)]
    pub allow_deletion: bool,
}

/// Worktree lifecycle states
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum WorktreeState {
    Created,
    Developing,
    Conflicted,
    Resolving,
    Ready,
    PrCreated,
    Merged,
    Cleanup,
    Removed,
    // New states for enhanced lifecycle
    DirtyMainRecovery,
    FeatureExtraction,
    StaleBranch,
    OrphanedWorktree,
    NeedsRebase,
    NeedsSquash,
    ReadyForReview,
    Approved,
    Blocked,
}

/// Available actions for worktree lifecycle management
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum WorktreeAction {
    CreateWorktree,
    CreateBranch,
    PushBranch,
    CreatePr,
    MergePr,
    ResolveConflicts,
    RebaseMain,
    CleanupWorktree,
    RemoveBranch,
    ResetMain,
    // New actions for enhanced workflow
    ExtractFeature,
    RecoverDirtyMain,
    SquashCommits,
    UpdateBranch,
    SyncRemote,
    MarkStale,
    ForcePush,
    AbortRebase,
    StashChanges,
    ApplyStash,
    CreateBackup,
    RestoreBackup,
    ValidateBranch,
    RunTests,
    DeployPreview,
}

/// Phase of the action
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum Phase {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// Condition for status tracking
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Condition {
    /// Condition type
    pub condition_type: String,

    /// Condition status
    pub status: ConditionStatus,

    /// Timestamp of last transition
    pub last_transition_time: String,

    /// Reason for condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable condition message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Condition status
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

/// History entry for tracking state transitions
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct HistoryEntry {
    /// Timestamp of the transition
    pub timestamp: String,

    /// State at this point
    pub state: WorktreeState,

    /// Action performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<WorktreeAction>,

    /// Whether the action was successful
    pub success: bool,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl WorktreeChangeRequest {
    /// Create a new WorktreeChangeRequest for a branch
    pub fn create(branch_name: &str) -> Self {
        let _now = chrono::Utc::now().to_rfc3339();

        Self {
            metadata: kube::api::ObjectMeta {
                name: Some(branch_name.to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: WorktreeChangeRequestSpec {
                branch: branch_name.to_string(),
                domains: WorktreeDomains {
                    local: LocalDomain {
                        exists: false,
                        current: false,
                        last_commit: None,
                        ahead: 0,
                        behind: 0,
                        dirty: false,
                        stale: false,
                        last_activity: None,
                        protection_status: None,
                    },
                    remote: RemoteDomain {
                        exists: false,
                        last_commit: None,
                        upstream: None,
                    },
                    worktree: WorktreeDomain {
                        exists: false,
                        path: None,
                        dirty: false,
                        conflicted: false,
                        rebase_in_progress: false,
                        orphaned: false,
                        size_bytes: None,
                        last_access: None,
                        has_stash: false,
                        stash_count: 0,
                    },
                    pr: PullRequestDomain {
                        exists: false,
                        number: None,
                        url: None,
                        state: None,
                        title: None,
                        labels: Vec::new(),
                    },
                },
                state: WorktreeState::Created,
                action: None,
                priority: 5,
                retry_count: 0,
                max_retries: 3,
            },
        }
    }

    /// Check if all domains are in sync
    pub fn is_synchronized(&self) -> bool {
        let domains = &self.spec.domains;

        // Check if local and remote are in sync
        let local_remote_sync = domains.local.exists == domains.remote.exists;

        // Check if worktree exists when it should
        let worktree_sync = if domains.local.exists {
            domains.worktree.exists
        } else {
            !domains.worktree.exists
        };

        // Check if PR exists when it should
        let pr_sync = if self.spec.state == WorktreeState::PrCreated
            || self.spec.state == WorktreeState::Merged
        {
            domains.pr.exists
        } else {
            !domains.pr.exists
        };

        local_remote_sync && worktree_sync && pr_sync
    }

    /// Determine the next action based on current state and domain status
    pub fn determine_next_action(&mut self) -> Option<WorktreeAction> {
        let domains = &self.spec.domains;

        match self.spec.state {
            WorktreeState::Created => {
                if !domains.local.exists {
                    Some(WorktreeAction::CreateBranch)
                } else if !domains.worktree.exists {
                    Some(WorktreeAction::CreateWorktree)
                } else {
                    Some(WorktreeAction::CreateWorktree)
                }
            }
            WorktreeState::Developing => {
                if domains.worktree.dirty {
                    None // Let user continue developing
                } else if !domains.remote.exists {
                    Some(WorktreeAction::PushBranch)
                } else if domains.local.behind > 0 {
                    Some(WorktreeAction::RebaseMain)
                } else {
                    Some(WorktreeAction::CreatePr)
                }
            }
            WorktreeState::Conflicted => Some(WorktreeAction::ResolveConflicts),
            WorktreeState::Resolving => {
                if domains.worktree.conflicted {
                    Some(WorktreeAction::ResolveConflicts)
                } else {
                    Some(WorktreeAction::RebaseMain)
                }
            }
            WorktreeState::Ready => {
                if !domains.remote.exists {
                    Some(WorktreeAction::PushBranch)
                } else if !domains.pr.exists {
                    Some(WorktreeAction::CreatePr)
                } else {
                    None // Ready state achieved
                }
            }
            WorktreeState::ReadyForReview => {
                if !domains.remote.exists {
                    Some(WorktreeAction::PushBranch)
                } else if !domains.pr.exists {
                    Some(WorktreeAction::CreatePr)
                } else {
                    None // Ready for review
                }
            }
            WorktreeState::PrCreated => {
                if let Some(PrState::Merged) = domains.pr.state {
                    Some(WorktreeAction::MergePr)
                } else {
                    None // Wait for PR to be merged
                }
            }
            WorktreeState::Merged => Some(WorktreeAction::CleanupWorktree),
            WorktreeState::Cleanup => {
                if domains.worktree.exists {
                    Some(WorktreeAction::CleanupWorktree)
                } else if domains.local.exists {
                    Some(WorktreeAction::RemoveBranch)
                } else {
                    Some(WorktreeAction::RemoveBranch)
                }
            }
            WorktreeState::Removed => {
                None // Terminal state
            }
            // New enhanced states
            WorktreeState::DirtyMainRecovery => Some(WorktreeAction::RecoverDirtyMain),
            WorktreeState::FeatureExtraction => Some(WorktreeAction::ExtractFeature),
            WorktreeState::StaleBranch => Some(WorktreeAction::MarkStale),
            WorktreeState::OrphanedWorktree => Some(WorktreeAction::CleanupWorktree),
            WorktreeState::NeedsRebase => Some(WorktreeAction::UpdateBranch),
            WorktreeState::NeedsSquash => Some(WorktreeAction::SquashCommits),
            WorktreeState::Approved => {
                None // Wait for merge
            }
            WorktreeState::Blocked => {
                None // Wait for issues to be resolved
            }
        }
    }

    /// Get a summary of the current state
    pub fn get_summary(&self) -> String {
        let domains = &self.spec.domains;
        format!(
            "Branch: {} | State: {:?} | Local: {} | Remote: {} | Worktree: {} | PR: {}",
            self.spec.branch,
            self.spec.state,
            domains.local.exists,
            domains.remote.exists,
            domains.worktree.exists,
            domains.pr.exists
        )
    }
}

/// Generate CRD YAML
pub fn generate_crd_yaml() -> String {
    serde_yaml::to_string(&WorktreeChangeRequest::crd()).expect("Failed to serialize CRD")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_creation() {
        let crd = WorktreeChangeRequest::create("feature/test");
        assert_eq!(crd.spec.branch, "feature/test");
        assert_eq!(crd.spec.state, WorktreeState::Created);
    }

    #[test]
    fn test_synchronization_check() {
        let mut crd = WorktreeChangeRequest::create("feature/test");
        crd.spec.domains.local.exists = true;
        crd.spec.domains.remote.exists = true;
        crd.spec.domains.worktree.exists = true;
        crd.spec.domains.pr.exists = false;

        assert!(crd.is_synchronized());
    }

    #[test]
    fn test_crd_yaml_generation() {
        let yaml = generate_crd_yaml();
        assert!(yaml.contains("WorktreeChangeRequest"));
        assert!(yaml.contains("hooksmith.dev"));
    }
}
