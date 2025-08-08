use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct CrateStabilityInfo {
    crate_name: String,
    crate_path: String,
    change_velocity: f64,
    api_stability_score: f64,
    coupling_score: f64,
    extraction_readiness: ExtractionReadiness,
    git_commits: u32,
    recent_changes: u32,
    dependent_crates: Vec<String>,
    api_surface_changes: u32,
    stability_issues: Vec<String>,
    extraction_recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum ExtractionReadiness {
    Ready,
    NeedsStabilization,
    TooVolatile,
    NotReady,
}

#[derive(Debug, Clone)]
struct CrateDependency {
    dependent_crate: String,
    usage_count: u32,
    critical_path: bool,
}

#[derive(Debug)]
struct CrateStabilityAnalysis {
    crates: Vec<CrateStabilityInfo>,
    extraction_candidates: Vec<String>,
    stabilization_plan: Vec<String>,
    external_repo_strategy: Vec<String>,
    overall_stability_score: f64,
}

fn analyze_crate_stability() -> Result<CrateStabilityAnalysis, Box<dyn std::error::Error>> {
    println!("🔧 Analyzing crate stability for extraction readiness...");

    // Find all Cargo.toml files
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;

    let cargo_files = String::from_utf8(output.stdout)?;
    let mut crates = Vec::new();

    for cargo_file in cargo_files.lines() {
        if let Some(crate_analysis) = analyze_single_crate_stability(cargo_file)? {
            crates.push(crate_analysis);
        }
    }

    // Sort by extraction readiness (most ready first)
    crates.sort_by(|a, b| {
        b.extraction_readiness
            .partial_cmp(&a.extraction_readiness)
            .unwrap()
    });

    // Generate analysis
    let extraction_candidates = identify_extraction_candidates(&crates);
    let stabilization_plan = generate_stabilization_plan(&crates);
    let external_repo_strategy = generate_external_repo_strategy(&crates);
    let overall_stability_score = calculate_overall_stability_score(&crates);

    Ok(CrateStabilityAnalysis {
        crates,
        extraction_candidates,
        stabilization_plan,
        external_repo_strategy,
        overall_stability_score,
    })
}

fn analyze_single_crate_stability(
    cargo_path: &str,
) -> Result<Option<CrateStabilityInfo>, Box<dyn std::error::Error>> {
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

    let crate_path = crate_dir.to_string_lossy().to_string();

    // Analyze Git history
    let git_commits = count_git_commits(&crate_path)?;
    let recent_changes = count_recent_changes(&crate_path)?;
    let change_velocity = calculate_change_velocity(git_commits, recent_changes)?;

    // Analyze API stability
    let api_surface_changes = count_api_surface_changes(&crate_path)?;
    let api_stability_score = calculate_api_stability_score(api_surface_changes, git_commits)?;

    // Analyze coupling
    let dependent_crates = find_dependent_crates(&crate_name)?;
    let coupling_score = calculate_coupling_score(&dependent_crates)?;

    // Determine extraction readiness
    let extraction_readiness = determine_extraction_readiness(
        change_velocity,
        api_stability_score,
        coupling_score,
        git_commits,
        &dependent_crates,
    );

    // Identify stability issues
    let stability_issues = identify_stability_issues(
        change_velocity,
        api_stability_score,
        coupling_score,
        git_commits,
        &crate_name,
    );

    // Generate extraction recommendations
    let extraction_recommendations = generate_extraction_recommendations(
        &extraction_readiness,
        &crate_name,
        change_velocity,
        api_stability_score,
        coupling_score,
    );

    Ok(Some(CrateStabilityInfo {
        crate_name,
        crate_path,
        change_velocity,
        api_stability_score,
        coupling_score,
        extraction_readiness,
        git_commits,
        recent_changes,
        dependent_crates,
        api_surface_changes,
        stability_issues,
        extraction_recommendations,
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

fn count_git_commits(crate_path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--", crate_path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().count() as u32)
}

fn count_recent_changes(crate_path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["log", "--oneline", "--since", "30 days", "--", crate_path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    Ok(commits.lines().count() as u32)
}

fn calculate_change_velocity(
    git_commits: u32,
    recent_changes: u32,
) -> Result<f64, Box<dyn std::error::Error>> {
    if git_commits == 0 {
        return Ok(0.0);
    }

    // Calculate velocity as recent changes / total commits
    let velocity = recent_changes as f64 / git_commits as f64;
    Ok(velocity.min(1.0))
}

fn count_api_surface_changes(crate_path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // Look for public API changes in recent commits
    let output = Command::new("git")
        .args(&["log", "--oneline", "--since", "30 days", "--", crate_path])
        .output()?;

    let commits = String::from_utf8(output.stdout)?;
    let mut api_changes = 0u32;

    for line in commits.lines() {
        if line.contains("pub")
            || line.contains("struct")
            || line.contains("enum")
            || line.contains("trait")
        {
            api_changes += 1;
        }
    }

    Ok(api_changes)
}

fn calculate_api_stability_score(
    api_surface_changes: u32,
    git_commits: u32,
) -> Result<f64, Box<dyn std::error::Error>> {
    if git_commits == 0 {
        return Ok(1.0);
    }

    // API stability = 1 - (api_changes / total_commits)
    let stability = 1.0 - (api_surface_changes as f64 / git_commits as f64);
    Ok(stability.max(0.0))
}

fn find_dependent_crates(crate_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Find crates that depend on this crate
    let output = Command::new("find")
        .args(&[".", "-name", "Cargo.toml", "-type", "f"])
        .output()?;

    let cargo_files = String::from_utf8(output.stdout)?;
    let mut dependents = Vec::new();

    for cargo_file in cargo_files.lines() {
        if let Ok(content) = std::fs::read_to_string(cargo_file) {
            if content.contains(crate_name) && !cargo_file.contains(crate_name) {
                if let Some(dep_crate) = Path::new(cargo_file).parent().and_then(|p| p.file_name())
                {
                    dependents.push(dep_crate.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(dependents)
}

fn calculate_coupling_score(
    dependent_crates: &[String],
) -> Result<f64, Box<dyn std::error::Error>> {
    // Higher coupling = more dependents
    let coupling = dependent_crates.len() as f64;

    // Normalize to 0-1 scale (0 = no coupling, 1 = high coupling)
    let normalized_coupling = (coupling / 10.0).min(1.0);
    Ok(normalized_coupling)
}

fn determine_extraction_readiness(
    change_velocity: f64,
    api_stability_score: f64,
    coupling_score: f64,
    git_commits: u32,
    dependent_crates: &[String],
) -> ExtractionReadiness {
    // Ready for extraction if:
    // - Low change velocity (< 0.3)
    // - High API stability (> 0.7)
    // - Multiple dependents (> 2)
    // - Sufficient history (> 10 commits)

    if change_velocity < 0.3
        && api_stability_score > 0.7
        && dependent_crates.len() > 2
        && git_commits > 10
    {
        ExtractionReadiness::Ready
    } else if change_velocity < 0.5 && api_stability_score > 0.5 && dependent_crates.len() > 1 {
        ExtractionReadiness::NeedsStabilization
    } else if change_velocity > 0.8 {
        ExtractionReadiness::TooVolatile
    } else {
        ExtractionReadiness::NotReady
    }
}

fn identify_stability_issues(
    change_velocity: f64,
    api_stability_score: f64,
    coupling_score: f64,
    git_commits: u32,
    crate_name: &str,
) -> Vec<String> {
    let mut issues = Vec::new();

    if change_velocity > 0.7 {
        issues.push("High change velocity - too volatile for extraction".to_string());
    }

    if api_stability_score < 0.5 {
        issues.push("Low API stability - frequent interface changes".to_string());
    }

    if git_commits < 5 {
        issues.push("Insufficient Git history for extraction".to_string());
    }

    if coupling_score < 0.1 {
        issues.push("Low coupling - not used by other crates".to_string());
    }

    if crate_name.contains("utils") && git_commits < 10 {
        issues.push("Utility crate needs more development history".to_string());
    }

    issues
}

fn generate_extraction_recommendations(
    extraction_readiness: &ExtractionReadiness,
    crate_name: &str,
    change_velocity: f64,
    api_stability_score: f64,
    coupling_score: f64,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match extraction_readiness {
        ExtractionReadiness::Ready => {
            recommendations.push("Ready for extraction to external repository".to_string());
            recommendations.push("Create dedicated repo with CI/CD".to_string());
            recommendations.push("Set up versioning and publishing".to_string());
        }
        ExtractionReadiness::NeedsStabilization => {
            recommendations.push("Stabilize API before extraction".to_string());
            recommendations.push("Reduce change velocity".to_string());
            recommendations.push("Add comprehensive tests".to_string());
        }
        ExtractionReadiness::TooVolatile => {
            recommendations.push("Too volatile - keep internal for now".to_string());
            recommendations.push("Focus on API stability".to_string());
            recommendations.push("Reduce coupling with other crates".to_string());
        }
        ExtractionReadiness::NotReady => {
            recommendations.push("Not ready for extraction".to_string());
            recommendations.push("Build more usage and stability".to_string());
            recommendations.push("Establish clear API boundaries".to_string());
        }
    }

    recommendations
}

fn identify_extraction_candidates(crates: &[CrateStabilityInfo]) -> Vec<String> {
    crates
        .iter()
        .filter(|c| matches!(c.extraction_readiness, ExtractionReadiness::Ready))
        .map(|c| c.crate_name.clone())
        .collect()
}

fn generate_stabilization_plan(crates: &[CrateStabilityInfo]) -> Vec<String> {
    let mut plan = Vec::new();

    let needs_stabilization: Vec<_> = crates
        .iter()
        .filter(|c| {
            matches!(
                c.extraction_readiness,
                ExtractionReadiness::NeedsStabilization
            )
        })
        .collect();

    if !needs_stabilization.is_empty() {
        plan.push("Phase 1 - Stabilize APIs:".to_string());
        for crate_analysis in needs_stabilization {
            plan.push(format!(
                "  • {}: Reduce change velocity ({:.1})",
                crate_analysis.crate_name, crate_analysis.change_velocity
            ));
            plan.push(format!(
                "    Improve API stability ({:.1})",
                crate_analysis.api_stability_score
            ));
        }
    }

    let too_volatile: Vec<_> = crates
        .iter()
        .filter(|c| matches!(c.extraction_readiness, ExtractionReadiness::TooVolatile))
        .collect();

    if !too_volatile.is_empty() {
        plan.push("Phase 2 - Reduce Volatility:".to_string());
        for crate_analysis in too_volatile {
            plan.push(format!(
                "  • {}: High change velocity ({:.1})",
                crate_analysis.crate_name, crate_analysis.change_velocity
            ));
            plan.push(format!(
                "    Focus on API stability ({:.1})",
                crate_analysis.api_stability_score
            ));
        }
    }

    plan.push("Phase 3 - Build Usage:".to_string());
    plan.push("  • Increase coupling with other crates".to_string());
    plan.push("  • Establish clear API boundaries".to_string());
    plan.push("  • Add comprehensive test coverage".to_string());

    plan
}

fn generate_external_repo_strategy(crates: &[CrateStabilityInfo]) -> Vec<String> {
    let mut strategy = Vec::new();

    let ready_crates: Vec<_> = crates
        .iter()
        .filter(|c| matches!(c.extraction_readiness, ExtractionReadiness::Ready))
        .collect();

    if !ready_crates.is_empty() {
        strategy.push("Immediate Extraction Candidates:".to_string());
        for crate_analysis in ready_crates {
            strategy.push(format!(
                "  • {} → hooksmith-{}",
                crate_analysis.crate_name, crate_analysis.crate_name
            ));
            strategy.push(format!(
                "    Commits: {} | Dependents: {}",
                crate_analysis.git_commits,
                crate_analysis.dependent_crates.len()
            ));
        }
    }

    strategy.push("Extraction Process:".to_string());
    strategy.push("  1. Create new repository (e.g., hooksmith-git-utils)".to_string());
    strategy.push("  2. Move crate files to new repo".to_string());
    strategy.push("  3. Set up CI/CD and versioning".to_string());
    strategy.push("  4. Update dependencies in main repo".to_string());
    strategy.push("  5. Publish to crates.io or private registry".to_string());

    strategy.push("Dependency Updates:".to_string());
    strategy.push("  [dependencies]".to_string());
    strategy.push(
        "  git-utils = { git = \"https://github.com/hooksmith/git-utils\", tag = \"v1.0.0\" }"
            .to_string(),
    );

    strategy
}

fn calculate_overall_stability_score(crates: &[CrateStabilityInfo]) -> f64 {
    if crates.is_empty() {
        return 0.0;
    }

    let total_score: f64 = crates
        .iter()
        .map(|c| {
            let velocity_score = 1.0 - c.change_velocity;
            let api_score = c.api_stability_score;
            let coupling_score = c.coupling_score;

            (velocity_score + api_score + coupling_score) / 3.0
        })
        .sum();

    total_score / crates.len() as f64
}

fn generate_crate_stability_report(analysis: &CrateStabilityAnalysis) {
    println!("\n🔧 Crate Stability Analysis");
    println!("===========================");

    // Show crate analysis
    println!("\n📊 Crate Stability Overview:");
    for crate_analysis in &analysis.crates {
        let readiness_icon = match crate_analysis.extraction_readiness {
            ExtractionReadiness::Ready => "🟢",
            ExtractionReadiness::NeedsStabilization => "🟡",
            ExtractionReadiness::TooVolatile => "🔴",
            ExtractionReadiness::NotReady => "⚪",
        };

        println!(
            "  {} {} ({} commits, {:.1} velocity, {:.1} API stability)",
            readiness_icon,
            crate_analysis.crate_name,
            crate_analysis.git_commits,
            crate_analysis.change_velocity,
            crate_analysis.api_stability_score
        );

        if !crate_analysis.dependent_crates.is_empty() {
            println!(
                "    Dependents: {}",
                crate_analysis.dependent_crates.join(", ")
            );
        }

        if !crate_analysis.stability_issues.is_empty() {
            println!("    Issues: {}", crate_analysis.stability_issues.join(", "));
        }

        if !crate_analysis.extraction_recommendations.is_empty() {
            println!(
                "    Recommendations: {}",
                crate_analysis.extraction_recommendations.join(", ")
            );
        }
        println!();
    }

    // Show statistics
    println!("\n📈 Stability Metrics:");
    println!(
        "  • Overall stability score: {:.1}%",
        analysis.overall_stability_score * 100.0
    );
    println!("  • Total crates: {}", analysis.crates.len());

    let ready_count = analysis
        .crates
        .iter()
        .filter(|c| matches!(c.extraction_readiness, ExtractionReadiness::Ready))
        .count();
    let needs_stabilization_count = analysis
        .crates
        .iter()
        .filter(|c| {
            matches!(
                c.extraction_readiness,
                ExtractionReadiness::NeedsStabilization
            )
        })
        .count();
    let too_volatile_count = analysis
        .crates
        .iter()
        .filter(|c| matches!(c.extraction_readiness, ExtractionReadiness::TooVolatile))
        .count();

    println!("  • Ready for extraction: {}", ready_count);
    println!("  • Needs stabilization: {}", needs_stabilization_count);
    println!("  • Too volatile: {}", too_volatile_count);

    // Show extraction candidates
    if !analysis.extraction_candidates.is_empty() {
        println!("\n🎯 Extraction Candidates:");
        for candidate in &analysis.extraction_candidates {
            println!("  • {}", candidate);
        }
    }

    // Show stabilization plan
    if !analysis.stabilization_plan.is_empty() {
        println!("\n🔧 Stabilization Plan:");
        for step in &analysis.stabilization_plan {
            println!("  {}", step);
        }
    }

    // Show external repo strategy
    if !analysis.external_repo_strategy.is_empty() {
        println!("\n📦 External Repository Strategy:");
        for step in &analysis.external_repo_strategy {
            println!("  {}", step);
        }
    }

    // Summary
    println!("\n📋 Summary:");
    if analysis.overall_stability_score > 0.7 {
        println!("  • High crate stability");
        println!("  • Good extraction readiness");
        println!("  • Ready for external repositories");
    } else if analysis.overall_stability_score > 0.5 {
        println!("  • Moderate crate stability");
        println!("  • Some crates ready for extraction");
        println!("  • Focus on stabilization");
    } else {
        println!("  • Low crate stability");
        println!("  • Prioritize stabilization");
        println!("  • Build usage and API stability");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Crate Stability Analyzer");
    println!("===========================");
    println!("Analyzing crate stability for extraction readiness...");
    println!();

    // Analyze crate stability
    let analysis = analyze_crate_stability()?;

    // Generate comprehensive report
    generate_crate_stability_report(&analysis);

    println!("\n✅ Crate stability analysis complete!");
    println!("🔧 Stability assessed");
    println!("🎯 Extraction candidates identified");
    println!("📦 External repo strategy ready");
    println!("🔧 Stabilization plan prepared");

    Ok(())
}
