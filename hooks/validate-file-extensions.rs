#!/usr/bin/env rust-script
//! Pre-commit hook to validate file extensions against whitelist
//! This script ensures only allowed file extensions are committed
//! Only .rs and .jsonc files are allowed as source files
//! Use --check-shell-scripts to specifically flag shell scripts that need migration

use std::collections::HashSet;
use std::env;
use std::path::Path;

/// Whitelist of allowed file extensions for source files
/// Only .rs and .jsonc files are allowed as manually maintained source files
/// All other file types must be code-generated
const ALLOWED_EXTENSIONS: &[&str] = &[
    // Rust source files (manually maintained)
    "rs",
    // JSON with comments configuration files (manually maintained)
    "jsonc",
];

/// Shell script extensions that should be migrated to Rust
const SHELL_SCRIPT_EXTENSIONS: &[&str] = &[
    "sh", "bash", "zsh", "fish", "ksh", "csh", "tcsh",
];

/// Directories to exclude from validation
const EXCLUDED_DIRS: &[&str] = &[
    "target", "dist", "build", "node_modules", ".git",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file1> [file2] ... [--check-shell-scripts]", args[0]);
        eprintln!("  --check-shell-scripts: Flag shell scripts that need migration to Rust");
        std::process::exit(1);
    }

    let check_shell_scripts = args.iter().any(|arg| arg == "--check-shell-scripts");
    let file_args: Vec<&String> = args.iter().filter(|arg| *arg != "--check-shell-scripts").skip(1).collect();

    if file_args.is_empty() {
        eprintln!("No files specified");
        std::process::exit(1);
    }

    let allowed_extensions: HashSet<&str> = ALLOWED_EXTENSIONS.iter().copied().collect();
    let shell_extensions: HashSet<&str> = SHELL_SCRIPT_EXTENSIONS.iter().copied().collect();
    let excluded_dirs: HashSet<&str> = EXCLUDED_DIRS.iter().copied().collect();
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut shell_scripts = Vec::new();

    // Process each file argument
    for file_path in file_args {
        let path = Path::new(file_path);
        
        // Skip excluded directories
        if path.components().any(|component| {
            if let std::path::Component::Normal(name) = component {
                excluded_dirs.contains(name.to_str().unwrap_or(""))
            } else {
                false
            }
        }) {
            continue;
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if shell_extensions.contains(ext_str) {
                    shell_scripts.push(format!(
                        "Shell script '{}' should be migrated to Rust-based script",
                        file_path
                    ));
                    if !check_shell_scripts {
                        errors.push(format!(
                            "File '{}' has shell script extension '{}'. Shell scripts should be migrated to Rust-based scripts.",
                            file_path, ext_str
                        ));
                    }
                } else if !allowed_extensions.contains(ext_str) {
                    errors.push(format!(
                        "File '{}' has disallowed extension '{}'. Only .rs and .jsonc files are allowed as source files. Other file types must be code-generated.",
                        file_path, ext_str
                    ));
                }
            } else {
                warnings.push(format!(
                    "File '{}' has invalid extension encoding",
                    file_path
                ));
            }
        } else {
            // Files without extensions are allowed (like README, Makefile, etc.)
            println!("✅ {} (no extension - allowed)", file_path);
        }
    }

    // Report results
    if check_shell_scripts && !shell_scripts.is_empty() {
        eprintln!("\n🐚 Shell scripts that need migration to Rust:");
        for script in &shell_scripts {
            eprintln!("   {}", script);
        }
        eprintln!("\nUse 'cargo xtask migrate-shell-scripts' to convert these to Rust-based scripts.");
        std::process::exit(1);
    }

    if !errors.is_empty() {
        eprintln!("\n❌ File extension validation failed:");
        for error in &errors {
            eprintln!("   {}", error);
        }
        eprintln!("\nAllowed source file extensions: {}", ALLOWED_EXTENSIONS.join(", "));
        eprintln!("All other file types must be code-generated using xtask commands.");
        std::process::exit(1);
    }

    if !warnings.is_empty() {
        eprintln!("\n⚠️  Warnings:");
        for warning in &warnings {
            eprintln!("   {}", warning);
        }
    }

    if errors.is_empty() && warnings.is_empty() {
        println!("✅ All file extensions validated successfully");
    }

    Ok(())
} 
