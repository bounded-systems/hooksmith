#!/bin/bash

# Sync All Remote Branches to Worktrees
# Creates worktrees for all remote branches that don't already exist locally

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
Sync All Remote Branches to Worktrees

Usage: $0 [options]

Options:
  --dry-run           Show what would be done without making changes
  --skip-main         Skip creating worktree for main branch
  --force             Force recreation of existing worktrees
  --help              Show this usage information

Examples:
  $0                    # Create worktrees for all remote branches
  $0 --dry-run         # Show what would be created
  $0 --skip-main       # Create worktrees except for main
  $0 --force           # Recreate existing worktrees

This script will:
1. Fetch all remote branches
2. Create worktrees for branches that don't exist locally
3. Skip main branch unless --skip-main is not specified
4. Show summary of created worktrees
EOF
}

# Function to check dependencies
check_dependencies() {
    if ! command -v git &> /dev/null; then
        log_error "Git is required but not installed"
        exit 1
    fi
}

# Function to fetch all remote branches
fetch_remote_branches() {
    log_info "Fetching all remote branches..."
    git fetch --all --prune
    log_success "Remote branches fetched"
}

# Function to get remote branches (excluding main if requested)
get_remote_branches() {
    local skip_main="$1"
    local branches=()
    
    # Get clean list of remote branches
    while IFS= read -r line; do
        # Skip empty lines
        if [ -z "$line" ]; then
            continue
        fi
        
        # Clean the line and extract branch name
        local clean_line=$(echo "$line" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        
        # Skip HEAD -> main reference
        if [[ "$clean_line" =~ origin/HEAD ]]; then
            continue
        fi
        
        # Extract branch name from origin/branch-name format
        local branch_name="${clean_line#origin/}"
        
        # Skip main if requested
        if [ "$skip_main" = true ] && [ "$branch_name" = "main" ]; then
            continue
        fi
        
        # Skip empty branch names
        if [ -z "$branch_name" ]; then
            continue
        fi
        
        branches+=("$branch_name")
    done < <(git branch -r | grep "origin/" | sort)
    
    echo "${branches[@]}"
}

# Function to check if worktree exists
worktree_exists() {
    local branch_name="$1"
    local worktree_path="worktrees/${branch_name//\//\/}"
    
    if git worktree list | grep -q "$worktree_path"; then
        return 0  # Worktree exists
    else
        return 1  # Worktree doesn't exist
    fi
}

# Function to create worktree for a branch
create_worktree() {
    local branch_name="$1"
    local force="$2"
    local worktree_path="worktrees/${branch_name//\//\/}"
    
    log_info "Processing branch: $branch_name"
    
    # Check if worktree already exists
    if worktree_exists "$branch_name"; then
        if [ "$force" = true ]; then
            log_warning "Removing existing worktree for $branch_name"
            git worktree remove "$worktree_path" 2>/dev/null || true
            git branch -D "$branch_name" 2>/dev/null || true
        else
            log_warning "Worktree already exists for $branch_name, skipping"
            return 1
        fi
    fi
    
    # Create the worktree
    log_info "Creating worktree for branch: $branch_name"
    log_info "Worktree path: $worktree_path"
    
    # Check if local branch exists
    if git show-ref --verify --quiet refs/heads/"$branch_name"; then
        # Local branch exists, create worktree without -b flag
        if git worktree add "$worktree_path" "$branch_name"; then
            log_success "Successfully created worktree for existing branch: $branch_name"
            return 0
        else
            log_error "Failed to create worktree for existing branch: $branch_name"
            return 1
        fi
    else
        # Local branch doesn't exist, create new branch from remote
        if git worktree add "$worktree_path" -b "$branch_name" "origin/$branch_name"; then
            log_success "Successfully created worktree for new branch: $branch_name"
            return 0
        else
            log_error "Failed to create worktree for new branch: $branch_name"
            return 1
        fi
    fi
}

# Function to sync all remote branches
sync_all_branches() {
    local dry_run="$1"
    local skip_main="$2"
    local force="$3"
    
    log_header "SYNCING ALL REMOTE BRANCHES"
    
    # Fetch remote branches
    fetch_remote_branches
    
    # Get list of remote branches
    local branches
    read -ra branches <<< "$(get_remote_branches "$skip_main")"
    
    log_info "Found ${#branches[@]} remote branches to process"
    
    local created_count=0
    local skipped_count=0
    local failed_count=0
    
    # Process each branch
    for branch in "${branches[@]}"; do
        if [ "$dry_run" = true ]; then
            if worktree_exists "$branch"; then
                log_info "DRY RUN: Would skip existing worktree for $branch"
                ((skipped_count++))
            else
                log_info "DRY RUN: Would create worktree for $branch"
                ((created_count++))
            fi
        else
            if create_worktree "$branch" "$force"; then
                ((created_count++))
            else
                ((skipped_count++))
            fi
        fi
    done
    
    # Show summary
    log_header "SYNC SUMMARY"
    log_info "Branches processed: ${#branches[@]}"
    log_success "Created: $created_count"
    log_warning "Skipped: $skipped_count"
    log_error "Failed: $failed_count"
    
    if [ "$dry_run" = false ]; then
        log_info "Use './worktree-lifecycle/bin/worktree-lifecycle.sh status' to see all worktrees"
    fi
}

# Main execution
main() {
    # Parse command line arguments
    local dry_run=false
    local skip_main=false
    local force=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --skip-main)
                skip_main=true
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
    
    # Run the sync
    sync_all_branches "$dry_run" "$skip_main" "$force"
}

# Run main function
main "$@" 