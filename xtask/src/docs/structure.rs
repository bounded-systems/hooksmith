//! Repository structure documentation generator
//!
//! Introspects the repository to generate comprehensive structure documentation.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repository structure information
#[derive(Debug)]
struct StructureInfo {
    total_files: usize,
    rust_files: usize,
    config_files: usize,
    doc_files: usize,
    script_files: usize,
    directories: Vec<String>,
    file_types: HashMap<String, usize>,
}

/// Generate structure documentation
pub fn generate_structure_docs() -> Result<String> {
    let mut content = String::new();

    content.push_str("# Repository Structure\n\n");
    content.push_str("This document shows the complete file structure of the repository.\n\n");

    // Get repository structure
    let structure = analyze_repository_structure()?;

    content.push_str("## 📁 File Structure\n\n");
    content.push_str("```\n");
    content.push_str(&generate_tree_structure()?);
    content.push_str("```\n\n");

    content.push_str("## 📊 File Count Summary\n\n");
    content.push_str(&format!(
        "- **Total Files**:       {}\n",
        structure.total_files
    ));
    content.push_str(&format!(
        "- **Rust Files**:        {} (.rs)\n",
        structure.rust_files
    ));
    content.push_str(&format!(
        "- **Configuration Files**:        {} (.toml, .yaml, .rc)\n",
        structure.config_files
    ));
    content.push_str(&format!(
        "- **Documentation**:        {} (.md)\n",
        structure.doc_files
    ));
    content.push_str(&format!(
        "- **Scripts**:        {} (.sh)\n",
        structure.script_files
    ));
    content.push_str("\n");

    // File type breakdown
    content.push_str("## 📋 File Type Breakdown\n\n");
    content.push_str("| Extension | Count | Description |\n");
    content.push_str("|-----------|-------|-------------|\n");

    for (ext, count) in &structure.file_types {
        let description = match ext.as_str() {
            "rs" => "Rust source files",
            "toml" => "Cargo and configuration files",
            "md" => "Markdown documentation",
            "yml" | "yaml" => "YAML configuration files",
            "json" => "JSON schema and config files",
            "sh" => "Shell scripts",
            "wit" => "WebAssembly Interface Type definitions",
            "css" => "Stylesheet files",
            "html" => "HTML documentation",
            "pdf" => "PDF documentation",
            "epub" => "EPUB documentation",
            _ => "Other files",
        };
        content.push_str(&format!("| .{} | {} | {} |\n", ext, count, description));
    }

    content.push_str("\n");

    // Component breakdown
    content.push_str("## 🧩 Component Breakdown\n\n");
    content.push_str(&generate_component_breakdown()?);

    // Git information
    content.push_str("## 📈 Repository Information\n\n");
    content.push_str(&generate_git_info()?);

    content.push_str("\n---\n\n");
    content.push_str("*Generated on ");
    content.push_str(
        &chrono::Utc::now()
            .format("%a %b %e %H:%M:%S %Z %Y")
            .to_string(),
    );
    content.push_str(" using `cargo xtask gen-docs-comprehensive`. This file is auto-generated and should not be edited manually.*\n");

    Ok(content)
}

/// Analyze repository structure
fn analyze_repository_structure() -> Result<StructureInfo> {
    let mut info = StructureInfo {
        total_files: 0,
        rust_files: 0,
        config_files: 0,
        doc_files: 0,
        script_files: 0,
        directories: Vec::new(),
        file_types: HashMap::new(),
    };

    let entries = fs::read_dir(".").context("Failed to read repository root")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !dir_name.starts_with('.') && dir_name != "target" {
                info.directories.push(dir_name.to_string());
            }
        } else if path.is_file() {
            info.total_files += 1;

            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                let ext = extension.to_lowercase();
                *info.file_types.entry(ext.clone()).or_insert(0) += 1;

                match ext.as_str() {
                    "rs" => info.rust_files += 1,
                    "toml" | "yaml" | "yml" | "json" => info.config_files += 1,
                    "md" => info.doc_files += 1,
                    "sh" => info.script_files += 1,
                    _ => {}
                }
            }
        }
    }

    // Count files in subdirectories
    count_files_recursive(".", &mut info)?;

    Ok(info)
}

/// Count files recursively in directories
fn count_files_recursive(dir: &str, info: &mut StructureInfo) -> Result<()> {
    let entries = fs::read_dir(dir).context(format!("Failed to read directory: {}", dir))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !dir_name.starts_with('.') && dir_name != "target" {
                count_files_recursive(&path.to_string_lossy(), info)?;
            }
        } else if path.is_file() {
            info.total_files += 1;

            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                let ext = extension.to_lowercase();
                *info.file_types.entry(ext.clone()).or_insert(0) += 1;

                match ext.as_str() {
                    "rs" => info.rust_files += 1,
                    "toml" | "yaml" | "yml" | "json" => info.config_files += 1,
                    "md" => info.doc_files += 1,
                    "sh" => info.script_files += 1,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

/// Generate tree structure using git ls-tree
fn generate_tree_structure() -> Result<String> {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .output()
        .context("Failed to get git tree structure")?;

    let files = String::from_utf8_lossy(&output.stdout);
    let mut tree_lines: Vec<String> = files.lines().map(|s| s.to_string()).collect();
    tree_lines.sort();

    let mut tree_structure = String::new();
    tree_structure.push_str(".\n");

    for file in tree_lines {
        if !file.starts_with("target/") && !file.starts_with(".git/") {
            let parts: Vec<&str> = file.split('/').collect();
            for (i, part) in parts.iter().enumerate() {
                let indent = "  ".repeat(i);
                if i == parts.len() - 1 {
                    tree_structure.push_str(&format!("{}├── {}\n", indent, part));
                } else {
                    tree_structure.push_str(&format!("{}├── {}/\n", indent, part));
                }
            }
        }
    }

    Ok(tree_structure)
}

/// Generate component breakdown
fn generate_component_breakdown() -> Result<String> {
    let mut content = String::new();

    // Check for components directory
    if Path::new("components").exists() {
        content.push_str("### Components\n\n");
        content.push_str("| Component | Description | Status |\n");
        content.push_str("|-----------|-------------|--------|\n");

        let components = vec![
            ("cli-core", "Core CLI functionality", "✅ Active"),
            ("git-filter", "Git filter system", "✅ Active"),
            ("hook-builder", "Hook building system", "✅ Active"),
            ("worktree-runner", "Worktree management", "✅ Active"),
        ];

        for (name, description, status) in components {
            content.push_str(&format!("| {} | {} | {} |\n", name, description, status));
        }
        content.push_str("\n");
    }

    // Check for main source structure
    content.push_str("### Source Structure\n\n");
    content.push_str("| Directory | Purpose |\n");
    content.push_str("|-----------|---------|\n");
    content.push_str("| `src/` | Main application source code |\n");
    content.push_str("| `src/commands/` | CLI command implementations |\n");
    content.push_str("| `src/modules/` | Core functionality modules |\n");
    content.push_str("| `xtask/` | Build and code generation tasks |\n");
    content.push_str("| `tests/` | Test files and integration tests |\n");
    content.push_str("| `docs/` | Generated and hand-written documentation |\n");
    content.push_str("| `examples/` | Example code and demonstrations |\n");
    content.push_str("| `hooks/` | Git hook scripts and configurations |\n");
    content.push_str("| `schemas/` | JSON schema definitions |\n");
    content.push_str("| `wit/` | WebAssembly Interface Type definitions |\n");
    content.push_str("\n");

    Ok(content)
}

/// Generate git repository information
fn generate_git_info() -> Result<String> {
    let mut content = String::new();

    // Get git status
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to get git status")?;

    let status_output = String::from_utf8_lossy(&status.stdout);
    let modified_files = status_output.lines().count();

    // Get branch information
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    let branch_output = String::from_utf8_lossy(&branch.stdout);
    let current_branch = branch_output.trim();

    // Get commit count
    let commit_count = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .context("Failed to get commit count")?;

    let commit_output = String::from_utf8_lossy(&commit_count.stdout);
    let commits = commit_output.trim();

    content.push_str(&format!("- **Current Branch**: {}\n", current_branch));
    content.push_str(&format!("- **Total Commits**: {}\n", commits));
    content.push_str(&format!("- **Modified Files**: {}\n", modified_files));

    Ok(content)
}
