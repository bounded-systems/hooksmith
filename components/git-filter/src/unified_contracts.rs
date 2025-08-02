use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ============================================================================
// Shared Primitives
// ============================================================================

/// SHA-1 hash regex pattern
pub static SHA1_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9a-f]{40}$").unwrap());

/// Valid filename regex pattern (allows alphanumeric, dots, underscores, hyphens, slashes)
pub static VALID_FILENAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9._/-]+$").unwrap());

/// Valid character regex pattern (basic printable ASCII + tab)
pub static VALID_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\x20-\x7E\t]$").unwrap());

// ============================================================================
// Blob Contracts
// ============================================================================

/// Contract for a single line in a blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobLineContract {
    /// The line content
    pub line: String,
}

impl BlobLineContract {
    /// Validate that all characters in the line are valid
    pub fn validate(&self) -> bool {
        self.line.chars().all(|c| {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            VALID_CHAR_RE.is_match(encoded)
        })
    }

    /// Get validation errors for this line
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (i, c) in self.line.chars().enumerate() {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            if !VALID_CHAR_RE.is_match(encoded) {
                errors.push(format!(
                    "Position {}: Invalid character {:?} (code: 0x{:x})",
                    i, c, c as u32
                ));
            }
        }

        errors
    }

    /// Get a summary of the line contract
    pub fn summary(&self) -> String {
        if self.validate() {
            format!("✅ Line '{}' valid ({} chars)", self.line, self.line.len())
        } else {
            format!(
                "❌ Line '{}' invalid: {}",
                self.line,
                self.get_errors().join(", ")
            )
        }
    }
}

/// Contract for a complete blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobContract {
    /// SHA-1 hash of the blob
    pub oid: String,
    /// Size of the blob in bytes
    pub size: usize,
    /// Lines in the blob
    pub lines: Vec<BlobLineContract>,
}

impl BlobContract {
    /// Validate the blob contract
    pub fn validate(&self) -> bool {
        SHA1_RE.is_match(&self.oid)
            && self.lines.iter().all(|l| l.validate())
            && self.size == self.lines.iter().map(|l| l.line.len()).sum::<usize>()
    }

    /// Get validation errors for this blob
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !SHA1_RE.is_match(&self.oid) {
            errors.push(format!("Invalid SHA-1 hash: {}", self.oid));
        }

        for (i, line) in self.lines.iter().enumerate() {
            if !line.validate() {
                errors.push(format!("Line {}: {}", i + 1, line.get_errors().join(", ")));
            }
        }

        let expected_size: usize = self.lines.iter().map(|l| l.line.len()).sum();
        if self.size != expected_size {
            errors.push(format!(
                "Size mismatch: expected {}, got {}",
                expected_size, self.size
            ));
        }

        errors
    }

    /// Get a summary of the blob contract
    pub fn summary(&self) -> String {
        if self.validate() {
            format!(
                "✅ Blob {} valid ({} lines, {} bytes)",
                &self.oid[..8],
                self.lines.len(),
                self.size
            )
        } else {
            format!(
                "❌ Blob {} invalid: {}",
                &self.oid[..8],
                self.get_errors().join(", ")
            )
        }
    }
}

// ============================================================================
// Tree Contracts
// ============================================================================

/// Tree mode enum with type-safe values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeMode {
    /// Regular file (non-executable)
    File = 0o100644,
    /// Executable file
    Executable = 0o100755,
    /// Directory
    Directory = 0o040000,
    /// Symbolic link
    Symlink = 0o120000,
}

impl TreeMode {
    pub fn parse_from_str(s: &str) -> Option<TreeMode> {
        match s {
            "100644" => Some(TreeMode::File),
            "100755" => Some(TreeMode::Executable),
            "040000" => Some(TreeMode::Directory),
            "120000" => Some(TreeMode::Symlink),
            _ => None,
        }
    }

    pub fn to_mode_string(&self) -> String {
        match self {
            TreeMode::File => "100644".to_string(),
            TreeMode::Executable => "100755".to_string(),
            TreeMode::Directory => "040000".to_string(),
            TreeMode::Symlink => "120000".to_string(),
        }
    }

    /// Get description of the tree mode
    pub fn description(&self) -> &'static str {
        match self {
            TreeMode::File => "Regular file (non-executable)",
            TreeMode::Executable => "Executable file",
            TreeMode::Directory => "Directory",
            TreeMode::Symlink => "Symbolic link",
        }
    }
}

/// Contract for a single tree entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntryContract {
    /// Mode as string (e.g., "100644")
    pub mode: String,
    /// SHA-1 hash of the object
    pub oid: String,
    /// Filename
    pub filename: String,
}

impl TreeEntryContract {
    /// Validate the tree entry contract
    pub fn validate(&self) -> bool {
        TreeMode::parse_from_str(&self.mode).is_some()
            && SHA1_RE.is_match(&self.oid)
            && VALID_FILENAME_RE.is_match(&self.filename)
    }

    /// Get validation errors for this tree entry
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if TreeMode::parse_from_str(&self.mode).is_none() {
            errors.push(format!("Invalid tree mode: {}", self.mode));
        }

        if !SHA1_RE.is_match(&self.oid) {
            errors.push(format!("Invalid SHA-1 hash: {}", self.oid));
        }

        if !VALID_FILENAME_RE.is_match(&self.filename) {
            errors.push(format!("Invalid filename: {}", self.filename));
        }

        errors
    }

    /// Get a summary of the tree entry contract
    pub fn summary(&self) -> String {
        if self.validate() {
            let mode = TreeMode::parse_from_str(&self.mode).unwrap();
            format!(
                "✅ Entry '{}' valid ({} -> {})",
                self.filename,
                mode.description(),
                &self.oid[..8]
            )
        } else {
            format!(
                "❌ Entry '{}' invalid: {}",
                self.filename,
                self.get_errors().join(", ")
            )
        }
    }
}

/// Contract for a complete tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeContract {
    /// Tree entries
    pub entries: Vec<TreeEntryContract>,
}

impl TreeContract {
    /// Validate the tree contract
    pub fn validate(&self) -> bool {
        self.entries.iter().all(|e| e.validate())
            && self
                .entries
                .windows(2)
                .all(|w| w[0].filename <= w[1].filename) // enforce sorting
    }

    /// Get validation errors for this tree
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.validate() {
                errors.push(format!(
                    "Entry {}: {}",
                    i + 1,
                    entry.get_errors().join(", ")
                ));
            }
        }

        // Check sorting
        for i in 0..self.entries.len().saturating_sub(1) {
            if self.entries[i].filename > self.entries[i + 1].filename {
                errors.push(format!(
                    "Entries not sorted: '{}' > '{}'",
                    self.entries[i].filename,
                    self.entries[i + 1].filename
                ));
            }
        }

        errors
    }

    /// Get a summary of the tree contract
    pub fn summary(&self) -> String {
        if self.validate() {
            format!("✅ Tree valid ({} entries)", self.entries.len())
        } else {
            format!("❌ Tree invalid: {}", self.get_errors().join(", "))
        }
    }
}

// ============================================================================
// Commit Contracts
// ============================================================================

/// Contract for a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitContract {
    /// Tree SHA-1 hash
    pub tree: String,
    /// Parent commit SHA-1 hashes
    pub parents: Vec<String>,
    /// Author information
    pub author: String,
    /// Committer information
    pub committer: String,
    /// Commit message
    pub message: String,
}

impl CommitContract {
    /// Validate the commit contract
    pub fn validate(&self) -> bool {
        SHA1_RE.is_match(&self.tree)
            && self.parents.iter().all(|p| SHA1_RE.is_match(p))
            && !self.author.trim().is_empty()
            && !self.committer.trim().is_empty()
            && !self.message.trim().is_empty()
    }

    /// Get validation errors for this commit
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !SHA1_RE.is_match(&self.tree) {
            errors.push(format!("Invalid tree hash: {}", self.tree));
        }

        for (i, parent) in self.parents.iter().enumerate() {
            if !SHA1_RE.is_match(parent) {
                errors.push(format!("Invalid parent {} hash: {}", i + 1, parent));
            }
        }

        if self.author.trim().is_empty() {
            errors.push("Empty author".to_string());
        }

        if self.committer.trim().is_empty() {
            errors.push("Empty committer".to_string());
        }

        if self.message.trim().is_empty() {
            errors.push("Empty message".to_string());
        }

        errors
    }

    /// Get a summary of the commit contract
    pub fn summary(&self) -> String {
        if self.validate() {
            format!(
                "✅ Commit {} valid ({} parents)",
                &self.tree[..8],
                self.parents.len()
            )
        } else {
            format!(
                "❌ Commit {} invalid: {}",
                &self.tree[..8],
                self.get_errors().join(", ")
            )
        }
    }
}

// ============================================================================
// Tag Contracts
// ============================================================================

/// Contract for a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagContract {
    /// Object SHA-1 hash
    pub object: String,
    /// Object type (commit/tree/blob/tag)
    pub obj_type: String,
    /// Tag name
    pub tag: String,
    /// Tagger information
    pub tagger: String,
    /// Tag message
    pub message: String,
}

impl TagContract {
    /// Validate the tag contract
    pub fn validate(&self) -> bool {
        SHA1_RE.is_match(&self.object)
            && matches!(self.obj_type.as_str(), "commit" | "tree" | "blob" | "tag")
            && !self.tag.trim().is_empty()
            && !self.tagger.trim().is_empty()
            && !self.message.trim().is_empty()
    }

    /// Get validation errors for this tag
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !SHA1_RE.is_match(&self.object) {
            errors.push(format!("Invalid object hash: {}", self.object));
        }

        if !matches!(self.obj_type.as_str(), "commit" | "tree" | "blob" | "tag") {
            errors.push(format!("Invalid object type: {}", self.obj_type));
        }

        if self.tag.trim().is_empty() {
            errors.push("Empty tag name".to_string());
        }

        if self.tagger.trim().is_empty() {
            errors.push("Empty tagger".to_string());
        }

        if self.message.trim().is_empty() {
            errors.push("Empty message".to_string());
        }

        errors
    }

    /// Get a summary of the tag contract
    pub fn summary(&self) -> String {
        if self.validate() {
            format!(
                "✅ Tag '{}' valid ({} -> {})",
                self.tag,
                self.obj_type,
                &self.object[..8]
            )
        } else {
            format!(
                "❌ Tag '{}' invalid: {}",
                self.tag,
                self.get_errors().join(", ")
            )
        }
    }
}

// ============================================================================
// Unified Git Object Contract
// ============================================================================

/// Unified enum for all Git object types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum GitObject {
    /// Blob object
    #[serde(rename = "blob")]
    Blob(BlobContract),
    /// Tree object
    #[serde(rename = "tree")]
    Tree(TreeContract),
    /// Commit object
    #[serde(rename = "commit")]
    Commit(CommitContract),
    /// Tag object
    #[serde(rename = "tag")]
    Tag(TagContract),
}

impl GitObject {
    /// Validate the Git object
    pub fn validate(&self) -> bool {
        match self {
            GitObject::Blob(blob) => blob.validate(),
            GitObject::Tree(tree) => tree.validate(),
            GitObject::Commit(commit) => commit.validate(),
            GitObject::Tag(tag) => tag.validate(),
        }
    }

    /// Get validation errors for this Git object
    pub fn get_errors(&self) -> Vec<String> {
        match self {
            GitObject::Blob(blob) => blob.get_errors(),
            GitObject::Tree(tree) => tree.get_errors(),
            GitObject::Commit(commit) => commit.get_errors(),
            GitObject::Tag(tag) => tag.get_errors(),
        }
    }

    /// Get a summary of the Git object
    pub fn summary(&self) -> String {
        match self {
            GitObject::Blob(blob) => blob.summary(),
            GitObject::Tree(tree) => tree.summary(),
            GitObject::Commit(commit) => commit.summary(),
            GitObject::Tag(tag) => tag.summary(),
        }
    }

    /// Get the object type as a string
    pub fn kind(&self) -> &'static str {
        match self {
            GitObject::Blob(_) => "blob",
            GitObject::Tree(_) => "tree",
            GitObject::Commit(_) => "commit",
            GitObject::Tag(_) => "tag",
        }
    }
}

// ============================================================================
// Unified Validator
// ============================================================================

/// Unified validator for Git objects
pub struct UnifiedValidator {
    /// Whether to validate blobs
    validate_blobs: bool,
    /// Whether to validate trees
    validate_trees: bool,
    /// Whether to validate commits
    validate_commits: bool,
    /// Whether to validate tags
    validate_tags: bool,
}

impl Default for UnifiedValidator {
    fn default() -> Self {
        Self {
            validate_blobs: true,
            validate_trees: true,
            validate_commits: true,
            validate_tags: true,
        }
    }
}

impl UnifiedValidator {
    /// Create a new unified validator
    pub fn new(
        validate_blobs: bool,
        validate_trees: bool,
        validate_commits: bool,
        validate_tags: bool,
    ) -> Self {
        Self {
            validate_blobs,
            validate_trees,
            validate_commits,
            validate_tags,
        }
    }

    /// Validate a single Git object
    pub fn validate_object(&self, obj: &GitObject) -> bool {
        match obj {
            GitObject::Blob(_) if !self.validate_blobs => true,
            GitObject::Tree(_) if !self.validate_trees => true,
            GitObject::Commit(_) if !self.validate_commits => true,
            GitObject::Tag(_) if !self.validate_tags => true,
            _ => obj.validate(),
        }
    }

    /// Validate multiple Git objects
    pub fn validate_objects(&self, objects: &[GitObject]) -> Vec<bool> {
        objects
            .iter()
            .map(|obj| self.validate_object(obj))
            .collect()
    }

    /// Get a summary of validation results
    pub fn summarize_validation(&self, objects: &[GitObject]) -> String {
        let total = objects.len();
        let valid = objects
            .iter()
            .filter(|obj| self.validate_object(obj))
            .count();
        let invalid = total - valid;

        let mut type_counts = std::collections::HashMap::new();
        for obj in objects {
            *type_counts.entry(obj.kind()).or_insert(0) += 1;
        }

        let type_summary: Vec<String> = type_counts
            .iter()
            .map(|(kind, count)| format!("{} {}", count, kind))
            .collect();

        format!(
            "Git objects: {} total ({} valid, {} invalid) - {}",
            total,
            valid,
            invalid,
            type_summary.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_line_contract() {
        let valid_line = BlobLineContract {
            line: "Hello, World!".to_string(),
        };
        assert!(valid_line.validate());

        let invalid_line = BlobLineContract {
            line: "Hello\x00World!".to_string(),
        };
        assert!(!invalid_line.validate());
        assert!(!invalid_line.get_errors().is_empty());
    }

    #[test]
    fn test_blob_contract() {
        let blob = BlobContract {
            oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            size: 13,
            lines: vec![
                BlobLineContract {
                    line: "Hello, ".to_string(),
                },
                BlobLineContract {
                    line: "World!".to_string(),
                },
            ],
        };
        assert!(blob.validate());

        let invalid_blob = BlobContract {
            oid: "invalid".to_string(),
            size: 0,
            lines: vec![],
        };
        assert!(!invalid_blob.validate());
    }

    #[test]
    fn test_tree_mode() {
        assert_eq!(TreeMode::parse_from_str("100644"), Some(TreeMode::File));
        assert_eq!(TreeMode::parse_from_str("100755"), Some(TreeMode::Executable));
        assert_eq!(TreeMode::parse_from_str("040000"), Some(TreeMode::Directory));
        assert_eq!(TreeMode::parse_from_str("invalid"), None);
    }

    #[test]
    fn test_tree_entry_contract() {
        let entry = TreeEntryContract {
            mode: "100644".to_string(),
            oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            filename: "README.md".to_string(),
        };
        assert!(entry.validate());

        let invalid_entry = TreeEntryContract {
            mode: "invalid".to_string(),
            oid: "invalid".to_string(),
            filename: "file\x00name.txt".to_string(),
        };
        assert!(!invalid_entry.validate());
    }

    #[test]
    fn test_tree_contract() {
        let tree = TreeContract {
            entries: vec![
                TreeEntryContract {
                    mode: "100644".to_string(),
                    oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                    filename: "README.md".to_string(),
                },
                TreeEntryContract {
                    mode: "100755".to_string(),
                    oid: "b2c3d4e5f6789012345678901234567890abcde".to_string(),
                    filename: "script.sh".to_string(),
                },
            ],
        };
        assert!(tree.validate());
    }

    #[test]
    fn test_commit_contract() {
        let commit = CommitContract {
            tree: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            parents: vec!["b2c3d4e5f6789012345678901234567890abcde".to_string()],
            author: "John Doe <john@example.com>".to_string(),
            committer: "John Doe <john@example.com>".to_string(),
            message: "Initial commit".to_string(),
        };
        assert!(commit.validate());
    }

    #[test]
    fn test_tag_contract() {
        let tag = TagContract {
            object: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            obj_type: "commit".to_string(),
            tag: "v1.0.0".to_string(),
            tagger: "John Doe <john@example.com>".to_string(),
            message: "Release v1.0.0".to_string(),
        };
        assert!(tag.validate());
    }

    #[test]
    fn test_git_object_enum() {
        let blob = GitObject::Blob(BlobContract {
            oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            size: 5,
            lines: vec![BlobLineContract {
                line: "Hello".to_string(),
            }],
        });
        assert!(blob.validate());
        assert_eq!(blob.kind(), "blob");

        let tree = GitObject::Tree(TreeContract {
            entries: vec![TreeEntryContract {
                mode: "100644".to_string(),
                oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                filename: "README.md".to_string(),
            }],
        });
        assert!(tree.validate());
        assert_eq!(tree.kind(), "tree");
    }

    #[test]
    fn test_unified_validator() {
        let validator = UnifiedValidator::default();

        let objects = vec![
            GitObject::Blob(BlobContract {
                oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                size: 5,
                lines: vec![BlobLineContract {
                    line: "Hello".to_string(),
                }],
            }),
            GitObject::Tree(TreeContract {
                entries: vec![TreeEntryContract {
                    mode: "100644".to_string(),
                    oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                    filename: "README.md".to_string(),
                }],
            }),
        ];

        let results = validator.validate_objects(&objects);
        assert_eq!(results, vec![true, true]);

        let summary = validator.summarize_validation(&objects);
        assert!(summary.contains("2 total"));
        assert!(summary.contains("1 blob"));
        assert!(summary.contains("1 tree"));
    }
}
