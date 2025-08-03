#!/bin/bash

# Monitor File Distribution Changes
# Tracks file distribution over time to monitor migration progress

set -e

echo "📊 File Distribution Monitor"
echo "============================"

# Get current timestamp
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
echo "📅 Analysis time: $TIMESTAMP"
echo ""

# Get current file distribution
echo "📋 Current File Distribution:"
echo "-----------------------------"
git ls-files | sed 's|.*\.||' | sort | uniq -c | sort -nr
echo ""

# Calculate key metrics
TOTAL_FILES=$(git ls-files | wc -l | tr -d ' ')
ALLOWED_SOURCE=$(git ls-files | grep -E '\.(rs|jsonc)$' | wc -l | tr -d ' ')
GENERATED_NEED_CHECKSUMS=$(git ls-files | while read file; do if head -1 "$file" 2>/dev/null | grep -q "@generated"; then echo "1"; fi; done | wc -l | tr -d ' ')
PROBLEMATIC=$(git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | wc -l | tr -d ' ')

# Check checksum status
HEADER_COUNT=$(git ls-files | xargs grep -l "@generated" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
CHECKSUM_COUNT=$(git ls-files | xargs grep -l "@checksum" 2>/dev/null | wc -l | tr -d ' ' || echo "0")

echo "📈 Key Metrics:"
echo "==============="
echo "   • Total files: $TOTAL_FILES"
echo "   • Allowed source files (.rs, .jsonc): $ALLOWED_SOURCE"
echo "   • Generated files needing checksums: $GENERATED_NEED_CHECKSUMS"
echo "   • Problematic files: $PROBLEMATIC"
echo "   • Files with @generated headers: $HEADER_COUNT"
echo "   • Files with @checksum headers: $CHECKSUM_COUNT"
echo ""

# Calculate progress percentages
if [ $GENERATED_NEED_CHECKSUMS -gt 0 ]; then
    CHECKSUM_PROGRESS=$((CHECKSUM_COUNT * 100 / GENERATED_NEED_CHECKSUMS))
    echo "📊 Migration Progress:"
    echo "====================="
    echo "   • Checksum migration: $CHECKSUM_COUNT/$GENERATED_NEED_CHECKSUMS ($CHECKSUM_PROGRESS%)"
    
    if [ $CHECKSUM_PROGRESS -eq 100 ]; then
        echo "   ✅ Checksum migration complete!"
    elif [ $CHECKSUM_PROGRESS -gt 75 ]; then
        echo "   🔄 Checksum migration nearly complete"
    elif [ $CHECKSUM_PROGRESS -gt 50 ]; then
        echo "   🔄 Checksum migration in progress"
    elif [ $CHECKSUM_PROGRESS -gt 25 ]; then
        echo "   🔄 Checksum migration started"
    else
        echo "   🔧 Checksum migration not started"
    fi
    echo ""
fi

# Show recent changes (if git history available)
echo "🔄 Recent Changes:"
echo "=================="
if git log --oneline -5 --name-status 2>/dev/null | head -10; then
    echo ""
else
    echo "   • No recent changes detected"
    echo ""
fi

# Show files that still need checksums
echo "🔧 Files Still Needing Checksums:"
echo "================================="
NEED_CHECKSUMS=$(git ls-files | grep -E '\.(md|toml|yml|yaml|json|wit|jql|jsonl|gitignore|gitattributes)$|^(CODEOWNERS|Makefile|\.editorconfig|\.envrc)$' | while read file; do
    if ! grep -q "@checksum" "$file" 2>/dev/null; then
        echo "   • $file"
    fi
done | head -10)

if [ -n "$NEED_CHECKSUMS" ]; then
    echo "$NEED_CHECKSUMS"
    REMAINING=$(git ls-files | grep -E '\.(md|toml|yml|yaml|json|wit|jql|jsonl|gitignore|gitattributes)$|^(CODEOWNERS|Makefile|\.editorconfig|\.envrc)$' | while read file; do
        if ! grep -q "@checksum" "$file" 2>/dev/null; then
            echo "1"
        fi
    done | wc -l | tr -d ' ')
    echo "   ... and $REMAINING more files"
else
    echo "   ✅ All generated files have checksums!"
fi
echo ""

# Show problematic files
echo "🚫 Problematic Files (Need attention):"
echo "======================================"
PROBLEMATIC_FILES=$(git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | head -10)
if [ -n "$PROBLEMATIC_FILES" ]; then
    echo "$PROBLEMATIC_FILES" | while read file; do
        echo "   • $file"
    done
    REMAINING_PROBLEMATIC=$(git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | wc -l | tr -d ' ')
    if [ $REMAINING_PROBLEMATIC -gt 10 ]; then
        echo "   ... and $((REMAINING_PROBLEMATIC - 10)) more files"
    fi
else
    echo "   ✅ No problematic files found!"
fi
echo ""

# Quick validation check
echo "🔍 Quick Validation Check:"
echo "========================="
if command -v cargo >/dev/null 2>&1; then
    echo "   • Cargo available: ✅"
    if [ -f "hooks/validate-checksums.rs" ]; then
        echo "   • Validation script exists: ✅"
        echo "   • Pre-commit hook ready: ✅"
    else
        echo "   • Validation script exists: ❌"
        echo "   • Pre-commit hook ready: ❌"
    fi
else
    echo "   • Cargo available: ❌"
fi

if [ -f "lefthook.yml" ]; then
    echo "   • Lefthook configuration: ✅"
else
    echo "   • Lefthook configuration: ❌"
fi

if [ -f "config/file-policy.jsonc" ]; then
    echo "   • File policy configuration: ✅"
else
    echo "   • File policy configuration: ❌"
fi

if [ -f "config/generated-files.jsonc" ]; then
    echo "   • Generated files registry: ✅"
else
    echo "   • Generated files registry: ❌"
fi
echo ""

# Summary
echo "📋 Summary:"
echo "==========="
echo "   • Repository has $TOTAL_FILES total files"
echo "   • $ALLOWED_SOURCE files are allowed source files"
echo "   • $GENERATED_NEED_CHECKSUMS files need checksums"
echo "   • $CHECKSUM_COUNT files have checksums ($CHECKSUM_PROGRESS%)"
echo "   • $PROBLEMATIC files need attention"
echo ""

if [ $CHECKSUM_PROGRESS -eq 100 ] && [ $PROBLEMATIC -eq 0 ]; then
    echo "🎉 All files are properly configured!"
    echo "✅ Pre-commit validation should work perfectly."
else
    echo "🔧 Migration still in progress:"
    if [ $CHECKSUM_PROGRESS -lt 100 ]; then
        echo "   • Add checksums to $((GENERATED_NEED_CHECKSUMS - CHECKSUM_COUNT)) more files"
    fi
    if [ $PROBLEMATIC -gt 0 ]; then
        echo "   • Address $PROBLEMATIC problematic files"
    fi
fi

echo ""
echo "💡 Use './scripts/analyze-file-distribution.sh' for detailed analysis"
echo "💡 Use './scripts/test-pre-commit-checksums.sh' to test validation" 
