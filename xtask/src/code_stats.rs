//! Code Statistics and Quality Analysis
//!
//! This module provides comprehensive Rust code quality analysis including:
//! - Code statistics (LOC, functions, structs, etc.)
//! - Clippy warnings and lints
//! - Dead code detection
//! - Build timing information
//! - Code quality metrics

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

/// Code statistics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStats {
    /// Total lines of code
    pub total_lines: usize,
    /// Lines of Rust code
    pub rust_lines: usize,
    /// Number of functions
    pub functions: usize,
    /// Number of structs
    pub structs: usize,
    /// Number of enums
    pub enums: usize,
    /// Number of modules
    pub modules: usize,
    /// Number of traits
    pub traits: usize,
    /// Number of implementations
    pub impls: usize,
    /// Number of constants
    pub constants: usize,
    /// Number of type aliases
    pub type_aliases: usize,
    /// Number of macros
    pub macros: usize,
}

/// Clippy analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippyAnalysis {
    /// Total number of warnings
    pub warnings: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of dead code warnings
    pub dead_code: usize,
    /// Number of unused import warnings
    pub unused_imports: usize,
    /// Number of style warnings
    pub style_warnings: usize,
    /// Number of complexity warnings
    pub complexity_warnings: usize,
    /// Number of performance warnings
    pub performance_warnings: usize,
    /// Number of correctness warnings
    pub correctness_warnings: usize,
    /// Detailed warning messages
    pub warning_details: Vec<String>,
}

/// Build timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTiming {
    /// Total build time in seconds
    pub total_time: f64,
    /// Compilation time in seconds
    pub compilation_time: f64,
    /// Linking time in seconds
    pub linking_time: f64,
    /// Number of crates compiled
    pub crates_compiled: usize,
    /// Time per crate in seconds
    pub time_per_crate: f64,
}

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Cyclomatic complexity average
    pub avg_complexity: f64,
    /// Function length average
    pub avg_function_length: f64,
    /// Module count
    pub module_count: usize,
    /// Test coverage percentage (if available)
    pub test_coverage: Option<f64>,
    /// Documentation coverage percentage
    pub doc_coverage: f64,
}

/// Comprehensive code analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisReport {
    /// Code statistics
    pub stats: CodeStats,
    /// Clippy analysis
    pub clippy: ClippyAnalysis,
    /// Build timing
    pub build_timing: BuildTiming,
    /// Quality metrics
    pub quality: QualityMetrics,
    /// Overall quality score (0-100)
    pub quality_score: f64,
    /// Quality grade (A-F)
    pub quality_grade: String,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Code stats command configuration
#[derive(Debug, Clone)]
pub struct CodeStatsConfig {
    /// Whether to show detailed breakdown
    pub detailed: bool,
    /// Whether to run clippy analysis
    pub clippy: bool,
    /// Whether to show build timing
    pub timing: bool,
    /// Whether to fail on warnings
    pub strict: bool,
    /// Output format
    pub format: OutputFormat,
}

/// Output format options
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Human-readable table
    Table,
    /// JSON output
    Json,
    /// Markdown output
    Markdown,
}

/// Code stats CLI commands
#[derive(Debug, Clone)]
pub struct CodeStatsCli {
    #[command(subcommand)]
    command: CodeStatsCommands,
}

/// Code stats subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum CodeStatsCommands {
    /// Generate comprehensive code analysis report
    Report {
        /// Show detailed breakdown
        #[arg(long)]
        detailed: bool,
        /// Run clippy analysis
        #[arg(long, default_value = "true")]
        clippy: bool,
        /// Show build timing
        #[arg(long, default_value = "true")]
        timing: bool,
        /// Fail on warnings
        #[arg(long)]
        strict: bool,
        /// Output format (table, json, markdown)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Run clippy analysis only
    Clippy {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
        /// Output format (table, json, markdown)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Get code statistics only
    Stats {
        /// Show detailed breakdown
        #[arg(long)]
        detailed: bool,
        /// Output format (table, json, markdown)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Get build timing information
    Timing {
        /// Output format (table, json, markdown)
        #[arg(long, default_value = "table")]
        format: String,
    },
}

impl Default for CodeStatsConfig {
    fn default() -> Self {
        Self {
            detailed: false,
            clippy: true,
            timing: true,
            strict: false,
            format: OutputFormat::Table,
        }
    }
}

/// Run code stats command
pub async fn run_code_stats_command(command: CodeStatsCommands) -> Result<()> {
    match command {
        CodeStatsCommands::Report {
            detailed,
            clippy,
            timing,
            strict,
            format,
        } => {
            let config = CodeStatsConfig {
                detailed,
                clippy,
                timing,
                strict,
                format: parse_output_format(&format)?,
            };
            generate_code_analysis_report(config).await
        }
        CodeStatsCommands::Clippy { strict, format } => {
            let clippy_result = run_clippy_analysis(strict).await?;
            match parse_output_format(&format)? {
                OutputFormat::Table => print_clippy_table(&clippy_result)?,
                OutputFormat::Json => print_clippy_json(&clippy_result)?,
                OutputFormat::Markdown => print_clippy_markdown(&clippy_result)?,
            }
            Ok(())
        }
        CodeStatsCommands::Stats { detailed, format } => {
            let stats = analyze_code_stats().await?;
            match parse_output_format(&format)? {
                OutputFormat::Table => print_stats_table(&stats, detailed)?,
                OutputFormat::Json => print_stats_json(&stats)?,
                OutputFormat::Markdown => print_stats_markdown(&stats, detailed)?,
            }
            Ok(())
        }
        CodeStatsCommands::Timing { format } => {
            let timing = analyze_build_timing().await?;
            match parse_output_format(&format)? {
                OutputFormat::Table => print_timing_table(&timing)?,
                OutputFormat::Json => print_timing_json(&timing)?,
                OutputFormat::Markdown => print_timing_markdown(&timing)?,
            }
            Ok(())
        }
    }
}

/// Generate comprehensive code analysis report
async fn generate_code_analysis_report(config: CodeStatsConfig) -> Result<()> {
    println!("🔍 Hooksmith Code Analysis Report");
    println!("==================================");

    let mut report = CodeAnalysisReport {
        stats: analyze_code_stats().await?,
        clippy: if config.clippy {
            run_clippy_analysis(config.strict).await?
        } else {
            ClippyAnalysis {
                warnings: 0,
                errors: 0,
                dead_code: 0,
                unused_imports: 0,
                style_warnings: 0,
                complexity_warnings: 0,
                performance_warnings: 0,
                correctness_warnings: 0,
                warning_details: Vec::new(),
            }
        },
        build_timing: if config.timing {
            analyze_build_timing().await?
        } else {
            BuildTiming {
                total_time: 0.0,
                compilation_time: 0.0,
                linking_time: 0.0,
                crates_compiled: 0,
                time_per_crate: 0.0,
            }
        },
        quality: analyze_quality_metrics().await?,
        quality_score: 0.0,
        quality_grade: "F".to_string(),
        recommendations: Vec::new(),
    };

    // Calculate quality score
    report.quality_score = calculate_quality_score(&report);
    report.quality_grade = calculate_quality_grade(report.quality_score);
    report.recommendations = generate_recommendations(&report);

    match config.format {
        OutputFormat::Table => print_analysis_table(&report, &config)?,
        OutputFormat::Json => print_analysis_json(&report)?,
        OutputFormat::Markdown => print_analysis_markdown(&report, &config)?,
    }

    if config.strict && (report.clippy.warnings > 0 || report.clippy.errors > 0) {
        anyhow::bail!(
            "Code quality check failed: {} warnings, {} errors",
            report.clippy.warnings,
            report.clippy.errors
        );
    }

    Ok(())
}

/// Analyze code statistics
async fn analyze_code_stats() -> Result<CodeStats> {
    // Count lines of Rust code
    let rust_lines = count_rust_lines().await?;
    
    // Count code elements using ripgrep
    let functions = count_pattern("^ *(pub )?fn ").await?;
    let structs = count_pattern("^ *(pub )?struct ").await?;
    let enums = count_pattern("^ *(pub )?enum ").await?;
    let modules = count_pattern("^ *(pub )?mod ").await?;
    let traits = count_pattern("^ *(pub )?trait ").await?;
    let impls = count_pattern("^ *impl ").await?;
    let constants = count_pattern("^ *(pub )?const ").await?;
    let type_aliases = count_pattern("^ *(pub )?type ").await?;
    let macros = count_pattern("^ *(pub )?macro_rules! ").await?;

    Ok(CodeStats {
        total_lines: rust_lines,
        rust_lines,
        functions,
        structs,
        enums,
        modules,
        traits,
        impls,
        constants,
        type_aliases,
        macros,
    })
}

/// Count lines of Rust code
async fn count_rust_lines() -> Result<usize> {
    let output = Command::new("find")
        .args(&["src", "xtask/src", "components", "-name", "*.rs"])
        .output()
        .context("Failed to find Rust files")?;

    let files = String::from_utf8_lossy(&output.stdout);
    let mut total_lines = 0;

    for file in files.lines() {
        if !file.trim().is_empty() {
            let output = Command::new("wc")
                .args(&["-l", file])
                .output()
                .context(format!("Failed to count lines in {}", file))?;
            
            let line_count: usize = String::from_utf8_lossy(&output.stdout)
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            
            total_lines += line_count;
        }
    }

    Ok(total_lines)
}

/// Count pattern occurrences using ripgrep
async fn count_pattern(pattern: &str) -> Result<usize> {
    let output = Command::new("rg")
        .args(&["--count", pattern, "src", "xtask/src", "components"])
        .output();

    match output {
        Ok(output) => {
            let count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| line.split(':').last().unwrap_or("0").parse::<usize>().unwrap_or(0))
                .sum();
            Ok(count)
        }
        Err(_) => {
            // Fallback to grep if ripgrep is not available
            let output = Command::new("grep")
                .args(&["-r", "--count", pattern, "src", "xtask/src", "components"])
                .output()
                .context("Failed to count pattern occurrences")?;
            
            let count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| line.split(':').last().unwrap_or("0").parse::<usize>().unwrap_or(0))
                .sum();
            Ok(count)
        }
    }
}

/// Run clippy analysis
async fn run_clippy_analysis(strict: bool) -> Result<ClippyAnalysis> {
    let mut args = vec!["clippy", "--all-targets", "--all-features"];
    
    if strict {
        args.extend_from_slice(&["--", "-D", "warnings"]);
    }

    let output = Command::new("cargo")
        .args(&args)
        .output();

    let mut analysis = ClippyAnalysis {
        warnings: 0,
        errors: 0,
        dead_code: 0,
        unused_imports: 0,
        style_warnings: 0,
        complexity_warnings: 0,
        performance_warnings: 0,
        correctness_warnings: 0,
        warning_details: Vec::new(),
    };

    match output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Parse clippy output
            for line in stderr.lines() {
                if line.contains("warning:") {
                    analysis.warnings += 1;
                    analysis.warning_details.push(line.to_string());
                    
                    if line.contains("dead_code") {
                        analysis.dead_code += 1;
                    } else if line.contains("unused_imports") || line.contains("unused_import") {
                        analysis.unused_imports += 1;
                    } else if line.contains("clippy::style") {
                        analysis.style_warnings += 1;
                    } else if line.contains("clippy::complexity") {
                        analysis.complexity_warnings += 1;
                    } else if line.contains("clippy::perf") {
                        analysis.performance_warnings += 1;
                    } else if line.contains("clippy::correctness") {
                        analysis.correctness_warnings += 1;
                    }
                } else if line.contains("error:") {
                    analysis.errors += 1;
                    analysis.warning_details.push(line.to_string());
                }
            }
        }
        Err(_) => {
            // If clippy fails, return empty analysis
            analysis.warning_details.push("Clippy analysis failed - clippy may not be installed".to_string());
        }
    }

    Ok(analysis)
}

/// Analyze build timing
async fn analyze_build_timing() -> Result<BuildTiming> {
    let start = std::time::Instant::now();
    
    let output = Command::new("cargo")
        .args(&["build", "--timings"])
        .output()
        .context("Failed to run cargo build with timings")?;

    let build_time = start.elapsed().as_secs_f64();

    // Try to parse timing information from output
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut timing = BuildTiming {
        total_time: build_time,
        compilation_time: build_time * 0.8, // Estimate
        linking_time: build_time * 0.2,     // Estimate
        crates_compiled: 0,
        time_per_crate: 0.0,
    };

    // Count crates compiled
    timing.crates_compiled = output_str.lines()
        .filter(|line| line.contains("Compiling"))
        .count();

    if timing.crates_compiled > 0 {
        timing.time_per_crate = timing.total_time / timing.crates_compiled as f64;
    }

    Ok(timing)
}

/// Analyze quality metrics
async fn analyze_quality_metrics() -> Result<QualityMetrics> {
    // Calculate average function length
    let function_lines = count_function_lines().await?;
    let function_count = count_pattern("^ *(pub )?fn ").await?;
    let avg_function_length = if function_count > 0 {
        function_lines as f64 / function_count as f64
    } else {
        0.0
    };

    // Calculate module count
    let module_count = count_pattern("^ *(pub )?mod ").await?;

    // Estimate documentation coverage
    let doc_comments = count_pattern("///").await?;
    let total_items = count_pattern("^ *(pub )?(fn|struct|enum|trait|mod) ").await?;
    let doc_coverage = if total_items > 0 {
        (doc_comments as f64 / total_items as f64) * 100.0
    } else {
        0.0
    };

    Ok(QualityMetrics {
        avg_complexity: 5.0, // Placeholder - would need more sophisticated analysis
        avg_function_length,
        module_count,
        test_coverage: None, // Would need to run tests with coverage
        doc_coverage,
    })
}

/// Count function lines
async fn count_function_lines() -> Result<usize> {
    let output = Command::new("rg")
        .args(&["-A", "1", "^ *(pub )?fn ", "src", "xtask/src", "components"])
        .output();

    match output {
        Ok(output) => {
            let lines = String::from_utf8_lossy(&output.stdout).lines().count();
            Ok(lines)
        }
        Err(_) => {
            // Fallback
            Ok(0)
        }
    }
}

/// Calculate quality score (0-100)
fn calculate_quality_score(report: &CodeAnalysisReport) -> f64 {
    let mut score = 100.0;

    // Deduct points for warnings and errors
    score -= report.clippy.warnings as f64 * 2.0;
    score -= report.clippy.errors as f64 * 10.0;

    // Deduct points for dead code
    score -= report.clippy.dead_code as f64 * 3.0;

    // Deduct points for unused imports
    score -= report.clippy.unused_imports as f64 * 1.0;

    // Deduct points for poor documentation coverage
    if report.quality.doc_coverage < 50.0 {
        score -= (50.0 - report.quality.doc_coverage) * 0.5;
    }

    // Deduct points for very long functions
    if report.quality.avg_function_length > 50.0 {
        score -= (report.quality.avg_function_length - 50.0) * 0.2;
    }

    score.max(0.0).min(100.0)
}

/// Calculate quality grade (A-F)
fn calculate_quality_grade(score: f64) -> String {
    match score {
        90.0..=100.0 => "A".to_string(),
        80.0..<90.0 => "B".to_string(),
        70.0..<80.0 => "C".to_string(),
        60.0..<70.0 => "D".to_string(),
        50.0..<60.0 => "E".to_string(),
        _ => "F".to_string(),
    }
}

/// Generate recommendations
fn generate_recommendations(report: &CodeAnalysisReport) -> Vec<String> {
    let mut recommendations = Vec::new();

    if report.clippy.warnings > 0 {
        recommendations.push(format!("Fix {} clippy warnings", report.clippy.warnings));
    }

    if report.clippy.dead_code > 0 {
        recommendations.push(format!("Remove {} dead code items", report.clippy.dead_code));
    }

    if report.clippy.unused_imports > 0 {
        recommendations.push(format!("Remove {} unused imports", report.clippy.unused_imports));
    }

    if report.quality.doc_coverage < 80.0 {
        recommendations.push(format!("Improve documentation coverage (currently {:.1}%)", report.quality.doc_coverage));
    }

    if report.quality.avg_function_length > 30.0 {
        recommendations.push(format!("Reduce average function length (currently {:.1} lines)", report.quality.avg_function_length));
    }

    if recommendations.is_empty() {
        recommendations.push("Code quality is excellent! Keep up the good work.".to_string());
    }

    recommendations
}

/// Print analysis table
fn print_analysis_table(report: &CodeAnalysisReport, config: &CodeStatsConfig) -> Result<()> {
    println!("\n📊 Code Statistics");
    println!("==================");
    println!("Total lines of Rust code: {}", report.stats.rust_lines);
    println!("Functions: {}", report.stats.functions);
    println!("Structs: {}", report.stats.structs);
    println!("Enums: {}", report.stats.enums);
    println!("Modules: {}", report.stats.modules);
    println!("Traits: {}", report.stats.traits);
    println!("Implementations: {}", report.stats.impls);

    if config.clippy {
        println!("\n🔍 Clippy Analysis");
        println!("==================");
        println!("Warnings: {}", report.clippy.warnings);
        println!("Errors: {}", report.clippy.errors);
        println!("Dead code: {}", report.clippy.dead_code);
        println!("Unused imports: {}", report.clippy.unused_imports);
        println!("Style warnings: {}", report.clippy.style_warnings);
        println!("Complexity warnings: {}", report.clippy.complexity_warnings);
        println!("Performance warnings: {}", report.clippy.performance_warnings);
        println!("Correctness warnings: {}", report.clippy.correctness_warnings);
    }

    if config.timing {
        println!("\n⏱️  Build Timing");
        println!("================");
        println!("Total build time: {:.2}s", report.build_timing.total_time);
        println!("Crates compiled: {}", report.build_timing.crates_compiled);
        println!("Time per crate: {:.2}s", report.build_timing.time_per_crate);
    }

    println!("\n📈 Quality Metrics");
    println!("==================");
    println!("Documentation coverage: {:.1}%", report.quality.doc_coverage);
    println!("Average function length: {:.1} lines", report.quality.avg_function_length);
    println!("Module count: {}", report.quality.module_count);

    println!("\n🎯 Overall Quality");
    println!("==================");
    println!("Quality score: {:.1}/100", report.quality_score);
    println!("Quality grade: {}", report.quality_grade);

    if !report.recommendations.is_empty() {
        println!("\n💡 Recommendations");
        println!("==================");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }
    }

    Ok(())
}

/// Print analysis JSON
fn print_analysis_json(report: &CodeAnalysisReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Print analysis markdown
fn print_analysis_markdown(report: &CodeAnalysisReport, config: &CodeStatsConfig) -> Result<()> {
    println!("# Hooksmith Code Analysis Report\n");

    println!("## Code Statistics\n");
    println!("| Metric | Count |");
    println!("|--------|-------|");
    println!("| Total lines of Rust code | {} |", report.stats.rust_lines);
    println!("| Functions | {} |", report.stats.functions);
    println!("| Structs | {} |", report.stats.structs);
    println!("| Enums | {} |", report.stats.enums);
    println!("| Modules | {} |", report.stats.modules);
    println!("| Traits | {} |", report.stats.traits);
    println!("| Implementations | {} |", report.stats.impls);

    if config.clippy {
        println!("\n## Clippy Analysis\n");
        println!("| Metric | Count |");
        println!("|--------|-------|");
        println!("| Warnings | {} |", report.clippy.warnings);
        println!("| Errors | {} |", report.clippy.errors);
        println!("| Dead code | {} |", report.clippy.dead_code);
        println!("| Unused imports | {} |", report.clippy.unused_imports);
        println!("| Style warnings | {} |", report.clippy.style_warnings);
        println!("| Complexity warnings | {} |", report.clippy.complexity_warnings);
        println!("| Performance warnings | {} |", report.clippy.performance_warnings);
        println!("| Correctness warnings | {} |", report.clippy.correctness_warnings);
    }

    if config.timing {
        println!("\n## Build Timing\n");
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Total build time | {:.2}s |", report.build_timing.total_time);
        println!("| Crates compiled | {} |", report.build_timing.crates_compiled);
        println!("| Time per crate | {:.2}s |", report.build_timing.time_per_crate);
    }

    println!("\n## Quality Metrics\n");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Documentation coverage | {:.1}% |", report.quality.doc_coverage);
    println!("| Average function length | {:.1} lines |", report.quality.avg_function_length);
    println!("| Module count | {} |", report.quality.module_count);

    println!("\n## Overall Quality\n");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Quality score | {:.1}/100 |", report.quality_score);
    println!("| Quality grade | {} |", report.quality_grade);

    if !report.recommendations.is_empty() {
        println!("\n## Recommendations\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }
    }

    Ok(())
}

/// Print clippy table
fn print_clippy_table(analysis: &ClippyAnalysis) -> Result<()> {
    println!("🔍 Clippy Analysis Results");
    println!("==========================");
    println!("Warnings: {}", analysis.warnings);
    println!("Errors: {}", analysis.errors);
    println!("Dead code: {}", analysis.dead_code);
    println!("Unused imports: {}", analysis.unused_imports);
    println!("Style warnings: {}", analysis.style_warnings);
    println!("Complexity warnings: {}", analysis.complexity_warnings);
    println!("Performance warnings: {}", analysis.performance_warnings);
    println!("Correctness warnings: {}", analysis.correctness_warnings);

    if !analysis.warning_details.is_empty() {
        println!("\n📋 Warning Details:");
        for detail in &analysis.warning_details {
            println!("  {}", detail);
        }
    }

    Ok(())
}

/// Print clippy JSON
fn print_clippy_json(analysis: &ClippyAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(analysis)?;
    println!("{}", json);
    Ok(())
}

/// Print clippy markdown
fn print_clippy_markdown(analysis: &ClippyAnalysis) -> Result<()> {
    println!("# Clippy Analysis Results\n");
    println!("| Metric | Count |");
    println!("|--------|-------|");
    println!("| Warnings | {} |", analysis.warnings);
    println!("| Errors | {} |", analysis.errors);
    println!("| Dead code | {} |", analysis.dead_code);
    println!("| Unused imports | {} |", analysis.unused_imports);
    println!("| Style warnings | {} |", analysis.style_warnings);
    println!("| Complexity warnings | {} |", analysis.complexity_warnings);
    println!("| Performance warnings | {} |", analysis.performance_warnings);
    println!("| Correctness warnings | {} |", analysis.correctness_warnings);

    if !analysis.warning_details.is_empty() {
        println!("\n## Warning Details\n");
        for detail in &analysis.warning_details {
            println!("- {}", detail);
        }
    }

    Ok(())
}

/// Print stats table
fn print_stats_table(stats: &CodeStats, detailed: bool) -> Result<()> {
    println!("📊 Code Statistics");
    println!("==================");
    println!("Total lines of Rust code: {}", stats.rust_lines);
    println!("Functions: {}", stats.functions);
    println!("Structs: {}", stats.structs);
    println!("Enums: {}", stats.enums);
    println!("Modules: {}", stats.modules);
    println!("Traits: {}", stats.traits);
    println!("Implementations: {}", stats.impls);

    if detailed {
        println!("Constants: {}", stats.constants);
        println!("Type aliases: {}", stats.type_aliases);
        println!("Macros: {}", stats.macros);
    }

    Ok(())
}

/// Print stats JSON
fn print_stats_json(stats: &CodeStats) -> Result<()> {
    let json = serde_json::to_string_pretty(stats)?;
    println!("{}", json);
    Ok(())
}

/// Print stats markdown
fn print_stats_markdown(stats: &CodeStats, detailed: bool) -> Result<()> {
    println!("# Code Statistics\n");
    println!("| Metric | Count |");
    println!("|--------|-------|");
    println!("| Total lines of Rust code | {} |", stats.rust_lines);
    println!("| Functions | {} |", stats.functions);
    println!("| Structs | {} |", stats.structs);
    println!("| Enums | {} |", stats.enums);
    println!("| Modules | {} |", stats.modules);
    println!("| Traits | {} |", stats.traits);
    println!("| Implementations | {} |", stats.impls);

    if detailed {
        println!("| Constants | {} |", stats.constants);
        println!("| Type aliases | {} |", stats.type_aliases);
        println!("| Macros | {} |", stats.macros);
    }

    Ok(())
}

/// Print timing table
fn print_timing_table(timing: &BuildTiming) -> Result<()> {
    println!("⏱️  Build Timing");
    println!("================");
    println!("Total build time: {:.2}s", timing.total_time);
    println!("Crates compiled: {}", timing.crates_compiled);
    println!("Time per crate: {:.2}s", timing.time_per_crate);
    Ok(())
}

/// Print timing JSON
fn print_timing_json(timing: &BuildTiming) -> Result<()> {
    let json = serde_json::to_string_pretty(timing)?;
    println!("{}", json);
    Ok(())
}

/// Print timing markdown
fn print_timing_markdown(timing: &BuildTiming) -> Result<()> {
    println!("# Build Timing\n");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Total build time | {:.2}s |", timing.total_time);
    println!("| Crates compiled | {} |", timing.crates_compiled);
    println!("| Time per crate | {:.2}s |", timing.time_per_crate);
    Ok(())
}

/// Parse output format from string
fn parse_output_format(format: &str) -> Result<OutputFormat> {
    match format.to_lowercase().as_str() {
        "table" => Ok(OutputFormat::Table),
        "json" => Ok(OutputFormat::Json),
        "markdown" => Ok(OutputFormat::Markdown),
        _ => anyhow::bail!("Unknown output format: {}", format),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output_format() {
        assert!(matches!(
            parse_output_format("table").unwrap(),
            OutputFormat::Table
        ));
        assert!(matches!(
            parse_output_format("json").unwrap(),
            OutputFormat::Json
        ));
        assert!(matches!(
            parse_output_format("markdown").unwrap(),
            OutputFormat::Markdown
        ));
        assert!(parse_output_format("unknown").is_err());
    }

    #[test]
    fn test_calculate_quality_grade() {
        assert_eq!(calculate_quality_grade(95.0), "A");
        assert_eq!(calculate_quality_grade(85.0), "B");
        assert_eq!(calculate_quality_grade(75.0), "C");
        assert_eq!(calculate_quality_grade(65.0), "D");
        assert_eq!(calculate_quality_grade(55.0), "E");
        assert_eq!(calculate_quality_grade(45.0), "F");
    }

    #[test]
    fn test_calculate_quality_score() {
        let mut report = CodeAnalysisReport {
            stats: CodeStats {
                total_lines: 1000,
                rust_lines: 1000,
                functions: 50,
                structs: 20,
                enums: 10,
                modules: 5,
                traits: 5,
                impls: 30,
                constants: 10,
                type_aliases: 5,
                macros: 2,
            },
            clippy: ClippyAnalysis {
                warnings: 0,
                errors: 0,
                dead_code: 0,
                unused_imports: 0,
                style_warnings: 0,
                complexity_warnings: 0,
                performance_warnings: 0,
                correctness_warnings: 0,
                warning_details: Vec::new(),
            },
            build_timing: BuildTiming {
                total_time: 10.0,
                compilation_time: 8.0,
                linking_time: 2.0,
                crates_compiled: 5,
                time_per_crate: 2.0,
            },
            quality: QualityMetrics {
                avg_complexity: 5.0,
                avg_function_length: 20.0,
                module_count: 5,
                test_coverage: None,
                doc_coverage: 90.0,
            },
            quality_score: 0.0,
            quality_grade: "A".to_string(),
            recommendations: Vec::new(),
        };

        let score = calculate_quality_score(&report);
        assert!(score > 90.0); // Should be high for clean code
    }
} 
