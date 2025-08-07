#!/bin/bash

# generate-gitattributes.sh
# A wrapper script for generating .gitattributes using hyperpolyglot
# 
# Usage:
#   ./scripts/generate-gitattributes.sh [options]
#
# Options:
#   -o, --output FILE     Output file (default: .gitattributes)
#   -f, --force          Overwrite existing file without confirmation
#   -b, --backup         Create backup of existing .gitattributes
#   -d, --dry-run        Show what would be generated without writing
#   -h, --help           Show this help message

set -euo pipefail

# Default values
OUTPUT_FILE=".gitattributes"
FORCE=false
BACKUP=false
DRY_RUN=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Help function
show_help() {
    cat << EOF
generate-gitattributes.sh - Generate .gitattributes using hyperpolyglot

Usage:
    $0 [options]

Options:
    -o, --output FILE     Output file (default: .gitattributes)
    -f, --force          Overwrite existing file without confirmation
    -b, --backup         Create backup of existing .gitattributes
    -d, --dry-run        Show what would be generated without writing
    -h, --help           Show this help message

Examples:
    $0                    # Generate .gitattributes in current directory
    $0 -o custom.gitattributes  # Generate to custom file
    $0 -f -b             # Force overwrite with backup
    $0 -d                # Dry run to see what would be generated

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -b|--backup)
            BACKUP=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
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

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}" >&2
    exit 1
fi

# Check if hyperpolyglot is installed
if ! command -v hyply > /dev/null 2>&1; then
    echo -e "${YELLOW}Installing hyperpolyglot...${NC}"
    cargo install hyperpolyglot
fi

# Check if the Rust script exists and compile it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_SCRIPT="$SCRIPT_DIR/generate-gitattributes.rs"
BINARY="$SCRIPT_DIR/generate-gitattributes"

if [[ ! -f "$RUST_SCRIPT" ]]; then
    echo -e "${RED}Error: Rust script not found at $RUST_SCRIPT${NC}" >&2
    exit 1
fi

# Compile the Rust script if binary doesn't exist or is older than source
if [[ ! -f "$BINARY" ]] || [[ "$RUST_SCRIPT" -nt "$BINARY" ]]; then
    echo -e "${BLUE}Compiling Rust script...${NC}"
    rustc "$RUST_SCRIPT" -o "$BINARY"
fi

# Handle existing file
if [[ -f "$OUTPUT_FILE" ]]; then
    if [[ "$FORCE" == false ]] && [[ "$DRY_RUN" == false ]]; then
        echo -e "${YELLOW}Warning: $OUTPUT_FILE already exists${NC}"
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Aborted${NC}"
            exit 1
        fi
    fi
    
    if [[ "$BACKUP" == true ]] && [[ "$DRY_RUN" == false ]]; then
        BACKUP_FILE="${OUTPUT_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
        echo -e "${BLUE}Creating backup: $BACKUP_FILE${NC}"
        cp "$OUTPUT_FILE" "$BACKUP_FILE"
    fi
fi

# Run the generation
echo -e "${BLUE}Generating $OUTPUT_FILE using hyperpolyglot...${NC}"

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}DRY RUN - No files will be modified${NC}"
    # Create a temporary file for dry run
    TEMP_FILE=$(mktemp)
    "$BINARY" "$TEMP_FILE"
    echo -e "${GREEN}Generated content would be:${NC}"
    echo "=========================================="
    cat "$TEMP_FILE"
    echo "=========================================="
    rm "$TEMP_FILE"
else
    "$BINARY" "$OUTPUT_FILE"
    echo -e "${GREEN}✅ Successfully generated $OUTPUT_FILE${NC}"
    
    # Show file stats
    if [[ -f "$OUTPUT_FILE" ]]; then
        LINE_COUNT=$(wc -l < "$OUTPUT_FILE")
        FILE_SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)
        echo -e "${BLUE}📊 File stats: $LINE_COUNT lines, $FILE_SIZE${NC}"
    fi
fi

echo -e "${GREEN}🎉 Done!${NC}"
