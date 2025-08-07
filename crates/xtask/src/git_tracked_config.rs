use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Tracked Git configuration file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedGitConfig {
    /// File name (e.g., ".gitattributes", ".gitignore")
    pub file: String,
    /// Purpose/description of the file
    pub purpose: String,
    /// Whether the file is tracked by Git
    pub tracked: bool,
    /// File format type
    pub format: GitConfigFormat,
    /// Whether this file is optional
    pub optional: bool,
    /// File content hash for validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// Git configuration file formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitConfigFormat {
    /// Line-based format (one setting per line)
    LineBased,
    /// Glob pattern format
    Glob,
    /// INI configuration format
    Ini,
    /// JSON format
    Json,
    /// JSONC format (JSON with comments)
    Jsonc,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
}

/// Complete tracked Git configuration manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTrackedConfigManifest {
    /// All tracked configuration files
    pub files: Vec<TrackedGitConfig>,
    /// Repository metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryMetadata>,
    /// Validation rules
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub validation_rules: HashMap<String, String>,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Repository name
    pub name: String,
    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Whether submodules are used
    pub has_submodules: bool,
    /// Number of tracked config files
    pub config_file_count: usize,
}

/// Standard tracked Git configuration files
pub const STANDARD_TRACKED_CONFIGS: &[(&str, &str, GitConfigFormat, bool)] = &[
    (".gitattributes", "File behavior: diff, merge, binary, eol", GitConfigFormat::LineBased, false),
    (".gitignore", "Ignore untracked files", GitConfigFormat::Glob, false),
    (".gitmodules", "Submodule configuration", GitConfigFormat::Ini, true),
    (".mailmap", "Canonical author mapping", GitConfigFormat::LineBased, true),
];

impl GitTrackedConfigManifest {
    /// Create a new manifest
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            repository: None,
            validation_rules: HashMap::new(),
        }
    }

    /// Generate standard tracked config files
    pub fn generate_standard_configs() -> Self {
        let mut manifest = Self::new();
        
        for (file, purpose, format, optional) in STANDARD_TRACKED_CONFIGS {
            manifest.files.push(TrackedGitConfig {
                file: file.to_string(),
                purpose: purpose.to_string(),
                tracked: true,
                format: format.clone(),
                optional: *optional,
                content_hash: None,
                size_bytes: None,
                last_modified: None,
            });
        }
        
        manifest
    }

    /// Scan repository for tracked config files
    pub fn scan_repository<P: AsRef<Path>>(repo_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manifest = Self::new();
        
        // Get list of tracked files
        let output = Command::new("git")
            .args(["ls-files"])
            .current_dir(repo_path.as_ref())
            .output()?;
        
        let tracked_files = String::from_utf8_lossy(&output.stdout);
        
        // Filter for config-like files
        for line in tracked_files.lines() {
            if let Some(config) = Self::parse_tracked_config(line) {
                manifest.files.push(config);
            }
        }
        
        // Add repository metadata
        manifest.repository = Some(Self::get_repository_metadata(repo_path)?);
        
        Ok(manifest)
    }

    /// Parse a tracked file line to see if it's a config file
    fn parse_tracked_config(file_path: &str) -> Option<TrackedGitConfig> {
        // Check if it's a known config file
        for (known_file, purpose, format, optional) in STANDARD_TRACKED_CONFIGS {
            if file_path == *known_file {
                return Some(TrackedGitConfig {
                    file: known_file.to_string(),
                    purpose: purpose.to_string(),
                    tracked: true,
                    format: format.clone(),
                    optional: *optional,
                    content_hash: None,
                    size_bytes: None,
                    last_modified: None,
                });
            }
        }
        
        // Check for other .git* files
        if file_path.starts_with(".git") && !file_path.starts_with(".git/") {
            return Some(TrackedGitConfig {
                file: file_path.to_string(),
                purpose: "Custom Git configuration file".to_string(),
                tracked: true,
                format: GitConfigFormat::LineBased,
                optional: true,
                content_hash: None,
                size_bytes: None,
                last_modified: None,
            });
        }
        
        None
    }

    /// Get repository metadata
    fn get_repository_metadata<P: AsRef<Path>>(repo_path: P) -> Result<RepositoryMetadata, Box<dyn std::error::Error>> {
        // Get repository name from remote
        let name = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .current_dir(repo_path.as_ref())
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    url.split('/').last().map(|s| s.replace(".git", ""))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());
        
        // Check if submodules exist
        let has_submodules = Path::new(repo_path.as_ref()).join(".gitmodules").exists();
        
        Ok(RepositoryMetadata {
            name,
            url: None,
            has_submodules,
            config_file_count: 0, // Will be set later
        })
    }

    /// Validate all tracked config files
    pub fn validate_files<P: AsRef<Path>>(&self, repo_path: P) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for config in &self.files {
            let file_path = repo_path.as_ref().join(&config.file);
            
            if !file_path.exists() {
                if !config.optional {
                    errors.push(format!("Required config file missing: {}", config.file));
                }
                continue;
            }
            
            // Validate file format
            if let Err(e) = Self::validate_file_format(&file_path, &config.format) {
                errors.push(format!("Invalid format for {}: {}", config.file, e));
            }
            
            // Update file metadata
            if let Ok(metadata) = fs::metadata(&file_path) {
                // Update size
                if let Ok(size) = metadata.len().try_into() {
                    // Note: We can't update the struct directly here, but we could
                    // return the updated manifest or use a mutable reference
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate file format
    fn validate_file_format(file_path: &Path, format: &GitConfigFormat) -> Result<(), String> {
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Err("Failed to read file".to_string()),
        };
        
        match format {
            GitConfigFormat::Ini => {
                // Basic INI validation
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('[') {
                        if !trimmed.contains('=') {
                            return Err("Invalid INI format: missing '='".to_string());
                        }
                    }
                }
            }
            GitConfigFormat::Json => {
                // JSON validation
                if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                    return Err(format!("Invalid JSON: {}", e));
                }
            }
            GitConfigFormat::Jsonc => {
                // JSONC validation (strip comments first)
                let stripped = content.lines()
                    .filter(|line| !line.trim().starts_with("//") && !line.trim().starts_with("/*"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Err(e) = serde_json::from_str::<serde_json::Value>(&stripped) {
                    return Err(format!("Invalid JSONC: {}", e));
                }
            }
            GitConfigFormat::Yaml => {
                // YAML validation
                if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    return Err(format!("Invalid YAML: {}", e));
                }
            }
            GitConfigFormat::Toml => {
                // TOML validation
                if let Err(e) = toml::from_str::<toml::Value>(&content) {
                    return Err(format!("Invalid TOML: {}", e));
                }
            }
            _ => {
                // Line-based and glob formats don't need strict validation
            }
        }
        
        Ok(())
    }

    /// Convert to JSONC format
    pub fn to_jsonc(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut jsonc = serde_json::to_string_pretty(&self)?;
        
        // Add schema and documentation
        let schema = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Tracked Configuration Manifest",
  "description": "Manifest of tracked Git configuration files in the repository",
  "type": "object",
  "properties": {
    "files": {
      "type": "array",
      "description": "Tracked Git configuration files",
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "description": "File name (e.g., .gitattributes, .gitignore)"
          },
          "purpose": {
            "type": "string",
            "description": "Purpose/description of the file"
          },
          "tracked": {
            "type": "boolean",
            "description": "Whether the file is tracked by Git"
          },
          "format": {
            "type": "string",
            "description": "File format type"
          },
          "optional": {
            "type": "boolean",
            "description": "Whether this file is optional"
          },
          "content_hash": {
            "type": "string",
            "description": "File content hash for validation"
          },
          "size_bytes": {
            "type": "integer",
            "description": "File size in bytes"
          },
          "last_modified": {
            "type": "string",
            "description": "Last modified timestamp"
          }
        }
      }
    },
    "repository": {
      "type": "object",
      "description": "Repository metadata",
      "properties": {
        "name": {
          "type": "string",
          "description": "Repository name"
        },
        "url": {
          "type": "string",
          "description": "Repository URL"
        },
        "has_submodules": {
          "type": "boolean",
          "description": "Whether submodules are used"
        },
        "config_file_count": {
          "type": "integer",
          "description": "Number of tracked config files"
        }
      }
    },
    "validation_rules": {
      "type": "object",
      "description": "Validation rules for config files"
    }
  }
}"#;

        // Replace the opening brace with schema and documentation
        jsonc = jsonc.replacen("{", &format!("{}{{", schema), 1);
        
        Ok(jsonc)
    }

    /// Generate a summary of tracked config files
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("# Git Tracked Configuration Files\n\n");
        
        if let Some(repo) = &self.repository {
            summary.push_str(&format!("## Repository: {}\n", repo.name));
            summary.push_str(&format!("- Has submodules: {}\n", repo.has_submodules));
            summary.push_str(&format!("- Config files: {}\n\n", self.files.len()));
        }
        
        summary.push_str("## Tracked Configuration Files\n\n");
        
        for config in &self.files {
            let status = if config.tracked { "✅" } else { "❌" };
            let optional = if config.optional { " (optional)" } else { "" };
            
            summary.push_str(&format!("{} **{}**{}\n", status, config.file, optional));
            summary.push_str(&format!("   Purpose: {}\n", config.purpose));
            summary.push_str(&format!("   Format: {:?}\n", config.format));
            
            if let Some(size) = config.size_bytes {
                summary.push_str(&format!("   Size: {} bytes\n", size));
            }
            
            summary.push_str("\n");
        }
        
        summary
    }
}

/// Command-line interface for tracked Git config management
pub struct GitTrackedConfigManager;

impl GitTrackedConfigManager {
    /// Scan and generate manifest
    pub fn scan_repository<P: AsRef<Path>>(repo_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let manifest = GitTrackedConfigManifest::scan_repository(repo_path)?;
        
        // Validate files
        if let Err(errors) = manifest.validate_files(repo_path.as_ref()) {
            eprintln!("Validation warnings:");
            for error in errors {
                eprintln!("  - {}", error);
            }
        }
        
        // Generate JSONC
        let jsonc = manifest.to_jsonc()?;
        fs::write(output_path, jsonc)?;
        
        println!("✅ Git tracked config manifest generated successfully");
        Ok(())
    }

    /// Generate standard config templates
    pub fn generate_templates<P: AsRef<Path>>(output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)?;
        
        // Generate .gitattributes template
        let gitattributes = r#"# Git Attributes
# 
# This file controls how Git handles different file types.

# Text files with LF line endings
*.sh text eol=lf
*.bat text eol=crlf

# Binary files
*.jpg binary -diff -merge
*.png binary -diff -merge
*.zip binary -diff -merge

# Custom diff drivers
*.json diff=json
*.yaml diff=yaml

# Merge strategies
*.lock merge=union

# Export filtering
secret.key export-ignore

# Linguist overrides (GitHub)
*.ts linguist-language=TypeScript
docs/** linguist-documentation
"#;
        fs::write(output_dir.join(".gitattributes"), gitattributes)?;
        
        // Generate .gitignore template
        let gitignore = r#"# Git Ignore
# 
# This file specifies files that should be ignored by Git.

# Build artifacts
target/
dist/
build/

# Dependencies
node_modules/
vendor/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Environment files
.env
.env.local
.env.production

# Temporary files
*.tmp
*.temp
"#;
        fs::write(output_dir.join(".gitignore"), gitignore)?;
        
        // Generate .gitmodules template
        let gitmodules = r#"# Git Submodules
# 
# This file declares submodules for this repository.
# Currently, no submodules are used.
# 
# Example submodule declaration:
# [submodule "vendor/libfoo"]
#   path = vendor/libfoo
#   url = https://github.com/foo/libfoo.git
"#;
        fs::write(output_dir.join(".gitmodules"), gitmodules)?;
        
        // Generate .mailmap template
        let mailmap = r#"# Mailmap
# 
# This file maps author names and emails to canonical forms.
# Format: <canonical-email> <canonical-name> <alias-email> <alias-name>
# 
# Example:
# john.doe@company.com John Doe john.doe@gmail.com John Doe
# jane.smith@company.com Jane Smith jane@example.com Jane
"#;
        fs::write(output_dir.join(".mailmap"), mailmap)?;
        
        println!("✅ Git config templates generated successfully");
        Ok(())
    }

    /// Validate all tracked config files
    pub fn validate_files<P: AsRef<Path>>(repo_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let manifest = GitTrackedConfigManifest::scan_repository(repo_path.as_ref())?;
        
        println!("🔍 Validating tracked Git config files...");
        
        if let Err(errors) = manifest.validate_files(repo_path) {
            eprintln!("❌ Validation errors:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            return Err("Validation failed".into());
        }
        
        println!("✅ All tracked config files are valid");
        Ok(())
    }

    /// List tracked config files
    pub fn list_files<P: AsRef<Path>>(repo_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let manifest = GitTrackedConfigManifest::scan_repository(repo_path)?;
        
        println!("{}", manifest.generate_summary());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_standard_configs() {
        let manifest = GitTrackedConfigManifest::generate_standard_configs();
        assert_eq!(manifest.files.len(), STANDARD_TRACKED_CONFIGS.len());
        
        let gitattributes = manifest.files.iter().find(|f| f.file == ".gitattributes").unwrap();
        assert_eq!(gitattributes.purpose, "File behavior: diff, merge, binary, eol");
        assert!(!gitattributes.optional);
    }

    #[test]
    fn test_validate_file_format() {
        // Test INI format
        let temp_dir = tempfile::tempdir().unwrap();
        let ini_file = temp_dir.path().join("test.ini");
        fs::write(&ini_file, "[section]\nkey=value\n").unwrap();
        
        assert!(GitTrackedConfigManifest::validate_file_format(&ini_file, &GitConfigFormat::Ini).is_ok());
        
        // Test invalid INI
        fs::write(&ini_file, "invalid line without equals\n").unwrap();
        assert!(GitTrackedConfigManifest::validate_file_format(&ini_file, &GitConfigFormat::Ini).is_err());
    }
}
