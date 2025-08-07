use anyhow::Result;
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "github-workflow-generator")]
#[command(about = "Generate GitHub Actions workflows from hooksmith schema")]
struct Args {
    /// Output directory for workflow files
    #[arg(long, default_value = ".github/workflows")]
    output_dir: String,
    
    /// Generate all workflows
    #[arg(long)]
    all: bool,
    
    /// Specific event to generate (e.g., push, pull_request)
    #[arg(long)]
    event: Option<String>,
    
    /// Include activity types filter
    #[arg(long)]
    with_types: bool,
}

/// Mapping from GitHub events to Git hooks
#[derive(Debug)]
struct EventHookMapping {
    github_event: String,
    git_hooks: Vec<String>,
    activity_types: Vec<String>,
    description: String,
}

impl EventHookMapping {
    fn new(github_event: &str, git_hooks: Vec<&str>, activity_types: Vec<&str>, description: &str) -> Self {
        Self {
            github_event: github_event.to_string(),
            git_hooks: git_hooks.iter().map(|s| s.to_string()).collect(),
            activity_types: activity_types.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
        }
    }
}

fn get_event_mappings() -> Vec<EventHookMapping> {
    vec![
        EventHookMapping::new(
            "push",
            vec!["pre-commit", "commit-msg", "post-commit"],
            vec!["created", "deleted"],
            "Validates commits and changes pushed to the repository"
        ),
        EventHookMapping::new(
            "pull_request",
            vec!["pre-commit", "commit-msg", "prepare-commit-msg"],
            vec!["opened", "synchronize", "reopened", "closed"],
            "Validates pull request changes and commit messages"
        ),
        EventHookMapping::new(
            "pull_request_target",
            vec!["pre-receive", "update", "post-receive"],
            vec!["opened", "synchronize", "reopened", "closed"],
            "Validates pull request changes from the target branch context"
        ),
        EventHookMapping::new(
            "issues",
            vec!["commit-msg"],
            vec!["opened", "edited", "closed", "reopened"],
            "Validates issue creation and updates"
        ),
        EventHookMapping::new(
            "issue_comment",
            vec!["commit-msg"],
            vec!["created", "edited", "deleted"],
            "Validates issue comments"
        ),
        EventHookMapping::new(
            "release",
            vec!["pre-receive", "post-receive"],
            vec!["created", "edited", "deleted", "published", "unpublished"],
            "Validates release creation and updates"
        ),
        EventHookMapping::new(
            "create",
            vec!["pre-receive", "post-receive"],
            vec!["tag", "branch"],
            "Validates branch and tag creation"
        ),
        EventHookMapping::new(
            "delete",
            vec!["pre-receive", "post-receive"],
            vec!["tag", "branch"],
            "Validates branch and tag deletion"
        ),
        EventHookMapping::new(
            "workflow_dispatch",
            vec!["pre-commit", "post-commit"],
            vec![],
            "Manual workflow trigger for validation"
        ),
        EventHookMapping::new(
            "workflow_run",
            vec!["pre-commit", "post-commit"],
            vec!["requested", "completed"],
            "Validates workflow runs"
        ),
    ]
}

fn generate_workflow_yaml(mapping: &EventHookMapping, with_types: bool) -> String {
    let mut yaml = format!(
        "name: hooksmith-{}-validation\n",
        mapping.github_event.replace('_', "-")
    );
    
    yaml.push_str("on:\n");
    yaml.push_str(&format!("  {}:\n", mapping.github_event));
    
    if with_types && !mapping.activity_types.is_empty() {
        yaml.push_str("    types:\n");
        for activity_type in &mapping.activity_types {
            yaml.push_str(&format!("      - {}\n", activity_type));
        }
    }
    
    yaml.push_str("\n");
    yaml.push_str("jobs:\n");
    yaml.push_str("  validate:\n");
    yaml.push_str("    runs-on: ubuntu-latest\n");
    yaml.push_str("    steps:\n");
    yaml.push_str("      - name: Checkout code\n");
    yaml.push_str("        uses: actions/checkout@v4\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          fetch-depth: 0\n");
    yaml.push_str("\n");
    yaml.push_str("      - name: Setup Rust\n");
    yaml.push_str("        uses: actions-rs/toolchain@v1\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          toolchain: stable\n");
    yaml.push_str("          override: true\n");
    yaml.push_str("\n");
    yaml.push_str("      - name: Build hooksmith\n");
    yaml.push_str("        run: cargo build --release\n");
    yaml.push_str("\n");
    
    // Add validation steps for each mapped hook
    for hook in &mapping.git_hooks {
        yaml.push_str(&format!("      - name: Run {} validation\n", hook));
        yaml.push_str(&format!("        run: ./target/release/{}\n", hook));
        yaml.push_str("        env:\n");
        yaml.push_str("          GITHUB_EVENT_PATH: ${{ github.event_path }}\n");
        yaml.push_str("          GITHUB_EVENT_NAME: ${{ github.event_name }}\n");
        yaml.push_str("          GITHUB_REPOSITORY: ${{ github.repository }}\n");
        yaml.push_str("          GITHUB_REF: ${{ github.ref }}\n");
        yaml.push_str("          GITHUB_SHA: ${{ github.sha }}\n");
        yaml.push_str("\n");
    }
    
    yaml.push_str("      - name: Run GitHub event validation\n");
    yaml.push_str(&format!("        run: ./target/release/github-{}\n", mapping.github_event.replace('_', "-")));
    yaml.push_str("        env:\n");
    yaml.push_str("          GITHUB_EVENT_PATH: ${{ github.event_path }}\n");
    yaml.push_str("          GITHUB_EVENT_NAME: ${{ github.event_name }}\n");
    yaml.push_str("          GITHUB_REPOSITORY: ${{ github.repository }}\n");
    yaml.push_str("          GITHUB_REF: ${{ github.ref }}\n");
    yaml.push_str("          GITHUB_SHA: ${{ github.sha }}\n");
    
    yaml
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mappings = get_event_mappings();
    
    // Create output directory
    fs::create_dir_all(&args.output_dir)?;
    
    if args.all {
        println!("🚀 Generating all GitHub Actions workflows...");
        
        for mapping in &mappings {
            let filename = format!("hooksmith-{}.yml", mapping.github_event);
            let filepath = Path::new(&args.output_dir).join(&filename);
            
            let yaml = generate_workflow_yaml(mapping, args.with_types);
            fs::write(&filepath, yaml)?;
            
            println!("✅ Generated: {}", filepath.display());
        }
        
        // Generate a comprehensive workflow that handles multiple events
        let comprehensive_yaml = generate_comprehensive_workflow(&mappings);
        let comprehensive_path = Path::new(&args.output_dir).join("hooksmith-comprehensive.yml");
        fs::write(&comprehensive_path, comprehensive_yaml)?;
        println!("✅ Generated: {}", comprehensive_path.display());
        
    } else if let Some(event) = args.event {
        if let Some(mapping) = mappings.iter().find(|m| m.github_event == event) {
            let filename = format!("hooksmith-{}.yml", event);
            let filepath = Path::new(&args.output_dir).join(&filename);
            
            let yaml = generate_workflow_yaml(mapping, args.with_types);
            fs::write(&filepath, yaml)?;
            
            println!("✅ Generated: {}", filepath.display());
        } else {
            eprintln!("❌ Unknown event: {}", event);
            eprintln!("Available events: {}", 
                mappings.iter().map(|m| &m.github_event).collect::<Vec<_>>().join(", "));
        }
    } else {
        println!("📋 Available events:");
        for mapping in &mappings {
            println!("  - {}: {}", mapping.github_event, mapping.description);
        }
        println!("\nUse --all to generate all workflows or --event <event> for a specific event");
    }
    
    Ok(())
}

fn generate_comprehensive_workflow(mappings: &[EventHookMapping]) -> String {
    let mut yaml = String::new();
    yaml.push_str("name: Hooksmith Comprehensive Validation\n");
    yaml.push_str("\n");
    yaml.push_str("on:\n");
    
    // Add all events
    for mapping in mappings {
        yaml.push_str(&format!("  {}:\n", mapping.github_event));
        if !mapping.activity_types.is_empty() {
            yaml.push_str("    types:\n");
            for activity_type in &mapping.activity_types {
                yaml.push_str(&format!("      - {}\n", activity_type));
            }
        }
    }
    
    yaml.push_str("\n");
    yaml.push_str("jobs:\n");
    yaml.push_str("  validate:\n");
    yaml.push_str("    runs-on: ubuntu-latest\n");
    yaml.push_str("    steps:\n");
    yaml.push_str("      - name: Checkout code\n");
    yaml.push_str("        uses: actions/checkout@v4\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          fetch-depth: 0\n");
    yaml.push_str("\n");
    yaml.push_str("      - name: Setup Rust\n");
    yaml.push_str("        uses: actions-rs/toolchain@v1\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          toolchain: stable\n");
    yaml.push_str("          override: true\n");
    yaml.push_str("\n");
    yaml.push_str("      - name: Build hooksmith\n");
    yaml.push_str("        run: cargo build --release\n");
    yaml.push_str("\n");
    yaml.push_str("      - name: Run event-specific validation\n");
    yaml.push_str("        run: |\n");
    yaml.push_str("          case ${{ github.event_name }} in\n");
    
    for mapping in mappings {
        yaml.push_str(&format!("            {})\n", mapping.github_event));
        yaml.push_str(&format!("              ./target/release/github-{}\n", mapping.github_event.replace('_', "-")));
        yaml.push_str("              ;;\n");
    }
    
    yaml.push_str("          esac\n");
    yaml.push_str("        env:\n");
    yaml.push_str("          GITHUB_EVENT_PATH: ${{ github.event_path }}\n");
    yaml.push_str("          GITHUB_EVENT_NAME: ${{ github.event_name }}\n");
    yaml.push_str("          GITHUB_REPOSITORY: ${{ github.repository }}\n");
    yaml.push_str("          GITHUB_REF: ${{ github.ref }}\n");
    yaml.push_str("          GITHUB_SHA: ${{ github.sha }}\n");
    
    yaml
}
