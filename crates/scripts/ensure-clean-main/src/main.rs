use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("cleanup");

    match command {
        "cleanup" => run_main_cleanup().await,
        "commit-all" => commit_all_worktrees().await,
        "push-all" => push_all_worktrees().await,
        "sync-all" => sync_all_worktrees().await,
        "cleanup-worktrees" => cleanup_worktrees().await,
        "remove-worktree" => {
            let worktree_name = args.get(2).cloned();
            remove_worktree(worktree_name).await
        }
        "help" => {
            println!("🧹 Hooksmith Worktree Management Commands");
            println!("==========================================");
            println!("cleanup           - Clean main branch and move changes to worktrees");
            println!("commit-all        - Commit changes in all worktrees");
            println!("push-all          - Push all worktree branches to remote");
            println!("sync-all          - Commit and push all worktrees");
            println!("cleanup-worktrees - Remove worktrees that are already integrated");
            println!(
                "remove-worktree   - Remove a specific worktree (usage: remove-worktree <name>)"
            );
            println!("help              - Show this help message");
            Ok(())
        }
        _ => {
            println!("❌ Unknown command: {}", command);
            println!("Use 'help' to see available commands");
            Ok(())
        }
    }
}

async fn run_main_cleanup() -> Result<()> {
    println!("🔍 Checking if main is ahead of origin/main...");

    // Get the main repository path (works from any worktree)
    let main_repo_path = get_main_repo_path()?;
    let current_dir = std::env::current_dir()?;

    // Check if we're in a worktree or main
    let is_worktree = current_dir != main_repo_path;
    let current_branch = get_current_branch()?;

    if is_worktree {
        println!("📍 Currently in worktree: {}", current_branch);
        println!("🔄 Switching to main repository for cleanup...");

        // Switch to main repository
        std::env::set_current_dir(&main_repo_path)?;
    }

    // Check if main is ahead of origin/main
    let ahead_commits = get_ahead_commits()?;

    // Check for uncommitted changes
    let uncommitted = get_uncommitted_changes()?;

    if ahead_commits.is_empty() && uncommitted.is_empty() {
        println!("✅ Main is clean and up to date with origin/main");
        return Ok(());
    }

    if !ahead_commits.is_empty() {
        println!(
            "⚠️  Main is ahead of origin/main by {} commit(s):",
            ahead_commits.len()
        );
        for commit in &ahead_commits {
            println!("   - {}", commit);
        }
    }

    if !uncommitted.is_empty() {
        println!("⚠️  Found uncommitted changes:");
        for change in &uncommitted {
            println!("   - {}", change);
        }
    }

    // Create timestamp for worktree name
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let branch_name = format!("fix/main-cleanup-{}", timestamp);

    println!("🔄 Creating worktree: {}", branch_name);

    // Create worktree
    create_worktree(&branch_name, &branch_name)?;

    // Switch to worktree
    println!("🔄 Switching to worktree...");
    switch_to_worktree(&branch_name)?;

    // Copy changes from main to worktree
    if !uncommitted.is_empty() {
        println!("🔄 Copying changes to worktree...");
        copy_changes_to_worktree(&main_repo_path)?;

        println!("🔄 Committing changes in worktree...");
        commit_changes(&branch_name)?;
    }

    // Switch back to main
    println!("🔄 Switching back to main...");
    switch_to_main(&main_repo_path)?;

    // Reset main to match origin/main
    println!("🔄 Resetting main to match origin/main...");
    reset_main()?;

    println!("✅ Main cleanup completed!");
    println!(
        "📁 Changes moved to worktree: worktree-{}",
        branch_name.replace('/', "-")
    );
    println!("🌿 Branch: {}", branch_name);

    // Return to original directory if we were in a worktree
    if is_worktree {
        std::env::set_current_dir(&current_dir)?;
        println!("📍 Returned to worktree: {}", current_branch);
    }

    Ok(())
}

fn get_main_repo_path() -> Result<std::path::PathBuf> {
    // Get the git root directory (works from any worktree)
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get git root directory")?;

    let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(git_root))
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn get_ahead_commits() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["log", "--oneline", "origin/main..HEAD"])
        .output()
        .context("Failed to get ahead commits")?;

    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(commits)
}

fn get_uncommitted_changes() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to get uncommitted changes")?;

    let changes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(changes)
}

fn create_worktree(branch_name: &str, worktree_name: &str) -> Result<()> {
    // First create the branch
    Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .context("Failed to create branch")?;

    // Create worktree
    let output = Command::new("cargo")
        .args([
            "xtask",
            "worktree",
            "create",
            "--branch",
            branch_name,
            "--switch",
        ])
        .output()
        .context("Failed to create worktree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree: {}", stderr);
    }

    Ok(())
}

fn switch_to_worktree(worktree_name: &str) -> Result<()> {
    // Convert branch name to worktree directory name (replace slashes with dashes)
    let worktree_dir = worktree_name.replace('/', "-");
    let worktree_path = format!("worktree-{}", worktree_dir);

    if !Path::new(&worktree_path).exists() {
        anyhow::bail!("Worktree directory not found: {}", worktree_path);
    }

    // Change to worktree directory
    std::env::set_current_dir(&worktree_path).context(format!(
        "Failed to change to worktree directory: {}",
        worktree_path
    ))?;

    Ok(())
}

fn commit_changes(branch_name: &str) -> Result<()> {
    // Add all changes
    Command::new("git")
        .args(["add", "."])
        .output()
        .context("Failed to add changes")?;

    // Commit changes
    let commit_message = format!("feat: Move main changes to worktree {}", branch_name);
    Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
        .context("Failed to commit changes")?;

    Ok(())
}

fn switch_to_main(main_repo_path: &std::path::Path) -> Result<()> {
    // Change back to main directory
    std::env::set_current_dir(main_repo_path).context("Failed to change back to main directory")?;

    // Switch to main branch
    Command::new("git")
        .args(["checkout", "main"])
        .output()
        .context("Failed to switch to main")?;

    Ok(())
}

fn copy_changes_to_worktree(main_repo_path: &std::path::Path) -> Result<()> {
    // Copy modified files from main to worktree
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(main_repo_path)
        .output()
        .context("Failed to get modified files")?;

    let modified_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    for file in &modified_files {
        let main_file = main_repo_path.join(file);
        let worktree_file = format!("./{}", file);

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&worktree_file).parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory for {}", worktree_file))?;
        }

        // Copy file
        std::fs::copy(&main_file, &worktree_file).context(format!(
            "Failed to copy {} to {}",
            main_file.display(),
            worktree_file
        ))?;
    }

    // Copy untracked files
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(main_repo_path)
        .output()
        .context("Failed to get untracked files")?;

    let untracked_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    for file in &untracked_files {
        let main_file = main_repo_path.join(file);
        let worktree_file = format!("./{}", file);

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&worktree_file).parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory for {}", worktree_file))?;
        }

        // Copy file
        std::fs::copy(&main_file, &worktree_file).context(format!(
            "Failed to copy {} to {}",
            main_file.display(),
            worktree_file
        ))?;
    }

    Ok(())
}

async fn commit_all_worktrees() -> Result<()> {
    println!("🔄 Committing changes in all worktrees...");

    let main_repo_path = get_main_repo_path()?;
    let worktrees = get_worktree_list()?;

    for worktree in &worktrees {
        let worktree_path = &worktree.path;
        let branch = &worktree.branch;

        println!(
            "📁 Processing worktree: {} (branch: {})",
            worktree_path, branch
        );

        // Check if worktree has changes
        let has_changes = check_worktree_changes(worktree_path)?;

        if has_changes {
            println!("   🔄 Committing changes...");
            commit_worktree_changes(worktree_path, branch)?;
        } else {
            println!("   ✅ No changes to commit");
        }
    }

    println!("✅ All worktrees processed!");
    Ok(())
}

async fn push_all_worktrees() -> Result<()> {
    println!("🔄 Pushing all worktree branches to remote...");

    let main_repo_path = get_main_repo_path()?;
    let worktrees = get_worktree_list()?;

    for worktree in &worktrees {
        let worktree_path = &worktree.path;
        let branch = &worktree.branch;

        println!(
            "📁 Pushing worktree: {} (branch: {})",
            worktree_path, branch
        );

        // Push the branch
        push_worktree_branch(worktree_path, branch)?;
    }

    println!("✅ All worktrees pushed!");
    Ok(())
}

async fn sync_all_worktrees() -> Result<()> {
    println!("🔄 Syncing all worktrees (commit + push)...");

    // First commit all changes
    commit_all_worktrees().await?;

    // Then push all branches
    push_all_worktrees().await?;

    println!("✅ All worktrees synced!");
    Ok(())
}

#[derive(Debug)]
struct WorktreeInfo {
    path: String,
    branch: String,
}

fn get_worktree_list() -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("Failed to get worktree list")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_worktree: Option<WorktreeInfo> = None;

    for line in output_str.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if let Some(worktree) = current_worktree.take() {
                worktrees.push(worktree);
            }

            // Start new worktree
            let path = line[9..].to_string(); // Remove "worktree " prefix
            current_worktree = Some(WorktreeInfo {
                path,
                branch: String::new(),
            });
        } else if line.starts_with("branch ") && current_worktree.is_some() {
            // Extract branch name and clean it up
            let branch = line[8..].to_string(); // Remove "branch " prefix
            let clean_branch = branch.replace("refs/heads/", "").replace("efs/heads/", "");
            if let Some(worktree) = &mut current_worktree {
                worktree.branch = clean_branch;
            }
        }
    }

    // Add the last worktree
    if let Some(worktree) = current_worktree {
        worktrees.push(worktree);
    }

    Ok(worktrees)
}

fn check_worktree_changes(worktree_path: &str) -> Result<bool> {
    // Check if worktree directory exists
    if !std::path::Path::new(worktree_path).exists() {
        println!(
            "   ⚠️  Worktree directory does not exist: {}",
            worktree_path
        );
        return Ok(false);
    }

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()
        .context(format!("Failed to check status in {}", worktree_path))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(!output_str.trim().is_empty())
}

fn commit_worktree_changes(worktree_path: &str, branch: &str) -> Result<()> {
    // Add all changes
    Command::new("git")
        .args(["add", "."])
        .current_dir(worktree_path)
        .output()
        .context(format!("Failed to add changes in {}", worktree_path))?;

    // Commit changes
    let commit_message = format!("feat: Update worktree {}", branch);
    Command::new("git")
        .args(["commit", "-m", &commit_message])
        .current_dir(worktree_path)
        .output()
        .context(format!("Failed to commit changes in {}", worktree_path))?;

    Ok(())
}

fn push_worktree_branch(worktree_path: &str, branch: &str) -> Result<()> {
    // Check if worktree directory exists
    if !std::path::Path::new(worktree_path).exists() {
        println!(
            "   ⚠️  Worktree directory does not exist: {}",
            worktree_path
        );
        return Ok(());
    }

    // Skip if branch is empty
    if branch.is_empty() {
        println!("   ⚠️  Skipping empty branch name");
        return Ok(());
    }

    Command::new("git")
        .args(["push", "origin", branch])
        .current_dir(worktree_path)
        .output()
        .context(format!(
            "Failed to push branch {} from {}",
            branch, worktree_path
        ))?;

    Ok(())
}

fn reset_main() -> Result<()> {
    Command::new("git")
        .args(["reset", "--hard", "origin/main"])
        .output()
        .context("Failed to reset main")?;

    Ok(())
}

async fn cleanup_worktrees() -> Result<()> {
    println!("🧹 Cleaning up worktrees that are already integrated...");

    let worktrees = get_worktree_list()?;
    let mut removed_count = 0;

    for worktree in &worktrees {
        let worktree_path = &worktree.path;
        let branch = &worktree.branch;

        // Skip main repository
        if worktree_path == "/Users/bobby/dev/repos/hooksmith" {
            continue;
        }

        // Check if worktree directory exists
        if !std::path::Path::new(worktree_path).exists() {
            println!("🗑️  Removing non-existent worktree: {}", worktree_path);
            remove_worktree_internal(worktree_path, branch)?;
            removed_count += 1;
            continue;
        }

        // Check if branch is already merged into main
        let is_merged = check_if_branch_merged(branch)?;
        if is_merged {
            println!(
                "✅ Removing merged worktree: {} (branch: {})",
                worktree_path, branch
            );
            remove_worktree_internal(worktree_path, branch)?;
            removed_count += 1;
        } else {
            println!(
                "⏳ Keeping worktree: {} (branch: {}) - not yet merged",
                worktree_path, branch
            );
        }
    }

    if removed_count > 0 {
        println!("✅ Cleaned up {} worktree(s)", removed_count);
    } else {
        println!("✅ No worktrees to clean up");
    }

    Ok(())
}

async fn remove_worktree(worktree_name: Option<String>) -> Result<()> {
    match worktree_name {
        Some(name) => {
            println!("🗑️  Removing worktree: {}", name);

            // Find the worktree by name
            let worktrees = get_worktree_list()?;
            let worktree = worktrees
                .iter()
                .find(|w| w.path.contains(&name) || w.branch.contains(&name));

            match worktree {
                Some(w) => {
                    remove_worktree_internal(&w.path, &w.branch)?;
                    println!("✅ Successfully removed worktree: {}", w.path);
                }
                None => {
                    println!("❌ Worktree not found: {}", name);
                    println!("Available worktrees:");
                    for w in &worktrees {
                        println!("  - {} (branch: {})", w.path, w.branch);
                    }
                }
            }
        }
        None => {
            println!("❌ Please specify a worktree name");
            println!("Usage: remove-worktree <name>");
        }
    }

    Ok(())
}

fn remove_worktree_internal(worktree_path: &str, branch: &str) -> Result<()> {
    let main_repo_path = get_main_repo_path()?;

    // Remove the worktree directory
    if std::path::Path::new(worktree_path).exists() {
        std::fs::remove_dir_all(worktree_path).context(format!(
            "Failed to remove worktree directory: {}",
            worktree_path
        ))?;
    }

    // Remove the worktree from git
    Command::new("git")
        .args(["worktree", "remove", worktree_path])
        .current_dir(&main_repo_path)
        .output()
        .context(format!(
            "Failed to remove worktree from git: {}",
            worktree_path
        ))?;

    // Delete the branch if it exists and is not main
    if !branch.is_empty() && branch != "main" {
        Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(&main_repo_path)
            .output()
            .context(format!("Failed to delete branch: {}", branch))?;
    }

    Ok(())
}

fn check_if_branch_merged(branch: &str) -> Result<bool> {
    if branch.is_empty() || branch == "main" {
        return Ok(false);
    }

    let output = Command::new("git")
        .args(["branch", "--merged", "main"])
        .output()
        .context("Failed to check merged branches")?;

    let merged_branches = String::from_utf8_lossy(&output.stdout);
    Ok(merged_branches.lines().any(|line| line.trim() == branch))
}
