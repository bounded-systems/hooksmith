#!/bin/bash

# Migrate All Checksums
# This script adds checksums to all generated files that don't have them yet

set -e

echo "🚀 Migrating All Checksums"
echo "=========================="

# Get list of files that need checksums
echo "📋 Finding files that need checksums..."
FILES_NEEDING_CHECKSUMS=$(git ls-files | grep -E '\.(md|toml|yml|yaml|json|wit|jql|jsonl|gitignore|gitattributes)$|^(CODEOWNERS|Makefile|\.editorconfig|\.envrc)$' | while read file; do
    if ! grep -q "@checksum" "$file" 2>/dev/null; then
        echo "$file"
    fi
done)

TOTAL_FILES=$(echo "$FILES_NEEDING_CHECKSUMS" | wc -l | tr -d ' ')
echo "Found $TOTAL_FILES files needing checksums"
echo ""

# Create a temporary file list
TEMP_FILE_LIST="/tmp/files_needing_checksums.txt"
echo "$FILES_NEEDING_CHECKSUMS" > "$TEMP_FILE_LIST"

# Process files in batches
BATCH_SIZE=10
COUNTER=0

echo "🔧 Adding checksums to files..."
echo ""

while IFS= read -r file; do
    if [[ -n "$file" ]]; then
        COUNTER=$((COUNTER + 1))
        echo "[$COUNTER/$TOTAL_FILES] Processing $file..."
        
        # Add checksum using our Rust script
        if RUSTC_WRAPPER="" ./target/debug/add-checksums "$file" > /dev/null 2>&1; then
            echo "   ✅ Success"
        else
            echo "   ❌ Failed"
        fi
        
        # Process in batches to avoid overwhelming the system
        if [[ $((COUNTER % BATCH_SIZE)) -eq 0 ]]; then
            echo "   📊 Processed $COUNTER/$TOTAL_FILES files..."
            echo ""
        fi
    fi
done < "$TEMP_FILE_LIST"

# Clean up
rm -f "$TEMP_FILE_LIST"

echo ""
echo "✅ Migration completed!"
echo ""
echo "📊 Summary:"
echo "   • Processed $TOTAL_FILES files"
echo "   • Added checksums to generated files"
echo ""
echo "🔍 Next steps:"
echo "   1. Test validation: ./target/debug/validate-checksums <file>"
echo "   2. Test pre-commit hooks: git commit --allow-empty -m 'Test pre-commit validation'"
echo "   3. Monitor progress: ./scripts/monitor-file-distribution.sh" 