use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Git object types for snapshot categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    Stash,
    Worktree,
    Index,
    Reflog,
    Remote,
    Config,
    Unreachable,
}

/// Git snapshot entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshotEntry {
    pub object_type: GitObjectType,
    pub sha: Option<String>,
    pub path: Option<String>,
    pub ref_name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub raw_line: String,
}

/// Git snapshot containing all repository state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub entries: Vec<GitSnapshotEntry>,
    pub summary: GitSnapshotSummary,
}

/// Summary statistics for the Git snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshotSummary {
    pub commits: usize,
    pub trees: usize,
    pub blobs: usize,
    pub tags: usize,
    pub stashes: usize,
    pub worktrees: usize,
    pub index_entries: usize,
    pub reflog_entries: usize,
    pub remotes: usize,
    pub config_entries: usize,
    pub unreachable: usize,
    pub total_objects: usize,
}

impl GitSnapshotSummary {
    pub fn new() -> Self {
        Self {
            commits: 0,
            trees: 0,
            blobs: 0,
            tags: 0,
            stashes: 0,
            worktrees: 0,
            index_entries: 0,
            reflog_entries: 0,
            remotes: 0,
            config_entries: 0,
            unreachable: 0,
            total_objects: 0,
        }
    }

    pub fn update_counts(&mut self, entries: &[GitSnapshotEntry]) {
        self.commits = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Commit)
            .count();
        self.trees = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Tree)
            .count();
        self.blobs = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Blob)
            .count();
        self.tags = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Tag)
            .count();
        self.stashes = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Stash)
            .count();
        self.worktrees = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Worktree)
            .count();
        self.index_entries = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Index)
            .count();
        self.reflog_entries = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Reflog)
            .count();
        self.remotes = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Remote)
            .count();
        self.config_entries = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Config)
            .count();
        self.unreachable = entries
            .iter()
            .filter(|e| e.object_type == GitObjectType::Unreachable)
            .count();
        self.total_objects = entries.len();
    }
}

/// Git snapshot collector with comprehensive state capture
pub struct GitSnapshotCollector;

impl GitSnapshotCollector {
    /// Collect all Git objects (commits, trees, blobs) using efficient batch processing
    pub fn collect_objects() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        // Use the efficient pipeline: git rev-list --all --objects | git cat-file --batch-check
        let rev_list = Command::new("git")
            .args(["rev-list", "--all", "--objects"])
            .output()?;

        if !rev_list.status.success() {
            return Err(format!(
                "git rev-list failed: {}",
                String::from_utf8_lossy(&rev_list.stderr)
            )
            .into());
        }

        let mut cat_file = Command::new("git")
            .args([
                "cat-file",
                "--batch-check=%(objectname) %(objecttype) %(rest)",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        // Write rev-list output to cat-file stdin
        if let Some(stdin) = cat_file.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(&rev_list.stdout)?;
        }

        let cat_output = cat_file.wait_with_output()?;
        if !cat_output.status.success() {
            return Err(format!(
                "git cat-file failed: {}",
                String::from_utf8_lossy(&cat_output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(cat_output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let sha = parts[0].to_string();
                let object_type_str = parts[1];
                let rest = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    String::new()
                };

                let object_type = match object_type_str {
                    "commit" => GitObjectType::Commit,
                    "tree" => GitObjectType::Tree,
                    "blob" => GitObjectType::Blob,
                    "tag" => GitObjectType::Tag,
                    _ => GitObjectType::Blob, // Default to blob
                };

                // Parse path from rest if available
                let path = if !rest.is_empty() {
                    Some(rest.trim().to_string())
                } else {
                    None
                };

                entries.push(GitSnapshotEntry {
                    object_type,
                    sha: Some(sha),
                    path,
                    ref_name: None,
                    metadata: HashMap::new(),
                    raw_line: line.to_string(),
                });
            }
        }

        Ok(entries)
    }

    /// Collect all refs (branches, tags, remotes)
    pub fn collect_refs() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["show-ref", "--head"]).output()?;

        if !output.status.success() {
            return Err(format!(
                "git show-ref failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let sha = parts[0].to_string();
                let ref_name = parts[1].to_string();

                let object_type = if ref_name.starts_with("refs/tags/") {
                    GitObjectType::Tag
                } else {
                    GitObjectType::Commit
                };

                entries.push(GitSnapshotEntry {
                    object_type,
                    sha: Some(sha),
                    path: None,
                    ref_name: Some(ref_name),
                    metadata: HashMap::new(),
                    raw_line: line.to_string(),
                });
            }
        }

        Ok(entries)
    }

    /// Collect stashes
    pub fn collect_stashes() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["stash", "list", "--pretty=format:%H %gd %gs"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "git stash list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let sha = parts[0].to_string();
                let stash_ref = parts[1].to_string();
                let message = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    String::new()
                };

                let mut metadata = HashMap::new();
                metadata.insert("stash_ref".to_string(), stash_ref);
                metadata.insert("message".to_string(), message);

                entries.push(GitSnapshotEntry {
                    object_type: GitObjectType::Stash,
                    sha: Some(sha),
                    path: None,
                    ref_name: None,
                    metadata,
                    raw_line: line.to_string(),
                });
            }
        }

        Ok(entries)
    }

    /// Collect worktrees
    pub fn collect_worktrees() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "git worktree list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();
        let mut current_metadata = HashMap::new();
        let mut current_path: Option<String> = None;

        for line in stdout.lines() {
            if line.trim().is_empty() {
                // End of worktree entry
                if let Some(path) = current_path {
                    entries.push(GitSnapshotEntry {
                        object_type: GitObjectType::Worktree,
                        sha: None,
                        path: Some(path.clone()),
                        ref_name: None,
                        metadata: current_metadata.clone(),
                        raw_line: format!("worktree: {}", path),
                    });
                }
                current_metadata.clear();
                current_path = None;
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let key = parts[0];
                let value = parts[1];

                match key {
                    "worktree" => {
                        current_path = Some(value.to_string());
                    }
                    "HEAD" => {
                        current_metadata.insert("head".to_string(), value.to_string());
                    }
                    "branch" => {
                        current_metadata.insert("branch".to_string(), value.to_string());
                    }
                    _ => {
                        current_metadata.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        // Handle last worktree if exists
        if let Some(path) = current_path {
            entries.push(GitSnapshotEntry {
                object_type: GitObjectType::Worktree,
                sha: None,
                path: Some(path.clone()),
                ref_name: None,
                metadata: current_metadata,
                raw_line: format!("worktree: {}", path),
            });
        }

        Ok(entries)
    }

    /// Collect index entries
    pub fn collect_index() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["ls-files", "--stage"]).output()?;

        if !output.status.success() {
            return Err(format!(
                "git ls-files --stage failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let mode_sha_stage = parts[0];
                let path = parts[1];

                let mode_sha_parts: Vec<&str> = mode_sha_stage.split(' ').collect();
                if mode_sha_parts.len() >= 3 {
                    let mode = mode_sha_parts[0];
                    let sha = mode_sha_parts[1];
                    let stage = mode_sha_parts[2];

                    let mut metadata = HashMap::new();
                    metadata.insert("mode".to_string(), mode.to_string());
                    metadata.insert("stage".to_string(), stage.to_string());

                    entries.push(GitSnapshotEntry {
                        object_type: GitObjectType::Index,
                        sha: Some(sha.to_string()),
                        path: Some(path.to_string()),
                        ref_name: None,
                        metadata,
                        raw_line: line.to_string(),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Collect reflogs
    pub fn collect_reflogs() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["reflog", "--all", "--pretty=format:%H %gD %gs"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "git reflog failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let sha = parts[0].to_string();
                let ref_name = parts[1].to_string();
                let message = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    String::new()
                };

                let mut metadata = HashMap::new();
                metadata.insert("message".to_string(), message);

                entries.push(GitSnapshotEntry {
                    object_type: GitObjectType::Reflog,
                    sha: Some(sha),
                    path: None,
                    ref_name: Some(ref_name),
                    metadata,
                    raw_line: line.to_string(),
                });
            }
        }

        Ok(entries)
    }

    /// Collect remotes
    pub fn collect_remotes() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["remote", "-v"]).output()?;

        if !output.status.success() {
            return Err(format!(
                "git remote failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                let direction = parts[2].to_string();

                let mut metadata = HashMap::new();
                metadata.insert("url".to_string(), url);
                metadata.insert("direction".to_string(), direction);

                entries.push(GitSnapshotEntry {
                    object_type: GitObjectType::Remote,
                    sha: None,
                    path: Some(name),
                    ref_name: None,
                    metadata,
                    raw_line: line.to_string(),
                });
            }
        }

        Ok(entries)
    }

    /// Collect configuration
    pub fn collect_config() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["config", "--list", "--show-origin"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "git config failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() >= 2 {
                let origin = parts[0].to_string();
                let key_value = parts[1].to_string();

                let kv_parts: Vec<&str> = key_value.splitn(2, '=').collect();
                if kv_parts.len() >= 2 {
                    let key = kv_parts[0].to_string();
                    let value = kv_parts[1].to_string();

                    let mut metadata = HashMap::new();
                    metadata.insert("origin".to_string(), origin);
                    metadata.insert("value".to_string(), value);

                    entries.push(GitSnapshotEntry {
                        object_type: GitObjectType::Config,
                        sha: None,
                        path: Some(key),
                        ref_name: None,
                        metadata,
                        raw_line: line.to_string(),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Collect unreachable objects (optional)
    pub fn collect_unreachable() -> Result<Vec<GitSnapshotEntry>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["fsck", "--no-reflogs", "--unreachable", "--full"])
            .output()?;

        // Note: This command may fail if there are no unreachable objects
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse fsck output format
            if line.contains("unreachable") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let sha = parts[1].to_string();
                    let object_type_str = parts[0];

                    let object_type = match object_type_str {
                        "commit" => GitObjectType::Commit,
                        "tree" => GitObjectType::Tree,
                        "blob" => GitObjectType::Blob,
                        "tag" => GitObjectType::Tag,
                        _ => GitObjectType::Unreachable,
                    };

                    entries.push(GitSnapshotEntry {
                        object_type,
                        sha: Some(sha),
                        path: None,
                        ref_name: None,
                        metadata: HashMap::new(),
                        raw_line: line.to_string(),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Create a complete Git snapshot
    pub fn create_snapshot(
        include_unreachable: bool,
    ) -> Result<GitSnapshot, Box<dyn std::error::Error>> {
        let mut all_entries = Vec::new();

        // Collect all Git state
        all_entries.extend(Self::collect_objects()?);
        all_entries.extend(Self::collect_refs()?);
        all_entries.extend(Self::collect_stashes()?);
        all_entries.extend(Self::collect_worktrees()?);
        all_entries.extend(Self::collect_index()?);
        all_entries.extend(Self::collect_reflogs()?);
        all_entries.extend(Self::collect_remotes()?);
        all_entries.extend(Self::collect_config()?);

        if include_unreachable {
            all_entries.extend(Self::collect_unreachable()?);
        }

        let mut summary = GitSnapshotSummary::new();
        summary.update_counts(&all_entries);

        Ok(GitSnapshot {
            entries: all_entries,
            summary,
        })
    }

    /// Get object type for a SHA
    fn get_object_type(sha: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["cat-file", "-t", sha]).output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Ok("blob".to_string()) // Default to blob if we can't determine
        }
    }
}

/// Format snapshot as line-based output
pub fn format_snapshot_line_based(snapshot: &GitSnapshot) -> String {
    let mut output = String::new();

    // Group entries by type
    let mut by_type: HashMap<GitObjectType, Vec<&GitSnapshotEntry>> = HashMap::new();
    for entry in &snapshot.entries {
        by_type
            .entry(entry.object_type.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    // Output each type with header
    for (object_type, entries) in by_type {
        output.push_str(&format!(
            "== {} ==\n",
            format!("{:?}", object_type).to_uppercase()
        ));
        for entry in entries {
            output.push_str(&entry.raw_line);
            output.push('\n');
        }
        output.push('\n');
    }

    // Add summary
    output.push_str("== SUMMARY ==\n");
    output.push_str(&format!(
        "Total objects: {}\n",
        snapshot.summary.total_objects
    ));
    output.push_str(&format!("Commits: {}\n", snapshot.summary.commits));
    output.push_str(&format!("Trees: {}\n", snapshot.summary.trees));
    output.push_str(&format!("Blobs: {}\n", snapshot.summary.blobs));
    output.push_str(&format!("Tags: {}\n", snapshot.summary.tags));
    output.push_str(&format!("Stashes: {}\n", snapshot.summary.stashes));
    output.push_str(&format!("Worktrees: {}\n", snapshot.summary.worktrees));
    output.push_str(&format!(
        "Index entries: {}\n",
        snapshot.summary.index_entries
    ));
    output.push_str(&format!(
        "Reflog entries: {}\n",
        snapshot.summary.reflog_entries
    ));
    output.push_str(&format!("Remotes: {}\n", snapshot.summary.remotes));
    output.push_str(&format!(
        "Config entries: {}\n",
        snapshot.summary.config_entries
    ));
    if snapshot.summary.unreachable > 0 {
        output.push_str(&format!("Unreachable: {}\n", snapshot.summary.unreachable));
    }

    output
}
