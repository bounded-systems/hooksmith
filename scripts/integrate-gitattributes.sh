#!/bin/bash

# integrate-gitattributes.sh
# Integration guide and setup script for the hyperpolyglot-based .gitattributes workflow
#
# This script demonstrates how to integrate the gitattributes workflow
# with various CI/CD systems and development workflows.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Log function
log() {
    local level="$1"
    shift
    local message="$*"
    echo -e "${level}${message}${NC}"
}

# Print header
print_header() {
    echo
    log "$CYAN" "🚀 GitAttributes Integration Guide"
    log "$CYAN" "=================================="
    echo
}

# Print section
print_section() {
    local title="$1"
    echo
    log "$BLUE" "📋 $title"
    log "$BLUE" "$(printf '=%.0s' {1..50})"
    echo
}

# Check prerequisites
check_prerequisites() {
    print_section "Checking Prerequisites"
    
    local missing=()
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        missing+=("Git repository")
    else
        log "$GREEN" "✅ Git repository found"
    fi
    
    # Check if Rust is installed
    if ! command -v rustc > /dev/null 2>&1; then
        missing+=("Rust toolchain")
    else
        log "$GREEN" "✅ Rust toolchain found"
    fi
    
    # Check if hyperpolyglot is installed
    if ! command -v hyply > /dev/null 2>&1; then
        log "$YELLOW" "⚠️  hyperpolyglot not found - will install automatically"
    else
        log "$GREEN" "✅ hyperpolyglot found"
    fi
    
    # Check if scripts exist
    if [[ ! -f "scripts/generate-gitattributes.sh" ]]; then
        missing+=("generate-gitattributes.sh script")
    else
        log "$GREEN" "✅ generate-gitattributes.sh found"
    fi
    
    if [[ ! -f "scripts/ci-gitattributes.sh" ]]; then
        missing+=("ci-gitattributes.sh script")
    else
        log "$GREEN" "✅ ci-gitattributes.sh found"
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log "$RED" "❌ Missing prerequisites:"
        for item in "${missing[@]}"; do
            log "$RED" "   - $item"
        done
        return 1
    fi
    
    log "$GREEN" "✅ All prerequisites met!"
    return 0
}

# Setup scripts
setup_scripts() {
    print_section "Setting Up Scripts"
    
    # Make scripts executable
    chmod +x scripts/generate-gitattributes.sh
    chmod +x scripts/ci-gitattributes.sh
    
    # Compile Rust script
    if [[ -f "scripts/generate-gitattributes.rs" ]]; then
        log "$BLUE" "🔧 Compiling Rust script..."
        rustc scripts/generate-gitattributes.rs -o scripts/generate-gitattributes
        log "$GREEN" "✅ Rust script compiled"
    fi
    
    log "$GREEN" "✅ Scripts ready for use"
}

# Test the workflow
test_workflow() {
    print_section "Testing Workflow"
    
    log "$BLUE" "🧪 Running dry-run test..."
    if ./scripts/generate-gitattributes.sh -d > /dev/null 2>&1; then
        log "$GREEN" "✅ Workflow test successful"
    else
        log "$RED" "❌ Workflow test failed"
        return 1
    fi
}

# Show integration examples
show_integrations() {
    print_section "Integration Examples"
    
    log "$CYAN" "1. GitHub Actions Integration"
    cat << 'EOF'
Add to your .github/workflows/ci.yml:
```yaml
- name: Update .gitattributes
  run: |
    ./scripts/ci-gitattributes.sh -f
    git add .gitattributes
    git commit -m "Update .gitattributes" || echo "No changes"
```
EOF

    log "$CYAN" "2. Lefthook Integration"
    cat << 'EOF'
Add to your lefthook.yml:
```yaml
pre-commit:
  commands:
    gitattributes:
      run: ./scripts/ci-gitattributes.sh -c || (echo "Update .gitattributes needed" && exit 1)
```
EOF

    log "$CYAN" "3. Pre-commit Hook"
    cat << 'EOF'
Add to .git/hooks/pre-commit:
```bash
#!/bin/bash
if ! ./scripts/ci-gitattributes.sh -c; then
    echo "⚠️  .gitattributes needs updating"
    echo "Run: ./scripts/generate-gitattributes.sh"
    exit 1
fi
```
EOF

    log "$CYAN" "4. Local Development"
    cat << 'EOF'
For local development, run:
```bash
# Generate .gitattributes
./scripts/generate-gitattributes.sh

# Check if update needed
./scripts/ci-gitattributes.sh -c

# Force update
./scripts/ci-gitattributes.sh -f
```
EOF
}

# Show usage examples
show_usage() {
    print_section "Usage Examples"
    
    log "$CYAN" "Basic Usage:"
    echo "  ./scripts/generate-gitattributes.sh          # Generate .gitattributes"
    echo "  ./scripts/generate-gitattributes.sh -d       # Dry run"
    echo "  ./scripts/generate-gitattributes.sh -f -b    # Force with backup"
    echo
    
    log "$CYAN" "CI Usage:"
    echo "  ./scripts/ci-gitattributes.sh -c             # Check if update needed"
    echo "  ./scripts/ci-gitattributes.sh -f             # Force update"
    echo "  ./scripts/ci-gitattributes.sh -v             # Verbose output"
    echo
    
    log "$CYAN" "GitHub Actions:"
    echo "  # Manual trigger via GitHub Actions UI"
    echo "  # Automatic trigger on file changes"
    echo "  # Automatic commit and push of changes"
}

# Show benefits
show_benefits() {
    print_section "Benefits"
    
    log "$GREEN" "✅ Accurate Language Detection"
    echo "   - Uses hyperpolyglot for fast, accurate detection"
    echo "   - Supports 400+ programming languages"
    echo "   - Based on GitHub's Linguist library"
    echo
    
    log "$GREEN" "✅ Automated Workflow"
    echo "   - Automatic detection of file type changes"
    echo "   - CI/CD integration with GitHub Actions"
    echo "   - Checksum-based change detection"
    echo
    
    log "$GREEN" "✅ GitHub Integration"
    echo "   - Proper language statistics on GitHub"
    echo "   - Accurate syntax highlighting"
    echo "   - Excludes generated, documentation, and vendored files"
    echo
    
    log "$GREEN" "✅ Performance"
    echo "   - Fast detection with hyperpolyglot"
    echo "   - Incremental updates"
    echo "   - Multi-threaded processing"
}

# Main execution
main() {
    print_header
    
    log "$BLUE" "This script helps you integrate the hyperpolyglot-based .gitattributes workflow"
    log "$BLUE" "into your development and CI/CD pipelines."
    echo
    
    # Check prerequisites
    if ! check_prerequisites; then
        log "$RED" "❌ Please fix the missing prerequisites and run again"
        exit 1
    fi
    
    # Setup scripts
    setup_scripts
    
    # Test workflow
    if ! test_workflow; then
        log "$RED" "❌ Workflow test failed. Please check the setup."
        exit 1
    fi
    
    # Show integrations
    show_integrations
    
    # Show usage
    show_usage
    
    # Show benefits
    show_benefits
    
    echo
    log "$GREEN" "🎉 Integration guide complete!"
    log "$GREEN" "Your hyperpolyglot-based .gitattributes workflow is ready to use."
    echo
}

# Run main function
main "$@"
