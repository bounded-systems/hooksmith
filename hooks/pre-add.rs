#!/usr/bin/env cargo-eval
//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! toml = "0.8"
//! dirs = "5.0"
//! json_comments = "0.2"
//! sha2 = "0.10"
//! ```
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;
// use json_comments::StripComments; // TODO: Add this dependency if needed
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct PreAddEvent<'a> {
    timestamp: String,
    level: &'a str,
    action: &'a str,
    message: &'a str,
    details: Option<&'a str>,
}

fn log_event(level: &str, action: &str, message: &str, details: Option<&str>) {
    let event = PreAddEvent {
        timestamp: Utc::now().to_rfc3339(),
        level,
        action,
        message,
        details,
    };
    println!("{}", serde_json::to_string(&event).unwrap());
}

fn main() {
    if let Err(e) = run_pre_add_validation() {
        log_event("error", "pre_add_failed", &format!("Pre-add validation failed: {e}"), None);
        std::process::exit(1);
    }
}

fn run_pre_add_validation() -> Result<()> {
    log_event("info", "pre_add_start", "Starting pre-add validation", None);

    // Get staged files
    let staged_files = get_staged_files()?;
    
    if staged_files.is_empty() {
        log_event("info", "pre_add_complete", "No files staged, validation complete", None);
        return Ok(());
    }

    log_event("info", "pre_add_files", &format!("Validating {} staged files", staged_files.len()), None);

    // Load checksum registry
    let registry = load_checksum_registry()?;
    
    let mut violations = Vec::new();
    let mut allowed_files = Vec::new();

    for file_path in staged_files {
        match validate_file(&file_path, &registry) {
            Ok(validation_result) => {
                match validation_result {
                    ValidationResult::Allowed(reason) => {
                        allowed_files.push((file_path.clone(), reason.clone()));
                        log_event("info", "file_allowed", &format!("File allowed: {}", file_path), Some(&reason));
                    }
                    ValidationResult::Blocked(reason) => {
                        violations.push((file_path.clone(), reason.clone()));
                        log_event("error", "file_blocked", &format!("File blocked: {}", file_path), Some(&reason));
                    }
                }
            }
            Err(e) => {
                violations.push((file_path.clone(), format!("Validation error: {}", e)));
                log_event("error", "validation_error", &format!("Validation error for {}: {}", file_path, e), None);
            }
        }
    }

    // Report results
    if violations.is_empty() {
        log_event("info", "pre_add_success", &format!("All {} files allowed", allowed_files.len()), None);
        println!("✅ Pre-add validation passed! All {} files are allowed.", allowed_files.len());
        
        if !allowed_files.is_empty() {
            println!("\n📋 Allowed files:");
            for (file, reason) in allowed_files {
                println!("   ✅ {} ({})", file, reason);
            }
        }
    } else {
        log_event("error", "pre_add_violations", &format!("Found {} violations", violations.len()), None);
        println!("❌ Pre-add validation failed! Found {} violations:", violations.len());
        
        for (file, reason) in &violations {
            println!("   ❌ {} ({})", file, reason);
        }
        
        println!("\n🔧 To fix violations:");
        println!("   - Convert files to .rs or .jsonc for manual maintenance");
        println!("   - Add files to generated-files.jsonc registry with proper checksums");
        println!("   - Run: cargo xtask gen-all --validate");
        
        return Err(anyhow::anyhow!("Pre-add validation failed with {} violations", violations.len()));
    }

    Ok(())
}

fn get_staged_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to run git diff --cached --name-only")?;

    if !output.status.success() {
        anyhow::bail!("git diff --cached --name-only failed");
    }

    let output_str = String::from_utf8(output.stdout)
        .context("Failed to parse git output as UTF-8")?;

    let files: Vec<String> = output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

#[derive(Debug)]
struct ChecksumRegistry {
    files: Vec<RegistryFile>,
}

#[derive(Debug)]
struct RegistryFile {
    path: String,
    checksum: String,
    extension: String,
}

fn load_checksum_registry() -> Result<ChecksumRegistry> {
    let registry_path = Path::new("config/generated-files.jsonc");
    
    if !registry_path.exists() {
        return Ok(ChecksumRegistry { files: Vec::new() });
    }

    let content = fs::read_to_string(registry_path)
        .context("Failed to read generated-files.jsonc")?;

    // Parse JSONC (temporarily disabled StripComments)
    // let stripped = StripComments::new(content.as_bytes());
    // let json_value: serde_json::Value = serde_json::from_reader(stripped)
    //     .context("Failed to parse generated-files.jsonc")?;
    
    // For now, parse as regular JSON (will fail if there are comments)
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse generated-files.jsonc")?;

    let files_array = json_value.get("files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'files' array in generated-files.jsonc"))?;

    let mut files = Vec::new();
    for file_value in files_array {
        if let (Some(path), Some(checksum), Some(extension)) = (
            file_value.get("path").and_then(|p| p.as_str()),
            file_value.get("checksum").and_then(|c| c.as_str()),
            file_value.get("extension").and_then(|e| e.as_str()),
        ) {
            files.push(RegistryFile {
                path: path.to_string(),
                checksum: checksum.to_string(),
                extension: extension.to_string(),
            });
        }
    }

    Ok(ChecksumRegistry { files })
}

#[derive(Debug)]
enum ValidationResult {
    Allowed(String),
    Blocked(String),
}

fn validate_file(file_path: &str, registry: &ChecksumRegistry) -> Result<ValidationResult> {
    let path = Path::new(file_path);
    
    // Get file extension
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if it's a manually allowed extension (.rs or .jsonc)
    if extension == "rs" || extension == "jsonc" {
        return Ok(ValidationResult::Allowed(format!("Manual file (.{})", extension)));
    }

    // Check if file exists in registry
    let registry_entry = registry.files.iter()
        .find(|f| f.path == file_path);

    match registry_entry {
        Some(entry) => {
            // File is in registry, validate checksum
            let actual_checksum = compute_file_checksum(file_path)?;
            
            if actual_checksum == entry.checksum {
                Ok(ValidationResult::Allowed(format!("Generated file with valid checksum (.{})", entry.extension)))
            } else {
                Ok(ValidationResult::Blocked(format!("Generated file with invalid checksum (.{})", entry.extension)))
            }
        }
        None => {
            // File not in registry and not manually allowed
            Ok(ValidationResult::Blocked(format!("File not in registry and not manually allowed (.{})", extension)))
        }
    }
}

fn compute_file_checksum(file_path: &str) -> Result<String> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    
    Ok(format!("{:x}", result))
} 
