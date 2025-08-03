#!/bin/bash

# Script to clean up the generated files registry by removing entries
# for files that are ignored by git (logs, generated files, etc.)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REGISTRY_FILE="config/generated-files.jsonc"
BACKUP_FILE="config/generated-files.jsonc.backup.$(date +%Y%m%d_%H%M%S)"

echo -e "${BLUE}Cleaning up generated files registry...${NC}"

# Create backup
echo -e "${YELLOW}Creating backup: $BACKUP_FILE${NC}"
cp "$REGISTRY_FILE" "$BACKUP_FILE"

# Get all registry paths
REGISTRY_PATHS=$(jq -r '.files[].path' "$REGISTRY_FILE")

echo -e "${YELLOW}Checking for files that should be removed from registry...${NC}"

# Files to remove (ignored by git or no longer exist)
FILES_TO_REMOVE=()

for path in $REGISTRY_PATHS; do
    if [ -f "$path" ]; then
        # Check if file is ignored by git
        if git check-ignore "$path" >/dev/null 2>&1; then
            echo -e "${YELLOW}Found ignored file: $path${NC}"
            FILES_TO_REMOVE+=("$path")
        fi
    else
        # File doesn't exist
        echo -e "${RED}Found non-existent file: $path${NC}"
        FILES_TO_REMOVE+=("$path")
    fi
done

if [ ${#FILES_TO_REMOVE[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ No files need to be removed from registry${NC}"
else
    echo -e "\n${YELLOW}Removing ${#FILES_TO_REMOVE[@]} files from registry...${NC}"
    
    # Remove each file from registry
    for path in "${FILES_TO_REMOVE[@]}"; do
        echo -e "${YELLOW}Removing: $path${NC}"
        
        # Create temporary file for jq operation
        TEMP_FILE=$(mktemp)
        
        # Remove the entry
        jq --arg path "$path" 'del(.files[] | select(.path == $path))' "$REGISTRY_FILE" > "$TEMP_FILE"
        mv "$TEMP_FILE" "$REGISTRY_FILE"
        
        echo -e "${GREEN}✓ Removed $path${NC}"
    done
    
    echo -e "\n${GREEN}✓ Cleanup complete!${NC}"
fi

echo -e "${BLUE}Backup saved as: $BACKUP_FILE${NC}"

# Verify the cleanup
echo -e "\n${YELLOW}Verifying registry after cleanup...${NC}"
./scripts/compare-generated-files.sh

echo -e "\n${GREEN}Registry cleanup complete!${NC}" 
