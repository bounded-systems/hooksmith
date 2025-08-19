use hyperpolyglot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct LanguageInfo {
    #[serde(rename = "type")]
    kind: String,
    extensions: Option<Vec<String>>,
    filenames: Option<Vec<String>>,
    interpreters: Option<Vec<String>>,
    color: Option<String>,
    aliases: Option<Vec<String>>,
    group: Option<String>,
    tm_scope: Option<String>,
    ace_mode: Option<String>,
    codemirror_mode: Option<String>,
    language_id: Option<u64>,
}

#[derive(Debug)]
struct LanguageComparison {
    linguist_name: String,
    linguist_info: LanguageInfo,
    hyperpolyglot_detection: Option<String>,
    extensions_match: bool,
    type_matches: bool,
    suggested_gitattributes: Vec<String>,
}

#[derive(Debug, Clone)]
struct FileAnalysis {
    path: PathBuf,
    linguist_language: Option<String>,
    hyperpolyglot_detection: Option<String>,
    suggested_override: Option<String>,
    confidence: f64,
}

fn load_languages_yml() -> Result<HashMap<String, LanguageInfo>, Box<dyn std::error::Error>> {
    // Try to load from local file first
    let local_path = "languages.yml";
    if Path::new(local_path).exists() {
        let content = fs::read_to_string(local_path)?;
        let languages: HashMap<String, LanguageInfo> = serde_yaml::from_str(&content)?;
        return Ok(languages);
    }

    // If not found locally, try to download from GitHub
    println!("📥 Downloading languages.yml from GitHub Linguist...");
    let response = reqwest::blocking::get(
        "https://raw.githubusercontent.com/github-linguist/linguist/main/lib/linguist/languages.yml"
    )?;

    let content = response.text()?;
    let languages: HashMap<String, LanguageInfo> = serde_yaml::from_str(&content)?;

    // Cache the file locally
    fs::write(local_path, &content)?;
    println!("✅ Cached languages.yml locally");

    Ok(languages)
}

fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
}

fn get_filename(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

fn find_language_by_extension(
    extension: &str,
    languages: &HashMap<String, LanguageInfo>,
) -> Vec<String> {
    let mut matches = Vec::new();

    for (name, info) in languages {
        if let Some(extensions) = &info.extensions {
            if extensions.contains(&extension.to_string()) {
                matches.push(name.clone());
            }
        }
    }

    matches
}

fn find_language_by_filename(
    filename: &str,
    languages: &HashMap<String, LanguageInfo>,
) -> Vec<String> {
    let mut matches = Vec::new();

    for (name, info) in languages {
        if let Some(filenames) = &info.filenames {
            if filenames.contains(&filename.to_string()) {
                matches.push(name.clone());
            }
        }
    }

    matches
}

fn analyze_file(file_path: &Path, languages: &HashMap<String, LanguageInfo>) -> FileAnalysis {
    let extension = get_file_extension(file_path);
    let filename = get_filename(file_path);

    // Try hyperpolyglot detection
    let hyperpolyglot_detection = hyperpolyglot::detect(file_path)
        .ok()
        .flatten()
        .map(|d| format!("{:?}", d));

    // Find linguist matches
    let mut linguist_matches = Vec::new();

    if let Some(ext) = &extension {
        linguist_matches.extend(find_language_by_extension(ext, languages));
    }

    if let Some(name) = &filename {
        linguist_matches.extend(find_language_by_filename(name, languages));
    }

    let linguist_language = linguist_matches.first().cloned();

    // Calculate confidence based on matches
    let confidence = if linguist_language.is_some() && hyperpolyglot_detection.is_some() {
        if linguist_language.as_ref().unwrap() == hyperpolyglot_detection.as_ref().unwrap() {
            1.0
        } else {
            0.5
        }
    } else if linguist_language.is_some() || hyperpolyglot_detection.is_some() {
        0.7
    } else {
        0.0
    };

    // Suggest gitattributes override if needed
    let suggested_override = if confidence < 0.8 {
        if let Some(lang) = &linguist_language {
            Some(format!("linguist-language={}", lang))
        } else if let Some(detection) = &hyperpolyglot_detection {
            Some(format!("linguist-language={}", detection))
        } else {
            None
        }
    } else {
        None
    };

    FileAnalysis {
        path: file_path.to_path_buf(),
        linguist_language,
        hyperpolyglot_detection,
        suggested_override,
        confidence,
    }
}

fn compare_languages(languages: &HashMap<String, LanguageInfo>) -> Vec<LanguageComparison> {
    let mut comparisons = Vec::new();

    for (name, info) in languages {
        // For demonstration, we'll analyze a few key languages
        if ["Rust", "JavaScript", "Python", "Markdown", "JSON", "YAML"].contains(&name.as_str()) {
            let comparison = LanguageComparison {
                linguist_name: name.clone(),
                linguist_info: info.clone(),
                hyperpolyglot_detection: None, // Would need to test with actual files
                extensions_match: true,        // Placeholder
                type_matches: true,            // Placeholder
                suggested_gitattributes: Vec::new(),
            };
            comparisons.push(comparison);
        }
    }

    comparisons
}

fn print_language_comparison(comparisons: &[LanguageComparison]) {
    println!("🔍 Language Comparison Analysis");
    println!("==============================");
    println!();

    for comparison in comparisons {
        println!("📋 {}", comparison.linguist_name);
        println!("   Type: {}", comparison.linguist_info.kind);
        println!("   Extensions: {:?}", comparison.linguist_info.extensions);
        println!("   Filenames: {:?}", comparison.linguist_info.filenames);
        println!("   Color: {:?}", comparison.linguist_info.color);
        println!("   Group: {:?}", comparison.linguist_info.group);
        println!();
    }
}

fn print_file_analysis(analyses: &[FileAnalysis]) {
    println!("📊 File Analysis Results");
    println!("=======================");
    println!();

    let mut by_confidence = analyses.to_vec();
    by_confidence.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let mut high_confidence = Vec::new();
    let mut medium_confidence = Vec::new();
    let mut low_confidence = Vec::new();

    for analysis in &by_confidence {
        match analysis.confidence {
            c if c >= 0.8 => high_confidence.push(analysis),
            c if c >= 0.5 => medium_confidence.push(analysis),
            _ => low_confidence.push(analysis),
        }
    }

    println!("✅ High Confidence ({} files):", high_confidence.len());
    for analysis in high_confidence.iter().take(5) {
        println!(
            "   {} (confidence: {:.1})",
            analysis.path.display(),
            analysis.confidence
        );
    }
    if high_confidence.len() > 5 {
        println!("   ... and {} more", high_confidence.len() - 5);
    }
    println!();

    println!("⚠️  Medium Confidence ({} files):", medium_confidence.len());
    for analysis in medium_confidence.iter().take(5) {
        println!(
            "   {} (confidence: {:.1})",
            analysis.path.display(),
            analysis.confidence
        );
        if let Some(override_suggestion) = &analysis.suggested_override {
            println!("     Suggested: {}", override_suggestion);
        }
    }
    if medium_confidence.len() > 5 {
        println!("   ... and {} more", medium_confidence.len() - 5);
    }
    println!();

    println!("❌ Low Confidence ({} files):", low_confidence.len());
    for analysis in low_confidence.iter().take(5) {
        println!(
            "   {} (confidence: {:.1})",
            analysis.path.display(),
            analysis.confidence
        );
        if let Some(override_suggestion) = &analysis.suggested_override {
            println!("     Suggested: {}", override_suggestion);
        }
    }
    if low_confidence.len() > 5 {
        println!("   ... and {} more", low_confidence.len() - 5);
    }
    println!();
}

fn generate_gitattributes_suggestions(analyses: &[FileAnalysis]) {
    println!("💡 Suggested .gitattributes Overrides");
    println!("====================================");
    println!();

    let mut suggestions = HashMap::new();

    for analysis in analyses {
        if let Some(override_suggestion) = &analysis.suggested_override {
            let extension = get_file_extension(&analysis.path);
            if let Some(ext) = extension {
                suggestions.insert(ext, override_suggestion.clone());
            }
        }
    }

    if suggestions.is_empty() {
        println!("✅ No suggestions needed - all files are properly classified!");
        return;
    }

    println!("# Suggested .gitattributes overrides");
    println!("# Generated by language-reconciliation tool");
    println!();

    for (extension, override_suggestion) in suggestions {
        println!("*{} {}", extension, override_suggestion);
    }
    println!();

    println!("💡 To apply these suggestions:");
    println!("   1. Add the above lines to your .gitattributes file");
    println!("   2. Commit the changes");
    println!("   3. Run this tool again to verify improvements");
}

fn get_git_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-files", "--cached", "--full-name", "--exclude-standard"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get git files".into());
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(s))
        .collect();

    Ok(files)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("language-reconciliation - Compare Linguist and hyperpolyglot language detection");
        println!();
        println!("Usage:");
        println!(
            "  {} languages              - Show language comparison",
            args[0]
        );
        println!(
            "  {} analyze               - Analyze repository files",
            args[0]
        );
        println!(
            "  {} suggestions           - Generate .gitattributes suggestions",
            args[0]
        );
        println!(
            "  {} full                  - Run complete analysis",
            args[0]
        );
        println!();
        println!("This tool loads GitHub Linguist's languages.yml and compares");
        println!("it with hyperpolyglot's detection to ensure perfect alignment.");
        return ExitCode::SUCCESS;
    }

    // Check if we're in a git repository
    if Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .is_err()
    {
        eprintln!("❌ Error: Not in a git repository");
        return ExitCode::FAILURE;
    }

    // Load languages.yml
    let languages = match load_languages_yml() {
        Ok(languages) => {
            println!(
                "✅ Loaded {} languages from GitHub Linguist",
                languages.len()
            );
            languages
        }
        Err(e) => {
            eprintln!("❌ Error loading languages.yml: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match args[1].as_str() {
        "languages" => {
            let comparisons = compare_languages(&languages);
            print_language_comparison(&comparisons);
        }
        "analyze" | "suggestions" | "full" => {
            let files = match get_git_files() {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("❌ Error getting git files: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            println!("🔍 Analyzing {} files...", files.len());

            let analyses: Vec<FileAnalysis> = files
                .iter()
                .map(|file| analyze_file(file, &languages))
                .collect();

            if args[1] == "analyze" || args[1] == "full" {
                print_file_analysis(&analyses);
            }

            if args[1] == "suggestions" || args[1] == "full" {
                generate_gitattributes_suggestions(&analyses);
            }
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            eprintln!("Use 'languages', 'analyze', 'suggestions', or 'full'");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
