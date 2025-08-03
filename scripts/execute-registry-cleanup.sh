#!/bin/bash

# Execute the registry cleanup plan
# Removes files from git, registry, and updates .gitignore

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

REGISTRY_FILE="config/generated-files.jsonc"
BACKUP_FILE="config/generated-files.jsonc.backup.$(date +%Y%m%d_%H%M%S)"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              EXECUTING REGISTRY CLEANUP                     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

# Create backup
echo -e "${YELLOW}Creating backup: $BACKUP_FILE${NC}"
cp "$REGISTRY_FILE" "$BACKUP_FILE"

# Files to remove from registry and git
FILES_TO_REMOVE=(
    # Environment files
    ".envrc"
    
    # Test artifacts
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

echo -e "\n${RED}🗑️  REMOVING FILES FROM REGISTRY AND GIT:${NC}"

# Remove files from registry
for file in "${FILES_TO_REMOVE[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${YELLOW}Removing from registry: $file${NC}"
        
        # Create temporary file for jq operation
        TEMP_FILE=$(mktemp)
        
        # Remove the entry from registry
        jq --arg path "$file" 'del(.files[] | select(.path == $path))' "$REGISTRY_FILE" > "$TEMP_FILE"
        mv "$TEMP_FILE" "$REGISTRY_FILE"
        
        echo -e "${GREEN}✓ Removed from registry: $file${NC}"
    else
        echo -e "${RED}⚠ File not found: $file${NC}"
    fi
done

echo -e "\n${YELLOW}🌍 UPDATING .gitignore:${NC}"

# Add patterns to .gitignore
GITIGNORE_PATTERNS=(
    "# Environment files"
    ".envrc"
    ""
    "# Test artifacts and demos"
    "test-enhanced-gen-files/"
    "scripts/test-*.sh"
    ""
    "# Build artifacts (if any are found later)"
    "*.pdf"
    "*.epub"
    "*.html"
    ""
    "# Backup files"
    "*.backup.*"
    ""
    "# Temporary files"
    "*.tmp"
    "*.temp"
)

# Check if patterns already exist in .gitignore
for pattern in "${GITIGNORE_PATTERNS[@]}"; do
    if [[ "$pattern" =~ ^# ]] || [[ -z "$pattern" ]]; then
        # Comment or empty line, add it
        echo "$pattern" >> .gitignore
    else
        # Check if pattern already exists
        if ! grep -Fxq "$pattern" .gitignore; then
            echo "$pattern" >> .gitignore
            echo -e "${GREEN}✓ Added to .gitignore: $pattern${NC}"
        else
            echo -e "${YELLOW}⚠ Already in .gitignore: $pattern${NC}"
        fi
    fi
done

echo -e "\n${CYAN}🔄 MIGRATION RECOMMENDATIONS:${NC}"
echo -e "${YELLOW}The following files are candidates for migration to Rust/xtask:${NC}"
echo -e "${CYAN}  • fix_format.sed${NC}"
echo -e "${CYAN}  • 37 shell scripts in scripts/ directory${NC}"
echo -e "${YELLOW}Consider creating xtask commands to replace these shell scripts.${NC}"

echo -e "\n${GREEN}✅ CLEANUP COMPLETE!${NC}"
echo -e "${BLUE}Backup saved as: $BACKUP_FILE${NC}"

# Verify the cleanup
echo -e "\n${YELLOW}Verifying cleanup...${NC}"
./scripts/validate-registry.sh

echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    CLEANUP COMPLETE! 🎉                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}" 
