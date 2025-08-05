#!/bin/bash

# Clean up old worktrees that are no longer needed
# Direct approach to remove obsolete worktrees

set -e

echo "🧹 CLEANING UP OLD WORKTREES"
echo "============================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local state=$1
    local message=$2
    case $state in
        "ERROR") echo -e "${RED}❌ $message${NC}" ;;
        "SUCCESS") echo -e "${GREEN}✅ $message${NC}" ;;
        "WARNING") echo -e "${YELLOW}⚠️  $message${NC}" ;;
        "INFO") echo -e "${BLUE}ℹ️  $message${NC}" ;;
        *) echo "📝 $message" ;;
    esac
}

# List of worktrees to remove (old conflicted ones)
OLD_WORKTREES=(
    "worktree-fix-main-cleanup-20250804-211403"
    "worktree-fix-workspace-config"
    "worktree-fix-workspace-dependencies"
    "worktree-fix-xtask-cleanup"
)

# Function to remove a worktree
remove_worktree() {
    local worktree_name=$1
    
    if [ ! -d "$worktree_name" ]; then
        print_status "WARNING" "Worktree $worktree_name does not exist"
        return
    fi
    
    print_status "INFO" "Removing worktree: $worktree_name"
    
    # Abort any ongoing operations first
    cd "$worktree_name"
    if git status | grep -q "rebase"; then
        print_status "INFO" "Aborting rebase in $worktree_name"
        git rebase --abort 2>/dev/null || true
    fi
    cd ..
    
    # Get branch name before removal
    cd "$worktree_name"
    local branch=$(git branch --show-current)
    cd ..
    
    # Remove worktree
    print_status "INFO" "Removing worktree directory"
    git worktree remove "$worktree_name" --force 2>/dev/null || {
        print_status "WARNING" "Could not remove worktree, trying to delete directory"
        rm -rf "$worktree_name"
    }
    
    # Remove branch if it exists
    if [ -n "$branch" ]; then
        print_status "INFO" "Removing branch: $branch"
        git branch -D "$branch" 2>/dev/null || true
    fi
    
    print_status "SUCCESS" "Removed worktree $worktree_name"
    echo ""
}

# Function to create PR for ready worktree
create_pr_for_ready() {
    local worktree_name="worktree-management-improvements"
    
    if [ ! -d "$worktree_name" ]; then
        print_status "WARNING" "Ready worktree $worktree_name does not exist"
        return
    fi
    
    print_status "INFO" "Creating PR for ready worktree: $worktree_name"
    
    cd "$worktree_name"
    local branch=$(git branch --show-current)
    
    if [ -n "$branch" ]; then
        # Check if branch exists on origin
        if git ls-remote --heads origin "$branch" | grep -q "$branch"; then
            local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
            if [[ "$repo_url" == *"github.com"* ]]; then
                local pr_url="$repo_url/compare/main...$branch"
                print_status "SUCCESS" "Create PR at: $pr_url"
            fi
        fi
    fi
    
    cd ..
}

# Main function
main() {
    echo "🗑️  Removing old conflicted worktrees..."
    echo ""
    
    for worktree in "${OLD_WORKTREES[@]}"; do
        remove_worktree "$worktree"
    done
    
    echo "🚀 Creating PR for ready worktree..."
    echo ""
    create_pr_for_ready
    
    echo "🎉 Cleanup completed!"
    echo ""
    echo "📊 Final Status:"
    ./scripts/worktree-status-report.sh
}

# Run main function
main "$@" 
