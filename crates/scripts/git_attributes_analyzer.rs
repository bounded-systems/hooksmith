use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct FileTypeInfo {
    extension: String,
    count: u32,
    total_size: u64,
    avg_size: u64,
    delta_excluded: u32,
    compression_ratio: f64,
    recommendation: String,
}

#[derive(Debug, Clone)]
struct GitAttributeRule {
    pattern: String,
    attribute: String,
    value: String,
    reason: String,
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

fn get_file_extensions() -> Result<Vec<FileTypeInfo>, Box<dyn std::error::Error>> {
    let mut file_types: HashMap<String, FileTypeInfo> = HashMap::new();
    
    // Get all files with their sizes
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
                    // Extract file extension
                    let extension = if let Some(dot_pos) = path.rfind('.') {
                        path[dot_pos..].to_string()
                    } else {
                        "(no extension)".to_string()
                    };
                    
                    let file_type = file_types.entry(extension.clone()).or_insert(FileTypeInfo {
                        extension,
                        count: 0,
                        total_size: 0,
                        avg_size: 0,
                        delta_excluded: 0,
                        compression_ratio: 0.0,
                        recommendation: String::new(),
                    });
                    
                    file_type.count += 1;
                    file_type.total_size += size;
                    
                    // Check if this file type should be excluded from delta compression
                    if should_exclude_from_delta(&path, size) {
                        file_type.delta_excluded += 1;
                    }
                }
            }
        }
    }
    
    // Calculate averages and recommendations
    for file_type in file_types.values_mut() {
        file_type.avg_size = if file_type.count > 0 {
            file_type.total_size / file_type.count
        } else {
            0
        };
        
        file_type.compression_ratio = calculate_compression_ratio(&file_type.extension);
        file_type.recommendation = generate_recommendation(file_type);
    }
    
    Ok(file_types.into_values().collect())
}

fn should_exclude_from_delta(path: &str, size: u64) -> bool {
    // Large files (>1MB) are often excluded
    if size > 1024 * 1024 {
        return true;
    }
    
    // Common compressed/binary formats
    let compressed_extensions = [
        ".zip", ".tar.gz", ".tar.bz2", ".rar", ".7z",
        ".gz", ".bz2", ".xz", ".lzma",
        ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico",
        ".mp3", ".mp4", ".avi", ".mov", ".wmv",
        ".pdf", ".doc", ".docx", ".xls", ".xlsx",
        ".iso", ".bin", ".exe", ".dll", ".so", ".dylib",
        ".jar", ".war", ".ear", ".apk",
        ".db", ".sqlite", ".sqlite3",
    ];
    
    let path_lower = path.to_lowercase();
    for ext in &compressed_extensions {
        if path_lower.ends_with(ext) {
            return true;
        }
    }
    
    false
}

fn calculate_compression_ratio(extension: &str) -> f64 {
    // Estimate compression ratios based on file type
    match extension.to_lowercase().as_str() {
        ".txt" | ".md" | ".rst" | ".adoc" => 0.7, // Text files compress well
        ".json" | ".xml" | ".yaml" | ".yml" => 0.6, // Structured text
        ".rs" | ".py" | ".js" | ".ts" | ".java" | ".cpp" | ".c" => 0.5, // Source code
        ".html" | ".css" | ".scss" | ".less" => 0.4, // Web files
        ".zip" | ".tar.gz" | ".rar" => 0.0, // Already compressed
        ".png" | ".jpg" | ".gif" => 0.1, // Images (minimal compression)
        ".mp3" | ".mp4" | ".avi" => 0.05, // Media files
        ".pdf" | ".doc" | ".xls" => 0.2, // Documents
        ".exe" | ".dll" | ".so" => 0.1, // Binaries
        _ => 0.3, // Default
    }
}

fn generate_recommendation(file_type: &FileTypeInfo) -> String {
    let avg_size_mb = file_type.avg_size as f64 / (1024.0 * 1024.0);
    
    if file_type.extension == "(no extension)" {
        return "Consider adding file extensions for better analysis".to_string();
    }
    
    if file_type.count == 0 {
        return "No files found".to_string();
    }
    
    // Large files should be excluded from delta compression
    if avg_size_mb > 1.0 {
        return format!("Exclude from delta: {} -delta", file_type.extension);
    }
    
    // Already compressed files
    if file_type.compression_ratio < 0.1 {
        return format!("Exclude from delta: {} -delta", file_type.extension);
    }
    
    // Small files that don't benefit from delta compression
    if file_type.avg_size < 1024 {
        return "Small files - delta compression not beneficial".to_string();
    }
    
    // Good candidates for delta compression
    if file_type.compression_ratio > 0.4 && file_type.avg_size > 8192 {
        return "Good candidate for delta compression".to_string();
    }
    
    "Standard delta compression".to_string()
}

fn get_current_gitattributes() -> Result<Vec<GitAttributeRule>, Box<dyn std::error::Error>> {
    let mut rules = Vec::new();
    
    // Check if .gitattributes exists
    let attr_output = Command::new("git")
        .args(&["check-attr", "-a", "--", "src/"])
        .output();
    
    if attr_output.is_ok() {
        // Parse existing .gitattributes rules
        let attr_content = std::fs::read_to_string(".gitattributes").unwrap_or_default();
        
        for line in attr_content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                rules.push(GitAttributeRule {
                    pattern: parts[0].to_string(),
                    attribute: parts[1].to_string(),
                    value: parts.get(2).unwrap_or(&"").to_string(),
                    reason: "Existing rule".to_string(),
                });
            }
        }
    }
    
    Ok(rules)
}

fn show_file_type_analysis(file_types: &[FileTypeInfo]) {
    println!("📊 File Type Analysis:");
    println!("======================");
    
    // Sort by total size
    let mut sorted_types: Vec<&FileTypeInfo> = file_types.iter().collect();
    sorted_types.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    
    for file_type in sorted_types.iter().take(15) {
        let excluded_percentage = if file_type.count > 0 {
            (file_type.delta_excluded as f64 / file_type.count as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{} {}:", 
            if file_type.extension == "(no extension)" { "📄" } else { "📁" },
            file_type.extension);
        println!("   Count: {}", file_type.count);
        println!("   Total size: {}", format_size(file_type.total_size));
        println!("   Average size: {}", format_size(file_type.avg_size));
        println!("   Delta excluded: {} ({:.1}%)", file_type.delta_excluded, excluded_percentage);
        println!("   Compression ratio: {:.1}%", file_type.compression_ratio * 100.0);
        println!("   Recommendation: {}", file_type.recommendation);
        println!();
    }
}

fn show_recommended_attributes(file_types: &[FileTypeInfo]) {
    println!("⚙️  Recommended .gitattributes Rules:");
    println!("====================================");
    
    let mut recommendations = Vec::new();
    
    for file_type in file_types {
        if file_type.recommendation.contains("-delta") {
            recommendations.push(format!("{} -delta", file_type.extension));
        }
    }
    
    if recommendations.is_empty() {
        println!("No specific delta exclusions recommended.");
        println!("Current file types are well-suited for delta compression.");
    } else {
        println!("Add these rules to .gitattributes:");
        println!();
        for rec in recommendations {
            println!("{}", rec);
        }
    }
    println!();
}

fn show_optimization_insights(file_types: &[FileTypeInfo]) {
    println!("💡 Optimization Insights:");
    println!("========================");
    
    let total_files: u32 = file_types.iter().map(|ft| ft.count).sum();
    let total_size: u64 = file_types.iter().map(|ft| ft.total_size).sum();
    let excluded_files: u32 = file_types.iter().map(|ft| ft.delta_excluded).sum();
    
    println!("Total files: {}", total_files);
    println!("Total size: {}", format_size(total_size));
    println!("Files excluded from delta: {} ({:.1}%)", 
        excluded_files,
        if total_files > 0 { (excluded_files as f64 / total_files as f64) * 100.0 } else { 0.0 });
    
    // Find large file types
    let large_types: Vec<&FileTypeInfo> = file_types
        .iter()
        .filter(|ft| ft.avg_size > 1024 * 1024)
        .collect();
    
    if !large_types.is_empty() {
        println!();
        println!("🔧 Large file types detected:");
        for ft in large_types {
            println!("   • {} (avg: {})", ft.extension, format_size(ft.avg_size));
        }
        println!("   Consider Git LFS for these file types");
    }
    
    // Find compressed file types
    let compressed_types: Vec<&FileTypeInfo> = file_types
        .iter()
        .filter(|ft| ft.compression_ratio < 0.1)
        .collect();
    
    if !compressed_types.is_empty() {
        println!();
        println!("🔧 Already compressed file types:");
        for ft in compressed_types {
            println!("   • {} (compression: {:.1}%)", ft.extension, ft.compression_ratio * 100.0);
        }
        println!("   These should be excluded from delta compression");
    }
    
    println!();
    println!("🚀 Pro Tips:");
    println!("• Use '*.zip -delta' for compressed archives");
    println!("• Use '*.png -delta' for images");
    println!("• Use '*.pdf -delta' for documents");
    println!("• Use '*.exe -delta' for binaries");
    println!("• Consider Git LFS for files > 1 MB");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Git Attributes Analysis for Delta Compression");
    println!("================================================");
    println!();
    
    let file_types = get_file_extensions()?;
    if file_types.is_empty() {
        println!("❌ No files found. Are you in a Git repository?");
        return Ok(());
    }
    
    show_file_type_analysis(&file_types);
    show_recommended_attributes(&file_types);
    show_optimization_insights(&file_types);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Large files (>1MB) should be excluded from delta compression");
    println!("• Already compressed files (zip, png, etc.) don't benefit from deltas");
    println!("• Small files (<1KB) are often stored raw anyway");
    println!("• Text files and source code compress well with deltas");
    println!("• Use .gitattributes to control delta compression per file type");
    
    Ok(())
}
