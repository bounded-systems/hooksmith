#!/bin/bash

# Auto Merge All PRs
# Automatically merges all PRs for worktrees using gh pr merge --delete-branch

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

# Function to show usage
show_usage() {
    cat << EOF
Auto Merge All PRs

Usage: $0 [options]

Options:
  --dry-run           Show what would be done without making changes
  --skip-main         Skip main branch (always skipped by default)
  --force             Force merge even if checks are failing
  --help              Show this usage information

Examples:
  $0                    # Merge all PRs for worktrees
  $0 --dry-run         # Show what would be merged
  $0 --force           # Force merge even with failing checks

This script will:
1. Find all worktrees with open PRs
2. Merge them using gh pr merge --delete-branch
3. Skip main branch automatically
4. Show summary of merged PRs
EOF
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) is required but not installed"
        exit 1
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Function to get worktree branches
get_worktree_branches() {
    local branches=()
    
    while IFS= read -r line; do
        # Extract branch name from worktree list
        local branch_name=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\]/\1/p')
        
        # Skip main branch
        if [ "$branch_name" = "main" ]; then
            continue
        fi
        
        # Skip empty branch names
        if [ -z "$branch_name" ]; then
            continue
        fi
        
        branches+=("$branch_name")
    done < <(git worktree list)
    
    echo "${branches[@]}"
}

# Function to check if branch has open PR
branch_has_open_pr() {
    local branch_name="$1"
    
    # Check if branch has an open PR
    if gh pr list --head "$branch_name" --json number --jq 'length' 2>/dev/null | grep -q -v "^0$"; then
        return 0  # Has open PR
    else
        return 1  # No open PR
    fi
}

# Function to get PR number for branch
get_pr_number() {
    local branch_name="$1"
    
    gh pr list --head "$branch_name" --json number --jq '.[0].number' 2>/dev/null
}

# Function to merge PR
merge_pr() {
    local pr_number="$1"
    local branch_name="$2"
    local force="$3"
    
    log_info "Merging PR #$pr_number for branch: $branch_name"
    
    local merge_args="--delete-branch"
    if [ "$force" = true ]; then
        merge_args="$merge_args --force"
    fi
    
    if gh pr merge "$pr_number" $merge_args; then
        log_success "Successfully merged PR #$pr_number for branch: $branch_name"
        return 0
    else
        log_error "Failed to merge PR #$pr_number for branch: $branch_name"
        return 1
    fi
}

# Function to auto merge all PRs
auto_merge_all_prs() {
    local dry_run="$1"
    local force="$2"
    
    log_header "AUTO MERGING ALL PRS"
    
    # Get list of worktree branches
    local branches
    read -ra branches <<< "$(get_worktree_branches)"
    
    log_info "Found ${#branches[@]} worktree branches to check"
    
    local merged_count=0
    local skipped_count=0
    local failed_count=0
    
    # Process each branch
    for branch in "${branches[@]}"; do
        log_info "Checking branch: $branch"
        
        if branch_has_open_pr "$branch"; then
            local pr_number=$(get_pr_number "$branch")
            
            if [ "$dry_run" = true ]; then
                log_info "DRY RUN: Would merge PR #$pr_number for branch: $branch"
                ((merged_count++))
            else
                if merge_pr "$pr_number" "$branch" "$force"; then
                    ((merged_count++))
                else
                    ((failed_count++))
                fi
            fi
        else
            log_info "No open PR found for branch: $branch"
            ((skipped_count++))
        fi
    done
    
    # Show summary
    log_header "AUTO MERGE SUMMARY"
    log_info "Branches checked: ${#branches[@]}"
    log_success "Merged: $merged_count"
    log_warning "Skipped: $skipped_count"
    log_error "Failed: $failed_count"
    
    if [ "$dry_run" = false ] && [ $merged_count -gt 0 ]; then
        log_info "Use './worktree-lifecycle/bin/worktree-lifecycle.sh cleanup' to clean up merged worktrees"
    fi
}

# Main execution
main() {
    # Parse command line arguments
    local dry_run=false
    local force=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Check dependencies
    check_dependencies
    
    # Run the auto merge
    auto_merge_all_prs "$dry_run" "$force"
}

# Run main function
main "$@" 