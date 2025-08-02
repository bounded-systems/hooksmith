use crate::tree_contract::{TreeEntryContract, TreeObjectContract};
use serde::{Deserialize, Serialize};

/// Git object contract - discriminated union for different Git object types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GitObjectContract {
    /// Blob contract - represents the entire file as stored in Git
    Blob(BlobContract),
    /// Tree contract - represents a Git tree object
    Tree(TreeObjectContract),
    // Future: Commit, Tag
    // Commit(CommitContract),
    // Tag(TagContract),
}

/// Blob contract - represents the entire file as stored in Git
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobContract {
    /// SHA-1/SHA-256 hash
    pub id: String,
    /// Size in bytes
    pub size: usize,
    /// Encoding (only UTF-8 for now)
    pub encoding: String,
    /// Lines split by \n
    pub lines: Vec<String>,
    /// Validation result
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl BlobContract {
    /// Create a new blob contract
    pub fn new(id: String, size: usize, content: &[u8]) -> Self {
        // Validate UTF-8 encoding
        let (encoding, lines, valid, errors) = match std::str::from_utf8(content) {
            Ok(text) => {
                let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                ("utf-8".to_string(), lines, true, Vec::new())
            }
            Err(e) => {
                let errors = vec![format!("Invalid UTF-8: {}", e)];
                ("invalid".to_string(), Vec::new(), false, errors)
            }
        };

        Self {
            id,
            size,
            encoding,
            lines,
            valid,
            errors,
        }
    }

    /// Get a summary of the blob contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Blob {} accepted ({} bytes, {} lines, encoding: {})",
                &self.id[..self.id.len().min(8)],
                self.size,
                self.lines.len(),
                self.encoding
            )
        } else {
            format!(
                "❌ Blob {} rejected: {}",
                &self.id[..self.id.len().min(8)],
                self.errors.join(", ")
            )
        }
    }

    /// Check if the blob is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Blob line contract - represents a single line in a blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobLineContract {
    /// Line number (1-based)
    pub line_number: usize,
    /// Line text content
    pub text: String,
    /// Whether the line is valid
    pub valid: bool,
    /// Validation errors for this line
    pub errors: Vec<String>,
    /// Line action (accept/reject/fix)
    pub action: LineAction,
}

impl BlobLineContract {
    /// Create a new line contract
    pub fn new(line_number: usize, text: String) -> Self {
        let (valid, errors, action) = Self::validate_line(&text);

        Self {
            line_number,
            text,
            valid,
            errors,
            action,
        }
    }

    /// Validate a line according to character rules
    fn validate_line(text: &str) -> (bool, Vec<String>, LineAction) {
        let mut errors = Vec::new();
        let mut has_forbidden_chars = false;
        let mut has_mixed_eol = false;

        // Check for forbidden characters
        for (i, ch) in text.chars().enumerate() {
            if !Self::is_allowed_character(ch) {
                has_forbidden_chars = true;
                errors.push(format!("Forbidden character at position {}: {:?}", i, ch));
            }
        }

        // Check for mixed line endings
        if text.contains('\r') {
            has_mixed_eol = true;
            errors.push("Mixed line endings detected".to_string());
        }

        // Determine action
        let action = if has_forbidden_chars {
            LineAction::Reject
        } else if has_mixed_eol {
            LineAction::Fix
        } else {
            LineAction::Accept
        };

        let valid = action == LineAction::Accept;

        (valid, errors, action)
    }

    /// Check if a character is allowed
    fn is_allowed_character(ch: char) -> bool {
        match ch {
            // ASCII printable characters
            '\x20'..='\x7E' => true,
            // Safe control characters
            '\t' | '\n' => true,
            // All other characters are forbidden
            _ => false,
        }
    }

    /// Get a summary of the line contract
    pub fn summary(&self) -> String {
        match self.action {
            LineAction::Accept => {
                format!(
                    "✅ Line {} accepted ({} chars)",
                    self.line_number,
                    self.text.len()
                )
            }
            LineAction::Reject => {
                format!(
                    "❌ Line {} rejected: {}",
                    self.line_number,
                    self.errors.join(", ")
                )
            }
            LineAction::Fix => {
                format!(
                    "🔧 Line {} needs fixing: {}",
                    self.line_number,
                    self.errors.join(", ")
                )
            }
        }
    }

    /// Check if the line is valid
    pub fn is_valid(&self) -> bool {
        self.valid
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

/// Line action for blob line contracts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineAction {
    /// Accept this line as-is
    Accept,
    /// Reject this line
    Reject,
    /// Fix this line (e.g., normalize EOL)
    Fix,
}

/// Diff line type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineType {
    /// Context line (unchanged)
    Context,
    /// Added line
    Add,
    /// Removed line
    Remove,
}

/// Diff line in a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Type of diff line
    pub line_type: DiffLineType,
    /// Line content
    pub content: String,
    /// Whether the line is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl DiffLine {
    /// Create a new diff line
    pub fn new(line_type: DiffLineType, content: String) -> Self {
        let (valid, errors) = Self::validate_content(&content);

        Self {
            line_type,
            content,
            valid,
            errors,
        }
    }

    /// Validate line content
    fn validate_content(content: &str) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        // Check for forbidden characters
        for (i, ch) in content.chars().enumerate() {
            if !BlobLineContract::is_allowed_character(ch) {
                errors.push(format!("Forbidden character at position {}: {:?}", i, ch));
            }
        }

        let valid = errors.is_empty();
        (valid, errors)
    }
}

/// Blob chunk contract - represents a diff hunk (set of lines that changed together)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobChunkContract {
    /// Diff header (e.g., "@@ -1,3 +1,4 @@")
    pub header: String,
    /// Starting line number in old version
    pub old_start: usize,
    /// Number of lines in old version
    pub old_lines: usize,
    /// Starting line number in new version
    pub new_start: usize,
    /// Number of lines in new version
    pub new_lines: usize,
    /// Lines in this chunk
    pub lines: Vec<DiffLine>,
    /// Whether the chunk is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl BlobChunkContract {
    /// Create a new chunk contract
    pub fn new(
        header: String,
        old_start: usize,
        old_lines: usize,
        new_start: usize,
        new_lines: usize,
        lines: Vec<DiffLine>,
    ) -> Self {
        let (valid, errors) = Self::validate_chunk(&lines);

        Self {
            header,
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines,
            valid,
            errors,
        }
    }

    /// Validate a chunk
    fn validate_chunk(lines: &[DiffLine]) -> (bool, Vec<String>) {
        let mut errors = Vec::new();
        let mut invalid_lines = 0;

        for (i, line) in lines.iter().enumerate() {
            if !line.valid {
                invalid_lines += 1;
                errors.extend(line.errors.iter().map(|e| format!("Line {}: {}", i + 1, e)));
            }
        }

        let valid = invalid_lines == 0;
        (valid, errors)
    }

    /// Get a summary of the chunk contract
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "✅ Chunk {} valid ({} lines, {} old, {} new)",
                self.header,
                self.lines.len(),
                self.old_lines,
                self.new_lines
            )
        } else {
            format!(
                "❌ Chunk {} invalid: {}",
                self.header,
                self.errors.join(", ")
            )
        }
    }

    /// Check if the chunk is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Git object validator that processes different types of Git objects
pub struct GitObjectValidator {
    /// Whether to validate lines individually
    validate_lines: bool,
    /// Whether to validate chunks
    validate_chunks: bool,
    /// Whether to validate tree entries
    validate_tree_entries: bool,
}

impl Default for GitObjectValidator {
    fn default() -> Self {
        Self {
            validate_lines: true,
            validate_chunks: false,
            validate_tree_entries: true,
        }
    }
}

impl GitObjectValidator {
    /// Create a new Git object validator
    pub fn new(validate_lines: bool, validate_chunks: bool, validate_tree_entries: bool) -> Self {
        Self {
            validate_lines,
            validate_chunks,
            validate_tree_entries,
        }
    }

    /// Validate a blob and return the contract
    pub fn validate_blob(&self, id: &str, content: &[u8]) -> BlobContract {
        BlobContract::new(id.to_string(), content.len(), content)
    }

    /// Validate all lines in a blob
    pub fn validate_blob_lines(&self, blob: &BlobContract) -> Vec<BlobLineContract> {
        if !self.validate_lines {
            return Vec::new();
        }

        blob.lines
            .iter()
            .enumerate()
            .map(|(i, line)| BlobLineContract::new(i + 1, line.clone()))
            .collect()
    }

    /// Create a chunk contract from diff lines
    pub fn create_chunk_contract(
        &self,
        header: &str,
        old_start: usize,
        old_lines: usize,
        new_start: usize,
        new_lines: usize,
        diff_lines: Vec<(DiffLineType, String)>,
    ) -> BlobChunkContract {
        let lines: Vec<DiffLine> = diff_lines
            .into_iter()
            .map(|(line_type, content)| DiffLine::new(line_type, content))
            .collect();

        BlobChunkContract::new(
            header.to_string(),
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines,
        )
    }

    /// Validate a complete Git object
    pub fn validate_git_object(&self, id: &str, content: &[u8]) -> GitObjectContract {
        let blob = self.validate_blob(id, content);
        GitObjectContract::Blob(blob)
    }

    /// Validate a tree object
    pub fn validate_tree_object(
        &self,
        id: &str,
        entries: Vec<TreeEntryContract>,
    ) -> GitObjectContract {
        let tree = TreeObjectContract::new(id.to_string(), entries);
        GitObjectContract::Tree(tree)
    }

    /// Get a summary of validation results
    pub fn summarize_validation(&self, blob: &BlobContract, lines: &[BlobLineContract]) -> String {
        let line_summary = if self.validate_lines {
            let total_lines = lines.len();
            let accepted_lines = lines.iter().filter(|l| l.is_valid()).count();
            let rejected_lines = lines.iter().filter(|l| l.is_rejected()).count();
            let fixed_lines = lines.iter().filter(|l| l.needs_fixing()).count();

            format!(
                "Lines: {} total, {} accepted, {} rejected, {} need fixing",
                total_lines, accepted_lines, rejected_lines, fixed_lines
            )
        } else {
            "Line validation disabled".to_string()
        };

        format!("Blob: {} | {}", blob.summary(), line_summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_contract_creation() {
        let content = b"Hello, World!\nThis is a test.\n";
        let contract = BlobContract::new("abc123".to_string(), content.len(), content);

        assert_eq!(contract.id, "abc123");
        assert_eq!(contract.size, content.len());
        assert_eq!(contract.encoding, "utf-8");
        assert_eq!(contract.lines.len(), 2);
        assert!(contract.is_valid());
    }

    #[test]
    fn test_blob_contract_invalid_utf8() {
        let content = b"Hello\x80World";
        let contract = BlobContract::new("def456".to_string(), content.len(), content);

        assert_eq!(contract.encoding, "invalid");
        assert!(!contract.is_valid());
        assert!(!contract.errors.is_empty());
    }

    #[test]
    fn test_line_contract_creation() {
        let contract = BlobLineContract::new(1, "Hello, World!".to_string());

        assert_eq!(contract.line_number, 1);
        assert_eq!(contract.text, "Hello, World!");
        assert!(contract.is_valid());
        assert_eq!(contract.action, LineAction::Accept);
    }

    #[test]
    fn test_line_contract_forbidden_chars() {
        let contract = BlobLineContract::new(1, "Hello\x00World".to_string());

        assert!(!contract.is_valid());
        assert_eq!(contract.action, LineAction::Reject);
        assert!(!contract.errors.is_empty());
    }

    #[test]
    fn test_line_contract_mixed_eol() {
        let contract = BlobLineContract::new(1, "Hello\r\nWorld".to_string());

        assert!(!contract.is_valid());
        assert_eq!(contract.action, LineAction::Fix);
        assert!(!contract.errors.is_empty());
    }

    #[test]
    fn test_chunk_contract_creation() {
        let lines = vec![
            DiffLine::new(DiffLineType::Context, "Line 1".to_string()),
            DiffLine::new(DiffLineType::Add, "Line 2".to_string()),
            DiffLine::new(DiffLineType::Remove, "Line 3".to_string()),
        ];

        let contract = BlobChunkContract::new("@@ -1,2 +1,3 @@".to_string(), 1, 2, 1, 3, lines);

        assert_eq!(contract.header, "@@ -1,2 +1,3 @@");
        assert_eq!(contract.old_start, 1);
        assert_eq!(contract.old_lines, 2);
        assert_eq!(contract.new_start, 1);
        assert_eq!(contract.new_lines, 3);
        assert_eq!(contract.lines.len(), 3);
        assert!(contract.is_valid());
    }

    #[test]
    fn test_git_object_validator() {
        let validator = GitObjectValidator::default();
        let content = b"Line 1\nLine 2\n";
        let blob = validator.validate_blob("abc123", content);

        assert!(blob.is_valid());
        assert_eq!(blob.lines.len(), 2);

        let lines = validator.validate_blob_lines(&blob);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| l.is_valid()));
    }
}
