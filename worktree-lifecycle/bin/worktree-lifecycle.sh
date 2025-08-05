#!/bin/bash

# Worktree Lifecycle CLI
# Main entry point for worktree lifecycle management

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_header() {
    echo -e "${PURPLE}=== $1 ===${NC}"
}

# Function to show usage
show_usage() {
    cat << EOF
Worktree Lifecycle CLI v1.0.0

Usage: $0 <command> [options]

Commands:
  status              Show comprehensive worktree status
  create              Create a new worktree for development
  sync                Sync all remote branches (main to base, others to worktrees)
  update-to-main      Update all worktrees to origin/main
  detect-orphaned     Detect branches not in worktrees
  process             Process all worktrees through state machine
  create-prs          Create PRs for ready worktrees
  auto-merge          Auto merge all PRs for worktrees
  resolve-conflicts   Resolve conflicts in worktrees
  cleanup             Clean up obsolete worktrees
  demo                Demonstrate complete workflow

Options:
  --json              Output in JSON format
  --dry-run           Show what would be done without making changes
  --verbose           Show detailed output
  --quiet             Suppress non-error output
  --help              Show this usage information

Examples:
  $0 status                    # Show worktree status
  $0 status --json            # Show status in JSON format
  $0 create feature/new-feature  # Create new worktree
  $0 sync --dry-run           # Sync all branches (main to base, others to worktrees)
  $0 update-to-main --dry-run # Update all worktrees to origin/main
  $0 detect-orphaned          # Detect branches not in worktrees
  $0 process --dry-run        # Show what would be processed
  $0 create-prs               # Create PRs for ready worktrees
  $0 auto-merge --dry-run     # Auto merge all PRs (dry run)
  $0 resolve-conflicts        # Resolve conflicts
  $0 cleanup                  # Clean up obsolete worktrees
  $0 demo                     # Demonstrate complete workflow

State Machine:
  CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED
      ↓         ↓
  CONFLICTED → RESOLVING

For more information, see: $PROJECT_DIR/docs/COMPREHENSIVE_WORKTREE_WORKFLOW.md
EOF
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI (gh) not found. PR creation will be limited."
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Function to update all worktrees to main
run_update_to_main() {
    local dry_run=false
    local create_prs=false
    local force=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --create-prs)
                create_prs=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # Build arguments for the update script
    local script_args=""
    if [ "$dry_run" = true ]; then
        script_args="$script_args --dry-run"
    fi
    if [ "$create_prs" = true ]; then
        script_args="$script_args --create-prs"
    fi
    if [ "$force" = true ]; then
        script_args="$script_args --force"
    fi
    
    # Run the update script
    "$PROJECT_DIR/../scripts/update-all-worktrees-to-main.sh" $script_args
}

# Function to detect orphaned branches
run_detect_orphaned() {
    local create_worktrees=false
    local delete_branches=false
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --create-worktrees)
                create_worktrees=true
                shift
                ;;
            --delete-branches)
                delete_branches=true
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
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # Build arguments for the detect-orphaned script
    local script_args=""
    if [ "$create_worktrees" = true ]; then
        script_args="$script_args --create-worktrees"
    fi
    if [ "$delete_branches" = true ]; then
        script_args="$script_args --delete-branches"
    fi
    if [ "$dry_run" = true ]; then
        script_args="$script_args --dry-run"
    fi
    
    # Run the detect-orphaned script
    "$PROJECT_DIR/../scripts/detect-orphaned-branches.sh" $script_args
}

# Function to sync all remote branches
run_sync() {
    local dry_run=false
    local skip_main=false
    local force=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --skip-main)
                skip_main=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # Build arguments for the sync script
    local script_args=""
    if [ "$dry_run" = true ]; then
        script_args="$script_args --dry-run"
    fi
    if [ "$skip_main" = true ]; then
        script_args="$script_args --skip-main"
    fi
    if [ "$force" = true ]; then
        script_args="$script_args --force"
    fi
    
    # Run the sync script
    "$PROJECT_DIR/../scripts/sync-all-remote-branches.sh" $script_args
}

# Function to create new worktree
run_create() {
    local branch_name=""
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options and get branch name
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            -*)
                shift
                ;;
            *)
                if [ -z "$branch_name" ]; then
                    branch_name="$1"
                fi
                shift
                ;;
        esac
    done
    
    if [ -z "$branch_name" ]; then
        log_error "Branch name is required. Usage: $0 create <branch-name>"
        exit 1
    fi
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would create worktree for branch: $branch_name"
        return 0
    fi
    
    # Create worktree directory path
    local worktree_path="worktrees/${branch_name//\//\/}"
    
    log_info "Creating worktree for branch: $branch_name"
    log_info "Worktree path: $worktree_path"
    
    # Check if worktree already exists
    if git worktree list | grep -q "$worktree_path"; then
        log_warning "Worktree already exists at: $worktree_path"
        return 1
    fi
    
    # Create the worktree
    if git worktree add "$worktree_path" -b "$branch_name"; then
        log_success "Successfully created worktree for branch: $branch_name"
        log_info "Worktree location: $worktree_path"
        log_info "Next steps:"
        log_info "  cd $worktree_path"
        log_info "  # Make your changes"
        log_info "  git add . && git commit -m 'your message'"
        log_info "  git push -u origin $branch_name"
        return 0
    else
        log_error "Failed to create worktree for branch: $branch_name"
        return 1
    fi
}

# Function to run status report
run_status() {
    local json_output=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --json)
                json_output=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [ "$json_output" = true ]; then
        # TODO: Implement JSON output
        log_info "JSON output not yet implemented"
        return 1
    fi
    
    "$PROJECT_DIR/scripts/status_report.sh"
}

# Function to process worktrees
run_process() {
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would process worktrees through state machine"
        # TODO: Implement dry-run mode
        return 0
    fi
    
    # Source the state machine library
    source "$PROJECT_DIR/lib/state_machine.sh"
    
    # Run the state machine process
    process_all_worktrees
}

# Function to create PRs
run_create_prs() {
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would create PRs for ready worktrees"
        # TODO: Implement dry-run mode
        return 0
    fi
    
    "$PROJECT_DIR/scripts/pr_creator.sh"
}

# Function to auto merge PRs
run_auto_merge() {
    local dry_run=false
    local verbose=false
    local quiet=false
    local force=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # Build arguments for the auto-merge script
    local script_args=""
    if [ "$dry_run" = true ]; then
        script_args="$script_args --dry-run"
    fi
    if [ "$force" = true ]; then
        script_args="$script_args --force"
    fi
    
    # Run the auto-merge script
    "$PROJECT_DIR/../scripts/auto-merge-all-prs.sh" $script_args
}

# Function to resolve conflicts
run_resolve_conflicts() {
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would resolve conflicts in worktrees"
        # TODO: Implement dry-run mode
        return 0
    fi
    
    "$PROJECT_DIR/scripts/conflict_resolver.sh"
}

# Function to cleanup worktrees
run_cleanup() {
    local dry_run=false
    local verbose=false
    local quiet=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --quiet)
                quiet=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [ "$dry_run" = true ]; then
        log_info "DRY RUN: Would clean up obsolete worktrees"
        # TODO: Implement dry-run mode
        return 0
    fi
    
    # TODO: Implement cleanup functionality
    log_info "Cleanup functionality not yet implemented"
    return 1
}

# Function to run demo
run_demo() {
    log_header "WORKTREE LIFECYCLE DEMO"
    echo ""
    
    log_info "Step 1: Checking current status"
    run_status
    echo ""
    
    log_info "Step 2: Processing worktrees through state machine"
    run_process
    echo ""
    
    log_info "Step 3: Creating PRs for ready worktrees"
    run_create_prs
    echo ""
    
    log_success "Demo completed!"
}

# Main execution
main() {
    # Check dependencies
    check_dependencies
    
    # Parse command
    local command="${1:-help}"
    shift 2>/dev/null || true
    
    case "$command" in
        "status")
            run_status "$@"
            ;;
        "create")
            run_create "$@"
            ;;
        "sync")
            run_sync "$@"
            ;;
        "update-to-main")
            run_update_to_main "$@"
            ;;
        "detect-orphaned")
            run_detect_orphaned "$@"
            ;;
        "process")
            run_process "$@"
            ;;
        "create-prs")
            run_create_prs "$@"
            ;;
        "auto-merge")
            run_auto_merge "$@"
            ;;
        "resolve-conflicts")
            run_resolve_conflicts "$@"
            ;;
        "cleanup")
            run_cleanup "$@"
            ;;
        "demo")
            run_demo "$@"
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function
main "$@" 
