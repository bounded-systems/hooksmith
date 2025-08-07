#!/bin/bash

# demo-workflow.sh
# Demo script showing the complete hyperpolyglot + git ls-files workflow
#
# This script demonstrates the dual enforcement strategy:
# 1. Local development with Git hooks
# 2. CI/CD with GitHub Actions

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Log function
log() {
    local level="$1"
    shift
    local message="$*"
    echo -e "${level}${message}${NC}"
}

# Print section header
print_section() {
    local title="$1"
    echo
    log "$PURPLE" "🎯 $title"
    log "$PURPLE" "$(printf '=%.0s' {1..60})"
    echo
}

# Demo the complete workflow
main() {
    log "$CYAN" "🚀 Hyperpolyglot + Git ls-files Workflow Demo"
    log "$CYAN" "================================================"
    echo
    
    log "$BLUE" "This demo shows the complete workflow for:"
    log "$BLUE" "• Using git ls-files to get tracked files"
    log "$BLUE" "• Using hyperpolyglot for language detection"
    log "$BLUE" "• Generating .gitattributes for GitHub Linguist"
    log "$BLUE" "• Dual enforcement with local hooks and CI"
    echo
    
    # Step 1: Show current repository state
    print_section "Step 1: Current Repository State"
    
    log "$GREEN" "📁 Repository files:"
    git ls-files --cached --full-name | head -10 | while read file; do
        echo "   $file"
    done
    echo "... and $(($(git ls-files --cached | wc -l) - 10)) more files"
    
    log "$GREEN" "📊 Current .gitattributes:"
    if [[ -f ".gitattributes" ]]; then
        echo "   ✅ .gitattributes exists ($(wc -l < .gitattributes) lines)"
    else
        echo "   ❌ No .gitattributes file"
    fi
    
    # Step 2: Demonstrate hyperpolyglot detection
    print_section "Step 2: Hyperpolyglot Language Detection"
    
    log "$GREEN" "🔍 Detecting languages for sample files:"
    
    # Test on a few sample files
    SAMPLE_FILES=$(find . -name "*.rs" -o -name "*.sh" -o -name "*.yml" | head -3)
    
    for file in $SAMPLE_FILES; do
        if [[ -f "$file" ]]; then
            LANGUAGE=$(hyply --breakdown "$file" 2>/dev/null | grep -E '^[0-9.]+%' | head -1 | awk '{print $2}' || echo "Unknown")
            echo "   $file → $LANGUAGE"
        fi
    done
    
    # Step 3: Show .gitattributes generation
    print_section "Step 3: .gitattributes Generation"
    
    log "$GREEN" "🔄 Generating .gitattributes..."
    
    # Create a temporary .gitattributes for demo
    TEMP_GITATTRIBUTES=".gitattributes.demo"
    ./scripts/generate-gitattributes.sh -o "$TEMP_GITATTRIBUTES" -f
    
    log "$GREEN" "📋 Generated .gitattributes preview:"
    head -20 "$TEMP_GITATTRIBUTES"
    echo "..."
    
    # Step 4: Demonstrate local hook validation
    print_section "Step 4: Local Hook Validation (Simulated)"
    
    log "$GREEN" "🔍 Simulating pre-commit hook validation..."
    
    # Simulate the pre-commit hook logic
    STAGED_FILES=$(git diff --cached --name-only 2>/dev/null || echo "")
    
    if [[ -n "$STAGED_FILES" ]]; then
        log "$YELLOW" "📁 Found staged files:"
        echo "$STAGED_FILES" | while read file; do
            if [[ -f "$file" ]] && [[ "$file" =~ \.[a-zA-Z0-9]+$ ]]; then
                if grep -q "^$file linguist-language=" "$TEMP_GITATTRIBUTES" 2>/dev/null; then
                    echo "   ✅ $file (has .gitattributes entry)"
                else
                    LANGUAGE=$(hyply --breakdown "$file" 2>/dev/null | grep -E '^[0-9.]+%' | head -1 | awk '{print $2}' || echo "Unknown")
                    echo "   ❌ $file (missing entry, detected: $LANGUAGE)"
                fi
            fi
        done
    else
        log "$GREEN" "✅ No staged files to validate"
    fi
    
    # Step 5: Show CI validation simulation
    print_section "Step 5: CI Validation (Simulated)"
    
    log "$GREEN" "🔍 Simulating GitHub Actions validation..."
    
    # Simulate CI checks
    if [[ -f "$TEMP_GITATTRIBUTES" ]]; then
        log "$GREEN" "✅ .gitattributes file exists"
        
        # Check format
        if grep -q "# LANGUAGE DETECTION RESULTS" "$TEMP_GITATTRIBUTES"; then
            log "$GREEN" "✅ .gitattributes format is valid"
        else
            log "$RED" "❌ .gitattributes format is invalid"
        fi
        
        # Show language statistics
        log "$GREEN" "📊 Language statistics:"
        grep "linguist-language=" "$TEMP_GITATTRIBUTES" | \
            sed 's/.*linguist-language=//' | \
            sort | uniq -c | \
            sort -nr | \
            while read count language; do
                echo "   $language: $count files"
            done
    else
        log "$RED" "❌ No .gitattributes file found"
    fi
    
    # Step 6: Show integration examples
    print_section "Step 6: Integration Examples"
    
    log "$CYAN" "🔧 Lefthook Integration:"
    cat << 'EOF'
# lefthook.yml
pre-commit:
  commands:
    gitattributes:
      run: |
        # Validate staged files have .gitattributes entries
        STAGED_FILES=$(git diff --cached --name-only)
        for file in $STAGED_FILES; do
          if [[ "$file" =~ \.[a-zA-Z0-9]+$ ]]; then
            if ! grep -q "^$file linguist-language=" .gitattributes; then
              echo "Missing .gitattributes entry for $file"
              exit 1
            fi
          fi
        done
EOF

    log "$CYAN" "🚀 GitHub Actions Integration:"
    cat << 'EOF'
# .github/workflows/language-validation.yml
- name: Validate .gitattributes
  run: |
    # Same logic as pre-commit hook
    # But runs on all PR changes
    CHANGED_FILES=$(git diff --name-only origin/main...HEAD)
    for file in $CHANGED_FILES; do
      # Validation logic here
    done
EOF
    
    # Step 7: Show usage commands
    print_section "Step 7: Usage Commands"
    
    log "$GREEN" "📝 Common commands:"
    echo
    echo "  # Generate .gitattributes"
    echo "  ./scripts/generate-gitattributes.sh"
    echo
    echo "  # Check if update needed"
    echo "  ./scripts/ci-gitattributes.sh -c"
    echo
    echo "  # Force update"
    echo "  ./scripts/ci-gitattributes.sh -f"
    echo
    echo "  # Dry run"
    echo "  ./scripts/generate-gitattributes.sh -d"
    echo
    echo "  # Run local hooks"
    echo "  lefthook run pre-commit"
    echo
    echo "  # Show language statistics"
    echo "  grep 'linguist-language=' .gitattributes | sed 's/.*=//' | sort | uniq -c"
    
    # Step 8: Cleanup and summary
    print_section "Step 8: Summary"
    
    log "$GREEN" "✅ Workflow Components:"
    echo "   • git ls-files - Get tracked files"
    echo "   • hyperpolyglot - Detect languages"
    echo "   • .gitattributes - GitHub Linguist integration"
    echo "   • Local hooks - Fast feedback"
    echo "   • CI validation - Repository enforcement"
    echo
    
    log "$GREEN" "🎯 Benefits:"
    echo "   • Accurate language statistics on GitHub"
    echo "   • Proper syntax highlighting"
    echo "   • Automated file type detection"
    echo "   • Dual enforcement (local + CI)"
    echo "   • Fast local feedback"
    echo
    
    log "$GREEN" "🚀 Next Steps:"
    echo "   1. Review the generated .gitattributes"
    echo "   2. Commit the changes to your repository"
    echo "   3. Set up the GitHub Actions workflow"
    echo "   4. Configure Lefthook for local hooks"
    echo "   5. Test the complete workflow"
    echo
    
    # Cleanup
    if [[ -f "$TEMP_GITATTRIBUTES" ]]; then
        rm "$TEMP_GITATTRIBUTES"
    fi
    
    log "$CYAN" "🎉 Demo complete! Your hyperpolyglot + git ls-files workflow is ready."
}

# Run the demo
main "$@"
