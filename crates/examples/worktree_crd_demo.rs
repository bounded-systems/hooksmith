use anyhow::Result;
use serde_json;
use std::path::PathBuf;
use tempfile::tempdir;
use tracing::{info, warn};

use worktree_runner::{crd::WorktreeChangeRequest, ToolConfig, WorktreeRunner};

/// Comprehensive demo of the Worktree CRD Lifecycle System
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🎭 Worktree CRD Lifecycle System Demo");
    println!("=====================================");
    println!();

    // Create a temporary directory for the demo
    let temp_dir = tempdir()?;
    let repo_path = temp_dir.path().join("repo");
    let worktree_base = temp_dir.path().join("worktrees");
    let storage_dir = temp_dir.path().join(".worktree-state");

    // Initialize the worktree runner
    let mut runner = WorktreeRunner::new();

    // Initialize the CRD system
    info!("Initializing CRD system...");
    runner
        .init_crd_system(
            repo_path.clone(),
            worktree_base.clone(),
            storage_dir.clone(),
            None, // No GitHub token for demo
        )
        .await?;

    println!("✅ CRD system initialized");
    println!();

    // Demo 1: Create a new branch and worktree
    println!("1. Creating a new feature branch...");
    demo_create_feature_branch(&mut runner, "feature/new-feature").await?;
    println!();

    // Demo 2: Show reconciliation
    println!("2. Running reconciliation...");
    let crds = runner.reconcile().await?;
    println!("   Found {} CRDs", crds.len());
    for crd in &crds {
        println!("   {}", crd.get_summary());
    }
    println!();

    // Demo 3: Show status
    println!("3. Current status:");
    let status = runner.get_status().await?;
    for crd in &status {
        println!("   {}", crd.get_summary());
    }
    println!();

    // Demo 4: Export CRDs
    println!("4. Exporting CRDs...");
    let storage = runner.storage.as_ref().unwrap();
    let export_path = temp_dir.path().join("export.json");
    storage
        .export_crds(worktree_runner::storage::ExportFormat::Json, &export_path)
        .await?;
    println!("   Exported to: {:?}", export_path);
    println!();

    // Demo 5: Show storage statistics
    println!("5. Storage statistics:");
    let stats = storage.get_stats().await?;
    println!("   Total CRDs: {}", stats.total_crds);
    println!("   Active CRDs: {}", stats.active_crds);
    println!("   Completed CRDs: {}", stats.completed_crds);
    println!("   Failed CRDs: {}", stats.failed_crds);
    println!("   Storage size: {} bytes", stats.storage_size_bytes);
    println!();

    // Demo 6: Simulate state transitions
    println!("6. Simulating state transitions...");
    demo_state_transitions(&mut runner).await?;
    println!();

    // Demo 7: Show final status
    println!("7. Final status after transitions:");
    let final_status = runner.get_status().await?;
    for crd in &final_status {
        println!("   {}", crd.get_summary());
        if let Some(action) = &crd.spec.action {
            println!("     Next action: {:?}", action);
        }
    }
    println!();

    println!("✅ Demo completed successfully!");
    println!("📁 Demo files are in: {:?}", temp_dir.path());

    Ok(())
}

/// Demo creating a feature branch
async fn demo_create_feature_branch(runner: &mut WorktreeRunner, branch_name: &str) -> Result<()> {
    // Create a new CRD for the branch
    let mut crd = WorktreeChangeRequest::new(branch_name);

    // Simulate the branch being created
    crd.spec.domains.local.exists = true;
    crd.spec.domains.local.current = false;
    crd.spec.domains.local.ahead = 0;
    crd.spec.domains.local.behind = 0;

    // Transition to developing state
    crd.transition_to(
        worktree_runner::crd::WorktreeState::Developing,
        Some(worktree_runner::crd::WorktreeAction::CreateWorktree),
        true,
        Some("Branch created successfully".to_string()),
    );

    // Save the CRD
    let storage = runner.storage.as_ref().unwrap();
    storage.save_crd(&crd).await?;

    println!("   Created CRD for branch: {}", branch_name);
    println!("   State: {:?}", crd.spec.state);

    Ok(())
}

/// Demo state transitions
async fn demo_state_transitions(runner: &mut WorktreeRunner) -> Result<()> {
    let storage = runner.storage.as_ref().unwrap();
    let crds = storage.load_all_crds().await?;

    for (branch_name, mut crd) in crds {
        println!("   Processing branch: {}", branch_name);

        // Simulate different state transitions based on current state
        match crd.spec.state {
            worktree_runner::crd::WorktreeState::Created => {
                // Transition to developing
                crd.transition_to(
                    worktree_runner::crd::WorktreeState::Developing,
                    Some(worktree_runner::crd::WorktreeAction::CreateWorktree),
                    true,
                    Some("Worktree created".to_string()),
                );
                println!("     CREATED → DEVELOPING");
            }

            worktree_runner::crd::WorktreeState::Developing => {
                // Simulate worktree becoming clean and ready
                crd.spec.domains.worktree.dirty = false;
                crd.spec.domains.local.ahead = 2;
                crd.spec.domains.local.behind = 0;

                crd.transition_to(
                    worktree_runner::crd::WorktreeState::Ready,
                    Some(worktree_runner::crd::WorktreeAction::PushBranch),
                    true,
                    Some("Worktree is clean and ready".to_string()),
                );
                println!("     DEVELOPING → READY");
            }

            worktree_runner::crd::WorktreeState::Ready => {
                // Simulate pushing to remote
                crd.spec.domains.remote.exists = true;

                crd.transition_to(
                    worktree_runner::crd::WorktreeState::PrCreated,
                    Some(worktree_runner::crd::WorktreeAction::CreatePr),
                    true,
                    Some("Branch pushed to remote".to_string()),
                );
                println!("     READY → PR_CREATED");
            }

            worktree_runner::crd::WorktreeState::PrCreated => {
                // Simulate PR being merged
                crd.spec.domains.pr.exists = true;
                crd.spec.domains.pr.state = Some(worktree_runner::crd::PrState::Merged);

                crd.transition_to(
                    worktree_runner::crd::WorktreeState::Merged,
                    Some(worktree_runner::crd::WorktreeAction::CleanupWorktree),
                    true,
                    Some("PR merged".to_string()),
                );
                println!("     PR_CREATED → MERGED");
            }

            worktree_runner::crd::WorktreeState::Merged => {
                // Simulate cleanup
                crd.spec.domains.worktree.exists = false;
                crd.spec.domains.local.exists = false;
                crd.spec.domains.remote.exists = false;

                crd.transition_to(
                    worktree_runner::crd::WorktreeState::Removed,
                    None,
                    true,
                    Some("Cleanup completed".to_string()),
                );
                println!("     MERGED → REMOVED");
            }

            _ => {
                println!("     No transition needed for state: {:?}", crd.spec.state);
            }
        }

        // Save the updated CRD
        storage.save_crd(&crd).await?;
    }

    Ok(())
}

/// Helper function to print CRD details
fn print_crd_details(crd: &WorktreeChangeRequest) {
    println!("Branch: {}", crd.spec.branch);
    println!("  State: {:?}", crd.spec.state);
    println!("  Local: {}", crd.spec.domains.local.exists);
    println!("  Remote: {}", crd.spec.domains.remote.exists);
    println!("  Worktree: {}", crd.spec.domains.worktree.exists);
    println!("  PR: {}", crd.spec.domains.pr.exists);
    println!("  Synchronized: {}", crd.is_synchronized());
    if let Some(action) = &crd.spec.action {
        println!("  Next action: {:?}", action);
    }
    println!();
}
