#!/bin/bash
# Safe Push Script
# Enforces constraints, adds helpful defaults, and provides JSONL logging
# Usage: ./safe-push.sh [options] [remote] [branch]

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
ICON_PUSH="🚀"
ICON_SECURITY="🔒"
ICON_LOG="📝"

# Configuration
DEFAULT_REMOTE="origin"
LOG_FILE="${LOG_FILE:-.hooksmith/logs/safe-push.jsonl}"
MAX_RETRIES=3
RETRY_DELAY=2

# Forbidden flags that are dangerous
FORBIDDEN_FLAGS=(
    "--force" "-f" "--force-with-lease" "--force-with-lease=*"
    "--no-verify" "-n" "--delete" "-d" "--mirror"
    "--all" "--tags" "--branches" "--prune"
)

# Safe defaults to always include
SAFE_DEFAULTS=(
    "--atomic"
    "--follow-tags"
    "--porcelain"
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
        "push")
            echo -e "${PURPLE}${ICON_PUSH} ${message}${NC}"
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

# Function to check if branch is protected
is_protected_branch() {
    local branch=$1
    case $branch in
        "main"|"master"|"develop"|"production"|"staging")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to validate arguments and reject forbidden flags
validate_arguments() {
    local args=("$@")
    local forbidden_found=()
    
    for arg in "${args[@]}"; do
        for forbidden in "${FORBIDDEN_FLAGS[@]}"; do
            if [[ "$arg" == "$forbidden" ]] || [[ "$arg" =~ ^"$forbidden"= ]]; then
                forbidden_found+=("$arg")
            fi
        done
    done
    
    if [[ ${#forbidden_found[@]} -gt 0 ]]; then
        print_status "security" "Forbidden flags detected:"
        for flag in "${forbidden_found[@]}"; do
            echo "  • $flag"
        done
        print_status "error" "These flags are not allowed in safe-push"
        print_status "info" "Use 'git push' directly if you really need these flags"
        
        log_event "security_violation" "Forbidden flags detected" "{\"forbidden_flags\": [$(printf '"%s",' "${forbidden_found[@]}" | sed 's/,$//')]}"
        exit 1
    fi
}

# Function to check for uncommitted changes
check_uncommitted_changes() {
    if ! git diff-index --quiet HEAD --; then
        print_status "warning" "You have uncommitted changes"
        print_status "info" "Consider committing or stashing changes before pushing"
        
        log_event "warning" "Uncommitted changes detected" '{"has_uncommitted": true}'
        
        read -p "Continue with push anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "info" "Push cancelled by user"
            log_event "cancelled" "Push cancelled by user due to uncommitted changes"
            exit 0
        fi
    else
        log_event "check" "Working directory is clean" '{"has_uncommitted": false}'
    fi
}

# Function to check for protected branch pushes
check_protected_branch() {
    local current_branch=$(get_current_branch)
    
    if is_protected_branch "$current_branch"; then
        print_status "security" "Direct push to protected branch '$current_branch' detected!"
        print_status "error" "Direct pushes to protected branches are forbidden"
        print_status "info" "Use pull requests or merge requests instead"
        
        log_event "security_violation" "Attempted push to protected branch" "{\"branch\": \"$current_branch\", \"protected\": true}"
        exit 1
    fi
    
    log_event "check" "Not pushing to protected branch" "{\"branch\": \"$current_branch\", \"protected\": false}"
}

# Function to check if upstream is set
check_upstream() {
    local current_branch=$(get_current_branch)
    local upstream=$(git rev-parse --abbrev-ref @{u} 2>/dev/null || echo "")
    
    if [[ -z "$upstream" ]]; then
        print_status "warning" "No upstream branch set for '$current_branch'"
        print_status "info" "Setting upstream to origin/$current_branch"
        
        log_event "upstream_set" "Setting upstream branch" "{\"branch\": \"$current_branch\", \"upstream\": \"origin/$current_branch\"}"
        
        git branch --set-upstream-to="origin/$current_branch" "$current_branch"
    else
        log_event "check" "Upstream branch is set" "{\"branch\": \"$current_branch\", \"upstream\": \"$upstream\"}"
    fi
}

# Function to check for force push attempts
check_force_push() {
    local current_branch=$(get_current_branch)
    local upstream=$(git rev-parse --abbrev-ref @{u} 2>/dev/null || echo "")
    
    if [[ -n "$upstream" ]]; then
        local local_count=$(git rev-list --count "$upstream"..HEAD 2>/dev/null || echo "0")
        local remote_count=$(git rev-list --count HEAD.."$upstream" 2>/dev/null || echo "0")
        
        if [[ $remote_count -gt 0 ]]; then
            print_status "security" "Force push detected! Remote has $remote_count commit(s) ahead"
            print_status "error" "Force pushes are not allowed for safety"
            print_status "info" "Pull and rebase first, then push"
            
            log_event "security_violation" "Force push detected" "{\"local_ahead\": $local_count, \"remote_ahead\": $remote_count}"
            exit 1
        fi
        
        log_event "check" "No force push detected" "{\"local_ahead\": $local_count, \"remote_ahead\": $remote_count}"
    fi
}

# Function to build safe push command
build_push_command() {
    local args=("$@")
    local push_args=("push")
    
    # Add safe defaults
    push_args+=("${SAFE_DEFAULTS[@]}")
    
    # Add user arguments (already validated)
    push_args+=("${args[@]}")
    
    echo "${push_args[@]}"
}

# Function to parse porcelain output
parse_porcelain_output() {
    local output="$1"
    local results=()
    
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            # Parse porcelain format: <ref> <status> <summary>
            local parts=($line)
            if [[ ${#parts[@]} -ge 2 ]]; then
                local ref="${parts[0]}"
                local status="${parts[1]}"
                local summary="${parts[2]:-}"
                
                results+=("{\"ref\": \"$ref\", \"status\": \"$status\", \"summary\": \"$summary\"}")
            fi
        fi
    done <<< "$output"
    
    echo "[$(IFS=,; echo "${results[*]}")]"
}

# Function to execute push with retries
execute_push() {
    local push_cmd="$1"
    local retry_count=0
    
    while [[ $retry_count -lt $MAX_RETRIES ]]; do
        print_status "push" "Executing: git $push_cmd"
        log_event "push_start" "Starting push execution" "{\"command\": \"git $push_cmd\", \"attempt\": $((retry_count + 1))}"
        
        # Execute the push command
        local output
        local exit_code
        
        if output=$(git $push_cmd 2>&1); then
            # Success
            print_status "success" "Push completed successfully"
            
            # Parse porcelain output
            local parsed_results=$(parse_porcelain_output "$output")
            log_event "push_success" "Push completed successfully" "{\"results\": $parsed_results}"
            
            # Display results in a user-friendly way
            echo ""
            print_status "info" "Push Results:"
            echo "=================="
            while IFS= read -r line; do
                if [[ -n "$line" ]]; then
                    local parts=($line)
                    if [[ ${#parts[@]} -ge 2 ]]; then
                        local ref="${parts[0]}"
                        local status="${parts[1]}"
                        local summary="${parts[2]:-}"
                        
                        case "$status" in
                            "ok")
                                print_status "success" "$ref: Successfully pushed"
                                ;;
                            "up to date")
                                print_status "info" "$ref: Already up to date"
                                ;;
                            "rejected")
                                print_status "error" "$ref: Rejected - $summary"
                                ;;
                            *)
                                print_status "info" "$ref: $status - $summary"
                                ;;
                        esac
                    fi
                fi
            done <<< "$output"
            
            return 0
        else
            exit_code=$?
            retry_count=$((retry_count + 1))
            
            print_status "error" "Push failed (attempt $retry_count/$MAX_RETRIES)"
            log_event "push_failure" "Push failed" "{\"attempt\": $retry_count, \"exit_code\": $exit_code, \"output\": \"$output\"}"
            
            if [[ $retry_count -lt $MAX_RETRIES ]]; then
                print_status "info" "Retrying in $RETRY_DELAY seconds..."
                sleep $RETRY_DELAY
            else
                print_status "error" "Push failed after $MAX_RETRIES attempts"
                log_event "push_final_failure" "Push failed after maximum retries" "{\"max_retries\": $MAX_RETRIES, \"exit_code\": $exit_code}"
                return 1
            fi
        fi
    done
}

# Function to show usage
show_usage() {
    cat << EOF
Safe Push Script v$SCRIPT_VERSION

Usage: $0 [OPTIONS] [REMOTE] [BRANCH]

DESCRIPTION:
    Safe git push with enforced constraints and helpful defaults.
    Prevents dangerous operations and provides structured logging.

OPTIONS:
    -h, --help          Show this help message
    -v, --version       Show version information
    -l, --log-file      Specify custom log file (default: $LOG_FILE)
    --dry-run           Show what would be executed without running
    --verbose           Enable verbose output

EXAMPLES:
    $0                    # Push current branch to origin
    $0 origin main       # Push main branch to origin
    $0 --verbose         # Push with verbose output

SAFETY FEATURES:
    • Blocks dangerous flags (--force, --no-verify, etc.)
    • Prevents pushes to protected branches
    • Uses --atomic and --follow-tags by default
    • Provides structured JSONL logging
    • Retries failed pushes automatically

FORBIDDEN FLAGS:
    --force, -f, --force-with-lease, --no-verify, -n
    --delete, -d, --mirror, --all, --tags, --branches

LOG FILE:
    Events are logged to: $LOG_FILE

EOF
}

# Function to show version
show_version() {
    echo "Safe Push Script v$SCRIPT_VERSION"
    echo "Git repository: $(git rev-parse --show-toplevel 2>/dev/null || echo 'unknown')"
    echo "Current branch: $(get_current_branch 2>/dev/null || echo 'unknown')"
    echo "Log file: $LOG_FILE"
}

# Main function
main() {
    local args=()
    local dry_run=false
    local verbose=false
    
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
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
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
    log_event "script_start" "Safe push script started" "{\"version\": \"$SCRIPT_VERSION\", \"args\": [$(printf '"%s",' "${args[@]}" | sed 's/,$//')]}"
    
    # Check if we're in a git repository
    check_git_repo
    
    # Validate arguments
    validate_arguments "${args[@]}"
    
    # Run safety checks
    check_uncommitted_changes
    check_protected_branch
    check_upstream
    check_force_push
    
    # Build push command
    local push_cmd=$(build_push_command "${args[@]}")
    
    if [[ "$verbose" == "true" ]]; then
        print_status "info" "Push command: git $push_cmd"
    fi
    
    if [[ "$dry_run" == "true" ]]; then
        print_status "info" "DRY RUN: Would execute: git $push_cmd"
        log_event "dry_run" "Dry run completed" "{\"command\": \"git $push_cmd\"}"
        exit 0
    fi
    
    # Execute push
    if execute_push "$push_cmd"; then
        print_status "success" "Safe push completed successfully!"
        log_event "script_success" "Safe push script completed successfully"
        exit 0
    else
        print_status "error" "Safe push failed!"
        log_event "script_failure" "Safe push script failed"
        exit 1
    fi
}

# Run main function with all arguments
main "$@" 
