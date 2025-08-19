use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum AnalysisModule {
    BlobSize,        // Basic blob size analysis
    Deduplication,   // Blob reuse and deduplication
    Packfile,        // Packfile and delta chain analysis
    ObjectModel,     // Git object model analysis
    FileTypes,       // File type optimization
    RustSpecific,    // Rust project analysis
    FrequentWrites,  // Frequent write detection
    Performance,     // Performance impact analysis
    LfsOptimization, // Git LFS optimization
}

impl AnalysisModule {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "blob-size" | "blob" => Some(AnalysisModule::BlobSize),
            "dedup" | "deduplication" => Some(AnalysisModule::Deduplication),
            "pack" | "packfile" => Some(AnalysisModule::Packfile),
            "object" | "objects" => Some(AnalysisModule::ObjectModel),
            "file-types" | "types" => Some(AnalysisModule::FileTypes),
            "rust" | "rust-specific" => Some(AnalysisModule::RustSpecific),
            "writes" | "frequent-writes" => Some(AnalysisModule::FrequentWrites),
            "perf" | "performance" => Some(AnalysisModule::Performance),
            "lfs" | "lfs-optimization" => Some(AnalysisModule::LfsOptimization),
            _ => None,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            AnalysisModule::BlobSize => "Basic blob size distribution analysis",
            AnalysisModule::Deduplication => "Blob reuse and deduplication patterns",
            AnalysisModule::Packfile => "Packfile and delta chain optimization",
            AnalysisModule::ObjectModel => "Git object model and relationships",
            AnalysisModule::FileTypes => "File type optimization recommendations",
            AnalysisModule::RustSpecific => "Rust-specific project analysis",
            AnalysisModule::FrequentWrites => "Frequent write file detection",
            AnalysisModule::Performance => "Performance impact analysis",
            AnalysisModule::LfsOptimization => "Git LFS optimization for binary hooks",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            AnalysisModule::BlobSize => "📊",
            AnalysisModule::Deduplication => "🔄",
            AnalysisModule::Packfile => "📦",
            AnalysisModule::ObjectModel => "🧠",
            AnalysisModule::FileTypes => "📁",
            AnalysisModule::RustSpecific => "🦀",
            AnalysisModule::FrequentWrites => "📝",
            AnalysisModule::Performance => "⚡",
            AnalysisModule::LfsOptimization => "📦",
        }
    }
}

#[derive(Debug)]
struct AnalysisResult {
    module: AnalysisModule,
    summary: String,
    recommendations: Vec<String>,
    metrics: HashMap<String, f64>,
}

fn run_blob_size_analysis() -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    println!("📊 Running blob size analysis...");

    // Simplified blob size analysis
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut small_count = 0;
    let mut medium_count = 0;
    let mut large_count = 0;
    let mut huge_count = 0;

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(size) = u64::from_str(parts[1]) {
                match size {
                    0..=1023 => small_count += 1,
                    1024..=8191 => medium_count += 1,
                    8192..=204800 => large_count += 1,
                    _ => huge_count += 1,
                }
            }
        }
    }

    let total = small_count + medium_count + large_count + huge_count;
    let sweet_spot_percentage = if total > 0 {
        (large_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let mut metrics = HashMap::new();
    metrics.insert("small_files".to_string(), small_count as f64);
    metrics.insert("medium_files".to_string(), medium_count as f64);
    metrics.insert("large_files".to_string(), large_count as f64);
    metrics.insert("huge_files".to_string(), huge_count as f64);
    metrics.insert("sweet_spot_percentage".to_string(), sweet_spot_percentage);

    let summary = format!(
        "Found {} files in the 8-200 KB sweet spot ({:.1}%)",
        large_count, sweet_spot_percentage
    );

    let mut recommendations = Vec::new();
    if huge_count > 0 {
        recommendations.push("Consider Git LFS for files > 1 MB".to_string());
    }
    if small_count > total / 2 {
        recommendations.push("Many small files - consider consolidation".to_string());
    }

    Ok(AnalysisResult {
        module: AnalysisModule::BlobSize,
        summary,
        recommendations,
        metrics,
    })
}

fn run_deduplication_analysis() -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    println!("🔄 Running deduplication analysis...");

    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut hash_counts: HashMap<String, u32> = HashMap::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            *hash_counts.entry(hash).or_insert(0) += 1;
        }
    }

    let total_references: u32 = hash_counts.values().sum();
    let unique_objects = hash_counts.len() as u32;
    let reuse_ratio = if unique_objects > 0 {
        total_references as f64 / unique_objects as f64
    } else {
        0.0
    };

    let mut metrics = HashMap::new();
    metrics.insert("total_references".to_string(), total_references as f64);
    metrics.insert("unique_objects".to_string(), unique_objects as f64);
    metrics.insert("reuse_ratio".to_string(), reuse_ratio);

    let summary = format!("Average reuse ratio: {:.2}x", reuse_ratio);

    let mut recommendations = Vec::new();
    if reuse_ratio > 1.5 {
        recommendations.push("Good deduplication - efficient storage".to_string());
    } else if reuse_ratio < 1.1 {
        recommendations.push("Low deduplication - consider optimization".to_string());
    }

    Ok(AnalysisResult {
        module: AnalysisModule::Deduplication,
        summary,
        recommendations,
        metrics,
    })
}

fn run_file_type_analysis() -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    println!("📁 Running file type analysis...");

    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut file_types: HashMap<String, u32> = HashMap::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let path = parts[2..].join(" ");
            if let Some(dot_pos) = path.rfind('.') {
                let extension = path[dot_pos..].to_string();
                *file_types.entry(extension).or_insert(0) += 1;
            }
        }
    }

    let total_files: u32 = file_types.values().sum();
    let text_files = file_types
        .iter()
        .filter(|(ext, _)| ext.contains(".rs") || ext.contains(".py") || ext.contains(".md"))
        .map(|(_, &count)| count)
        .sum::<u32>();

    let text_percentage = if total_files > 0 {
        (text_files as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };

    let mut metrics = HashMap::new();
    metrics.insert("total_files".to_string(), total_files as f64);
    metrics.insert("text_files".to_string(), text_files as f64);
    metrics.insert("text_percentage".to_string(), text_percentage);

    let summary = format!(
        "{} text files ({:.1}%) - good for delta compression",
        text_files, text_percentage
    );

    let mut recommendations = Vec::new();
    if text_percentage > 70.0 {
        recommendations.push("High text file ratio - excellent for delta compression".to_string());
    } else if text_percentage < 30.0 {
        recommendations.push("Low text file ratio - consider file type optimization".to_string());
    }

    Ok(AnalysisResult {
        module: AnalysisModule::FileTypes,
        summary,
        recommendations,
        metrics,
    })
}

fn run_frequent_writes_analysis() -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    println!("📝 Running frequent writes analysis...");

    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut log_files = 0;
    let mut cache_files = 0;
    let mut build_files = 0;
    let mut temp_files = 0;

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let path = parts[2..].join(" ").to_lowercase();

            if path.contains(".log") || path.contains(".out") || path.contains(".err") {
                log_files += 1;
            } else if path.contains("cache") || path.contains("tmp") || path.contains("temp") {
                cache_files += 1;
            } else if path.contains("build") || path.contains("dist") || path.contains("target") {
                build_files += 1;
            } else if path.contains(".tmp") || path.contains(".temp") || path.contains(".swp") {
                temp_files += 1;
            }
        }
    }

    let total_problematic = log_files + cache_files + build_files + temp_files;

    let mut metrics = HashMap::new();
    metrics.insert("log_files".to_string(), log_files as f64);
    metrics.insert("cache_files".to_string(), cache_files as f64);
    metrics.insert("build_files".to_string(), build_files as f64);
    metrics.insert("temp_files".to_string(), temp_files as f64);
    metrics.insert("total_problematic".to_string(), total_problematic as f64);

    let summary = format!(
        "Found {} files that should be in .gitignore",
        total_problematic
    );

    let mut recommendations = Vec::new();
    if log_files > 0 {
        recommendations.push("Add *.log to .gitignore".to_string());
    }
    if cache_files > 0 {
        recommendations.push("Add cache/ and tmp/ to .gitignore".to_string());
    }
    if build_files > 0 {
        recommendations.push("Add build/ and dist/ to .gitignore".to_string());
    }
    if temp_files > 0 {
        recommendations.push("Add *.tmp and *.temp to .gitignore".to_string());
    }

    Ok(AnalysisResult {
        module: AnalysisModule::FrequentWrites,
        summary,
        recommendations,
        metrics,
    })
}

fn show_analysis_results(results: &[AnalysisResult]) {
    println!("\n📊 Analysis Results:");
    println!("===================");

    for result in results {
        println!(
            "{} {}: {}",
            result.module.emoji(),
            result.module.description(),
            result.summary
        );

        if !result.recommendations.is_empty() {
            println!("   Recommendations:");
            for rec in &result.recommendations {
                println!("   • {}", rec);
            }
        }
        println!();
    }
}

fn show_available_modules() {
    println!("🔧 Available Analysis Modules:");
    println!("=============================");

    for module in [
        AnalysisModule::BlobSize,
        AnalysisModule::Deduplication,
        AnalysisModule::Packfile,
        AnalysisModule::ObjectModel,
        AnalysisModule::FileTypes,
        AnalysisModule::RustSpecific,
        AnalysisModule::FrequentWrites,
        AnalysisModule::Performance,
        AnalysisModule::LfsOptimization,
    ] {
        println!(
            "{} {}: {}",
            module.emoji(),
            module.description(),
            format!("--module {}", format!("{:?}", module).to_lowercase())
        );
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("🔧 Modular Git Analysis Tool");
        println!("===========================");
        println!();
        println!("Usage: cargo run --bin modular_analyzer -- [modules...]");
        println!();
        show_available_modules();
        println!("Examples:");
        println!("  cargo run --bin modular_analyzer -- blob-size dedup");
        println!("  cargo run --bin modular_analyzer -- rust-specific");
        println!("  cargo run --bin modular_analyzer -- file-types frequent-writes");
        println!("  cargo run --bin modular_analyzer -- all");
        return Ok(());
    }

    let mut modules_to_run = Vec::new();

    for arg in &args[1..] {
        if arg == "all" {
            modules_to_run = vec![
                AnalysisModule::BlobSize,
                AnalysisModule::Deduplication,
                AnalysisModule::FileTypes,
                AnalysisModule::FrequentWrites,
                AnalysisModule::LfsOptimization,
            ];
            break;
        } else if let Some(module) = AnalysisModule::from_str(arg) {
            modules_to_run.push(module);
        } else {
            println!("❌ Unknown module: {}", arg);
            show_available_modules();
            return Ok(());
        }
    }

    if modules_to_run.is_empty() {
        println!("❌ No valid modules specified");
        show_available_modules();
        return Ok(());
    }

    println!("🚀 Running {} analysis modules...", modules_to_run.len());
    println!();

    let mut results = Vec::new();

    for module in modules_to_run {
        let result = match module {
            AnalysisModule::BlobSize => run_blob_size_analysis(),
            AnalysisModule::Deduplication => run_deduplication_analysis(),
            AnalysisModule::FileTypes => run_file_type_analysis(),
            AnalysisModule::FrequentWrites => run_frequent_writes_analysis(),
            AnalysisModule::LfsOptimization => run_lfs_optimization_analysis(),
            _ => {
                println!("⚠️  Module {:?} not yet implemented", module);
                continue;
            }
        };

        match result {
            Ok(result) => results.push(result),
            Err(e) => println!("❌ Error running {:?}: {}", module, e),
        }
    }

    show_analysis_results(&results);

    println!("💡 Key Insights:");
    println!("=================");
    println!("• Run specific modules for focused analysis");
    println!("• Use 'all' for comprehensive analysis");
    println!("• Each module provides targeted recommendations");
    println!("• Combine modules for different use cases");

    Ok(())
}

fn run_lfs_optimization_analysis() -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    println!("📦 Running LFS optimization analysis...");

    let output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut lfs_candidates = 0;
    let mut shared_binaries = 0;
    let mut total_potential_savings = 0;

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1];

            // Get file size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;

            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    // Check if this is an LFS candidate
                    if size > 1024 * 1024 {
                        // 1 MB
                        lfs_candidates += 1;
                        total_potential_savings += size - 130; // LFS pointer is ~130 bytes
                    }
                }
            }
        }
    }

    let mut metrics = HashMap::new();
    metrics.insert("lfs_candidates".to_string(), lfs_candidates as f64);
    metrics.insert("shared_binaries".to_string(), shared_binaries as f64);
    metrics.insert(
        "total_potential_savings".to_string(),
        total_potential_savings as f64,
    );

    let summary = format!(
        "Found {} LFS candidates with {:.2} MB potential savings",
        lfs_candidates,
        total_potential_savings as f64 / (1024.0 * 1024.0)
    );

    let mut recommendations = Vec::new();
    if lfs_candidates > 0 {
        recommendations.push("Consider Git LFS for large binary files".to_string());
        recommendations.push("Add .gitattributes rules for binary file types".to_string());
    }
    if shared_binaries > 0 {
        recommendations.push("Use symbolic links for identical binary hooks".to_string());
    }

    Ok(AnalysisResult {
        module: AnalysisModule::LfsOptimization,
        summary,
        recommendations,
        metrics,
    })
}
