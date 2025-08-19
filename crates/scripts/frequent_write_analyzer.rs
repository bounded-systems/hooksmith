use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct FrequentWriteFile {
    path: String,
    size: u64,
    file_type: FrequentWriteType,
    write_frequency: WriteFrequency,
    git_impact: GitImpact,
    recommendation: String,
}

#[derive(Debug, Clone, PartialEq)]
enum FrequentWriteType {
    Config,         // .gitignore, .env, config files
    Log,           // .log, .out, .err files
    Cache,         // Cache files, temporary data
    Build,         // Build artifacts, generated files
    Lock,          // Lock files, .lock files
    Temp,          // Temporary files
    Other,
}

#[derive(Debug, Clone)]
enum WriteFrequency {
    VeryHigh,     // Written multiple times per session
    High,         // Written frequently
    Moderate,     // Written occasionally
    Low,          // Written rarely
}

#[derive(Debug, Clone)]
enum GitImpact {
    Critical,      // Should never be in Git
    High,         // Should be in .gitignore
    Moderate,     // Consider ignoring
    Low,          // Can be tracked if needed
}

impl FrequentWriteType {
    fn from_path(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        
        if matches!(path_lower.as_str(), 
            ".gitignore" | ".env" | ".env.local" | ".env.production" | ".env.development" |
            "config.json" | "settings.json" | "preferences.json" | ".editorconfig" |
            ".eslintrc" | ".prettierrc" | "tsconfig.json" | "package.json" | "Cargo.toml") {
            FrequentWriteType::Config
        }
        else if matches!(path_lower.as_str(), 
            ".log" | ".out" | ".err" | "debug.log" | "error.log" | "access.log" |
            "stdout.log" | "stderr.log" | "application.log" | "server.log") {
            FrequentWriteType::Log
        }
        else if matches!(path_lower.as_str(), 
            ".cache" | "cache/" | ".tmp" | "tmp/" | "temp/" | ".temp" |
            "node_modules/" | "target/" | "build/" | "dist/" | ".build" |
            "*.cache" | "*.tmp" | "*.temp") {
            FrequentWriteType::Cache
        }
        else if matches!(path_lower.as_str(), 
            "build/" | "dist/" | "out/" | "generated/" | "compiled/" |
            "*.min.js" | "*.min.css" | "*.bundle.js" | "*.chunk.js" |
            "*.o" | "*.obj" | "*.exe" | "*.dll" | "*.so" | "*.dylib") {
            FrequentWriteType::Build
        }
        else if matches!(path_lower.as_str(), 
            "*.lock" | ".lock" | "lockfile" | "package-lock.json" | "yarn.lock" |
            "Cargo.lock" | "composer.lock" | "Gemfile.lock" | "Pipfile.lock") {
            FrequentWriteType::Lock
        }
        else if matches!(path_lower.as_str(), 
            "*.tmp" | "*.temp" | "*.swp" | "*.swo" | "*~" | ".#*" |
            ".DS_Store" | "Thumbs.db" | "desktop.ini") {
            FrequentWriteType::Temp
        }
        else {
            FrequentWriteType::Other
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            FrequentWriteType::Config => "⚙️",
            FrequentWriteType::Log => "📝",
            FrequentWriteType::Cache => "🗄️",
            FrequentWriteType::Build => "🔧",
            FrequentWriteType::Lock => "🔒",
            FrequentWriteType::Temp => "📄",
            FrequentWriteType::Other => "❓",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            FrequentWriteType::Config => "Configuration files",
            FrequentWriteType::Log => "Log files",
            FrequentWriteType::Cache => "Cache and temporary files",
            FrequentWriteType::Build => "Build artifacts",
            FrequentWriteType::Lock => "Lock files",
            FrequentWriteType::Temp => "Temporary files",
            FrequentWriteType::Other => "Other files",
        }
    }

    fn write_frequency(&self) -> WriteFrequency {
        match self {
            FrequentWriteType::Config => WriteFrequency::Moderate,
            FrequentWriteType::Log => WriteFrequency::VeryHigh,
            FrequentWriteType::Cache => WriteFrequency::High,
            FrequentWriteType::Build => WriteFrequency::High,
            FrequentWriteType::Lock => WriteFrequency::Moderate,
            FrequentWriteType::Temp => WriteFrequency::VeryHigh,
            FrequentWriteType::Other => WriteFrequency::Low,
        }
    }

    fn git_impact(&self) -> GitImpact {
        match self {
            FrequentWriteType::Config => GitImpact::Moderate, // Some config files should be tracked
            FrequentWriteType::Log => GitImpact::Critical,
            FrequentWriteType::Cache => GitImpact::Critical,
            FrequentWriteType::Build => GitImpact::Critical,
            FrequentWriteType::Lock => GitImpact::High, // Some lock files should be tracked
            FrequentWriteType::Temp => GitImpact::Critical,
            FrequentWriteType::Other => GitImpact::Low,
        }
    }
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

fn get_frequent_write_files() -> Result<Vec<FrequentWriteFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    // Get all tracked files
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let hash = parts[0].to_string();
            let path = parts[2..].join(" ");
            
            // Get file size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", &hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let file_type = FrequentWriteType::from_path(&path);
                    let write_frequency = file_type.write_frequency();
                    let git_impact = file_type.git_impact();
                    
                    let recommendation = generate_recommendation(&file_type, &path, size);
                    
                    files.push(FrequentWriteFile {
                        path,
                        size,
                        file_type,
                        write_frequency,
                        git_impact,
                        recommendation,
                    });
                }
            }
        }
    }
    
    Ok(files)
}

fn generate_recommendation(file_type: &FrequentWriteType, path: &str, size: u64) -> String {
    match file_type {
        FrequentWriteType::Config => {
            if path.contains(".env") {
                "Should be in .gitignore - contains sensitive data".to_string()
            } else if path.contains(".gitignore") {
                "Should be tracked in Git".to_string()
            } else {
                "Consider if this config should be tracked".to_string()
            }
        }
        FrequentWriteType::Log => {
            "Should be in .gitignore - logs change frequently".to_string()
        }
        FrequentWriteType::Cache => {
            "Should be in .gitignore - cache files change frequently".to_string()
        }
        FrequentWriteType::Build => {
            "Should be in .gitignore - build artifacts".to_string()
        }
        FrequentWriteType::Lock => {
            if path.contains("package-lock.json") || path.contains("Cargo.lock") {
                "Should be tracked for reproducible builds".to_string()
            } else {
                "Consider if this lock file should be tracked".to_string()
            }
        }
        FrequentWriteType::Temp => {
            "Should be in .gitignore - temporary files".to_string()
        }
        FrequentWriteType::Other => {
            "Monitor for appropriate handling".to_string()
        }
    }
}

fn show_frequent_write_analysis(files: &[FrequentWriteFile]) {
    println!("📝 Frequent Write File Analysis:");
    println!("================================");
    
    // Group by file type
    let mut by_type: HashMap<FrequentWriteType, Vec<&FrequentWriteFile>> = HashMap::new();
    for file in files {
        by_type.entry(file.file_type.clone()).or_insert_with(Vec::new).push(file);
    }
    
    for file_type in [
        FrequentWriteType::Config,
        FrequentWriteType::Log,
        FrequentWriteType::Cache,
        FrequentWriteType::Build,
        FrequentWriteType::Lock,
        FrequentWriteType::Temp,
        FrequentWriteType::Other,
    ] {
        if let Some(type_files) = by_type.get(&file_type) {
            let total_count = type_files.len();
            let total_size: u64 = type_files.iter().map(|f| f.size).sum();
            let avg_size = if total_count > 0 { total_size / total_count as u64 } else { 0 };
            
            println!("{} {}:", file_type.emoji(), file_type.description());
            println!("   Files: {}", total_count);
            println!("   Total size: {}", format_size(total_size));
            println!("   Average size: {}", format_size(avg_size));
            println!("   Write frequency: {:?}", file_type.write_frequency());
            println!("   Git impact: {:?}", file_type.git_impact());
            println!();
        }
    }
}

fn show_critical_files(files: &[FrequentWriteFile]) {
    println!("⚠️  Critical Files (Should be in .gitignore):");
    println!("=============================================");
    
    let critical_files: Vec<&FrequentWriteFile> = files.iter()
        .filter(|f| matches!(f.git_impact, GitImpact::Critical))
        .collect();
    
    if critical_files.is_empty() {
        println!("No critical files found.");
        println!();
        return;
    }
    
    for file in critical_files {
        println!("• {} ({}) - {}", 
            file.path, 
            format_size(file.size),
            file.recommendation);
    }
    println!();
}

fn show_gitignore_recommendations(files: &[FrequentWriteFile]) {
    println!("🔧 .gitignore Recommendations:");
    println!("=============================");
    
    let mut recommendations = Vec::new();
    
    // Check for log files
    let log_files: Vec<&FrequentWriteFile> = files.iter()
        .filter(|f| f.file_type == FrequentWriteType::Log)
        .collect();
    
    if !log_files.is_empty() {
        recommendations.push("*.log".to_string());
        recommendations.push("*.out".to_string());
        recommendations.push("*.err".to_string());
    }
    
    // Check for cache files
    let cache_files: Vec<&FrequentWriteFile> = files.iter()
        .filter(|f| f.file_type == FrequentWriteType::Cache)
        .collect();
    
    if !cache_files.is_empty() {
        recommendations.push(".cache/".to_string());
        recommendations.push("cache/".to_string());
        recommendations.push("tmp/".to_string());
        recommendations.push("temp/".to_string());
    }
    
    // Check for build files
    let build_files: Vec<&FrequentWriteFile> = files.iter()
        .filter(|f| f.file_type == FrequentWriteType::Build)
        .collect();
    
    if !build_files.is_empty() {
        recommendations.push("build/".to_string());
        recommendations.push("dist/".to_string());
        recommendations.push("out/".to_string());
        recommendations.push("*.min.js".to_string());
        recommendations.push("*.min.css".to_string());
    }
    
    // Check for temp files
    let temp_files: Vec<&FrequentWriteFile> = files.iter()
        .filter(|f| f.file_type == FrequentWriteType::Temp)
        .collect();
    
    if !temp_files.is_empty() {
        recommendations.push("*.tmp".to_string());
        recommendations.push("*.temp".to_string());
        recommendations.push(".DS_Store".to_string());
        recommendations.push("Thumbs.db".to_string());
    }
    
    if recommendations.is_empty() {
        println!("No specific .gitignore recommendations needed.");
    } else {
        println!("Add these to .gitignore:");
        for rec in recommendations {
            println!("{}", rec);
        }
    }
    println!();
}

fn show_performance_impact(files: &[FrequentWriteFile]) {
    println!("⚡ Performance Impact Analysis:");
    println!("==============================");
    
    let mut very_high_count = 0;
    let mut high_count = 0;
    let mut moderate_count = 0;
    let mut low_count = 0;
    
    for file in files {
        match file.write_frequency {
            WriteFrequency::VeryHigh => very_high_count += 1,
            WriteFrequency::High => high_count += 1,
            WriteFrequency::Moderate => moderate_count += 1,
            WriteFrequency::Low => low_count += 1,
        }
    }
    
    let total_files = files.len();
    
    println!("Very high write frequency: {} files ({:.1}%)", 
        very_high_count, if total_files > 0 { (very_high_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("High write frequency: {} files ({:.1}%)", 
        high_count, if total_files > 0 { (high_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Moderate write frequency: {} files ({:.1}%)", 
        moderate_count, if total_files > 0 { (moderate_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Low write frequency: {} files ({:.1}%)", 
        low_count, if total_files > 0 { (low_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Frequent Write File Analysis");
    println!("==============================");
    println!();
    
    let files = get_frequent_write_files()?;
    if files.is_empty() {
        println!("❌ No files found. Are you in a Git repository?");
        return Ok(());
    }
    
    show_frequent_write_analysis(&files);
    show_critical_files(&files);
    show_gitignore_recommendations(&files);
    show_performance_impact(&files);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Files with frequent writes bloat Git history");
    println!("• Log files should never be tracked in Git");
    println!("• Cache and build files should be ignored");
    println!("• Some lock files should be tracked for reproducibility");
    println!("• Temporary files should always be ignored");
    println!("• Configuration files need careful consideration");
    
    Ok(())
}
