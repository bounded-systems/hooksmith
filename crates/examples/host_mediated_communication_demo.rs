//! Host-Mediated Communication Demo
//!
//! This example demonstrates the different communication patterns available in Hooksmith:
//! 1. Direct component linking (fastest)
//! 2. Event-driven communication (system operations)
//! 3. wRPC distributed communication (scalable)
//!
//! Run with: cargo run --example host_mediated_communication_demo

use anyhow::Result;
use hooksmith::orchestrator::HooksmithOrchestrator;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Hooksmith Host-Mediated Communication Demo");
    println!("=============================================\n");

    // Initialize orchestrator
    let mut orchestrator = HooksmithOrchestrator::new().await?;
    println!("✅ Orchestrator initialized");

    // Demo data
    let contract_data = r#"
    {
        "name": "demo-contract",
        "version": "1.0.0",
        "rules": ["no-secrets", "valid-json", "required-fields"],
        "metadata": {
            "author": "demo-user",
            "created": "2024-01-01T00:00:00Z"
        }
    }
    "#;

    // Test different communication patterns
    demo_direct_linking(&orchestrator, contract_data).await?;
    demo_event_driven(&orchestrator, contract_data).await?;
    demo_wrpc_communication(&orchestrator, contract_data).await?;
    demo_adaptive_pattern_selection(&orchestrator, contract_data).await?;

    println!("\n🎉 Demo completed successfully!");
    Ok(())
}

/// Demo 1: Direct Component Linking (Fastest Path)
async fn demo_direct_linking(
    orchestrator: &HooksmithOrchestrator,
    contract_data: &str,
) -> Result<()> {
    println!("\n🔗 Demo 1: Direct Component Linking");
    println!("-----------------------------------");

    let start = Instant::now();

    // Try to load components for direct linking
    if let Err(e) = orchestrator.load_linked_components().await {
        println!("⚠️  Direct linking not available: {}", e);
        println!("   This is expected if components aren't built yet");
        return Ok(());
    }

    // Check if direct linking is available
    if orchestrator.has_linked_component("contract-checker") {
        println!("✅ Direct linking available for contract-checker");

        let validation_start = Instant::now();
        let result = orchestrator.validate_contract_direct(contract_data).await?;
        let validation_duration = validation_start.elapsed();

        println!(
            "✅ Direct validation completed in {:?}",
            validation_duration
        );
        println!("   Success: {}", result.success);
        println!("   Errors: {}", result.errors.len());
        println!("   Warnings: {}", result.warnings.len());

        if let Some(details) = &result.details {
            println!("   Details: {}", details);
        }
    } else {
        println!("❌ Direct linking not available for contract-checker");
        println!("   Components may need to be built with: cargo component build");
    }

    let total_duration = start.elapsed();
    println!("⏱️  Total direct linking demo time: {:?}", total_duration);

    Ok(())
}

/// Demo 2: Event-Driven Communication (System Operations)
async fn demo_event_driven(
    orchestrator: &HooksmithOrchestrator,
    contract_data: &str,
) -> Result<()> {
    println!("\n🔄 Demo 2: Event-Driven Communication");
    println!("-------------------------------------");

    let start = Instant::now();

    // Create a file to validate
    let file_path = "demo-contract.json";
    tokio::fs::write(file_path, contract_data).await?;
    println!("📄 Created demo file: {}", file_path);

    // Validate via event-driven communication
    let validation_start = Instant::now();
    let result = orchestrator
        .validate_contract_via_events("demo-contract", file_path, contract_data, true, true)
        .await?;
    let validation_duration = validation_start.elapsed();

    println!(
        "✅ Event-driven validation completed in {:?}",
        validation_duration
    );
    println!("   Success: {}", result.success);
    println!("   Errors: {}", result.errors.len());
    println!("   Warnings: {}", result.warnings.len());

    if let Some(details) = &result.details {
        println!("   Details: {}", details);
    }

    // Clean up
    let _ = tokio::fs::remove_file(file_path).await;
    println!("🗑️  Cleaned up demo file");

    let total_duration = start.elapsed();
    println!("⏱️  Total event-driven demo time: {:?}", total_duration);

    Ok(())
}

/// Demo 3: wRPC Distributed Communication
async fn demo_wrpc_communication(
    orchestrator: &HooksmithOrchestrator,
    contract_data: &str,
) -> Result<()> {
    println!("\n🌐 Demo 3: wRPC Distributed Communication");
    println!("----------------------------------------");

    let start = Instant::now();

    // Try to initialize wRPC client (this would fail if no server is running)
    match orchestrator.init_wrpc_client("tcp://localhost:8080").await {
        Ok(_) => {
            println!("✅ wRPC client initialized");

            let validation_start = Instant::now();
            let result = orchestrator
                .validate_contract_distributed(contract_data)
                .await?;
            let validation_duration = validation_start.elapsed();

            println!(
                "✅ Distributed validation completed in {:?}",
                validation_duration
            );
            println!("   Success: {}", result.success);
            println!("   Errors: {}", result.errors.len());
            println!("   Warnings: {}", result.warnings.len());
        }
        Err(e) => {
            println!("⚠️  wRPC client initialization failed: {}", e);
            println!("   This is expected if no wRPC server is running");
            println!("   To test wRPC, start a server with: cargo run --example wrpc_server");
        }
    }

    let total_duration = start.elapsed();
    println!("⏱️  Total wRPC demo time: {:?}", total_duration);

    Ok(())
}

/// Demo 4: Adaptive Pattern Selection
async fn demo_adaptive_pattern_selection(
    orchestrator: &HooksmithOrchestrator,
    contract_data: &str,
) -> Result<()> {
    println!("\n🎯 Demo 4: Adaptive Pattern Selection");
    println!("------------------------------------");

    let start = Instant::now();

    // Simulate different operation types
    let operations = vec![
        ("pure-computation", "validation"),
        ("system-operation", "file-read"),
        ("distributed", "remote-validation"),
        ("performance-critical", "batch-processing"),
    ];

    for (operation_type, operation_name) in operations {
        println!(
            "\n🔍 Testing operation: {} ({})",
            operation_name, operation_type
        );

        let operation_start = Instant::now();

        // In a real implementation, this would use the adaptive pattern selection
        let pattern = match operation_type {
            "pure-computation" => {
                if orchestrator.has_linked_component("contract-checker") {
                    "Direct Linking"
                } else {
                    "Event-Driven (fallback)"
                }
            }
            "system-operation" => "Event-Driven",
            "distributed" => "wRPC",
            "performance-critical" => "Direct Linking",
            _ => "Event-Driven",
        };

        println!("   Selected pattern: {}", pattern);

        // Simulate operation execution
        match pattern {
            "Direct Linking" => {
                if orchestrator.has_linked_component("contract-checker") {
                    let _ = orchestrator.validate_contract_direct(contract_data).await?;
                    println!("   ✅ Executed via direct linking");
                } else {
                    println!("   ⚠️  Direct linking not available, using fallback");
                    let _ = orchestrator
                        .validate_contract_via_events("demo", "data", contract_data, true, false)
                        .await?;
                }
            }
            "Event-Driven" => {
                let _ = orchestrator
                    .validate_contract_via_events("demo", "data", contract_data, true, false)
                    .await?;
                println!("   ✅ Executed via event-driven communication");
            }
            "wRPC" => {
                // Simulate wRPC call
                println!("   🌐 Simulating wRPC call...");
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                println!("   ✅ Executed via wRPC");
            }
            _ => {
                println!("   ❌ Unknown pattern");
            }
        }

        let operation_duration = operation_start.elapsed();
        println!("   ⏱️  Operation completed in {:?}", operation_duration);
    }

    let total_duration = start.elapsed();
    println!(
        "\n⏱️  Total adaptive selection demo time: {:?}",
        total_duration
    );

    Ok(())
}

/// Helper function to demonstrate WIT interface generation
fn demonstrate_wit_interfaces() {
    println!("\n📝 WIT Interface Examples");
    println!("------------------------");

    // Example WIT interface for validation
    let validation_wit = r#"
package hooksmith:validation;

interface validation {
    validate-contract: func(
        contract-name: string,
        content: string,
        config: validation-config
    ) -> result<validation-result, validation-error>;
}

type validation-config = record {
    strict: bool,
    store-proof: bool,
    max-errors: u32,
};

type validation-result = record {
    success: bool,
    errors: list<string>,
    warnings: list<string>,
    details: option<string>,
};

export validation;
"#;

    println!("✅ Validation interface defined");
    println!("   - Strongly typed function signatures");
    println!("   - Error handling with result types");
    println!("   - Structured data types");

    // Example bindgen usage
    let bindgen_example = r#"
// In your Rust code:
use wasmtime::component::bindgen;

bindgen!({
    path: "../wit",
    world: "hooksmith-world",
    async: true,
});

// Generated bindings provide type-safe component communication
let result = world.validate_contract("my-contract", content, config).await?;
"#;

    println!("✅ Bindgen generates type-safe bindings");
    println!("   - Automatic WIT to Rust type conversion");
    println!("   - Async function support");
    println!("   - Compile-time type checking");
}

/// Helper function to show performance characteristics
fn show_performance_characteristics() {
    println!("\n📊 Performance Characteristics");
    println!("-----------------------------");

    let characteristics = vec![
        (
            "Direct Linking",
            "~100ns",
            "Very High",
            "Low",
            "Pure computation chains",
        ),
        (
            "Event-Driven",
            "~1-10ms",
            "High",
            "Medium",
            "System operations",
        ),
        (
            "wRPC Local",
            "~100-500μs",
            "High",
            "Medium",
            "Process isolation",
        ),
        (
            "wRPC Network",
            "~10-100ms",
            "Medium",
            "High",
            "Distributed systems",
        ),
    ];

    for (pattern, latency, throughput, complexity, use_case) in characteristics {
        println!("🔹 {}:", pattern);
        println!("   Latency: {}", latency);
        println!("   Throughput: {}", throughput);
        println!("   Complexity: {}", complexity);
        println!("   Best for: {}", use_case);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_direct_linking_demo() {
        let orchestrator = HooksmithOrchestrator::new().await.unwrap();
        let contract_data = r#"{"name": "test"}"#;

        // This should not fail even if components aren't loaded
        let result = demo_direct_linking(&orchestrator, contract_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_driven_demo() {
        let orchestrator = HooksmithOrchestrator::new().await.unwrap();
        let contract_data = r#"{"name": "test"}"#;

        let result = demo_event_driven(&orchestrator, contract_data).await;
        assert!(result.is_ok());
    }
}
