use serde::{Deserialize, Serialize};
use crate::blob_contract::BlobContract;
use crate::tree_contract::{TreeEntryContract, TreeValidator};

/// Git object contract - represents the validation contract for Git objects with attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitObjectContract {
    /// Object type (blob, tree, commit, tag)
    pub object_type: GitObjectType,
    /// Object ID (SHA-1/SHA-256)
    pub oid: String,
    /// Object size in bytes
    pub size: usize,
    /// Whether the object is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Git attributes for this object
    pub attributes: Option<Vec<String>>,
    /// Nested contracts (for tree objects)
    pub nested_contracts: Option<Vec<GitObjectContract>>,
}

/// Git object types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitObjectType {
    /// Blob (file)
    Blob,
    /// Tree (directory)
    Tree,
    /// Commit
    Commit,
    /// Tag
    Tag,
}

impl GitObjectContract {
    /// Create a new Git object contract
    pub fn new(object_type: GitObjectType, oid: String, size: usize) -> Self {
        Self {
            object_type,
            oid,
            size,
            valid: true,
            errors: Vec::new(),
            attributes: None,
            nested_contracts: None,
        }
    }

    /// Create a new Git object contract with attributes
    pub fn new_with_attributes(
        object_type: GitObjectType,
        oid: String,
        size: usize,
        attributes: Option<Vec<String>>,
    ) -> Self {
        Self {
            object_type,
            oid,
            size,
            valid: true,
            errors: Vec::new(),
            attributes,
            nested_contracts: None,
        }
    }

    /// Add attributes to the object contract
    pub fn add_attributes(&mut self, attributes: Vec<String>) {
        self.attributes = Some(attributes);
    }

    /// Check if this object has a specific attribute
    pub fn has_attribute(&self, attribute: &str) -> bool {
        if let Some(ref attrs) = self.attributes {
            attrs.iter().any(|attr| attr == attribute)
        } else {
            false
        }
    }

    /// Get the value of a key=value attribute
    pub fn get_attribute_value(&self, key: &str) -> Option<&str> {
        if let Some(ref attrs) = self.attributes {
            for attr in attrs {
                if let Some(value) = attr.strip_prefix(&format!("{}=", key)) {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Validate attributes for a Git object
    pub fn validate_attributes(&mut self, filepath: Option<&str>) -> bool {
        let mut valid = true;

        if let Some(ref attrs) = self.attributes {
            // Validate attribute format
            for attr in attrs {
                if !Self::is_valid_attribute_format(attr) {
                    self.errors
                        .push(format!("Invalid attribute format: {}", attr));
                    valid = false;
                }
            }

            // Check for linguist-generated attribute if filepath is provided
            if let Some(path) = filepath {
                let has_linguist_generated = self.has_attribute("linguist-generated=true");
                let is_generated = Self::is_generated_file(path);

                if is_generated && !has_linguist_generated {
                    self.errors.push(format!(
                        "Generated file '{}' must have 'linguist-generated=true' attribute",
                        path
                    ));
                    valid = false;
                }

                if !is_generated && has_linguist_generated {
                    self.errors.push(format!(
                        "Non-generated file '{}' should not have 'linguist-generated=true' attribute",
                        path
                    ));
                    // This is a warning, not necessarily an error
                }
            }
        } else {
            // Check if this is a generated file that needs linguist-generated=true
            if let Some(path) = filepath {
                if Self::is_generated_file(path) {
                    self.errors.push(format!(
                        "Generated file '{}' must have 'linguist-generated=true' attribute",
                        path
                    ));
                    valid = false;
                }
            }
        }

        if !valid {
            self.valid = false;
        }

        valid
    }

    /// Check if an attribute has valid format
    fn is_valid_attribute_format(attr: &str) -> bool {
        match attr {
            "linguist-generated=true" | "linguist-generated=false" => true,
            "-diff" | "-merge" | "-export-ignore" | "-export-subst" => true,
            attr if attr.starts_with("linguist-") => true,
            attr if attr.starts_with("-") => true,
            attr if attr.contains('=') => true,
            _ => false,
        }
    }

    /// Check if a file path indicates it's a generated file
    fn is_generated_file(filepath: &str) -> bool {
        let generated_patterns = [
            "target/",
            "gen/",
            "generated/",
            "build/",
            "dist/",
            "node_modules/",
            ".git/",
            "*.min.js",
            "*.min.css",
            "*.bundle.js",
            "*.bundle.css",
        ];

        for pattern in &generated_patterns {
            if pattern.ends_with('/') {
                // Directory pattern
                if filepath.starts_with(pattern) {
                    return true;
                }
            } else if let Some(suffix) = pattern.strip_prefix('*') {
                // Wildcard pattern
                if filepath.ends_with(suffix) {
                    return true;
                }
            }
        }

        false
    }

    /// Get a summary of the Git object contract
    pub fn summary(&self) -> String {
        if self.valid {
            let mut summary = format!(
                "✅ Git object {} ({:?}) valid ({} bytes)",
                &self.oid[..8],
                self.object_type,
                self.size
            );

            if let Some(ref attrs) = self.attributes {
                if !attrs.is_empty() {
                    summary.push_str(&format!(" [attributes: {}]", attrs.join(", ")));
                }
            }

            summary
        } else {
            format!(
                "❌ Git object {} ({:?}) invalid: {}",
                &self.oid[..8],
                self.object_type,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the object is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Add a validation error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }

    /// Convert from a TreeEntryContract
    pub fn from_tree_entry(entry: &TreeEntryContract) -> Self {
        let object_type = match entry.object_type {
            crate::tree_contract::TreeObjectType::Blob => GitObjectType::Blob,
            crate::tree_contract::TreeObjectType::Tree => GitObjectType::Tree,
        };

        let mut contract = Self::new_with_attributes(
            object_type,
            entry.object_id.clone(),
            0, // Size will be set separately
            entry.attributes.clone(),
        );

        if !entry.valid {
            contract.valid = false;
            contract.errors = entry.errors.clone();
        }

        contract
    }

    /// Convert from a BlobContract
    pub fn from_blob_contract(blob: &BlobContract) -> Self {
        let mut contract = Self::new_with_attributes(
            GitObjectType::Blob,
            blob.oid.clone(),
            blob.size,
            blob.attributes.clone(),
        );

        if blob.action == crate::blob_contract::BlobAction::Reject {
            contract.valid = false;
            contract.errors.push("Blob validation failed".to_string());
        }

        contract
    }
}

/// Git object validator that processes Git object validation with attributes
pub struct GitObjectValidator {
    /// Whether to validate blobs
    #[allow(dead_code)]
    validate_blobs: bool,
    /// Whether to validate lines
    #[allow(dead_code)]
    validate_lines: bool,
    /// Whether to validate attributes
    validate_attributes: bool,
    /// Whether to enforce generated rules
    #[allow(dead_code)]
    enforce_generated_rules: bool,
    /// Tree validator for nested validation
    #[allow(dead_code)]
    tree_validator: TreeValidator,
}

impl Default for GitObjectValidator {
    fn default() -> Self {
        Self {
            validate_blobs: true,
            validate_lines: true,
            validate_attributes: true,
            enforce_generated_rules: true,
            tree_validator: TreeValidator::default(),
        }
    }
}

impl GitObjectValidator {
    /// Create a new Git object validator
    pub fn new(
        validate_blobs: bool,
        validate_lines: bool,
        validate_attributes: bool,
        enforce_generated_rules: bool,
        tree_validator: TreeValidator,
    ) -> Self {
        Self {
            validate_blobs,
            validate_lines,
            validate_attributes,
            enforce_generated_rules,
            tree_validator,
        }
    }

    /// Validate a Git object with attributes
    pub fn validate_object(
        &self,
        object_type: GitObjectType,
        oid: String,
        size: usize,
        attributes: Option<Vec<String>>,
        filepath: Option<&str>,
    ) -> GitObjectContract {
        let mut contract =
            GitObjectContract::new_with_attributes(object_type, oid, size, attributes);

        if self.validate_attributes {
            contract.validate_attributes(filepath);
        }

        contract
    }

    /// Validate a tree entry with attributes
    pub fn validate_tree_entry(&self, entry: &TreeEntryContract) -> GitObjectContract {
        let mut contract = GitObjectContract::from_tree_entry(entry);

        if self.validate_attributes {
            contract.validate_attributes(Some(&entry.filename));
        }

        contract
    }

    /// Validate a blob with attributes
    pub fn validate_blob(&self, blob: &BlobContract, filepath: Option<&str>) -> GitObjectContract {
        let mut contract = GitObjectContract::from_blob_contract(blob);

        if self.validate_attributes {
            contract.validate_attributes(filepath);
        }

        contract
    }

    /// Get a summary of validation results
    pub fn summarize_validation(&self, contracts: &[GitObjectContract]) -> String {
        let valid_count = contracts.iter().filter(|c| c.is_valid()).count();
        let total_count = contracts.len();
        let invalid_count = total_count - valid_count;

        let mut summary = format!(
            "Git Object Validation Summary: {} valid, {} invalid (total: {})",
            valid_count, invalid_count, total_count
        );

        if invalid_count > 0 {
            summary.push_str("\n\nInvalid objects:");
            for contract in contracts.iter().filter(|c| !c.is_valid()) {
                summary.push_str(&format!("\n  - {}", contract.summary()));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_object_contract_creation() {
        let contract = GitObjectContract::new(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
        );

        assert!(contract.is_valid());
        assert_eq!(contract.object_type, GitObjectType::Blob);
        assert_eq!(contract.size, 1024);
    }

    #[test]
    fn test_git_object_contract_with_attributes() {
        let attributes = vec!["linguist-generated=true".to_string(), "-diff".to_string()];

        let mut contract = GitObjectContract::new_with_attributes(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
            Some(attributes.clone()),
        );

        assert!(contract.has_attribute("linguist-generated=true"));
        assert!(contract.has_attribute("-diff"));
        assert!(!contract.has_attribute("nonexistent"));

        assert_eq!(
            contract.get_attribute_value("linguist-generated"),
            Some("true")
        );
    }

    #[test]
    fn test_generated_file_validation() {
        let mut contract = GitObjectContract::new(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
        );

        // Generated file without linguist-generated=true should fail
        contract.validate_attributes(Some("target/build/file.js"));
        assert!(!contract.is_valid());
        assert!(contract
            .errors
            .iter()
            .any(|e| e.contains("linguist-generated=true")));

        // Generated file with linguist-generated=true should pass
        let mut contract2 = GitObjectContract::new_with_attributes(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
            Some(vec!["linguist-generated=true".to_string()]),
        );
        contract2.validate_attributes(Some("target/build/file.js"));
        assert!(contract2.is_valid());

        // Non-generated file with linguist-generated=true should warn
        let mut contract3 = GitObjectContract::new_with_attributes(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
            Some(vec!["linguist-generated=true".to_string()]),
        );
        contract3.validate_attributes(Some("src/main.rs"));
        assert!(contract3.is_valid()); // Warning doesn't make it invalid
    }

    #[test]
    fn test_git_object_validator() {
        let tree_validator = TreeValidator::new(true, true, true);
        let validator = GitObjectValidator::new(true, true, true, tree_validator);

        let contract = validator.validate_object(
            GitObjectType::Blob,
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            1024,
            Some(vec!["linguist-generated=true".to_string()]),
            Some("target/file.js"),
        );

        assert!(contract.is_valid());
    }
}
