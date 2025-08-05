#!/bin/bash

# Intelligent Worktree Cleanup
# Analyzes worktrees and makes decisions about their fate

set -e

echo "🧠 INTELLIGENT WORKTREE CLEANUP"
echo "==============================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_status() {
    local state=$1
    local message=$2
    case $state in
        "ERROR") echo -e "${RED}❌ $message${NC}" ;;
        "SUCCESS") echo -e "${GREEN}✅ $message${NC}" ;;
        "WARNING") echo -e "${YELLOW}⚠️  $message${NC}" ;;
        "INFO") echo -e "${BLUE}ℹ️  $message${NC}" ;;
        "DECISION") echo -e "${PURPLE}🤔 $message${NC}" ;;
        *) echo "📝 $message" ;;
    esac
}

# Function to analyze worktree and make decision
analyze_worktree() {
    local worktree_path=$1
    local worktree_name=$(basename "$worktree_path")
    
    echo "=== ANALYZING: $worktree_name ==="
    
    if [ ! -d "$worktree_path" ]; then
        print_status "ERROR" "Worktree $worktree_name does not exist"
        return
    fi
    
    cd "$worktree_path"
    
    # Get branch name
    local branch=$(git branch --show-current)
    print_status "INFO" "Branch: $branch"
    
    # Check commit history
    local commit_count=$(git log --oneline --since="1 week ago" | wc -l)
    local last_commit=$(git log --oneline -1)
    print_status "INFO" "Recent commits: $commit_count"
    print_status "INFO" "Last commit: $last_commit"
    
    # Check for conflicts
    local conflicts=$(git diff --name-only --diff-filter=U 2>/dev/null || true)
    local rebase_status=$(git status | grep "rebase" || true)
    
    if [ -n "$conflicts" ] || [ -n "$rebase_status" ]; then
        print_status "WARNING" "Worktree has conflicts or is in rebase state"
        
        # Check if this worktree is from old development
        local worktree_date=$(echo "$worktree_name" | grep -o "202508[0-9][0-9]" || echo "")
        if [ -n "$worktree_date" ]; then
            print_status "DECISION" "This appears to be an old worktree from $worktree_date"
            print_status "DECISION" "Recommendation: REMOVE (likely obsolete)"
            echo "REMOVE"
            cd - > /dev/null
            return
        fi
    fi
    
    # Check if branch is behind main
    local behind_count=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
    if [ "$behind_count" -gt 5 ]; then
        print_status "WARNING" "Branch is $behind_count commits behind main"
        print_status "DECISION" "Recommendation: REMOVE (too far behind)"
        echo "REMOVE"
        cd - > /dev/null
        return
    fi
    
    # Check if branch exists on origin
    if git ls-remote --heads origin "$branch" | grep -q "$branch"; then
        print_status "INFO" "Branch exists on origin"
        
        # Check if merged
        if git branch --merged origin/main | grep -q "$branch"; then
            print_status "SUCCESS" "Branch is merged"
            print_status "DECISION" "Recommendation: CLEANUP (merged)"
            echo "CLEANUP"
            cd - > /dev/null
            return
        else
            print_status "INFO" "Branch not merged"
            print_status "DECISION" "Recommendation: KEEP (active development)"
            echo "KEEP"
            cd - > /dev/null
            return
        fi
    else
        print_status "WARNING" "Branch does not exist on origin"
        print_status "DECISION" "Recommendation: REMOVE (no remote branch)"
        echo "REMOVE"
        cd - > /dev/null
        return
    fi
}

# Function to execute decision
execute_decision() {
    local worktree_path=$1
    local decision=$2
    local worktree_name=$(basename "$worktree_path")
    
    case $decision in
        "REMOVE")
            print_status "INFO" "Removing worktree $worktree_name"
            cd "$worktree_path"
            local branch=$(git branch --show-current)
            cd ..
            
            # Abort any ongoing operations
            cd "$worktree_path"
            if git status | grep -q "rebase"; then
                git rebase --abort 2>/dev/null || true
            fi
            cd ..
            
            # Remove worktree
            git worktree remove "$worktree_name" --force 2>/dev/null || {
                print_status "WARNING" "Could not remove worktree, trying to delete directory"
                rm -rf "$worktree_name"
            }
            
            # Remove branch if it exists
            if [ -n "$branch" ]; then
                git branch -D "$branch" 2>/dev/null || true
            fi
            
            print_status "SUCCESS" "Removed worktree $worktree_name"
            ;;
        "CLEANUP")
            print_status "INFO" "Cleaning up merged worktree $worktree_name"
            cd "$worktree_path"
            local branch=$(git branch --show-current)
            cd ..
            
            if [ -n "$branch" ]; then
                git worktree remove "$worktree_name" --force
                git branch -d "$branch" 2>/dev/null || true
                print_status "SUCCESS" "Cleaned up worktree $worktree_name"
            fi
            ;;
        "KEEP")
            print_status "INFO" "Keeping worktree $worktree_name"
            ;;
    esac
}

# Function to create PR for ready worktrees
create_prs_for_ready() {
    echo ""
    echo "🚀 CREATING PRS FOR READY WORKTREES"
    echo "==================================="
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2 | grep -v "^$(pwd)$")
    
    for worktree in $worktrees; do
        if [ -d "$worktree" ]; then
            cd "$worktree"
            
            # Check if ready for PR
            local behind_count=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
            local uncommitted=$(git status --porcelain)
            local branch=$(git branch --show-current)
            
            if [ "$behind_count" -eq 0 ] && [ -z "$uncommitted" ] && [ -n "$branch" ]; then
                if git ls-remote --heads origin "$branch" | grep -q "$branch"; then
                    print_status "SUCCESS" "Creating PR for $worktree"
                    local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
                    if [[ "$repo_url" == *"github.com"* ]]; then
                        local pr_url="$repo_url/compare/main...$branch"
                        print_status "INFO" "Create PR at: $pr_url"
                    fi
                fi
            fi
            
            cd - > /dev/null
        fi
    done
}

# Main function
main() {
    echo "🔍 Analyzing all worktrees..."
    echo ""
    
    # Get list of worktrees
    local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2 | grep -v "^$(pwd)$")
    
    local decisions=()
    
    for worktree in $worktrees; do
        if [ -d "$worktree" ]; then
            local decision=$(analyze_worktree "$worktree")
            decisions+=("$worktree:$decision")
            echo ""
        fi
    done
    
    echo "📋 DECISIONS SUMMARY"
    echo "===================="
    for decision in "${decisions[@]}"; do
        local worktree=$(echo "$decision" | cut -d: -f1)
        local action=$(echo "$decision" | cut -d: -f2)
        local worktree_name=$(basename "$worktree")
        print_status "INFO" "$worktree_name: $action"
    done
    
    echo ""
    echo "🚀 EXECUTING DECISIONS"
    echo "======================"
    
    for decision in "${decisions[@]}"; do
        local worktree=$(echo "$decision" | cut -d: -f1)
        local action=$(echo "$decision" | cut -d: -f2)
        execute_decision "$worktree" "$action"
    done
    
    # Create PRs for ready worktrees
    create_prs_for_ready
    
    echo ""
    echo "🎉 Intelligent cleanup completed!"
    echo ""
    echo "📊 Final Status:"
    ./scripts/worktree-status-report.sh
}

# Run main function
main "$@" 
