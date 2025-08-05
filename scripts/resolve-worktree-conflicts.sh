#!/bin/bash

# Comprehensive Worktree Conflict Resolution Script
# This script handles worktree conflicts, rebases, and lifecycle management

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if we're in a rebase state
is_rebasing() {
    git status --porcelain | grep -q "^UU\|^AA\|^DD" || git status | grep -q "rebase in progress"
}

# Function to safely abort rebase
abort_rebase() {
    local worktree_path="$1"
    log_info "Aborting rebase in $worktree_path"
    cd "$worktree_path"
    
    if is_rebasing; then
        git rebase --abort
        log_success "Rebase aborted successfully"
    else
        log_info "No rebase in progress"
    fi
    
    cd - > /dev/null
}

# Function to stash changes
stash_changes() {
    local worktree_path="$1"
    log_info "Stashing changes in $worktree_path"
    cd "$worktree_path"
    
    if ! git diff --quiet; then
        git stash push -m "Auto-stash during conflict resolution $(date)"
        log_success "Changes stashed"
    else
        log_info "No changes to stash"
    fi
    
    cd - > /dev/null
}

# Function to resolve conflicts in a worktree
resolve_worktree_conflicts() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Processing worktree: $worktree_path (branch: $branch_name)"
    
    if [ ! -d "$worktree_path" ]; then
        log_error "Worktree directory does not exist: $worktree_path"
        return 1
    fi
    
    cd "$worktree_path"
    
    # Check current status
    local status=$(git status --porcelain)
    local is_rebase_state=$(is_rebasing && echo "true" || echo "false")
    
    log_info "Current status: $(git status --short)"
    
    if [ "$is_rebase_state" = "true" ]; then
        log_warning "Rebase in progress - aborting to preserve state"
        git rebase --abort
        log_success "Rebase aborted"
    fi
    
    # Stash any uncommitted changes
    if ! git diff --quiet; then
        stash_changes "$worktree_path"
    fi
    
    # Try to rebase onto main
    log_info "Attempting to rebase onto main"
    if git rebase main; then
        log_success "Rebase successful"
    else
        log_warning "Rebase failed - preserving worktree state"
        git rebase --abort
    fi
    
    cd - > /dev/null
}

# Function to push worktree branch
push_worktree_branch() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Pushing branch $branch_name"
    cd "$worktree_path"
    
    if git push origin "$branch_name"; then
        log_success "Branch pushed successfully"
    else
        log_warning "Push failed - branch may already be up to date"
    fi
    
    cd - > /dev/null
}

# Function to clean up merged worktrees
cleanup_merged_worktree() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Checking if worktree $branch_name is merged"
    cd "$worktree_path"
    
    # Check if branch is merged into main
    if git branch --merged main | grep -q "$branch_name"; then
        log_info "Branch $branch_name is merged - cleaning up"
        
        # Remove worktree
        cd - > /dev/null
        git worktree remove "$worktree_path" --force
        
        # Delete branch from origin
        git push origin --delete "$branch_name" 2>/dev/null || true
        
        log_success "Merged worktree cleaned up"
    else
        log_info "Branch $branch_name is not merged - keeping worktree"
    fi
    
    cd - > /dev/null
}

# Main execution
main() {
    log_info "Starting comprehensive worktree conflict resolution"
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
    
    if [ -z "$worktrees" ]; then
        log_info "No worktrees found"
        return 0
    fi
    
    # Process each worktree
    while IFS= read -r worktree_path; do
        if [ -z "$worktree_path" ]; then
            continue
        fi
        
        # Get branch name from worktree path
        local branch_name=$(basename "$worktree_path")
        
        log_info "Processing worktree: $worktree_path"
        
        # Resolve conflicts
        resolve_worktree_conflicts "$worktree_path" "$branch_name"
        
        # Push branch
        push_worktree_branch "$worktree_path" "$branch_name"
        
        # Check if merged and cleanup if needed
        cleanup_merged_worktree "$worktree_path" "$branch_name"
        
        echo "---"
    done <<< "$worktrees"
    
    log_success "Worktree conflict resolution completed"
}

# Run main function
main "$@" 
