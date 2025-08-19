use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct CrateAnalysis {
    crate_name: String,
    crate_type: CrateType,
    line_count: u32,
    file_count: u32,
    binary_count: u32,
    library_count: u32,
    dependencies: Vec<String>,
    contract_count: u32,
    test_count: u32,
    granularity_score: f64,
    recommendations: Vec<String>,
    stability_issues: Vec<String>,
}

#[derive(Debug, Clone)]
enum CrateType {
    Binary,
    Library,
    Mixed,
}

#[derive(Debug, Clone)]
enum GranularityLevel {
    Optimal,
    TooSmall,
    TooLarge,
    Critical,
}

#[derive(Debug)]
struct CrateGranularityAnalysis {
    crates: Vec<CrateAnalysis>,
    overall_score: f64,
    granularity_recommendations: Vec<String>,
    refactoring_plan: Vec<String>,
    contract_optimization: Vec<String>,
}

fn analyze_crate_granularity() -> Result<CrateGranularityAnalysis, Box<dyn std::error::Error>> {
    println!("📦 Analyzing crate granularity for contract optimization...");

    // Find all Cargo.toml files
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;

    let cargo_files = String::from_utf8(output.stdout)?;
    let mut crates = Vec::new();

    for cargo_file in cargo_files.lines() {
        if let Some(crate_analysis) = analyze_single_crate(cargo_file)? {
            crates.push(crate_analysis);
        }
    }

    // Sort by granularity score (best first)
    crates.sort_by(|a, b| {
        b.granularity_score
            .partial_cmp(&a.granularity_score)
            .unwrap()
    });

    // Generate recommendations
    let granularity_recommendations = generate_granularity_recommendations(&crates);
    let refactoring_plan = generate_refactoring_plan(&crates);
    let contract_optimization = generate_contract_optimization(&crates);
    let overall_score = calculate_overall_score(&crates);

    Ok(CrateGranularityAnalysis {
        crates,
        overall_score,
        granularity_recommendations,
        refactoring_plan,
        contract_optimization,
    })
}

fn analyze_single_crate(
    cargo_path: &str,
) -> Result<Option<CrateAnalysis>, Box<dyn std::error::Error>> {
    let crate_dir = Path::new(cargo_path).parent().ok_or("Invalid cargo path")?;
    let crate_name = crate_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Skip certain crates
    if should_skip_crate(&crate_name) {
        return Ok(None);
    }

    // Analyze crate structure
    let (line_count, file_count) = count_lines_and_files(crate_dir)?;
    let (binary_count, library_count) = count_binaries_and_libraries(crate_dir)?;
    let dependencies = extract_dependencies(cargo_path)?;
    let contract_count = count_contracts(crate_dir)?;
    let test_count = count_tests(crate_dir)?;

    // Determine crate type
    let crate_type = if binary_count > 0 && library_count > 0 {
        CrateType::Mixed
    } else if binary_count > 0 {
        CrateType::Binary
    } else {
        CrateType::Library
    };

    // Calculate granularity score
    let granularity_score = calculate_granularity_score(
        line_count,
        file_count,
        binary_count,
        library_count,
        contract_count,
        test_count,
        &crate_name,
    );

    // Generate recommendations
    let recommendations = generate_crate_recommendations(
        line_count,
        file_count,
        binary_count,
        library_count,
        contract_count,
        test_count,
        &crate_name,
        &crate_type,
    );

    // Identify stability issues
    let stability_issues = identify_stability_issues(
        line_count,
        file_count,
        contract_count,
        &crate_name,
        &crate_type,
    );

    Ok(Some(CrateAnalysis {
        crate_name,
        crate_type,
        line_count,
        file_count,
        binary_count,
        library_count,
        dependencies,
        contract_count,
        test_count,
        granularity_score,
        recommendations,
        stability_issues,
    }))
}

fn should_skip_crate(crate_name: &str) -> bool {
    let skip_patterns = [
        "target",
        "node_modules",
        ".git",
        ".cargo",
        "examples",
        "tests",
        "benches",
        "docs",
    ];

    for pattern in &skip_patterns {
        if crate_name.contains(pattern) {
            return true;
        }
    }

    false
}

fn count_lines_and_files(crate_dir: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[crate_dir.to_str().unwrap(), "-name", "*.rs", "-type", "f"])
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    let file_count = files.lines().count() as u32;

    let mut total_lines = 0u32;
    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            total_lines += content.lines().count() as u32;
        }
    }

    Ok((total_lines, file_count))
}

fn count_binaries_and_libraries(
    crate_dir: &Path,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let src_dir = crate_dir.join("src");
    let mut binary_count = 0u32;
    let mut library_count = 0u32;

    if src_dir.exists() {
        // Count binaries
        let bin_output = Command::new("find")
            .args(&[
                src_dir.to_str().unwrap(),
                "-name",
                "*.rs",
                "-path",
                "*/bin/*",
            ])
            .output()?;
        binary_count = String::from_utf8(bin_output.stdout)?.lines().count() as u32;

        // Count libraries (lib.rs or mod.rs files)
        let lib_output = Command::new("find")
            .args(&[
                src_dir.to_str().unwrap(),
                "-name",
                "lib.rs",
                "-o",
                "-name",
                "mod.rs",
            ])
            .output()?;
        library_count = String::from_utf8(lib_output.stdout)?.lines().count() as u32;
    }

    Ok((binary_count, library_count))
}

fn extract_dependencies(cargo_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Simplified dependency extraction
    let content = std::fs::read_to_string(cargo_path)?;
    let mut dependencies = Vec::new();

    for line in content.lines() {
        if line.contains("=") && (line.contains("path") || line.contains("git")) {
            if let Some(dep) = line.split('=').next() {
                dependencies.push(dep.trim().to_string());
            }
        }
    }

    Ok(dependencies)
}

fn count_contracts(crate_dir: &Path) -> Result<u32, Box<dyn std::error::Error>> {
    // Count files that might contain contracts
    let output = Command::new("find")
        .args(&[crate_dir.to_str().unwrap(), "-name", "*.rs", "-type", "f"])
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    let mut contract_count = 0u32;

    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("contract") || content.contains("Contract") {
                contract_count += 1;
            }
        }
    }

    Ok(contract_count)
}

fn count_tests(crate_dir: &Path) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("find")
        .args(&[crate_dir.to_str().unwrap(), "-name", "*.rs", "-type", "f"])
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    let mut test_count = 0u32;

    for file in files.lines() {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                test_count += 1;
            }
        }
    }

    Ok(test_count)
}

fn calculate_granularity_score(
    line_count: u32,
    file_count: u32,
    binary_count: u32,
    library_count: u32,
    contract_count: u32,
    test_count: u32,
    crate_name: &str,
) -> f64 {
    let mut score = 0.0;

    // Line count factor (0-0.3)
    if line_count >= 100 && line_count <= 800 {
        score += 0.3; // Optimal size
    } else if line_count >= 50 && line_count <= 1500 {
        score += 0.2; // Good size
    } else if line_count < 50 {
        score += 0.1; // Too small
    } else {
        score += 0.0; // Too large
    }

    // File count factor (0-0.2)
    if file_count >= 3 && file_count <= 15 {
        score += 0.2; // Optimal file count
    } else if file_count >= 1 && file_count <= 25 {
        score += 0.1; // Acceptable
    }

    // Contract count factor (0-0.2)
    if contract_count == 1 {
        score += 0.2; // Perfect - one contract per crate
    } else if contract_count > 1 {
        score += 0.1; // Multiple contracts - consider splitting
    }

    // Test coverage factor (0-0.2)
    if test_count > 0 {
        score += 0.2; // Has tests
    }

    // Crate type factor (0-0.1)
    if binary_count == 1 && library_count == 0 {
        score += 0.1; // Single binary crate
    } else if library_count == 1 && binary_count == 0 {
        score += 0.1; // Single library crate
    }

    score
}

fn generate_crate_recommendations(
    line_count: u32,
    file_count: u32,
    binary_count: u32,
    library_count: u32,
    contract_count: u32,
    test_count: u32,
    crate_name: &str,
    crate_type: &CrateType,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Size recommendations
    if line_count > 1500 {
        recommendations.push("Consider splitting into smaller crates".to_string());
    } else if line_count < 50 {
        recommendations.push("Consider merging with related crates".to_string());
    }

    // Contract recommendations
    if contract_count > 1 {
        recommendations.push("Multiple contracts detected - consider splitting".to_string());
    } else if contract_count == 0 {
        recommendations.push("No contracts found - ensure contract coverage".to_string());
    }

    // Test recommendations
    if test_count == 0 {
        recommendations.push("Add tests for contract validation".to_string());
    }

    // Type-specific recommendations
    match crate_type {
        CrateType::Mixed => {
            recommendations.push("Mixed binary/library - consider separating concerns".to_string());
        }
        CrateType::Binary => {
            if binary_count > 1 {
                recommendations.push("Multiple binaries - consider separate crates".to_string());
            }
        }
        CrateType::Library => {
            if library_count > 1 {
                recommendations.push("Multiple libraries - consider separate crates".to_string());
            }
        }
    }

    recommendations
}

fn identify_stability_issues(
    line_count: u32,
    file_count: u32,
    contract_count: u32,
    crate_name: &str,
    crate_type: &CrateType,
) -> Vec<String> {
    let mut issues = Vec::new();

    if line_count > 2000 {
        issues.push("Very large crate - high SHA churn risk".to_string());
    }

    if file_count > 30 {
        issues.push("Many files - consider modularization".to_string());
    }

    if contract_count > 2 {
        issues.push("Multiple contracts - violates single responsibility".to_string());
    }

    if crate_name.contains("utils") && line_count < 100 {
        issues.push("Small utility crate - consider merging".to_string());
    }

    issues
}

fn generate_granularity_recommendations(crates: &[CrateAnalysis]) -> Vec<String> {
    let mut recommendations = Vec::new();

    let optimal_crates = crates.iter().filter(|c| c.granularity_score > 0.7).count();
    let too_large_crates = crates.iter().filter(|c| c.line_count > 1500).count();
    let too_small_crates = crates.iter().filter(|c| c.line_count < 50).count();

    if optimal_crates < crates.len() / 2 {
        recommendations.push("Most crates need granularity optimization".to_string());
    }

    if too_large_crates > 0 {
        recommendations.push("Large crates detected - prioritize splitting".to_string());
    }

    if too_small_crates > 0 {
        recommendations.push("Small crates detected - consider merging".to_string());
    }

    recommendations
        .push("Aim for 100-800 LOC per crate for optimal contract granularity".to_string());
    recommendations.push("Keep crates focused on single contract responsibility".to_string());
    recommendations.push("Ensure each crate has independent testability".to_string());

    recommendations
}

fn generate_refactoring_plan(crates: &[CrateAnalysis]) -> Vec<String> {
    let mut plan = Vec::new();

    // Identify crates to split
    let large_crates: Vec<_> = crates.iter().filter(|c| c.line_count > 1500).collect();
    if !large_crates.is_empty() {
        plan.push("Phase 1 - Split Large Crates:".to_string());
        for crate_analysis in large_crates {
            plan.push(format!(
                "  • Split {} ({} LOC) into focused modules",
                crate_analysis.crate_name, crate_analysis.line_count
            ));
        }
    }

    // Identify crates to merge
    let small_crates: Vec<_> = crates.iter().filter(|c| c.line_count < 50).collect();
    if !small_crates.is_empty() {
        plan.push("Phase 2 - Merge Small Crates:".to_string());
        for crate_analysis in small_crates {
            plan.push(format!(
                "  • Merge {} ({} LOC) with related crates",
                crate_analysis.crate_name, crate_analysis.line_count
            ));
        }
    }

    // Contract optimization
    let multi_contract_crates: Vec<_> = crates.iter().filter(|c| c.contract_count > 1).collect();
    if !multi_contract_crates.is_empty() {
        plan.push("Phase 3 - Contract Separation:".to_string());
        for crate_analysis in multi_contract_crates {
            plan.push(format!(
                "  • Separate contracts in {} ({} contracts)",
                crate_analysis.crate_name, crate_analysis.contract_count
            ));
        }
    }

    plan.push("Phase 4 - Test Coverage:".to_string());
    plan.push("  • Ensure each crate has comprehensive tests".to_string());
    plan.push("  • Add contract validation tests".to_string());

    plan
}

fn generate_contract_optimization(crates: &[CrateAnalysis]) -> Vec<String> {
    let mut optimizations = Vec::new();

    let total_contracts: u32 = crates.iter().map(|c| c.contract_count).sum();
    let total_crates = crates.len() as u32;

    optimizations.push(format!(
        "Total contracts: {} across {} crates",
        total_contracts, total_crates
    ));

    if total_contracts > total_crates {
        optimizations
            .push("Multiple contracts per crate - optimize for single responsibility".to_string());
    }

    let contract_crates: Vec<_> = crates.iter().filter(|c| c.contract_count > 0).collect();
    optimizations.push(format!("Crates with contracts: {}", contract_crates.len()));

    for crate_analysis in contract_crates {
        optimizations.push(format!(
            "  • {}: {} contracts",
            crate_analysis.crate_name, crate_analysis.contract_count
        ));
    }

    optimizations.push("Use crate SHA for contract memoization".to_string());
    optimizations.push("Implement crate-level fix plan caching".to_string());
    optimizations.push("Ensure contract boundaries align with crate boundaries".to_string());

    optimizations
}

fn calculate_overall_score(crates: &[CrateAnalysis]) -> f64 {
    if crates.is_empty() {
        return 0.0;
    }

    let total_score: f64 = crates.iter().map(|c| c.granularity_score).sum();
    total_score / crates.len() as f64
}

fn generate_crate_granularity_report(analysis: &CrateGranularityAnalysis) {
    println!("\n📦 Crate Granularity Analysis");
    println!("=============================");

    // Show crate analysis
    println!("\n📊 Crate Analysis:");
    for crate_analysis in &analysis.crates {
        let crate_type_icon = match crate_analysis.crate_type {
            CrateType::Binary => "🔧",
            CrateType::Library => "📚",
            CrateType::Mixed => "🔀",
        };

        let granularity_icon = if crate_analysis.granularity_score > 0.7 {
            "🟢"
        } else if crate_analysis.granularity_score > 0.5 {
            "🟡"
        } else {
            "🔴"
        };

        println!(
            "  {} {} {} ({} LOC, {} files, {:.1} score)",
            granularity_icon,
            crate_type_icon,
            crate_analysis.crate_name,
            crate_analysis.line_count,
            crate_analysis.file_count,
            crate_analysis.granularity_score
        );

        if crate_analysis.contract_count > 0 {
            println!(
                "    Contracts: {} | Tests: {}",
                crate_analysis.contract_count, crate_analysis.test_count
            );
        }

        if !crate_analysis.recommendations.is_empty() {
            println!(
                "    Recommendations: {}",
                crate_analysis.recommendations.join(", ")
            );
        }

        if !crate_analysis.stability_issues.is_empty() {
            println!("    Issues: {}", crate_analysis.stability_issues.join(", "));
        }
        println!();
    }

    // Show statistics
    println!("\n📈 Granularity Statistics:");
    println!("  • Overall score: {:.1}%", analysis.overall_score * 100.0);
    println!("  • Total crates: {}", analysis.crates.len());

    let optimal_count = analysis
        .crates
        .iter()
        .filter(|c| c.granularity_score > 0.7)
        .count();
    let large_count = analysis
        .crates
        .iter()
        .filter(|c| c.line_count > 1500)
        .count();
    let small_count = analysis.crates.iter().filter(|c| c.line_count < 50).count();

    println!("  • Optimal crates: {}", optimal_count);
    println!("  • Large crates (>1500 LOC): {}", large_count);
    println!("  • Small crates (<50 LOC): {}", small_count);

    // Show recommendations
    if !analysis.granularity_recommendations.is_empty() {
        println!("\n💡 Granularity Recommendations:");
        for rec in &analysis.granularity_recommendations {
            println!("  • {}", rec);
        }
    }

    // Show refactoring plan
    if !analysis.refactoring_plan.is_empty() {
        println!("\n🔧 Refactoring Plan:");
        for step in &analysis.refactoring_plan {
            println!("  {}", step);
        }
    }

    // Show contract optimization
    if !analysis.contract_optimization.is_empty() {
        println!("\n🎯 Contract Optimization:");
        for opt in &analysis.contract_optimization {
            println!("  • {}", opt);
        }
    }

    // Summary
    println!("\n📋 Summary:");
    if analysis.overall_score > 0.7 {
        println!("  • Excellent crate granularity");
        println!("  • Optimal for contract memoization");
        println!("  • Good SHA stability");
    } else if analysis.overall_score > 0.5 {
        println!("  • Moderate crate granularity");
        println!("  • Consider optimizations");
        println!("  • Monitor large crates");
    } else {
        println!("  • Poor crate granularity");
        println!("  • Prioritize refactoring");
        println!("  • Implement aggressive splitting");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Crate Granularity Analyzer");
    println!("=============================");
    println!("Analyzing crate sizes for optimal contract modularity...");
    println!();

    // Analyze crate granularity
    let analysis = analyze_crate_granularity()?;

    // Generate comprehensive report
    generate_crate_granularity_report(&analysis);

    println!("\n✅ Crate granularity analysis complete!");
    println!("📦 Crates analyzed");
    println!("🎯 Granularity optimized");
    println!("🔧 Refactoring plan ready");
    println!("📊 Contract boundaries defined");

    Ok(())
}
