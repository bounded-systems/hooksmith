use serde::{Deserialize, Serialize};

/// Filename contract - flat unscoped contract for filename validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilenameContract {
    /// Filename (must not be empty and optionally must not contain path separators)
    pub filename: String,
    /// Whether the filename is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl FilenameContract {
    /// Create a new filename contract (allows subdirectories)
    pub fn new(filename: String) -> Self {
        let (valid, errors) = Self::validate_filename(&filename, false);
        Self {
            filename,
            valid,
            errors,
        }
    }

    /// Create a new filename contract that blocks path separators
    pub fn new_strict(filename: String) -> Self {
        let (valid, errors) = Self::validate_filename(&filename, true);
        Self {
            filename,
            valid,
            errors,
        }
    }

    /// Validate a filename according to the specified rules
    fn validate_filename(filename: &str, strict: bool) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if filename is empty
        if filename.is_empty() {
            errors.push("Filename must not be empty".to_string());
        }

        // If strict mode, check for path separators
        if strict && filename.contains('/') {
            errors.push("Filename must not contain '/'".to_string());
        }

        let valid = errors.is_empty();
        (valid, errors)
    }

    /// Get a summary of the filename contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!("✅ Filename '{}' valid", self.filename)
        } else {
            format!(
                "❌ Filename '{}' invalid: {}",
                self.filename,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the filename is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the filename as a string
    pub fn as_str(&self) -> &str {
        &self.filename
    }

    /// Check if the filename contains path separators
    pub fn contains_path_separators(&self) -> bool {
        self.filename.contains('/')
    }

    /// Check if the filename is empty
    pub fn is_empty(&self) -> bool {
        self.filename.is_empty()
    }
}

/// Filename validator that processes filename validation
#[derive(Default)]
pub struct FilenameValidator {
    /// Whether to use strict validation (block path separators)
    strict: bool,
}

impl FilenameValidator {
    /// Create a new filename validator
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Validate a single filename
    pub fn validate_filename(&self, filename: &str) -> FilenameContract {
        if self.strict {
            FilenameContract::new_strict(filename.to_string())
        } else {
            FilenameContract::new(filename.to_string())
        }
    }

    /// Validate multiple filenames
    pub fn validate_filenames(&self, filenames: Vec<String>) -> Vec<FilenameContract> {
        filenames
            .into_iter()
            .map(|filename| self.validate_filename(&filename))
            .collect()
    }

    /// Get a summary of filename validation results
    pub fn summarize_validation(&self, contracts: &[FilenameContract]) -> String {
        let total_filenames = contracts.len();
        let valid_filenames = contracts.iter().filter(|c| c.is_valid()).count();
        let invalid_filenames = total_filenames - valid_filenames;

        format!(
            "Filenames: {total_filenames} total ({valid_filenames} valid, {invalid_filenames} invalid)"
        )
    }

    /// Check if all filenames are valid
    pub fn all_valid(&self, contracts: &[FilenameContract]) -> bool {
        contracts.iter().all(|c| c.is_valid())
    }

    /// Get invalid filenames
    pub fn get_invalid_filenames<'a>(
        &self,
        contracts: &'a [FilenameContract],
    ) -> Vec<&'a FilenameContract> {
        contracts.iter().filter(|c| !c.is_valid()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_contract_creation() {
        let contract = FilenameContract::new("README.md".to_string());

        assert_eq!(contract.filename, "README.md");
        assert!(contract.is_valid());
        assert!(contract.errors.is_empty());
    }

    #[test]
    fn test_filename_contract_empty() {
        let contract = FilenameContract::new("".to_string());

        assert!(!contract.is_valid());
        assert!(!contract.errors.is_empty());
        assert!(contract
            .errors
            .contains(&"Filename must not be empty".to_string()));
    }

    #[test]
    fn test_filename_contract_strict_valid() {
        let contract = FilenameContract::new_strict("README.md".to_string());

        assert_eq!(contract.filename, "README.md");
        assert!(contract.is_valid());
        assert!(contract.errors.is_empty());
    }

    #[test]
    fn test_filename_contract_strict_invalid() {
        let contract = FilenameContract::new_strict("src/main.rs".to_string());

        assert!(!contract.is_valid());
        assert!(!contract.errors.is_empty());
        assert!(contract
            .errors
            .contains(&"Filename must not contain '/'".to_string()));
    }

    #[test]
    fn test_filename_contract_non_strict_allows_paths() {
        let contract = FilenameContract::new("src/main.rs".to_string());

        assert_eq!(contract.filename, "src/main.rs");
        assert!(contract.is_valid());
        assert!(contract.errors.is_empty());
    }

    #[test]
    fn test_filename_validator() {
        let validator = FilenameValidator::default();
        let filenames = vec![
            "README.md".to_string(),
            "src/main.rs".to_string(),
            "".to_string(),
        ];

        let contracts = validator.validate_filenames(filenames);

        assert_eq!(contracts.len(), 3);
        assert!(contracts[0].is_valid()); // README.md
        assert!(contracts[1].is_valid()); // src/main.rs (allowed in non-strict)
        assert!(!contracts[2].is_valid()); // empty
    }

    #[test]
    fn test_filename_validator_strict() {
        let validator = FilenameValidator::new(true);
        let filenames = vec![
            "README.md".to_string(),
            "src/main.rs".to_string(),
            "".to_string(),
        ];

        let contracts = validator.validate_filenames(filenames);

        assert_eq!(contracts.len(), 3);
        assert!(contracts[0].is_valid()); // README.md
        assert!(!contracts[1].is_valid()); // src/main.rs (blocked in strict)
        assert!(!contracts[2].is_valid()); // empty
    }

    #[test]
    fn test_filename_validator_summary() {
        let validator = FilenameValidator::default();
        let contracts = vec![
            FilenameContract::new("README.md".to_string()),
            FilenameContract::new("src/main.rs".to_string()),
            FilenameContract::new("".to_string()),
        ];

        let summary = validator.summarize_validation(&contracts);
        assert_eq!(summary, "Filenames: 3 total (2 valid, 1 invalid)");
    }
}
