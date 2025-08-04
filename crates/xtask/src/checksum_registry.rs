use crate::checksum::compute_file_checksum;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Registry entry for a generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFileEntry {
    pub slug: String,
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub checksum: String,
}

/// Complete checksum registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumRegistry {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub title: String,
    pub description: String,
    pub files: Vec<GeneratedFileEntry>,
    pub ignore: IgnoreRules,
}

/// Ignore rules for the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRules {
    pub dirs: Vec<String>,
    pub patterns: Vec<String>,
}

impl ChecksumRegistry {
    /// Load the checksum registry from config/generated-files.jsonc
    pub fn load() -> Result<Self> {
        let registry_path = Path::new("config/generated-files.jsonc");
        let content = fs::read_to_string(registry_path).context(format!(
            "Failed to read registry: {}",
            registry_path.display()
        ))?;

        // Strip comments and parse JSON
        let json_content = strip_jsonc_comments(&content);
        let registry: ChecksumRegistry =
            serde_json::from_str(&json_content).context("Failed to parse checksum registry")?;

        Ok(registry)
    }

    /// Save the checksum registry to config/generated-files.jsonc
    pub fn save(&self) -> Result<()> {
        let registry_path = Path::new("config/generated-files.jsonc");
        let json_content =
            serde_json::to_string_pretty(self).context("Failed to serialize checksum registry")?;

        // Convert to JSONC format with comments
        let jsonc_content = format!(
            "{{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"Hooksmith Generated Files\",\n  \"description\": \"Registry of generated files with stable slugs and checksums for validation and regeneration.\",\n  \n  // All files listed here are assumed to be generated\n  // No redundant generated: true flags needed\n  \"files\": [\n{}\n  ],\n  \"ignore\": {{\n    \"dirs\": [\n{}\n    ],\n    \"patterns\": [\n{}\n    ]\n  }}\n}}\n",
            self.files.iter()
                .map(|f| format!("    {{\n      \"slug\": \"{}\",\n      \"path\": \"{}\",\n      \"type\": \"{}\",\n      \"checksum\": \"{}\"\n    }}", f.slug, f.path, f.file_type, f.checksum))
                .collect::<Vec<_>>()
                .join(",\n"),
            self.ignore.dirs.iter()
                .map(|d| format!("      \"{}\"", d))
                .collect::<Vec<_>>()
                .join(",\n"),
            self.ignore.patterns.iter()
                .map(|p| format!("      \"{}\"", p))
                .collect::<Vec<_>>()
                .join(",\n")
        );

        fs::write(registry_path, jsonc_content).context(format!(
            "Failed to write registry: {}",
            registry_path.display()
        ))?;

        Ok(())
    }

    /// Find a file entry by path
    pub fn find_by_path(&self, path: &str) -> Option<&GeneratedFileEntry> {
        self.files.iter().find(|f| f.path == path)
    }

    /// Find a file entry by slug
    pub fn find_by_slug(&self, slug: &str) -> Option<&GeneratedFileEntry> {
        self.files.iter().find(|f| f.slug == slug)
    }

    /// Update checksum for a file
    pub fn update_checksum(&mut self, path: &str, new_checksum: &str) -> Result<()> {
        if let Some(entry) = self.files.iter_mut().find(|f| f.path == path) {
            entry.checksum = new_checksum.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("File not found in registry: {}", path))
        }
    }

    /// Add a new file entry
    pub fn add_file(&mut self, slug: String, path: String, file_type: String, checksum: String) {
        let entry = GeneratedFileEntry {
            slug,
            path,
            file_type,
            checksum,
        };
        self.files.push(entry);
    }

    /// Remove a file entry
    pub fn remove_file(&mut self, path: &str) -> Result<()> {
        let initial_len = self.files.len();
        self.files.retain(|f| f.path != path);

        if self.files.len() == initial_len {
            Err(anyhow::anyhow!("File not found in registry: {}", path))
        } else {
            Ok(())
        }
    }

    /// Validate all files in the registry
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        for entry in &self.files {
            let file_path = Path::new(&entry.path);

            if !file_path.exists() {
                report.add_error(&entry.path, "File does not exist");
                continue;
            }

            match self.validate_file(entry) {
                Ok(true) => report.add_success(&entry.path),
                Ok(false) => report.add_error(&entry.path, "Checksum mismatch"),
                Err(e) => report.add_error(&entry.path, &format!("Validation error: {}", e)),
            }
        }

        Ok(report)
    }

    /// Validate a single file entry
    pub fn validate_file(&self, entry: &GeneratedFileEntry) -> Result<bool> {
        let file_path = Path::new(&entry.path);

        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", entry.path));
        }

        let content = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {}", entry.path))?;

        // Compute checksum of content (excluding header)
        let computed_checksum = compute_file_checksum(&content);

        Ok(computed_checksum == entry.checksum)
    }

    /// Compute and update checksums for all files
    pub fn update_all_checksums(&mut self) -> Result<UpdateReport> {
        let mut report = UpdateReport::new();

        for entry in &mut self.files {
            let file_path = Path::new(&entry.path);

            if !file_path.exists() {
                report.add_error(&entry.path, "File does not exist");
                continue;
            }

            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let old_checksum = entry.checksum.clone();
                    let new_checksum = compute_file_checksum(&content);
                    entry.checksum = new_checksum.clone();

                    if old_checksum != new_checksum {
                        report.add_updated(&entry.path, &old_checksum, &new_checksum);
                    } else {
                        report.add_unchanged(&entry.path);
                    }
                }
                Err(e) => {
                    report.add_error(&entry.path, &format!("Failed to read file: {}", e));
                }
            }
        }

        Ok(report)
    }
}

/// Validation report for registry validation
#[derive(Debug)]
pub struct ValidationReport {
    pub successful: Vec<String>,
    pub errors: Vec<(String, String)>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_success(&mut self, path: &str) {
        self.successful.push(path.to_string());
    }

    pub fn add_error(&mut self, path: &str, error: &str) {
        self.errors.push((path.to_string(), error.to_string()));
    }

    pub fn is_successful(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn total_files(&self) -> usize {
        self.successful.len() + self.errors.len()
    }
}

/// Update report for checksum updates
#[derive(Debug)]
pub struct UpdateReport {
    pub unchanged: Vec<String>,
    pub updated: Vec<(String, String, String)>, // path, old_checksum, new_checksum
    pub errors: Vec<(String, String)>,
}

impl UpdateReport {
    pub fn new() -> Self {
        Self {
            unchanged: Vec::new(),
            updated: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_unchanged(&mut self, path: &str) {
        self.unchanged.push(path.to_string());
    }

    pub fn add_updated(&mut self, path: &str, old_checksum: &str, new_checksum: &str) {
        self.updated.push((
            path.to_string(),
            old_checksum.to_string(),
            new_checksum.to_string(),
        ));
    }

    pub fn add_error(&mut self, path: &str, error: &str) {
        self.errors.push((path.to_string(), error.to_string()));
    }

    pub fn total_files(&self) -> usize {
        self.unchanged.len() + self.updated.len() + self.errors.len()
    }
}

/// Strip JSONC comments from content
fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut i = 0;

    while i < content.len() {
        let c = content.chars().nth(i).unwrap();

        if escape_next {
            result.push(c);
            escape_next = false;
            i += 1;
            continue;
        }

        if c == '\\' {
            escape_next = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }

        if !in_string {
            // Check for single-line comment
            if c == '/' && i + 1 < content.len() && content.chars().nth(i + 1) == Some('/') {
                // Skip to end of line
                while i < content.len() && content.chars().nth(i) != Some('\n') {
                    i += 1;
                }
                continue;
            }

            // Check for multi-line comment
            if c == '/' && i + 1 < content.len() && content.chars().nth(i + 1) == Some('*') {
                // Skip to end of comment
                i += 2;
                while i + 1 < content.len() {
                    if content.chars().nth(i) == Some('*')
                        && content.chars().nth(i + 1) == Some('/')
                    {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_strip_jsonc_comments() {
        let input = r#"{
  // This is a comment
  "key": "value", /* another comment */
  "array": [
    "item1", // inline comment
    "item2"
  ]
}"#;

        let expected = r#"{
  "key": "value", 
  "array": [
    "item1", 
    "item2"
  ]
}"#;

        let result = strip_jsonc_comments(input);
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        report.add_success("file1.md");
        report.add_error("file2.md", "Checksum mismatch");

        assert_eq!(report.successful.len(), 1);
        assert_eq!(report.errors.len(), 1);
        assert!(!report.is_successful());
        assert_eq!(report.total_files(), 2);
    }

    #[test]
    fn test_update_report() {
        let mut report = UpdateReport::new();
        report.add_unchanged("file1.md");
        report.add_updated("file2.md", "old123", "new456");
        report.add_error("file3.md", "File not found");

        assert_eq!(report.unchanged.len(), 1);
        assert_eq!(report.updated.len(), 1);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.total_files(), 3);
    }
}
