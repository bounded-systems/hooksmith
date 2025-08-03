#!/bin/bash

# Optimized CI build with distributed caching
# This script is designed for CI environments with build optimization features

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

# Configuration
export CARGO_INCREMENTAL=0
export SCCACHE_CACHE_SIZE="20G"
export RUSTC_WRAPPER="sccache"

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    log_error "Must be run from the project root directory"
    exit 1
fi

echo "🏗️ Starting optimized CI build..."
echo "=================================="

# 1. Setup sccache
log_info "Setting up sccache..."
if command -v sccache >/dev/null 2>&1; then
    sccache --show-stats
    log_success "sccache is available"
else
    log_warning "sccache not found - builds will not be cached"
fi

# 2. Setup cargo-hakari
log_info "Setting up cargo-hakari..."
if command -v cargo-hakari >/dev/null 2>&1; then
    # Generate hakari.toml if it doesn't exist
    if [[ ! -f "hakari.toml" ]]; then
        log_info "Generating hakari.toml..."
        cargo hakari generate
    fi
    
    # Update hakari configuration
    log_info "Updating hakari configuration..."
    cargo hakari update
    log_success "cargo-hakari configured"
else
    log_warning "cargo-hakari not found - workspace optimization disabled"
fi

# 3. Clean build for CI
log_info "Cleaning previous build artifacts..."
cargo clean

# 4. Build with optimizations
log_info "Building with optimizations..."
cargo build --release --all-targets --all-features

# 5. Run tests with cargo-nextest if available
log_info "Running tests..."
if command -v cargo-nextest >/dev/null 2>&1; then
    log_info "Using cargo-nextest for faster test execution..."
    cargo nextest run --all-targets --all-features --test-threads=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
else
    log_info "Using standard cargo test..."
    cargo test --all-targets --all-features
fi

# 6. Generate documentation
log_info "Generating documentation..."
cargo doc --all-features --no-deps

# 7. Show build statistics
log_info "Build statistics:"
if command -v sccache >/dev/null 2>&1; then
    echo "📊 sccache Statistics:"
    sccache --show-stats
    echo
fi

# 8. Show cache usage
echo "💾 Cache Usage:"
if [[ -d "${HOME}/.cache/sccache" ]]; then
    du -sh "${HOME}/.cache/sccache" 2>/dev/null || echo "sccache cache not accessible"
else
    echo "sccache cache directory not found"
fi

if [[ -d ".cargo/hakari" ]]; then
    du -sh ".cargo/hakari" 2>/dev/null || echo "hakari cache not accessible"
else
    echo "hakari cache directory not found"
fi

echo
log_success "✅ CI build complete!"
echo
echo "📊 Build Summary:"
echo "  • Release build completed"
echo "  • All tests passed"
echo "  • Documentation generated"
echo "  • Build artifacts in target/release/"
echo
echo "🚀 Ready for deployment!" 
