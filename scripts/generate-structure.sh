#!/bin/bash

# Generate Repository Structure Documentation
# This script creates a comprehensive overview of the repository structure

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📁 Generating repository structure documentation...${NC}"

# Create scripts directory if it doesn't exist
mkdir -p scripts

# Generate basic structure
echo "# Repository Structure" > STRUCTURE.md
echo "" >> STRUCTURE.md
echo "This document shows the complete file structure of the repository." >> STRUCTURE.md
echo "" >> STRUCTURE.md

# Add tree structure
echo "## 📁 File Structure" >> STRUCTURE.md
echo "" >> STRUCTURE.md
echo '```' >> STRUCTURE.md
git ls-tree -r HEAD | awk '{print $4}' | tree --fromfile >> STRUCTURE.md
echo '```' >> STRUCTURE.md
echo "" >> STRUCTURE.md

# Add file count summary
echo "## 📊 File Count Summary" >> STRUCTURE.md
echo "" >> STRUCTURE.md

# Count different file types
TOTAL_FILES=$(git ls-tree -r HEAD | wc -l)
RUST_FILES=$(git ls-tree -r HEAD | grep '\.rs$' | wc -l)
CONFIG_FILES=$(git ls-tree -r HEAD | grep -E '\.(toml|yaml|yml|rc)$' | wc -l)
DOC_FILES=$(git ls-tree -r HEAD | grep '\.md$' | wc -l)
SCRIPT_FILES=$(git ls-tree -r HEAD | grep '\.sh$' | wc -l)

echo "- **Total Files**: $TOTAL_FILES" >> STRUCTURE.md
echo "- **Rust Files**: $RUST_FILES (.rs)" >> STRUCTURE.md
echo "- **Configuration Files**: $CONFIG_FILES (.toml, .yaml, .rc)" >> STRUCTURE.md
echo "- **Documentation**: $DOC_FILES (.md)" >> STRUCTURE.md
echo "- **Scripts**: $SCRIPT_FILES (.sh)" >> STRUCTURE.md
echo "" >> STRUCTURE.md

# Add generation timestamp
echo "---" >> STRUCTURE.md
echo "" >> STRUCTURE.md
echo "*Generated on $(date) using \`git ls-tree -r HEAD\`*" >> STRUCTURE.md

echo -e "${GREEN}✅ Repository structure documentation generated in STRUCTURE.md${NC}"

# Also create a simple tree output
echo -e "${BLUE}📋 Current repository tree:${NC}"
git ls-tree -r HEAD | awk '{print $4}' | tree --fromfile

echo -e "${GREEN}✅ Structure generation complete!${NC}" 
