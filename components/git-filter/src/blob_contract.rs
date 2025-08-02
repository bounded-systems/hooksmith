use serde::{Deserialize, Serialize};

/// Git blob contract - represents the validation contract for a single Git blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobContract {
    /// Blob SHA-1/SHA-256
    pub oid: String,
    /// Blob size in bytes
    pub size: usize,
    /// True if UTF-8 valid
    pub valid_utf8: bool,
    /// True if LF-only (normalized line endings)
    pub normalized_eol: bool,
    /// True if any forbidden character found
    pub has_forbidden_byte: bool,
    /// Positions of forbidden bytes (if any)
    pub forbidden_byte_positions: Option<Vec<usize>>,
    /// Git attributes for this blob (e.g., ["linguist-generated=true", "-diff"])
    pub attributes: Option<Vec<String>>,
    /// Action to take (accept/reject)
    pub action: BlobAction,
}

/// Action to take for a blob
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobAction {
    /// Accept this blob
    Accept,
    /// Reject this blob
    Reject,
}

impl BlobContract {
    /// Create a new blob contract
    pub fn new(oid: String, size: usize) -> Self {
        Self {
            oid,
            size,
            valid_utf8: true,
            normalized_eol: true,
            has_forbidden_byte: false,
            forbidden_byte_positions: None,
            attributes: None,
            action: BlobAction::Accept,
        }
    }

    /// Create a new blob contract with attributes
    pub fn new_with_attributes(oid: String, size: usize, attributes: Option<Vec<String>>) -> Self {
        Self {
            oid,
            size,
            valid_utf8: true,
            normalized_eol: true,
            has_forbidden_byte: false,
            forbidden_byte_positions: None,
            attributes,
            action: BlobAction::Accept,
        }
    }

    /// Mark blob as having forbidden bytes
    pub fn add_forbidden_byte(&mut self, position: usize) {
        self.has_forbidden_byte = true;
        if self.action == BlobAction::Accept {
            self.action = BlobAction::Reject;
        }

        if let Some(ref mut positions) = self.forbidden_byte_positions {
            positions.push(position);
        } else {
            self.forbidden_byte_positions = Some(vec![position]);
        }
    }

    /// Mark blob as having invalid UTF-8
    pub fn mark_invalid_utf8(&mut self) {
        self.valid_utf8 = false;
        self.action = BlobAction::Reject;
    }

    /// Mark blob as having non-normalized line endings
    pub fn mark_non_normalized_eol(&mut self) {
        self.normalized_eol = false;
        // Note: This doesn't automatically reject, just marks as non-normalized
    }

    /// Add attributes to the blob contract
    pub fn add_attributes(&mut self, attributes: Vec<String>) {
        self.attributes = Some(attributes);
    }

    /// Check if this blob has a specific attribute
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

    /// Validate attributes for a blob (called when processing tree entries)
    pub fn validate_attributes_for_path(&mut self, filepath: &str) -> bool {
        let mut valid = true;

        // Check if this is a generated file that should have linguist-generated=true
        if Self::is_generated_file(filepath) {
            if !self.has_attribute("linguist-generated=true") {
                // Mark as rejected if generated file doesn't have the required attribute
                self.action = BlobAction::Reject;
                valid = false;
            }
        } else {
            // Check if non-generated file incorrectly has linguist-generated=true
            if self.has_attribute("linguist-generated=true") {
                // This is a warning but doesn't necessarily reject the blob
                // Could be logged or handled differently
            }
        }

        valid
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

    /// Get a summary of the blob contract
    pub fn summary(&self) -> String {
        if self.action == BlobAction::Accept {
            let mut summary = format!(
                "✅ Blob {} accepted ({} bytes, UTF-8: {}, EOL: {})",
                &self.oid[..8],
                self.size,
                if self.valid_utf8 { "valid" } else { "invalid" },
                if self.normalized_eol {
                    "normalized"
                } else {
                    "mixed"
                }
            );

            if let Some(ref attrs) = self.attributes {
                if !attrs.is_empty() {
                    summary.push_str(&format!(" [attributes: {}]", attrs.join(", ")));
                }
            }

            summary
        } else {
            let mut reasons = Vec::new();
            if !self.valid_utf8 {
                reasons.push("invalid UTF-8");
            }
            if self.has_forbidden_byte {
                reasons.push("forbidden bytes");
            }
            if let Some(ref attrs) = self.attributes {
                if !attrs.is_empty() {
                    reasons.push("attribute validation failed");
                }
            }
            format!(
                "❌ Blob {} rejected: {}",
                &self.oid[..8],
                reasons.join(", ")
            )
        }
    }

    /// Check if the blob should be accepted
    pub fn is_accepted(&self) -> bool {
        self.action == BlobAction::Accept
    }
}

/// Per-byte audit record for detailed byte-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobByteAudit {
    /// Blob SHA
    pub oid: String,
    /// Byte index in blob
    pub offset: usize,
    /// Raw byte value
    pub byte: u8,
    /// Classification of this byte
    pub class: ByteClass,
    /// Whether this byte is allowed
    pub allowed: bool,
}

/// Byte classification for audit records
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ByteClass {
    /// ASCII printable character
    Ascii,
    /// UTF-8 lead byte
    Utf8Lead,
    /// UTF-8 continuation byte
    Utf8Cont,
    /// Safe control character (tab, LF)
    SafeControl,
    /// Forbidden character
    Forbidden,
}

impl BlobByteAudit {
    /// Create a new byte audit record
    pub fn new(oid: String, offset: usize, byte: u8, class: ByteClass, allowed: bool) -> Self {
        Self {
            oid,
            offset,
            byte,
            class,
            allowed,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self.class {
            ByteClass::Ascii => format!("ASCII '{}' (0x{:02X})", char::from(self.byte), self.byte),
            ByteClass::SafeControl => match self.byte {
                0x09 => "Tab (0x09)".to_string(),
                0x0A => "Line feed (0x0A)".to_string(),
                _ => format!("Safe control (0x{:02X})", self.byte),
            },
            ByteClass::Utf8Lead => format!("UTF-8 lead (0x{:02X})", self.byte),
            ByteClass::Utf8Cont => format!("UTF-8 continuation (0x{:02X})", self.byte),
            ByteClass::Forbidden => format!("Forbidden (0x{:02X})", self.byte),
        }
    }
}

/// Blob validator that processes Git blobs using contracts
pub struct BlobValidator {
    /// Whether to normalize line endings
    normalize_line_endings: bool,
    /// Whether to apply binary heuristic
    apply_binary_heuristic: bool,
    /// Binary threshold percentage
    binary_threshold: f64,
    /// Whether to generate byte audit records
    generate_audit: bool,
}

impl Default for BlobValidator {
    fn default() -> Self {
        Self {
            normalize_line_endings: true,
            apply_binary_heuristic: true,
            binary_threshold: 30.0,
            generate_audit: false,
        }
    }
}

impl BlobValidator {
    /// Create a new blob validator
    pub fn new(
        normalize_line_endings: bool,
        apply_binary_heuristic: bool,
        binary_threshold: f64,
        generate_audit: bool,
    ) -> Self {
        Self {
            normalize_line_endings,
            apply_binary_heuristic,
            binary_threshold,
            generate_audit,
        }
    }

    /// Classify a byte according to the contract rules
    fn classify_byte(byte: u8) -> (ByteClass, bool) {
        match byte {
            // ASCII printable characters (0x20-0x7E)
            0x20..=0x7E => (ByteClass::Ascii, true),

            // Safe control characters
            0x09 | 0x0A => (ByteClass::SafeControl, true), // TAB, LF

            // Forbidden control characters
            0x00..=0x08 | 0x0B..=0x1F | 0x7F => (ByteClass::Forbidden, false),

            // UTF-8 continuation bytes (0x80-0xBF)
            0x80..=0xBF => (ByteClass::Utf8Cont, true),

            // UTF-8 lead bytes (0xC0-0xFD)
            0xC0..=0xFD => (ByteClass::Utf8Lead, true),

            // Invalid UTF-8 bytes (0xFE-0xFF)
            0xFE..=0xFF => (ByteClass::Forbidden, false),
        }
    }

    /// Validate a Git blob using the contract system
    pub fn validate_blob(
        &self,
        oid: &str,
        content: &[u8],
    ) -> (BlobContract, Vec<u8>, Vec<BlobByteAudit>) {
        let mut contract = BlobContract::new(oid.to_string(), content.len());
        let mut processed_content = Vec::new();
        let mut audit_records = Vec::new();

        // First, validate UTF-8
        match std::str::from_utf8(content) {
            Ok(_) => contract.valid_utf8 = true,
            Err(_) => {
                contract.mark_invalid_utf8();
                return (contract, processed_content, audit_records);
            }
        }

        // Process each byte
        let mut i = 0;
        let mut has_cr = false;
        let mut has_crlf = false;

        while i < content.len() {
            let byte = content[i];
            let (class, allowed) = Self::classify_byte(byte);

            // Generate audit record if requested
            if self.generate_audit {
                audit_records.push(BlobByteAudit::new(
                    oid.to_string(),
                    i,
                    byte,
                    class.clone(),
                    allowed,
                ));
            }

            // Check if character is forbidden
            if !allowed {
                contract.add_forbidden_byte(i);
            }

            // Handle line ending normalization
            if self.normalize_line_endings {
                match byte {
                    0x0D if i + 1 < content.len() && content[i + 1] == 0x0A => {
                        // CRLF -> LF
                        processed_content.push(0x0A);
                        i += 2;
                        has_crlf = true;
                        continue;
                    }
                    0x0D => {
                        // CR -> LF
                        processed_content.push(0x0A);
                        i += 1;
                        has_cr = true;
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

            // Track line ending types
            if byte == 0x0D {
                has_cr = true;
            }
        }

        // Mark if line endings are not normalized
        if has_cr || has_crlf {
            contract.mark_non_normalized_eol();
        }

        // Apply binary heuristic
        if self.apply_binary_heuristic {
            let forbidden_percentage = if contract.size > 0 {
                (contract
                    .forbidden_byte_positions
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0) as f64
                    / contract.size as f64)
                    * 100.0
            } else {
                0.0
            };

            if forbidden_percentage > self.binary_threshold {
                contract.action = BlobAction::Reject;
            }
        }

        (contract, processed_content, audit_records)
    }

    /// Validate a blob and return only the contract (no audit records)
    pub fn validate_blob_simple(&self, oid: &str, content: &[u8]) -> (BlobContract, Vec<u8>) {
        let (contract, processed, _) = self.validate_blob(oid, content);
        (contract, processed)
    }

    /// Get detailed byte analysis for a blob
    pub fn analyze_blob_bytes(&self, oid: &str, content: &[u8]) -> Vec<BlobByteAudit> {
        let mut audit_records = Vec::new();

        for (i, &byte) in content.iter().enumerate() {
            let (class, allowed) = Self::classify_byte(byte);
            audit_records.push(BlobByteAudit::new(oid.to_string(), i, byte, class, allowed));
        }

        audit_records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_contract_creation() {
        let contract = BlobContract::new("abc123".to_string(), 100);
        assert_eq!(contract.oid, "abc123");
        assert_eq!(contract.size, 100);
        assert!(contract.valid_utf8);
        assert!(contract.normalized_eol);
        assert!(!contract.has_forbidden_byte);
        assert_eq!(contract.action, BlobAction::Accept);
    }

    #[test]
    fn test_blob_contract_forbidden_bytes() {
        let mut contract = BlobContract::new("abc123".to_string(), 100);
        contract.add_forbidden_byte(5);
        contract.add_forbidden_byte(10);

        assert!(contract.has_forbidden_byte);
        assert_eq!(contract.action, BlobAction::Reject);
        assert_eq!(contract.forbidden_byte_positions, Some(vec![5, 10]));
    }

    #[test]
    fn test_blob_validator_utf8_validation() {
        let validator = BlobValidator::default();
        let valid_content = b"Hello, World!\n";
        let (contract, processed, _) = validator.validate_blob("abc123", valid_content);

        assert!(contract.valid_utf8);
        assert!(contract.is_accepted());

        let invalid_content = b"Hello\x00World";
        let (contract, _, _) = validator.validate_blob("def456", invalid_content);

        assert!(!contract.valid_utf8);
        assert!(!contract.is_accepted());
    }

    #[test]
    fn test_blob_validator_line_endings() {
        let validator = BlobValidator::new(true, false, 30.0, false);
        let mixed_content = b"Line 1\r\nLine 2\nLine 3\r";
        let (contract, processed, _) = validator.validate_blob("abc123", mixed_content);

        assert!(!contract.normalized_eol);
        assert_eq!(processed, b"Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_byte_audit_creation() {
        let audit = BlobByteAudit::new("abc123".to_string(), 5, b'A', ByteClass::Ascii, true);

        assert_eq!(audit.oid, "abc123");
        assert_eq!(audit.offset, 5);
        assert_eq!(audit.byte, b'A');
        assert_eq!(audit.class, ByteClass::Ascii);
        assert!(audit.allowed);
    }
}
