#!/bin/bash

# Create PRs for Ready Worktrees
# Automatically creates pull requests for worktrees that are ready

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

# Function to check if worktree is ready for PR
is_ready_for_pr() {
    local worktree_path="$1"
    local branch_name="$2"
    
    cd "$worktree_path"
    
    # Check if clean
    local status=$(git status --porcelain)
    if [ -n "$status" ]; then
        cd - > /dev/null
        return 1
    fi
    
    # Check if up to date with main
    local behind_count=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
    if [ "$behind_count" -gt 0 ]; then
        cd - > /dev/null
        return 1
    fi
    
    # Check if has commits ahead of main
    local ahead_count=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
    if [ "$ahead_count" -eq 0 ]; then
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
}

# Function to push branch
push_branch() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Pushing branch $branch_name"
    cd "$worktree_path"
    
    if git push origin "$branch_name"; then
        log_success "Branch pushed successfully"
        cd - > /dev/null
        return 0
    else
        log_warning "Push failed - branch may already be up to date"
        cd - > /dev/null
        return 1
    fi
}

# Function to create PR using GitHub CLI
create_pr_with_gh() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Creating PR for branch $branch_name using GitHub CLI"
    cd "$worktree_path"
    
    # Get commit message for PR title
    local commit_msg=$(git log --oneline -1)
    local pr_title="$(echo "$commit_msg" | cut -d' ' -f2-)"
    
    # Get PR body from commit messages
    local pr_body=$(git log --oneline main..HEAD | head -5 | sed 's/^/- /')
    
    if gh pr create --title "$pr_title" --body "$pr_body" --base main --head "$branch_name"; then
        log_success "PR created successfully"
        cd - > /dev/null
        return 0
    else
        log_warning "Failed to create PR with GitHub CLI"
        cd - > /dev/null
        return 1
    fi
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

# Function to process ready worktree
process_ready_worktree() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_info "Processing ready worktree: $worktree_path (branch: $branch_name)"
    
    # Push branch
    if push_branch "$worktree_path" "$branch_name"; then
        # Try to create PR with GitHub CLI
        if command -v gh >/dev/null 2>&1; then
            if create_pr_with_gh "$worktree_path" "$branch_name"; then
                log_success "PR created successfully for $branch_name"
                return 0
            fi
        fi
        
        # Fallback: generate PR URL
        local pr_url=$(generate_pr_url "$branch_name")
        log_info "PR URL generated: $pr_url"
        log_warning "Please create PR manually using the URL above"
        return 0
    else
        log_error "Failed to push branch $branch_name"
        return 1
    fi
}

# Main execution
main() {
    log_header "CREATE WORKTREE PRs"
    echo ""
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
    
    if [ -z "$worktrees" ]; then
        log_info "No worktrees found"
        return 0
    fi
    
    local ready_worktrees=()
    local processed_count=0
    
    # Find ready worktrees
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
        
        # Check if ready for PR
        if is_ready_for_pr "$worktree_path" "$branch_name"; then
            ready_worktrees+=("$worktree_path|$branch_name")
        fi
    done <<< "$worktrees"
    
    # Process ready worktrees
    if [ ${#ready_worktrees[@]} -eq 0 ]; then
        log_info "No worktrees ready for PR creation"
        return 0
    fi
    
    log_info "Found ${#ready_worktrees[@]} worktree(s) ready for PR creation"
    echo ""
    
    for worktree_info in "${ready_worktrees[@]}"; do
        IFS='|' read -r worktree_path branch_name <<< "$worktree_info"
        
        if process_ready_worktree "$worktree_path" "$branch_name"; then
            processed_count=$((processed_count + 1))
        fi
        
        echo "---"
    done
    
    log_success "Processed $processed_count worktree(s)"
}

# Run main function
main "$@" 
