//! Script to generate Cargo.toml files for Rust crates in the repository
//!
//! This script scans the repository for Rust entry points (main.rs, lib.rs)
//! and generates appropriate Cargo.toml files for crates that don't have them.
//! It also attempts to infer dependencies from use statements.

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct CargoToml {
    package: Package,
    dependencies: Option<HashMap<String, Dependency>>,
    dev_dependencies: Option<HashMap<String, Dependency>>,
    features: Option<HashMap<String, Vec<String>>>,
    lib: Option<Lib>,
    bin: Option<Vec<Bin>>,
    workspace: Option<Workspace>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
    authors: Option<Vec<String>>,
    description: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    keywords: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        features: Option<Vec<String>>,
        optional: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Lib {
    name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Bin {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Debug)]
struct CrateInfo {
    path: PathBuf,
    name: String,
    crate_type: CrateType,
    dependencies: HashSet<String>,
    internal_dependencies: HashSet<String>,
}

#[derive(Debug)]
enum CrateType {
    Binary,
    Library,
    Both,
}

#[derive(Parser)]
#[command(
    name = "generate-cargo-toml",
    about = "Generate Cargo.toml files for Rust crates",
    version,
    long_about = "Scans the repository for Rust entry points and generates appropriate Cargo.toml files for crates that don't have them."
)]
struct Cli {
    /// Verbose output
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    verbose: bool,

    /// Dry run (show what would be done)
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔍 Cargo.toml Generator");
        println!("Verbose mode: enabled");
        println!(
            "Dry run mode: {}",
            if cli.dry_run { "enabled" } else { "disabled" }
        );
        println!();
    }
    println!("🔍 Scanning repository for Rust crates...");

    let repo_root = find_repo_root()?;
    let crates = find_rust_crates(&repo_root)?;

    println!("📦 Found {} potential Rust crates:", crates.len());
    for crate_info in &crates {
        println!(
            "  - {} ({:?}) at {}",
            crate_info.name,
            crate_info.crate_type,
            crate_info.path.display()
        );
    }

    let missing_crates = find_missing_cargo_toml(&crates)?;

    if missing_crates.is_empty() {
        println!("✅ All crates already have Cargo.toml files!");
        return Ok(());
    }

    println!(
        "\n📝 Generating Cargo.toml files for {} crates:",
        missing_crates.len()
    );

    for crate_info in missing_crates {
        generate_cargo_toml(crate_info)?;
    }

    println!("\n🎉 Cargo.toml generation complete!");
    println!("💡 Next steps:");
    println!("   1. Review the generated Cargo.toml files");
    println!("   2. Add specific dependency versions");
    println!("   3. Configure features and workspace settings");
    println!("   4. Run 'cargo check' to verify the configuration");

    Ok(())
}

fn find_repo_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let mut path = current_dir.as_path();

    while path.parent().is_some() {
        if path.join("Cargo.toml").exists() || path.join(".git").exists() {
            return Ok(path.to_path_buf());
        }
        path = path.parent().unwrap();
    }

    Err(anyhow::anyhow!(
        "Could not find repository root (no Cargo.toml or .git found)"
    ))
}

fn find_rust_crates(repo_root: &Path) -> Result<Vec<CrateInfo>> {
    let mut crates = Vec::new();
    let mut visited = HashSet::new();

    find_crates_recursive(repo_root, repo_root, &mut crates, &mut visited)?;

    Ok(crates)
}

fn find_crates_recursive(
    repo_root: &Path,
    current_path: &Path,
    crates: &mut Vec<CrateInfo>,
    visited: &mut HashSet<PathBuf>,
) -> Result<()> {
    if visited.contains(&current_path.to_path_buf()) {
        return Ok(());
    }
    visited.insert(current_path.to_path_buf());

    // Skip common directories that shouldn't contain crates
    let skip_dirs = [
        "target",
        ".git",
        "node_modules",
        ".cargo",
        "docs",
        "examples",
        "tests",
        "benches",
    ];

    let dir_name = current_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if skip_dirs.contains(&dir_name) {
        return Ok(());
    }

    // Check if this directory contains Rust entry points
    let has_main = current_path.join("src/main.rs").exists();
    let has_lib = current_path.join("src/lib.rs").exists();
    let has_bin = current_path.join("src/bin").exists()
        && current_path.join("src/bin").read_dir()?.next().is_some();

    if has_main || has_lib || has_bin {
        let crate_type = match (has_main || has_bin, has_lib) {
            (true, true) => CrateType::Both,
            (true, false) => CrateType::Binary,
            (false, true) => CrateType::Library,
            (false, false) => return Ok(()),
        };

        let name = infer_crate_name(current_path, repo_root);
        let dependencies = analyze_dependencies(current_path)?;
        let internal_dependencies = find_internal_dependencies(current_path, repo_root)?;

        crates.push(CrateInfo {
            path: current_path.to_path_buf(),
            name,
            crate_type,
            dependencies,
            internal_dependencies,
        });
    }

    // Recursively search subdirectories
    if let Ok(entries) = current_path.read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_crates_recursive(repo_root, &path, crates, visited)?;
            }
        }
    }

    Ok(())
}

fn infer_crate_name(crate_path: &Path, _repo_root: &Path) -> String {
    // Try to get name from existing Cargo.toml first
    if let Ok(content) = fs::read_to_string(crate_path.join("Cargo.toml")) {
        if let Ok(cargo_toml) = toml::from_str::<CargoToml>(&content) {
            return cargo_toml.package.name;
        }
    }

    // Infer from directory name
    let dir_name = crate_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Convert to valid crate name
    dir_name.replace('-', "_").to_lowercase()
}

fn analyze_dependencies(crate_path: &Path) -> Result<HashSet<String>> {
    let mut dependencies = HashSet::new();
    let src_path = crate_path.join("src");

    if !src_path.exists() {
        return Ok(dependencies);
    }

    let use_regex = Regex::new(r"use\s+([a-zA-Z_][a-zA-Z0-9_]*(?:::[a-zA-Z_][a-zA-Z0-9_]*)*)")?;
    let extern_regex = Regex::new(r"extern\s+crate\s+([a-zA-Z_][a-zA-Z0-9_]*)")?;

    analyze_dependencies_recursive(&src_path, &use_regex, &extern_regex, &mut dependencies)?;

    Ok(dependencies)
}

fn analyze_dependencies_recursive(
    path: &Path,
    use_regex: &Regex,
    extern_regex: &Regex,
    dependencies: &mut HashSet<String>,
) -> Result<()> {
    if let Ok(entries) = path.read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                analyze_dependencies_recursive(&path, use_regex, extern_regex, dependencies)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Find use statements
                    for cap in use_regex.captures_iter(&content) {
                        if let Some(crate_name) = cap.get(1) {
                            let name = crate_name.as_str().split("::").next().unwrap_or("");
                            if !name.is_empty() && !is_std_crate(name) {
                                dependencies.insert(name.to_string());
                            }
                        }
                    }

                    // Find extern crate statements
                    for cap in extern_regex.captures_iter(&content) {
                        if let Some(crate_name) = cap.get(1) {
                            dependencies.insert(crate_name.as_str().to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn is_std_crate(name: &str) -> bool {
    let std_crates = [
        "std",
        "core",
        "alloc",
        "collections",
        "io",
        "fmt",
        "str",
        "string",
        "vec",
        "option",
        "result",
        "boxed",
        "rc",
        "arc",
        "cell",
        "refcell",
        "sync",
        "thread",
        "time",
        "fs",
        "path",
        "env",
        "process",
        "os",
        "mem",
        "ptr",
        "slice",
        "iter",
        "clone",
        "copy",
        "default",
        "debug",
        "display",
        "drop",
        "eq",
        "partial_eq",
        "ord",
        "partial_ord",
        "hash",
        "send",
        "sync_trait",
        "unsafe_cell",
        "phantom_data",
        "marker",
        "pin",
        "unpin",
        "sized",
        "unsize",
        "coerce_unsized",
        "dispatch_from_dyn",
        "receiver",
        "fn",
        "fn_mut",
        "fn_once",
        "termination",
        "try",
        "from",
        "into",
        "as_ref",
        "as_mut",
        "borrow",
        "borrow_mut",
        "to_owned",
        "clone_from",
        "deref",
        "deref_mut",
        "index",
        "index_mut",
        "range",
        "range_from",
        "range_full",
        "range_inclusive",
        "range_to",
        "range_to_inclusive",
    ];

    std_crates.contains(&name)
}

fn find_internal_dependencies(crate_path: &Path, repo_root: &Path) -> Result<HashSet<String>> {
    let mut internal_deps = HashSet::new();
    let src_path = crate_path.join("src");

    if !src_path.exists() {
        return Ok(internal_deps);
    }

    let use_regex = Regex::new(r"use\s+([a-zA-Z_][a-zA-Z0-9_]*(?:::[a-zA-Z_][a-zA-Z0-9_]*)*)")?;

    analyze_dependencies_recursive(&src_path, &use_regex, &Regex::new("")?, &mut internal_deps)?;

    // Filter to only internal dependencies
    let mut filtered_deps = HashSet::new();
    for dep in internal_deps {
        let potential_path = repo_root.join(&dep);
        if potential_path.exists() && potential_path.join("Cargo.toml").exists() {
            filtered_deps.insert(dep);
        }
    }

    Ok(filtered_deps)
}

fn find_missing_cargo_toml(crates: &[CrateInfo]) -> Result<Vec<&CrateInfo>> {
    let mut missing = Vec::new();

    for crate_info in crates {
        let cargo_toml_path = crate_info.path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            missing.push(crate_info);
        }
    }

    Ok(missing)
}

fn generate_cargo_toml(crate_info: &CrateInfo) -> Result<()> {
    println!("  📝 Generating Cargo.toml for {}", crate_info.name);

    let package = Package {
        name: crate_info.name.clone(),
        version: "0.1.0".to_string(),
        edition: "2021".to_string(),
        authors: Some(vec!["Hooksmith Team".to_string()]),
        description: Some(format!("{} component for Hooksmith", crate_info.name)),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/bdelanghe/hooksmith".to_string()),
        documentation: None,
        keywords: Some(vec!["hooksmith".to_string(), "component".to_string()]),
        categories: Some(vec!["development-tools".to_string()]),
    };

    let mut dependencies = HashMap::new();

    // Add common dependencies based on crate type
    match crate_info.crate_type {
        CrateType::Binary => {
            dependencies.insert(
                "clap".to_string(),
                Dependency::Detailed {
                    version: Some("4.0".to_string()),
                    path: None,
                    features: Some(vec!["derive".to_string()]),
                    optional: None,
                },
            );
            dependencies.insert("anyhow".to_string(), Dependency::Version("1.0".to_string()));
            dependencies.insert(
                "tracing".to_string(),
                Dependency::Version("0.1".to_string()),
            );
        }
        CrateType::Library => {
            dependencies.insert(
                "serde".to_string(),
                Dependency::Detailed {
                    version: Some("1.0".to_string()),
                    path: None,
                    features: Some(vec!["derive".to_string()]),
                    optional: None,
                },
            );
            dependencies.insert(
                "thiserror".to_string(),
                Dependency::Version("1.0".to_string()),
            );
        }
        CrateType::Both => {
            dependencies.insert(
                "clap".to_string(),
                Dependency::Detailed {
                    version: Some("4.0".to_string()),
                    path: None,
                    features: Some(vec!["derive".to_string()]),
                    optional: None,
                },
            );
            dependencies.insert(
                "serde".to_string(),
                Dependency::Detailed {
                    version: Some("1.0".to_string()),
                    path: None,
                    features: Some(vec!["derive".to_string()]),
                    optional: None,
                },
            );
            dependencies.insert("anyhow".to_string(), Dependency::Version("1.0".to_string()));
            dependencies.insert(
                "thiserror".to_string(),
                Dependency::Version("1.0".to_string()),
            );
        }
    }

    // Add inferred external dependencies
    for dep in &crate_info.dependencies {
        if !crate_info.internal_dependencies.contains(dep) {
            dependencies.insert(dep.clone(), Dependency::Version("*".to_string()));
        }
    }

    // Add internal dependencies
    for dep in &crate_info.internal_dependencies {
        dependencies.insert(
            dep.clone(),
            Dependency::Detailed {
                version: None,
                path: Some(format!("../{dep}")),
                features: None,
                optional: None,
            },
        );
    }

    let mut cargo_toml = CargoToml {
        package,
        dependencies: Some(dependencies),
        dev_dependencies: None,
        features: None,
        lib: None,
        bin: None,
        workspace: None,
    };

    // Add lib/bin configuration
    match crate_info.crate_type {
        CrateType::Library => {
            cargo_toml.lib = Some(Lib {
                name: Some(crate_info.name.clone()),
                path: Some("src/lib.rs".to_string()),
            });
        }
        CrateType::Binary => {
            cargo_toml.bin = Some(vec![Bin {
                name: crate_info.name.clone(),
                path: "src/main.rs".to_string(),
            }]);
        }
        CrateType::Both => {
            cargo_toml.lib = Some(Lib {
                name: Some(crate_info.name.clone()),
                path: Some("src/lib.rs".to_string()),
            });
            cargo_toml.bin = Some(vec![Bin {
                name: crate_info.name.clone(),
                path: "src/main.rs".to_string(),
            }]);
        }
    }

    let toml_string = toml::to_string_pretty(&cargo_toml)?;
    let cargo_toml_path = crate_info.path.join("Cargo.toml");

    fs::write(cargo_toml_path, toml_string)?;

    println!(
        "    ✅ Generated {}",
        crate_info.path.join("Cargo.toml").display()
    );

    Ok(())
}
