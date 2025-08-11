use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
struct TreeStabilityInfo {
    tree_path: String,
    tree_sha: String,
    file_count: u32,
    total_size: u64,
    change_frequency: u32,
    tree_churn_score: f64,
    stability_level: TreeStabilityLevel,
    volatile_files: Vec<String>,
    stable_files: Vec<String>,
    subtree_impact: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum TreeStabilityLevel {
    Stable,
    SemiVolatile,
    Volatile,
    Critical,
}

#[derive(Debug, Clone)]
struct TreeChurnAnalysis {
    tree_sha: String,
    churn_count: u32,
    affected_files: Vec<String>,
    parent_impact: bool,
    subtree_cascade: Vec<String>,
}

#[derive(Debug)]
struct TreeStabilityAnalysis {
    trees: Vec<TreeStabilityInfo>,
    tree_churn_map: HashMap<String, TreeChurnAnalysis>,
    memoization_recommendations: Vec<String>,
    tree_structure_optimizations: Vec<String>,
    overall_stability_score: f64,
}

fn analyze_tree_stability() -> Result<TreeStabilityAnalysis, Box<dyn std::error::Error>> {
    println!("🌳 Analyzing Git tree object stability...");
    
    // Get all trees in the repository
    let output = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;
    
    let files_output = String::from_utf8(output.stdout)?;
    let mut tree_groups: HashMap<String, Vec<String>> = HashMap::new();
    
    // Group files by directory (tree)
    for line in files_output.lines() {
        let path = line.trim();
        if !path.is_empty() {
            let tree_path = extract_tree_path(path);
            tree_groups.entry(tree_path).or_default().push(path.to_string());
        }
    }
    
    let mut trees = Vec::new();
    let mut tree_churn_map = HashMap::new();
    
    // Analyze each tree
    for (tree_path, files) in tree_groups {
        let tree_analysis = analyze_single_tree(&tree_path, &files)?;
        trees.push(tree_analysis);
        
        // Analyze tree churn
        if let Some(churn_analysis) = analyze_tree_churn(&tree_path)? {
            tree_churn_map.insert(tree_path.clone(), churn_analysis);
        }
    }
    
    // Sort by stability (most stable first)
    trees.sort_by(|a, b| a.stability_level.partial_cmp(&b.stability_level).unwrap());
    
    // Generate recommendations
    let memoization_recommendations = generate_memoization_recommendations(&trees);
    let tree_structure_optimizations = generate_tree_structure_optimizations(&trees);
    let overall_stability_score = calculate_overall_stability_score(&trees);
    
    Ok(TreeStabilityAnalysis {
        trees,
        tree_churn_map,
        memoization_recommendations,
        tree_structure_optimizations,
        overall_stability_score,
    })
}

fn extract_tree_path(file_path: &str) -> String {
    let path_parts: Vec<&str> = file_path.split('/').collect();
    
    if path_parts.len() <= 1 {
        "root".to_string()
    } else {
        path_parts[0].to_string()
    }
}

fn analyze_single_tree(tree_path: &str, files: &[String]) -> Result<TreeStabilityInfo, Box<dyn std::error::Error>> {
    let file_count = files.len() as u32;
    let mut total_size = 0u64;
    let mut volatile_files = Vec::new();
    let mut stable_files = Vec::new();
    
    // Analyze each file in the tree
    for file in files {
        let size = get_file_size(file)?;
        total_size += size;
        
        let change_frequency = get_change_frequency(file)?;
        let churn_score = calculate_file_churn(file)?;
        
        if change_frequency > 5 || churn_score > 0.6 {
            volatile_files.push(file.clone());
        } else {
            stable_files.push(file.clone());
        }
    }
    
    // Calculate tree churn score
    let tree_churn_score = calculate_tree_churn_score(tree_path)?;
    
    // Determine stability level
    let stability_level = determine_tree_stability_level(
        file_count,
        volatile_files.len(),
        tree_churn_score,
        tree_path,
    );
    
    // Calculate subtree impact
    let subtree_impact = calculate_subtree_impact(tree_path, files)?;
    
    // Get tree SHA (simplified - in real implementation, you'd get the actual tree SHA)
    let tree_sha = format!("tree_{}", tree_path.replace('/', "_"));
    
    Ok(TreeStabilityInfo {
        tree_path: tree_path.to_string(),
        tree_sha,
        file_count,
        total_size,
        change_frequency: volatile_files.len() as u32,
        tree_churn_score,
        stability_level,
        volatile_files,
        stable_files,
        subtree_impact,
    })
}

fn get_file_size(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["cat-file", "-s", "HEAD:"])
        .arg(path)
        .output()?;
    
    let size_str = String::from_utf8(output.stdout)?;
    Ok(size_str.trim().parse().unwrap_or(0))
}

fn get_change_frequency(path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--follow", "--", path])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().count() as u32)
}

fn calculate_file_churn(path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Simplified churn calculation
    let change_frequency = get_change_frequency(path)?;
    
    if change_frequency == 0 {
        Ok(0.0)
    } else {
        // Normalize to 0-1 scale
        Ok((change_frequency as f64 / 10.0).min(1.0))
    }
}

fn calculate_tree_churn_score(tree_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Get all files in this tree
    let output = Command::new("git")
        .args(&["ls-tree", "--name-only", "HEAD"])
        .arg(tree_path)
        .output()?;
    
    let files_output = String::from_utf8(output.stdout)?;
    let files: Vec<&str> = files_output.lines().collect();
    
    if files.is_empty() {
        return Ok(0.0);
    }
    
    let mut total_churn = 0.0;
    let mut file_count = 0;
    
    for file in files {
        if let Ok(churn) = calculate_file_churn(file) {
            total_churn += churn;
            file_count += 1;
        }
    }
    
    if file_count > 0 {
        Ok(total_churn / file_count as f64)
    } else {
        Ok(0.0)
    }
}

fn determine_tree_stability_level(
    file_count: u32,
    volatile_count: usize,
    churn_score: f64,
    tree_path: &str,
) -> TreeStabilityLevel {
    let volatile_ratio = volatile_count as f64 / file_count.max(1) as f64;
    
    // Root tree gets special consideration
    if tree_path == "root" {
        if volatile_ratio > 0.3 || churn_score > 0.7 {
            return TreeStabilityLevel::Critical;
        } else if volatile_ratio > 0.1 || churn_score > 0.5 {
            return TreeStabilityLevel::Volatile;
        }
    }
    
    // Regular trees
    if volatile_ratio > 0.5 || churn_score > 0.8 {
        TreeStabilityLevel::Critical
    } else if volatile_ratio > 0.3 || churn_score > 0.6 {
        TreeStabilityLevel::Volatile
    } else if volatile_ratio > 0.1 || churn_score > 0.3 {
        TreeStabilityLevel::SemiVolatile
    } else {
        TreeStabilityLevel::Stable
    }
}

fn calculate_subtree_impact(tree_path: &str, files: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut impact = Vec::new();
    
    // Check if this tree affects parent trees
    if tree_path != "root" {
        let parent_tree = extract_parent_tree(tree_path);
        if parent_tree != tree_path {
            impact.push(format!("Affects parent tree: {}", parent_tree));
        }
    }
    
    // Check for volatile files that could cascade
    for file in files {
        let change_frequency = get_change_frequency(file)?;
        if change_frequency > 10 {
            impact.push(format!("High-churn file: {} ({} changes)", file, change_frequency));
        }
    }
    
    Ok(impact)
}

fn extract_parent_tree(tree_path: &str) -> String {
    let parts: Vec<&str> = tree_path.split('/').collect();
    if parts.len() > 1 {
        parts[..parts.len() - 1].join("/")
    } else {
        "root".to_string()
    }
}

fn analyze_tree_churn(tree_path: &str) -> Result<Option<TreeChurnAnalysis>, Box<dyn std::error::Error>> {
    // Get recent commits that affected this tree
    let output = Command::new("git")
        .args(&["log", "--oneline", "--name-only", "--", tree_path])
        .output()?;
    
    let log_output = String::from_utf8(output.stdout)?;
    let commits: Vec<&str> = log_output.lines().collect();
    
    if commits.len() < 2 {
        return Ok(None);
    }
    
    let churn_count = commits.len() as u32 / 2; // Rough estimate
    let affected_files = extract_affected_files(&log_output);
    let parent_impact = tree_path != "root";
    let subtree_cascade = calculate_cascade_effect(tree_path)?;
    
    Ok(Some(TreeChurnAnalysis {
        tree_sha: format!("tree_{}", tree_path.replace('/', "_")),
        churn_count,
        affected_files,
        parent_impact,
        subtree_cascade,
    }))
}

fn extract_affected_files(log_output: &str) -> Vec<String> {
    let mut files = Vec::new();
    
    for line in log_output.lines() {
        if !line.starts_with("commit") && !line.is_empty() && !line.contains(' ') {
            files.push(line.to_string());
        }
    }
    
    files
}

fn calculate_cascade_effect(tree_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut cascade = Vec::new();
    
    // Check if this tree affects other trees
    if tree_path.contains("hooks") {
        cascade.push("Affects hook execution".to_string());
    }
    
    if tree_path.contains("src") {
        cascade.push("Affects compilation".to_string());
    }
    
    if tree_path == "root" {
        cascade.push("Affects entire repository".to_string());
    }
    
    Ok(cascade)
}

fn generate_memoization_recommendations(trees: &[TreeStabilityInfo]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let stable_trees = trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Stable)).count();
    let volatile_trees = trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Volatile | TreeStabilityLevel::Critical)).count();
    
    if stable_trees > 5 {
        recommendations.push("Multiple stable trees detected - implement tree SHA memoization".to_string());
    }
    
    if volatile_trees > 3 {
        recommendations.push("High tree volatility - use subtree isolation strategies".to_string());
    }
    
    // Specific recommendations for each stability level
    for tree in trees {
        match tree.stability_level {
            TreeStabilityLevel::Stable => {
                recommendations.push(format!("Memoize fix plans for tree: {} (SHA: {})", tree.tree_path, tree.tree_sha));
            }
            TreeStabilityLevel::SemiVolatile => {
                recommendations.push(format!("Use conditional memoization for tree: {}", tree.tree_path));
            }
            TreeStabilityLevel::Volatile => {
                recommendations.push(format!("Isolate volatile files in tree: {} to prevent cascade", tree.tree_path));
            }
            TreeStabilityLevel::Critical => {
                recommendations.push(format!("Critical tree: {} - implement aggressive isolation", tree.tree_path));
            }
        }
    }
    
    recommendations.push("Use tree SHA as contract scope identifier".to_string());
    recommendations.push("Implement tree-level fix plan caching".to_string());
    
    recommendations
}

fn generate_tree_structure_optimizations(trees: &[TreeStabilityInfo]) -> Vec<String> {
    let mut optimizations = Vec::new();
    
    for tree in trees {
        if tree.volatile_files.len() > tree.stable_files.len() {
            optimizations.push(format!("Split volatile files from tree: {} into separate subtree", tree.tree_path));
        }
        
        if tree.tree_path == "root" && tree.volatile_files.len() > 0 {
            optimizations.push("Move volatile files from root to dedicated subtrees".to_string());
        }
        
        if tree.file_count > 20 {
            optimizations.push(format!("Large tree: {} - consider splitting into smaller subtrees", tree.tree_path));
        }
    }
    
    optimizations.push("Use deep, narrow folder structures to localize churn".to_string());
    optimizations.push("Avoid filename collisions across modules".to_string());
    optimizations.push("Keep root tree stable by isolating volatile files".to_string());
    
    optimizations
}

fn calculate_overall_stability_score(trees: &[TreeStabilityInfo]) -> f64 {
    if trees.is_empty() {
        return 1.0;
    }
    
    let total_trees = trees.len() as f64;
    let stable_trees = trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Stable)).count() as f64;
    let critical_trees = trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Critical)).count() as f64;
    
    let base_score = stable_trees / total_trees;
    let penalty = critical_trees / total_trees * 0.5;
    
    (base_score - penalty).max(0.0)
}

fn generate_tree_stability_report(analysis: &TreeStabilityAnalysis) {
    println!("\n🌳 Git Tree Stability Analysis");
    println!("=============================");
    
    // Show tree stability levels
    println!("\n📊 Tree Stability Overview:");
    for tree in &analysis.trees {
        let stability_icon = match tree.stability_level {
            TreeStabilityLevel::Stable => "🟢",
            TreeStabilityLevel::SemiVolatile => "🟡",
            TreeStabilityLevel::Volatile => "🟠",
            TreeStabilityLevel::Critical => "🔴",
        };
        
        println!("  {} {} ({} files, {:.1}% churn)", 
            stability_icon, tree.tree_path, tree.file_count, tree.tree_churn_score * 100.0);
        
        if !tree.volatile_files.is_empty() {
            println!("    Volatile files: {}", tree.volatile_files.len());
        }
        
        if !tree.subtree_impact.is_empty() {
            println!("    Impact: {}", tree.subtree_impact.join(", "));
        }
        println!();
    }
    
    // Show tree churn analysis
    if !analysis.tree_churn_map.is_empty() {
        println!("\n🔄 Tree Churn Analysis:");
        for (tree_path, churn) in &analysis.tree_churn_map {
            println!("  • {}: {} changes", tree_path, churn.churn_count);
            if churn.parent_impact {
                println!("    Affects parent trees");
            }
            if !churn.subtree_cascade.is_empty() {
                println!("    Cascade: {}", churn.subtree_cascade.join(", "));
            }
        }
    }
    
    // Show stability metrics
    println!("\n📈 Stability Metrics:");
    println!("  • Overall stability score: {:.1}%", analysis.overall_stability_score * 100.0);
    println!("  • Total trees: {}", analysis.trees.len());
    
    let stable_count = analysis.trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Stable)).count();
    let critical_count = analysis.trees.iter().filter(|t| matches!(t.stability_level, TreeStabilityLevel::Critical)).count();
    
    println!("  • Stable trees: {}", stable_count);
    println!("  • Critical trees: {}", critical_count);
    
    // Show memoization recommendations
    if !analysis.memoization_recommendations.is_empty() {
        println!("\n💡 Memoization Recommendations:");
        for rec in &analysis.memoization_recommendations {
            println!("  • {}", rec);
        }
    }
    
    // Show tree structure optimizations
    if !analysis.tree_structure_optimizations.is_empty() {
        println!("\n🔧 Tree Structure Optimizations:");
        for opt in &analysis.tree_structure_optimizations {
            println!("  • {}", opt);
        }
    }
    
    // Summary
    println!("\n📋 Summary:");
    if analysis.overall_stability_score > 0.8 {
        println!("  • Tree structure is stable");
        println!("  • Good memoization potential");
        println!("  • Efficient contract caching");
    } else if analysis.overall_stability_score > 0.6 {
        println!("  • Moderate tree stability");
        println!("  • Consider optimizations");
        println!("  • Monitor critical trees");
    } else {
        println!("  • Significant tree instability");
        println!("  • Prioritize tree structure optimization");
        println!("  • Implement aggressive isolation");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Git Tree Stability Analyzer");
    println!("=============================");
    println!("Analyzing tree object stability for contract optimization...");
    println!();
    
    // Analyze tree stability
    let analysis = analyze_tree_stability()?;
    
    // Generate comprehensive report
    generate_tree_stability_report(&analysis);
    
    println!("\n✅ Tree stability analysis complete!");
    println!("🌳 Tree objects analyzed");
    println!("🔄 Churn patterns identified");
    println!("💡 Memoization strategies ready");
    println!("🔧 Structure optimizations prepared");
    
    Ok(())
}
