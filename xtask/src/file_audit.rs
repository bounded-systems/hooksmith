use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct FileTypeConfig {
    pub allowed_types: Vec<String>,
    pub generated_types: Vec<String>,
    pub markers: HashMap<String, String>,
    pub excluded_paths: Vec<String>,
    pub manual_files: Vec<String>,
    pub generation_commands: HashMap<String, String>,
}

impl FileTypeConfig {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/file_types.yaml");
        if !config_path.exists() {
            bail!("File type configuration not found: config/file_types.yaml");
        }

        let content =
            fs::read_to_string(config_path).context("Failed to read file type configuration")?;

        let config: FileTypeConfig =
            serde_yaml::from_str(&content).context("Failed to parse file type configuration")?;

        Ok(config)
    }

    pub fn is_allowed_type(&self, extension: &str) -> bool {
        self.allowed_types.contains(&extension.to_string())
    }

    pub fn is_generated_type(&self, extension: &str) -> bool {
        self.generated_types.contains(&extension.to_string())
    }

    pub fn get_marker(&self, extension: &str) -> Option<&String> {
        self.markers.get(extension)
    }

    pub fn is_excluded_path(&self, path: &str) -> bool {
        for pattern in &self.excluded_paths {
            if path.contains(pattern) {
                return true;
            }
        }
        false
    }

    pub fn is_manual_file(&self, path: &str) -> bool {
        for pattern in &self.manual_files {
            if path == pattern || path.ends_with(pattern) {
                return true;
            }
        }
        false
    }

    pub fn get_generation_command(&self, extension: &str) -> Option<&String> {
        self.generation_commands.get(extension)
    }
}

#[derive(Debug, Serialize)]
pub struct FileAuditResult {
    pub total_files: usize,
    pub allowed_files: usize,
    pub generated_files: usize,
    pub manual_files: usize,
    pub forbidden_files: Vec<String>,
    pub missing_markers: Vec<String>,
    pub errors: Vec<String>,
}

impl FileAuditResult {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            allowed_files: 0,
            generated_files: 0,
            manual_files: 0,
            forbidden_files: Vec::new(),
            missing_markers: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.forbidden_files.is_empty()
            || !self.missing_markers.is_empty()
            || !self.errors.is_empty()
    }

    pub fn print_summary(&self) {
        println!("📊 File Audit Summary");
        println!("====================");
        println!("Total files checked: {}", self.total_files);
        println!("Allowed files: {}", self.allowed_files);
        println!("Generated files: {}", self.generated_files);
        println!("Manual files: {}", self.manual_files);
        println!("");

        if !self.forbidden_files.is_empty() {
            println!("❌ Forbidden file types:");
            for file in &self.forbidden_files {
                println!("   - {}", file);
            }
            println!("");
        }

        if !self.missing_markers.is_empty() {
            println!("❌ Generated files missing markers:");
            for file in &self.missing_markers {
                println!("   - {}", file);
            }
            println!("");
        }

        if !self.errors.is_empty() {
            println!("❌ Errors:");
            for error in &self.errors {
                println!("   - {}", error);
            }
            println!("");
        }

        if self.has_errors() {
            println!("🔧 To fix issues:");
            println!("1. Remove forbidden files or add them to allowed_types");
            println!("2. Regenerate files with missing markers:");
            println!("   cargo xtask gen-all");
            println!("3. Run validation again:");
            println!("   cargo xtask check-files");
        } else {
            println!("✅ All files are properly configured!");
        }
    }
}

pub fn check_files() -> Result<FileAuditResult> {
    let config = FileTypeConfig::load()?;
    let mut result = FileAuditResult::new();

    println!("🔍 Checking file types and generation markers...");

    // Get all tracked files from git
    let files_output = std::process::Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to get git files")?;

    let files_list =
        String::from_utf8(files_output.stdout).context("Failed to parse git files output")?;

    for file_path in files_list.lines() {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            continue;
        }

        result.total_files += 1;

        // Check if file is excluded
        if config.is_excluded_path(file_path) {
            continue;
        }

        // Check if file is manually maintained
        if config.is_manual_file(file_path) {
            result.manual_files += 1;
            continue;
        }

        // Get file extension
        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if file type is allowed
        if !config.is_allowed_type(&extension) {
            result.forbidden_files.push(file_path.to_string());
            continue;
        }

        result.allowed_files += 1;

        // Check if file type should be generated
        if config.is_generated_type(&extension) {
            result.generated_files += 1;

            // Check for generation marker
            match check_generation_marker(file_path, &config, &extension) {
                Ok(true) => {
                    println!("   ✅ Generated file with marker: {}", file_path);
                }
                Ok(false) => {
                    println!("   ❌ Generated file missing marker: {}", file_path);
                    result.missing_markers.push(file_path.to_string());
                }
                Err(e) => {
                    let error_msg = format!("Error checking {}: {}", file_path, e);
                    println!("   ⚠️  {}", error_msg);
                    result.errors.push(error_msg);
                }
            }
        } else {
            println!("   ✅ Manual file: {}", file_path);
        }
    }

    Ok(result)
}

fn check_generation_marker(
    file_path: &str,
    config: &FileTypeConfig,
    extension: &str,
) -> Result<bool> {
    let content =
        fs::read_to_string(file_path).context(format!("Failed to read file: {}", file_path))?;

    if let Some(expected_marker) = config.get_marker(extension) {
        // Check if the marker is present in the file
        if content.contains(expected_marker) {
            return Ok(true);
        }

        // Also check for alternative markers (backward compatibility)
        let alternative_markers = [
            "auto-generated",
            "@generated",
            "generated by",
            "This file is auto-generated",
        ];

        for marker in &alternative_markers {
            if content.contains(marker) {
                return Ok(true);
            }
        }

        return Ok(false);
    }

    // If no marker is defined for this type, consider it valid
    Ok(true)
}

pub fn validate_generated_files() -> Result<()> {
    let config = FileTypeConfig::load()?;
    let mut errors = Vec::new();

    println!("🔍 Validating generated files...");

    // Get all tracked files from git
    let files_output = std::process::Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to get git files")?;

    let files_list =
        String::from_utf8(files_output.stdout).context("Failed to parse git files output")?;

    for file_path in files_list.lines() {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            continue;
        }

        // Skip excluded and manual files
        if config.is_excluded_path(file_path) || config.is_manual_file(file_path) {
            continue;
        }

        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check generated files
        if config.is_generated_type(&extension) {
            match check_generation_marker(file_path, &config, &extension) {
                Ok(true) => {
                    println!("   ✅ Valid: {}", file_path);
                }
                Ok(false) => {
                    let error = format!("Generated file missing marker: {}", file_path);
                    println!("   ❌ {}", error);
                    errors.push(error);
                }
                Err(e) => {
                    let error = format!("Error checking {}: {}", file_path, e);
                    println!("   ⚠️  {}", error);
                    errors.push(error);
                }
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Generated file validation failed:\n{}", errors.join("\n"));
    }

    println!("✅ All generated files are valid!");
    Ok(())
}

pub fn list_forbidden_files() -> Result<Vec<String>> {
    let config = FileTypeConfig::load()?;
    let mut forbidden_files = Vec::new();

    let files_output = std::process::Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to get git files")?;

    let files_list =
        String::from_utf8(files_output.stdout).context("Failed to parse git files output")?;

    for file_path in files_list.lines() {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            continue;
        }

        if config.is_excluded_path(file_path) || config.is_manual_file(file_path) {
            continue;
        }

        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !config.is_allowed_type(&extension) {
            forbidden_files.push(file_path.to_string());
        }
    }

    Ok(forbidden_files)
}

pub fn list_missing_markers() -> Result<Vec<String>> {
    let config = FileTypeConfig::load()?;
    let mut missing_markers = Vec::new();

    let files_output = std::process::Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to get git files")?;

    let files_list =
        String::from_utf8(files_output.stdout).context("Failed to parse git files output")?;

    for file_path in files_list.lines() {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            continue;
        }

        if config.is_excluded_path(file_path) || config.is_manual_file(file_path) {
            continue;
        }

        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if config.is_generated_type(&extension) {
            match check_generation_marker(file_path, &config, &extension) {
                Ok(false) => {
                    missing_markers.push(file_path.to_string());
                }
                _ => {}
            }
        }
    }

    Ok(missing_markers)
}
