//! Source code extraction for documentation generation
//!
//! This module extracts project data directly from the source code,
//! including Cargo.toml, file structure, and component information.

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, Package};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Project data extracted from source code
#[derive(Debug)]
pub struct ProjectData {
    pub name: String,
    pub description: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub features: Vec<String>,
    pub structure: String,
    pub components: Vec<ComponentData>,
    pub workspace_members: Vec<String>,
    pub git_info: GitInfo,
}

/// Component data extracted from component directories
#[derive(Debug, Clone)]
pub struct ComponentData {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
    pub path: PathBuf,
    pub has_readme: bool,
    pub has_tests: bool,
}

/// Git repository information
#[derive(Debug)]
pub struct GitInfo {
    pub branch: String,
    pub commit: String,
    pub remote_url: Option<String>,
    pub last_commit_date: String,
}

/// Extract all project data from source code
pub fn extract_project_data() -> Result<ProjectData> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .exec()
        .context("Failed to get cargo metadata")?;

    let root_package = metadata
        .root_package()
        .context("Failed to get root package")?;

    let dependencies = extract_dependencies(&metadata, root_package)?;
    let features = extract_features(root_package)?;
    let structure = extract_project_structure()?;
    let components = extract_components(&metadata)?;
    let git_info = extract_git_info()?;
    let license = extract_license()?;

    Ok(ProjectData {
        name: root_package.name.clone(),
        description: root_package.description.clone().unwrap_or_default(),
        version: root_package.version.to_string(),
        authors: root_package.authors.clone(),
        license,
        dependencies,
        features,
        structure,
        components,
        workspace_members: metadata
            .workspace_members
            .iter()
            .map(|id| id.to_string())
            .collect(),
        git_info,
    })
}

/// Extract dependencies from Cargo.toml
fn extract_dependencies(
    metadata: &Metadata,
    root_package: &Package,
) -> Result<HashMap<String, String>> {
    let mut dependencies = HashMap::new();

    // Extract direct dependencies
    for dep in &root_package.dependencies {
        let version = match &dep.kind {
            cargo_metadata::DependencyKind::Normal => dep.req.to_string(),
            _ => "dev".to_string(),
        };
        dependencies.insert(dep.name.clone(), version);
    }

    Ok(dependencies)
}

/// Extract features from Cargo.toml
fn extract_features(root_package: &Package) -> Result<Vec<String>> {
    let mut features = Vec::new();

    // Extract features from Cargo.toml
    for (name, _) in &root_package.features {
        features.push(name.to_string());
    }

    Ok(features)
}

/// Extract project structure from file system
fn extract_project_structure() -> Result<String> {
    let mut structure = String::new();
    let root = Path::new(".");

    fn build_tree(path: &Path, prefix: &str, is_last: bool) -> Result<String> {
        let mut result = String::new();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

        result.push_str(prefix);
        if is_last {
            result.push_str("└── ");
        } else {
            result.push_str("├── ");
        }
        result.push_str(name);

        if path.is_dir() {
            result.push_str("/\n");
            let entries = fs::read_dir(path)
                .context(format!("Failed to read directory: {:?}", path))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let binding = e.file_name();
                    let name = binding.to_string_lossy();
                    !name.starts_with('.') && name != "target" && name != "node_modules"
                })
                .collect::<Vec<_>>();

            for (i, entry) in entries.iter().enumerate() {
                let is_last = i == entries.len() - 1;
                let new_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                result.push_str(&build_tree(&entry.path(), &new_prefix, is_last)?);
            }
        } else {
            result.push_str("\n");
        }

        Ok(result)
    }

    structure.push_str(&build_tree(root, "", true)?);
    Ok(structure)
}

/// Extract component information from component directories
fn extract_components(metadata: &Metadata) -> Result<Vec<ComponentData>> {
    let mut components = Vec::new();
    let components_dir = Path::new("components");

    if !components_dir.exists() {
        return Ok(components);
    }

    let entries = fs::read_dir(components_dir).context("Failed to read components directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read components directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Try to find package info for this component
            let component_package = metadata
                .packages
                .iter()
                .find(|p| p.name == name || p.name.contains(name));

            let description = if let Some(pkg) = component_package {
                pkg.description.clone().unwrap_or_default()
            } else {
                extract_component_description(&path)?
            };

            let dependencies = if let Some(pkg) = component_package {
                pkg.dependencies.iter().map(|d| d.name.clone()).collect()
            } else {
                extract_component_dependencies(&path)?
            };

            let features = if let Some(pkg) = component_package {
                pkg.features.keys().cloned().collect()
            } else {
                Vec::new()
            };

            let has_readme = path.join("README.md").exists();
            let has_tests = path.join("tests").exists() || path.join("src").join("lib.rs").exists();

            components.push(ComponentData {
                name: name.to_string(),
                description,
                dependencies,
                features,
                path: path.to_path_buf(),
                has_readme,
                has_tests,
            });
        }
    }

    Ok(components)
}

/// Extract component description from README or Cargo.toml
fn extract_component_description(component_path: &Path) -> Result<String> {
    // Try README first
    let readme_path = component_path.join("README.md");
    if readme_path.exists() {
        let content = fs::read_to_string(&readme_path)
            .context(format!("Failed to read README: {:?}", readme_path))?;

        // Extract first paragraph as description
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with('#') {
                return Ok(first_line.trim_start_matches('#').trim().to_string());
            }
        }
    }

    // Try Cargo.toml
    let cargo_path = component_path.join("Cargo.toml");
    if cargo_path.exists() {
        let content = fs::read_to_string(&cargo_path)
            .context(format!("Failed to read Cargo.toml: {:?}", cargo_path))?;

        // Simple extraction of description
        for line in content.lines() {
            if line.trim().starts_with("description = ") {
                let desc = line
                    .split('=')
                    .nth(1)
                    .and_then(|s| s.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                    .unwrap_or("Component");
                return Ok(desc.to_string());
            }
        }
    }

    Ok("Component".to_string())
}

/// Extract component dependencies from Cargo.toml
fn extract_component_dependencies(component_path: &Path) -> Result<Vec<String>> {
    let cargo_path = component_path.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&cargo_path)
        .context(format!("Failed to read Cargo.toml: {:?}", cargo_path))?;

    let mut dependencies = Vec::new();
    let mut in_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[dependencies]" {
            in_dependencies = true;
            continue;
        }

        if in_dependencies {
            if trimmed.starts_with('[') {
                break; // New section
            }

            if let Some(dep_name) = trimmed.split('=').next() {
                let dep_name = dep_name.trim();
                if !dep_name.is_empty() && !dep_name.starts_with('#') {
                    dependencies.push(dep_name.to_string());
                }
            }
        }
    }

    Ok(dependencies)
}

/// Extract Git repository information
fn extract_git_info() -> Result<GitInfo> {
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get git branch")?;
    let branch = String::from_utf8_lossy(&branch.stdout).trim().to_string();

    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to get git commit")?;
    let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();

    let remote_url = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    let last_commit = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .context("Failed to get last commit date")?;
    let last_commit_date = String::from_utf8_lossy(&last_commit.stdout)
        .trim()
        .to_string();

    Ok(GitInfo {
        branch,
        commit,
        remote_url,
        last_commit_date,
    })
}

/// Extract license information
fn extract_license() -> Result<Option<String>> {
    let license_files = ["LICENSE", "LICENSE.txt", "LICENSE.md"];

    for license_file in &license_files {
        let path = Path::new(license_file);
        if path.exists() {
            let content = fs::read_to_string(path)
                .context(format!("Failed to read license file: {:?}", path))?;

            // Extract first line as license type
            if let Some(first_line) = content.lines().next() {
                let license = first_line.trim();
                if !license.is_empty() {
                    return Ok(Some(license.to_string()));
                }
            }
        }
    }

    Ok(None)
}
