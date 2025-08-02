//! Documentation manifest management
//!
//! Defines the documentation generation configuration and manages the documentation manifest.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Documentation configuration for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocConfig {
    /// Path to the documentation file
    pub path: String,
    /// Generator to use for this file
    pub generator: String,
    /// Whether this file can be partially hand-edited
    #[serde(default)]
    pub partial_edit: bool,
    /// Template to use (if applicable)
    #[serde(default)]
    pub template: Option<String>,
    /// Additional configuration
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

/// Documentation manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationManifest {
    /// List of documentation files to generate
    pub docs: Vec<DocConfig>,
    /// Global configuration
    #[serde(default)]
    pub global_config: std::collections::HashMap<String, serde_json::Value>,
}

impl DocumentationManifest {
    /// Load the documentation manifest from file
    pub fn load() -> Result<Self> {
        let manifest_path = Path::new("config/docs_manifest.yaml");

        if manifest_path.exists() {
            let content = fs::read_to_string(manifest_path)
                .context("Failed to read documentation manifest")?;

            let manifest: DocumentationManifest =
                serde_yaml::from_str(&content).context("Failed to parse documentation manifest")?;

            Ok(manifest)
        } else {
            // Return default manifest if file doesn't exist
            Ok(Self::default())
        }
    }

    /// Save the documentation manifest to file
    pub fn save(&self) -> Result<()> {
        let manifest_path = Path::new("config/docs_manifest.yaml");

        // Ensure config directory exists
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content =
            serde_yaml::to_string(self).context("Failed to serialize documentation manifest")?;

        fs::write(manifest_path, content).context("Failed to write documentation manifest")?;

        Ok(())
    }

    /// Get documentation config by path
    pub fn get_by_path(&self, path: &str) -> Option<&DocConfig> {
        self.docs.iter().find(|doc| doc.path == path)
    }

    /// Get documentation configs by generator
    pub fn get_by_generator(&self, generator: &str) -> Vec<&DocConfig> {
        self.docs
            .iter()
            .filter(|doc| doc.generator == generator)
            .collect()
    }

    /// Add a new documentation configuration
    pub fn add_doc(&mut self, config: DocConfig) {
        // Remove existing config with same path
        self.docs.retain(|doc| doc.path != config.path);
        self.docs.push(config);
    }

    /// Remove a documentation configuration
    pub fn remove_doc(&mut self, path: &str) {
        self.docs.retain(|doc| doc.path != path);
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        // Check for duplicate paths
        let mut paths = std::collections::HashSet::new();
        for doc in &self.docs {
            if !paths.insert(&doc.path) {
                errors.push(format!("Duplicate path: {}", doc.path));
            }
        }

        // Check for valid generators
        let valid_generators = [
            "cli_help",
            "structure",
            "examples",
            "component_readme",
            "template",
        ];

        for doc in &self.docs {
            if !valid_generators.contains(&doc.generator.as_str()) {
                errors.push(format!(
                    "Invalid generator '{}' for path: {}",
                    doc.generator, doc.path
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!(
                "Documentation manifest validation failed:\n{}",
                errors.join("\n")
            );
        }
    }
}

impl Default for DocumentationManifest {
    fn default() -> Self {
        Self {
            docs: vec![
                DocConfig {
                    path: "docs/CLI_HELP.md".to_string(),
                    generator: "cli_help".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "docs/STRUCTURE.md".to_string(),
                    generator: "structure".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "docs/EXAMPLES.md".to_string(),
                    generator: "examples".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "components/cli-core/README.md".to_string(),
                    generator: "component_readme".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "components/git-filter/README.md".to_string(),
                    generator: "component_readme".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "components/hook-builder/README.md".to_string(),
                    generator: "component_readme".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "components/worktree-runner/README.md".to_string(),
                    generator: "component_readme".to_string(),
                    partial_edit: false,
                    template: None,
                    config: std::collections::HashMap::new(),
                },
                DocConfig {
                    path: "README.md".to_string(),
                    generator: "template".to_string(),
                    partial_edit: true,
                    template: Some("README".to_string()),
                    config: std::collections::HashMap::new(),
                },
            ],
            global_config: std::collections::HashMap::new(),
        }
    }
}

/// Generate the default documentation manifest
pub fn generate_default_manifest() -> Result<()> {
    let manifest = DocumentationManifest::default();
    manifest.save()?;
    println!("✅ Generated default documentation manifest at config/docs_manifest.yaml");
    Ok(())
}

/// Validate and update the documentation manifest
pub fn validate_and_update_manifest() -> Result<()> {
    let manifest = DocumentationManifest::load()?;

    // Validate the manifest
    manifest.validate()?;

    // Save the validated manifest
    manifest.save()?;

    println!("✅ Documentation manifest validated and updated");
    Ok(())
}

/// List all documentation configurations
pub fn list_documentation_configs() -> Result<()> {
    let manifest = DocumentationManifest::load()?;

    println!("Documentation Configuration:");
    println!("============================\n");

    for doc in &manifest.docs {
        println!("📄 {}", doc.path);
        println!("   Generator: {}", doc.generator);
        println!("   Partial Edit: {}", doc.partial_edit);
        if let Some(template) = &doc.template {
            println!("   Template: {}", template);
        }
        if !doc.config.is_empty() {
            println!("   Config: {:?}", doc.config);
        }
        println!();
    }

    Ok(())
}
