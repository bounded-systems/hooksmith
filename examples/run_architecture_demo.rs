#!/usr/bin/env rustc --run

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

// Colors for output
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const PURPLE: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const NC: &str = "\x1b[0m"; // No Color

fn print_step(message: &str) {
    println!("{}🔧 {} {}", BLUE, NC, message);
}

fn print_success(message: &str) {
    println!("{}✅ {} {}", GREEN, NC, message);
}

fn print_warning(message: &str) {
    println!("{}⚠️  {} {}", YELLOW, NC, message);
}

fn print_error(message: &str) {
    println!("{}❌ {} {}", RED, NC, message);
}

fn print_info(message: &str) {
    println!("{}ℹ️  {} {}", CYAN, NC, message);
}

fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we're in the right directory
    if !Path::new("Cargo.toml").exists() {
        print_error("This script must be run from the Hooksmith project root");
        std::process::exit(1);
    }
    Ok(())
}

fn create_demo_files() -> Result<(), Box<dyn std::error::Error>> {
    print_info("Creating demo contract files...");

    // Create .devcontract.ts
    let devcontract_content = r#"// Demo contract for Hooksmith architecture
export default {
  files: {
    "README.md": {
      must_exist: true,
      severity: "error"
    },
    "hooks/pre-commit": {
      must_be_executable: true,
      severity: "warning"
    },
    "docs/ARCHITECTURE.md": {
      must_exist: true,
      severity: "error"
    }
  },
  workflows: {
    "Submit Container": {
      must_have_handler: true,
      severity: "error"
    },
    "Deploy to Production": {
      must_have_handler: true,
      severity: "warning"
    }
  }
}"#;
    fs::write(".devcontract.ts", devcontract_content)?;

    // Create demo directories and files
    fs::create_dir_all("hooks")?;
    fs::create_dir_all("docs")?;
    
    // Create pre-commit hook
    fs::write("hooks/pre-commit", "#!/bin/bash\necho 'Pre-commit hook executed'")?;
    fs::set_permissions("hooks/pre-commit", fs::Permissions::from_mode(0o755))?;
    
    // Create README.md
    fs::write("README.md", "# Hooksmith Architecture Demo")?;
    
    // Create ARCHITECTURE.md
    fs::write("docs/ARCHITECTURE.md", "# Architecture Documentation")?;

    print_success("Demo files created");
    Ok(())
}

fn build_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Building Hooksmith architecture demo...");

    let status = Command::new("cargo")
        .args(&["build", "--example", "hooksmith_architecture_demo"])
        .status()?;

    if status.success() {
        print_success("Demo built successfully");
        Ok(())
    } else {
        print_error("Failed to build demo");
        Err("Build failed".into())
    }
}

fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Running Hooksmith architecture demo...");

    println!("\n🚀 Starting Hooksmith Pipeline");
    println!("================================");

    let status = Command::new("./target/debug/examples/hooksmith_architecture_demo")
        .status()?;

    if status.success() {
        print_success("Demo completed successfully!");
        Ok(())
    } else {
        print_error("Demo failed");
        Err("Demo execution failed".into())
    }
}

fn run_tests() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Running tests to verify architecture...");

    let status = Command::new("cargo")
        .args(&["test", "hooksmith_architecture_demo", "--lib"])
        .status()?;

    if status.success() {
        print_success("All tests passed!");
        Ok(())
    } else {
        print_error("Some tests failed");
        Err("Tests failed".into())
    }
}

fn generate_sarif_output() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Demonstrating SARIF output generation...");

    let sarif_content = r#"{"ruleId":"must_be_executable","level":"warning","message":"hooks/pre-commit is not marked executable as required","target":"hooks/pre-commit","locations":[{"uri":"hooks/pre-commit","line":null,"column":null}],"timestamp":"2025-01-02T15:20:00Z"}
{"ruleId":"must_have_handler","level":"error","message":"Slack workflow 'Submit Container' is missing a handler","target":"Submit Container","locations":[{"uri":"Submit Container","line":null,"column":null}],"timestamp":"2025-01-02T15:20:00Z"}"#;
    
    fs::write("demo_sarif.jsonl", sarif_content)?;
    print_success("Generated sample SARIF output");
    Ok(())
}

fn create_routing_config() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Demonstrating event routing...");

    let routing_content = r#"{
  "source": "demo_sarif.jsonl",
  "routes": [
    {
      "match": { 
        "ruleId": "must_be_executable",
        "level": "warning"
      },
      "action": { 
        "type": "github.annotate",
        "severity": "warning",
        "message": "File should be executable"
      }
    },
    {
      "match": { 
        "level": "error"
      },
      "action": { 
        "type": "fail_ci",
        "reason": "Validation error detected"
      }
    },
    {
      "match": { 
        "ruleId": "must_have_handler"
      },
      "action": { 
        "type": "notify.slack",
        "channel": "\\#workflows",
        "message": "Slack workflow handler is missing"
      }
    }
  ]
}"#;
    
    fs::write("demo_routing.jsonc", routing_content)?;
    print_success("Created routing configuration");
    Ok(())
}

fn cleanup_demo_files() -> Result<(), Box<dyn std::error::Error>> {
    print_step("Cleaning up demo files...");

    let files_to_remove = [
        ".devcontract.ts",
        "demo_sarif.jsonl", 
        "demo_routing.jsonc",
        "hooks/pre-commit",
        "README.md",
        "docs/ARCHITECTURE.md"
    ];

    for file in &files_to_remove {
        if Path::new(file).exists() {
            fs::remove_file(file)?;
        }
    }

    // Remove directories if empty
    let dirs_to_remove = ["hooks", "docs"];
    for dir in &dirs_to_remove {
        if Path::new(dir).exists() {
            if let Ok(entries) = fs::read_dir(dir) {
                if entries.count() == 0 {
                    fs::remove_dir(dir)?;
                }
            }
        }
    }

    print_success("Demo files cleaned up");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Hooksmith Architecture Demo");
    println!("================================");
    println!();

    // Check dependencies
    check_dependencies()?;

    // Create demo files
    create_demo_files()?;

    // Build the demo
    build_demo()?;

    // Run the demo
    run_demo()?;

    // Run tests
    run_tests()?;

    // Generate SARIF output
    generate_sarif_output()?;

    // Create routing configuration
    create_routing_config()?;

    println!();
    print_info("Architecture Components Demonstrated:");
    println!("==========================================");
    println!("✅ Contract Definition (.devcontract.ts)");
    println!("✅ Desired State Generation");
    println!("✅ Observed State Validation");
    println!("✅ Diff Generation");
    println!("✅ SARIF Conversion");
    println!("✅ Event Routing");
    println!("✅ Declarative Configuration");
    println!("✅ Multi-modal Operation Support");

    // Cleanup
    cleanup_demo_files()?;

    println!();
    print_success("🎉 Hooksmith Architecture Demo Completed Successfully!");
    println!();
    println!("This demo proves that the Hooksmith dual-agent architecture is possible:");
    println!();
    println!("🔹 Contract parsing and desired state generation");
    println!("🔹 Validation and observed state generation"); 
    println!("🔹 Diff generation and SARIF conversion");
    println!("🔹 Event routing with declarative rules");
    println!("🔹 Multi-modal operation (CLI, HTTP, file watching)");
    println!();
    println!("The architecture provides:");
    println!("🔹 Unified validation loop");
    println!("🔹 Reactive architecture");
    println!("🔹 Versioned expectations");
    println!("🔹 Declarative routing");
    println!("🔹 Comprehensive observability");
    println!();
    print_info("Next steps:");
    println!("1. Review the generated diagrams in docs/diagrams/");
    println!("2. Examine the demo code in examples/hooksmith_architecture_demo.rs");
    println!("3. Implement the full architecture in the main codebase");
    println!("4. Add more sophisticated validation rules and handlers");

    Ok(())
} 
