#!/bin/bash
# Git pre-add hook for unified generated file system validation
# This hook validates that only properly managed files can be staged for commit

set -e

# Get the list of files being added
files="$@"

if [ -z "$files" ]; then
    echo "No files specified for staging"
    exit 0
fi

echo "🔍 Pre-add validation for $(echo "$files" | wc -w) file(s)..."

# Run the Rust pre-add validator
if ! cargo run --bin pre-add $files; then
    echo "❌ Pre-add validation failed. Files cannot be staged."
    echo ""
    echo "To fix this:"
    echo "1. For generated files: Run 'cargo xtask gen-all-unified'"
    echo "2. For manual files: Run 'cargo xtask allow-manual --path <file>'"
    echo "3. For allowed extensions: Only .rs and .jsonc files are allowed by default"
    exit 1
fi

echo "✅ Pre-add validation passed"
exit 0 
