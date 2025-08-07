use hooksmith::modules::functional_contract_pipeline::{
    FunctionalContractPipeline, HookEvent, ConcernSymbol, ContractSymbol,
    sarif_roles::{SarifFirstPipeline, GitMetadata, AuditPolicy, QueryCriteria, AuditAction, AuditResult},
    create_audit_policy,
};
use serde_sarif::sarif::SarifLog;
use std::collections::HashMap;

/// Example demonstrating SARIF-first architecture with new roles
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 SARIF-First Architecture Demo");
    println!("================================\n");

    // Demonstrate the complete SARIF-first pipeline
    demonstrate_sarif_first_pipeline()?;

    // Demonstrate individual roles
    demonstrate_individual_roles()?;

    // Demonstrate audit policies
    demonstrate_audit_policies()?;

    // Demonstrate SARIF querying
    demonstrate_sarif_querying()?;

    // Demonstrate provenance tracking
    demonstrate_provenance_tracking()?;

    println!("✅ SARIF-first architecture demo completed successfully!");
    Ok(())
}

/// Demonstrate the complete SARIF-first pipeline
fn demonstrate_sarif_first_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Complete SARIF-First Pipeline");
    println!("---------------------------------\n");

    let mut pipeline = SarifFirstPipeline::new();
    
    // Create Git metadata for provenance
    let git_metadata = GitMetadata::new(
        "abc123def456".to_string(),
        "tree789".to_string(),
        HookEvent::PreCommit,
    );

    println!("📋 Running SARIF-first pipeline for PreCommit hook...");
    
    let (sarif_log, audit_result) = pipeline.run_pipeline(HookEvent::PreCommit, git_metadata);

    println!("  ✅ SARIF log generated with {} runs", sarif_log.runs.len());
    
    for (i, run) in sarif_log.runs.iter().enumerate() {
        println!("    Run {}: {} results", i + 1, run.results.len());
        
        for (j, result) in run.results.iter().enumerate() {
            println!("      Result {}: {} ({})", j + 1, result.rule_id, result.level);
        }
    }

    println!("  ✅ Audit result: {}", if audit_result.is_pass() { "PASS" } else { "FAIL" });
    
    if !audit_result.violations().is_empty() {
        println!("    Violations: {}", audit_result.violations().len());
    }
    
    if !audit_result.warnings().is_empty() {
        println!("    Warnings: {}", audit_result.warnings().len());
    }

    println!();
    Ok(())
}

/// Demonstrate individual roles
fn demonstrate_individual_roles() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Individual Roles Demonstration");
    println!("--------------------------------\n");

    // 1. Hook role
    println!("1️⃣ Hook Role");
    let hook = crate::modules::functional_contract_pipeline::sarif_roles::roles::Hook::new(HookEvent::PreCommit);
    let concerns = hook.collect_concerns();
    println!("   Collected {} concerns: {:?}", concerns.len(), concerns);
    println!();

    // 2. Concern role
    println!("2️⃣ Concern Role");
    let concern = crate::modules::functional_contract_pipeline::sarif_roles::roles::Concern::new(ConcernSymbol::Index);
    let snapshot = concern.take_snapshot();
    println!("   Snapshot taken for concern: {}", snapshot.symbol.name());
    println!();

    // 3. Specifier role
    println!("3️⃣ Specifier Role");
    let specifier = crate::modules::functional_contract_pipeline::sarif_roles::roles::Specifier::new();
    let contracts = vec![ContractSymbol::new("must-exist")];
    let expectations = specifier.generate_expectations(&contracts);
    println!("   Generated {} expectations from {} contracts", expectations.len(), contracts.len());
    println!();

    // 4. Verifier role
    println!("4️⃣ Verifier Role");
    let verifier = crate::modules::functional_contract_pipeline::sarif_roles::roles::Verifier::new(
        crate::modules::functional_contract_pipeline::high_performance_diff::DiffStrategy::JsonPatch
    );
    let snapshots = vec![snapshot];
    let sarif_log = verifier.verify_and_emit_sarif(&snapshots, &expectations, &HookEvent::PreCommit);
    println!("   Generated SARIF log with {} runs", sarif_log.runs.len());
    println!();

    // 5. Stegrapher role
    println!("5️⃣ Stegrapher Role");
    let mut stegrapher = crate::modules::functional_contract_pipeline::sarif_roles::roles::Stegrapher::new();
    let git_metadata = GitMetadata::new(
        "abc123".to_string(),
        "def456".to_string(),
        HookEvent::PreCommit,
    );
    let indexed_sarif = stegrapher.index_and_tag_sarif(&sarif_log, &git_metadata);
    println!("   Indexed SARIF with {} entries", stegrapher.indexed_entries.len());
    println!();

    // 6. Auditor role
    println!("6️⃣ Auditor Role");
    let auditor = crate::modules::functional_contract_pipeline::sarif_roles::roles::Auditor::new();
    let audit_result = auditor.audit_sarif(&indexed_sarif);
    println!("   Audit result: {}", if audit_result.is_pass() { "PASS" } else { "FAIL" });
    println!();

    Ok(())
}

/// Demonstrate audit policies
fn demonstrate_audit_policies() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Audit Policies Demonstration");
    println!("------------------------------\n");

    let mut auditor = crate::modules::functional_contract_pipeline::sarif_roles::roles::Auditor::new();

    // Policy 1: No executable files
    let policy1 = create_audit_policy(
        "no-executable-files".to_string(),
        "Prevent executable files from being committed".to_string(),
        Some("TreeExecutable".to_string()),
        Some(crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Error),
        Some("PreCommit".to_string()),
        AuditAction::Fail,
    );
    auditor.add_policy(policy1);

    // Policy 2: Line ending normalization
    let policy2 = create_audit_policy(
        "line-ending-normalization".to_string(),
        "Ensure line endings are normalized".to_string(),
        Some("AttrLineEndingNormalization".to_string()),
        Some(crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Warning),
        Some("PreCommit".to_string()),
        AuditAction::Warn,
    );
    auditor.add_policy(policy2);

    // Policy 3: Index validation
    let policy3 = create_audit_policy(
        "index-validation".to_string(),
        "Ensure index is in valid state".to_string(),
        Some("Index".to_string()),
        Some(crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Error),
        Some("PreCommit".to_string()),
        AuditAction::Fail,
    );
    auditor.add_policy(policy3);

    println!("✅ Added {} audit policies:", auditor.policies.len());
    for policy in &auditor.policies {
        println!("   • {}: {}", policy.name, policy.description);
    }
    println!();

    Ok(())
}

/// Demonstrate SARIF querying
fn demonstrate_sarif_querying() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 SARIF Querying Demonstration");
    println!("------------------------------\n");

    // Create a sample SARIF log
    let sarif_log = create_sample_sarif_log();

    let auditor = crate::modules::functional_contract_pipeline::sarif_roles::roles::Auditor::new();

    // Query 1: All Index concerns
    let criteria1 = QueryCriteria::new().with_concern("Index".to_string());
    let results1 = auditor.query_sarif(&sarif_log, &criteria1);
    println!("📋 Query: Index concerns");
    println!("   Found {} results", results1.len());
    println!();

    // Query 2: All Error severity
    let criteria2 = QueryCriteria::new().with_severity(crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Error);
    let results2 = auditor.query_sarif(&sarif_log, &criteria2);
    println!("📋 Query: Error severity");
    println!("   Found {} results", results2.len());
    println!();

    // Query 3: PreCommit hook events
    let criteria3 = QueryCriteria::new().with_hook_event("PreCommit".to_string());
    let results3 = auditor.query_sarif(&sarif_log, &criteria3);
    println!("📋 Query: PreCommit hook events");
    println!("   Found {} results", results3.len());
    println!();

    // Query 4: Combined criteria
    let criteria4 = QueryCriteria::new()
        .with_concern("TreeExecutable".to_string())
        .with_severity(crate::modules::functional_contract_pipeline::symbols::RuleSeverity::Error)
        .with_hook_event("PreCommit".to_string());
    let results4 = auditor.query_sarif(&sarif_log, &criteria4);
    println!("📋 Query: TreeExecutable + Error + PreCommit");
    println!("   Found {} results", results4.len());
    println!();

    Ok(())
}

/// Demonstrate provenance tracking
fn demonstrate_provenance_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Provenance Tracking Demonstration");
    println!("-----------------------------------\n");

    let mut stegrapher = crate::modules::functional_contract_pipeline::sarif_roles::roles::Stegrapher::new();
    
    // Create sample SARIF log
    let sarif_log = create_sample_sarif_log();
    
    // Create Git metadata with provenance
    let git_metadata = GitMetadata::new(
        "abc123def456".to_string(),
        "tree789".to_string(),
        HookEvent::PreCommit,
    );

    println!("📋 Indexing SARIF with provenance...");
    let indexed_sarif = stegrapher.index_and_tag_sarif(&sarif_log, &git_metadata);

    println!("✅ Indexed {} entries with provenance", stegrapher.indexed_entries.len());
    println!("✅ Created {} provenance mappings", stegrapher.provenance_map.len());

    // Show provenance for first entry
    if let Some((entry_id, _)) = stegrapher.indexed_entries.iter().next() {
        if let Some(provenance) = stegrapher.provenance_map.get(entry_id) {
            println!("\n📋 Provenance for entry {}:", entry_id);
            for (key, value) in provenance {
                println!("   {}: {}", key, value);
            }
        }
    }

    println!();
    Ok(())
}

/// Create a sample SARIF log for demonstration
fn create_sample_sarif_log() -> SarifLog {
    use serde_sarif::sarif::{ArtifactLocation, Location, Message, PhysicalLocation, Result as SarifResult, Run, Tool, ToolComponent};
    use std::collections::HashMap;

    let results = vec![
        SarifResult::builder()
            .rule_id("Index-missing".to_string())
            .level("error")
            .message(Message::builder()
                .text("Index does not exist".to_string())
                .build())
            .locations(vec![Location::builder()
                .physical_location(PhysicalLocation::builder()
                    .artifact_location(ArtifactLocation::builder()
                        .uri("git://concern/Index".to_string())
                        .build())
                    .build())
                .build()])
            .properties(HashMap::from([
                ("concern".to_string(), serde_json::Value::String("Index".to_string())),
                ("diff_type".to_string(), serde_json::Value::String("Missing".to_string())),
                ("origin".to_string(), serde_json::Value::String("hook/PreCommit".to_string())),
                ("severity".to_string(), serde_json::Value::String("Error".to_string())),
            ]))
            .build(),
        SarifResult::builder()
            .rule_id("TreeExecutable-mismatch".to_string())
            .level("warning")
            .message(Message::builder()
                .text("File mode changed from 100644 to 100755".to_string())
                .build())
            .locations(vec![Location::builder()
                .physical_location(PhysicalLocation::builder()
                    .artifact_location(ArtifactLocation::builder()
                        .uri("git://concern/TreeExecutable".to_string())
                        .build())
                    .build())
                .build()])
            .properties(HashMap::from([
                ("concern".to_string(), serde_json::Value::String("TreeExecutable".to_string())),
                ("diff_type".to_string(), serde_json::Value::String("Mismatch".to_string())),
                ("origin".to_string(), serde_json::Value::String("hook/PreCommit".to_string())),
                ("severity".to_string(), serde_json::Value::String("Warning".to_string())),
            ]))
            .build(),
    ];

    let tool = Tool::builder()
        .driver(ToolComponent::builder()
            .name("Hooksmith Contract Validator")
            .version("1.0.0")
            .build())
        .build();

    let run = Run::builder()
        .tool(tool)
        .results(results)
        .build();

    SarifLog::builder()
        .version("2.1.0")
        .runs(vec![run])
        .build()
}

/// Demonstrate slot-based schema compliance
fn demonstrate_slot_based_schema() {
    println!("📊 Slot-Based Schema Compliance");
    println!("-------------------------------\n");

    let schema = vec![
        ("Hook", "Trigger or Git action", "Concern[]", "No", "✅", "❌"),
        ("Concern", "Git object", "Snapshot", "No", "✅", "❌"),
        ("Specifier", "Concern → Contract", "Contract", "No", "✅", "❌"),
        ("Verifier", "Snapshot + Contract", "SARIF log (diff)", "No", "✅", "✅"),
        ("Stegrapher", "SARIF log + metadata", "Indexed SARIF entries", "No", "✅", "✅"),
        ("Auditor", "Query on SARIF log", "Pass / Fail", "✅", "✅", "✅ (optional)"),
    ];

    println!("Role\t\tInput\t\t\tOutput\t\t\tFails?\tStateless\tEmits SARIF");
    println!("----\t\t-----\t\t\t------\t\t\t------\t---------\t-----------");
    
    for (role, input, output, fails, stateless, sarif) in schema {
        println!("{}\t{}\t{}\t{}\t{}\t{}", role, input, output, fails, stateless, sarif);
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sarif_first_pipeline() {
        let result = demonstrate_sarif_first_pipeline();
        assert!(result.is_ok());
    }

    #[test]
    fn test_individual_roles() {
        let result = demonstrate_individual_roles();
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_policies() {
        let result = demonstrate_audit_policies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_sarif_querying() {
        let result = demonstrate_sarif_querying();
        assert!(result.is_ok());
    }

    #[test]
    fn test_provenance_tracking() {
        let result = demonstrate_provenance_tracking();
        assert!(result.is_ok());
    }

    #[test]
    fn test_sample_sarif_log() {
        let sarif_log = create_sample_sarif_log();
        assert!(!sarif_log.runs.is_empty());
        assert!(!sarif_log.runs[0].results.is_empty());
    }
}
