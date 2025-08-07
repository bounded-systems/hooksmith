#!/usr/bin/env bash

set -euo pipefail

# CI script for .gitattributes generation
# This script wraps the generate-gitattributes.rs Rust script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -c, --check     Check if .gitattributes needs updating (returns 0 if up-to-date, 1 if needs update)
    -f, --force     Force update .gitattributes
    -h, --help      Show this help message

Examples:
    $0 -c          # Check if update is needed
    $0 -f          # Force update .gitattributes
EOF
}

# Function to check if hyperpolyglot is available
check_hyperpolyglot() {
    if ! command -v hyply &> /dev/null; then
        print_status $RED "❌ Error: hyperpolyglot (hyply) is not installed"
        print_status $YELLOW "💡 Install it with: cargo install hyperpolyglot"
        exit 1
    fi
}

# Function to check if .gitattributes needs updating
check_gitattributes() {
    print_status $BLUE "🔍 Checking if .gitattributes needs updating..."
    
    # Create temporary files for comparison
    local temp_bin=$(mktemp)
    local temp_output=$(mktemp)
    trap "rm -f '$temp_bin' '$temp_output'" EXIT
    
    # Generate new .gitattributes to temp file
    cd "$PROJECT_ROOT"
    if rustc --version &> /dev/null; then
        rustc -o "$temp_bin" scripts/generate-gitattributes.rs 2>/dev/null || {
            print_status $RED "❌ Error: Failed to compile generate-gitattributes.rs"
            return 1
        }
        "$temp_bin" "$temp_output" > /dev/null 2>&1 || {
            print_status $RED "❌ Error: Failed to run generate-gitattributes"
            return 1
        }
    else
        print_status $RED "❌ Error: Rust compiler not found"
        return 1
    fi
    
    # Compare with existing .gitattributes
    if [[ -f ".gitattributes" ]]; then
        if diff -q "$temp_output" ".gitattributes" > /dev/null 2>&1; then
            print_status $GREEN "✅ .gitattributes is up to date"
            return 0
        else
            print_status $YELLOW "⚠️  .gitattributes needs updating"
            return 1
        fi
    else
        print_status $YELLOW "⚠️  .gitattributes file not found, needs to be created"
        return 1
    fi
}

# Function to force update .gitattributes
force_update() {
    print_status $BLUE "🔄 Force updating .gitattributes..."
    
    cd "$PROJECT_ROOT"
    
    # Backup existing .gitattributes if it exists
    if [[ -f ".gitattributes" ]]; then
        cp ".gitattributes" ".gitattributes.backup"
        print_status $BLUE "📋 Backed up existing .gitattributes"
    fi
    
    # Compile and run the generate-gitattributes script
    if rustc --version &> /dev/null; then
        local temp_bin=$(mktemp)
        trap "rm -f '$temp_bin'" EXIT
        
        if rustc -o "$temp_bin" scripts/generate-gitattributes.rs; then
            "$temp_bin" ".gitattributes"
            print_status $GREEN "✅ Successfully updated .gitattributes"
        else
            print_status $RED "❌ Error: Failed to compile generate-gitattributes.rs"
            return 1
        fi
    else
        print_status $RED "❌ Error: Rust compiler not found"
        return 1
    fi
}

# Main script logic
main() {
    # Parse command line arguments
    local check_mode=false
    local force_mode=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -c|--check)
                check_mode=true
                shift
                ;;
            -f|--force)
                force_mode=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_status $RED "❌ Error: Unknown option '$1'"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Check if at least one mode is specified
    if [[ "$check_mode" == false && "$force_mode" == false ]]; then
        print_status $RED "❌ Error: Must specify either -c (check) or -f (force)"
        show_usage
        exit 1
    fi
    
    # Check if both modes are specified
    if [[ "$check_mode" == true && "$force_mode" == true ]]; then
        print_status $RED "❌ Error: Cannot specify both check and force modes"
        exit 1
    fi
    
    # Check dependencies
    check_hyperpolyglot
    
    # Execute requested mode
    if [[ "$check_mode" == true ]]; then
        check_gitattributes
        exit $?
    elif [[ "$force_mode" == true ]]; then
        force_update
        exit $?
    fi
}

# Run main function with all arguments
main "$@"
