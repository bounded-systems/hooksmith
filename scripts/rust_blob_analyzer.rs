use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct RustBlobInfo {
    path: String,
    blob_size: u64,
    line_count: Option<u32>,
    complexity_score: f64,
    performance_impact: String,
    optimization_recommendation: String,
}

#[derive(Debug, Clone)]
struct RustModuleAnalysis {
    module_path: String,
    total_blob_size: u64,
    file_count: u32,
    average_blob_size: f64,
    largest_file: String,
    largest_file_size: u64,
    optimization_opportunities: Vec<String>,
}

#[derive(Debug)]
struct RustBlobAnalysis {
    rust_files: Vec<RustBlobInfo>,
    modules: Vec<RustModuleAnalysis>,
    total_blob_size: u64,
    performance_concerns: Vec<String>,
    optimization_recommendations: Vec<String>,
}

fn analyze_rust_blobs() -> Result<Vec<RustBlobInfo>, Box<dyn std::error::Error>> {
    println!("🦀 Analyzing Rust file blob sizes...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut rust_files = Vec::new();
    
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1];
            
            // Focus on Rust files
            if path.ends_with(".rs") {
                // Get blob size using git cat-file
                let size_output = Command::new("git")
                    .args(&["cat-file", "-s", hash])
                    .output()?;
                
                if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                    if let Ok(blob_size) = u64::from_str(size_str.trim()) {
                        let line_count = get_line_count(&path);
                        let complexity_score = calculate_complexity_score(blob_size, line_count);
                        let performance_impact = assess_performance_impact(blob_size);
                        let optimization_recommendation = generate_optimization_recommendation(&path, blob_size, complexity_score);
                        
                        rust_files.push(RustBlobInfo {
                            path,
                            blob_size,
                            line_count,
                            complexity_score,
                            performance_impact,
                            optimization_recommendation,
                        });
                    }
                }
            }
        }
    }
    
    println!("🦀 Found {} Rust files", rust_files.len());
    Ok(rust_files)
}

fn get_line_count(path: &str) -> Option<u32> {
    // Try to get line count using wc -l
    let output = Command::new("wc")
        .args(&["-l", path])
        .output();
    
    if let Ok(output) = output {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            if let Some(line) = output_str.lines().next() {
                if let Some(count_str) = line.split_whitespace().next() {
                    if let Ok(count) = u32::from_str(count_str) {
                        return Some(count);
                    }
                }
            }
        }
    }
    
    None
}

fn calculate_complexity_score(blob_size: u64, line_count: Option<u32>) -> f64 {
    let size_factor = (blob_size as f64 / 1024.0).min(10.0); // Cap at 10 KB factor
    
    let line_factor = if let Some(lines) = line_count {
        (lines as f64 / 100.0).min(5.0) // Cap at 500 lines factor
    } else {
        1.0
    };
    
    size_factor * line_factor
}

fn assess_performance_impact(blob_size: u64) -> String {
    match blob_size {
        0..=1024 => "Minimal".to_string(),
        1025..=10240 => "Low".to_string(),
        10241..=102400 => "Moderate".to_string(),
        102401..=1024000 => "High".to_string(),
        _ => "Critical".to_string(),
    }
}

fn generate_optimization_recommendation(path: &str, blob_size: u64, complexity_score: f64) -> String {
    let path_lower = path.to_lowercase();
    
    if blob_size > 100 * 1024 { // 100 KB
        if path_lower.contains("main.rs") {
            "Consider splitting main.rs into smaller modules".to_string()
        } else if path_lower.contains("lib.rs") {
            "Consider breaking lib.rs into focused modules".to_string()
        } else {
            "Consider splitting large file into smaller modules".to_string()
        }
    } else if complexity_score > 5.0 {
        "High complexity - consider refactoring into smaller functions".to_string()
    } else if blob_size < 1024 {
        "Small file - good for incremental compilation".to_string()
    } else {
        "File size is reasonable for Git and compilation".to_string()
    }
}

fn analyze_rust_modules(rust_files: &[RustBlobInfo]) -> Vec<RustModuleAnalysis> {
    println!("📦 Analyzing Rust modules...");
    
    let mut module_groups: HashMap<String, Vec<&RustBlobInfo>> = HashMap::new();
    
    // Group files by module path
    for file in rust_files {
        let module_path = get_module_path(&file.path);
        module_groups.entry(module_path).or_insert_with(Vec::new).push(file);
    }
    
    let mut modules = Vec::new();
    
    for (module_path, files) in module_groups {
        let total_blob_size: u64 = files.iter().map(|f| f.blob_size).sum();
        let file_count = files.len() as u32;
        let average_blob_size = total_blob_size as f64 / file_count as f64;
        
        // Find largest file
        let largest_file = files.iter()
            .max_by_key(|f| f.blob_size)
            .unwrap();
        
        let optimization_opportunities = generate_module_optimizations(&module_path, &files);
        
        modules.push(RustModuleAnalysis {
            module_path,
            total_blob_size,
            file_count,
            average_blob_size,
            largest_file: largest_file.path.clone(),
            largest_file_size: largest_file.blob_size,
            optimization_opportunities,
        });
    }
    
    println!("📦 Found {} Rust modules", modules.len());
    modules
}

fn get_module_path(file_path: &str) -> String {
    // Extract module path from file path
    if let Some(slash_pos) = file_path.rfind('/') {
        if slash_pos > 0 {
            return file_path[..slash_pos].to_string();
        }
    }
    
    // If no slash, it's in root
    "root".to_string()
}

fn generate_module_optimizations(_module_path: &str, files: &[&RustBlobInfo]) -> Vec<String> {
    let mut opportunities = Vec::new();
    
    let total_size: u64 = files.iter().map(|f| f.blob_size).sum();
    let file_count = files.len();
    
    if total_size > 500 * 1024 { // 500 KB
        opportunities.push("Large module - consider splitting into submodules".to_string());
    }
    
    if file_count > 10 {
        opportunities.push("Many files - consider organizing into subdirectories".to_string());
    }
    
    let large_files: Vec<&&RustBlobInfo> = files.iter()
        .filter(|f| f.blob_size > 50 * 1024) // 50 KB
        .collect();
    
    if !large_files.is_empty() {
        opportunities.push(format!("{} large files - consider refactoring", large_files.len()));
    }
    
    opportunities
}

fn generate_rust_blob_report(
    rust_files: &[RustBlobInfo],
    modules: &[RustModuleAnalysis],
) -> RustBlobAnalysis {
    println!("\n🦀 Rust Blob Analysis Report");
    println!("=============================");
    
    let total_blob_size: u64 = rust_files.iter().map(|f| f.blob_size).sum();
    let mut performance_concerns = Vec::new();
    let mut optimization_recommendations = Vec::new();
    
    // Analyze blob size distribution
    let mut size_categories = HashMap::new();
    for file in rust_files {
        let category = match file.blob_size {
            0..=1024 => "tiny",
            1025..=10240 => "small",
            10241..=102400 => "medium",
            102401..=1024000 => "large",
            _ => "huge",
        };
        *size_categories.entry(category).or_insert(0) += 1;
    }
    
    println!("\n📊 Blob Size Distribution:");
    for (category, count) in size_categories {
        println!("  • {}: {} files", category, count);
    }
    
    // Show largest files
    let mut sorted_files = rust_files.to_vec();
    sorted_files.sort_by(|a, b| b.blob_size.cmp(&a.blob_size));
    
    println!("\n🔍 Largest Rust Files (Top 10):");
    for file in sorted_files.iter().take(10) {
        println!("  • {} ({} bytes) - {} impact", 
            file.path, file.blob_size, file.performance_impact);
        println!("    Recommendation: {}", file.optimization_recommendation);
    }
    
    // Show module analysis
    if !modules.is_empty() {
        println!("\n📦 Module Analysis:");
        
        // Sort by total blob size
        let mut sorted_modules = modules.to_vec();
        sorted_modules.sort_by(|a, b| b.total_blob_size.cmp(&a.total_blob_size));
        
        for module in sorted_modules.iter().take(5) {
            println!("  • {} ({} files, {:.1} KB total)", 
                module.module_path, module.file_count, module.total_blob_size as f64 / 1024.0);
            println!("    Largest: {} ({} bytes)", module.largest_file, module.largest_file_size);
            for opt in &module.optimization_opportunities {
                println!("    - {}", opt);
            }
        }
        
        if modules.len() > 5 {
            println!("  ... and {} more modules", modules.len() - 5);
        }
    }
    
    // Generate performance concerns
    let large_files: Vec<&RustBlobInfo> = rust_files.iter()
        .filter(|f| f.blob_size > 100 * 1024)
        .collect();
    
    if !large_files.is_empty() {
        performance_concerns.push(format!("{} large files may impact compilation speed", large_files.len()));
    }
    
    let huge_files: Vec<&RustBlobInfo> = rust_files.iter()
        .filter(|f| f.blob_size > 500 * 1024)
        .collect();
    
    if !huge_files.is_empty() {
        performance_concerns.push(format!("{} huge files may impact IDE performance", huge_files.len()));
    }
    
    // Generate optimization recommendations
    if total_blob_size > 10 * 1024 * 1024 { // 10 MB
        optimization_recommendations.push("Consider splitting large modules to improve compilation speed".to_string());
    }
    
    if large_files.len() > 10 {
        optimization_recommendations.push("Many large files - consider refactoring for better incremental compilation".to_string());
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("  • Total Rust files: {}", rust_files.len());
    println!("  • Total blob size: {:.2} MB", total_blob_size as f64 / (1024.0 * 1024.0));
    println!("  • Average blob size: {:.1} KB", total_blob_size as f64 / rust_files.len() as f64 / 1024.0);
    println!("  • Modules analyzed: {}", modules.len());
    
    if !performance_concerns.is_empty() {
        println!("\n⚠️  Performance Concerns:");
        for concern in &performance_concerns {
            println!("  • {}", concern);
        }
    }
    
    if !optimization_recommendations.is_empty() {
        println!("\n💡 Optimization Recommendations:");
        for rec in &optimization_recommendations {
            println!("  • {}", rec);
        }
    }
    
    RustBlobAnalysis {
        rust_files: rust_files.to_vec(),
        modules: modules.to_vec(),
        total_blob_size,
        performance_concerns,
        optimization_recommendations,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Rust Blob Analyzer");
    println!("=====================");
    println!("Analyzing Rust files by actual blob sizes...");
    println!();
    
    // Analyze Rust blobs
    let rust_files = analyze_rust_blobs()?;
    
    // Analyze modules
    let modules = analyze_rust_modules(&rust_files);
    
    // Generate comprehensive report
    let _analysis = generate_rust_blob_report(&rust_files, &modules);
    
    println!("\n✅ Analysis complete!");
    println!("🦀 Rust blob sizes analyzed");
    println!("📦 Module structure examined");
    println!("💡 Performance insights ready");
    println!("🚀 Optimization recommendations prepared");
    
    Ok(())
}
