use crate::modules::contract_state_machine::{ContractState, TransitionEvent};
use crate::modules::hierarchical_validation::{ValidationNote, ValidationScope};
use anyhow::{anyhow, Result};
use chrono::Utc;
use jsonschema::{Draft, JSONSchema};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Contract validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// Validation errors if any
    pub errors: Vec<ValidationError>,
    /// Validation warnings if any
    pub warnings: Vec<ValidationWarning>,
    /// Schema hash for verification
    pub schema_hash: String,
    /// Content hash for verification
    pub content_hash: String,
    /// Timestamp of validation
    pub timestamp: String,
    /// Tool that performed validation
    pub validated_by: String,
    /// Contract type being validated
    pub contract_type: String,
    /// Validation metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Validation error with detailed information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Path to the error in the data
    pub path: Option<String>,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

/// Validation warning with detailed information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Path to the warning in the data
    pub path: Option<String>,
    /// Additional warning details
    pub details: Option<serde_json::Value>,
}

/// Contract definition with schema and validation rules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractDefinition {
    /// Contract name/identifier
    pub name: String,
    /// Contract version
    pub version: String,
    /// Contract description
    pub description: Option<String>,
    /// JSON Schema for validation
    pub schema: serde_json::Value,
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Contract metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Validation rule for contract enforcement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Rule type
    pub rule_type: RuleType,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Whether rule is required
    pub required: bool,
    /// Rule severity
    pub severity: RuleSeverity,
}

/// Types of validation rules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RuleType {
    /// JSON Schema validation
    JsonSchema,
    /// Pattern matching (regex)
    Pattern,
    /// File size limits
    FileSize,
    /// File extension validation
    FileExtension,
    /// Custom validation function
    Custom,
}

/// Severity levels for validation rules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RuleSeverity {
    /// Information only
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical error
    Critical,
}

/// Contract validator that uses JSON Schema and Git notes
#[derive(Debug)]
pub struct ContractValidator {
    /// Raw JSON schemas for validation
    schemas: HashMap<String, serde_json::Value>,
    /// Contract definitions
    contracts: HashMap<String, ContractDefinition>,
    /// Git notes manager for proof storage
    git_notes_manager: GitNotesManager,
}

impl ContractValidator {
    /// Create a new contract validator
    pub fn new() -> Result<Self> {
        Ok(Self {
            schemas: HashMap::new(),
            contracts: HashMap::new(),
            git_notes_manager: GitNotesManager::new()?,
        })
    }

    /// Register a contract definition with JSON Schema
    pub fn register_contract(&mut self, contract: ContractDefinition) -> Result<()> {
        // Store the contract and raw schema
        let contract_name = contract.name.clone();
        self.contracts
            .insert(contract_name.clone(), contract.clone());
        self.schemas.insert(contract_name, contract.schema.clone());
        Ok(())
    }

    /// Validate data against a contract
    pub fn validate_contract(
        &self,
        contract_name: &str,
        data: &serde_json::Value,
        file_path: &str,
    ) -> Result<ContractValidationResult> {
        let contract = self
            .contracts
            .get(contract_name)
            .ok_or_else(|| anyhow!("Contract '{}' not found", contract_name))?;

        let mut result = ContractValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            schema_hash: self.compute_schema_hash(contract)?,
            content_hash: self.compute_content_hash(data)?,
            timestamp: Utc::now().to_rfc3339(),
            validated_by: format!("hooksmith {}", env!("CARGO_PKG_VERSION")),
            contract_type: contract_name.to_string(),
            metadata: HashMap::new(),
        };

        // Validate against JSON Schema
        if let Some(schema) = self.schemas.get(contract_name) {
            let schema_leaked: &'static serde_json::Value = Box::leak(Box::new(schema.clone()));
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(schema_leaked)?;
            let errors: Vec<_> = compiled
                .validate(data)
                .err()
                .map(|e| e.collect())
                .unwrap_or_default();
            if !errors.is_empty() {
                result.is_valid = false;
                for error in errors {
                    result.errors.push(ValidationError {
                        code: "JSON_SCHEMA_ERROR".to_string(),
                        message: error.to_string(),
                        path: Some(error.instance_path.to_string()),
                        details: Some(serde_json::json!({
                            "error": error.to_string(),
                            "instance_path": error.instance_path.to_string(),
                        })),
                    });
                }
            }
        }

        // Apply custom validation rules
        for rule in &contract.rules {
            self.apply_validation_rule(rule, data, file_path, &mut result)?;
        }

        Ok(result)
    }

    /// Apply a validation rule to the data
    fn apply_validation_rule(
        &self,
        rule: &ValidationRule,
        data: &serde_json::Value,
        file_path: &str,
        result: &mut ContractValidationResult,
    ) -> Result<()> {
        match &rule.rule_type {
            RuleType::Pattern => {
                if let Some(pattern) = rule.parameters.get("pattern").and_then(|p| p.as_str()) {
                    let regex = regex::Regex::new(pattern)?;
                    if let Some(content) = data.as_str() {
                        if !regex.is_match(content) {
                            let error = ValidationError {
                                code: "PATTERN_MISMATCH".to_string(),
                                message: format!("Content does not match pattern: {pattern}"),
                                path: None,
                                details: Some(serde_json::json!({
                                    "pattern": pattern,
                                    "content_preview": &content[..content.len().min(100)]
                                })),
                            };

                            match rule.severity {
                                RuleSeverity::Error | RuleSeverity::Critical => {
                                    result.errors.push(error);
                                    result.is_valid = false;
                                }
                                RuleSeverity::Warning => result.warnings.push(ValidationWarning {
                                    code: error.code,
                                    message: error.message,
                                    path: error.path,
                                    details: error.details,
                                }),
                                RuleSeverity::Info => {
                                    // Just log info, don't add to result
                                }
                            }
                        }
                    }
                }
            }
            RuleType::FileSize => {
                if let Some(max_size) = rule.parameters.get("max_size").and_then(|s| s.as_u64()) {
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        let file_size = metadata.len();
                        if file_size > max_size {
                            let error = ValidationError {
                                code: "FILE_SIZE_EXCEEDED".to_string(),
                                message: format!(
                                    "File size {file_size} exceeds maximum {max_size}"
                                ),
                                path: None,
                                details: Some(serde_json::json!({
                                    "file_size": file_size,
                                    "max_size": max_size
                                })),
                            };

                            match rule.severity {
                                RuleSeverity::Error | RuleSeverity::Critical => {
                                    result.errors.push(error);
                                    result.is_valid = false;
                                }
                                RuleSeverity::Warning => result.warnings.push(ValidationWarning {
                                    code: error.code,
                                    message: error.message,
                                    path: error.path,
                                    details: error.details,
                                }),
                                RuleSeverity::Info => {}
                            }
                        }
                    }
                }
            }
            RuleType::FileExtension => {
                if let Some(extensions) =
                    rule.parameters.get("extensions").and_then(|e| e.as_array())
                {
                    if let Some(extension) = Path::new(file_path).extension() {
                        let ext_str = extension.to_string_lossy();
                        let allowed = extensions
                            .iter()
                            .any(|ext| ext.as_str().map(|e| e == ext_str).unwrap_or(false));

                        if !allowed {
                            let error = ValidationError {
                                code: "INVALID_FILE_EXTENSION".to_string(),
                                message: format!("File extension '{ext_str}' not allowed"),
                                path: None,
                                details: Some(serde_json::json!({
                                    "extension": ext_str,
                                    "allowed_extensions": extensions
                                })),
                            };

                            match rule.severity {
                                RuleSeverity::Error | RuleSeverity::Critical => {
                                    result.errors.push(error);
                                    result.is_valid = false;
                                }
                                RuleSeverity::Warning => result.warnings.push(ValidationWarning {
                                    code: error.code,
                                    message: error.message,
                                    path: error.path,
                                    details: error.details,
                                }),
                                RuleSeverity::Info => {}
                            }
                        }
                    }
                }
            }
            _ => {
                // Custom rules would be implemented here
            }
        }

        Ok(())
    }

    /// Compute SHA-256 hash of schema
    fn compute_schema_hash(&self, contract: &ContractDefinition) -> Result<String> {
        let schema_json = serde_json::to_string(&contract.schema)?;
        let mut hasher = Sha256::new();
        hasher.update(schema_json.as_bytes());
        let hash = hasher.finalize();
        Ok(format!("sha256:{hash:x}"))
    }

    /// Compute SHA-256 hash of content
    fn compute_content_hash(&self, data: &serde_json::Value) -> Result<String> {
        let content_json = serde_json::to_string(data)?;
        let mut hasher = Sha256::new();
        hasher.update(content_json.as_bytes());
        let hash = hasher.finalize();
        Ok(format!("sha256:{hash:x}"))
    }

    /// Store validation result as Git note
    pub fn store_validation_proof(
        &self,
        file_path: &str,
        result: &ContractValidationResult,
    ) -> Result<()> {
        let note_content = serde_json::to_string_pretty(result)?;
        self.git_notes_manager.add_note(file_path, &note_content)?;
        Ok(())
    }

    /// Verify validation proof from Git note
    pub fn verify_validation_proof(&self, file_path: &str, expected_hash: &str) -> Result<bool> {
        let note_content = self.git_notes_manager.get_note(file_path)?;
        let stored_result: ContractValidationResult = serde_json::from_str(&note_content)?;

        // Verify the hash matches
        Ok(stored_result.content_hash == expected_hash)
    }

    /// Generate JSON Schema for a Rust type
    pub fn generate_schema<T: JsonSchema>() -> serde_json::Value {
        let schema = schema_for!(T);
        serde_json::to_value(&schema).unwrap()
    }

    /// Create a validation note for the contract state machine
    pub fn create_validation_note(
        &self,
        file: &str,
        scope: ValidationScope,
        state: &ContractState,
        event: &TransitionEvent,
        hash: &str,
        validation_result: &ContractValidationResult,
    ) -> ValidationNote {
        let mut metadata = HashMap::new();
        metadata.insert("state".to_string(), serde_json::json!(format!("{state:?}")));
        metadata.insert("event".to_string(), serde_json::json!(format!("{event:?}")));
        metadata.insert(
            "schema_hash".to_string(),
            serde_json::json!(validation_result.schema_hash),
        );
        metadata.insert(
            "contract_type".to_string(),
            serde_json::json!(validation_result.contract_type),
        );
        metadata.insert(
            "validated_by".to_string(),
            serde_json::json!(validation_result.validated_by),
        );

        ValidationNote {
            scope: format!("{scope:?}"),
            file: file.to_string(),
            range: None,
            hash: hash.to_string(),
            parent_scope: None,
            parent_hash: None,
            child_scopes: Vec::new(),
            validated: validation_result.is_valid,
            validation_errors: validation_result
                .errors
                .iter()
                .map(
                    |e| crate::modules::hierarchical_validation::ValidationError {
                        message: e.message.clone(),
                        severity: "error".to_string(),
                        line: None,
                        char: None,
                    },
                )
                .collect(),
            contract_type: validation_result.contract_type.clone(),
            tool: validation_result.validated_by.clone(),
            timestamp: Utc::now().to_rfc3339(),
            commit_hash: None,
            validation_duration_ms: 0,
            metadata,
        }
    }
}

/// Git notes manager for storing validation proofs
#[derive(Debug)]
#[allow(dead_code)]
pub struct GitNotesManager {
    /// Git repository path
    repo_path: String,
}

impl GitNotesManager {
    /// Create a new Git notes manager
    pub fn new() -> Result<Self> {
        let repo_path = std::env::current_dir()?.to_string_lossy().to_string();
        Ok(Self { repo_path })
    }

    /// Add a note for a file
    pub fn add_note(&self, file_path: &str, content: &str) -> Result<()> {
        let _note_ref = format!("refs/notes/validation/{}", file_path.replace('/', "_"));

        // This is a simplified implementation
        // In a real implementation, you would use git2 or similar to actually create Git notes
        println!("Would add Git note for {file_path}: {content}");

        Ok(())
    }

    /// Get a note for a file
    pub fn get_note(&self, file_path: &str) -> Result<String> {
        let _note_ref = format!("refs/notes/validation/{}", file_path.replace('/', "_"));

        // This is a simplified implementation
        // In a real implementation, you would use git2 or similar to actually read Git notes
        println!("Would get Git note for {file_path}");

        // Return empty string for now - in real implementation this would read from Git notes
        Ok("{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_contract_validation() {
        let mut validator = ContractValidator::new().unwrap();

        // Create a simple contract definition
        let contract = ContractDefinition {
            name: "test_contract".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test contract".to_string()),
            schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "age": { "type": "integer", "minimum": 0 }
                },
                "required": ["name", "age"]
            }),
            rules: vec![ValidationRule {
                name: "name_length".to_string(),
                description: Some("Name must be at least 2 characters".to_string()),
                rule_type: RuleType::Pattern,
                parameters: json!({
                    "pattern": "^.{2,}$"
                })
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect(),
                required: true,
                severity: RuleSeverity::Error,
            }],
            metadata: HashMap::new(),
        };

        validator.register_contract(contract).unwrap();

        // Test valid data
        let valid_data = json!({
            "name": "John Doe",
            "age": 30
        });

        let result = validator
            .validate_contract("test_contract", &valid_data, "test.json")
            .unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        // Test invalid data
        let invalid_data = json!({
            "name": "J", // Too short
            "age": 30
        });

        let result = validator
            .validate_contract("test_contract", &invalid_data, "test.json")
            .unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_schema_generation() {
        let schema = ContractValidator::generate_schema::<ContractValidationResult>();
        assert!(schema.is_object());
        assert_eq!(schema["title"], "ContractValidationResult");
    }
}
