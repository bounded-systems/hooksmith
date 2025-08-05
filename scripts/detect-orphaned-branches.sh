#!/bin/bash

# Detect Orphaned Branches
# Finds branches that exist locally but aren't in worktrees (except main)

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
Detect Orphaned Branches

Usage: $0 [options]

Options:
  --create-worktrees    Create worktrees for orphaned branches
  --delete-branches     Delete orphaned branches (use with caution)
  --dry-run            Show what would be done without making changes
  --help               Show this usage information

Examples:
  $0                    # Show orphaned branches
  $0 --dry-run         # Show what would be done
  $0 --create-worktrees # Create worktrees for orphaned branches
  $0 --delete-branches  # Delete orphaned branches

This script will:
1. Find local branches that aren't in worktrees
2. Exclude main branch from orphaned list
3. Provide options to create worktrees or delete branches
4. Show summary of actions taken
EOF
}

# Function to check dependencies
check_dependencies() {
    if ! command -v git &> /dev/null; then
        log_error "Git is required but not installed"
        exit 1
    fi
}

# Function to get worktree branches
get_worktree_branches() {
    local branches=()
    
    while IFS= read -r line; do
        # Extract branch name from worktree list
        local branch_name=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\]/\1/p')
        
        # Skip empty branch names
        if [ -z "$branch_name" ]; then
            continue
        fi
        
        branches+=("$branch_name")
    done < <(git worktree list)
    
    echo "${branches[@]}"
}

# Function to get all local branches
get_local_branches() {
    local branches=()
    
    while IFS= read -r line; do
        # Clean the line and extract branch name
        local clean_line=$(echo "$line" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        
        # Remove asterisk for current branch
        local branch_name=$(echo "$clean_line" | sed 's/^\* //')
        
        # Skip empty branch names and malformed lines
        if [ -z "$branch_name" ] || [[ "$branch_name" =~ ^[[:space:]]*$ ]] || [[ "$branch_name" =~ ^\+ ]]; then
            continue
        fi
        
        branches+=("$branch_name")
    done < <(git branch --list)
    
    echo "${branches[@]}"
}

# Function to find orphaned branches
find_orphaned_branches() {
    local worktree_branches
    read -ra worktree_branches <<< "$(get_worktree_branches)"
    
    local local_branches
    read -ra local_branches <<< "$(get_local_branches)"
    
    local orphaned=()
    
    for branch in "${local_branches[@]}"; do
        # Skip main branch
        if [ "$branch" = "main" ]; then
            continue
        fi
        
        # Check if branch is in worktrees
        local found=false
        for worktree_branch in "${worktree_branches[@]}"; do
            if [ "$branch" = "$worktree_branch" ]; then
                found=true
                break
            fi
        done
        
        if [ "$found" = false ]; then
            orphaned+=("$branch")
        fi
    done
    
    echo "${orphaned[@]}"
}

# Function to create worktree for orphaned branch
create_worktree_for_branch() {
    local branch_name="$1"
    local dry_run="$2"
    local worktree_path="worktrees/${branch_name//\//\/}"
    
    log_info "Processing orphaned branch: $branch_name"
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would create worktree for $branch_name"
        return 0
    fi
    
    # Check if worktree already exists
    if git worktree list | grep -q "$worktree_path"; then
        log_warning "Worktree already exists at: $worktree_path"
        return 1
    fi
    
    # Create the worktree
    log_info "Creating worktree for branch: $branch_name"
    log_info "Worktree path: $worktree_path"
    
    if git worktree add "$worktree_path" "$branch_name"; then
        log_success "Successfully created worktree for branch: $branch_name"
        return 0
    else
        log_error "Failed to create worktree for branch: $branch_name"
        return 1
    fi
}

# Function to delete orphaned branch
delete_orphaned_branch() {
    local branch_name="$1"
    local dry_run="$2"
    
    log_info "Processing orphaned branch for deletion: $branch_name"
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would delete branch $branch_name"
        return 0
    fi
    
    # Check if branch is merged
    if git branch --merged main | grep -q "^[[:space:]]*$branch_name$"; then
        log_info "Branch $branch_name is merged, deleting..."
        if git branch -d "$branch_name"; then
            log_success "Successfully deleted merged branch: $branch_name"
            return 0
        else
            log_error "Failed to delete merged branch: $branch_name"
            return 1
        fi
    else
        log_warning "Branch $branch_name is not merged, use -D to force delete"
        if git branch -D "$branch_name"; then
            log_success "Successfully force deleted branch: $branch_name"
            return 0
        else
            log_error "Failed to force delete branch: $branch_name"
            return 1
        fi
    fi
}

# Function to handle orphaned branches
handle_orphaned_branches() {
    local create_worktrees="$1"
    local delete_branches="$2"
    local dry_run="$3"
    
    log_header "DETECTING ORPHANED BRANCHES"
    
    # Find orphaned branches
    local orphaned
    read -ra orphaned <<< "$(find_orphaned_branches)"
    
    if [ ${#orphaned[@]} -eq 0 ] || [ -z "${orphaned[*]}" ]; then
        log_success "No orphaned branches found! All branches (except main) are properly managed as worktrees."
        return 0
    fi
    
    log_warning "Found ${#orphaned[@]} orphaned branches:"
    for branch in "${orphaned[@]}"; do
        log_warning "  - $branch"
    done
    
    if [ "$create_worktrees" = true ]; then
        log_header "CREATING WORKTREES FOR ORPHANED BRANCHES"
        
        local created_count=0
        local failed_count=0
        
        for branch in "${orphaned[@]}"; do
            if create_worktree_for_branch "$branch" "$dry_run"; then
                ((created_count++))
            else
                ((failed_count++))
            fi
        done
        
        log_header "WORKTREE CREATION SUMMARY"
        log_success "Created: $created_count"
        log_error "Failed: $failed_count"
        
    elif [ "$delete_branches" = true ]; then
        log_header "DELETING ORPHANED BRANCHES"
        
        local deleted_count=0
        local failed_count=0
        
        for branch in "${orphaned[@]}"; do
            if delete_orphaned_branch "$branch" "$dry_run"; then
                ((deleted_count++))
            else
                ((failed_count++))
            fi
        done
        
        log_header "BRANCH DELETION SUMMARY"
        log_success "Deleted: $deleted_count"
        log_error "Failed: $failed_count"
        
    else
        log_header "ORPHANED BRANCHES DETECTED"
        log_info "Use --create-worktrees to create worktrees for these branches"
        log_info "Use --delete-branches to delete these branches (use with caution)"
        log_info "Use --dry-run to see what would be done"
    fi
}

# Main execution
main() {
    # Parse command line arguments
    local create_worktrees=false
    local delete_branches=false
    local dry_run=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --create-worktrees)
                create_worktrees=true
                shift
                ;;
            --delete-branches)
                delete_branches=true
                shift
                ;;
            --dry-run)
                dry_run=true
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
    
    # Handle orphaned branches
    handle_orphaned_branches "$create_worktrees" "$delete_branches" "$dry_run"
}

# Run main function
main "$@" 