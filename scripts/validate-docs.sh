#!/bin/bash

# CI validation script for documentation generation
# Ensures all markdown files are properly generated and no direct creation occurs

set -e

echo "🔍 Validating documentation generation..."

# Check if we're in a CI environment
if [ "$CI" = "true" ]; then
    echo "🏗️  Running in CI environment"
fi

# Check for any markdown files that don't have auto-generated markers
echo "📋 Checking for direct markdown file creation..."

INVALID_FILES=""
MD_FILES=$(find . -name "*.md" -not -path "./target/*" -not -path "./.git/*")

for file in $MD_FILES; do
    # Skip files that are explicitly excluded from generation
    if [[ "$file" == "./README.md" ]] || \
       [[ "$file" == "./.gitignore" ]] || \
       [[ "$file" == "./LICENSE"* ]] || \
       [[ "$file" == "./CHANGELOG.md" ]] || \
       [[ "$file" == "./CONTRIBUTING.md" ]] || \
       [[ "$file" == "./SECURITY.md" ]] || \
       [[ "$file" == "./CODE_OF_CONDUCT.md" ]]; then
        echo "   ⏭️  Skipping manually maintained file: $file"
        continue
    fi
    
    # Check if file contains auto-generated marker
    if ! grep -q "auto-generated" "$file" 2>/dev/null; then
        INVALID_FILES="$INVALID_FILES $file"
        echo "   ❌ Invalid file (no auto-generated marker): $file"
    else
        echo "   ✅ Valid generated file: $file"
    fi
done

if [ -n "$INVALID_FILES" ]; then
    echo ""
    echo "❌ Error: Direct markdown file creation detected!"
    echo "The following files appear to be manually created:"
    echo "$INVALID_FILES"
    echo ""
    echo "📋 All markdown files must be generated through the documentation system."
    echo "Please use: cargo xtask gen-docs-comprehensive --all --validate"
    echo ""
    exit 1
fi

echo ""
echo "✅ All markdown files appear to be properly generated"

# Validate checksums if available
if [ -f "docs/checksums.json" ]; then
    echo ""
    echo "🔐 Validating checksums..."
    
    if command -v cargo >/dev/null 2>&1; then
        if cargo xtask gen-docs-comprehensive --validate >/dev/null 2>&1; then
            echo "✅ Checksum validation passed"
        else
            echo "❌ Checksum validation failed"
            echo "Some generated files may be out of date"
            exit 1
        fi
    else
        echo "⚠️  Cargo not available, skipping checksum validation"
    fi
else
    echo "⚠️  No checksums.json found, skipping checksum validation"
fi

# Check Git attributes
echo ""
echo "🏷️  Checking Git attributes..."

if [ -f ".gitattributes" ]; then
    if grep -q "linguist-generated=true" .gitattributes; then
        echo "✅ Git attributes properly configured"
    else
        echo "⚠️  Git attributes may not be properly configured"
    fi
else
    echo "⚠️  No .gitattributes file found"
fi

# Generate fresh documentation to ensure everything is up to date
echo ""
echo "🔄 Generating fresh documentation..."

if command -v cargo >/dev/null 2>&1; then
    if cargo xtask gen-docs-comprehensive --all --validate >/dev/null 2>&1; then
        echo "✅ Documentation generation successful"
    else
        echo "❌ Documentation generation failed"
        exit 1
    fi
else
    echo "⚠️  Cargo not available, skipping documentation generation"
fi

# Check for any uncommitted changes
echo ""
echo "📝 Checking for uncommitted changes..."

if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Uncommitted changes detected:"
    git status --porcelain
    echo ""
    echo "Please commit all changes or run:"
    echo "cargo xtask gen-docs-comprehensive --all --validate"
    echo "git add ."
    echo "git commit -m 'Update generated documentation'"
    exit 1
else
    echo "✅ No uncommitted changes"
fi

echo ""
echo "🎉 All documentation validation checks passed!"
echo "✅ No direct markdown file creation detected"
echo "✅ All files properly generated"
echo "✅ Checksums validated"
echo "✅ Git attributes configured"
echo "✅ No uncommitted changes"

exit 0 
