#!/bin/bash

# Script to fix the generated files registry by:
# 1. Adding missing files tracked by git
# 2. Fixing invalid checksums for specific files

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REGISTRY_FILE="config/generated-files.jsonc"
BACKUP_FILE="config/generated-files.jsonc.backup.$(date +%Y%m%d_%H%M%S)"

echo -e "${BLUE}Fixing generated files registry...${NC}"

# Create backup
echo -e "${YELLOW}Creating backup: $BACKUP_FILE${NC}"
cp "$REGISTRY_FILE" "$BACKUP_FILE"

# Function to generate checksum for a file
generate_checksum() {
    local file="$1"
    sha256sum "$file" | cut -d' ' -f1 | cut -c1-8
}

# Function to generate slug from path
generate_slug() {
    local path="$1"
    echo "$path" | sed 's/[^a-zA-Z0-9]/_/g' | sed 's/__*/_/g' | sed 's/^_//' | sed 's/_$//'
}

# Function to determine file type from extension
get_file_type() {
    local path="$1"
    local ext="${path##*.}"
    case "$ext" in
        md) echo "md" ;;
        toml) echo "toml" ;;
        sh) echo "sh" ;;
        json) echo "json" ;;
        wit) echo "wit" ;;
        hbs) echo "hbs" ;;
        css) echo "css" ;;
        sed) echo "sed" ;;
        jsonl) echo "jsonl" ;;
        yml|yaml) echo "yaml" ;;
        CODEOWNERS) echo "CODEOWNERS" ;;
        makefile|Makefile) echo "makefile" ;;
        editorconfig) echo "editorconfig" ;;
        envrc) echo "envrc" ;;
        gitignore) echo "gitignore" ;;
        gitattributes) echo "gitattributes" ;;
        *) echo "unknown" ;;
    esac
}

# Fix specific checksums first
echo -e "${YELLOW}Fixing invalid checksums...${NC}"

# Create temporary file for jq operations
TEMP_FILE=$(mktemp)

# Fix .editorconfig checksum
echo -e "${YELLOW}Fixing .editorconfig checksum...${NC}"
NEW_CHECKSUM=$(generate_checksum ".editorconfig")
jq --arg path ".editorconfig" --arg checksum "$NEW_CHECKSUM" \
   '(.files[] | select(.path == $path) | .checksum) = $checksum' \
   "$REGISTRY_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$REGISTRY_FILE"

# Fix .envrc checksum
echo -e "${YELLOW}Fixing .envrc checksum...${NC}"
NEW_CHECKSUM=$(generate_checksum ".envrc")
jq --arg path ".envrc" --arg checksum "$NEW_CHECKSUM" \
   '(.files[] | select(.path == $path) | .checksum) = $checksum' \
   "$REGISTRY_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$REGISTRY_FILE"

# Fix ARCHITECTURE.md checksum
echo -e "${YELLOW}Fixing ARCHITECTURE.md checksum...${NC}"
NEW_CHECKSUM=$(generate_checksum "ARCHITECTURE.md")
jq --arg path "ARCHITECTURE.md" --arg checksum "$NEW_CHECKSUM" \
   '(.files[] | select(.path == $path) | .checksum) = $checksum' \
   "$REGISTRY_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$REGISTRY_FILE"

echo -e "${GREEN}✓ Fixed invalid checksums${NC}"

# Now add missing files
echo -e "${YELLOW}Adding missing files to registry...${NC}"

# List of missing files (from the comparison script output)
MISSING_FILES=(
    ".cargo/aliases.toml"
    ".cargo/config.toml"
    ".github/workflows/ci.yml"
    ".github/workflows/contract-check.yml"
    ".github/workflows/contract-validation.yml"
    ".github/workflows/verify-hooksmith.yml"
    ".trunk/.gitignore"
    ".trunk/configs/.markdownlint.yaml"
    ".trunk/configs/.rustfmt.toml"
    ".trunk/trunk.yaml"
    "CHECKSUM_SYSTEM_IMPLEMENTATION.md"
    "CHECKSUM_SYSTEM_INTEGRATION_COMPLETE.md"
    "components/worktree-runner/.wtp.yml"
    "COMPREHENSIVE_FILE_POLICY_REFACTOR.md"
    "config/contract-state-machine.yaml"
    "config/contract-state-machine.yml"
    "config/docs_manifest.yaml"
    "config/docs_manifest.yml"
    "config/file_types.yaml"
    "config/state-transitions.yml"
    "docs/state-transitions.yaml"
    "ENHANCED_CHECKSUM_SYSTEM_COMPLETE.md"
    "ENVRRC_EDITORCONFIG_INTEGRATION_SUMMARY.md"
    "FILE_DISTRIBUTION_ANALYSIS_SUMMARY.md"
    "lefthook-example.yml"
    "lefthook.yml"
    "PRE_COMMIT_CHECKSUM_INTEGRATION_COMPLETE.md"
    "scripts/add-checksums-corrected.sh"
    "scripts/add-checksums-to-files.sh"
    "scripts/analyze-file-distribution.sh"
    "scripts/enhanced-file-analysis.sh"
    "scripts/enhanced-gen-files.sh"
    "scripts/integrate-checksum-system.sh"
    "scripts/migrate-all-checksums.sh"
    "scripts/monitor-file-distribution.sh"
    "scripts/simple-checksum-demo.sh"
    "scripts/test-add-checksums.sh"
    "scripts/test-checksum-system.sh"
    "scripts/test-checksum-with-real-files.sh"
    "scripts/test-envrc-editorconfig.sh"
    "scripts/test-pre-commit-checksums.sh"
    "test-enhanced-gen-files/.gitignore"
    "test-enhanced-gen-files/Cargo.toml"
    "test-enhanced-gen-files/config.json"
    "test-enhanced-gen-files/hooksmith.wit"
    "test-enhanced-gen-files/README.md"
)

# Add each missing file to the registry
for file in "${MISSING_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${YELLOW}Adding: $file${NC}"
        
        checksum=$(generate_checksum "$file")
        slug=$(generate_slug "$file")
        type=$(get_file_type "$file")
        
        # Create new entry
        NEW_ENTRY=$(jq -n \
            --arg path "$file" \
            --arg checksum "$checksum" \
            --arg slug "$slug" \
            --arg type "$type" \
            '{path: $path, checksum: $checksum, slug: $slug, type: $type}')
        
        # Add to registry
        jq --argjson entry "$NEW_ENTRY" '.files += [$entry]' "$REGISTRY_FILE" > "$TEMP_FILE"
        mv "$TEMP_FILE" "$REGISTRY_FILE"
        
        echo -e "${GREEN}✓ Added $file (checksum: $checksum)${NC}"
    else
        echo -e "${RED}⚠ File not found: $file${NC}"
    fi
done

# Clean up
rm -f "$TEMP_FILE"

echo -e "\n${GREEN}✓ Registry update complete!${NC}"
echo -e "${BLUE}Backup saved as: $BACKUP_FILE${NC}"

# Verify the update
echo -e "\n${YELLOW}Verifying registry...${NC}"
./scripts/compare-generated-files.sh

echo -e "\n${GREEN}Registry fix complete!${NC}" 
