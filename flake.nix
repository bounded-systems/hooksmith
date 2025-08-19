{
  description = "Hooksmith: Comprehensive Git repository analysis, validation, and optimization tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, fenix, pre-commit-hooks }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        
        # Use fenix for consistent Rust toolchain
        rustToolchain = fenix.packages.${system}.stable.toolchain;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common build inputs for all targets
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        
        buildInputs = with pkgs; [
          openssl
          git # For git2 dependency - essential for Hooksmith
        ] ++ lib.optionals stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.CoreServices
        ];

        # Filter source to only include relevant files for better caching
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common arguments for all crane builds - hermetic and deterministic
        commonArgs = {
          inherit src nativeBuildInputs buildInputs;
          
          # Deterministic build settings
          CARGO_PROFILE_RELEASE_LTO = "true";
          CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
          CARGO_PROFILE_RELEASE_OPT_LEVEL = "s";
          CARGO_PROFILE_RELEASE_RPATH = "false";
          CARGO_PROFILE_RELEASE_INCREMENTAL = "false";
          
          # Reproducible builds - fixed timestamps and sources
          SOURCE_DATE_EPOCH = "1";
          
          # Hermetic build environment
          CARGO_NET_OFFLINE = "true";
          CARGO_HOME = "$TMPDIR/cargo-home";
          
          # Security and determinism
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          
          # Disable tests for Nix builds since they may require git context
          doCheck = false;
          
          # Ensure clean environment
          HOME = "/homeless-shelter";
          TMPDIR = "/tmp";
        };

        # Build dependencies separately for better caching
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "hooksmith-deps";
        });

        # Main xtask package - the primary build orchestrator
        xtask = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "xtask";
          version = "0.1.0";
          cargoExtraArgs = "-p xtask";
          
          meta = with pkgs.lib; {
            description = "Hooksmith build system and code generation tool";
            homepage = "https://github.com/pushd-web/hooksmith";
            license = licenses.mit;
            maintainers = [ "hooksmith-team" ];
          };
        });

        # Core validation tools package
        coreTools = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "hooksmith-core";
          version = "0.1.0";
          cargoExtraArgs = "-p core -p files -p tree -p snapshot -p inspector -p hooks";
          
          meta = with pkgs.lib; {
            description = "Hooksmith core validation and inspection tools";
            homepage = "https://github.com/pushd-web/hooksmith";
            license = licenses.mit;
            maintainers = [ "hooksmith-team" ];
          };
        });

        # Analysis tools package (the main value of Hooksmith)
        analysisTools = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "hooksmith-analysis";
          version = "0.1.0";
          # Build all the analysis binaries mentioned in Cargo.toml
          cargoExtraArgs = "--bins";
          
          # Ensure we build the analysis scripts specifically
          postInstall = ''
            # Analysis tools are the main binaries we want to expose
            echo "Built analysis tools:"
            ls -la $out/bin/ | grep -E "(analyzer|auditor|tracker|reporter|optimizer|extractor|rewriter)"
          '';
          
          meta = with pkgs.lib; {
            description = "Hooksmith Git repository analysis and optimization tools";
            homepage = "https://github.com/pushd-web/hooksmith";
            license = licenses.mit;
            maintainers = [ "hooksmith-team" ];
          };
        });

        # Combined package with all Hooksmith tools
        hooksmithSuite = pkgs.symlinkJoin {
          name = "hooksmith-suite";
          paths = [ xtask coreTools analysisTools ];
          
          postBuild = ''
            # Create convenience scripts and documentation
            mkdir -p $out/bin $out/share/doc/hooksmith
            
            # Main hooksmith script that delegates to xtask
            cat > $out/bin/hooksmith-suite << 'EOF'
#!/bin/sh
echo "🔨 Hooksmith Suite - Git Repository Analysis & Optimization"
echo "=========================================================="
echo ""
echo "🎯 Key Analysis Tools:"
echo "  repository_size_auditor     - Analyze repo size and health"
echo "  rust_blob_analyzer          - Analyze Rust file blob sizes" 
echo "  git_delta_analyzer          - Find delta compression opportunities"
echo "  packfile_delta_analyzer     - Analyze actual packfile compression"
echo "  git_lfs_auto_tracker        - Auto-detect LFS candidates"
echo "  file_churn_analyzer         - Analyze file change patterns"
echo "  tree_object_stability_auditor - Audit Git tree stability"
echo "  git_history_cleanliness_analyzer - Analyze Git history quality"
echo ""
echo "🔧 Build & Validation Tools:"
echo "  xtask                       - Build system and orchestration"
echo "  tree                        - Tree validation"
echo "  files                       - File validation" 
echo "  inspector                   - Repository inspection"
echo ""
echo "📚 Usage:"
echo "  nix run .#repository_size_auditor  - Quick repo health check"
echo "  nix run .#xtask -- --help          - Build system help"
echo "  nix develop                         - Enter dev environment"
echo ""
echo "Available binaries in PATH:"
ls $out/bin/ | grep -v hooksmith-suite | sort | sed 's/^/  /'
EOF
            chmod +x $out/bin/hooksmith-suite
            
            # Create quick analysis script
            cat > $out/bin/hooksmith-analyze << 'EOF'
#!/bin/sh
echo "🔍 Running Hooksmith Quick Analysis..."
echo ""
echo "1. Repository size audit..."
repository_size_auditor || echo "  ⚠️  repository_size_auditor not found"
echo ""
echo "2. Rust blob analysis..."
rust_blob_analyzer || echo "  ⚠️  rust_blob_analyzer not found"
echo ""
echo "3. Git delta opportunities..."
git_delta_analyzer || echo "  ⚠️  git_delta_analyzer not found"
echo ""
echo "✅ Quick analysis complete!"
EOF
            chmod +x $out/bin/hooksmith-analyze
          '';
        };

        # Pre-commit hooks configuration
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # Nix formatting and linting
            alejandra.enable = true;          # Nix formatter
            statix.enable = true;             # Nix linter
            deadnix.enable = true;            # Dead Nix code detector
            
            # Rust formatting and linting
            rustfmt.enable = true;            # Rust formatter
            clippy = {
              enable = true;
              description = "Lint Rust code.";
              entry = "${rustToolchain}/bin/cargo-clippy clippy --all-targets --all-features -- -D warnings";
              language = "system";
              files = "\\.(rs)$";
              pass_filenames = false;
            };
            
            # Shell script linting
            shellcheck.enable = true;         # Shell script linter
            shfmt.enable = true;              # Shell script formatter
            
            # General formatting
            prettier = {
              enable = true;
              description = "Format with prettier";
              entry = "${pkgs.nodePackages.prettier}/bin/prettier --write";
              language = "system";
              files = "\\.(js|ts|jsx|tsx|json|yaml|yml|md)$";
            };
            
            # Trailing whitespace
            trailing-whitespace = {
              enable = true;
              description = "Remove trailing whitespace";
              entry = "${pkgs.python3Packages.pre-commit-hooks}/bin/trailing-whitespace-fixer";
              language = "system";
              types = [ "text" ];
            };
            
            # End of file fixer
            end-of-file-fixer = {
              enable = true;
              description = "Ensure files end with newline";
              entry = "${pkgs.python3Packages.pre-commit-hooks}/bin/end-of-file-fixer";
              language = "system";
              types = [ "text" ];
            };
          };
        };

      in {
        packages = {
          default = hooksmithSuite;
          hooksmith-suite = hooksmithSuite;
          xtask = xtask;
          core-tools = coreTools;
          analysis-tools = analysisTools;
          
          # Individual analysis tools for convenience
          repository_size_auditor = analysisTools;
          rust_blob_analyzer = analysisTools;
          git_delta_analyzer = analysisTools;
          git_lfs_auto_tracker = analysisTools;
          file_churn_analyzer = analysisTools;
          tree_object_stability_auditor = analysisTools;
          git_history_cleanliness_analyzer = analysisTools;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = hooksmithSuite;
            exePath = "/bin/hooksmith-suite";
          };
          
          hooksmith-suite = flake-utils.lib.mkApp {
            drv = hooksmithSuite;
            exePath = "/bin/hooksmith-suite";
          };
          
          analyze = flake-utils.lib.mkApp {
            drv = hooksmithSuite;
            exePath = "/bin/hooksmith-analyze";
          };
          
          xtask = flake-utils.lib.mkApp {
            drv = xtask;
            exePath = "/bin/xtask";
          };
          
          # Key analysis tools as apps
          repository_size_auditor = flake-utils.lib.mkApp {
            drv = analysisTools;
            exePath = "/bin/repository_size_auditor";
          };
          
          rust_blob_analyzer = flake-utils.lib.mkApp {
            drv = analysisTools;
            exePath = "/bin/rust_blob_analyzer";
          };
          
          git_delta_analyzer = flake-utils.lib.mkApp {
            drv = analysisTools;
            exePath = "/bin/git_delta_analyzer";
          };
          
          git_lfs_auto_tracker = flake-utils.lib.mkApp {
            drv = analysisTools;
            exePath = "/bin/git_lfs_auto_tracker";
          };
          
          file_churn_analyzer = flake-utils.lib.mkApp {
            drv = analysisTools;
            exePath = "/bin/file_churn_analyzer";
          };
        };

        devShells.default = craneLib.devShell {
          # Import pre-commit hooks into the shell
          inputsFrom = [ pre-commit-check ];
          
          # Additional dev tools beyond what's needed for building
          packages = with pkgs; [
            # Rust development tools
            rustToolchain
            cargo-nextest
            cargo-watch
            cargo-edit
            cargo-audit
            cargo-deny
            bacon
            
            # Git and analysis tools
            git
            git-lfs
            jq
            tree
            ripgrep
            fd
            
            # Build and development tools
            just
            gnumake
            
            # Pre-commit and formatting tools
            pre-commit
            alejandra      # Nix formatter
            statix         # Nix linter
            deadnix        # Dead Nix code detector
            nil            # Nix language server
            shellcheck     # Shell script linter
            shfmt          # Shell formatter
            nodePackages.prettier  # JS/JSON/YAML/MD formatter
            
            # WASM development (since Hooksmith has WASM components)
            wasmtime
            
            # Documentation tools
            mdbook
            
            # Additional tools for Git analysis
            git-filter-repo # For repository extraction tools
          ];

          # Environment variables for development
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          HOOKSMITH_DEV_MODE = "1";
          
          # Combined shell hook: pre-commit setup + welcome message
          shellHook = ''
            # Pre-commit hooks setup
            ${pre-commit-check.shellHook}
            
            # Hooksmith welcome message
            echo "🔨 Hooksmith Development Environment"
            echo "===================================="
            echo ""
            echo "🦀 Rust workspace tools:"
            echo "  cargo build              - Build entire workspace"
            echo "  cargo test               - Run workspace tests"
            echo "  cargo run -p xtask -- build - Build via xtask"
            echo "  cargo run -p xtask -- --help - Xtask help"
            echo ""
            echo "🔍 Analysis tools (Nix builds):"
            echo "  nix build .#analysis-tools  - Build all analysis tools"
            echo "  nix run .#analyze           - Quick analysis suite"
            echo "  nix run .#repository_size_auditor - Repo size audit"
            echo "  nix run .#rust_blob_analyzer - Rust blob analysis"
            echo "  nix run .#git_delta_analyzer - Delta compression analysis"
            echo ""
            echo "🎯 Key binaries to test:"
            echo "  repository_size_auditor     - Start here for repo health"
            echo "  rust_blob_analyzer          - Rust-specific analysis"
            echo "  git_lfs_auto_tracker        - LFS optimization"
            echo "  file_churn_analyzer         - Change pattern analysis"
            echo "  tree_object_stability_auditor - Git tree stability"
            echo ""
            echo "🧹 Code quality:"
            echo "  pre-commit run --all-files  - Run all formatting/linting"
            echo "  just bootstrap              - Setup development environment"
            echo "  just fmt                    - Format all code"
            echo "  just lint                   - Lint all code"
            echo ""
            echo "📚 Documentation:"
            echo "  See WARP.md for comprehensive development guide"
            echo "  See README.md for tool descriptions and examples"
            echo ""
            echo "🔧 Environment:"
            echo "  Rust: $(rustc --version)"
            echo "  Git: $(git --version)"
            echo "  wasmtime: $(wasmtime --version 2>/dev/null || echo 'not available')"
            echo ""
            echo "💡 Quick start:"
            echo "  just bootstrap  # Initial setup"
            echo "  cargo run --bin repository_size_auditor"
            echo "  cargo run -p xtask -- gen-docs"
          '';
        };

        # Pre-commit checks (for CI and nix flake check)
        checks = {
          pre-commit-check = pre-commit-check;
        };

        # Formatting
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
