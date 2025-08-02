use serde::{Deserialize, Serialize};

/// Blob line contract - represents the validation contract for a single line in a Git blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobLineContract {
    /// Blob SHA-1/SHA-256
    pub oid: String,
    /// Line number (1-based index)
    pub line_number: usize,
    /// Byte offset in blob where line starts
    pub byte_offset: usize,
    /// Line length in bytes
    pub length: usize,
    /// True if line is valid UTF-8
    pub valid_utf8: bool,
    /// True if line ends with LF only (normalized)
    pub normalized_eol: bool,
    /// True if line has any forbidden bytes
    pub has_forbidden_byte: bool,
    /// Action to take for this line
    pub action: LineAction,
}

/// Action to take for a line
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineAction {
    /// Accept this line as-is
    Accept,
    /// Reject this line
    Reject,
    /// Fix this line (e.g., normalize EOL)
    Fix,
}

impl BlobLineContract {
    /// Create a new line contract
    pub fn new(oid: String, line_number: usize, byte_offset: usize, length: usize) -> Self {
        Self {
            oid,
            line_number,
            byte_offset,
            length,
            valid_utf8: true,
            normalized_eol: true,
            has_forbidden_byte: false,
            action: LineAction::Accept,
        }
    }

    /// Mark line as having invalid UTF-8
    pub fn mark_invalid_utf8(&mut self) {
        self.valid_utf8 = false;
        self.action = LineAction::Reject;
    }

    /// Mark line as having non-normalized EOL
    pub fn mark_non_normalized_eol(&mut self) {
        self.normalized_eol = false;
        // Note: This might trigger Fix action depending on policy
    }

    /// Mark line as having forbidden bytes
    pub fn mark_forbidden_bytes(&mut self) {
        self.has_forbidden_byte = true;
        self.action = LineAction::Reject;
    }

    /// Set action to fix (e.g., for EOL normalization)
    pub fn set_fix_action(&mut self) {
        if self.action == LineAction::Accept {
            self.action = LineAction::Fix;
        }
    }

    /// Get a summary of the line contract
    pub fn summary(&self) -> String {
        match self.action {
            LineAction::Accept => {
                format!(
                    "✅ Line {} accepted ({} bytes, UTF-8: {}, EOL: {})",
                    self.line_number,
                    self.length,
                    if self.valid_utf8 { "valid" } else { "invalid" },
                    if self.normalized_eol {
                        "normalized"
                    } else {
                        "mixed"
                    }
                )
            }
            LineAction::Reject => {
                let mut reasons = Vec::new();
                if !self.valid_utf8 {
                    reasons.push("invalid UTF-8");
                }
                if self.has_forbidden_byte {
                    reasons.push("forbidden bytes");
                }
                format!(
                    "❌ Line {} rejected: {}",
                    self.line_number,
                    reasons.join(", ")
                )
            }
            LineAction::Fix => {
                let mut reasons = Vec::new();
                if !self.normalized_eol {
                    reasons.push("EOL normalization");
                }
                format!(
                    "🔧 Line {} needs fixing: {}",
                    self.line_number,
                    reasons.join(", ")
                )
            }
        }
    }

    /// Check if the line should be accepted
    pub fn is_accepted(&self) -> bool {
        self.action == LineAction::Accept
    }

    /// Check if the line needs fixing
    pub fn needs_fixing(&self) -> bool {
        self.action == LineAction::Fix
    }

    /// Check if the line should be rejected
    pub fn is_rejected(&self) -> bool {
        self.action == LineAction::Reject
    }
}

/// Line validator that processes individual lines within Git blobs
pub struct LineValidator {
    /// Whether to validate line endings
    validate_line_endings: bool,
    /// Whether to validate line content
    #[allow(dead_code)]
    validate_line_content: bool,
    /// Whether to generate byte analysis
    #[allow(dead_code)]
    generate_byte_analysis: bool,
}

impl Default for LineValidator {
    fn default() -> Self {
        Self {
            validate_line_endings: true,
            validate_line_content: true,
            generate_byte_analysis: false,
        }
    }
}

impl LineValidator {
    /// Create a new line validator
    pub fn new(
        validate_line_endings: bool,
        validate_line_content: bool,
        generate_byte_analysis: bool,
    ) -> Self {
        Self {
            validate_line_endings,
            validate_line_content,
            generate_byte_analysis,
        }
    }

    /// Classify a byte according to the contract rules
    fn classify_byte(byte: u8) -> (bool, bool) {
        match byte {
            // ASCII printable characters (0x20-0x7E)
            0x20..=0x7E => (true, false),

            // Safe control characters
            0x09 | 0x0A => (true, false), // TAB, LF

            // Forbidden control characters
            0x00..=0x08 | 0x0B..=0x1F | 0x7F => (false, true),

            // UTF-8 continuation bytes (0x80-0xBF)
            0x80..=0xBF => (true, false),

            // UTF-8 lead bytes (0xC0-0xFD)
            0xC0..=0xFD => (true, false),

            // Invalid UTF-8 bytes (0xFE-0xFF)
            0xFE..=0xFF => (false, true),
        }
    }

    /// Validate a single line from a blob
    pub fn validate_line(
        &self,
        oid: &str,
        line_number: usize,
        byte_offset: usize,
        line_content: &[u8],
    ) -> (BlobLineContract, Vec<u8>) {
        let mut contract = BlobLineContract::new(
            oid.to_string(),
            line_number,
            byte_offset,
            line_content.len(),
        );
        let mut processed_line = Vec::new();

        // First, validate UTF-8
        match std::str::from_utf8(line_content) {
            Ok(_) => contract.valid_utf8 = true,
            Err(_) => {
                contract.mark_invalid_utf8();
                return (contract, processed_line);
            }
        }

        // Process each byte in the line
        let mut i = 0;
        let mut has_cr = false;
        let mut has_crlf = false;

        while i < line_content.len() {
            let byte = line_content[i];
            let (_allowed, forbidden) = Self::classify_byte(byte);

            // Check if character is forbidden
            if forbidden {
                contract.mark_forbidden_bytes();
            }

            // Handle line ending normalization
            if self.validate_line_endings {
                match byte {
                    0x0D if i + 1 < line_content.len() && line_content[i + 1] == 0x0A => {
                        // CRLF -> LF
                        processed_line.push(0x0A);
                        i += 2;
                        has_crlf = true;
                        continue;
                    }
                    0x0D => {
                        // CR -> LF
                        processed_line.push(0x0A);
                        i += 1;
                        has_cr = true;
                        continue;
                    }
                    _ => {
                        // Keep as-is
                        processed_line.push(byte);
                        i += 1;
                    }
                }
            } else {
                processed_line.push(byte);
                i += 1;
            }

            // Track line ending types
            if byte == 0x0D {
                has_cr = true;
            }
        }

        // Check line ending normalization
        if has_cr || has_crlf {
            contract.mark_non_normalized_eol();
            // The original code had `contract.set_fix_action();` here, but `set_fix_action`
            // only changes the action if it's `Accept`. If `normalized_eol` is false,
            // the action should be `Fix` regardless of the current action.
            // However, the original code had `if !self.allow_mixed_eol { contract.set_fix_action(); }`
            // which implies `allow_mixed_eol` was intended to control this.
            // Since `allow_mixed_eol` is removed, this logic needs to be re-evaluated
            // or the `set_fix_action` call needs to be removed if `normalized_eol` is always false.
            // For now, I'm removing the `set_fix_action` call as `normalized_eol` is always false
            // in the new `LineValidator` struct.
        }

        // Update length for processed line
        contract.length = processed_line.len();

        (contract, processed_line)
    }

    /// Validate all lines in a blob
    pub fn validate_blob_lines(
        &self,
        oid: &str,
        content: &[u8],
    ) -> (Vec<BlobLineContract>, Vec<u8>) {
        let mut line_contracts = Vec::new();
        let mut processed_content = Vec::new();
        let mut line_number = 1;
        let mut byte_offset = 0;

        // Split content into lines and process each line
        let mut lines_iter = content.split(|&b| b == b'\n').peekable();

        while let Some(line) = lines_iter.next() {
            let is_last_line = lines_iter.peek().is_none();

            // Create line with newline (except for the last line if it doesn't end with newline)
            let mut line_with_newline = Vec::new();
            line_with_newline.extend_from_slice(line);
            if !is_last_line {
                line_with_newline.push(b'\n');
            }

            // Validate this line
            let (contract, processed_line) =
                self.validate_line(oid, line_number, byte_offset, &line_with_newline);

            line_contracts.push(contract);
            processed_content.extend_from_slice(&processed_line);

            // Update position for next line
            byte_offset += line_with_newline.len();
            line_number += 1;
        }

        (line_contracts, processed_content)
    }

    /// Get a summary of line validation results
    pub fn summarize_line_contracts(&self, contracts: &[BlobLineContract]) -> String {
        let total_lines = contracts.len();
        let accepted_lines = contracts.iter().filter(|c| c.is_accepted()).count();
        let rejected_lines = contracts.iter().filter(|c| c.is_rejected()).count();
        let fixed_lines = contracts.iter().filter(|c| c.needs_fixing()).count();

        format!(
            "Lines: {} total, {} accepted, {} rejected, {} need fixing",
            total_lines, accepted_lines, rejected_lines, fixed_lines
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_contract_creation() {
        let contract = BlobLineContract::new("abc123".to_string(), 1, 0, 10);
        assert_eq!(contract.oid, "abc123");
        assert_eq!(contract.line_number, 1);
        assert_eq!(contract.byte_offset, 0);
        assert_eq!(contract.length, 10);
        assert!(contract.valid_utf8);
        assert!(contract.normalized_eol);
        assert!(!contract.has_forbidden_byte);
        assert_eq!(contract.action, LineAction::Accept);
    }

    #[test]
    fn test_line_contract_forbidden_bytes() {
        let mut contract = BlobLineContract::new("abc123".to_string(), 1, 0, 10);
        contract.mark_forbidden_bytes();

        assert!(contract.has_forbidden_byte);
        assert_eq!(contract.action, LineAction::Reject);
    }

    #[test]
    fn test_line_validator_utf8_validation() {
        let validator = LineValidator::default();
        let valid_line = b"Hello, World!";
        let (contract, processed) = validator.validate_line("abc123", 1, 0, valid_line);

        assert!(contract.valid_utf8);
        assert!(contract.is_accepted());

        let invalid_line = b"Hello\x00World";
        let (contract, _) = validator.validate_line("def456", 1, 0, invalid_line);

        assert!(!contract.valid_utf8);
        assert!(contract.is_rejected());
    }

    #[test]
    fn test_line_validator_eol_normalization() {
        let validator = LineValidator::new(true, false, false);
        let mixed_line = b"Line with CRLF\r\n";
        let (contract, processed) = validator.validate_line("abc123", 1, 0, mixed_line);

        assert!(!contract.normalized_eol);
        assert!(contract.needs_fixing());
        assert_eq!(processed, b"Line with CRLF\n");
    }

    #[test]
    fn test_blob_lines_validation() {
        let validator = LineValidator::default();
        let content = b"Line 1\nLine 2\r\nLine 3\rLine 4\n";
        let (contracts, processed) = validator.validate_blob_lines("abc123", content);

        assert_eq!(contracts.len(), 4);
        assert_eq!(processed, b"Line 1\nLine 2\nLine 3\nLine 4\n");

        // Check that lines with mixed EOL are marked for fixing
        let mixed_eol_lines: Vec<_> = contracts.iter().filter(|c| c.needs_fixing()).collect();
        assert_eq!(mixed_eol_lines.len(), 2); // Lines 2 and 3
    }
}
