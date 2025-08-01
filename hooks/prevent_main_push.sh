#!/bin/bash

# Prevent pushing to main from CLI prototype worktree
# This ensures the CLI prototype never accidentally merges into main

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the current branch name
current_branch=$(git symbolic-ref --short HEAD 2>/dev/null)

# Check if we're in the pushd-cli-prototype branch
if [ "$current_branch" = "pushd-cli-prototype" ]; then
    echo -e "${YELLOW}🔒 CLI Prototype Safety Check${NC}"
    
    # Check if we're trying to push to main
    while read local_ref local_sha remote_ref remote_sha
    do
        if [ "$remote_ref" = "refs/heads/main" ]; then
            echo -e "${RED}❌ ERROR: Cannot push to main from pushd-cli-prototype branch!${NC}"
            echo ""
            echo "This is a safety measure to prevent accidental merges."
            echo ""
            echo "Instead, please:"
            echo "  1. Push to origin/pushd-cli-prototype: git push origin pushd-cli-prototype"
            echo "  2. Create a pull request: gh pr create --title 'CLI Prototype: ...'"
            echo "  3. Get code review before merging"
            echo ""
            echo "If you really need to push to main (emergency only):"
            echo "  git push origin pushd-cli-prototype:main --force"
            echo ""
            exit 1
        fi
    done
fi

echo -e "${YELLOW}✅ CLI Prototype safety check passed${NC}"
exit 0 
