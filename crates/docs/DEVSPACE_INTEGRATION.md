# Devspace Integration for Worktree Management

## Overview

This document describes the integration of Devspace with our worktree CRD system. Devspace is a Rust-based CLI tool designed to simplify working with multiple Git worktrees and open PRs across repositories, making it an ideal complement to our declarative CRD approach.

## What is Devspace?

Devspace is a Rust CLI tool created by muzomer that:
- Manages multiple Git worktrees across repositories
- Provides context switching between different features/PRs
- Automatically handles trunk branch checkout and PR-based worktrees
- Organizes worktrees in a structured directory layout

## Architecture Integration

### Hybrid Approach

Our system uses a hybrid approach that combines:
1. **Devspace** - Interactive worktree management and context switching
2. **CRD System** - Declarative state management and lifecycle control
3. **Tool Integration** - Automatic tool selection and fallback

### Directory Structure

Devspace organizes files like this:
```
repositories_dir/
  project-A/
  project-B/
worktrees_dir/
  project-A/
    feature/foo
  project-B/
    feature/another
```

## Integration Features

### 1. Automatic Tool Selection

The system automatically selects Devspace for appropriate operations:

```rust
fn is_tool_suitable(&self, tool: &WorktreeTool, operation: &ToolOperation) -> bool {
    match (tool, operation) {
        (WorktreeTool::Devspace, ToolOperation::CreateWorktree) => true,
        (WorktreeTool::Devspace, ToolOperation::SwitchContext) => true,
        (WorktreeTool::Devspace, ToolOperation::ListWorktrees) => true,
        // ... other tools
    }
}
```

### 2. Devspace Operations

#### Create Worktree
```bash
# Create a new worktree using Devspace
cargo run -p worktree-runner --bin crd-cli -- tools devspace create --branch feature/xyz
```

#### Switch Context
```bash
# Switch to a different context/worktree
cargo run -p worktree-runner --bin crd-cli -- tools devspace switch --context feature/xyz
```

#### List Worktrees
```bash
# List all available contexts/worktrees
cargo run -p worktree-runner --bin crd-cli -- tools devspace list
```

### 3. CRD Integration

Devspace operations are integrated with our CRD system:

```rust
// When creating a worktree, the system can use Devspace
match action {
    WorktreeAction::CreateWorktree => {
        // Try Devspace first, fallback to other tools
        enhanced_ops.create_worktree_with_setup(&branch_name, &[]).await
    }
    WorktreeAction::SwitchContext => {
        // Use Devspace for context switching
        enhanced_ops.switch_context(&context_name).await
    }
}
```

## Workflow Integration

### 1. Development Workflow

```bash
# 1. Create a new feature using Devspace
cargo run -p worktree-runner --bin crd-cli -- tools devspace create --branch feature/xyz

# 2. Switch to the new context
cargo run -p worktree-runner --bin crd-cli -- tools devspace switch --context feature/xyz

# 3. Work in the isolated environment
cd worktrees_dir/project-A/feature/xyz
# Make changes, commit, etc.

# 4. Let the CRD system handle lifecycle
cargo run -p worktree-runner --bin crd-cli -- reconcile
```

### 2. Multi-Repository Workflow

Devspace excels at managing multiple repositories:

```bash
# Create worktrees across multiple repos
cargo run -p worktree-runner --bin crd-cli -- tools devspace create --branch feature/xyz
# This creates worktrees in both frontend and backend repos if configured

# Switch context across all repos
cargo run -p worktree-runner --bin crd-cli -- tools devspace switch --context feature/xyz
```

### 3. Context Switching

```bash
# List available contexts
cargo run -p worktree-runner --bin crd-cli -- tools devspace list

# Switch between different features
cargo run -p worktree-runner --bin crd-cli -- tools devspace switch --context feature/abc
cargo run -p worktree-runner --bin crd-cli -- tools devspace switch --context bugfix/123
```

## Benefits of Devspace Integration

### 1. Context Isolation

- **Separate directories** for each feature/PR
- **Clean context switching** between different work items
- **Isolated environments** prevent cross-contamination

### 2. Multi-Repository Support

- **Unified management** across multiple repositories
- **Consistent naming** and organization
- **Cross-repo operations** with single commands

### 3. Developer Experience

- **Intuitive CLI** for common operations
- **Visual organization** of worktrees
- **Fast context switching** between features

### 4. CRD Complementarity

- **Devspace** handles the interactive/organizational aspects
- **CRD System** handles the declarative lifecycle management
- **Best of both worlds** - ergonomic interface + robust state management

## Tool Comparison

| Feature | Devspace | Workbloom | Gwtr | Git |
|---------|----------|-----------|------|-----|
| Context switching | ✅ | ❌ | ❌ | ❌ |
| Multi-repo support | ✅ | ❌ | ❌ | ❌ |
| Worktree creation | ✅ | ✅ | ✅ | ✅ |
| Bulk operations | ❌ | ❌ | ✅ | ❌ |
| PR integration | ✅ | ❌ | ❌ | ❌ |
| File copying | ❌ | ✅ | ❌ | ❌ |

## Configuration

### Devspace Configuration

Devspace can be configured via environment variables or config files:

```bash
# Set repositories directory
export DEVSPACE_REPOS_DIR="/path/to/repositories"

# Set worktrees directory  
export DEVSPACE_WORKTREES_DIR="/path/to/worktrees"

# Enable multi-repo mode
export DEVSPACE_MULTI_REPO=true
```

### Integration Configuration

Our system respects Devspace's configuration:

```rust
// The system automatically detects Devspace configuration
let devspace_config = std::env::var("DEVSPACE_REPOS_DIR")
    .ok()
    .map(|dir| PathBuf::from(dir));

// Use Devspace directories if available
if let Some(repos_dir) = devspace_config {
    // Integrate with Devspace structure
}
```

## Advanced Features

### 1. Hybrid Controller

```rust
// Controller that uses Devspace for worktree ops
async fn reconcile_with_devspace(
    crd: Arc<WorktreeChangeRequest>,
    context: Arc<ControllerContext>,
) -> Result<Action> {
    // Use Devspace for worktree operations
    if let Some(WorktreeAction::CreateWorktree) = crd.spec.action {
        let result = context.enhanced_ops.create_worktree_with_setup(
            &crd.spec.branch, 
            &[]
        ).await?;
        
        if result.tool_used == "devspace" {
            info!("Created worktree using Devspace");
        }
    }
    
    // Use CRD system for lifecycle management
    // ... rest of reconciliation logic
}
```

### 2. Context-Aware Operations

```rust
// Operations that are context-aware
pub async fn context_aware_operation(&self, operation: &str) -> Result<ToolResult> {
    // Get current context from Devspace
    let current_context = self.get_current_context().await?;
    
    // Perform operation in context
    match operation {
        "build" => self.build_in_context(&current_context).await,
        "test" => self.test_in_context(&current_context).await,
        "deploy" => self.deploy_in_context(&current_context).await,
        _ => Err(anyhow::anyhow!("Unknown operation"))
    }
}
```

### 3. Multi-Repository Coordination

```rust
// Coordinate operations across multiple repositories
pub async fn coordinate_across_repos(&self, operation: &str) -> Result<Vec<ToolResult>> {
    let repos = self.get_configured_repos().await?;
    let mut results = Vec::new();
    
    for repo in repos {
        let result = self.execute_in_repo(&repo, operation).await?;
        results.push(result);
    }
    
    Ok(results)
}
```

## Community Feedback

From the Rust community:

> "I created [Devspace] … and it works well for me daily"

This feedback highlights Devspace's reliability in daily Git worktree workflows.

## Best Practices

### 1. Tool Selection

- **Use Devspace** for context switching and multi-repo operations
- **Use Workbloom** for worktree creation with environment setup
- **Use Gwtr** for bulk operations and cleanup
- **Use Git** as fallback for all operations

### 2. Workflow Organization

- **Keep repositories clean** - use Devspace for feature isolation
- **Regular context switching** - switch between features frequently
- **Consistent naming** - use descriptive branch/context names
- **Cleanup regularly** - let the CRD system handle lifecycle

### 3. Integration Patterns

- **Devspace for organization** - manage worktree structure
- **CRD for lifecycle** - handle state transitions and cleanup
- **Tool integration for operations** - automatic tool selection
- **CLI for interaction** - unified command interface

## Future Enhancements

### 1. Advanced Devspace Integration

- **Context-aware CRDs** - CRDs that understand Devspace contexts
- **Multi-repo CRDs** - CRDs that span multiple repositories
- **Context switching events** - Automatic CRD updates on context switch

### 2. Enhanced Tooling

- **Devspace plugins** - Custom plugins for specific workflows
- **IDE integration** - VS Code/IntelliJ support for Devspace
- **CI/CD integration** - Automated testing in Devspace contexts

### 3. Production Features

- **Context persistence** - Save and restore context state
- **Context sharing** - Share contexts across team members
- **Context templates** - Predefined context configurations

## Conclusion

The Devspace integration provides an excellent complement to our CRD system:

- **Devspace** handles the interactive and organizational aspects
- **CRD System** handles the declarative lifecycle management
- **Tool Integration** provides automatic tool selection and fallback
- **CLI** provides a unified interface for all operations

This hybrid approach gives developers the best of both worlds: ergonomic worktree management with robust state tracking and lifecycle control. 
