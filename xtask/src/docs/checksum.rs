//! Checksum validation for generated documentation
//!
//! This module provides checksum validation to ensure generated files
//! are properly created and not manually edited.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Checksum data for generated files
#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksumData {
    pub files: HashMap<String, String>,
    pub generated_at: String,
    pub generator_version: String,
}

/// Generate checksum for content
pub fn generate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Save checksum data to file
pub fn save_checksum_data(checksums: &ChecksumData, output_dir: &Path) -> Result<()> {
    let checksum_file = output_dir.join("checksums.json");
    let content =
        serde_json::to_string_pretty(checksums).context("Failed to serialize checksum data")?;

    fs::write(&checksum_file, content).context("Failed to write checksum file")?;

    Ok(())
}

/// Load checksum data from file
pub fn load_checksum_data(output_dir: &Path) -> Result<ChecksumData> {
    let checksum_file = output_dir.join("checksums.json");

    if !checksum_file.exists() {
        return Ok(ChecksumData {
            files: HashMap::new(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
        });
    }

    let content = fs::read_to_string(&checksum_file).context("Failed to read checksum file")?;

    let checksums: ChecksumData =
        serde_json::from_str(&content).context("Failed to parse checksum data")?;

    Ok(checksums)
}

/// Validate generated files against checksums
pub fn validate_generated_files(output_dir: &Path) -> Result<()> {
    let checksums = load_checksum_data(output_dir)?;
    let mut errors = Vec::new();

    // Find all markdown files in the output directory
    let markdown_files = find_markdown_files(output_dir)?;

    for file_path in markdown_files {
        let relative_path = file_path
            .strip_prefix(output_dir)
            .context("Failed to get relative path")?
            .to_string_lossy()
            .to_string();

        if let Some(expected_checksum) = checksums.files.get(&relative_path) {
            let content = fs::read_to_string(&file_path)
                .context(format!("Failed to read file: {:?}", file_path))?;

            let actual_checksum = generate_checksum(&content);

            if &actual_checksum != expected_checksum {
                errors.push(format!(
                    "Checksum mismatch for {}: expected {}, got {}",
                    relative_path, expected_checksum, actual_checksum
                ));
            }
        } else {
            errors.push(format!(
                "No checksum found for generated file: {}",
                relative_path
            ));
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Checksum validation failed:\n{}", errors.join("\n"));
    }

    println!("✅ All generated files validated against checksums");
    Ok(())
}

/// Find all markdown files in a directory recursively
fn find_markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    let entries = fs::read_dir(dir).context(format!("Failed to read directory: {:?}", dir))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "md" {
                    files.push(path);
                }
            }
        } else if path.is_dir() {
            // Skip certain directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && name != "node_modules" && !name.starts_with('.') {
                let sub_files = find_markdown_files(&path)?;
                files.extend(sub_files);
            }
        }
    }

    Ok(files)
}

/// Update checksums for generated files
pub fn update_checksums(output_dir: &Path, generated_files: &[(String, String)]) -> Result<()> {
    let mut checksums = ChecksumData {
        files: HashMap::new(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    for (file_path, content) in generated_files {
        let checksum = generate_checksum(content);
        checksums.files.insert(file_path.clone(), checksum);
    }

    save_checksum_data(&checksums, output_dir)?;

    println!("✅ Checksums updated for {} files", generated_files.len());
    Ok(())
}

/// Generate checksum report
pub fn generate_checksum_report(output_dir: &Path) -> Result<String> {
    let checksums = load_checksum_data(output_dir)?;
    let markdown_files = find_markdown_files(output_dir)?;

    let mut report = String::new();
    report.push_str("# Documentation Checksum Report\n\n");
    report.push_str(&format!("Generated at: {}\n", checksums.generated_at));
    report.push_str(&format!(
        "Generator version: {}\n",
        checksums.generator_version
    ));
    report.push_str(&format!("Total files: {}\n\n", checksums.files.len()));

    report.push_str("## File Checksums\n\n");
    report.push_str("| File | Checksum | Status |\n");
    report.push_str("|------|----------|--------|\n");

    for file_path in markdown_files {
        let relative_path = file_path
            .strip_prefix(output_dir)
            .context("Failed to get relative path")?
            .to_string_lossy()
            .to_string();

        if let Some(checksum) = checksums.files.get(&relative_path) {
            let content = fs::read_to_string(&file_path)
                .context(format!("Failed to read file: {:?}", file_path))?;

            let actual_checksum = generate_checksum(&content);
            let status = if checksum == &actual_checksum {
                "✅ Valid"
            } else {
                "❌ Mismatch"
            };

            report.push_str(&format!(
                "| {} | {} | {} |\n",
                relative_path, checksum, status
            ));
        } else {
            report.push_str(&format!("| {} | N/A | ⚠️ No checksum |\n", relative_path));
        }
    }

    report.push_str("\n## Validation\n\n");
    report.push_str("All generated files should have matching checksums.\n");
    report.push_str("Files with mismatched checksums may have been manually edited.\n");

    Ok(report)
}
