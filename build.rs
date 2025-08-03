use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this script if any of these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=components/");
    println!("cargo:rerun-if-changed=config/");
    println!("cargo:rerun-if-changed=schemas/");

    // Generate version information
    generate_version_info();

    // Generate feature flags
    generate_feature_flags();

    // Generate WIT bindings if needed
    generate_wit_bindings();

    // Generate documentation constants
    generate_doc_constants();

    // Set up conditional compilation
    setup_conditional_compilation();
}

fn generate_version_info() {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let authors = env::var("CARGO_PKG_AUTHORS").unwrap_or_else(|_| "Hooksmith Team".to_string());
    let name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "hooksmith".to_string());
    let description = env::var("CARGO_PKG_DESCRIPTION").unwrap_or_else(|_| {
        "CLI tool for building Rust binaries into Lefthook hooks with WASM components".to_string()
    });

    let version_info = format!(
        r#"
/// Auto-generated version information
pub const VERSION: &str = "{}";
pub const AUTHORS: &str = "{}";
pub const NAME: &str = "{}";
pub const DESCRIPTION: &str = "{}";
pub const BUILD_TIMESTAMP: &str = "{}";
"#,
        version,
        authors,
        name,
        description,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string()
    );

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let version_file = Path::new(&out_dir).join("version.rs");
    fs::write(version_file, version_info).expect("Failed to write version.rs");
    println!("cargo:rustc-env=VERSION_RS={}", out_dir);
}

fn generate_feature_flags() {
    let features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let feature_flags = format!(
        r#"
/// Auto-generated feature flags
pub const TARGET_FEATURES: &str = "{}";
pub const TARGET_ARCH: &str = "{}";
pub const TARGET_OS: &str = "{}";
pub const IS_WASM: bool = {};
pub const IS_WASI: bool = {};
pub const IS_NATIVE: bool = {};
"#,
        features,
        target_arch,
        target_os,
        target_arch == "wasm32",
        target_os == "wasi",
        target_arch != "wasm32" && target_os != "wasi"
    );

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let features_file = Path::new(&out_dir).join("features.rs");
    fs::write(features_file, feature_flags).expect("Failed to write features.rs");
    println!("cargo:rustc-env=FEATURES_RS={}", out_dir);
}

fn generate_wit_bindings() {
    // Check if WIT files exist and generate bindings if needed
    let wit_files = [
        "wit/hooksmith.wit",
        "wit/worktree-runner.wit",
        "components/hook-builder/wit/hook-builder.wit",
        "components/worktree-runner/wit/worktree-runner.wit",
    ];

    for wit_file in &wit_files {
        if Path::new(wit_file).exists() {
            println!("cargo:rerun-if-changed={}", wit_file);
            println!("cargo:warning=WIT file found: {}", wit_file);
        }
    }

    // Generate WIT bindings if wit-bindgen is available
    // Check if wit-bindgen feature is enabled (placeholder for now)
    if false {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let bindings_dir = Path::new(&out_dir).join("wit-bindings");
        fs::create_dir_all(&bindings_dir).expect("Failed to create bindings directory");

        // This would be expanded with actual WIT binding generation
        let bindings_info = r#"
/// Auto-generated WIT bindings placeholder
/// This will be populated by wit-bindgen when the feature is enabled
pub const WIT_BINDINGS_AVAILABLE: bool = true;
"#;

        let bindings_file = bindings_dir.join("mod.rs");
        fs::write(bindings_file, bindings_info).expect("Failed to write WIT bindings");
        println!(
            "cargo:rustc-env=WIT_BINDINGS_DIR={}",
            bindings_dir.display()
        );
    }
}

fn generate_doc_constants() {
    let doc_constants = r#"
/// Auto-generated documentation constants
pub const DOCS_URL: &str = "https://docs.rs/hooksmith";
pub const REPOSITORY_URL: &str = "https://github.com/bdelanghe/hooksmith";
pub const LICENSE: &str = "MIT";
pub const KEYWORDS: &[&str] = &["cli", "hooks", "lefthook", "wasm", "git"];
pub const CATEGORIES: &[&str] = &["command-line-utilities", "development-tools"];

/// Component information
pub const COMPONENTS: &[&str] = &[
    "cli-core",
    "git-filter",
    "hook-builder",
    "worktree-runner",
    "lefthook-rs",
    "xtask",
];
"#;

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let docs_file = Path::new(&out_dir).join("docs.rs");
    fs::write(docs_file, doc_constants).expect("Failed to write docs.rs");
    println!("cargo:rustc-env=DOCS_RS={}", out_dir);
}

fn setup_conditional_compilation() {
    // Set up conditional compilation flags based on features
    // Note: These are placeholders for future feature support
    // if cfg!(feature = "wasm") {
    //     println!("cargo:rustc-cfg=target_arch=\"wasm32\"");
    // }

    // if cfg!(feature = "wasi") {
    //     println!("cargo:rustc-cfg=target_os=\"wasi\"");
    // }

    // if cfg!(feature = "native") {
    //     println!("cargo:rustc-cfg=target_arch=\"native\"");
    // }

    // Set up debug/release flags
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    println!("cargo:rustc-cfg=profile=\"{}\"", profile);

    // Set up feature flags for conditional compilation
    // Note: These are placeholders for future feature support
    // let features = [
    //     "wasm",
    //     "wasi",
    //     "native",
    //     "wit-bindgen",
    //     "docs",
    //     "tests",
    //     "examples",
    // ];

    // for feature in &features {
    //     if cfg!(feature = feature) {
    //         println!("cargo:rustc-cfg=feature=\"{}\"", feature);
    //     }
    // }
}
