//! Template system for generating documentation and configuration files
//!
//! This module provides a type-safe template system that replaces Handlebars
//! templates with Rust structs and pretty-printers.

use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Display;

pub mod api;
pub mod diagrams;
pub mod examples;
pub mod readme;

/// Trait for all templates that can be rendered to strings
pub trait Template: Display {
    /// Render the template to a string
    fn render(&self) -> String {
        self.to_string()
    }

    /// Validate the template data
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Get the template name/identifier
    fn name(&self) -> &str;
}

/// Template engine that manages multiple templates
pub struct TemplateEngine {
    templates: HashMap<String, Box<dyn Template>>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template with the engine
    pub fn register<T: Template + 'static>(&mut self, template: T) {
        self.templates
            .insert(template.name().to_string(), Box::new(template));
    }

    /// Render a template by name
    pub fn render(&self, name: &str) -> Result<String> {
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", name))?;
        Ok(template.render())
    }

    /// Validate all registered templates
    pub fn validate_all(&self) -> Result<()> {
        for (name, template) in &self.templates {
            template
                .validate()
                .map_err(|e| anyhow::anyhow!("Template '{}' validation failed: {}", name, e))?;
        }
        Ok(())
    }

    /// Get a list of all registered template names
    pub fn template_names(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Project metadata for templates
#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub description: String,
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    pub authors: Vec<String>,
    #[allow(dead_code)]
    pub repository: Option<String>,
    #[allow(dead_code)]
    pub license: Option<String>,
    #[allow(dead_code)]
    pub keywords: Vec<String>,
    #[allow(dead_code)]
    pub categories: Vec<String>,
}

impl ProjectData {
    /// Create project data from Cargo.toml
    pub fn from_cargo_toml() -> Result<Self> {
        // This will be implemented to read from Cargo.toml
        // For now, return a placeholder
        Ok(Self {
            name: "hooksmith".to_string(),
            description: "Hooksmith bridges the gap between modern Git workflow tools and WebAssembly components".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Hooksmith Team".to_string()],
            repository: Some("https://github.com/hooksmith/hooksmith".to_string()),
            license: Some("MIT".to_string()),
            keywords: vec!["git", "hooks", "wasm", "rust", "lefthook"].into_iter().map(|s| s.to_string()).collect(),
            categories: vec!["development-tools".to_string()],
        })
    }
}

/// Module information for documentation
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    pub description: String,
    pub public_items: Vec<PublicItem>,
}

#[derive(Debug, Clone)]
pub struct PublicItem {
    pub name: String,
    #[allow(dead_code)]
    pub item_type: ItemType,
    pub description: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    #[allow(dead_code)]
    Function,
    #[allow(dead_code)]
    Struct,
    #[allow(dead_code)]
    Enum,
    #[allow(dead_code)]
    Trait,
    #[allow(dead_code)]
    Module,
    #[allow(dead_code)]
    Constant,
}

/// Example information for documentation
#[derive(Debug, Clone)]
pub struct ExampleInfo {
    pub name: String,
    pub description: String,
    pub code: String,
    pub output: Option<String>,
}

/// API documentation information
#[derive(Debug, Clone)]
pub struct ApiDocumentation {
    pub modules: Vec<ModuleInfo>,
    pub examples: Vec<ExampleInfo>,
}

impl ApiDocumentation {
    /// Generate API documentation from Rust source code
    pub fn from_rust_sources() -> Result<Self> {
        // This will be implemented to extract from Rust source
        // For now, return a placeholder
        Ok(Self {
            modules: vec![],
            examples: vec![],
        })
    }

    /// Render the API documentation as Markdown
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Render modules
        for module in &self.modules {
            output.push_str(&format!("## {}\n\n{}\n\n", module.name, module.description));

            for item in &module.public_items {
                output.push_str(&format!("### {}\n\n{}\n\n", item.name, item.description));
                if let Some(sig) = &item.signature {
                    output.push_str(&format!("```rust\n{sig}\n```\n\n"));
                }
            }
        }

        // Render examples
        if !self.examples.is_empty() {
            output.push_str("## Examples\n\n");
            for example in &self.examples {
                output.push_str(&format!(
                    "### {}\n\n{}\n\n",
                    example.name, example.description
                ));
                output.push_str(&format!("```rust\n{}\n```\n\n", example.code));
                if let Some(output_text) = &example.output {
                    output.push_str(&format!("Output:\n\n```\n{output_text}\n```\n\n"));
                }
            }
        }

        output
    }
}
