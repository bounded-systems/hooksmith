use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct ModularizationCandidate {
    path: String,
    size: u64,
    similarity_score: f64,
    similar_files: Vec<String>,
    modularization_type: String,
    recommendation: String,
}

#[derive(Debug, Clone)]
struct DeltaCompressionGroup {
    base_file: String,
    delta_files: Vec<String>,
    compression_ratio: f64,
    total_size: u64,
    compressed_size: u64,
}

#[derive(Debug, Clone)]
struct CodePattern {
    pattern_type: String,
    files: Vec<String>,
    repetition_count: u32,
    potential_savings: u64,
}

#[derive(Debug)]
struct ModularizationAnalysis {
    candidates: Vec<ModularizationCandidate>,
    delta_groups: Vec<DeltaCompressionGroup>,
    code_patterns: Vec<CodePattern>,
    total_potential_savings: u64,
    recommendations: Vec<String>,
}

fn analyze_modularization_candidates() -> Result<Vec<ModularizationCandidate>, Box<dyn std::error::Error>> {
    println!("🔍 Analyzing modularization candidates...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut candidates = Vec::new();
    let mut file_groups: HashMap<String, Vec<String>> = HashMap::new();
    
    // First pass: group files by extension and size
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1];
            
            // Focus on code files
            if is_code_file(&path) {
                let size_output = Command::new("git")
                    .args(&["cat-file", "-s", hash])
                    .output()?;
                
                if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                    if let Ok(size) = u64::from_str(size_str.trim()) {
                        let extension = get_file_extension(&path);
                        let key = format!("{}:{}", extension, size_category(size));
                        file_groups.entry(key).or_insert_with(Vec::new).push(path);
                    }
                }
            }
        }
    }
    
    // Second pass: analyze candidates
    for (group_key, files) in file_groups {
        if files.len() > 1 {
            let (mod_type, recommendation) = analyze_modularization_type(&group_key, &files);
            let similarity_score = calculate_similarity_score(&files);
            
            for file in &files {
                candidates.push(ModularizationCandidate {
                    path: file.clone(),
                    size: get_file_size(file)?,
                    similarity_score,
                    similar_files: files.iter().filter(|&f| f != file).cloned().collect(),
                    modularization_type: mod_type.clone(),
                    recommendation: recommendation.clone(),
                });
            }
        }
    }
    
    println!("🔍 Found {} modularization candidates", candidates.len());
    Ok(candidates)
}

fn is_code_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.contains(".rs") || 
    path_lower.contains(".py") || 
    path_lower.contains(".js") || 
    path_lower.contains(".ts") || 
    path_lower.contains(".go") || 
    path_lower.contains(".java") || 
    path_lower.contains(".cpp") || 
    path_lower.contains(".c") ||
    path_lower.contains(".h") ||
    path_lower.contains(".hpp")
}

fn get_file_extension(path: &str) -> String {
    if let Some(dot_pos) = path.rfind('.') {
        path[dot_pos..].to_string()
    } else {
        "".to_string()
    }
}

fn size_category(size: u64) -> String {
    match size {
        0..=1024 => "tiny".to_string(),
        1025..=10240 => "small".to_string(),
        10241..=102400 => "medium".to_string(),
        102401..=1024000 => "large".to_string(),
        _ => "huge".to_string(),
    }
}

fn analyze_modularization_type(group_key: &str, files: &[String]) -> (String, String) {
    let parts: Vec<&str> = group_key.split(':').collect();
    if parts.len() >= 2 {
        let extension = parts[0];
        let size_cat = parts[1];
        
        match extension {
            ".rs" => {
                if files.len() > 3 {
                    ("Module Extraction".to_string(), "Consider extracting common functionality into shared modules".to_string())
                } else {
                    ("Function Extraction".to_string(), "Extract common functions into utility modules".to_string())
                }
            },
            ".py" => {
                ("Package Refactoring".to_string(), "Consider creating a shared package for common functionality".to_string())
            },
            ".js" | ".ts" => {
                ("Component Extraction".to_string(), "Extract common components into shared libraries".to_string())
            },
            _ => {
                ("Code Consolidation".to_string(), "Consider consolidating similar code patterns".to_string())
            }
        }
    } else {
        ("General Refactoring".to_string(), "Review for code duplication and modularization opportunities".to_string())
    }
}

fn calculate_similarity_score(files: &[String]) -> f64 {
    // Simple similarity score based on file count and naming patterns
    let count_factor = files.len() as f64;
    let naming_similarity = calculate_naming_similarity(files);
    
    (count_factor * naming_similarity).min(1.0)
}

fn calculate_naming_similarity(files: &[String]) -> f64 {
    if files.len() < 2 {
        return 0.0;
    }
    
    let mut common_prefix = String::new();
    let first_file = &files[0];
    
    // Find common prefix
    for (i, ch) in first_file.chars().enumerate() {
        if files.iter().all(|f| f.chars().nth(i) == Some(ch)) {
            common_prefix.push(ch);
        } else {
            break;
        }
    }
    
    common_prefix.len() as f64 / first_file.len() as f64
}

fn get_file_size(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-files", "--stage", path])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    if let Some(line) = output_str.lines().next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[1];
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    return Ok(size);
                }
            }
        }
    }
    
    Ok(0)
}

fn analyze_delta_compression_groups() -> Result<Vec<DeltaCompressionGroup>, Box<dyn std::error::Error>> {
    println!("🔄 Analyzing delta compression groups...");
    
    // This would ideally use git verify-pack, but for now we'll simulate
    // In a real implementation, you'd parse the output of:
    // git verify-pack -v .git/objects/pack/*.idx
    
    let mut delta_groups = Vec::new();
    
    // For demonstration, we'll create some mock delta groups
    // In practice, this would analyze actual packfile data
    
    println!("🔄 Found {} delta compression groups", delta_groups.len());
    Ok(delta_groups)
}

fn analyze_code_patterns() -> Result<Vec<CodePattern>, Box<dyn std::error::Error>> {
    println!("📊 Analyzing code patterns...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut pattern_groups: HashMap<String, Vec<String>> = HashMap::new();
    
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            
            if is_code_file(&path) {
                let pattern = identify_code_pattern(&path);
                pattern_groups.entry(pattern).or_insert_with(Vec::new).push(path);
            }
        }
    }
    
    let mut code_patterns = Vec::new();
    
    for (pattern_type, files) in pattern_groups {
        if files.len() > 1 {
            let repetition_count = files.len() as u32;
            let potential_savings = calculate_pattern_savings(&files);
            
            code_patterns.push(CodePattern {
                pattern_type,
                files,
                repetition_count,
                potential_savings,
            });
        }
    }
    
    println!("📊 Found {} code patterns", code_patterns.len());
    Ok(code_patterns)
}

fn identify_code_pattern(path: &str) -> String {
    let path_lower = path.to_lowercase();
    
    if path_lower.contains("test") || path_lower.contains("spec") {
        "Test Pattern".to_string()
    } else if path_lower.contains("config") || path_lower.contains("settings") {
        "Configuration Pattern".to_string()
    } else if path_lower.contains("util") || path_lower.contains("helper") {
        "Utility Pattern".to_string()
    } else if path_lower.contains("model") || path_lower.contains("entity") {
        "Model Pattern".to_string()
    } else if path_lower.contains("controller") || path_lower.contains("handler") {
        "Controller Pattern".to_string()
    } else if path_lower.contains("service") || path_lower.contains("provider") {
        "Service Pattern".to_string()
    } else {
        "General Pattern".to_string()
    }
}

fn calculate_pattern_savings(files: &[String]) -> u64 {
    // Estimate potential savings from modularization
    let total_size: u64 = files.iter()
        .map(|f| get_file_size(f).unwrap_or(0))
        .sum();
    
    // Assume 30% savings from modularization
    (total_size as f64 * 0.3) as u64
}

fn generate_modularization_report(
    candidates: &[ModularizationCandidate],
    delta_groups: &[DeltaCompressionGroup],
    code_patterns: &[CodePattern],
) -> ModularizationAnalysis {
    println!("\n🔧 Modularization Analysis Report");
    println!("================================");
    
    let total_potential_savings: u64 = candidates.iter().map(|c| c.size).sum();
    let mut recommendations = Vec::new();
    
    // Show modularization candidates
    if !candidates.is_empty() {
        println!("\n🔍 Modularization Candidates ({}):", candidates.len());
        
        // Group by modularization type
        let mut type_groups: HashMap<String, Vec<&ModularizationCandidate>> = HashMap::new();
        for candidate in candidates {
            type_groups.entry(candidate.modularization_type.clone()).or_insert_with(Vec::new).push(candidate);
        }
        
        for (mod_type, candidates) in type_groups {
            println!("\n  📦 {} ({} files):", mod_type, candidates.len());
            for candidate in candidates.iter().take(3) {
                println!("    • {} (similarity: {:.2})", candidate.path, candidate.similarity_score);
                println!("      Recommendation: {}", candidate.recommendation);
            }
            if candidates.len() > 3 {
                println!("    ... and {} more", candidates.len() - 3);
            }
        }
        
        recommendations.push("Extract common functionality into shared modules".to_string());
        recommendations.push("Consider creating utility libraries for repeated patterns".to_string());
    }
    
    // Show code patterns
    if !code_patterns.is_empty() {
        println!("\n📊 Code Patterns ({}):", code_patterns.len());
        
        // Sort by potential savings
        let mut sorted_patterns = code_patterns.to_vec();
        sorted_patterns.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));
        
        for pattern in sorted_patterns.iter().take(5) {
            println!("  • {} ({} files, {} bytes potential savings)", 
                pattern.pattern_type, pattern.repetition_count, pattern.potential_savings);
            for file in pattern.files.iter().take(2) {
                println!("    - {}", file);
            }
            if pattern.files.len() > 2 {
                println!("    ... and {} more", pattern.files.len() - 2);
            }
        }
        
        if code_patterns.len() > 5 {
            println!("  ... and {} more patterns", code_patterns.len() - 5);
        }
        
        recommendations.push("Consolidate similar code patterns into shared components".to_string());
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("  • Modularization candidates: {}", candidates.len());
    println!("  • Delta compression groups: {}", delta_groups.len());
    println!("  • Code patterns: {}", code_patterns.len());
    println!("  • Total potential savings: {:.2} KB", total_potential_savings as f64 / 1024.0);
    
    if !recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &recommendations {
            println!("  • {}", rec);
        }
    }
    
    ModularizationAnalysis {
        candidates: candidates.to_vec(),
        delta_groups: delta_groups.to_vec(),
        code_patterns: code_patterns.to_vec(),
        total_potential_savings,
        recommendations,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Modularization Analyzer");
    println!("==========================");
    println!("Analyzing code for modularization opportunities...");
    println!();
    
    // Analyze modularization candidates
    let candidates = analyze_modularization_candidates()?;
    
    // Analyze delta compression groups
    let delta_groups = analyze_delta_compression_groups()?;
    
    // Analyze code patterns
    let code_patterns = analyze_code_patterns()?;
    
    // Generate comprehensive report
    let analysis = generate_modularization_report(&candidates, &delta_groups, &code_patterns);
    
    println!("\n✅ Analysis complete!");
    println!("🔧 Modularization opportunities identified");
    println!("📊 Code patterns analyzed");
    println!("💡 Refactoring recommendations ready");
    println!("🚀 Ready for code optimization");
    
    Ok(())
}
