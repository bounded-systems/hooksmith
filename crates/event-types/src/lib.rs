//! Shared Event Types for Hooksmith
//!
//! This crate defines the event types used across the Hooksmith system for
//! communication between native Rust handlers and WIT components.
//!
//! The event system provides:
//! - Type-safe communication between components
//! - Schema validation for events
//! - Clear separation between system operations and pure computation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System events for operations that require native Rust capabilities
/// (file I/O, Git operations, process management, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SystemEvent {
    /// Read a file from the filesystem
    FileRead {
        /// Path to the file to read
        path: String,
        /// Whether to read as binary
        binary: Option<bool>,
    },

    /// Write content to a file
    FileWrite {
        /// Path to the file to write
        path: String,
        /// Content to write
        content: String,
        /// Whether to create parent directories
        create_dirs: Option<bool>,
    },

    /// Delete a file or directory
    FileDelete {
        /// Path to delete
        path: String,
        /// Whether to recursively delete directories
        recursive: Option<bool>,
    },

    /// Git commit operation
    GitCommit {
        /// Commit message
        message: String,
        /// Files to include in commit
        files: Vec<String>,
        /// Whether to stage all changes
        all: Option<bool>,
    },

    /// Git push operation
    GitPush {
        /// Remote name
        remote: String,
        /// Branch name
        branch: String,
        /// Whether to set upstream
        set_upstream: Option<bool>,
    },

    /// Git pull operation
    GitPull {
        /// Remote name
        remote: String,
        /// Branch name
        branch: String,
    },

    /// Execute a system command
    ProcessExecute {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
        /// Working directory
        cwd: Option<String>,
        /// Environment variables
        env: Option<HashMap<String, String>>,
    },

    /// Create a worktree
    WorktreeCreate {
        /// Branch name for the worktree
        branch: String,
        /// Base path for the worktree
        base_path: Option<String>,
    },

    /// Switch to a worktree
    WorktreeSwitch {
        /// Worktree name
        worktree: String,
    },

    /// Remove a worktree
    WorktreeRemove {
        /// Worktree name
        worktree: String,
        /// Whether to also remove the branch
        with_branch: Option<bool>,
    },

    /// Validate a file against a schema
    SchemaValidate {
        /// File path to validate
        file_path: String,
        /// Schema to validate against
        schema_path: String,
        /// Validation options
        options: Option<SchemaValidationOptions>,
    },

    /// Generate documentation
    DocsGenerate {
        /// Output directory
        output_dir: String,
        /// Documentation format
        format: Option<String>,
        /// Whether to include examples
        include_examples: Option<bool>,
    },
}

/// Options for schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationOptions {
    /// Whether to be strict about validation
    pub strict: Option<bool>,
    /// Whether to allow additional properties
    pub additional_properties: Option<bool>,
    /// Custom validation rules
    pub custom_rules: Option<HashMap<String, serde_json::Value>>,
}

/// Computation events for pure logic operations that can be handled by WIT components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ComputationEvent {
    /// Validate a contract
    ValidationRequest {
        /// Contract name
        contract_name: String,
        /// Contract content to validate
        content: String,
        /// Validation configuration
        config: Option<ValidationConfig>,
    },

    /// Result of contract validation
    ValidationResult {
        /// Contract name that was validated
        contract_name: String,
        /// Validation result
        result: ValidationResult,
    },

    /// Check a contract against rules
    ContractCheckRequest {
        /// Contract data to check
        data: String,
        /// Rules to check against
        rules: Vec<String>,
        /// Check configuration
        config: Option<ContractCheckConfig>,
    },

    /// Result of contract checking
    ContractCheckResult {
        /// Contract data that was checked
        data: String,
        /// Check result
        result: ContractCheckResult,
    },

    /// Calculate checksum for data
    ChecksumCalculate {
        /// Data to calculate checksum for
        data: String,
        /// Checksum algorithm
        algorithm: String,
    },

    /// Result of checksum calculation
    ChecksumResult {
        /// Original data
        data: String,
        /// Calculated checksum
        checksum: String,
        /// Algorithm used
        algorithm: String,
    },

    /// Transform data using a specific format
    DataTransform {
        /// Input data
        input: String,
        /// Input format
        input_format: String,
        /// Output format
        output_format: String,
        /// Transformation options
        options: Option<HashMap<String, serde_json::Value>>,
    },

    /// Result of data transformation
    TransformResult {
        /// Original input
        input: String,
        /// Transformed output
        output: String,
        /// Input format
        input_format: String,
        /// Output format
        output_format: String,
    },

    /// Evaluate a policy against data
    PolicyEvaluate {
        /// Policy to evaluate
        policy: String,
        /// Data to evaluate against
        data: String,
        /// Evaluation context
        context: Option<HashMap<String, serde_json::Value>>,
    },

    /// Result of policy evaluation
    PolicyResult {
        /// Policy that was evaluated
        policy: String,
        /// Evaluation result
        result: PolicyEvaluationResult,
    },
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to be strict about validation
    pub strict: Option<bool>,
    /// Whether to store proof of validation
    pub store_proof: Option<bool>,
    /// Maximum number of errors to report
    pub max_errors: Option<u32>,
    /// Custom validation rules
    pub custom_rules: Option<HashMap<String, serde_json::Value>>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation was successful
    pub success: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Additional details
    pub details: Option<String>,
    /// Validation metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Contract check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCheckConfig {
    /// Whether to be strict about checking
    pub strict: Option<bool>,
    /// Whether to include line numbers in violations
    pub include_line_numbers: Option<bool>,
    /// Custom check rules
    pub custom_rules: Option<HashMap<String, serde_json::Value>>,
}

/// Contract check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCheckResult {
    /// Whether the contract is valid
    pub valid: bool,
    /// Violations found
    pub violations: Vec<Violation>,
    /// Overall score (0.0 to 1.0)
    pub score: f64,
    /// Check metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Rule that was violated
    pub rule: String,
    /// Severity of the violation
    pub severity: ViolationSeverity,
    /// Violation message
    pub message: String,
    /// Line number where violation occurred
    pub line: Option<u32>,
    /// Column number where violation occurred
    pub column: Option<u32>,
    /// Additional context
    pub context: Option<String>,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Error - must be fixed
    Error,
    /// Warning - should be addressed
    Warning,
    /// Info - informational only
    Info,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    /// Whether the policy evaluation passed
    pub passed: bool,
    /// Policy decision
    pub decision: PolicyDecision,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Evaluation metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Policy decision types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// Allow the operation
    Allow,
    /// Deny the operation
    Deny,
    /// Require additional review
    Review,
    /// Conditional approval
    Conditional,
}

/// Main event type that can be either system or computation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", content = "event")]
pub enum HooksmithEvent {
    /// System event requiring native Rust capabilities
    System(SystemEvent),
    /// Computation event for pure logic operations
    Computation(ComputationEvent),
}

/// Event metadata for tracking and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub id: String,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Actor that emitted the event
    pub actor: String,
    /// Session ID for correlation
    pub session_id: Option<String>,
    /// Correlation ID for request/response pairs
    pub correlation_id: Option<String>,
    /// Event priority
    pub priority: Option<EventPriority>,
    /// Event tags for filtering
    pub tags: Option<Vec<String>>,
}

/// Event priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority - can be processed asynchronously
    Low,
    /// Normal priority - standard processing
    Normal,
    /// High priority - process before normal events
    High,
    /// Critical priority - process immediately
    Critical,
}

/// Complete event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Event payload
    pub payload: HooksmithEvent,
}

impl Event {
    /// Create a new event
    pub fn new(
        actor: String,
        payload: HooksmithEvent,
        session_id: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                actor,
                session_id,
                correlation_id: None,
                priority: Some(EventPriority::Normal),
                tags: None,
            },
            payload,
        }
    }

    /// Create a new system event
    pub fn system(
        actor: String,
        system_event: SystemEvent,
        session_id: Option<String>,
    ) -> Self {
        Self::new(actor, HooksmithEvent::System(system_event), session_id)
    }

    /// Create a new computation event
    pub fn computation(
        actor: String,
        computation_event: ComputationEvent,
        session_id: Option<String>,
    ) -> Self {
        Self::new(actor, HooksmithEvent::Computation(computation_event), session_id)
    }

    /// Set correlation ID for request/response pairs
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Set event priority
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.metadata.priority = Some(priority);
        self
    }

    /// Add tags for filtering
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = Some(tags);
        self
    }
}

/// Event handler trait for processing events
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle(&self, event: &Event) -> anyhow::Result<()>;

    /// Get the event types this handler can process
    fn supported_events(&self) -> Vec<String>;

    /// Get the handler name
    fn name(&self) -> &str;
}

/// Event handler registration
#[derive(Debug, Clone)]
pub struct HandlerRegistration {
    /// Handler name
    pub name: String,
    /// Supported event types
    pub supported_events: Vec<String>,
    /// Handler priority (lower numbers = higher priority)
    pub priority: u32,
}

impl HandlerRegistration {
    /// Create a new handler registration
    pub fn new(name: String, supported_events: Vec<String>, priority: u32) -> Self {
        Self {
            name,
            supported_events,
            priority,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_event_serialization() {
        let event = SystemEvent::FileRead {
            path: "/tmp/test.txt".to_string(),
            binary: Some(false),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: SystemEvent = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            SystemEvent::FileRead { path, binary } => {
                assert_eq!(path, "/tmp/test.txt");
                assert_eq!(binary, Some(false));
            }
            _ => panic!("Expected FileRead event"),
        }
    }

    #[test]
    fn test_computation_event_serialization() {
        let event = ComputationEvent::ValidationRequest {
            contract_name: "test-contract".to_string(),
            content: r#"{"name": "test"}"#.to_string(),
            config: Some(ValidationConfig {
                strict: Some(true),
                store_proof: Some(false),
                max_errors: Some(10),
                custom_rules: None,
            }),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ComputationEvent = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            ComputationEvent::ValidationRequest { contract_name, content, config } => {
                assert_eq!(contract_name, "test-contract");
                assert_eq!(content, r#"{"name": "test"}"#);
                assert!(config.is_some());
            }
            _ => panic!("Expected ValidationRequest event"),
        }
    }

    #[test]
    fn test_event_creation() {
        let system_event = SystemEvent::FileRead {
            path: "/tmp/test.txt".to_string(),
            binary: None,
        };

        let event = Event::system("test-actor".to_string(), system_event, Some("session-123".to_string()));
        
        assert_eq!(event.metadata.actor, "test-actor");
        assert!(event.metadata.session_id.is_some());
        assert_eq!(event.metadata.session_id.unwrap(), "session-123");
        
        match event.payload {
            HooksmithEvent::System(SystemEvent::FileRead { path, .. }) => {
                assert_eq!(path, "/tmp/test.txt");
            }
            _ => panic!("Expected system event"),
        }
    }

    #[test]
    fn test_event_with_correlation_id() {
        let computation_event = ComputationEvent::ValidationRequest {
            contract_name: "test".to_string(),
            content: "test".to_string(),
            config: None,
        };

        let event = Event::computation("test-actor".to_string(), computation_event, None)
            .with_correlation_id("corr-123".to_string());

        assert_eq!(event.metadata.correlation_id, Some("corr-123".to_string()));
    }
} 