use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
pub struct DocsCommand {
    #[command(subcommand)]
    pub subcommand: DocsSubcommand,
}

#[derive(Subcommand)]
pub enum DocsSubcommand {
    /// Extract documentation from Rust source files
    Extract(ExtractCommand),
    /// Generate markdown from extracted documentation
    Generate(GenerateCommand),
    /// Extract and generate in one step
    All(AllCommand),
}

#[derive(Parser)]
struct ExtractCommand {
    /// Source directory to scan for .rs files
    #[clap(long, default_value = "src")]
    source: PathBuf,

    /// Output directory for extracted JSONC files
    #[clap(long, default_value = "generated-sources/docs")]
    output: PathBuf,

    /// Include test files
    #[clap(long)]
    include_tests: bool,
}

#[derive(Parser)]
struct GenerateCommand {
    /// Input directory containing JSONC files
    #[clap(long, default_value = "generated-sources/docs")]
    input: PathBuf,

    /// Output directory for generated markdown
    #[clap(long, default_value = "docs")]
    output: PathBuf,

    /// Template to use for generation
    #[clap(long, default_value = "default")]
    template: String,
}

#[derive(Parser)]
struct AllCommand {
    /// Source directory to scan for .rs files
    #[clap(long, default_value = "src")]
    source: PathBuf,

    /// Output directory for generated markdown
    #[clap(long, default_value = "docs")]
    output: PathBuf,

    /// Include test files
    #[clap(long)]
    include_tests: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocItem {
    /// File path relative to project root
    path: String,
    /// Module/function name
    name: String,
    /// Type of item (module, function, struct, enum, etc.)
    item_type: String,
    /// Documentation comments
    docs: Vec<String>,
    /// Source code signature (if available)
    signature: Option<String>,
    /// Line number where item starts
    line_start: usize,
    /// Line number where item ends
    line_end: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocFile {
    /// File path relative to project root
    path: String,
    /// File-level documentation
    file_docs: Vec<String>,
    /// Items in the file
    items: Vec<DocItem>,
    /// Checksum of the file content
    checksum: String,
}

pub fn run(cmd: DocsCommand) -> Result<()> {
    match cmd.subcommand {
        DocsSubcommand::Extract(cmd) => extract_docs(cmd),
        DocsSubcommand::Generate(cmd) => generate_docs(cmd),
        DocsSubcommand::All(cmd) => {
            let extract_cmd = ExtractCommand {
                source: cmd.source.clone(),
                output: PathBuf::from("generated-sources/docs"),
                include_tests: cmd.include_tests,
            };
            extract_docs(extract_cmd)?;

            let generate_cmd = GenerateCommand {
                input: PathBuf::from("generated-sources/docs"),
                output: cmd.output,
                template: "default".to_string(),
            };
            generate_docs(generate_cmd)
        }
    }
}

fn extract_docs(cmd: ExtractCommand) -> Result<()> {
    println!("🔍 Extracting documentation from Rust source files...");

    // Create output directory
    fs::create_dir_all(&cmd.output)?;

    // Find all .rs files
    let rs_files = find_rs_files(&cmd.source, cmd.include_tests)?;
    println!("Found {} Rust files", rs_files.len());

    for file_path in rs_files {
        let relative_path = file_path.strip_prefix(".").unwrap_or(&file_path);
        println!("Processing: {}", relative_path.display());

        let doc_file = extract_file_docs(&file_path)?;

        // Write JSONC file
        let output_path = cmd.output.join(format!(
            "{}.jsonc",
            relative_path
                .to_string_lossy()
                .replace('/', "_")
                .replace(".rs", "")
        ));

        let jsonc_content = format!(
            "// @generated\n// Documentation extracted from {}\n// @checksum: {}\n\n{}",
            relative_path.display(),
            doc_file.checksum,
            serde_json::to_string_pretty(&doc_file)?
        );

        fs::write(&output_path, jsonc_content)?;
        println!("  → {}", output_path.display());
    }

    println!("✅ Documentation extraction complete!");
    Ok(())
}

fn generate_docs(cmd: GenerateCommand) -> Result<()> {
    println!("📝 Generating markdown documentation...");

    // Create output directory
    fs::create_dir_all(&cmd.output)?;

    // Find all JSONC files
    let jsonc_files = find_jsonc_files(&cmd.input)?;
    println!("Found {} JSONC files", jsonc_files.len());

    for file_path in jsonc_files {
        let relative_path = file_path.strip_prefix(&cmd.input).unwrap_or(&file_path);
        println!("Processing: {}", relative_path.display());

        let content = fs::read_to_string(&file_path)?;
        let doc_file: DocFile = serde_json::from_str(&content)?;

        // Generate markdown
        let markdown = generate_markdown(&doc_file, &cmd.template)?;

        // Write markdown file
        let output_path = cmd.output.join(format!(
            "{}.md",
            relative_path.to_string_lossy().replace(".jsonc", "")
        ));

        let markdown_content = format!(
            "<!-- @generated -->\n<!-- @checksum: {} -->\n\n{}",
            doc_file.checksum, markdown
        );

        fs::write(&output_path, markdown_content)?;
        println!("  → {}", output_path.display());
    }

    println!("✅ Markdown generation complete!");
    Ok(())
}

fn find_rs_files(source_dir: &Path, include_tests: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Skip test files unless explicitly included
            if !include_tests
                && (path.to_string_lossy().contains("test")
                    || path.to_string_lossy().contains("tests"))
            {
                continue;
            }

            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn find_jsonc_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonc") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn extract_file_docs(file_path: &Path) -> Result<DocFile> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Compute checksum
    let mut hasher = sha2::Sha256::new();
    hasher.update(content.as_bytes());
    let checksum = format!("{:x}", hasher.finalize());

    // Extract file-level documentation
    let file_docs = extract_file_level_docs(&lines);

    // Extract item-level documentation
    let items = extract_item_docs(&lines)?;

    Ok(DocFile {
        path: file_path.to_string_lossy().to_string(),
        file_docs,
        items,
        checksum,
    })
}

fn extract_file_level_docs(lines: &[&str]) -> Vec<String> {
    let mut docs = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") {
            docs.push(trimmed[3..].trim().to_string());
        } else if trimmed.starts_with("///") && docs.is_empty() {
            // Only include /// comments at the top of the file
            docs.push(trimmed[3..].trim().to_string());
        } else if !trimmed.starts_with("///") && !trimmed.starts_with("//!") {
            // Stop at first non-doc comment line
            break;
        }
    }

    docs
}

fn extract_item_docs(lines: &[&str]) -> Result<Vec<DocItem>> {
    let mut items = Vec::new();
    let mut current_docs = Vec::new();
    let mut in_item = false;
    let mut item_start = 0;
    let mut item_name = String::new();
    let mut item_type = String::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("///") {
            current_docs.push(trimmed[3..].trim().to_string());
        } else if trimmed.starts_with("//!") {
            // File-level docs, skip
            continue;
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // This is code, check if it's a new item
            if let Some((name, item_type_str)) = parse_item_declaration(trimmed) {
                // Save previous item if we have one
                if in_item && !item_name.is_empty() {
                    items.push(DocItem {
                        path: String::new(), // Will be set by caller
                        name: item_name.clone(),
                        item_type: item_type.clone(),
                        docs: current_docs.clone(),
                        signature: Some(lines[item_start..=i].join("\n")),
                        line_start: item_start + 1,
                        line_end: i + 1,
                    });
                }

                // Start new item
                item_name = name.to_string();
                item_type = item_type_str.to_string();
                item_start = i;
                in_item = true;
                current_docs.clear();
            }
        }
    }

    // Don't forget the last item
    if in_item && !item_name.is_empty() {
        items.push(DocItem {
            path: String::new(),
            name: item_name,
            item_type,
            docs: current_docs,
            signature: Some(lines[item_start..].join("\n")),
            line_start: item_start + 1,
            line_end: lines.len(),
        });
    }

    Ok(items)
}

fn parse_item_declaration(line: &str) -> Option<(&str, &str)> {
    // Simple parsing for common Rust item declarations
    let patterns = [
        ("fn ", "function"),
        ("struct ", "struct"),
        ("enum ", "enum"),
        ("trait ", "trait"),
        ("impl ", "impl"),
        ("mod ", "module"),
        ("pub fn ", "function"),
        ("pub struct ", "struct"),
        ("pub enum ", "enum"),
        ("pub trait ", "trait"),
        ("pub mod ", "module"),
    ];

    for (pattern, item_type) in patterns {
        if line.starts_with(pattern) {
            let name_start = pattern.len();
            let name_end = line[name_start..]
                .find(|c: char| c.is_whitespace() || c == '(' || c == '{')
                .unwrap_or(line[name_start..].len());
            let name = &line[name_start..name_start + name_end];
            return Some((name, item_type));
        }
    }

    None
}

fn generate_markdown(doc_file: &DocFile, template: &str) -> Result<String> {
    let mut markdown = String::new();

    // File header
    markdown.push_str(&format!(
        "# {}\n\n",
        doc_file.path.split('/').last().unwrap_or("Unknown")
    ));

    // File-level documentation
    if !doc_file.file_docs.is_empty() {
        markdown.push_str("## Overview\n\n");
        for doc in &doc_file.file_docs {
            markdown.push_str(&format!("{}\n\n", doc));
        }
    }

    // Group items by type
    let mut items_by_type: HashMap<String, Vec<&DocItem>> = HashMap::new();
    for item in &doc_file.items {
        items_by_type
            .entry(item.item_type.clone())
            .or_insert_with(Vec::new)
            .push(item);
    }

    // Generate sections for each type
    for (item_type, items) in items_by_type {
        let type_title = match item_type.as_str() {
            "function" => "Functions",
            "struct" => "Structs",
            "enum" => "Enums",
            "trait" => "Traits",
            "impl" => "Implementations",
            "module" => "Modules",
            _ => &item_type,
        };

        markdown.push_str(&format!("## {}\n\n", type_title));

        for item in items {
            markdown.push_str(&format!("### {}\n\n", item.name));

            // Documentation
            for doc in &item.docs {
                markdown.push_str(&format!("{}\n\n", doc));
            }

            // Signature (if available)
            if let Some(signature) = &item.signature {
                markdown.push_str("```rust\n");
                markdown.push_str(signature);
                markdown.push_str("\n```\n\n");
            }
        }
    }

    Ok(markdown)
}
