#!/bin/bash

# Update Worktrees to Main
# Systematically updates all worktrees to be based on current main

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

# Function to update a worktree to main
update_worktree_to_main() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Updating worktree: $branch_name"
    
    # Check if worktree exists
    if [ ! -d "$worktree_path" ]; then
        log_warning "Worktree does not exist: $worktree_path"
        return 1
    fi
    
    cd "$worktree_path"
    
    # Check if already up to date
    local commits_behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
    if [ "$commits_behind" -eq 0 ]; then
        log_info "Worktree $branch_name is already up to date"
        cd - > /dev/null
        return 0
    fi
    
    # Check if branch is merged
    if git branch --merged origin/main | grep -q "$branch_name"; then
        log_info "Branch $branch_name is merged - cleaning up"
        cd - > /dev/null
        git worktree remove "$worktree_path" 2>/dev/null || true
        git branch -D "$branch_name" 2>/dev/null || true
        return 0
    fi
    
    # Try to rebase onto main
    log_info "Attempting to rebase $branch_name onto main"
    if git rebase origin/main; then
        log_success "Successfully rebased $branch_name onto main"
        cd - > /dev/null
        return 0
    else
        log_warning "Rebase failed for $branch_name - creating fresh branch"
        cd - > /dev/null
        
        # Remove old worktree and create fresh one
        git worktree remove "$worktree_path" 2>/dev/null || true
        git branch -D "$branch_name" 2>/dev/null || true
        
        # Create new worktree based on main
        git worktree add "$worktree_path" -b "$branch_name"
        log_success "Created fresh worktree for $branch_name based on main"
        return 0
    fi
}

# Function to process all worktrees
process_all_worktrees() {
    log_header "UPDATING ALL WORKTREES TO MAIN"
    echo ""
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "workdir" | awk '{print $2}')
    
    local updated_count=0
    local total_count=0
    
    for worktree_path in $worktrees; do
        # Skip the main worktree
        if [[ "$worktree_path" == *"/hooksmith" ]] && [[ "$worktree_path" != *"/worktrees/"* ]]; then
            continue
        fi
        
        total_count=$((total_count + 1))
        
        # Get branch name from worktree path
        local branch_name=$(basename "$worktree_path")
        
        if update_worktree_to_main "$worktree_path" "$branch_name"; then
            updated_count=$((updated_count + 1))
        fi
    done
    
    echo ""
    log_success "Updated $updated_count out of $total_count worktrees"
}

# Function to show status
show_status() {
    log_header "WORKTREE STATUS AFTER UPDATE"
    echo ""
    
    if [ -f "./worktree-lifecycle/bin/worktree-lifecycle.sh" ]; then
        ./worktree-lifecycle/bin/worktree-lifecycle.sh status
    else
        git worktree list
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [update|status|help]"
    echo ""
    echo "Commands:"
    echo "  update  - Update all worktrees to be based on current main"
    echo "  status  - Show worktree status"
    echo "  help    - Show this usage information"
    echo ""
    echo "Examples:"
    echo "  $0 update  # Update all worktrees to main"
    echo "  $0 status  # Show current status"
    echo ""
}

# Main execution
main() {
    case "${1:-help}" in
        "update")
            process_all_worktrees
            echo ""
            show_status
            ;;
        "status")
            show_status
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function
main "$@" 
