#!/bin/bash

# Test script to add checksums to a few files
# This is a test version of the full migration script

set -e

echo "🧪 Testing Checksum Addition"
echo "============================"

# Test files
TEST_FILES=(
    ".cargo/aliases.toml"
    ".cargo/config.toml"
    ".editorconfig"
)

echo "📋 Testing with ${#TEST_FILES[@]} files..."
echo ""

for file in "${TEST_FILES[@]}"; do
    echo "🔍 Processing $file..."
    
    # Check current state
    echo "   Before:"
    head -3 "$file"
    
    # Add checksum
    if ! grep -q "@checksum" "$file" 2>/dev/null; then
        # Determine file type
        file_type=""
        if [[ "$file" == *.toml ]]; then
            file_type="toml"
        elif [[ "$file" == ".editorconfig" ]]; then
            file_type="editorconfig"
        fi
        
        # Get content without header
        content=$(tail -n +2 "$file")
        
        # Compute checksum
        checksum=$(echo -n "$content" | sha256sum | cut -c1-8)
        
        # Create new content
        header=$(head -1 "$file")
        new_content="$header\n# @checksum: $checksum\n$content"
        
        # Write back to file
        echo -e "$new_content" > "$file"
        
        echo "   ✅ Added checksum: $checksum"
    else
        echo "   ⏭️  Already has checksum"
    fi
    
    echo "   After:"
    head -3 "$file"
    echo ""
done

echo "✅ Test completed!"
echo ""
echo "🔍 Next steps:"
echo "   1. Test validation: ./target/debug/validate-checksums .cargo/aliases.toml"
echo "   2. If successful, run the full migration script" 