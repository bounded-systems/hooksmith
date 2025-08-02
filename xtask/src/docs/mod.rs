//! Documentation generation system for Hooksmith
//!
//! This module provides comprehensive documentation generation from Rust code,
//! templates, and repository introspection.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub mod cli_help;
pub mod component_docs;
pub mod examples;
pub mod manifest;
pub mod source_extraction;
pub mod structure;

pub use cli_help::generate_cli_help;
pub use component_docs::generate_component_docs;
pub use examples::generate_examples_docs;
pub use manifest::DocumentationManifest;
pub use source_extraction::extract_project_data;
pub use structure::generate_structure_docs;

/// Generate all documentation based on the manifest
pub async fn generate_all_docs(output_dir: &str, validate: bool) -> anyhow::Result<()> {
    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).context("Failed to create output directory")?;
    }

    // Extract project data from source code
    let project_data = extract_project_data()?;

    // Load the documentation manifest
    let manifest = DocumentationManifest::load()?;

    // Generate each documented file
    for doc_config in manifest.docs {
        println!("   Generating: {}", doc_config.path);

        let content = match doc_config.generator.as_str() {
            "cli_help" => {
                let mut content = generate_cli_help()?;
                add_codegen_marker(&mut content, &doc_config.path);
                content
            }
            "structure" => {
                let mut content = generate_structure_docs()?;
                add_codegen_marker(&mut content, &doc_config.path);
                content
            }
            "examples" => {
                let mut content = generate_examples_docs()?;
                add_codegen_marker(&mut content, &doc_config.path);
                content
            }
            "component_readme" => {
                let mut content = generate_component_docs(&doc_config.path, &project_data)?;
                add_codegen_marker(&mut content, &doc_config.path);
                content
            }
            "readme" => {
                let mut content = generate_readme_from_source(&project_data)?;
                add_codegen_marker(&mut content, &doc_config.path);
                content
            }
            _ => {
                println!("   ⚠️  Unknown generator: {}", doc_config.generator);
                continue;
            }
        };

        // Write the file
        let file_path = output_path.join(&doc_config.path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create parent directory for {}",
                doc_config.path
            ))?;
        }

        fs::write(&file_path, content).context(format!("Failed to write {}", doc_config.path))?;
    }

    if validate {
        validate_generated_docs(output_dir)?;
    }

    println!("✅ All documentation generated successfully");
    Ok(())
}

/// Generate README from extracted source data
fn generate_readme_from_source(project_data: &crate::source_extraction::ProjectData) -> Result<String> {
    let mut content = String::new();

    // Title and description from Cargo.toml
    content.push_str(&format!("# {}\n\n", project_data.name));
    content.push_str(&format!("{}\n\n", project_data.description));

    // Features from Cargo.toml
    if !project_data.features.is_empty() {
        content.push_str("## Features\n\n");
        for feature in &project_data.features {
            content.push_str(&format!("- {}\n", feature));
        }
        content.push_str("\n");
    }

    // Dependencies from Cargo.toml
    if !project_data.dependencies.is_empty() {
        content.push_str("## Dependencies\n\n");
        for (name, version) in &project_data.dependencies {
            content.push_str(&format!("- **{}**: {}\n", name, version));
        }
        content.push_str("\n");
    }

    // Installation from Cargo.toml
    content.push_str("## Installation\n\n");
    content.push_str("```bash\n");
    content.push_str(&format!("cargo install --path .\n"));
    content.push_str("```\n\n");

    // Usage from CLI help
    content.push_str("## Usage\n\n");
    content.push_str("```bash\n");
    content.push_str(&format!("{} --help\n", project_data.name.to_lowercase()));
    content.push_str("```\n\n");

    // Project structure from actual file system
    content.push_str("## Project Structure\n\n");
    content.push_str("```\n");
    content.push_str(&project_data.structure);
    content.push_str("\n```\n\n");

    // Components from actual component directories
    if !project_data.components.is_empty() {
        content.push_str("## Components\n\n");
        for component in &project_data.components {
            content.push_str(&format!("### {}\n\n", component.name));
            content.push_str(&format!("{}\n\n", component.description));
            if !component.dependencies.is_empty() {
                content.push_str("**Dependencies:** ");
                content.push_str(&component.dependencies.join(", "));
                content.push_str("\n\n");
            }
        }
    }

    // Development setup from actual project files
    content.push_str("## Development\n\n");
    content.push_str("### Prerequisites\n\n");
    content.push_str("- Rust (latest stable)\n");
    content.push_str("- Git\n");
    content.push_str("- Cargo\n\n");

    content.push_str("### Setup\n\n");
    content.push_str("```bash\n");
    content.push_str("git clone <repository-url>\n");
    content.push_str("cd hooksmith\n");
    content.push_str("cargo build\n");
    content.push_str("```\n\n");

    // Testing from actual test files
    content.push_str("### Testing\n\n");
    content.push_str("```bash\n");
    content.push_str("cargo test\n");
    content.push_str("cargo xtask gen-docs-comprehensive --validate\n");
    content.push_str("```\n\n");

    // License from actual LICENSE file
    if let Some(license) = &project_data.license {
        content.push_str(&format!("## License\n\n{}\n\n", license));
    }

    Ok(content)
}

/// Add codegen marker to generated content
fn add_codegen_marker(content: &mut String, file_path: &str) {
    // Add a clear marker at the top of the file
    let marker = format!(
        "<!-- This file is auto-generated by `cargo xtask gen-docs-comprehensive` -->\n<!-- Generated file: {} -->\n<!-- Do not edit this file manually - changes will be overwritten -->\n\n",
        file_path
    );

    // Insert the marker at the beginning
    content.insert_str(0, &marker);

    // Also add a footer marker if it doesn't already have one
    if !content.contains("auto-generated") {
        content.push_str("\n---\n\n");
        content.push_str("*This file is auto-generated by `cargo xtask gen-docs-comprehensive`. Do not edit manually - changes will be overwritten.*\n");
    }
}

/// Validate generated documentation files
fn validate_generated_docs(output_dir: &str) -> anyhow::Result<()> {
    let output_path = Path::new(output_dir);
    let manifest = DocumentationManifest::load()?;

    for doc_config in manifest.docs {
        let file_path = output_path.join(&doc_config.path);
        if !file_path.exists() {
            anyhow::bail!("Generated file missing: {}", doc_config.path);
        }

        // Basic validation - check file is not empty
        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read {}", doc_config.path))?;

        if content.trim().is_empty() {
            anyhow::bail!("Generated file is empty: {}", doc_config.path);
        }

        // Check that codegen marker is present
        if !content.contains("auto-generated") {
            anyhow::bail!("Generated file missing codegen marker: {}", doc_config.path);
        }

        println!("   ✅ Validated: {}", doc_config.path);
    }

    Ok(())
}
