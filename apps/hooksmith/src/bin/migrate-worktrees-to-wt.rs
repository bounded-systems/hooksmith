use hooksmith::{get_worktrees, log_error, log_info, log_success, log_warning, run_git_command};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Worktree Migration Script");
    log_info("This script will move existing worktrees to the .wt directory");
    println!();

    // Check if we're in a git repository
    if !is_git_repository()? {
        log_error("Not in a git repository");
        std::process::exit(1);
    }

    // Create .wt directory if it doesn't exist
    fs::create_dir_all(".wt")?;

    // Run migration
    migrate_worktrees()?;

    println!();
    cleanup_old_directories()?;

    log_success("Migration script completed!");
    log_info("All worktrees should now be in the .wt directory");

    Ok(())
}

fn is_git_repository() -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()?;

    Ok(output.status.success())
}

fn migrate_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Starting worktree migration to .wt directory");

    let current_dir = env::current_dir()?;
    let worktrees = get_worktrees()?;
    let mut migrated_count = 0;
    let mut skipped_count = 0;

    for worktree_path in worktrees {
        let worktree_path = std::path::PathBuf::from(worktree_path);
        // Skip the main worktree
        if worktree_path == current_dir {
            log_info(&format!(
                "Skipping main worktree: {}",
                worktree_path.display()
            ));
            continue;
        }

        // Get branch name
        let branch_name = get_worktree_branch(&worktree_path)?;
        if branch_name.is_empty() {
            log_warning(&format!(
                "Could not determine branch name for worktree: {}",
                worktree_path.display()
            ));
            continue;
        }

        // Check if worktree is in the repository
        let repo_root = get_repo_root()?;
        if worktree_path.starts_with(&repo_root) {
            // Worktree is in repository, move it
            if move_worktree(&worktree_path, &branch_name)? {
                migrated_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            // Worktree is external
            handle_external_worktree(&worktree_path, &branch_name)?;
            skipped_count += 1;
        }
    }

    log_success("Migration complete!");
    log_info(&format!("Migrated: {} worktrees", migrated_count));
    log_info(&format!("Skipped: {} worktrees", skipped_count));

    Ok(())
}

fn get_worktree_branch(worktree_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Try to get branch from git worktree list
    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = worktree_list.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("worktree ") && line.contains(worktree_path.to_str().unwrap()) {
            if i + 1 < lines.len() && lines[i + 1].starts_with("branch ") {
                let branch_line = lines[i + 1];
                let branch = branch_line
                    .replace("branch ", "")
                    .replace("refs/heads/", "");
                return Ok(branch);
            }
        }
    }

    // Fallback: try to get branch from .git file content
    let git_file = worktree_path.join(".git");
    if git_file.exists() {
        let content = fs::read_to_string(&git_file)?;
        if let Some(git_ref) = content.lines().find(|line| line.contains("refs/heads/")) {
            let branch = git_ref.replace("refs/heads/", "").trim().to_string();
            if !branch.is_empty() {
                return Ok(branch);
            }
        }
    }

    Ok(String::new())
}

fn move_worktree(
    worktree_path: &Path,
    branch_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Create .wt directory if it doesn't exist
    fs::create_dir_all(".wt")?;

    // Determine new path in .wt directory
    let new_name = branch_name.replace("/", "-");
    let new_path = PathBuf::from(".wt").join(&new_name);

    log_info(&format!(
        "Moving worktree from {} to {}",
        worktree_path.display(),
        new_path.display()
    ));

    // Check if destination already exists
    if new_path.exists() {
        log_warning(&format!(
            "Destination {} already exists. Skipping.",
            new_path.display()
        ));
        return Ok(false);
    }

    // Move the worktree
    fs::rename(worktree_path, &new_path)?;
    log_success(&format!(
        "Successfully moved worktree to {}",
        new_path.display()
    ));

    // Update the worktree in git
    if let Ok(_) = run_git_command(&["worktree", "remove", worktree_path.to_str().unwrap()]) {
        log_info("Removed old worktree reference");
    }

    // Add the new worktree location
    if let Ok(_) = run_git_command(&["worktree", "add", new_path.to_str().unwrap(), branch_name]) {
        log_success("Added new worktree reference");
    } else {
        log_warning("Could not add new worktree reference, but files are moved");
    }

    Ok(true)
}

fn handle_external_worktree(
    worktree_path: &Path,
    branch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log_warning(&format!(
        "Found external worktree: {}",
        worktree_path.display()
    ));
    log_info("This worktree is outside the repository and cannot be moved automatically.");
    log_info("You may want to manually move it to .wt/{branch_name}");

    // Create a placeholder in .wt directory
    let new_name = branch_name.replace("/", "-");
    let new_path = PathBuf::from(".wt").join(&new_name);

    if !new_path.exists() {
        log_info(&format!(
            "Creating placeholder directory: {}",
            new_path.display()
        ));
        fs::create_dir_all(&new_path)?;

        let readme_content = format!(
            "# Placeholder for external worktree: {}\n# Original location: {}\n# Please move this worktree manually if needed",
            branch_name, worktree_path.display()
        );
        fs::write(new_path.join("README.md"), readme_content)?;
    }

    Ok(())
}

fn get_repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    let repo_root = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(repo_root))
}

fn cleanup_old_directories() -> Result<(), Box<dyn std::error::Error>> {
    log_info("Checking for old worktree directories...");

    let old_dirs = vec![
        "worktree-cleanup-remote-branches",
        "worktree-feat-systems-diagrams",
        "worktree-lifecycle",
    ];

    for dir in old_dirs {
        let dir_path = Path::new(dir);
        if dir_path.exists() && is_worktree(dir_path)? {
            log_warning(&format!("Found old worktree directory: {}", dir));
            log_info("This should have been moved by the migration script.");
            log_info("You can safely remove it if it's no longer needed.");
        }
    }

    Ok(())
}

fn is_worktree(dir: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(dir.is_dir() && dir.join(".git").exists())
}
