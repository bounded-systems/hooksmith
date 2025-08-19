# Worktree CRD Lifecycle System

## Overview

The Worktree CRD (Custom Resource Definition) Lifecycle System provides synchronized, self-healing worktree management across four domains:

1. **Local Branch** - Git branch in `.git/refs/heads/`
2. **Remote Branch** - Corresponding `origin/*` ref
3. **Worktree** - Physical working directory (e.g., `worktrees/foo`)
4. **Draft PR** - GitHub Pull Request (usually in draft state)

## Architecture

### Core Components

#### 1. WorktreeChangeRequest (CRD)
The central data structure that tracks the state of a branch across all four domains:

```rust
pub struct WorktreeChangeRequest {
    pub metadata: WorktreeMetadata,
    pub spec: WorktreeSpec,
    pub status: WorktreeStatus,
}
```

#### 2. State Machine Engine
Handles reconciliation logic and executes actions based on current state:

```rust
pub struct WorktreeStateMachine {
    repo_path: PathBuf,
    worktree_base: PathBuf,
    github_token: Option<String>,
}
```

#### 3. Storage System
Persists CRDs to disk in JSON format:

```rust
pub struct WorktreeStorage {
    storage_dir: PathBuf,
}
```

### Lifecycle States

The system manages worktrees through these states:

```
CREATED → DEVELOPING → CONFLICTED → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED
```

- **CREATED**: Worktree created but no commits yet
- **DEVELOPING**: Worktree has uncommitted changes
- **CONFLICTED**: Worktree has rebase conflicts
- **RESOLVING**: Resolving conflicts or rebasing
- **READY**: Worktree ready for PR (clean, ahead of main)
- **PR_CREATED**: Pull request created
- **MERGED**: PR merged into main
- **CLEANUP**: Cleaning up worktree
- **REMOVED**: Worktree removed

### Available Actions

The system can perform these actions:

- `create_branch` - Create a new local branch
- `create_worktree` - Create a worktree for a branch
- `push_branch` - Push branch to remote
- `create_pr` - Create GitHub PR
- `merge_pr` - Merge PR into main
- `resolve_conflicts` - Resolve merge/rebase conflicts
- `rebase_main` - Rebase branch onto main
- `cleanup_worktree` - Remove worktree directory
- `remove_branch` - Delete local branch
- `reset_main` - Reset main to match origin/main

## Usage

### Basic Setup

```rust
use worktree_runner::WorktreeRunner;

let mut runner = WorktreeRunner::new();
runner.init_crd_system(
    PathBuf::from("."),           // Repository path
    PathBuf::from("worktrees"),   // Worktree base directory
    PathBuf::from(".worktree-state"), // Storage directory
    Some("ghp_...".to_string()),  // GitHub token (optional)
).await?;
```

### Running Reconciliation

```rust
// Scan all branches and execute necessary actions
let crds = runner.reconcile().await?;
println!("Processed {} CRDs", crds.len());
```

### Getting Status

```rust
// Get status of all worktrees
let status = runner.get_status().await?;
for crd in &status {
    println!("{}", crd.get_summary());
}

// Get status of specific branch
if let Some(crd) = runner.get_branch_status("feature/xyz").await? {
    println!("{}", crd.get_summary());
}
```

### CLI Usage

The system includes a CLI tool for easy interaction:

```bash
# Initialize the CRD system
cargo run --bin crd-cli -- init

# Run reconciliation
cargo run --bin crd-cli -- reconcile

# Get status
cargo run --bin crd-cli -- status

# Get status for specific branch
cargo run --bin crd-cli -- status --branch feature/xyz

# Export CRDs
cargo run --bin crd-cli -- export --output crds.json --format json

# Get storage statistics
cargo run --bin crd-cli -- stats

# Clean up old CRDs
cargo run --bin crd-cli -- cleanup --max-age-days 30

# Run demo
cargo run --bin crd-cli -- demo
```

## JSON Schema

The CRD follows this JSON schema:

```json
{
  "metadata": {
    "name": "feature/xyz",
    "namespace": "default",
    "created": "2025-01-15T10:30:00Z",
    "lastModified": "2025-01-15T11:45:00Z",
    "version": "v1"
  },
  "spec": {
    "branch": "feature/xyz",
    "domains": {
      "local": {
        "exists": true,
        "current": false,
        "lastCommit": "abc123...",
        "ahead": 2,
        "behind": 0
      },
      "remote": {
        "exists": true,
        "lastCommit": "def456...",
        "upstream": "origin/feature/xyz"
      },
      "worktree": {
        "exists": true,
        "path": "/path/to/worktrees/feature/xyz",
        "dirty": false,
        "conflicted": false,
        "rebaseInProgress": false
      },
      "pr": {
        "exists": true,
        "number": 123,
        "url": "https://github.com/owner/repo/pull/123",
        "state": "open",
        "title": "Feature XYZ",
        "labels": ["feature"]
      }
    },
    "state": "PR_CREATED",
    "action": "merge_pr",
    "priority": 5,
    "retryCount": 0,
    "maxRetries": 3
  },
  "status": {
    "phase": "Pending",
    "message": "Ready for merge",
    "lastTransitionTime": "2025-01-15T11:45:00Z",
    "conditions": [],
    "history": [
      {
        "timestamp": "2025-01-15T10:30:00Z",
        "state": "CREATED",
        "action": "create_worktree",
        "success": true,
        "message": "Worktree created successfully"
      }
    ]
  }
}
```

## Storage

CRDs are stored in JSON files in the `.worktree-state/` directory:

```
.worktree-state/
├── feature_xyz.json
├── bugfix_123.json
└── hotfix_urgent.json
```

### Storage Operations

```rust
use worktree_runner::storage::WorktreeStorage;

let storage = WorktreeStorage::new(PathBuf::from(".worktree-state"));
storage.init().await?;

// Save CRD
storage.save_crd(&crd).await?;

// Load CRD
let crd = storage.load_crd("feature/xyz").await?;

// Load all CRDs
let all_crds = storage.load_all_crds().await?;

// Export to different formats
storage.export_crds(ExportFormat::Json, &PathBuf::from("export.json")).await?;
storage.export_crds(ExportFormat::Yaml, &PathBuf::from("export.yaml")).await?;
storage.export_crds(ExportFormat::Csv, &PathBuf::from("export.csv")).await?;

// Get statistics
let stats = storage.get_stats().await?;
println!("Total CRDs: {}", stats.total_crds);

// Clean up old CRDs
let deleted = storage.cleanup_old_crds(30).await?;
println!("Deleted {} old CRDs", deleted);
```

## State Machine Logic

### State Determination

The system determines the current state based on domain information:

1. **MERGED**: PR state is "merged"
2. **PR_CREATED**: PR exists
3. **CONFLICTED**: Worktree has conflicts or rebase in progress
4. **DEVELOPING**: Worktree is dirty
5. **READY**: Clean worktree, ahead of main, not behind
6. **RESOLVING**: Behind main
7. **CREATED**: Default state

### Action Determination

The system determines the next action based on current state and domain status:

- **CREATED**: Create branch → Create worktree
- **DEVELOPING**: Push branch → Rebase main → Create PR
- **CONFLICTED**: Resolve conflicts
- **RESOLVING**: Resolve conflicts → Rebase main
- **READY**: Push branch → Create PR
- **PR_CREATED**: Merge PR
- **MERGED**: Cleanup worktree
- **CLEANUP**: Cleanup worktree → Remove branch
- **REMOVED**: Terminal state

## Synchronization

The system ensures all four domains are synchronized:

```rust
// Check if all domains are in sync
let synchronized = crd.is_synchronized();

// This checks:
// - Local and remote branches exist together
// - Worktree exists when local branch exists
// - PR exists when in PR_CREATED or MERGED state
```

## Dirty Main Recovery

The system can handle dirty main branches by:

1. Detecting uncommitted changes on main
2. Creating a new feature branch from main
3. Committing the changes to the feature branch
4. Pushing the feature branch
5. Creating a draft PR
6. Resetting main to match origin/main exactly

## Integration with Existing Tools

The system integrates with:

- **Git**: All Git operations
- **GitHub CLI**: PR creation and management
- **Worktree tools**: wtp, wt, treekanga
- **CI/CD**: Can be run in automation

## Configuration

The system can be configured via the `ToolConfig`:

```rust
let config = ToolConfig {
    preferred_tool: Some("wtp".to_string()),
    worktree_base: Some("worktrees".to_string()),
    run_setup: true,
    setup_commands: vec!["cargo build".to_string()],
    copy_env: true,
    env_files: vec![".env".to_string()],
};

let runner = WorktreeRunner::with_config(config);
```

## Examples

### Complete Workflow Example

```rust
use worktree_runner::WorktreeRunner;

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = WorktreeRunner::new();
    
    // Initialize CRD system
    runner.init_crd_system(
        PathBuf::from("."),
        PathBuf::from("worktrees"),
        PathBuf::from(".worktree-state"),
        Some(std::env::var("GITHUB_TOKEN")?),
    ).await?;
    
    // Run reconciliation
    let crds = runner.reconcile().await?;
    
    // Print results
    for crd in &crds {
        println!("{}", crd.get_summary());
        if let Some(action) = &crd.spec.action {
            println!("  Next action: {:?}", action);
        }
    }
    
    Ok(())
}
```

### CLI Example

```bash
# Initialize system
cargo run --bin crd-cli -- init

# Run reconciliation
cargo run --bin crd-cli -- reconcile --output-format json

# Get status
cargo run --bin crd-cli -- status

# Export to CSV
cargo run --bin crd-cli -- export --output status.csv --format csv
```

## Testing

Run the comprehensive demo:

```bash
cargo run --example worktree_crd_demo
```

Run tests:

```bash
cargo test
```

## Benefits

1. **Declarative**: State is explicit and traceable
2. **Idempotent**: Operations can be run multiple times safely
3. **Self-healing**: Automatically detects and fixes drift
4. **Auditable**: Complete history of all state transitions
5. **GitOps-style**: Treats branches like live resources
6. **Main protection**: Keeps main clean and synchronized

## Future Enhancements

1. **Webhook integration**: Real-time updates from GitHub
2. **Metrics and monitoring**: Prometheus metrics
3. **Multi-repo support**: Manage multiple repositories
4. **Advanced conflict resolution**: Automated conflict resolution
5. **Policy engine**: Custom policies for different branch types
6. **Dashboard**: Web UI for monitoring and management 
