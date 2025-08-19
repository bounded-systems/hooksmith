use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
struct ContractStabilityIssue {
    file_path: String,
    issue_type: StabilityIssueType,
    severity: IssueSeverity,
    description: String,
    recommendation: String,
    sha_churn_score: f64,
    object_size: u64,
    change_frequency: u32,
}

#[derive(Debug, Clone)]
enum StabilityIssueType {
    LargeBlob,
    FrequentChanges,
    ModularBoundaryViolation,
    HighShaChurn,
    UnstableContract,
    PoorDeltaCompression,
}

#[derive(Debug, Clone)]
enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
struct ContractStabilityAnalysis {
    issues: Vec<ContractStabilityIssue>,
    stability_score: f64,
    recommendations: Vec<String>,
    modular_boundaries: HashMap<String, Vec<String>>,
    sha_churn_analysis: HashMap<String, f64>,
}

fn analyze_contract_stability() -> Result<ContractStabilityAnalysis, Box<dyn std::error::Error>> {
    println!("🔒 Analyzing contract stability based on Git object patterns...");

    // Get all files in the repository
    let output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let files_output = String::from_utf8(output.stdout)?;
    let mut issues = Vec::new();
    let mut sha_churn_analysis = HashMap::new();

    for line in files_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3];

            // Skip certain file types
            if should_skip_file(path) {
                continue;
            }

            // Analyze file stability
            let file_analysis = analyze_file_stability(path)?;

            if let Some(issue) = file_analysis {
                issues.push(issue);
            }

            // Track SHA churn
            if let Some(churn_score) = calculate_sha_churn(path)? {
                sha_churn_analysis.insert(path.to_string(), churn_score);
            }
        }
    }

    // Analyze modular boundaries
    let modular_boundaries = analyze_modular_boundaries(&issues);

    // Calculate overall stability score
    let stability_score = calculate_stability_score(&issues);

    // Generate recommendations
    let recommendations = generate_stability_recommendations(&issues, &sha_churn_analysis);

    Ok(ContractStabilityAnalysis {
        issues,
        stability_score,
        recommendations,
        modular_boundaries,
        sha_churn_analysis,
    })
}

fn should_skip_file(path: &str) -> bool {
    let skip_patterns = [
        ".git/",
        "target/",
        "node_modules/",
        ".cargo/",
        "*.log",
        "*.tmp",
        "*.cache",
        "*.lock",
    ];

    for pattern in &skip_patterns {
        if path.contains(pattern) {
            return true;
        }
    }

    false
}

fn analyze_file_stability(
    path: &str,
) -> Result<Option<ContractStabilityIssue>, Box<dyn std::error::Error>> {
    // Get file size
    let size_output = Command::new("git")
        .args(&["cat-file", "-s", "HEAD:"])
        .arg(path)
        .output()?;

    let size_str = String::from_utf8(size_output.stdout)?;
    let object_size: u64 = size_str.trim().parse().unwrap_or(0);

    // Get change frequency
    let change_frequency = get_change_frequency(path)?;

    // Calculate SHA churn score
    let sha_churn_score = calculate_file_sha_churn(path)?;

    // Determine issues
    let mut issues = Vec::new();

    // Large blob detection
    if object_size > 100 * 1024 {
        // 100KB
        issues.push(StabilityIssueType::LargeBlob);
    }

    // Frequent changes detection
    if change_frequency > 10 {
        issues.push(StabilityIssueType::FrequentChanges);
    }

    // High SHA churn detection
    if sha_churn_score > 0.7 {
        issues.push(StabilityIssueType::HighShaChurn);
    }

    // Poor delta compression detection
    if object_size > 50 * 1024 && sha_churn_score > 0.5 {
        issues.push(StabilityIssueType::PoorDeltaCompression);
    }

    // Unstable contract detection
    if change_frequency > 5 && sha_churn_score > 0.6 {
        issues.push(StabilityIssueType::UnstableContract);
    }

    if issues.is_empty() {
        return Ok(None);
    }

    // Determine severity
    let severity = determine_severity(&issues, object_size, change_frequency, sha_churn_score);

    // Generate description and recommendation
    let description =
        generate_issue_description(&issues, object_size, change_frequency, sha_churn_score);
    let recommendation = generate_issue_recommendation(&issues, path);

    Ok(Some(ContractStabilityIssue {
        file_path: path.to_string(),
        issue_type: issues[0].clone(), // Primary issue
        severity,
        description,
        recommendation,
        sha_churn_score,
        object_size,
        change_frequency,
    }))
}

fn get_change_frequency(path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--follow", "--", path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().count() as u32)
}

fn calculate_file_sha_churn(path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Get recent commits for this file
    let output = Command::new("git")
        .args(&["log", "--format=%H", "--follow", "-n", "10", "--", path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    let commit_hashes: Vec<&str> = commits.lines().collect();

    if commit_hashes.len() < 2 {
        return Ok(0.0);
    }

    // Calculate SHA differences between consecutive commits
    let mut total_diff = 0;
    let mut comparisons = 0;

    for i in 0..commit_hashes.len() - 1 {
        let current_sha = commit_hashes[i];
        let previous_sha = commit_hashes[i + 1];

        // Get file content at each commit
        let current_content = get_file_content_at_commit(path, current_sha)?;
        let previous_content = get_file_content_at_commit(path, previous_sha)?;

        // Calculate similarity (simplified)
        let similarity = calculate_content_similarity(&current_content, &previous_content);
        total_diff += (1.0 - similarity) as u32;
        comparisons += 1;
    }

    if comparisons > 0 {
        Ok(total_diff as f64 / comparisons as f64)
    } else {
        Ok(0.0)
    }
}

fn get_file_content_at_commit(
    path: &str,
    commit: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["show", &format!("{}:{}", commit, path)])
        .output();

    match output {
        Ok(output) => Ok(String::from_utf8(output.stdout).unwrap_or_default()),
        Err(_) => Ok(String::new()),
    }
}

fn calculate_content_similarity(content1: &str, content2: &str) -> f64 {
    if content1.is_empty() && content2.is_empty() {
        return 1.0;
    }

    if content1.is_empty() || content2.is_empty() {
        return 0.0;
    }

    let len1 = content1.len();
    let len2 = content2.len();
    let max_len = len1.max(len2) as f64;

    // Simple similarity based on length difference
    let length_diff = (len1 as i32 - len2 as i32).abs() as f64;
    let similarity = 1.0 - (length_diff / max_len);

    similarity.max(0.0)
}

fn calculate_sha_churn(path: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    let churn_score = calculate_file_sha_churn(path)?;
    if churn_score > 0.0 {
        Ok(Some(churn_score))
    } else {
        Ok(None)
    }
}

fn determine_severity(
    issues: &[StabilityIssueType],
    size: u64,
    frequency: u32,
    churn: f64,
) -> IssueSeverity {
    let mut severity_score = 0;

    for issue in issues {
        match issue {
            StabilityIssueType::LargeBlob => {
                if size > 1024 * 1024 {
                    // 1MB
                    severity_score += 3;
                } else {
                    severity_score += 1;
                }
            }
            StabilityIssueType::FrequentChanges => {
                if frequency > 20 {
                    severity_score += 3;
                } else if frequency > 10 {
                    severity_score += 2;
                } else {
                    severity_score += 1;
                }
            }
            StabilityIssueType::HighShaChurn => {
                if churn > 0.8 {
                    severity_score += 3;
                } else if churn > 0.6 {
                    severity_score += 2;
                } else {
                    severity_score += 1;
                }
            }
            StabilityIssueType::UnstableContract => severity_score += 3,
            StabilityIssueType::PoorDeltaCompression => severity_score += 2,
            StabilityIssueType::ModularBoundaryViolation => severity_score += 2,
        }
    }

    match severity_score {
        0..=2 => IssueSeverity::Low,
        3..=5 => IssueSeverity::Medium,
        6..=8 => IssueSeverity::High,
        _ => IssueSeverity::Critical,
    }
}

fn generate_issue_description(
    issues: &[StabilityIssueType],
    size: u64,
    frequency: u32,
    churn: f64,
) -> String {
    let mut descriptions = Vec::new();

    for issue in issues {
        match issue {
            StabilityIssueType::LargeBlob => {
                descriptions.push(format!("Large blob ({} KB)", size / 1024));
            }
            StabilityIssueType::FrequentChanges => {
                descriptions.push(format!("Frequently changed ({} commits)", frequency));
            }
            StabilityIssueType::HighShaChurn => {
                descriptions.push(format!("High SHA churn ({:.1}%)", churn * 100.0));
            }
            StabilityIssueType::UnstableContract => {
                descriptions
                    .push("Unstable contract - frequent changes with high churn".to_string());
            }
            StabilityIssueType::PoorDeltaCompression => {
                descriptions.push("Poor delta compression potential".to_string());
            }
            StabilityIssueType::ModularBoundaryViolation => {
                descriptions.push("Modular boundary violation".to_string());
            }
        }
    }

    descriptions.join(", ")
}

fn generate_issue_recommendation(issues: &[StabilityIssueType], path: &str) -> String {
    let mut recommendations = Vec::new();

    for issue in issues {
        match issue {
            StabilityIssueType::LargeBlob => {
                recommendations.push("Consider splitting into smaller modules".to_string());
            }
            StabilityIssueType::FrequentChanges => {
                recommendations.push("Extract stable logic to separate files".to_string());
            }
            StabilityIssueType::HighShaChurn => {
                recommendations.push("Freeze interface or use contract memoization".to_string());
            }
            StabilityIssueType::UnstableContract => {
                recommendations.push("Implement contract caching by object SHA".to_string());
            }
            StabilityIssueType::PoorDeltaCompression => {
                recommendations.push("Refactor for better delta compression".to_string());
            }
            StabilityIssueType::ModularBoundaryViolation => {
                recommendations.push("Respect modular boundaries".to_string());
            }
        }
    }

    if recommendations.is_empty() {
        "Monitor for stability issues".to_string()
    } else {
        recommendations.join("; ")
    }
}

fn analyze_modular_boundaries(issues: &[ContractStabilityIssue]) -> HashMap<String, Vec<String>> {
    let mut boundaries = HashMap::new();

    // Group issues by module
    for issue in issues {
        let module = extract_module_from_path(&issue.file_path);
        boundaries
            .entry(module)
            .or_insert_with(Vec::new)
            .push(issue.file_path.clone());
    }

    boundaries
}

fn extract_module_from_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        if parts[0] == "src" && parts.len() > 2 {
            parts[1].to_string()
        } else if parts[0] == "crates" && parts.len() > 2 {
            format!("crates/{}", parts[1])
        } else {
            parts[0].to_string()
        }
    } else {
        "root".to_string()
    }
}

fn calculate_stability_score(issues: &[ContractStabilityIssue]) -> f64 {
    if issues.is_empty() {
        return 1.0;
    }

    let total_issues = issues.len() as f64;
    let critical_issues = issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Critical))
        .count() as f64;
    let high_issues = issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::High))
        .count() as f64;
    let medium_issues = issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Medium))
        .count() as f64;

    let weighted_score =
        (critical_issues * 4.0 + high_issues * 2.0 + medium_issues * 1.0) / total_issues;

    (1.0 - weighted_score).max(0.0)
}

fn generate_stability_recommendations(
    issues: &[ContractStabilityIssue],
    sha_churn: &HashMap<String, f64>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    let critical_count = issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Critical))
        .count();
    let high_count = issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::High))
        .count();

    if critical_count > 0 {
        recommendations.push(
            "Critical stability issues detected - prioritize contract optimization".to_string(),
        );
    }

    if high_count > 5 {
        recommendations
            .push("Multiple high-severity issues - implement contract memoization".to_string());
    }

    let avg_churn: f64 = sha_churn.values().sum::<f64>() / sha_churn.len().max(1) as f64;
    if avg_churn > 0.5 {
        recommendations.push("High average SHA churn - consider freezing interfaces".to_string());
    }

    let large_blobs = issues
        .iter()
        .filter(|i| matches!(i.issue_type, StabilityIssueType::LargeBlob))
        .count();
    if large_blobs > 3 {
        recommendations
            .push("Multiple large blobs - implement modularization strategy".to_string());
    }

    recommendations.push("Use git attributes for contract-aware filtering".to_string());
    recommendations.push("Implement contract.lock system for object identity tracking".to_string());

    recommendations
}

fn generate_contract_stability_report(analysis: &ContractStabilityAnalysis) {
    println!("\n🔒 Contract Stability Analysis");
    println!("=============================");

    // Show critical and high severity issues
    let critical_issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Critical))
        .collect();

    let high_issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::High))
        .collect();

    if !critical_issues.is_empty() {
        println!("\n🚨 Critical Stability Issues:");
        for issue in &critical_issues {
            println!(
                "  • {} ({:.1}KB, {} changes, {:.1}% churn)",
                issue.file_path,
                issue.object_size as f64 / 1024.0,
                issue.change_frequency,
                issue.sha_churn_score * 100.0
            );
            println!("    {}", issue.description);
            println!("    Recommendation: {}", issue.recommendation);
            println!();
        }
    }

    if !high_issues.is_empty() {
        println!("\n⚠️  High Severity Issues:");
        for issue in high_issues.iter().take(5) {
            println!(
                "  • {} ({:.1}KB, {} changes, {:.1}% churn)",
                issue.file_path,
                issue.object_size as f64 / 1024.0,
                issue.change_frequency,
                issue.sha_churn_score * 100.0
            );
            println!("    {}", issue.description);
            println!();
        }

        if high_issues.len() > 5 {
            println!(
                "  ... and {} more high-severity issues",
                high_issues.len() - 5
            );
        }
    }

    // Show stability score
    println!("\n📊 Stability Metrics:");
    println!(
        "  • Overall stability score: {:.1}%",
        analysis.stability_score * 100.0
    );
    println!("  • Total issues: {}", analysis.issues.len());
    println!("  • Critical issues: {}", critical_issues.len());
    println!("  • High severity issues: {}", high_issues.len());

    // Show SHA churn analysis
    if !analysis.sha_churn_analysis.is_empty() {
        let avg_churn: f64 = analysis.sha_churn_analysis.values().sum::<f64>()
            / analysis.sha_churn_analysis.len() as f64;
        println!("  • Average SHA churn: {:.1}%", avg_churn * 100.0);

        let high_churn_files: Vec<_> = analysis
            .sha_churn_analysis
            .iter()
            .filter(|(_, &churn)| churn > 0.7)
            .take(3)
            .collect();

        if !high_churn_files.is_empty() {
            println!("  • High churn files:");
            for (file, churn) in high_churn_files {
                println!("    - {} ({:.1}% churn)", file, churn * 100.0);
            }
        }
    }

    // Show modular boundaries
    if !analysis.modular_boundaries.is_empty() {
        println!("\n📦 Modular Boundary Analysis:");
        for (module, files) in &analysis.modular_boundaries {
            if files.len() > 1 {
                println!(
                    "  • {}: {} files with stability issues",
                    module,
                    files.len()
                );
            }
        }
    }

    // Show recommendations
    if !analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &analysis.recommendations {
            println!("  • {}", rec);
        }
    }

    // Summary
    println!("\n📈 Summary:");
    if analysis.stability_score > 0.8 {
        println!("  • Contract system is stable");
        println!("  • Good object stability");
        println!("  • Efficient caching potential");
    } else if analysis.stability_score > 0.6 {
        println!("  • Moderate stability issues");
        println!("  • Consider optimizations");
        println!("  • Monitor high-churn files");
    } else {
        println!("  • Significant stability issues");
        println!("  • Prioritize contract optimization");
        println!("  • Implement memoization strategy");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Contract Stability Analyzer");
    println!("=============================");
    println!("Analyzing Git object stability for contract optimization...");
    println!();

    // Analyze contract stability
    let analysis = analyze_contract_stability()?;

    // Generate comprehensive report
    generate_contract_stability_report(&analysis);

    println!("\n✅ Contract stability analysis complete!");
    println!("🔒 Stability issues identified");
    println!("📊 SHA churn analyzed");
    println!("💡 Recommendations ready");
    println!("🎯 Optimization strategy prepared");

    Ok(())
}
