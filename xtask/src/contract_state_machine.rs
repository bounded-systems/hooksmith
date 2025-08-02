use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Contract state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractState {
    UNTRACKED,
    UNVALIDATED,
    VALIDATED,
    LOCKED,
}

impl ContractState {
    /// Convert state to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            ContractState::UNTRACKED => "UNTRACKED",
            ContractState::UNVALIDATED => "UNVALIDATED",
            ContractState::VALIDATED => "VALIDATED",
            ContractState::LOCKED => "LOCKED",
        }
    }

    /// Parse state from string
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

/// Contract state machine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineConfig {
    pub states: HashMap<String, StateDefinition>,
    pub transitions: HashMap<String, TransitionDefinition>,
    pub contract_types: HashMap<String, ContractTypeDefinition>,
    pub validation_thresholds: ValidationThresholds,
}

/// State definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub description: String,
    pub allowed_transitions: Vec<String>,
    pub validation_rules: Vec<String>,
}

/// Transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefinition {
    pub from: Vec<String>,
    pub to: String,
    pub action: String,
    pub trigger: String,
    pub validation: Vec<String>,
}

/// Contract type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTypeDefinition {
    pub description: String,
    pub validation_steps: Vec<String>,
    pub metadata_fields: Vec<String>,
}

/// Validation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationThresholds {
    pub max_file_size_mb: u64,
    pub max_validation_time_seconds: u64,
    pub max_errors_per_file: u32,
    pub max_warnings_per_file: u32,
    pub stale_threshold_days: u32,
}

/// Contract state machine
pub struct StateMachine {
    config: StateMachineConfig,
}

impl StateMachine {
    /// Create a new state machine with default configuration
    pub fn new() -> Result<Self> {
        let config = Self::load_default_config()?;
        Ok(StateMachine { config })
    }

    /// Load configuration from file
    pub fn from_config_file(path: &Path) -> Result<Self> {
        let config = Self::load_config_from_file(path)?;
        Ok(StateMachine { config })
    }

    /// Check if a transition is valid
    pub fn is_valid_transition(
        &self,
        from_state: &ContractState,
        to_state: &ContractState,
        transition: &str,
    ) -> Result<bool> {
        let transition_def = self
            .config
            .transitions
            .get(transition)
            .context("Transition not found")?;

        // Check if from state is allowed
        let from_str = from_state.to_string();
        if !transition_def.from.contains(&from_str) {
            return Ok(false);
        }

        // Check if to state matches
        let to_str = to_state.to_string();
        if transition_def.to != to_str {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get allowed transitions for a state
    pub fn get_allowed_transitions(&self, state: &ContractState) -> Result<Vec<String>> {
        let state_str = state.to_string();
        let state_def = self
            .config
            .states
            .get(&state_str)
            .context("State not found")?;

        Ok(state_def.allowed_transitions.clone())
    }

    /// Validate a contract state
    pub fn validate_state(&self, state: &ContractState, file_path: &Path) -> Result<ValidationResult> {
        let state_str = state.to_string();
        let state_def = self
            .config
            .states
            .get(&state_str)
            .context("State not found")?;

        let mut result = ValidationResult::new();
        let start_time = SystemTime::now();

        // Check validation rules
        for rule in &state_def.validation_rules {
            match self.validate_rule(rule, file_path)? {
                RuleValidation::Valid => result.add_success(rule),
                RuleValidation::Invalid(reason) => result.add_error(rule, &reason),
                RuleValidation::Warning(reason) => result.add_warning(rule, &reason),
            }
        }

        // Check thresholds
        let elapsed = start_time.elapsed()?;
        if elapsed.as_secs() > self.config.validation_thresholds.max_validation_time_seconds {
            result.add_error(
                "validation_time",
                &format!("Validation took {} seconds, exceeded limit", elapsed.as_secs()),
            );
        }

        if result.error_count() > self.config.validation_thresholds.max_errors_per_file {
            result.add_error(
                "error_count",
                &format!("Too many errors: {}", result.error_count()),
            );
        }

        if result.warning_count() > self.config.validation_thresholds.max_warnings_per_file {
            result.add_warning(
                "warning_count",
                &format!("Too many warnings: {}", result.warning_count()),
            );
        }

        Ok(result)
    }

    /// Validate a specific rule
    fn validate_rule(&self, rule: &str, file_path: &Path) -> Result<RuleValidation> {
        match rule {
            "File must exist in repository" => {
                if file_path.exists() {
                    Ok(RuleValidation::Valid)
                } else {
                    Ok(RuleValidation::Invalid("File does not exist".to_string()))
                }
            }
            "No contract attributes set" => {
                // This would check .gitattributes
                Ok(RuleValidation::Valid) // Placeholder
            }
            "File must have contract attribute in .gitattributes" => {
                // This would check .gitattributes
                Ok(RuleValidation::Valid) // Placeholder
            }
            "No Git note exists for this file" => {
                // This would check Git notes
                Ok(RuleValidation::Valid) // Placeholder
            }
            "File content must be readable" => {
                if std::fs::read_to_string(file_path).is_ok() {
                    Ok(RuleValidation::Valid)
                } else {
                    Ok(RuleValidation::Invalid("File is not readable".to_string()))
                }
            }
            "Git note must exist with valid state" => {
                // This would check Git notes
                Ok(RuleValidation::Valid) // Placeholder
            }
            "File hash must match note hash" => {
                // This would check hash comparison
                Ok(RuleValidation::Valid) // Placeholder
            }
            "Contract validation must pass" => {
                // This would run contract validation
                Ok(RuleValidation::Valid) // Placeholder
            }
            "Timestamp must be recent (within 30 days)" => {
                // This would check timestamp
                Ok(RuleValidation::Valid) // Placeholder
            }
            "No modifications allowed without re-validation" => {
                // This would check file modification time
                Ok(RuleValidation::Valid) // Placeholder
            }
            "Parent scope must be validated" => {
                // This would check parent scope
                Ok(RuleValidation::Valid) // Placeholder
            }
            _ => Ok(RuleValidation::Warning(format!("Unknown rule: {}", rule))),
        }
    }

    /// Load default configuration
    fn load_default_config() -> Result<StateMachineConfig> {
        // This would load from the YAML file
        // For now, return a minimal config
        let mut states = HashMap::new();
        states.insert(
            "UNTRACKED".to_string(),
            StateDefinition {
                description: "File has no contract or codegen attribute".to_string(),
                allowed_transitions: vec!["UNVALIDATED".to_string()],
                validation_rules: vec![
                    "File must exist in repository".to_string(),
                    "No contract attributes set".to_string(),
                ],
            },
        );
        states.insert(
            "UNVALIDATED".to_string(),
            StateDefinition {
                description: "File has contract/codegen attribute but no proof".to_string(),
                allowed_transitions: vec!["VALIDATED".to_string()],
                validation_rules: vec![
                    "File must have contract attribute in .gitattributes".to_string(),
                    "No Git note exists for this file".to_string(),
                    "File content must be readable".to_string(),
                ],
            },
        );
        states.insert(
            "VALIDATED".to_string(),
            StateDefinition {
                description: "File has contract/codegen attribute + matching note with hash".to_string(),
                allowed_transitions: vec!["LOCKED".to_string(), "UNVALIDATED".to_string()],
                validation_rules: vec![
                    "Git note must exist with valid state".to_string(),
                    "File hash must match note hash".to_string(),
                    "Contract validation must pass".to_string(),
                    "Timestamp must be recent (within 30 days)".to_string(),
                ],
            },
        );
        states.insert(
            "LOCKED".to_string(),
            StateDefinition {
                description: "File is validated + no further modifications allowed".to_string(),
                allowed_transitions: vec!["UNVALIDATED".to_string()],
                validation_rules: vec![
                    "File must be in VALIDATED state".to_string(),
                    "No modifications allowed without re-validation".to_string(),
                    "Parent scope must be validated".to_string(),
                ],
            },
        );

        let mut transitions = HashMap::new();
        transitions.insert(
            "detect_contract".to_string(),
            TransitionDefinition {
                from: vec!["UNTRACKED".to_string()],
                to: "UNVALIDATED".to_string(),
                action: "Tag file via .gitattributes".to_string(),
                trigger: "File modification detected".to_string(),
                validation: vec![
                    "File must exist".to_string(),
                    "Contract attribute must be set".to_string(),
                    "No existing Git note".to_string(),
                ],
            },
        );
        transitions.insert(
            "validate_contract".to_string(),
            TransitionDefinition {
                from: vec!["UNVALIDATED".to_string()],
                to: "VALIDATED".to_string(),
                action: "Run validator → create Git note with hash".to_string(),
                trigger: "Manual validation or CI".to_string(),
                validation: vec![
                    "Contract validation must pass".to_string(),
                    "Hash must be computed correctly".to_string(),
                    "Git note must be created".to_string(),
                    "Parent scope must be validated".to_string(),
                ],
            },
        );

        let validation_thresholds = ValidationThresholds {
            max_file_size_mb: 100,
            max_validation_time_seconds: 300,
            max_errors_per_file: 100,
            max_warnings_per_file: 1000,
            stale_threshold_days: 30,
        };

        Ok(StateMachineConfig {
            states,
            transitions,
            contract_types: HashMap::new(),
            validation_thresholds,
        })
    }

    /// Load configuration from file
    fn load_config_from_file(path: &Path) -> Result<StateMachineConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: StateMachineConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        Ok(config)
    }
}

/// Rule validation result
#[derive(Debug, Clone)]
pub enum RuleValidation {
    Valid,
    Invalid(String),
    Warning(String),
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub successes: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub warnings: Vec<(String, String)>,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            successes: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_success(&mut self, rule: &str) {
        self.successes.push(rule.to_string());
    }

    pub fn add_error(&mut self, rule: &str, reason: &str) {
        self.errors.push((rule.to_string(), reason.to_string()));
    }

    pub fn add_warning(&mut self, rule: &str, reason: &str) {
        self.warnings.push((rule.to_string(), reason.to_string()));
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> u32 {
        self.errors.len() as u32
    }

    pub fn warning_count(&self) -> u32 {
        self.warnings.len() as u32
    }

    pub fn success_count(&self) -> u32 {
        self.successes.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let sm = StateMachine::new().unwrap();
        assert!(!sm.config.states.is_empty());
        assert!(!sm.config.transitions.is_empty());
    }

    #[test]
    fn test_valid_transition() {
        let sm = StateMachine::new().unwrap();
        let result = sm
            .is_valid_transition(
                &ContractState::UNTRACKED,
                &ContractState::UNVALIDATED,
                "detect_contract",
            )
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_invalid_transition() {
        let sm = StateMachine::new().unwrap();
        let result = sm
            .is_valid_transition(
                &ContractState::VALIDATED,
                &ContractState::UNTRACKED,
                "detect_contract",
            )
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_state_validation() {
        let sm = StateMachine::new().unwrap();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = sm
            .validate_state(&ContractState::UNTRACKED, temp_file.path())
            .unwrap();
        assert!(result.is_valid());
    }
} 