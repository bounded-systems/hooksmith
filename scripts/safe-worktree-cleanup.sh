#!/bin/bash

# Safe worktree cleanup script
# Checks for uncommitted changes before removing worktrees

set -e

echo "🔍 Checking worktrees for uncommitted changes..."

# Get list of worktrees
worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)

for worktree_path in $worktrees; do
    # Skip the main worktree
    if [[ "$worktree_path" == "$(pwd)" ]]; then
        echo "⏭️  Skipping main worktree: $worktree_path"
        continue
    fi
    
    echo "📁 Checking worktree: $worktree_path"
    
    # Check if worktree directory exists
    if [[ ! -d "$worktree_path" ]]; then
        echo "🗑️  Removing non-existent worktree: $worktree_path"
        git worktree remove "$worktree_path" --force
        continue
    fi
    
    # Check for uncommitted changes
    cd "$worktree_path"
    
    # Get status
    status=$(git status --porcelain)
    
    if [[ -n "$status" ]]; then
        echo "⚠️  WARNING: Uncommitted changes found in $worktree_path"
        echo "   Changes:"
        echo "$status" | sed 's/^/   /'
        echo "   Please commit or stash changes before removing this worktree"
        echo ""
    else
        echo "✅ No uncommitted changes found in $worktree_path"
        echo "🗑️  Removing worktree: $worktree_path"
        git worktree remove "$worktree_path" --force
    fi
    
    # Go back to main directory
    cd - > /dev/null
done

echo "🎉 Worktree cleanup completed!"
