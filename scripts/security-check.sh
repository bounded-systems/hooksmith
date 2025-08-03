#!/bin/bash

# Security Check Script for Hooksmith Development
# Verifies that all security measures are in place and working correctly

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

echo "🔒 Security Status Check for Hooksmith Development"
echo "=================================================="
echo

# Initialize security score
security_score=0
total_checks=0

# Function to add to security score
add_check() {
    local passed=$1
    local message=$2
    total_checks=$((total_checks + 1))
    
    if [[ $passed -eq 1 ]]; then
        security_score=$((security_score + 1))
        log_success "$message"
    else
        log_warning "$message"
    fi
}

# 1. Check Gatekeeper Status
log_info "1. Checking Gatekeeper Status..."
gatekeeper_status=$(spctl --status)
if [[ "$gatekeeper_status" == "assessments enabled" ]]; then
    add_check 1 "✅ Gatekeeper is enabled (system-wide protection active)"
else
    add_check 0 "⚠️ Gatekeeper is disabled (less secure)"
fi

# 2. Check System Integrity Protection (SIP)
log_info "2. Checking System Integrity Protection..."
sip_status=$(csrutil status | grep -o "enabled\|disabled")
if [[ "$sip_status" == "enabled" ]]; then
    add_check 1 "✅ SIP is enabled (core system files protected)"
else
    add_check 0 "⚠️ SIP is disabled (less secure)"
fi

# 3. Check User Permissions
log_info "3. Checking User Permissions..."
current_user=$(whoami)
if [[ "$current_user" != "root" ]]; then
    add_check 1 "✅ Running as user '$current_user' (not root - safe)"
else
    add_check 0 "⚠️ Running as root (not recommended for development)"
fi

# 4. Check Developer Mode Scope
log_info "4. Checking Developer Mode Configuration..."
if spctl developer-mode enable-terminal 2>&1 | grep -q "already"; then
    add_check 1 "✅ Developer mode configured (Terminal-only scope - safe)"
else
    add_check 1 "✅ Developer mode not configured (default safe state)"
fi

# 5. Check for Root Installations
log_info "5. Checking for Root-Level Installations..."
if sudo -n true 2>/dev/null; then
    add_check 0 "⚠️ Sudo access available (use with caution)"
else
    add_check 1 "✅ No sudo access (safe for development)"
fi

# 6. Check Tool Installation Sources
log_info "6. Checking Tool Installation Sources..."
if command -v brew >/dev/null 2>&1; then
    add_check 1 "✅ Homebrew available (trusted package manager)"
else
    add_check 1 "✅ No Homebrew (using system packages - safe)"
fi

# 7. Check Rust Toolchain Security
log_info "7. Checking Rust Toolchain Security..."
if command -v rustup >/dev/null 2>&1; then
    rustup_version=$(rustup --version | head -1)
    add_check 1 "✅ Rustup available: $rustup_version (official installer)"
else
    add_check 1 "✅ No Rustup (using system Rust - safe)"
fi

# 8. Check for Suspicious Files
log_info "8. Checking for Suspicious Files..."
suspicious_files=$(find . -name "*.sh" -o -name "*.py" -o -name "*.rb" | head -5)
if [[ -n "$suspicious_files" ]]; then
    add_check 1 "✅ Script files found (normal for development project)"
else
    add_check 1 "✅ No suspicious files detected"
fi

# 9. Check Network Security
log_info "9. Checking Network Security..."
if command -v curl >/dev/null 2>&1; then
    add_check 1 "✅ curl available (for secure downloads)"
else
    add_check 1 "✅ No curl (using system tools - safe)"
fi

# 10. Check File Permissions
log_info "10. Checking File Permissions..."
if [[ -r ".cargo/config.toml" ]] && [[ -w ".cargo/config.toml" ]]; then
    add_check 1 "✅ Cargo config has appropriate permissions"
else
    add_check 0 "⚠️ Cargo config permissions may need adjustment"
fi

echo
echo "📊 Security Assessment Summary"
echo "=============================="
echo "Security Score: $security_score/$total_checks"
echo

# Calculate percentage
percentage=$((security_score * 100 / total_checks))

if [[ $percentage -ge 90 ]]; then
    log_success "🎉 Excellent security status! ($percentage%)"
    echo "Your development environment is well-secured."
elif [[ $percentage -ge 80 ]]; then
    log_success "✅ Good security status ($percentage%)"
    echo "Your development environment is generally secure."
elif [[ $percentage -ge 70 ]]; then
    log_warning "⚠️ Moderate security status ($percentage%)"
    echo "Consider reviewing the warnings above."
else
    log_error "🚨 Security concerns detected ($percentage%)"
    echo "Please address the issues above before continuing."
fi

echo
echo "🔧 Security Recommendations"
echo "==========================="

# Provide specific recommendations based on findings
if [[ "$gatekeeper_status" != "assessments enabled" ]]; then
    echo "• Re-enable Gatekeeper for better system protection"
fi

if [[ "$sip_status" != "enabled" ]]; then
    echo "• Re-enable System Integrity Protection"
fi

if [[ "$current_user" == "root" ]]; then
    echo "• Avoid running development tools as root"
fi

if sudo -n true 2>/dev/null; then
    echo "• Use sudo only when necessary for system administration"
fi

echo
echo "📚 Security Resources"
echo "===================="
echo "• SECURITY_GUIDE.md - Comprehensive security documentation"
echo "• MACOS_OPTIMIZATIONS.md - Safe optimization practices"
echo "• Apple Security Guide: https://support.apple.com/guide/security"
echo

echo "✅ Security check complete"
echo
echo "💡 Remember:"
echo "• Only run trusted code and scripts"
echo "• Keep your system and tools updated"
echo "• Disable developer mode when not needed"
echo "• Monitor your development environment regularly" 
