#!/bin/bash

# Update All Worktrees to Main
# Updates all worktrees to be based on origin/main and pushes them

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
Update All Worktrees to Main

Usage: $0 [options]

Options:
  --dry-run           Show what would be done without making changes
  --create-prs        Create PRs for updated worktrees
  --force             Force push even if conflicts
  --help              Show this usage information

Examples:
  $0                    # Update all worktrees to origin/main
  $0 --dry-run         # Show what would be updated
  $0 --create-prs      # Update and create PRs
  $0 --force           # Force update even with conflicts

This script will:
1. Update all worktrees to be based on origin/main
2. Push updated branches to remote
3. Create PRs if --create-prs is specified
4. Skip main branch (stays in base)
EOF
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI (gh) not found. PR creation will be limited."
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Function to get worktree branches (excluding main)
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

# Function to update worktree to main
update_worktree_to_main() {
    local worktree_path="$1"
    local branch_name="$2"
    local dry_run="$3"
    local force="$4"
    
    log_info "Processing worktree: $branch_name"
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would update $branch_name to origin/main"
        return 0
    fi
    
    # Change to worktree directory
    cd "$worktree_path" || {
        log_error "Failed to change to worktree directory: $worktree_path"
        return 1
    }
    
    # Fetch latest origin/main
    git fetch origin main
    
    # Check if we're behind origin/main
    local behind_count
    behind_count=$(git rev-list HEAD..origin/main --count)
    
    if [ "$behind_count" -eq 0 ]; then
        log_info "Worktree $branch_name is already up to date with origin/main"
        cd - > /dev/null
        return 0
    fi
    
    log_info "Worktree $branch_name is $behind_count commits behind origin/main"
    
    # Reset to origin/main
    if git reset --hard origin/main; then
        log_success "Successfully updated $branch_name to origin/main"
    else
        log_error "Failed to update $branch_name to origin/main"
        cd - > /dev/null
        return 1
    fi
    
    # Push to remote
    if git push --force-with-lease origin "$branch_name"; then
        log_success "Successfully pushed $branch_name to remote"
    else
        log_warning "Failed to push $branch_name to remote"
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
}

# Function to create PR for worktree
create_pr_for_worktree() {
    local branch_name="$1"
    local dry_run="$2"
    
    log_info "Creating PR for branch: $branch_name"
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would create PR for $branch_name"
        return 0
    fi
    
    # Check if PR already exists
    if gh pr list --head "$branch_name" --json number --jq 'length' 2>/dev/null | grep -q -v "^0$"; then
        log_warning "PR already exists for $branch_name"
        return 0
    fi
    
    # Create PR
    if gh pr create --title "feat: Update $branch_name to latest main" --body "Updated $branch_name to be based on the latest origin/main

## Changes
- Updated branch to latest origin/main
- Ensured compatibility with current main branch
- Ready for review and merge"; then
        log_success "Successfully created PR for $branch_name"
        return 0
    else
        log_error "Failed to create PR for $branch_name"
        return 1
    fi
}

# Function to update all worktrees
update_all_worktrees() {
    local dry_run="$1"
    local create_prs="$2"
    local force="$3"
    
    log_header "UPDATING ALL WORKTREES TO MAIN"
    
    # Get list of worktree branches
    local branches
    read -ra branches <<< "$(get_worktree_branches)"
    
    log_info "Found ${#branches[@]} worktrees to update"
    
    local updated_count=0
    local failed_count=0
    local pr_count=0
    
    # Process each worktree
    for branch in "${branches[@]}"; do
        # Get worktree path
        local worktree_path
        worktree_path=$(git worktree list | grep "\[$branch\]" | awk '{print $1}')
        
        if [ -z "$worktree_path" ]; then
            log_error "Could not find worktree path for branch: $branch"
            ((failed_count++))
            continue
        fi
        
        if update_worktree_to_main "$worktree_path" "$branch" "$dry_run" "$force"; then
            ((updated_count++))
            
            # Create PR if requested
            if [ "$create_prs" = true ]; then
                if create_pr_for_worktree "$branch" "$dry_run"; then
                    ((pr_count++))
                fi
            fi
        else
            ((failed_count++))
        fi
    done
    
    # Show summary
    log_header "UPDATE SUMMARY"
    log_info "Worktrees processed: ${#branches[@]}"
    log_success "Updated: $updated_count"
    log_error "Failed: $failed_count"
    
    if [ "$create_prs" = true ]; then
        log_success "PRs created: $pr_count"
    fi
    
    if [ "$dry_run" = false ]; then
        log_info "Use './worktree-lifecycle/bin/worktree-lifecycle.sh status' to see updated worktree status"
    fi
}

# Main execution
main() {
    # Parse command line arguments
    local dry_run=false
    local create_prs=false
    local force=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --create-prs)
                create_prs=true
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
    
    # Update all worktrees
    update_all_worktrees "$dry_run" "$create_prs" "$force"
}

# Run main function
main "$@" 