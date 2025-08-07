#!/bin/bash

# ci-gitattributes.sh
# CI/CD integration script for generating .gitattributes using hyperpolyglot
# 
# This script is designed to run in CI environments and automatically
# update .gitattributes when file types change in the repository.
#
# Usage:
#   ./scripts/ci-gitattributes.sh [options]
#
# Options:
#   -c, --check-only     Only check if .gitattributes needs updating
#   -f, --force          Force update even if no changes detected
#   -v, --verbose        Verbose output
#   -h, --help           Show this help message

set -euo pipefail

# Default values
CHECK_ONLY=false
FORCE=false
VERBOSE=false

# Colors for output (only if not in CI)
if [[ -t 1 ]] && [[ -z "${CI:-}" ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Help function
show_help() {
    cat << EOF
ci-gitattributes.sh - CI/CD integration for .gitattributes generation

Usage:
    $0 [options]

Options:
    -c, --check-only     Only check if .gitattributes needs updating
    -f, --force          Force update even if no changes detected
    -v, --verbose        Verbose output
    -h, --help           Show this help message

Environment Variables:
    CI                   Set to any value to enable CI mode (no colors, no prompts)
    GITHUB_ACTIONS       Set by GitHub Actions
    GITLAB_CI           Set by GitLab CI
    CIRCLE_CI           Set by CircleCI

Examples:
    $0                    # Generate .gitattributes if needed
    $0 -c                # Check if update is needed
    $0 -f                # Force update
    $0 -v                # Verbose output

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--check-only)
            CHECK_ONLY=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}" >&2
            show_help
            exit 1
            ;;
    esac
done

# Log function
log() {
    local level="$1"
    shift
    local message="$*"
    echo -e "${level}${message}${NC}"
}

# Verbose log function
vlog() {
    if [[ "$VERBOSE" == true ]]; then
        log "$BLUE" "[VERBOSE] $*"
    fi
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log "$RED" "Error: Not in a git repository"
    exit 1
fi

# Check if hyperpolyglot is installed
if ! command -v hyply > /dev/null 2>&1; then
    log "$YELLOW" "Installing hyperpolyglot..."
    cargo install hyperpolyglot
fi

# Get the script directory and binary path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_SCRIPT="$SCRIPT_DIR/generate-gitattributes.rs"
BINARY="$SCRIPT_DIR/generate-gitattributes"

# Compile the Rust script if needed
if [[ ! -f "$BINARY" ]] || [[ "$RUST_SCRIPT" -nt "$BINARY" ]]; then
    vlog "Compiling Rust script..."
    rustc "$RUST_SCRIPT" -o "$BINARY"
fi

# Function to generate a checksum of file types in the repository
generate_file_type_checksum() {
    # Get all tracked files and their extensions
    git ls-files --cached --full-name | \
        grep -E '\.[a-zA-Z0-9]+$' | \
        sed 's/.*\././' | \
        sort | uniq -c | \
        sha256sum | cut -d' ' -f1
}

# Function to check if .gitattributes needs updating
check_if_update_needed() {
    local current_checksum
    local new_checksum
    
    # Get current checksum from .gitattributes if it exists
    if [[ -f ".gitattributes" ]]; then
        current_checksum=$(grep -E '^# @checksum:' .gitattributes | head -1 | cut -d' ' -f3 || echo "")
    else
        current_checksum=""
    fi
    
    # Generate new checksum
    new_checksum=$(generate_file_type_checksum)
    
    vlog "Current checksum: ${current_checksum:-none}"
    vlog "New checksum: $new_checksum"
    
    if [[ "$current_checksum" == "$new_checksum" ]]; then
        log "$GREEN" "✅ .gitattributes is up to date"
        return 0
    else
        log "$YELLOW" "⚠️  .gitattributes needs updating"
        return 1
    fi
}

# Main execution
main() {
    log "$BLUE" "🔍 Checking .gitattributes status..."
    
    if check_if_update_needed; then
        if [[ "$FORCE" == true ]]; then
            log "$YELLOW" "⚠️  Force update requested"
        else
            log "$GREEN" "✅ No update needed"
            exit 0
        fi
    fi
    
    if [[ "$CHECK_ONLY" == true ]]; then
        log "$YELLOW" "📋 Check-only mode: .gitattributes needs updating"
        exit 1
    fi
    
    # Create backup if in CI mode
    if [[ -f ".gitattributes" ]] && [[ -n "${CI:-}" ]]; then
        cp .gitattributes .gitattributes.backup
        vlog "Created backup: .gitattributes.backup"
    fi
    
    # Generate new .gitattributes
    log "$BLUE" "🔄 Generating new .gitattributes..."
    "$BINARY" ".gitattributes"
    
    # Update checksum in the generated file
    local new_checksum=$(generate_file_type_checksum)
    sed -i.bak "s/^# @checksum:.*/# @checksum: $new_checksum/" .gitattributes
    rm -f .gitattributes.bak
    
    log "$GREEN" "✅ Successfully updated .gitattributes"
    
    # Show summary
    if [[ -f ".gitattributes" ]]; then
        local line_count=$(wc -l < .gitattributes)
        local file_size=$(du -h .gitattributes | cut -f1)
        log "$BLUE" "📊 File stats: $line_count lines, $file_size"
    fi
    
    # In CI mode, suggest committing the changes
    if [[ -n "${CI:-}" ]]; then
        log "$YELLOW" "💡 Consider committing the updated .gitattributes file"
        log "$YELLOW" "   git add .gitattributes"
        log "$YELLOW" "   git commit -m 'Update .gitattributes with latest file types'"
    fi
}

# Run main function
main "$@"
