#!/bin/bash
# Safe Commit Script
# Enforces constraints, adds helpful defaults, and provides JSONL logging
# Usage: ./safe-commit.sh [options] [--] [files...]

set -euo pipefail

# Script version
SCRIPT_VERSION="1.0.0"

# Colors for output
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
ICON_COMMIT="📝"
ICON_SECURITY="🔒"
ICON_LOG="📝"

# Configuration
LOG_FILE="${LOG_FILE:-.hooksmith/logs/safe-commit.jsonl}"
MAX_COMMIT_LENGTH=72
MIN_COMMIT_LENGTH=10

# Forbidden flags that are dangerous
FORBIDDEN_FLAGS=(
    "--no-verify" "-n" "--no-gpg-sign" "--no-post-rewrite"
    "--allow-empty-message" "--no-edit" "--only"
)

# Safe defaults to always include
SAFE_DEFAULTS=(
    "--verify"
    "--gpg-sign"
)

# Conventional commit types
CONVENTIONAL_TYPES=(
    "feat" "fix" "docs" "style" "refactor" "test" "chore"
    "perf" "ci" "build" "revert" "wip" "security"
)

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
        "commit")
            echo -e "${CYAN}${ICON_COMMIT} ${message}${NC}"
            ;;
        "security")
            echo -e "${RED}${ICON_SECURITY} ${message}${NC}"
            ;;
    esac
}

# Function to log JSONL events
log_event() {
    local event_type=$1
    local message=$2
    local data=${3:-{}}
    
    # Ensure log directory exists
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Create JSONL event
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local event_json=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "event_type": "$event_type",
  "message": "$message",
  "data": $data,
  "script_version": "$SCRIPT_VERSION",
  "git_repo": "$(git rev-parse --show-toplevel 2>/dev/null || echo 'unknown')",
  "branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
}
EOF
)
    
    echo "$event_json" >> "$LOG_FILE"
}

# Function to check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_status "error" "Not in a git repository"
        log_event "error" "Not in a git repository" '{"error": "not_git_repo"}'
        exit 1
    fi
}

# Function to get current branch
get_current_branch() {
    git rev-parse --abbrev-ref HEAD
}

# Function to validate arguments and reject forbidden flags
validate_arguments() {
    local args=("$@")
    local forbidden_found=()
    
    for arg in "${args[@]}"; do
        for forbidden in "${FORBIDDEN_FLAGS[@]}"; do
            if [[ "$arg" == "$forbidden" ]]; then
                forbidden_found+=("$arg")
            fi
        done
    done
    
    if [[ ${#forbidden_found[@]} -gt 0 ]]; then
        print_status "security" "Forbidden flags detected:"
        for flag in "${forbidden_found[@]}"; do
            echo "  • $flag"
        done
        print_status "error" "These flags are not allowed in safe-commit"
        print_status "info" "Use 'git commit' directly if you really need these flags"
        
        log_event "security_violation" "Forbidden flags detected" "{\"forbidden_flags\": [$(printf '"%s",' "${forbidden_found[@]}" | sed 's/,$//')]}"
        exit 1
    fi
}

# Function to check for staged changes
check_staged_changes() {
    if ! git diff --cached --quiet; then
        local staged_files=$(git diff --cached --name-only | wc -l)
        print_status "info" "Found $staged_files staged file(s)"
        log_event "check" "Staged changes found" "{\"staged_files\": $staged_files}"
    else
        print_status "warning" "No staged changes found"
        print_status "info" "Use 'git add' to stage changes before committing"
        
        log_event "warning" "No staged changes found" '{"staged_files": 0}'
        
        read -p "Continue with commit anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "info" "Commit cancelled by user"
            log_event "cancelled" "Commit cancelled by user due to no staged changes"
            exit 0
        fi
    fi
}

# Function to validate commit message format
validate_commit_message() {
    local message="$1"
    
    # Check if message is empty
    if [[ -z "$message" ]]; then
        print_status "error" "Commit message cannot be empty"
        log_event "validation_error" "Empty commit message" '{"error": "empty_message"}'
        return 1
    fi
    
    # Check minimum length
    if [[ ${#message} -lt $MIN_COMMIT_LENGTH ]]; then
        print_status "warning" "Commit message is too short (${#message} chars, minimum $MIN_COMMIT_LENGTH)"
        log_event "validation_warning" "Short commit message" "{\"length\": ${#message}, \"minimum\": $MIN_COMMIT_LENGTH}"
    fi
    
    # Check maximum length
    if [[ ${#message} -gt $MAX_COMMIT_LENGTH ]]; then
        print_status "warning" "Commit message is too long (${#message} chars, maximum $MAX_COMMIT_LENGTH)"
        log_event "validation_warning" "Long commit message" "{\"length\": ${#message}, \"maximum\": $MAX_COMMIT_LENGTH}"
    fi
    
    # Check conventional commit format
    local is_conventional=false
    for type in "${CONVENTIONAL_TYPES[@]}"; do
        if [[ "$message" =~ ^$type(\([^)]+\))?: ]]; then
            is_conventional=true
            break
        fi
    done
    
    if [[ "$is_conventional" == "false" ]]; then
        print_status "warning" "Commit message should follow conventional format: type(scope): description"
        print_status "info" "Examples: feat: add new feature, fix(auth): resolve login issue"
        log_event "validation_warning" "Non-conventional commit message" "{\"message\": \"$message\"}"
    else
        log_event "validation_success" "Conventional commit message format" "{\"message\": \"$message\"}"
    fi
    
    return 0
}

# Function to check for dangerous patterns in commit message
check_dangerous_patterns() {
    local message="$1"
    local dangerous_patterns=(
        "WIP" "TODO" "FIXME" "HACK" "TEMP" "DEBUG" "TEST"
        "password" "secret" "api_key" "token" "private_key"
    )
    
    local found_patterns=()
    
    for pattern in "${dangerous_patterns[@]}"; do
        if [[ "$message" =~ $pattern ]]; then
            found_patterns+=("$pattern")
        fi
    done
    
    if [[ ${#found_patterns[@]} -gt 0 ]]; then
        print_status "warning" "Potentially dangerous patterns found in commit message:"
        for pattern in "${found_patterns[@]}"; do
            echo "  • $pattern"
        done
        print_status "info" "Consider reviewing the commit message"
        
        log_event "validation_warning" "Dangerous patterns in commit message" "{\"patterns\": [$(printf '"%s",' "${found_patterns[@]}" | sed 's/,$//')]}"
    else
        log_event "validation_success" "No dangerous patterns in commit message"
    fi
}

# Function to check for sensitive data in staged files
check_sensitive_data() {
    local sensitive_patterns=(
        "password.*=.*['\"][^'\"]*['\"]"
        "secret.*=.*['\"][^'\"]*['\"]"
        "api_key.*=.*['\"][^'\"]*['\"]"
        "token.*=.*['\"][^'\"]*['\"]"
        "private_key.*=.*['\"][^'\"]*['\"]"
        "BEGIN.*PRIVATE.*KEY"
        "BEGIN.*RSA.*PRIVATE.*KEY"
        "BEGIN.*DSA.*PRIVATE.*KEY"
        "BEGIN.*EC.*PRIVATE.*KEY"
    )
    
    local staged_files=$(git diff --cached --name-only)
    local has_sensitive_data=false
    local sensitive_files=()
    
    for file in $staged_files; do
        if [[ -f "$file" ]]; then
            for pattern in "${sensitive_patterns[@]}"; do
                if grep -qi "$pattern" "$file" 2>/dev/null; then
                    has_sensitive_data=true
                    sensitive_files+=("$file")
                    break
                fi
            done
        fi
    done
    
    if [[ "$has_sensitive_data" == "true" ]]; then
        print_status "security" "Potential sensitive data found in staged files:"
        for file in "${sensitive_files[@]}"; do
            echo "  • $file"
        done
        print_status "error" "Please review these files before committing"
        
        log_event "security_violation" "Sensitive data detected" "{\"sensitive_files\": [$(printf '"%s",' "${sensitive_files[@]}" | sed 's/,$//')]}"
        
        read -p "Continue with commit anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "info" "Commit cancelled by user"
            log_event "cancelled" "Commit cancelled by user due to sensitive data"
            exit 0
        fi
    else
        log_event "security_check" "No sensitive data detected in staged files"
    fi
}

# Function to build safe commit command
build_commit_command() {
    local args=("$@")
    local commit_args=("commit")
    
    # Add safe defaults
    commit_args+=("${SAFE_DEFAULTS[@]}")
    
    # Add user arguments (already validated)
    commit_args+=("${args[@]}")
    
    echo "${commit_args[@]}"
}

# Function to execute commit
execute_commit() {
    local commit_cmd="$1"
    
    print_status "commit" "Executing: git $commit_cmd"
    log_event "commit_start" "Starting commit execution" "{\"command\": \"git $commit_cmd\"}"
    
    # Execute the commit command
    local output
    local exit_code
    
    if output=$(git $commit_cmd 2>&1); then
        # Success
        local commit_hash=$(git rev-parse --short HEAD)
        print_status "success" "Commit created successfully: $commit_hash"
        
        log_event "commit_success" "Commit created successfully" "{\"commit_hash\": \"$commit_hash\", \"output\": \"$output\"}"
        
        # Display commit info
        echo ""
        print_status "info" "Commit Details:"
        echo "=================="
        echo "Hash: $commit_hash"
        echo "Message: $(git log -1 --pretty=format:%s)"
        echo "Author: $(git log -1 --pretty=format:%an)"
        echo "Date: $(git log -1 --pretty=format:%ad)"
        
        return 0
    else
        exit_code=$?
        print_status "error" "Commit failed"
        log_event "commit_failure" "Commit failed" "{\"exit_code\": $exit_code, \"output\": \"$output\"}"
        
        echo "Error output:"
        echo "$output"
        return 1
    fi
}

# Function to show usage
show_usage() {
    cat << EOF
Safe Commit Script v$SCRIPT_VERSION

Usage: $0 [OPTIONS] [--] [FILES...]

DESCRIPTION:
    Safe git commit with enforced constraints and helpful defaults.
    Prevents dangerous operations and provides structured logging.

OPTIONS:
    -h, --help          Show this help message
    -v, --version       Show version information
    -l, --log-file      Specify custom log file (default: $LOG_FILE)
    -m, --message       Commit message
    -a, --all           Stage all modified files
    --dry-run           Show what would be executed without running
    --verbose           Enable verbose output

EXAMPLES:
    $0 -m "feat: add new feature"     # Commit with message
    $0 -a -m "fix: resolve bug"       # Stage all and commit
    $0 --dry-run -m "test message"    # Show what would be executed

SAFETY FEATURES:
    • Blocks dangerous flags (--no-verify, etc.)
    • Validates commit message format
    • Checks for sensitive data in staged files
    • Uses --verify and --gpg-sign by default
    • Provides structured JSONL logging

FORBIDDEN FLAGS:
    --no-verify, -n, --no-gpg-sign, --no-post-rewrite
    --allow-empty-message, --no-edit, --only

CONVENTIONAL COMMIT TYPES:
    feat, fix, docs, style, refactor, test, chore
    perf, ci, build, revert, wip, security

LOG FILE:
    Events are logged to: $LOG_FILE

EOF
}

# Function to show version
show_version() {
    echo "Safe Commit Script v$SCRIPT_VERSION"
    echo "Git repository: $(git rev-parse --show-toplevel 2>/dev/null || echo 'unknown')"
    echo "Current branch: $(get_current_branch 2>/dev/null || echo 'unknown')"
    echo "Log file: $LOG_FILE"
}

# Main function
main() {
    local args=()
    local dry_run=false
    local verbose=false
    local commit_message=""
    local stage_all=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--version)
                show_version
                exit 0
                ;;
            -l|--log-file)
                LOG_FILE="$2"
                shift 2
                ;;
            -m|--message)
                commit_message="$2"
                shift 2
                ;;
            -a|--all)
                stage_all=true
                shift
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --)
                shift
                args+=("$@")
                break
                ;;
            -*)
                print_status "error" "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                args+=("$1")
                shift
                ;;
        esac
    done
    
    # Initialize logging
    log_event "script_start" "Safe commit script started" "{\"version\": \"$SCRIPT_VERSION\", \"args\": [$(printf '"%s",' "${args[@]}" | sed 's/,$//')]}"
    
    # Check if we're in a git repository
    check_git_repo
    
    # Validate arguments
    validate_arguments "${args[@]}"
    
    # Stage all files if requested
    if [[ "$stage_all" == "true" ]]; then
        print_status "info" "Staging all modified files"
        git add -A
        log_event "stage_all" "Staged all modified files"
    fi
    
    # Check for staged changes
    check_staged_changes
    
    # Check for sensitive data
    check_sensitive_data
    
    # Validate commit message if provided
    if [[ -n "$commit_message" ]]; then
        if ! validate_commit_message "$commit_message"; then
            exit 1
        fi
        check_dangerous_patterns "$commit_message"
    fi
    
    # Build commit command
    local commit_cmd=$(build_commit_command "${args[@]}")
    
    if [[ "$verbose" == "true" ]]; then
        print_status "info" "Commit command: git $commit_cmd"
    fi
    
    if [[ "$dry_run" == "true" ]]; then
        print_status "info" "DRY RUN: Would execute: git $commit_cmd"
        log_event "dry_run" "Dry run completed" "{\"command\": \"git $commit_cmd\"}"
        exit 0
    fi
    
    # Execute commit
    if execute_commit "$commit_cmd"; then
        print_status "success" "Safe commit completed successfully!"
        log_event "script_success" "Safe commit script completed successfully"
        exit 0
    else
        print_status "error" "Safe commit failed!"
        log_event "script_failure" "Safe commit script failed"
        exit 1
    fi
}

# Run main function with all arguments
main "$@" 
