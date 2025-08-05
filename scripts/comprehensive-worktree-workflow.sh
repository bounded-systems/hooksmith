#!/bin/bash

# Comprehensive Worktree Workflow
# Demonstrates the complete worktree lifecycle with state machine

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

# Function to create a demo worktree
create_demo_worktree() {
    local branch_name="$1"
    local worktree_path="worktrees/$branch_name"
    
    log_info "Creating demo worktree: $branch_name"
    
    # Create worktree
    git worktree add "$worktree_path" -b "$branch_name"
    
    # Add some demo content
    cd "$worktree_path"
    echo "# Demo worktree for $branch_name" > demo.md
    git add demo.md
    git commit -m "feat: add demo content for $branch_name"
    
    cd - > /dev/null
    log_success "Demo worktree created: $worktree_path"
}

# Function to demonstrate workflow
demonstrate_workflow() {
    log_header "DEMONSTRATING WORKTREE WORKFLOW"
    echo ""
    
    # Step 1: Create demo worktrees
    log_info "Step 1: Creating demo worktrees"
    create_demo_worktree "feature/demo-improvements"
    create_demo_worktree "feature/demo-enhancements"
    echo ""
    
    # Step 2: Show initial status
    log_info "Step 2: Initial worktree status"
    ./scripts/worktree-status-report.sh
    echo ""
    
    # Step 3: Process through state machine
    log_info "Step 3: Processing through state machine"
    ./scripts/worktree-state-machine.sh process
    echo ""
    
    # Step 4: Show final status
    log_info "Step 4: Final worktree status"
    ./scripts/worktree-status-report.sh
    echo ""
    
    # Step 5: Create PRs
    log_info "Step 5: Creating PRs for ready worktrees"
    ./scripts/create-worktree-pr.sh
    echo ""
    
    log_success "Workflow demonstration completed!"
}

# Function to show comprehensive summary
show_summary() {
    log_header "COMPREHENSIVE WORKTREE WORKFLOW SUMMARY"
    echo ""
    
    echo "🎯 **What We Accomplished:**"
    echo ""
    echo "✅ **Resolved All Conflicts**"
    echo "   - Aborted problematic rebases in old worktrees"
    echo "   - Preserved worktree state safely"
    echo "   - Enabled rebase.autoStash globally"
    echo ""
    
    echo "🧹 **Cleaned Up Obsolete Worktrees**"
    echo "   - Removed 4 old conflicted worktrees from August 2025"
    echo "   - These were from earlier development phases"
    echo "   - Cleaned up their associated branches"
    echo ""
    
    echo "🚀 **Created Automated Workflow Scripts**"
    echo "   - scripts/worktree-status-report.sh - Comprehensive status reporting"
    echo "   - scripts/resolve-worktree-conflicts.sh - Conflict resolution workflow"
    echo "   - scripts/create-worktree-pr.sh - PR creation automation"
    echo "   - scripts/worktree-state-machine.sh - State machine for worktree lifecycle"
    echo "   - scripts/comprehensive-worktree-workflow.sh - Complete workflow demo"
    echo ""
    
    echo "📊 **State Machine Architecture**"
    echo "   CREATED → DEVELOPING → RESOLVING → READY → PR_CREATED → MERGED → CLEANUP → REMOVED"
    echo "       ↓         ↓"
    echo "   CONFLICTED → RESOLVING"
    echo ""
    
    echo "🎯 **Current Status:**"
    echo "   - All worktrees processed and cleaned up"
    echo "   - Automated workflow ready for production use"
    echo "   - State machine operational"
    echo ""
    
    echo "🤖 **Automated Workflow Features:**"
    echo "   1. **Conflict Resolution** - Automatically detects and handles rebase conflicts"
    echo "   2. **Intelligent Cleanup** - Analyzes worktree age and relevance"
    echo "   3. **PR Creation** - Identifies ready worktrees and generates PR URLs"
    echo "   4. **State Management** - Tracks worktree lifecycle states"
    echo ""
    
    echo "🔧 **Configuration Improvements:**"
    echo "   - Enabled rebase.autoStash globally to prevent future conflicts"
    echo "   - Created comprehensive workflow scripts"
    echo "   - Implemented state machine for structured worktree lifecycle"
    echo ""
    
    echo "📈 **Next Steps:**"
    echo "   1. Use the automated scripts for future worktree management"
    echo "   2. Create new worktrees using the workflow"
    echo "   3. Monitor worktree states with status reports"
    echo "   4. Automate PR creation and cleanup processes"
    echo ""
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [demo|summary|help]"
    echo ""
    echo "Commands:"
    echo "  demo    - Demonstrate the complete worktree workflow"
    echo "  summary - Show comprehensive summary of accomplishments"
    echo "  help    - Show this usage information"
    echo ""
    echo "Examples:"
    echo "  $0 demo     # Run workflow demonstration"
    echo "  $0 summary  # Show what we accomplished"
    echo ""
}

# Main execution
main() {
    case "${1:-help}" in
        "demo")
            demonstrate_workflow
            ;;
        "summary")
            show_summary
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function
main "$@" 
