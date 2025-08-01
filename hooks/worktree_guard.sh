#!/bin/bash

# Worktree Guard Script
# This script enforces proper CLI usage for worktree operations
# Prevents direct git checkout -b and ensures CLI is used

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔒 Worktree Safety Check${NC}"

# Get current branch and worktree info
current_branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached")
worktree_root=$(git rev-parse --show-toplevel 2>/dev/null)
worktree_path=$(git rev-parse --git-dir 2>/dev/null)

# Check if we're in a worktree
is_worktree=false
if [[ "$worktree_path" == *"/.git/worktrees/"* ]]; then
    is_worktree=true
    worktree_name=$(basename "$(dirname "$worktree_path")")
    echo -e "${GREEN}✅ Running in worktree: ${worktree_name}${NC}"
fi

# Check if we're in the CLI prototype worktree
if [[ "$current_branch" == "pushd-cli-prototype" ]] || [[ "$is_worktree" == true && "$worktree_name" == "pushd-cli-prototype" ]]; then
    echo -e "${GREEN}✅ CLI prototype worktree detected${NC}"
    
    # Check for CLI binary
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust toolchain.${NC}"
        exit 1
    fi
    
    # Check if CLI is built
    if [[ ! -f ".cli-helper/target/release/pushd-cli" ]] && [[ ! -f ".cli-helper/target/debug/pushd-cli" ]]; then
        echo -e "${YELLOW}⚠️  CLI not built. Building now...${NC}"
        cd .cli-helper
        ./build.sh
        cd ..
    fi
    
    # Validate worktree setup
    if [[ "$is_worktree" == true ]]; then
        echo -e "${GREEN}✅ Worktree setup validated${NC}"
    else
        echo -e "${YELLOW}⚠️  Not in a worktree. Consider using: pushd-cli worktree create <name>${NC}"
    fi
    
    exit 0
fi

# Check if we're trying to create a branch directly
if [[ "$current_branch" == "main" ]]; then
    # Look for recent git checkout -b commands in git reflog
    recent_checkout=$(git reflog --oneline -10 | grep -E "checkout: moving from.*to.*" | head -1)
    
    if [[ -n "$recent_checkout" ]]; then
        echo -e "${RED}❌ Direct branch creation detected!${NC}"
        echo ""
        echo "Please use the CLI instead:"
        echo "  pushd-cli worktree create <branch-name>"
        echo ""
        echo "This ensures proper worktree setup and validation."
        echo ""
        echo "Recent checkout detected: $recent_checkout"
        exit 1
    fi
fi

# Check for worktree-related files
worktree_files=$(find . -name "*.worktree" -o -name "worktree-*" 2>/dev/null | head -5)
if [[ -n "$worktree_files" ]]; then
    echo -e "${YELLOW}⚠️  Worktree files detected:${NC}"
    echo "$worktree_files"
    echo ""
    echo "Consider using CLI for worktree management:"
    echo "  pushd-cli worktree list"
    echo "  pushd-cli worktree create <name>"
    echo "  pushd-cli worktree remove <name>"
fi

# Validate CLI commands are available
if command -v cargo &> /dev/null; then
    # Check if we can run CLI commands
    if [[ -f ".cli-helper/target/release/pushd-cli" ]] || [[ -f ".cli-helper/target/debug/pushd-cli" ]]; then
        echo -e "${GREEN}✅ CLI available for worktree operations${NC}"
    else
        echo -e "${YELLOW}⚠️  CLI not built. Run: cd .cli-helper && ./build.sh${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Rust toolchain not available${NC}"
fi

echo -e "${GREEN}✅ Worktree safety check passed${NC}"
exit 0 
