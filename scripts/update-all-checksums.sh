#!/bin/bash

# Script to update all checksums in the generated files registry
# This will recalculate checksums for all files and update the registry

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REGISTRY_FILE="config/generated-files.jsonc"
BACKUP_FILE="config/generated-files.jsonc.backup.$(date +%Y%m%d_%H%M%S)"

echo -e "${BLUE}Updating all checksums in generated files registry...${NC}"

# Create backup
echo -e "${YELLOW}Creating backup: $BACKUP_FILE${NC}"
cp "$REGISTRY_FILE" "$BACKUP_FILE"

# Function to generate checksum for a file
generate_checksum() {
    local file="$1"
    sha256sum "$file" | cut -d' ' -f1 | cut -c1-8
}

# Get all registry entries
REGISTRY_ENTRIES=$(jq -r '.files[] | "\(.path)|\(.checksum)"' "$REGISTRY_FILE")

echo -e "${YELLOW}Updating checksums for all files...${NC}"

UPDATED_COUNT=0
SKIPPED_COUNT=0

while IFS='|' read -r path old_checksum; do
    if [ -f "$path" ]; then
        new_checksum=$(generate_checksum "$path")
        
        if [ "$new_checksum" != "$old_checksum" ]; then
            echo -e "${YELLOW}Updating: $path (${old_checksum} → ${new_checksum})${NC}"
            
            # Create temporary file for jq operation
            TEMP_FILE=$(mktemp)
            
            # Update the checksum
            jq --arg path "$path" --arg checksum "$new_checksum" \
               '(.files[] | select(.path == $path) | .checksum) = $checksum' \
               "$REGISTRY_FILE" > "$TEMP_FILE"
            mv "$TEMP_FILE" "$REGISTRY_FILE"
            
            UPDATED_COUNT=$((UPDATED_COUNT + 1))
        else
            echo -e "${GREEN}✓ Current: $path (checksum: $old_checksum)${NC}"
            SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
        fi
    else
        echo -e "${RED}⚠ Missing: $path${NC}"
    fi
done <<< "$REGISTRY_ENTRIES"

echo -e "\n${BLUE}=== UPDATE SUMMARY ===${NC}"
echo -e "Files updated: ${YELLOW}$UPDATED_COUNT${NC}"
echo -e "Files unchanged: ${GREEN}$SKIPPED_COUNT${NC}"
echo -e "Backup saved: ${BLUE}$BACKUP_FILE${NC}"

echo -e "\n${GREEN}✓ All checksums updated!${NC}"

# Verify the update
echo -e "\n${YELLOW}Verifying registry after update...${NC}"
./scripts/validate-registry.sh

echo -e "\n${GREEN}Checksum update complete!${NC}" 
