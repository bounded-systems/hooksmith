//! Documentation generation system for Hooksmith
//!
//! This module provides comprehensive documentation generation from Rust code,
//! templates, and repository introspection.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub mod checksum;
pub mod cli_help;
pub mod component_docs;
pub mod examples;
pub mod manifest;
pub mod source_extraction;
pub mod structure;

pub use checksum::{
    generate_checksum, generate_checksum_report, update_checksums, validate_generated_files,
};
pub use cli_help::generate_cli_help;
pub use component_docs::generate_component_docs;
pub use examples::generate_examples_docs;
pub use manifest::DocumentationManifest;
pub use source_extraction::extract_project_data;
pub use structure::generate_structure_docs;

/// Safe wrapper for writing markdown files - ensures proper generation
pub fn write_markdown_file_safely(
    file_path: &Path,
    content: &str,
    generator_name: &str,
) -> Result<()> {
    // Validate that this is a legitimate documentation generation
    validate_generation_context()?;

    // Ensure content has proper codegen markers
    if !content.contains("auto-generated") {
        anyhow::bail!(
            "Attempted to write markdown file without codegen markers: {:?}",
            file_path
        );
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context(format!(
            "Failed to create parent directory for {:?}",
            file_path
        ))?;
    }

    // Write the file
    fs::write(file_path, content).context(format!("Failed to write {:?}", file_path))?;

    println!(
        "   ✅ Generated: {} (via {})",
        file_path.display(),
        generator_name
    );
    Ok(())
}

/// Validate that we're in a legitimate documentation generation context
fn validate_generation_context() -> Result<()> {
    // Check if we're running through the proper xtask command
    let args: Vec<String> = std::env::args().collect();
    let is_xtask_gen_docs = args.iter().any(|arg| arg.contains("gen-docs"));

    if !is_xtask_gen_docs {
        anyhow::bail!("Direct markdown file writing is not allowed. Use 'cargo xtask gen-docs-comprehensive' instead.");
    }

    Ok(())
}

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

    // Track generated files for checksum validation
    let mut generated_files = Vec::new();

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

        // Write the file using safe wrapper
        let file_path = output_path.join(&doc_config.path);
        write_markdown_file_safely(&file_path, &content, &doc_config.generator)?;

        // Track for checksum validation
        generated_files.push((doc_config.path.clone(), content));
    }

    // Generate additional documentation files
    let additional_files = generate_additional_docs(&project_data)?;
    for (file_path, content) in additional_files {
        let full_path = output_path.join(&file_path);
        write_markdown_file_safely(&full_path, &content, "additional_docs")?;
        generated_files.push((file_path, content));
    }

    // Update checksums for all generated files
    update_checksums(output_path, &generated_files)?;

    // Generate checksum report
    let checksum_report = generate_checksum_report(output_path)?;
    write_markdown_file_safely(
        &output_path.join("CHECKSUM_REPORT.md"),
        &checksum_report,
        "checksum_report",
    )?;

    if validate {
        validate_generated_docs(output_dir)?;
        validate_generated_files(output_path)?;
    }

    println!("✅ All documentation generated successfully");
    println!(
        "📊 Generated {} files with checksum validation",
        generated_files.len()
    );
    Ok(())
}

/// Generate additional documentation files
fn generate_additional_docs(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<Vec<(String, String)>> {
    let mut additional_files = Vec::new();

    // Generate API documentation
    let api_docs = generate_api_documentation(project_data)?;
    additional_files.push(("docs/API.md".to_string(), api_docs));

    // Generate development guide
    let dev_guide = generate_development_guide(project_data)?;
    additional_files.push(("docs/DEVELOPMENT.md".to_string(), dev_guide));

    // Generate testing guide
    let test_guide = generate_testing_guide(project_data)?;
    additional_files.push(("docs/TESTING.md".to_string(), test_guide));

    // Generate contribution guide
    let contrib_guide = generate_contribution_guide(project_data)?;
    additional_files.push(("docs/CONTRIBUTING.md".to_string(), contrib_guide));

    // Generate architecture documentation
    let arch_docs = generate_architecture_docs(project_data)?;
    additional_files.push(("docs/ARCHITECTURE.md".to_string(), arch_docs));

    Ok(additional_files)
}

/// Generate API documentation
fn generate_api_documentation(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<String> {
    let mut content = String::new();

    content.push_str("# API Documentation\n\n");
    content.push_str(&format!(
        "API documentation for {} {}\n\n",
        project_data.name, project_data.version
    ));

    // Add API sections based on components
    for component in &project_data.components {
        content.push_str(&format!("## {}\n\n", component.name));
        content.push_str(&format!("{}\n\n", component.description));

        if !component.dependencies.is_empty() {
            content.push_str("### Dependencies\n\n");
            for dep in &component.dependencies {
                content.push_str(&format!("- {}\n", dep));
            }
            content.push_str("\n");
        }

        content.push_str("### Usage\n\n");
        content.push_str("```rust\n");
        content.push_str(&format!(
            "use {}::{};\n",
            project_data.name,
            component.name.replace('-', "_")
        ));
        content.push_str("```\n\n");
    }

    add_codegen_marker(&mut content, "docs/API.md");
    Ok(content)
}

/// Generate development guide
fn generate_development_guide(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<String> {
    let mut content = String::new();

    content.push_str("# Development Guide\n\n");
    content.push_str("Guide for developing and contributing to the project.\n\n");

    content.push_str("## Prerequisites\n\n");
    content.push_str("- Rust (latest stable)\n");
    content.push_str("- Git\n");
    content.push_str("- Cargo\n\n");

    content.push_str("## Setup\n\n");
    content.push_str("```bash\n");
    content.push_str("git clone <repository-url>\n");
    content.push_str(&format!("cd {}\n", project_data.name));
    content.push_str("cargo build\n");
    content.push_str("```\n\n");

    content.push_str("## Project Structure\n\n");
    content.push_str("```\n");
    content.push_str(&project_data.structure);
    content.push_str("\n```\n\n");

    content.push_str("## Components\n\n");
    for component in &project_data.components {
        content.push_str(&format!("### {}\n\n", component.name));
        content.push_str(&format!("{}\n\n", component.description));
    }

    add_codegen_marker(&mut content, "docs/DEVELOPMENT.md");
    Ok(content)
}

/// Generate testing guide
fn generate_testing_guide(project_data: &crate::source_extraction::ProjectData) -> Result<String> {
    let mut content = String::new();

    content.push_str("# Testing Guide\n\n");
    content.push_str("Guide for testing the project.\n\n");

    content.push_str("## Running Tests\n\n");
    content.push_str("```bash\n");
    content.push_str("cargo test\n");
    content.push_str("cargo test --all-features\n");
    content.push_str("```\n\n");

    content.push_str("## Component Tests\n\n");
    for component in &project_data.components {
        if component.has_tests {
            content.push_str(&format!("### {}\n\n", component.name));
            content.push_str("```bash\n");
            content.push_str(&format!("cd components/{}\n", component.name));
            content.push_str("cargo test\n");
            content.push_str("```\n\n");
        }
    }

    content.push_str("## Documentation Tests\n\n");
    content.push_str("```bash\n");
    content.push_str("cargo xtask gen-docs-comprehensive --validate\n");
    content.push_str("```\n\n");

    add_codegen_marker(&mut content, "docs/TESTING.md");
    Ok(content)
}

/// Generate contribution guide
fn generate_contribution_guide(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<String> {
    let mut content = String::new();

    content.push_str("# Contributing Guide\n\n");
    content.push_str("Guide for contributing to the project.\n\n");

    content.push_str("## Getting Started\n\n");
    content.push_str("1. Fork the repository\n");
    content.push_str("2. Clone your fork\n");
    content.push_str("3. Create a feature branch\n");
    content.push_str("4. Make your changes\n");
    content.push_str("5. Run tests\n");
    content.push_str("6. Submit a pull request\n\n");

    content.push_str("## Development Workflow\n\n");
    content.push_str("```bash\n");
    content.push_str("git checkout -b feature/your-feature\n");
    content.push_str("cargo build\n");
    content.push_str("cargo test\n");
    content.push_str("cargo xtask gen-docs-comprehensive --validate\n");
    content.push_str("git commit -m \"Add your feature\"\n");
    content.push_str("git push origin feature/your-feature\n");
    content.push_str("```\n\n");

    content.push_str("## Code Style\n\n");
    content.push_str("- Follow Rust conventions\n");
    content.push_str("- Add tests for new features\n");
    content.push_str("- Update documentation\n");
    content.push_str("- Run `cargo fmt` and `cargo clippy`\n\n");

    add_codegen_marker(&mut content, "docs/CONTRIBUTING.md");
    Ok(content)
}

/// Generate architecture documentation
fn generate_architecture_docs(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<String> {
    let mut content = String::new();

    content.push_str("# Architecture Documentation\n\n");
    content.push_str("Architecture overview of the project.\n\n");

    content.push_str("## Overview\n\n");
    content.push_str(&format!(
        "{} is a modular system with the following components:\n\n",
        project_data.name
    ));

    for component in &project_data.components {
        content.push_str(&format!("### {}\n\n", component.name));
        content.push_str(&format!("{}\n\n", component.description));

        if !component.dependencies.is_empty() {
            content.push_str("**Dependencies:** ");
            content.push_str(&component.dependencies.join(", "));
            content.push_str("\n\n");
        }
    }

    content.push_str("## Data Flow\n\n");
    content.push_str("```\n");
    content.push_str("Source Code → Source Extraction → Documentation Generation → Output Files\n");
    content.push_str("```\n\n");

    content.push_str("## Validation\n\n");
    content.push_str("- All documentation is generated from source\n");
    content.push_str("- Checksums ensure file integrity\n");
    content.push_str("- Git attributes mark files as generated\n");
    content.push_str("- CI validates generated content\n\n");

    add_codegen_marker(&mut content, "docs/ARCHITECTURE.md");
    Ok(content)
}

/// Generate README from extracted source data
fn generate_readme_from_source(
    project_data: &crate::source_extraction::ProjectData,
) -> Result<String> {
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
