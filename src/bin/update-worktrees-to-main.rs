use hooksmith::{
    get_worktrees, log_error, log_header, log_info, log_success, log_warning, run_git_command,
    run_git_command_in_dir,
};
use std::env;
use std::path::Path;
use std::process::Command;

fn update_worktree_to_main(worktree_path: &str, branch_name: &str) -> Result<bool, String> {
    log_info(&format!("Updating worktree: {}", branch_name));

    // Check if worktree exists
    if !Path::new(worktree_path).exists() {
        log_warning(&format!("Worktree does not exist: {}", worktree_path));
        return Err("Worktree does not exist".to_string());
    }

    // Check if already up to date
    let commits_behind =
        run_git_command_in_dir(&["rev-list", "--count", "HEAD..origin/main"], worktree_path)
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .unwrap_or(0);

    if commits_behind == 0 {
        log_info(&format!("Worktree {} is already up to date", branch_name));
        return Ok(true);
    }

    // Check if branch is merged
    let merged_branches =
        run_git_command_in_dir(&["branch", "--merged", "origin/main"], worktree_path)?;
    if merged_branches
        .lines()
        .any(|line| line.trim() == format!("* {}", branch_name))
    {
        log_info(&format!("Branch {} is merged - cleaning up", branch_name));

        // Remove worktree
        let output = Command::new("git")
            .args(&["worktree", "remove", worktree_path])
            .output()
            .map_err(|e| format!("Failed to remove worktree: {}", e))?;

        if output.status.success() {
            log_success(&format!("Removed merged worktree: {}", branch_name));
        }

        // Delete branch
        let output = Command::new("git")
            .args(&["branch", "-D", branch_name])
            .output()
            .map_err(|e| format!("Failed to delete branch: {}", e))?;

        if output.status.success() {
            log_success(&format!("Deleted merged branch: {}", branch_name));
        }

        return Ok(true);
    }

    // Try to rebase onto main
    log_info(&format!("Attempting to rebase {} onto main", branch_name));
    let output = Command::new("git")
        .args(&["rebase", "origin/main"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to rebase: {}", e))?;

    if output.status.success() {
        log_success(&format!("Successfully rebased {} onto main", branch_name));
        Ok(true)
    } else {
        log_warning(&format!(
            "Rebase failed for {} - creating fresh branch",
            branch_name
        ));

        // Remove old worktree and create fresh one
        let _ = Command::new("git")
            .args(&["worktree", "remove", worktree_path])
            .output();

        let _ = Command::new("git")
            .args(&["branch", "-D", branch_name])
            .output();

        // Create new worktree based on main
        let output = Command::new("git")
            .args(&["worktree", "add", worktree_path, "-b", branch_name])
            .output()
            .map_err(|e| format!("Failed to create fresh worktree: {}", e))?;

        if output.status.success() {
            log_success(&format!(
                "Created fresh worktree for {} based on main",
                branch_name
            ));
            Ok(true)
        } else {
            Err("Failed to create fresh worktree".to_string())
        }
    }
}

fn process_all_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    log_header("UPDATING ALL WORKTREES TO MAIN");
    println!();

    let worktrees = get_worktrees()?;
    let mut updated_count = 0;
    let mut total_count = 0;

    for worktree_path in &worktrees {
        // Skip the main worktree
        if worktree_path.contains("/hooksmith") && !worktree_path.contains("/worktrees/") {
            continue;
        }

        total_count += 1;

        // Get branch name from worktree path
        let branch_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        if update_worktree_to_main(worktree_path, &branch_name)? {
            updated_count += 1;
        }
    }

    println!();
    log_success(&format!(
        "Updated {} out of {} worktrees",
        updated_count, total_count
    ));

    Ok(())
}

fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    log_header("WORKTREE STATUS AFTER UPDATE");
    println!();

    // Try to run worktree-lifecycle status if available
    let lifecycle_script = "./worktree-lifecycle/bin/worktree-lifecycle.sh";
    if Path::new(lifecycle_script).exists() {
        let output = Command::new(lifecycle_script)
            .arg("status")
            .output()
            .map_err(|e| format!("Failed to run lifecycle script: {}", e))?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            // Fallback to git worktree list
            let worktrees = run_git_command(&["worktree", "list"])?;
            println!("{}", worktrees);
        }
    } else {
        // Fallback to git worktree list
        let worktrees = run_git_command(&["worktree", "list"])?;
        println!("{}", worktrees);
    }

    Ok(())
}

fn show_usage() {
    println!("Usage: update-worktrees-to-main [update|status|help]");
    println!();
    println!("Commands:");
    println!("  update  - Update all worktrees to be based on current main");
    println!("  status  - Show worktree status");
    println!("  help    - Show this usage information");
    println!();
    println!("Examples:");
    println!("  update-worktrees-to-main update  # Update all worktrees to main");
    println!("  update-worktrees-to-main status  # Show current status");
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let help = "help".to_string();
    let command = args.get(1).unwrap_or(&help);

    match command.as_str() {
        "update" => {
            process_all_worktrees()?;
            println!();
            show_status()?;
        }
        "status" => {
            show_status()?;
        }
        "help" | _ => {
            show_usage();
        }
    }

    Ok(())
}
