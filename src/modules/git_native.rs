use anyhow::{bail, Result};
use std::collections::HashMap;

/// Git-native object types that map directly to git2::ObjectType and gix::ObjectKind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitObjectType {
    /// File contents (git2::ObjectType::Blob, gix::ObjectKind::Blob)
    Blob,
    /// Directory structure (git2::ObjectType::Tree, gix::ObjectKind::Tree)
    Tree,
    /// Commit history (git2::ObjectType::Commit, gix::ObjectKind::Commit)
    Commit,
    /// Annotated tag (git2::ObjectType::Tag, gix::ObjectKind::Tag)
    Tag,
}

/// Git namespaced metadata types (not objects but tracked by Git)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitMetadataType {
    /// References (heads, tags, etc.) - tracked in .git/refs/
    Ref,
    /// Notes (commit-attached metadata) - tracked in .git/refs/notes/
    Note,
    /// Attributes (file-based config) - tracked in working tree or .git/info
    Attr,
}

/// Git-native validation context
pub struct GitNativeValidator {
    object_counts: HashMap<GitObjectType, u32>,
    metadata_counts: HashMap<GitMetadataType, u32>,
}

impl GitNativeValidator {
    /// Create a new Git-native validator instance
    pub fn new() -> Self {
        Self {
            object_counts: HashMap::new(),
            metadata_counts: HashMap::new(),
        }
    }

    /// Validate Git object types against canonical Git types
    pub fn validate_object_types(&self, object_counts: &HashMap<String, u32>) -> Result<()> {
        let mut errors = Vec::new();
        
        for (object_type, count) in object_counts {
            match object_type.as_str() {
                "blob" | "tree" | "commit" | "tag" => {
                    // These are valid Git object types
                }
                _ => {
                    errors.push(format!("Unknown Git object type '{}' found {} times", object_type, count));
                }
            }
        }
        
        if !errors.is_empty() {
            bail!("Git object validation failed:\n{}", errors.join("\n"));
        }
        
        Ok(())
    }

    /// Map string object types to Git-native enums
    pub fn map_object_type(object_type: &str) -> Option<GitObjectType> {
        match object_type {
            "blob" => Some(GitObjectType::Blob),
            "tree" => Some(GitObjectType::Tree),
            "commit" => Some(GitObjectType::Commit),
            "tag" => Some(GitObjectType::Tag),
            _ => None,
        }
    }

    /// Map string metadata types to Git-native enums
    pub fn map_metadata_type(metadata_type: &str) -> Option<GitMetadataType> {
        match metadata_type {
            "ref" => Some(GitMetadataType::Ref),
            "note" => Some(GitMetadataType::Note),
            "attr" => Some(GitMetadataType::Attr),
            _ => None,
        }
    }

    /// Get canonical Git object type names
    pub fn canonical_object_types() -> Vec<&'static str> {
        vec!["blob", "tree", "commit", "tag"]
    }

    /// Get canonical Git metadata type names
    pub fn canonical_metadata_types() -> Vec<&'static str> {
        vec!["ref", "note", "attr"]
    }

    /// Validate that all concerns are Git-native
    pub fn validate_concerns(concerns: &[String]) -> Result<()> {
        let canonical_objects = Self::canonical_object_types();
        let canonical_metadata = Self::canonical_metadata_types();
        let all_canonical: Vec<&str> = canonical_objects.iter().chain(canonical_metadata.iter()).cloned().collect();
        
        let mut errors = Vec::new();
        
        for concern in concerns {
            if !all_canonical.contains(&concern.as_str()) {
                errors.push(format!("Non-Git-native concern '{}' not allowed", concern));
            }
        }
        
        if !errors.is_empty() {
            bail!("Hook concerns validation failed:\n{}", errors.join("\n"));
        }
        
        Ok(())
    }
}

impl Default for GitNativeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_object_type() {
        assert_eq!(GitNativeValidator::map_object_type("blob"), Some(GitObjectType::Blob));
        assert_eq!(GitNativeValidator::map_object_type("tree"), Some(GitObjectType::Tree));
        assert_eq!(GitNativeValidator::map_object_type("commit"), Some(GitObjectType::Commit));
        assert_eq!(GitNativeValidator::map_object_type("tag"), Some(GitObjectType::Tag));
        assert_eq!(GitNativeValidator::map_object_type("invalid"), None);
    }

    #[test]
    fn test_map_metadata_type() {
        assert_eq!(GitNativeValidator::map_metadata_type("ref"), Some(GitMetadataType::Ref));
        assert_eq!(GitNativeValidator::map_metadata_type("note"), Some(GitMetadataType::Note));
        assert_eq!(GitNativeValidator::map_metadata_type("attr"), Some(GitMetadataType::Attr));
        assert_eq!(GitNativeValidator::map_metadata_type("invalid"), None);
    }

    #[test]
    fn test_validate_concerns() {
        let valid_concerns = vec!["blob".to_string(), "tree".to_string(), "ref".to_string()];
        assert!(GitNativeValidator::validate_concerns(&valid_concerns).is_ok());
        
        let invalid_concerns = vec!["blob".to_string(), "contract-violation".to_string()];
        assert!(GitNativeValidator::validate_concerns(&invalid_concerns).is_err());
    }
}
