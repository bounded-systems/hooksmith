#!/bin/bash

# Worktree State Machine
# Manages the complete lifecycle of worktrees with state transitions

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# State machine states
STATES_CREATED="Worktree created"
STATES_DEVELOPING="Worktree has uncommitted changes"
STATES_CONFLICTED="Worktree has rebase conflicts"
STATES_RESOLVING="Resolving conflicts"
STATES_RESOLVED="Conflicts resolved"
STATES_READY="Worktree ready for PR"
STATES_PR_CREATED="PR created"
STATES_MERGED="PR merged"
STATES_CLEANUP="Cleaning up worktree"
STATES_REMOVED="Worktree removed"

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

# Function to get worktree state
get_worktree_state() {
    local worktree_path="$1"
    local branch_name="$2"
    
    cd "$worktree_path"
    
    # Get current branch
    local current_branch=$(git branch --show-current)
    
    # Get status
    local status=$(git status --porcelain)
    local is_clean=$([ -z "$status" ] && echo "true" || echo "false")
    
    # Check if rebasing
    local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
    
    # Check if merged into main
    local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
    
    # Get commit count ahead/behind main
    local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
    local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
    
    cd - > /dev/null
    
    # Determine state
    if [ "$is_merged" = "true" ]; then
        echo "MERGED"
    elif [ "$is_rebasing" = "true" ]; then
        echo "CONFLICTED"
    elif [ "$is_clean" = "false" ]; then
        echo "DEVELOPING"
    elif [ "$ahead_behind" -gt 0 ] && [ "$behind_ahead" -eq 0 ]; then
        echo "READY"
    elif [ "$behind_ahead" -gt 0 ]; then
        echo "RESOLVING"
    else
        echo "CREATED"
    fi
}

# Function to transition worktree state
transition_state() {
    local worktree_path="$1"
    local branch_name="$2"
    local current_state="$3"
    local target_state="$4"
    
    log_info "Transitioning $branch_name: $current_state → $target_state"
    
    case "$target_state" in
        "RESOLVING")
            cd "$worktree_path"
            if git rebase main; then
                log_success "Rebase successful"
                cd - > /dev/null
                return 0
            else
                log_warning "Rebase failed - aborting"
                git rebase --abort
                cd - > /dev/null
                return 1
            fi
            ;;
        "READY")
            cd "$worktree_path"
            if git push origin "$branch_name"; then
                log_success "Branch pushed successfully"
                cd - > /dev/null
                return 0
            else
                log_warning "Push failed"
                cd - > /dev/null
                return 1
            fi
            ;;
        "PR_CREATED")
            # Create PR using GitHub CLI or generate URL
            local pr_url=$(generate_pr_url "$branch_name")
            log_info "PR URL: $pr_url"
            return 0
            ;;
        "CLEANUP")
            cd "$worktree_path"
            cd - > /dev/null
            git worktree remove "$worktree_path" --force
            git push origin --delete "$branch_name" 2>/dev/null || true
            log_success "Worktree cleaned up"
            return 0
            ;;
        *)
            log_warning "Unknown target state: $target_state"
            return 1
            ;;
    esac
}

# Function to generate PR URL
generate_pr_url() {
    local branch_name="$1"
    local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
    
    if [[ "$repo_url" == *"github.com"* ]]; then
        echo "$repo_url/compare/main...$branch_name"
    else
        echo "Unknown repository URL"
    fi
}

# Function to process worktree through state machine
process_worktree() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Processing worktree: $worktree_path (branch: $branch_name)"
    
    # Get current state
    local current_state=$(get_worktree_state "$worktree_path" "$branch_name")
    log_info "Current state: $current_state"
    
    # Determine next state
    local next_state=""
    case "$current_state" in
        "CREATED")
            next_state="DEVELOPING"
            ;;
        "DEVELOPING")
            # Check if ready to move to next state
            cd "$worktree_path"
            if [ -z "$(git status --porcelain)" ]; then
                next_state="RESOLVING"
            fi
            cd - > /dev/null
            ;;
        "CONFLICTED")
            next_state="RESOLVING"
            ;;
        "RESOLVING")
            next_state="READY"
            ;;
        "READY")
            next_state="PR_CREATED"
            ;;
        "PR_CREATED")
            next_state="MERGED"
            ;;
        "MERGED")
            next_state="CLEANUP"
            ;;
        "CLEANUP")
            next_state="REMOVED"
            ;;
        *)
            log_warning "Unknown state: $current_state"
            return 1
            ;;
    esac
    
    if [ -n "$next_state" ]; then
        if transition_state "$worktree_path" "$branch_name" "$current_state" "$next_state"; then
            log_success "Successfully transitioned to $next_state"
            return 0
        else
            log_warning "Failed to transition to $next_state"
            return 1
        fi
    else
        log_info "No transition needed"
        return 0
    fi
}

# Function to print state machine diagram
print_diagram() {
    log_header "WORKTREE STATE MACHINE DIAGRAM"
    echo ""
    echo "CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED"
    echo "    ↓         ↓"
    echo "CONFLICTED → RESOLVING"
    echo ""
    echo "State Descriptions:"
    echo "  CREATED: $STATES_CREATED"
    echo "  DEVELOPING: $STATES_DEVELOPING"
    echo "  CONFLICTED: $STATES_CONFLICTED"
    echo "  RESOLVING: $STATES_RESOLVING"
    echo "  RESOLVED: $STATES_RESOLVED"
    echo "  READY: $STATES_READY"
    echo "  PR_CREATED: $STATES_PR_CREATED"
    echo "  MERGED: $STATES_MERGED"
    echo "  CLEANUP: $STATES_CLEANUP"
    echo "  REMOVED: $STATES_REMOVED"
}

# Function to process all worktrees
process_all_worktrees() {
    log_header "PROCESSING ALL WORKTREES"
    echo ""
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
    
    if [ -z "$worktrees" ]; then
        log_info "No worktrees found"
        return 0
    fi
    
    local processed_count=0
    local success_count=0
    
    # Process each worktree
    while IFS= read -r worktree_path; do
        if [ -z "$worktree_path" ]; then
            continue
        fi
        
        # Get branch name from worktree path
        local branch_name=$(basename "$worktree_path")
        
        # Skip main worktree
        if [ "$branch_name" = "hooksmith" ]; then
            continue
        fi
        
        processed_count=$((processed_count + 1))
        
        if process_worktree "$worktree_path" "$branch_name"; then
            success_count=$((success_count + 1))
        fi
        
        echo "---"
    done <<< "$worktrees"
    
    log_success "Processed $processed_count worktree(s), $success_count successful"
}

# Main execution
main() {
    case "${1:-process}" in
        "diagram")
            print_diagram
            ;;
        "process")
            process_all_worktrees
            ;;
        "status")
            ./scripts/worktree-status-report.sh
            ;;
        *)
            echo "Usage: $0 [diagram|process|status]"
            echo "  diagram: Show state machine diagram"
            echo "  process: Process all worktrees through state machine"
            echo "  status: Show current worktree status"
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 
