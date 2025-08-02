use serde::{Deserialize, Serialize};

/// Character-level contract for individual characters in filenames
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharContract {
    /// The character value
    pub char: char,
    /// Whether the character is valid
    pub valid: bool,
    /// Validation error (if any)
    pub error: Option<String>,
}

impl CharContract {
    /// Create a new character contract
    pub fn new(ch: char) -> Self {
        let (valid, error) = Self::validate_character(ch);
        Self {
            char: ch,
            valid,
            error,
        }
    }

    /// Validate a character according to ASCII printable rules
    fn validate_character(ch: char) -> (bool, Option<String>) {
        let code = ch as u32;

        // Allow only printable ASCII (space to ~), excluding control chars
        if code >= 0x20 && code <= 0x7e {
            (true, None)
        } else {
            (
                false,
                Some(format!("Invalid character: {:?} (code: 0x{:x})", ch, code)),
            )
        }
    }

    /// Get a summary of the character contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!("✅ Char '{}' valid", self.char)
        } else {
            format!(
                "❌ Char '{}' invalid: {}",
                self.char,
                self.error.as_ref().unwrap()
            )
        }
    }

    /// Check if the character is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the character as a string
    pub fn as_str(&self) -> String {
        self.char.to_string()
    }
}

/// Tree filename contract with character-level validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeFilenameContractChars {
    /// Filename as array of validated characters
    pub filename: Vec<CharContract>,
    /// Whether the filename is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl TreeFilenameContractChars {
    /// Create a new tree filename contract from a string
    pub fn new(filename: String) -> Self {
        let (char_contracts, valid, errors) = Self::validate_filename(&filename);
        Self {
            filename: char_contracts,
            valid,
            errors,
        }
    }

    /// Create a new tree filename contract from individual characters
    pub fn from_chars(chars: Vec<char>) -> Self {
        let char_contracts: Vec<CharContract> = chars.into_iter().map(CharContract::new).collect();
        let (valid, errors) = Self::validate_char_contracts(&char_contracts);
        Self {
            filename: char_contracts,
            valid,
            errors,
        }
    }

    /// Validate a filename string and return character contracts
    fn validate_filename(filename: &str) -> (Vec<CharContract>, bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check if filename is empty
        if filename.is_empty() {
            errors.push("Filename must have at least one character".to_string());
            return (Vec::new(), false, errors);
        }

        // Validate each character
        let char_contracts: Vec<CharContract> = filename.chars().map(CharContract::new).collect();
        let (valid, char_errors) = Self::validate_char_contracts(&char_contracts);

        // Combine errors
        errors.extend(char_errors);

        (char_contracts, valid && errors.is_empty(), errors)
    }

    /// Validate character contracts
    fn validate_char_contracts(char_contracts: &[CharContract]) -> (bool, Vec<String>) {
        let mut errors = Vec::new();
        let mut invalid_chars = 0;

        for (i, char_contract) in char_contracts.iter().enumerate() {
            if !char_contract.is_valid() {
                invalid_chars += 1;
                if let Some(error) = &char_contract.error {
                    errors.push(format!("Position {}: {}", i, error));
                }
            }
        }

        let valid = invalid_chars == 0;
        (valid, errors)
    }

    /// Get a summary of the tree filename contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Filename '{}' valid ({} chars)",
                self.as_string(),
                self.filename.len()
            )
        } else {
            format!(
                "❌ Filename '{}' invalid: {}",
                self.as_string(),
                self.errors.join(", ")
            )
        }
    }

    /// Check if the filename is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the filename as a string
    pub fn as_string(&self) -> String {
        self.filename.iter().map(|c| c.char).collect()
    }

    /// Get the filename length
    pub fn len(&self) -> usize {
        self.filename.len()
    }

    /// Check if the filename is empty
    pub fn is_empty(&self) -> bool {
        self.filename.is_empty()
    }

    /// Get invalid characters
    pub fn get_invalid_chars(&self) -> Vec<&CharContract> {
        self.filename.iter().filter(|c| !c.is_valid()).collect()
    }

    /// Get valid characters
    pub fn get_valid_chars(&self) -> Vec<&CharContract> {
        self.filename.iter().filter(|c| c.is_valid()).collect()
    }

    /// Get character at specific position
    pub fn get_char_at(&self, position: usize) -> Option<&CharContract> {
        self.filename.get(position)
    }
}

/// Tree filename validator with character-level validation
pub struct TreeFilenameCharsValidator {
    /// Whether to allow path separators
    allow_path_separators: bool,
}

impl Default for TreeFilenameCharsValidator {
    fn default() -> Self {
        Self {
            allow_path_separators: false,
        }
    }
}

impl TreeFilenameCharsValidator {
    /// Create a new tree filename validator
    pub fn new(allow_path_separators: bool) -> Self {
        Self {
            allow_path_separators,
        }
    }

    /// Validate a single filename
    pub fn validate_filename(&self, filename: &str) -> TreeFilenameContractChars {
        let mut contract = TreeFilenameContractChars::new(filename.to_string());

        // Additional validation for path separators if not allowed
        if !self.allow_path_separators && filename.contains('/') {
            contract.valid = false;
            contract
                .errors
                .push("Filename must not contain '/'".to_string());
        }

        contract
    }

    /// Validate multiple filenames
    pub fn validate_filenames(&self, filenames: Vec<String>) -> Vec<TreeFilenameContractChars> {
        filenames
            .into_iter()
            .map(|filename| self.validate_filename(&filename))
            .collect()
    }

    /// Get a summary of filename validation results
    pub fn summarize_validation(&self, contracts: &[TreeFilenameContractChars]) -> String {
        let total_filenames = contracts.len();
        let valid_filenames = contracts.iter().filter(|c| c.is_valid()).count();
        let invalid_filenames = total_filenames - valid_filenames;
        let total_chars: usize = contracts.iter().map(|c| c.len()).sum();
        let invalid_chars: usize = contracts.iter().map(|c| c.get_invalid_chars().len()).sum();

        format!(
            "Filenames: {} total ({} valid, {} invalid) | Characters: {} total ({} invalid)",
            total_filenames, valid_filenames, invalid_filenames, total_chars, invalid_chars
        )
    }

    /// Check if all filenames are valid
    pub fn all_valid(&self, contracts: &[TreeFilenameContractChars]) -> bool {
        contracts.iter().all(|c| c.is_valid())
    }

    /// Get invalid filenames
    pub fn get_invalid_filenames<'a>(
        &self,
        contracts: &'a [TreeFilenameContractChars],
    ) -> Vec<&'a TreeFilenameContractChars> {
        contracts.iter().filter(|c| !c.is_valid()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_contract_creation() {
        let contract = CharContract::new('a');

        assert_eq!(contract.char, 'a');
        assert!(contract.is_valid());
        assert!(contract.error.is_none());
    }

    #[test]
    fn test_char_contract_invalid() {
        let contract = CharContract::new('\x00');

        assert_eq!(contract.char, '\x00');
        assert!(!contract.is_valid());
        assert!(contract.error.is_some());
    }

    #[test]
    fn test_char_contract_printable_ascii() {
        // Test printable ASCII range (0x20 to 0x7e)
        for code in 0x20..=0x7e {
            let ch = char::from_u32(code).unwrap();
            let contract = CharContract::new(ch);
            assert!(
                contract.is_valid(),
                "Character {:?} (0x{:x}) should be valid",
                ch,
                code
            );
        }
    }

    #[test]
    fn test_char_contract_control_chars() {
        // Test control characters (should be invalid)
        for code in 0x00..0x20 {
            if let Some(ch) = char::from_u32(code) {
                let contract = CharContract::new(ch);
                assert!(
                    !contract.is_valid(),
                    "Control character {:?} (0x{:x}) should be invalid",
                    ch,
                    code
                );
            }
        }
    }

    #[test]
    fn test_tree_filename_contract_creation() {
        let contract = TreeFilenameContractChars::new("README.md".to_string());

        assert_eq!(contract.len(), 9);
        assert!(contract.is_valid());
        assert!(contract.errors.is_empty());
        assert_eq!(contract.as_string(), "README.md");
    }

    #[test]
    fn test_tree_filename_contract_empty() {
        let contract = TreeFilenameContractChars::new("".to_string());

        assert!(contract.is_empty());
        assert!(!contract.is_valid());
        assert!(!contract.errors.is_empty());
        assert!(contract
            .errors
            .contains(&"Filename must have at least one character".to_string()));
    }

    #[test]
    fn test_tree_filename_contract_invalid_chars() {
        let contract = TreeFilenameContractChars::new("file\x00name.txt".to_string());

        assert!(!contract.is_valid());
        assert!(!contract.errors.is_empty());
        assert_eq!(contract.len(), 13);

        let invalid_chars = contract.get_invalid_chars();
        assert_eq!(invalid_chars.len(), 1);
        assert_eq!(invalid_chars[0].char, '\x00');
    }

    #[test]
    fn test_tree_filename_contract_from_chars() {
        let chars = vec!['R', 'E', 'A', 'D', 'M', 'E', '.', 'm', 'd'];
        let contract = TreeFilenameContractChars::from_chars(chars);

        assert_eq!(contract.len(), 9);
        assert!(contract.is_valid());
        assert_eq!(contract.as_string(), "README.md");
    }

    #[test]
    fn test_tree_filename_validator() {
        let validator = TreeFilenameCharsValidator::default();
        let filenames = vec![
            "README.md".to_string(),
            "file\x00name.txt".to_string(),
            "".to_string(),
        ];

        let contracts = validator.validate_filenames(filenames);

        assert_eq!(contracts.len(), 3);
        assert!(contracts[0].is_valid()); // README.md
        assert!(!contracts[1].is_valid()); // file\x00name.txt
        assert!(!contracts[2].is_valid()); // empty
    }

    #[test]
    fn test_tree_filename_validator_path_separators() {
        let validator = TreeFilenameCharsValidator::new(false); // Don't allow path separators
        let contract = validator.validate_filename("src/main.rs");

        assert!(!contract.is_valid());
        assert!(contract
            .errors
            .contains(&"Filename must not contain '/'".to_string()));
    }

    #[test]
    fn test_tree_filename_validator_allow_path_separators() {
        let validator = TreeFilenameCharsValidator::new(true); // Allow path separators
        let contract = validator.validate_filename("src/main.rs");

        assert!(contract.is_valid());
        assert!(contract.errors.is_empty());
    }

    #[test]
    fn test_tree_filename_validator_summary() {
        let validator = TreeFilenameCharsValidator::default();
        let contracts = vec![
            TreeFilenameContractChars::new("README.md".to_string()),
            TreeFilenameContractChars::new("file\x00name.txt".to_string()),
            TreeFilenameContractChars::new("".to_string()),
        ];

        let summary = validator.summarize_validation(&contracts);
        assert_eq!(
            summary,
            "Filenames: 3 total (1 valid, 2 invalid) | Characters: 20 total (1 invalid)"
        );
    }
}
