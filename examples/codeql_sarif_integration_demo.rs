use anyhow::Result;
use std::process::Command;

/// Example demonstrating SARIF and CodeQL integration with Hooksmith
///
/// This example shows how to:
/// 1. Run CodeQL analysis on a Rust project
/// 2. Convert results to structured JSONL events
/// 3. Integrate with existing validation pipeline
/// 4. Merge multiple analysis results
/// 5. Validate and convert between formats
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Hooksmith SARIF/CodeQL Integration Demo");
    println!("==========================================\n");

    // Check if CodeQL CLI is available
    let codeql_available = Command::new("codeql").arg("version").output().is_ok();

    if !codeql_available {
        println!("⚠️  CodeQL CLI not found. Install with: brew install codeql");
        println!(
            "   This demo will show the integration structure without running actual analysis.\n"
        );
    }

    // Example 1: Basic SARIF operations
    println!("📋 Example 1: Basic SARIF Operations");
    println!("-----------------------------------");

    // Create a sample JSONL file with validation events
    create_sample_jsonl()?;

    // Convert JSONL to SARIF
    run_xtask_command(&[
        "sarif",
        "jsonl-to-sarif",
        "--input",
        "sample-events.jsonl",
        "--output",
        "sample-events.sarif",
        "--validate",
    ])
    .await?;

    // Convert SARIF back to JSONL
    run_xtask_command(&[
        "sarif",
        "sarif-to-jsonl",
        "--input",
        "sample-events.sarif",
        "--output",
        "converted-events.jsonl",
        "--validate",
    ])
    .await?;

    // Example 2: CodeQL Analysis (if available)
    if codeql_available {
        println!("\n📋 Example 2: CodeQL Analysis");
        println!("----------------------------");

        // Run CodeQL analysis on the current project
        run_xtask_command(&[
            "sarif",
            "codeql-analysis",
            "--db-dir",
            "codeql-db",
            "--query-suite",
            "codeql-cpp-queries:Security-and-quality.qls",
            "--language",
            "cpp",
            "--build-command",
            "cargo build",
            "--output",
            "codeql-results.sarif",
            "--to-jsonl",
        ])
        .await?;
    }

    // Example 3: Validation Pipeline Integration
    println!("\n📋 Example 3: Validation Pipeline Integration");
    println!("-------------------------------------------");

    // Run the full integration pipeline
    run_xtask_command(&[
        "sarif",
        "integrate",
        "--run-analysis",
        "--to-jsonl",
        "--merge",
        "--output-dir",
        "validation-results",
    ])
    .await?;

    // Example 4: SARIF Validation and Merging
    println!("\n📋 Example 4: SARIF Validation and Merging");
    println!("----------------------------------------");

    // Create multiple sample SARIF files
    create_sample_sarif_files()?;

    // Validate SARIF files
    run_xtask_command(&["sarif", "validate", "--file", "sample-1.sarif", "--strict"]).await?;

    run_xtask_command(&["sarif", "validate", "--file", "sample-2.sarif", "--strict"]).await?;

    // Merge SARIF files
    run_xtask_command(&[
        "sarif",
        "merge",
        "--inputs",
        "sample-1.sarif",
        "sample-2.sarif",
        "--output",
        "merged-results.sarif",
        "--validate",
    ])
    .await?;

    // Example 5: Integration with Auto-Push Pipeline
    println!("\n📋 Example 5: Integration with Auto-Push Pipeline");
    println!("-----------------------------------------------");

    // Show how to integrate with the existing structured logging
    demonstrate_auto_push_integration().await?;

    println!("\n✅ Demo completed successfully!");
    println!("📁 Generated files:");
    println!("   - sample-events.jsonl");
    println!("   - sample-events.sarif");
    println!("   - converted-events.jsonl");
    println!("   - sample-1.sarif");
    println!("   - sample-2.sarif");
    println!("   - merged-results.sarif");
    println!("   - validation-results/ (directory)");

    Ok(())
}

/// Create a sample JSONL file with validation events
fn create_sample_jsonl() -> Result<()> {
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;

    let events = vec![
        json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "error",
            "action": "validation",
            "message": "Potential buffer overflow in unsafe block",
            "details": {
                "file": "src/unsafe_code.rs",
                "line": 42,
                "column": 15,
                "rule_id": "cpp/security/buffer-overflow",
                "tool": "codeql"
            }
        }),
        json!({
            "timestamp": "2024-01-15T10:30:01Z",
            "level": "warning",
            "action": "validation",
            "message": "Missing error handling for Result",
            "details": {
                "file": "src/error_handling.rs",
                "line": 67,
                "column": 8,
                "rule_id": "rust/missing-error-handling",
                "tool": "clippy"
            }
        }),
        json!({
            "timestamp": "2024-01-15T10:30:02Z",
            "level": "info",
            "action": "validation",
            "message": "CodeQL analysis completed",
            "details": {
                "total_results": 15,
                "errors": 3,
                "warnings": 12,
                "tool": "codeql"
            }
        }),
    ];

    let events_count = events.len();
    let mut file = File::create("sample-events.jsonl")?;
    for event in events {
        writeln!(file, "{event}")?;
    }

    println!("✅ Created sample JSONL file with {events_count} events");
    Ok(())
}

/// Create sample SARIF files for testing
fn create_sample_sarif_files() -> Result<()> {
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;

    let sarif_1 = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "CodeQL",
                    "version": "2.15.0"
                }
            },
            "results": [{
                "ruleId": "cpp/security/buffer-overflow",
                "level": "error",
                "message": {
                    "text": "Potential buffer overflow in unsafe block"
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "src/unsafe_code.rs"
                        },
                        "region": {
                            "startLine": 42,
                            "startColumn": 15
                        }
                    }
                }]
            }]
        }]
    });

    let sarif_2 = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Clippy",
                    "version": "1.75.0"
                }
            },
            "results": [{
                "ruleId": "rust/missing-error-handling",
                "level": "warning",
                "message": {
                    "text": "Missing error handling for Result"
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "src/error_handling.rs"
                        },
                        "region": {
                            "startLine": 67,
                            "startColumn": 8
                        }
                    }
                }]
            }]
        }]
    });

    let mut file1 = File::create("sample-1.sarif")?;
    writeln!(file1, "{}", serde_json::to_string_pretty(&sarif_1)?)?;

    let mut file2 = File::create("sample-2.sarif")?;
    writeln!(file2, "{}", serde_json::to_string_pretty(&sarif_2)?)?;

    println!("✅ Created sample SARIF files");
    Ok(())
}

/// Run an xtask command
async fn run_xtask_command(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "xtask"]);
    cmd.args(args);

    println!("   Running: cargo run --bin xtask {}", args.join(" "));

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("   {}", stdout.trim());
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("   Error: {}", stderr.trim());
    }

    Ok(())
}

/// Demonstrate integration with auto-push pipeline
async fn demonstrate_auto_push_integration() -> Result<()> {
    println!("   🔄 Running validation pipeline with CodeQL integration...");

    // This would typically be part of your auto-push workflow
    // The structured events from CodeQL would be integrated into
    // the existing JSONL pipeline alongside Clippy, cargo check, etc.

    println!("   📊 Validation pipeline would:");
    println!("      1. Run cargo clippy --message-format=json");
    println!("      2. Run cargo check --message-format=json");
    println!("      3. Run CodeQL analysis (if configured)");
    println!("      4. Convert all results to unified JSONL format");
    println!("      5. Validate against schemas");
    println!("      6. Commit and push if validation passes");

    // Show how the events would look in the pipeline
    use serde_json::json;
    let pipeline_event = json!({
        "timestamp": "2024-01-15T10:35:00Z",
        "level": "info",
        "action": "validation_pipeline",
        "message": "Validation pipeline completed with CodeQL integration",
        "details": {
            "tools": ["clippy", "cargo_check", "codeql"],
            "total_results": 25,
            "errors": 2,
            "warnings": 23,
            "pipeline_status": "success"
        }
    });

    println!("   📝 Example pipeline event:");
    println!("      {}", serde_json::to_string_pretty(&pipeline_event)?);

    Ok(())
}
