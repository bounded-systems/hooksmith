use serde::{Deserialize, Serialize};

/// Tree mode contract - represents allowed Git tree modes (restricted set)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeMode {
    /// Regular file (non-executable)
    RegularFile = 0o100644,
    /// Regular file (executable)
    ExecutableFile = 0o100755,
    /// Tree (directory)
    Tree = 0o040000,
}

impl TreeMode {
    /// Create a TreeMode from a string representation (restricted set)
    pub fn from_str(mode: &str) -> Option<Self> {
        match mode {
            "100644" => Some(TreeMode::RegularFile),
            "100755" => Some(TreeMode::ExecutableFile),
            "040000" => Some(TreeMode::Tree),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            TreeMode::RegularFile => "100644".to_string(),
            TreeMode::ExecutableFile => "100755".to_string(),
            TreeMode::Tree => "040000".to_string(),
        }
    }

    /// Get the description of this mode
    pub fn description(&self) -> &'static str {
        match self {
            TreeMode::RegularFile => "Regular file (non-executable)",
            TreeMode::ExecutableFile => "Regular file (executable)",
            TreeMode::Tree => "Tree (directory)",
        }
    }

    /// Check if this mode represents a tree (directory)
    pub fn is_tree(&self) -> bool {
        matches!(self, TreeMode::Tree)
    }

    /// Check if this mode represents a blob (file)
    pub fn is_blob(&self) -> bool {
        !self.is_tree()
    }

    /// Get the object type for this mode
    pub fn object_type(&self) -> TreeObjectType {
        if self.is_tree() {
            TreeObjectType::Tree
        } else {
            TreeObjectType::Blob
        }
    }
}

/// Tree object type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeObjectType {
    /// Blob (file)
    Blob,
    /// Tree (directory)
    Tree,
}

/// Tree entry contract - represents a single entry in a Git tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntryContract {
    /// Tree mode (permissions and type)
    pub mode: TreeMode,
    /// Filename (must not be empty)
    pub filename: String,
    /// Object ID (SHA-1 hash)
    pub object_id: String,
    /// Object type (explicit, must match mode)
    pub object_type: TreeObjectType,
    /// Whether the entry is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl TreeEntryContract {
    /// Create a new tree entry contract
    pub fn new(mode: &str, filename: String, object_id: String) -> Self {
        let (mode_enum, valid, errors) = Self::validate_entry(mode, &filename, &object_id);
        let object_type = mode_enum.object_type();

        Self {
            mode: mode_enum,
            filename,
            object_id,
            object_type,
            valid,
            errors,
        }
    }

    /// Create a new tree entry contract with explicit type validation
    pub fn new_with_type(mode: &str, filename: String, object_id: String, object_type: TreeObjectType) -> Self {
        let (mode_enum, mut valid, mut errors) = Self::validate_entry(mode, &filename, &object_id);
        
        // Validate that the explicit type matches the mode
        let expected_type = mode_enum.object_type();
        if object_type != expected_type {
            valid = false;
            errors.push(format!(
                "Type mismatch: expected {:?} for mode {}, got {:?}",
                expected_type, mode, object_type
            ));
        }

        Self {
            mode: mode_enum,
            filename,
            object_id,
            object_type,
            valid,
            errors,
        }
    }

    /// Validate a tree entry
    fn validate_entry(
        mode: &str,
        filename: &str,
        object_id: &str,
    ) -> (TreeMode, bool, Vec<String>) {
        let mut errors = Vec::new();

        // Validate mode
        let mode_enum = match TreeMode::from_str(mode) {
            Some(m) => m,
            None => {
                errors.push(format!("Invalid tree mode: {}", mode));
                TreeMode::RegularFile // Default fallback
            }
        };

        // Validate filename
        if filename.is_empty() {
            errors.push("Filename cannot be empty".to_string());
        }

        // Validate object ID (SHA-1 format)
        if !Self::is_valid_sha1(object_id) {
            errors.push(format!("Invalid object ID format: {}", object_id));
        }

        let valid = errors.is_empty();
        (mode_enum, valid, errors)
    }

    /// Check if object ID is valid SHA-1 format
    fn is_valid_sha1(object_id: &str) -> bool {
        object_id.len() == 40 && object_id.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get a summary of the tree entry contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Entry '{}' ({}) -> {}",
                self.filename,
                self.mode.description(),
                &self.object_id[..8]
            )
        } else {
            format!(
                "❌ Entry '{}' invalid: {}",
                self.filename,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the entry is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the mode as a string
    pub fn mode_string(&self) -> String {
        self.mode.to_string()
    }
}

/// Tree object contract - represents a complete Git tree (flat list of entries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeObjectContract {
    /// Tree entries (each validated independently)
    pub entries: Vec<TreeEntryContract>,
}

impl TreeObjectContract {
    /// Create a new tree object contract
    pub fn new(id: String, entries: Vec<TreeEntryContract>) -> Self {
        let (valid, errors) = Self::validate_tree(&entries);

        Self {
            id,
            entries,
            valid,
            errors,
        }
    }

    /// Validate a tree object
    fn validate_tree(entries: &[TreeEntryContract]) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if all entries are valid
        let invalid_entries: Vec<_> = entries.iter().filter(|e| !e.is_valid()).collect();
        if !invalid_entries.is_empty() {
            errors.push(format!("{} invalid entries found", invalid_entries.len()));
            for entry in invalid_entries {
                errors.extend(
                    entry
                        .errors
                        .iter()
                        .map(|e| format!("Entry '{}': {}", entry.filename, e)),
                );
            }
        }

        // Check for duplicate filenames
        let mut filenames: Vec<_> = entries.iter().map(|e| &e.filename).collect();
        filenames.sort();
        filenames.dedup();
        if filenames.len() != entries.len() {
            errors.push("Duplicate filenames found in tree".to_string());
        }

        // Check if entries are sorted (Git requirement)
        let mut sorted_entries: Vec<_> = entries.iter().map(|e| &e.filename).collect();
        sorted_entries.sort();
        let current_entries: Vec<_> = entries.iter().map(|e| &e.filename).collect();
        if sorted_entries != current_entries {
            errors.push("Tree entries are not sorted by filename".to_string());
        }

        let valid = errors.is_empty();
        (valid, errors)
    }

    /// Get a summary of the tree object contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Tree {} valid ({} entries)",
                &self.id[..self.id.len().min(8)],
                self.entries.len()
            )
        } else {
            format!(
                "❌ Tree {} invalid: {}",
                &self.id[..self.id.len().min(8)],
                self.errors.join(", ")
            )
        }
    }

    /// Check if the tree is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get entries by type
    pub fn get_entries_by_type(&self, object_type: TreeObjectType) -> Vec<&TreeEntryContract> {
        self.entries
            .iter()
            .filter(|e| e.object_type == object_type)
            .collect()
    }

    /// Get blob entries
    pub fn get_blob_entries(&self) -> Vec<&TreeEntryContract> {
        self.get_entries_by_type(TreeObjectType::Blob)
    }

    /// Get tree entries
    pub fn get_tree_entries(&self) -> Vec<&TreeEntryContract> {
        self.get_entries_by_type(TreeObjectType::Tree)
    }

    /// Find entry by filename
    pub fn find_entry(&self, filename: &str) -> Option<&TreeEntryContract> {
        self.entries.iter().find(|e| e.filename == filename)
    }
}

/// Tree validator that processes Git tree objects
pub struct TreeValidator {
    /// Whether to validate individual entries
    validate_entries: bool,
    /// Whether to check for duplicate filenames
    check_duplicates: bool,
    /// Whether to enforce sorting
    enforce_sorting: bool,
}

impl Default for TreeValidator {
    fn default() -> Self {
        Self {
            validate_entries: true,
            check_duplicates: true,
            enforce_sorting: true,
        }
    }
}

impl TreeValidator {
    /// Create a new tree validator
    pub fn new(validate_entries: bool, check_duplicates: bool, enforce_sorting: bool) -> Self {
        Self {
            validate_entries,
            check_duplicates,
            enforce_sorting,
        }
    }

    /// Validate a tree entry
    pub fn validate_entry(&self, mode: &str, filename: &str, object_id: &str) -> TreeEntryContract {
        TreeEntryContract::new(mode, filename.to_string(), object_id.to_string())
    }

    /// Create a tree object from raw entries
    pub fn create_tree_object(
        &self,
        id: &str,
        raw_entries: Vec<(String, String, String)>, // (mode, filename, object_id)
    ) -> TreeObjectContract {
        let entries: Vec<TreeEntryContract> = raw_entries
            .into_iter()
            .map(|(mode, filename, object_id)| TreeEntryContract::new(&mode, filename, object_id))
            .collect();

        TreeObjectContract::new(id.to_string(), entries)
    }

    /// Get a summary of tree validation results
    pub fn summarize_tree(&self, tree: &TreeObjectContract) -> String {
        let total_entries = tree.entries.len();
        let valid_entries = tree.entries.iter().filter(|e| e.is_valid()).count();
        let blob_entries = tree.get_blob_entries().len();
        let tree_entries = tree.get_tree_entries().len();

        format!(
            "Tree: {} | Entries: {} total ({} valid, {} blobs, {} trees)",
            tree.summary(),
            total_entries,
            valid_entries,
            blob_entries,
            tree_entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_mode_creation() {
        let mode = TreeMode::from_str("100644").unwrap();
        assert_eq!(mode, TreeMode::RegularFile);
        assert_eq!(mode.to_string(), "100644");
        assert_eq!(mode.description(), "Regular file (non-executable)");
        assert!(mode.is_blob());
        assert!(!mode.is_tree());
        assert_eq!(mode.object_type(), TreeObjectType::Blob);
    }

    #[test]
    fn test_tree_mode_tree() {
        let mode = TreeMode::from_str("040000").unwrap();
        assert_eq!(mode, TreeMode::Tree);
        assert_eq!(mode.to_string(), "040000");
        assert_eq!(mode.description(), "Tree (directory)");
        assert!(!mode.is_blob());
        assert!(mode.is_tree());
        assert_eq!(mode.object_type(), TreeObjectType::Tree);
    }

    #[test]
    fn test_tree_mode_invalid() {
        let mode = TreeMode::from_str("999999");
        assert!(mode.is_none());
    }

    #[test]
    fn test_tree_entry_contract_creation() {
        let entry = TreeEntryContract::new(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        );

        assert_eq!(entry.mode, TreeMode::RegularFile);
        assert_eq!(entry.filename, "README.md");
        assert_eq!(entry.object_type, TreeObjectType::Blob);
        assert!(entry.is_valid());
    }

    #[test]
    fn test_tree_entry_contract_invalid_mode() {
        let entry = TreeEntryContract::new(
            "999999",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        );

        assert!(!entry.is_valid());
        assert!(!entry.errors.is_empty());
    }

    #[test]
    fn test_tree_entry_contract_invalid_object_id() {
        let entry =
            TreeEntryContract::new("100644", "README.md".to_string(), "invalid".to_string());

        assert!(!entry.is_valid());
        assert!(!entry.errors.is_empty());
    }

    #[test]
    fn test_tree_object_contract_creation() {
        let entries = vec![
            TreeEntryContract::new(
                "100644",
                "README.md".to_string(),
                "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            ),
            TreeEntryContract::new(
                "040000",
                "src".to_string(),
                "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            ),
        ];

        let tree = TreeObjectContract::new("tree123".to_string(), entries);

        assert_eq!(tree.id, "tree123");
        assert_eq!(tree.entries.len(), 2);
        assert!(tree.is_valid());
        assert_eq!(tree.get_blob_entries().len(), 1);
        assert_eq!(tree.get_tree_entries().len(), 1);
    }

    #[test]
    fn test_tree_validator() {
        let validator = TreeValidator::default();
        let raw_entries = vec![
            (
                "100644".to_string(),
                "README.md".to_string(),
                "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            ),
            (
                "040000".to_string(),
                "src".to_string(),
                "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            ),
        ];

        let tree = validator.create_tree_object("tree123", raw_entries);

        assert!(tree.is_valid());
        assert_eq!(tree.entries.len(), 2);
        assert_eq!(tree.get_blob_entries().len(), 1);
        assert_eq!(tree.get_tree_entries().len(), 1);
    }
}
