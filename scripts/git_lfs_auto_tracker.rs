use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct LfsCandidate {
    path: String,
    size: u64,
    mime_type: String,
    occurrences: u32,
    is_binary: bool,
    lfs_score: f64,
    recommendation: String,
}

#[derive(Debug)]
struct LfsAnalysis {
    candidates: Vec<LfsCandidate>,
    total_savings: u64,
    recommendations: Vec<String>,
    lfs_rules: Vec<String>,
}

fn analyze_lfs_candidates() -> Result<LfsAnalysis, Box<dyn std::error::Error>> {
    println!("📦 Analyzing Git LFS candidates...");
    
    // Get all files in the repository
    let output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let files_output = String::from_utf8(output.stdout)?;
    let mut candidates = Vec::new();
    let mut file_hashes = HashMap::new();
    
    for line in files_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let mode = parts[0];
            let hash = parts[1];
            let path = parts[3];
            
            // Get file size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;
            
            let size_str = String::from_utf8(size_output.stdout)?;
            let size: u64 = size_str.trim().parse().unwrap_or(0);
            
            // Count occurrences of this hash
            *file_hashes.entry(hash.to_string()).or_insert(0) += 1;
            
            // Check if file should be considered for LFS
            if should_consider_for_lfs(path, size) {
                let mime_type = detect_mime_type(path);
                let is_binary = is_binary_file(&mime_type);
                let occurrences = file_hashes[hash];
                
                let lfs_score = calculate_lfs_score(size, is_binary, occurrences);
                let recommendation = generate_recommendation(size, is_binary, occurrences, path);
                
                candidates.push(LfsCandidate {
                    path: path.to_string(),
                    size,
                    mime_type,
                    occurrences,
                    is_binary,
                    lfs_score,
                    recommendation,
                });
            }
        }
    }
    
    // Sort by LFS score
    candidates.sort_by(|a, b| b.lfs_score.partial_cmp(&a.lfs_score).unwrap());
    
    // Generate LFS rules
    let lfs_rules = generate_lfs_rules(&candidates);
    
    // Calculate total savings
    let total_savings: u64 = candidates.iter()
        .filter(|c| c.lfs_score > 0.7)
        .map(|c| c.size * c.occurrences as u64)
        .sum();
    
    // Generate recommendations
    let mut recommendations = Vec::new();
    if total_savings > 10 * 1024 * 1024 { // 10MB
        recommendations.push("Large repository - consider aggressive LFS tracking".to_string());
    }
    
    let binary_count = candidates.iter().filter(|c| c.is_binary).count();
    if binary_count > 10 {
        recommendations.push("Many binary files detected - implement LFS tracking".to_string());
    }
    
    let duplicate_count = candidates.iter().filter(|c| c.occurrences > 1).count();
    if duplicate_count > 5 {
        recommendations.push("Multiple duplicate files - LFS can help with deduplication".to_string());
    }
    
    Ok(LfsAnalysis {
        candidates,
        total_savings,
        recommendations,
        lfs_rules,
    })
}

fn should_consider_for_lfs(path: &str, size: u64) -> bool {
    // Size threshold: 1MB or larger
    if size < 1024 * 1024 {
        return false;
    }
    
    // Skip certain file types that shouldn't be in LFS
    let skip_extensions = [
        ".md", ".txt", ".json", ".yml", ".yaml", ".toml", 
        ".rs", ".py", ".js", ".ts", ".sh", ".bash"
    ];
    
    let path_lower = path.to_lowercase();
    for ext in &skip_extensions {
        if path_lower.ends_with(ext) {
            return false;
        }
    }
    
    true
}

fn detect_mime_type(path: &str) -> String {
    // Use file command to detect MIME type
    let output = Command::new("file")
        .args(&["--mime-type", "-b", path])
        .output();
    
    match output {
        Ok(output) => String::from_utf8(output.stdout).unwrap_or_else(|_| "unknown".to_string()),
        Err(_) => "unknown".to_string(),
    }
}

fn is_binary_file(mime_type: &str) -> bool {
    !mime_type.starts_with("text/") && 
    !mime_type.contains("json") && 
    !mime_type.contains("xml") &&
    !mime_type.contains("javascript") &&
    !mime_type.contains("css")
}

fn calculate_lfs_score(size: u64, is_binary: bool, occurrences: u32) -> f64 {
    let mut score = 0.0;
    
    // Size factor (0-0.4)
    if size >= 10 * 1024 * 1024 { // 10MB+
        score += 0.4;
    } else if size >= 5 * 1024 * 1024 { // 5MB+
        score += 0.3;
    } else if size >= 1024 * 1024 { // 1MB+
        score += 0.2;
    }
    
    // Binary factor (0-0.3)
    if is_binary {
        score += 0.3;
    }
    
    // Duplication factor (0-0.3)
    if occurrences > 5 {
        score += 0.3;
    } else if occurrences > 2 {
        score += 0.2;
    } else if occurrences > 1 {
        score += 0.1;
    }
    
    score
}

fn generate_recommendation(size: u64, is_binary: bool, occurrences: u32, path: &str) -> String {
    let mut reasons = Vec::new();
    
    if size >= 10 * 1024 * 1024 {
        reasons.push("very large file".to_string());
    } else if size >= 5 * 1024 * 1024 {
        reasons.push("large file".to_string());
    } else if size >= 1024 * 1024 {
        reasons.push("medium-large file".to_string());
    }
    
    if is_binary {
        reasons.push("binary content".to_string());
    }
    
    if occurrences > 1 {
        reasons.push(format!("appears {} times", occurrences));
    }
    
    if reasons.is_empty() {
        "consider for LFS".to_string()
    } else {
        format!("LFS recommended: {}", reasons.join(", "))
    }
}

fn generate_lfs_rules(candidates: &[LfsCandidate]) -> Vec<String> {
    let mut rules = Vec::new();
    
    // Group by extension
    let mut extension_groups: HashMap<String, Vec<&LfsCandidate>> = HashMap::new();
    
    for candidate in candidates {
        if candidate.lfs_score > 0.7 { // High confidence candidates
            if let Some(ext) = Path::new(&candidate.path).extension() {
                let ext_str = format!("*.{}", ext.to_string_lossy());
                extension_groups.entry(ext_str).or_default().push(candidate);
            }
        }
    }
    
    // Generate rules for extensions with multiple high-scoring files
    for (extension, files) in extension_groups {
        if files.len() >= 2 {
            rules.push(format!("{} filter=lfs diff=lfs merge=lfs -text", extension));
        }
    }
    
    // Add specific large files
    for candidate in candidates {
        if candidate.lfs_score > 0.8 && candidate.size > 5 * 1024 * 1024 {
            rules.push(format!("{} filter=lfs diff=lfs merge=lfs -text", candidate.path));
        }
    }
    
    rules
}

fn generate_lfs_report(analysis: &LfsAnalysis) {
    println!("\n📦 Git LFS Auto-Tracker Analysis");
    println!("==================================");
    
    // Show top candidates
    println!("\n🎯 Top LFS Candidates:");
    for (i, candidate) in analysis.candidates.iter().take(10).enumerate() {
        println!("  {}. {} ({:.1} MB)", i + 1, candidate.path, candidate.size as f64 / (1024.0 * 1024.0));
        println!("     Score: {:.1} | Type: {} | Occurrences: {}", 
            candidate.lfs_score, candidate.mime_type, candidate.occurrences);
        println!("     Recommendation: {}", candidate.recommendation);
        println!();
    }
    
    if analysis.candidates.len() > 10 {
        println!("  ... and {} more candidates", analysis.candidates.len() - 10);
    }
    
    // Show statistics
    println!("\n📊 Statistics:");
    println!("  • Total candidates: {}", analysis.candidates.len());
    println!("  • Binary files: {}", analysis.candidates.iter().filter(|c| c.is_binary).count());
    println!("  • Duplicate files: {}", analysis.candidates.iter().filter(|c| c.occurrences > 1).count());
    println!("  • Potential savings: {:.2} MB", analysis.total_savings as f64 / (1024.0 * 1024.0));
    
    // Show recommendations
    if !analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &analysis.recommendations {
            println!("  • {}", rec);
        }
    }
    
    // Show LFS rules
    if !analysis.lfs_rules.is_empty() {
        println!("\n📋 Suggested .gitattributes rules:");
        for rule in &analysis.lfs_rules {
            println!("  {}", rule);
        }
        
        println!("\n🔧 To apply these rules:");
        println!("  echo '{}' >> .gitattributes", analysis.lfs_rules.join("\n  "));
    }
    
    // Summary
    println!("\n📈 Summary:");
    if analysis.candidates.is_empty() {
        println!("  • No LFS candidates found");
        println!("  • Repository is well-optimized for Git storage");
    } else {
        println!("  • {} files recommended for LFS tracking", analysis.candidates.len());
        println!("  • Estimated savings: {:.2} MB", analysis.total_savings as f64 / (1024.0 * 1024.0));
        println!("  • Consider implementing LFS for binary files");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Git LFS Auto-Tracker");
    println!("=======================");
    println!("Automatically detecting files for Git LFS tracking...");
    println!();
    
    // Analyze LFS candidates
    let analysis = analyze_lfs_candidates()?;
    
    // Generate comprehensive report
    generate_lfs_report(&analysis);
    
    println!("\n✅ LFS analysis complete!");
    println!("🎯 Candidates identified");
    println!("📋 Rules generated");
    println!("💡 Recommendations ready");
    println!("🔧 Ready for implementation");
    
    Ok(())
}
