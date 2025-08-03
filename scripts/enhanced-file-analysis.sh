#!/bin/bash

# Enhanced File Analysis
# Shows file distribution with generation and checksum status

set -e

echo "📊 Enhanced File Distribution Analysis"
echo "======================================"

# Get current timestamp
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
echo "📅 Analysis time: $TIMESTAMP"
echo ""

# Function to check if file is generated
is_generated() {
    local file="$1"
    if head -1 "$file" 2>/dev/null | grep -q "@generated"; then
        return 0
    else
        return 1
    fi
}

# Function to check if file has valid checksum
has_valid_checksum() {
    local file="$1"
    if grep -q "@checksum" "$file" 2>/dev/null; then
        # Test with validation script
        if RUSTC_WRAPPER="" ./target/debug/validate-checksums "$file" >/dev/null 2>&1; then
            return 0
        else
            return 1
        fi
    else
        return 1
    fi
}

# Function to get file status
get_file_status() {
    local file="$1"
    local extension="$2"
    
    if is_generated "$file"; then
        if has_valid_checksum "$file"; then
            echo "✅ Generated + Valid Checksum"
        else
            echo "⚠️  Generated + Invalid/Missing Checksum"
        fi
    else
        case "$extension" in
            "rs"|"jsonc")
                echo "📝 Source File (Allowed)"
                ;;
            *)
                echo "❌ Non-Generated (Problematic)"
                ;;
        esac
    fi
}

# Get file distribution with status
echo "📋 File Distribution with Status:"
echo "================================="

# Process each file and collect data
git ls-files | while read file; do
    if [[ -f "$file" ]]; then
        # Get extension
        extension=$(echo "$file" | sed 's|.*\.||' | tr '[:upper:]' '[:lower:]')
        if [[ "$extension" == "$file" ]]; then
            # No extension
            basename=$(basename "$file")
            case "$basename" in
                "CODEOWNERS"|"Makefile"|".editorconfig"|".envrc"|".gitignore"|".gitattributes"*)
                    extension="$basename"
                    ;;
                *)
                    extension="no-ext"
                    ;;
            esac
        fi
        
        # Get status
        status=$(get_file_status "$file" "$extension")
        
        # Store data
        echo "$extension|$status"
    fi
done | sort | uniq -c | sort -nr | while read count info; do
    extension=$(echo "$info" | cut -d'|' -f1)
    status=$(echo "$info" | cut -d'|' -f2)
    
    # Format the output
    printf "%4s %-20s %s\n" "$count" "$extension" "$status"
done

echo ""

# Summary statistics
echo "📈 Summary Statistics:"
echo "====================="

TOTAL_FILES=$(git ls-files | wc -l | tr -d ' ')
GENERATED_FILES=$(git ls-files | while read file; do if is_generated "$file"; then echo "1"; fi; done | wc -l | tr -d ' ')
VALID_CHECKSUMS=$(git ls-files | while read file; do if has_valid_checksum "$file"; then echo "1"; fi; done | wc -l | tr -d ' ')
SOURCE_FILES=$(git ls-files | grep -E '\.(rs|jsonc)$' | wc -l | tr -d ' ')
PROBLEMATIC_FILES=$(git ls-files | grep -E '\.(sh|disabled|pdf|html|hbs|dot|css|sed|backup|shellcheckrc)$' | wc -l | tr -d ' ')

echo "   • Total files: $TOTAL_FILES"
echo "   • Generated files: $GENERATED_FILES"
echo "   • Files with valid checksums: $VALID_CHECKSUMS"
echo "   • Source files (.rs, .jsonc): $SOURCE_FILES"
echo "   • Problematic files: $PROBLEMATIC_FILES"
echo ""

# Calculate percentages
if [ $GENERATED_FILES -gt 0 ]; then
    CHECKSUM_PERCENTAGE=$((VALID_CHECKSUMS * 100 / GENERATED_FILES))
    echo "📊 Checksum Coverage: $VALID_CHECKSUMS/$GENERATED_FILES ($CHECKSUM_PERCENTAGE%)"
    
    if [ $CHECKSUM_PERCENTAGE -eq 100 ]; then
        echo "   ✅ All generated files have valid checksums!"
    elif [ $CHECKSUM_PERCENTAGE -gt 90 ]; then
        echo "   🔄 Nearly complete - just a few files need checksums"
    elif [ $CHECKSUM_PERCENTAGE -gt 75 ]; then
        echo "   🔄 Good progress - most files have checksums"
    elif [ $CHECKSUM_PERCENTAGE -gt 50 ]; then
        echo "   🔄 In progress - about half the files have checksums"
    else
        echo "   🔧 Early stages - many files still need checksums"
    fi
fi

echo ""

# Show files that need attention
echo "🔧 Files Needing Attention:"
echo "=========================="

echo "📝 Generated files without valid checksums:"
GENERATED_NO_CHECKSUM=0
git ls-files | while read file; do
    if is_generated "$file" && ! has_valid_checksum "$file"; then
        echo "   • $file"
        GENERATED_NO_CHECKSUM=$((GENERATED_NO_CHECKSUM + 1))
    fi
done | head -10

if [ $GENERATED_NO_CHECKSUM -gt 10 ]; then
    echo "   ... and $((GENERATED_NO_CHECKSUM - 10)) more files"
fi

echo ""
echo "❌ Non-generated files with disallowed extensions:"
NON_GENERATED_PROBLEMATIC=0
git ls-files | while read file; do
    if ! is_generated "$file"; then
        extension=$(echo "$file" | sed 's|.*\.||' | tr '[:upper:]' '[:lower:]')
        case "$extension" in
            "rs"|"jsonc"|"no-ext")
                # These are allowed
                ;;
            *)
                echo "   • $file (extension: $extension)"
                NON_GENERATED_PROBLEMATIC=$((NON_GENERATED_PROBLEMATIC + 1))
                ;;
        esac
    fi
done | head -10

if [ $NON_GENERATED_PROBLEMATIC -gt 10 ]; then
    echo "   ... and $((NON_GENERATED_PROBLEMATIC - 10)) more files"
fi

echo ""

echo "💡 Next Steps:"
echo "=============="
echo "   • Run './scripts/migrate-all-checksums.sh' to add missing checksums"
echo "   • Convert problematic files to Rust or remove them"
echo "   • Test pre-commit validation with 'git commit --allow-empty'" 
