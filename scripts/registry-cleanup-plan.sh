#!/bin/bash

# Comprehensive cleanup plan for the generated files registry
# Identifies files that should be removed from version control and registry

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                REGISTRY CLEANUP ANALYSIS                    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

# Get all registry entries
REGISTRY_ENTRIES=$(jq -r '.files[].path' config/generated-files.jsonc | sort)

echo -e "\n${CYAN}🔍 Analyzing registry entries for cleanup candidates...${NC}"

# Categories for cleanup
BUILD_ARTIFACTS=()
ENVIRONMENT_FILES=()
TEST_ARTIFACTS=()
DEPRECATED_FILES=()
MIGRATION_CANDIDATES=()

echo -e "\n${PURPLE}📋 Categorizing files for cleanup:${NC}"

while IFS= read -r path; do
    filename=$(basename "$path")
    extension="${filename##*.}"
    
    # Build artifacts (should be generated, not committed)
    if [[ "$extension" =~ ^(pdf|epub|html)$ ]] || [[ "$filename" =~ \.(pdf|epub|html)$ ]]; then
        BUILD_ARTIFACTS+=("$path")
        echo -e "${RED}  🗑️  BUILD_ARTIFACT: $path${NC}"
    
    # Environment-specific files (should be in .gitignore)
    elif [[ "$path" =~ \.envrc$ ]] || [[ "$path" =~ /direnv/ ]]; then
        ENVIRONMENT_FILES+=("$path")
        echo -e "${YELLOW}  🌍 ENVIRONMENT: $path${NC}"
    
    # Test artifacts and demos
    elif [[ "$path" =~ generated_file_demo ]] || [[ "$path" =~ \.disabled$ ]] || [[ "$path" =~ test-.* ]]; then
        TEST_ARTIFACTS+=("$path")
        echo -e "${YELLOW}  🧪 TEST_ARTIFACT: $path${NC}"
    
    # Deprecated or duplicate files
    elif [[ "$path" =~ -safechars$ ]] || [[ "$path" =~ -blob-contract$ ]] || [[ "$path" =~ backup ]]; then
        DEPRECATED_FILES+=("$path")
        echo -e "${YELLOW}  📦 DEPRECATED: $path${NC}"
    
    # Migration candidates (shell scripts that could be Rust/xtask)
    elif [[ "$extension" = "sh" ]] && [[ "$path" =~ ^scripts/ ]] && [[ ! "$path" =~ ^scripts/(compare|validate|update|cleanup|fix|registry-status) ]]; then
        MIGRATION_CANDIDATES+=("$path")
        echo -e "${CYAN}  🔄 MIGRATION_CANDIDATE: $path${NC}"
    
    # Sed files (could be replaced with Rust)
    elif [[ "$extension" = "sed" ]]; then
        MIGRATION_CANDIDATES+=("$path")
        echo -e "${CYAN}  🔄 MIGRATION_CANDIDATE: $path${NC}"
    fi
done <<< "$REGISTRY_ENTRIES"

echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    CLEANUP SUMMARY                           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${RED}🗑️  BUILD ARTIFACTS (${#BUILD_ARTIFACTS[@]} files):${NC}"
if [ ${#BUILD_ARTIFACTS[@]} -gt 0 ]; then
    printf '%s\n' "${BUILD_ARTIFACTS[@]}" | while read -r file; do
        echo -e "${RED}  • $file${NC}"
    done
else
    echo -e "${GREEN}  (none found)${NC}"
fi

echo -e "\n${YELLOW}🌍 ENVIRONMENT FILES (${#ENVIRONMENT_FILES[@]} files):${NC}"
if [ ${#ENVIRONMENT_FILES[@]} -gt 0 ]; then
    printf '%s\n' "${ENVIRONMENT_FILES[@]}" | while read -r file; do
        echo -e "${YELLOW}  • $file${NC}"
    done
else
    echo -e "${GREEN}  (none found)${NC}"
fi

echo -e "\n${YELLOW}🧪 TEST ARTIFACTS (${#TEST_ARTIFACTS[@]} files):${NC}"
if [ ${#TEST_ARTIFACTS[@]} -gt 0 ]; then
    printf '%s\n' "${TEST_ARTIFACTS[@]}" | while read -r file; do
        echo -e "${YELLOW}  • $file${NC}"
    done
else
    echo -e "${GREEN}  (none found)${NC}"
fi

echo -e "\n${YELLOW}📦 DEPRECATED FILES (${#DEPRECATED_FILES[@]} files):${NC}"
if [ ${#DEPRECATED_FILES[@]} -gt 0 ]; then
    printf '%s\n' "${DEPRECATED_FILES[@]}" | while read -r file; do
        echo -e "${YELLOW}  • $file${NC}"
    done
else
    echo -e "${GREEN}  (none found)${NC}"
fi

echo -e "\n${CYAN}🔄 MIGRATION CANDIDATES (${#MIGRATION_CANDIDATES[@]} files):${NC}"
if [ ${#MIGRATION_CANDIDATES[@]} -gt 0 ]; then
    printf '%s\n' "${MIGRATION_CANDIDATES[@]}" | while read -r file; do
        echo -e "${CYAN}  • $file${NC}"
    done
else
    echo -e "${GREEN}  (none found)${NC}"
fi

# Calculate totals
TOTAL_CLEANUP=$(( ${#BUILD_ARTIFACTS[@]} + ${#ENVIRONMENT_FILES[@]} + ${#TEST_ARTIFACTS[@]} + ${#DEPRECATED_FILES[@]} ))
TOTAL_MIGRATION=${#MIGRATION_CANDIDATES[@]}

echo -e "\n${PURPLE}📊 CLEANUP STATISTICS:${NC}"
echo -e "${BLUE}  • Total files for removal: ${RED}$TOTAL_CLEANUP${NC}"
echo -e "${BLUE}  • Migration candidates: ${CYAN}$TOTAL_MIGRATION${NC}"
echo -e "${BLUE}  • Total registry entries: $(echo "$REGISTRY_ENTRIES" | wc -l)${NC}"

echo -e "\n${GREEN}✅ RECOMMENDED ACTIONS:${NC}"
echo -e "${YELLOW}  1. Remove build artifacts from git and registry${NC}"
echo -e "${YELLOW}  2. Add environment files to .gitignore${NC}"
echo -e "${YELLOW}  3. Clean up test artifacts and demos${NC}"
echo -e "${YELLOW}  4. Remove deprecated/duplicate files${NC}"
echo -e "${CYAN}  5. Plan migration of shell scripts to Rust/xtask${NC}"

echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              READY FOR CLEANUP EXECUTION                     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}" 
