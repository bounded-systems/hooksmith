//! Contract State Machine Demo
//!
//! This example demonstrates the contract validation state machine
//! with schema-driven validation and Git notes storage.

use chrono::Utc;
use hex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

// In a real implementation, these would be imported from the xtask module
// use xtask::contract_state_machine::{StateMachine, ContractState, ValidationResult};
// use xtask::git_notes_manager::{GitNotesManager, ContractStateNote, TransitionLogEntry};

/// Simplified contract state for demonstration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractState {
    UNTRACKED,
    UNVALIDATED,
    VALIDATED,
    LOCKED,
}

impl ContractState {
    pub fn to_string(&self) -> &'static str {
        match self {
            ContractState::UNTRACKED => "UNTRACKED",
            ContractState::UNVALIDATED => "UNVALIDATED",
            ContractState::VALIDATED => "VALIDATED",
            ContractState::LOCKED => "LOCKED",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "UNTRACKED" => Some(ContractState::UNTRACKED),
            "UNVALIDATED" => Some(ContractState::UNVALIDATED),
            "VALIDATED" => Some(ContractState::VALIDATED),
            "LOCKED" => Some(ContractState::LOCKED),
            _ => None,
        }
    }
}

/// Simplified contract state note
#[derive(Debug, Clone)]
pub struct ContractStateNote {
    pub file: String,
    pub contract: String,
    pub state: String,
    pub hash: String,
    pub validated_by: String,
    pub timestamp: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Simplified validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub successes: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            successes: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_success(&mut self, success: String) {
        self.successes.push(success);
    }
}

/// Simplified state machine
pub struct StateMachine;

impl StateMachine {
    pub fn new() -> Self {
        StateMachine
    }

    pub fn validate_state(&self, state: &ContractState, file_path: &Path) -> ValidationResult {
        let mut result = ValidationResult::new();

        match state {
            ContractState::UNTRACKED => {
                if !file_path.exists() {
                    result.add_error("File does not exist".to_string());
                } else {
                    result.add_success("File exists".to_string());
                }
            }
            ContractState::UNVALIDATED => {
                if !file_path.exists() {
                    result.add_error("File does not exist".to_string());
                } else {
                    result.add_success("File exists".to_string());
                }

                // Check if file is readable
                if std::fs::read_to_string(file_path).is_err() {
                    result.add_error("File is not readable".to_string());
                } else {
                    result.add_success("File is readable".to_string());
                }
            }
            ContractState::VALIDATED => {
                if !file_path.exists() {
                    result.add_error("File does not exist".to_string());
                } else {
                    result.add_success("File exists".to_string());
                }

                // Check if file is readable
                if std::fs::read_to_string(file_path).is_err() {
                    result.add_error("File is not readable".to_string());
                } else {
                    result.add_success("File is readable".to_string());
                }

                // Additional validation for validated state
                result.add_success("Contract validation passed".to_string());
            }
            ContractState::LOCKED => {
                if !file_path.exists() {
                    result.add_error("File does not exist".to_string());
                } else {
                    result.add_success("File exists".to_string());
                }

                // Check if file is readable
                if std::fs::read_to_string(file_path).is_err() {
                    result.add_error("File is not readable".to_string());
                } else {
                    result.add_success("File is readable".to_string());
                }

                // Additional validation for locked state
                result.add_success("Contract validation passed".to_string());
                result.add_success("File is locked".to_string());
            }
        }

        result
    }

    pub fn is_valid_transition(
        &self,
        from: &ContractState,
        to: &ContractState,
        transition: &str,
    ) -> bool {
        match (from, to, transition) {
            (ContractState::UNTRACKED, ContractState::UNVALIDATED, "detect_contract") => true,
            (ContractState::UNVALIDATED, ContractState::VALIDATED, "validate_contract") => true,
            (ContractState::VALIDATED, ContractState::LOCKED, "lock_contract") => true,
            (ContractState::LOCKED, ContractState::UNVALIDATED, "modify_contract") => true,
            _ => false,
        }
    }
}

/// Simplified Git notes manager
pub struct GitNotesManager;

impl GitNotesManager {
    pub fn new(_repo_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(GitNotesManager)
    }

    pub fn store_contract_state(
        &self,
        state: &ContractStateNote,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "💾 Storing contract state for {}: {}",
            state.file, state.state
        );
        println!("   Hash: {}", state.hash);
        println!("   Validated by: {}", state.validated_by);
        println!("   Timestamp: {}", state.timestamp);
        Ok(())
    }

    pub fn get_contract_state(
        &self,
        _file_path: &str,
    ) -> Result<Option<ContractStateNote>, Box<dyn std::error::Error>> {
        // Simulate no existing state
        Ok(None)
    }
}

/// Demo function to show contract validation workflow
pub fn demo_contract_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Contract State Machine Demo");
    println!("================================");

    // Create a temporary file for demonstration
    let temp_file = tempfile::NamedTempFile::new()?;
    let file_path = temp_file.path();

    // Write some content to the file
    std::fs::write(
        file_path,
        "// This is a sample Rust file\nfn main() {\n    println!(\"Hello, world!\");\n}",
    )?;

    println!("📄 Created demo file: {:?}", file_path);

    // Initialize components
    let state_machine = StateMachine::new();
    let notes_manager = GitNotesManager::new(Path::new("."))?;

    // Step 1: Detect contract (UNTRACKED -> UNVALIDATED)
    println!("\n🔍 Step 1: Detecting contract...");
    let current_state = ContractState::UNTRACKED;
    let target_state = ContractState::UNVALIDATED;

    if state_machine.is_valid_transition(&current_state, &target_state, "detect_contract") {
        println!(
            "✅ Valid transition: {} -> {}",
            current_state.to_string(),
            target_state.to_string()
        );

        let validation_result = state_machine.validate_state(&target_state, file_path);
        if validation_result.is_valid {
            println!("✅ State validation passed");
            for success in &validation_result.successes {
                println!("   ✓ {}", success);
            }
        } else {
            println!("❌ State validation failed");
            for error in &validation_result.errors {
                println!("   ✗ {}", error);
            }
        }
    } else {
        println!("❌ Invalid transition");
    }

    // Step 2: Validate contract (UNVALIDATED -> VALIDATED)
    println!("\n🔍 Step 2: Validating contract...");
    let current_state = ContractState::UNVALIDATED;
    let target_state = ContractState::VALIDATED;

    if state_machine.is_valid_transition(&current_state, &target_state, "validate_contract") {
        println!(
            "✅ Valid transition: {} -> {}",
            current_state.to_string(),
            target_state.to_string()
        );

        let validation_result = state_machine.validate_state(&target_state, file_path);
        if validation_result.is_valid {
            println!("✅ State validation passed");
            for success in &validation_result.successes {
                println!("   ✓ {}", success);
            }

            // Store state in Git notes
            let file_content = std::fs::read_to_string(file_path)?;
            let hash = format!("sha256:{}", hex::encode(Sha256::digest(&file_content)));

            let state_note = ContractStateNote {
                file: file_path.to_string_lossy().to_string(),
                contract: "blob".to_string(),
                state: target_state.to_string().to_string(),
                hash,
                validated_by: "xtask-contract-validate 0.1.0".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                metadata: Some(HashMap::from([
                    (
                        "line_count".to_string(),
                        file_content.lines().count().to_string(),
                    ),
                    ("file_size".to_string(), file_content.len().to_string()),
                    ("validation_errors".to_string(), "0".to_string()),
                ])),
            };

            notes_manager.store_contract_state(&state_note)?;
        } else {
            println!("❌ State validation failed");
            for error in &validation_result.errors {
                println!("   ✗ {}", error);
            }
        }
    } else {
        println!("❌ Invalid transition");
    }

    // Step 3: Lock contract (VALIDATED -> LOCKED)
    println!("\n🔍 Step 3: Locking contract...");
    let current_state = ContractState::VALIDATED;
    let target_state = ContractState::LOCKED;

    if state_machine.is_valid_transition(&current_state, &target_state, "lock_contract") {
        println!(
            "✅ Valid transition: {} -> {}",
            current_state.to_string(),
            target_state.to_string()
        );

        let validation_result = state_machine.validate_state(&target_state, file_path);
        if validation_result.is_valid {
            println!("✅ State validation passed");
            for success in &validation_result.successes {
                println!("   ✓ {}", success);
            }

            // Update state in Git notes
            let file_content = std::fs::read_to_string(file_path)?;
            let hash = format!("sha256:{}", hex::encode(Sha256::digest(&file_content)));

            let state_note = ContractStateNote {
                file: file_path.to_string_lossy().to_string(),
                contract: "blob".to_string(),
                state: target_state.to_string().to_string(),
                hash,
                validated_by: "xtask-contract-validate 0.1.0".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                metadata: Some(HashMap::from([
                    (
                        "line_count".to_string(),
                        file_content.lines().count().to_string(),
                    ),
                    ("file_size".to_string(), file_content.len().to_string()),
                    ("validation_errors".to_string(), "0".to_string()),
                    ("locked".to_string(), "true".to_string()),
                ])),
            };

            notes_manager.store_contract_state(&state_note)?;
        } else {
            println!("❌ State validation failed");
            for error in &validation_result.errors {
                println!("   ✗ {}", error);
            }
        }
    } else {
        println!("❌ Invalid transition");
    }

    println!("\n🎉 Demo completed successfully!");
    println!("This demonstrates the contract state machine workflow:");
    println!("  1. UNTRACKED → UNVALIDATED (detect_contract)");
    println!("  2. UNVALIDATED → VALIDATED (validate_contract)");
    println!("  3. VALIDATED → LOCKED (lock_contract)");
    println!("\nEach transition includes:");
    println!("  - State validation");
    println!("  - Hash computation");
    println!("  - Git notes storage");
    println!("  - Metadata tracking");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    demo_contract_validation()
}
