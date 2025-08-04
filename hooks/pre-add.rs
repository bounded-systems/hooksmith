//! Pre-Add Hook for Unified Generated File System
//!
//! This hook validates that only properly managed files can be staged for commit:
//! - Files registered in generated-files.jsonc with valid checksums
//! - Files with allowed extensions (.rs, .jsonc)
//! - Files explicitly allowed in manual-files.jsonc

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Manual files registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManualFilesRegistry {
    #[serde(rename = "$schema")]
    schema: String,
    title: String,
    description: String,
    manual: Vec<String>,
    metadata: ManualFilesMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManualFilesMetadata {
    last_updated: String,
    version: String,
    description: String,
}

/// Generated files registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedFileEntry {
    slug: String,
    path: String,
    #[serde(alias = "type")]
    extension: String,
    checksum: String,
    #[serde(default)]
    file_type: String,
}

/// Generated files registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedFilesRegistry {
    #[serde(rename = "$schema")]
    schema: String,
    description: String,
    files: Vec<GeneratedFileEntry>,
    ignore: IgnoreRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IgnoreRules {
    dirs: Vec<String>,
    patterns: Vec<String>,
}

/// File policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilePolicy {
    allowed_extensions: Vec<String>,
    generated_extensions: Vec<String>,
}

/// Pre-add hook validator
struct PreAddValidator {
    project_root: PathBuf,
    allowed_extensions: HashSet<String>,
    generated_extensions: HashSet<String>,
    manual_files: HashSet<String>,
    generated_files: Vec<GeneratedFileEntry>,
}

impl PreAddValidator {
    /// Create a new pre-add validator
    fn new() -> Result<Self> {
        let project_root = env::current_dir()?;
        
        // Load file policy
        let policy_path = project_root.join("config").join("file-policy.jsonc");
        let policy_content = fs::read_to_string(&policy_path)
            .with_context(|| format!("Failed to read file policy: {}", policy_path.display()))?;
        let policy: FilePolicy = serde_json::from_str(&policy_content)
            .with_context(|| "Failed to parse file policy")?;
        
        let allowed_extensions: HashSet<String> = policy.allowed_extensions.into_iter().collect();
        let generated_extensions: HashSet<String> = policy.generated_extensions.into_iter().collect();
        
        // Load manual files registry
        let manual_files_path = project_root.join("config").join("manual-files.jsonc");
        let manual_content = fs::read_to_string(&manual_files_path)
            .with_context(|| format!("Failed to read manual files registry: {}", manual_files_path.display()))?;
        let manual_registry: ManualFilesRegistry = serde_json::from_str(&manual_content)
            .with_context(|| "Failed to parse manual files registry")?;
        let manual_files: HashSet<String> = manual_registry.manual.into_iter().collect();
        
        // Load generated files registry
        let generated_files_path = project_root.join("config").join("generated-files.jsonc");
        let generated_content = fs::read_to_string(&generated_files_path)
            .with_context(|| format!("Failed to read generated files registry: {}", generated_files_path.display()))?;
        let generated_registry: GeneratedFilesRegistry = serde_json::from_str(&generated_content)
            .with_context(|| "Failed to parse generated files registry")?;
        
        Ok(Self {
            project_root,
            allowed_extensions,
            generated_extensions,
            manual_files,
            generated_files: generated_registry.files,
        })
    }
    
    /// Get file extension from path
    fn get_extension(&self, path: &str) -> Option<String> {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
    
    /// Check if file is allowed by extension
    fn is_allowed_extension(&self, path: &str) -> bool {
        if let Some(ext) = self.get_extension(path) {
            self.allowed_extensions.contains(&ext)
        } else {
            false
        }
    }
    
    /// Check if file is in manual files registry
    fn is_manual_file(&self, path: &str) -> bool {
        self.manual_files.contains(path)
    }
    
    /// Find generated file entry by path
    fn find_generated_file(&self, path: &str) -> Option<&GeneratedFileEntry> {
        self.generated_files.iter().find(|entry| entry.path == path)
    }
    
    /// Compute file checksum
    fn compute_checksum(&self, path: &str) -> Result<String> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result)[..8].to_string())
    }
    
    /// Validate a single file
    fn validate_file(&self, path: &str) -> Result<ValidationResult> {
        // Check if file exists
        if !Path::new(path).exists() {
            return Ok(ValidationResult::Error(format!("File does not exist: {}", path)));
        }
        
        // Check if it's a manual file
        if self.is_manual_file(path) {
            return Ok(ValidationResult::Allowed(format!("Manual file: {}", path)));
        }
        
        // Check if it has an allowed extension
        if self.is_allowed_extension(path) {
            return Ok(ValidationResult::Allowed(format!("Allowed extension: {}", path)));
        }
        
        // Check if it's a generated file
        if let Some(entry) = self.find_generated_file(path) {
            let current_checksum = self.compute_checksum(path)?;
            
            if current_checksum == entry.checksum {
                return Ok(ValidationResult::Allowed(format!("Valid generated file: {}", path)));
            } else {
                return Ok(ValidationResult::Error(format!(
                    "Generated file checksum mismatch: {} (expected: {}, got: {})\n\
                     Run 'cargo xtask gen-all-unified' to regenerate files",
                    path, entry.checksum, current_checksum
                )));
            }
        }
        
        // File is not allowed
        Ok(ValidationResult::Error(format!(
            "File not allowed: {}\n\
             This file is not:\n\
             - A manually allowed file (use 'cargo xtask allow-manual --path {0}')\n\
             - A file with allowed extension (.rs, .jsonc)\n\
             - A registered generated file\n\
             \n\
             To add this file to manual files:\n\
             cargo xtask allow-manual --path {0}",
            path
        )))
    }
    
    /// Validate all staged files
    fn validate_staged_files(&self) -> Result<ValidationSummary> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 2 {
            return Err(anyhow::anyhow!("Usage: {} <file1> [file2] ...", args[0]));
        }
        
        let files = &args[1..];
        let mut summary = ValidationSummary::new();
        
        println!("🔍 Pre-add validation for {} file(s)...", files.len());
        
        for file in files {
            match self.validate_file(file)? {
                ValidationResult::Allowed(reason) => {
                    summary.allowed.push((file.to_string(), reason));
                    println!("✅ {}", reason);
                }
                ValidationResult::Error(error) => {
                    summary.errors.push((file.to_string(), error));
                    println!("❌ {}", error);
                }
            }
        }
        
        Ok(summary)
    }
}

/// Validation result for a single file
#[derive(Debug)]
enum ValidationResult {
    Allowed(String),
    Error(String),
}

/// Summary of validation results
#[derive(Debug)]
struct ValidationSummary {
    allowed: Vec<(String, String)>,
    errors: Vec<(String, String)>,
}

impl ValidationSummary {
    fn new() -> Self {
        Self {
            allowed: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
    
    fn print_summary(&self) {
        println!("\n📊 Validation Summary:");
        println!("   ✅ Allowed: {}", self.allowed.len());
        println!("   ❌ Errors: {}", self.errors.len());
        
        if !self.allowed.is_empty() {
            println!("\n✅ Allowed files:");
            for (file, reason) in &self.allowed {
                println!("   - {} ({})", file, reason);
            }
        }
        
        if !self.errors.is_empty() {
            println!("\n❌ Blocked files:");
            for (file, error) in &self.errors {
                println!("   - {}: {}", file, error);
            }
        }
    }
}

fn main() -> Result<()> {
    let validator = PreAddValidator::new()?;
    let summary = validator.validate_staged_files()?;
    
    summary.print_summary();
    
    if !summary.is_success() {
        std::process::exit(1);
    }
    
    Ok(())
} 
