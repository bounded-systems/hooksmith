use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct FilePolicy {
    pub allowed_extensions: Vec<String>,
    pub generated_extensions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub exempt_files: Vec<String>,
    pub generated_markers: HashMap<String, String>,
    pub generation_commands: HashMap<String, String>,
}

impl FilePolicy {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/file-policy.jsonc");
        if !config_path.exists() {
            bail!("File policy configuration not found: config/file-policy.jsonc");
        }

        let content = fs::read_to_string(config_path)
            .context("Failed to read file policy configuration")?;

        // Parse JSONC (JSON with comments) by stripping comments
        let json_content = strip_jsonc_comments(&content);
        
        let policy: FilePolicy = serde_json::from_str(&json_content)
            .context("Failed to parse file policy configuration")?;

        Ok(policy)
    }

    pub fn is_allowed_extension(&self, extension: &str) -> bool {
        self.allowed_extensions.contains(&extension.to_string())
    }

    pub fn is_generated_extension(&self, extension: &str) -> bool {
        self.generated_extensions.contains(&extension.to_string())
    }

    pub fn should_ignore_path(&self, path: &str) -> bool {
        for pattern in &self.ignore_paths {
            if path.contains(pattern) || path.ends_with(pattern) {
                return true;
            }
        }
        false
    }

    pub fn is_exempt_file(&self, path: &str) -> bool {
        for pattern in &self.exempt_files {
            if path == pattern || path.ends_with(pattern) {
                return true;
            }
        }
        false
    }

    pub fn get_generated_marker(&self, extension: &str) -> Option<&String> {
        self.generated_markers.get(extension)
    }
}

#[derive(Debug, Serialize)]
pub struct StrictFileValidationResult {
    pub total_files: usize,
    pub allowed_files: usize,
    pub generated_files: usize,
    pub ignored_files: usize,
    pub exempt_files: usize,
    pub violations: Vec<FileViolation>,
    pub errors: Vec<String>,
}

impl StrictFileValidationResult {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            allowed_files: 0,
            generated_files: 0,
            ignored_files: 0,
            exempt_files: 0,
            violations: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty() || !self.errors.is_empty()
    }

    pub fn print_summary(&self) {
        println!("📊 Strict File Extension Policy Validation Summary");
        println!("Total files checked: {}", self.total_files);
        println!("✅ Allowed files (.rs, .jsonc): {}", self.allowed_files);
        println!("🔧 Generated files: {}", self.generated_files);
        println!("🚫 Ignored files: {}", self.ignored_files);
        println!("🛡️  Exempt files: {}", self.exempt_files);
        println!();

        if self.has_violations() {
            println!("❌ Policy violations found:");
            for violation in &self.violations {
                match violation {
                    FileViolation::DisallowedExtension { file, extension } => {
                        println!("   ❌ Disallowed extension '{}' in: {}", extension, file);
                    }
                    FileViolation::MissingGeneratedHeader { file, extension } => {
                        println!("   ❌ Missing generated header in: {} (extension: {})", file, extension);
                    }
                }
            }

            if !self.errors.is_empty() {
                println!("   ❌ Errors:");
                for error in &self.errors {
                    println!("      - {}", error);
                }
            }

            println!();
            println!("🔧 To fix violations:");
            println!("   - Convert files to .rs or .jsonc for manual maintenance");
            println!("   - Add generated headers to files that should be code-generated");
            println!("   - Run: cargo xtask gen-all --validate");
        } else {
            println!("✅ All files comply with the strict extension policy!");
        }
    }
}

#[derive(Debug, Serialize)]
pub enum FileViolation {
    DisallowedExtension { file: String, extension: String },
    MissingGeneratedHeader { file: String, extension: String },
}

pub fn validate_files() -> Result<StrictFileValidationResult> {
    let policy = FilePolicy::load()?;
    let mut result = StrictFileValidationResult::new();

    println!("🔍 Validating files against strict extension policy...");

    // Walk the repository
    for entry in WalkDir::new(".")
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories
        if path.is_dir() {
            continue;
        }

        let path_str = path.to_string_lossy();
        result.total_files += 1;

        // Check if path should be ignored
        if policy.should_ignore_path(&path_str) {
            result.ignored_files += 1;
            continue;
        }

        // Check if file is exempt
        if policy.is_exempt_file(&path_str) {
            result.exempt_files += 1;
            continue;
        }

        // Get file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Check if extension is allowed
        if policy.is_allowed_extension(extension) {
            result.allowed_files += 1;
            continue;
        }

        // Check if extension is generated
        if policy.is_generated_extension(extension) {
            // Verify it has the generated header
            match check_generated_header(path, &policy, extension) {
                Ok(true) => {
                    result.generated_files += 1;
                }
                Ok(false) => {
                    result.violations.push(FileViolation::MissingGeneratedHeader {
                        file: path_str.to_string(),
                        extension: extension.to_string(),
                    });
                }
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to check generated header for {}: {}",
                        path_str, e
                    ));
                }
            }
            continue;
        }

        // Extension is not allowed
        result.violations.push(FileViolation::DisallowedExtension {
            file: path_str.to_string(),
            extension: extension.to_string(),
        });
    }

    Ok(result)
}

fn check_generated_header(path: &Path, policy: &FilePolicy, extension: &str) -> Result<bool> {
    let marker = match policy.get_generated_marker(extension) {
        Some(marker) => marker,
        None => return Ok(false), // No marker defined for this extension
    };

    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    Ok(content.contains(marker))
}

fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut lines = content.lines();
    let mut in_string = false;
    let mut escape_next = false;

    for line in lines {
        let mut stripped_line = String::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if escape_next {
                stripped_line.push(ch);
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                stripped_line.push(ch);
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                stripped_line.push(ch);
                continue;
            }

            if !in_string && ch == '/' && chars.peek() == Some(&'/') {
                // Found a comment, skip the rest of the line
                break;
            }

            stripped_line.push(ch);
        }

        // Only add non-empty lines
        if !stripped_line.trim().is_empty() {
            result.push_str(&stripped_line);
            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_comments() {
        let input = r#"{
  // This is a comment
  "key": "value", // Another comment
  "nested": {
    "key": "value" // Comment in nested object
  }
}"#;

        let expected = r#"{
  "key": "value",
  "nested": {
    "key": "value"
  }
}"#;

        assert_eq!(strip_jsonc_comments(input).trim(), expected.trim());
    }

    #[test]
    fn test_strip_jsonc_comments_with_strings() {
        let input = r#"{
  "key": "value // not a comment",
  "url": "https://example.com // not a comment either"
}"#;

        let expected = r#"{
  "key": "value // not a comment",
  "url": "https://example.com // not a comment either"
}"#;

        assert_eq!(strip_jsonc_comments(input).trim(), expected.trim());
    }
} 
