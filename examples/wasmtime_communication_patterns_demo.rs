//! Wasmtime Component Communication Patterns Demo
//!
//! This example demonstrates the different communication patterns available
//! in Hooksmith's hybrid WIT + native Rust architecture:
//!
//! 1. Direct Component Linking (fast path for pure computation)
//! 2. Event-Driven Communication (for system operations)
//! 3. Shared Memory (for performance-critical operations)

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::orchestrator::HooksmithOrchestrator;
use crate::xtask::event_bus::{EventBusConfig, HooksmithEvent};

/// Demo different Wasmtime communication patterns
pub async fn run_wasmtime_communication_demo() -> Result<()> {
    println!("🚀 Wasmtime Component Communication Patterns Demo");
    println!("================================================\n");

    // Initialize orchestrator
    let mut orchestrator = HooksmithOrchestrator::new().await?;

    // Demo 1: Direct Component Linking (Fast Path)
    println!("1️⃣ Direct Component Linking (Fast Path)");
    println!("----------------------------------------");
    await_direct_linking_demo(&mut orchestrator).await?;

    // Demo 2: Event-Driven Communication
    println!("\n2️⃣ Event-Driven Communication");
    println!("-------------------------------");
    await_event_driven_demo(&orchestrator).await?;

    // Demo 3: Performance Comparison
    println!("\n3️⃣ Performance Comparison");
    println!("---------------------------");
    await_performance_comparison_demo(&orchestrator).await?;

    // Demo 4: Hybrid Workflow
    println!("\n4️⃣ Hybrid Workflow");
    println!("-------------------");
    await_hybrid_workflow_demo(&orchestrator).await?;

    println!("\n✅ All communication pattern demos completed successfully!");
    Ok(())
}

/// Demo 1: Direct Component Linking
async fn await_direct_linking_demo(orchestrator: &mut HooksmithOrchestrator) -> Result<()> {
    println!("Loading components for direct linking...");

    // Initialize component linker
    orchestrator.init_component_linker().await?;

    // Load linked components
    match orchestrator.load_linked_components().await {
        Ok(_) => {
            println!("✅ Components loaded successfully");
            println!(
                "Linked components: {:?}",
                orchestrator.list_linked_components()
            );

            // Demonstrate direct validation call
            let contract_data = r#"{"name": "test_contract", "version": "1.0.0"}"#;
            let start = Instant::now();

            let result = orchestrator.validate_contract_direct(contract_data).await?;
            let duration = start.elapsed();

            println!("✅ Direct validation completed in {:?}", duration);
            println!("   Result: {:?}", result.success);

            if orchestrator.has_linked_component("contract-checker") {
                println!("✅ Direct linking available for contract-checker");
            } else {
                println!("⚠️  Direct linking not available, using event-driven fallback");
            }
        }
        Err(e) => {
            println!("⚠️  Direct linking failed: {}", e);
            println!("   Falling back to event-driven communication");
        }
    }

    Ok(())
}

/// Demo 2: Event-Driven Communication
async fn await_event_driven_demo(orchestrator: &HooksmithOrchestrator) -> Result<()> {
    println!("Demonstrating event-driven communication...");

    // Step 1: File read via events
    println!("📁 Step 1: Reading file via events");
    let file_content = orchestrator
        .read_file_via_events("examples/test_contract.json")
        .await?;
    println!("   File content length: {} bytes", file_content.len());

    // Step 2: Validation via events
    println!("🔍 Step 2: Validating contract via events");
    let validation_result = orchestrator
        .validate_contract_via_events(
            "test_contract",
            "examples/test_contract.json",
            &file_content,
            true,
            true,
        )
        .await?;
    println!("   Validation result: {:?}", validation_result.success);

    // Step 3: Store proof via events
    println!("💾 Step 3: Storing validation proof via events");
    orchestrator
        .store_proof_via_events("examples/test_contract.json", &validation_result)
        .await?;
    println!("   Proof stored successfully");

    // Get event bus statistics
    let stats = orchestrator.get_event_bus_statistics().await?;
    println!("📊 Event bus statistics:");
    println!("   Total events: {}", stats.total_events);
    println!("   Total handlers: {}", stats.total_handlers);
    println!("   Active subscriptions: {}", stats.active_subscriptions);
    println!("   Registered components: {}", stats.registered_components);
    println!("   Native handlers: {}", stats.registered_native_handlers);

    Ok(())
}

/// Demo 3: Performance Comparison
async fn await_performance_comparison_demo(orchestrator: &HooksmithOrchestrator) -> Result<()> {
    println!("Comparing performance of different communication patterns...");

    let contract_data = r#"{"name": "performance_test", "version": "1.0.0"}"#;
    let iterations = 100;

    // Test direct linking performance
    println!("🏃 Testing direct linking performance...");
    let direct_start = Instant::now();

    for _ in 0..iterations {
        let _result = orchestrator.validate_contract_direct(contract_data).await?;
    }

    let direct_duration = direct_start.elapsed();
    let direct_avg = direct_duration / iterations;

    println!(
        "   Direct linking: {} iterations in {:?} (avg: {:?})",
        iterations, direct_duration, direct_avg
    );

    // Test event-driven performance
    println!("📡 Testing event-driven performance...");
    let event_start = Instant::now();

    for _ in 0..iterations {
        let _result = orchestrator
            .validate_contract_via_events(
                "performance_test",
                "test.json",
                contract_data,
                true,
                false,
            )
            .await?;
    }

    let event_duration = event_start.elapsed();
    let event_avg = event_duration / iterations;

    println!(
        "   Event-driven: {} iterations in {:?} (avg: {:?})",
        iterations, event_duration, event_avg
    );

    // Performance comparison
    let speedup = event_avg.as_nanos() as f64 / direct_avg.as_nanos() as f64;
    println!("📈 Performance comparison:");
    println!(
        "   Direct linking is {:.1}x faster than event-driven",
        speedup
    );

    if speedup > 10.0 {
        println!("   ✅ Direct linking provides significant performance benefit");
    } else if speedup > 2.0 {
        println!("   ⚠️  Direct linking provides moderate performance benefit");
    } else {
        println!("   ℹ️  Performance difference is minimal");
    }

    Ok(())
}

/// Demo 4: Hybrid Workflow
async fn await_hybrid_workflow_demo(orchestrator: &HooksmithOrchestrator) -> Result<()> {
    println!("Demonstrating hybrid workflow combining multiple patterns...");

    // Create a complex workflow that uses both patterns
    let workflow_id = Uuid::new_v4().to_string();
    println!("🔄 Workflow ID: {}", workflow_id);

    // Step 1: System operation (event-driven) - Read multiple files
    println!("📁 Step 1: Reading multiple files (event-driven)");
    let files = vec!["contract.json", "schema.json", "config.json"];
    let mut file_contents = Vec::new();

    for file in &files {
        let content = orchestrator.read_file_via_events(file).await?;
        file_contents.push((file.to_string(), content));
        println!("   Read {}: {} bytes", file, content.len());
    }

    // Step 2: Pure computation (direct linking) - Validate each file
    println!("🔍 Step 2: Validating files (direct linking)");
    let mut validation_results = Vec::new();

    for (file, content) in &file_contents {
        let start = Instant::now();
        let result = orchestrator.validate_contract_direct(content).await?;
        let duration = start.elapsed();

        validation_results.push((file.clone(), result, duration));
        println!(
            "   Validated {}: {:?} in {:?}",
            file, result.success, duration
        );
    }

    // Step 3: System operation (event-driven) - Store results
    println!("💾 Step 3: Storing validation results (event-driven)");
    for (file, result, duration) in &validation_results {
        orchestrator.store_proof_via_events(file, result).await?;
        println!(
            "   Stored proof for {} (validation took {:?})",
            file, duration
        );
    }

    // Step 4: Generate summary (pure computation)
    println!("📊 Step 4: Generating summary (direct linking)");
    let total_files = validation_results.len();
    let successful_validations = validation_results
        .iter()
        .filter(|(_, result, _)| result.success)
        .count();
    let total_duration: Duration = validation_results
        .iter()
        .map(|(_, _, duration)| *duration)
        .sum();

    println!("   Summary:");
    println!("     Total files: {}", total_files);
    println!("     Successful validations: {}", successful_validations);
    println!("     Total validation time: {:?}", total_duration);
    println!(
        "     Average validation time: {:?}",
        total_duration / total_files as u32
    );

    // Step 5: System operation (event-driven) - Commit to Git
    println!("🔗 Step 5: Committing to Git (event-driven)");
    let commit_event = HooksmithEvent::new(
        "hybrid_workflow".to_string(),
        "git_commit_request".to_string(),
        json!({
            "message": format!("Validation results for workflow {}", workflow_id),
            "files": files,
            "summary": {
                "total_files": total_files,
                "successful_validations": successful_validations,
                "total_duration_ms": total_duration.as_millis()
            }
        }),
    );

    orchestrator.route_event(commit_event).await?;
    println!("   Git commit initiated");

    println!("✅ Hybrid workflow completed successfully!");
    Ok(())
}

/// Demo component linking with proper WIT interfaces
pub async fn demo_component_linking_with_wit() -> Result<()> {
    println!("🔗 Component Linking with WIT Interfaces Demo");
    println!("=============================================\n");

    // This would demonstrate proper WIT interface usage
    // In a real implementation, we would:
    // 1. Define WIT interfaces for component communication
    // 2. Use wit-bindgen to generate Rust bindings
    // 3. Link components with proper type safety

    println!("📝 WIT Interface Example:");
    println!("```wit");
    println!("// wit/validation-handler.wit");
    println!("package hooksmith:validation;");
    println!("");
    println!("interface validation {{");
    println!("  validate-contract: func(");
    println!("    contract-name: string,");
    println!("    content: string,");
    println!("    config: validation-config");
    println!("  ) -> validation-result;");
    println!("}}");
    println!("");
    println!("export validation;");
    println!("```");
    println!("");
    println!("```wit");
    println!("// wit/contract-checker.wit");
    println!("package hooksmith:contract-checker;");
    println!("");
    println!("import hooksmith:validation/validation;");
    println!("");
    println!("interface contract-checker {{");
    println!("  check-contract: func(");
    println!("    contract-data: string,");
    println!("    rules: list<string>");
    println!("  ) -> check-result;");
    println!("}}");
    println!("");
    println!("export contract-checker;");
    println!("```");
    println!("");
    println!("🎯 Benefits of WIT interfaces:");
    println!("   ✅ Type-safe communication");
    println!("   ✅ Zero-copy data transfer");
    println!("   ✅ Nanosecond latency");
    println!("   ✅ Language agnostic");
    println!("   ✅ Canonical ABI compliance");

    Ok(())
}

/// Demo shared memory for performance-critical operations
pub async fn demo_shared_memory() -> Result<()> {
    println!("🧠 Shared Memory Demo (Performance-Critical Operations)");
    println!("=====================================================\n");

    println!("⚠️  Shared Memory Considerations:");
    println!("   • Only available within same process");
    println!("   • Requires careful synchronization");
    println!("   • Manual memory management");
    println!("   • Use sparingly and carefully");
    println!("");
    println!("📝 Shared Memory Example:");
    println!("```rust");
    println!("// Create shared memory for large dataset processing");
    println!("let shared_mem = SharedMemory::new(&engine, MemoryType::shared(1024, 10240))?;");
    println!("");
    println!("// Pass to multiple components");
    println!(
        "let component_a = Component::from_file(&engine, \"data-processor-a.component.wasm\")?;"
    );
    println!("let instance_a = linker.instantiate(&mut store, &component_a)?;");
    println!("instance_a.exports().set_shared_memory(shared_mem.clone())?;");
    println!("");
    println!(
        "let component_b = Component::from_file(&engine, \"data-processor-b.component.wasm\")?;"
    );
    println!("let instance_b = linker.instantiate(&mut store, &component_b)?;");
    println!("instance_b.exports().set_shared_memory(shared_mem)?;");
    println!("```");
    println!("");
    println!("🎯 Use cases for shared memory:");
    println!("   • Large dataset processing");
    println!("   • Real-time data analysis");
    println!("   • High-frequency trading algorithms");
    println!("   • Scientific computing");
    println!("   • Image/video processing");

    Ok(())
}

/// Main demo runner
pub async fn run_all_communication_demos() -> Result<()> {
    println!("🚀 Hooksmith Wasmtime Communication Patterns");
    println!("============================================\n");

    // Run main communication patterns demo
    run_wasmtime_communication_demo().await?;

    println!("\n" . "=".repeat(60) . "\n");

    // Run WIT interface demo
    demo_component_linking_with_wit().await?;

    println!("\n" . "=".repeat(60) . "\n");

    // Run shared memory demo
    demo_shared_memory().await?;

    println!("\n🎉 All communication pattern demos completed!");
    println!("📚 Check the documentation for more details:");
    println!("   • docs/WASMTIME_COMPONENT_COMMUNICATION.md");
    println!("   • docs/HYBRID_ARCHITECTURE_INTEGRATION.md");
    println!("   • docs/HYBRID_ARCHITECTURE_FINAL_SUMMARY.md");

    Ok(())
}
