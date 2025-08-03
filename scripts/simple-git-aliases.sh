#!/bin/bash
# Simple Safe Git Aliases Setup Script
# This script configures basic git aliases without complex escaping

set -e

# Colors for clean output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print styled output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success")
            echo -e "${GREEN}✅ ${message}${NC}"
            ;;
        "error")
            echo -e "${RED}❌ ${message}${NC}"
            ;;
        "warning")
            echo -e "${YELLOW}⚠️  ${message}${NC}"
            ;;
        "info")
            echo -e "${BLUE}ℹ️  ${message}${NC}"
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

# Main function to set up aliases
setup_aliases() {
    print_status "info" "Setting up simple safe git aliases..."
    
    # Check if we're in a git repository
    check_git_repo
    
    # Add aliases one by one to avoid escaping issues
    print_status "info" "Adding safe-push alias..."
    git config alias.safe-push '!f() { for arg in "$@"; do case "$arg" in --force|-f|--force-with-lease|--no-verify|-n) echo "❌ Forbidden flag: $arg"; echo "ℹ️  Use git push directly if you really need to force push"; exit 1 ;; esac; done; current_branch=$(git rev-parse --abbrev-ref HEAD); case $current_branch in main|master|develop|production) echo "❌ Direct pushes to protected branch $current_branch are forbidden!"; echo "ℹ️  Use pull requests or merge requests instead"; exit 1 ;; esac; echo "🚀 Pushing to origin/$current_branch"; git push "$@"; if [[ $? == 0 ]]; then echo "✅ Successfully pushed to origin/$current_branch"; else echo "❌ Push failed"; exit 1; fi; }; f'
    
    print_status "info" "Adding safe-commit alias..."
    git config alias.safe-commit '!f() { for arg in "$@"; do case "$arg" in --no-verify|-n) echo "❌ Forbidden flag: $arg"; echo "ℹ️  Pre-commit hooks are required for code quality"; exit 1 ;; esac; done; echo "📝 Creating commit..."; git commit "$@"; if [[ $? == 0 ]]; then commit_hash=$(git rev-parse --short HEAD); echo "✅ Commit created: $commit_hash"; else echo "❌ Commit failed"; exit 1; fi; }; f'
    
    print_status "info" "Adding auto-push alias..."
    git config alias.auto-push '!f() { echo "🚀 Starting auto-push workflow..."; cargo run -p xtask -- clean-auto-push -m "$1" "$@"; }; f'
    
    print_status "info" "Adding status-clean alias..."
    git config alias.status-clean '!f() { echo "🔍 Repository Status:"; echo "==================="; current_branch=$(git rev-parse --abbrev-ref HEAD); echo "🌿 Current branch: $current_branch"; if git diff-index --quiet HEAD --; then echo "✅ Working directory clean"; else echo "⚠️  Uncommitted changes detected"; git status --short; fi; ahead_count=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0"); behind_count=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo "0"); if [[ $ahead_count != "0" ]]; then echo "📤 Ahead of remote: $ahead_count commit(s)"; fi; if [[ $behind_count != "0" ]]; then echo "📥 Behind remote: $behind_count commit(s)"; fi; if [[ $ahead_count == "0" && $behind_count == "0" ]]; then echo "✅ Up to date with remote"; fi; }; f'
    
    print_status "success" "Simple safe git aliases configured successfully!"
    print_status "info" "Available aliases:"
    echo "  • git safe-push    - Safe push with validation"
    echo "  • git safe-commit  - Safe commit with validation"
    echo "  • git auto-push    - Use xtask clean auto-push workflow"
    echo "  • git status-clean - Clean status display"
}

# Function to test aliases
test_aliases() {
    print_status "info" "Testing simple safe git aliases..."
    
    # Test safe-push with forbidden flag
    print_status "info" "Testing safe-push with --force flag..."
    if git safe-push --force 2>&1 | grep -q "Forbidden flag"; then
        print_status "success" "safe-push correctly blocked --force flag"
    else
        print_status "error" "safe-push failed to block --force flag"
    fi
    
    # Test safe-commit with forbidden flag
    print_status "info" "Testing safe-commit with --no-verify flag..."
    if git safe-commit --no-verify 2>&1 | grep -q "Forbidden flag"; then
        print_status "success" "safe-commit correctly blocked --no-verify flag"
    else
        print_status "error" "safe-commit failed to block --no-verify flag"
    fi
    
    # Test status-clean
    print_status "info" "Testing status-clean..."
    git status-clean
    
    print_status "success" "Alias tests completed!"
}

# Function to show usage
show_usage() {
    echo "Simple Safe Git Aliases Setup Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  setup     - Set up simple safe git aliases (default)"
    echo "  test      - Test the safe aliases"
    echo "  help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup                    # Set up aliases"
    echo "  git safe-push origin main   # Safe push"
    echo "  git safe-commit -m 'feat: add feature'  # Safe commit"
    echo "  git auto-push 'feat: update'            # Use xtask workflow"
}

# Main script logic
case "${1:-setup}" in
    "setup")
        setup_aliases
        ;;
    "test")
        test_aliases
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