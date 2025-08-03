#!/bin/bash

# macOS Optimization Script for Hooksmith
# Safely enables developer mode and other macOS-specific optimizations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    log_error "This script is for macOS only"
    exit 1
fi

echo "🍎 macOS Optimization Setup for Hooksmith"
echo "=========================================="
echo

# 1. Check macOS version
log_info "Checking macOS version..."
macos_version=$(sw_vers -productVersion)
echo "  macOS Version: $macos_version"

# Check if we're on a supported version
if [[ "$macos_version" < "11.0" ]]; then
    log_warning "macOS 11.0+ recommended for optimal performance"
fi

echo

# 2. Check current Gatekeeper status
log_info "Checking Gatekeeper status..."
gatekeeper_status=$(spctl --status)
echo "  Gatekeeper: $gatekeeper_status"

if [[ "$gatekeeper_status" == "assessments enabled" ]]; then
    log_success "✅ Gatekeeper is enabled (recommended)"
else
    log_warning "⚠️ Gatekeeper is disabled (less secure)"
fi

echo

# 3. Enable developer mode for Terminal
log_info "Setting up developer mode for Terminal..."

# Check if developer mode is already enabled
if spctl developer-mode enable-terminal 2>/dev/null; then
    log_success "✅ Developer mode enabled for Terminal"
    echo "  Terminal has been added to Developer Tools"
else
    log_warning "⚠️ Could not enable developer mode"
    echo "  This may require manual setup"
fi

echo

# 4. Provide setup instructions
log_info "Manual Setup Required:"
echo "=========================="
echo
echo "1. Open System Settings → Privacy & Security → Developer Tools"
echo "2. Find 'Terminal' in the list and check the box"
echo "3. Restart your terminal application"
echo "4. Optionally run 'cargo clean' to clear any cached Gatekeeper effects"
echo

# 5. Security information
log_info "Security Information:"
echo "======================="
echo
echo "✅ What this does:"
echo "  • Allows Terminal and its child processes to bypass quarantine checks"
echo "  • Enables faster execution of development tools (cargo, cargo-nextest, etc.)"
echo "  • Reduces Gatekeeper overhead for iterative development"
echo
echo "🛡️ Security scope:"
echo "  • Only affects Terminal and processes launched from it"
echo "  • GUI apps launched from Finder are still protected"
echo "  • System-wide Gatekeeper protection remains active"
echo
echo "⚠️ Risks to be aware of:"
echo "  • Terminal can now run unsigned binaries without prompts"
echo "  • Only enable on trusted development machines"
echo "  • Don't enable on production or shared systems"
echo

# 6. Verification steps
log_info "Verification Steps:"
echo "====================="
echo
echo "After completing the manual setup:"
echo
echo "1. Test faster execution:"
echo "   cargo test --verbose"
echo "   # Should run without Gatekeeper delays"
echo
echo "2. Test cargo-nextest:"
echo "   cargo nextest run"
echo "   # Should execute tests faster"
echo
echo "3. Monitor performance:"
echo "   ./scripts/build-stats.sh"
echo "   # Check for improved build times"
echo

# 7. Disable instructions
log_info "How to Disable:"
echo "=================="
echo
echo "To disable developer mode:"
echo "1. Run: spctl developer-mode disable-terminal"
echo "2. Remove Terminal from System Settings → Privacy & Security → Developer Tools"
echo "3. Restart terminal application"
echo

# 8. Additional optimizations
log_info "Additional Optimizations:"
echo "============================"
echo
echo "✅ Already configured in Hooksmith:"
echo "  • LLD linker for optimal Apple Silicon performance"
echo "  • split-debuginfo = 'unpacked' for faster debug builds"
echo "  • sccache for build caching"
echo "  • cargo-hakari for workspace optimization"
echo "  • cargo-nextest for parallel testing"
echo

# 9. Performance monitoring
log_info "Performance Monitoring:"
echo "=========================="
echo
echo "Monitor optimization effectiveness:"
echo "• make stats          # Build performance statistics"
echo "• cargo build --timings  # Detailed build timing analysis"
echo "• sccache --show-stats   # Cache hit rates"
echo

# 10. Best practices
log_info "Best Practices:"
echo "=================="
echo
echo "✅ Recommended:"
echo "  • Only enable on development workstations"
echo "  • Keep Terminal in Developer Tools list minimal"
echo "  • Regularly audit what's running in your terminal"
echo "  • Disable when not actively developing"
echo
echo "❌ Avoid:"
echo "  • Enabling on production or shared systems"
echo "  • Running untrusted scripts without verification"
echo "  • Leaving enabled on systems with multiple users"
echo

echo
log_success "🎉 macOS optimization setup complete!"
echo
echo "Next steps:"
echo "1. Complete the manual setup in System Settings"
echo "2. Restart your terminal"
echo "3. Test the optimizations with: make dev"
echo "4. Monitor performance with: make stats"
echo
echo "For more information, see: MACOS_OPTIMIZATIONS.md" 
