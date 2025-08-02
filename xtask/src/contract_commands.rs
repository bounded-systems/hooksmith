use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::Subcommand;
use std::collections::HashMap;
use std::path::Path;

use crate::contract_state_machine::{ContractState, StateMachine, ValidationResult};
use crate::git_notes_manager::{ContractStateNote, GitNotesManager, TransitionLogEntry};

use super::ContractCommands;

/// Run contract commands
pub async fn run(command: ContractCommands) -> Result<()> {
    match command {
        ContractCommands::Validate {
            file,
            contract_type,
            store,
        } => {
            validate_contract(&file, &contract_type, store).await?;
        }
        ContractCommands::Audit {
            file,
            strict,
            merkle_only,
            transitions_only,
        } => {
            audit_contracts(file, strict, merkle_only, transitions_only).await?;
        }
        ContractCommands::List { detailed } => {
            list_contracts(detailed).await?;
        }
        ContractCommands::Cleanup { days, dry_run } => {
            cleanup_contracts(days, dry_run).await?;
        }
    }
    Ok(())
}

/// Validate a contract file
async fn validate_contract(file_path: &str, contract_type: &str, store: bool) -> Result<()> {
    println!("🔍 Validating contract: {}", file_path);
    println!("   Type: {}", contract_type);
    println!("   Store: {}", store);

    // Initialize components
    let state_machine = StateMachine::new()?;
    let notes_manager = GitNotesManager::new(Path::new("."))?;

    // Check if file exists
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", file_path);
    }

    // Get current state
    let current_state = if let Some(note) = notes_manager.get_contract_state(file_path)? {
        ContractState::from_string(&note.state).unwrap_or(ContractState::UNTRACKED)
    } else {
        ContractState::UNTRACKED
    };

    println!("   Current state: {}", current_state.to_string());

    // Determine target state based on current state
    let target_state = match current_state {
        ContractState::UNTRACKED => ContractState::UNVALIDATED,
        ContractState::UNVALIDATED => ContractState::VALIDATED,
        ContractState::VALIDATED => ContractState::VALIDATED, // Already valid
        ContractState::LOCKED => ContractState::LOCKED,       // Already locked
    };

    // Validate transition
    let transition = match (current_state, target_state) {
        (ContractState::UNTRACKED, ContractState::UNVALIDATED) => "detect_contract",
        (ContractState::UNVALIDATED, ContractState::VALIDATED) => "validate_contract",
        _ => "no_transition",
    };

    if transition != "no_transition" {
        let is_valid =
            state_machine.is_valid_transition(&current_state, &target_state, transition)?;
        if !is_valid {
            anyhow::bail!(
                "Invalid transition: {} -> {}",
                current_state.to_string(),
                target_state.to_string()
            );
        }
    }

    // Run validation
    let validation_result = state_machine.validate_state(&target_state, path)?;

    if !validation_result.is_valid() {
        println!("❌ Validation failed:");
        for (rule, error) in &validation_result.errors {
            println!("   - {}: {}", rule, error);
        }
        anyhow::bail!("Contract validation failed");
    }

    println!("✅ Validation successful:");
    println!("   - Successes: {}", validation_result.success_count());
    println!("   - Warnings: {}", validation_result.warning_count());

    // Store state if requested
    if store {
        let file_content = std::fs::read_to_string(path)?;
        let hash = format!("sha256:{}", sha256::digest(&file_content));

        let state_note = ContractStateNote {
            file: file_path.to_string(),
            contract: contract_type.to_string(),
            state: target_state.to_string(),
            hash,
            validated_by: format!("xtask-contract-validate {}", env!("CARGO_PKG_VERSION")),
            timestamp: Utc::now().to_rfc3339(),
            parent_scope: None,
            parent_hash: None,
            metadata: Some(HashMap::from([
                (
                    "line_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        file_content.lines().count(),
                    )),
                ),
                (
                    "file_size".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(file_content.len())),
                ),
                (
                    "validation_errors".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        validation_result.error_count(),
                    )),
                ),
                (
                    "validation_warnings".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        validation_result.warning_count(),
                    )),
                ),
            ])),
        };

        notes_manager.store_contract_state(&state_note)?;

        // Log transition
        let transition_entry = TransitionLogEntry {
            transition: transition.to_string(),
            from: current_state.to_string(),
            to: target_state.to_string(),
            file: file_path.to_string(),
            hash: state_note.hash.clone(),
            tool: state_note.validated_by.clone(),
            timestamp: state_note.timestamp.clone(),
            reason: Some("Manual validation".to_string()),
            commit_hash: None,
            user: None,
            environment: Some("local".to_string()),
            metadata: Some(HashMap::from([
                (
                    "validation_duration_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(0)),
                ),
                (
                    "errors_found".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        validation_result.error_count(),
                    )),
                ),
                (
                    "warnings_found".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        validation_result.warning_count(),
                    )),
                ),
            ])),
        };

        notes_manager.store_transition_log(&transition_entry)?;
        println!("💾 State stored in Git notes");
    }

    Ok(())
}

/// Audit contract states
async fn audit_contracts(
    file: Option<String>,
    strict: bool,
    merkle_only: bool,
    transitions_only: bool,
) -> Result<()> {
    println!("🔍 Auditing contract states...");

    let notes_manager = GitNotesManager::new(Path::new("."))?;
    let state_machine = StateMachine::new()?;

    if let Some(specific_file) = file {
        // Audit specific file
        audit_single_file(&notes_manager, &state_machine, &specific_file, strict).await?;
    } else {
        // Audit all files
        let contract_files = notes_manager.list_contract_files()?;

        if contract_files.is_empty() {
            println!("ℹ️  No contract files found");
            return Ok(());
        }

        println!("📋 Found {} contract files", contract_files.len());

        let mut total_errors = 0;
        let mut total_warnings = 0;

        for file_path in contract_files {
            match audit_single_file(&notes_manager, &state_machine, &file_path, false).await {
                Ok((errors, warnings)) => {
                    total_errors += errors;
                    total_warnings += warnings;
                }
                Err(e) => {
                    println!("❌ Failed to audit {}: {}", file_path, e);
                    total_errors += 1;
                }
            }
        }

        println!("📊 Audit Summary:");
        println!("   - Total files: {}", contract_files.len());
        println!("   - Total errors: {}", total_errors);
        println!("   - Total warnings: {}", total_warnings);

        if strict && total_errors > 0 {
            anyhow::bail!("Audit failed with {} errors", total_errors);
        }
    }

    Ok(())
}

/// Audit a single file
async fn audit_single_file(
    notes_manager: &GitNotesManager,
    state_machine: &StateMachine,
    file_path: &str,
    strict: bool,
) -> Result<(u32, u32)> {
    println!("   Auditing: {}", file_path);

    let mut errors = 0;
    let mut warnings = 0;

    // Check if file exists
    let path = Path::new(file_path);
    if !path.exists() {
        println!("   ❌ File does not exist");
        errors += 1;
        return Ok((errors, warnings));
    }

    // Get contract state
    match notes_manager.get_contract_state(file_path)? {
        Some(state_note) => {
            println!("   📝 State: {}", state_note.state);
            println!("   🔗 Hash: {}", state_note.hash);

            // Validate state
            let contract_state =
                ContractState::from_string(&state_note.state).unwrap_or(ContractState::UNTRACKED);

            let validation_result = state_machine.validate_state(&contract_state, path)?;

            if !validation_result.is_valid() {
                println!("   ❌ State validation failed:");
                for (rule, error) in &validation_result.errors {
                    println!("      - {}: {}", rule, error);
                    errors += 1;
                }
            }

            // Check hash
            let file_content = std::fs::read_to_string(path)?;
            let expected_hash = format!("sha256:{}", sha256::digest(&file_content));

            if state_note.hash != expected_hash {
                println!("   ❌ Hash mismatch:");
                println!("      Expected: {}", expected_hash);
                println!("      Found: {}", state_note.hash);
                errors += 1;
            }

            // Check timestamp
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&state_note.timestamp) {
                let age = Utc::now() - timestamp.naive_utc().and_utc();
                if age > Duration::days(30) {
                    println!("   ⚠️  State is stale ({} days old)", age.num_days());
                    warnings += 1;
                }
            }

            warnings += validation_result.warning_count();
        }
        None => {
            println!("   ⚠️  No contract state found");
            warnings += 1;
        }
    }

    if strict && errors > 0 {
        anyhow::bail!("File {} has {} errors", file_path, errors);
    }

    Ok((errors, warnings))
}

/// List all contract files
async fn list_contracts(detailed: bool) -> Result<()> {
    println!("📋 Listing contract files...");

    let notes_manager = GitNotesManager::new(Path::new("."))?;
    let contract_files = notes_manager.list_contract_files()?;

    if contract_files.is_empty() {
        println!("ℹ️  No contract files found");
        return Ok(());
    }

    println!("Found {} contract files:", contract_files.len());

    for file_path in contract_files {
        if detailed {
            if let Some(state_note) = notes_manager.get_contract_state(&file_path)? {
                println!("   📄 {}", file_path);
                println!("      State: {}", state_note.state);
                println!("      Contract: {}", state_note.contract);
                println!("      Hash: {}", state_note.hash);
                println!("      Validated by: {}", state_note.validated_by);
                println!("      Timestamp: {}", state_note.timestamp);

                if let Some(metadata) = &state_note.metadata {
                    println!("      Metadata:");
                    for (key, value) in metadata {
                        println!("        {}: {}", key, value);
                    }
                }
                println!();
            }
        } else {
            println!("   📄 {}", file_path);
        }
    }

    Ok(())
}

/// Clean up old contract states
async fn cleanup_contracts(days: u32, dry_run: bool) -> Result<()> {
    println!("🧹 Cleaning up contract states...");
    println!("   Days to keep: {}", days);
    println!("   Dry run: {}", dry_run);

    let notes_manager = GitNotesManager::new(Path::new("."))?;
    let contract_files = notes_manager.list_contract_files()?;

    let cutoff_date = Utc::now() - Duration::days(days as i64);
    let mut deleted_count = 0;

    for file_path in contract_files {
        if let Some(state_note) = notes_manager.get_contract_state(&file_path)? {
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&state_note.timestamp) {
                if timestamp.naive_utc().and_utc() < cutoff_date {
                    println!(
                        "   🗑️  {} ({} days old)",
                        file_path,
                        (Utc::now() - timestamp.naive_utc().and_utc()).num_days()
                    );

                    if !dry_run {
                        notes_manager.delete_contract_state(&file_path)?;
                    }
                    deleted_count += 1;
                }
            }
        }
    }

    if dry_run {
        println!("   📊 Would delete {} contract states", deleted_count);
    } else {
        println!("   📊 Deleted {} contract states", deleted_count);
    }

    Ok(())
}
