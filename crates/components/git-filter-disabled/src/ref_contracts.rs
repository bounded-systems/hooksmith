use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ============================================================================
// Ref-Specific Shared Primitives
// ============================================================================

/// Valid ref name regex pattern (Git ref naming rules)
pub static VALID_REF_NAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(refs/(heads|tags|remotes|notes|stash)|HEAD|ORIG_HEAD|MERGE_HEAD|CHERRY_PICK_HEAD|REBASE_HEAD|FETCH_HEAD|AUTO_MERGE)$").unwrap()
});

/// Valid branch name regex pattern (simplified, allows alphanumeric, hyphens, underscores)
pub static VALID_BRANCH_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

/// Valid tag name regex pattern
pub static VALID_TAG_NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap());

/// Valid remote name regex pattern
pub static VALID_REMOTE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

// ============================================================================
// Ref Types and Enums
// ============================================================================

/// Type of Git reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefType {
    /// Branch reference (refs/heads/*)
    Branch,
    /// Tag reference (refs/tags/*)
    Tag,
    /// Remote reference (refs/remotes/*)
    Remote,
    /// Note reference (refs/notes/*)
    Note,
    /// Stash reference (refs/stash)
    Stash,
    /// HEAD reference
    Head,
    /// Special pseudoref (ORIG_HEAD, MERGE_HEAD, etc.)
    PseudoRef,
}

impl RefType {
    /// Determine ref type from ref name
    pub fn from_name(name: &str) -> Option<Self> {
        if name == "HEAD" {
            Some(RefType::Head)
        } else if name.starts_with("refs/heads/") {
            Some(RefType::Branch)
        } else if name.starts_with("refs/tags/") {
            Some(RefType::Tag)
        } else if name.starts_with("refs/remotes/") {
            Some(RefType::Remote)
        } else if name.starts_with("refs/notes/") {
            Some(RefType::Note)
        } else if name == "refs/stash" {
            Some(RefType::Stash)
        } else if matches!(
            name,
            "ORIG_HEAD"
                | "MERGE_HEAD"
                | "CHERRY_PICK_HEAD"
                | "REBASE_HEAD"
                | "FETCH_HEAD"
                | "AUTO_MERGE"
        ) {
            Some(RefType::PseudoRef)
        } else {
            None
        }
    }

    /// Get description of the ref type
    pub fn description(&self) -> &'static str {
        match self {
            RefType::Branch => "Branch reference",
            RefType::Tag => "Tag reference",
            RefType::Remote => "Remote reference",
            RefType::Note => "Note reference",
            RefType::Stash => "Stash reference",
            RefType::Head => "HEAD reference",
            RefType::PseudoRef => "Special pseudoref",
        }
    }

    /// Get the expected target object type for this ref type
    pub fn expected_target_type(&self) -> &'static str {
        match self {
            RefType::Branch => "commit",
            RefType::Tag => "tag",
            RefType::Remote => "commit",
            RefType::Note => "blob",
            RefType::Stash => "commit",
            RefType::Head => "commit", // Usually points to a commit via symbolic ref
            RefType::PseudoRef => "commit",
        }
    }
}

/// Storage type for Git references
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefStorageType {
    /// Loose ref (individual file in .git/refs/)
    Loose,
    /// Packed ref (in .git/packed-refs)
    Packed,
    /// Pseudoref (HEAD, etc.)
    PseudoRef,
}

// ============================================================================
// Ref Name Contract
// ============================================================================

/// Contract for validating ref names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefNameContract {
    /// The ref name
    pub name: String,
    /// The ref type derived from the name
    pub ref_type: Option<RefType>,
    /// Whether the ref name is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl RefNameContract {
    /// Create a new ref name contract
    pub fn new(name: String) -> Self {
        let (ref_type, valid, errors) = Self::validate_ref_name(&name);
        Self {
            name,
            ref_type,
            valid,
            errors,
        }
    }

    /// Validate a ref name according to Git rules
    fn validate_ref_name(name: &str) -> (Option<RefType>, bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if name is empty
        if name.is_empty() {
            errors.push("Ref name must not be empty".to_string());
            return (None, false, errors);
        }

        // Determine ref type
        let ref_type = RefType::from_name(name);
        if ref_type.is_none() {
            errors.push(format!("Invalid ref name format: {name}"));
        }

        // Check against valid ref name pattern
        if !VALID_REF_NAME_RE.is_match(name) {
            errors.push(format!("Ref name '{name}' does not match valid pattern"));
        }

        // Additional validation based on ref type
        if let Some(ref_type) = &ref_type {
            match ref_type {
                RefType::Branch => {
                    let branch_name = name.strip_prefix("refs/heads/").unwrap_or(name);
                    if !VALID_BRANCH_NAME_RE.is_match(branch_name) {
                        errors.push(format!("Invalid branch name: {branch_name}"));
                    }
                }
                RefType::Tag => {
                    let tag_name = name.strip_prefix("refs/tags/").unwrap_or(name);
                    if !VALID_TAG_NAME_RE.is_match(tag_name) {
                        errors.push(format!("Invalid tag name: {tag_name}"));
                    }
                }
                RefType::Remote => {
                    let remote_name = name.strip_prefix("refs/remotes/").unwrap_or(name);
                    if !VALID_REMOTE_NAME_RE.is_match(remote_name) {
                        errors.push(format!("Invalid remote name: {remote_name}"));
                    }
                }
                _ => {}
            }
        }

        let valid = errors.is_empty();
        (ref_type, valid, errors)
    }

    /// Get a summary of the ref name contract
    pub fn summary(&self) -> String {
        if self.valid {
            let ref_type_desc = self
                .ref_type
                .as_ref()
                .map(|rt| rt.description())
                .unwrap_or("Unknown");
            format!("✅ Ref '{}' valid ({})", self.name, ref_type_desc)
        } else {
            format!("❌ Ref '{}' invalid: {}", self.name, self.errors.join(", "))
        }
    }

    /// Check if the ref name is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the ref type
    pub fn get_ref_type(&self) -> Option<&RefType> {
        self.ref_type.as_ref()
    }

    /// Extract the short name (without refs/ prefix)
    pub fn short_name(&self) -> &str {
        if self.name.starts_with("refs/heads/") {
            &self.name[11..] // "refs/heads/".len()
        } else if self.name.starts_with("refs/tags/") {
            &self.name[10..] // "refs/tags/".len()
        } else if self.name.starts_with("refs/remotes/") {
            &self.name[13..] // "refs/remotes/".len()
        } else {
            &self.name
        }
    }
}

// ============================================================================
// Ref Target Contract
// ============================================================================

/// Contract for validating ref targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefTargetContract {
    /// The ref name
    pub ref_name: String,
    /// The target object ID
    pub target_oid: String,
    /// Whether the target is symbolic (points to another ref)
    pub is_symbolic: bool,
    /// The symbolic target (if is_symbolic is true)
    pub symbolic_target: Option<String>,
    /// The expected object type for this ref
    pub expected_type: String,
    /// Whether the target is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl RefTargetContract {
    /// Create a new ref target contract for a direct object reference
    pub fn new_direct(ref_name: String, target_oid: String) -> Self {
        let (expected_type, valid, errors) = Self::validate_direct_target(&ref_name, &target_oid);
        Self {
            ref_name,
            target_oid,
            is_symbolic: false,
            symbolic_target: None,
            expected_type,
            valid,
            errors,
        }
    }

    /// Create a new ref target contract for a symbolic reference
    pub fn new_symbolic(ref_name: String, symbolic_target: String) -> Self {
        let (expected_type, valid, errors) =
            Self::validate_symbolic_target(&ref_name, &symbolic_target);
        Self {
            ref_name,
            target_oid: String::new(),
            is_symbolic: true,
            symbolic_target: Some(symbolic_target),
            expected_type,
            valid,
            errors,
        }
    }

    /// Validate a direct object target
    fn validate_direct_target(ref_name: &str, target_oid: &str) -> (String, bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if target OID is valid SHA-1
        if !crate::unified_contracts::SHA1_RE.is_match(target_oid) {
            errors.push(format!("Invalid target OID: {target_oid}"));
        }

        // Determine expected object type based on ref type
        let expected_type = RefType::from_name(ref_name)
            .map(|rt| rt.expected_target_type().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let valid = errors.is_empty();
        (expected_type, valid, errors)
    }

    /// Validate a symbolic target
    fn validate_symbolic_target(
        ref_name: &str,
        symbolic_target: &str,
    ) -> (String, bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if symbolic target is a valid ref name
        if !crate::unified_contracts::VALID_FILENAME_RE.is_match(symbolic_target) {
            errors.push(format!("Invalid symbolic target: {symbolic_target}"));
        }

        // Determine expected object type based on ref type
        let expected_type = RefType::from_name(ref_name)
            .map(|rt| rt.expected_target_type().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let valid = errors.is_empty();
        (expected_type, valid, errors)
    }

    /// Get a summary of the ref target contract
    pub fn summary(&self) -> String {
        if self.valid {
            if self.is_symbolic {
                format!(
                    "✅ Ref '{}' symbolic -> '{}' (expects {})",
                    self.ref_name,
                    self.symbolic_target.as_ref().unwrap(),
                    self.expected_type
                )
            } else {
                format!(
                    "✅ Ref '{}' -> {} (expects {})",
                    self.ref_name,
                    &self.target_oid[..8],
                    self.expected_type
                )
            }
        } else {
            format!(
                "❌ Ref '{}' invalid: {}",
                self.ref_name,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the ref target is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the target OID (for direct refs) or symbolic target (for symbolic refs)
    pub fn get_target(&self) -> &str {
        if self.is_symbolic {
            self.symbolic_target.as_ref().unwrap()
        } else {
            &self.target_oid
        }
    }
}

// ============================================================================
// Ref Meta Contract
// ============================================================================

/// Contract for ref metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefMetaContract {
    /// The ref name
    pub ref_name: String,
    /// Storage type (loose, packed, pseudoref)
    pub storage_type: RefStorageType,
    /// Upstream tracking information (for branches)
    pub upstream: Option<UpstreamInfo>,
    /// Reflog information
    pub reflog: Option<ReflogInfo>,
    /// Whether the metadata is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

/// Upstream tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    /// Remote name
    pub remote: String,
    /// Branch name on remote
    pub branch: String,
    /// Whether the upstream is valid
    pub valid: bool,
}

/// Reflog information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflogInfo {
    /// Number of reflog entries
    pub entry_count: usize,
    /// Latest reflog entry
    pub latest_entry: Option<ReflogEntry>,
    /// Whether the reflog is valid
    pub valid: bool,
}

/// Single reflog entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflogEntry {
    /// Old object ID
    pub old_oid: String,
    /// New object ID
    pub new_oid: String,
    /// Author information
    pub author: String,
    /// Timestamp
    pub timestamp: String,
    /// Action description
    pub action: String,
}

impl RefMetaContract {
    /// Create a new ref meta contract
    pub fn new(ref_name: String, storage_type: RefStorageType) -> Self {
        let (valid, errors) = Self::validate_metadata(&ref_name, &storage_type);
        Self {
            ref_name,
            storage_type,
            upstream: None,
            reflog: None,
            valid,
            errors,
        }
    }

    /// Create a new ref meta contract with upstream information
    pub fn with_upstream(
        ref_name: String,
        storage_type: RefStorageType,
        upstream: Option<UpstreamInfo>,
    ) -> Self {
        let (valid, errors) = Self::validate_metadata(&ref_name, &storage_type);
        Self {
            ref_name,
            storage_type,
            upstream,
            reflog: None,
            valid,
            errors,
        }
    }

    /// Create a new ref meta contract with reflog information
    pub fn with_reflog(
        ref_name: String,
        storage_type: RefStorageType,
        reflog: Option<ReflogInfo>,
    ) -> Self {
        let (valid, errors) = Self::validate_metadata(&ref_name, &storage_type);
        Self {
            ref_name,
            storage_type,
            upstream: None,
            reflog,
            valid,
            errors,
        }
    }

    /// Validate ref metadata
    fn validate_metadata(ref_name: &str, storage_type: &RefStorageType) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        // Validate ref name
        if ref_name.is_empty() {
            errors.push("Ref name must not be empty".to_string());
        }

        // Validate storage type consistency
        match storage_type {
            RefStorageType::PseudoRef => {
                if !matches!(
                    ref_name,
                    "HEAD"
                        | "ORIG_HEAD"
                        | "MERGE_HEAD"
                        | "CHERRY_PICK_HEAD"
                        | "REBASE_HEAD"
                        | "FETCH_HEAD"
                        | "AUTO_MERGE"
                ) {
                    errors.push(format!(
                        "Pseudoref storage type inconsistent with ref name: {ref_name}"
                    ));
                }
            }
            RefStorageType::Loose | RefStorageType::Packed => {
                if matches!(
                    ref_name,
                    "HEAD"
                        | "ORIG_HEAD"
                        | "MERGE_HEAD"
                        | "CHERRY_PICK_HEAD"
                        | "REBASE_HEAD"
                        | "FETCH_HEAD"
                        | "AUTO_MERGE"
                ) {
                    errors.push(format!(
                        "Regular storage type inconsistent with pseudoref name: {ref_name}"
                    ));
                }
            }
        }

        let valid = errors.is_empty();
        (valid, errors)
    }

    /// Get a summary of the ref meta contract
    pub fn summary(&self) -> String {
        if self.valid {
            let mut parts = vec![format!(
                "✅ Ref '{}' ({:?})",
                self.ref_name, self.storage_type
            )];

            if let Some(upstream) = &self.upstream {
                parts.push(format!("upstream: {}/{}", upstream.remote, upstream.branch));
            }

            if let Some(reflog) = &self.reflog {
                parts.push(format!("reflog: {} entries", reflog.entry_count));
            }

            parts.join(", ")
        } else {
            format!(
                "❌ Ref '{}' invalid: {}",
                self.ref_name,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the ref metadata is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the storage type
    pub fn get_storage_type(&self) -> &RefStorageType {
        &self.storage_type
    }

    /// Check if this ref has upstream tracking
    pub fn has_upstream(&self) -> bool {
        self.upstream.is_some()
    }

    /// Check if this ref has reflog
    pub fn has_reflog(&self) -> bool {
        self.reflog.is_some()
    }
}

// ============================================================================
// Unified Ref Contract
// ============================================================================

/// Unified contract for Git references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefContract {
    /// Ref name contract
    pub name: RefNameContract,
    /// Ref target contract
    pub target: RefTargetContract,
    /// Ref metadata contract
    pub metadata: RefMetaContract,
    /// Whether the entire ref contract is valid
    pub valid: bool,
    /// Overall validation errors
    pub errors: Vec<String>,
}

impl RefContract {
    /// Create a new ref contract
    pub fn new(ref_name: String, target_oid: String, storage_type: RefStorageType) -> Self {
        let name = RefNameContract::new(ref_name.clone());
        let target = RefTargetContract::new_direct(ref_name.clone(), target_oid);
        let metadata = RefMetaContract::new(ref_name, storage_type);

        let (valid, errors) = Self::validate_ref_contract(&name, &target, &metadata);

        Self {
            name,
            target,
            metadata,
            valid,
            errors,
        }
    }

    /// Create a new symbolic ref contract
    pub fn new_symbolic(
        ref_name: String,
        symbolic_target: String,
        storage_type: RefStorageType,
    ) -> Self {
        let name = RefNameContract::new(ref_name.clone());
        let target = RefTargetContract::new_symbolic(ref_name.clone(), symbolic_target);
        let metadata = RefMetaContract::new(ref_name, storage_type);

        let (valid, errors) = Self::validate_ref_contract(&name, &target, &metadata);

        Self {
            name,
            target,
            metadata,
            valid,
            errors,
        }
    }

    /// Validate the entire ref contract
    fn validate_ref_contract(
        name: &RefNameContract,
        target: &RefTargetContract,
        metadata: &RefMetaContract,
    ) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if all components are valid
        if !name.is_valid() {
            errors.extend(name.errors.clone());
        }

        if !target.is_valid() {
            errors.extend(target.errors.clone());
        }

        if !metadata.is_valid() {
            errors.extend(metadata.errors.clone());
        }

        // Cross-validation: check if ref type matches expected target type
        if let Some(ref_type) = name.get_ref_type() {
            let expected_type = ref_type.expected_target_type();
            if target.expected_type != expected_type {
                errors.push(format!(
                    "Ref type mismatch: {} expects {}, but target expects {}",
                    ref_type.description(),
                    expected_type,
                    target.expected_type
                ));
            }
        }

        let valid = errors.is_empty();
        (valid, errors)
    }

    /// Get a summary of the ref contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Ref '{}' valid ({})",
                self.name.name,
                self.name
                    .get_ref_type()
                    .map(|rt| rt.description())
                    .unwrap_or("Unknown")
            )
        } else {
            format!(
                "❌ Ref '{}' invalid: {}",
                self.name.name,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the ref contract is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the ref name
    pub fn get_name(&self) -> &str {
        &self.name.name
    }

    /// Get the ref type
    pub fn get_ref_type(&self) -> Option<&RefType> {
        self.name.get_ref_type()
    }

    /// Get the target (OID or symbolic ref)
    pub fn get_target(&self) -> &str {
        self.target.get_target()
    }

    /// Check if this is a symbolic ref
    pub fn is_symbolic(&self) -> bool {
        self.target.is_symbolic
    }
}

// ============================================================================
// Ref Validator
// ============================================================================

/// Validator for Git references
pub struct RefValidator {
    /// Whether to validate ref names
    validate_names: bool,
    /// Whether to validate ref targets
    validate_targets: bool,
    /// Whether to validate ref metadata
    validate_metadata: bool,
}

impl Default for RefValidator {
    fn default() -> Self {
        Self {
            validate_names: true,
            validate_targets: true,
            validate_metadata: true,
        }
    }
}

impl RefValidator {
    /// Create a new ref validator
    pub fn new(validate_names: bool, validate_targets: bool, validate_metadata: bool) -> Self {
        Self {
            validate_names,
            validate_targets,
            validate_metadata,
        }
    }

    /// Validate a single ref contract
    pub fn validate_ref(&self, ref_contract: &RefContract) -> bool {
        if !self.validate_names && !ref_contract.name.is_valid() {
            return false;
        }
        if !self.validate_targets && !ref_contract.target.is_valid() {
            return false;
        }
        if !self.validate_metadata && !ref_contract.metadata.is_valid() {
            return false;
        }
        ref_contract.is_valid()
    }

    /// Validate multiple ref contracts
    pub fn validate_refs(&self, ref_contracts: &[RefContract]) -> Vec<bool> {
        ref_contracts.iter().map(|r| self.validate_ref(r)).collect()
    }

    /// Get a summary of validation results
    pub fn summarize_validation(&self, ref_contracts: &[RefContract]) -> String {
        let total = ref_contracts.len();
        let valid = ref_contracts
            .iter()
            .filter(|r| self.validate_ref(r))
            .count();
        let invalid = total - valid;

        let mut type_counts = std::collections::HashMap::new();
        for ref_contract in ref_contracts {
            if let Some(ref_type) = ref_contract.get_ref_type() {
                *type_counts.entry(format!("{ref_type:?}")).or_insert(0) += 1;
            }
        }

        let type_summary: Vec<String> = type_counts
            .iter()
            .map(|(ref_type, count)| format!("{count} {ref_type}"))
            .collect();

        format!(
            "Refs: {total} total ({valid} valid, {invalid} invalid) - {}",
            type_summary.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_name_contract() {
        let valid_ref = RefNameContract::new("refs/heads/main".to_string());
        assert!(valid_ref.is_valid());
        assert_eq!(valid_ref.get_ref_type(), Some(&RefType::Branch));
        assert_eq!(valid_ref.short_name(), "main");

        let invalid_ref = RefNameContract::new("invalid/ref/name".to_string());
        assert!(!invalid_ref.is_valid());
    }

    #[test]
    fn test_ref_target_contract() {
        let direct_target = RefTargetContract::new_direct(
            "refs/heads/main".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        );
        assert!(direct_target.is_valid());
        assert!(!direct_target.is_symbolic);

        let symbolic_target =
            RefTargetContract::new_symbolic("HEAD".to_string(), "refs/heads/main".to_string());
        assert!(symbolic_target.is_valid());
        assert!(symbolic_target.is_symbolic);
    }

    #[test]
    fn test_ref_meta_contract() {
        let meta = RefMetaContract::new("refs/heads/main".to_string(), RefStorageType::Loose);
        assert!(meta.is_valid());

        let upstream_info = UpstreamInfo {
            remote: "origin".to_string(),
            branch: "main".to_string(),
            valid: true,
        };

        let meta_with_upstream = RefMetaContract::with_upstream(
            "refs/heads/main".to_string(),
            RefStorageType::Loose,
            Some(upstream_info),
        );
        assert!(meta_with_upstream.is_valid());
        assert!(meta_with_upstream.has_upstream());
    }

    #[test]
    fn test_ref_contract() {
        let ref_contract = RefContract::new(
            "refs/heads/main".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            RefStorageType::Loose,
        );
        assert!(ref_contract.is_valid());
        assert_eq!(ref_contract.get_name(), "refs/heads/main");
        assert_eq!(ref_contract.get_ref_type(), Some(&RefType::Branch));
        assert!(!ref_contract.is_symbolic());
    }

    #[test]
    fn test_symbolic_ref_contract() {
        let symbolic_ref = RefContract::new_symbolic(
            "HEAD".to_string(),
            "refs/heads/main".to_string(),
            RefStorageType::PseudoRef,
        );
        assert!(symbolic_ref.is_valid());
        assert!(symbolic_ref.is_symbolic());
    }

    #[test]
    fn test_ref_validator() {
        let validator = RefValidator::default();

        let refs = vec![
            RefContract::new(
                "refs/heads/main".to_string(),
                "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                RefStorageType::Loose,
            ),
            RefContract::new_symbolic(
                "HEAD".to_string(),
                "refs/heads/main".to_string(),
                RefStorageType::PseudoRef,
            ),
        ];

        let results = validator.validate_refs(&refs);
        assert_eq!(results, vec![true, true]);

        let summary = validator.summarize_validation(&refs);
        assert!(summary.contains("2 total"));
        assert!(summary.contains("Branch"));
        assert!(summary.contains("Head"));
    }
}
