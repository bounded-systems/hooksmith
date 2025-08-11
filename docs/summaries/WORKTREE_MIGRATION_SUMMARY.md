# Worktree Migration to .wt Directory

## Overview

This document summarizes the changes made to ensure all worktrees are created in the `.wt` directory within the repository, rather than in various external locations.

## Problem

Previously, worktrees were being created in inconsistent locations:
- Some in `worktrees/` directory
- Some in external directories outside the repository
- Some in the main repository directory

## Solution

### 1. Configuration Updates

Updated the worktree configuration files to use `.wt` as the base directory:

**`.worktree-config.jsonc`:**
- Changed `"worktree_base": "worktrees"` to `"worktree_base": ".wt"`
- Updated `existing_worktrees` paths to use `.wt/` prefix
- Updated `branch_patterns` templates to use `.wt/{branch}`

**`.worktree-config.json`:**
- Changed `"worktree_base": "worktrees"` to `"worktree_base": ".wt"`

### 2. Script Updates

Updated all worktree management scripts to use `.wt` directory:

**Shell Scripts:**
- `scripts/sync-all-remote-branches.sh` - Updated worktree paths
- `scripts/comprehensive-worktree-workflow.sh` - Updated demo worktree creation
- `scripts/detect-orphaned-branches.sh` - Updated worktree creation paths

**Rust Code:**
- `crates/xtask/src/worktree.rs` - Updated default base directory to `.wt/`
- `src/main.rs` - Updated default base directory to `.wt`

### 3. Migration Script

Created `scripts/migrate-worktrees-to-wt.sh` to:
- Move existing worktrees to `.wt` directory
- Handle external worktrees with placeholders
- Clean up old worktree directories

### 4. External Worktree Handling

For the external worktree at `/Users/bobby/dev/repos/feat-shell-migration-v2`:
- Created placeholder in `.wt/feat-shell-migration-v3/`
- Added documentation for manual migration
- Provided instructions for moving the worktree

## Current Status

### Worktrees in Repository
- ✅ `.wt/ensure-worktree-folder-structure` - Already in correct location

### External Worktrees
- ⚠️ `/Users/bobby/dev/repos/feat-shell-migration-v2` - External worktree (branch: `feat-shell-migration-v3`)
  - Placeholder created in `.wt/feat-shell-migration-v3/`
  - Manual migration required

### Old Worktree Directories
- `worktree-cleanup-remote-branches` - Can be removed if no longer needed
- `worktree-feat-systems-diagrams` - Can be removed if no longer needed
- `worktree-lifecycle` - Can be removed if no longer needed

## Next Steps

1. **Manual Migration of External Worktree:**
   ```bash
   # Remove external worktree
   cd /Users/bobby/dev/repos/feat-shell-migration-v2
   git worktree remove .
   
   # Create new worktree in .wt directory
   git worktree add .wt/feat-shell-migration-v3 feat-shell-migration-v3
   
   # Remove placeholder
   rm -rf .wt/feat-shell-migration-v3/README.md
   ```

2. **Clean Up Old Directories:**
   ```bash
   # Remove old worktree directories if no longer needed
   rm -rf worktree-cleanup-remote-branches
   rm -rf worktree-feat-systems-diagrams
   rm -rf worktree-lifecycle
   ```

3. **Verify Configuration:**
   ```bash
   # Test worktree creation
   cargo xtask worktree create --branch test-migration
   ```

## Benefits

- **Consistent Structure:** All worktrees now use `.wt/` directory
- **Better Organization:** Worktrees are contained within the repository
- **Easier Management:** Standardized paths for all worktree operations
- **Improved Scripts:** All automation scripts now use consistent paths

## Configuration Files Updated

- `.worktree-config.jsonc`
- `.worktree-config.json`
- `scripts/sync-all-remote-branches.sh`
- `scripts/comprehensive-worktree-workflow.sh`
- `scripts/detect-orphaned-branches.sh`
- `crates/xtask/src/worktree.rs`
- `src/main.rs`

## Migration Script

The migration script `scripts/migrate-worktrees-to-wt.sh` can be used to:
- Automatically move worktrees to `.wt` directory
- Create placeholders for external worktrees
- Clean up old worktree directories
- Provide migration status and instructions 
