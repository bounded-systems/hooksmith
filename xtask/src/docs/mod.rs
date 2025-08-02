//! Documentation generation system for Hooksmith
//!
//! This module provides comprehensive documentation generation from Rust code,
//! templates, and repository introspection.

pub mod cli_help;
pub mod structure;
pub mod examples;
pub mod component_docs;
pub mod templates;
pub mod manifest;

pub use cli_help::generate_cli_help;
pub use structure::generate_structure_docs;
pub use examples::generate_examples_docs;
pub use component_docs::generate_component_docs;
pub use templates::generate_from_template;
pub use manifest::DocumentationManifest;

/// Generate all documentation based on the manifest
pub async fn generate_all_docs(output_dir: &str, validate: bool) -> anyhow::Result<()> {
    use std::path::Path;
    use std::fs;

    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).context("Failed to create output directory")?;
    }

    // Load the documentation manifest
    let manifest = DocumentationManifest::load()?;

    // Generate each documented file
    for doc_config in manifest.docs {
        println!("   Generating: {}", doc_config.path);
        
        match doc_config.generator.as_str() {
            "cli_help" => {
                let content = generate_cli_help()?;
                fs::write(output_path.join(&doc_config.path), content)
                    .context(format!("Failed to write {}", doc_config.path))?;
            }
            "structure" => {
                let content = generate_structure_docs()?;
                fs::write(output_path.join(&doc_config.path), content)
                    .context(format!("Failed to write {}", doc_config.path))?;
            }
            "examples" => {
                let content = generate_examples_docs()?;
                fs::write(output_path.join(&doc_config.path), content)
                    .context(format!("Failed to write {}", doc_config.path))?;
            }
            "component_readme" => {
                let content = generate_component_docs(&doc_config.path)?;
                fs::write(output_path.join(&doc_config.path), content)
                    .context(format!("Failed to write {}", doc_config.path))?;
            }
            _ => {
                println!("   ⚠️  Unknown generator: {}", doc_config.generator);
            }
        }
    }

    if validate {
        validate_generated_docs(output_dir)?;
    }

    println!("✅ All documentation generated successfully");
    Ok(())
}

/// Validate generated documentation files
fn validate_generated_docs(output_dir: &str) -> anyhow::Result<()> {
    use std::path::Path;
    use std::fs;

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

        println!("   ✅ Validated: {}", doc_config.path);
    }

    Ok(())
} 
