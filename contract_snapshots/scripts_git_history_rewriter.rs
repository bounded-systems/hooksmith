use std::collections::HashMap;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone)]
struct RewritePlan {
    operation_type: RewriteOperation,
    target_commits: Vec<String>,
    affected_files: Vec<String>,
    estimated_impact: RewriteImpact,
    safety_checks: Vec<SafetyCheck>,
    rollback_plan: RollbackPlan,
    contract_implications: Vec<String>,
}

#[derive(Debug, Clone)]
enum RewriteOperation {
    InteractiveRebase,
    FilterRepo,
    BFGCleaner,
    SquashCommits,
    RemoveFiles,
    ChangeAuthor,
    CustomCommand,
}

#[derive(Debug, Clone)]
enum RewriteImpact {
    Low,
    Medium,
    High,
    Critical,
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

#[derive(Debug, Clone)]
struct RollbackPlan {
    backup_branch: String,
    backup_remote: String,
    restore_commands: Vec<String>,
    verification_steps: Vec<String>,
}

#[derive(Debug)]
struct GitHistoryRewriter {
    current_state: RepositoryState,
    rewrite_plan: RewritePlan,
    contract_analysis: ContractAnalysis,
    safety_analysis: SafetyAnalysis,
    execution_plan: Vec<String>,
}

#[derive(Debug, Clone)]
struct RepositoryState {
    current_branch: String,
    total_commits: u32,
    total_branches: u32,
    remote_branches: Vec<String>,
    active_contracts: Vec<String>,
    recent_activity: Vec<String>,
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

fn analyze_git_history_rewrite() -> Result<GitHistoryRewriter, Box<dyn std::error::Error>> {
    println!("🔄 Analyzing Git history rewrite safety and impact...");
    
    // Analyze current repository state
    let current_state = analyze_repository_state()?;
    
    // Create rewrite plan
    let rewrite_plan = create_rewrite_plan(&current_state)?;
    
    // Analyze contract implications
    let contract_analysis = analyze_contract_implications(&current_state, &rewrite_plan)?;
    
    // Perform safety analysis
    let safety_analysis = perform_safety_analysis(&current_state, &rewrite_plan)?;
    
    // Generate execution plan
    let execution_plan = generate_execution_plan(&rewrite_plan, &safety_analysis);
    
    Ok(GitHistoryRewriter {
        current_state,
        rewrite_plan,
        contract_analysis,
        safety_analysis,
        execution_plan,
    })
}

fn analyze_repository_state() -> Result<RepositoryState, Box<dyn std::error::Error>> {
    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    let current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();
    
    // Count total commits
    let commits_output = Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output()?;
    let total_commits: u32 = String::from_utf8(commits_output.stdout)?.trim().parse().unwrap_or(0);
    
    // Count total branches
    let branches_output = Command::new("git")
        .args(&["branch", "-r"])
        .output()?;
    let branches_str = String::from_utf8(branches_output.stdout)?;
    let total_branches = branches_str.lines().count() as u32;
    
    // Get remote branches
    let remote_branches: Vec<String> = branches_str
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    
    // Analyze active contracts (simplified)
    let active_contracts = find_active_contracts()?;
    
    // Get recent activity
    let recent_activity = get_recent_activity()?;
    
    Ok(RepositoryState {
        current_branch,
        total_commits,
        total_branches,
        remote_branches,
        active_contracts,
        recent_activity,
    })
}

fn find_active_contracts() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Look for contract-related files
    let output = Command::new("find")
        .args(&[".", "-name", "*.rs", "-type", "f"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut contracts = Vec::new();
    
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") || content.contains("Contract") {
                contracts.push(file.to_string());
            }
        }
    }
    
    Ok(contracts)
}

fn get_recent_activity() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "10"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().map(|s| s.to_string()).collect())
}

fn create_rewrite_plan(state: &RepositoryState) -> Result<RewritePlan, Box<dyn std::error::Error>> {
    // Determine operation type based on repository state
    let operation_type = determine_operation_type(state)?;
    
    // Identify target commits
    let target_commits = identify_target_commits(state)?;
    
    // Identify affected files
    let affected_files = identify_affected_files(&target_commits)?;
    
    // Estimate impact
    let estimated_impact = estimate_rewrite_impact(state, &target_commits, &affected_files);
    
    // Create safety checks
    let safety_checks = create_safety_checks(state, &operation_type);
    
    // Create rollback plan
    let rollback_plan = create_rollback_plan(state);
    
    // Analyze contract implications
    let contract_implications = analyze_contract_implications_for_plan(state, &target_commits);
    
    Ok(RewritePlan {
        operation_type,
        target_commits,
        affected_files,
        estimated_impact,
        safety_checks,
        rollback_plan,
        contract_implications,
    })
}

fn determine_operation_type(state: &RepositoryState) -> Result<RewriteOperation, Box<dyn std::error::Error>> {
    // Analyze repository to determine best operation type
    if state.total_commits < 100 {
        Ok(RewriteOperation::InteractiveRebase)
    } else if state.total_commits > 1000 {
        Ok(RewriteOperation::FilterRepo)
    } else {
        Ok(RewriteOperation::SquashCommits)
    }
}

fn identify_target_commits(state: &RepositoryState) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Get recent commits that might be candidates for rewriting
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "20"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().map(|s| s.split_whitespace().next().unwrap_or("").to_string()).collect())
}

fn identify_affected_files(target_commits: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut affected_files = Vec::new();
    
    for commit in target_commits {
        let output = Command::new("git")
            .args(&["show", "--name-only", "--pretty=format:", commit])
            .output();
        
        if let Ok(output) = output {
            let files = String::from_utf8(output.stdout)?;
            for file in files.lines() {
                if !file.trim().is_empty() {
                    affected_files.push(file.trim().to_string());
                }
            }
        }
    }
    
    // Remove duplicates
    affected_files.sort();
    affected_files.dedup();
    
    Ok(affected_files)
}

fn estimate_rewrite_impact(state: &RepositoryState, target_commits: &[String], affected_files: &[String]) -> RewriteImpact {
    let commit_ratio = target_commits.len() as f64 / state.total_commits as f64;
    let file_ratio = affected_files.len() as f64 / 1000.0; // Assume 1000 files is significant
    
    if commit_ratio > 0.5 || file_ratio > 0.3 {
        RewriteImpact::Critical
    } else if commit_ratio > 0.2 || file_ratio > 0.1 {
        RewriteImpact::High
    } else if commit_ratio > 0.1 || file_ratio > 0.05 {
        RewriteImpact::Medium
    } else {
        RewriteImpact::Low
    }
}

fn create_safety_checks(state: &RepositoryState, operation: &RewriteOperation) -> Vec<SafetyCheck> {
    let mut checks = Vec::new();
    
    // Check if repository is clean
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
        recommendation: "Commit or stash all changes before rewriting".to_string(),
    });
    
    // Check for remote branches
    let remote_check = if state.remote_branches.len() > 1 {
        CheckStatus::Warning
    } else {
        CheckStatus::Passed
    };
    
    checks.push(SafetyCheck {
        name: "Remote Branch Impact".to_string(),
        status: remote_check,
        description: "Check impact on remote branches".to_string(),
        recommendation: "Coordinate with team members before rewriting".to_string(),
    });
    
    // Check for active contracts
    let contract_check = if !state.active_contracts.is_empty() {
        CheckStatus::Critical
    } else {
        CheckStatus::Passed
    };
    
    checks.push(SafetyCheck {
        name: "Active Contract Impact".to_string(),
        status: contract_check,
        description: "Check for active contracts that depend on commit SHAs".to_string(),
        recommendation: "Review and update all contract references".to_string(),
    });
    
    checks
}

fn create_rollback_plan(state: &RepositoryState) -> RollbackPlan {
    let backup_branch = format!("backup-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let backup_remote = "origin".to_string();
    
    let restore_commands = vec![
        format!("git checkout {}", backup_branch),
        format!("git branch -D {}", state.current_branch),
        format!("git checkout -b {}", state.current_branch),
        format!("git push --force {} {}", backup_remote, state.current_branch),
    ];
    
    let verification_steps = vec![
        "Verify all files are present".to_string(),
        "Check that contracts are still valid".to_string(),
        "Confirm CI/CD pipelines work".to_string(),
        "Test all functionality".to_string(),
    ];
    
    RollbackPlan {
        backup_branch,
        backup_remote,
        restore_commands,
        verification_steps,
    }
}

fn analyze_contract_implications_for_plan(state: &RepositoryState, target_commits: &[String]) -> Vec<String> {
    let mut implications = Vec::new();
    
    if !state.active_contracts.is_empty() {
        implications.push("Active contracts will be invalidated by SHA changes".to_string());
        implications.push("All contract references must be updated".to_string());
        implications.push("Hooksmith contract system will need revalidation".to_string());
    }
    
    if target_commits.len() > 10 {
        implications.push("Large number of commits will change SHAs".to_string());
        implications.push("Extensive contract revalidation required".to_string());
    }
    
    implications.push("Consider creating new contract snapshots after rewrite".to_string());
    implications.push("Update all CI/CD references to new commit SHAs".to_string());
    
    implications
}

fn analyze_contract_implications(state: &RepositoryState, plan: &RewritePlan) -> Result<ContractAnalysis, Box<dyn std::error::Error>> {
    let mut affected_contracts = Vec::new();
    let mut contract_hashes = Vec::new();
    
    // Find contracts that might be affected
    for contract_file in &state.active_contracts {
        affected_contracts.push(contract_file.clone());
        
        // Extract potential contract hashes
        if let Ok(content) = std::fs::read_to_string(contract_file) {
            // Look for SHA-like patterns
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
    } else if plan.target_commits.len() > 20 {
        0.6 // Medium risk for large rewrites
    } else {
        0.3 // Low risk for small rewrites
    };
    
    // Generate mitigation strategies
    let mut mitigation_strategies = Vec::new();
    
    if invalidation_risk > 0.5 {
        mitigation_strategies.push("Create contract snapshots before rewrite".to_string());
        mitigation_strategies.push("Update all contract references after rewrite".to_string());
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

fn perform_safety_analysis(state: &RepositoryState, plan: &RewritePlan) -> Result<SafetyAnalysis, Box<dyn std::error::Error>> {
    let checks = plan.safety_checks.clone();
    
    // Calculate overall safety score
    let total_checks = checks.len() as f64;
    let passed_checks = checks.iter().filter(|c| matches!(c.status, CheckStatus::Passed)).count() as f64;
    let warning_checks = checks.iter().filter(|c| matches!(c.status, CheckStatus::Warning)).count() as f64;
    let failed_checks = checks.iter().filter(|c| matches!(c.status, CheckStatus::Failed)).count() as f64;
    let critical_checks = checks.iter().filter(|c| matches!(c.status, CheckStatus::Critical)).count() as f64;
    
    let safety_score = (passed_checks + warning_checks * 0.5) / total_checks;
    let final_score = safety_score - (critical_checks * 0.3 / total_checks);
    
    // Identify critical risks
    let mut critical_risks = Vec::new();
    
    if critical_checks > 0.0 {
        critical_risks.push("Critical safety checks failed".to_string());
    }
    
    if matches!(plan.estimated_impact, RewriteImpact::Critical) {
        critical_risks.push("High impact rewrite detected".to_string());
    }
    
    if !state.active_contracts.is_empty() {
        critical_risks.push("Active contracts will be invalidated".to_string());
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
    recommendations.push("Test rewrite on isolated branch first".to_string());
    
    Ok(SafetyAnalysis {
        checks,
        overall_safety_score: final_score.max(0.0),
        critical_risks,
        recommendations,
    })
}

fn generate_execution_plan(plan: &RewritePlan, safety: &SafetyAnalysis) -> Vec<String> {
    let mut execution_plan = Vec::new();
    
    if safety.overall_safety_score < 0.5 {
        execution_plan.push("🚨 STOP: Critical safety issues prevent rewrite".to_string());
        return execution_plan;
    }
    
    execution_plan.push("📋 Pre-Rewrite Preparation:".to_string());
    execution_plan.push("  1. Create backup branch".to_string());
    execution_plan.push("  2. Notify all team members".to_string());
    execution_plan.push("  3. Create contract snapshots".to_string());
    execution_plan.push("  4. Test on isolated branch".to_string());
    
    execution_plan.push("🔧 Rewrite Execution:".to_string());
    match plan.operation_type {
        RewriteOperation::InteractiveRebase => {
            execution_plan.push("  git rebase -i HEAD~20".to_string());
        }
        RewriteOperation::FilterRepo => {
            execution_plan.push("  git filter-repo --path 'build/' --invert-paths".to_string());
        }
        RewriteOperation::BFGCleaner => {
            execution_plan.push("  bfg --delete-files '*.log'".to_string());
        }
        RewriteOperation::SquashCommits => {
            execution_plan.push("  git rebase -i --root".to_string());
        }
        _ => {
            execution_plan.push("  [Custom rewrite command]".to_string());
        }
    }
    
    execution_plan.push("✅ Post-Rewrite Actions:".to_string());
    execution_plan.push("  1. Force push to remote".to_string());
    execution_plan.push("  2. Update all contract references".to_string());
    execution_plan.push("  3. Revalidate Hooksmith contracts".to_string());
    execution_plan.push("  4. Update CI/CD configurations".to_string());
    execution_plan.push("  5. Notify all consumers".to_string());
    
    execution_plan
}

fn generate_git_history_rewrite_report(rewriter: &GitHistoryRewriter) {
    println!("\n🔄 Git History Rewrite Analysis");
    println!("===============================");
    
    // Show current repository state
    println!("\n📊 Current Repository State:");
    println!("  • Current branch: {}", rewriter.current_state.current_branch);
    println!("  • Total commits: {}", rewriter.current_state.total_commits);
    println!("  • Total branches: {}", rewriter.current_state.total_branches);
    println!("  • Active contracts: {}", rewriter.current_state.active_contracts.len());
    
    // Show rewrite plan
    println!("\n🎯 Rewrite Plan:");
    println!("  • Operation type: {:?}", rewriter.rewrite_plan.operation_type);
    println!("  • Target commits: {}", rewriter.rewrite_plan.target_commits.len());
    println!("  • Affected files: {}", rewriter.rewrite_plan.affected_files.len());
    println!("  • Estimated impact: {:?}", rewriter.rewrite_plan.estimated_impact);
    
    // Show safety analysis
    println!("\n🔒 Safety Analysis:");
    println!("  • Overall safety score: {:.1}%", rewriter.safety_analysis.overall_safety_score * 100.0);
    
    let passed_checks = rewriter.safety_analysis.checks.iter().filter(|c| matches!(c.status, CheckStatus::Passed)).count();
    let warning_checks = rewriter.safety_analysis.checks.iter().filter(|c| matches!(c.status, CheckStatus::Warning)).count();
    let failed_checks = rewriter.safety_analysis.checks.iter().filter(|c| matches!(c.status, CheckStatus::Failed)).count();
    let critical_checks = rewriter.safety_analysis.checks.iter().filter(|c| matches!(c.status, CheckStatus::Critical)).count();
    
    println!("  • Passed checks: {}", passed_checks);
    println!("  • Warning checks: {}", warning_checks);
    println!("  • Failed checks: {}", failed_checks);
    println!("  • Critical checks: {}", critical_checks);
    
    // Show contract analysis
    println!("\n📋 Contract Analysis:");
    println!("  • Affected contracts: {}", rewriter.contract_analysis.affected_contracts.len());
    println!("  • Contract hashes: {}", rewriter.contract_analysis.contract_hashes.len());
    println!("  • Invalidation risk: {:.1}%", rewriter.contract_analysis.invalidation_risk * 100.0);
    
    // Show critical risks
    if !rewriter.safety_analysis.critical_risks.is_empty() {
        println!("\n🚨 Critical Risks:");
        for risk in &rewriter.safety_analysis.critical_risks {
            println!("  • {}", risk);
        }
    }
    
    // Show recommendations
    if !rewriter.safety_analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &rewriter.safety_analysis.recommendations {
            println!("  • {}", rec);
        }
    }
    
    // Show execution plan
    if !rewriter.execution_plan.is_empty() {
        println!("\n🔧 Execution Plan:");
        for step in &rewriter.execution_plan {
            println!("  {}", step);
        }
    }
    
    // Show rollback plan
    println!("\n🔄 Rollback Plan:");
    println!("  • Backup branch: {}", rewriter.rewrite_plan.rollback_plan.backup_branch);
    println!("  • Backup remote: {}", rewriter.rewrite_plan.rollback_plan.backup_remote);
    
    // Summary
    println!("\n📋 Summary:");
    if rewriter.safety_analysis.overall_safety_score > 0.8 {
        println!("  • Safe to proceed with caution");
        println!("  • Follow execution plan carefully");
        println!("  • Monitor for issues");
    } else if rewriter.safety_analysis.overall_safety_score > 0.5 {
        println!("  • Proceed with extreme caution");
        println!("  • Address critical issues first");
        println!("  • Consider alternative approaches");
    } else {
        println!("  • DO NOT PROCEED");
        println!("  • Critical safety issues detected");
        println!("  • Resolve issues before considering rewrite");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Git History Rewriter");
    println!("=======================");
    println!("Analyzing Git history rewrite safety and impact...");
    println!();
    
    // Analyze Git history rewrite
    let rewriter = analyze_git_history_rewrite()?;
    
    // Generate comprehensive report
    generate_git_history_rewrite_report(&rewriter);
    
    println!("\n✅ Git history rewrite analysis complete!");
    println!("🔄 Safety assessed");
    println!("📋 Contract implications analyzed");
    println!("🔧 Execution plan prepared");
    println!("🔄 Rollback plan ready");
    
    Ok(())
}
