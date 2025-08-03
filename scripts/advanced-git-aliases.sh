#!/bin/bash
# Advanced Safe Git Aliases Setup Script
# Uses safe-push.sh and safe-commit.sh scripts for maximum safety
# Usage: ./advanced-git-aliases.sh [setup|test|remove]

set -euo pipefail

# Colors for clean output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Icons for structured output
ICON_SUCCESS="✅"
ICON_ERROR="❌"
ICON_WARNING="⚠️"
ICON_INFO="ℹ️"
ICON_SETUP="🔧"
ICON_TEST="🧪"
ICON_REMOVE="🗑️"

# Function to print styled output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success")
            echo -e "${GREEN}${ICON_SUCCESS} ${message}${NC}"
            ;;
        "error")
            echo -e "${RED}${ICON_ERROR} ${message}${NC}"
            ;;
        "warning")
            echo -e "${YELLOW}${ICON_WARNING} ${message}${NC}"
            ;;
        "info")
            echo -e "${BLUE}${ICON_INFO} ${message}${NC}"
            ;;
        "setup")
            echo -e "${PURPLE}${ICON_SETUP} ${message}${NC}"
            ;;
        "test")
            echo -e "${CYAN}${ICON_TEST} ${message}${NC}"
            ;;
        "remove")
            echo -e "${RED}${ICON_REMOVE} ${message}${NC}"
            ;;
    esac
}

# Function to check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_status "error" "Not in a git repository"
        exit 1
    fi
}

# Function to check if safe scripts exist
check_safe_scripts() {
    local script_dir="$(dirname "$0")"
    
    if [[ ! -f "$script_dir/safe-push.sh" ]]; then
        print_status "error" "safe-push.sh not found in $script_dir"
        exit 1
    fi
    
    if [[ ! -f "$script_dir/safe-commit.sh" ]]; then
        print_status "error" "safe-commit.sh not found in $script_dir"
        exit 1
    fi
    
    if [[ ! -x "$script_dir/safe-push.sh" ]]; then
        print_status "error" "safe-push.sh is not executable"
        exit 1
    fi
    
    if [[ ! -x "$script_dir/safe-commit.sh" ]]; then
        print_status "error" "safe-commit.sh is not executable"
        exit 1
    fi
}

# Function to get absolute path to script
get_script_path() {
    local script_name="$1"
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    echo "$script_dir/$script_name"
}

# Function to set up advanced aliases
setup_aliases() {
    print_status "setup" "Setting up advanced safe git aliases..."
    
    # Check prerequisites
    check_git_repo
    check_safe_scripts
    
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    
    # Add safe-push alias
    print_status "info" "Adding safe-push alias..."
    git config alias.safe-push "!$script_dir/safe-push.sh"
    
    # Add safe-commit alias
    print_status "info" "Adding safe-commit alias..."
    git config alias.safe-commit "!$script_dir/safe-commit.sh"
    
    # Add auto-push alias (uses xtask)
    print_status "info" "Adding auto-push alias..."
    git config alias.auto-push '!f() { echo "🚀 Starting auto-push workflow..."; cargo run -p xtask -- clean-auto-push -m "$1" "$@"; }; f'
    
    # Add status-clean alias
    print_status "info" "Adding status-clean alias..."
    git config alias.status-clean '!f() { echo "🔍 Repository Status:"; echo "==================="; current_branch=$(git rev-parse --abbrev-ref HEAD); echo "🌿 Current branch: $current_branch"; if git diff-index --quiet HEAD --; then echo "✅ Working directory clean"; else echo "⚠️  Uncommitted changes detected"; git status --short; fi; ahead_count=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0"); behind_count=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo "0"); if [[ $ahead_count != "0" ]]; then echo "📤 Ahead of remote: $ahead_count commit(s)"; fi; if [[ $behind_count != "0" ]]; then echo "📥 Behind remote: $behind_count commit(s)"; fi; if [[ $ahead_count == "0" && $behind_count == "0" ]]; then echo "✅ Up to date with remote"; fi; }; f'
    
    # Add safe-workflow alias (commit + push in one command)
    print_status "info" "Adding safe-workflow alias..."
    git config alias.safe-workflow '!f() { echo "🔄 Safe workflow: commit + push"; if git safe-commit "$@"; then echo "📤 Pushing changes..."; git safe-push; else echo "❌ Commit failed, skipping push"; exit 1; fi; }; f'
    
    # Add safe-add alias (stage files with safety checks)
    print_status "info" "Adding safe-add alias..."
    git config alias.safe-add '!f() { echo "📁 Safe add: staging files with checks..."; for file in "$@"; do if [[ -f "$file" ]]; then echo "  • Adding $file"; git add "$file"; else echo "  ⚠️  File not found: $file"; fi; done; echo "✅ Files staged successfully"; }; f'
    
    # Add safe-branch alias (create branch with safety checks)
    print_status "info" "Adding safe-branch alias..."
    git config alias.safe-branch '!f() { if [[ $# -eq 0 ]]; then echo "❌ Branch name required"; echo "Usage: git safe-branch <branch-name>"; exit 1; fi; branch_name="$1"; if git show-ref --verify --quiet refs/heads/"$branch_name"; then echo "❌ Branch $branch_name already exists"; exit 1; fi; echo "🌿 Creating branch: $branch_name"; git checkout -b "$branch_name"; echo "✅ Branch $branch_name created and checked out"; }; f'
    
    # Add safe-merge alias (merge with safety checks)
    print_status "info" "Adding safe-merge alias..."
    git config alias.safe-merge '!f() { if [[ $# -eq 0 ]]; then echo "❌ Branch name required"; echo "Usage: git safe-merge <branch-name>"; exit 1; fi; branch_name="$1"; if ! git show-ref --verify --quiet refs/heads/"$branch_name"; then echo "❌ Branch $branch_name does not exist"; exit 1; fi; current_branch=$(git rev-parse --abbrev-ref HEAD); if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then echo "❌ Merging into protected branch $current_branch is not allowed"; exit 1; fi; echo "🔄 Merging $branch_name into $current_branch"; git merge "$branch_name"; echo "✅ Merge completed successfully"; }; f'
    
    # Add safe-rebase alias (rebase with safety checks)
    print_status "info" "Adding safe-rebase alias..."
    git config alias.safe-rebase '!f() { if [[ $# -eq 0 ]]; then echo "❌ Branch name required"; echo "Usage: git safe-rebase <branch-name>"; exit 1; fi; branch_name="$1"; if ! git show-ref --verify --quiet refs/heads/"$branch_name"; then echo "❌ Branch $branch_name does not exist"; exit 1; fi; current_branch=$(git rev-parse --abbrev-ref HEAD); if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then echo "❌ Rebasing protected branch $current_branch is not allowed"; exit 1; fi; echo "🔄 Rebasing $current_branch onto $branch_name"; git rebase "$branch_name"; echo "✅ Rebase completed successfully"; }; f'
    
    # Add safe-log alias (clean log display)
    print_status "info" "Adding safe-log alias..."
    git config alias.safe-log '!f() { echo "📋 Git Log:"; echo "==========="; git log --oneline --graph --decorate --all "$@"; }; f'
    
    # Add safe-diff alias (clean diff display)
    print_status "info" "Adding safe-diff alias..."
    git config alias.safe-diff '!f() { echo "📊 Git Diff:"; echo "============"; git diff --color=always "$@"; }; f'
    
    # Add safe-stash alias (stash with safety checks)
    print_status "info" "Adding safe-stash alias..."
    git config alias.safe-stash '!f() { if git diff-index --quiet HEAD --; then echo "✅ No changes to stash"; exit 0; fi; echo "💾 Stashing changes..."; git stash push -m "${1:-WIP: $(date)}"; echo "✅ Changes stashed successfully"; }; f'
    
    # Add safe-clean alias (clean with safety checks)
    print_status "info" "Adding safe-clean alias..."
    git config alias.safe-clean '!f() { echo "🧹 Safe clean: removing untracked files..."; echo "Files to be removed:"; git clean -n; read -p "Continue? (y/N): " -n 1 -r; echo; if [[ $REPLY =~ ^[Yy]$ ]]; then git clean -f; echo "✅ Clean completed"; else echo "❌ Clean cancelled"; fi; }; f'
    
    print_status "success" "Advanced safe git aliases configured successfully!"
    print_status "info" "Available aliases:"
    echo "  • git safe-push      - Safe push with comprehensive checks"
    echo "  • git safe-commit    - Safe commit with validation"
    echo "  • git auto-push      - Use xtask clean auto-push workflow"
    echo "  • git status-clean   - Clean status display"
    echo "  • git safe-workflow  - Commit + push in one command"
    echo "  • git safe-add       - Stage files with safety checks"
    echo "  • git safe-branch    - Create branch with safety checks"
    echo "  • git safe-merge     - Merge with safety checks"
    echo "  • git safe-rebase    - Rebase with safety checks"
    echo "  • git safe-log       - Clean log display"
    echo "  • git safe-diff      - Clean diff display"
    echo "  • git safe-stash     - Stash with safety checks"
    echo "  • git safe-clean     - Clean with safety checks"
}

# Function to test aliases
test_aliases() {
    print_status "test" "Testing advanced safe git aliases..."
    
    # Check prerequisites
    check_git_repo
    check_safe_scripts
    
    # Test safe-push with forbidden flag
    print_status "info" "Testing safe-push with --force flag..."
    if git safe-push --force 2>&1 | grep -q "Forbidden flags detected"; then
        print_status "success" "safe-push correctly blocked --force flag"
    else
        print_status "error" "safe-push failed to block --force flag"
    fi
    
    # Test safe-commit with forbidden flag
    print_status "info" "Testing safe-commit with --no-verify flag..."
    if git safe-commit --no-verify 2>&1 | grep -q "Forbidden flags detected"; then
        print_status "success" "safe-commit correctly blocked --no-verify flag"
    else
        print_status "error" "safe-commit failed to block --no-verify flag"
    fi
    
    # Test safe-branch without name
    print_status "info" "Testing safe-branch without branch name..."
    if git safe-branch 2>&1 | grep -q "Branch name required"; then
        print_status "success" "safe-branch correctly requires branch name"
    else
        print_status "error" "safe-branch failed to require branch name"
    fi
    
    # Test status-clean
    print_status "info" "Testing status-clean..."
    git status-clean
    
    # Test safe-log
    print_status "info" "Testing safe-log..."
    git safe-log -5
    
    print_status "success" "Advanced alias tests completed!"
}

# Function to remove aliases
remove_aliases() {
    print_status "remove" "Removing advanced safe git aliases..."
    
    # Check if we're in a git repository
    check_git_repo
    
    # Remove aliases
    local aliases=(
        "safe-push" "safe-commit" "auto-push" "status-clean"
        "safe-workflow" "safe-add" "safe-branch" "safe-merge"
        "safe-rebase" "safe-log" "safe-diff" "safe-stash" "safe-clean"
    )
    
    for alias in "${aliases[@]}"; do
        if git config --get alias."$alias" > /dev/null 2>&1; then
            git config --unset alias."$alias"
            print_status "info" "Removed alias: $alias"
        else
            print_status "info" "Alias not found: $alias"
        fi
    done
    
    print_status "success" "Advanced safe git aliases removed successfully!"
}

# Function to show usage
show_usage() {
    cat << EOF
Advanced Safe Git Aliases Setup Script

Usage: $0 [COMMAND]

COMMANDS:
    setup     - Set up advanced safe git aliases (default)
    test      - Test the safe aliases
    remove    - Remove all safe aliases
    help      - Show this help message

DESCRIPTION:
    This script sets up comprehensive safe git aliases that use the
    safe-push.sh and safe-commit.sh scripts for maximum safety.

FEATURES:
    • Uses safe-push.sh and safe-commit.sh scripts
    • Comprehensive safety checks and validation
    • JSONL logging for all operations
    • Structured, colored output with icons
    • Prevents dangerous operations
    • Enforces best practices

EXAMPLES:
    $0 setup                    # Set up aliases
    git safe-push origin main   # Safe push with full validation
    git safe-commit -m 'feat: add feature'  # Safe commit with validation
    git safe-workflow -m 'feat: update'     # Commit + push in one command

SAFETY FEATURES:
    • Blocks dangerous flags (--force, --no-verify, etc.)
    • Prevents pushes to protected branches
    • Validates commit messages
    • Checks for sensitive data
    • Uses atomic operations
    • Provides comprehensive logging

EOF
}

# Function to show current aliases
show_aliases() {
    print_status "info" "Current git aliases:"
    echo "=================="
    git config --get-regexp alias | grep -E "alias\.(safe|auto)" | while read -r line; do
        echo "  $line"
    done
}

# Main script logic
case "${1:-setup}" in
    "setup")
        setup_aliases
        show_aliases
        ;;
    "test")
        test_aliases
        ;;
    "remove")
        remove_aliases
        ;;
    "list"|"show")
        show_aliases
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        print_status "error" "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac 
