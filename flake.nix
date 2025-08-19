{
  description = "Hooksmith - Comprehensive Git Hook Management Suite";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      craneLib = crane.mkLib pkgs;

      # Clean, hash-stable source for reproducible builds
      src = craneLib.cleanCargoSource (craneLib.path ./.);

      # Common build configuration
      commonArgs = {
        inherit src;
        # These help crane skip rebuilds:
        cargoToml = ./Cargo.toml;
        cargoLock = ./Cargo.lock;

        # Build tools
        nativeBuildInputs = with pkgs; [
          pkg-config
          installShellFiles
        ];

        # Runtime dependencies
        buildInputs = with pkgs;
          [
            openssl
            git # Essential for git2 dependency
            libgit2
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
          ];

        # Environment variables for builds
        # Temporarily disable RUST_SRC_PATH to avoid path issues
        # RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        RUSTFLAGS = "-Dwarnings";
        LIBGIT2_SYS_USE_PKG_CONFIG = "1";
        OPENSSL_NO_VENDOR = "1";

        # Don't check tests during build to avoid issues
        doCheck = false;
      };

      # Build dependencies only (for faster rebuilds)
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Main GBA (Git Blob Analysis) package
      gba = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "gba";
          cargoExtraArgs = "--package gba";

          postInstall = ''
            installShellCompletion --cmd gba \
              --bash <($out/bin/gba completions bash) \
              --fish <($out/bin/gba completions fish) \
              --zsh <($out/bin/gba completions zsh)
          '';
        });

      # Main hooksmith package
      hooksmith = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--bin hooksmith";

          postInstall = ''
            installShellCompletion --cmd hooksmith \
              --bash <($out/bin/hooksmith completions bash) \
              --fish <($out/bin/hooksmith completions fish) \
              --zsh <($out/bin/hooksmith completions zsh)
          '';
        });

      # Analysis tools package
      analysis-tools = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "hooksmith-analysis-tools";
          cargoExtraArgs = "--bin repository_size_auditor --bin rust_blob_analyzer --bin git_delta_analyzer --bin file_churn_analyzer";
        });

      # Git hooks package
      git-hooks = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "hooksmith-git-hooks";
          cargoExtraArgs = "-p git-hooks";
        });

      # Development tools package
      dev-tools = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          pname = "hooksmith-dev-tools";
          cargoExtraArgs = "-p xtask";
        });

      # Comprehensive test suite
      test-suite = craneLib.cargoTest (commonArgs
        // {
          inherit cargoArtifacts;
          cargoTestExtraArgs = "--workspace --all-features";
        });

      # Linting and formatting checks
      lint-checks = craneLib.cargoClippy (commonArgs
        // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
        });

      # Documentation build
      docs = craneLib.cargoDoc (commonArgs
        // {
          inherit cargoArtifacts;
          cargoDocExtraArgs = "--workspace --all-features";
        });

      # Security audit as a derivation
      security-audit = pkgs.stdenv.mkDerivation {
        name = "hooksmith-security-audit";
        inherit src;
        nativeBuildInputs = with pkgs; [cargo-audit cargo-deny];
        buildPhase = ''
          export CARGO_HOME=$(mktemp -d)
          cargo audit
          cargo deny check
        '';
        installPhase = ''
          mkdir -p $out
          echo "Security audit completed successfully" > $out/audit-report.txt
        '';
      };

      # License compliance check
      license-check = pkgs.stdenv.mkDerivation {
        name = "hooksmith-license-check";
        inherit src;
        nativeBuildInputs = with pkgs; [cargo-deny];
        buildPhase = ''
          export CARGO_HOME=$(mktemp -d)
          cargo deny check licenses
        '';
        installPhase = ''
          mkdir -p $out
          echo "License check completed successfully" > $out/license-report.txt
        '';
      };

      # Consistent rust toolchain for dev shell and builds
      rustToolchain = with pkgs; [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
        # Add rust source for RUST_SRC_PATH
        (pkgs.rust.packages.stable.rustPlatform.rustcSrc or pkgs.rustPlatform.rustcSrc)
      ];

      # All development tools in one shell
      devToolsShell = pkgs.mkShell {
        packages = rustToolchain ++ (with pkgs;
          [
            # Build tools
            pkg-config
            just

            # Development and testing
            cargo-watch
            cargo-nextest
            bacon
            cargo-audit
            cargo-deny

            # Analysis and benchmarking
            hyperfine
            tokei

            # Git and VCS tools
            git
            git-lfs

            # System utilities
            jq
            tree
            ripgrep
            fd

            # Documentation
            mdbook

            # Security scanning (optional)
          ]
          ++ (
            if system == "aarch64-darwin" || system == "x86_64-darwin"
            then [
              libiconv
            ]
            else [
              openssl
            ]
          ));

        # Environment setup
        CARGO_PROFILE_DEV_DEBUG = "0"; # Faster builds
        RUSTFLAGS = "-C debuginfo=1 -Dwarnings"; # Minimal debug info + enforce warnings
        HOOKSMITH_DEV_MODE = "1";
        # Temporarily disable RUST_SRC_PATH to avoid path issues
        # RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        LIBGIT2_SYS_USE_PKG_CONFIG = "1";
        OPENSSL_NO_VENDOR = "1";

        shellHook = ''
          # Disable sccache in Nix environment to avoid read-only fs issues
          unset RUSTC_WRAPPER

          echo "🔨 Git Blob Analysis Tools Development Environment"
          echo "=================================================="
          echo ""
          echo "🦀 Fast inner loop (Cargo in Nix shell):"
          echo "  just build          - Build GBA meta CLI"
          echo "  just test           - Run tests"
          echo "  just check          - Quick check"
          echo "  just watch          - Watch and rebuild"
          echo "  just run -- --help  - Show GBA commands"
          echo ""
          echo "📦 Reproducible builds & tasks (Pure Nix):"
          echo "  nix build           - Build GBA meta CLI"
          echo "  nix run -- --help   - Run GBA with help"
          echo "  nix build .#analysis-tools  - Build analysis tools"
          echo "  nix build .#git-hooks       - Build git hooks"
          echo "  nix build .#dev-tools       - Build dev tools"
          echo ""
          echo "🧪 Testing & Quality (Nix derivations):"
          echo "  nix build .#test-suite      - Run complete test suite"
          echo "  nix build .#lint-checks     - Run clippy lints"
          echo "  nix build .#docs            - Build documentation"
          echo ""
          echo "🛡️ Security & Compliance (Nix derivations):"
          echo "  nix build .#security-audit  - Security audit"
          echo "  nix build .#license-check   - License compliance"
          echo ""
          echo "🎯 Just commands (hybrid Cargo+Nix):"
          echo "  just analyze-all    - All analysis tools"
          echo "  just security-audit - Complete security audit"
          echo "  just nix-build      - Build with Nix"
          echo ""
          echo "🔧 Toolchain:"
          echo "  Rust: $(rustc --version)"
          echo "  Nix:  $(nix --version)"
          echo "  Just: $(just --version)"
          echo ""
          echo "Ready for Git blob analysis! 🚀"
        '';
      };
    in {
      # Multiple packages for different use cases
      packages = {
        default = gba;
        inherit gba hooksmith analysis-tools git-hooks dev-tools;
      };

      # Development environment
      devShells.default = devToolsShell;

      # CI/CD checks as Nix derivations
      checks = {
        default = gba;
        gba-build = gba;
        hooksmith-build = hooksmith;
        inherit test-suite lint-checks security-audit license-check;
        docs-build = docs;
      };

      # Apps for easy nix run usage
      apps = {
        default = flake-utils.lib.mkApp {
          drv = gba;
          name = "gba";
        };
        gba = flake-utils.lib.mkApp {
          drv = gba;
          name = "gba";
        };
        hooksmith = flake-utils.lib.mkApp {
          drv = hooksmith;
          name = "hooksmith";
        };
      };
    });
}
