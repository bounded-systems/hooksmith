//! Documentation manifest configuration generation
//!
//! This module provides Rust structs for generating documentation manifest
//! configuration files from code rather than manually maintaining YAML files.

use crate::config::ConfigFile;
use serde::{Deserialize, Serialize};

/// Documentation section definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocSection {
    pub name: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl DocSection {
    /// Create a new documentation section
    pub fn new(name: &str, title: &str, description: &str, source: &str, output: &str) -> Self {
        Self {
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            source: source.to_string(),
            output: output.to_string(),
            template: None,
            metadata: None,
        }
    }

    /// Add a template to the section
    pub fn with_template(mut self, template: &str) -> Self {
        self.template = Some(template.to_string());
        self
    }
}

/// Documentation format definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocFormat {
    pub name: String,
    pub extension: String,
    pub generator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_yaml::Value>,
}

impl DocFormat {
    /// Create a new documentation format
    pub fn new(name: &str, extension: &str, generator: &str) -> Self {
        Self {
            name: name.to_string(),
            extension: extension.to_string(),
            generator: generator.to_string(),
            options: None,
        }
    }
}

/// Main documentation manifest configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct DocsManifest {
    pub sections: Vec<DocSection>,
    pub formats: Vec<DocFormat>,
    pub output_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl ConfigFile for DocsManifest {}

impl DocsManifest {
    /// Generate default documentation manifest
    pub fn generate() -> Self {
        Self {
            sections: vec![
                DocSection::new(
                    "readme",
                    "README",
                    "Project overview and getting started guide",
                    "src/main.rs",
                    "README.md",
                )
                .with_template("readme"),
                DocSection::new(
                    "api",
                    "API Documentation",
                    "Complete API reference documentation",
                    "src/",
                    "docs/api.md",
                )
                .with_template("api"),
                DocSection::new(
                    "examples",
                    "Examples",
                    "Code examples and usage patterns",
                    "examples/",
                    "docs/examples.md",
                )
                .with_template("examples"),
                DocSection::new(
                    "architecture",
                    "Architecture",
                    "System architecture and design decisions",
                    "src/",
                    "docs/architecture.md",
                )
                .with_template("architecture"),
                DocSection::new(
                    "development",
                    "Development Guide",
                    "Development setup and contribution guidelines",
                    "CONTRIBUTING.md",
                    "docs/development.md",
                ),
                DocSection::new(
                    "contracts",
                    "Contract System",
                    "Contract validation system documentation",
                    "src/modules/contract_state_machine.rs",
                    "docs/contracts.md",
                )
                .with_template("contracts"),
                DocSection::new(
                    "git_filters",
                    "Git Filters",
                    "Git filter system documentation",
                    "components/git-filter/",
                    "docs/git_filters.md",
                )
                .with_template("git_filters"),
            ],
            formats: vec![
                DocFormat::new("markdown", "md", "markdown"),
                DocFormat::new("html", "html", "html"),
                DocFormat::new("pdf", "pdf", "pandoc"),
                DocFormat::new("epub", "epub", "pandoc"),
            ],
            output_dir: "docs/".to_string(),
            metadata: None,
        }
    }

    /// Add a new documentation section
    pub fn add_section(&mut self, section: DocSection) {
        self.sections.push(section);
    }

    /// Add a new documentation format
    pub fn add_format(&mut self, format: DocFormat) {
        self.formats.push(format);
    }

    /// Get section by name
    pub fn get_section(&self, name: &str) -> Option<&DocSection> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// Get format by name
    pub fn get_format(&self, name: &str) -> Option<&DocFormat> {
        self.formats.iter().find(|f| f.name == name)
    }
}
