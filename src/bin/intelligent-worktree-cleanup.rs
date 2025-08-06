use std::process::Command;
use std::path::Path;
use hooksmith::{print_status, run_git_command, run_git_command_in_dir, get_worktrees, CleanupDecision};

fn analyze_worktree(worktree_path: &str) -> Result<CleanupDecision, String> {
    let worktree_name = Path::new(worktree_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    println!("=== ANALYZING: {} ===", worktree_name);
    
    if !Path::new(worktree_path).exists() {
        print_status("ERROR", &format!("Worktree {} does not exist", worktree_name));
        return Err("Worktree does not exist".to_string());
    }
    
    // Get branch name
    let branch = run_git_command_in_dir(&["branch", "--show-current"], worktree_path)?;
    print_status("INFO", &format!("Branch: {}", branch));
    
    // Check commit history
    let commit_count = run_git_command_in_dir(&["log", "--oneline", "--since=1 week ago"], worktree_path)
        .unwrap_or_else(|_| "".to_string())
        .lines()
        .count();
    let last_commit = run_git_command_in_dir(&["log", "--oneline", "-1"], worktree_path)
        .unwrap_or_else(|_| "".to_string());
    print_status("INFO", &format!("Recent commits: {}", commit_count));
    print_status("INFO", &format!("Last commit: {}", last_commit));
    
    // Check for conflicts
    let conflicts = run_git_command_in_dir(&["diff", "--name-only", "--diff-filter=U"], worktree_path)
        .unwrap_or_else(|_| "".to_string());
    let rebase_status = run_git_command_in_dir(&["status"], worktree_path)
        .unwrap_or_else(|_| "".to_string());
    
    if !conflicts.is_empty() || rebase_status.contains("rebase") {
        print_status("WARNING", "Worktree has conflicts or is in rebase state");
        
        // Check if this worktree is from old development
        if worktree_name.contains("202508") {
            print_status("DECISION", "This appears to be an old worktree from 202508");
            print_status("DECISION", "Recommendation: REMOVE (likely obsolete)");
            return Ok(CleanupDecision::Remove);
        }
    }
    
    // Check if branch is behind main
    let behind_count = run_git_command_in_dir(&["rev-list", "--count", "HEAD..origin/main"], worktree_path)
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    if behind_count > 5 {
        print_status("WARNING", &format!("Branch is {} commits behind main", behind_count));
        print_status("DECISION", "Recommendation: REMOVE (too far behind)");
        return Ok(CleanupDecision::Remove);
    }
    
    // Check if branch exists on origin
    let remote_check = run_git_command_in_dir(&["ls-remote", "--heads", "origin", &branch], worktree_path);
    if remote_check.is_ok() && !remote_check.unwrap().is_empty() {
        print_status("INFO", "Branch exists on origin");
        
        // Check if merged
        let merged_branches = run_git_command_in_dir(&["branch", "--merged", "origin/main"], worktree_path)?;
        if merged_branches.lines().any(|line| line.trim() == format!("* {}", branch)) {
            print_status("SUCCESS", "Branch is merged");
            print_status("DECISION", "Recommendation: CLEANUP (merged)");
            return Ok(CleanupDecision::Cleanup);
        } else {
            print_status("INFO", "Branch not merged");
            print_status("DECISION", "Recommendation: KEEP (active development)");
            return Ok(CleanupDecision::Keep);
        }
    } else {
        print_status("INFO", "Branch does not exist on origin");
        
        // Check if branch is merged locally
        let local_merged = run_git_command_in_dir(&["branch", "--merged", "main"], worktree_path)?;
        if local_merged.lines().any(|line| line.trim() == format!("* {}", branch)) {
            print_status("SUCCESS", "Branch is merged locally");
            print_status("DECISION", "Recommendation: CLEANUP (merged locally)");
            return Ok(CleanupDecision::Cleanup);
        } else {
            print_status("DECISION", "Recommendation: KEEP (local development)");
            return Ok(CleanupDecision::Keep);
        }
    }
}

fn cleanup_worktree(worktree_path: &str, decision: CleanupDecision) -> Result<bool, String> {
    let worktree_name = Path::new(worktree_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    match decision {
        CleanupDecision::Remove => {
            print_status("INFO", &format!("Removing worktree: {}", worktree_name));
            
            // Remove worktree
            let output = Command::new("git")
                .args(&["worktree", "remove", "--force", worktree_path])
                .output()
                .map_err(|e| format!("Failed to remove worktree: {}", e))?;
            
            if output.status.success() {
                print_status("SUCCESS", &format!("Removed worktree: {}", worktree_name));
                Ok(true)
            } else {
                print_status("ERROR", &format!("Failed to remove worktree: {}", worktree_name));
                Ok(false)
            }
        }
        CleanupDecision::Cleanup => {
            print_status("INFO", &format!("Cleaning up worktree: {}", worktree_name));
            
            // Get branch name
            let branch = run_git_command_in_dir(&["branch", "--show-current"], worktree_path)?;
            
            // Delete branch
            let output = Command::new("git")
                .args(&["branch", "-d", &branch])
                .output()
                .map_err(|e| format!("Failed to delete branch: {}", e))?;
            
            if output.status.success() {
                print_status("SUCCESS", &format!("Deleted branch: {}", branch));
            } else {
                print_status("WARNING", &format!("Failed to delete branch: {}", branch));
            }
            
            // Remove worktree
            let output = Command::new("git")
                .args(&["worktree", "remove", "--force", worktree_path])
                .output()
                .map_err(|e| format!("Failed to remove worktree: {}", e))?;
            
            if output.status.success() {
                print_status("SUCCESS", &format!("Removed worktree: {}", worktree_name));
                Ok(true)
            } else {
                print_status("ERROR", &format!("Failed to remove worktree: {}", worktree_name));
                Ok(false)
            }
        }
        CleanupDecision::Keep => {
            print_status("INFO", &format!("Keeping worktree: {}", worktree_name));
            Ok(true)
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 INTELLIGENT WORKTREE CLEANUP");
    println!("===============================");
    println!();
    
    let worktrees = get_worktrees()?;
    
    if worktrees.is_empty() {
        print_status("INFO", "No worktrees found");
        return Ok(());
    }
    
    let mut decisions = Vec::new();
    
    // Analyze each worktree
    for worktree_path in &worktrees {
        match analyze_worktree(worktree_path) {
            Ok(decision) => {
                decisions.push((worktree_path.clone(), decision));
            }
            Err(e) => {
                print_status("ERROR", &format!("Failed to analyze worktree: {}", e));
            }
        }
        println!();
    }
    
    // Summary
    let mut remove_count = 0;
    let mut cleanup_count = 0;
    let mut keep_count = 0;
    
    for (worktree_path, decision) in &decisions {
        match decision {
            CleanupDecision::Remove => remove_count += 1,
            CleanupDecision::Cleanup => cleanup_count += 1,
            CleanupDecision::Keep => keep_count += 1,
        }
    }
    
    println!("=== SUMMARY ===");
    println!("Remove: {}", remove_count);
    println!("Cleanup: {}", cleanup_count);
    println!("Keep: {}", keep_count);
    println!();
    
    // Execute decisions
    if !decisions.is_empty() {
        println!("=== EXECUTING DECISIONS ===");
        println!();
        
        for (worktree_path, decision) in &decisions {
            if let Err(e) = cleanup_worktree(worktree_path, decision.clone()) {
                print_status("ERROR", &format!("Failed to execute decision: {}", e));
            }
            println!();
        }
    }
    
    Ok(())
}
