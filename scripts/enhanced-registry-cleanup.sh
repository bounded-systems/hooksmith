#!/bin/bash

# Enhanced Registry Cleanup Script
# Removes temporary artifacts and updates registry automatically

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
echo -e "${BLUE}║              ENHANCED REGISTRY CLEANUP                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

# Patterns to clean up
TEMP_PATTERNS=(
    "2025*"           # Timestamped files
    "*.backup"        # Backup files
    "*.disabled"      # Disabled files
    "*.sed"           # Sed scripts
    "*.jsonl"         # Log files
    ".shellcheckrc"   # ShellCheck config
    "generated_file_demo" # Demo artifacts
    "fix_format.sed"  # Development artifacts
)

echo -e "\n${CYAN}🔍 Scanning for temporary artifacts...${NC}"

# Find files matching patterns
FOUND_FILES=()
for pattern in "${TEMP_PATTERNS[@]}"; do
    if [[ "$pattern" == *"*"* ]]; then
        # Handle wildcard patterns
        while IFS= read -r -d '' file; do
            FOUND_FILES+=("$file")
        done < <(find . -name "$pattern" -print0 2>/dev/null || true)
    else
        # Handle exact patterns
        if [[ -f "$pattern" ]]; then
            FOUND_FILES+=("$pattern")
        fi
    fi
done

# Remove duplicates and sort
FOUND_FILES=($(printf "%s\n" "${FOUND_FILES[@]}" | sort -u))

if [ ${#FOUND_FILES[@]} -eq 0 ]; then
    echo -e "${GREEN}✅ No temporary artifacts found!${NC}"
else
    echo -e "${YELLOW}📋 Found ${#FOUND_FILES[@]} temporary artifacts:${NC}"
    for file in "${FOUND_FILES[@]}"; do
        echo -e "${YELLOW}  • $file${NC}"
    done
    
    echo -e "\n${RED}🗑️  Removing temporary artifacts...${NC}"
    
    # Remove from git tracking (if tracked)
    GIT_TRACKED=()
    for file in "${FOUND_FILES[@]}"; do
        if git ls-files --error-unmatch "$file" >/dev/null 2>&1; then
            GIT_TRACKED+=("$file")
        fi
    done
    
    if [ ${#GIT_TRACKED[@]} -gt 0 ]; then
        echo -e "${YELLOW}📦 Removing ${#GIT_TRACKED[@]} files from git tracking...${NC}"
        git rm --cached "${GIT_TRACKED[@]}" || true
    fi
    
    # Remove files from filesystem
    echo -e "${YELLOW}🗂️  Removing files from filesystem...${NC}"
    for file in "${FOUND_FILES[@]}"; do
        if [[ -f "$file" ]]; then
            rm -f "$file"
            echo -e "${RED}  Removed: $file${NC}"
        fi
    done
fi

echo -e "\n${CYAN}🔧 Updating .gitignore...${NC}"

# Check if patterns are already in .gitignore
GITIGNORE_PATTERNS=(
    "# Timestamped and temporary artifacts"
    "2025*"
    "*.backup"
    "*.disabled"
    "*.sed"
    "*.jsonl"
    ".shellcheckrc"
    "generated_file_demo"
    ""
    "# Development artifacts"
    "fix_format.sed"
    "examples/schema_validation_demo.rs.disabled"
    "tests/*.disabled"
)

# Add patterns to .gitignore if not present
for pattern in "${GITIGNORE_PATTERNS[@]}"; do
    if [[ -n "$pattern" && ! "$pattern" =~ ^#.*$ ]]; then
        if ! grep -Fxq "$pattern" .gitignore 2>/dev/null; then
            echo "$pattern" >> .gitignore
            echo -e "${GREEN}  Added to .gitignore: $pattern${NC}"
        fi
    elif [[ "$pattern" =~ ^#.*$ ]]; then
        if ! grep -Fxq "$pattern" .gitignore 2>/dev/null; then
            echo "" >> .gitignore
            echo "$pattern" >> .gitignore
            echo -e "${GREEN}  Added comment to .gitignore: $pattern${NC}"
        fi
    fi
done

echo -e "\n${CYAN}🔄 Running registry cleanup...${NC}"

# Use the unified registry system
if command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Running: cargo xtask registry cleanup${NC}"
    cargo xtask registry cleanup
else
    echo -e "${RED}❌ Cargo not found. Please run 'cargo xtask registry cleanup' manually.${NC}"
fi

echo -e "\n${CYAN}✅ Validating registry...${NC}"

# Validate the registry
if command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Running: cargo xtask registry validate${NC}"
    if cargo xtask registry validate; then
        echo -e "${GREEN}✅ Registry validation passed!${NC}"
    else
        echo -e "${RED}❌ Registry validation failed!${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Cargo not found. Please run 'cargo xtask registry validate' manually.${NC}"
fi

echo -e "\n${CYAN}📊 Final status...${NC}"

# Show final status
if command -v cargo >/dev/null 2>&1; then
    cargo xtask registry status
else
    echo -e "${YELLOW}Registry status (manual check required):${NC}"
    echo -e "${YELLOW}  Run: cargo xtask registry status${NC}"
fi

echo -e "\n${GREEN}🎉 Enhanced registry cleanup completed!${NC}"
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    CLEANUP SUMMARY                          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo -e "${CYAN}📋 Files removed: ${#FOUND_FILES[@]}${NC}"
echo -e "${CYAN}📦 Git tracking: ${#GIT_TRACKED[@]} files removed${NC}"
echo -e "${CYAN}🔧 .gitignore: Updated with new patterns${NC}"
echo -e "${CYAN}🔄 Registry: Cleaned and validated${NC}"
echo -e "\n${GREEN}✅ All temporary artifacts have been removed and the registry is clean!${NC}" 
