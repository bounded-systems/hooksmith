use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct RustFileInfo {
    path: String,
    size: u64,
    file_type: RustFileType,
    is_tracked: bool,
    is_generated: bool,
    dependency_count: u32,
    complexity_score: f64,
    performance_impact: PerformanceImpact,
}

#[derive(Debug, Clone, PartialEq)]
enum RustFileType {
    Source,      // .rs files
    Config,      // Cargo.toml, Cargo.lock
    Documentation, // .md, .txt
    Generated,   // build.rs output, codegen
    Binary,      // executables, libraries
    Asset,       // images, data files
    Other,
}

#[derive(Debug, Clone)]
struct PerformanceImpact {
    compilation_speed: CompilationSpeed,
    ide_performance: IDEPerformance,
    cache_efficiency: CacheEfficiency,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
enum CompilationSpeed {
    Fast,      // < 10KB
    Moderate,  // 10-50KB
    Slow,      // 50-100KB
    VerySlow,  // > 100KB
}

#[derive(Debug, Clone)]
enum IDEPerformance {
    Excellent, // < 5KB
    Good,      // 5-20KB
    Moderate,  // 20-50KB
    Poor,      // > 50KB
}

#[derive(Debug, Clone)]
enum CacheEfficiency {
    High,      // Small files, good for incremental compilation
    Medium,    // Moderate size, some cache pressure
    Low,       // Large files, poor cache efficiency
}

impl RustFileType {
    fn from_path(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        
        if path_lower.ends_with(".rs") {
            RustFileType::Source
        } else if path_lower.ends_with("cargo.toml") || path_lower.ends_with("cargo.lock") {
            RustFileType::Config
        } else if path_lower.ends_with(".md") || path_lower.ends_with(".txt") || path_lower.ends_with(".rst") {
            RustFileType::Documentation
        } else if path_lower.contains("target/") || path_lower.contains("build/") || path_lower.ends_with(".generated.rs") {
            RustFileType::Generated
        } else if path_lower.ends_with(".exe") || path_lower.ends_with(".dll") || path_lower.ends_with(".so") || path_lower.ends_with(".dylib") {
            RustFileType::Binary
        } else if path_lower.ends_with(".png") || path_lower.ends_with(".jpg") || path_lower.ends_with(".json") || path_lower.ends_with(".yaml") {
            RustFileType::Asset
        } else {
            RustFileType::Other
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            RustFileType::Source => "🦀",
            RustFileType::Config => "⚙️",
            RustFileType::Documentation => "📚",
            RustFileType::Generated => "🔧",
            RustFileType::Binary => "📦",
            RustFileType::Asset => "📁",
            RustFileType::Other => "📄",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            RustFileType::Source => "Rust source files",
            RustFileType::Config => "Cargo configuration",
            RustFileType::Documentation => "Documentation files",
            RustFileType::Generated => "Generated/build files",
            RustFileType::Binary => "Binary artifacts",
            RustFileType::Asset => "Data/assets",
            RustFileType::Other => "Other files",
        }
    }
}

impl PerformanceImpact {
    fn new(size: u64) -> Self {
        let compilation_speed = match size {
            0..=10*1024 => CompilationSpeed::Fast,
            10*1024..=50*1024 => CompilationSpeed::Moderate,
            50*1024..=100*1024 => CompilationSpeed::Slow,
            _ => CompilationSpeed::VerySlow,
        };
        
        let ide_performance = match size {
            0..=5*1024 => IDEPerformance::Excellent,
            5*1024..=20*1024 => IDEPerformance::Good,
            20*1024..=50*1024 => IDEPerformance::Moderate,
            _ => IDEPerformance::Poor,
        };
        
        let cache_efficiency = match size {
            0..=20*1024 => CacheEfficiency::High,
            20*1024..=50*1024 => CacheEfficiency::Medium,
            _ => CacheEfficiency::Low,
        };
        
        let mut recommendations = Vec::new();
        
        if size > 100*1024 {
            recommendations.push("Consider splitting into submodules".to_string());
            recommendations.push("Large files slow down incremental compilation".to_string());
        } else if size > 50*1024 {
            recommendations.push("Monitor compilation performance".to_string());
            recommendations.push("Consider breaking into smaller modules".to_string());
        }
        
        if size > 20*1024 {
            recommendations.push("May impact IDE performance (rust-analyzer)".to_string());
        }
        
        PerformanceImpact {
            compilation_speed,
            ide_performance,
            cache_efficiency,
            recommendations,
        }
    }
}

#[derive(Debug)]
struct RustProjectStats {
    total_files: u32,
    total_size: u64,
    source_files: u32,
    source_size: u64,
    config_files: u32,
    config_size: u64,
    generated_files: u32,
    generated_size: u64,
    binary_files: u32,
    binary_size: u64,
    by_type: HashMap<RustFileType, u32>,
    by_type_size: HashMap<RustFileType, u64>,
    duplicate_blobs: u32,
    large_files: u32,
    oversized_rust_files: u32,
    performance_issues: u32,
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    
    match size {
        0..=KB-1 => format!("{} B", size),
        KB..=MB-1 => format!("{:.1} KB", size as f64 / KB as f64),
        MB..=GB-1 => format!("{:.1} MB", size as f64 / MB as f64),
        _ => format!("{:.1} GB", size as f64 / GB as f64),
    }
}

fn get_rust_project_files() -> Result<Vec<RustFileInfo>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut file_counts: HashMap<String, u32> = HashMap::new();
    
    // Get all tracked files
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    
    // Count references and collect file info
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let hash = parts[0].to_string();
            let path = parts[2..].join(" ");
            
            // Skip common ignored directories
            if path.contains("target/") || path.contains(".cargo/") || path.contains("node_modules/") {
                continue;
            }
            
            *file_counts.entry(hash.clone()).or_insert(0) += 1;
            
            // Get file size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", &hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let file_type = RustFileType::from_path(&path);
                    let is_generated = path.contains("target/") || path.contains("build/") || 
                                     path.ends_with(".generated.rs") || path.contains("generated/");
                    
                    files.push(RustFileInfo {
                        path,
                        size,
                        file_type,
                        is_tracked: true,
                        is_generated,
                        dependency_count: 0, // Would need more complex analysis
                        complexity_score: calculate_complexity_score(&path, size),
                        performance_impact: PerformanceImpact::new(size),
                    });
                }
            }
        }
    }
    
    Ok(files)
}

fn calculate_complexity_score(path: &str, size: u64) -> f64 {
    let mut score = 0.0;
    
    // Size factor
    score += (size as f64 / 1024.0).min(100.0) * 0.1;
    
    // Path depth factor
    let depth = path.matches('/').count();
    score += depth as f64 * 0.5;
    
    // File type factor
    if path.ends_with(".rs") {
        score += 10.0; // Source files are more complex
    }
    
    score
}

fn analyze_cargo_dependencies() -> Result<HashMap<String, u32>, Box<dyn std::error::Error>> {
    let mut deps = HashMap::new();
    
    // Try to read Cargo.toml
    let cargo_content = std::fs::read_to_string("Cargo.toml");
    if let Ok(content) = cargo_content {
        for line in content.lines() {
            if line.trim().starts_with("[dependencies]") {
                // This is a simplified parser - in practice you'd want a proper TOML parser
                continue;
            }
            if line.contains("=") && !line.trim().starts_with('#') {
                if let Some(dep_name) = line.split('=').next() {
                    let name = dep_name.trim().to_string();
                    if !name.is_empty() && !name.starts_with('[') {
                        *deps.entry(name).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    
    Ok(deps)
}

fn calculate_project_stats(files: &[RustFileInfo]) -> RustProjectStats {
    let mut stats = RustProjectStats {
        total_files: 0,
        total_size: 0,
        source_files: 0,
        source_size: 0,
        config_files: 0,
        config_size: 0,
        generated_files: 0,
        generated_size: 0,
        binary_files: 0,
        binary_size: 0,
        by_type: HashMap::new(),
        by_type_size: HashMap::new(),
        duplicate_blobs: 0,
        large_files: 0,
        oversized_rust_files: 0,
        performance_issues: 0,
    };
    
    for file in files {
        stats.total_files += 1;
        stats.total_size += file.size;
        
        *stats.by_type.entry(file.file_type.clone()).or_insert(0) += 1;
        *stats.by_type_size.entry(file.file_type.clone()).or_insert(0) += file.size;
        
        match file.file_type {
            RustFileType::Source => {
                stats.source_files += 1;
                stats.source_size += file.size;
                
                // Check for oversized Rust files
                if file.size > 100 * 1024 { // > 100KB
                    stats.oversized_rust_files += 1;
                }
                
                // Check for performance issues
                if file.size > 50 * 1024 { // > 50KB
                    stats.performance_issues += 1;
                }
            }
            RustFileType::Config => {
                stats.config_files += 1;
                stats.config_size += file.size;
            }
            RustFileType::Generated => {
                stats.generated_files += 1;
                stats.generated_size += file.size;
            }
            RustFileType::Binary => {
                stats.binary_files += 1;
                stats.binary_size += file.size;
            }
            _ => {}
        }
        
        if file.size > 1024 * 1024 {
            stats.large_files += 1;
        }
    }
    
    stats
}

fn show_rust_project_analysis(stats: &RustProjectStats) {
    println!("🦀 Rust Project Git Analysis:");
    println!("=============================");
    
    println!("📊 File Type Distribution:");
    for file_type in [
        RustFileType::Source,
        RustFileType::Config,
        RustFileType::Documentation,
        RustFileType::Generated,
        RustFileType::Binary,
        RustFileType::Asset,
        RustFileType::Other,
    ] {
        let count = stats.by_type.get(&file_type).unwrap_or(&0);
        let size = stats.by_type_size.get(&file_type).unwrap_or(&0);
        let percentage = if stats.total_files > 0 {
            (*count as f64 / stats.total_files as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{} {}: {} ({:.1}%) - {}", 
            file_type.emoji(),
            file_type.description(),
            count,
            percentage,
            format_size(*size));
    }
    println!();
    
    println!("📈 Project Statistics:");
    println!("   Total files: {}", stats.total_files);
    println!("   Total size: {}", format_size(stats.total_size));
    println!("   Source files: {} ({})", stats.source_files, format_size(stats.source_size));
    println!("   Config files: {} ({})", stats.config_files, format_size(stats.config_size));
    println!("   Large files (>1MB): {}", stats.large_files);
    println!("   Oversized Rust files (>100KB): {}", stats.oversized_rust_files);
    println!("   Performance issues (>50KB): {}", stats.performance_issues);
    println!();
}

fn show_performance_analysis(files: &[RustFileInfo]) {
    println!("⚡ Performance Analysis:");
    println!("=======================");
    
    let rust_files: Vec<&RustFileInfo> = files.iter()
        .filter(|f| f.file_type == RustFileType::Source)
        .collect();
    
    if rust_files.is_empty() {
        println!("No Rust source files found.");
        return;
    }
    
    // Compilation speed analysis
    let mut fast_count = 0;
    let mut moderate_count = 0;
    let mut slow_count = 0;
    let mut very_slow_count = 0;
    
    // IDE performance analysis
    let mut excellent_count = 0;
    let mut good_count = 0;
    let mut moderate_count_ide = 0;
    let mut poor_count = 0;
    
    for file in &rust_files {
        match file.performance_impact.compilation_speed {
            CompilationSpeed::Fast => fast_count += 1,
            CompilationSpeed::Moderate => moderate_count += 1,
            CompilationSpeed::Slow => slow_count += 1,
            CompilationSpeed::VerySlow => very_slow_count += 1,
        }
        
        match file.performance_impact.ide_performance {
            IDEPerformance::Excellent => excellent_count += 1,
            IDEPerformance::Good => good_count += 1,
            IDEPerformance::Moderate => moderate_count_ide += 1,
            IDEPerformance::Poor => poor_count += 1,
        }
    }
    
    println!("🔧 Compilation Speed:");
    println!("   Fast (<10KB): {} files", fast_count);
    println!("   Moderate (10-50KB): {} files", moderate_count);
    println!("   Slow (50-100KB): {} files", slow_count);
    println!("   Very Slow (>100KB): {} files", very_slow_count);
    println!();
    
    println!("💻 IDE Performance (rust-analyzer):");
    println!("   Excellent (<5KB): {} files", excellent_count);
    println!("   Good (5-20KB): {} files", good_count);
    println!("   Moderate (20-50KB): {} files", moderate_count_ide);
    println!("   Poor (>50KB): {} files", poor_count);
    println!();
    
    // Show oversized files
    let oversized: Vec<&RustFileInfo> = rust_files.iter()
        .filter(|f| f.size > 100 * 1024)
        .collect();
    
    if !oversized.is_empty() {
        println!("⚠️  Oversized Rust Files (>100KB):");
        for file in oversized {
            println!("   • {} ({})", file.path, format_size(file.size));
        }
        println!();
    }
}

fn show_rust_specific_recommendations(stats: &RustProjectStats, files: &[RustFileInfo]) {
    println!("⚙️  Rust-Specific Recommendations:");
    println!("==================================");
    
    // Check for generated files in Git
    if stats.generated_files > 0 {
        println!("🔧 Generated files detected in Git:");
        println!("   • {} generated files found", stats.generated_files);
        println!("   • Consider adding to .gitignore:");
        println!("     target/");
        println!("     build/");
        println!("     *.generated.rs");
        println!("     generated/");
    }
    
    // Check for oversized Rust files
    if stats.oversized_rust_files > 0 {
        println!("🔧 Oversized Rust files detected:");
        println!("   • {} files > 100KB", stats.oversized_rust_files);
        println!("   • These slow down incremental compilation");
        println!("   • Consider splitting into submodules");
        println!("   • Large files hurt IDE performance (rust-analyzer)");
    }
    
    // Check for performance issues
    if stats.performance_issues > 0 {
        println!("🔧 Performance issues detected:");
        println!("   • {} files > 50KB", stats.performance_issues);
        println!("   • Monitor compilation times");
        println!("   • Consider breaking into smaller modules");
    }
    
    // Check for large files
    if stats.large_files > 0 {
        println!("🔧 Large files detected:");
        println!("   • {} files > 1MB", stats.large_files);
        println!("   • Consider Git LFS for large assets");
        println!("   • Review if large files should be in Git");
    }
    
    // Check source file distribution
    let avg_source_size = if stats.source_files > 0 {
        stats.source_size / stats.source_files
    } else { 0 };
    
    if avg_source_size > 50 * 1024 { // 50KB average
        println!("🔧 Large source files detected:");
        println!("   • Average source file size: {}", format_size(avg_source_size));
        println!("   • Consider splitting large modules");
    }
    
    // Check binary files
    if stats.binary_files > 0 {
        println!("🔧 Binary files in Git:");
        println!("   • {} binary files found", stats.binary_files);
        println!("   • Consider excluding from Git or using Git LFS");
    }
    
    println!();
    println!("🚀 Rust + Git Pro Tips:");
    println!("• Use .gitignore for target/, .cargo/, build/");
    println!("• Keep Cargo.lock in Git for reproducible builds");
    println!("• Use cargo vendor sparingly (can bloat repo)");
    println!("• Consider rustfmt in pre-commit hooks");
    println!("• Use cargo check in CI, not cargo build");
    println!("• Monitor file sizes for IDE performance");
}

fn show_cargo_optimization_tips() {
    println!("📦 Cargo + Git Optimization:");
    println!("============================");
    
    println!("✅ Best Practices:");
    println!("• Keep Cargo.toml and Cargo.lock in Git");
    println!("• Ignore target/ and .cargo/ directories");
    println!("• Use cargo check for CI (faster than build)");
    println!("• Consider cargo vendor for offline builds");
    println!("• Use rustfmt to maintain consistent formatting");
    
    println!();
    println!("🔧 .gitignore for Rust projects:");
    println!("target/");
    println!("Cargo.lock  # Only for libraries, keep for binaries");
    println!(".cargo/");
    println!("*.generated.rs");
    println!("build/");
    println!("dist/");
    
    println!();
    println!("⚡ Performance Tips:");
    println!("• Use cargo check instead of cargo build in CI");
    println!("• Consider sccache for build caching");
    println!("• Use cargo-udeps to find unused dependencies");
    println!("• Profile with cargo build --release");
    println!("• Keep .rs files under 100KB for optimal performance");
    println!("• Monitor rust-analyzer performance with large files");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Rust Project Git Analysis");
    println!("============================");
    println!();
    
    let files = get_rust_project_files()?;
    if files.is_empty() {
        println!("❌ No Rust project files found. Are you in a Rust repository?");
        return Ok(());
    }
    
    let stats = calculate_project_stats(&files);
    
    show_rust_project_analysis(&stats);
    show_performance_analysis(&files);
    show_rust_specific_recommendations(&stats, &files);
    show_cargo_optimization_tips();
    
    println!("💡 Key Insights for Rust + Git:");
    println!("================================");
    println!("• Git stores Rust source as content-addressed blobs");
    println!("• Build artifacts (target/) should be ignored by Git");
    println!("• Cargo.lock should be tracked for reproducible builds");
    println!("• Large generated files can bloat Git history");
    println!("• Rust source files compress well with delta compression");
    println!("• Consider Git LFS for large assets or binaries");
    println!("• File size affects incremental compilation performance");
    println!("• Large .rs files slow down rust-analyzer and IDE tools");
    
    Ok(())
}
