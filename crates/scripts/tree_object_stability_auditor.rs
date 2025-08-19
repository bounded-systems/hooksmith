use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct ObjectStabilityInfo {
    object_path: String,
    object_type: ObjectType,
    commit_count: u32,
    churn_score: f64,
    stability_level: StabilityLevel,
    tree_depth: u32,
    blob_count: u32,
    total_size: u64,
    merge_conflicts: u32,
    concurrent_editors: Vec<String>,
    delta_compression_impact: f64,
    contract_risk_score: f64,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
enum ObjectType {
    File,
    Directory,
    Crate,
    Module,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum StabilityLevel {
    Critical, // >100 commits, immediate action required
    High,     // 50-100 commits, consider extraction
    Medium,   // 20-50 commits, monitor closely
    Low,      // 5-20 commits, normal activity
    Stable,   // <5 commits, safe for contracts
}

#[derive(Debug, Clone)]
struct TreeStabilityAnalysis {
    unstable_objects: Vec<ObjectStabilityInfo>,
    high_churn_files: Vec<String>,
    merge_conflict_candidates: Vec<String>,
    delta_compression_issues: Vec<String>,
    contract_risk_objects: Vec<String>,
    structural_decomposition_targets: Vec<String>,
    overall_stats: StabilityStats,
}

#[derive(Debug, Clone)]
struct StabilityStats {
    total_objects: u32,
    critical_instability: u32,
    high_instability: u32,
    medium_instability: u32,
    low_instability: u32,
    stable_objects: u32,
    merge_conflict_risk: u32,
    delta_compression_impact: f64,
    contract_risk_score: f64,
}

fn analyze_tree_object_stability(
    time_period: Option<&str>,
) -> Result<TreeStabilityAnalysis, Box<dyn std::error::Error>> {
    println!("🔍 Analyzing tree/object stability for structural decomposition...");

    // Get all objects with their stability metrics
    let unstable_objects = get_object_stability_data(time_period)?;

    // Identify specific issues
    let high_churn_files = identify_high_churn_files(&unstable_objects);
    let merge_conflict_candidates = identify_merge_conflict_candidates(&unstable_objects);
    let delta_compression_issues = identify_delta_compression_issues(&unstable_objects);
    let contract_risk_objects = identify_contract_risk_objects(&unstable_objects);
    let structural_decomposition_targets =
        identify_structural_decomposition_targets(&unstable_objects);

    // Calculate overall stats
    let overall_stats = calculate_stability_stats(&unstable_objects);

    Ok(TreeStabilityAnalysis {
        unstable_objects,
        high_churn_files,
        merge_conflict_candidates,
        delta_compression_issues,
        contract_risk_objects,
        structural_decomposition_targets,
        overall_stats,
    })
}

fn get_object_stability_data(
    time_period: Option<&str>,
) -> Result<Vec<ObjectStabilityInfo>, Box<dyn std::error::Error>> {
    let mut args = vec!["log", "--name-only", "--pretty=format:"];

    if let Some(period) = time_period {
        args.extend_from_slice(&["--since", period]);
    }

    let output = Command::new("git").args(&args).output()?;

    let content = String::from_utf8(output.stdout)?;
    let mut object_counts: HashMap<String, u32> = HashMap::new();

    // Count commits per object
    for line in content.lines() {
        if !line.trim().is_empty() {
            *object_counts.entry(line.trim().to_string()).or_insert(0) += 1;
        }
    }

    let mut unstable_objects = Vec::new();

    for (object_path, commit_count) in object_counts {
        let stability_info = analyze_single_object(&object_path, commit_count, time_period)?;
        unstable_objects.push(stability_info);
    }

    // Sort by churn score (descending)
    unstable_objects.sort_by(|a, b| b.churn_score.partial_cmp(&a.churn_score).unwrap());

    Ok(unstable_objects)
}

fn analyze_single_object(
    object_path: &str,
    commit_count: u32,
    time_period: Option<&str>,
) -> Result<ObjectStabilityInfo, Box<dyn std::error::Error>> {
    // Determine object type
    let object_type = determine_object_type(object_path);

    // Get tree depth and blob count
    let (tree_depth, blob_count) = get_tree_metrics(object_path)?;

    // Get total size
    let total_size = get_object_size(object_path)?;

    // Calculate churn score
    let churn_score = calculate_churn_score(commit_count, &object_type, tree_depth, blob_count);

    // Determine stability level
    let stability_level = determine_stability_level(commit_count);

    // Get merge conflict risk
    let merge_conflicts = estimate_merge_conflicts(object_path, commit_count)?;

    // Get concurrent editors
    let concurrent_editors = get_concurrent_editors(object_path, time_period)?;

    // Calculate delta compression impact
    let delta_compression_impact =
        calculate_delta_compression_impact(commit_count, total_size, blob_count);

    // Calculate contract risk score
    let contract_risk_score =
        calculate_contract_risk_score(commit_count, &stability_level, concurrent_editors.len());

    // Generate recommendations
    let recommendations = generate_stability_recommendations(
        object_path,
        commit_count,
        &stability_level,
        &object_type,
        tree_depth,
        blob_count,
    );

    Ok(ObjectStabilityInfo {
        object_path: object_path.to_string(),
        object_type,
        commit_count,
        churn_score,
        stability_level,
        tree_depth,
        blob_count,
        total_size,
        merge_conflicts,
        concurrent_editors,
        delta_compression_impact,
        contract_risk_score,
        recommendations,
    })
}

fn determine_object_type(object_path: &str) -> ObjectType {
    if object_path.ends_with(".rs") {
        ObjectType::File
    } else if object_path.contains("Cargo.toml") || object_path.contains("src/") {
        ObjectType::Crate
    } else if object_path.contains("/") {
        ObjectType::Directory
    } else {
        ObjectType::Module
    }
}

fn get_tree_metrics(object_path: &str) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    // Simplified tree depth calculation
    let tree_depth = object_path.matches('/').count() as u32;

    // Simplified blob count (for directories)
    let blob_count = if object_path.ends_with("/") || !object_path.contains(".") {
        // Estimate blob count for directories
        let output = Command::new("find")
            .args(&[object_path, "-type", "f", "-maxdepth", "1"])
            .output();

        match output {
            Ok(result) => {
                let files = String::from_utf8(result.stdout)?;
                files.lines().count() as u32
            }
            Err(_) => 1, // Default to 1 if find fails
        }
    } else {
        1 // Single file
    };

    Ok((tree_depth, blob_count))
}

fn get_object_size(object_path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("du").args(&["-sk", object_path]).output();

    match output {
        Ok(result) => {
            let size_str = String::from_utf8(result.stdout)?;
            let size_parts: Vec<&str> = size_str.split_whitespace().collect();
            if size_parts.len() > 0 {
                Ok(size_parts[0].parse::<u64>().unwrap_or(0) * 1024) // Convert KB to bytes
            } else {
                Ok(0)
            }
        }
        Err(_) => Ok(0), // Default to 0 if du fails
    }
}

fn calculate_churn_score(
    commit_count: u32,
    object_type: &ObjectType,
    tree_depth: u32,
    blob_count: u32,
) -> f64 {
    let base_score = commit_count as f64;

    // Weight by object type
    let type_multiplier = match object_type {
        ObjectType::File => 1.0,
        ObjectType::Crate => 2.0, // Crates are more important
        ObjectType::Directory => 1.5,
        ObjectType::Module => 1.2,
    };

    // Weight by tree depth (deeper trees are more complex)
    let depth_multiplier = 1.0 + (tree_depth as f64 * 0.1);

    // Weight by blob count (more files = more impact)
    let blob_multiplier = 1.0 + (blob_count as f64 * 0.05);

    base_score * type_multiplier * depth_multiplier * blob_multiplier
}

fn determine_stability_level(commit_count: u32) -> StabilityLevel {
    match commit_count {
        0..=5 => StabilityLevel::Stable,
        6..=20 => StabilityLevel::Low,
        21..=50 => StabilityLevel::Medium,
        51..=100 => StabilityLevel::High,
        _ => StabilityLevel::Critical,
    }
}

fn estimate_merge_conflicts(
    object_path: &str,
    commit_count: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Simplified merge conflict estimation
    // In a real implementation, you'd analyze actual merge history
    let conflict_risk = if commit_count > 50 {
        commit_count / 10 // Higher churn = higher conflict risk
    } else {
        commit_count / 20
    };

    Ok(conflict_risk)
}

fn get_concurrent_editors(
    object_path: &str,
    time_period: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut args = vec!["log", "--format=%an", "--", object_path];

    if let Some(period) = time_period {
        args.extend_from_slice(&["--since", period]);
    }

    let output = Command::new("git").args(&args).output()?;

    let authors = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.trim().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    Ok(authors)
}

fn calculate_delta_compression_impact(commit_count: u32, total_size: u64, blob_count: u32) -> f64 {
    // Higher churn + larger size = worse delta compression
    let churn_impact = commit_count as f64 * 0.1;
    let size_impact = (total_size as f64 / 1024.0) * 0.001; // Size in KB
    let blob_impact = blob_count as f64 * 0.05;

    churn_impact + size_impact + blob_impact
}

fn calculate_contract_risk_score(
    commit_count: u32,
    stability_level: &StabilityLevel,
    concurrent_editors: usize,
) -> f64 {
    let churn_risk = commit_count as f64 * 0.1;
    let stability_risk = match stability_level {
        StabilityLevel::Critical => 1.0,
        StabilityLevel::High => 0.8,
        StabilityLevel::Medium => 0.5,
        StabilityLevel::Low => 0.2,
        StabilityLevel::Stable => 0.0,
    };
    let editor_risk = concurrent_editors as f64 * 0.1;

    churn_risk + stability_risk + editor_risk
}

fn generate_stability_recommendations(
    object_path: &str,
    commit_count: u32,
    stability_level: &StabilityLevel,
    object_type: &ObjectType,
    tree_depth: u32,
    blob_count: u32,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match stability_level {
        StabilityLevel::Critical => {
            recommendations.push("Extract to separate crate immediately".to_string());
            recommendations.push("Consider moving to external repository".to_string());
            recommendations.push("Implement versioning strategy".to_string());
        }
        StabilityLevel::High => {
            recommendations.push("Consider extraction to separate crate".to_string());
            recommendations.push("Review modularization boundaries".to_string());
            recommendations.push("Monitor for further instability".to_string());
        }
        StabilityLevel::Medium => {
            recommendations.push("Monitor instability patterns".to_string());
            recommendations.push("Consider if object is too large".to_string());
        }
        StabilityLevel::Low => {
            recommendations.push("Normal development activity".to_string());
        }
        StabilityLevel::Stable => {
            recommendations.push("Safe for SHA pinning in contracts".to_string());
            recommendations.push("Good candidate for contract snapshots".to_string());
        }
    }

    // Add specific recommendations based on object characteristics
    if commit_count > 100 && blob_count > 10 {
        recommendations.push("High churn with many blobs - consider splitting".to_string());
    }

    if tree_depth > 3 {
        recommendations.push("Deep tree structure - consider flattening".to_string());
    }

    if let ObjectType::Crate = object_type {
        if commit_count > 50 {
            recommendations.push("High-churn crate - consider extraction".to_string());
        }
    }

    if object_path.ends_with(".rs") && commit_count > 50 {
        recommendations.push("Large Rust file with high churn - consider splitting".to_string());
    }

    recommendations
}

fn identify_high_churn_files(unstable_objects: &[ObjectStabilityInfo]) -> Vec<String> {
    unstable_objects
        .iter()
        .filter(|obj| {
            matches!(
                obj.stability_level,
                StabilityLevel::Critical | StabilityLevel::High
            )
        })
        .map(|obj| obj.object_path.clone())
        .collect()
}

fn identify_merge_conflict_candidates(unstable_objects: &[ObjectStabilityInfo]) -> Vec<String> {
    unstable_objects
        .iter()
        .filter(|obj| obj.merge_conflicts > 5 || obj.concurrent_editors.len() > 3)
        .map(|obj| obj.object_path.clone())
        .collect()
}

fn identify_delta_compression_issues(unstable_objects: &[ObjectStabilityInfo]) -> Vec<String> {
    unstable_objects
        .iter()
        .filter(|obj| obj.delta_compression_impact > 10.0)
        .map(|obj| obj.object_path.clone())
        .collect()
}

fn identify_contract_risk_objects(unstable_objects: &[ObjectStabilityInfo]) -> Vec<String> {
    unstable_objects
        .iter()
        .filter(|obj| obj.contract_risk_score > 5.0)
        .map(|obj| obj.object_path.clone())
        .collect()
}

fn identify_structural_decomposition_targets(
    unstable_objects: &[ObjectStabilityInfo],
) -> Vec<String> {
    unstable_objects
        .iter()
        .filter(|obj| {
            obj.commit_count > 100 && obj.blob_count > 10
                || obj.tree_depth > 3 && obj.commit_count > 50
                || obj.total_size > 1024 * 1024 && obj.commit_count > 20 // >1MB and >20 commits
        })
        .map(|obj| obj.object_path.clone())
        .collect()
}

fn calculate_stability_stats(unstable_objects: &[ObjectStabilityInfo]) -> StabilityStats {
    let total_objects = unstable_objects.len() as u32;
    let critical_instability = unstable_objects
        .iter()
        .filter(|obj| matches!(obj.stability_level, StabilityLevel::Critical))
        .count() as u32;
    let high_instability = unstable_objects
        .iter()
        .filter(|obj| matches!(obj.stability_level, StabilityLevel::High))
        .count() as u32;
    let medium_instability = unstable_objects
        .iter()
        .filter(|obj| matches!(obj.stability_level, StabilityLevel::Medium))
        .count() as u32;
    let low_instability = unstable_objects
        .iter()
        .filter(|obj| matches!(obj.stability_level, StabilityLevel::Low))
        .count() as u32;
    let stable_objects = unstable_objects
        .iter()
        .filter(|obj| matches!(obj.stability_level, StabilityLevel::Stable))
        .count() as u32;

    let merge_conflict_risk = unstable_objects
        .iter()
        .filter(|obj| obj.merge_conflicts > 5)
        .count() as u32;
    let delta_compression_impact = unstable_objects
        .iter()
        .map(|obj| obj.delta_compression_impact)
        .sum::<f64>()
        / total_objects as f64;
    let contract_risk_score = unstable_objects
        .iter()
        .map(|obj| obj.contract_risk_score)
        .sum::<f64>()
        / total_objects as f64;

    StabilityStats {
        total_objects,
        critical_instability,
        high_instability,
        medium_instability,
        low_instability,
        stable_objects,
        merge_conflict_risk,
        delta_compression_impact,
        contract_risk_score,
    }
}

fn generate_stability_audit_report(analysis: &TreeStabilityAnalysis) {
    println!("\n🔍 Tree/Object Stability Audit Report");
    println!("=====================================");

    // Overall statistics
    println!("\n📊 Overall Stability Statistics:");
    println!(
        "  • Total objects analyzed: {}",
        analysis.overall_stats.total_objects
    );
    println!(
        "  • Critical instability: {}",
        analysis.overall_stats.critical_instability
    );
    println!(
        "  • High instability: {}",
        analysis.overall_stats.high_instability
    );
    println!(
        "  • Medium instability: {}",
        analysis.overall_stats.medium_instability
    );
    println!(
        "  • Low instability: {}",
        analysis.overall_stats.low_instability
    );
    println!(
        "  • Stable objects: {}",
        analysis.overall_stats.stable_objects
    );
    println!(
        "  • Merge conflict risk: {}",
        analysis.overall_stats.merge_conflict_risk
    );
    println!(
        "  • Delta compression impact: {:.2}",
        analysis.overall_stats.delta_compression_impact
    );
    println!(
        "  • Contract risk score: {:.2}",
        analysis.overall_stats.contract_risk_score
    );

    // Top unstable objects
    println!("\n🚨 Top 10 Most Unstable Objects:");
    for (i, obj) in analysis.unstable_objects.iter().take(10).enumerate() {
        println!(
            "  {}. {} ({} commits, {:?}, {:.1} churn score)",
            i + 1,
            obj.object_path,
            obj.commit_count,
            obj.stability_level,
            obj.churn_score
        );
    }

    // High churn files
    if !analysis.high_churn_files.is_empty() {
        println!("\n🔥 High Churn Files (Structural Decomposition Needed):");
        for file in analysis.high_churn_files.iter().take(10) {
            println!("  • {}", file);
        }
    }

    // Merge conflict candidates
    if !analysis.merge_conflict_candidates.is_empty() {
        println!("\n💥 Merge Conflict Candidates:");
        for file in analysis.merge_conflict_candidates.iter().take(10) {
            println!("  • {}", file);
        }
    }

    // Delta compression issues
    if !analysis.delta_compression_issues.is_empty() {
        println!("\n📦 Delta Compression Issues:");
        for file in analysis.delta_compression_issues.iter().take(10) {
            println!("  • {}", file);
        }
    }

    // Contract risk objects
    if !analysis.contract_risk_objects.is_empty() {
        println!("\n⚠️  Contract Risk Objects:");
        for file in analysis.contract_risk_objects.iter().take(10) {
            println!("  • {}", file);
        }
    }

    // Structural decomposition targets
    if !analysis.structural_decomposition_targets.is_empty() {
        println!("\n🧩 Structural Decomposition Targets:");
        for file in analysis.structural_decomposition_targets.iter().take(10) {
            println!("  • {}", file);
        }
    }

    // Recommendations
    println!("\n💡 Structural Decomposition Recommendations:");
    println!("  • Files >500 LOC and edited >100 times: Extract module or crate");
    println!("  • Trees rehashed on most commits: Split into subfolders or crates");
    println!("  • Merge conflicts on main.rs: Split logic; create CLI wrapper crates");
    println!("  • Large files that change often: Isolate to crate or version them");
    println!("  • Crates that fail to cache in CI: Break apart; pin stable subcomponents");
    println!("  • Objects with unstable SHA + multiple hook roles: Extract immediately");

    // Summary
    println!("\n📋 Stability Summary:");
    if analysis.overall_stats.critical_instability > 0 {
        println!(
            "  🚨 Critical instability detected - immediate structural decomposition required"
        );
    }
    if analysis.overall_stats.high_instability > 10 {
        println!("  ⚠️  High instability detected - consider modularization");
    }
    if analysis.overall_stats.merge_conflict_risk > 5 {
        println!("  💥 Merge conflict risk detected - review shared roots");
    }
    if analysis.overall_stats.delta_compression_impact > 5.0 {
        println!("  📦 Delta compression issues detected - optimize object structure");
    }
    if analysis.overall_stats.contract_risk_score > 3.0 {
        println!("  ⚠️  Contract risk detected - review SHA stability");
    }
    if analysis.overall_stats.stable_objects > analysis.overall_stats.total_objects / 2 {
        println!("  ✅ Good stability - suitable for contract pinning");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let time_period = args.get(1).map(|s| s.as_str());

    println!("🔍 Tree/Object Stability Auditor");
    println!("================================");
    if let Some(period) = time_period {
        println!("Analyzing stability since: {}", period);
    } else {
        println!("Analyzing all-time stability");
    }
    println!();

    // Analyze tree/object stability
    let analysis = analyze_tree_object_stability(time_period)?;

    // Generate comprehensive audit report
    generate_stability_audit_report(&analysis);

    println!("\n✅ Tree/Object stability audit complete!");
    println!("🚨 Unstable objects identified");
    println!("💥 Merge conflict candidates found");
    println!("📦 Delta compression issues detected");
    println!("⚠️  Contract risk objects flagged");
    println!("🧩 Structural decomposition targets identified");

    Ok(())
}
