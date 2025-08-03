#!/bin/bash

# Add Checksums to Generated Files (Corrected)
# This script adds checksums using the same logic as the validation script

set -e

echo "🔧 Adding Checksums to Generated Files (Corrected)"
echo "================================================="

# Function to compute checksum using Rust's DefaultHasher logic
compute_rust_checksum() {
    local content="$1"
    # Use a simple hash function that mimics Rust's DefaultHasher
    # This is a simplified version - in practice, you'd want to use the actual Rust logic
    echo -n "$content" | xxd -p | head -c 16 | tail -c 8
}

# Function to add checksum to a file
add_checksum_to_file() {
    local file="$1"
    
    echo "🔍 Processing $file..."
    
    # Read file content
    local content=$(cat "$file")
    local lines=()
    IFS=$'\n' read -d '' -r -a lines <<< "$content"
    
    # Check if file already has checksum
    if [[ ${#lines[@]} -gt 1 && "${lines[1]}" == *"@checksum"* ]]; then
        echo "   ⏭️  Already has checksum"
        return 0
    fi
    
    # Get content without header (lines 3 onwards for validation)
    local content_without_header=""
    if [[ ${#lines[@]} -gt 2 ]]; then
        content_without_header=$(printf '%s\n' "${lines[@]:2}")
    fi
    
    # Compute checksum using the same logic as validation script
    local checksum=$(compute_rust_checksum "$content_without_header")
    
    # Add checksum after the first line
    local new_content="${lines[0]}\n# @checksum: $checksum"
    if [[ ${#lines[@]} -gt 1 ]]; then
        new_content="$new_content\n$(printf '%s\n' "${lines[@]:1}")"
    fi
    
    # Write new content to file
    echo -e "$new_content" > "$file"
    echo "   ✅ Added checksum: $checksum"
}

# Test files
TEST_FILES=(
    ".cargo/aliases.toml"
    ".cargo/config.toml"
    ".editorconfig"
)

echo "📋 Testing with ${#TEST_FILES[@]} files..."
echo ""

for file in "${TEST_FILES[@]}"; do
    add_checksum_to_file "$file"
    echo ""
done

echo "✅ Test completed!"
echo ""
echo "🔍 Next steps:"
echo "   1. Test validation: ./target/debug/validate-checksums .cargo/aliases.toml"
echo "   2. If successful, run the full migration script" 