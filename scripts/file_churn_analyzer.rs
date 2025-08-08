use std::collections::HashMap;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone)]
struct FileChurnInfo {
    file_path: String,
    commit_count: u32,
    last_modified: String,
    first_modified: String,
    authors: Vec<String>,
    churn_score: f64,
    churn_category: ChurnCategory,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
enum ChurnCategory {
    Critical,    // > 100 commits - extract immediately
    High,        // 50-100 commits - consider extraction
    Medium,      // 20-50 commits - monitor
    Low,         // 5-20 commits - normal
    Stable,      // < 5 commits - safe for SHA pinning
}

#[derive(Debug, Clone)]
struct DirectoryChurnInfo {
    directory: String,
    total_files: u32,
    churning_files: u32,
    total_commits: u32,
    average_churn: f64,
    churn_score: f64,
    extraction_candidates: Vec<String>,
    modularization_score: f64,
}

#[derive(Debug, Clone)]
struct ChurnAnalysis {
    file_churn: Vec<FileChurnInfo>,
    directory_churn: Vec<DirectoryChurnInfo>,
    hot_files: Vec<String>,
    extraction_candidates: Vec<String>,
    gitignore_candidates: Vec<String>,
    lfs_candidates: Vec<String>,
    contract_stable_files: Vec<String>,
    overall_stats: ChurnStats,
}

#[derive(Debug, Clone)]
struct ChurnStats {
    total_files: u32,
    total_commits: u32,
    critical_churn_files: u32,
    high_churn_files: u32,
    medium_churn_files: u32,
    low_churn_files: u32,
    stable_files: u32,
    average_churn_per_file: f64,
}

fn analyze_file_churn(time_period: Option<&str>) -> Result<ChurnAnalysis, Box<dyn std::error::Error>> {
    println!("🔍 Analyzing file churn patterns...");
    
    // Get all files with their commit counts
    let file_churn = get_file_churn_data(time_period)?;
    
    // Analyze directory-level churn
    let directory_churn = analyze_directory_churn(&file_churn)?;
    
    // Identify hot files and extraction candidates
    let hot_files = identify_hot_files(&file_churn);
    let extraction_candidates = identify_extraction_candidates(&file_churn, &directory_churn);
    let gitignore_candidates = identify_gitignore_candidates(&file_churn);
    let lfs_candidates = identify_lfs_candidates(&file_churn);
    let contract_stable_files = identify_contract_stable_files(&file_churn);
    
    // Calculate overall stats
    let overall_stats = calculate_churn_stats(&file_churn);
    
    Ok(ChurnAnalysis {
        file_churn,
        directory_churn,
        hot_files,
        extraction_candidates,
        gitignore_candidates,
        lfs_candidates,
        contract_stable_files,
        overall_stats,
    })
}

fn get_file_churn_data(time_period: Option<&str>) -> Result<Vec<FileChurnInfo>, Box<dyn std::error::Error>> {
    let mut args = vec!["log", "--name-only", "--pretty=format:"];
    
    if let Some(period) = time_period {
        args.extend_from_slice(&["--since", period]);
    }
    
    let output = Command::new("git")
        .args(&args)
        .output()?;
    
    let content = String::from_utf8(output.stdout)?;
    let mut file_counts: HashMap<String, u32> = HashMap::new();
    
    // Count commits per file
    for line in content.lines() {
        if !line.trim().is_empty() {
            *file_counts.entry(line.trim().to_string()).or_insert(0) += 1;
        }
    }
    
    let mut file_churn = Vec::new();
    
    for (file_path, commit_count) in file_counts {
        let churn_info = analyze_single_file(&file_path, commit_count, time_period)?;
        file_churn.push(churn_info);
    }
    
    // Sort by commit count (descending)
    file_churn.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
    
    Ok(file_churn)
}

fn analyze_single_file(file_path: &str, commit_count: u32, time_period: Option<&str>) -> Result<FileChurnInfo, Box<dyn std::error::Error>> {
    // Get last modified date
    let last_modified = get_last_modified(file_path)?;
    
    // Get first modified date
    let first_modified = get_first_modified(file_path)?;
    
    // Get authors
    let authors = get_file_authors(file_path, time_period)?;
    
    // Calculate churn score
    let churn_score = calculate_churn_score(commit_count, &last_modified, &first_modified);
    
    // Determine churn category
    let churn_category = determine_churn_category(commit_count);
    
    // Generate recommendations
    let recommendations = generate_file_recommendations(file_path, commit_count, &churn_category);
    
    Ok(FileChurnInfo {
        file_path: file_path.to_string(),
        commit_count,
        last_modified,
        first_modified,
        authors,
        churn_score,
        churn_category,
        recommendations,
    })
}

fn get_last_modified(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short", "--", file_path])
        .output()?;
    
    let date = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(date)
}

fn get_first_modified(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--reverse", "--format=%cd", "--date=short", "--", file_path])
        .output()?;
    
    let date = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(date)
}

fn get_file_authors(file_path: &str, time_period: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut args = vec!["log", "--format=%an", "--", file_path];
    
    if let Some(period) = time_period {
        args.extend_from_slice(&["--since", period]);
    }
    
    let output = Command::new("git")
        .args(&args)
        .output()?;
    
    let authors = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.trim().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    Ok(authors)
}

fn calculate_churn_score(commit_count: u32, last_modified: &str, first_modified: &str) -> f64 {
    // Simple churn score based on commit count
    // In a real implementation, you might consider time period, file size, etc.
    commit_count as f64
}

fn determine_churn_category(commit_count: u32) -> ChurnCategory {
    match commit_count {
        0..=5 => ChurnCategory::Stable,
        6..=20 => ChurnCategory::Low,
        21..=50 => ChurnCategory::Medium,
        51..=100 => ChurnCategory::High,
        _ => ChurnCategory::Critical,
    }
}

fn generate_file_recommendations(file_path: &str, commit_count: u32, category: &ChurnCategory) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    match category {
        ChurnCategory::Critical => {
            recommendations.push("Extract to separate crate immediately".to_string());
            recommendations.push("Consider moving to external repository".to_string());
            recommendations.push("Add to .gitignore if generated content".to_string());
        }
        ChurnCategory::High => {
            recommendations.push("Consider extraction to separate crate".to_string());
            recommendations.push("Review modularization boundaries".to_string());
            recommendations.push("Monitor for further churn increase".to_string());
        }
        ChurnCategory::Medium => {
            recommendations.push("Monitor churn patterns".to_string());
            recommendations.push("Consider if file is too large".to_string());
        }
        ChurnCategory::Low => {
            recommendations.push("Normal development activity".to_string());
        }
        ChurnCategory::Stable => {
            recommendations.push("Safe for SHA pinning in contracts".to_string());
            recommendations.push("Good candidate for contract snapshots".to_string());
        }
    }
    
    // Add specific recommendations based on file type
    if file_path.ends_with(".rs") && commit_count > 50 {
        recommendations.push("Consider splitting large Rust file".to_string());
    }
    
    if file_path.contains("target/") || file_path.contains("build/") {
        recommendations.push("Add to .gitignore".to_string());
    }
    
    if file_path.ends_with(".exe") || file_path.ends_with(".dll") || file_path.ends_with(".so") {
        recommendations.push("Consider Git LFS tracking".to_string());
    }
    
    recommendations
}

fn analyze_directory_churn(file_churn: &[FileChurnInfo]) -> Result<Vec<DirectoryChurnInfo>, Box<dyn std::error::Error>> {
    let mut directory_stats: HashMap<String, Vec<&FileChurnInfo>> = HashMap::new();
    
    // Group files by directory
    for file_info in file_churn {
        if let Some(dir) = Path::new(&file_info.file_path).parent() {
            let dir_str = dir.to_string_lossy().to_string();
            directory_stats.entry(dir_str).or_insert_with(Vec::new).push(file_info);
        }
    }
    
    let mut directory_churn = Vec::new();
    
    for (directory, files) in directory_stats {
        let total_files = files.len() as u32;
        let churning_files = files.iter().filter(|f| matches!(f.churn_category, ChurnCategory::High | ChurnCategory::Critical)).count() as u32;
        let total_commits: u32 = files.iter().map(|f| f.commit_count).sum();
        let average_churn = total_commits as f64 / total_files as f64;
        
        let churn_score = calculate_directory_churn_score(&files);
        let extraction_candidates = identify_directory_extraction_candidates(&files);
        let modularization_score = calculate_modularization_score(&files);
        
        directory_churn.push(DirectoryChurnInfo {
            directory,
            total_files,
            churning_files,
            total_commits,
            average_churn,
            churn_score,
            extraction_candidates,
            modularization_score,
        });
    }
    
    // Sort by churn score (descending)
    directory_churn.sort_by(|a, b| b.churn_score.partial_cmp(&a.churn_score).unwrap());
    
    Ok(directory_churn)
}

fn calculate_directory_churn_score(files: &[&FileChurnInfo]) -> f64 {
    let total_commits: u32 = files.iter().map(|f| f.commit_count).sum();
    let critical_files = files.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Critical)).count();
    let high_files = files.iter().filter(|f| matches!(f.churn_category, ChurnCategory::High)).count();
    
    // Weight critical files more heavily
    (total_commits as f64) + (critical_files as f64 * 100.0) + (high_files as f64 * 50.0)
}

fn identify_directory_extraction_candidates(files: &[&FileChurnInfo]) -> Vec<String> {
    files.iter()
        .filter(|f| matches!(f.churn_category, ChurnCategory::High | ChurnCategory::Critical))
        .map(|f| f.file_path.clone())
        .collect()
}

fn calculate_modularization_score(files: &[&FileChurnInfo]) -> f64 {
    let total_files = files.len() as f64;
    let churning_files = files.iter()
        .filter(|f| matches!(f.churn_category, ChurnCategory::High | ChurnCategory::Critical))
        .count() as f64;
    
    // Lower score = better modularization (fewer churning files)
    (total_files - churning_files) / total_files
}

fn identify_hot_files(file_churn: &[FileChurnInfo]) -> Vec<String> {
    file_churn.iter()
        .filter(|f| matches!(f.churn_category, ChurnCategory::Critical | ChurnCategory::High))
        .map(|f| f.file_path.clone())
        .collect()
}

fn identify_extraction_candidates(file_churn: &[FileChurnInfo], directory_churn: &[DirectoryChurnInfo]) -> Vec<String> {
    let mut candidates = Vec::new();
    
    // Add files with critical churn
    for file in file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Critical)) {
        candidates.push(file.file_path.clone());
    }
    
    // Add directories with poor modularization scores
    for dir in directory_churn.iter().filter(|d| d.modularization_score < 0.5) {
        candidates.extend(dir.extraction_candidates.clone());
    }
    
    candidates
}

fn identify_gitignore_candidates(file_churn: &[FileChurnInfo]) -> Vec<String> {
    file_churn.iter()
        .filter(|f| {
            f.file_path.contains("target/") ||
            f.file_path.contains("build/") ||
            f.file_path.contains(".cache") ||
            f.file_path.ends_with(".log") ||
            f.file_path.ends_with(".tmp")
        })
        .map(|f| f.file_path.clone())
        .collect()
}

fn identify_lfs_candidates(file_churn: &[FileChurnInfo]) -> Vec<String> {
    file_churn.iter()
        .filter(|f| {
            f.file_path.ends_with(".exe") ||
            f.file_path.ends_with(".dll") ||
            f.file_path.ends_with(".so") ||
            f.file_path.ends_with(".dylib") ||
            f.file_path.ends_with(".zip") ||
            f.file_path.ends_with(".tar.gz")
        })
        .map(|f| f.file_path.clone())
        .collect()
}

fn identify_contract_stable_files(file_churn: &[FileChurnInfo]) -> Vec<String> {
    file_churn.iter()
        .filter(|f| matches!(f.churn_category, ChurnCategory::Stable | ChurnCategory::Low))
        .map(|f| f.file_path.clone())
        .collect()
}

fn calculate_churn_stats(file_churn: &[FileChurnInfo]) -> ChurnStats {
    let total_files = file_churn.len() as u32;
    let total_commits: u32 = file_churn.iter().map(|f| f.commit_count).sum();
    let average_churn_per_file = total_commits as f64 / total_files as f64;
    
    let critical_churn_files = file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Critical)).count() as u32;
    let high_churn_files = file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::High)).count() as u32;
    let medium_churn_files = file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Medium)).count() as u32;
    let low_churn_files = file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Low)).count() as u32;
    let stable_files = file_churn.iter().filter(|f| matches!(f.churn_category, ChurnCategory::Stable)).count() as u32;
    
    ChurnStats {
        total_files,
        total_commits,
        critical_churn_files,
        high_churn_files,
        medium_churn_files,
        low_churn_files,
        stable_files,
        average_churn_per_file,
    }
}

fn generate_churn_report(analysis: &ChurnAnalysis) {
    println!("\n🔍 File Churn Analysis Report");
    println!("=============================");
    
    // Overall statistics
    println!("\n📊 Overall Statistics:");
    println!("  • Total files analyzed: {}", analysis.overall_stats.total_files);
    println!("  • Total commits: {}", analysis.overall_stats.total_commits);
    println!("  • Average churn per file: {:.1}", analysis.overall_stats.average_churn_per_file);
    println!("  • Critical churn files: {}", analysis.overall_stats.critical_churn_files);
    println!("  • High churn files: {}", analysis.overall_stats.high_churn_files);
    println!("  • Medium churn files: {}", analysis.overall_stats.medium_churn_files);
    println!("  • Low churn files: {}", analysis.overall_stats.low_churn_files);
    println!("  • Stable files: {}", analysis.overall_stats.stable_files);
    
    // Top churning files
    println!("\n🔥 Top 10 Most Churning Files:");
    for (i, file) in analysis.file_churn.iter().take(10).enumerate() {
        println!("  {}. {} ({} commits, {:?})", 
            i + 1, file.file_path, file.commit_count, file.churn_category);
    }
    
    // Directory analysis
    println!("\n📁 Directory Churn Analysis:");
    for dir in analysis.directory_churn.iter().take(10) {
        println!("  • {}: {} files, {:.1} avg churn, {:.1}% modularization", 
            dir.directory, dir.total_files, dir.average_churn, dir.modularization_score * 100.0);
    }
    
    // Hot files
    if !analysis.hot_files.is_empty() {
        println!("\n🚨 Hot Files (High/Critical Churn):");
        for file in analysis.hot_files.iter().take(10) {
            println!("  • {}", file);
        }
    }
    
    // Extraction candidates
    if !analysis.extraction_candidates.is_empty() {
        println!("\n📦 Extraction Candidates:");
        for file in analysis.extraction_candidates.iter().take(10) {
            println!("  • {}", file);
        }
    }
    
    // Gitignore candidates
    if !analysis.gitignore_candidates.is_empty() {
        println!("\n🚫 .gitignore Candidates:");
        for file in analysis.gitignore_candidates.iter().take(10) {
            println!("  • {}", file);
        }
    }
    
    // LFS candidates
    if !analysis.lfs_candidates.is_empty() {
        println!("\n💾 Git LFS Candidates:");
        for file in analysis.lfs_candidates.iter().take(10) {
            println!("  • {}", file);
        }
    }
    
    // Contract stable files
    if !analysis.contract_stable_files.is_empty() {
        println!("\n🔒 Contract-Stable Files:");
        for file in analysis.contract_stable_files.iter().take(10) {
            println!("  • {}", file);
        }
    }
    
    // Recommendations
    println!("\n💡 Recommendations:");
    println!("  • Files with >100 commits: Extract to separate crates");
    println!("  • Files with 50-100 commits: Consider extraction");
    println!("  • Directories with <50% modularization: Review boundaries");
    println!("  • Stable files: Safe for SHA pinning in contracts");
    println!("  • Generated files: Add to .gitignore");
    println!("  • Binary files: Consider Git LFS tracking");
    
    // Summary
    println!("\n📋 Summary:");
    if analysis.overall_stats.critical_churn_files > 0 {
        println!("  🚨 Critical churn detected - immediate action required");
    }
    if analysis.overall_stats.high_churn_files > 10 {
        println!("  ⚠️  High churn detected - consider modularization");
    }
    if analysis.overall_stats.stable_files > analysis.overall_stats.total_files / 2 {
        println!("  ✅ Good stability - suitable for contract pinning");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let time_period = args.get(1).map(|s| s.as_str());
    
    println!("🔍 File Churn Analyzer");
    println!("======================");
    if let Some(period) = time_period {
        println!("Analyzing churn since: {}", period);
    } else {
        println!("Analyzing all-time churn");
    }
    println!();
    
    // Analyze file churn
    let analysis = analyze_file_churn(time_period)?;
    
    // Generate comprehensive report
    generate_churn_report(&analysis);
    
    println!("\n✅ File churn analysis complete!");
    println!("🔍 Hot files identified");
    println!("📦 Extraction candidates found");
    println!("🚫 .gitignore candidates suggested");
    println!("💾 LFS candidates identified");
    println!("🔒 Contract-stable files identified");
    
    Ok(())
}
