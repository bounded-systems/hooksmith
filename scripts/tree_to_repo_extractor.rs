use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct ExtractionPlan {
    source_path: String,
    target_repo: String,
    extraction_type: ExtractionType,
    affected_commits: Vec<String>,
    affected_files: Vec<String>,
    sha_mapping: HashMap<String, String>,
    safety_checks: Vec<SafetyCheck>,
    execution_steps: Vec<String>,
    dry_run_preview: DryRunPreview,
    mermaid_diagram: String,
}

#[derive(Debug, Clone)]
struct DryRunPreview {
    before_state: RepositorySnapshot,
    after_state: RepositorySnapshot,
    sha_mapping: HashMap<String, String>,
    affected_contracts: Vec<String>,
    estimated_savings: String,
}

#[derive(Debug, Clone)]
struct RepositorySnapshot {
    total_commits: u32,
    total_files: u32,
    total_size: String,
    crate_count: u32,
    contract_count: u32,
}

#[derive(Debug, Clone)]
enum ExtractionType {
    FilterRepo,
    SubtreeSplit,
    ManualExtraction,
}

#[derive(Debug, Clone)]
struct SafetyCheck {
    name: String,
    status: CheckStatus,
    description: String,
    recommendation: String,
}

#[derive(Debug, Clone)]
enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Critical,
}

#[derive(Debug)]
struct TreeToRepoExtractor {
    current_state: RepositoryState,
    extraction_plan: ExtractionPlan,
    contract_analysis: ContractAnalysis,
    safety_analysis: SafetyAnalysis,
    execution_plan: Vec<String>,
    dry_run_mode: bool,
}

#[derive(Debug, Clone)]
struct RepositoryState {
    current_branch: String,
    total_commits: u32,
    affected_commits: u32,
    affected_files: u32,
    contract_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
struct ContractAnalysis {
    affected_contracts: Vec<String>,
    contract_hashes: Vec<String>,
    invalidation_risk: f64,
    mitigation_strategies: Vec<String>,
}

#[derive(Debug, Clone)]
struct SafetyAnalysis {
    checks: Vec<SafetyCheck>,
    overall_safety_score: f64,
    critical_risks: Vec<String>,
    recommendations: Vec<String>,
}

fn analyze_tree_to_repo_extraction(
    source_path: &str,
    target_repo: &str,
    dry_run: bool,
) -> Result<TreeToRepoExtractor, Box<dyn std::error::Error>> {
    println!(
        "🌳 Analyzing tree-to-repo extraction for '{}' -> '{}'...",
        source_path, target_repo
    );
    if dry_run {
        println!("🔍 DRY-RUN MODE: No actual changes will be made");
    }

    // Analyze current repository state
    let current_state = analyze_repository_state(source_path)?;

    // Create extraction plan
    let extraction_plan =
        create_extraction_plan(source_path, target_repo, &current_state, dry_run)?;

    // Analyze contract implications
    let contract_analysis = analyze_contract_implications(&current_state, &extraction_plan)?;

    // Perform safety analysis
    let safety_analysis = perform_safety_analysis(&current_state, &extraction_plan)?;

    // Generate execution plan
    let execution_plan = generate_execution_plan(&extraction_plan, &safety_analysis, dry_run);

    Ok(TreeToRepoExtractor {
        current_state,
        extraction_plan,
        contract_analysis,
        safety_analysis,
        execution_plan,
        dry_run_mode: dry_run,
    })
}

fn analyze_repository_state(
    source_path: &str,
) -> Result<RepositoryState, Box<dyn std::error::Error>> {
    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    let current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    // Count total commits
    let commits_output = Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output()?;
    let total_commits: u32 = String::from_utf8(commits_output.stdout)?
        .trim()
        .parse()
        .unwrap_or(0);

    // Count affected commits
    let affected_commits_output = Command::new("git")
        .args(&["log", "--oneline", "--", source_path])
        .output()?;
    let affected_commits = String::from_utf8(affected_commits_output.stdout)?
        .lines()
        .count() as u32;

    // Count affected files
    let affected_files_output = Command::new("git")
        .args(&["log", "--name-only", "--pretty=format:", "--", source_path])
        .output()?;
    let affected_files = String::from_utf8(affected_files_output.stdout)?
        .lines()
        .count() as u32;

    // Find contract dependencies
    let contract_dependencies = find_contract_dependencies(source_path)?;

    Ok(RepositoryState {
        current_branch,
        total_commits,
        affected_commits,
        affected_files,
        contract_dependencies,
    })
}

fn find_contract_dependencies(
    source_path: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[source_path, "-name", "*.rs", "-type", "f"])
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    let mut dependencies = Vec::new();

    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") || content.contains("Contract") {
                dependencies.push(file.to_string());
            }
        }
    }

    Ok(dependencies)
}

fn create_extraction_plan(
    source_path: &str,
    target_repo: &str,
    state: &RepositoryState,
    dry_run: bool,
) -> Result<ExtractionPlan, Box<dyn std::error::Error>> {
    // Determine extraction type
    let extraction_type = determine_extraction_type(source_path, state)?;

    // Get affected commits
    let affected_commits = get_affected_commits(source_path)?;

    // Get affected files
    let affected_files = get_affected_files(source_path)?;

    // Create SHA mapping
    let sha_mapping = create_sha_mapping(&affected_commits)?;

    // Create safety checks
    let safety_checks = create_safety_checks(source_path, state, &extraction_type);

    // Generate execution steps
    let execution_steps = generate_execution_steps(source_path, target_repo, &extraction_type);

    // Create dry-run preview
    let dry_run_preview =
        create_dry_run_preview(source_path, state, &affected_files, &sha_mapping)?;

    // Generate mermaid diagram
    let mermaid_diagram = generate_mermaid_diagram(
        source_path,
        target_repo,
        &extraction_type,
        &affected_commits,
        &affected_files,
    );

    Ok(ExtractionPlan {
        source_path: source_path.to_string(),
        target_repo: target_repo.to_string(),
        extraction_type,
        affected_commits,
        affected_files,
        sha_mapping,
        safety_checks,
        execution_steps,
        dry_run_preview,
        mermaid_diagram,
    })
}

fn create_dry_run_preview(
    source_path: &str,
    state: &RepositoryState,
    affected_files: &[String],
    sha_mapping: &HashMap<String, String>,
) -> Result<DryRunPreview, Box<dyn std::error::Error>> {
    // Get current repository snapshot
    let before_state = get_repository_snapshot()?;

    // Estimate after state
    let after_state = estimate_after_state(&before_state, affected_files.len());

    // Calculate estimated savings
    let estimated_savings =
        calculate_estimated_savings(affected_files.len(), state.affected_commits);

    Ok(DryRunPreview {
        before_state,
        after_state,
        sha_mapping: sha_mapping.clone(),
        affected_contracts: state.contract_dependencies.clone(),
        estimated_savings,
    })
}

fn get_repository_snapshot() -> Result<RepositorySnapshot, Box<dyn std::error::Error>> {
    // Count total commits
    let commits_output = Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output()?;
    let total_commits: u32 = String::from_utf8(commits_output.stdout)?
        .trim()
        .parse()
        .unwrap_or(0);

    // Count total files
    let files_output = Command::new("git").args(&["ls-files"]).output()?;
    let total_files = String::from_utf8(files_output.stdout)?.lines().count() as u32;

    // Get repository size
    let size_output = Command::new("du").args(&["-sh", ".git"]).output()?;
    let total_size = String::from_utf8(size_output.stdout)?.trim().to_string();

    // Count crates
    let crates_output = Command::new("find")
        .args(&["crates", "-name", "Cargo.toml", "-type", "f"])
        .output();
    let crate_count = if let Ok(output) = crates_output {
        String::from_utf8(output.stdout)?.lines().count() as u32
    } else {
        0
    };

    // Count contracts (simplified)
    let contract_count = 0; // Would need more sophisticated contract detection

    Ok(RepositorySnapshot {
        total_commits,
        total_files,
        total_size,
        crate_count,
        contract_count,
    })
}

fn estimate_after_state(before: &RepositorySnapshot, affected_files: usize) -> RepositorySnapshot {
    let affected_files_u32 = affected_files as u32;

    RepositorySnapshot {
        total_commits: before.total_commits.saturating_sub(affected_files_u32 / 10), // Rough estimate
        total_files: before.total_files.saturating_sub(affected_files_u32),
        total_size: format!("{} (estimated reduction)", before.total_size),
        crate_count: before.crate_count.saturating_sub(1), // Assuming one crate extracted
        contract_count: before.contract_count,
    }
}

fn calculate_estimated_savings(affected_files: usize, affected_commits: u32) -> String {
    let file_savings = affected_files * 2; // Rough estimate in KB
    let commit_savings = affected_commits as usize * 1; // Rough estimate in KB

    format!(
        "~{} KB (files: {} KB, commits: {} KB)",
        file_savings + commit_savings,
        file_savings,
        commit_savings
    )
}

fn generate_mermaid_diagram(
    source_path: &str,
    target_repo: &str,
    extraction_type: &ExtractionType,
    affected_commits: &[String],
    affected_files: &[String],
) -> String {
    let mut diagram = String::new();
    diagram.push_str("graph TD\n");
    diagram.push_str("    A[Original Repository] --> B[Extraction Process]\n");
    diagram.push_str("    B --> C[New Repository]\n");
    diagram.push_str("    B --> D[Cleaned Repository]\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Extraction Details\"\n");
    diagram.push_str(&format!("        E[Source Path: {}]\n", source_path));
    diagram.push_str(&format!("        F[Target Repo: {}]\n", target_repo));
    diagram.push_str(&format!("        G[Method: {:?}]\n", extraction_type));
    diagram.push_str(&format!(
        "        H[Affected Commits: {}]\n",
        affected_commits.len()
    ));
    diagram.push_str(&format!(
        "        I[Affected Files: {}]\n",
        affected_files.len()
    ));
    diagram.push_str("    end\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Safety Checks\"\n");
    diagram.push_str("        J[Working Directory Clean]\n");
    diagram.push_str("        K[Contract Dependencies]\n");
    diagram.push_str("        L[Extraction Method Safety]\n");
    diagram.push_str("    end\n");
    diagram.push_str("    \n");
    diagram.push_str("    subgraph \"Post-Extraction\"\n");
    diagram.push_str("        M[Update Contract References]\n");
    diagram.push_str("        N[Revalidate Hooksmith Contracts]\n");
    diagram.push_str("        O[Update CI/CD Configurations]\n");
    diagram.push_str("    end\n");

    diagram
}

fn determine_extraction_type(
    source_path: &str,
    state: &RepositoryState,
) -> Result<ExtractionType, Box<dyn std::error::Error>> {
    // Check if path exists
    if !Path::new(source_path).exists() {
        return Err("Source path does not exist".into());
    }

    // Determine best extraction method based on repository state
    if state.affected_commits > 1000 {
        Ok(ExtractionType::FilterRepo)
    } else if state.affected_commits > 100 {
        Ok(ExtractionType::SubtreeSplit)
    } else {
        Ok(ExtractionType::ManualExtraction)
    }
}

fn get_affected_commits(source_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--", source_path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    Ok(commits
        .lines()
        .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        .collect())
}

fn get_affected_files(source_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--name-only", "--pretty=format:", "--", source_path])
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    let mut unique_files = std::collections::HashSet::new();

    for line in files.lines() {
        if !line.trim().is_empty() {
            unique_files.insert(line.trim().to_string());
        }
    }

    Ok(unique_files.into_iter().collect())
}

fn create_sha_mapping(
    affected_commits: &[String],
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();

    for commit in affected_commits {
        // Get the full SHA for each commit
        let output = Command::new("git").args(&["rev-parse", commit]).output();

        if let Ok(output) = output {
            let full_sha = String::from_utf8(output.stdout)?.trim().to_string();
            mapping.insert(commit.clone(), full_sha);
        }
    }

    Ok(mapping)
}

fn create_safety_checks(
    source_path: &str,
    state: &RepositoryState,
    extraction_type: &ExtractionType,
) -> Vec<SafetyCheck> {
    let mut checks = Vec::new();

    // Check if source path exists
    let path_exists = Path::new(source_path).exists();
    checks.push(SafetyCheck {
        name: "Source Path Exists".to_string(),
        status: if path_exists {
            CheckStatus::Passed
        } else {
            CheckStatus::Critical
        },
        description: "Check if the source path exists".to_string(),
        recommendation: "Ensure the source path is correct".to_string(),
    });

    // Check if working directory is clean
    let clean_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output();

    let clean_status = if let Ok(output) = clean_output {
        if output.stdout.is_empty() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        }
    } else {
        CheckStatus::Critical
    };

    checks.push(SafetyCheck {
        name: "Working Directory Clean".to_string(),
        status: clean_status,
        description: "Check if working directory has uncommitted changes".to_string(),
        recommendation: "Commit or stash all changes before extraction".to_string(),
    });

    // Check for contract dependencies
    let contract_check = if !state.contract_dependencies.is_empty() {
        CheckStatus::Warning
    } else {
        CheckStatus::Passed
    };

    checks.push(SafetyCheck {
        name: "Contract Dependencies".to_string(),
        status: contract_check,
        description: "Check for contracts that depend on the extracted code".to_string(),
        recommendation: "Review and update contract references".to_string(),
    });

    // Check extraction type safety
    let extraction_check = match extraction_type {
        ExtractionType::FilterRepo => CheckStatus::Passed,
        ExtractionType::SubtreeSplit => CheckStatus::Warning,
        ExtractionType::ManualExtraction => CheckStatus::Failed,
    };

    checks.push(SafetyCheck {
        name: "Extraction Method Safety".to_string(),
        status: extraction_check,
        description: "Check if the extraction method is safe".to_string(),
        recommendation: "Use git filter-repo for large extractions".to_string(),
    });

    checks
}

fn generate_execution_steps(
    source_path: &str,
    target_repo: &str,
    extraction_type: &ExtractionType,
) -> Vec<String> {
    let mut steps = Vec::new();

    match extraction_type {
        ExtractionType::FilterRepo => {
            steps.push("1. Clone the repository locally".to_string());
            steps.push(format!("   git clone --no-local . ../{}", target_repo));
            steps.push("2. Navigate to the cloned repository".to_string());
            steps.push(format!("   cd ../{}", target_repo));
            steps.push("3. Extract the subdirectory with full history".to_string());
            steps.push(format!(
                "   git filter-repo --subdirectory-filter {}",
                source_path
            ));
            steps.push("4. Add the new remote".to_string());
            steps.push(format!(
                "   git remote add origin https://github.com/yourorg/{}",
                target_repo
            ));
            steps.push("5. Push to the new repository".to_string());
            steps.push("   git push -u origin main".to_string());
            steps.push("6. Remove from original repository".to_string());
            steps.push(format!(
                "   git filter-repo --path {} --invert-paths",
                source_path
            ));
        }
        ExtractionType::SubtreeSplit => {
            steps.push("1. Create a subtree split branch".to_string());
            steps.push(format!(
                "   git subtree split --prefix={} -b {}-history",
                source_path, target_repo
            ));
            steps.push("2. Checkout the split branch".to_string());
            steps.push(format!("   git checkout {}-history", target_repo));
            steps.push("3. Create new repository".to_string());
            steps.push(format!("   mkdir ../{}", target_repo));
            steps.push("4. Initialize new repository".to_string());
            steps.push(format!("   cd ../{} && git init", target_repo));
            steps.push("5. Add the split branch as remote".to_string());
            steps.push("   git remote add origin ../original-repo".to_string());
            steps.push("6. Pull the split history".to_string());
            steps.push(format!("   git pull origin {}-history", target_repo));
        }
        ExtractionType::ManualExtraction => {
            steps.push("1. Create new repository".to_string());
            steps.push(format!("   mkdir ../{}", target_repo));
            steps.push("2. Copy files manually".to_string());
            steps.push(format!("   cp -r {} ../{}", source_path, target_repo));
            steps.push("3. Initialize git repository".to_string());
            steps.push(format!("   cd ../{} && git init", target_repo));
            steps.push("4. Add and commit files".to_string());
            steps.push("   git add . && git commit -m 'Initial extraction'".to_string());
        }
    }

    steps
}

fn analyze_contract_implications(
    state: &RepositoryState,
    plan: &ExtractionPlan,
) -> Result<ContractAnalysis, Box<dyn std::error::Error>> {
    let mut affected_contracts = Vec::new();
    let mut contract_hashes = Vec::new();

    // Find contracts that might be affected
    for contract_file in &state.contract_dependencies {
        affected_contracts.push(contract_file.clone());

        // Extract potential contract hashes
        if let Ok(content) = std::fs::read_to_string(contract_file) {
            for line in content.lines() {
                if line.contains("commit") || line.contains("sha") {
                    contract_hashes.push(line.to_string());
                }
            }
        }
    }

    // Calculate invalidation risk
    let invalidation_risk = if !affected_contracts.is_empty() {
        0.8 // High risk if contracts exist
    } else if plan.affected_commits.len() > 100 {
        0.6 // Medium risk for large extractions
    } else {
        0.3 // Low risk for small extractions
    };

    // Generate mitigation strategies
    let mut mitigation_strategies = Vec::new();

    if invalidation_risk > 0.5 {
        mitigation_strategies.push("Create contract snapshots before extraction".to_string());
        mitigation_strategies.push("Update all contract references after extraction".to_string());
        mitigation_strategies.push("Revalidate all Hooksmith contracts".to_string());
    }

    mitigation_strategies.push("Use --depth=1 clones for isolated testing".to_string());
    mitigation_strategies.push("Coordinate with all contract consumers".to_string());

    Ok(ContractAnalysis {
        affected_contracts,
        contract_hashes,
        invalidation_risk,
        mitigation_strategies,
    })
}

fn perform_safety_analysis(
    state: &RepositoryState,
    plan: &ExtractionPlan,
) -> Result<SafetyAnalysis, Box<dyn std::error::Error>> {
    let checks = plan.safety_checks.clone();

    // Calculate overall safety score
    let total_checks = checks.len() as f64;
    let passed_checks = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Passed))
        .count() as f64;
    let warning_checks = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Warning))
        .count() as f64;
    let failed_checks = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Failed))
        .count() as f64;
    let critical_checks = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Critical))
        .count() as f64;

    let safety_score = (passed_checks + warning_checks * 0.5) / total_checks;
    let final_score = safety_score - (critical_checks * 0.3 / total_checks);

    // Identify critical risks
    let mut critical_risks = Vec::new();

    if critical_checks > 0.0 {
        critical_risks.push("Critical safety checks failed".to_string());
    }

    if state.affected_commits > 1000 {
        critical_risks.push("Large extraction detected".to_string());
    }

    if !state.contract_dependencies.is_empty() {
        critical_risks.push("Contract dependencies will be affected".to_string());
    }

    // Generate recommendations
    let mut recommendations = Vec::new();

    if final_score < 0.5 {
        recommendations.push("DO NOT PROCEED - Critical safety issues detected".to_string());
    } else if final_score < 0.7 {
        recommendations.push("Proceed with extreme caution".to_string());
    } else {
        recommendations.push("Proceed with standard precautions".to_string());
    }

    recommendations.push("Create comprehensive backup before proceeding".to_string());
    recommendations.push("Coordinate with all team members".to_string());
    recommendations.push("Test extraction on isolated branch first".to_string());

    Ok(SafetyAnalysis {
        checks,
        overall_safety_score: final_score.max(0.0),
        critical_risks,
        recommendations,
    })
}

fn generate_execution_plan(
    plan: &ExtractionPlan,
    safety: &SafetyAnalysis,
    dry_run: bool,
) -> Vec<String> {
    let mut execution_plan = Vec::new();

    if safety.overall_safety_score < 0.5 {
        execution_plan.push("🚨 STOP: Critical safety issues prevent extraction".to_string());
        return execution_plan;
    }

    if dry_run {
        execution_plan.push("🔍 DRY-RUN MODE: No actual changes will be made".to_string());
    }

    execution_plan.push("📋 Pre-Extraction Preparation:".to_string());
    execution_plan.push("  1. Create backup branch".to_string());
    execution_plan.push("  2. Notify all team members".to_string());
    execution_plan.push("  3. Create contract snapshots".to_string());
    execution_plan.push("  4. Test on isolated branch".to_string());

    execution_plan.push("🔧 Extraction Execution:".to_string());
    for step in &plan.execution_steps {
        execution_plan.push(format!("  {}", step));
    }

    execution_plan.push("✅ Post-Extraction Actions:".to_string());
    execution_plan.push("  1. Update all contract references".to_string());
    execution_plan.push("  2. Revalidate Hooksmith contracts".to_string());
    execution_plan.push("  3. Update CI/CD configurations".to_string());
    execution_plan.push("  4. Notify all consumers".to_string());

    execution_plan
}

fn generate_tree_to_repo_extraction_report(extractor: &TreeToRepoExtractor) {
    println!("\n🌳 Tree-to-Repo Extraction Analysis");
    println!("===================================");

    if extractor.dry_run_mode {
        println!("🔍 DRY-RUN MODE: No actual changes will be made");
        println!();
    }

    // Show current repository state
    println!("\n📊 Current Repository State:");
    println!(
        "  • Current branch: {}",
        extractor.current_state.current_branch
    );
    println!(
        "  • Total commits: {}",
        extractor.current_state.total_commits
    );
    println!(
        "  • Affected commits: {}",
        extractor.current_state.affected_commits
    );
    println!(
        "  • Affected files: {}",
        extractor.current_state.affected_files
    );
    println!(
        "  • Contract dependencies: {}",
        extractor.current_state.contract_dependencies.len()
    );

    // Show extraction plan
    println!("\n🎯 Extraction Plan:");
    println!("  • Source path: {}", extractor.extraction_plan.source_path);
    println!("  • Target repo: {}", extractor.extraction_plan.target_repo);
    println!(
        "  • Extraction type: {:?}",
        extractor.extraction_plan.extraction_type
    );
    println!(
        "  • Affected commits: {}",
        extractor.extraction_plan.affected_commits.len()
    );
    println!(
        "  • Affected files: {}",
        extractor.extraction_plan.affected_files.len()
    );
    println!(
        "  • SHA mappings: {}",
        extractor.extraction_plan.sha_mapping.len()
    );

    // Show dry-run preview
    if extractor.dry_run_mode {
        println!("\n🔍 Dry-Run Preview:");
        println!("  📊 Before Extraction:");
        println!(
            "    • Total commits: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .before_state
                .total_commits
        );
        println!(
            "    • Total files: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .before_state
                .total_files
        );
        println!(
            "    • Repository size: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .before_state
                .total_size
        );
        println!(
            "    • Crate count: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .before_state
                .crate_count
        );

        println!("  📊 After Extraction:");
        println!(
            "    • Total commits: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .after_state
                .total_commits
        );
        println!(
            "    • Total files: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .after_state
                .total_files
        );
        println!(
            "    • Repository size: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .after_state
                .total_size
        );
        println!(
            "    • Crate count: {}",
            extractor
                .extraction_plan
                .dry_run_preview
                .after_state
                .crate_count
        );

        println!(
            "  💾 Estimated Savings: {}",
            extractor.extraction_plan.dry_run_preview.estimated_savings
        );
        println!(
            "  🔗 SHA Mappings: {}",
            extractor.extraction_plan.dry_run_preview.sha_mapping.len()
        );
    }

    // Show safety analysis
    println!("\n🔒 Safety Analysis:");
    println!(
        "  • Overall safety score: {:.1}%",
        extractor.safety_analysis.overall_safety_score * 100.0
    );

    let passed_checks = extractor
        .safety_analysis
        .checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Passed))
        .count();
    let warning_checks = extractor
        .safety_analysis
        .checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Warning))
        .count();
    let failed_checks = extractor
        .safety_analysis
        .checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Failed))
        .count();
    let critical_checks = extractor
        .safety_analysis
        .checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Critical))
        .count();

    println!("  • Passed checks: {}", passed_checks);
    println!("  • Warning checks: {}", warning_checks);
    println!("  • Failed checks: {}", failed_checks);
    println!("  • Critical checks: {}", critical_checks);

    // Show contract analysis
    println!("\n📋 Contract Analysis:");
    println!(
        "  • Affected contracts: {}",
        extractor.contract_analysis.affected_contracts.len()
    );
    println!(
        "  • Contract hashes: {}",
        extractor.contract_analysis.contract_hashes.len()
    );
    println!(
        "  • Invalidation risk: {:.1}%",
        extractor.contract_analysis.invalidation_risk * 100.0
    );

    // Show critical risks
    if !extractor.safety_analysis.critical_risks.is_empty() {
        println!("\n🚨 Critical Risks:");
        for risk in &extractor.safety_analysis.critical_risks {
            println!("  • {}", risk);
        }
    }

    // Show recommendations
    if !extractor.safety_analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &extractor.safety_analysis.recommendations {
            println!("  • {}", rec);
        }
    }

    // Show execution plan
    if !extractor.execution_plan.is_empty() {
        println!("\n🔧 Execution Plan:");
        for step in &extractor.execution_plan {
            println!("  {}", step);
        }
    }

    // Show mermaid diagram
    if extractor.dry_run_mode {
        println!("\n📊 Mermaid Diagram:");
        println!("```mermaid");
        println!("{}", extractor.extraction_plan.mermaid_diagram);
        println!("```");
    }

    // Summary
    println!("\n📋 Summary:");
    if extractor.safety_analysis.overall_safety_score > 0.8 {
        println!("  • Safe to proceed with caution");
        println!("  • Follow execution plan carefully");
        println!("  • Monitor for issues");
    } else if extractor.safety_analysis.overall_safety_score > 0.5 {
        println!("  • Proceed with extreme caution");
        println!("  • Address critical issues first");
        println!("  • Consider alternative approaches");
    } else {
        println!("  • DO NOT PROCEED");
        println!("  • Critical safety issues detected");
        println!("  • Resolve issues before considering extraction");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <source_path> <target_repo> [--dry-run]", args[0]);
        println!(
            "Example: {} crates/tooling/dircheck hooksmith-dircheck --dry-run",
            args[0]
        );
        return Ok(());
    }

    let source_path = &args[1];
    let target_repo = &args[2];
    let dry_run = args.contains(&"--dry-run".to_string());

    println!("🌳 Tree-to-Repo Extractor");
    println!("=========================");
    println!(
        "Analyzing extraction from '{}' to '{}'...",
        source_path, target_repo
    );
    if dry_run {
        println!("🔍 DRY-RUN MODE: No actual changes will be made");
    }
    println!();

    // Analyze tree-to-repo extraction
    let extractor = analyze_tree_to_repo_extraction(source_path, target_repo, dry_run)?;

    // Generate comprehensive report
    generate_tree_to_repo_extraction_report(&extractor);

    println!("\n✅ Tree-to-repo extraction analysis complete!");
    println!("🌳 Extraction plan prepared");
    println!("🔒 Safety assessed");
    println!("📋 Contract implications analyzed");
    println!("🔧 Execution plan ready");
    if dry_run {
        println!("🔍 Dry-run preview generated");
        println!("📊 Mermaid diagram created");
    }

    Ok(())
}
