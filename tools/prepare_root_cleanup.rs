use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

struct FileMover {
    moved_count: usize,
}

impl FileMover {
    fn new() -> Self {
        Self { moved_count: 0 }
    }

    fn mv_if(&mut self, src: &str, dst: &str) -> Result<()> {
        if Path::new(src).exists() {
            let dst_path = Path::new(dst);

            // Create destination directory if it doesn't exist
            if let Some(parent) = dst_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .context(format!("Failed to create directory: {}", parent.display()))?;
                }
            }

            // Use git mv for tracked files
            let output = Command::new("git")
                .args(["ls-files", "--error-unmatch", src])
                .output();

            match output {
                Ok(_) => {
                    // File is tracked by git, use git mv
                    let status = Command::new("git")
                        .args(["mv", "-k", src, dst])
                        .status()
                        .context(format!("Failed to git mv {} to {}", src, dst))?;

                    if status.success() {
                        println!("   ✅ Moved: {} → {}", src, dst);
                        self.moved_count += 1;
                    } else {
                        println!("   ❌ Failed to move: {} → {}", src, dst);
                    }
                }
                Err(_) => {
                    // File is not tracked by git, use regular mv
                    fs::rename(src, dst).context(format!("Failed to move {} to {}", src, dst))?;
                    println!("   ✅ Moved (untracked): {} → {}", src, dst);
                    self.moved_count += 1;
                }
            }
        } else {
            println!("   ⚠️  Not found: {}", src);
        }

        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        let directories = vec![
            ".hooksmith/contracts",
            ".hooksmith/snapshots",
            ".hooksmith/agreements",
            ".hooksmith/ops",
            ".hooksmith/worktrees",
            ".hooksmith/generated",
            ".hooksmith/metadata",
            "docs/reports",
            "infra/docker",
            "tools/gen-src",
            "tests/root",
            "apps",
        ];

        println!("📁 Creating canonical homes...");
        for dir in directories {
            if !Path::new(dir).exists() {
                fs::create_dir_all(dir).context(format!("Failed to create directory: {}", dir))?;
                println!("   Created: {}", dir);
            }
        }

        Ok(())
    }

    fn move_reports_and_summaries(&mut self) -> Result<()> {
        println!("📋 Moving reports and summaries...");

        let reports = vec![
            "ALL_PRS_SUMMARY.md",
            "BINARY_CLEANUP_SUMMARY.md",
            "BRANCH_MERGE_SUMMARY.md",
            "FINAL_PR_APPROVAL_SUMMARY.md",
            "FINAL_PR_SUMMARY.md",
            "FINAL_UPSTREAM_MERGE_SUMMARY.md",
            "GITHUB_ACTIONS_WORKFLOW_SUMMARY.md",
            "GIT_SCOPE_IMPLEMENTATION_SUMMARY.md",
            "GIT_VIEW_SCOPES.md",
            "HOOKSMITH_COMPLETE_SUMMARY.md",
            "MERGE_CONFLICTS_SUMMARY.md",
            "PR_MERGE_SUMMARY.md",
            "PR_SUMMARY.md",
            "PUSH_SUMMARY.md",
            "README_2025-08-08.md",
            "README_DIRCHECK.md",
            "SHELL_TO_RUST_CONVERSION_SUMMARY.md",
            "SHELL_TO_RUST_MIGRATION_COMPLETE.md",
            "SHELL_TO_RUST_MIGRATION_PROGRESS.md",
            "SHELL_TO_RUST_MIGRATION_SUMMARY.md",
            "WORKTREE_MIGRATION_COMPLETE.md",
            "WORKTREE_MIGRATION_SUMMARY.md",
            "WORKTREE_SYNC_STRATEGY.md",
        ];

        for report in reports {
            self.mv_if(report, &format!("docs/reports/{}", report))?;
        }

        Ok(())
    }

    fn move_hooksmith_artifacts(&mut self) -> Result<()> {
        println!("🔧 Moving Hooksmith artifacts...");

        self.mv_if("agreement.json", ".hooksmith/agreements/agreement.json")?;
        self.mv_if("contract_snapshots", ".hooksmith/snapshots")?;
        self.mv_if("contracts", ".hooksmith/contracts")?;
        self.mv_if("sha_mapping.txt", ".hooksmith/metadata/sha_mapping.txt")?;

        Ok(())
    }

    fn move_worktree_and_ops(&mut self) -> Result<()> {
        println!("🌳 Moving worktree and ops files...");

        self.mv_if(".wb", ".hooksmith/ops/.wb")?;
        self.mv_if(".workbloom", ".hooksmith/ops/.workbloom")?;
        self.mv_if("worktree-lifecycle", ".hooksmith/worktrees/lifecycle")?;
        self.mv_if(".worktree-config.json", ".hooksmith/worktrees/config.json")?;
        self.mv_if(
            ".worktree-config.jsonc",
            ".hooksmith/worktrees/config.jsonc",
        )?;

        Ok(())
    }

    fn move_generated_files(&mut self) -> Result<()> {
        println!("⚙️  Moving generated files...");

        self.mv_if("gen", "tools/gen-src")?;
        self.mv_if("generated-sources", ".hooksmith/generated")?;

        Ok(())
    }

    fn move_docker_and_infra(&mut self) -> Result<()> {
        println!("🐳 Moving Docker and infra files...");

        self.mv_if("Dockerfile", "infra/docker/Dockerfile")?;
        self.mv_if("docker-compose.yml", "infra/docker/docker-compose.yml")?;
        self.mv_if("docker-bake.hcl", "infra/docker/docker-bake.hcl")?;

        Ok(())
    }

    fn move_linguist_config(&mut self) -> Result<()> {
        println!("📊 Moving linguist config...");

        self.mv_if("languages.yml", ".github/linguist.yml")?;

        Ok(())
    }

    fn move_test_files(&mut self) -> Result<()> {
        println!("🧪 Moving test files...");

        let test_files = vec![
            "test-agreement.txt",
            "test-bad-commit.txt",
            "test-prepare-msg.txt",
            "test_files.txt",
            "test-file.rs",
            "test-validation-hooks.rs",
            "test_attribute_concerns.rs",
            "test_comprehensive_coverage.rs",
            "test_sbom.rs",
            "test-git-proxy.rs",
            "test.gitattributes",
        ];

        for test_file in test_files {
            self.mv_if(test_file, &format!("tests/root/{}", test_file))?;
        }

        self.mv_if("test-enhanced-gen-files", "tests/test-enhanced-gen-files")?;

        Ok(())
    }

    fn handle_root_src(&mut self) -> Result<()> {
        println!("📦 Checking for root src/ directory...");

        if Path::new("src").exists() {
            println!("   Found src/ at root. Moving to apps/hooksmith...");
            fs::create_dir_all("apps/hooksmith")
                .context("Failed to create apps/hooksmith directory")?;

            self.mv_if("src", "apps/hooksmith/src")?;
            println!("   ⚠️  You may need to create apps/hooksmith/Cargo.toml manually");
        }

        Ok(())
    }

    fn check_build_rs(&self) -> Result<()> {
        println!("🔨 Checking for root build.rs...");

        if Path::new("build.rs").exists() {
            println!("   ⚠️  Found root build.rs. Consider moving it into the crate that needs it (e.g., crates/<name>/build.rs)");
        }

        Ok(())
    }

    fn get_moved_count(&self) -> usize {
        self.moved_count
    }
}

fn main() -> Result<()> {
    println!("🚀 Preparing Root Cleanup (Rust Edition)");
    println!("=====================================");

    let mut mover = FileMover::new();

    // Create necessary directories
    mover.create_directories()?;

    println!("📦 Moving files to their proper homes...");

    // Move files by category
    mover.move_reports_and_summaries()?;
    mover.move_hooksmith_artifacts()?;
    mover.move_worktree_and_ops()?;
    mover.move_generated_files()?;
    mover.move_docker_and_infra()?;
    mover.move_linguist_config()?;
    mover.move_test_files()?;
    mover.handle_root_src()?;
    mover.check_build_rs()?;

    println!("\n📊 Migration Summary:");
    println!("   Files moved: {}", mover.get_moved_count());
    println!("   Directories created: 12");

    println!("\n✅ Move plan applied (where files existed).");
    println!("📝 Review diffs before committing:");
    println!("   git status");
    println!("   git diff --cached");

    Ok(())
}
