use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct LfsHookCandidate {
    path: String,
    size: u64,
    hash: String,
    hook_type: String,
    lfs_pointer: String,
    potential_savings: u64,
}

#[derive(Debug, Clone)]
struct SharedBinaryHook {
    hash: String,
    size: u64,
    paths: Vec<String>,
    hook_types: Vec<String>,
    reuse_count: u32,
    lfs_benefit: f64,
}

#[derive(Debug)]
struct LfsOptimizationPlan {
    candidates: Vec<LfsHookCandidate>,
    shared_binaries: Vec<SharedBinaryHook>,
    total_savings: u64,
    gitattributes_rules: Vec<String>,
    migration_commands: Vec<String>,
}

fn detect_lfs_hook_candidates() -> Result<Vec<LfsHookCandidate>, Box<dyn std::error::Error>> {
    println!("🔍 Detecting LFS hook candidates...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut candidates = Vec::new();
    
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1].to_string();
            
            // Focus on hook-related files
            if path.contains(".hooksmith/hooks/") || path.contains("hooks/") {
                // Get file size
                let size_output = Command::new("git")
                    .args(&["cat-file", "-s", &hash])
                    .output()?;
                
                if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                    if let Ok(size) = u64::from_str(size_str.trim()) {
                        // Check if this is a candidate for LFS
                        if is_lfs_candidate(&path, size) {
                            let hook_type = determine_hook_type(&path);
                            let lfs_pointer = generate_lfs_pointer(&hash, size);
                            let potential_savings = calculate_lfs_savings(size);
                            
                            candidates.push(LfsHookCandidate {
                                path,
                                size,
                                hash,
                                hook_type,
                                lfs_pointer,
                                potential_savings,
                            });
                        }
                    }
                }
            }
        }
    }
    
    println!("🔍 Found {} LFS hook candidates", candidates.len());
    Ok(candidates)
}

fn is_lfs_candidate(path: &str, size: u64) -> bool {
    let path_lower = path.to_lowercase();
    
    // Size-based criteria for hooks
    if size > 1 * 1024 * 1024 { // 1 MB
        return true;
    }
    
    // Binary file types that should be in LFS
    path_lower.contains(".exe") || 
    path_lower.contains(".dll") || 
    path_lower.contains(".so") || 
    path_lower.contains(".dylib") ||
    path_lower.contains(".bin") ||
    path_lower.contains(".wasm") ||
    path_lower.contains(".o") ||
    path_lower.contains(".a") ||
    path_lower.contains(".dylib") ||
    path_lower.contains(".framework")
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

fn generate_lfs_pointer(hash: &str, size: u64) -> String {
    // Generate a mock LFS pointer (in real implementation, this would be the actual pointer)
    format!("version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {}", hash, size)
}

fn calculate_lfs_savings(size: u64) -> u64 {
    // LFS pointer is ~130 bytes, so savings = original_size - 130
    if size > 130 {
        size - 130
    } else {
        0
    }
}

fn analyze_shared_binary_hooks() -> Result<Vec<SharedBinaryHook>, Box<dyn std::error::Error>> {
    println!("🔄 Analyzing shared binary hooks...");
    
    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;
    
    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut hash_groups: HashMap<String, Vec<(String, String)>> = HashMap::new();
    
    // First pass: group files by hash
    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1].to_string();
            
            if path.contains(".hooksmith/hooks/") || path.contains("hooks/") {
                hash_groups.entry(hash.clone()).or_insert_with(Vec::new).push((path, hash));
            }
        }
    }
    
    let mut shared_binaries = Vec::new();
    
    // Second pass: analyze shared binaries
    for (hash, files) in hash_groups {
        if files.len() > 1 { // Only consider files that are reused
            let mut paths = Vec::new();
            let mut hook_types = Vec::new();
            
            for (path, _) in &files {
                paths.push(path.clone());
                hook_types.push(determine_hook_type(path));
            }
            
            // Get size for this hash
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", &hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let reuse_count = files.len() as u32;
                    let lfs_benefit = calculate_lfs_benefit(size, reuse_count);
                    
                    shared_binaries.push(SharedBinaryHook {
                        hash,
                        size,
                        paths,
                        hook_types,
                        reuse_count,
                        lfs_benefit,
                    });
                }
            }
        }
    }
    
    println!("🔄 Found {} shared binary hooks", shared_binaries.len());
    Ok(shared_binaries)
}

fn calculate_lfs_benefit(size: u64, reuse_count: u32) -> f64 {
    // Calculate benefit based on size and reuse count
    let size_factor = (size as f64 / (1024.0 * 1024.0)).min(10.0); // Cap at 10 MB factor
    let reuse_factor = reuse_count as f64;
    
    size_factor * reuse_factor
}

fn generate_optimization_plan(
    candidates: &[LfsHookCandidate],
    shared_binaries: &[SharedBinaryHook],
) -> LfsOptimizationPlan {
    println!("\n📦 LFS Hook Optimization Plan");
    println!("=============================");
    
    let total_savings: u64 = candidates.iter().map(|c| c.potential_savings).sum();
    let mut gitattributes_rules = Vec::new();
    let mut migration_commands = Vec::new();
    
    // Generate .gitattributes rules
    let mut extensions = std::collections::HashSet::new();
    for candidate in candidates {
        if let Some(dot_pos) = candidate.path.rfind('.') {
            let extension = candidate.path[dot_pos..].to_string();
            extensions.insert(extension);
        }
    }
    
    for extension in extensions {
        gitattributes_rules.push(format!("*{} filter=lfs diff=lfs merge=lfs -text", extension));
    }
    
    // Generate migration commands
    migration_commands.push("git lfs install".to_string());
    
    for candidate in candidates {
        if let Some(dot_pos) = candidate.path.rfind('.') {
            let extension = &candidate.path[dot_pos..];
            migration_commands.push(format!("git lfs track \"*{}\"", extension));
        }
    }
    
    migration_commands.push("git add .gitattributes".to_string());
    migration_commands.push("git commit -m \"Add Git LFS tracking for binary hooks\"".to_string());
    
    // Show candidates
    if !candidates.is_empty() {
        println!("\n🔍 LFS Hook Candidates ({}):", candidates.len());
        for candidate in candidates.iter().take(5) {
            println!("  • {} ({} bytes) - {}: {} bytes savings", 
                candidate.path, candidate.size, candidate.hook_type, candidate.potential_savings);
        }
        if candidates.len() > 5 {
            println!("  ... and {} more candidates", candidates.len() - 5);
        }
    }
    
    // Show shared binaries
    if !shared_binaries.is_empty() {
        println!("\n🔄 Shared Binary Hooks ({}):", shared_binaries.len());
        
        // Sort by benefit
        let mut sorted_binaries = shared_binaries.to_vec();
        sorted_binaries.sort_by(|a, b| b.lfs_benefit.partial_cmp(&a.lfs_benefit).unwrap());
        
        for binary in sorted_binaries.iter().take(5) {
            println!("  • {} ({} bytes) - reused {}x (benefit: {:.2})", 
                binary.hash[..8].to_string(), binary.size, binary.reuse_count, binary.lfs_benefit);
            for path in binary.paths.iter().take(3) {
                println!("    - {}", path);
            }
            if binary.paths.len() > 3 {
                println!("    ... and {} more", binary.paths.len() - 3);
            }
        }
        
        if shared_binaries.len() > 5 {
            println!("  ... and {} more shared binaries", shared_binaries.len() - 5);
        }
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("  • LFS candidates: {}", candidates.len());
    println!("  • Shared binaries: {}", shared_binaries.len());
    println!("  • Total potential savings: {:.2} MB", total_savings as f64 / (1024.0 * 1024.0));
    
    if !gitattributes_rules.is_empty() {
        println!("\n📝 Suggested .gitattributes rules:");
        println!("================================");
        for rule in &gitattributes_rules {
            println!("{}", rule);
        }
    }
    
    if !migration_commands.is_empty() {
        println!("\n🔧 Migration commands:");
        println!("=====================");
        for cmd in &migration_commands {
            println!("{}", cmd);
        }
    }
    
    LfsOptimizationPlan {
        candidates: candidates.to_vec(),
        shared_binaries: shared_binaries.to_vec(),
        total_savings,
        gitattributes_rules,
        migration_commands,
    }
}

fn generate_hook_deduplication_plan(shared_binaries: &[SharedBinaryHook]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    for binary in shared_binaries {
        if binary.reuse_count > 2 {
            recommendations.push(format!(
                "Consider symbolic link for {} (reused {}x across {} hook types)",
                binary.hash[..8].to_string(),
                binary.reuse_count,
                binary.hook_types.len()
            ));
        }
    }
    
    recommendations
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 LFS Hook Optimizer");
    println!("=====================");
    println!("Optimizing binary hooks with Git LFS...");
    println!();
    
    // Detect LFS candidates
    let lfs_candidates = detect_lfs_hook_candidates()?;
    
    // Analyze shared binaries
    let shared_binaries = analyze_shared_binary_hooks()?;
    
    // Generate optimization plan
    let optimization_plan = generate_optimization_plan(&lfs_candidates, &shared_binaries);
    
    // Generate deduplication recommendations
    let dedup_recommendations = generate_hook_deduplication_plan(&shared_binaries);
    
    if !dedup_recommendations.is_empty() {
        println!("\n💡 Deduplication Recommendations:");
        println!("================================");
        for rec in &dedup_recommendations {
            println!("  • {}", rec);
        }
    }
    
    println!("\n✅ Optimization complete!");
    println!("📦 Git LFS integration ready for binary hooks");
    println!("🔄 Shared binary detection complete");
    println!("💡 Deduplication opportunities identified");
    println!("🚀 Ready for migration to LFS");
    
    Ok(())
}
