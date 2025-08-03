#!/bin/bash
# Generate generated-files.jsonc configuration from repository file tree
# This script scans the repository and creates a JSONC file listing all files
# that should be generated, with stable slugs derived from their paths.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Generating generated-files.jsonc configuration...${NC}"

# Output file
OUTPUT_FILE="config/generated-files.jsonc"

# Function to convert path to slug
path_to_slug() {
    local path="$1"
    local extension="$2"
    
    # For files without extensions, use the full path
    if [[ -z "$extension" ]]; then
        local slug_path="$path"
    else
        # Remove the extension from the path for slug generation
        local slug_path="${path%.*}"
    fi
    
    # Convert to kebab-case
    # Replace slashes and dots with hyphens, convert to lowercase
    echo "$slug_path" | sed 's/[\/\.]/-/g' | tr '[:upper:]' '[:lower:]' | sed 's/-*$//'
}

# Function to get file extension
get_extension() {
    local file="$1"
    
    # Handle files that start with a dot (like .gitignore, .editorconfig)
    if [[ "$file" == .* ]]; then
        # For files like .gitignore, .editorconfig, etc., no extension
        echo ""
    else
        local ext="${file##*.}"
        if [[ "$ext" == "$file" ]]; then
            echo ""
        else
            echo "$ext"
        fi
    fi
}

# Start the JSONC file
cat > "$OUTPUT_FILE" << 'EOF'
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Hooksmith Generated Files",
  "description": "List of generated files with stable slugs for referencing in CI and automation.",
  
  // All files listed here are assumed to be generated
  // No redundant generated: true flags needed
  "files": [
EOF

# File patterns to include (files that should be generated)
INCLUDE_PATTERNS=(
    "*.md"
    "*.sh"
    "*.toml"
    "*.wit"
    "*.hbs"
    "*.json"
    "*.jsonl"
    "*.css"
    "*.sed"
    ".gitignore"
    ".gitattributes"
    "CODEOWNERS"
    ".editorconfig"
    ".envrc"
    "Makefile"
    "generated_file_demo"
)

# Directories to exclude
EXCLUDE_DIRS=(
    "target"
    "dist"
    "node_modules"
    ".git"
    "logs"
    "status-trends"
    ".cargo/hakari"
    ".hooks"
    ".trunk"
    ".cargo"
)

# Build find command with exclusions
FIND_CMD="find . -type f"
for dir in "${EXCLUDE_DIRS[@]}"; do
    FIND_CMD="$FIND_CMD -not -path './$dir/*'"
done

# Add include patterns
FIND_CMD="$FIND_CMD \("
for i in "${!INCLUDE_PATTERNS[@]}"; do
    if [[ $i -gt 0 ]]; then
        FIND_CMD="$FIND_CMD -o"
    fi
    FIND_CMD="$FIND_CMD -name '${INCLUDE_PATTERNS[$i]}'"
done
FIND_CMD="$FIND_CMD \)"

# Find files and process them
FIRST=true
while IFS= read -r -d '' file; do
    # Skip if it's a .rs or .jsonc file (these are source files)
    if [[ "$file" == *.rs ]] || [[ "$file" == *.jsonc ]]; then
        continue
    fi
    
    # Get relative path (remove leading ./)
    relative_path="${file#./}"
    
    # Get extension
    extension=$(get_extension "$relative_path")
    
    # Generate slug
    slug=$(path_to_slug "$relative_path" "$extension")
    
    # Add comma if not first
    if [[ "$FIRST" == "true" ]]; then
        FIRST=false
    else
        echo "," >> "$OUTPUT_FILE"
    fi
    
    # Add file entry
    cat >> "$OUTPUT_FILE" << EOF
    {
      "slug": "$slug",
      "path": "$relative_path",
      "extension": "$extension"
    }
EOF
    
    echo -e "${GREEN}✅ Added: $relative_path (slug: $slug)${NC}"
    
done < <(eval "$FIND_CMD" -print0 | sort -z)

# Close the JSONC file
cat >> "$OUTPUT_FILE" << 'EOF'
  ]
}
EOF

echo -e "${GREEN}🎉 Generated $OUTPUT_FILE successfully!${NC}"
echo -e "${BLUE}📊 You can now use:${NC}"
echo -e "   cargo xtask gen-files --slug=<slug>"
echo -e "   cargo xtask gen-files --file-type=<extension>" 
