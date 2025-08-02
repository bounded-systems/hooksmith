use crate::actions::{ActionResolver, GitOperation, HookAction};
use crate::blob_contract::{BlobByteAudit, BlobContract, BlobValidator};
use crate::contract::{CharValidator, FileValidationResult};
use crate::error::FilterError;
use crate::line_contract::{BlobLineContract, LineAction, LineValidator};
use crate::state::FileState;
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::{debug, error, info};

/// A filter driver that can process files based on Git attributes
pub trait FilterDriver {
    /// Process a file with the given content and file state
    fn process(
        &self,
        content: &[u8],
        file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError>;

    /// Get the name of this filter driver
    fn name(&self) -> &str;
}

/// Safe ASCII filter that validates file content contains only safe characters
pub struct SafeAsciiFilter {
    /// Whether to normalize line endings
    normalize_eol: bool,
    /// Whether to allow tabs
    allow_tabs: bool,
    /// Whether to allow newlines
    allow_newlines: bool,
    /// Whether to allow spaces
    allow_spaces: bool,
    /// Whether to allow printable ASCII
    allow_printable: bool,
}

impl Default for SafeAsciiFilter {
    fn default() -> Self {
        Self {
            normalize_eol: true,
            allow_tabs: true,
            allow_newlines: true,
            allow_spaces: true,
            allow_printable: true,
        }
    }
}

impl SafeAsciiFilter {
    /// Create a new SafeAsciiFilter with custom settings
    pub fn new(
        normalize_eol: bool,
        allow_tabs: bool,
        allow_newlines: bool,
        allow_spaces: bool,
        allow_printable: bool,
    ) -> Self {
        Self {
            normalize_eol,
            allow_tabs,
            allow_newlines,
            allow_spaces,
            allow_printable,
        }
    }

    /// Validate that content contains only safe ASCII characters
    fn validate_safe_ascii(&self, content: &[u8]) -> Result<(), FilterError> {
        for (i, &byte) in content.iter().enumerate() {
            let is_valid = match byte {
                0x09 if self.allow_tabs => true,             // TAB
                0x0A if self.allow_newlines => true,         // LF
                0x20 if self.allow_spaces => true,           // SPACE
                0x21..=0x7E if self.allow_printable => true, // Printable ASCII
                _ => false,
            };

            if !is_valid {
                error!("Invalid character 0x{:02X} at position {}", byte, i);
                return Err(FilterError::InvalidCharacter);
            }
        }
        Ok(())
    }

    /// Normalize line endings to LF
    fn normalize_eol(&self, content: &[u8]) -> Vec<u8> {
        if !self.normalize_eol {
            return content.to_vec();
        }

        let mut normalized = Vec::new();
        let mut i = 0;

        while i < content.len() {
            match content[i] {
                0x0D if i + 1 < content.len() && content[i + 1] == 0x0A => {
                    // CRLF -> LF
                    normalized.push(0x0A);
                    i += 2;
                }
                0x0D => {
                    // CR -> LF
                    normalized.push(0x0A);
                    i += 1;
                }
                _ => {
                    // Keep as-is
                    normalized.push(content[i]);
                    i += 1;
                }
            }
        }

        normalized
    }
}

impl FilterDriver for SafeAsciiFilter {
    fn process(
        &self,
        content: &[u8],
        _file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError> {
        debug!(
            "Processing file with SafeAsciiFilter for operation {:?}",
            operation
        );

        // Normalize line endings first
        let normalized = self.normalize_eol(content);

        // Validate safe ASCII characters
        self.validate_safe_ascii(&normalized)?;

        info!("Successfully processed file with SafeAsciiFilter");
        Ok(normalized)
    }

    fn name(&self) -> &str {
        "safe-ascii"
    }
}

/// Character contract filter that uses the Dev Contract system for precise validation
pub struct CharContractFilter {
    /// Character validator
    validator: CharValidator,
}

impl Default for CharContractFilter {
    fn default() -> Self {
        Self {
            validator: CharValidator::default(),
        }
    }
}

impl CharContractFilter {
    /// Create a new CharContractFilter with custom settings
    pub fn new(
        normalize_line_endings: bool,
        apply_binary_heuristic: bool,
        binary_threshold: f64,
    ) -> Self {
        Self {
            validator: CharValidator::new(
                normalize_line_endings,
                apply_binary_heuristic,
                binary_threshold,
            ),
        }
    }
}

impl FilterDriver for CharContractFilter {
    fn process(
        &self,
        content: &[u8],
        _file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError> {
        debug!(
            "Processing file with CharContractFilter for operation {:?}",
            operation
        );

        // Validate file using character contracts
        let (result, processed_content) = self.validator.validate_file(content);

        if !result.is_valid {
            error!("File validation failed: {}", result.summary());
            return Err(FilterError::InvalidCharacter);
        }

        info!(
            "Successfully processed file with CharContractFilter: {}",
            result.summary()
        );
        Ok(processed_content)
    }

    fn name(&self) -> &str {
        "char-contract"
    }
}

/// Blob contract filter that validates Git blobs using the contract system
pub struct BlobContractFilter {
    /// Blob validator
    validator: BlobValidator,
    /// Whether to generate audit records
    generate_audit: bool,
}

impl Default for BlobContractFilter {
    fn default() -> Self {
        Self {
            validator: BlobValidator::default(),
            generate_audit: false,
        }
    }
}

impl BlobContractFilter {
    /// Create a new BlobContractFilter with custom settings
    pub fn new(
        normalize_line_endings: bool,
        apply_binary_heuristic: bool,
        binary_threshold: f64,
        generate_audit: bool,
    ) -> Self {
        Self {
            validator: BlobValidator::new(
                normalize_line_endings,
                apply_binary_heuristic,
                binary_threshold,
                generate_audit,
            ),
            generate_audit,
        }
    }

    /// Generate a mock OID for testing (in real usage, this would come from Git)
    fn generate_mock_oid(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl FilterDriver for BlobContractFilter {
    fn process(
        &self,
        content: &[u8],
        _file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError> {
        debug!(
            "Processing blob with BlobContractFilter for operation {:?}",
            operation
        );

        // Generate a mock OID (in real usage, this would be the actual Git blob OID)
        let oid = self.generate_mock_oid(content);

        // Validate blob using contract system
        let (contract, processed_content, audit_records) =
            self.validator.validate_blob(&oid, content);

        // Log the contract result
        info!("Blob contract result: {}", contract.summary());

        // If audit records were generated, log them
        if self.generate_audit && !audit_records.is_empty() {
            debug!(
                "Generated {} audit records for blob {}",
                audit_records.len(),
                oid
            );

            // Log forbidden bytes if any
            let forbidden_audits: Vec<_> = audit_records
                .iter()
                .filter(|audit| !audit.allowed)
                .collect();

            if !forbidden_audits.is_empty() {
                error!(
                    "Found {} forbidden bytes in blob {}: {:?}",
                    forbidden_audits.len(),
                    oid,
                    forbidden_audits
                        .iter()
                        .map(|a| a.description())
                        .collect::<Vec<_>>()
                );
            }
        }

        // Check if blob should be accepted
        if !contract.is_accepted() {
            error!("Blob {} rejected: {}", oid, contract.summary());
            return Err(FilterError::InvalidCharacter);
        }

        info!(
            "Successfully processed blob {} with BlobContractFilter",
            oid
        );
        Ok(processed_content)
    }

    fn name(&self) -> &str {
        "blob-contract"
    }
}

/// Combined blob and line contract filter that validates both blob-level and line-level contracts
pub struct CombinedContractFilter {
    /// Blob validator
    blob_validator: BlobValidator,
    /// Line validator
    line_validator: LineValidator,
    /// Whether to generate line contracts
    generate_line_contracts: bool,
}

impl Default for CombinedContractFilter {
    fn default() -> Self {
        Self {
            blob_validator: BlobValidator::default(),
            line_validator: LineValidator::default(),
            generate_line_contracts: true,
        }
    }
}

impl CombinedContractFilter {
    /// Create a new CombinedContractFilter with custom settings
    pub fn new(
        normalize_line_endings: bool,
        apply_binary_heuristic: bool,
        binary_threshold: f64,
        allow_mixed_eol: bool,
        generate_line_contracts: bool,
    ) -> Self {
        Self {
            blob_validator: BlobValidator::new(
                normalize_line_endings,
                apply_binary_heuristic,
                binary_threshold,
                false, // Don't generate byte audits by default
            ),
            line_validator: LineValidator::new(
                normalize_line_endings,
                allow_mixed_eol,
                false, // Don't generate byte analysis by default
            ),
            generate_line_contracts,
        }
    }

    /// Generate a mock OID for testing (in real usage, this would come from Git)
    fn generate_mock_oid(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl FilterDriver for CombinedContractFilter {
    fn process(
        &self,
        content: &[u8],
        _file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError> {
        debug!(
            "Processing blob with CombinedContractFilter for operation {:?}",
            operation
        );

        // Generate a mock OID (in real usage, this would be the actual Git blob OID)
        let oid = self.generate_mock_oid(content);

        // First, validate at blob level
        let (blob_contract, processed_content, _) =
            self.blob_validator.validate_blob(&oid, content);

        // Log the blob contract result
        info!("Blob contract result: {}", blob_contract.summary());

        // If blob is rejected, stop here
        if !blob_contract.is_accepted() {
            error!("Blob {} rejected: {}", oid, blob_contract.summary());
            return Err(FilterError::InvalidCharacter);
        }

        // Then, validate at line level if requested
        if self.generate_line_contracts {
            let (line_contracts, _) = self.line_validator.validate_blob_lines(&oid, content);

            // Log line contract summary
            let line_summary = self
                .line_validator
                .summarize_line_contracts(&line_contracts);
            info!("Line contracts: {}", line_summary);

            // Check if any lines are rejected
            let rejected_lines: Vec<_> =
                line_contracts.iter().filter(|c| c.is_rejected()).collect();

            if !rejected_lines.is_empty() {
                error!(
                    "Found {} rejected lines in blob {}",
                    rejected_lines.len(),
                    oid
                );
                for line_contract in &rejected_lines {
                    error!("  {}", line_contract.summary());
                }
                return Err(FilterError::InvalidCharacter);
            }

            // Log lines that need fixing
            let fixed_lines: Vec<_> = line_contracts.iter().filter(|c| c.needs_fixing()).collect();

            if !fixed_lines.is_empty() {
                info!(
                    "Found {} lines that need fixing in blob {}",
                    fixed_lines.len(),
                    oid
                );
                for line_contract in &fixed_lines {
                    info!("  {}", line_contract.summary());
                }
            }

            // Log detailed line contracts for debugging
            debug!("Detailed line contracts for blob {}:", oid);
            for line_contract in &line_contracts {
                debug!("  {}", line_contract.summary());
            }
        }

        info!(
            "Successfully processed blob {} with CombinedContractFilter",
            oid
        );
        Ok(processed_content)
    }

    fn name(&self) -> &str {
        "combined-contract"
    }
}

/// A filter that can handle multiple filter drivers
pub struct MultiFilter {
    drivers: HashMap<String, Box<dyn FilterDriver>>,
    action_resolver: ActionResolver,
}

impl MultiFilter {
    /// Create a new MultiFilter
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            action_resolver: ActionResolver::new(),
        }
    }

    /// Add a filter driver
    pub fn add_driver(&mut self, name: &str, driver: Box<dyn FilterDriver>) {
        self.drivers.insert(name.to_string(), driver);
    }

    /// Process a file using the appropriate filter drivers based on file state
    pub fn process_file(
        &self,
        content: &[u8],
        file_state: &FileState,
        operation: &GitOperation,
    ) -> Result<Vec<u8>, FilterError> {
        let mut processed_content = content.to_vec();

        // Resolve actions for this file state and operation
        let actions = self.action_resolver.resolve_actions(file_state, operation);

        debug!(
            "Resolved {} actions for operation {:?}",
            actions.len(),
            operation
        );

        for action in actions {
            match action {
                HookAction::ValidateSafeAscii => {
                    if let Some(driver) = self.drivers.get("safe-ascii") {
                        processed_content =
                            driver.process(&processed_content, file_state, operation)?;
                    } else {
                        // Use default SafeAsciiFilter
                        let default_filter = SafeAsciiFilter::default();
                        processed_content =
                            default_filter.process(&processed_content, file_state, operation)?;
                    }
                }
                HookAction::NormalizeEol { target_eol } => {
                    debug!("Normalizing EOL to {}", target_eol);
                    processed_content = self.normalize_eol(&processed_content, &target_eol)?;
                }
                HookAction::EnforceEncoding { encoding } => {
                    debug!("Enforcing encoding {}", encoding);
                    processed_content = self.enforce_encoding(&processed_content, &encoding)?;
                }
                HookAction::RunFilterDriver {
                    driver_name,
                    operation: _,
                } => {
                    if let Some(driver) = self.drivers.get(&driver_name) {
                        processed_content =
                            driver.process(&processed_content, file_state, operation)?;
                    } else {
                        return Err(FilterError::DriverError(format!(
                            "Filter driver '{}' not found",
                            driver_name
                        )));
                    }
                }
                _ => {
                    debug!("Skipping unsupported action: {:?}", action);
                }
            }
        }

        Ok(processed_content)
    }

    /// Normalize end-of-line characters
    fn normalize_eol(&self, content: &[u8], target_eol: &str) -> Result<Vec<u8>, FilterError> {
        let mut normalized = Vec::new();
        let mut i = 0;

        while i < content.len() {
            match content[i] {
                0x0D if i + 1 < content.len() && content[i + 1] == 0x0A => {
                    // CRLF -> target EOL
                    match target_eol {
                        "lf" => normalized.push(0x0A),
                        "crlf" => {
                            normalized.push(0x0D);
                            normalized.push(0x0A);
                        }
                        _ => {
                            normalized.push(0x0D);
                            normalized.push(0x0A);
                        }
                    }
                    i += 2;
                }
                0x0D => {
                    // CR -> target EOL
                    match target_eol {
                        "lf" => normalized.push(0x0A),
                        "crlf" => {
                            normalized.push(0x0D);
                            normalized.push(0x0A);
                        }
                        _ => normalized.push(0x0A),
                    }
                    i += 1;
                }
                0x0A => {
                    // LF -> target EOL
                    match target_eol {
                        "lf" => normalized.push(0x0A),
                        "crlf" => {
                            normalized.push(0x0D);
                            normalized.push(0x0A);
                        }
                        _ => normalized.push(0x0A),
                    }
                    i += 1;
                }
                _ => {
                    normalized.push(content[i]);
                    i += 1;
                }
            }
        }

        Ok(normalized)
    }

    /// Enforce encoding
    fn enforce_encoding(&self, content: &[u8], encoding: &str) -> Result<Vec<u8>, FilterError> {
        // For now, just validate UTF-8
        if encoding.to_lowercase() == "utf-8" {
            std::str::from_utf8(content)
                .map(|_| content.to_vec())
                .map_err(|e| FilterError::InvalidEncoding(format!("Invalid UTF-8: {}", e)))
        } else {
            // For other encodings, we'd need additional libraries
            // For now, just return the content as-is
            Ok(content.to_vec())
        }
    }
}

/// Run the filter as a Git process filter
pub fn run_process_filter() -> Result<(), FilterError> {
    // For now, implement a simple stdin/stdout filter
    // This avoids the complex gix-filter API issues
    let mut filter = MultiFilter::new();

    // Add the safe-ascii filter driver
    filter.add_driver("safe-ascii", Box::new(SafeAsciiFilter::default()));

    // Read from stdin
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;

    // Process the content
    let file_state = FileState::default();
    let operation = GitOperation::Add;

    let processed = filter.process_file(&input, &file_state, &operation)?;

    // Write to stdout
    std::io::stdout().write_all(&processed)?;

    Ok(())
}
