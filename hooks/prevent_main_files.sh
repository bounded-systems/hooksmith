#!/bin/bash

# Prevent creating files named "main" in CLI prototype worktree
# This ensures no local files can be accidentally named "main"

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the current branch name
current_branch=$(git symbolic-ref --short HEAD 2>/dev/null)

# Check if we're in the pushd-cli-prototype branch
if [ "$current_branch" = "pushd-cli-prototype" ]; then
    echo -e "${YELLOW}🔒 CLI Prototype File Safety Check${NC}"
    
    # Check for any files named "main" in the staging area or working directory
    main_files=$(find . -name "main*" -type f 2>/dev/null | grep -v ".git" | head -10)
    
    if [ -n "$main_files" ]; then
        echo -e "${RED}❌ ERROR: Found files named 'main' in CLI prototype worktree!${NC}"
        echo ""
        echo "This is a safety measure to prevent confusion with the main branch."
        echo ""
        echo "Files found:"
        echo "$main_files"
        echo ""
        echo "Please rename these files to something more descriptive:"
        echo "  - main.rs → cli_main.rs or app.rs"
        echo "  - main.py → cli_main.py or app.py"
        echo "  - main.js → cli_main.js or app.js"
        echo "  - main.sh → cli_main.sh or app.sh"
        echo ""
        echo "This helps prevent accidental confusion with the main branch."
        exit 1
    fi
    
    # Check for directories named "main"
    main_dirs=$(find . -name "main" -type d 2>/dev/null | grep -v ".git" | head -10)
    
    if [ -n "$main_dirs" ]; then
        echo -e "${RED}❌ ERROR: Found directories named 'main' in CLI prototype worktree!${NC}"
        echo ""
        echo "This is a safety measure to prevent confusion with the main branch."
        echo ""
        echo "Directories found:"
        echo "$main_dirs"
        echo ""
        echo "Please rename these directories to something more descriptive:"
        echo "  - main/ → cli/ or app/ or src/"
        echo ""
        echo "This helps prevent accidental confusion with the main branch."
        exit 1
    fi
fi

echo -e "${YELLOW}✅ CLI Prototype file safety check passed${NC}"
exit 0 
