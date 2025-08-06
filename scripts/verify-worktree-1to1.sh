#!/bin/bash

# Verify Worktree 1:1 Sync with Remote Branches
# Checks if worktrees match remote branches exactly

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

log_header() {
    echo -e "${PURPLE}=== $1 ===${NC}"
}

# Function to get remote branches (excluding main)
get_remote_branches() {
    git branch -r | grep "origin/" | grep -v "origin/main" | grep -v "origin/HEAD" | sed 's/origin\///' | sort
}

# Function to get worktree branches
get_worktree_branches() {
    git worktree list | grep -v "main" | sed -n 's/.*\[\([^]]*\)\]/\1/p' | sort
}

# Function to check if worktree exists for a branch
worktree_exists() {
    local branch_name="$1"
    git worktree list | grep -q "\[$branch_name\]"
}

# Function to check if remote branch exists
remote_branch_exists() {
    local branch_name="$1"
    git ls-remote --heads origin "$branch_name" | grep -q "$branch_name"
}

# Function to verify 1:1 mapping
verify_worktree_sync() {
    log_header "VERIFYING WORKTREE 1:1 SYNC"
    
    # Fetch latest remote branches
    log_info "Fetching remote branches..."
    git fetch --all --prune > /dev/null 2>&1
    
    # Get current state
    local remote_branches=($(get_remote_branches))
    local worktree_branches=($(get_worktree_branches))
    
    log_info "Remote branches (${#remote_branches[@]}): ${remote_branches[*]}"
    log_info "Worktree branches (${#worktree_branches[@]}): ${worktree_branches[*]}"
    
    # Find missing worktrees (remote branches without worktrees)
    local missing_worktrees=()
    for branch in "${remote_branches[@]}"; do
        if ! worktree_exists "$branch"; then
            missing_worktrees+=("$branch")
        fi
    done
    
    # Find orphaned worktrees (worktrees without remote branches)
    local orphaned_worktrees=()
    for branch in "${worktree_branches[@]}"; do
        if ! remote_branch_exists "$branch"; then
            orphaned_worktrees+=("$branch")
        fi
    done
    
    # Report results
    echo ""
    log_header "VERIFICATION RESULTS"
    
    if [ ${#missing_worktrees[@]} -eq 0 ] && [ ${#orphaned_worktrees[@]} -eq 0 ]; then
        log_success "✅ PERFECT SYNC: All worktrees have corresponding remote branches"
        log_success "✅ All remote branches have corresponding worktrees"
        return 0
    else
        if [ ${#missing_worktrees[@]} -gt 0 ]; then
            log_warning "⚠️  MISSING WORKTREES (${#missing_worktrees[@]}): ${missing_worktrees[*]}"
        fi
        
        if [ ${#orphaned_worktrees[@]} -gt 0 ]; then
            log_warning "⚠️  ORPHANED WORKTREES (${#orphaned_worktrees[@]}): ${orphaned_worktrees[*]}"
        fi
        
        log_info "💡 Run './scripts/simple-worktree-sync.sh' to fix the sync"
        return 1
    fi
}

# Function to show current status
show_status() {
    log_header "CURRENT STATUS"
    
    echo "Worktrees:"
    git worktree list
    
    echo ""
    echo "Remote branches:"
    git branch -r | grep "origin/" | grep -v "origin/main" | grep -v "origin/HEAD" | sort
}

# Main execution
main() {
    case "${1:-verify}" in
        "verify")
            verify_worktree_sync
            ;;
        "status")
            show_status
            ;;
        "help"|"--help"|"-h")
            echo "Usage: $0 [verify|status|help]"
            echo ""
            echo "Commands:"
            echo "  verify  - Check if worktrees are in 1:1 sync with remote branches (default)"
            echo "  status  - Show current worktree and remote branch status"
            echo "  help    - Show this help message"
            ;;
        *)
            log_error "Unknown command: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 