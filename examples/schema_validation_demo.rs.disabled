//! Example demonstrating Lefthook schema validation
//!
//! This example shows how to use the enhanced Lefthook wrapper with
//! official JSON schema validation.

use hooksmith::modules::lefthook;
use serde_json::json;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Lefthook Schema Validation Demo");
    println!("===================================");

    // Example 1: Validate a valid configuration
    println!("\n1. Testing valid configuration...");
    let valid_config = json!({
        "pre-commit": {
            "commands": {
                "rustfmt": {
                    "run": "cargo fmt --all -- --check",
                    "glob": "*.rs",
                    "stage_fixed": true
                },
                "clippy": {
                    "run": "cargo clippy --all-targets --all-features -- -D warnings",
                    "glob": "*.rs"
                }
            }
        },
        "pre-push": {
            "commands": {
                "test": {
                    "run": "cargo test --all-targets --all-features --release"
                }
            }
        }
    });

    match lefthook::validate_against_schema(&valid_config).await {
        Ok(()) => println!("✅ Valid configuration passed schema validation"),
        Err(e) => println!("❌ Valid configuration failed: {}", e),
    }

    // Example 2: Validate an invalid configuration
    println!("\n2. Testing invalid configuration...");
    let invalid_config = json!({
        "pre-commit": {
            "invalid_field": "this should not be allowed",
            "commands": {
                "test": {
                    "invalid_property": "this should fail"
                }
            }
        }
    });

    match lefthook::validate_against_schema(&invalid_config).await {
        Ok(()) => println!("❌ Invalid configuration unexpectedly passed"),
        Err(e) => println!("✅ Invalid configuration correctly failed: {}", e),
    }

    // Example 3: Generate a configuration with schema validation
    println!("\n3. Generating configuration with schema validation...");
    let output_path = Path::new("examples/generated_lefthook.yml");

    match lefthook::generate_lefthook_config(
        output_path,
        "target/hooks",
        Some(vec!["components/worktree-runner".to_string()]),
        true, // Enable schema validation
    )
    .await
    {
        Ok(()) => println!("✅ Configuration generated and validated successfully"),
        Err(e) => println!("❌ Configuration generation failed: {}", e),
    }

    // Example 4: Validate an existing configuration file
    println!("\n4. Validating existing configuration file...");
    let existing_config_path = Path::new("lefthook.yml");

    if existing_config_path.exists() {
        match lefthook::validate_existing_config(existing_config_path).await {
            Ok(()) => println!("✅ Existing configuration is valid"),
            Err(e) => println!("❌ Existing configuration is invalid: {}", e),
        }
    } else {
        println!("⚠️  No existing lefthook.yml file found to validate");
    }

    println!("\n🎉 Schema validation demo completed!");
    Ok(())
}
