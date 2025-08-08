use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
enum FileTypeCategory {
    TextCode,      // .rs, .py, .ts, .js, .sh, .go, .java, .cpp
    TextData,      // .json, .yml, .yaml, .xml, .toml, .ini
    Documentation, // .md, .txt, .rst, .adoc
    BinaryAsset,   // .png, .jpg, .gif, .pdf, .woff, .ttf
    BinaryCode,    // .exe, .dll, .so, .dylib, .jar, .war
    Archive,       // .zip, .tar.gz, .rar, .7z
    Database,      // .sqlite, .db, .sql
    Generated,     // build outputs, generated code
    Other,
}

#[derive(Debug, Clone)]
struct FileTypeInfo {
    category: FileTypeCategory,
    extension: String,
    count: u32,
    total_size: u64,
    avg_size: u64,
    delta_compression: DeltaCompression,
    git_recommendation: String,
    performance_impact: PerformanceImpact,
}

#[derive(Debug, Clone)]
enum DeltaCompression {
    Excellent,  // Text files with high similarity
    Good,       // Text files with moderate similarity
    Poor,       // Binary files with some patterns
    None,       // Opaque binary, no delta benefit
}

#[derive(Debug, Clone)]
enum PerformanceImpact {
    Low,        // Small files, minimal impact
    Moderate,   // Medium files, some impact
    High,       // Large files, significant impact
    Critical,   // Very large files, major impact
}

impl FileTypeCategory {
    fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_lowercase();
        
        // Text Code
        if matches!(ext_lower.as_str(), 
            ".rs" | ".py" | ".ts" | ".js" | ".sh" | ".go" | ".java" | ".cpp" | ".c" | ".h" |
            ".cs" | ".php" | ".rb" | ".pl" | ".lua" | ".swift" | ".kt" | ".scala") {
            FileTypeCategory::TextCode
        }
        // Text Data
        else if matches!(ext_lower.as_str(), 
            ".json" | ".yml" | ".yaml" | ".xml" | ".toml" | ".ini" | ".cfg" | ".conf" |
            ".csv" | ".tsv" | ".log" | ".lock") {
            FileTypeCategory::TextData
        }
        // Documentation
        else if matches!(ext_lower.as_str(), 
            ".md" | ".txt" | ".rst" | ".adoc" | ".tex" | ".html" | ".htm" | ".css" |
            ".scss" | ".less" | ".sass") {
            FileTypeCategory::Documentation
        }
        // Binary Assets
        else if matches!(ext_lower.as_str(), 
            ".png" | ".jpg" | ".jpeg" | ".gif" | ".bmp" | ".ico" | ".svg" | ".pdf" |
            ".woff" | ".woff2" | ".ttf" | ".otf" | ".eot" | ".mp3" | ".mp4" | ".avi" |
            ".mov" | ".wmv" | ".flv" | ".webm") {
            FileTypeCategory::BinaryAsset
        }
        // Binary Code
        else if matches!(ext_lower.as_str(), 
            ".exe" | ".dll" | ".so" | ".dylib" | ".jar" | ".war" | ".ear" | ".apk" |
            ".deb" | ".rpm" | ".msi" | ".app" | ".bin") {
            FileTypeCategory::BinaryCode
        }
        // Archive
        else if matches!(ext_lower.as_str(), 
            ".zip" | ".tar.gz" | ".tar.bz2" | ".rar" | ".7z" | ".gz" | ".bz2" | ".xz" |
            ".lzma" | ".tar" | ".tgz") {
            FileTypeCategory::Archive
        }
        // Database
        else if matches!(ext_lower.as_str(), 
            ".sqlite" | ".sqlite3" | ".db" | ".sql" | ".mdb" | ".accdb") {
            FileTypeCategory::Database
        }
        // Generated
        else if matches!(ext_lower.as_str(), 
            ".generated.rs" | ".generated.py" | ".min.js" | ".min.css" | ".map" |
            ".wasm" | ".o" | ".obj" | ".a" | ".lib") {
            FileTypeCategory::Generated
        }
        else {
            FileTypeCategory::Other
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            FileTypeCategory::TextCode => "🦀",
            FileTypeCategory::TextData => "📊",
            FileTypeCategory::Documentation => "📚",
            FileTypeCategory::BinaryAsset => "🖼️",
            FileTypeCategory::BinaryCode => "📦",
            FileTypeCategory::Archive => "🗜️",
            FileTypeCategory::Database => "🗄️",
            FileTypeCategory::Generated => "🔧",
            FileTypeCategory::Other => "📄",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            FileTypeCategory::TextCode => "Source code files",
            FileTypeCategory::TextData => "Structured data files",
            FileTypeCategory::Documentation => "Documentation files",
            FileTypeCategory::BinaryAsset => "Binary assets (images, fonts, media)",
            FileTypeCategory::BinaryCode => "Binary executables and libraries",
            FileTypeCategory::Archive => "Compressed archives",
            FileTypeCategory::Database => "Database files",
            FileTypeCategory::Generated => "Generated/build artifacts",
            FileTypeCategory::Other => "Other files",
        }
    }

    fn delta_compression(&self) -> DeltaCompression {
        match self {
            FileTypeCategory::TextCode => DeltaCompression::Excellent,
            FileTypeCategory::TextData => DeltaCompression::Good,
            FileTypeCategory::Documentation => DeltaCompression::Excellent,
            FileTypeCategory::BinaryAsset => DeltaCompression::Poor,
            FileTypeCategory::BinaryCode => DeltaCompression::None,
            FileTypeCategory::Archive => DeltaCompression::None,
            FileTypeCategory::Database => DeltaCompression::Poor,
            FileTypeCategory::Generated => DeltaCompression::Poor,
            FileTypeCategory::Other => DeltaCompression::Good,
        }
    }

    fn git_recommendation(&self, avg_size: u64) -> String {
        match self {
            FileTypeCategory::TextCode => {
                if avg_size > 100 * 1024 {
                    "Consider splitting large modules".to_string()
                } else {
                    "Good for delta compression".to_string()
                }
            }
            FileTypeCategory::TextData => {
                if avg_size > 50 * 1024 {
                    "Consider breaking into smaller chunks".to_string()
                } else {
                    "Good for delta compression".to_string()
                }
            }
            FileTypeCategory::Documentation => {
                "Excellent for delta compression".to_string()
            }
            FileTypeCategory::BinaryAsset => {
                if avg_size > 1024 * 1024 {
                    "Use Git LFS for large assets".to_string()
                } else {
                    "Consider excluding from delta compression".to_string()
                }
            }
            FileTypeCategory::BinaryCode => {
                "Should be excluded from Git or use Git LFS".to_string()
            }
            FileTypeCategory::Archive => {
                "Should be excluded from Git or use external storage".to_string()
            }
            FileTypeCategory::Database => {
                "Should be excluded from Git or use Git LFS".to_string()
            }
            FileTypeCategory::Generated => {
                "Should be added to .gitignore".to_string()
            }
            FileTypeCategory::Other => {
                "Monitor for appropriate handling".to_string()
            }
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

fn get_file_type_analysis() -> Result<Vec<FileTypeInfo>, Box<dyn std::error::Error>> {
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
            
            // Skip common ignored directories
            if path.contains("target/") || path.contains(".cargo/") || path.contains("node_modules/") {
                continue;
            }
            
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
                    
                    let category = FileTypeCategory::from_extension(&extension);
                    
                    let file_type = file_types.entry(extension.clone()).or_insert(FileTypeInfo {
                        category: category.clone(),
                        extension,
                        count: 0,
                        total_size: 0,
                        avg_size: 0,
                        delta_compression: category.delta_compression(),
                        git_recommendation: String::new(),
                        performance_impact: PerformanceImpact::Low,
                    });
                    
                    file_type.count += 1;
                    file_type.total_size += size;
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
        
        file_type.git_recommendation = file_type.category.git_recommendation(file_type.avg_size);
        file_type.performance_impact = calculate_performance_impact(file_type.avg_size);
    }
    
    Ok(file_types.into_values().collect())
}

fn calculate_performance_impact(avg_size: u64) -> PerformanceImpact {
    match avg_size {
        0..=10*1024 => PerformanceImpact::Low,
        10*1024..=100*1024 => PerformanceImpact::Moderate,
        100*1024..=1024*1024 => PerformanceImpact::High,
        _ => PerformanceImpact::Critical,
    }
}

fn show_file_type_analysis(file_types: &[FileTypeInfo]) {
    println!("📊 File Type Analysis by Category:");
    println!("==================================");
    
    // Group by category
    let mut by_category: HashMap<FileTypeCategory, Vec<&FileTypeInfo>> = HashMap::new();
    for ft in file_types {
        by_category.entry(ft.category.clone()).or_insert_with(Vec::new).push(ft);
    }
    
    for category in [
        FileTypeCategory::TextCode,
        FileTypeCategory::TextData,
        FileTypeCategory::Documentation,
        FileTypeCategory::BinaryAsset,
        FileTypeCategory::BinaryCode,
        FileTypeCategory::Archive,
        FileTypeCategory::Database,
        FileTypeCategory::Generated,
        FileTypeCategory::Other,
    ] {
        if let Some(files) = by_category.get(&category) {
            let total_count: u32 = files.iter().map(|f| f.count).sum();
            let total_size: u64 = files.iter().map(|f| f.total_size).sum();
            let avg_size = if total_count > 0 { total_size / total_count } else { 0 };
            
            println!("{} {}:", category.emoji(), category.description());
            println!("   Files: {}", total_count);
            println!("   Total size: {}", format_size(total_size));
            println!("   Average size: {}", format_size(avg_size));
            println!("   Delta compression: {:?}", files[0].delta_compression);
            println!("   Recommendation: {}", files[0].git_recommendation);
            println!();
        }
    }
}

fn show_delta_compression_analysis(file_types: &[FileTypeInfo]) {
    println!("🔗 Delta Compression Analysis:");
    println!("==============================");
    
    let mut excellent_count = 0;
    let mut good_count = 0;
    let mut poor_count = 0;
    let mut none_count = 0;
    
    for ft in file_types {
        match ft.delta_compression {
            DeltaCompression::Excellent => excellent_count += ft.count,
            DeltaCompression::Good => good_count += ft.count,
            DeltaCompression::Poor => poor_count += ft.count,
            DeltaCompression::None => none_count += ft.count,
        }
    }
    
    let total_files: u32 = file_types.iter().map(|ft| ft.count).sum();
    
    println!("Excellent delta compression: {} files ({:.1}%)", 
        excellent_count, if total_files > 0 { (excellent_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Good delta compression: {} files ({:.1}%)", 
        good_count, if total_files > 0 { (good_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Poor delta compression: {} files ({:.1}%)", 
        poor_count, if total_files > 0 { (poor_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("No delta compression: {} files ({:.1}%)", 
        none_count, if total_files > 0 { (none_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!();
}

fn show_performance_analysis(file_types: &[FileTypeInfo]) {
    println!("⚡ Performance Impact Analysis:");
    println!("==============================");
    
    let mut low_count = 0;
    let mut moderate_count = 0;
    let mut high_count = 0;
    let mut critical_count = 0;
    
    for ft in file_types {
        match ft.performance_impact {
            PerformanceImpact::Low => low_count += ft.count,
            PerformanceImpact::Moderate => moderate_count += ft.count,
            PerformanceImpact::High => high_count += ft.count,
            PerformanceImpact::Critical => critical_count += ft.count,
        }
    }
    
    let total_files: u32 = file_types.iter().map(|ft| ft.count).sum();
    
    println!("Low impact: {} files ({:.1}%)", 
        low_count, if total_files > 0 { (low_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Moderate impact: {} files ({:.1}%)", 
        moderate_count, if total_files > 0 { (moderate_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("High impact: {} files ({:.1}%)", 
        high_count, if total_files > 0 { (high_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!("Critical impact: {} files ({:.1}%)", 
        critical_count, if total_files > 0 { (critical_count as f64 / total_files as f64) * 100.0 } else { 0.0 });
    println!();
}

fn show_optimization_recommendations(file_types: &[FileTypeInfo]) {
    println!("⚙️  Optimization Recommendations:");
    println!("===============================");
    
    // Find problematic file types
    let large_binaries: Vec<&FileTypeInfo> = file_types.iter()
        .filter(|ft| ft.category == FileTypeCategory::BinaryAsset && ft.avg_size > 1024 * 1024)
        .collect();
    
    let archives: Vec<&FileTypeInfo> = file_types.iter()
        .filter(|ft| ft.category == FileTypeCategory::Archive)
        .collect();
    
    let generated: Vec<&FileTypeInfo> = file_types.iter()
        .filter(|ft| ft.category == FileTypeCategory::Generated)
        .collect();
    
    if !large_binaries.is_empty() {
        println!("🔧 Large binary assets detected:");
        for ft in large_binaries {
            println!("   • {} files with avg size {}", ft.extension, format_size(ft.avg_size));
        }
        println!("   Consider using Git LFS for these file types");
    }
    
    if !archives.is_empty() {
        println!("🔧 Archive files detected:");
        for ft in archives {
            println!("   • {} files with avg size {}", ft.extension, format_size(ft.avg_size));
        }
        println!("   Consider excluding from Git or using external storage");
    }
    
    if !generated.is_empty() {
        println!("🔧 Generated files detected:");
        for ft in generated {
            println!("   • {} files with avg size {}", ft.extension, format_size(ft.avg_size));
        }
        println!("   Consider adding to .gitignore");
    }
    
    println!();
    println!("🚀 Pro Tips:");
    println!("• Text files (.rs, .py, .md) compress well with deltas");
    println!("• Binary files (.png, .exe) should use Git LFS if large");
    println!("• Archives (.zip, .tar.gz) should be excluded from Git");
    println!("• Generated files should be in .gitignore");
    println!("• Keep files under 512KB for optimal delta compression");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 File Type Analysis for Git Optimization");
    println!("==========================================");
    println!();
    
    let file_types = get_file_type_analysis()?;
    if file_types.is_empty() {
        println!("❌ No files found. Are you in a Git repository?");
        return Ok(());
    }
    
    show_file_type_analysis(&file_types);
    show_delta_compression_analysis(&file_types);
    show_performance_analysis(&file_types);
    show_optimization_recommendations(&file_types);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Text files (.rs, .md, .json) compress well with deltas");
    println!("• Binary files (.png, .exe) should use Git LFS if large");
    println!("• Archives (.zip, .tar.gz) should be excluded from Git");
    println!("• Generated files should be in .gitignore");
    println!("• Files >512KB are often skipped for delta compression");
    println!("• Multiple identical files cost almost nothing (deduplication)");
    
    Ok(())
}
