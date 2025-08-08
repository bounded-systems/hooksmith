//! Repair Planning System Demo
//! 
//! This example demonstrates the complete repair planning pipeline:
//! 
//! 1. Detect a formatting violation
//! 2. Investigate the root cause
//! 3. Dispatch to appropriate fixers
//! 4. Create a repair plan
//! 5. Execute the repairs

use hooksmith::modules::functional_contract_pipeline::repair_planning::{
    TriageOfficer, Violation, ConcernSnapshot, RepairPlan, FixCategory, RuleSeverity
};
use hooksmith::modules::functional_contract_pipeline::symbols::ConcernSymbol;
use std::collections::HashMap;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🔧 Repair Planning System Demo");
    println!("==============================\n");

    // Step 1: Create a violation (simulating what the Auditor would detect)
    let violation = create_formatting_violation();
    println!("📋 Detected Violation:");
    println!("   Concern: {:?}", violation.concern);
    println!("   Contract: {}", violation.contract);
    println!("   Message: {}", violation.message);
    println!("   Location: {:?}", violation.location);
    println!("   Severity: {:?}", violation.severity);
    println!();

    // Step 2: Create a snapshot of the concern
    let snapshot = create_concern_snapshot();
    println!("📸 Concern Snapshot:");
    println!("   Hash: {}", snapshot.hash);
    println!("   Data: {}", snapshot.data);
    println!();

    // Step 3: Create Triage Officer and generate repair plan
    let mut triage_officer = TriageOfficer::new();
    let plan = triage_officer.create_plan(&violation, &snapshot)?;
    
    println!("🎯 Repair Plan Generated:");
    println!("   Plan ID: {}", plan.id);
    println!("   Dispatcher: {}", plan.dispatcher);
    println!("   Actions: {}", plan.actions.len());
    println!();

    // Step 4: Display the root cause analysis
    println!("🔍 Root Cause Analysis:");
    println!("   Primary Cause: {}", plan.root_cause.primary_cause);
    println!("   Confidence: {:.2}", plan.root_cause.confidence);
    println!("   Fix Categories: {:?}", plan.root_cause.fix_categories);
    println!("   Contributing Factors:");
    for factor in &plan.root_cause.contributing_factors {
        println!("     - {}", factor);
    }
    println!();

    // Step 5: Display the repair actions
    println!("🛠️  Repair Actions:");
    for (i, action) in plan.actions.iter().enumerate() {
        println!("   {}. {} ({})", i + 1, action.fixer_id, action.action_type.name());
        println!("      Target: {:?}", action.target_path);
        println!("      Required: {}", action.required);
        println!("      Priority: {}", action.priority);
        if !action.dependencies.is_empty() {
            println!("      Dependencies: {:?}", action.dependencies);
        }
        println!();
    }

    // Step 6: Simulate executing the repairs
    println!("⚡ Executing Repairs:");
    for action in &plan.actions {
        println!("   Running: {} on {:?}", action.fixer_id, action.target_path);
        
        // In a real implementation, this would execute the actual fixer
        match action.fixer_id.as_str() {
            "fixer.rustfmt" => {
                println!("     ✅ Applied rustfmt formatting");
            }
            "fixer.dprint" => {
                println!("     ✅ Applied dprint formatting");
            }
            "fixer.trunk" => {
                println!("     ✅ Applied trunk fixes");
            }
            _ => {
                println!("     ✅ Applied {} fix", action.fixer_id);
            }
        }
    }
    println!();

    // Step 7: Show the final result
    println!("🎉 Repair Complete!");
    println!("   All {} actions executed successfully", plan.actions.len());
    println!("   Concern should now pass validation");
    println!();

    // Step 8: Demonstrate caching
    println!("💾 Caching Demo:");
    let cached_plan = triage_officer.create_plan(&violation, &snapshot)?;
    println!("   Retrieved cached plan: {}", cached_plan.id);
    println!("   Cache hit: {}", cached_plan.id == plan.id);
    println!();

    Ok(())
}

/// Create a sample formatting violation
fn create_formatting_violation() -> Violation {
    Violation {
        concern: ConcernSymbol::TreeFile,
        contract: "format".to_string(),
        message: "Rust formatting violation: inconsistent indentation and missing newline at end of file".to_string(),
        location: Some("src/main.rs".to_string()),
        severity: RuleSeverity::Error,
        details: {
            let mut details = HashMap::new();
            details.insert("line".to_string(), serde_json::json!(5));
            details.insert("column".to_string(), serde_json::json!(12));
            details.insert("expected".to_string(), serde_json::json!("4 spaces"));
            details.insert("found".to_string(), serde_json::json!("2 spaces"));
            details
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

/// Create a sample concern snapshot
fn create_concern_snapshot() -> ConcernSnapshot {
    let content = r#"fn main() {
  println!("Hello, world!");
}"#;
    
    ConcernSnapshot::new(
        ConcernSymbol::TreeFile,
        serde_json::json!({
            "content": content,
            "path": "src/main.rs",
            "size": content.len(),
            "language": "rust"
        }),
        {
            let mut metadata = HashMap::new();
            metadata.insert("file_type".to_string(), serde_json::json!("rust"));
            metadata.insert("has_main".to_string(), serde_json::json!(true));
            metadata
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let violation = create_formatting_violation();
        assert_eq!(violation.concern, ConcernSymbol::TreeFile);
        assert_eq!(violation.contract, "format");
        assert!(violation.message.contains("formatting violation"));
    }

    #[test]
    fn test_snapshot_creation() {
        let snapshot = create_concern_snapshot();
        assert_eq!(snapshot.symbol, ConcernSymbol::TreeFile);
        assert!(snapshot.data.get("content").is_some());
        assert!(snapshot.data.get("path").is_some());
    }

    #[test]
    fn test_repair_plan_creation() {
        let violation = create_formatting_violation();
        let snapshot = create_concern_snapshot();
        let mut triage_officer = TriageOfficer::new();
        
        let plan = triage_officer.create_plan(&violation, &snapshot).unwrap();
        
        assert_eq!(plan.concern, ConcernSymbol::TreeFile);
        assert_eq!(plan.contract, "format");
        assert!(!plan.actions.is_empty());
        assert!(plan.root_cause.confidence > 0.0);
    }
}
