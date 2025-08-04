//! Unified Generated File System
//!
//! This module provides a single source of truth for all generated files.
//! Each generated file has a JSONC source file in `generated-sources/` that
//! defines its path, slug, and content. The generator processes these sources,
//! adds checksums, and updates the registry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
// use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Source file definition for a generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSource {
    /// Target file path relative to project root
    pub path: String,
    /// Unique identifier for this generated file
    pub slug: String,
    /// Raw content to be written to the target file
    pub content: String,
    /// Optional description of what this file contains
    #[serde(default)]
    pub description: Option<String>,
    /// Optional list of other source files this depends on
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Registry entry for a generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFileEntry {
    /// Unique identifier
    pub slug: String,
    /// File path relative to project root
    pub path: String,
    /// File extension
    #[serde(alias = "type")]
    pub extension: String,
    /// SHA256 checksum of the file content (excluding header)
    pub checksum: String,
    /// File type for generation commands
    #[serde(default)]
    pub file_type: String,
}

/// Registry of all generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFilesRegistry {
    /// Schema reference
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Registry title
    pub title: String,
    /// Registry description
    pub description: String,
    /// List of all generated files
    pub files: Vec<GeneratedFileEntry>,
    /// Directories to ignore
    pub ignore: IgnoreRules,
}

/// Rules for ignoring files and directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRules {
    /// Directories to ignore
    pub dirs: Vec<String>,
    /// File patterns to ignore
    pub patterns: Vec<String>,
}

/// Unified generator for all generated files
pub struct UnifiedGenerator {
    /// Project root directory
    project_root: PathBuf,
    /// Generated sources directory
    sources_dir: PathBuf,
    /// Registry file path
    registry_path: PathBuf,
}

impl UnifiedGenerator {
    /// Create a new unified generator
    pub fn new(project_root: PathBuf) -> Self {
        let sources_dir = project_root.join("generated-sources");
        let registry_path = project_root.join("config").join("generated-files.jsonc");

        Self {
            project_root,
            sources_dir,
            registry_path,
        }
    }

    /// Load all source files from the generated-sources directory
    pub fn load_sources(&self) -> Result<Vec<GeneratedSource>> {
        let mut sources = Vec::new();

        if !self.sources_dir.exists() {
            return Ok(sources);
        }

        for entry in fs::read_dir(&self.sources_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonc") {
                // Skip schema files
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if filename == "schema.jsonc" {
                    continue;
                }

                let content = fs::read_to_string(&path)?;
                let source: GeneratedSource = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse source file: {}", path.display()))?;
                sources.push(source);
            }
        }

        Ok(sources)
    }

    /// Load the current registry
    pub fn load_registry(&self) -> Result<GeneratedFilesRegistry> {
        if !self.registry_path.exists() {
            return Ok(GeneratedFilesRegistry {
                schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
                title: "Hooksmith Generated Files".to_string(),
                description: "Registry of generated files with stable slugs and checksums for validation and regeneration.".to_string(),
                files: Vec::new(),
                ignore: IgnoreRules {
                    dirs: vec![
                        "target/".to_string(),
                        "dist/".to_string(),
                        "node_modules/".to_string(),
                        "logs/".to_string(),
                        "status-trends/".to_string(),
                        "generated_file_demo/".to_string(),
                        ".cargo/".to_string(),
                        ".trunk/".to_string(),
                        ".hooks/".to_string(),
                        ".git/".to_string(),
                    ],
                    patterns: vec![
                        "*.lock".to_string(),
                        "*.jsonl".to_string(),
                    ],
                },
            });
        }

        let content = fs::read_to_string(&self.registry_path)?;
        let registry: GeneratedFilesRegistry =
            serde_json::from_str(&content).with_context(|| {
                format!("Failed to parse registry: {}", self.registry_path.display())
            })?;

        Ok(registry)
    }

    /// Save the registry to disk
    pub fn save_registry(&self, registry: &GeneratedFilesRegistry) -> Result<()> {
        // Ensure the config directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(registry)?;
        fs::write(&self.registry_path, content)?;

        Ok(())
    }

    /// Compute checksum for content (excluding header)
    pub fn compute_checksum(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..8].to_string() // First 8 characters
    }

    /// Get file extension from path
    pub fn get_extension(&self, path: &str) -> String {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Get file type for generation commands
    pub fn get_file_type(&self, path: &str) -> String {
        let extension = self.get_extension(path);
        match extension.as_str() {
            "md" => "md".to_string(),
            "yml" | "yaml" => "yaml".to_string(),
            "toml" => "toml".to_string(),
            "json" => "json".to_string(),
            "jsonc" => "jsonc".to_string(),
            "gitignore" => "gitignore".to_string(),
            "gitattributes" => "gitattributes".to_string(),
            "CODEOWNERS" => "CODEOWNERS".to_string(),
            "wit" => "wit".to_string(),
            "makefile" => "makefile".to_string(),
            "editorconfig" => "editorconfig".to_string(),
            "envrc" => "envrc".to_string(),
            _ => extension,
        }
    }

    /// Generate header with checksum for a file
    pub fn generate_header(&self, content: &str, file_type: &str) -> String {
        let checksum = self.compute_checksum(content);

        match file_type {
            "md" => format!(
                "<!-- @generated by xtask gen-files --file-type=md -->\n<!-- @checksum: {} -->\n\n",
                checksum
            ),
            "yml" | "yaml" => format!(
                "# @generated by xtask gen-files --file-type={}\n# @checksum: {}\n\n",
                file_type, checksum
            ),
            "toml" => format!(
                "# @generated by xtask gen-files --file-type=toml\n# @checksum: {}\n\n",
                checksum
            ),
            "json" | "jsonc" => format!(
                "// @generated by xtask gen-files --file-type={}\n// @checksum: {}\n\n",
                file_type, checksum
            ),
            "gitignore" | "gitattributes" => format!(
                "# @generated by xtask gen-files --file-type={}\n# @checksum: {}\n\n",
                file_type, checksum
            ),
            "CODEOWNERS" => format!(
                "# @generated by xtask gen-files --file-type=CODEOWNERS\n# @checksum: {}\n\n",
                checksum
            ),
            "wit" => format!(
                "// @generated by xtask gen-files --file-type=wit\n// @checksum: {}\n\n",
                checksum
            ),
            "makefile" => format!(
                "# @generated by xtask gen-files --file-type=makefile\n# @checksum: {}\n\n",
                checksum
            ),
            "editorconfig" => format!(
                "# @generated by xtask gen-files --file-type=editorconfig\n# @checksum: {}\n\n",
                checksum
            ),
            "envrc" => format!(
                "# @generated by xtask gen-files --file-type=envrc\n# @checksum: {}\n\n",
                checksum
            ),
            _ => format!(
                "# @generated by xtask gen-files --file-type={}\n# @checksum: {}\n\n",
                file_type, checksum
            ),
        }
    }

    /// Pretty print content based on file type
    pub fn pretty_print(&self, content: &str, file_type: &str) -> Result<String> {
        match file_type {
            "json" | "jsonc" => {
                // Parse and pretty print JSON
                let parsed: serde_json::Value = serde_json::from_str(content)?;
                Ok(serde_json::to_string_pretty(&parsed)?)
            }
            "toml" => {
                // Parse and pretty print TOML
                let parsed: toml::Value = toml::from_str(content)?;
                Ok(toml::to_string_pretty(&parsed)?)
            }
            "yaml" | "yml" => {
                // Parse and pretty print YAML
                let parsed: serde_yaml::Value = serde_yaml::from_str(content)?;
                Ok(serde_yaml::to_string(&parsed)?)
            }
            _ => {
                // For other file types, return content as-is
                Ok(content.to_string())
            }
        }
    }

    /// Generate a single file from source
    pub fn generate_file(&self, source: &GeneratedSource) -> Result<GeneratedFileEntry> {
        let file_type = self.get_file_type(&source.path);
        let extension = self.get_extension(&source.path);

        // Pretty print the content
        let pretty_content = self.pretty_print(&source.content, &file_type)?;

        // Generate header with checksum
        let header = self.generate_header(&pretty_content, &file_type);

        // Combine header and content
        let final_content = format!("{}{}", header, pretty_content);

        // Write to target path
        let target_path = self.project_root.join(&source.path);

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&target_path, final_content)?;

        // Create registry entry
        let entry = GeneratedFileEntry {
            slug: source.slug.clone(),
            path: source.path.clone(),
            extension,
            checksum: self.compute_checksum(&pretty_content),
            file_type,
        };

        Ok(entry)
    }

    /// Generate all files from sources
    pub fn generate_all(&self) -> Result<GeneratedFilesRegistry> {
        println!("🚀 Generating all files from unified sources...");

        // Load sources
        let sources = self.load_sources()?;
        println!("📁 Found {} source files", sources.len());

        // Load current registry
        let mut registry = self.load_registry()?;

        // Clear existing files
        registry.files.clear();

        // Generate each file
        for source in &sources {
            println!("  📄 Generating: {}", source.path);
            let entry = self.generate_file(source)?;
            registry.files.push(entry);
        }

        // Save updated registry
        self.save_registry(&registry)?;

        println!("✅ Generated {} files", registry.files.len());
        println!("📋 Registry updated: {}", self.registry_path.display());

        Ok(registry)
    }

    /// Clean all generated files
    pub fn clean_all(&self) -> Result<()> {
        println!("🧹 Cleaning all generated files...");

        let registry = self.load_registry()?;
        let mut cleaned_count = 0;

        for entry in &registry.files {
            let target_path = self.project_root.join(&entry.path);
            if target_path.exists() {
                fs::remove_file(&target_path)?;
                cleaned_count += 1;
                println!("  🗑️  Removed: {}", entry.path);
            }
        }

        println!("✅ Cleaned {} files", cleaned_count);
        Ok(())
    }

    /// Validate that all generated files match their registry entries
    pub fn validate_all(&self) -> Result<bool> {
        println!("🔍 Validating generated files...");

        let registry = self.load_registry()?;
        let mut all_valid = true;

        for entry in &registry.files {
            let target_path = self.project_root.join(&entry.path);

            if !target_path.exists() {
                println!("  ❌ Missing: {}", entry.path);
                all_valid = false;
                continue;
            }

            let content = fs::read_to_string(&target_path)?;

            // Extract content without header (everything after the first double newline)
            let content_without_header = if let Some(pos) = content.find("\n\n") {
                &content[pos + 2..]
            } else {
                &content
            };

            let current_checksum = self.compute_checksum(content_without_header);

            if current_checksum != entry.checksum {
                println!(
                    "  ❌ Checksum mismatch: {} (expected: {}, got: {})",
                    entry.path, entry.checksum, current_checksum
                );
                all_valid = false;
            } else {
                println!("  ✅ Valid: {}", entry.path);
            }
        }

        if all_valid {
            println!("✅ All generated files are valid");
        } else {
            println!("❌ Some generated files are invalid");
        }

        Ok(all_valid)
    }
}
