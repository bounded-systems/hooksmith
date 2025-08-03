#!/bin/bash

# Comprehensive validation script for the generated files registry
# This script validates checksums, file existence, and registry consistency

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REGISTRY_FILE="config/generated-files.jsonc"

echo -e "${BLUE}Validating generated files registry...${NC}"

# Function to generate checksum for a file
generate_checksum() {
    local file="$1"
    sha256sum "$file" | cut -d' ' -f1 | cut -c1-8
}

# Initialize counters
TOTAL_FILES=0
VALID_FILES=0
INVALID_CHECKSUMS=0
MISSING_FILES=0
IGNORED_FILES=0

echo -e "${YELLOW}Checking file existence and checksums...${NC}"

# Get all registry entries
REGISTRY_ENTRIES=$(jq -r '.files[] | "\(.path)|\(.checksum)"' "$REGISTRY_FILE")

while IFS='|' read -r path checksum; do
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    if [ -f "$path" ]; then
        # File exists, check if it's ignored by git
        if git check-ignore "$path" >/dev/null 2>&1; then
            echo -e "${YELLOW}⚠ IGNORED: $path (should not be in registry)${NC}"
            IGNORED_FILES=$((IGNORED_FILES + 1))
        else
            # File exists and is tracked, validate checksum
            current_checksum=$(generate_checksum "$path")
            if [ "$current_checksum" = "$checksum" ]; then
                echo -e "${GREEN}✓ VALID: $path${NC}"
                VALID_FILES=$((VALID_FILES + 1))
            else
                echo -e "${RED}✗ INVALID CHECKSUM: $path (expected: $checksum, actual: $current_checksum)${NC}"
                INVALID_CHECKSUMS=$((INVALID_CHECKSUMS + 1))
            fi
        fi
    else
        echo -e "${RED}✗ MISSING: $path${NC}"
        MISSING_FILES=$((MISSING_FILES + 1))
    fi
done <<< "$REGISTRY_ENTRIES"

echo -e "\n${BLUE}=== VALIDATION SUMMARY ===${NC}"
echo -e "Total registry entries: ${BLUE}$TOTAL_FILES${NC}"
echo -e "Valid files: ${GREEN}$VALID_FILES${NC}"
echo -e "Invalid checksums: ${RED}$INVALID_CHECKSUMS${NC}"
echo -e "Missing files: ${RED}$MISSING_FILES${NC}"
echo -e "Ignored files (should be removed): ${YELLOW}$IGNORED_FILES${NC}"

# Check for missing files in registry
echo -e "\n${YELLOW}Checking for git-tracked files missing from registry...${NC}"
GIT_FILES=$(git ls-files | grep -E '\.(md|toml|sh|json|wit|hbs|css|sed|jsonl|yml|yaml|editorconfig|envrc|gitignore|gitattributes)$|^(CODEOWNERS|Makefile)$' | sort)
REGISTRY_PATHS=$(jq -r '.files[].path' "$REGISTRY_FILE" | sort)
MISSING_IN_REGISTRY=$(comm -23 <(echo "$GIT_FILES") <(echo "$REGISTRY_PATHS"))

MISSING_COUNT=$(echo "$MISSING_IN_REGISTRY" | wc -l)

if [ "$MISSING_COUNT" -gt 0 ]; then
    echo -e "${RED}Found $MISSING_COUNT files missing from registry:${NC}"
    echo "$MISSING_IN_REGISTRY" | while read -r file; do
        echo -e "${RED}- $file${NC}"
    done
else
    echo -e "${GREEN}✓ All git-tracked files are in registry${NC}"
fi

# Overall status
echo -e "\n${BLUE}=== OVERALL STATUS ===${NC}"
if [ $INVALID_CHECKSUMS -eq 0 ] && [ $MISSING_FILES -eq 0 ] && [ $IGNORED_FILES -eq 0 ] && [ "$MISSING_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ Registry is VALID and COMPLETE${NC}"
    exit 0
else
    echo -e "${RED}✗ Registry has ISSUES that need attention${NC}"
    
    if [ $INVALID_CHECKSUMS -gt 0 ]; then
        echo -e "${YELLOW}  - Run: ./scripts/fix-generated-files-registry.sh${NC}"
    fi
    
    if [ $IGNORED_FILES -gt 0 ]; then
        echo -e "${YELLOW}  - Run: ./scripts/cleanup-registry.sh${NC}"
    fi
    
    if [ "$MISSING_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}  - Add missing files to registry${NC}"
    fi
    
    exit 1
fi 
