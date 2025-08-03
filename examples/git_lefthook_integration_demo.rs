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
use serde_json::Value;
use std::collections::HashMap;

use hooksmith::git_lefthook_integration::{
    ContractValidationResult, ContractViolation, GitLefthookIntegration, ViolationSeverity,
};

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
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::CommitStarted,
        commit_context,
    )?;
    println!("   Commit Started → {:?}", integration.current_state());

    // Simulate hook started
    let hook_context = serde_json::json!({
        "hook": "post-commit",
        "command": "cargo test",
        "files": ["src/contract.rs", "src/validation.rs"]
    });
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::HookStarted,
        hook_context,
    )?;
    println!("   Hook Started → {:?}", integration.current_state());

    // Simulate hook completed
    let hook_complete_context = serde_json::json!({
        "exit_code": 0,
        "duration_ms": 1250,
        "output": "running 12 tests\ntest result: ok. 12 passed; 0 failed"
    });
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::HookCompleted,
        hook_complete_context,
    )?;
    println!("   Hook Completed → {:?}", integration.current_state());

    // Simulate commit completed
    let commit_complete_context = serde_json::json!({
        "hash": "a1b2c3d4e5f6",
        "insertions": 45,
        "deletions": 12
    });
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::CommitCompleted,
        commit_complete_context,
    )?;
    println!("   Commit Completed → {:?}", integration.current_state());

    // Simulate push started
    let push_context = serde_json::json!({
        "remote": "origin",
        "branch": "feature/contract-validation"
    });
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::PushStarted,
        push_context,
    )?;
    println!("   Push Started → {:?}", integration.current_state());

    // Simulate push completed
    let push_complete_context = serde_json::json!({
        "objects": 8,
        "deltas": 3,
        "remote_url": "https://github.com/user/repo.git"
    });
    integration.transition(
        hooksmith::git_lefthook_integration::GitWorkflowEvent::PushCompleted,
        push_complete_context,
    )?;
    println!("   Push Completed → {:?}", integration.current_state());

    // Demonstrate contract validation with SARIF integration
    println!("\n🔍 Contract Validation with SARIF");
    println!("--------------------------------");

    // Add some contract validation results
    let validation_result = ContractValidationResult {
        is_valid: false,
        contract_id: "file-extension-policy".to_string(),
        file: "src/old_file.py".to_string(),
        errors: vec![
            ContractViolation {
                id: "ext-001".to_string(),
                rule_id: "file-extension-only-rs".to_string(),
                message: "File has .py extension, only .rs files allowed".to_string(),
                severity: ViolationSeverity::Error,
                file: "src/old_file.py".to_string(),
                line: Some(1),
                column: Some(1),
                end_line: Some(1),
                end_column: Some(1),
                details: Some(serde_json::json!({
                    "expected_extension": ".rs",
                    "actual_extension": ".py",
                    "policy": "strict-file-extension-enforcement"
                })),
                fingerprint: Some("ext-py-file-001".to_string()),
                blocked_by: None,
            },
        ],
        warnings: vec![
            ContractViolation {
                id: "ext-002".to_string(),
                rule_id: "file-extension-warning".to_string(),
                message: "Consider migrating Python file to Rust".to_string(),
                severity: ViolationSeverity::Warning,
                file: "src/old_file.py".to_string(),
                line: Some(1),
                column: Some(1),
                end_line: Some(1),
                end_column: Some(1),
                details: Some(serde_json::json!({
                    "migration_guide": "docs/migration/python-to-rust.md",
                    "priority": "medium"
                })),
                fingerprint: Some("ext-py-migration-001".to_string()),
                blocked_by: None,
            },
        ],
        sarif_result: None, // Will be generated automatically
        blocked_by: None,
        timestamp: Utc::now(),
    };

    integration.add_validation_result(validation_result)?;
    println!("   ✅ Added contract validation result");

    // Add another validation result with blocking dependencies
    let blocking_validation = ContractValidationResult {
        is_valid: false,
        contract_id: "code-quality".to_string(),
        file: "src/contract.rs".to_string(),
        errors: vec![
            ContractViolation {
                id: "qual-001".to_string(),
                rule_id: "clippy-warnings".to_string(),
                message: "Clippy warnings must be resolved".to_string(),
                severity: ViolationSeverity::Error,
                file: "src/contract.rs".to_string(),
                line: Some(42),
                column: Some(10),
                end_line: Some(42),
                end_column: Some(25),
                details: Some(serde_json::json!({
                    "clippy_warning": "unused_variable",
                    "suggestion": "prefix with underscore or remove"
                })),
                fingerprint: Some("clippy-unused-var-001".to_string()),
                blocked_by: Some(vec!["file-extension-policy".to_string()]),
            },
        ],
        warnings: vec![],
        sarif_result: None,
        blocked_by: Some(vec!["file-extension-policy".to_string()]),
        timestamp: Utc::now(),
    };

    integration.add_validation_result(blocking_validation)?;
    println!("   ✅ Added blocking validation result");

    // Show validation results
    println!("\n📊 Validation Results");
    println!("-------------------");
    for result in integration.validation_results() {
        println!("   Contract: {}", result.contract_id);
        println!("   File: {}", result.file);
        println!("   Valid: {}", result.is_valid);
        println!("   Errors: {}", result.errors.len());
        println!("   Warnings: {}", result.warnings.len());
        if let Some(ref blocked_by) = result.blocked_by {
            println!("   Blocked by: {:?}", blocked_by);
        }
        println!();
    }

    // Generate SARIF document
    println!("📄 Generating SARIF Document");
    println!("---------------------------");
    let sarif_document = integration.generate_sarif_document()?;
    println!("   ✅ Generated SARIF document ({} bytes)", sarif_document.len());

    // Show SARIF results
    println!("\n🔍 SARIF Results");
    println!("---------------");
    for result in integration.sarif_results() {
        println!("   Rule: {}", result.rule_id);
        println!("   Level: {}", result.level);
        println!("   Message: {}", result.message);
        if let Some(ref location) = result.location {
            if let Some(ref region) = location.physical_location.region {
                println!("   Location: {}:{}-{}", 
                    location.physical_location.artifact_location.uri,
                    region.start_line.unwrap_or(0),
                    region.end_line.unwrap_or(0)
                );
            }
        }
        println!();
    }

    // Demonstrate event blocking
    println!("🔒 Event Blocking Dependencies");
    println!("-----------------------------");
    integration.add_blocking_dependency("code-quality", "file-extension-policy");
    println!("   ✅ Added blocking dependency: code-quality → file-extension-policy");

    // Show final state
    println!("\n🏁 Final State");
    println!("-------------");
    println!("   Session ID: {}", integration.session_id());
    println!("   Current State: {:?}", integration.current_state());
    println!("   Validation Results: {}", integration.validation_results().len());
    println!("   SARIF Results: {}", integration.sarif_results().len());

    println!("\n✅ Demo completed successfully!");
    println!("\n💡 Next Steps:");
    println!("   - Run: cargo xtask git-lefthook workflow --message 'feat: demo'");
    println!("   - Run: cargo xtask git-lefthook validate --contract-id demo --file src/main.rs --rule-id demo-rule --message 'Demo validation'");
    println!("   - Run: cargo xtask git-lefthook generate-sarif --output demo-results.sarif");

    Ok(())
} 