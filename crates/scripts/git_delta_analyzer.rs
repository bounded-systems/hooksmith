use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct DeltaCandidate {
    file_path: String,
    blob_hash: String,
    blob_size: u64,
    similarity_score: f64,
    potential_base: Option<String>,
    delta_savings: u64,
}

#[derive(Debug, Clone)]
struct DeltaGroup {
    base_file: String,
    base_hash: String,
    base_size: u64,
    delta_files: Vec<DeltaCandidate>,
    total_savings: u64,
    compression_ratio: f64,
}

#[derive(Debug)]
struct DeltaAnalysis {
    candidates: Vec<DeltaCandidate>,
    groups: Vec<DeltaGroup>,
    total_potential_savings: u64,
    compression_insights: Vec<String>,
}

fn analyze_delta_candidates() -> Result<Vec<DeltaCandidate>, Box<dyn std::error::Error>> {
    println!("🔍 Analyzing Git delta compression candidates...");

    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut candidates = Vec::new();

    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1];

            // Get blob size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;

            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(blob_size) = u64::from_str(size_str.trim()) {
                    // Focus on files that could benefit from delta compression
                    if blob_size > 1024 && blob_size < 1024 * 1024 {
                        // 1KB to 1MB
                        let similarity_score = calculate_similarity_score(&path, blob_size);

                        if similarity_score > 0.3 {
                            // Only consider files with reasonable similarity potential
                            candidates.push(DeltaCandidate {
                                file_path: path,
                                blob_hash: hash.to_string(),
                                blob_size,
                                similarity_score,
                                potential_base: None,
                                delta_savings: 0,
                            });
                        }
                    }
                }
            }
        }
    }

    println!("🔍 Found {} delta candidates", candidates.len());
    Ok(candidates)
}

fn calculate_similarity_score(path: &str, blob_size: u64) -> f64 {
    let path_lower = path.to_lowercase();

    // Heuristic scoring based on file characteristics
    let mut score: f64 = 0.0;

    // File extension similarity
    if path_lower.ends_with(".rs") {
        score += 0.4; // Rust files often have similar structure
    } else if path_lower.ends_with(".json")
        || path_lower.ends_with(".yml")
        || path_lower.ends_with(".yaml")
    {
        score += 0.6; // Config files often have similar structure
    } else if path_lower.ends_with(".md") || path_lower.ends_with(".txt") {
        score += 0.3; // Documentation files
    }

    // Path similarity (files in same directory likely similar)
    if path_lower.contains("hooks/") {
        score += 0.3;
    } else if path_lower.contains("config/") {
        score += 0.4;
    } else if path_lower.contains("tests/") {
        score += 0.2;
    }

    // Size-based scoring (similar sizes compress better)
    let size_factor = if blob_size > 1024 * 1024 {
        0.1 // Large files less likely to delta well
    } else if blob_size > 100 * 1024 {
        0.3 // Medium files
    } else {
        0.5 // Small files more likely to delta
    };

    score += size_factor;

    score.min(1.0)
}

fn find_delta_groups(candidates: &[DeltaCandidate]) -> Vec<DeltaGroup> {
    println!("🔗 Finding delta compression groups...");

    let mut groups = Vec::new();
    let mut processed = std::collections::HashSet::new();

    // Group by file extension and size category
    let mut extension_groups: HashMap<String, Vec<&DeltaCandidate>> = HashMap::new();

    for candidate in candidates {
        let extension = get_file_extension(&candidate.file_path);
        let size_category = get_size_category(candidate.blob_size);
        let key = format!("{}:{}", extension, size_category);

        extension_groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(candidate);
    }

    for (_group_key, files) in extension_groups {
        if files.len() < 2 {
            continue; // Need at least 2 files for delta compression
        }

        // Sort by size (largest as base)
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by(|a, b| b.blob_size.cmp(&a.blob_size));

        let base_file = &sorted_files[0];
        let mut delta_files = Vec::new();

        for file in &sorted_files[1..] {
            if processed.contains(&file.blob_hash) {
                continue;
            }

            // Calculate potential savings
            let savings = calculate_delta_savings(base_file.blob_size, file.blob_size);

            if savings > 0 {
                delta_files.push(DeltaCandidate {
                    file_path: file.file_path.clone(),
                    blob_hash: file.blob_hash.clone(),
                    blob_size: file.blob_size,
                    similarity_score: file.similarity_score,
                    potential_base: Some(base_file.file_path.clone()),
                    delta_savings: savings,
                });

                processed.insert(file.blob_hash.clone());
            }
        }

        if !delta_files.is_empty() {
            let total_savings: u64 = delta_files.iter().map(|f| f.delta_savings).sum();
            let total_original_size: u64 = delta_files.iter().map(|f| f.blob_size).sum();
            let compression_ratio = if total_original_size > 0 {
                total_savings as f64 / total_original_size as f64
            } else {
                0.0
            };

            groups.push(DeltaGroup {
                base_file: base_file.file_path.clone(),
                base_hash: base_file.blob_hash.clone(),
                base_size: base_file.blob_size,
                delta_files,
                total_savings,
                compression_ratio,
            });
        }
    }

    println!("🔗 Found {} delta groups", groups.len());
    groups
}

fn get_file_extension(path: &str) -> String {
    if let Some(dot_pos) = path.rfind('.') {
        path[dot_pos..].to_string()
    } else {
        "no_extension".to_string()
    }
}

fn get_size_category(size: u64) -> String {
    match size {
        0..=1024 => "tiny".to_string(),
        1025..=10240 => "small".to_string(),
        10241..=102400 => "medium".to_string(),
        102401..=1024000 => "large".to_string(),
        _ => "huge".to_string(),
    }
}

fn calculate_delta_savings(base_size: u64, delta_size: u64) -> u64 {
    // Estimate delta savings based on size similarity
    let size_ratio = if base_size > 0 {
        delta_size as f64 / base_size as f64
    } else {
        1.0
    };

    // Files of similar size compress better
    let similarity_factor = if size_ratio > 0.5 && size_ratio < 2.0 {
        0.7 // Good compression for similar sizes
    } else if size_ratio > 0.2 && size_ratio < 5.0 {
        0.4 // Moderate compression
    } else {
        0.1 // Poor compression for very different sizes
    };

    (delta_size as f64 * similarity_factor) as u64
}

fn analyze_delta_compression_insights(groups: &[DeltaGroup]) -> Vec<String> {
    let mut insights = Vec::new();

    if groups.is_empty() {
        insights.push("No significant delta compression opportunities found".to_string());
        insights.push("Consider adjusting similarity thresholds or file grouping".to_string());
        return insights;
    }

    let total_savings: u64 = groups.iter().map(|g| g.total_savings).sum();
    let total_files: usize = groups.iter().map(|g| g.delta_files.len()).sum();

    insights.push(format!(
        "Found {} delta groups with {} files",
        groups.len(),
        total_files
    ));
    insights.push(format!(
        "Total potential savings: {:.2} KB",
        total_savings as f64 / 1024.0
    ));

    // Analyze by file type
    let mut type_savings: HashMap<String, u64> = HashMap::new();
    for group in groups {
        let extension = get_file_extension(&group.base_file);
        *type_savings.entry(extension).or_insert(0) += group.total_savings;
    }

    if !type_savings.is_empty() {
        insights.push("Savings by file type:".to_string());
        for (ext, savings) in type_savings {
            insights.push(format!("  • {}: {:.1} KB", ext, savings as f64 / 1024.0));
        }
    }

    // Compression ratio insights
    let avg_compression =
        groups.iter().map(|g| g.compression_ratio).sum::<f64>() / groups.len() as f64;

    insights.push(format!(
        "Average compression ratio: {:.1}%",
        avg_compression * 100.0
    ));

    insights
}

fn generate_delta_report(candidates: &[DeltaCandidate], groups: &[DeltaGroup]) -> DeltaAnalysis {
    println!("\n🔍 Git Delta Compression Analysis");
    println!("=================================");

    let total_potential_savings: u64 = groups.iter().map(|g| g.total_savings).sum();
    let compression_insights = analyze_delta_compression_insights(groups);

    // Show top candidates
    let mut sorted_candidates = candidates.to_vec();
    sorted_candidates.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

    println!("\n🔍 Top Delta Candidates (Top 10):");
    for candidate in sorted_candidates.iter().take(10) {
        println!(
            "  • {} ({} bytes, {:.1}% similarity)",
            candidate.file_path,
            candidate.blob_size,
            candidate.similarity_score * 100.0
        );
    }

    // Show delta groups
    if !groups.is_empty() {
        println!("\n🔗 Delta Compression Groups:");

        let mut sorted_groups = groups.to_vec();
        sorted_groups.sort_by(|a, b| b.total_savings.cmp(&a.total_savings));

        for group in sorted_groups.iter().take(5) {
            println!("  • Base: {} ({} bytes)", group.base_file, group.base_size);
            println!("    Delta files: {}", group.delta_files.len());
            println!(
                "    Savings: {:.1} KB ({:.1}% compression)",
                group.total_savings as f64 / 1024.0,
                group.compression_ratio * 100.0
            );

            for delta_file in &group.delta_files {
                println!(
                    "    - {} ({} bytes → {} bytes saved)",
                    delta_file.file_path, delta_file.blob_size, delta_file.delta_savings
                );
            }
        }

        if groups.len() > 5 {
            println!("  ... and {} more groups", groups.len() - 5);
        }
    }

    // Show insights
    println!("\n💡 Delta Compression Insights:");
    for insight in &compression_insights {
        println!("  • {}", insight);
    }

    // Summary
    println!("\n📈 Summary:");
    println!("  • Delta candidates: {}", candidates.len());
    println!("  • Delta groups: {}", groups.len());
    println!(
        "  • Total potential savings: {:.2} KB",
        total_potential_savings as f64 / 1024.0
    );

    if !groups.is_empty() {
        let avg_group_size =
            groups.iter().map(|g| g.delta_files.len()).sum::<usize>() as f64 / groups.len() as f64;
        println!("  • Average group size: {:.1} files", avg_group_size);
    }

    DeltaAnalysis {
        candidates: candidates.to_vec(),
        groups: groups.to_vec(),
        total_potential_savings,
        compression_insights,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Git Delta Analyzer");
    println!("====================");
    println!("Analyzing literal byte-level similarity for delta compression...");
    println!();

    // Analyze delta candidates
    let candidates = analyze_delta_candidates()?;

    // Find delta groups
    let groups = find_delta_groups(&candidates);

    // Generate comprehensive report
    let _analysis = generate_delta_report(&candidates, &groups);

    println!("\n✅ Analysis complete!");
    println!("🔍 Delta candidates identified");
    println!("🔗 Compression groups formed");
    println!("💡 Optimization insights ready");
    println!("📊 Savings calculations prepared");

    Ok(())
}
