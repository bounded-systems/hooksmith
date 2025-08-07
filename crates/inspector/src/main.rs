use clap::{Parser, Subcommand};
use dircheck_core::git_inspector::{GitInspector, GitInspectionReport, format_inspection_report_markdown};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "git-inspector")]
#[command(about = "Comprehensive Git repository inspection and analysis tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform comprehensive repository inspection
    Inspect {
        /// Output format (markdown, json)
        #[arg(long, default_value = "markdown")]
        format: String,
        
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Check if repository is clean (no unreachable objects)
    Clean,
    
    /// Analyze local/remote branch synchronization
    Sync,
    
    /// List unreachable objects with details
    Unreachable {
        /// Show first few lines of blob content
        #[arg(long)]
        show_content: bool,
    },
    
    /// Generate recovery suggestions for unreachable objects
    Recovery,
    
    /// Show worktree information
    Worktrees,
    
    /// Show notes structure and usage
    Notes,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Inspect { format, output } => {
            println!("🔍 Performing comprehensive Git repository inspection...");
            let report = GitInspector::inspect_repository()?;
            
            let content = match format.as_str() {
                "markdown" => format_inspection_report_markdown(&report),
                "json" => serde_json::to_string_pretty(&report)?,
                _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
            };
            
            if let Some(output_path) = output {
                fs::write(output_path, content)?;
                println!("✅ Report written to file");
            } else {
                println!("{}", content);
            }
        }
        
        Commands::Clean => {
            let is_clean = GitInspector::is_repository_clean()?;
            if is_clean {
                println!("✅ Repository is clean - no unreachable objects found");
            } else {
                println!("⚠️  Repository has unreachable objects");
                let unreachable = dircheck_core::git_inspector::GitInspector::analyze_unreachable()?;
                println!("Found {} unreachable objects", unreachable.total_unreachable);
            }
        }
        
        Commands::Sync => {
            println!("🔄 Analyzing local/remote branch synchronization...");
            let comparison = GitInspector::get_branch_comparison()?;
            
            let mut synced = Vec::new();
            let mut local_only = Vec::new();
            let mut remote_only = Vec::new();
            
            for (branch, status) in comparison {
                match status {
                    dircheck_core::git_inspector::BranchStatus::Synced => synced.push(branch),
                    dircheck_core::git_inspector::BranchStatus::LocalOnly => local_only.push(branch),
                    dircheck_core::git_inspector::BranchStatus::RemoteOnly => remote_only.push(branch),
                    dircheck_core::git_inspector::BranchStatus::OutOfSync => println!("⚠️  {} (out of sync)", branch),
                }
            }
            
            println!("\n📊 Branch Synchronization Status:");
            println!("✅ Synced branches: {}", synced.len());
            for branch in synced {
                println!("  - {}", branch);
            }
            
            if !local_only.is_empty() {
                println!("\n🏠 Local-only branches: {}", local_only.len());
                for branch in local_only {
                    println!("  - {}", branch);
                }
            }
            
            if !remote_only.is_empty() {
                println!("\n🌐 Remote-only branches: {}", remote_only.len());
                for branch in remote_only {
                    println!("  - {}", branch);
                }
            }
        }
        
        Commands::Unreachable { show_content } => {
            println!("🔍 Analyzing unreachable objects...");
            let unreachable = dircheck_core::git_inspector::GitInspector::analyze_unreachable()?;
            
            println!("Found {} unreachable objects:", unreachable.total_unreachable);
            for (object_type, count) in &unreachable.by_type {
                println!("  - {}: {}", object_type, count);
            }
            
            if show_content {
                println!("\n📄 Object details:");
                for object in &unreachable.objects {
                    println!("== {} ({}) ==", object.sha, object.object_type);
                    if let Some(size) = object.size {
                        println!("Size: {} bytes", size);
                    }
                    if let Some(first_line) = &object.first_line {
                        println!("First line: {}", first_line);
                    }
                    println!();
                }
            }
        }
        
        Commands::Recovery => {
            println!("🔧 Generating recovery suggestions...");
            let unreachable = dircheck_core::git_inspector::GitInspector::analyze_unreachable()?;
            let suggestions = GitInspector::generate_recovery_suggestions(&unreachable);
            
            for suggestion in suggestions {
                println!("💡 {}", suggestion);
            }
        }
        
        Commands::Worktrees => {
            println!("🌳 Worktree Information:");
            let output = std::process::Command::new("git")
                .args(["worktree", "list", "--porcelain"])
                .output()?;
            
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let mut current_worktree = None;
                
                for line in stdout.lines() {
                    if line.starts_with("worktree ") {
                        if let Some(path) = line.strip_prefix("worktree ") {
                            current_worktree = Some(path.to_string());
                            println!("\n📁 Worktree: {}", path);
                        }
                    } else if line.starts_with("HEAD ") {
                        if let Some(sha) = line.strip_prefix("HEAD ") {
                            println!("  HEAD: {}", sha);
                        }
                    } else if line.starts_with("branch ") {
                        if let Some(branch) = line.strip_prefix("branch ") {
                            println!("  Branch: {}", branch);
                        }
                    }
                }
            } else {
                println!("❌ Failed to get worktree information");
            }
        }
        
        Commands::Notes => {
            println!("📝 Git Notes Structure:");
            println!("\nCanonical Note Ref Layout: notes/{{git_object}}");
            println!("\nGit object types:");
            println!("  - commit");
            println!("  - tree");
            println!("  - blob");
            println!("  - tag");
            
            println!("\nProposed Git note refs:");
            println!("Git Object\tCanonical Note Ref\tExample Use");
            println!("commit\t\trefs/notes/commit\tValidation status, CI metadata");
            println!("tree\t\trefs/notes/tree\tDirectory structure validation");
            println!("blob\t\trefs/notes/blob\tFile-level lint/language detection");
            println!("tag\t\trefs/notes/tag\tRelease changelogs, build hashes");
            
            println!("\nExtended with purpose-based subrefs:");
            println!("  refs/notes/blob/language");
            println!("  refs/notes/blob/lint");
            println!("  refs/notes/tree/dircheck");
            println!("  refs/notes/commit/contracts");
            println!("  refs/notes/tag/releases");
            
            // Check for existing notes
            let output = std::process::Command::new("git")
                .args(["notes", "--ref=refs/notes/commit", "list"])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)?;
                    if !stdout.trim().is_empty() {
                        println!("\n📋 Existing commit notes:");
                        for line in stdout.lines() {
                            println!("  {}", line);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
