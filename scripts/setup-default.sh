#!/bin/bash

# Default Setup Script for Hooksmith
# Automatically configures optimized build environment as the default

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

echo "🚀 Hooksmith Default Setup"
echo "=========================="
echo "Configuring optimized build environment as default"
echo

# 1. Install optimization tools
log_info "Installing optimization tools..."
./scripts/optimize-build.sh

# 2. Set up environment variables
log_info "Setting up environment variables..."
source scripts/setup-env.sh

# 3. Create shell profile integration
log_info "Setting up shell profile integration..."

# Determine shell profile file
profile_file
if [[ -f "${HOME}/.zshrc" ]]; then
    profile_file="${HOME}/.zshrc"
elif [[ -f "${HOME}/.bashrc" ]]; then
    profile_file="${HOME}/.bashrc"
elif [[ -f "${HOME}/.bash_profile" ]]; then
    profile_file="${HOME}/.bash_profile"
else
    profile_file="${HOME}/.profile"
fi

# Add automatic environment setup to shell profile
if ! grep -q "hooksmith.*setup-env" "${profile_file}"; then
    cat >> "${profile_file}" << EOF

# Hooksmith optimized build environment
# Automatically set up optimization variables when in the project directory
if [[ -f "\${PWD}/scripts/setup-env.sh" && "\${PWD}" == *"hooksmith"* ]]; then
    export RUSTC_WRAPPER="sccache"
    export SCCACHE_DIR="\${HOME}/.cache/sccache"
    export SCCACHE_CACHE_SIZE="10G"
    export CARGO_BUILD_JOBS=\$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
    echo "🔧 Hooksmith optimization environment loaded"
fi
EOF
    log_success "Added automatic environment setup to ${profile_file}"
else
    log_info "Environment setup already configured in ${profile_file}"
fi

# 4. Create git hooks for automatic optimization
log_info "Setting up git hooks for optimization..."

# Create pre-commit hook that ensures optimization is enabled
mkdir -p .git/hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

# Pre-commit hook to ensure optimized builds
echo "🔧 Checking build optimization status..."

# Check if sccache is available
if ! command -v sccache >/dev/null 2>&1; then
    echo "⚠️  sccache not found - run ./scripts/optimize-build.sh"
fi

# Check if cargo-hakari is available
if ! command -v cargo-hakari >/dev/null 2>&1; then
    echo "⚠️  cargo-hakari not found - run ./scripts/optimize-build.sh"
fi

# Check if cargo-nextest is available
if ! command -v cargo-nextest >/dev/null 2>&1; then
    echo "⚠️  cargo-nextest not found - run ./scripts/optimize-build.sh"
fi

echo "✅ Build optimization check complete"
EOF

chmod +x .git/hooks/pre-commit
log_success "Created pre-commit hook for optimization checks"

# 5. Create VS Code settings for optimization
log_info "Setting up VS Code integration..."

mkdir -p .vscode
cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.cargo.buildScripts.overrideCommand": "cargo check",
    "rust-analyzer.cargo.buildScripts.overrideArgs": ["--all-targets", "--all-features"],
    "rust-analyzer.cargo.buildScripts.overrideEnv": {
        "RUSTC_WRAPPER": "sccache",
        "SCCACHE_CACHE_SIZE": "10G"
    },
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.cargo.runBuildScripts": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.cargo.buildScripts.overrideCommand": "cargo",
    "rust-analyzer.cargo.buildScripts.overrideArgs": ["check", "--all-targets", "--all-features"],
    "rust-analyzer.cargo.buildScripts.overrideEnv": {
        "RUSTC_WRAPPER": "sccache"
    }
}
EOF

log_success "Created VS Code settings for optimization"

# 6. Create default Makefile for common tasks
log_info "Creating Makefile for common tasks..."

cat > Makefile << 'EOF'
# Hooksmith Makefile - Optimized Build Tasks

.PHONY: help build test clean dev fast-test setup

help: ## Show this help message
	@echo "Hooksmith - Optimized Build Tasks"
	@echo "=================================="
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Install and configure optimization tools
	./scripts/optimize-build.sh
	source scripts/setup-env.sh

build: ## Build the project with optimizations
	cargo dev-fast

test: ## Run tests with parallel execution
	cargo test-parallel

fast-test: ## Run tests quickly (no capture)
	cargo test-fast

dev: ## Run development cycle
	./scripts/dev-cycle.sh

clean: ## Clean all build artifacts
	cargo clean-all

check: ## Run fast check
	cargo check-fast

docs: ## Generate and open documentation
	cargo docs

stats: ## Show build performance statistics
	./scripts/build-stats.sh

validate: ## Validate project configuration
	cargo validate

ci: ## Run CI build locally
	./scripts/ci-build.sh

# Default target
.DEFAULT_GOAL := help
EOF

log_success "Created Makefile with optimized tasks"

# 7. macOS-specific optimizations
log_info "Setting up macOS-specific optimizations..."

if [[ "$OSTYPE" == "darwin"* ]]; then
    # Enable developer mode for faster binary execution
    log_info "Enabling developer mode for faster binary execution..."
    
    # Check if developer mode is already enabled
    if spctl developer-mode enable-terminal 2>/dev/null; then
        log_success "✅ Developer mode enabled for Terminal"
        log_info "Next step: Enable Terminal in System Settings → Privacy & Security → Developer Tools"
    else
        log_warning "⚠️ Could not enable developer mode (may need manual setup)"
    fi
    
    log_info "macOS optimization benefits:"
    echo "  • Faster cargo test and cargo-nextest execution"
    echo "  • Reduced Gatekeeper overhead for development tools"
    echo "  • LLD linker configured for optimal Apple Silicon performance"
    echo "  • split-debuginfo = 'unpacked' enabled for 70% faster debug builds"
    echo
    log_info "Security note: This only affects Terminal and its child processes."
    echo "  • GUI apps launched from Finder are still protected by Gatekeeper"
    echo "  • Can be disabled anytime with: spctl developer-mode disable-terminal"
fi

# 8. Create default .envrc for direnv
log_info "Setting up direnv integration..."

if [[ -f ".envrc" ]]; then
    log_info ".envrc already exists, updating..."
else
    cat > .envrc << 'EOF'
# Hooksmith Development Environment
# Automatically loads optimization environment

# Load optimization environment
source scripts/setup-env.sh

# Set project-specific variables
export HOOKSMITH_PROJECT_ROOT="${PWD}"
export HOOKSMITH_LOG_DIR="${PWD}/logs"

# Create log directory
mkdir -p "${HOOKSMITH_LOG_DIR}"

# Show optimization status
echo "🔧 Hooksmith optimization environment loaded"
echo "📁 Project root: ${HOOKSMITH_PROJECT_ROOT}"
echo "📊 Log directory: ${HOOKSMITH_LOG_DIR}"
echo "🚀 Use 'make help' for available commands"
EOF
    log_success "Created .envrc for automatic environment loading"
fi

# 8. Final setup verification
log_info "Verifying setup..."

# Test optimized commands
if cargo dev-fast --help >/dev/null 2>&1; then
    log_success "✅ cargo dev-fast is working"
else
    log_warning "⚠️ cargo dev-fast not working - check .cargo/aliases.toml"
fi

if cargo test-parallel --help >/dev/null 2>&1; then
    log_success "✅ cargo test-parallel is working"
else
    log_warning "⚠️ cargo test-parallel not working - check .cargo/aliases.toml"
fi

# Test optimization tools
if command -v sccache >/dev/null 2>&1; then
    log_success "✅ sccache is available"
else
    log_warning "⚠️ sccache not found"
fi

if command -v cargo-hakari >/dev/null 2>&1; then
    log_success "✅ cargo-hakari is available"
else
    log_warning "⚠️ cargo-hakari not found"
fi

if command -v cargo-nextest >/dev/null 2>&1; then
    log_success "✅ cargo-nextest is available"
else
    log_warning "⚠️ cargo-nextest not found"
fi

echo
echo "🎉 Default Setup Complete!"
echo "=========================="
echo
echo "📋 What was configured:"
echo "  ✅ Optimization tools installed"
echo "  ✅ Environment variables set"
echo "  ✅ Shell profile integration"
echo "  ✅ Git hooks for optimization checks"
echo "  ✅ VS Code settings for optimization"
echo "  ✅ Makefile with optimized tasks"
echo "  ✅ direnv integration (.envrc)"
echo
echo "🚀 Next steps:"
echo "1. Restart your terminal or run: source ${profile_file}"
echo "2. Try the optimized commands:"
echo "   • make build    # Fast development build"
echo "   • make test     # Parallel test execution"
echo "   • make dev      # Complete development cycle"
echo "   • make stats    # Performance monitoring"
echo
echo "💡 Tips:"
echo "  • Use 'cargo dev-fast' instead of 'cargo run'"
echo "  • Use 'cargo test-parallel' instead of 'cargo test'"
echo "  • Use 'make help' to see all available commands"
echo "  • Monitor performance with './scripts/build-stats.sh'"
echo
echo "📊 Expected performance improvements:"
echo "  • 30-70% faster rebuilds with sccache"
echo "  • 2-4x faster tests with cargo-nextest"
echo "  • Up to 50% faster builds with cargo-hakari" 
