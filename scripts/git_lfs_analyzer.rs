use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct LfsCandidate {
    path: String,
    size: u64,
    file_type: String,
    lfs_recommendation: String,
    reason: String,
}

#[derive(Debug, Clone)]
struct BinaryHookInfo {
    path: String,
    size: u64,
    hash: String,
    hook_type: String,
    potential_reuse: Vec<String>,
    reuse_score: f64,
}

#[derive(Debug)]
struct LfsAnalysisResult {
    candidates: Vec<LfsCandidate>,
    binary_hooks: Vec<BinaryHookInfo>,
    total_lfs_savings: u64,
    recommendations: Vec<String>,
}

fn get_lfs_candidates() -> Result<Vec<LfsCandidate>, Box<dyn std::error::Error>> {
    println!("🔍 Detecting Git LFS candidates...");
    
    // Use git ls-files for current working tree
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut candidates = Vec::new();
    
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
                    // Check if file is a candidate for Git LFS
                    let (should_track, reason) = analyze_lfs_candidate(&path, size);
                    
                    if should_track {
                        let file_type = determine_file_type(&path);
                        let lfs_recommendation = generate_lfs_recommendation(&path, size, &file_type);
                        
                        candidates.push(LfsCandidate {
                            path,
                            size,
                            file_type,
                            lfs_recommendation,
                            reason,
                        });
                    }
                }
            }
        }
    }
    
    println!("🔍 Found {} Git LFS candidates", candidates.len());
    Ok(candidates)
}

fn analyze_lfs_candidate(path: &str, size: u64) -> (bool, String) {
    let path_lower = path.to_lowercase();
    
    // Size-based criteria
    if size > 50 * 1024 * 1024 { // 50 MB
        return (true, "Very large file (>50 MB)".to_string());
    }
    
    if size > 10 * 1024 * 1024 { // 10 MB
        return (true, "Large file (>10 MB)".to_string());
    }
    
    // File type-based criteria
    if path_lower.contains(".psd") || path_lower.contains(".ai") || path_lower.contains(".sketch") {
        return (true, "Design file (Photoshop, Illustrator, Sketch)".to_string());
    }
    
    if path_lower.contains(".mp4") || path_lower.contains(".mov") || path_lower.contains(".avi") {
        return (true, "Video file".to_string());
    }
    
    if path_lower.contains(".wav") || path_lower.contains(".mp3") || path_lower.contains(".flac") {
        return (true, "Audio file".to_string());
    }
    
    if path_lower.contains(".zip") || path_lower.contains(".tar.gz") || path_lower.contains(".rar") {
        return (true, "Archive file".to_string());
    }
    
    if path_lower.contains(".iso") || path_lower.contains(".dmg") {
        return (true, "Disk image".to_string());
    }
    
    if path_lower.contains(".bin") || path_lower.contains(".exe") || path_lower.contains(".dll") {
        return (true, "Binary executable".to_string());
    }
    
    if path_lower.contains(".model") || path_lower.contains(".pkl") || path_lower.contains(".h5") {
        return (true, "Machine learning model".to_string());
    }
    
    if path_lower.contains(".pdf") && size > 5 * 1024 * 1024 { // 5 MB PDFs
        return (true, "Large PDF file".to_string());
    }
    
    if path_lower.contains(".png") && size > 2 * 1024 * 1024 { // 2 MB PNGs
        return (true, "Large image file".to_string());
    }
    
    if path_lower.contains(".jpg") && size > 5 * 1024 * 1024 { // 5 MB JPGs
        return (true, "Large image file".to_string());
    }
    
    (false, "".to_string())
}

fn determine_file_type(path: &str) -> String {
    let path_lower = path.to_lowercase();
    
    if path_lower.contains(".psd") || path_lower.contains(".ai") || path_lower.contains(".sketch") {
        "Design".to_string()
    } else if path_lower.contains(".mp4") || path_lower.contains(".mov") || path_lower.contains(".avi") {
        "Video".to_string()
    } else if path_lower.contains(".wav") || path_lower.contains(".mp3") || path_lower.contains(".flac") {
        "Audio".to_string()
    } else if path_lower.contains(".zip") || path_lower.contains(".tar.gz") || path_lower.contains(".rar") {
        "Archive".to_string()
    } else if path_lower.contains(".iso") || path_lower.contains(".dmg") {
        "Disk Image".to_string()
    } else if path_lower.contains(".bin") || path_lower.contains(".exe") || path_lower.contains(".dll") {
        "Binary".to_string()
    } else if path_lower.contains(".model") || path_lower.contains(".pkl") || path_lower.contains(".h5") {
        "ML Model".to_string()
    } else if path_lower.contains(".pdf") {
        "PDF".to_string()
    } else if path_lower.contains(".png") || path_lower.contains(".jpg") || path_lower.contains(".jpeg") {
        "Image".to_string()
    } else {
        "Other".to_string()
    }
}

fn generate_lfs_recommendation(path: &str, size: u64, file_type: &str) -> String {
    let extension = if let Some(dot_pos) = path.rfind('.') {
        &path[dot_pos..]
    } else {
        ""
    };
    
    format!("git lfs track \"*{}\"", extension)
}

fn analyze_binary_hooks() -> Result<Vec<BinaryHookInfo>, Box<dyn std::error::Error>> {
    println!("🔧 Analyzing binary hooks for reuse opportunities...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut binary_hooks = Vec::new();
    let mut hook_files: HashMap<String, Vec<String>> = HashMap::new();
    
    // First pass: collect all hook files
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            
            if path.contains(".hooksmith/hooks/") || path.contains("hooks/") {
                let hash = parts[1].to_string();
                hook_files.entry(hash).or_insert_with(Vec::new).push(path);
            }
        }
    }
    
    // Second pass: analyze binary hooks
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1].to_string();
            
            if path.contains(".hooksmith/hooks/") || path.contains("hooks/") {
                // Get file size
                let size_output = Command::new("git")
                    .args(&["cat-file", "-s", &hash])
                    .output()?;
                
                if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                    if let Ok(size) = u64::from_str(size_str.trim()) {
                        // Check if this is a binary file
                        if size > 1024 && is_binary_hook(&path) {
                            let hook_type = determine_hook_type(&path);
                            let potential_reuse = find_potential_reuse(&hash, &hook_files);
                            let reuse_score = calculate_reuse_score(&potential_reuse, size);
                            
                            binary_hooks.push(BinaryHookInfo {
                                path,
                                size,
                                hash,
                                hook_type,
                                potential_reuse,
                                reuse_score,
                            });
                        }
                    }
                }
            }
        }
    }
    
    println!("🔧 Found {} binary hooks", binary_hooks.len());
    Ok(binary_hooks)
}

fn is_binary_hook(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    
    // Check for binary extensions
    path_lower.contains(".exe") || 
    path_lower.contains(".dll") || 
    path_lower.contains(".so") || 
    path_lower.contains(".dylib") ||
    path_lower.contains(".bin") ||
    path_lower.contains(".wasm") ||
    path_lower.contains(".o") ||
    path_lower.contains(".a")
}

fn determine_hook_type(path: &str) -> String {
    let path_lower = path.to_lowercase();
    
    if path_lower.contains("pre-") {
        "Pre-hook".to_string()
    } else if path_lower.contains("post-") {
        "Post-hook".to_string()
    } else if path_lower.contains("github") {
        "GitHub Hook".to_string()
    } else if path_lower.contains("git") {
        "Git Hook".to_string()
    } else {
        "Generic Hook".to_string()
    }
}

fn find_potential_reuse(hash: &str, hook_files: &HashMap<String, Vec<String>>) -> Vec<String> {
    if let Some(paths) = hook_files.get(hash) {
        paths.clone()
    } else {
        Vec::new()
    }
}

fn calculate_reuse_score(potential_reuse: &[String], size: u64) -> f64 {
    let reuse_count = potential_reuse.len() as f64;
    let size_factor = (size as f64 / (1024.0 * 1024.0)).min(10.0); // Cap at 10 MB factor
    
    reuse_count * size_factor
}

fn generate_lfs_analysis_report(candidates: &[LfsCandidate], binary_hooks: &[BinaryHookInfo]) -> LfsAnalysisResult {
    println!("\n📦 Git LFS Analysis Report");
    println!("==========================");
    
    let total_lfs_savings: u64 = candidates.iter().map(|c| c.size).sum();
    let mut recommendations = Vec::new();
    
    // LFS Candidates
    if !candidates.is_empty() {
        println!("\n🔍 Git LFS Candidates ({}):", candidates.len());
        for candidate in candidates.iter().take(10) {
            println!("  • {} ({} bytes) - {}: {}", 
                candidate.path, candidate.size, candidate.file_type, candidate.reason);
            println!("    Recommendation: {}", candidate.lfs_recommendation);
        }
        if candidates.len() > 10 {
            println!("  ... and {} more candidates", candidates.len() - 10);
        }
        
        recommendations.push(format!("Track {} files with Git LFS", candidates.len()));
        recommendations.push("Add .gitattributes rules for detected file types".to_string());
    }
    
    // Binary Hooks Analysis
    if !binary_hooks.is_empty() {
        println!("\n🔧 Binary Hook Reuse Opportunities ({}):", binary_hooks.len());
        
        // Sort by reuse score
        let mut sorted_hooks = binary_hooks.to_vec();
        sorted_hooks.sort_by(|a, b| b.reuse_score.partial_cmp(&a.reuse_score).unwrap());
        
        for hook in sorted_hooks.iter().take(5) {
            println!("  • {} ({} bytes) - {} (reuse score: {:.2})", 
                hook.path, hook.size, hook.hook_type, hook.reuse_score);
            if !hook.potential_reuse.is_empty() {
                println!("    Potential reuse in: {}", hook.potential_reuse.join(", "));
            }
        }
        
        if binary_hooks.len() > 5 {
            println!("  ... and {} more binary hooks", binary_hooks.len() - 5);
        }
        
        recommendations.push("Consider consolidating similar binary hooks".to_string());
        recommendations.push("Use symbolic links for identical binary hooks".to_string());
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("  • Git LFS candidates: {}", candidates.len());
    println!("  • Binary hooks: {}", binary_hooks.len());
    println!("  • Potential LFS savings: {:.2} MB", total_lfs_savings as f64 / (1024.0 * 1024.0));
    
    if !recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &recommendations {
            println!("  • {}", rec);
        }
    }
    
    LfsAnalysisResult {
        candidates: candidates.to_vec(),
        binary_hooks: binary_hooks.to_vec(),
        total_lfs_savings,
        recommendations,
    }
}

fn generate_gitattributes_template(candidates: &[LfsCandidate]) -> String {
    let mut file_types: HashMap<String, Vec<String>> = HashMap::new();
    
    for candidate in candidates {
        if let Some(dot_pos) = candidate.path.rfind('.') {
            let extension = candidate.path[dot_pos..].to_string();
            file_types.entry(extension).or_insert_with(Vec::new).push(candidate.path.clone());
        }
    }
    
    let mut gitattributes = String::from("# Git LFS tracking rules\n");
    gitattributes.push_str("# Generated by Git LFS Analyzer\n\n");
    
    for (extension, _) in file_types {
        gitattributes.push_str(&format!("*{} filter=lfs diff=lfs merge=lfs -text\n", extension));
    }
    
    gitattributes
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Git LFS Analyzer");
    println!("===================");
    println!("Detecting large files and binary hook reuse opportunities...");
    println!();
    
    // Get LFS candidates
    let lfs_candidates = get_lfs_candidates()?;
    
    // Analyze binary hooks
    let binary_hooks = analyze_binary_hooks()?;
    
    // Generate comprehensive report
    let analysis_result = generate_lfs_analysis_report(&lfs_candidates, &binary_hooks);
    
    // Generate .gitattributes template if needed
    if !lfs_candidates.is_empty() {
        println!("\n📝 Suggested .gitattributes rules:");
        println!("================================");
        println!("{}", generate_gitattributes_template(&lfs_candidates));
    }
    
    println!("\n✅ Analysis complete!");
    println!("📦 Git LFS integration ready");
    println!("🔧 Binary hook optimization opportunities identified");
    println!("💡 Recommendations for repository optimization");
    
    Ok(())
}
