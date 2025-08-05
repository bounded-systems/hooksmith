#!/bin/bash

# Comprehensive Worktree Status Report
# Shows detailed status of all worktrees and their branches

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

# Function to get worktree status
get_worktree_status() {
    local worktree_path="$1"
    local branch_name="$2"
    
    cd "$worktree_path"
    
    # Get current branch
    local current_branch=$(git branch --show-current)
    
    # Get status
    local status=$(git status --porcelain)
    local is_clean=$([ -z "$status" ] && echo "clean" || echo "dirty")
    
    # Check if rebasing
    local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
    
    # Check if branch exists on origin
    local remote_exists=$(git ls-remote --heads origin "$current_branch" | grep -q "$current_branch" && echo "true" || echo "false")
    
    # Check if merged into main
    local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
    
    # Get commit count ahead/behind main
    local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
    local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
    
    cd - > /dev/null
    
    echo "$current_branch|$is_clean|$is_rebasing|$remote_exists|$is_merged|$ahead_behind|$behind_ahead"
}

# Function to determine worktree state
determine_state() {
    local is_clean="$1"
    local is_rebasing="$2"
    local remote_exists="$3"
    local is_merged="$4"
    local ahead_behind="$5"
    local behind_ahead="$6"
    
    if [ "$is_merged" = "true" ]; then
        echo "MERGED"
    elif [ "$is_rebasing" = "true" ]; then
        echo "CONFLICTED"
    elif [ "$is_clean" = "dirty" ]; then
        echo "DEVELOPING"
    elif [ "$ahead_behind" -gt 0 ] && [ "$behind_ahead" -eq 0 ]; then
        echo "READY"
    elif [ "$behind_ahead" -gt 0 ]; then
        echo "OUTDATED"
    else
        echo "UNKNOWN"
    fi
}

# Function to print worktree status
print_worktree_status() {
    local worktree_path="$1"
    local branch_name="$2"
    local status_info="$3"
    
    IFS='|' read -r current_branch is_clean is_rebasing remote_exists is_merged ahead_behind behind_ahead <<< "$status_info"
    local state=$(determine_state "$is_clean" "$is_rebasing" "$remote_exists" "$is_merged" "$ahead_behind" "$behind_ahead")
    
    echo -e "${CYAN}📁 Worktree:${NC} $worktree_path"
    echo -e "   ${BLUE}Branch:${NC} $current_branch"
    echo -e "   ${BLUE}State:${NC} $state"
    echo -e "   ${BLUE}Status:${NC} $is_clean"
    echo -e "   ${BLUE}Rebasing:${NC} $is_rebasing"
    echo -e "   ${BLUE}Remote:${NC} $remote_exists"
    echo -e "   ${BLUE}Merged:${NC} $is_merged"
    echo -e "   ${BLUE}Commits:${NC} +$ahead_behind -$behind_ahead"
    echo ""
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

# Main execution
main() {
    log_header "WORKTREE STATUS REPORT"
    echo ""
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
    
    if [ -z "$worktrees" ]; then
        log_info "No worktrees found"
        return 0
    fi
    
    local ready_worktrees=()
    local conflicted_worktrees=()
    local merged_worktrees=()
    local developing_worktrees=()
    
    # Process each worktree
    while IFS= read -r worktree_path; do
        if [ -z "$worktree_path" ]; then
            continue
        fi
        
        # Get branch name from worktree path
        local branch_name=$(basename "$worktree_path")
        
        # Get status
        local status_info=$(get_worktree_status "$worktree_path" "$branch_name")
        IFS='|' read -r current_branch is_clean is_rebasing remote_exists is_merged ahead_behind behind_ahead <<< "$status_info"
        local state=$(determine_state "$is_clean" "$is_rebasing" "$remote_exists" "$is_merged" "$ahead_behind" "$behind_ahead")
        
        # Print status
        print_worktree_status "$worktree_path" "$branch_name" "$status_info"
        
        # Categorize worktree
        case "$state" in
            "READY")
                ready_worktrees+=("$current_branch")
                ;;
            "CONFLICTED")
                conflicted_worktrees+=("$current_branch")
                ;;
            "MERGED")
                merged_worktrees+=("$current_branch")
                ;;
            "DEVELOPING")
                developing_worktrees+=("$current_branch")
                ;;
        esac
        
    done <<< "$worktrees"
    
    # Summary
    log_header "SUMMARY"
    echo ""
    
    if [ ${#ready_worktrees[@]} -gt 0 ]; then
        log_success "Ready for PR: ${ready_worktrees[*]}"
        for branch in "${ready_worktrees[@]}"; do
            local pr_url=$(generate_pr_url "$branch")
            echo "   PR URL: $pr_url"
        done
        echo ""
    fi
    
    if [ ${#conflicted_worktrees[@]} -gt 0 ]; then
        log_warning "Conflicted: ${conflicted_worktrees[*]}"
        echo ""
    fi
    
    if [ ${#merged_worktrees[@]} -gt 0 ]; then
        log_info "Merged (ready for cleanup): ${merged_worktrees[*]}"
        echo ""
    fi
    
    if [ ${#developing_worktrees[@]} -gt 0 ]; then
        log_info "Developing: ${developing_worktrees[*]}"
        echo ""
    fi
    
    if [ ${#ready_worktrees[@]} -eq 0 ] && [ ${#conflicted_worktrees[@]} -eq 0 ] && [ ${#merged_worktrees[@]} -eq 0 ] && [ ${#developing_worktrees[@]} -eq 0 ]; then
        log_info "No worktrees to process"
    fi
}

# Run main function
main "$@" 
