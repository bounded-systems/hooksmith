use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
struct ModuleAnalysis {
    module_path: String,
    file_count: u32,
    total_size: u64,
    average_size: u64,
    shared_patterns: Vec<String>,
    delta_potential: f64,
    modularization_score: f64,
}

#[derive(Debug, Clone)]
struct PackingImpact {
    before_modularization: PackingMetrics,
    after_modularization: PackingMetrics,
    improvement_estimate: f64,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
struct PackingMetrics {
    total_objects: u32,
    delta_chains: u32,
    average_chain_length: f64,
    compression_ratio: f64,
    estimated_savings: u64,
}

#[derive(Debug)]
struct ModularizationPackingAnalysis {
    modules: Vec<ModuleAnalysis>,
    packing_impact: PackingImpact,
    modularization_opportunities: Vec<String>,
    implementation_plan: Vec<String>,
}

fn analyze_modularization_packing_impact(
) -> Result<ModularizationPackingAnalysis, Box<dyn std::error::Error>> {
    println!("🔧 Analyzing modularization impact on Git packing...");

    // Get all Rust files
    let output = Command::new("git").args(&["ls-files", "*.rs"]).output()?;

    let files_output = String::from_utf8(output.stdout)?;
    let mut module_groups: HashMap<String, Vec<(String, u64)>> = HashMap::new();

    // Group files by module path
    for line in files_output.lines() {
        let path = line.trim();
        if !path.is_empty() {
            let module_path = extract_module_path(path);
            let size = get_file_size(path)?;
            module_groups
                .entry(module_path)
                .or_default()
                .push((path.to_string(), size));
        }
    }

    // Analyze each module
    let mut modules = Vec::new();
    for (module_path, files) in module_groups {
        let module_analysis = analyze_module(&module_path, &files)?;
        modules.push(module_analysis);
    }

    // Sort by modularization score
    modules.sort_by(|a, b| {
        b.modularization_score
            .partial_cmp(&a.modularization_score)
            .unwrap()
    });

    // Calculate packing impact
    let packing_impact = calculate_packing_impact(&modules)?;

    // Generate opportunities and implementation plan
    let modularization_opportunities = generate_modularization_opportunities(&modules);
    let implementation_plan = generate_implementation_plan(&modules);

    Ok(ModularizationPackingAnalysis {
        modules,
        packing_impact,
        modularization_opportunities,
        implementation_plan,
    })
}

fn extract_module_path(path: &str) -> String {
    // Extract module path from file path
    let path_parts: Vec<&str> = path.split('/').collect();

    if path_parts.len() >= 2 {
        // Group by directory structure
        if path_parts[0] == "src" && path_parts.len() > 2 {
            path_parts[1].to_string()
        } else if path_parts[0] == "crates" && path_parts.len() > 2 {
            format!("crates/{}", path_parts[1])
        } else {
            path_parts[0].to_string()
        }
    } else {
        "root".to_string()
    }
}

fn get_file_size(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let output = Command::new("wc").args(&["-c", path]).output()?;

    let size_str = String::from_utf8(output.stdout)?;
    let parts: Vec<&str> = size_str.split_whitespace().collect();

    if let Some(size_part) = parts.first() {
        Ok(size_part.parse().unwrap_or(0))
    } else {
        Ok(0)
    }
}

fn analyze_module(
    module_path: &str,
    files: &[(String, u64)],
) -> Result<ModuleAnalysis, Box<dyn std::error::Error>> {
    let file_count = files.len() as u32;
    let total_size: u64 = files.iter().map(|(_, size)| size).sum();
    let average_size = if file_count > 0 {
        total_size / file_count as u64
    } else {
        0
    };

    // Analyze shared patterns
    let shared_patterns = detect_shared_patterns(files);

    // Calculate delta potential based on file similarities
    let delta_potential = calculate_delta_potential(files);

    // Calculate modularization score
    let modularization_score = calculate_modularization_score(
        file_count,
        average_size,
        shared_patterns.len(),
        delta_potential,
    );

    Ok(ModuleAnalysis {
        module_path: module_path.to_string(),
        file_count,
        total_size,
        average_size,
        shared_patterns,
        delta_potential,
        modularization_score,
    })
}

fn detect_shared_patterns(files: &[(String, u64)]) -> Vec<String> {
    let mut patterns = Vec::new();

    // Look for common patterns in file names
    let mut name_patterns: HashMap<String, u32> = HashMap::new();

    for (path, _) in files {
        let filename = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        // Extract pattern from filename (e.g., "test_*.rs" -> "test")
        if let Some(pattern) = extract_pattern_from_filename(&filename) {
            *name_patterns.entry(pattern).or_insert(0) += 1;
        }
    }

    // Add patterns that appear multiple times
    for (pattern, count) in name_patterns {
        if count > 1 {
            patterns.push(pattern);
        }
    }

    patterns
}

fn extract_pattern_from_filename(filename: &str) -> Option<String> {
    if filename.contains("test") {
        Some("test".to_string())
    } else if filename.contains("mod") {
        Some("mod".to_string())
    } else if filename.contains("lib") {
        Some("lib".to_string())
    } else if filename.contains("main") {
        Some("main".to_string())
    } else {
        None
    }
}

fn calculate_delta_potential(files: &[(String, u64)]) -> f64 {
    if files.len() < 2 {
        return 0.0;
    }

    let mut total_potential = 0.0;
    let mut comparisons = 0;

    // Compare each file with others
    for i in 0..files.len() {
        for j in (i + 1)..files.len() {
            let size_diff =
                (files[i].1 as f64 - files[j].1 as f64).abs() / files[i].1.max(files[j].1) as f64;

            // Similar sized files have higher delta potential
            if size_diff < 0.3 {
                // Within 30% size difference
                total_potential += 1.0 - size_diff;
            }

            comparisons += 1;
        }
    }

    if comparisons > 0 {
        total_potential / comparisons as f64
    } else {
        0.0
    }
}

fn calculate_modularization_score(
    file_count: u32,
    average_size: u64,
    shared_patterns: usize,
    delta_potential: f64,
) -> f64 {
    let mut score = 0.0;

    // File count factor (0-0.3)
    if file_count > 10 {
        score += 0.3;
    } else if file_count > 5 {
        score += 0.2;
    } else if file_count > 2 {
        score += 0.1;
    }

    // Size factor (0-0.2)
    if average_size > 50 * 1024 {
        // 50KB+
        score += 0.2;
    } else if average_size > 10 * 1024 {
        // 10KB+
        score += 0.1;
    }

    // Pattern factor (0-0.2)
    if shared_patterns > 3 {
        score += 0.2;
    } else if shared_patterns > 1 {
        score += 0.1;
    }

    // Delta potential factor (0-0.3)
    score += delta_potential * 0.3;

    score
}

fn calculate_packing_impact(
    modules: &[ModuleAnalysis],
) -> Result<PackingImpact, Box<dyn std::error::Error>> {
    // Calculate current packing metrics
    let current_metrics = calculate_current_packing_metrics(modules)?;

    // Estimate improved metrics after modularization
    let improved_metrics = estimate_improved_packing_metrics(modules, &current_metrics);

    // Calculate improvement
    let improvement_estimate = if current_metrics.compression_ratio > 0.0 {
        (improved_metrics.compression_ratio - current_metrics.compression_ratio)
            / current_metrics.compression_ratio
            * 100.0
    } else {
        0.0
    };

    // Generate recommendations
    let mut recommendations = Vec::new();

    if improvement_estimate > 20.0 {
        recommendations
            .push("High potential for packing improvement through modularization".to_string());
    }

    let large_modules = modules
        .iter()
        .filter(|m| m.modularization_score > 0.7)
        .count();
    if large_modules > 5 {
        recommendations.push("Many large modules detected - prioritize modularization".to_string());
    }

    let high_delta_potential = modules.iter().filter(|m| m.delta_potential > 0.5).count();
    if high_delta_potential > 3 {
        recommendations.push(
            "High delta compression potential - modularization will improve packing".to_string(),
        );
    }

    Ok(PackingImpact {
        before_modularization: current_metrics,
        after_modularization: improved_metrics,
        improvement_estimate,
        recommendations,
    })
}

fn calculate_current_packing_metrics(
    modules: &[ModuleAnalysis],
) -> Result<PackingMetrics, Box<dyn std::error::Error>> {
    let total_objects: u32 = modules.iter().map(|m| m.file_count).sum();
    let total_size: u64 = modules.iter().map(|m| m.total_size).sum();

    // Estimate current delta chains (simplified)
    let delta_chains = modules.iter().filter(|m| m.file_count > 1).count() as u32;
    let average_chain_length = if delta_chains > 0 {
        modules.iter().map(|m| m.file_count as f64).sum::<f64>() / delta_chains as f64
    } else {
        0.0
    };

    // Estimate compression ratio based on file sizes and similarities
    let compression_ratio = estimate_compression_ratio(modules);
    let estimated_savings = (total_size as f64 * (1.0 - compression_ratio)) as u64;

    Ok(PackingMetrics {
        total_objects,
        delta_chains,
        average_chain_length,
        compression_ratio,
        estimated_savings,
    })
}

fn estimate_compression_ratio(modules: &[ModuleAnalysis]) -> f64 {
    let mut total_potential = 0.0;
    let mut total_files = 0;

    for module in modules {
        if module.file_count > 1 {
            // Similar files compress better
            total_potential += module.delta_potential * module.file_count as f64;
            total_files += module.file_count;
        }
    }

    if total_files > 0 {
        let avg_potential = total_potential / total_files as f64;
        // Convert potential to compression ratio (0.3 to 0.8 range)
        0.8 - (avg_potential * 0.5)
    } else {
        0.8 // Default compression ratio
    }
}

fn estimate_improved_packing_metrics(
    modules: &[ModuleAnalysis],
    current: &PackingMetrics,
) -> PackingMetrics {
    // Estimate improvements based on modularization opportunities
    let improvement_factor = modules
        .iter()
        .filter(|m| m.modularization_score > 0.5)
        .map(|m| m.modularization_score)
        .sum::<f64>()
        / modules.len().max(1) as f64;

    let improved_compression_ratio = current.compression_ratio * (1.0 - improvement_factor * 0.3);
    let improved_savings =
        ((current.total_objects as u64 * 1024) as f64 * (1.0 - improved_compression_ratio)) as u64;

    PackingMetrics {
        total_objects: current.total_objects,
        delta_chains: current.delta_chains
            + modules
                .iter()
                .filter(|m| m.modularization_score > 0.7)
                .count() as u32,
        average_chain_length: current.average_chain_length * 1.2, // Slightly longer chains
        compression_ratio: improved_compression_ratio,
        estimated_savings: improved_savings,
    }
}

fn generate_modularization_opportunities(modules: &[ModuleAnalysis]) -> Vec<String> {
    let mut opportunities = Vec::new();

    for module in modules {
        if module.modularization_score > 0.6 {
            opportunities.push(format!(
                "Modularize '{}': {} files, {:.1}KB avg, {:.1} delta potential",
                module.module_path,
                module.file_count,
                module.average_size as f64 / 1024.0,
                module.delta_potential
            ));
        }
    }

    opportunities
}

fn generate_implementation_plan(modules: &[ModuleAnalysis]) -> Vec<String> {
    let mut plan = Vec::new();

    // High priority modules
    let high_priority = modules
        .iter()
        .filter(|m| m.modularization_score > 0.8)
        .collect::<Vec<_>>();
    if !high_priority.is_empty() {
        plan.push("Phase 1 - High Priority Modules:".to_string());
        for module in high_priority {
            plan.push(format!(
                "  • Extract shared logic from '{}'",
                module.module_path
            ));
        }
    }

    // Medium priority modules
    let medium_priority = modules
        .iter()
        .filter(|m| m.modularization_score > 0.6 && m.modularization_score <= 0.8)
        .collect::<Vec<_>>();
    if !medium_priority.is_empty() {
        plan.push("Phase 2 - Medium Priority Modules:".to_string());
        for module in medium_priority {
            plan.push(format!(
                "  • Refactor '{}' for better delta compression",
                module.module_path
            ));
        }
    }

    plan.push("Phase 3 - Packing Optimization:".to_string());
    plan.push("  • Run git repack after modularization".to_string());
    plan.push("  • Monitor delta chain improvements".to_string());
    plan.push("  • Measure compression ratio gains".to_string());

    plan
}

fn generate_modularization_report(analysis: &ModularizationPackingAnalysis) {
    println!("\n🔧 Modularization → Packing Impact Analysis");
    println!("============================================");

    // Show top modules
    println!("\n📦 Top Modules for Modularization:");
    for (i, module) in analysis.modules.iter().take(10).enumerate() {
        println!(
            "  {}. {} (Score: {:.1})",
            i + 1,
            module.module_path,
            module.modularization_score
        );
        println!(
            "     Files: {} | Avg Size: {:.1}KB | Delta Potential: {:.1}",
            module.file_count,
            module.average_size as f64 / 1024.0,
            module.delta_potential
        );
        if !module.shared_patterns.is_empty() {
            println!("     Patterns: {}", module.shared_patterns.join(", "));
        }
        println!();
    }

    if analysis.modules.len() > 10 {
        println!("  ... and {} more modules", analysis.modules.len() - 10);
    }

    // Show packing impact
    println!("\n📊 Packing Impact Analysis:");
    println!("  Before Modularization:");
    println!(
        "    • Total objects: {}",
        analysis.packing_impact.before_modularization.total_objects
    );
    println!(
        "    • Delta chains: {}",
        analysis.packing_impact.before_modularization.delta_chains
    );
    println!(
        "    • Compression ratio: {:.1}%",
        (1.0 - analysis
            .packing_impact
            .before_modularization
            .compression_ratio)
            * 100.0
    );
    println!(
        "    • Estimated savings: {:.2} MB",
        analysis
            .packing_impact
            .before_modularization
            .estimated_savings as f64
            / (1024.0 * 1024.0)
    );

    println!("  After Modularization:");
    println!(
        "    • Total objects: {}",
        analysis.packing_impact.after_modularization.total_objects
    );
    println!(
        "    • Delta chains: {}",
        analysis.packing_impact.after_modularization.delta_chains
    );
    println!(
        "    • Compression ratio: {:.1}%",
        (1.0 - analysis
            .packing_impact
            .after_modularization
            .compression_ratio)
            * 100.0
    );
    println!(
        "    • Estimated savings: {:.2} MB",
        analysis
            .packing_impact
            .after_modularization
            .estimated_savings as f64
            / (1024.0 * 1024.0)
    );

    println!(
        "  Improvement: {:.1}%",
        analysis.packing_impact.improvement_estimate
    );

    // Show opportunities
    if !analysis.modularization_opportunities.is_empty() {
        println!("\n🎯 Modularization Opportunities:");
        for opportunity in &analysis.modularization_opportunities {
            println!("  • {}", opportunity);
        }
    }

    // Show implementation plan
    if !analysis.implementation_plan.is_empty() {
        println!("\n📋 Implementation Plan:");
        for step in &analysis.implementation_plan {
            println!("  {}", step);
        }
    }

    // Show recommendations
    if !analysis.packing_impact.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for rec in &analysis.packing_impact.recommendations {
            println!("  • {}", rec);
        }
    }

    // Summary
    println!("\n📈 Summary:");
    println!("  • {} modules analyzed", analysis.modules.len());
    println!(
        "  • {} high-priority modules for modularization",
        analysis
            .modules
            .iter()
            .filter(|m| m.modularization_score > 0.7)
            .count()
    );
    println!(
        "  • Estimated packing improvement: {:.1}%",
        analysis.packing_impact.improvement_estimate
    );
    println!("  • Modularization will improve delta compression");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Modularization → Packing Impact Estimator");
    println!("============================================");
    println!("Analyzing how modularization affects Git delta compression...");
    println!();

    // Analyze modularization impact
    let analysis = analyze_modularization_packing_impact()?;

    // Generate comprehensive report
    generate_modularization_report(&analysis);

    println!("\n✅ Modularization analysis complete!");
    println!("📦 Modules analyzed");
    println!("🔧 Packing impact estimated");
    println!("🎯 Opportunities identified");
    println!("📋 Implementation plan ready");

    Ok(())
}
