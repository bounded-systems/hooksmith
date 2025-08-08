use std::collections::HashMap;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone)]
struct RepoSizeMetrics {
    git_directory_size: u64,
    working_directory_size: u64,
    total_tracked_files: u32,
    total_blobs: u32,
    total_commits: u32,
    largest_file_size: u64,
    largest_file_path: String,
    crate_count: u32,
    deep_tree_depth: u32,
    packfile_count: u32,
    packfile_size: u64,
}

#[derive(Debug, Clone)]
struct SizeThreshold {
    name: String,
    current: u64,
    threshold: u64,
    unit: String,
    status: ThresholdStatus,
    recommendation: String,
}

#[derive(Debug, Clone)]
enum ThresholdStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug)]
struct RepoSizeAnalysis {
    metrics: RepoSizeMetrics,
    thresholds: Vec<SizeThreshold>,
    recommendations: Vec<String>,
    health_score: f64,
    split_candidates: Vec<String>,
    optimization_plan: Vec<String>,
}

fn analyze_repository_size() -> Result<RepoSizeAnalysis, Box<dyn std::error::Error>> {
    println!("📏 Analyzing repository size and health...");
    
    // Collect metrics
    let metrics = collect_repo_metrics()?;
    
    // Check thresholds
    let thresholds = check_size_thresholds(&metrics)?;
    
    // Generate recommendations
    let recommendations = generate_size_recommendations(&metrics, &thresholds);
    let health_score = calculate_health_score(&thresholds);
    let split_candidates = identify_split_candidates(&metrics)?;
    let optimization_plan = generate_optimization_plan(&metrics, &thresholds);
    
    Ok(RepoSizeAnalysis {
        metrics,
        thresholds,
        recommendations,
        health_score,
        split_candidates,
        optimization_plan,
    })
}

fn collect_repo_metrics() -> Result<RepoSizeMetrics, Box<dyn std::error::Error>> {
    // Git directory size
    let git_size = get_git_directory_size()?;
    
    // Working directory size
    let working_size = get_working_directory_size()?;
    
    // Total tracked files
    let tracked_files = count_tracked_files()?;
    
    // Total blobs
    let total_blobs = count_total_blobs()?;
    
    // Total commits
    let total_commits = count_total_commits()?;
    
    // Largest file
    let (largest_size, largest_path) = find_largest_file()?;
    
    // Crate count
    let crate_count = count_crates()?;
    
    // Deep tree depth
    let deep_tree_depth = calculate_deep_tree_depth()?;
    
    // Packfile info
    let (packfile_count, packfile_size) = get_packfile_info()?;
    
    Ok(RepoSizeMetrics {
        git_directory_size: git_size,
        working_directory_size: working_size,
        total_tracked_files: tracked_files,
        total_blobs,
        total_commits,
        largest_file_size: largest_size,
        largest_file_path: largest_path,
        crate_count,
        deep_tree_depth,
        packfile_count,
        packfile_size,
    })
}

fn get_git_directory_size() -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("du")
        .args(&["-sh", ".git"])
        .output()?;
    
    let size_str = String::from_utf8(output.stdout)?;
    let size_parts: Vec<&str> = size_str.split_whitespace().collect();
    
    if let Some(size_part) = size_parts.first() {
        parse_size_string(size_part)
    } else {
        Ok(0)
    }
}

fn get_working_directory_size() -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("du")
        .args(&["-sh", "."])
        .output()?;
    
    let size_str = String::from_utf8(output.stdout)?;
    let size_parts: Vec<&str> = size_str.split_whitespace().collect();
    
    if let Some(size_part) = size_parts.first() {
        parse_size_string(size_part)
    } else {
        Ok(0)
    }
}

fn parse_size_string(size_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let size_str = size_str.trim();
    
    if size_str.ends_with('K') {
        let num = size_str[..size_str.len()-1].parse::<f64>()?;
        Ok((num * 1024.0) as u64)
    } else if size_str.ends_with('M') {
        let num = size_str[..size_str.len()-1].parse::<f64>()?;
        Ok((num * 1024.0 * 1024.0) as u64)
    } else if size_str.ends_with('G') {
        let num = size_str[..size_str.len()-1].parse::<f64>()?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
    } else {
        Ok(size_str.parse::<u64>().unwrap_or(0))
    }
}

fn count_tracked_files() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-files"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    Ok(files.lines().count() as u32)
}

fn count_total_blobs() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let objects = String::from_utf8(output.stdout)?;
    let blob_count = objects.lines()
        .filter(|line| !line.contains(' ')) // Blobs don't have spaces
        .count();
    
    Ok(blob_count as u32)
}

fn count_total_commits() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output()?;
    
    let count_str = String::from_utf8(output.stdout)?;
    Ok(count_str.trim().parse().unwrap_or(0))
}

fn find_largest_file() -> Result<(u64, String), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut largest_size = 0u64;
    let mut largest_path = String::new();
    
    for line in files.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3];
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", "HEAD:"])
                .arg(path)
                .output();
            
            if let Ok(size_output) = size_output {
                let size_str = String::from_utf8(size_output.stdout)?;
                if let Ok(size) = size_str.trim().parse::<u64>() {
                    if size > largest_size {
                        largest_size = size;
                        largest_path = path.to_string();
                    }
                }
            }
        }
    }
    
    Ok((largest_size, largest_path))
}

fn count_crates() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;
    
    let cargo_files = String::from_utf8(output.stdout)?;
    Ok(cargo_files.lines().count() as u32)
}

fn calculate_deep_tree_depth() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut max_depth = 0u32;
    
    for line in files.lines() {
        let depth = line.matches('/').count() as u32;
        if depth > max_depth {
            max_depth = depth;
        }
    }
    
    Ok(max_depth)
}

fn get_packfile_info() -> Result<(u32, u64), Box<dyn std::error::Error>> {
    let pack_dir = Path::new(".git/objects/pack");
    if !pack_dir.exists() {
        return Ok((0, 0));
    }
    
    let output = Command::new("find")
        .args(&[".git/objects/pack", "-name", "*.pack", "-type", "f"])
        .output()?;
    
    let pack_files = String::from_utf8(output.stdout)?;
    let pack_count = pack_files.lines().count() as u32;
    
    let mut total_size = 0u64;
    for pack_file in pack_files.lines() {
        if let Ok(metadata) = std::fs::metadata(pack_file) {
            total_size += metadata.len();
        }
    }
    
    Ok((pack_count, total_size))
}

fn check_size_thresholds(metrics: &RepoSizeMetrics) -> Result<Vec<SizeThreshold>, Box<dyn std::error::Error>> {
    let mut thresholds = Vec::new();
    
    // Git directory size threshold (300 MB)
    let git_size_mb = metrics.git_directory_size as f64 / (1024.0 * 1024.0);
    let git_status = if git_size_mb < 300.0 { ThresholdStatus::Healthy } 
                    else if git_size_mb < 500.0 { ThresholdStatus::Warning }
                    else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Git Directory Size".to_string(),
        current: metrics.git_directory_size,
        threshold: 300 * 1024 * 1024, // 300 MB
        unit: "MB".to_string(),
        status: git_status.clone(),
        recommendation: if git_size_mb > 300.0 {
            "Consider git gc --aggressive and git prune".to_string()
        } else {
            "Git directory size is healthy".to_string()
        },
    });
    
    // Working directory size threshold (200 MB)
    let working_size_mb = metrics.working_directory_size as f64 / (1024.0 * 1024.0);
    let working_status = if working_size_mb < 200.0 { ThresholdStatus::Healthy }
                        else if working_size_mb < 500.0 { ThresholdStatus::Warning }
                        else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Working Directory Size".to_string(),
        current: metrics.working_directory_size,
        threshold: 200 * 1024 * 1024, // 200 MB
        unit: "MB".to_string(),
        status: working_status.clone(),
        recommendation: if working_size_mb > 200.0 {
            "Consider splitting large files or using Git LFS".to_string()
        } else {
            "Working directory size is healthy".to_string()
        },
    });
    
    // Total tracked files threshold (5,000)
    let files_status = if metrics.total_tracked_files < 5000 { ThresholdStatus::Healthy }
                      else if metrics.total_tracked_files < 10000 { ThresholdStatus::Warning }
                      else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Total Tracked Files".to_string(),
        current: metrics.total_tracked_files as u64,
        threshold: 5000,
        unit: "files".to_string(),
        status: files_status.clone(),
        recommendation: if metrics.total_tracked_files > 5000 {
            "Consider .gitignore for generated files".to_string()
        } else {
            "File count is healthy".to_string()
        },
    });
    
    // Largest file size threshold (100 KB for .rs, 1 MB for assets)
    let largest_file_mb = metrics.largest_file_size as f64 / (1024.0 * 1024.0);
    let largest_status = if largest_file_mb < 1.0 { ThresholdStatus::Healthy }
                        else if largest_file_mb < 10.0 { ThresholdStatus::Warning }
                        else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Largest File Size".to_string(),
        current: metrics.largest_file_size,
        threshold: 1024 * 1024, // 1 MB
        unit: "MB".to_string(),
        status: largest_status.clone(),
        recommendation: if largest_file_mb > 1.0 {
            format!("Consider Git LFS for large file: {}", metrics.largest_file_path)
        } else {
            "File sizes are healthy".to_string()
        },
    });
    
    // Crate count threshold (50)
    let crate_status = if metrics.crate_count < 50 { ThresholdStatus::Healthy }
                      else if metrics.crate_count < 100 { ThresholdStatus::Warning }
                      else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Crate Count".to_string(),
        current: metrics.crate_count as u64,
        threshold: 50,
        unit: "crates".to_string(),
        status: crate_status.clone(),
        recommendation: if metrics.crate_count > 50 {
            "Consider splitting workspace into multiple repos".to_string()
        } else {
            "Crate count is healthy".to_string()
        },
    });
    
    // Deep tree depth threshold (10 levels)
    let depth_status = if metrics.deep_tree_depth < 10 { ThresholdStatus::Healthy }
                      else if metrics.deep_tree_depth < 15 { ThresholdStatus::Warning }
                      else { ThresholdStatus::Critical };
    
    thresholds.push(SizeThreshold {
        name: "Deep Tree Depth".to_string(),
        current: metrics.deep_tree_depth as u64,
        threshold: 10,
        unit: "levels".to_string(),
        status: depth_status.clone(),
        recommendation: if metrics.deep_tree_depth > 10 {
            "Consider flattening directory structure".to_string()
        } else {
            "Directory depth is healthy".to_string()
        },
    });
    
    Ok(thresholds)
}

fn generate_size_recommendations(metrics: &RepoSizeMetrics, thresholds: &[SizeThreshold]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let critical_count = thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Critical)).count();
    let warning_count = thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Warning)).count();
    
    if critical_count > 0 {
        recommendations.push("Critical size issues detected - immediate action required".to_string());
    }
    
    if warning_count > 3 {
        recommendations.push("Multiple size warnings - consider repository optimization".to_string());
    }
    
    if metrics.git_directory_size > 300 * 1024 * 1024 {
        recommendations.push("Large .git directory - run git gc --aggressive".to_string());
    }
    
    if metrics.largest_file_size > 1024 * 1024 {
        recommendations.push("Large files detected - implement Git LFS".to_string());
    }
    
    if metrics.crate_count > 50 {
        recommendations.push("Many crates - consider splitting workspace".to_string());
    }
    
    if metrics.deep_tree_depth > 10 {
        recommendations.push("Deep directory structure - consider flattening".to_string());
    }
    
    recommendations.push("Use .gitattributes for binary file handling".to_string());
    recommendations.push("Implement proper .gitignore for build artifacts".to_string());
    recommendations.push("Consider shallow clones for CI/CD".to_string());
    
    recommendations
}

fn calculate_health_score(thresholds: &[SizeThreshold]) -> f64 {
    if thresholds.is_empty() {
        return 1.0;
    }
    
    let total_thresholds = thresholds.len() as f64;
    let healthy_count = thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Healthy)).count() as f64;
    let warning_count = thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Warning)).count() as f64;
    let critical_count = thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Critical)).count() as f64;
    
    let score = (healthy_count + warning_count * 0.5) / total_thresholds;
    score - (critical_count * 0.3 / total_thresholds)
}

fn identify_split_candidates(metrics: &RepoSizeMetrics) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut candidates = Vec::new();
    
    if metrics.crate_count > 30 {
        candidates.push("High crate count - consider splitting workspace".to_string());
    }
    
    if metrics.git_directory_size > 200 * 1024 * 1024 {
        candidates.push("Large .git directory - consider repository split".to_string());
    }
    
    if metrics.total_tracked_files > 3000 {
        candidates.push("Many tracked files - consider modularization".to_string());
    }
    
    // Check for large crates that could be extracted
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;
    
    let cargo_files = String::from_utf8(output.stdout)?;
    for cargo_file in cargo_files.lines() {
        if let Some(crate_dir) = Path::new(cargo_file).parent() {
            if let Ok(metadata) = std::fs::metadata(crate_dir) {
                let dir_size = metadata.len();
                if dir_size > 10 * 1024 * 1024 { // 10MB
                    if let Some(crate_name) = crate_dir.file_name() {
                        candidates.push(format!("Large crate: {} ({} MB)", 
                            crate_name.to_string_lossy(), dir_size / (1024 * 1024)));
                    }
                }
            }
        }
    }
    
    Ok(candidates)
}

fn generate_optimization_plan(metrics: &RepoSizeMetrics, thresholds: &[SizeThreshold]) -> Vec<String> {
    let mut plan = Vec::new();
    
    let critical_thresholds: Vec<_> = thresholds.iter()
        .filter(|t| matches!(t.status, ThresholdStatus::Critical))
        .collect();
    
    if !critical_thresholds.is_empty() {
        plan.push("Phase 1 - Critical Issues (Immediate):".to_string());
        for threshold in critical_thresholds {
            plan.push(format!("  • {}", threshold.recommendation));
        }
    }
    
    let warning_thresholds: Vec<_> = thresholds.iter()
        .filter(|t| matches!(t.status, ThresholdStatus::Warning))
        .collect();
    
    if !warning_thresholds.is_empty() {
        plan.push("Phase 2 - Warning Issues (Short-term):".to_string());
        for threshold in warning_thresholds {
            plan.push(format!("  • {}", threshold.recommendation));
        }
    }
    
    plan.push("Phase 3 - Maintenance (Ongoing):".to_string());
    plan.push("  • Run git gc --aggressive monthly".to_string());
    plan.push("  • Monitor .gitignore for new build artifacts".to_string());
    plan.push("  • Review large files quarterly".to_string());
    plan.push("  • Consider shallow clones for CI/CD".to_string());
    
    plan
}

fn generate_repository_size_report(analysis: &RepoSizeAnalysis) {
    println!("\n📏 Repository Size Audit");
    println!("=========================");
    
    // Show metrics
    println!("\n📊 Repository Metrics:");
    println!("  • Git directory size: {:.1} MB", 
        analysis.metrics.git_directory_size as f64 / (1024.0 * 1024.0));
    println!("  • Working directory size: {:.1} MB", 
        analysis.metrics.working_directory_size as f64 / (1024.0 * 1024.0));
    println!("  • Total tracked files: {}", analysis.metrics.total_tracked_files);
    println!("  • Total blobs: {}", analysis.metrics.total_blobs);
    println!("  • Total commits: {}", analysis.metrics.total_commits);
    println!("  • Crate count: {}", analysis.metrics.crate_count);
    println!("  • Deep tree depth: {}", analysis.metrics.deep_tree_depth);
    println!("  • Packfiles: {} ({:.1} MB)", 
        analysis.metrics.packfile_count,
        analysis.metrics.packfile_size as f64 / (1024.0 * 1024.0));
    
    if analysis.metrics.largest_file_size > 0 {
        println!("  • Largest file: {} ({:.1} MB)", 
            analysis.metrics.largest_file_path,
            analysis.metrics.largest_file_size as f64 / (1024.0 * 1024.0));
    }
    
    // Show thresholds
    println!("\n🎯 Size Thresholds:");
    for threshold in &analysis.thresholds {
        let status_icon = match threshold.status {
            ThresholdStatus::Healthy => "🟢",
            ThresholdStatus::Warning => "🟡",
            ThresholdStatus::Critical => "🔴",
        };
        
        let current_value = if threshold.unit == "MB" {
            format!("{:.1} MB", threshold.current as f64 / (1024.0 * 1024.0))
        } else {
            format!("{} {}", threshold.current, threshold.unit)
        };
        
        let threshold_value = if threshold.unit == "MB" {
            format!("{:.1} MB", threshold.threshold as f64 / (1024.0 * 1024.0))
        } else {
            format!("{} {}", threshold.threshold, threshold.unit)
        };
        
        println!("  {} {}: {} (threshold: {})", 
            status_icon, threshold.name, current_value, threshold_value);
        println!("    {}", threshold.recommendation);
    }
    
    // Show health score
    println!("\n📈 Health Score:");
    println!("  • Overall health: {:.1}%", analysis.health_score * 100.0);
    
    let healthy_count = analysis.thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Healthy)).count();
    let warning_count = analysis.thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Warning)).count();
    let critical_count = analysis.thresholds.iter().filter(|t| matches!(t.status, ThresholdStatus::Critical)).count();
    
    println!("  • Healthy thresholds: {}", healthy_count);
    println!("  • Warning thresholds: {}", warning_count);
    println!("  • Critical thresholds: {}", critical_count);
    
    // Show split candidates
    if !analysis.split_candidates.is_empty() {
        println!("\n🔀 Split Candidates:");
        for candidate in &analysis.split_candidates {
            println!("  • {}", candidate);
        }
    }
    
    // Show recommendations
    if !analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &analysis.recommendations {
            println!("  • {}", rec);
        }
    }
    
    // Show optimization plan
    if !analysis.optimization_plan.is_empty() {
        println!("\n🔧 Optimization Plan:");
        for step in &analysis.optimization_plan {
            println!("  {}", step);
        }
    }
    
    // Summary
    println!("\n📋 Summary:");
    if analysis.health_score > 0.8 {
        println!("  • Repository size is healthy");
        println!("  • Good performance expected");
        println!("  • Continue monitoring");
    } else if analysis.health_score > 0.6 {
        println!("  • Repository size needs attention");
        println!("  • Consider optimizations");
        println!("  • Monitor growth trends");
    } else {
        println!("  • Repository size is problematic");
        println!("  • Immediate action required");
        println!("  • Consider splitting or cleanup");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📏 Repository Size Auditor");
    println!("==========================");
    println!("Analyzing repository size and health...");
    println!();
    
    // Analyze repository size
    let analysis = analyze_repository_size()?;
    
    // Generate comprehensive report
    generate_repository_size_report(&analysis);
    
    println!("\n✅ Repository size audit complete!");
    println!("📏 Size metrics collected");
    println!("🎯 Thresholds evaluated");
    println!("💡 Recommendations ready");
    println!("🔧 Optimization plan prepared");
    
    Ok(())
}
