use crate::modules::contract_state_machine::{ContractState, TransitionEvent};
use crate::modules::contract_validation::{
    ContractDefinition, ContractValidator, RuleSeverity, RuleType, ValidationRule,
};
use crate::modules::hierarchical_validation::ValidationScope;
use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use serde_json::json;
use std::collections::HashMap;

/// Commands for contract validation operations
#[derive(Debug, Subcommand)]
pub enum ContractValidationCommand {
    /// Validate data against a contract
    Validate(ValidateArgs),
    /// Generate JSON Schema from Rust types
    GenerateSchema(GenerateSchemaArgs),
    /// Store validation proof as Git note
    StoreProof(StoreProofArgs),
    /// Verify validation proof from Git note
    VerifyProof(VerifyProofArgs),
    /// Create a new contract definition
    Create(CreateArgs),
    /// List registered contracts
    List,
    /// Run the contract validation demo
    Demo,
}

/// Arguments for contract validation
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Contract name to validate against
    #[arg(long)]
    pub contract: String,

    /// File path to validate
    #[arg(long)]
    pub file: String,

    /// Whether to store proof after validation
    #[arg(long, default_value = "false")]
    pub store_proof: bool,

    /// Whether to use strict validation
    #[arg(long, default_value = "false")]
    pub strict: bool,
}

/// Arguments for schema generation
#[derive(Debug, Args)]
pub struct GenerateSchemaArgs {
    /// Rust type to generate schema for
    #[arg(long)]
    pub type_name: String,

    /// Output file for the schema
    #[arg(long)]
    pub output: Option<String>,
}

/// Arguments for storing validation proofs
#[derive(Debug, Args)]
pub struct StoreProofArgs {
    /// File path to store proof for
    #[arg(long)]
    pub file: String,

    /// Validation result JSON
    #[arg(long)]
    pub result: String,
}

/// Arguments for verifying validation proofs
#[derive(Debug, Args)]
pub struct VerifyProofArgs {
    /// File path to verify proof for
    #[arg(long)]
    pub file: String,

    /// Expected content hash
    #[arg(long)]
    pub hash: String,
}

/// Arguments for creating new contracts
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Contract name
    #[arg(long)]
    pub name: String,

    /// Contract version
    #[arg(long, default_value = "1.0.0")]
    pub version: String,

    /// Contract description
    #[arg(long)]
    pub description: Option<String>,

    /// JSON Schema file
    #[arg(long)]
    pub schema_file: String,

    /// Output file for contract definition
    #[arg(long)]
    pub output: Option<String>,
}

/// Run contract validation commands
pub async fn run_contract_validation(cmd: ContractValidationCommand) -> Result<()> {
    match cmd {
        ContractValidationCommand::Validate(args) => validate_contract(args).await,
        ContractValidationCommand::GenerateSchema(args) => generate_schema(args).await,
        ContractValidationCommand::StoreProof(args) => store_proof(args).await,
        ContractValidationCommand::VerifyProof(args) => verify_proof(args).await,
        ContractValidationCommand::Create(args) => create_contract(args).await,
        ContractValidationCommand::List => list_contracts().await,
        ContractValidationCommand::Demo => run_demo().await,
    }
}

async fn validate_contract(args: ValidateArgs) -> Result<()> {
    println!("🔧 Validating contract: {}", style(&args.contract).cyan());

    let mut validator = ContractValidator::new()?;

    // Load contract definition (in a real implementation, this would load from file)
    let contract = load_contract_definition(&args.contract)?;
    validator.register_contract(contract)?;

    // Load and parse the file to validate
    let content = std::fs::read_to_string(&args.file)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    // Perform validation
    let result = validator.validate_contract(&args.contract, &data, &args.file)?;

    // Display results
    if result.is_valid {
        println!("✅ {}", style("Validation PASSED").green());
    } else {
        println!("❌ {}", style("Validation FAILED").red());
    }

    println!("Schema hash: {}", style(&result.schema_hash).dim());
    println!("Content hash: {}", style(&result.content_hash).dim());
    println!("Validated by: {}", style(&result.validated_by).dim());

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for error in &result.errors {
            println!("  ❌ {}: {}", error.code, error.message);
            if let Some(path) = &error.path {
                println!("     Path: {path}");
            }
        }
    }

    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  ⚠️  {}: {}", warning.code, warning.message);
        }
    }

    // Store proof if requested
    if args.store_proof {
        validator.store_validation_proof(&args.file, &result)?;
        println!("💾 Validation proof stored as Git note");
    }

    // Create validation note for state machine
    let _note = validator.create_validation_note(
        &args.file,
        ValidationScope::File,
        &ContractState::VALIDATED,
        &TransitionEvent::ValidateContract,
        &result.content_hash,
        &result,
    );

    println!("📝 Created validation note for state machine");

    Ok(())
}

async fn generate_schema(args: GenerateSchemaArgs) -> Result<()> {
    println!(
        "📋 Generating JSON Schema for type: {}",
        style(&args.type_name).cyan()
    );

    // This is a simplified implementation
    // In a real implementation, you would use reflection or a type registry
    let schema = match args.type_name.as_str() {
        "User" => {
            #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            struct User {
                id: u32,
                username: String,
                email: String,
                #[serde(default)]
                is_active: bool,
            }
            ContractValidator::generate_schema::<User>()
        }
        "Config" => {
            #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            struct Config {
                version: String,
                environment: String,
                settings: HashMap<String, serde_json::Value>,
            }
            ContractValidator::generate_schema::<Config>()
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown type: {}", args.type_name));
        }
    };

    let schema_json = serde_json::to_string_pretty(&schema)?;

    if let Some(output) = args.output {
        std::fs::write(&output, schema_json)?;
        println!("✅ Schema written to: {}", style(&output).green());
    } else {
        println!("{schema_json}");
    }

    Ok(())
}

async fn store_proof(args: StoreProofArgs) -> Result<()> {
    println!(
        "💾 Storing validation proof for: {}",
        style(&args.file).cyan()
    );

    let validator = ContractValidator::new()?;
    let result: crate::modules::contract_validation::ContractValidationResult =
        serde_json::from_str(&args.result)?;

    validator.store_validation_proof(&args.file, &result)?;
    println!("✅ Validation proof stored as Git note");

    Ok(())
}

async fn verify_proof(args: VerifyProofArgs) -> Result<()> {
    println!(
        "🔍 Verifying validation proof for: {}",
        style(&args.file).cyan()
    );

    let validator = ContractValidator::new()?;
    let is_valid = validator.verify_validation_proof(&args.file, &args.hash)?;

    if is_valid {
        println!("✅ {}", style("Proof verification PASSED").green());
    } else {
        println!("❌ {}", style("Proof verification FAILED").red());
    }

    Ok(())
}

async fn create_contract(args: CreateArgs) -> Result<()> {
    println!("📝 Creating contract: {}", style(&args.name).cyan());

    // Load schema from file
    let schema_content = std::fs::read_to_string(&args.schema_file)?;
    let schema: serde_json::Value = serde_json::from_str(&schema_content)?;

    // Create contract definition
    let contract = ContractDefinition {
        name: args.name,
        version: args.version,
        description: args.description,
        schema,
        rules: Vec::new(), // Would be loaded from separate file in real implementation
        metadata: HashMap::new(),
    };

    let contract_json = serde_json::to_string_pretty(&contract)?;

    if let Some(output) = args.output {
        std::fs::write(&output, contract_json)?;
        println!("✅ Contract written to: {}", style(&output).green());
    } else {
        println!("{contract_json}");
    }

    Ok(())
}

async fn list_contracts() -> Result<()> {
    println!("📋 Registered contracts:");

    // In a real implementation, this would load from a registry or database
    let contracts = vec![
        ("user_contract", "1.0.0", "User data validation"),
        ("config_contract", "1.0.0", "Configuration validation"),
        ("api_contract", "1.0.0", "API response validation"),
    ];

    for (name, version, description) in contracts {
        println!(
            "  • {} {} - {}",
            style(name).cyan(),
            style(version).dim(),
            description
        );
    }

    Ok(())
}

async fn run_demo() -> Result<()> {
    println!("🎬 Running contract validation demo...");

    // This would run the full demo from the example
    // For now, just show a simplified version
    let mut validator = ContractValidator::new()?;

    // Create a simple contract
    let contract = ContractDefinition {
        name: "demo_contract".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Demo contract for validation".to_string()),
        schema: json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" },
                "count": { "type": "integer", "minimum": 0 }
            },
            "required": ["message", "count"]
        }),
        rules: vec![ValidationRule {
            name: "message_length".to_string(),
            description: Some("Message must be at least 5 characters".to_string()),
            rule_type: RuleType::Pattern,
            parameters: json!({
                "pattern": "^.{5,}$"
            })
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
            required: true,
            severity: RuleSeverity::Error,
        }],
        metadata: HashMap::new(),
    };

    validator.register_contract(contract)?;

    // Test validation
    let valid_data = json!({
        "message": "Hello, world!",
        "count": 42
    });

    let result = validator.validate_contract("demo_contract", &valid_data, "demo.json")?;

    if result.is_valid {
        println!("✅ Demo validation PASSED");
    } else {
        println!("❌ Demo validation FAILED");
    }

    println!("🎉 Demo complete!");

    Ok(())
}

fn load_contract_definition(name: &str) -> Result<ContractDefinition> {
    // In a real implementation, this would load from a file or database
    // For now, return a simple contract
    Ok(ContractDefinition {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some("Auto-generated contract".to_string()),
        schema: json!({
            "type": "object",
            "properties": {
                "data": { "type": "string" }
            },
            "required": ["data"]
        }),
        rules: Vec::new(),
        metadata: HashMap::new(),
    })
}
