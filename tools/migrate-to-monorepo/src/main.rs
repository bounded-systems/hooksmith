use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Migration plan for moving files to monorepo orchestrator structure
#[derive(Debug)]
struct MigrationPlan {
    /// Files to move and their target locations
    moves: Vec<(String, String)>,
    /// Directories to create
    create_dirs: Vec<String>,
    /// Files to create with content
    create_files: Vec<(String, String)>,
}

impl MigrationPlan {
    fn new() -> Self {
        Self {
            moves: Vec::new(),
            create_dirs: Vec::new(),
            create_files: Vec::new(),
        }
    }

    /// Add a file move operation
    fn add_move(&mut self, from: &str, to: &str) {
        self.moves.push((from.to_string(), to.to_string()));
    }

    /// Add a directory creation
    fn add_create_dir(&mut self, dir: &str) {
        self.create_dirs.push(dir.to_string());
    }

    /// Add a file creation with content
    fn add_create_file(&mut self, path: &str, content: &str) {
        self.create_files.push((path.to_string(), content.to_string()));
    }

    /// Build the complete migration plan
    fn build() -> Self {
        let mut plan = Self::new();

        // Create necessary directories
        plan.add_create_dir("docs/summaries");
        plan.add_create_dir("schemas");
        plan.add_create_dir("apps");
        plan.add_create_dir("tools");
        plan.add_create_dir("infra");
        plan.add_create_dir("examples");
        plan.add_create_dir("tests");
        plan.add_create_dir(".hooksmith/agreements");
        plan.add_create_dir(".hooksmith/actors");
        plan.add_create_dir(".hooksmith/snapshots");
        plan.add_create_dir(".hooksmith/cache");
        plan.add_create_dir(".hooksmith/logs");
        plan.add_create_dir(".hooksmith/hooks");
        plan.add_create_dir(".hooksmith/refs");

        // Move summary and implementation docs
        plan.add_move("*_SUMMARY.md", "docs/summaries/");
        plan.add_move("*_IMPLEMENTATION_*.md", "docs/summaries/");

        // Move schemas and config files
        plan.add_move("languages.yml", "schemas/");
        plan.add_move("lefthook.yml", ".github/");
        plan.add_move("agreement.json", "docs/");

        // Move Hooksmith configuration
        plan.add_move("contracts", ".hooksmith/agreements/");
        plan.add_move("contract_snapshots", ".hooksmith/snapshots/");

        // Move generated files
        plan.add_move("generated-sources", "crates/");

        // Move examples and test files
        plan.add_move("examples", "examples/");
        plan.add_move("test-*", "tests/");
        plan.add_move("test_*", "tests/");

        // Move binaries and applications
        plan.add_move("src", "apps/hooksmith-core");
        plan.add_move("standalone-auditor", "apps/");

        // Move tools and generators
        plan.add_move("scripts", "tools/");
        plan.add_move("hooks", "tools/lefthook-rs");
        plan.add_move("wit", "tools/");
        plan.add_move("worktree-lifecycle", "tools/");

        // Move infrastructure and config
        plan.add_move("config", "infra/config-model");
        plan.add_move("schemas", "infra/schemas");
        plan.add_move("docker-compose.yml", "infra/");
        plan.add_move("docker-bake.hcl", "infra/");
        plan.add_move("Dockerfile", "infra/");

        // Create .hooksmith/.gitignore
        plan.add_create_file(
            ".hooksmith/.gitignore",
            "cache/\nlogs/\n*.tmp\n",
        );

        plan
    }
}

/// Execute a git mv command
fn git_mv(from: &str, to: &str) -> Result<()> {
    println!("📁 Moving {} → {}", from, to);
    
    let output = Command::new("git")
        .args(["mv", from, to])
        .output()
        .with_context(|| format!("Failed to move {} to {}", from, to))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("fatal: bad source") || stderr.contains("does not exist") {
            println!("  ⚠️  {} not found, skipping", from);
        } else {
            anyhow::bail!("git mv failed: {}", stderr);
        }
    } else {
        println!("  ✅ Moved {} → {}", from, to);
    }

    Ok(())
}

/// Create a directory
fn create_dir(dir: &str) -> Result<()> {
    println!("📁 Creating directory: {}", dir);
    
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory: {}", dir))?;
    
    println!("  ✅ Created {}", dir);
    Ok(())
}

/// Create a file with content
fn create_file(path: &str, content: &str) -> Result<()> {
    println!("📁 Creating file: {}", path);
    
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for: {}", path))?;
    }
    
    fs::write(path, content)
        .with_context(|| format!("Failed to create file: {}", path))?;
    
    println!("  ✅ Created {}", path);
    Ok(())
}

/// Execute the migration plan
fn execute_migration(plan: &MigrationPlan) -> Result<()> {
    println!("🚀 Starting migration to monorepo orchestrator root layout...");
    println!();

    // Create directories first
    for dir in &plan.create_dirs {
        create_dir(dir)?;
    }
    println!();

    // Move files
    for (from, to) in &plan.moves {
        git_mv(from, to)?;
    }
    println!();

    // Create files
    for (path, content) in &plan.create_files {
        create_file(path, content)?;
    }
    println!();

    println!("✅ Migration completed!");
    println!();
    println!("📋 Next steps:");
    println!("1. Review the changes: git status");
            println!("2. Test the monorepo root contract:");
        println!("   cd apps/standalone-auditor && cargo run -- HEAD ../../.hooksmith/agreements/object-names@root-minimal.json");
        println!("   cd apps/standalone-auditor && cargo run -- HEAD ../../.hooksmith/agreements/object-names@v1.json");
    println!("3. Commit the changes: git commit -m 'chore: migrate to monorepo orchestrator root layout'");
    println!();
    println!("🎯 Target root layout (Monorepo Orchestrator):");
    println!("  - .gitignore");
    println!("  - .gitattributes");
    println!("  - .github/             # CI/CD");
    println!("  - .hooksmith/          # All Hooksmith configuration");
    println!("  - Cargo.toml           # Workspace manifest only");
    println!("  - Cargo.lock           # Workspace lockfile");
    println!("  - README.md");
    println!("  - CONTRIBUTING.md");
    println!("  - LICENSE");
    println!("  - crates/              # Shared libraries");
    println!("  - apps/                # Binaries/CLIs/services");
    println!("  - tools/               # Dev tooling, analyzers, generators");
    println!("  - infra/               # Deploy, IaC, pipelines");
    println!("  - docs/                # Documentation");
    println!("  - schemas/             # Machine-checked schemas");
    println!("  - contracts/           # Optional if not under .hooksmith/");
    println!("  - examples/            # Repo-wide examples");
    println!("  - tests/               # Integration tests");

    Ok(())
}

/// Show a dry run of the migration plan
fn dry_run(plan: &MigrationPlan) -> Result<()> {
    println!("🔍 DRY RUN - Migration plan preview:");
    println!();

    println!("📁 Directories to create:");
    for dir in &plan.create_dirs {
        println!("  + {}", dir);
    }
    println!();

    println!("📁 Files to move:");
    for (from, to) in &plan.moves {
        println!("  {} → {}", from, to);
    }
    println!();

    println!("📁 Files to create:");
    for (path, _) in &plan.create_files {
        println!("  + {}", path);
    }
    println!();

    println!("💡 Run with --execute to perform the actual migration");

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let is_dry_run = args.len() > 1 && args[1] == "--dry-run";
    let is_execute = args.len() > 1 && args[1] == "--execute";

    if !is_dry_run && !is_execute {
        println!("Usage: {} [--dry-run|--execute]", args[0]);
        println!();
        println!("Options:");
        println!("  --dry-run    Show what would be migrated (default)");
        println!("  --execute    Perform the actual migration");
        println!();
        println!("This tool migrates the repository to a monorepo orchestrator layout.");
        return Ok(());
    }

    let plan = MigrationPlan::build();

    if is_dry_run {
        dry_run(&plan)?;
    } else if is_execute {
        execute_migration(&plan)?;
    }

    Ok(())
}
