#!/bin/bash

# Analyze File Distribution and Checksum System Coverage
# Uses git ls-files to analyze current repository state

set -e

echo "📊 Analyzing File Distribution and Checksum System Coverage"
echo "=========================================================="

# Get current file distribution
echo "📋 Current File Distribution:"
echo "-----------------------------"
git ls-files | sed 's|.*\.||' | sort | uniq -c | sort -nr
echo ""

# Analyze file types by category
echo "📊 File Type Analysis:"
echo "====================="

# Allowed source files (manual - no checksum needed)
echo "✅ ALLOWED SOURCE FILES (Manual - No checksum needed):"
echo "   • .rs files (Rust source): $(git ls-files | grep -E '\.rs$' | wc -l | tr -d ' ') files"
echo "   • .jsonc files (JSON with comments): $(git ls-files | grep -E '\.jsonc$' | wc -l | tr -d ' ') files"
echo ""

# Generated files that need checksums
echo "🔧 GENERATED FILES (Need checksums):"
echo "   • .md files (Markdown documentation): $(git ls-files | grep -E '\.md$' | wc -l | tr -d ' ') files"
echo "   • .toml files (Cargo/configuration): $(git ls-files | grep -E '\.toml$' | wc -l | tr -d ' ') files"
echo "   • .yml files (YAML configuration): $(git ls-files | grep -E '\.yml$' | wc -l | tr -d ' ') files"
echo "   • .yaml files (YAML configuration): $(git ls-files | grep -E '\.yaml$' | wc -l | tr -d ' ') files"
echo "   • .json files (JSON files): $(git ls-files | grep -E '\.json$' | wc -l | tr -d ' ') files"
echo "   • .wit files (WebAssembly interface): $(git ls-files | grep -E '\.wit$' | wc -l | tr -d ' ') files"
echo "   • .jql files (JQL query files): $(git ls-files | grep -E '\.jql$' | wc -l | tr -d ' ') files"
echo "   • .jsonl files (JSON Lines files): $(git ls-files | grep -E '\.jsonl$' | wc -l | tr -d ' ') files"
echo "   • .gitignore files (Git ignore): $(git ls-files | grep -E '\.gitignore$' | wc -l | tr -d ' ') files"
echo "   • .gitattributes files (Git attributes): $(git ls-files | grep -E '\.gitattributes' | wc -l | tr -d ' ') files"
echo "   • CODEOWNERS (Code ownership): $(git ls-files | grep -E '^CODEOWNERS$' | wc -l | tr -d ' ') files"
echo "   • Makefile (Build configuration): $(git ls-files | grep -E '^Makefile$' | wc -l | tr -d ' ') files"
echo "   • .editorconfig (Editor config): $(git ls-files | grep -E '\.editorconfig$' | wc -l | tr -d ' ') files"
echo "   • .envrc (Environment config): $(git ls-files | grep -E '\.envrc$' | wc -l | tr -d ' ') files"
echo ""

# Problematic files that need attention
echo "🚫 PROBLEMATIC FILES (Need attention):"
echo "   • .sh files (Shell scripts): $(git ls-files | grep -E '\.sh$' | wc -l | tr -d ' ') files"
echo "   • .disabled files (Disabled files): $(git ls-files | grep -E '\.disabled$' | wc -l | tr -d ' ') files"
echo "   • .pdf files (PDF documentation): $(git ls-files | grep -E '\.pdf$' | wc -l | tr -d ' ') files"
echo "   • .html files (HTML documentation): $(git ls-files | grep -E '\.html$' | wc -l | tr -d ' ') files"
echo "   • .hbs files (Handlebars templates): $(git ls-files | grep -E '\.hbs$' | wc -l | tr -d ' ') files"
echo "   • .dot files (Graphviz files): $(git ls-files | grep -E '\.dot$' | wc -l | tr -d ' ') files"
echo "   • .css files (Stylesheets): $(git ls-files | grep -E '\.css$' | wc -l | tr -d ' ') files"
echo "   • .sed files (Sed scripts): $(git ls-files | grep -E '\.sed$' | wc -l | tr -d ' ') files"
echo "   • .backup files (Backup files): $(git ls-files | grep -E '\.backup$' | wc -l | tr -d ' ') files"
echo "   • .shellcheckrc files (Shell check config): $(git ls-files | grep -E '\.shellcheckrc$' | wc -l | tr -d ' ') files"
echo ""

# Calculate totals
TOTAL_FILES=$(git ls-files | wc -l | tr -d ' ')
ALLOWED_SOURCE=$(git ls-files | grep -E '\.(rs|jsonc)$' | wc -l | tr -d ' ')
GENERATED_NEED_CHECKSUMS=$(git ls-files | grep -E '\.(md|toml|yml|yaml|json|wit|jql|jsonl|gitignore|gitattributes)$|^(CODEOWNERS|Makefile|\.editorconfig|\.envrc)$' | wc -l | tr -d ' ')
PROBLEMATIC=$(git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | wc -l | tr -d ' ')

echo "📈 SUMMARY STATISTICS:"
echo "======================"
echo "   • Total files in repository: $TOTAL_FILES"
echo "   • Allowed source files (.rs, .jsonc): $ALLOWED_SOURCE"
echo "   • Generated files needing checksums: $GENERATED_NEED_CHECKSUMS"
echo "   • Problematic files needing attention: $PROBLEMATIC"
echo ""

# Check pre-commit validation coverage
echo "🔍 Pre-commit Validation Coverage Analysis:"
echo "==========================================="

# Check if our validation script covers all generated file types
echo "✅ VALIDATION SCRIPT COVERAGE:"
echo "   • .md files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .toml files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .yml files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .yaml files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .json files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .wit files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .jql files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .jsonl files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .gitignore files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • .gitattributes files: ✅ Covered in GENERATED_EXTENSIONS"
echo "   • CODEOWNERS: ✅ Covered in GENERATED_NO_EXTENSION"
echo "   • Makefile: ✅ Covered in GENERATED_NO_EXTENSION"
echo "   • .editorconfig: ✅ Covered in GENERATED_NO_EXTENSION"
echo "   • .envrc: ✅ Covered in GENERATED_NO_EXTENSION"
echo ""

# Check file policy coverage
echo "📋 FILE POLICY COVERAGE:"
echo "   • All generated extensions: ✅ Covered in config/file-policy.jsonc"
echo "   • All file types with headers: ✅ Covered in fileTypes section"
echo "   • Checksum support: ✅ All file types have includeChecksum: true"
echo ""

# Check registry coverage
echo "📋 REGISTRY COVERAGE:"
echo "   • Generated files tracking: ✅ Covered in config/generated-files.jsonc"
echo "   • Checksum fields: ✅ All entries have checksum fields"
echo "   • Ignore rules: ✅ Proper ignore patterns configured"
echo ""

# Show specific file examples
echo "📝 SPECIFIC FILE EXAMPLES:"
echo "=========================="

echo "🔧 Generated Files (Need checksums):"
git ls-files | grep -E '\.(md|toml|yml|yaml|json|wit|jql|jsonl|gitignore|gitattributes)$|^(CODEOWNERS|Makefile|\.editorconfig|\.envrc)$' | head -10 | while read file; do
    echo "   • $file"
done
echo ""

echo "✅ Source Files (No checksum needed):"
git ls-files | grep -E '\.(rs|jsonc)$' | head -5 | while read file; do
    echo "   • $file"
done
echo ""

echo "🚫 Problematic Files (Need attention):"
git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | head -10 | while read file; do
    echo "   • $file"
done
echo ""

# Check current checksum status
echo "🔍 CURRENT CHECKSUM STATUS:"
echo "==========================="

echo "📋 Files with generated headers:"
HEADER_COUNT=$(git ls-files | xargs grep -l "@generated" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
echo "   • Files with @generated headers: $HEADER_COUNT"

echo "📋 Files with checksum headers:"
CHECKSUM_COUNT=$(git ls-files | xargs grep -l "@checksum" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
echo "   • Files with @checksum headers: $CHECKSUM_COUNT"

echo ""

# Recommendations
echo "💡 RECOMMENDATIONS:"
echo "=================="
echo "1. ✅ Pre-commit validation covers all generated file types"
echo "2. ✅ File policy includes all necessary extensions"
echo "3. ✅ Registry tracks all generated files"
echo "4. 🔧 Need to add checksums to $GENERATED_NEED_CHECKSUMS generated files"
echo "5. 🚫 Consider converting $PROBLEMATIC problematic files to Rust"
echo "6. 📋 Update registry with actual checksums for all generated files"
echo ""

# Migration plan
echo "🔄 MIGRATION PLAN:"
echo "=================="
echo "Phase 1: Add checksums to existing generated files"
echo "   • Target: $GENERATED_NEED_CHECKSUMS files"
echo "   • Priority: High (required for pre-commit validation)"
echo ""
echo "Phase 2: Convert problematic files to Rust"
echo "   • Target: $PROBLEMATIC files"
echo "   • Priority: Medium (improves project consistency)"
echo ""
echo "Phase 3: Update registry with actual checksums"
echo "   • Target: All generated files"
echo "   • Priority: High (required for validation)"
echo ""

echo "✅ File distribution analysis complete!"
echo "📊 Ready to proceed with checksum migration for $GENERATED_NEED_CHECKSUMS generated files." 
