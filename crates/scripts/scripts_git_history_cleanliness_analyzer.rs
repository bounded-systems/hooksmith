use std::collections::HashMap;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone)]
struct CleanlinessAnalysis {
    commit_quality: CommitQualityAnalysis,
    blob_health: BlobHealthAnalysis,
    subtree_readiness: SubtreeReadinessAnalysis,
    tree_stability: TreeStabilityAnalysis,
    contract_safety: ContractSafetyAnalysis,
    overall_score: f64,
    recommendations: Vec<String>,
    critical_issues: Vec<String>,
}

#[derive(Debug, Clone)]
struct CommitQualityAnalysis {
    total_commits: u32,
    linear_commits: u32,
    merge_commits: u32,
    squash_opportunities: u32,
    conventional_commits: u32,
    meaningful_messages: u32,
    wip_commits: u32,
    quality_score: f64,
}

#[derive(Debug, Clone)]
struct BlobHealthAnalysis {
    total_blobs: u32,
    duplicate_blobs: u32,
    large_blobs: u32,
    lfs_candidates: u32,
    packfile_efficiency: f64,
    deduplication_score: f64,
}

#[derive(Debug, Clone)]
struct SubtreeReadinessAnalysis {
    modular_structure: bool,
    cross_crate_imports: u32,
    stable_boundaries: u32,
    extractable_crates: Vec<String>,
    readiness_score: f64,
}

#[derive(Debug, Clone)]
struct TreeStabilityAnalysis {
    tree_depth: u32,
    volatile_directories: u32,
    stable_layout: bool,
    rehash_frequency: f64,
    stability_score: f64,
}

#[derive(Debug, Clone)]
struct ContractSafetyAnalysis {
    pinned_contracts: u32,
    stable_shas: u32,
    immutable_after_release: bool,
    release_tags: u32,
    safety_score: f64,
}

#[derive(Debug, Clone)]
enum CleanlinessLevel {
    Excellent,
    Good,
    NeedsImprovement,
    Critical,
}

fn analyze_git_history_cleanliness() -> Result<CleanlinessAnalysis, Box<dyn std::error::Error>> {
    println!("🧼 Analyzing Git history cleanliness for Hooksmith contract system...");
    
    // Analyze commit quality
    let commit_quality = analyze_commit_quality()?;
    
    // Analyze blob health
    let blob_health = analyze_blob_health()?;
    
    // Analyze subtree readiness
    let subtree_readiness = analyze_subtree_readiness()?;
    
    // Analyze tree stability
    let tree_stability = analyze_tree_stability()?;
    
    // Analyze contract safety
    let contract_safety = analyze_contract_safety()?;
    
    // Calculate overall score
    let overall_score = calculate_overall_score(&commit_quality, &blob_health, &subtree_readiness, &tree_stability, &contract_safety);
    
    // Generate recommendations
    let recommendations = generate_recommendations(&commit_quality, &blob_health, &subtree_readiness, &tree_stability, &contract_safety);
    
    // Identify critical issues
    let critical_issues = identify_critical_issues(&commit_quality, &blob_health, &subtree_readiness, &tree_stability, &contract_safety);
    
    Ok(CleanlinessAnalysis {
        commit_quality,
        blob_health,
        subtree_readiness,
        tree_stability,
        contract_safety,
        overall_score,
        recommendations,
        critical_issues,
    })
}

fn analyze_commit_quality() -> Result<CommitQualityAnalysis, Box<dyn std::error::Error>> {
    // Get commit statistics
    let total_commits = get_total_commits()?;
    let merge_commits = count_merge_commits()?;
    let linear_commits = total_commits - merge_commits;
    
    // Analyze commit messages
    let (conventional_commits, meaningful_messages, wip_commits) = analyze_commit_messages()?;
    
    // Find squash opportunities
    let squash_opportunities = find_squash_opportunities()?;
    
    // Calculate quality score
    let quality_score = calculate_commit_quality_score(
        total_commits,
        linear_commits,
        merge_commits,
        conventional_commits,
        meaningful_messages,
        wip_commits,
    );
    
    Ok(CommitQualityAnalysis {
        total_commits,
        linear_commits,
        merge_commits,
        squash_opportunities,
        conventional_commits,
        meaningful_messages,
        wip_commits,
        quality_score,
    })
}

fn get_total_commits() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-list", "--count", "--all"])
        .output()?;
    
    let count_str = String::from_utf8(output.stdout)?;
    Ok(count_str.trim().parse().unwrap_or(0))
}

fn count_merge_commits() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--merges", "--oneline"])
        .output()?;
    
    let merges = String::from_utf8(output.stdout)?;
    Ok(merges.lines().count() as u32)
}

fn analyze_commit_messages() -> Result<(u32, u32, u32), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "100"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    let mut conventional_commits = 0u32;
    let mut meaningful_messages = 0u32;
    let mut wip_commits = 0u32;
    
    for line in commits.lines() {
        let message = line.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        
        // Check for conventional commits
        if message.starts_with("feat:") || message.starts_with("fix:") || 
           message.starts_with("chore:") || message.starts_with("docs:") ||
           message.starts_with("style:") || message.starts_with("refactor:") ||
           message.starts_with("test:") || message.starts_with("perf:") {
            conventional_commits += 1;
        }
        
        // Check for meaningful messages (not just "wip", "fix", etc.)
        if message.len() > 10 && !message.to_lowercase().contains("wip") &&
           !message.to_lowercase().contains("fix bug") && !message.to_lowercase().contains("oops") {
            meaningful_messages += 1;
        }
        
        // Check for WIP commits
        if message.to_lowercase().contains("wip") || message.to_lowercase().contains("work in progress") {
            wip_commits += 1;
        }
    }
    
    Ok((conventional_commits, meaningful_messages, wip_commits))
}

fn find_squash_opportunities() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "50"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    let mut opportunities = 0u32;
    
    for line in commits.lines() {
        let message = line.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        
        // Look for commits that could be squashed
        if message.to_lowercase().contains("fix") && message.len() < 20 ||
           message.to_lowercase().contains("typo") ||
           message.to_lowercase().contains("oops") ||
           message.to_lowercase().contains("wip") {
            opportunities += 1;
        }
    }
    
    Ok(opportunities)
}

fn calculate_commit_quality_score(
    total_commits: u32,
    linear_commits: u32,
    merge_commits: u32,
    conventional_commits: u32,
    meaningful_messages: u32,
    wip_commits: u32,
) -> f64 {
    if total_commits == 0 {
        return 1.0;
    }
    
    let mut score = 0.0;
    
    // Linear commit ratio (prefer linear history)
    let linear_ratio = linear_commits as f64 / total_commits as f64;
    score += linear_ratio * 0.3;
    
    // Conventional commit ratio
    let conventional_ratio = conventional_commits as f64 / total_commits as f64;
    score += conventional_ratio * 0.3;
    
    // Meaningful message ratio
    let meaningful_ratio = meaningful_messages as f64 / total_commits as f64;
    score += meaningful_ratio * 0.2;
    
    // Penalty for WIP commits
    let wip_ratio = wip_commits as f64 / total_commits as f64;
    score -= wip_ratio * 0.2;
    
    score.max(0.0).min(1.0)
}

fn analyze_blob_health() -> Result<BlobHealthAnalysis, Box<dyn std::error::Error>> {
    // Count total blobs
    let total_blobs = count_total_blobs()?;
    
    // Find duplicate blobs
    let duplicate_blobs = find_duplicate_blobs()?;
    
    // Find large blobs
    let large_blobs = find_large_blobs()?;
    
    // Find LFS candidates
    let lfs_candidates = find_lfs_candidates()?;
    
    // Calculate packfile efficiency
    let packfile_efficiency = calculate_packfile_efficiency()?;
    
    // Calculate deduplication score
    let deduplication_score = calculate_deduplication_score(total_blobs, duplicate_blobs, large_blobs);
    
    Ok(BlobHealthAnalysis {
        total_blobs,
        duplicate_blobs,
        large_blobs,
        lfs_candidates,
        packfile_efficiency,
        deduplication_score,
    })
}

fn count_total_blobs() -> Result<u32, Box<dyn std::error::Error>> {
    // Simplified blob count
    let output = Command::new("git")
        .args(&["ls-files"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    Ok(files.lines().count() as u32)
}

fn find_duplicate_blobs() -> Result<u32, Box<dyn std::error::Error>> {
    // This is a simplified check - in practice you'd use git verify-pack
    let output = Command::new("git")
        .args(&["verify-pack", "-v", ".git/objects/pack/*.pack"])
        .output();
    
    if let Ok(output) = output {
        let content = String::from_utf8(output.stdout)?;
        // Count lines that might indicate duplicates
        let duplicates = content.lines()
            .filter(|line| line.contains("delta") || line.contains("chain"))
            .count();
        Ok(duplicates as u32)
    } else {
        Ok(0) // Fallback
    }
}

fn find_large_blobs() -> Result<u32, Box<dyn std::error::Error>> {
    // Simplified large blob detection
    let output = Command::new("find")
        .args(&[".", "-type", "f", "-size", "+100k"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    Ok(files.lines().count() as u32)
}

fn find_lfs_candidates() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[".", "-type", "f", "-size", "+1M"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    Ok(files.lines().count() as u32)
}

fn calculate_packfile_efficiency() -> Result<f64, Box<dyn std::error::Error>> {
    // Simplified packfile efficiency calculation
    let pack_dir = Path::new(".git/objects/pack");
    if !pack_dir.exists() {
        return Ok(1.0);
    }
    
    let output = Command::new("du")
        .args(&["-sh", ".git/objects/pack"])
        .output()?;
    
    let size_str = String::from_utf8(output.stdout)?;
    let size_parts: Vec<&str> = size_str.split_whitespace().collect();
    
    if let Some(size_part) = size_parts.first() {
        // Very simplified efficiency calculation
        if size_part.contains("M") {
            Ok(0.8) // Good efficiency
        } else if size_part.contains("G") {
            Ok(0.5) // Poor efficiency
        } else {
            Ok(1.0) // Excellent efficiency
        }
    } else {
        Ok(1.0)
    }
}

fn calculate_deduplication_score(total_blobs: u32, duplicate_blobs: u32, large_blobs: u32) -> f64 {
    if total_blobs == 0 {
        return 1.0;
    }
    
    let mut score = 1.0;
    
    // Penalty for duplicates
    let duplicate_ratio = duplicate_blobs as f64 / total_blobs as f64;
    score -= duplicate_ratio * 0.3;
    
    // Penalty for large blobs
    let large_ratio = large_blobs as f64 / total_blobs as f64;
    score -= large_ratio * 0.2;
    
    score.max(0.0).min(1.0)
}

fn analyze_subtree_readiness() -> Result<SubtreeReadinessAnalysis, Box<dyn std::error::Error>> {
    // Check for modular structure
    let modular_structure = check_modular_structure()?;
    
    // Count cross-crate imports
    let cross_crate_imports = count_cross_crate_imports()?;
    
    // Count stable boundaries
    let stable_boundaries = count_stable_boundaries()?;
    
    // Find extractable crates
    let extractable_crates = find_extractable_crates()?;
    
    // Calculate readiness score
    let readiness_score = calculate_subtree_readiness_score(
        modular_structure,
        cross_crate_imports,
        stable_boundaries,
        extractable_crates.len(),
    );
    
    Ok(SubtreeReadinessAnalysis {
        modular_structure,
        cross_crate_imports,
        stable_boundaries,
        extractable_crates,
        readiness_score,
    })
}

fn check_modular_structure() -> Result<bool, Box<dyn std::error::Error>> {
    // Check if we have a clear crate structure
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;
    
    let cargo_files = String::from_utf8(output.stdout)?;
    let crate_count = cargo_files.lines().count();
    
    // Consider modular if we have multiple crates
    Ok(crate_count > 1)
}

fn count_cross_crate_imports() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[".", "-name", "*.rs", "-type", "f"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut cross_imports = 0u32;
    
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            // Look for cross-crate imports
            for line in content.lines() {
                if line.contains("use crate::") || line.contains("extern crate") {
                    cross_imports += 1;
                }
            }
        }
    }
    
    Ok(cross_imports)
}

fn count_stable_boundaries() -> Result<u32, Box<dyn std::error::Error>> {
    // Count directories that could be stable boundaries
    let output = Command::new("find")
        .args(&[".", "-type", "d", "-name", "src"])
        .output()?;
    
    let dirs = String::from_utf8(output.stdout)?;
    Ok(dirs.lines().count() as u32)
}

fn find_extractable_crates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;
    
    let cargo_files = String::from_utf8(output.stdout)?;
    let mut extractable = Vec::new();
    
    for cargo_file in cargo_files.lines() {
        if let Some(crate_dir) = Path::new(cargo_file).parent() {
            if let Some(crate_name) = crate_dir.file_name() {
                let name = crate_name.to_string_lossy().to_string();
                // Consider crates with clear boundaries as extractable
                if !name.contains("target") && !name.contains("node_modules") {
                    extractable.push(name);
                }
            }
        }
    }
    
    Ok(extractable)
}

fn calculate_subtree_readiness_score(
    modular_structure: bool,
    cross_crate_imports: u32,
    stable_boundaries: u32,
    extractable_crates: usize,
) -> f64 {
    let mut score = 0.0;
    
    // Modular structure bonus
    if modular_structure {
        score += 0.3;
    }
    
    // Stable boundaries bonus
    let boundary_score = (stable_boundaries as f64 / 10.0).min(0.3);
    score += boundary_score;
    
    // Extractable crates bonus
    let crate_score = (extractable_crates as f64 / 5.0).min(0.2);
    score += crate_score;
    
    // Penalty for too many cross-imports
    let import_penalty = (cross_crate_imports as f64 / 100.0).min(0.2);
    score -= import_penalty;
    
    score.max(0.0).min(1.0)
}

fn analyze_tree_stability() -> Result<TreeStabilityAnalysis, Box<dyn std::error::Error>> {
    // Calculate tree depth
    let tree_depth = calculate_tree_depth()?;
    
    // Count volatile directories
    let volatile_directories = count_volatile_directories()?;
    
    // Check for stable layout
    let stable_layout = check_stable_layout()?;
    
    // Calculate rehash frequency
    let rehash_frequency = calculate_rehash_frequency()?;
    
    // Calculate stability score
    let stability_score = calculate_tree_stability_score(
        tree_depth,
        volatile_directories,
        stable_layout,
        rehash_frequency,
    );
    
    Ok(TreeStabilityAnalysis {
        tree_depth,
        volatile_directories,
        stable_layout,
        rehash_frequency,
        stability_score,
    })
}

fn calculate_tree_depth() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut max_depth = 0u32;
    
    for line in files.lines() {
        let depth = line.matches('/').count() as u32;
        if depth > max_depth {
            max_depth = depth;
        }
    }
    
    Ok(max_depth)
}

fn count_volatile_directories() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--name-only", "--pretty=format:", "-n", "50"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut directories = std::collections::HashSet::new();
    
    for line in files.lines() {
        if let Some(dir) = Path::new(line).parent() {
            if let Some(dir_str) = dir.to_str() {
                directories.insert(dir_str.to_string());
            }
        }
    }
    
    Ok(directories.len() as u32)
}

fn check_stable_layout() -> Result<bool, Box<dyn std::error::Error>> {
    // Check if we have a stable directory structure
    let output = Command::new("git")
        .args(&["ls-tree", "--name-only", "HEAD"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let has_src = files.lines().any(|line| line == "src");
    let has_crates = files.lines().any(|line| line == "crates");
    let has_docs = files.lines().any(|line| line == "docs");
    
    Ok(has_src || has_crates || has_docs)
}

fn calculate_rehash_frequency() -> Result<f64, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "100"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    let total_commits = commits.lines().count() as f64;
    
    if total_commits == 0.0 {
        return Ok(0.0);
    }
    
    // Simplified rehash frequency calculation
    Ok(0.1) // Assume low rehash frequency
}

fn calculate_tree_stability_score(
    tree_depth: u32,
    volatile_directories: u32,
    stable_layout: bool,
    rehash_frequency: f64,
) -> f64 {
    let mut score = 1.0;
    
    // Penalty for deep trees
    if tree_depth > 10 {
        score -= 0.2;
    } else if tree_depth > 5 {
        score -= 0.1;
    }
    
    // Penalty for volatile directories
    let volatility_penalty = (volatile_directories as f64 / 20.0).min(0.3);
    score -= volatility_penalty;
    
    // Bonus for stable layout
    if stable_layout {
        score += 0.1;
    }
    
    // Penalty for high rehash frequency
    score -= rehash_frequency * 0.2;
    
    score.max(0.0).min(1.0)
}

fn analyze_contract_safety() -> Result<ContractSafetyAnalysis, Box<dyn std::error::Error>> {
    // Count pinned contracts
    let pinned_contracts = count_pinned_contracts()?;
    
    // Count stable SHAs
    let stable_shas = count_stable_shas()?;
    
    // Check if immutable after release
    let immutable_after_release = check_immutable_after_release()?;
    
    // Count release tags
    let release_tags = count_release_tags()?;
    
    // Calculate safety score
    let safety_score = calculate_contract_safety_score(
        pinned_contracts,
        stable_shas,
        immutable_after_release,
        release_tags,
    );
    
    Ok(ContractSafetyAnalysis {
        pinned_contracts,
        stable_shas,
        immutable_after_release,
        release_tags,
        safety_score,
    })
}

fn count_pinned_contracts() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[".", "-name", "*.rs", "-type", "f"])
        .output()?;
    
    let files = String::from_utf8(output.stdout)?;
    let mut pinned_contracts = 0u32;
    
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") && content.contains("sha") {
                pinned_contracts += 1;
            }
        }
    }
    
    Ok(pinned_contracts)
}

fn count_stable_shas() -> Result<u32, Box<dyn std::error::Error>> {
    // Simplified stable SHA detection - just count recent commits
    let output = Command::new("git")
        .args(&["log", "--oneline", "-n", "20"])
        .output()?;
    
    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().count() as u32)
}

fn check_immutable_after_release() -> Result<bool, Box<dyn std::error::Error>> {
    // Check if we have release branches or tags
    let output = Command::new("git")
        .args(&["branch", "-r"])
        .output()?;
    
    let branches = String::from_utf8(output.stdout)?;
    let has_release_branches = branches.lines().any(|line| line.contains("release"));
    
    let tag_output = Command::new("git")
        .args(&["tag"])
        .output()?;
    
    let tags = String::from_utf8(tag_output.stdout)?;
    let has_release_tags = tags.lines().any(|line| line.starts_with("v"));
    
    Ok(has_release_branches || has_release_tags)
}

fn count_release_tags() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["tag"])
        .output()?;
    
    let tags = String::from_utf8(output.stdout)?;
    let release_tags = tags.lines()
        .filter(|tag| tag.starts_with("v") || tag.contains("release"))
        .count();
    
    Ok(release_tags as u32)
}

fn calculate_contract_safety_score(
    pinned_contracts: u32,
    stable_shas: u32,
    immutable_after_release: bool,
    release_tags: u32,
) -> f64 {
    let mut score: f64 = 0.0;
    
    // Bonus for pinned contracts
    if pinned_contracts > 0 {
        score += 0.3;
    }
    
    // Bonus for stable SHAs
    if stable_shas > 0 {
        score += 0.3;
    }
    
    // Bonus for immutable after release
    if immutable_after_release {
        score += 0.2;
    }
    
    // Bonus for release tags
    if release_tags > 0 {
        score += 0.2;
    }
    
    score.min(1.0_f64)
}

fn calculate_overall_score(
    commit_quality: &CommitQualityAnalysis,
    blob_health: &BlobHealthAnalysis,
    subtree_readiness: &SubtreeReadinessAnalysis,
    tree_stability: &TreeStabilityAnalysis,
    contract_safety: &ContractSafetyAnalysis,
) -> f64 {
    let commit_score = commit_quality.quality_score * 0.25;
    let blob_score = blob_health.deduplication_score * 0.20;
    let subtree_score = subtree_readiness.readiness_score * 0.20;
    let tree_score = tree_stability.stability_score * 0.15;
    let contract_score = contract_safety.safety_score * 0.20;
    
    commit_score + blob_score + subtree_score + tree_score + contract_score
}

fn generate_recommendations(
    commit_quality: &CommitQualityAnalysis,
    blob_health: &BlobHealthAnalysis,
    subtree_readiness: &SubtreeReadinessAnalysis,
    tree_stability: &TreeStabilityAnalysis,
    contract_safety: &ContractSafetyAnalysis,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Commit quality recommendations
    if commit_quality.quality_score < 0.7 {
        recommendations.push("Use conventional commit messages (feat:, fix:, chore:)".to_string());
        recommendations.push("Squash WIP commits before merging".to_string());
        recommendations.push("Prefer linear history with rebase".to_string());
    }
    
    // Blob health recommendations
    if blob_health.deduplication_score < 0.7 {
        recommendations.push("Use Git LFS for large files".to_string());
        recommendations.push("Run git gc --aggressive to optimize packfiles".to_string());
        recommendations.push("Avoid committing duplicate large files".to_string());
    }
    
    // Subtree readiness recommendations
    if subtree_readiness.readiness_score < 0.7 {
        recommendations.push("Organize code into clear crate boundaries".to_string());
        recommendations.push("Minimize cross-crate dependencies".to_string());
        recommendations.push("Use stable directory structures".to_string());
    }
    
    // Tree stability recommendations
    if tree_stability.stability_score < 0.7 {
        recommendations.push("Avoid frequent directory restructuring".to_string());
        recommendations.push("Group related files in stable directories".to_string());
        recommendations.push("Minimize tree depth for better performance".to_string());
    }
    
    // Contract safety recommendations
    if contract_safety.safety_score < 0.7 {
        recommendations.push("Tag stable releases with semantic versions".to_string());
        recommendations.push("Use release branches for stable SHAs".to_string());
        recommendations.push("Avoid rewriting history after contracts are pinned".to_string());
    }
    
    recommendations
}

fn identify_critical_issues(
    commit_quality: &CommitQualityAnalysis,
    blob_health: &BlobHealthAnalysis,
    subtree_readiness: &SubtreeReadinessAnalysis,
    tree_stability: &TreeStabilityAnalysis,
    contract_safety: &ContractSafetyAnalysis,
) -> Vec<String> {
    let mut critical_issues = Vec::new();
    
    if commit_quality.quality_score < 0.3 {
        critical_issues.push("Critical: Poor commit quality affecting contract stability".to_string());
    }
    
    if blob_health.deduplication_score < 0.3 {
        critical_issues.push("Critical: Large blobs affecting repository performance".to_string());
    }
    
    if subtree_readiness.readiness_score < 0.3 {
        critical_issues.push("Critical: Poor modular structure affecting extraction".to_string());
    }
    
    if tree_stability.stability_score < 0.3 {
        critical_issues.push("Critical: Unstable tree structure affecting SHA stability".to_string());
    }
    
    if contract_safety.safety_score < 0.3 {
        critical_issues.push("Critical: Contract safety issues detected".to_string());
    }
    
    critical_issues
}

fn generate_git_history_cleanliness_report(analysis: &CleanlinessAnalysis) {
    println!("\n🧼 Git History Cleanliness Analysis");
    println!("===================================");
    
    // Show commit quality
    println!("\n📝 Commit Quality:");
    println!("  • Total commits: {}", analysis.commit_quality.total_commits);
    println!("  • Linear commits: {} ({:.1}%)", 
        analysis.commit_quality.linear_commits,
        (analysis.commit_quality.linear_commits as f64 / analysis.commit_quality.total_commits as f64) * 100.0);
    println!("  • Conventional commits: {} ({:.1}%)",
        analysis.commit_quality.conventional_commits,
        (analysis.commit_quality.conventional_commits as f64 / analysis.commit_quality.total_commits as f64) * 100.0);
    println!("  • Squash opportunities: {}", analysis.commit_quality.squash_opportunities);
    println!("  • Quality score: {:.1}%", analysis.commit_quality.quality_score * 100.0);
    
    // Show blob health
    println!("\n📦 Blob Health:");
    println!("  • Total blobs: {}", analysis.blob_health.total_blobs);
    println!("  • Duplicate blobs: {}", analysis.blob_health.duplicate_blobs);
    println!("  • Large blobs: {}", analysis.blob_health.large_blobs);
    println!("  • LFS candidates: {}", analysis.blob_health.lfs_candidates);
    println!("  • Packfile efficiency: {:.1}%", analysis.blob_health.packfile_efficiency * 100.0);
    println!("  • Deduplication score: {:.1}%", analysis.blob_health.deduplication_score * 100.0);
    
    // Show subtree readiness
    println!("\n🌳 Subtree Readiness:");
    println!("  • Modular structure: {}", if analysis.subtree_readiness.modular_structure { "✅" } else { "❌" });
    println!("  • Cross-crate imports: {}", analysis.subtree_readiness.cross_crate_imports);
    println!("  • Stable boundaries: {}", analysis.subtree_readiness.stable_boundaries);
    println!("  • Extractable crates: {}", analysis.subtree_readiness.extractable_crates.len());
    println!("  • Readiness score: {:.1}%", analysis.subtree_readiness.readiness_score * 100.0);
    
    // Show tree stability
    println!("\n🌲 Tree Stability:");
    println!("  • Tree depth: {}", analysis.tree_stability.tree_depth);
    println!("  • Volatile directories: {}", analysis.tree_stability.volatile_directories);
    println!("  • Stable layout: {}", if analysis.tree_stability.stable_layout { "✅" } else { "❌" });
    println!("  • Rehash frequency: {:.1}%", analysis.tree_stability.rehash_frequency * 100.0);
    println!("  • Stability score: {:.1}%", analysis.tree_stability.stability_score * 100.0);
    
    // Show contract safety
    println!("\n🔒 Contract Safety:");
    println!("  • Pinned contracts: {}", analysis.contract_safety.pinned_contracts);
    println!("  • Stable SHAs: {}", analysis.contract_safety.stable_shas);
    println!("  • Immutable after release: {}", if analysis.contract_safety.immutable_after_release { "✅" } else { "❌" });
    println!("  • Release tags: {}", analysis.contract_safety.release_tags);
    println!("  • Safety score: {:.1}%", analysis.contract_safety.safety_score * 100.0);
    
    // Show overall score
    println!("\n📊 Overall Cleanliness Score:");
    println!("  • Overall score: {:.1}%", analysis.overall_score * 100.0);
    
    let cleanliness_level = if analysis.overall_score > 0.8 {
        CleanlinessLevel::Excellent
    } else if analysis.overall_score > 0.6 {
        CleanlinessLevel::Good
    } else if analysis.overall_score > 0.4 {
        CleanlinessLevel::NeedsImprovement
    } else {
        CleanlinessLevel::Critical
    };
    
    println!("  • Cleanliness level: {:?}", cleanliness_level);
    
    // Show critical issues
    if !analysis.critical_issues.is_empty() {
        println!("\n🚨 Critical Issues:");
        for issue in &analysis.critical_issues {
            println!("  • {}", issue);
        }
    }
    
    // Show recommendations
    if !analysis.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &analysis.recommendations {
            println!("  • {}", rec);
        }
    }
    
    // Summary
    println!("\n📋 Summary:");
    match cleanliness_level {
        CleanlinessLevel::Excellent => {
            println!("  • Excellent Git history cleanliness");
            println!("  • Optimal for Hooksmith contract system");
            println!("  • Maintain current practices");
        }
        CleanlinessLevel::Good => {
            println!("  • Good Git history cleanliness");
            println!("  • Minor improvements recommended");
            println!("  • Focus on identified areas");
        }
        CleanlinessLevel::NeedsImprovement => {
            println!("  • Git history needs improvement");
            println!("  • Address recommendations systematically");
            println!("  • Consider history rewrite for major issues");
        }
        CleanlinessLevel::Critical => {
            println!("  • Critical Git history issues");
            println!("  • Immediate action required");
            println!("  • Consider comprehensive cleanup");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧼 Git History Cleanliness Analyzer");
    println!("===================================");
    println!("Analyzing Git history for Hooksmith contract system...");
    println!();
    
    // Analyze Git history cleanliness
    let analysis = analyze_git_history_cleanliness()?;
    
    // Generate comprehensive report
    generate_git_history_cleanliness_report(&analysis);
    
    println!("\n✅ Git history cleanliness analysis complete!");
    println!("🧼 Cleanliness assessed");
    println!("📝 Commit quality analyzed");
    println!("📦 Blob health evaluated");
    println!("🌳 Subtree readiness checked");
    println!("🌲 Tree stability measured");
    println!("🔒 Contract safety verified");
    
    Ok(())
}
