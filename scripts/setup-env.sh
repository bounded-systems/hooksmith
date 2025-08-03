#!/bin/bash

# Setup development environment with all optimizations
# This script sets environment variables for optimized Rust builds

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

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    log_error "Must be run from the project root directory"
    exit 1
fi

echo "🔧 Setting up development environment..."
echo "========================================"

# Set optimization environment variables
export RUSTC_WRAPPER="sccache"
export SCCACHE_DIR="${HOME}/.cache/sccache"
export SCCACHE_CACHE_SIZE="10G"
export SCCACHE_LOG="sccache.log"

# Enable parallel compilation for nightly
export RUSTFLAGS_NIGHTLY="-Z threads=8"

# Set number of parallel jobs
export CARGO_BUILD_JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

# Create cache directories if they don't exist
mkdir -p "${SCCACHE_DIR}"

echo "🔧 Environment variables set for optimized builds:"
echo "  RUSTC_WRAPPER: ${RUSTC_WRAPPER}"
echo "  SCCACHE_DIR: ${SCCACHE_DIR}"
echo "  SCCACHE_CACHE_SIZE: ${SCCACHE_CACHE_SIZE}"
echo "  CARGO_BUILD_JOBS: ${CARGO_BUILD_JOBS}"
echo "  RUSTFLAGS_NIGHTLY: ${RUSTFLAGS_NIGHTLY}"
echo

# Verify sccache is working
if command -v sccache >/dev/null 2>&1; then
    log_success "✅ sccache is available and configured"
    sccache --show-stats >/dev/null 2>&1 && log_success "✅ sccache is working"
else
    log_warning "⚠️ sccache not found - run ./scripts/optimize-build.sh to install"
fi

# Verify cargo-hakari is working
if command -v cargo-hakari >/dev/null 2>&1; then
    log_success "✅ cargo-hakari is available"
else
    log_warning "⚠️ cargo-hakari not found - run ./scripts/optimize-build.sh to install"
fi

# Verify cargo-nextest is working
if command -v cargo-nextest >/dev/null 2>&1; then
    log_success "✅ cargo-nextest is available"
else
    log_warning "⚠️ cargo-nextest not found - run ./scripts/optimize-build.sh to install"
fi

# Check cargo configuration
if [[ -f ".cargo/config.toml" ]]; then
    log_success "✅ .cargo/config.toml found"
else
    log_warning "⚠️ .cargo/config.toml missing - run ./scripts/optimize-build.sh"
fi

if [[ -f ".cargo/aliases.toml" ]]; then
    log_success "✅ .cargo/aliases.toml found"
else
    log_warning "⚠️ .cargo/aliases.toml missing - run ./scripts/optimize-build.sh"
fi

echo
log_success "🚀 Environment ready for optimized builds!"
echo
echo "💡 Next steps:"
echo "  • Use 'cargo dev-fast' for fast development builds"
echo "  • Use 'cargo test-parallel' for parallel testing"
echo "  • Use './scripts/dev-cycle.sh' for complete development workflow"
echo "  • Monitor performance with './scripts/build-stats.sh'"
echo
echo "📊 Expected performance improvements:"
echo "  • 30-70% faster rebuilds with sccache"
echo "  • 2-4x faster tests with cargo-nextest"
echo "  • Up to 50% faster builds with cargo-hakari"
echo "  • 20-30% faster linking with LLD/zld/mold"
echo "  • Up to 50% faster compilation with parallel frontend (nightly)" 
