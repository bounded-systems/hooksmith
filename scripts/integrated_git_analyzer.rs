use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct GitFileInfo {
    path: String,
    size: u64,
    hash: String,
    file_type: String,
    extension: Option<String>,
}

#[derive(Debug)]
struct AnalysisConcern {
    concern_type: String,
    severity: String,
    message: String,
    file_path: Option<String>,
    recommendation: String,
}

#[derive(Debug)]
struct BlobSizeConcern {
    file_path: String,
    size: u64,
    category: String,
    impact: String,
    recommendation: String,
}

#[derive(Debug)]
struct DeduplicationConcern {
    hash: String,
    reuse_count: u32,
    paths: Vec<String>,
    efficiency: String,
}

#[derive(Debug)]
struct FileTypeConcern {
    extension: String,
    count: u32,
    total_size: u64,
    delta_friendly: bool,
    recommendation: String,
}

#[derive(Debug)]
struct FrequentWriteConcern {
    file_path: String,
    write_type: String,
    frequency: String,
    git_impact: String,
    recommendation: String,
}

fn get_git_files() -> Result<Vec<GitFileInfo>, Box<dyn std::error::Error>> {
    println!("📁 Gathering Git file information...");
    
    // Use git ls-files for current working tree
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut files = Vec::new();
    
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let mode = parts[0];
            let hash = parts[1];
            let stage = parts[2];
            let path = parts[3..].join(" ");
            
            // Skip if not in stage 0 (normal files)
            if stage != "0" {
                continue;
            }
            
            // Get file size using git cat-file
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let extension = if let Some(dot_pos) = path.rfind('.') {
                        Some(path[dot_pos..].to_string())
                    } else {
                        None
                    };
                    
                    let file_type = if mode.starts_with("100644") {
                        "blob".to_string()
                    } else if mode.starts_with("100755") {
                        "executable".to_string()
                    } else if mode.starts_with("120000") {
                        "symlink".to_string()
                    } else {
                        "other".to_string()
                    };
                    
                    files.push(GitFileInfo {
                        path,
                        size,
                        hash: hash.to_string(),
                        file_type,
                        extension,
                    });
                }
            }
        }
    }
    
    println!("📊 Found {} files in working tree", files.len());
    Ok(files)
}

fn analyze_blob_sizes(files: &[GitFileInfo]) -> Vec<BlobSizeConcern> {
    println!("📊 Analyzing blob sizes...");
    
    let mut concerns = Vec::new();
    let mut size_categories = HashMap::new();
    
    for file in files {
        let category = match file.size {
            0..=1023 => "tiny",
            1024..=8191 => "small", 
            8192..=204800 => "sweet_spot",
            204801..=1048576 => "large",
            _ => "huge",
        };
        
        size_categories.entry(category).or_insert_with(Vec::new).push(file);
    }
    
    // Generate concerns for files outside the sweet spot
    for file in files {
        let (impact, recommendation) = match file.size {
            0..=1023 => (
                "Low".to_string(),
                "Consider consolidating tiny files".to_string()
            ),
            1024..=8191 => (
                "Moderate".to_string(),
                "Good size for delta compression".to_string()
            ),
            8192..=204800 => (
                "Optimal".to_string(),
                "Perfect size for Git delta compression".to_string()
            ),
            204801..=1048576 => (
                "High".to_string(),
                "Consider if this large file should be in Git LFS".to_string()
            ),
            _ => (
                "Critical".to_string(),
                "Should be moved to Git LFS or external storage".to_string()
            ),
        };
        
        if file.size > 204800 || file.size < 1024 {
            concerns.push(BlobSizeConcern {
                file_path: file.path.clone(),
                size: file.size,
                category: match file.size {
                    0..=1023 => "tiny".to_string(),
                    1024..=8191 => "small".to_string(),
                    8192..=204800 => "sweet_spot".to_string(),
                    204801..=1048576 => "large".to_string(),
                    _ => "huge".to_string(),
                },
                impact,
                recommendation,
            });
        }
    }
    
    println!("📊 Found {} blob size concerns", concerns.len());
    concerns
}

fn analyze_deduplication(files: &[GitFileInfo]) -> Vec<DeduplicationConcern> {
    println!("🔄 Analyzing deduplication patterns...");
    
    let mut hash_groups: HashMap<String, Vec<&GitFileInfo>> = HashMap::new();
    
    for file in files {
        hash_groups.entry(file.hash.clone()).or_insert_with(Vec::new).push(file);
    }
    
    let mut concerns = Vec::new();
    
    for (hash, file_group) in hash_groups {
        if file_group.len() > 1 {
            let paths: Vec<String> = file_group.iter().map(|f| f.path.clone()).collect();
            let efficiency = if file_group.len() > 5 {
                "Excellent".to_string()
            } else if file_group.len() > 2 {
                "Good".to_string()
            } else {
                "Moderate".to_string()
            };
            
            concerns.push(DeduplicationConcern {
                hash,
                reuse_count: file_group.len() as u32,
                paths,
                efficiency,
            });
        }
    }
    
    println!("🔄 Found {} deduplication patterns", concerns.len());
    concerns
}

fn analyze_file_types(files: &[GitFileInfo]) -> Vec<FileTypeConcern> {
    println!("📁 Analyzing file types...");
    
    let mut extension_stats: HashMap<String, (u32, u64)> = HashMap::new();
    
    for file in files {
        if let Some(ref ext) = file.extension {
            let (count, total_size) = extension_stats.entry(ext.clone()).or_insert((0, 0));
            *count += 1;
            *total_size += file.size;
        }
    }
    
    let mut concerns = Vec::new();
    
    for (extension, (count, total_size)) in extension_stats {
        let delta_friendly = matches!(
            extension.as_str(),
            ".rs" | ".py" | ".js" | ".ts" | ".md" | ".txt" | ".json" | ".yml" | ".yaml" | ".toml"
        );
        
        let recommendation = if delta_friendly {
            "Good for delta compression".to_string()
        } else if extension.contains(".png") || extension.contains(".jpg") || extension.contains(".pdf") {
            "Consider Git LFS for binary files".to_string()
        } else if extension.contains(".zip") || extension.contains(".tar") {
            "Avoid versioning archives in Git".to_string()
        } else {
            "Monitor for appropriate handling".to_string()
        };
        
        concerns.push(FileTypeConcern {
            extension,
            count,
            total_size,
            delta_friendly,
            recommendation,
        });
    }
    
    println!("📁 Found {} file type categories", concerns.len());
    concerns
}

fn analyze_frequent_writes(files: &[GitFileInfo]) -> Vec<FrequentWriteConcern> {
    println!("📝 Analyzing frequent write patterns...");
    
    let mut concerns = Vec::new();
    
    for file in files {
        let path_lower = file.path.to_lowercase();
        
        let (write_type, frequency, git_impact, recommendation) = if path_lower.contains(".log") || 
                                                                   path_lower.contains(".out") || 
                                                                   path_lower.contains(".err") {
            ("Log".to_string(), "Very High".to_string(), "Critical".to_string(), 
             "Should be in .gitignore".to_string())
        } else if path_lower.contains("cache") || path_lower.contains("tmp") || path_lower.contains("temp") {
            ("Cache".to_string(), "High".to_string(), "Critical".to_string(),
             "Should be in .gitignore".to_string())
        } else if path_lower.contains("build") || path_lower.contains("dist") || path_lower.contains("target") {
            ("Build".to_string(), "High".to_string(), "Critical".to_string(),
             "Should be in .gitignore".to_string())
        } else if path_lower.contains(".tmp") || path_lower.contains(".temp") || path_lower.contains(".swp") {
            ("Temp".to_string(), "Very High".to_string(), "Critical".to_string(),
             "Should be in .gitignore".to_string())
        } else if path_lower.contains(".env") {
            ("Config".to_string(), "Moderate".to_string(), "High".to_string(),
             "Contains sensitive data - should be in .gitignore".to_string())
        } else if path_lower.contains("package-lock.json") || path_lower.contains("cargo.lock") {
            ("Lock".to_string(), "Moderate".to_string(), "Low".to_string(),
             "Should be tracked for reproducible builds".to_string())
        } else {
            continue; // Skip files that don't need concern
        };
        
        concerns.push(FrequentWriteConcern {
            file_path: file.path.clone(),
            write_type,
            frequency,
            git_impact,
            recommendation,
        });
    }
    
    println!("📝 Found {} frequent write concerns", concerns.len());
    concerns
}

fn generate_concerns_report(
    blob_concerns: &[BlobSizeConcern],
    dedup_concerns: &[DeduplicationConcern], 
    file_type_concerns: &[FileTypeConcern],
    frequent_write_concerns: &[FrequentWriteConcern],
) {
    println!("\n📊 Integrated Git Analysis Report");
    println!("================================");
    
    // Blob Size Concerns
    if !blob_concerns.is_empty() {
        println!("\n📊 Blob Size Concerns ({}):", blob_concerns.len());
        for concern in blob_concerns.iter().take(5) {
            println!("  • {} ({} bytes) - {}: {}", 
                concern.file_path, concern.size, concern.impact, concern.recommendation);
        }
        if blob_concerns.len() > 5 {
            println!("  ... and {} more", blob_concerns.len() - 5);
        }
    }
    
    // Deduplication Patterns
    if !dedup_concerns.is_empty() {
        println!("\n🔄 Deduplication Patterns ({}):", dedup_concerns.len());
        for concern in dedup_concerns.iter().take(3) {
            println!("  • {} reused {}x - {} efficiency", 
                concern.hash[..8].to_string(), concern.reuse_count, concern.efficiency);
            for path in concern.paths.iter().take(2) {
                println!("    - {}", path);
            }
            if concern.paths.len() > 2 {
                println!("    ... and {} more", concern.paths.len() - 2);
            }
        }
        if dedup_concerns.len() > 3 {
            println!("  ... and {} more patterns", dedup_concerns.len() - 3);
        }
    }
    
    // File Type Analysis
    if !file_type_concerns.is_empty() {
        println!("\n📁 File Type Analysis ({}):", file_type_concerns.len());
        for concern in file_type_concerns.iter().take(5) {
            let delta_status = if concern.delta_friendly { "✅" } else { "❌" };
            println!("  • {} {} files ({} bytes total) {}: {}", 
                concern.extension, concern.count, concern.total_size, delta_status, concern.recommendation);
        }
        if file_type_concerns.len() > 5 {
            println!("  ... and {} more types", file_type_concerns.len() - 5);
        }
    }
    
    // Frequent Write Concerns
    if !frequent_write_concerns.is_empty() {
        println!("\n📝 Frequent Write Concerns ({}):", frequent_write_concerns.len());
        for concern in frequent_write_concerns.iter().take(5) {
            println!("  • {} ({}) - {} frequency, {} impact: {}", 
                concern.file_path, concern.write_type, concern.frequency, concern.git_impact, concern.recommendation);
        }
        if frequent_write_concerns.len() > 5 {
            println!("  ... and {} more", frequent_write_concerns.len() - 5);
        }
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("  • Total files analyzed: {}", blob_concerns.len() + file_type_concerns.len());
    println!("  • Blob size concerns: {}", blob_concerns.len());
    println!("  • Deduplication patterns: {}", dedup_concerns.len());
    println!("  • File type categories: {}", file_type_concerns.len());
    println!("  • Frequent write concerns: {}", frequent_write_concerns.len());
    
    println!("\n💡 Recommendations:");
    if !blob_concerns.is_empty() {
        println!("  • Review large files for Git LFS consideration");
    }
    if !frequent_write_concerns.is_empty() {
        println!("  • Update .gitignore for frequent write files");
    }
    if dedup_concerns.len() > 10 {
        println!("  • Good deduplication - efficient storage");
    }
    println!("  • Use git repack -Ad for optimal packing");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Integrated Git Analysis Tool");
    println!("==============================");
    println!("Hooking into concerns pipeline with git ls-files...");
    println!();
    
    // Get file information using git ls-files (single analysis)
    let files = get_git_files()?;
    
    if files.is_empty() {
        println!("❌ No files found in Git working tree");
        return Ok(());
    }
    
    // Run all analyses on the same dataset
    let blob_concerns = analyze_blob_sizes(&files);
    let dedup_concerns = analyze_deduplication(&files);
    let file_type_concerns = analyze_file_types(&files);
    let frequent_write_concerns = analyze_frequent_writes(&files);
    
    // Generate comprehensive report
    generate_concerns_report(
        &blob_concerns,
        &dedup_concerns,
        &file_type_concerns,
        &frequent_write_concerns,
    );
    
    println!("\n✅ Analysis complete!");
    println!("📊 Single-pass analysis using git ls-files");
    println!("🔄 Integrated with concerns pipeline");
    println!("💡 All recommendations based on actual working tree");
    
    Ok(())
}
