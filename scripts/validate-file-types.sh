#!/bin/bash

# validate-file-types.sh
# Enforces strict file type policies for PRs
# 
# Allowed: .rs, .jsonc files
# Blocked: All shell files (.sh, .bash, .zsh, etc.)
# 
# Usage:
#   ./scripts/validate-file-types.sh [options]
#
# Options:
#   -s, --strict     Exit with error if violations found
#   -v, --verbose    Verbose output
#   -h, --help       Show help

set -euo pipefail

# Default values
STRICT=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Log function
log() {
    local level="$1"
    shift
    local message="$*"
    echo -e "${level}${message}${NC}"
}

# Help function
show_help() {
    cat << EOF
validate-file-types.sh - Enforce strict file type policies

Usage:
    $0 [options]

Options:
    -s, --strict     Exit with error if violations found
    -v, --verbose    Verbose output
    -h, --help       Show help

File Type Policy:
    ✅ ALLOWED: .rs, .jsonc files
    ❌ BLOCKED: All shell files (.sh, .bash, .zsh, .fish, etc.)
    ⚠️  WARNED: Other file types (will be reported but not blocked)

Examples:
    $0                    # Check and report violations
    $0 -s                # Strict mode - exit with error if violations
    $0 -v                # Verbose output
    $0 -s -v             # Strict mode with verbose output

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--strict)
            STRICT=true
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

# Define file type policies
ALLOWED_EXTENSIONS=("rs" "jsonc")
BLOCKED_EXTENSIONS=("sh" "bash" "zsh" "fish" "csh" "ksh" "tcsh" "dash" "ash" "mksh" "yash" "posh" "rc" "es" "nu" "xonsh" "elvish" "nushell" "powershell" "ps1" "cmd" "bat" "com" "exe" "vbs" "js" "ts" "py" "rb" "pl" "php" "java" "cs" "cpp" "c" "h" "hpp" "cc" "cxx" "m" "mm" "swift" "go" "rs" "dart" "kt" "scala" "clj" "hs" "ml" "fs" "v" "zig" "nim" "crystal" "v" "odin" "jai" "carbon" "mojo" "carbon" "jai" "odin" "v" "crystal" "nim" "zig" "fs" "ml" "hs" "clj" "scala" "kt" "dart" "go" "swift" "mm" "m" "cxx" "cc" "hpp" "h" "c" "cpp" "cs" "java" "php" "pl" "rb" "py" "ts" "js" "exe" "com" "bat" "cmd" "ps1" "powershell" "nushell" "elvish" "xonsh" "nu" "es" "rc" "posh" "yash" "mksh" "ash" "dash" "tcsh" "ksh" "csh" "fish" "zsh" "bash" "sh")

# Function to get file extension
get_extension() {
    local file="$1"
    echo "${file##*.}"
}

# Function to check if extension is allowed
is_allowed() {
    local ext="$1"
    for allowed in "${ALLOWED_EXTENSIONS[@]}"; do
        if [[ "$ext" == "$allowed" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check if extension is blocked
is_blocked() {
    local ext="$1"
    for blocked in "${BLOCKED_EXTENSIONS[@]}"; do
        if [[ "$ext" == "$blocked" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to validate files
validate_files() {
    local files="$1"
    local context="$2"
    
    local allowed_files=()
    local blocked_files=()
    local other_files=()
    local no_extension_files=()
    
    vlog "Validating $context files..."
    
    while IFS= read -r file; do
        if [[ -z "$file" ]]; then
            continue
        fi
        
        # Skip directories
        if [[ -d "$file" ]]; then
            vlog "Skipping directory: $file"
            continue
        fi
        
        # Get file extension
        local ext=$(get_extension "$file")
        
        # Check if file has extension
        if [[ "$ext" == "$file" ]]; then
            # No extension
            no_extension_files+=("$file")
            vlog "File with no extension: $file"
        elif is_allowed "$ext"; then
            # Allowed extension
            allowed_files+=("$file")
            vlog "Allowed file: $file ($ext)"
        elif is_blocked "$ext"; then
            # Blocked extension
            blocked_files+=("$file")
            vlog "Blocked file: $file ($ext)"
        else
            # Other extension
            other_files+=("$file")
            vlog "Other file: $file ($ext)"
        fi
    done <<< "$files"
    
    # Report results
    echo
    log "$GREEN" "📊 File Type Validation Results ($context):"
    echo
    
    # Show allowed files
    if [[ ${#allowed_files[@]} -gt 0 ]]; then
        log "$GREEN" "✅ Allowed files (${#allowed_files[@]}):"
        for file in "${allowed_files[@]}"; do
            echo "   $file"
        done
        echo
    fi
    
    # Show blocked files
    if [[ ${#blocked_files[@]} -gt 0 ]]; then
        log "$RED" "❌ BLOCKED files (${#blocked_files[@]}):"
        for file in "${blocked_files[@]}"; do
            echo "   $file"
        done
        echo
    fi
    
    # Show other files
    if [[ ${#other_files[@]} -gt 0 ]]; then
        log "$YELLOW" "⚠️  Other files (${#other_files[@]}):"
        for file in "${other_files[@]}"; do
            local ext=$(get_extension "$file")
            echo "   $file ($ext)"
        done
        echo
    fi
    
    # Show files with no extension
    if [[ ${#no_extension_files[@]} -gt 0 ]]; then
        log "$YELLOW" "⚠️  Files with no extension (${#no_extension_files[@]}):"
        for file in "${no_extension_files[@]}"; do
            echo "   $file"
        done
        echo
    fi
    
    # Return status
    if [[ ${#blocked_files[@]} -gt 0 ]]; then
        return 1
    else
        return 0
    fi
}

# Main validation function
main() {
    log "$CYAN" "🔍 File Type Policy Validation"
    log "$CYAN" "============================="
    echo
    
    log "$BLUE" "Policy:"
    log "$GREEN" "  ✅ ALLOWED: ${ALLOWED_EXTENSIONS[*]}"
    log "$RED" "  ❌ BLOCKED: Shell files and others"
    log "$YELLOW" "  ⚠️  WARNED: Other file types"
    echo
    
    local has_violations=false
    
    # Check staged files (for pre-commit)
    if [[ -n "$(git diff --cached --name-only 2>/dev/null)" ]]; then
        local staged_files=$(git diff --cached --name-only)
        if ! validate_files "$staged_files" "staged"; then
            has_violations=true
        fi
    fi
    
    # Check all tracked files (for repository-wide validation)
    local all_files=$(git ls-files --cached --full-name)
    if ! validate_files "$all_files" "all tracked"; then
        has_violations=true
    fi
    
    # Check changed files in PR (for CI)
    if [[ -n "${GITHUB_BASE_REF:-}" ]]; then
        local changed_files=$(git diff --name-only origin/${GITHUB_BASE_REF}...HEAD 2>/dev/null || echo "")
        if [[ -n "$changed_files" ]]; then
            if ! validate_files "$changed_files" "PR changes"; then
                has_violations=true
            fi
        fi
    fi
    
    # Summary
    echo
    log "$CYAN" "📋 Summary:"
    if [[ "$has_violations" == true ]]; then
        log "$RED" "❌ File type policy violations found!"
        echo
        log "$YELLOW" "💡 To fix violations:"
        log "$YELLOW" "   • Remove blocked files"
        log "$YELLOW" "   • Convert shell scripts to Rust"
        log "$YELLOW" "   • Use .jsonc for configuration files"
        echo
        if [[ "$STRICT" == true ]]; then
            log "$RED" "🚫 Strict mode: Exiting with error"
            exit 1
        else
            log "$YELLOW" "⚠️  Non-strict mode: Continuing with warnings"
        fi
    else
        log "$GREEN" "✅ No file type policy violations found!"
        echo
        log "$GREEN" "🎉 All files comply with the policy"
    fi
}

# Run main function
main "$@"
