use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Path mapping entry representing a blob or tree with its path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathMappingEntry {
    pub kind: String, // "blob" or "tree"
    pub oid: String,  // Git object ID (SHA-1 or SHA-256)
    pub path: String, // File path relative to repo root
}

/// Path mapping system that uses Git notes to store OID-to-path mappings
pub struct PathMapping {
    notes_ref: String,
}

impl PathMapping {
    /// Create a new path mapping system with the specified notes reference
    pub fn new(notes_ref: Option<String>) -> Self {
        let notes_ref = notes_ref.unwrap_or_else(|| "refs/notes/hooksmith-paths".to_string());
        Self { notes_ref }
    }

    /// Build a path map for a specific commit
    pub fn build_path_map(&self, commit: &str) -> Result<Vec<PathMappingEntry>> {
        let mut entries = Vec::new();

        // Get blobs (OID -> path). One row per occurrence (multi-path blobs get multiple rows)
        let blobs_output = Command::new("git")
            .args(["rev-list", "--objects", commit])
            .output()
            .context("Failed to get blob objects")?;

        let blobs_str = String::from_utf8_lossy(&blobs_output.stdout);
        for line in blobs_str.lines() {
            if let Some((oid, path)) = line.split_once(' ') {
                if !path.is_empty() {
                    entries.push(PathMappingEntry {
                        kind: "blob".to_string(),
                        oid: oid.to_string(),
                        path: path.to_string(),
                    });
                }
            }
        }

        // Get trees (tree OID -> path of that subtree)
        let trees_output = Command::new("git")
            .args([
                "ls-tree",
                "-r",
                "-d",
                "--full-tree",
                "--format=%(objectname) %(path)",
                commit,
            ])
            .output()
            .context("Failed to get tree objects")?;

        let trees_str = String::from_utf8_lossy(&trees_output.stdout);
        for line in trees_str.lines() {
            if let Some((oid, path)) = line.split_once(' ') {
                if !path.is_empty() {
                    entries.push(PathMappingEntry {
                        kind: "tree".to_string(),
                        oid: oid.to_string(),
                        path: path.to_string(),
                    });
                }
            }
        }

        // Sort entries for deterministic output
        entries.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then(a.oid.cmp(&b.oid))
                .then(a.path.cmp(&b.path))
        });

        Ok(entries)
    }

    /// Convert path map entries to tab-separated format
    pub fn entries_to_tsv(&self, entries: &[PathMappingEntry]) -> String {
        entries
            .iter()
            .map(|entry| format!("{}\t{}\t{}", entry.kind, entry.oid, entry.path))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse tab-separated format back to path map entries
    pub fn tsv_to_entries(&self, tsv: &str) -> Result<Vec<PathMappingEntry>> {
        let mut entries = Vec::new();
        for line in tsv.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                entries.push(PathMappingEntry {
                    kind: parts[0].to_string(),
                    oid: parts[1].to_string(),
                    path: parts[2].to_string(),
                });
            }
        }
        Ok(entries)
    }

    /// Attach path map as a Git note to a commit
    pub fn attach_path_map(&self, commit: &str, entries: &[PathMappingEntry]) -> Result<()> {
        let tsv_content = self.entries_to_tsv(entries);

        // Create a temporary file with the content
        let temp_file =
            tempfile::NamedTempFile::new().context("Failed to create temporary file")?;
        std::fs::write(&temp_file, tsv_content).context("Failed to write to temporary file")?;

        // Add the note using the temporary file
        let status = Command::new("git")
            .args([
                "notes",
                "--ref",
                &self.notes_ref,
                "add",
                "-F",
                temp_file.path().to_str().unwrap(),
                commit,
            ])
            .status()
            .context("Failed to add Git note")?;

        if !status.success() {
            anyhow::bail!("Failed to add Git note: exit code {}", status);
        }

        Ok(())
    }

    /// Retrieve path map from a Git note
    pub fn get_path_map(&self, commit: &str) -> Result<Vec<PathMappingEntry>> {
        let output = Command::new("git")
            .args(["notes", "--ref", &self.notes_ref, "show", commit])
            .output()
            .context("Failed to get Git note")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get Git note: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let content = String::from_utf8_lossy(&output.stdout);
        self.tsv_to_entries(&content)
    }

    /// Create a bundle that includes both the commit and its path mapping notes
    pub fn create_bundle(&self, commit: &str, bundle_path: &Path, bundle_name: &str) -> Result<()> {
        // Ensure the bundle directory exists
        if let Some(parent) = bundle_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create bundle directory")?;
        }

        let bundle_file = bundle_path.join(bundle_name);
        let bundle_file_str = bundle_file.to_str().unwrap();

        // Create bundle with commit and notes ref
        let status = Command::new("git")
            .args(["bundle", "create", bundle_file_str, commit, &self.notes_ref])
            .status()
            .context("Failed to create Git bundle")?;

        if !status.success() {
            anyhow::bail!("Failed to create Git bundle: exit code {}", status);
        }

        println!("Created bundle: {}", bundle_file_str);
        Ok(())
    }

    /// Clone a bundle and extract the path mapping
    pub fn clone_bundle_and_extract_path_map(
        &self,
        bundle_path: &Path,
        clone_dir: &Path,
    ) -> Result<Vec<PathMappingEntry>> {
        // Clone the bundle
        let status = Command::new("git")
            .args([
                "clone",
                bundle_path.to_str().unwrap(),
                clone_dir.to_str().unwrap(),
            ])
            .status()
            .context("Failed to clone bundle")?;

        if !status.success() {
            anyhow::bail!("Failed to clone bundle: exit code {}", status);
        }

        // Get the HEAD commit
        let output = Command::new("git")
            .current_dir(clone_dir)
            .args(["rev-parse", "HEAD"])
            .output()
            .context("Failed to get HEAD commit")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get HEAD commit: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Extract path map from the note
        self.get_path_map(&commit)
    }

    /// Build a lookup map from OID to paths for efficient querying
    pub fn build_lookup_map(&self, entries: &[PathMappingEntry]) -> HashMap<String, Vec<String>> {
        let mut lookup = HashMap::new();
        for entry in entries {
            lookup
                .entry(entry.oid.clone())
                .or_insert_with(Vec::new)
                .push(entry.path.clone());
        }
        lookup
    }

    /// Get all paths for a given OID
    pub fn get_paths_for_oid(
        &self,
        lookup_map: &HashMap<String, Vec<String>>,
        oid: &str,
    ) -> Vec<String> {
        lookup_map.get(oid).cloned().unwrap_or_default()
    }

    /// Validate that a path map is consistent with the current repository state
    pub fn validate_path_map(&self, commit: &str, entries: &[PathMappingEntry]) -> Result<bool> {
        // Rebuild the path map for the current commit
        let current_entries = self.build_path_map(commit)?;

        // Compare the entries
        if entries.len() != current_entries.len() {
            return Ok(false);
        }

        // Sort both lists for comparison
        let mut sorted_entries = entries.to_vec();
        let mut sorted_current = current_entries;

        sorted_entries.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then(a.oid.cmp(&b.oid))
                .then(a.path.cmp(&b.path))
        });
        sorted_current.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then(a.oid.cmp(&b.oid))
                .then(a.path.cmp(&b.path))
        });

        Ok(sorted_entries == sorted_current)
    }
}

/// Run path mapping command
pub async fn run_path_mapping_command(command: crate::PathMappingCommands) -> Result<()> {
    use crate::PathMappingCommands;

    match command {
        PathMappingCommands::Build {
            commit,
            notes_ref,
            output,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let entries = mapping.build_path_map(&commit)?;

            let tsv_content = mapping.entries_to_tsv(&entries);

            if let Some(output_path) = output {
                std::fs::write(&output_path, tsv_content).context("Failed to write output file")?;
                println!("Path map written to: {}", output_path);
            } else {
                println!("{}", tsv_content);
            }

            println!(
                "Built path map with {} entries for commit: {}",
                entries.len(),
                commit
            );
        }

        PathMappingCommands::Attach {
            commit,
            input,
            notes_ref,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let content = std::fs::read_to_string(&input).context("Failed to read input file")?;

            let entries = mapping.tsv_to_entries(&content)?;
            mapping.attach_path_map(&commit, &entries)?;

            println!(
                "Attached path map with {} entries to commit: {}",
                entries.len(),
                commit
            );
        }

        PathMappingCommands::Get {
            commit,
            notes_ref,
            output,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let entries = mapping.get_path_map(&commit)?;

            let tsv_content = mapping.entries_to_tsv(&entries);

            if let Some(output_path) = output {
                std::fs::write(&output_path, tsv_content).context("Failed to write output file")?;
                println!("Path map written to: {}", output_path);
            } else {
                println!("{}", tsv_content);
            }

            println!(
                "Retrieved path map with {} entries from commit: {}",
                entries.len(),
                commit
            );
        }

        PathMappingCommands::Bundle {
            commit,
            bundle_dir,
            bundle_name,
            notes_ref,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let bundle_path = Path::new(&bundle_dir);

            mapping.create_bundle(&commit, bundle_path, &bundle_name)?;

            println!("Created bundle: {}/{}", bundle_dir, bundle_name);
        }

        PathMappingCommands::Extract {
            bundle_path,
            clone_dir,
            notes_ref,
            output,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let bundle_path = Path::new(&bundle_path);
            let clone_dir = Path::new(&clone_dir);

            let entries = mapping.clone_bundle_and_extract_path_map(bundle_path, clone_dir)?;

            let tsv_content = mapping.entries_to_tsv(&entries);

            if let Some(output_path) = output {
                std::fs::write(&output_path, tsv_content).context("Failed to write output file")?;
                println!("Path map written to: {}", output_path);
            } else {
                println!("{}", tsv_content);
            }

            println!(
                "Extracted path map with {} entries from bundle",
                entries.len()
            );
        }

        PathMappingCommands::Validate {
            commit,
            input,
            notes_ref,
        } => {
            let mapping = PathMapping::new(Some(notes_ref));
            let content = std::fs::read_to_string(&input).context("Failed to read input file")?;

            let entries = mapping.tsv_to_entries(&content)?;
            let is_valid = mapping.validate_path_map(&commit, &entries)?;

            if is_valid {
                println!("✓ Path map is valid for commit: {}", commit);
            } else {
                println!("✗ Path map is invalid for commit: {}", commit);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entries_to_tsv() {
        let mapping = PathMapping::new(None);
        let entries = vec![
            PathMappingEntry {
                kind: "blob".to_string(),
                oid: "abc123".to_string(),
                path: "src/main.rs".to_string(),
            },
            PathMappingEntry {
                kind: "tree".to_string(),
                oid: "def456".to_string(),
                path: "src".to_string(),
            },
        ];

        let tsv = mapping.entries_to_tsv(&entries);
        let expected = "blob\tabc123\tsrc/main.rs\ntree\tdef456\tsrc";
        assert_eq!(tsv, expected);
    }

    #[test]
    fn test_tsv_to_entries() {
        let mapping = PathMapping::new(None);
        let tsv = "blob\tabc123\tsrc/main.rs\ntree\tdef456\tsrc";
        let entries = mapping.tsv_to_entries(tsv).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, "blob");
        assert_eq!(entries[0].oid, "abc123");
        assert_eq!(entries[0].path, "src/main.rs");
        assert_eq!(entries[1].kind, "tree");
        assert_eq!(entries[1].oid, "def456");
        assert_eq!(entries[1].path, "src");
    }

    #[test]
    fn test_build_lookup_map() {
        let mapping = PathMapping::new(None);
        let entries = vec![
            PathMappingEntry {
                kind: "blob".to_string(),
                oid: "abc123".to_string(),
                path: "src/main.rs".to_string(),
            },
            PathMappingEntry {
                kind: "blob".to_string(),
                oid: "abc123".to_string(),
                path: "src/lib.rs".to_string(),
            },
        ];

        let lookup = mapping.build_lookup_map(&entries);
        let paths = mapping.get_paths_for_oid(&lookup, "abc123");

        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"src/main.rs".to_string()));
        assert!(paths.contains(&"src/lib.rs".to_string()));
    }
}
