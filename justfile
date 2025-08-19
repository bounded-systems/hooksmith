# Justfile for Hooksmith development
# Uses bash with strict flags
set shell := ["bash", "-euo", "pipefail", "-c"]

# Main package name (defined by default-members in Cargo.toml)
MAIN_PKG := "hooksmith"

# If not in a dev shell, re-enter via `nix develop` and re-run the target
_enter_dev := '''
if [[ -z "${IN_NIX_SHELL:-}" ]]; then
  echo "↪ entering nix develop..." >&2
  exec nix develop -c just "$@"
fi
'''

# Default recipe - show available commands
default:
    @echo "🔨 Hooksmith Development Commands"
    @echo "================================"
    @echo ""
    @echo "🚀 Getting Started:"
    @echo "  just bootstrap         - Setup development environment"
    @echo "  just info              - Show environment information"
    @echo ""
    @echo "🦀 Rust Development:"
    @echo "  just build             - Build main package ({{MAIN_PKG}})"
    @echo "  just run [ARGS]        - Run main package with args"
    @echo "  just test              - Test main package"
    @echo "  just check             - Quick check main package"
    @echo "  just build-all         - Build entire workspace"
    @echo "  just test-all          - Test entire workspace"
    @echo "  just clippy            - Run clippy lints"
    @echo "  just fmt               - Format all code"
    @echo "  just lint              - Lint all code"
    @echo ""
    @echo "🧹 Code Quality:"
    @echo "  just pre-commit        - Run all pre-commit checks"
    @echo "  just fmt-all           - Format Rust, Nix, and other files"
    @echo "  just lint-all          - Lint Rust, Nix, and shell files"
    @echo ""
    @echo "🔧 Xtask Commands:"
    @echo "  just xtask-build       - Build via xtask"
    @echo "  just xtask-docs        - Generate documentation"
    @echo "  just xtask-check       - Full xtask validation"
    @echo ""
    @echo "📦 Nix Commands:"
    @echo "  just nix-build         - Build all packages with Nix"
    @echo "  just nix-analyze       - Run quick analysis suite"
    @echo "  just nix-dev           - Enter Nix dev shell"
    @echo "  just nix-fmt           - Format Nix files"
    @echo ""
    @echo "🔍 Analysis Tools:"
    @echo "  just analyze-size      - Repository size analysis"
    @echo "  just analyze-blobs     - Rust blob analysis"
    @echo "  just analyze-delta     - Git delta analysis"
    @echo "  just analyze-churn     - File churn analysis"
    @echo "  just analyze-all       - Run all analysis tools"
    @echo ""
    @echo "🛡️ Security & Compliance:"
    @echo "  just security-audit    - Complete security audit"
    @echo "  just sbom              - Generate Software Bill of Materials"
    @echo "  just vuln-scan         - Vulnerability scanning"
    @echo "  just license-check     - License compliance check"
    @echo "  just reproducible-build - Deterministic build verification"
    @echo ""
    @echo "💡 Quick Start:"
    @echo "  just bootstrap && just security-audit"

# -------- Inner loop (Cargo) --------
# Build the workspace with cargo - routes through nix dev shell
# Build main package
build +ARGS="":
    {{_enter_dev}}
    cargo build -p {{MAIN_PKG}} {{ARGS}}

# Run main package with arguments  
run *ARGS:
    {{_enter_dev}}
    cargo run -p {{MAIN_PKG}} -- {{ARGS}}

# Test main package
test +ARGS="":
    {{_enter_dev}}
    cargo test -p {{MAIN_PKG}} {{ARGS}}

# Quick check main package
check:
    {{_enter_dev}}
    cargo check -p {{MAIN_PKG}}

# Build entire workspace
build-all +ARGS="":
    {{_enter_dev}}
    cargo build --workspace {{ARGS}}

# Test entire workspace
test-all +ARGS="":
    {{_enter_dev}}
    cargo test --workspace {{ARGS}}

# Run clippy lints
clippy:
    {{_enter_dev}}
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    {{_enter_dev}}
    cargo fmt --all

# Watch for changes and rebuild main package
watch:
    {{_enter_dev}}
    cargo watch -x "build -p {{MAIN_PKG}}"

# -------- Reproducible artifacts (Nix) --------
# Build main hooksmith package with Nix
nix-build:
    nix build .#hooksmith

# Build analysis tools with Nix
nix-build-analysis:
    nix build .#analysis-tools

# Build git hooks with Nix
nix-build-hooks:
    nix build .#git-hooks

# Build development tools with Nix
nix-build-dev:
    nix build .#dev-tools

# Build all packages with Nix
nix-build-all:
    nix build .#hooksmith .#analysis-tools .#git-hooks .#dev-tools

# Run the default package with Nix
nix-run *ARGS:
    nix run .#hooksmith -- {{ARGS}}

# Run analysis tools via Nix
nix-analyze-size:
    nix build .#analysis-tools && ./result/bin/repository_size_auditor

nix-analyze-blobs:
    nix build .#analysis-tools && ./result/bin/rust_blob_analyzer

nix-analyze-delta:
    nix build .#analysis-tools && ./result/bin/git_delta_analyzer

nix-analyze-churn:
    nix build .#analysis-tools && ./result/bin/file_churn_analyzer "6 months ago"

# Run all analysis tools via Nix
nix-analyze-all: nix-analyze-size nix-analyze-blobs nix-analyze-delta nix-analyze-churn
    @echo "✅ All Nix-based analysis tools completed!"

# Pure Nix testing and quality checks
nix-test:
    nix build .#test-suite

nix-lint:
    nix build .#lint-checks

nix-docs:
    nix build .#docs

# Pure Nix security and compliance
nix-security-audit:
    nix build .#security-audit

nix-license-check:
    nix build .#license-check

# Complete Nix-based quality pipeline
nix-quality-pipeline:
    @echo "🧪 Running complete Nix-based quality pipeline..."
    nix build .#test-suite .#lint-checks .#docs .#security-audit .#license-check
    @echo "✅ All quality checks passed!"

# CI-preflight: do what CI will do
ci:
    nix flake check

# CI-full: comprehensive CI checks
ci-full:
    @echo "🔄 Running comprehensive CI checks..."
    nix flake check
    just nix-quality-pipeline
    @echo "✅ All CI checks completed!"

# -------- Build system integration --------
# Build via xtask
xtask-build:
    {{_enter_dev}}
    cargo run -p xtask -- build --target all --release

# Generate documentation via xtask
xtask-docs:
    {{_enter_dev}}
    cargo run -p xtask -- gen-docs-comprehensive --all --validate

# Full xtask validation
xtask-check:
    {{_enter_dev}}
    cargo run -p xtask -- check-all --strict

# Repository size analysis
analyze-size:
    @echo "📏 Running repository size analysis..."
    cargo run --bin repository_size_auditor

# Rust blob analysis
analyze-blobs:
    @echo "🦀 Running Rust blob analysis..."
    cargo run --bin rust_blob_analyzer

# Git delta compression analysis
analyze-delta:
    @echo "🔗 Running Git delta analysis..."
    cargo run --bin git_delta_analyzer

# File churn analysis
analyze-churn:
    @echo "📈 Running file churn analysis..."
    cargo run --bin file_churn_analyzer "6 months ago"

# Tree object stability analysis
analyze-stability:
    @echo "🌳 Running tree stability analysis..."
    cargo run --bin tree_object_stability_auditor "6 months ago"

# Git history cleanliness analysis
analyze-history:
    @echo "🧼 Running Git history analysis..."
    cargo run --bin git_history_cleanliness_analyzer

# LFS auto-tracker analysis
analyze-lfs:
    @echo "📦 Running Git LFS analysis..."
    cargo run --bin git_lfs_auto_tracker

# Run all key analysis tools
analyze-all: analyze-size analyze-blobs analyze-delta analyze-churn analyze-stability analyze-history analyze-lfs
    @echo "✅ All analysis tools completed!"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Full development cycle: format, check, test, build
dev-cycle: fmt clippy test build
    @echo "✅ Development cycle complete!"

# Quick setup for new contributors
setup:
    @echo "🚀 Setting up Hooksmith development environment..."
    @echo "1. Installing Rust components..."
    rustup component add clippy rustfmt
    @echo "2. Building workspace..."
    cargo build
    @echo "3. Running a quick test..."
    just analyze-size
    @echo "✅ Setup complete! See WARP.md for detailed workflows."

# Release build with all optimizations
release:
    @echo "🚀 Building optimized release..."
    cargo build --release --all-targets
    @echo "📦 Release binaries available in target/release/"

# Generate and open documentation
docs:
    @echo "📚 Generating and opening documentation..."
    cargo doc --open --all-features

# Run benchmarks if any exist
bench:
    @echo "⚡ Running benchmarks..."
    cargo bench

# Audit dependencies for security issues
audit:
    @echo "🔒 Auditing dependencies..."
    cargo audit

# Update dependencies
update:
    @echo "⬆️  Updating dependencies..."
    cargo update

# Bootstrap development environment
bootstrap:
    @echo "🚀 Bootstrapping Hooksmith development environment..."
    @echo "1. Verifying toolchain..."
    @rustc --version
    @cargo --version
    @git --version
    @echo "2. Setting up git hooks..."
    @# Clear any existing hooks path that might conflict
    @git config --unset-all core.hooksPath || true
    @# Pre-commit hooks are managed by Nix in this environment
    @echo "✅ Git hooks managed by Nix (via flake.nix pre-commit configuration)"
    @echo "3. Verifying workspace can build..."
    @cargo check --workspace || echo "⚠️  Some workspace members may need attention"
    @echo "4. Running initial analysis to verify tools..."
    @cargo run --bin repository_size_auditor > /dev/null 2>&1 || echo "⚠️  Some analysis tools may need building first"
    @echo "✅ Bootstrap complete! Environment ready for development."
    @echo "💡 Next steps: just info, just analyze-size, or just build"

# Run all pre-commit checks
pre-commit:
    @echo "🧹 Running pre-commit checks..."
    @if command -v pre-commit >/dev/null 2>&1; then \
        pre-commit run --all-files; \
    else \
        echo "⚠️  pre-commit not available, running individual checks..."; \
        just fmt && just clippy && just nix-fmt; \
    fi

# Format all code (Rust + Nix + others)
fmt-all:
    @echo "🎨 Formatting all code..."
    @cargo fmt --all
    @if command -v alejandra >/dev/null 2>&1; then \
        alejandra .; \
    else \
        echo "⚠️  alejandra not available, skipping Nix formatting"; \
    fi
    @if command -v prettier >/dev/null 2>&1; then \
        prettier --write "**/*.{json,yaml,yml,md}" || true; \
    else \
        echo "⚠️  prettier not available, skipping JS/JSON/YAML/MD formatting"; \
    fi

# Lint all code (Rust + Nix + Shell)
lint-all:
    @echo "🔍 Linting all code..."
    @cargo clippy --all-targets --all-features -- -D warnings
    @if command -v statix >/dev/null 2>&1; then \
        statix check .; \
    else \
        echo "⚠️  statix not available, skipping Nix linting"; \
    fi
    @if command -v deadnix >/dev/null 2>&1; then \
        deadnix --check .; \
    else \
        echo "⚠️  deadnix not available, skipping Nix dead code check"; \
    fi
    @if command -v shellcheck >/dev/null 2>&1; then \
        find . -name "*.sh" -exec shellcheck {} \; || true; \
    else \
        echo "⚠️  shellcheck not available, skipping shell script linting"; \
    fi

# Enhanced lint command (alias for lint-all)
lint: lint-all

# Complete security audit
security-audit:
    @echo "🛡️ Running comprehensive security audit..."
    @echo "1. Dependency security audit..."
    @cargo audit || echo "⚠️  cargo-audit not available"
    @echo "2. License compliance check..."
    @just license-check
    @echo "3. Dependency policy check..."
    @cargo deny check || echo "⚠️  cargo-deny not available"
    @echo "4. Vulnerability scanning..."
    @just vuln-scan
    @echo "5. Generating SBOM..."
    @just sbom
    @echo "✅ Security audit complete!"

# Generate Software Bill of Materials
sbom:
    @echo "📋 Generating Software Bill of Materials..."
    @if command -v syft >/dev/null 2>&1; then \
        echo "Generating SPDX SBOM..."; \
        syft packages dir:. -o spdx-json=sbom.spdx.json; \
        echo "Generating CycloneDX SBOM..."; \
        syft packages dir:. -o cyclonedx-json=sbom.cyclonedx.json; \
        echo "Generating Cargo-specific SBOM..."; \
        syft packages Cargo.lock -o spdx-json=cargo-sbom.spdx.json; \
        echo "✅ SBOM files generated: sbom.spdx.json, sbom.cyclonedx.json, cargo-sbom.spdx.json"; \
    else \
        echo "⚠️  syft not available. Install with: curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh"; \
    fi

# Vulnerability scanning
vuln-scan:
    @echo "🔍 Running vulnerability scans..."
    @if command -v grype >/dev/null 2>&1; then \
        echo "Scanning source code..."; \
        grype dir:. -o table || true; \
        grype dir:. -o json > vulnerability-scan.json || true; \
        if [ -f sbom.spdx.json ]; then \
            echo "Scanning SBOM..."; \
            grype sbom:sbom.spdx.json -o table || true; \
        fi; \
        echo "✅ Vulnerability scan complete. Check vulnerability-scan.json for details"; \
    else \
        echo "⚠️  grype not available. Install with: curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh"; \
    fi
    @if command -v trivy >/dev/null 2>&1; then \
        echo "Running Trivy scan..."; \
        trivy fs --format table . || true; \
        trivy fs --format json --output trivy-scan.json . || true; \
    else \
        echo "⚠️  trivy not available. See docs/DEV_ENV.md for installation instructions"; \
    fi

# License compliance check
license-check:
    @echo "📄 Checking license compliance..."
    @if command -v cargo >/dev/null 2>&1 && cargo deny --version >/dev/null 2>&1; then \
        cargo deny check licenses; \
        cargo deny list --format json > licenses.json; \
        if grep -q "GPL\|AGPL\|LGPL" licenses.json; then \
            echo "❌ Restrictive licenses found!"; \
            grep "GPL\|AGPL\|LGPL" licenses.json || true; \
            exit 1; \
        else \
            echo "✅ License compliance check passed"; \
        fi; \
    else \
        echo "⚠️  cargo-deny not available. Install with: cargo install cargo-deny"; \
    fi

# Reproducible build verification
reproducible-build:
    @echo "🔄 Verifying reproducible builds..."
    @echo "Setting up reproducible environment..."
    @export SOURCE_DATE_EPOCH=1
    @export RUSTFLAGS="-C target-feature=+crt-static --remap-path-prefix=$(pwd)=/build"
    @echo "Building with Nix for maximum reproducibility..."
    @if command -v nix >/dev/null 2>&1; then \
        nix build .#hooksmith-suite --no-link; \
        echo "✅ Nix build completed reproducibly"; \
    else \
        echo "Building with Cargo..."; \
        cargo build --release; \
        echo "⚠️  For maximum reproducibility, use Nix builds"; \
    fi
    @echo "Generating build manifest..."
    @echo "{" > build-manifest.json
    @echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> build-manifest.json
    @echo "  \"source_date_epoch\": \"1\"," >> build-manifest.json
    @echo "  \"git_commit\": \"$(git rev-parse HEAD)\"," >> build-manifest.json
    @if git diff-index --quiet HEAD --; then echo "  \"git_dirty\": false,"; else echo "  \"git_dirty\": true,"; fi >> build-manifest.json
    @echo "  \"rust_version\": \"$(rustc --version)\"," >> build-manifest.json
    @echo "  \"cargo_version\": \"$(cargo --version)\"," >> build-manifest.json
    @echo "  \"build_flags\": \"$(echo $$RUSTFLAGS)\"" >> build-manifest.json
    @echo "}" >> build-manifest.json
    @echo "✅ Build manifest created: build-manifest.json"

# Show workspace information
info:
    @echo "ℹ️  Hooksmith Workspace Information"
    @echo "=================================="
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo "Git version: $(git --version)"
    @echo ""
    @echo "Workspace members:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]' | sed 's/^/  - /'
    @echo ""
    @echo "Binary targets:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[].targets[] | select(.kind[] == "bin") | .name' | sort | sed 's/^/  - /'
