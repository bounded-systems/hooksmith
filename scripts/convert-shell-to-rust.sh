#!/bin/bash

# Convert Shell Scripts to Rust
# This script analyzes shell scripts and creates corresponding Rust implementations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

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

# Function to analyze shell script and determine its purpose
analyze_script() {
    local script_path="$1"
    local script_name=$(basename "$script_path" .sh)
    
    log_info "Analyzing: $script_name"
    
    # Extract first few lines to understand purpose
    local first_lines=$(head -20 "$script_path")
    
    # Look for common patterns
    if echo "$first_lines" | grep -q "worktree"; then
        echo "worktree_management"
    elif echo "$first_lines" | grep -q "build\|compile"; then
        echo "build_script"
    elif echo "$first_lines" | grep -q "cleanup\|clean"; then
        echo "cleanup_script"
    elif echo "$first_lines" | grep -q "sync\|update"; then
        echo "sync_script"
    elif echo "$first_lines" | grep -q "verify\|check"; then
        echo "verification_script"
    elif echo "$first_lines" | grep -q "pr\|pull.*request"; then
        echo "pr_management"
    else
        echo "general_utility"
    fi
}

# Function to create Rust binary structure
create_rust_binary() {
    local script_path="$1"
    local script_name=$(basename "$script_path" .sh)
    local script_type="$2"
    
    local rust_file="src/bin/${script_name}.rs"
    
    log_info "Creating Rust binary: $rust_file"
    
    # Create the Rust file with basic structure
    cat > "$rust_file" << EOF
use std::process::Command;
use std::path::Path;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("$script_name");
    println!();
    
    // TODO: Implement functionality from $script_path
    log_info("Converting from shell script: $script_path");
    
    // Add specific implementation based on script type
    match "$script_type" {
        "worktree_management" => {
            log_info("This is a worktree management script");
            // TODO: Add worktree-specific functionality
        }
        "build_script" => {
            log_info("This is a build script");
            // TODO: Add build-specific functionality
        }
        "cleanup_script" => {
            log_info("This is a cleanup script");
            // TODO: Add cleanup-specific functionality
        }
        "sync_script" => {
            log_info("This is a sync script");
            // TODO: Add sync-specific functionality
        }
        "verification_script" => {
            log_info("This is a verification script");
            // TODO: Add verification-specific functionality
        }
        "pr_management" => {
            log_info("This is a PR management script");
            // TODO: Add PR-specific functionality
        }
        _ => {
            log_info("This is a general utility script");
            // TODO: Add general functionality
        }
    }
    
    log_success("Script execution completed");
    Ok(())
}
EOF
    
    log_success "Created Rust binary: $rust_file"
}

# Function to extract key functions from shell script
extract_functions() {
    local script_path="$1"
    
    # Extract function names
    local functions=$(grep -E "^[a-zA-Z_][a-zA-Z0-9_]*\(\)" "$script_path" | sed 's/()//' || true)
    
    if [ -n "$functions" ]; then
        log_info "Found functions: $functions"
        echo "$functions"
    else
        log_info "No functions found in script"
        echo ""
    fi
}

# Function to extract git commands from shell script
extract_git_commands() {
    local script_path="$1"
    
    # Extract git commands
    local git_commands=$(grep -E "git\s+" "$script_path" | head -10 || true)
    
    if [ -n "$git_commands" ]; then
        log_info "Found git commands:"
        echo "$git_commands" | while read -r cmd; do
            echo "  $cmd"
        done
    else
        log_info "No git commands found"
    fi
}

# Function to create conversion summary
create_conversion_summary() {
    local script_path="$1"
    local script_name=$(basename "$script_path" .sh)
    local script_type="$2"
    
    local summary_file="docs/conversion-summary.md"
    
    # Create summary directory if it doesn't exist
    mkdir -p "$(dirname "$summary_file")"
    
    # Append to summary file
    cat >> "$summary_file" << EOF

## $script_name

- **Original**: \`$script_path\`
- **Rust Binary**: \`src/bin/${script_name}.rs\`
- **Type**: $script_type
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
$(extract_functions "$script_path")

### Git Commands
$(extract_git_commands "$script_path")

---
EOF
    
    log_info "Added to conversion summary: $summary_file"
}

# Main execution
main() {
    log_header "SHELL TO RUST CONVERSION"
    echo ""
    
    # Find all shell scripts
    local shell_scripts=$(find . -name "*.sh" -type f | grep -v "convert-shell-to-rust.sh")
    
    if [ -z "$shell_scripts" ]; then
        log_info "No shell scripts found"
        return 0
    fi
    
    log_info "Found $(echo "$shell_scripts" | wc -l) shell scripts to convert"
    echo ""
    
    local converted_count=0
    
    # Process each shell script
    while IFS= read -r script_path; do
        if [ -z "$script_path" ]; then
            continue
        fi
        
        local script_name=$(basename "$script_path" .sh)
        local script_type=$(analyze_script "$script_path")
        
        log_header "CONVERTING: $script_name"
        echo ""
        
        # Create Rust binary
        create_rust_binary "$script_path" "$script_type"
        
        # Create conversion summary
        create_conversion_summary "$script_path" "$script_type"
        
        converted_count=$((converted_count + 1))
        
        echo "---"
        echo ""
        
    done <<< "$shell_scripts"
    
    log_success "Converted $converted_count shell scripts to Rust"
    echo ""
    log_info "Next steps:"
    log_info "1. Review the generated Rust files in src/bin/"
    log_info "2. Implement specific functionality for each script"
    log_info "3. Test the Rust binaries"
    log_info "4. Update any references to the old shell scripts"
}

# Run main function
main "$@" 
