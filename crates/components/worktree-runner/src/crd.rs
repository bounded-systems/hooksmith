use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata for the WorktreeChangeRequest CRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMetadata {
    pub name: String,
    pub namespace: String,
    pub created: DateTime<Utc>,
    pub last_modified: Option<DateTime<Utc>>,
    pub version: String,
}

impl Default for WorktreeMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            namespace: "default".to_string(),
            created: Utc::now(),
            last_modified: None,
            version: "v1".to_string(),
        }
    }
}

/// Domain state for local Git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDomain {
    pub exists: bool,
    pub current: bool,
    pub last_commit: Option<String>,
    pub ahead: i32,
    pub behind: i32,
}

impl Default for LocalDomain {
    fn default() -> Self {
        Self {
            exists: false,
            current: false,
            last_commit: None,
            ahead: 0,
            behind: 0,
        }
    }
}

/// Domain state for remote Git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDomain {
    pub exists: bool,
    pub last_commit: Option<String>,
    pub upstream: Option<String>,
}

impl Default for RemoteDomain {
    fn default() -> Self {
        Self {
            exists: false,
            last_commit: None,
            upstream: None,
        }
    }
}

/// Domain state for worktree directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeDomain {
    pub exists: bool,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub conflicted: bool,
    pub rebase_in_progress: bool,
}

impl Default for WorktreeDomain {
    fn default() -> Self {
        Self {
            exists: false,
            path: None,
            dirty: false,
            conflicted: false,
            rebase_in_progress: false,
        }
    }
}

/// Domain state for GitHub Pull Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestDomain {
    pub exists: bool,
    pub number: Option<i32>,
    pub url: Option<String>,
    pub state: Option<PrState>,
    pub title: Option<String>,
    pub labels: Vec<String>,
}

impl Default for PullRequestDomain {
    fn default() -> Self {
        Self {
            exists: false,
            number: None,
            url: None,
            state: None,
            title: None,
            labels: Vec::new(),
        }
    }
}

/// GitHub PR state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrState {
    Draft,
    Open,
    Closed,
    Merged,
}

/// All four domains combined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domains {
    pub local: LocalDomain,
    pub remote: RemoteDomain,
    pub worktree: WorktreeDomain,
    pub pr: PullRequestDomain,
}

impl Default for Domains {
    fn default() -> Self {
        Self {
            local: LocalDomain::default(),
            remote: RemoteDomain::default(),
            worktree: WorktreeDomain::default(),
            pr: PullRequestDomain::default(),
        }
    }
}

/// Lifecycle states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Available actions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Condition for status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: String,
    pub status: ConditionStatus,
    pub last_transition_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Condition status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

/// History entry for tracking state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub state: WorktreeState,
    pub action: Option<WorktreeAction>,
    pub success: bool,
    pub message: Option<String>,
}

/// Status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeStatus {
    pub phase: Phase,
    pub message: Option<String>,
    pub last_transition_time: DateTime<Utc>,
    pub conditions: Vec<Condition>,
    pub history: Vec<HistoryEntry>,
}

/// Phase of the action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// Specification for the WorktreeChangeRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSpec {
    pub branch: String,
    pub domains: Domains,
    pub state: WorktreeState,
    pub action: Option<WorktreeAction>,
    pub priority: i32,
    pub retry_count: i32,
    pub max_retries: i32,
}

impl Default for WorktreeSpec {
    fn default() -> Self {
        Self {
            branch: String::new(),
            domains: Domains::default(),
            state: WorktreeState::Created,
            action: None,
            priority: 5,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

/// The main WorktreeChangeRequest CRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeChangeRequest {
    pub metadata: WorktreeMetadata,
    pub spec: WorktreeSpec,
    pub status: WorktreeStatus,
}

impl Default for WorktreeChangeRequest {
    fn default() -> Self {
        Self {
            metadata: WorktreeMetadata::default(),
            spec: WorktreeSpec::default(),
            status: WorktreeStatus {
                phase: Phase::Pending,
                message: None,
                last_transition_time: Utc::now(),
                conditions: Vec::new(),
                history: Vec::new(),
            },
        }
    }
}

impl WorktreeChangeRequest {
    /// Create a new WorktreeChangeRequest for a branch
    pub fn new(branch_name: &str) -> Self {
        let mut crd = Self::default();
        crd.metadata.name = branch_name.to_string();
        crd.metadata.created = Utc::now();
        crd.spec.branch = branch_name.to_string();
        crd
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        self.metadata.last_modified = Some(Utc::now());
    }

    /// Add a history entry
<<<<<<< HEAD
    pub fn add_history_entry(
        &mut self,
        state: WorktreeState,
        action: Option<WorktreeAction>,
        success: bool,
        message: Option<String>,
    ) {
=======
    pub fn add_history_entry(&mut self, state: WorktreeState, action: Option<WorktreeAction>, success: bool, message: Option<String>) {
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        let entry = HistoryEntry {
            timestamp: Utc::now(),
            state,
            action,
            success,
            message,
        };
        self.status.history.push(entry);
    }

    /// Update the state and add history
<<<<<<< HEAD
    pub fn transition_to(
        &mut self,
        new_state: WorktreeState,
        action: Option<WorktreeAction>,
        success: bool,
        message: Option<String>,
    ) {
=======
    pub fn transition_to(&mut self, new_state: WorktreeState, action: Option<WorktreeAction>, success: bool, message: Option<String>) {
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        let old_state = self.spec.state.clone();
        self.spec.state = new_state;
        self.status.last_transition_time = Utc::now();
        self.add_history_entry(old_state, action, success, message);
        self.touch();
    }

    /// Check if all domains are in sync
    pub fn is_synchronized(&self) -> bool {
        let domains = &self.spec.domains;
<<<<<<< HEAD

        // Check if local and remote are in sync
        let local_remote_sync = domains.local.exists == domains.remote.exists;

=======
        
        // Check if local and remote are in sync
        let local_remote_sync = domains.local.exists == domains.remote.exists;
        
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        // Check if worktree exists when it should
        let worktree_sync = if domains.local.exists {
            domains.worktree.exists
        } else {
            !domains.worktree.exists
        };
<<<<<<< HEAD

        // Check if PR exists when it should
        let pr_sync = if self.spec.state == WorktreeState::PrCreated
            || self.spec.state == WorktreeState::Merged
        {
=======
        
        // Check if PR exists when it should
        let pr_sync = if self.spec.state == WorktreeState::PrCreated || self.spec.state == WorktreeState::Merged {
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
            domains.pr.exists
        } else {
            !domains.pr.exists
        };
<<<<<<< HEAD

=======
        
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        local_remote_sync && worktree_sync && pr_sync
    }

    /// Determine the next action based on current state and domain status
    pub fn determine_next_action(&mut self) -> Option<WorktreeAction> {
        let domains = &self.spec.domains;
<<<<<<< HEAD

=======
        
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        match self.spec.state {
            WorktreeState::Created => {
                if !domains.local.exists {
                    Some(WorktreeAction::CreateBranch)
                } else if !domains.worktree.exists {
                    Some(WorktreeAction::CreateWorktree)
                } else {
                    Some(WorktreeAction::CreateWorktree)
                }
<<<<<<< HEAD
            }
=======
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
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
<<<<<<< HEAD
            }
            WorktreeState::Conflicted => Some(WorktreeAction::ResolveConflicts),
=======
            },
            WorktreeState::Conflicted => {
                Some(WorktreeAction::ResolveConflicts)
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
            WorktreeState::Resolving => {
                if domains.worktree.conflicted {
                    Some(WorktreeAction::ResolveConflicts)
                } else {
                    Some(WorktreeAction::RebaseMain)
                }
<<<<<<< HEAD
            }
=======
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
            WorktreeState::Ready => {
                if !domains.remote.exists {
                    Some(WorktreeAction::PushBranch)
                } else if !domains.pr.exists {
                    Some(WorktreeAction::CreatePr)
                } else {
                    None // Ready state achieved
                }
<<<<<<< HEAD
            }
=======
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
            WorktreeState::PrCreated => {
                if let Some(PrState::Merged) = domains.pr.state {
                    Some(WorktreeAction::MergePr)
                } else {
                    None // Wait for PR to be merged
                }
<<<<<<< HEAD
            }
            WorktreeState::Merged => Some(WorktreeAction::CleanupWorktree),
=======
            },
            WorktreeState::Merged => {
                Some(WorktreeAction::CleanupWorktree)
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
            WorktreeState::Cleanup => {
                if domains.worktree.exists {
                    Some(WorktreeAction::CleanupWorktree)
                } else if domains.local.exists {
                    Some(WorktreeAction::RemoveBranch)
                } else {
                    Some(WorktreeAction::RemoveBranch)
                }
<<<<<<< HEAD
            }
            WorktreeState::Removed => {
                None // Terminal state
            }
=======
            },
            WorktreeState::Removed => {
                None // Terminal state
            },
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        }
    }

    /// Validate the CRD
    pub fn validate(&self) -> Result<()> {
        if self.spec.branch.is_empty() {
            anyhow::bail!("Branch name cannot be empty");
        }
<<<<<<< HEAD

        if self.spec.priority < 1 || self.spec.priority > 10 {
            anyhow::bail!("Priority must be between 1 and 10");
        }

        if self.spec.retry_count > self.spec.max_retries {
            anyhow::bail!("Retry count cannot exceed max retries");
        }

=======
        
        if self.spec.priority < 1 || self.spec.priority > 10 {
            anyhow::bail!("Priority must be between 1 and 10");
        }
        
        if self.spec.retry_count > self.spec.max_retries {
            anyhow::bail!("Retry count cannot exceed max retries");
        }
        
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_creation() {
        let crd = WorktreeChangeRequest::new("feature/test");
        assert_eq!(crd.spec.branch, "feature/test");
        assert_eq!(crd.spec.state, WorktreeState::Created);
    }

    #[test]
    fn test_state_transition() {
        let mut crd = WorktreeChangeRequest::new("feature/test");
        crd.transition_to(
            WorktreeState::Developing,
            Some(WorktreeAction::CreateWorktree),
            true,
<<<<<<< HEAD
            Some("Worktree created successfully".to_string()),
=======
            Some("Worktree created successfully".to_string())
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
        );
        assert_eq!(crd.spec.state, WorktreeState::Developing);
        assert_eq!(crd.status.history.len(), 1);
    }

    #[test]
    fn test_synchronization_check() {
        let mut crd = WorktreeChangeRequest::new("feature/test");
        crd.spec.domains.local.exists = true;
        crd.spec.domains.remote.exists = true;
        crd.spec.domains.worktree.exists = true;
        crd.spec.domains.pr.exists = false;
<<<<<<< HEAD

        assert!(crd.is_synchronized());
    }
}
=======
        
        assert!(crd.is_synchronized());
    }
} 
>>>>>>> 32d9c520 (feat: Enhanced worktree CRD system with Kubernetes integration)
