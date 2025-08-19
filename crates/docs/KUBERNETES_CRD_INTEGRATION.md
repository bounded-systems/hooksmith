# Kubernetes CRD Integration for Worktree Management

## Overview

This document describes the Kubernetes Custom Resource Definition (CRD) integration for the worktree lifecycle management system. We've successfully implemented a production-ready CRD system using the `kube-rs` ecosystem that provides declarative Git workflow management.

## Architecture

### Core Components

1. **WorktreeChangeRequest CRD** - Kubernetes-style custom resource
2. **kube-derive Integration** - Automatic CRD generation
3. **Schema Validation** - OpenAPI v3 schema generation
4. **CLI Integration** - Command-line CRD management

### Key Features

- ✅ **Automatic CRD Generation** - Using `kube-derive` macros
- ✅ **OpenAPI Schema** - Full type safety and validation
- ✅ **YAML Output** - Kubernetes-compatible manifests
- ✅ **CLI Commands** - Easy CRD creation and management
- ✅ **Tool Integration** - Works with existing worktree tools

## CRD Definition

### WorktreeChangeRequest Structure

```rust
#[derive(CustomResource, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[kube(
    group = "hooksmith.dev",
    version = "v1",
    kind = "WorktreeChangeRequest",
    namespaced
)]
pub struct WorktreeChangeRequestSpec {
    pub branch: String,
    pub domains: WorktreeDomains,
    pub state: WorktreeState,
    pub action: Option<WorktreeAction>,
    pub priority: i32,
    pub retry_count: i32,
    pub max_retries: i32,
}
```

### Four-Domain Model

The CRD tracks state across four domains:

1. **Local Domain** - Git local branch state
2. **Remote Domain** - Git remote branch state  
3. **Worktree Domain** - Physical worktree directory
4. **PR Domain** - GitHub Pull Request state

### Lifecycle States

```rust
pub enum WorktreeState {
    Created,      // Initial state
    Developing,   // Active development
    Conflicted,   // Merge/rebase conflicts
    Resolving,    // Resolving conflicts
    Ready,        // Ready for PR
    PrCreated,    // PR exists
    Merged,       // PR merged
    Cleanup,      // Cleaning up
    Removed,      // Terminal state
}
```

### Available Actions

```rust
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
```

## Usage Examples

### Generate CRD YAML

```bash
# Generate the Kubernetes CRD definition
cargo run -p worktree-runner --bin crd-cli -- kube generate-crd > worktree-crd.yaml

# Apply to Kubernetes cluster
kubectl apply -f worktree-crd.yaml
```

**Generated CRD includes:**
- Full OpenAPI v3 schema
- All enum values and types
- Required field validation
- Default values
- Proper Kubernetes metadata

### Create WorktreeChangeRequest

```bash
# Create a CRD for a new feature branch
cargo run -p worktree-runner --bin crd-cli -- kube create --branch feature/xyz

# Output: Kubernetes-compatible YAML
---
apiVersion: hooksmith.dev/v1
kind: WorktreeChangeRequest
metadata:
  name: feature/xyz
  namespace: default
spec:
  branch: feature/xyz
  state: Created
  priority: 5
  retry_count: 0
  max_retries: 3
  domains:
    local:
      exists: false
      current: false
      ahead: 0
      behind: 0
    remote:
      exists: false
    worktree:
      exists: false
      dirty: false
      conflicted: false
      rebase_in_progress: false
    pr:
      exists: false
      labels: []
```

### Apply to Kubernetes

```bash
# Create the CRD in Kubernetes
kubectl apply -f worktree-crd.yaml

# Create a WorktreeChangeRequest
kubectl apply -f feature-xyz.yaml

# List all WorktreeChangeRequests
kubectl get worktreechangerequests

# Get details of a specific CRD
kubectl describe worktreechangerequest feature/xyz
```

## Integration with Existing Tools

### Tool Integration Layer

The CRD system integrates with existing Rust worktree tools:

```rust
// Automatic tool selection for CRD actions
match action {
    WorktreeAction::CreateWorktree => {
        enhanced_ops.create_worktree_with_setup(&branch_name, &[]).await
    }
    WorktreeAction::CleanupWorktree => {
        enhanced_ops.tool_manager.execute_operation(
            ToolOperation::CleanupWorktree,
            &[&branch_name],
        ).await
    }
    // ... other actions
}
```

### Supported Tools

- **Workbloom** (`wb`) - Worktree creation and setup
- **Gwtr** (`gwtr`) - Bulk operations and cleanup
- **Git-worktree-cli** - Enhanced status
- **Git** - Fallback for all operations

## Benefits of Kubernetes CRD Approach

### 1. Declarative Management

```yaml
# Declare desired state
apiVersion: hooksmith.dev/v1
kind: WorktreeChangeRequest
metadata:
  name: feature/xyz
spec:
  branch: feature/xyz
  state: Ready
  priority: 1
```

### 2. Kubernetes Ecosystem Integration

- **kubectl** - Standard Kubernetes CLI
- **Helm** - Package management
- **ArgoCD** - GitOps deployment
- **Prometheus** - Metrics and monitoring
- **Grafana** - Dashboards

### 3. Production Features

- **Schema Validation** - Type safety at the API level
- **RBAC** - Role-based access control
- **Audit Logging** - Complete audit trail
- **Webhooks** - Event-driven automation
- **Custom Controllers** - Advanced reconciliation

### 4. Developer Experience

```bash
# Standard Kubernetes workflow
kubectl get worktreechangerequests
kubectl describe worktreechangerequest feature/xyz
kubectl edit worktreechangerequest feature/xyz
kubectl delete worktreechangerequest feature/xyz
```

## Advanced Features

### Custom Controllers

The system is designed to support Kubernetes controllers:

```rust
// Controller reconciliation loop
async fn reconcile(
    crd: Arc<WorktreeChangeRequest>,
    context: Arc<ControllerContext>,
) -> Result<Action> {
    // Determine current state
    let new_state = determine_current_state(&mut crd, &context.enhanced_ops).await?;
    
    // Execute actions
    if let Some(action) = crd.determine_next_action() {
        execute_action(&crd, &action, &context.enhanced_ops).await?;
    }
    
    Ok(Action::requeue(Duration::from_secs(30)))
}
```

### Status Tracking

```rust
pub struct WorktreeChangeRequestStatus {
    pub phase: Phase,                    // Pending/Running/Succeeded/Failed
    pub message: Option<String>,         // Human-readable status
    pub last_transition_time: String,    // Timestamp
    pub conditions: Vec<Condition>,      // Kubernetes conditions
    pub history: Vec<HistoryEntry>,      // State transition history
}
```

### Conditions and Events

```rust
// Kubernetes-style conditions
pub struct Condition {
    pub condition_type: String,          // e.g., "Synchronized"
    pub status: ConditionStatus,         // True/False/Unknown
    pub last_transition_time: String,
    pub reason: Option<String>,          // e.g., "AllDomainsInSync"
    pub message: Option<String>,
}
```

## Deployment Options

### 1. Local Development

```bash
# Generate and apply CRD locally
cargo run -p worktree-runner --bin crd-cli -- kube generate-crd > crd.yaml
kubectl apply -f crd.yaml

# Create WorktreeChangeRequests
cargo run -p worktree-runner --bin crd-cli -- kube create --branch feature/xyz
```

### 2. Kubernetes Cluster

```bash
# Deploy to production cluster
kubectl apply -f crd.yaml
kubectl apply -f worktreechangerequests/

# Monitor with standard tools
kubectl get events --field-selector involvedObject.kind=WorktreeChangeRequest
```

### 3. GitOps Workflow

```yaml
# ArgoCD Application
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: worktree-management
spec:
  source:
    repoURL: https://github.com/your-org/worktree-config
    path: k8s
  destination:
    server: https://kubernetes.default.svc
    namespace: worktree-system
```

## Monitoring and Observability

### Metrics

```rust
// Prometheus metrics
worktree_requests_total{state="developing"} 5
worktree_actions_duration_seconds{action="create_worktree"} 2.5
worktree_synchronization_status{domain="local"} 1
```

### Logging

```rust
// Structured logging
info!(
    "Reconciling WorktreeChangeRequest",
    branch = %crd.spec.branch,
    state = ?crd.spec.state,
    action = ?crd.spec.action
);
```

### Dashboards

- **Worktree Status Dashboard** - Overview of all worktrees
- **Action Performance Dashboard** - Timing and success rates
- **Domain Synchronization Dashboard** - Drift detection
- **Error Analysis Dashboard** - Failed actions and retries

## Future Enhancements

### 1. Advanced Controllers

- **Multi-cluster support** - Cross-cluster worktree management
- **Policy enforcement** - Custom admission controllers
- **Automated cleanup** - Background reconciliation
- **Webhook integration** - GitHub event processing

### 2. Enhanced Tooling

- **kubectl plugins** - Native Kubernetes CLI integration
- **IDE extensions** - VS Code/IntelliJ support
- **CI/CD integration** - Automated testing and deployment
- **Documentation generation** - Auto-generated docs

### 3. Enterprise Features

- **Multi-tenancy** - Namespace isolation
- **RBAC policies** - Fine-grained permissions
- **Audit logging** - Compliance requirements
- **Backup/restore** - Disaster recovery

## Conclusion

The Kubernetes CRD integration provides a robust, production-ready foundation for declarative worktree management. By leveraging the mature `kube-rs` ecosystem, we get:

- **Type Safety** - Compile-time validation
- **Schema Generation** - Automatic OpenAPI schemas
- **Kubernetes Integration** - Native Kubernetes tooling
- **Extensibility** - Easy to add new features
- **Production Readiness** - Enterprise-grade reliability

This approach gives developers the benefits of both worlds: the power of Kubernetes for orchestration and the simplicity of declarative Git workflow management. 
