use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct FilePolicy {
    #[serde(rename = "allowedExtensions")]
    pub allowed_extensions: Vec<String>,
    #[serde(rename = "generatedExtensions")]
    pub generated_extensions: Vec<String>,
    #[serde(rename = "useGitignore")]
    pub use_gitignore: Option<bool>,
    #[serde(rename = "generatedMarkers")]
    pub generated_markers: HashMap<String, CommentSyntax>,
    #[serde(rename = "generationCommands")]
    pub generation_commands: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedFilesConfig {
    pub files: Vec<GeneratedFile>,
    pub ignore: IgnoreRules,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedFile {
    pub slug: String,
    pub path: String,
    pub extension: String,
}

#[derive(Debug, Deserialize)]
pub struct IgnoreRules {
    pub dirs: Vec<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommentSyntax {
    pub prefix: String,
    pub suffix: String,
}

impl FilePolicy {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/file-policy.jsonc");
        if !config_path.exists() {
            bail!("File policy configuration not found: config/file-policy.jsonc");
        }

        let content =
            fs::read_to_string(config_path).context("Failed to read file policy configuration")?;

        // Use the main JSONC module's parsing
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
        // Use .gitignore if enabled
        if self.use_gitignore.unwrap_or(false) {
            // TODO: Implement .gitignore parsing
            // For now, use basic patterns
            let gitignore_patterns = [
                "target/",
                "dist/",
                "node_modules/",
                "*.lock",
                "*.jsonl",
                ".git/",
                "logs/",
                "status-trends/",
                ".cargo/hakari/",
                ".hooks/",
                ".trunk/",
                ".cargo/",
                "Cargo.lock",
            ];

            for pattern in &gitignore_patterns {
                if path.contains(pattern) || path.ends_with(pattern) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_exempt_file(&self, _path: &str) -> bool {
        // No exemptions allowed - strict policy only
        // Only .rs and .jsonc files can exist without headers
        false
    }

    pub fn get_generated_marker(&self, extension: &str) -> Option<&CommentSyntax> {
        self.generated_markers.get(extension)
    }
}

impl GeneratedFilesConfig {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/generated-files.jsonc");
        let content =
            fs::read_to_string(config_path).context("Failed to read generated-files.jsonc")?;

        let stripped_content = strip_jsonc_comments(&content);
        let config: GeneratedFilesConfig = serde_json::from_str(&stripped_content)
            .context("Failed to parse generated-files.jsonc")?;

        Ok(config)
    }

    pub fn should_ignore_path(&self, path: &str) -> bool {
        // Check directory patterns
        for dir_pattern in &self.ignore.dirs {
            if path.contains(dir_pattern) {
                return true;
            }
        }

        // Check file patterns
        for file_pattern in &self.ignore.patterns {
            if let Some(suffix) = file_pattern.strip_prefix('*') {
                // Remove leading *
                if path.ends_with(suffix) {
                    return true;
                }
            } else if path.ends_with(file_pattern) {
                return true;
            }
        }

        false
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
    pub allowed_directories: Vec<String>,
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
            allowed_directories: vec![
                "src/".to_string(),
                "components/".to_string(),
                "xtask/".to_string(),
                "config/".to_string(),
                "schemas/".to_string(),
                "docs/".to_string(),
                "examples/".to_string(),
                "tests/".to_string(),
                "scripts/".to_string(),
                "hooks/".to_string(),
                "wit/".to_string(),
                "completions/".to_string(),
                "diagrams/".to_string(),
                "generated-sources/".to_string(),
                "status-trends/".to_string(),
                "logs/".to_string(),
                ".github/".to_string(),
                ".hooksmith/".to_string(),
                "test-enhanced-gen-files/".to_string(),
            ],
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
        println!();

        // Show allowed directories
        println!("📁 Allowed directories:");
        for dir in &self.allowed_directories {
            println!("   📂 {}", dir);
        }
        println!();

        if self.has_violations() {
            println!("❌ Policy violations found:");
            for violation in &self.violations {
                match violation {
                    FileViolation::DisallowedExtension { file, extension, suggestion } => {
                        println!("   ❌ Disallowed extension '{extension}' in: {file}");
                        if let Some(suggestion) = suggestion {
                            println!("      💡 Suggestion: {}", suggestion);
                        }
                    }
                    FileViolation::MissingGeneratedHeader { file, extension } => {
                        println!(
                            "   ❌ Missing generated header in: {file} (extension: {extension})"
                        );
                    }
                }
            }

            if !self.errors.is_empty() {
                println!("   ❌ Errors:");
                for error in &self.errors {
                    println!("      - {error}");
                }
            }

            println!();
            println!("🔧 To fix violations:");
            println!("   - Convert files to .rs or .jsonc for manual maintenance");
            println!("   - Add generated headers to files that should be code-generated");
            println!("   - Run: cargo xtask gen-all --validate");
            println!("   - For .yaml files: rename to .yml (more standard)");
            println!("   - For files without extensions: add appropriate extension or add to .gitignore");
        } else {
            println!("✅ All files comply with the strict extension policy!");
        }
    }
}

#[derive(Debug, Serialize)]
pub enum FileViolation {
    DisallowedExtension { file: String, extension: String, suggestion: Option<String> },
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

        // No exemptions allowed - strict policy only
        // Only .rs and .jsonc files can exist without headers

        // Get file extension
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

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
                    result
                        .violations
                        .push(FileViolation::MissingGeneratedHeader {
                            file: path_str.to_string(),
                            extension: extension.to_string(),
                        });
                }
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to check generated header for {path_str}: {e}"
                    ));
                }
            }
            continue;
        }

        // Extension is not allowed - provide suggestions
        let suggestion = match extension {
            "yaml" => Some("Consider using .yml extension instead (more standard)"),
            "bash" => Some("Convert to .rs for Rust-based scripts"),
            "sed" => Some("Convert to .rs for Rust-based scripts"),
            "sh" => Some("Convert to .rs for Rust-based scripts"),
            "disabled" => Some("Remove .disabled extension or add to .gitignore"),
            "backup" => Some("Remove .backup extension or add to .gitignore"),
            "" => {
                // Check if it's a known file without extension
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                match filename {
                    "pre-add" => Some("Convert to .rs for Rust-based scripts"),
                    "CODEOWNERS" => Some("This file should have generated header"),
                    ".gitignore" => Some("This file should have generated header"),
                    ".gitattributes" => Some("This file should have generated header"),
                    ".editorconfig" => Some("This file should have generated header"),
                    ".envrc" => Some("This file should have generated header"),
                    _ => Some("Add appropriate extension or add to .gitignore"),
                }
            }
            _ => Some("Extension not allowed - convert to .rs or .jsonc for manual files"),
        };

        result.violations.push(FileViolation::DisallowedExtension {
            file: path_str.to_string(),
            extension: extension.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        });
    }

    Ok(result)
}

fn check_generated_header(path: &Path, policy: &FilePolicy, extension: &str) -> Result<bool> {
    let comment_syntax = match policy.get_generated_marker(extension) {
        Some(syntax) => syntax,
        None => return Ok(false), // No marker defined for this extension
    };

    let content =
        fs::read_to_string(path).context(format!("Failed to read file: {}", path.display()))?;

    // Handle JSON/JSONL files specially (no comments)
    if extension == "json" || extension == "jsonl" {
        return Ok(content.contains("\"_generated\"") || content.contains("\"_generated_by\""));
    }

    // For other files, check for the generated marker using comment syntax
    let marker = format!(
        "{} @generated {}",
        comment_syntax.prefix, comment_syntax.suffix
    );
    Ok(content.contains(&marker))
}

fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let lines = content.lines();
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
