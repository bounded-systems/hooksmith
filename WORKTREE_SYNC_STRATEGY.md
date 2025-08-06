# Worktree Sync Strategy

## Overview

The Worktree Sync Strategy is a comprehensive conflict-free worktree management system designed to prevent merge conflicts and maintain a clean, organized development workflow. It implements the **1:1:1:1:1 mapping model**:

- **1 Worktree** = **1 Local branch** = **1 Remote branch** = **1 Draft PR** = **Not main**

## Core Principles

### 🔄 **Upstream-First Sync Model**

1. **Treat origin/main as single source of truth**
   - Never develop directly on main
   - Always reset local main to match origin/main
   - Sync main first, then worktrees

2. **Isolated worktree development**
   - Each worktree = dedicated branch/PR
   - Never work directly on main in worktrees
   - Keep each unit of work isolated

3. **Pre-sync validation**
   - Commit local changes before syncing
   - Validate sync readiness
   - Auto-resolve trivial conflicts

## Strategy Implementation

### 📋 **Sync Process**

```bash
# 1. Sync main first (single source of truth)
git fetch origin
git checkout main
git reset --hard origin/main

# 2. Sync each worktree
for worktree in worktrees:
    git checkout feature/branch
    git merge --ff-only origin/main || git merge origin/main
```

### 🛠️ **CLI Integration**

```bash
# Run worktree sync strategy
cargo xtask worktree sync-strategy

# With validation
cargo xtask worktree sync-strategy --validate

# Generate sync report
cargo xtask worktree sync-strategy --report

# Force sync (ignore uncommitted changes)
cargo xtask worktree sync-strategy --force
```

### 📊 **Sync State Tracking**

The system tracks worktree sync states:

```json
{
  "branch": "feature/example",
  "is_ahead_of_main": true,
  "has_unmerged_main": false,
  "has_uncommitted": false,
  "pr_synced": true,
  "last_sync": "2024-01-15T10:30:00Z",
  "sync_status": "Clean"
}
```

### 🔍 **Sync Status Types**

- **Clean**: Worktree is synced and ready
- **UncommittedChanges**: Has uncommitted changes
- **MergeConflicts**: Has merge conflicts
- **AheadOfMain**: Has commits ahead of main
- **UnmergedMain**: Has unmerged main commits
- **OutOfSync**: Out of sync with remote

## Conflict Prevention

### ✅ **Best Practices**

1. **Always sync main first**
   ```bash
   git fetch origin
   git checkout main
   git reset --hard origin/main
   ```

2. **Commit before syncing**
   ```bash
   git diff --quiet || git commit -am "WIP"
   ```

3. **Use fast-forward merges**
   ```bash
   git merge --ff-only origin/main
   ```

4. **Validate sync readiness**
   ```bash
   # Check for uncommitted changes
   git diff --quiet && echo "Clean" || echo "Uncommitted changes!"
   ```

### 🛡️ **Auto-Resolution Features**

- **Git rerere**: Remember conflict resolutions
- **Auto-fix common cases**: Version bumps, changelog lines
- **Pre-sync validation**: Fail fast before merge conflicts

## Configuration

### 📁 **Report Generation**

Sync reports are written to `contract.report.worktrees.jsonc`:

```jsonc
{
  "feature/example": {
    "branch": "feature/example",
    "is_ahead_of_main": true,
    "has_unmerged_main": false,
    "has_uncommitted": false,
    "pr_synced": true,
    "last_sync": "2024-01-15T10:30:00Z",
    "sync_status": "AheadOfMain"
  }
}
```

### ⚙️ **Git Configuration**

```bash
# Enable rerere for conflict resolution memory
git config --global rerere.enabled true

# Increase rename limit for better conflict detection
git config --global merge.renamelimit 9999
```

## Integration with Hooksmith

### 🔗 **Event-Driven Architecture**

The sync strategy integrates with Hooksmith's event system:

- **Pre-sync events**: Validation and readiness checks
- **Sync events**: Progress tracking and conflict resolution
- **Post-sync events**: Report generation and status updates

### 📈 **Monitoring and Metrics**

- **Sync success rate**: Track successful vs failed syncs
- **Conflict frequency**: Monitor merge conflict patterns
- **Performance metrics**: Sync duration and efficiency

## Advanced Features

### 🤖 **Automated Workflow**

```bash
# Automated sync workflow
cargo xtask worktree sync-strategy --validate --report

# Continuous monitoring
cargo xtask worktree sync-strategy --watchdog --interval 300
```

### 🔄 **CI/CD Integration**

```yaml
# GitHub Actions example
- name: Sync Worktrees
  run: |
    cargo xtask worktree sync-strategy --validate --report
    if [ -f contract.report.worktrees.jsonc ]; then
      echo "Sync report generated"
    fi
```

### 📊 **Dashboard Integration**

The sync strategy integrates with Hooksmith's dashboard:

- **Real-time sync status**: Visual worktree status
- **Conflict alerts**: Immediate notification of issues
- **Sync history**: Track sync patterns over time

## Troubleshooting

### ❌ **Common Issues**

1. **Uncommitted changes blocking sync**
   ```bash
   # Commit changes first
   git commit -am "WIP: Save changes before sync"
   ```

2. **Merge conflicts**
   ```bash
   # Resolve conflicts manually
   git status
   # Edit conflicted files
   git add .
   git commit
   ```

3. **Worktree not found**
   ```bash
   # Check worktree list
   git worktree list
   # Recreate if needed
   git worktree add ../feature-name feature/feature-name
   ```

### 🔧 **Debug Commands**

```bash
# Check sync readiness
cargo xtask worktree sync-strategy --validate

# Generate detailed report
cargo xtask worktree sync-strategy --report

# Force sync (use with caution)
cargo xtask worktree sync-strategy --force
```

## Future Enhancements

### 🚀 **Planned Features**

1. **Smart conflict resolution**: AI-powered conflict resolution
2. **Predictive sync**: Anticipate and prevent conflicts
3. **Multi-repo sync**: Sync across multiple repositories
4. **Advanced reporting**: Detailed analytics and insights

### 📋 **Roadmap**

- [ ] **Phase 1**: Basic sync strategy implementation ✅
- [ ] **Phase 2**: Advanced conflict resolution
- [ ] **Phase 3**: Predictive sync capabilities
- [ ] **Phase 4**: Multi-repo support

## Conclusion

The Worktree Sync Strategy provides a robust, conflict-free approach to managing multiple Git worktrees. By following the 1:1:1:1:1 mapping model and implementing upstream-first sync principles, it ensures a clean, organized development workflow that prevents merge conflicts and maintains code quality.

The strategy integrates seamlessly with Hooksmith's event-driven architecture, providing real-time monitoring, automated conflict resolution, and comprehensive reporting capabilities. 