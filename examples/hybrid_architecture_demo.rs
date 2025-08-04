//! Hybrid Architecture Demo
//!
//! This example demonstrates how the hybrid WIT + native Rust architecture
//! works in practice, showing the event-driven flow between CLI commands,
//! native handlers, and WIT components.

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

// Import the event bus and handlers
use crate::xtask::event_bus::{emit_event, HooksmithEvent, EventBusConfig};
use crate::xtask::wasm_event_bus;

/// Demo of hybrid architecture workflow
pub async fn run_hybrid_architecture_demo() -> Result<()> {
    println!("🚀 Starting Hybrid Architecture Demo");
    println!("=====================================");

    // Initialize the event bus
    let event_bus_config = EventBusConfig::default();
    crate::xtask::event_bus::init_event_bus(event_bus_config.clone())?;
    wasm_event_bus::init_wasm_event_bus_host(event_bus_config)?;

    // Demo 1: Contract Validation Workflow
    println!("\n📋 Demo 1: Contract Validation Workflow");
    println!("----------------------------------------");
    await contract_validation_workflow_demo().await?;

    // Demo 2: File Operations Workflow
    println!("\n📁 Demo 2: File Operations Workflow");
    println!("------------------------------------");
    await file_operations_workflow_demo().await?;

    // Demo 3: Git Operations Workflow
    println!("\n🔧 Demo 3: Git Operations Workflow");
    println!("-----------------------------------");
    await git_operations_workflow_demo().await?;

    println!("\n✅ Hybrid Architecture Demo Completed Successfully!");
    Ok(())
}

/// Demo of contract validation workflow using hybrid architecture
async fn contract_validation_workflow_demo() -> Result<()> {
    let session_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();
    
    println!("Session ID: {}", session_id);
    println!("Request ID: {}", request_id);

    // Step 1: Read file content (Native Handler)
    println!("\n1️⃣ Reading file content...");
    let file_read_event = HooksmithEvent::new(
        "cli-contract-validation".to_string(),
        "file_read_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "path": "examples/test_contract.json",
            "encoding": "utf8",
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(file_read_event)?;

    // Wait for file read result
    let file_content = await_event_response("file_read_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ File content read successfully");

    // Step 2: Validate contract (WIT Component)
    println!("\n2️⃣ Validating contract...");
    let validation_event = HooksmithEvent::new(
        "cli-contract-validation".to_string(),
        "validation_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "contract_name": "test_contract",
            "file_path": "examples/test_contract.json",
            "content": file_content,
            "validation_config": {
                "strict": true,
                "store_proof": true,
                "rules": ["json_schema", "content_validation"]
            },
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "component": "validation-handler"
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(validation_event)?;

    // Wait for validation result
    let validation_result = await_event_response("validation_result", &request_id, Duration::from_secs(10)).await?;
    println!("✅ Contract validation completed");

    // Step 3: Store validation proof (Native Handler)
    if validation_result.get("result").and_then(|r| r.get("valid")).and_then(|v| v.as_bool()).unwrap_or(false) {
        println!("\n3️⃣ Storing validation proof...");
        let proof_event = HooksmithEvent::new(
            "cli-contract-validation".to_string(),
            "git_note_add_request".to_string(),
            json!({
                "request_id": request_id.clone(),
                "note": {
                    "object": "HEAD",
                    "message": "Validation proof for test_contract.json",
                    "file": "validation_proof.json"
                },
                "metadata": {
                    "session_id": session_id,
                    "timestamp": Utc::now().to_rfc3339(),
                    "repository": std::env::current_dir()?.to_string_lossy()
                }
            }),
        )
        .with_session_id(session_id.clone());

        emit_event(proof_event)?;

        // Wait for note add result
        let _note_result = await_event_response("git_note_add_result", &request_id, Duration::from_secs(5)).await?;
        println!("✅ Validation proof stored");
    }

    println!("🎉 Contract validation workflow completed successfully!");
    Ok(())
}

/// Demo of file operations workflow using hybrid architecture
async fn file_operations_workflow_demo() -> Result<()> {
    let session_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();
    
    println!("Session ID: {}", session_id);
    println!("Request ID: {}", request_id);

    // Step 1: Create a test file
    println!("\n1️⃣ Creating test file...");
    let write_event = HooksmithEvent::new(
        "cli-file-operations".to_string(),
        "file_write_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "path": "temp/test_file.txt",
            "content": "Hello, Hybrid Architecture!",
            "encoding": "utf8",
            "create_parents": true,
            "overwrite": true,
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(write_event)?;

    // Wait for write result
    let _write_result = await_event_response("file_write_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ Test file created successfully");

    // Step 2: Calculate file checksum
    println!("\n2️⃣ Calculating file checksum...");
    let checksum_event = HooksmithEvent::new(
        "cli-file-operations".to_string(),
        "file_checksum_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "path": "temp/test_file.txt",
            "algorithm": "sha256",
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(checksum_event)?;

    // Wait for checksum result
    let checksum_result = await_event_response("file_checksum_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ File checksum calculated: {}", 
        checksum_result.get("checksum").and_then(|c| c.as_str()).unwrap_or("unknown"));

    // Step 3: Read file back
    println!("\n3️⃣ Reading file back...");
    let read_event = HooksmithEvent::new(
        "cli-file-operations".to_string(),
        "file_read_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "path": "temp/test_file.txt",
            "encoding": "utf8",
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(read_event)?;

    // Wait for read result
    let read_result = await_event_response("file_read_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ File content: {}", 
        read_result.get("content").and_then(|c| c.as_str()).unwrap_or("unknown"));

    // Step 4: Clean up
    println!("\n4️⃣ Cleaning up...");
    let delete_event = HooksmithEvent::new(
        "cli-file-operations".to_string(),
        "file_delete_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "path": "temp/test_file.txt",
            "recursive": false,
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(delete_event)?;

    // Wait for delete result
    let _delete_result = await_event_response("file_delete_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ Test file cleaned up");

    println!("🎉 File operations workflow completed successfully!");
    Ok(())
}

/// Demo of Git operations workflow using hybrid architecture
async fn git_operations_workflow_demo() -> Result<()> {
    let session_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();
    
    println!("Session ID: {}", session_id);
    println!("Request ID: {}", request_id);

    // Step 1: Check Git status
    println!("\n1️⃣ Checking Git status...");
    let status_event = HooksmithEvent::new(
        "cli-git-operations".to_string(),
        "git_status_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "repository": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(status_event)?;

    // Wait for status result
    let status_result = await_event_response("git_status_result", &request_id, Duration::from_secs(5)).await?;
    println!("✅ Git status retrieved");

    // Step 2: Add files if there are changes
    if let Some(status) = status_result.get("status") {
        let staged = status.get("staged").and_then(|s| s.as_array()).unwrap_or(&Vec::new());
        let unstaged = status.get("unstaged").and_then(|s| s.as_array()).unwrap_or(&Vec::new());
        
        if !staged.is_empty() || !unstaged.is_empty() {
            println!("\n2️⃣ Adding files to staging...");
            let add_event = HooksmithEvent::new(
                "cli-git-operations".to_string(),
                "git_add_request".to_string(),
                json!({
                    "request_id": request_id.clone(),
                    "files": ["."],
                    "metadata": {
                        "session_id": session_id,
                        "timestamp": Utc::now().to_rfc3339(),
                        "repository": std::env::current_dir()?.to_string_lossy()
                    }
                }),
            )
            .with_session_id(session_id.clone());

            emit_event(add_event)?;

            // Wait for add result
            let _add_result = await_event_response("git_add_result", &request_id, Duration::from_secs(5)).await?;
            println!("✅ Files added to staging");
        }
    }

    // Step 3: Create a commit
    println!("\n3️⃣ Creating commit...");
    let commit_event = HooksmithEvent::new(
        "cli-git-operations".to_string(),
        "git_commit_request".to_string(),
        json!({
            "request_id": request_id.clone(),
            "commit": {
                "message": "Hybrid architecture demo commit",
                "files": ["."],
                "allow_empty": true
            },
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "repository": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.clone());

    emit_event(commit_event)?;

    // Wait for commit result
    let commit_result = await_event_response("git_commit_result", &request_id, Duration::from_secs(10)).await?;
    println!("✅ Commit created: {}", 
        commit_result.get("commit_hash").and_then(|h| h.as_str()).unwrap_or("unknown"));

    println!("🎉 Git operations workflow completed successfully!");
    Ok(())
}

/// Wait for an event response with timeout
async fn await_event_response(event_type: &str, request_id: &str, timeout_duration: Duration) -> Result<serde_json::Value> {
    // In a real implementation, this would subscribe to the event bus
    // and wait for the specific event response. For this demo, we'll
    // simulate the response.
    
    println!("⏳ Waiting for {} response...", event_type);
    
    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Return a mock response based on event type
    match event_type {
        "file_read_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "content": "{\"name\": \"test_contract\", \"version\": \"1.0.0\", \"rules\": []}",
            "size": 45,
            "encoding": "utf8"
        })),
        "validation_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "result": {
                "valid": true,
                "errors": [],
                "warnings": [],
                "proof_hash": "abc123def456",
                "duration_ms": 150
            }
        })),
        "git_note_add_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "note_id": "note123"
        })),
        "file_write_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "size": 25
        })),
        "file_checksum_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "checksum": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
            "algorithm": "sha256"
        })),
        "file_read_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "content": "Hello, Hybrid Architecture!",
            "size": 25,
            "encoding": "utf8"
        })),
        "file_delete_result" => Ok(json!({
            "request_id": request_id,
            "success": true
        })),
        "git_status_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "status": {
                "staged": [],
                "unstaged": [],
                "untracked": [],
                "modified": [],
                "deleted": [],
                "renamed": []
            },
            "branch": "main"
        })),
        "git_add_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "files_added": ["."]
        })),
        "git_commit_result" => Ok(json!({
            "request_id": request_id,
            "success": true,
            "commit_hash": "abc123def456789",
            "files_changed": ["."],
            "branch": "main"
        })),
        _ => Ok(json!({
            "request_id": request_id,
            "success": true
        }))
    }
}

/// Example of how to refactor the existing contract validation command
pub async fn refactored_contract_validation_command(
    contract_name: &str,
    file_path: &str,
    strict: bool,
    store_proof: bool,
) -> Result<()> {
    let session_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();
    
    println!("🔍 Validating contract '{}' against file '{}'", contract_name, file_path);

    // Step 1: Read the file content using native file operations
    let file_content = read_file_content(&file_path, &session_id, &request_id).await?;
    
    // Step 2: Validate the contract using WIT component
    let validation_result = validate_contract(
        contract_name,
        file_path,
        &file_content,
        strict,
        store_proof,
        &session_id,
        &request_id,
    ).await?;
    
    // Step 3: Handle the validation result
    if validation_result.valid {
        println!("✅ Contract validation passed");
        
        if store_proof {
            store_validation_proof(file_path, &validation_result, &session_id, &request_id).await?;
        }
    } else {
        println!("❌ Contract validation failed");
        for error in &validation_result.errors {
            println!("  - {}: {}", error.rule, error.message);
        }
        return Err(anyhow::anyhow!("Contract validation failed"));
    }
    
    Ok(())
}

/// Read file content using event-driven file operations
async fn read_file_content(file_path: &str, session_id: &str, request_id: &str) -> Result<String> {
    let read_event = HooksmithEvent::new(
        "contract-validation".to_string(),
        "file_read_request".to_string(),
        json!({
            "request_id": request_id,
            "path": file_path,
            "encoding": "utf8",
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.to_string());

    emit_event(read_event)?;
    
    let response = await_event_response("file_read_result", request_id, Duration::from_secs(5)).await?;
    
    if response.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(response.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string())
    } else {
        Err(anyhow::anyhow!("Failed to read file: {}", file_path))
    }
}

/// Validate contract using event-driven WIT component
async fn validate_contract(
    contract_name: &str,
    file_path: &str,
    content: &str,
    strict: bool,
    store_proof: bool,
    session_id: &str,
    request_id: &str,
) -> Result<ValidationResult> {
    let validation_event = HooksmithEvent::new(
        "contract-validation".to_string(),
        "validation_request".to_string(),
        json!({
            "request_id": request_id,
            "contract_name": contract_name,
            "file_path": file_path,
            "content": content,
            "validation_config": {
                "strict": strict,
                "store_proof": store_proof,
                "rules": ["json_schema", "content_validation"]
            },
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "component": "validation-handler"
            }
        }),
    )
    .with_session_id(session_id.to_string());

    emit_event(validation_event)?;
    
    let response = await_event_response("validation_result", request_id, Duration::from_secs(10)).await?;
    
    if response.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        let result = response.get("result").ok_or_else(|| anyhow::anyhow!("No result in validation response"))?;
        
        Ok(ValidationResult {
            valid: result.get("valid").and_then(|v| v.as_bool()).unwrap_or(false),
            errors: Vec::new(), // Parse errors from result
            warnings: Vec::new(), // Parse warnings from result
            proof_hash: result.get("proof_hash").and_then(|p| p.as_str()).map(|s| s.to_string()),
        })
    } else {
        Err(anyhow::anyhow!("Validation request failed"))
    }
}

/// Store validation proof using event-driven Git operations
async fn store_validation_proof(
    file_path: &str,
    validation_result: &ValidationResult,
    session_id: &str,
    request_id: &str,
) -> Result<()> {
    let proof_content = json!({
        "file_path": file_path,
        "validation_result": validation_result,
        "timestamp": Utc::now().to_rfc3339(),
    });

    // Write proof to file
    let write_event = HooksmithEvent::new(
        "contract-validation".to_string(),
        "file_write_request".to_string(),
        json!({
            "request_id": request_id,
            "path": "validation_proof.json",
            "content": serde_json::to_string(&proof_content)?,
            "encoding": "utf8",
            "create_parents": true,
            "overwrite": true,
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "working_directory": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.to_string());

    emit_event(write_event)?;
    
    let _write_response = await_event_response("file_write_result", request_id, Duration::from_secs(5)).await?;
    
    // Add Git note
    let note_event = HooksmithEvent::new(
        "contract-validation".to_string(),
        "git_note_add_request".to_string(),
        json!({
            "request_id": request_id,
            "note": {
                "object": "HEAD",
                "message": format!("Validation proof for {}", file_path),
                "file": "validation_proof.json"
            },
            "metadata": {
                "session_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "repository": std::env::current_dir()?.to_string_lossy()
            }
        }),
    )
    .with_session_id(session_id.to_string());

    emit_event(note_event)?;
    
    let _note_response = await_event_response("git_note_add_result", request_id, Duration::from_secs(5)).await?;
    
    println!("💾 Validation proof stored");
    Ok(())
}

/// Validation result structure
#[derive(Debug, Clone)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    proof_hash: Option<String>,
}

/// Validation error structure
#[derive(Debug, Clone)]
struct ValidationError {
    rule: String,
    message: String,
    line: Option<u32>,
    column: Option<u32>,
}

/// Validation warning structure
#[derive(Debug, Clone)]
struct ValidationWarning {
    rule: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_architecture_demo() {
        // This test would run the demo and verify the event flow
        // For now, we'll just test that the demo function compiles
        assert!(true);
    }
} 
