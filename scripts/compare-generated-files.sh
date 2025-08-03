#!/bin/bash

# Script to compare git-tracked files with the generated files registry
# and identify files that are tracked by git but missing from the registry

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Comparing git-tracked files with generated files registry...${NC}"

# Get all git-tracked files that match generated file patterns
echo -e "${YELLOW}Getting git-tracked files...${NC}"
GIT_FILES=$(git ls-files | grep -E '\.(md|toml|sh|json|wit|hbs|css|sed|jsonl|yml|yaml|CODEOWNERS|makefile|editorconfig|envrc|gitignore|gitattributes)$' | sort)

# Extract paths from the registry
echo -e "${YELLOW}Extracting paths from registry...${NC}"
REGISTRY_PATHS=$(jq -r '.files[].path' config/generated-files.jsonc | sort)

# Find files tracked by git but missing from registry
echo -e "${YELLOW}Finding missing files...${NC}"
MISSING_FILES=$(comm -23 <(echo "$GIT_FILES") <(echo "$REGISTRY_PATHS"))

# Count totals
GIT_COUNT=$(echo "$GIT_FILES" | wc -l)
REGISTRY_COUNT=$(echo "$REGISTRY_PATHS" | wc -l)
MISSING_COUNT=$(echo "$MISSING_FILES" | wc -l)

echo -e "\n${BLUE}=== SUMMARY ===${NC}"
echo -e "Git-tracked files: ${GREEN}$GIT_COUNT${NC}"
echo -e "Registry entries: ${GREEN}$REGISTRY_COUNT${NC}"
echo -e "Missing from registry: ${RED}$MISSING_COUNT${NC}"

if [ "$MISSING_COUNT" -gt 0 ]; then
    echo -e "\n${RED}=== MISSING FILES ===${NC}"
    echo "$MISSING_FILES" | while read -r file; do
        echo -e "${RED}- $file${NC}"
    done
    
    echo -e "\n${YELLOW}=== SUGGESTED ACTIONS ===${NC}"
    echo "1. Add missing files to config/generated-files.jsonc"
    echo "2. Generate checksums for new entries"
    echo "3. Update the registry with new entries"
else
    echo -e "\n${GREEN}✓ All git-tracked files are present in the registry!${NC}"
fi

# Also check for files in registry that are not tracked by git
echo -e "\n${YELLOW}Checking for registry entries not tracked by git...${NC}"
UNTRACKED_REGISTRY=$(comm -13 <(echo "$GIT_FILES") <(echo "$REGISTRY_PATHS"))
UNTRACKED_COUNT=$(echo "$UNTRACKED_REGISTRY" | wc -l)

if [ "$UNTRACKED_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}Registry entries not tracked by git: ${RED}$UNTRACKED_COUNT${NC}"
    echo "$UNTRACKED_REGISTRY" | while read -r file; do
        echo -e "${YELLOW}- $file${NC}"
    done
else
    echo -e "${GREEN}✓ All registry entries are tracked by git!${NC}"
fi

echo -e "\n${BLUE}Comparison complete!${NC}" 
