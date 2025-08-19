# Integrated Worktree CRD System

## Overview

This document describes the integrated worktree management system that combines our custom CRD (Custom Resource Definition) lifecycle management with existing Rust-based worktree tools for a comprehensive, self-healing Git workflow.

## Architecture

### Core Components

1. **WorktreeChangeRequest CRD** - Tracks state across four domains
2. **State Machine Engine** - Handles reconciliation and transitions
3. **Storage System** - Persists CRDs to disk
4. **Tool Integration Layer** - Bridges to existing Rust tools
5. **CLI Interface** - Unified command-line interface

### Tool Integration

The system integrates with these existing Rust worktree tools:

- **Workbloom** (`wb`) - Worktree creation and environment setup
- **Gwtr** (`gwtr`) - Bulk operations and cleanup
- **Git-worktree-cli** - Enhanced status and navigation
- **Git** - Fallback for all operations

## Tool Capabilities

### Workbloom Integration

```bash
# Create worktree with automatic setup
cargo run --bin crd-cli -- tools create --branch feature/xyz

# Setup environment for existing worktree
cargo run --bin crd-cli -- tools setup --branch feature/xyz

# Check tool status
cargo run --bin crd-cli -- tools status
```

**Capabilities:**
- Automatic worktree creation with environment setup
- File copying (`.env`, `.envrc`, config files)
- Interactive cleanup of merged worktrees
- Colorful terminal UI and progress feedback

### Gwtr Integration

```bash
# Bulk pull all worktrees
cargo run --bin crd-cli -- bulk pull

# Prune stale worktrees
cargo run --bin crd-cli -- bulk prune --force

# Get comprehensive status
cargo run --bin crd-cli -- bulk status
```

**Capabilities:**
- Named worktrees as `{repo_name}_{branch}` for directory hygiene
- Bulk pulling/updating of all worktrees from origin/main
- Smart pruning with `--dry-run` and `--force` options
- Built-in status display

### Git Worktree CLI Integration

```bash
# Enhanced status with PR integration
cargo run --bin crd-cli -- bulk status
```

**Capabilities:**
- GitHub PR integration and visual rendering
- Enhanced status display
- Navigation improvements

## State Machine Integration

The CRD system uses the integrated tools for specific operations:

### State Transitions with Tool Integration

```rust
// When transitioning from CREATED to DEVELOPING
match action {
    WorktreeAction::CreateWorktree => {
        // Use workbloom for creation with setup
        enhanced_ops.create_worktree_with_setup(&branch_name, &[]).await
    }
    WorktreeAction::CleanupWorktree => {
        // Use gwtr for smart cleanup
        enhanced_ops.prune_worktrees(false).await
    }
    WorktreeAction::BulkPull => {
        // Use gwtr for bulk operations
        enhanced_ops.bulk_pull_all().await
    }
}
```

### Tool Selection Logic

The system automatically selects the best tool for each operation:

```rust
fn is_tool_suitable(&self, tool: &WorktreeTool, operation: &ToolOperation) -> bool {
    match (tool, operation) {
        (WorktreeTool::Workbloom, ToolOperation::CreateWorktree) => true,
        (WorktreeTool::Workbloom, ToolOperation::SetupEnvironment) => true,
        (WorktreeTool::Gwtr, ToolOperation::BulkPull) => true,
        (WorktreeTool::Gwtr, ToolOperation::PruneWorktrees) => true,
        (WorktreeTool::GitWorktreeCli, ToolOperation::Status) => true,
        (WorktreeTool::Git, _) => true, // Git can do everything
        _ => false,
    }
}
```

## CLI Usage Examples

### Basic CRD Operations

```bash
# Initialize the system
cargo run --bin crd-cli -- init

# Run reconciliation
cargo run --bin crd-cli -- reconcile

# Get status
cargo run --bin crd-cli -- status

# Get status for specific branch
cargo run --bin crd-cli -- status --branch feature/xyz
```

### Tool Integration Commands

```bash
# Check available tools
cargo run --bin crd-cli -- tools status

# Create worktree with workbloom
cargo run --bin crd-cli -- tools create --branch feature/xyz

# Setup environment
cargo run --bin crd-cli -- tools setup --branch feature/xyz

# Bulk operations with gwtr
cargo run --bin crd-cli -- bulk pull
cargo run --bin crd-cli -- bulk prune --force
cargo run --bin crd-cli -- bulk status
```

### Export and Management

```bash
# Export CRDs
cargo run --bin crd-cli -- export --output crds.json --format json
cargo run --bin crd-cli -- export --output status.csv --format csv

# Cleanup old CRDs
cargo run --bin crd-cli -- cleanup --max-age-days 30

# Get statistics
cargo run --bin crd-cli -- stats
```

## Configuration

### Tool Preferences

The system can be configured to prefer specific tools:

```rust
let config = ToolConfig {
    preferred_tool: Some("wb".to_string()), // Prefer workbloom
    worktree_base: Some("worktrees".to_string()),
    run_setup: true,
    setup_commands: vec!["cargo build".to_string()],
    copy_env: true,
    env_files: vec![".env".to_string()],
};
```

### Automatic Tool Detection

The system automatically detects available tools:

```rust
// Get all available tools
let available_tools = WorktreeTool::get_available_tools();
println!("Available tools: {:?}", available_tools);

// Get tool status
let tool_status = runner.get_tool_status()?;
for tool in tool_status {
    println!("{}: {} (preferred: {})", 
        tool.name, 
        if tool.available { "available" } else { "not available" },
        tool.preferred
    );
}
```

## Workflow Integration

### Complete Development Workflow

1. **Create Feature Branch**
   ```bash
   cargo run --bin crd-cli -- tools create --branch feature/xyz
   ```

2. **Develop and Commit**
   ```bash
   # Work in the created worktree
   cd worktrees/feature/xyz
   # Make changes, commit, etc.
   ```

3. **Reconcile State**
   ```bash
   cargo run --bin crd-cli -- reconcile
   ```

4. **Bulk Operations**
   ```bash
   # Pull all worktrees
   cargo run --bin crd-cli -- bulk pull
   
   # Prune stale worktrees
   cargo run --bin crd-cli -- bulk prune
   ```

5. **Cleanup After Merge**
   ```bash
   # The system automatically detects merged PRs and cleans up
   cargo run --bin crd-cli -- reconcile
   ```

### CI/CD Integration

The system can be integrated into CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Reconcile worktrees
  run: |
    cargo run --bin crd-cli -- reconcile
    cargo run --bin crd-cli -- bulk pull
    cargo run --bin crd-cli -- bulk prune

- name: Export status
  run: |
    cargo run --bin crd-cli -- export --output status.json --format json
    cargo run --bin crd-cli -- stats
```

## Benefits of Integration

### 1. Best of Both Worlds

- **CRD System**: Declarative state management, self-healing, audit trail
- **Existing Tools**: Proven workflows, community support, rich features

### 2. Automatic Tool Selection

- System chooses the best tool for each operation
- Graceful fallback to git when specialized tools aren't available
- No manual tool selection required

### 3. Enhanced Operations

- **Workbloom**: Automatic environment setup and file copying
- **Gwtr**: Efficient bulk operations and smart cleanup
- **Git-worktree-cli**: Enhanced status with PR integration

### 4. Unified Interface

- Single CLI for all operations
- Consistent output formats
- Integrated error handling and logging

## Performance Considerations

### Tool Availability Detection

```rust
// Fast tool detection
pub fn is_available(&self) -> bool {
    Command::new(self.command_name())
        .arg("--version")
        .output()
        .is_ok()
}
```

### Caching

The system caches tool availability and version information to avoid repeated detection.

### Parallel Operations

Bulk operations can be parallelized when using tools like gwtr that support concurrent worktree operations.

## Error Handling

### Tool Fallback

```rust
// Automatic fallback to git
match tool {
    WorktreeTool::Workbloom => self.execute_with_workbloom(operation, args).await,
    WorktreeTool::Git => self.execute_with_git(operation, args).await,
    _ => {
        warn!("Tool not available, falling back to git");
        self.execute_with_git(operation, args).await
    }
}
```

### Error Reporting

All tool operations return structured results:

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub tool_used: String,
}
```

## Future Enhancements

### 1. Webhook Integration

- Real-time updates from GitHub when PRs are created/merged
- Automatic CRD state updates

### 2. Advanced Tool Integration

- Support for more worktree tools as they emerge
- Plugin system for custom tool integrations

### 3. Metrics and Monitoring

- Prometheus metrics for tool usage
- Performance monitoring for bulk operations

### 4. Policy Engine

- Custom policies for different branch types
- Automated compliance checking

## Conclusion

The integrated worktree CRD system provides a powerful, self-healing Git workflow that leverages the best features of existing Rust worktree tools while adding declarative state management and comprehensive audit trails. This hybrid approach gives developers the benefits of both worlds: proven tooling and modern GitOps-style management. 
