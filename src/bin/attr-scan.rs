use hyperpolyglot;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
struct GitAttributesEntry {
    pattern: String,
    attributes: HashMap<String, String>,
}

#[derive(Debug)]
struct FileClassification {
    path: PathBuf,
    linguist_language: Option<String>,
    is_vendored: bool,
    is_generated: bool,
    is_documentation: bool,
    is_detectable: bool,
    matched_pattern: Option<String>,
}

impl FileClassification {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            linguist_language: None,
            is_vendored: false,
            is_generated: false,
            is_documentation: false,
            is_detectable: false,
            matched_pattern: None,
        }
    }
}

fn parse_gitattributes(content: &str) -> Vec<GitAttributesEntry> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let pattern = parts[0].to_string();
        let mut attributes = HashMap::new();

        for attr in &parts[1..] {
            if attr.starts_with("linguist-") {
                if let Some((key, value)) = attr.split_once('=') {
                    attributes.insert(key.to_string(), value.to_string());
                } else {
                    // Boolean attributes like linguist-vendored
                    attributes.insert(attr.to_string(), "true".to_string());
                }
            }
        }

        if !attributes.is_empty() {
            entries.push(GitAttributesEntry {
                pattern,
                attributes,
            });
        }
    }

    entries
}

fn matches_pattern(pattern: &str, file_path: &Path) -> bool {
    // Simple glob-like pattern matching
    // This is a simplified version - in production you'd want a proper glob library

    let file_str = file_path.to_string_lossy();

    if pattern.contains('*') {
        // Convert glob pattern to regex-like matching
        let regex_pattern = pattern.replace(".", "\\.").replace("*", ".*");

        // Simple regex-like matching
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(&file_str);
        }
    } else {
        // Exact match
        return file_str == pattern;
    }

    false
}

fn classify_file(file_path: &Path, entries: &[GitAttributesEntry]) -> FileClassification {
    let mut classification = FileClassification::new(file_path.to_path_buf());

    for entry in entries {
        if matches_pattern(&entry.pattern, file_path) {
            classification.matched_pattern = Some(entry.pattern.clone());

            for (attr, value) in &entry.attributes {
                match attr.as_str() {
                    "linguist-language" => {
                        classification.linguist_language = Some(value.clone());
                    }
                    "linguist-vendored" => {
                        classification.is_vendored = value == "true";
                    }
                    "linguist-generated" => {
                        classification.is_generated = value == "true";
                    }
                    "linguist-documentation" => {
                        classification.is_documentation = value == "true";
                    }
                    "linguist-detectable" => {
                        classification.is_detectable = value == "true";
                    }
                    _ => {}
                }
            }
        }
    }

    // If no linguist-language is set in .gitattributes, try hyperpolyglot detection
    if classification.linguist_language.is_none() {
        if let Ok(Some(detection)) = hyperpolyglot::detect(file_path) {
            classification.linguist_language = Some(format!("{:?}", detection));
        }
    }

    classification
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

fn read_gitattributes() -> Result<String, Box<dyn std::error::Error>> {
    if Path::new(".gitattributes").exists() {
        Ok(fs::read_to_string(".gitattributes")?)
    } else {
        Ok(String::new())
    }
}

fn print_files_command(classifications: &[FileClassification], args: &[String]) {
    let mut filters = Vec::new();
    let mut exclude_filters = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--with" | "-w" => {
                if i + 1 < args.len() {
                    filters.push(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--without" | "-wo" => {
                if i + 1 < args.len() {
                    exclude_filters.push(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    for classification in classifications {
        let mut include = true;

        // Apply include filters
        for filter in &filters {
            let (attr, value) = if let Some(pos) = filter.find('=') {
                (&filter[..pos], &filter[pos + 1..])
            } else {
                (filter.as_str(), "")
            };

            let matches = match attr {
                "linguist-language" => classification
                    .linguist_language
                    .as_ref()
                    .map_or(false, |lang| lang == value),
                "linguist-vendored" => classification.is_vendored == (value == "true"),
                "linguist-generated" => classification.is_generated == (value == "true"),
                "linguist-documentation" => classification.is_documentation == (value == "true"),
                "linguist-detectable" => classification.is_detectable == (value == "true"),
                _ => false,
            };

            if !matches {
                include = false;
                break;
            }
        }

        // Apply exclude filters
        for filter in &exclude_filters {
            let (attr, value) = if let Some(pos) = filter.find('=') {
                (&filter[..pos], &filter[pos + 1..])
            } else {
                (filter.as_str(), "")
            };

            let matches = match attr {
                "linguist-language" => classification
                    .linguist_language
                    .as_ref()
                    .map_or(false, |lang| lang == value),
                "linguist-vendored" => classification.is_vendored == (value == "true"),
                "linguist-generated" => classification.is_generated == (value == "true"),
                "linguist-documentation" => classification.is_documentation == (value == "true"),
                "linguist-detectable" => classification.is_detectable == (value == "true"),
                _ => false,
            };

            if matches {
                include = false;
                break;
            }
        }

        if include {
            println!("{}", classification.path.display());
        }
    }
}

fn print_matrix_command(classifications: &[FileClassification], args: &[String]) {
    let default_group_by = "linguist-language".to_string();
    let group_by = args
        .iter()
        .position(|arg| arg == "--group-by" || arg == "-g")
        .and_then(|pos| args.get(pos + 1))
        .unwrap_or(&default_group_by);

    let mut groups: HashMap<String, Vec<&FileClassification>> = HashMap::new();

    for classification in classifications {
        let group_key = match group_by.as_str() {
            "linguist-language" => classification
                .linguist_language
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            "linguist-vendored" => {
                if classification.is_vendored {
                    "vendored".to_string()
                } else {
                    "not-vendored".to_string()
                }
            }
            "linguist-generated" => {
                if classification.is_generated {
                    "generated".to_string()
                } else {
                    "not-generated".to_string()
                }
            }
            "linguist-documentation" => {
                if classification.is_documentation {
                    "documentation".to_string()
                } else {
                    "not-documentation".to_string()
                }
            }
            "linguist-detectable" => {
                if classification.is_detectable {
                    "detectable".to_string()
                } else {
                    "not-detectable".to_string()
                }
            }
            _ => "unknown".to_string(),
        };

        groups
            .entry(group_key)
            .or_insert_with(Vec::new)
            .push(classification);
    }

    println!("📊 File Classification Matrix");
    println!("============================");
    println!();
    println!("Grouped by: {}", group_by);
    println!();

    let mut sorted_groups: Vec<_> = groups.iter().collect();
    sorted_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (group, files) in sorted_groups {
        println!("{} ({} files):", group, files.len());
        for file in files.iter().take(5) {
            println!("  {}", file.path.display());
        }
        if files.len() > 5 {
            println!("  ... and {} more", files.len() - 5);
        }
        println!();
    }
}

fn print_stats_command(classifications: &[FileClassification]) {
    println!("📈 File Classification Statistics");
    println!("================================");
    println!();

    let mut language_stats: HashMap<String, usize> = HashMap::new();
    let mut vendored_count = 0;
    let mut generated_count = 0;
    let mut documentation_count = 0;
    let mut detectable_count = 0;
    let mut unclassified_count = 0;

    for classification in classifications {
        if let Some(lang) = &classification.linguist_language {
            *language_stats.entry(lang.clone()).or_insert(0) += 1;
        } else {
            unclassified_count += 1;
        }

        if classification.is_vendored {
            vendored_count += 1;
        }
        if classification.is_generated {
            generated_count += 1;
        }
        if classification.is_documentation {
            documentation_count += 1;
        }
        if classification.is_detectable {
            detectable_count += 1;
        }
    }

    println!("📋 Language Distribution:");
    let mut sorted_langs: Vec<_> = language_stats.iter().collect();
    sorted_langs.sort_by(|a, b| b.1.cmp(a.1));

    for (lang, count) in sorted_langs {
        let percentage = (*count as f64 / classifications.len() as f64) * 100.0;
        println!("  {}: {} files ({:.1}%)", lang, count, percentage);
    }
    println!();

    println!("📊 Override Statistics:");
    println!(
        "  Vendored files: {} ({:.1}%)",
        vendored_count,
        (vendored_count as f64 / classifications.len() as f64) * 100.0
    );
    println!(
        "  Generated files: {} ({:.1}%)",
        generated_count,
        (generated_count as f64 / classifications.len() as f64) * 100.0
    );
    println!(
        "  Documentation: {} ({:.1}%)",
        documentation_count,
        (documentation_count as f64 / classifications.len() as f64) * 100.0
    );
    println!(
        "  Detectable files: {} ({:.1}%)",
        detectable_count,
        (detectable_count as f64 / classifications.len() as f64) * 100.0
    );
    println!(
        "  Unclassified: {} ({:.1}%)",
        unclassified_count,
        (unclassified_count as f64 / classifications.len() as f64) * 100.0
    );
    println!();

    // Show workflow recommendations
    println!("💡 Workflow Recommendations:");

    if vendored_count > 0 {
        println!(
            "  • Exclude {} vendored files from linting/formatting",
            vendored_count
        );
    }

    if generated_count > 0 {
        println!(
            "  • Skip diff review for {} generated files",
            generated_count
        );
    }

    if documentation_count > 0 {
        println!(
            "  • Route {} documentation files to doc validation tools",
            documentation_count
        );
    }

    let rust_files = language_stats.get("Rust").unwrap_or(&0);
    if *rust_files > 0 {
        println!("  • Run cargo fmt/clippy on {} Rust files", rust_files);
    }

    let json_files = language_stats.get("JSON").unwrap_or(&0);
    if *json_files > 0 {
        println!(
            "  • Validate {} JSON files with schema validation",
            json_files
        );
    }
}

fn print_workflow_examples(classifications: &[FileClassification]) {
    println!("🛠️  Workflow Examples");
    println!("=====================");
    println!();

    // Rust files
    let rust_files: Vec<_> = classifications
        .iter()
        .filter(|c| {
            c.linguist_language
                .as_ref()
                .map_or(false, |lang| lang == "Rust")
        })
        .collect();

    if !rust_files.is_empty() {
        println!("🔧 Rust Workflow:");
        println!("  # Format Rust files");
        println!(
            "  cargo fmt --check -- {}",
            rust_files
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!("  # Clippy check");
        println!(
            "  cargo clippy -- {}",
            rust_files
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!();
    }

    // Non-vendored files
    let non_vendored: Vec<_> = classifications.iter().filter(|c| !c.is_vendored).collect();

    if !non_vendored.is_empty() {
        println!("📝 Non-Vendored Files:");
        println!("  # Apply linting to non-vendored files");
        println!(
            "  echo '{}' | xargs my-linter",
            non_vendored
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!();
    }

    // Generated files
    let generated_files: Vec<_> = classifications.iter().filter(|c| c.is_generated).collect();

    if !generated_files.is_empty() {
        println!("⚙️  Generated Files:");
        println!("  # Skip generated files in diffs");
        println!(
            "  git diff -- {}",
            generated_files
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!("  # Validate generated files match source");
        println!(
            "  make generate && git diff --exit-code -- {}",
            generated_files
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!();
    }

    // Documentation files
    let doc_files: Vec<_> = classifications
        .iter()
        .filter(|c| c.is_documentation)
        .collect();

    if !doc_files.is_empty() {
        println!("📚 Documentation Files:");
        println!("  # Apply doc validation");
        println!(
            "  echo '{}' | xargs markdownlint",
            doc_files
                .iter()
                .map(|c| c.path.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!();
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("attr-scan - Parse and utilize .gitattributes linguist overrides");
        println!();
        println!("Usage:");
        println!(
            "  {} files [options]     - List files with filters",
            args[0]
        );
        println!(
            "  {} matrix [options]    - Show classification matrix",
            args[0]
        );
        println!("  {} stats               - Show statistics", args[0]);
        println!("  {} examples            - Show workflow examples", args[0]);
        println!();
        println!("Options:");
        println!("  --with <attr>=<value>    - Include files matching attribute");
        println!("  --without <attr>=<value> - Exclude files matching attribute");
        println!("  --group-by <attr>        - Group by attribute (for matrix)");
        println!();
        println!("Examples:");
        println!("  {} files --with linguist-language=Rust", args[0]);
        println!("  {} files --without linguist-vendored", args[0]);
        println!("  {} matrix --group-by linguist-language", args[0]);
        println!("  {} stats", args[0]);
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

    // Read .gitattributes
    let gitattributes_content = match read_gitattributes() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading .gitattributes: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Parse .gitattributes
    let entries = parse_gitattributes(&gitattributes_content);

    // Get all git files
    let files = match get_git_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error getting git files: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Classify all files (with parallel processing for large repos)
    let classifications: Vec<FileClassification> = if files.len() > 1000 {
        // Use parallel processing for large repositories
        let entries_arc = Arc::new(entries);
        let files_chunks: Vec<Vec<PathBuf>> = files
            .chunks(files.len() / num_cpus::get().max(1))
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut handles = vec![];
        for chunk in files_chunks {
            let entries_clone = Arc::clone(&entries_arc);
            let handle = thread::spawn(move || {
                chunk
                    .iter()
                    .map(|file| classify_file(file, &entries_clone))
                    .collect::<Vec<FileClassification>>()
            });
            handles.push(handle);
        }

        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    } else {
        // Sequential processing for smaller repositories
        files
            .iter()
            .map(|file| classify_file(file, &entries))
            .collect()
    };

    // Handle commands
    match args[1].as_str() {
        "files" => {
            print_files_command(&classifications, &args[2..]);
        }
        "matrix" => {
            print_matrix_command(&classifications, &args[2..]);
        }
        "stats" => {
            print_stats_command(&classifications);
        }
        "examples" => {
            print_workflow_examples(&classifications);
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            eprintln!("Use 'files', 'matrix', 'stats', or 'examples'");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
