#!/bin/bash

# Migrate Worktrees to .wt Directory
# This script moves existing worktrees to the .wt directory structure

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

# Function to check if a directory is a git worktree
is_worktree() {
    local dir="$1"
    if [ -d "$dir" ] && [ -f "$dir/.git" ]; then
        return 0
    fi
    return 1
}

# Function to get worktree branch name
get_worktree_branch() {
    local worktree_path="$1"
    local branch_name=""
    
    # Try to get branch from git worktree list
    branch_name=$(git worktree list --porcelain | grep "^worktree $worktree_path" -A 1 | grep "^branch" | cut -d' ' -f2 | sed 's|refs/heads/||')
    
    if [ -n "$branch_name" ]; then
        echo "$branch_name"
        return 0
    fi
    
    # Fallback: try to get branch from .git file content
    if [ -f "$worktree_path/.git" ]; then
        local git_ref=$(cat "$worktree_path/.git" | grep -o 'refs/heads/[^[:space:]]*' | head -1)
        if [ -n "$git_ref" ]; then
            echo "$git_ref" | sed 's|refs/heads/||'
            return 0
        fi
    fi
    
    return 1
}

# Function to move worktree to .wt directory
move_worktree() {
    local worktree_path="$1"
    local branch_name="$2"
    
    # Create .wt directory if it doesn't exist
    mkdir -p .wt
    
    # Determine new path in .wt directory
    # Replace slashes with dashes for directory name
    local new_name=$(echo "$branch_name" | sed 's|/|-|g')
    local new_path=".wt/$new_name"
    
    log_info "Moving worktree from $worktree_path to $new_path"
    
    # Check if destination already exists
    if [ -d "$new_path" ]; then
        log_warning "Destination $new_path already exists. Skipping."
        return 1
    fi
    
    # Move the worktree
    if mv "$worktree_path" "$new_path"; then
        log_success "Successfully moved worktree to $new_path"
        
        # Update the worktree in git
        if git worktree remove "$worktree_path" 2>/dev/null; then
            log_info "Removed old worktree reference"
        fi
        
        # Add the new worktree location
        if git worktree add "$new_path" "$branch_name" 2>/dev/null; then
            log_success "Added new worktree reference"
        else
            log_warning "Could not add new worktree reference, but files are moved"
        fi
        
        return 0
    else
        log_error "Failed to move worktree from $worktree_path to $new_path"
        return 1
    fi
}

# Function to handle external worktrees
handle_external_worktree() {
    local worktree_path="$1"
    local branch_name="$2"
    
    log_warning "Found external worktree: $worktree_path"
    log_info "This worktree is outside the repository and cannot be moved automatically."
    log_info "You may want to manually move it to .wt/$branch_name"
    
    # Create a placeholder in .wt directory
    local new_name=$(echo "$branch_name" | sed 's|/|-|g')
    local new_path=".wt/$new_name"
    
    if [ ! -d "$new_path" ]; then
        log_info "Creating placeholder directory: $new_path"
        mkdir -p "$new_path"
        echo "# Placeholder for external worktree: $branch_name" > "$new_path/README.md"
        echo "# Original location: $worktree_path" >> "$new_path/README.md"
        echo "# Please move this worktree manually if needed" >> "$new_path/README.md"
    fi
}

# Main migration function
migrate_worktrees() {
    log_info "Starting worktree migration to .wt directory"
    
    # Get all worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
    local migrated_count=0
    local skipped_count=0
    
    while IFS= read -r worktree_path; do
        if [ -z "$worktree_path" ]; then
            continue
        fi
        
        # Skip the main worktree
        if [ "$worktree_path" = "$(pwd)" ]; then
            log_info "Skipping main worktree: $worktree_path"
            continue
        fi
        
        # Get branch name
        local branch_name=$(get_worktree_branch "$worktree_path")
        if [ -z "$branch_name" ]; then
            log_warning "Could not determine branch name for worktree: $worktree_path"
            continue
        fi
        
        # Check if worktree is in the repository
        local repo_root=$(git rev-parse --show-toplevel)
        if [[ "$worktree_path" == "$repo_root"* ]]; then
            # Worktree is in repository, move it
            if move_worktree "$worktree_path" "$branch_name"; then
                migrated_count=$((migrated_count + 1))
            else
                skipped_count=$((skipped_count + 1))
            fi
        else
            # Worktree is external
            handle_external_worktree "$worktree_path" "$branch_name"
            skipped_count=$((skipped_count + 1))
        fi
        
    done <<< "$worktrees"
    
    log_success "Migration complete!"
    log_info "Migrated: $migrated_count worktrees"
    log_info "Skipped: $skipped_count worktrees"
}

# Function to clean up old worktree directories
cleanup_old_directories() {
    log_info "Checking for old worktree directories..."
    
    local old_dirs=(
        "worktree-cleanup-remote-branches"
        "worktree-feat-systems-diagrams"
        "worktree-lifecycle"
    )
    
    for dir in "${old_dirs[@]}"; do
        if [ -d "$dir" ] && is_worktree "$dir"; then
            log_warning "Found old worktree directory: $dir"
            log_info "This should have been moved by the migration script."
            log_info "You can safely remove it if it's no longer needed."
        fi
    done
}

# Main execution
main() {
    log_info "Worktree Migration Script"
    log_info "This script will move existing worktrees to the .wt directory"
    echo ""
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    # Create .wt directory if it doesn't exist
    mkdir -p .wt
    
    # Run migration
    migrate_worktrees
    
    echo ""
    cleanup_old_directories
    
    log_success "Migration script completed!"
    log_info "All worktrees should now be in the .wt directory"
}

# Run main function
main "$@" 
