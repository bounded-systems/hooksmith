//! Git + Lefthook Integration Demo
//!
//! This example demonstrates how to use the Git + Lefthook integration
//! with event-driven state machine and SARIF integration for contract validation.
//!
//! ## Features Demonstrated
//!
//! - **Structured Git Events**: Wrapping git commit/push with JSONL events
//! - **Lefthook Integration**: Capturing and normalizing Lefthook outputs
//! - **State Machine Integration**: Mapping events to state transitions
//! - **SARIF Integration**: Emitting contract violations as SARIF results
//! - **Event Blocking**: Dependency relationships between validation rules

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Simplified Git workflow states for demonstration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitWorkflowState {
    /// Initial state - no Git operations in progress
    IDLE,
    /// Git commit operation started
    COMMITTING,
    /// Lefthook hooks running after commit
    HookRunning,
    /// Commit completed successfully
    COMMITTED,
    /// Git push operation started
    PUSHING,
    /// Push completed successfully
    PUSHED,
    /// Error state - operation failed
    ERROR,
}

/// Simplified Git workflow events that trigger state transitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitWorkflowEvent {
    /// Git commit started
    CommitStarted,
    /// Lefthook hook started
    HookStarted,
    /// Lefthook hook completed
    HookCompleted,
    /// Commit completed
    CommitCompleted,
    /// Git push started
    PushStarted,
    /// Git push completed
    PushCompleted,
    /// Operation failed
    OperationFailed,
}

/// Simplified contract validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Contract ID
    pub contract_id: String,
    /// File being validated
    pub file: String,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<Utc>,
}

/// Simplified Git + Lefthook integration for demonstration
#[derive(Debug)]
pub struct GitLefthookIntegration {
    /// Current workflow state
    current_state: GitWorkflowState,
    /// Session ID for grouping related events
    session_id: String,
    /// Contract validation results
    validation_results: Vec<ContractValidationResult>,
}

impl GitLefthookIntegration {
    /// Create a new Git + Lefthook integration instance
    pub fn new() -> Self {
        Self {
            current_state: GitWorkflowState::IDLE,
            session_id: format!("demo-{}", chrono::Utc::now().timestamp()),
            validation_results: Vec::new(),
        }
    }

    /// Get the current workflow state
    pub fn current_state(&self) -> &GitWorkflowState {
        &self.current_state
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Transition to a new state based on an event
    pub fn transition(
        &mut self,
        event: GitWorkflowEvent,
        _context: serde_json::Value,
    ) -> Result<()> {
        self.current_state = match (&self.current_state, &event) {
            (GitWorkflowState::IDLE, GitWorkflowEvent::CommitStarted) => {
                GitWorkflowState::COMMITTING
            }
            (GitWorkflowState::COMMITTING, GitWorkflowEvent::HookStarted) => {
                GitWorkflowState::HookRunning
            }
            (GitWorkflowState::HookRunning, GitWorkflowEvent::HookCompleted) => {
                GitWorkflowState::COMMITTED
            }
            (GitWorkflowState::COMMITTED, GitWorkflowEvent::PushStarted) => {
                GitWorkflowState::PUSHING
            }
            (GitWorkflowState::PUSHING, GitWorkflowEvent::PushCompleted) => {
                GitWorkflowState::PUSHED
            }
            (_, GitWorkflowEvent::OperationFailed) => GitWorkflowState::ERROR,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid state transition: {:?} -> {:?}",
                    self.current_state,
                    event
                ));
            }
        };
        Ok(())
    }

    /// Add a validation result
    pub fn add_validation_result(&mut self, result: ContractValidationResult) {
        self.validation_results.push(result);
    }

    /// Get validation results
    pub fn validation_results(&self) -> &[ContractValidationResult] {
        &self.validation_results
    }
}

impl Default for GitLefthookIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Git + Lefthook Integration Demo");
    println!("===================================");

    // Initialize the integration
    let mut integration = GitLefthookIntegration::new();
    println!("✅ Initialized Git + Lefthook integration");
    println!("   Session ID: {}", integration.session_id());
    println!("   Current State: {:?}", integration.current_state());

    // Demonstrate state transitions
    println!("\n🔄 State Machine Transitions");
    println!("----------------------------");

    // Simulate commit started
    let commit_context = serde_json::json!({
        "message": "feat: add contract validation system",
        "files": ["src/contract.rs", "src/validation.rs"],
        "branch": "feature/contract-validation"
    });
    integration.transition(GitWorkflowEvent::CommitStarted, commit_context)?;
    println!("   Commit Started → {:?}", integration.current_state());

    // Simulate hook started
    let hook_context = serde_json::json!({
        "hook": "post-commit",
        "command": "cargo test",
        "files": ["src/contract.rs", "src/validation.rs"]
    });
    integration.transition(GitWorkflowEvent::HookStarted, hook_context)?;
    println!("   Hook Started → {:?}", integration.current_state());

    // Simulate hook completed
    let hook_complete_context = serde_json::json!({
        "exit_code": 0,
        "duration_ms": 1250,
        "output": "running 12 tests\ntest result: ok. 12 passed; 0 failed"
    });
    integration.transition(GitWorkflowEvent::HookCompleted, hook_complete_context)?;
    println!("   Hook Completed → {:?}", integration.current_state());

    // Simulate commit completed
    let commit_complete_context = serde_json::json!({
        "hash": "a1b2c3d4e5f6",
        "insertions": 45,
        "deletions": 12
    });
    integration.transition(GitWorkflowEvent::CommitCompleted, commit_complete_context)?;
    println!("   Commit Completed → {:?}", integration.current_state());

    // Simulate push started
    let push_context = serde_json::json!({
        "remote": "origin",
        "branch": "feature/contract-validation"
    });
    integration.transition(GitWorkflowEvent::PushStarted, push_context)?;
    println!("   Push Started → {:?}", integration.current_state());

    // Simulate push completed
    let push_complete_context = serde_json::json!({
        "objects": 8,
        "deltas": 3,
        "remote": "origin"
    });
    integration.transition(GitWorkflowEvent::PushCompleted, push_complete_context)?;
    println!("   Push Completed → {:?}", integration.current_state());

    // Demonstrate contract validation
    println!("\n📋 Contract Validation");
    println!("---------------------");

    // Add some example validation results
    let validation_result = ContractValidationResult {
        is_valid: true,
        contract_id: "file-extension-policy".to_string(),
        file: "src/contract.rs".to_string(),
        errors: vec![],
        warnings: vec!["Consider adding more comprehensive tests".to_string()],
        timestamp: Utc::now(),
    };
    integration.add_validation_result(validation_result);

    let validation_result2 = ContractValidationResult {
        is_valid: false,
        contract_id: "code-style-policy".to_string(),
        file: "src/validation.rs".to_string(),
        errors: vec!["Line 45: Function name should be snake_case".to_string()],
        warnings: vec![],
        timestamp: Utc::now(),
    };
    integration.add_validation_result(validation_result2);

    // Display validation results
    for result in integration.validation_results() {
        let status = if result.is_valid { "✅" } else { "❌" };
        println!("   {} {} ({})", status, result.contract_id, result.file);

        for error in &result.errors {
            println!("      ❌ Error: {}", error);
        }

        for warning in &result.warnings {
            println!("      ⚠️  Warning: {}", warning);
        }
    }

    // Demonstrate SARIF integration concept
    println!("\n🔍 SARIF Integration");
    println!("-------------------");
    println!("   📄 SARIF results would be generated for external tooling");
    println!("   🔗 Integration with CodeQL, SonarQube, etc.");
    println!("   📊 Structured reporting for CI/CD pipelines");

    // Demonstrate event blocking concept
    println!("\n🚫 Event Blocking");
    println!("----------------");
    println!("   🔒 Contract violations can block subsequent operations");
    println!("   ⛓️  Dependency relationships between validation rules");
    println!("   🎯 Granular control over workflow progression");

    println!("\n✅ Demo completed successfully!");
    println!("   Final State: {:?}", integration.current_state());
    println!(
        "   Total Validation Results: {}",
        integration.validation_results().len()
    );

    Ok(())
}
