{
  description = "Rust project with Nix + crane for fast development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.mkLib pkgs;

        # Clean, hash-stable source for reproducible builds
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        common = {
          inherit src;
          # These help crane skip rebuilds:
          cargoToml = ./Cargo.toml;
          cargoLock = ./Cargo.lock;
          
          # Native build inputs
          nativeBuildInputs = with pkgs; [ pkg-config ];
          
          # Build inputs
          buildInputs = with pkgs; [
            openssl
            git # Essential for git2 dependency
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
        };

        # Reproducible package (what `nix build` makes)
        myPackage = craneLib.buildPackage common;
      in {
        packages.default = myPackage;

        # Dev shell for fast local work; cargo is incremental here
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain with source for standard library
            cargo rustc rustfmt clippy rust-src
            
            # Development tools
            pkg-config
            just
            cargo-watch
            cargo-nextest
            bacon
            
            # Git and analysis tools
            git
            git-lfs
            jq
            tree
            ripgrep
            fd
            
            # For the git2 dependency
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          
          # Useful defaults for fast development
          CARGO_PROFILE_DEV_DEBUG = "0";  # Faster builds
          RUSTFLAGS = "-C debuginfo=1";   # Minimal debug info
          HOOKSMITH_DEV_MODE = "1";
          
          shellHook = ''
            echo "🔨 Hooksmith Development Environment"
            echo "===================================="
            echo ""
            echo "🦀 Fast inner loop (Cargo):"
            echo "  just build       - Build with Cargo"
            echo "  just test        - Run tests"  
            echo "  just check       - Quick check"
            echo "  just watch       - Watch and rebuild"
            echo ""
            echo "📦 Reproducible builds (Nix):"
            echo "  just nix-build   - Build with Nix"
            echo "  just nix-run     - Run with Nix"
            echo ""  
            echo "🎯 Analysis tools:"
            echo "  just analyze-size   - Repository analysis"
            echo "  just analyze-all    - All analysis tools"
            echo ""
            echo "Rust: $(rustc --version)"
            echo "Ready for development! 🚀"
          '';
        };

        # Optional: Nix "checks" to mirror CI
        checks.default = myPackage;
      });
}
