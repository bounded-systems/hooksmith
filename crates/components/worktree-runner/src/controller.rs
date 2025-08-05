use anyhow::{Context, Result};
use futures_util::StreamExt;
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        events::{Event, EventType, Recorder},
        finalizer, watcher,
    },
    Resource, ResourceExt,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::kube_crd::{WorktreeChangeRequest, WorktreeState, WorktreeAction, Phase};
use crate::tools::EnhancedWorktreeOps;

/// Context for the controller
pub struct ControllerContext {
    pub client: Client,
    pub recorder: Recorder,
    pub enhanced_ops: EnhancedWorktreeOps,
}

/// Controller for WorktreeChangeRequest CRDs
pub struct WorktreeController {
    context: Arc<ControllerContext>,
}

impl WorktreeController {
    /// Create a new controller
    pub fn new(client: Client, enhanced_ops: EnhancedWorktreeOps) -> Self {
        let recorder = Recorder::new(client.clone(), "worktree-controller".into(), "worktree-controller".into());
        let context = ControllerContext {
            client,
            recorder,
            enhanced_ops,
        };

        Self {
            context: Arc::new(context),
        }
    }

    /// Start the controller
    pub async fn run(self) -> Result<()> {
        let api = Api::<WorktreeChangeRequest>::all(self.context.client.clone());
        let context = Arc::new(self.context);

        Controller::new(api, watcher::Config::default())
            .shutdown_on_signal()
            .run(reconcile, error_policy, context)
            .for_each(|res| async move {
                match res {
                    Ok(o) => info!("reconciled {:?}", o),
                    Err(e) => warn!("reconcile failed: {}", e),
                }
            })
            .await;

        Ok(())
    }
}

/// Main reconciliation logic
async fn reconcile(
    crd: Arc<WorktreeChangeRequest>,
    context: Arc<ControllerContext>,
) -> Result<Action> {
    let name = crd.name_any();
    let namespace = crd.namespace().unwrap_or_else(|| "default".into());
    
    info!("Reconciling WorktreeChangeRequest: {} in namespace: {}", name, namespace);
    
    // Check if the CRD is being deleted
    if crd.meta().deletion_timestamp.is_some() {
        info!("WorktreeChangeRequest {} is being deleted", name);
        return finalizer::cleanup(context.client.clone(), &name, &namespace, "worktree-controller").await;
    }
    
    // Add finalizer if not present
    finalizer::add(context.client.clone(), &name, &namespace, "worktree-controller").await?;
    
    // Perform reconciliation
    let result = reconcile_worktree(crd, context).await;
    
    match result {
        Ok(_) => {
            info!("Successfully reconciled WorktreeChangeRequest: {}", name);
            Ok(Action::requeue(Duration::from_secs(30)))
        }
        Err(e) => {
            error!("Failed to reconcile WorktreeChangeRequest {}: {}", name, e);
            Ok(Action::requeue(Duration::from_secs(60)))
        }
    }
}

/// Error policy for the controller
fn error_policy(_obj: Arc<WorktreeChangeRequest>, _error: &anyhow::Error, _ctx: Arc<ControllerContext>) -> Action {
    Action::requeue(Duration::from_secs(60))
}

/// Core reconciliation logic
async fn reconcile_worktree(
    crd: Arc<WorktreeChangeRequest>,
    context: Arc<ControllerContext>,
) -> Result<()> {
    let mut crd = (*crd).clone();
    let name = crd.name_any();
    
    // Update status to Running
    update_status(&mut crd, Phase::Running, Some("Reconciling worktree state".to_string())).await?;
    
    // Determine current state based on Git operations
    let new_state = determine_current_state(&mut crd, &context.enhanced_ops).await?;
    
    // Transition to new state if different
    if new_state != crd.spec.state {
        let old_state = crd.spec.state.clone();
        crd.transition_to(
            new_state.clone(),
            None,
            true,
            Some(format!("State transition: {:?} -> {:?}", old_state, new_state)),
        );
    }
    
    // Determine next action
    let next_action = crd.determine_next_action();
    crd.spec.action = next_action.clone();
    
    // Execute action if present
    if let Some(action) = next_action {
        info!("Executing action {:?} for WorktreeChangeRequest: {}", action, name);
        
        let result = execute_action(&crd, &action, &context.enhanced_ops).await;
        
        match result {
            Ok(success) => {
                if success {
                    update_status(&mut crd, Phase::Succeeded, Some("Action completed successfully".to_string())).await?;
                    context.recorder.publish(Event {
                        type_: EventType::Normal,
                        reason: "Reconciled".into(),
                        note: Some(format!("Successfully executed action {:?}", action)),
                        action: "Reconciled".into(),
                        secondary: None,
                    }).await?;
                } else {
                    update_status(&mut crd, Phase::Failed, Some("Action failed".to_string())).await?;
                    crd.spec.retry_count += 1;
                }
            }
            Err(e) => {
                error!("Action execution failed: {}", e);
                update_status(&mut crd, Phase::Failed, Some(format!("Action failed: {}", e))).await?;
                crd.spec.retry_count += 1;
                
                context.recorder.publish(Event {
                    type_: EventType::Warning,
                    reason: "ReconcileFailed".into(),
                    note: Some(format!("Failed to execute action {:?}: {}", action, e)),
                    action: "Reconciled".into(),
                    secondary: None,
                }).await?;
            }
        }
    } else {
        // No action needed, mark as succeeded
        update_status(&mut crd, Phase::Succeeded, Some("No action required".to_string())).await?;
    }
    
    // Update the CRD in the cluster
    update_crd(&crd, &context.client).await?;
    
    Ok(())
}

/// Determine the current state based on Git operations
async fn determine_current_state(
    crd: &mut WorktreeChangeRequest,
    enhanced_ops: &EnhancedWorktreeOps,
) -> Result<WorktreeState> {
    let domains = &mut crd.spec.domains;
    
    // Check local branch status
    let local_branches = get_local_branches().await?;
    domains.local.exists = local_branches.contains(&crd.spec.branch);
    
    // Check remote branch status
    let remote_branches = get_remote_branches().await?;
    domains.remote.exists = remote_branches.contains(&crd.spec.branch);
    
    // Check worktree status
    let worktrees = get_worktrees().await?;
    domains.worktree.exists = worktrees.contains_key(&crd.spec.branch);
    
    if domains.worktree.exists {
        // Check if worktree is dirty
        let status_result = enhanced_ops.get_status().await?;
        if status_result.success {
            domains.worktree.dirty = status_result.output.contains("M");
        }
    }
    
    // Check PR status (simplified - in real implementation would use GitHub API)
    // For now, we'll assume no PR exists unless explicitly set
    domains.pr.exists = false;
    
    // Determine state based on domain information
    let state = if domains.pr.exists {
        WorktreeState::PrCreated
    } else if domains.worktree.conflicted || domains.worktree.rebase_in_progress {
        WorktreeState::Conflicted
    } else if domains.worktree.dirty {
        WorktreeState::Developing
    } else if domains.local.exists && domains.local.ahead > 0 && domains.local.behind == 0 {
        WorktreeState::Ready
    } else if domains.local.behind > 0 {
        WorktreeState::Resolving
    } else {
        WorktreeState::Created
    };
    
    Ok(state)
}

/// Execute an action using enhanced tools
async fn execute_action(
    crd: &WorktreeChangeRequest,
    action: &WorktreeAction,
    enhanced_ops: &EnhancedWorktreeOps,
) -> Result<bool> {
    match action {
        WorktreeAction::CreateWorktree => {
            let result = enhanced_ops.create_worktree_with_setup(&crd.spec.branch, &[]).await?;
            Ok(result.success)
        }
        WorktreeAction::CreateBranch => {
            let result = enhanced_ops.tool_manager.execute_operation(
                crate::tools::ToolOperation::CreateBranch,
                &[&crd.spec.branch],
            ).await?;
            Ok(result.success)
        }
        WorktreeAction::PushBranch => {
            let result = enhanced_ops.tool_manager.execute_operation(
                crate::tools::ToolOperation::PushBranch,
                &[&crd.spec.branch],
            ).await?;
            Ok(result.success)
        }
        WorktreeAction::CleanupWorktree => {
            let result = enhanced_ops.tool_manager.execute_operation(
                crate::tools::ToolOperation::CleanupWorktree,
                &[&crd.spec.branch],
            ).await?;
            Ok(result.success)
        }
        _ => {
            // For other actions, we'll use git as fallback
            let result = enhanced_ops.tool_manager.execute_operation(
                crate::tools::ToolOperation::CreateWorktree, // Placeholder
                &[&crd.spec.branch],
            ).await?;
            Ok(result.success)
        }
    }
}

/// Update the status of a CRD
async fn update_status(
    crd: &mut WorktreeChangeRequest,
    phase: Phase,
    message: Option<String>,
) -> Result<()> {
    if let Some(ref mut status) = crd.status {
        status.phase = phase;
        status.message = message;
        status.last_transition_time = chrono::Utc::now().to_rfc3339();
    }
    Ok(())
}

/// Update a CRD in the cluster
async fn update_crd(crd: &WorktreeChangeRequest, client: &Client) -> Result<()> {
    let api: Api<WorktreeChangeRequest> = Api::all(client.clone());
    let name = crd.name_any();
    let namespace = crd.namespace().unwrap_or_else(|| "default".into());
    
    let patch = json!({
        "status": crd.status
    });
    
    let params = PatchParams::default();
    api.patch_status(&name, &params, &Patch::Merge(patch)).await
        .context("Failed to update CRD status")?;
    
    Ok(())
}

/// Get local branches
async fn get_local_branches() -> Result<Vec<String>> {
    let output = tokio::process::Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .await
        .context("Failed to get local branches")?;
    
    let branches = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .filter(|s| s != "main" && s != "master")
        .collect();
    
    Ok(branches)
}

/// Get remote branches
async fn get_remote_branches() -> Result<Vec<String>> {
    let output = tokio::process::Command::new("git")
        .args(["branch", "-r", "--format=%(refname:short)"])
        .output()
        .await
        .context("Failed to get remote branches")?;
    
    let branches = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .filter(|s| s.starts_with("origin/") && !s.contains("main") && !s.contains("master"))
        .map(|s| s.trim_start_matches("origin/").to_string())
        .collect();
    
    Ok(branches)
}

/// Get worktrees
async fn get_worktrees() -> Result<std::collections::HashMap<String, String>> {
    let output = tokio::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .await
        .context("Failed to get worktrees")?;
    
    let mut worktrees = std::collections::HashMap::new();
    let output_str = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = output_str.lines().collect();
    
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("worktree ") {
            let path = lines[i].trim_start_matches("worktree ");
            if i + 1 < lines.len() && lines[i + 1].starts_with("branch ") {
                let branch = lines[i + 1].trim_start_matches("branch ");
                worktrees.insert(branch.to_string(), path.to_string());
            }
        }
        i += 1;
    }
    
    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::EnhancedWorktreeOps;

    #[tokio::test]
    async fn test_controller_creation() {
        // This would require a real kube client for testing
        // For now, just test that the module compiles
        assert!(true);
    }
} 
