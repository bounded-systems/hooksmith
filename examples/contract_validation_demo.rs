use hooksmith::modules::contract_state_machine::{ContractState, TransitionEvent};
use hooksmith::modules::contract_validation::{
    ContractDefinition, ContractValidator, RuleSeverity, RuleType, ValidationRule,
};
use hooksmith::modules::hierarchical_validation::ValidationScope;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// Example user contract that will be validated
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct User {
    id: u32,
    username: String,
    email: String,
    #[serde(default)]
    is_active: bool,
}

/// Example configuration contract
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Config {
    version: String,
    environment: String,
    settings: HashMap<String, serde_json::Value>,
}

fn main() -> anyhow::Result<()> {
    println!("🔧 Hooksmith Contract Validation Demo");
    println!("=====================================\n");

    // 1. Create contract validator
    let mut validator = ContractValidator::new()?;
    println!("✅ Created contract validator");

    // 2. Define contracts with JSON Schema
    let user_contract = create_user_contract()?;
    let config_contract = create_config_contract()?;

    // 3. Register contracts
    validator.register_contract(user_contract)?;
    validator.register_contract(config_contract)?;
    println!("✅ Registered contracts with JSON Schema");

    // 4. Generate and display JSON Schema
    println!("\n📋 Generated JSON Schema for User:");
    let user_schema = ContractValidator::generate_schema::<User>();
    println!("{}", serde_json::to_string_pretty(&user_schema)?);

    // 5. Test validation with valid data
    println!("\n🧪 Testing validation with valid user data:");
    let valid_user = json!({
        "id": 1,
        "username": "john_doe",
        "email": "john@example.com",
        "is_active": true
    });

    let result = validator.validate_contract("user_contract", &valid_user, "user.json")?;
    println!(
        "Validation result: {}",
        if result.is_valid {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!("Schema hash: {}", result.schema_hash);
    println!("Content hash: {}", result.content_hash);

    // 6. Test validation with invalid data
    println!("\n🧪 Testing validation with invalid user data:");
    let invalid_user = json!({
        "id": -1, // Invalid: negative ID
        "username": "j", // Invalid: too short
        "email": "invalid-email", // Invalid: not a valid email
        "is_active": true
    });

    let result = validator.validate_contract("user_contract", &invalid_user, "user.json")?;
    println!(
        "Validation result: {}",
        if result.is_valid {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    if !result.errors.is_empty() {
        println!("Errors:");
        for error in &result.errors {
            println!("  - {}: {}", error.code, error.message);
        }
    }

    // 7. Store validation proof as Git note
    println!("\n💾 Storing validation proof as Git note:");
    validator.store_validation_proof("user.json", &result)?;
    println!("✅ Validation proof stored");

    // 8. Verify validation proof
    println!("\n🔍 Verifying validation proof:");
    let is_valid = validator.verify_validation_proof("user.json", &result.content_hash)?;
    println!(
        "Proof verification: {}",
        if is_valid { "✅ VALID" } else { "❌ INVALID" }
    );

    // 9. Test config validation
    println!("\n🧪 Testing config validation:");
    let valid_config = json!({
        "version": "1.0.0",
        "environment": "production",
        "settings": {
            "debug": false,
            "timeout": 30
        }
    });

    let result = validator.validate_contract("config_contract", &valid_config, "config.json")?;
    println!(
        "Config validation: {}",
        if result.is_valid {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    // 10. Create validation note for state machine
    println!("\n📝 Creating validation note for state machine:");
    let note = validator.create_validation_note(
        "user.json",
        ValidationScope::File,
        &ContractState::VALIDATED,
        &TransitionEvent::ValidateContract,
        &result.content_hash,
        &result,
    );
    println!("✅ Created validation note: {:?}", note);

    // 11. Demonstrate schema evolution
    println!("\n🔄 Demonstrating schema evolution:");
    let old_schema = ContractValidator::generate_schema::<User>();
    let old_hash = compute_schema_hash(&old_schema)?;
    println!("Old schema hash: {}", old_hash);

    // Simulate schema change by adding a new field
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    struct UserV2 {
        id: u32,
        username: String,
        email: String,
        #[serde(default)]
        is_active: bool,
        #[serde(default)]
        created_at: String, // New field
    }

    let new_schema = ContractValidator::generate_schema::<UserV2>();
    let new_hash = compute_schema_hash(&new_schema)?;
    println!("New schema hash: {}", new_hash);
    println!("Schema changed: {}", old_hash != new_hash);

    println!("\n🎉 Contract validation demo complete!");
    Ok(())
}

/// Create a user contract with validation rules
fn create_user_contract() -> anyhow::Result<ContractDefinition> {
    let schema = json!({
        "type": "object",
        "properties": {
            "id": {
                "type": "integer",
                "minimum": 1,
                "description": "User ID must be positive"
            },
            "username": {
                "type": "string",
                "minLength": 3,
                "maxLength": 50,
                "pattern": "^[a-zA-Z0-9_]+$",
                "description": "Username must be 3-50 characters, alphanumeric + underscore"
            },
            "email": {
                "type": "string",
                "format": "email",
                "description": "Must be a valid email address"
            },
            "is_active": {
                "type": "boolean",
                "default": false,
                "description": "Whether the user account is active"
            }
        },
        "required": ["id", "username", "email"],
        "additionalProperties": false
    });

    let rules = vec![
        ValidationRule {
            name: "username_pattern".to_string(),
            description: Some("Username must match pattern".to_string()),
            rule_type: RuleType::Pattern,
            parameters: json!({
                "pattern": "^[a-zA-Z0-9_]{3,50}$"
            })
            .as_object()
            .unwrap()
            .clone(),
            required: true,
            severity: RuleSeverity::Error,
        },
        ValidationRule {
            name: "email_format".to_string(),
            description: Some("Email must be valid format".to_string()),
            rule_type: RuleType::Pattern,
            parameters: json!({
                "pattern": r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
            })
            .as_object()
            .unwrap()
            .clone(),
            required: true,
            severity: RuleSeverity::Error,
        },
    ];

    Ok(ContractDefinition {
        name: "user_contract".to_string(),
        version: "1.0.0".to_string(),
        description: Some("User data validation contract".to_string()),
        schema,
        rules,
        metadata: HashMap::new(),
    })
}

/// Create a config contract with validation rules
fn create_config_contract() -> anyhow::Result<ContractDefinition> {
    let schema = json!({
        "type": "object",
        "properties": {
            "version": {
                "type": "string",
                "pattern": r"^\d+\.\d+\.\d+$",
                "description": "Semantic version string"
            },
            "environment": {
                "type": "string",
                "enum": ["development", "staging", "production"],
                "description": "Deployment environment"
            },
            "settings": {
                "type": "object",
                "description": "Configuration settings"
            }
        },
        "required": ["version", "environment"],
        "additionalProperties": false
    });

    let rules = vec![
        ValidationRule {
            name: "version_format".to_string(),
            description: Some("Version must be semantic version".to_string()),
            rule_type: RuleType::Pattern,
            parameters: json!({
                "pattern": r"^\d+\.\d+\.\d+$"
            })
            .as_object()
            .unwrap()
            .clone(),
            required: true,
            severity: RuleSeverity::Error,
        },
        ValidationRule {
            name: "file_size_limit".to_string(),
            description: Some("Config file must be under 1MB".to_string()),
            rule_type: RuleType::FileSize,
            parameters: json!({
                "max_size": 1024 * 1024 // 1MB
            })
            .as_object()
            .unwrap()
            .clone(),
            required: true,
            severity: RuleSeverity::Warning,
        },
    ];

    Ok(ContractDefinition {
        name: "config_contract".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Configuration validation contract".to_string()),
        schema,
        rules,
        metadata: HashMap::new(),
    })
}

/// Compute SHA-256 hash of schema
fn compute_schema_hash(schema: &serde_json::Value) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let schema_json = serde_json::to_string(schema)?;
    let mut hasher = Sha256::new();
    hasher.update(schema_json.as_bytes());
    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_contract_creation() {
        let contract = create_user_contract().unwrap();
        assert_eq!(contract.name, "user_contract");
        assert_eq!(contract.version, "1.0.0");
        assert!(!contract.rules.is_empty());
    }

    #[test]
    fn test_config_contract_creation() {
        let contract = create_config_contract().unwrap();
        assert_eq!(contract.name, "config_contract");
        assert_eq!(contract.version, "1.0.0");
        assert!(!contract.rules.is_empty());
    }

    #[test]
    fn test_schema_generation() {
        let schema = ContractValidator::generate_schema::<User>();
        assert!(schema.is_object());
        assert_eq!(schema["title"], "User");
    }

    #[test]
    fn test_schema_evolution() {
        let old_schema = ContractValidator::generate_schema::<User>();
        let new_schema = ContractValidator::generate_schema::<UserV2>();

        let old_hash = compute_schema_hash(&old_schema).unwrap();
        let new_hash = compute_schema_hash(&new_schema).unwrap();

        assert_ne!(old_hash, new_hash, "Schema hashes should be different");
    }
}
