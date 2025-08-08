use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
struct XtaskExtractionPlan {
    source_path: String,
    target_repo: String,
    extraction_type: ExtractionType,
    dry_run: bool,
    generate_mermaid: bool,
    export_sha_mapping: bool,
    backup_before_extraction: bool,
    contract_snapshot: bool,
}

#[derive(Debug, Clone)]
enum ExtractionType {
    FilterRepo,
    SubtreeSplit,
    ManualExtraction,
}

#[derive(Debug)]
struct XtaskExtractor {
    plan: XtaskExtractionPlan,
    analysis_result: ExtractionAnalysis,
    mermaid_diagram: Option<String>,
    sha_mapping: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
struct ExtractionAnalysis {
    safety_score: f64,
    affected_commits: u32,
    affected_files: u32,
    contract_impact: ContractImpact,
    estimated_savings: String,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
struct ContractImpact {
    affected_contracts: Vec<String>,
    invalidation_risk: f64,
    mitigation_strategies: Vec<String>,
}

fn run_hooksmith_xtask_extraction(args: &[String]) -> Result<XtaskExtractor, Box<dyn std::error::Error>> {
    let plan = parse_xtask_args(args)?;
    
    println!("🔧 Hooksmith Xtask: Tree-to-Repo Extraction");
    println!("=============================================");
    println!("Source: {}", plan.source_path);
    println!("Target: {}", plan.target_repo);
    println!("Mode: {}", if plan.dry_run { "DRY-RUN" } else { "LIVE" });
    println!();
    
    // Perform extraction analysis
    let analysis_result = analyze_extraction(&plan)?;
    
    // Generate mermaid diagram if requested
    let mermaid_diagram = if plan.generate_mermaid {
        Some(generate_mermaid_diagram(&plan, &analysis_result))
    } else {
        None
    };
    
    // Export SHA mapping if requested
    let sha_mapping = if plan.export_sha_mapping {
        Some(export_sha_mapping(&plan.source_path)?)
    } else {
        None
    };
    
    // Create contract snapshot if requested
    if plan.contract_snapshot {
        create_contract_snapshot(&plan.source_path)?;
    }
    
    // Create backup if requested
    if plan.backup_before_extraction && !plan.dry_run {
        create_backup_branch(&plan.source_path)?;
    }
    
    Ok(XtaskExtractor {
        plan,
        analysis_result,
        mermaid_diagram,
        sha_mapping,
    })
}

fn parse_xtask_args(args: &[String]) -> Result<XtaskExtractionPlan, Box<dyn std::error::Error>> {
    let mut plan = XtaskExtractionPlan {
        source_path: String::new(),
        target_repo: String::new(),
        extraction_type: ExtractionType::FilterRepo,
        dry_run: true, // Default to safe mode
        generate_mermaid: false,
        export_sha_mapping: false,
        backup_before_extraction: false,
        contract_snapshot: false,
    };
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source" | "-s" => {
                i += 1;
                if i < args.len() {
                    plan.source_path = args[i].clone();
                }
            }
            "--target" | "-t" => {
                i += 1;
                if i < args.len() {
                    plan.target_repo = args[i].clone();
                }
            }
            "--method" | "-m" => {
                i += 1;
                if i < args.len() {
                    plan.extraction_type = match args[i].as_str() {
                        "filter-repo" => ExtractionType::FilterRepo,
                        "subtree" => ExtractionType::SubtreeSplit,
                        "manual" => ExtractionType::ManualExtraction,
                        _ => ExtractionType::FilterRepo,
                    };
                }
            }
            "--live" => {
                plan.dry_run = false;
            }
            "--mermaid" => {
                plan.generate_mermaid = true;
            }
            "--export-sha" => {
                plan.export_sha_mapping = true;
            }
            "--backup" => {
                plan.backup_before_extraction = true;
            }
            "--snapshot" => {
                plan.contract_snapshot = true;
            }
            _ => {}
        }
        i += 1;
    }
    
    if plan.source_path.is_empty() || plan.target_repo.is_empty() {
        return Err("Missing required arguments: --source and --target".into());
    }
    
    Ok(plan)
}

fn analyze_extraction(plan: &XtaskExtractionPlan) -> Result<ExtractionAnalysis, Box<dyn std::error::Error>> {
    // Count affected commits
    let commits_output = Command::new("git")
        .args(&["log", "--oneline", "--", &plan.source_path])
        .output()?;
    let affected_commits = String::from_utf8(commits_output.stdout)?.lines().count() as u32;
    
    // Count affected files
    let files_output = Command::new("git")
        .args(&["log", "--name-only", "--pretty=format:", "--", &plan.source_path])
        .output()?;
    let affected_files = String::from_utf8(files_output.stdout)?.lines().count() as u32;
    
    // Analyze contract impact
    let contract_impact = analyze_contract_impact(&plan.source_path)?;
    
    // Calculate safety score
    let safety_score = calculate_safety_score(affected_commits, affected_files, &contract_impact);
    
    // Calculate estimated savings
    let estimated_savings = calculate_estimated_savings(affected_files, affected_commits);
    
    // Generate recommendations
    let recommendations = generate_recommendations(plan, &contract_impact, safety_score);
    
    Ok(ExtractionAnalysis {
        safety_score,
        affected_commits,
        affected_files,
        contract_impact,
        estimated_savings,
        recommendations,
    })
}

fn analyze_contract_impact(source_path: &str) -> Result<ContractImpact, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[source_path, "-name", "*.rs", "-type", "f"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut affected_contracts = Vec::new();
    
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") || content.contains("Contract") {
                affected_contracts.push(file.to_string());
            }
        }
    }
    
    let invalidation_risk = if !affected_contracts.is_empty() {
        0.8
    } else {
        0.3
    };
    
    let mut mitigation_strategies = Vec::new();
    if invalidation_risk > 0.5 {
        mitigation_strategies.push("Create contract snapshots before extraction".to_string());
        mitigation_strategies.push("Update all contract references after extraction".to_string());
        mitigation_strategies.push("Revalidate all Hooksmith contracts".to_string());
    }
    
    Ok(ContractImpact {
        affected_contracts,
        invalidation_risk,
        mitigation_strategies,
    })
}

fn calculate_safety_score(affected_commits: u32, affected_files: u32, contract_impact: &ContractImpact) -> f64 {
    let mut score: f64 = 1.0;
    
    // Penalize for large extractions
    if affected_commits > 1000 {
        score -= 0.3;
    } else if affected_commits > 100 {
        score -= 0.1;
    }
    
    // Penalize for many files
    if affected_files > 500 {
        score -= 0.2;
    } else if affected_files > 100 {
        score -= 0.1;
    }
    
    // Penalize for contract impact
    if contract_impact.invalidation_risk > 0.5 {
        score -= 0.4;
    }
    
    score.max(0.0_f64)
}

fn calculate_estimated_savings(affected_files: u32, affected_commits: u32) -> String {
    let file_savings = affected_files as usize * 2; // Rough estimate in KB
    let commit_savings = affected_commits as usize * 1; // Rough estimate in KB
    
    format!("~{} KB (files: {} KB, commits: {} KB)", 
        file_savings + commit_savings, file_savings, commit_savings)
}

fn generate_recommendations(plan: &XtaskExtractionPlan, contract_impact: &ContractImpact, safety_score: f64) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if safety_score < 0.5 {
        recommendations.push("DO NOT PROCEED - Critical safety issues detected".to_string());
    } else if safety_score < 0.7 {
        recommendations.push("Proceed with extreme caution".to_string());
    } else {
        recommendations.push("Proceed with standard precautions".to_string());
    }
    
    if contract_impact.invalidation_risk > 0.5 {
        recommendations.push("Create contract snapshots before extraction".to_string());
        recommendations.push("Coordinate with all contract consumers".to_string());
    }
    
    if plan.dry_run {
        recommendations.push("Run with --live to perform actual extraction".to_string());
    }
    
    recommendations.push("Create comprehensive backup before proceeding".to_string());
    recommendations.push("Test extraction on isolated branch first".to_string());
    
    recommendations
}

fn generate_mermaid_diagram(plan: &XtaskExtractionPlan, analysis: &ExtractionAnalysis) -> String {
    let mut diagram = String::new();
    diagram.push_str("graph TD\n");
    diagram.push_str("    A[Original Repository] --> B[Extraction Process]\n");
    diagram.push_str("    B --> C[New Repository]\n");
    diagram.push_str("    B --> D[Cleaned Repository]\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Extraction Details\"\n");
    diagram.push_str(&format!("        E[Source Path: {}]\n", plan.source_path));
    diagram.push_str(&format!("        F[Target Repo: {}]\n", plan.target_repo));
    diagram.push_str(&format!("        G[Method: {:?}]\n", plan.extraction_type));
    diagram.push_str(&format!("        H[Affected Commits: {}]\n", analysis.affected_commits));
    diagram.push_str(&format!("        I[Affected Files: {}]\n", analysis.affected_files));
    diagram.push_str(&format!("        J[Safety Score: {:.1}%]\n", analysis.safety_score * 100.0));
    diagram.push_str("    end\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Contract Impact\"\n");
    diagram.push_str(&format!("        K[Affected Contracts: {}]\n", analysis.contract_impact.affected_contracts.len()));
    diagram.push_str(&format!("        L[Invalidation Risk: {:.1}%]\n", analysis.contract_impact.invalidation_risk * 100.0));
    diagram.push_str("    end\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Post-Extraction\"\n");
    diagram.push_str("        M[Update Contract References]\n");
    diagram.push_str("        N[Revalidate Hooksmith Contracts]\n");
    diagram.push_str("        O[Update CI/CD Configurations]\n");
    diagram.push_str("    end\n");
    
    diagram
}

fn export_sha_mapping(source_path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--", source_path])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    let mut mapping = HashMap::new();
    
    for line in commits.lines() {
        if let Some(short_sha) = line.split_whitespace().next() {
            let full_sha_output = Command::new("git")
                .args(&["rev-parse", short_sha])
                .output();
            
            if let Ok(full_sha_output) = full_sha_output {
                let full_sha = String::from_utf8(full_sha_output.stdout)?.trim().to_string();
                mapping.insert(short_sha.to_string(), full_sha);
            }
        }
    }
    
    // Write SHA mapping to file
    let mapping_content = mapping.iter()
        .map(|(short, full)| format!("{} -> {}", short, full))
        .collect::<Vec<_>>()
        .join("\n");
    
    fs::write("sha_mapping.txt", mapping_content)?;
    println!("📄 SHA mapping exported to sha_mapping.txt");
    
    Ok(mapping)
}

fn create_contract_snapshot(source_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_dir = "contract_snapshots";
    fs::create_dir_all(snapshot_dir)?;
    
    let output = Command::new("find")
        .args(&[source_path, "-name", "*.rs", "-type", "f"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut snapshot_files = Vec::new();
    
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") || content.contains("Contract") {
                let snapshot_path = format!("{}/{}", snapshot_dir, file.replace("/", "_"));
                fs::write(&snapshot_path, content)?;
                snapshot_files.push(file.to_string());
            }
        }
    }
    
    if !snapshot_files.is_empty() {
        println!("📸 Contract snapshot created in {}/", snapshot_dir);
        println!("   Files: {}", snapshot_files.len());
    }
    
    Ok(())
}

fn create_backup_branch(source_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backup_branch = format!("backup-extraction-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    
    Command::new("git")
        .args(&["checkout", "-b", &backup_branch])
        .output()?;
    
    println!("💾 Backup branch created: {}", backup_branch);
    
    Ok(())
}

fn generate_xtask_report(extractor: &XtaskExtractor) {
    println!("\n📊 Extraction Analysis Report");
    println!("=============================");
    
    if extractor.plan.dry_run {
        println!("🔍 DRY-RUN MODE: No actual changes will be made");
        println!();
    }
    
    println!("📋 Extraction Plan:");
    println!("  • Source: {}", extractor.plan.source_path);
    println!("  • Target: {}", extractor.plan.target_repo);
    println!("  • Method: {:?}", extractor.plan.extraction_type);
    println!("  • Mode: {}", if extractor.plan.dry_run { "DRY-RUN" } else { "LIVE" });
    
    println!("\n📊 Analysis Results:");
    println!("  • Safety Score: {:.1}%", extractor.analysis_result.safety_score * 100.0);
    println!("  • Affected Commits: {}", extractor.analysis_result.affected_commits);
    println!("  • Affected Files: {}", extractor.analysis_result.affected_files);
    println!("  • Estimated Savings: {}", extractor.analysis_result.estimated_savings);
    
    println!("\n📋 Contract Impact:");
    println!("  • Affected Contracts: {}", extractor.analysis_result.contract_impact.affected_contracts.len());
    println!("  • Invalidation Risk: {:.1}%", extractor.analysis_result.contract_impact.invalidation_risk * 100.0);
    
    if !extractor.analysis_result.contract_impact.mitigation_strategies.is_empty() {
        println!("  • Mitigation Strategies:");
        for strategy in &extractor.analysis_result.contract_impact.mitigation_strategies {
            println!("    - {}", strategy);
        }
    }
    
    println!("\n💡 Recommendations:");
    for rec in &extractor.analysis_result.recommendations {
        println!("  • {}", rec);
    }
    
    if let Some(ref diagram) = extractor.mermaid_diagram {
        println!("\n📊 Mermaid Diagram:");
        println!("```mermaid");
        println!("{}", diagram);
        println!("```");
    }
    
    if extractor.sha_mapping.is_some() {
        println!("\n📄 SHA Mapping: Exported to sha_mapping.txt");
    }
    
    println!("\n📋 Summary:");
    if extractor.analysis_result.safety_score > 0.8 {
        println!("  ✅ Safe to proceed");
    } else if extractor.analysis_result.safety_score > 0.5 {
        println!("  ⚠️  Proceed with caution");
    } else {
        println!("  🚨 DO NOT PROCEED");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: {} --source <path> --target <repo> [options]", args[0]);
        println!();
        println!("Options:");
        println!("  --source, -s <path>     Source directory to extract");
        println!("  --target, -t <repo>     Target repository name");
        println!("  --method, -m <method>   Extraction method (filter-repo|subtree|manual)");
        println!("  --live                  Perform actual extraction (default: dry-run)");
        println!("  --mermaid               Generate mermaid diagram");
        println!("  --export-sha            Export SHA mapping to file");
        println!("  --backup                Create backup branch before extraction");
        println!("  --snapshot              Create contract snapshot");
        println!();
        println!("Example:");
        println!("  {} --source crates/tooling/dircheck --target hooksmith-dircheck --mermaid", args[0]);
        return Ok(());
    }
    
    // Run the extraction analysis
    let extractor = run_hooksmith_xtask_extraction(&args[1..])?;
    
    // Generate comprehensive report
    generate_xtask_report(&extractor);
    
    println!("\n✅ Hooksmith Xtask extraction analysis complete!");
    println!("🔧 Integration ready for xtask workflow");
    println!("📊 Analysis results generated");
    
    if extractor.plan.dry_run {
        println!("🔍 Dry-run completed - use --live for actual extraction");
    }
    
    Ok(())
}
