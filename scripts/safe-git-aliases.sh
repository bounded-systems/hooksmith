#!/bin/bash
# Safe Git Aliases Setup Script
# This script configures git aliases that prevent dangerous operations
# and provide clean, structured output similar to the dashboard style

set -e

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
ICON_PUSH="🚀"
ICON_COMMIT="📝"
ICON_BRANCH="🌿"
ICON_CHECK="🔍"

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
        "push")
            echo -e "${PURPLE}${ICON_PUSH} ${message}${NC}"
            ;;
        "commit")
            echo -e "${CYAN}${ICON_COMMIT} ${message}${NC}"
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

# Function to get current branch
get_current_branch() {
    git rev-parse --abbrev-ref HEAD
}

# Function to check if branch is protected
is_protected_branch() {
    local branch=$1
    case $branch in
        "main"|"master"|"develop"|"production")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to validate commit message format
validate_commit_message() {
    local message=$1
    
    # Check if message is empty
    if [[ -z "$message" ]]; then
        print_status "error" "Commit message cannot be empty"
        return 1
    fi
    
    # Check conventional commit format
    if [[ ! "$message" =~ ^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: ]]; then
        print_status "warning" "Commit message should follow conventional format: type(scope): description"
        print_status "info" "Examples: feat: add new feature, fix(auth): resolve login issue"
    fi
    
    return 0
}

# Safe push alias function
safe_push() {
    print_status "info" "Running safe push validation..."
    
    # Check for forbidden flags
    for arg in "$@"; do
        case "$arg" in
            --force|-f|--force-with-lease|--force-with-lease=*)
                print_status "error" "Forbidden flag: $arg"
                print_status "info" "Use 'git push' directly if you really need to force push"
                exit 1
                ;;
            --no-verify|-n)
                print_status "error" "Forbidden flag: $arg"
                print_status "info" "Pre-push hooks are required for safety"
                exit 1
                ;;
        esac
    done
    
    # Check current branch
    local current_branch=$(get_current_branch)
    print_status "info" "Current branch: $current_branch"
    
    if is_protected_branch "$current_branch"; then
        print_status "error" "Direct pushes to protected branch '$current_branch' are forbidden!"
        print_status "info" "Use pull requests or merge requests instead"
        exit 1
    fi
    
    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        print_status "warning" "You have uncommitted changes"
        print_status "info" "Consider committing or stashing changes before pushing"
    fi
    
    # Check if branch is ahead of remote
    local ahead_count=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0")
    if [[ "$ahead_count" == "0" ]]; then
        print_status "info" "No commits to push"
        return 0
    fi
    
    print_status "push" "Pushing $ahead_count commit(s) to origin/$current_branch"
    
    # Execute the actual push
    if git push "$@"; then
        print_status "success" "Successfully pushed to origin/$current_branch"
    else
        print_status "error" "Push failed"
        exit 1
    fi
}

# Safe commit alias function
safe_commit() {
    print_status "info" "Running safe commit validation..."
    
    # Check for forbidden flags
    for arg in "$@"; do
        case "$arg" in
            --no-verify|-n)
                print_status "error" "Forbidden flag: $arg"
                print_status "info" "Pre-commit hooks are required for code quality"
                exit 1
                ;;
        esac
    done
    
    # Get commit message
    local message=""
    for arg in "$@"; do
        case "$arg" in
            -m|--message)
                shift
                message="$1"
                break
                ;;
        esac
    done
    
    # Validate commit message if provided
    if [[ -n "$message" ]]; then
        if ! validate_commit_message "$message"; then
            exit 1
        fi
    fi
    
    print_status "commit" "Creating commit..."
    
    # Execute the actual commit
    if git commit "$@"; then
        local commit_hash=$(git rev-parse --short HEAD)
        print_status "success" "Commit created: $commit_hash"
    else
        print_status "error" "Commit failed"
        exit 1
    fi
}

# Safe checkout alias function
safe_checkout() {
    print_status "info" "Running safe checkout validation..."
    
    # Check for forbidden flags
    for arg in "$@"; do
        case "$arg" in
            --force|-f)
                print_status "error" "Forbidden flag: $arg"
                print_status "info" "Use 'git checkout' directly if you really need to force checkout"
                exit 1
                ;;
        esac
    done
    
    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        print_status "warning" "You have uncommitted changes"
        print_status "info" "Consider committing or stashing changes before checkout"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "info" "Checkout cancelled"
            exit 0
        fi
    fi
    
    print_status "info" "Checking out..."
    
    # Execute the actual checkout
    if git checkout "$@"; then
        local current_branch=$(get_current_branch)
        print_status "success" "Switched to branch: $current_branch"
    else
        print_status "error" "Checkout failed"
        exit 1
    fi
}

# Main function to set up aliases
setup_aliases() {
    print_status "info" "Setting up safe git aliases..."
    
    # Check if we're in a git repository
    check_git_repo
    
    # Create the alias configuration
    local alias_config="
[alias]
    # Safe push - prevents dangerous flags and protected branch pushes
    safe-push = \"!f() { \
        for arg in \"\\$@\"; do \
            case \"\\$arg\" in \
                --force|-f|--force-with-lease|--force-with-lease=*) \
                    echo '❌ Forbidden flag: \\$arg'; \
                    echo 'ℹ️  Use git push directly if you really need to force push'; \
                    exit 1 ;; \
                --no-verify|-n) \
                    echo '❌ Forbidden flag: \\$arg'; \
                    echo 'ℹ️  Pre-push hooks are required for safety'; \
                    exit 1 ;; \
            esac; \
        done; \
        current_branch=\\\$(git rev-parse --abbrev-ref HEAD); \
        case \\$current_branch in \
            main|master|develop|production) \
                echo '❌ Direct pushes to protected branch \\$current_branch are forbidden!'; \
                echo 'ℹ️  Use pull requests or merge requests instead'; \
                exit 1 ;; \
        esac; \
        ahead_count=\\\$(git rev-list --count @{u}..HEAD 2>/dev/null || echo '0'); \
        if [[ \\$ahead_count == '0' ]]; then \
            echo 'ℹ️  No commits to push'; \
            exit 0; \
        fi; \
        echo '🚀 Pushing \\$ahead_count commit(s) to origin/\\$current_branch'; \
        git push \"\\$@\"; \
        if [[ \\$? == 0 ]]; then \
            echo '✅ Successfully pushed to origin/\\$current_branch'; \
        else \
            echo '❌ Push failed'; \
            exit 1; \
        fi; \
    }; f\"
    
    # Safe commit - prevents --no-verify and validates message format
    safe-commit = \"!f() { \
        for arg in \"\\$@\"; do \
            case \"\\$arg\" in \
                --no-verify|-n) \
                    echo '❌ Forbidden flag: \\$arg'; \
                    echo 'ℹ️  Pre-commit hooks are required for code quality'; \
                    exit 1 ;; \
            esac; \
        done; \
        echo '📝 Creating commit...'; \
        git commit \"\\$@\"; \
        if [[ \\$? == 0 ]]; then \
            commit_hash=\\\$(git rev-parse --short HEAD); \
            echo '✅ Commit created: \\$commit_hash'; \
        else \
            echo '❌ Commit failed'; \
            exit 1; \
        fi; \
    }; f\"
    
    # Safe checkout - prevents --force and warns about uncommitted changes
    safe-checkout = \"!f() { \
        for arg in \"\\$@\"; do \
            case \"\\$arg\" in \
                --force|-f) \
                    echo '❌ Forbidden flag: \\$arg'; \
                    echo 'ℹ️  Use git checkout directly if you really need to force checkout'; \
                    exit 1 ;; \
            esac; \
        done; \
        if ! git diff-index --quiet HEAD --; then \
            echo '⚠️  You have uncommitted changes'; \
            echo 'ℹ️  Consider committing or stashing changes before checkout'; \
            read -p 'Continue anyway? (y/N): ' -n 1 -r; \
            echo; \
            if [[ ! \\$REPLY =~ ^[Yy]\\$ ]]; then \
                echo 'ℹ️  Checkout cancelled'; \
                exit 0; \
            fi; \
        fi; \
        echo 'ℹ️  Checking out...'; \
        git checkout \"\\$@\"; \
        if [[ \\$? == 0 ]]; then \
            current_branch=\\\$(git rev-parse --abbrev-ref HEAD); \
            echo '✅ Switched to branch: \\$current_branch'; \
        else \
            echo '❌ Checkout failed'; \
            exit 1; \
        fi; \
    }; f\"
    
    # Auto-push workflow using xtask
    auto-push = \"!f() { \
        echo '🚀 Starting auto-push workflow...'; \
        cargo run -p xtask -- clean-auto-push -m \"\\$1\" \"\\$@\"; \
    }; f\"
    
    # Quick status with clean output
    status-clean = \"!f() { \
        echo '🔍 Repository Status:'; \
        echo '==================='; \
        current_branch=\\\$(git rev-parse --abbrev-ref HEAD); \
        echo '🌿 Current branch: \\$current_branch'; \
        if git diff-index --quiet HEAD --; then \
            echo '✅ Working directory clean'; \
        else \
            echo '⚠️  Uncommitted changes detected'; \
            git status --short; \
        fi; \
        ahead_count=\\\$(git rev-list --count @{u}..HEAD 2>/dev/null || echo '0'); \
        behind_count=\\\$(git rev-list --count HEAD..@{u} 2>/dev/null || echo '0'); \
        if [[ \\$ahead_count != '0' ]]; then \
            echo '📤 Ahead of remote: \\$ahead_count commit(s)'; \
        fi; \
        if [[ \\$behind_count != '0' ]]; then \
            echo '📥 Behind remote: \\$behind_count commit(s)'; \
        fi; \
        if [[ \\$ahead_count == '0' && \\$behind_count == '0' ]]; then \
            echo '✅ Up to date with remote'; \
        fi; \
    }; f\"
"
    
    # Write to git config
    echo "$alias_config" >> .git/config
    
    print_status "success" "Safe git aliases configured successfully!"
    print_status "info" "Available aliases:"
    echo "  • git safe-push    - Safe push with validation"
    echo "  • git safe-commit  - Safe commit with validation"
    echo "  • git safe-checkout - Safe checkout with warnings"
    echo "  • git auto-push    - Use xtask clean auto-push workflow"
    echo "  • git status-clean - Clean status display"
}

# Function to show usage
show_usage() {
    echo "Safe Git Aliases Setup Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  setup     - Set up safe git aliases (default)"
    echo "  test      - Test the safe aliases"
    echo "  help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup                    # Set up aliases"
    echo "  git safe-push origin main   # Safe push"
    echo "  git safe-commit -m 'feat: add feature'  # Safe commit"
    echo "  git auto-push 'feat: update'            # Use xtask workflow"
}

# Function to test aliases
test_aliases() {
    print_status "info" "Testing safe git aliases..."
    
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