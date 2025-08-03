use serde::{Deserialize, Serialize};

/// Character classification for byte-level validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharClass {
    /// Standard visible ASCII characters (0x20-0x7E)
    AsciiPrintable,
    /// Whitelisted control characters (tab, LF)
    SafeControl,
    /// UTF-8 lead byte (0xC0-0xFD)
    Utf8Lead,
    /// UTF-8 continuation byte (0x80-0xBF)
    Utf8Cont,
    /// Forbidden characters (NUL, control chars except LF/TAB)
    Forbidden,
}

/// Action to take when a character is encountered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharAction {
    /// Accept this character
    Accept,
    /// Reject this character (and the file)
    Reject,
}

/// Character Dev Contract - represents the validation contract for a single byte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterContract {
    /// Raw byte value (0-255)
    pub byte: u8,
    /// Classification of this byte
    pub class: CharClass,
    /// Whether this character is allowed
    pub allowed: bool,
    /// Action to take when this character is encountered
    pub action: CharAction,
}

impl CharacterContract {
    /// Create a character contract from a byte
    pub fn from_byte(byte: u8) -> Self {
        let (class, allowed, action) = Self::classify_byte(byte);

        Self {
            byte,
            class,
            allowed,
            action,
        }
    }

    /// Classify a byte according to the Dev Contract rules
    fn classify_byte(byte: u8) -> (CharClass, bool, CharAction) {
        match byte {
            // ASCII printable characters (0x20-0x7E)
            0x20..=0x7E => (CharClass::AsciiPrintable, true, CharAction::Accept),

            // Safe control characters
            0x09 | 0x0A => (CharClass::SafeControl, true, CharAction::Accept), // TAB, LF

            // Forbidden control characters
            0x00..=0x08 | 0x0B..=0x1F | 0x7F => (CharClass::Forbidden, false, CharAction::Reject),

            // UTF-8 continuation bytes (0x80-0xBF)
            0x80..=0xBF => (CharClass::Utf8Cont, true, CharAction::Accept),

            // UTF-8 lead bytes (0xC0-0xFD)
            0xC0..=0xFD => (CharClass::Utf8Lead, true, CharAction::Accept),

            // Invalid UTF-8 bytes (0xFE-0xFF)
            0xFE..=0xFF => (CharClass::Forbidden, false, CharAction::Reject),
        }
    }

    /// Check if this character is allowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }

    /// Get the action for this character
    pub fn get_action(&self) -> &CharAction {
        &self.action
    }

    /// Get a human-readable description of the character
    pub fn description(&self) -> String {
        match self.class {
            CharClass::AsciiPrintable => format!(
                "ASCII printable character '{}' (0x{:02X})",
                char::from(self.byte),
                self.byte
            ),
            CharClass::SafeControl => match self.byte {
                0x09 => "Tab character (0x09)".to_string(),
                0x0A => "Line feed (0x0A)".to_string(),
                _ => format!("Safe control character (0x{:02X})", self.byte),
            },
            CharClass::Utf8Lead => format!("UTF-8 lead byte (0x{:02X})", self.byte),
            CharClass::Utf8Cont => format!("UTF-8 continuation byte (0x{:02X})", self.byte),
            CharClass::Forbidden => format!("Forbidden character (0x{:02X})", self.byte),
        }
    }
}

/// File validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationResult {
    /// Whether the file is valid
    pub is_valid: bool,
    /// Total bytes processed
    pub total_bytes: usize,
    /// Number of forbidden characters found
    pub forbidden_count: usize,
    /// Percentage of forbidden characters
    pub forbidden_percentage: f64,
    /// First forbidden character found (if any)
    pub first_forbidden: Option<CharacterContract>,
    /// All validation errors
    pub errors: Vec<String>,
    /// UTF-8 validation result
    pub utf8_valid: bool,
    /// Line ending normalization applied
    pub line_endings_normalized: bool,
}

impl Default for FileValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl FileValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            total_bytes: 0,
            forbidden_count: 0,
            forbidden_percentage: 0.0,
            first_forbidden: None,
            errors: Vec::new(),
            utf8_valid: true,
            line_endings_normalized: false,
        }
    }

    /// Add a forbidden character
    pub fn add_forbidden(&mut self, contract: CharacterContract) {
        self.forbidden_count += 1;
        if self.first_forbidden.is_none() {
            self.first_forbidden = Some(contract);
        }
        self.update_percentage();
    }

    /// Add an error message
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Update the forbidden percentage
    fn update_percentage(&mut self) {
        if self.total_bytes > 0 {
            self.forbidden_percentage =
                (self.forbidden_count as f64 / self.total_bytes as f64) * 100.0;
        }
    }

    /// Check if the file should be rejected based on binary heuristic
    pub fn is_binary_heuristic(&self) -> bool {
        self.forbidden_percentage > 30.0
    }

    /// Get a summary of the validation
    pub fn summary(&self) -> String {
        if self.is_valid {
            format!(
                "✅ File is valid ({} bytes, {:.1}% forbidden chars)",
                self.total_bytes, self.forbidden_percentage
            )
        } else {
            format!("❌ File is invalid: {}", self.errors.join(", "))
        }
    }
}

/// Character-level file validator
pub struct CharValidator {
    /// Whether to normalize line endings
    normalize_line_endings: bool,
    /// Whether to apply binary heuristic
    apply_binary_heuristic: bool,
    /// Binary threshold percentage
    binary_threshold: f64,
}

impl Default for CharValidator {
    fn default() -> Self {
        Self {
            normalize_line_endings: true,
            apply_binary_heuristic: true,
            binary_threshold: 30.0,
        }
    }
}

impl CharValidator {
    /// Create a new character validator
    pub fn new(
        normalize_line_endings: bool,
        apply_binary_heuristic: bool,
        binary_threshold: f64,
    ) -> Self {
        Self {
            normalize_line_endings,
            apply_binary_heuristic,
            binary_threshold,
        }
    }

    /// Validate file content using character-level contracts
    pub fn validate_file(&self, content: &[u8]) -> (FileValidationResult, Vec<u8>) {
        let mut result = FileValidationResult::new();
        let mut processed_content = Vec::new();

        result.total_bytes = content.len();

        // First, validate UTF-8
        match std::str::from_utf8(content) {
            Ok(_) => result.utf8_valid = true,
            Err(e) => {
                result.utf8_valid = false;
                result.add_error(format!("Invalid UTF-8: {e}"));
                return (result, processed_content);
            }
        }

        // Process each byte
        let mut i = 0;
        while i < content.len() {
            let byte = content[i];
            let contract = CharacterContract::from_byte(byte);

            // Check if character is allowed
            if !contract.is_allowed() {
                result.add_forbidden(contract);
                if result.first_forbidden.is_none() {
                    result.add_error(format!("Forbidden character found: 0x{byte:02X}"));
                }
            }

            // Handle line ending normalization
            if self.normalize_line_endings {
                match byte {
                    0x0D if i + 1 < content.len() && content[i + 1] == 0x0A => {
                        // CRLF -> LF
                        processed_content.push(0x0A);
                        i += 2;
                        result.line_endings_normalized = true;
                        continue;
                    }
                    0x0D => {
                        // CR -> LF
                        processed_content.push(0x0A);
                        i += 1;
                        result.line_endings_normalized = true;
                        continue;
                    }
                    _ => {
                        // Keep as-is
                        processed_content.push(byte);
                        i += 1;
                    }
                }
            } else {
                processed_content.push(byte);
                i += 1;
            }
        }

        // Apply binary heuristic
        if self.apply_binary_heuristic && result.is_binary_heuristic() {
            result.add_error(format!(
                "File appears to be binary ({}% forbidden characters > {}% threshold)",
                result.forbidden_percentage, self.binary_threshold
            ));
        }

        // Update final validity
        result.is_valid = result.errors.is_empty();

        (result, processed_content)
    }

    /// Validate a file and return detailed character analysis
    pub fn analyze_file(&self, content: &[u8]) -> Vec<CharacterContract> {
        content
            .iter()
            .map(|&byte| CharacterContract::from_byte(byte))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_classification() {
        // ASCII printable
        let contract = CharacterContract::from_byte(b'A');
        assert_eq!(contract.class, CharClass::AsciiPrintable);
        assert!(contract.is_allowed());
        assert_eq!(contract.get_action(), &CharAction::Accept);

        // Safe control
        let contract = CharacterContract::from_byte(b'\n');
        assert_eq!(contract.class, CharClass::SafeControl);
        assert!(contract.is_allowed());

        // Forbidden
        let contract = CharacterContract::from_byte(0x00);
        assert_eq!(contract.class, CharClass::Forbidden);
        assert!(!contract.is_allowed());
        assert_eq!(contract.get_action(), &CharAction::Reject);
    }

    #[test]
    fn test_file_validation() {
        let validator = CharValidator::default();

        // Valid file
        let content = b"Hello, World!\nThis is valid text.";
        let (result, _processed) = validator.validate_file(content);
        assert!(result.is_valid);
        assert_eq!(result.total_bytes, content.len());
        assert_eq!(result.forbidden_count, 0);

        // Invalid file with forbidden character
        let content = b"Hello\x00World";
        let (result, _processed) = validator.validate_file(content);
        assert!(!result.is_valid);
        assert_eq!(result.forbidden_count, 1);
    }

    #[test]
    fn test_line_ending_normalization() {
        let validator = CharValidator::new(true, false, 30.0);

        let content = b"Line 1\r\nLine 2\nLine 3\rLine 4";
        let (result, processed) = validator.validate_file(content);

        assert!(result.line_endings_normalized);
        assert_eq!(processed, b"Line 1\nLine 2\nLine 3\nLine 4");
    }
}
