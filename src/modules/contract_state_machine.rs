use crate::modules::hierarchical_validation::{
    ValidationScope, ValidationNote, ValidationError, ValidationResult
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

/// Contract states in the validation lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractState {
    /// File has no contract or codegen attribute
    UNTRACKED,
    /// File has contract/codegen attribute but no proof (note)
    UNVALIDATED,
    /// File has contract/codegen attribute + matching note with hash
    VALIDATED,
    /// File is validated + no further modifications allowed without re-validation
    LOCKED,
}

/// Transition events that can change contract state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionEvent {
    /// Contract attribute detected in .gitattributes
    DetectContract,
    /// Run validator and create Git note with hash
    ValidateContract,
    /// Post-hook marks file as locked
    LockContract,
    /// Changing file invalidates proof
    ModifyContract,
    /// Regenerate codegen and hash-check
    RegenCodegen,
    /// CI verifies Merkle hashes and proof chain
    ReleaseProof,
}

/// State transition with conditions and actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: ContractState,
    pub to: ContractState,
    pub event: TransitionEvent,
    pub action: String,
    pub conditions: Vec<String>,
}

/// Contract state machine that enforces valid transitions
#[derive(Debug)]
pub struct ContractStateMachine {
    transitions: HashMap<(ContractState, TransitionEvent), StateTransition>,
    current_states: HashMap<String, ContractState>,
}

impl ContractStateMachine {
    /// Create a new contract state machine with predefined transitions
    pub fn new() -> Self {
        let mut machine = Self {
            transitions: HashMap::new(),
            current_states: HashMap::new(),
        };
        
        machine.initialize_transitions();
        machine
    }

    /// Initialize the state transition table
    fn initialize_transitions(&mut self) {
        let transitions = vec![
            // UNTRACKED → UNVALIDATED
            StateTransition {
                from: ContractState::UNTRACKED,
                to: ContractState::UNVALIDATED,
                event: TransitionEvent::DetectContract,
                action: "Tag file via .gitattributes".to_string(),
                conditions: vec![
                    "File matches contract pattern in .gitattributes".to_string(),
                    "No existing Git note for file".to_string(),
                ],
            },
            
            // UNVALIDATED → VALIDATED
            StateTransition {
                from: ContractState::UNVALIDATED,
                to: ContractState::VALIDATED,
                event: TransitionEvent::ValidateContract,
                action: "Run validator → create Git note with hash".to_string(),
                conditions: vec![
                    "Contract validation passes".to_string(),
                    "Content hash computed".to_string(),
                    "Git note created successfully".to_string(),
                ],
            },
            
            // VALIDATED → LOCKED
            StateTransition {
                from: ContractState::VALIDATED,
                to: ContractState::LOCKED,
                event: TransitionEvent::LockContract,
                action: "Post-hook marks file as locked".to_string(),
                conditions: vec![
                    "File is in committed state".to_string(),
                    "No pending changes".to_string(),
                ],
            },
            
            // LOCKED → UNVALIDATED
            StateTransition {
                from: ContractState::LOCKED,
                to: ContractState::UNVALIDATED,
                event: TransitionEvent::ModifyContract,
                action: "Changing file invalidates proof".to_string(),
                conditions: vec![
                    "File content has changed".to_string(),
                    "Hash no longer matches".to_string(),
                ],
            },
            
            // UNVALIDATED → VALIDATED (re-validation)
            StateTransition {
                from: ContractState::UNVALIDATED,
                to: ContractState::VALIDATED,
                event: TransitionEvent::ValidateContract,
                action: "Re-run validator with new content".to_string(),
                conditions: vec![
                    "Contract validation passes".to_string(),
                    "New content hash computed".to_string(),
                    "Git note updated".to_string(),
                ],
            },
            
            // UNVALIDATED → VALIDATED (codegen)
            StateTransition {
                from: ContractState::UNVALIDATED,
                to: ContractState::VALIDATED,
                event: TransitionEvent::RegenCodegen,
                action: "Regenerate codegen + hash-check".to_string(),
                conditions: vec![
                    "Codegen files regenerated".to_string(),
                    "New hashes computed".to_string(),
                    "Validation passes".to_string(),
                ],
            },
            
            // VALIDATED → VALIDATED (proof verification)
            StateTransition {
                from: ContractState::VALIDATED,
                to: ContractState::VALIDATED,
                event: TransitionEvent::ReleaseProof,
                action: "CI verifies Merkle hashes and proof chain".to_string(),
                conditions: vec![
                    "All child hashes match parent".to_string(),
                    "Merkle chain is valid".to_string(),
                    "No tampering detected".to_string(),
                ],
            },
            
            // LOCKED → LOCKED (proof verification)
            StateTransition {
                from: ContractState::LOCKED,
                to: ContractState::LOCKED,
                event: TransitionEvent::ReleaseProof,
                action: "CI verifies Merkle hashes and proof chain".to_string(),
                conditions: vec![
                    "All child hashes match parent".to_string(),
                    "Merkle chain is valid".to_string(),
                    "No tampering detected".to_string(),
                ],
            },
        ];

        for transition in transitions {
            self.transitions.insert(
                (transition.from.clone(), transition.event.clone()),
                transition,
            );
        }
    }

    /// Get the current state of a file
    pub fn get_state(&self, file: &str) -> ContractState {
        self.current_states.get(file).cloned().unwrap_or(ContractState::UNTRACKED)
    }

    /// Set the current state of a file
    pub fn set_state(&mut self, file: String, state: ContractState) {
        self.current_states.insert(file, state);
    }

    /// Check if a transition is valid
    pub fn can_transition(&self, from: &ContractState, event: &TransitionEvent) -> bool {
        self.transitions.contains_key(&(from.clone(), event.clone()))
    }

    /// Get the transition details for a state and event
    pub fn get_transition(&self, from: &ContractState, event: &TransitionEvent) -> Option<&StateTransition> {
        self.transitions.get(&(from.clone(), event.clone()))
    }

    /// Execute a state transition
    pub fn transition(&mut self, file: &str, event: TransitionEvent) -> Result<ContractState> {
        let current_state = self.get_state(file);
        
        if !self.can_transition(&current_state, &event) {
            return Err(anyhow!(
                "Invalid transition: {:?} → {:?} for file {}",
                current_state, event, file
            ));
        }

        let transition = self.get_transition(&current_state, &event)
            .ok_or_else(|| anyhow!("Transition not found"))?;

        // Validate transition conditions
        self.validate_transition_conditions(file, transition)?;

        // Execute the transition
        let new_state = transition.to.clone();
        self.set_state(file.to_string(), new_state.clone());

        Ok(new_state)
    }

    /// Validate transition conditions
    fn validate_transition_conditions(&self, file: &str, transition: &StateTransition) -> Result<()> {
        // This is a simplified validation - in practice, you would check each condition
        // against the actual file state, Git notes, etc.
        
        match transition.event {
            TransitionEvent::DetectContract => {
                // Check if file matches .gitattributes patterns
                if !self.file_has_contract_attribute(file)? {
                    return Err(anyhow!("File {} does not have contract attribute", file));
                }
                
                // Check if no existing Git note
                if self.has_git_note(file)? {
                    return Err(anyhow!("File {} already has Git note", file));
                }
            },
            
            TransitionEvent::ValidateContract => {
                // Check if validation would pass
                if !self.would_validation_pass(file)? {
                    return Err(anyhow!("Validation would fail for file {}", file));
                }
            },
            
            TransitionEvent::LockContract => {
                // Check if file is committed
                if !self.is_file_committed(file)? {
                    return Err(anyhow!("File {} is not committed", file));
                }
                
                // Check if no pending changes
                if self.has_pending_changes(file)? {
                    return Err(anyhow!("File {} has pending changes", file));
                }
            },
            
            TransitionEvent::ModifyContract => {
                // Check if file content has changed
                if !self.has_content_changed(file)? {
                    return Err(anyhow!("File {} content has not changed", file));
                }
            },
            
            TransitionEvent::RegenCodegen => {
                // Check if codegen regeneration would succeed
                if !self.would_codegen_succeed(file)? {
                    return Err(anyhow!("Codegen regeneration would fail for file {}", file));
                }
            },
            
            TransitionEvent::ReleaseProof => {
                // Check if Merkle chain is valid
                if !self.is_merkle_chain_valid(file)? {
                    return Err(anyhow!("Merkle chain is invalid for file {}", file));
                }
            },
        }

        Ok(())
    }

    /// Check if file has contract attribute in .gitattributes
    fn file_has_contract_attribute(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would read .gitattributes
        Ok(file.ends_with(".rs") || file.ends_with(".toml") || file.ends_with(".md"))
    }

    /// Check if file has existing Git note
    fn has_git_note(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would check Git notes
        Ok(false)
    }

    /// Check if validation would pass
    fn would_validation_pass(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would run actual validation
        Ok(true)
    }

    /// Check if file is committed
    fn is_file_committed(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would check Git status
        Ok(true)
    }

    /// Check if file has pending changes
    fn has_pending_changes(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would check Git status
        Ok(false)
    }

    /// Check if file content has changed
    fn has_content_changed(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would compare hashes
        Ok(true)
    }

    /// Check if codegen regeneration would succeed
    fn would_codegen_succeed(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would test codegen
        Ok(true)
    }

    /// Check if Merkle chain is valid
    fn is_merkle_chain_valid(&self, file: &str) -> Result<bool> {
        // Simplified implementation - would verify Merkle chain
        Ok(true)
    }

    /// Create a validation note for a state transition
    pub fn create_validation_note(
        &self,
        file: &str,
        scope: ValidationScope,
        state: &ContractState,
        event: &TransitionEvent,
        hash: &str,
    ) -> ValidationNote {
        ValidationNote {
            file: file.to_string(),
            contract: "validation".to_string(),
            state: format!("{:?}", state),
            scope: format!("{:?}", scope),
            hash: format!("sha256:{}", hash),
            validated_by: "xtask-contract-validate 0.2.0".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            parent_scope: None,
            parent_hash: None,
            child_scopes: vec![],
            validated: state == &ContractState::VALIDATED || state == &ContractState::LOCKED,
            validation_errors: vec![],
            contract_type: "rust_validation".to_string(),
            tool: "xtask-contract-validate".to_string(),
            commit_hash: None,
            validation_duration_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Create a transition log entry
    pub fn create_transition_log(
        &self,
        file: &str,
        from: &ContractState,
        to: &ContractState,
        event: &TransitionEvent,
        hash: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "transition": format!("{:?}", event),
            "from": format!("{:?}", from),
            "to": format!("{:?}", to),
            "file": file,
            "hash": format!("sha256:{}", hash),
            "tool": "xtask-contract-validate 0.2.0",
            "timestamp": Utc::now().to_rfc3339(),
            "commit_hash": None,
            "metadata": {
                "validation_errors": [],
                "duration_ms": 0
            }
        })
    }

    /// Get all valid transitions for a state
    pub fn get_valid_transitions(&self, state: &ContractState) -> Vec<&StateTransition> {
        self.transitions
            .iter()
            .filter(|((from_state, _), _)| from_state == state)
            .map(|(_, transition)| transition)
            .collect()
    }

    /// Get the state machine as a JSON schema
    pub fn to_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "states": {
                "UNTRACKED": "File has no contract or codegen attribute",
                "UNVALIDATED": "File has contract/codegen attribute but no proof (note)",
                "VALIDATED": "File has contract/codegen attribute + matching note with hash",
                "LOCKED": "File is validated + no further modifications allowed without re-validation"
            },
            "transitions": self.transitions
                .iter()
                .map(|((from, event), transition)| {
                    serde_json::json!({
                        "from": format!("{:?}", from),
                        "to": format!("{:?}", transition.to),
                        "event": format!("{:?}", event),
                        "action": transition.action,
                        "conditions": transition.conditions
                    })
                })
                .collect::<Vec<_>>()
        })
    }
}

impl Default for ContractStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let machine = ContractStateMachine::new();
        assert!(!machine.transitions.is_empty());
    }

    #[test]
    fn test_valid_transition() {
        let mut machine = ContractStateMachine::new();
        let file = "test.rs";
        
        // UNTRACKED → UNVALIDATED
        let result = machine.transition(file, TransitionEvent::DetectContract);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ContractState::UNVALIDATED);
    }

    #[test]
    fn test_invalid_transition() {
        let mut machine = ContractStateMachine::new();
        let file = "test.rs";
        
        // Try to go directly from UNTRACKED to VALIDATED (invalid)
        let result = machine.transition(file, TransitionEvent::ValidateContract);
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_chain() {
        let mut machine = ContractStateMachine::new();
        let file = "test.rs";
        
        // UNTRACKED → UNVALIDATED → VALIDATED → LOCKED
        assert_eq!(machine.get_state(file), ContractState::UNTRACKED);
        
        machine.transition(file, TransitionEvent::DetectContract).unwrap();
        assert_eq!(machine.get_state(file), ContractState::UNVALIDATED);
        
        machine.transition(file, TransitionEvent::ValidateContract).unwrap();
        assert_eq!(machine.get_state(file), ContractState::VALIDATED);
        
        machine.transition(file, TransitionEvent::LockContract).unwrap();
        assert_eq!(machine.get_state(file), ContractState::LOCKED);
    }

    #[test]
    fn test_schema_generation() {
        let machine = ContractStateMachine::new();
        let schema = machine.to_schema();
        
        assert!(schema.get("states").is_some());
        assert!(schema.get("transitions").is_some());
    }
} 
