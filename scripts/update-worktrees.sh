#!/bin/bash

# Update all worktrees to latest origin/main
# This script will rebase each worktree's base branch to origin/main

set -e

echo "🔄 Updating all worktrees to latest origin/main..."

# Get the latest from origin
echo "📥 Fetching latest from origin..."
git fetch origin

# Get list of worktrees (excluding the main worktree)
worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2 | grep -v "^$(pwd)$")

for worktree in $worktrees; do
    if [ -d "$worktree" ]; then
        echo ""
        echo "🔄 Updating worktree: $(basename "$worktree")"
        echo "   Path: $worktree"
        
        # Get the branch name for this worktree
        branch=$(cd "$worktree" && git branch --show-current)
        echo "   Branch: $branch"
        
        # Check if there are uncommitted changes
        cd "$worktree"
        if [ -n "$(git status --porcelain)" ]; then
            echo "   ⚠️  WARNING: Uncommitted changes detected!"
            echo "   📝 Changes:"
            git status --short
            echo "   💡 Consider committing or stashing changes before updating"
            echo "   ⏭️  Skipping this worktree..."
            cd - > /dev/null
            continue
        fi
        
        # Get current commit
        current_commit=$(git rev-parse HEAD)
        echo "   Current commit: $(git log --oneline -1)"
        
        # Check how many commits behind origin/main
        behind_count=$(git rev-list --count HEAD..origin/main)
        echo "   Behind origin/main by: $behind_count commits"
        
        if [ "$behind_count" -eq 0 ]; then
            echo "   ✅ Already up to date!"
            cd - > /dev/null
            continue
        fi
        
        # Rebase to origin/main
        echo "   🔄 Rebasing to origin/main..."
        if git rebase origin/main; then
            echo "   ✅ Successfully updated!"
            echo "   New commit: $(git log --oneline -1)"
        else
            echo "   ❌ Rebase failed! Manual intervention may be needed."
            echo "   💡 You can:"
            echo "      - cd $worktree"
            echo "      - git rebase --abort (to cancel)"
            echo "      - git rebase --continue (after resolving conflicts)"
        fi
        
        cd - > /dev/null
    fi
done

echo ""
echo "🎉 Worktree update process completed!"
echo ""
echo "📊 Summary:"
echo "   - Main worktree: $(cd /Users/bobby/dev/repos/hooksmith && git log --oneline -1)"
echo "   - Origin/main: $(git log --oneline origin/main -1)"
echo ""
echo "💡 Next steps:"
echo "   - Review any worktrees that had conflicts"
echo "   - Test your changes in updated worktrees"
echo "   - Create PRs for worktrees that are ready" 
