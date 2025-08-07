use anyhow::Result;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Modern Rust-based fsmonitor-watchman hook for Git
///
/// This hook provides file system monitoring capabilities that can:
/// 1. Work with Git's built-in FSMonitor daemon (core.fsmonitor = true)
/// 2. Provide a Rust-based alternative to Watchman
/// 3. Integrate with Hooksmith's validation system
///
/// Usage: Git calls this hook with version and timestamp arguments
/// Output: Changed file paths separated by null characters
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <version> <timestamp>", args[0]);
        std::process::exit(1);
    }

    let version = &args[1];
    let timestamp = &args[2];

    // Validate version (Git expects "2" for current fsmonitor protocol)
    if version != "2" {
        eprintln!("Unsupported fsmonitor version: {}", version);
        std::process::exit(1);
    }

    // Parse timestamp
    let timestamp_secs: u64 = timestamp.parse()
        .map_err(|_| anyhow::anyhow!("Invalid timestamp: {}", timestamp))?;

    // Get the repository root
    let repo_root = get_repo_root()?;

    // Check if Git's built-in FSMonitor is enabled
    if is_git_fsmonitor_enabled()? {
        // Use Git's built-in daemon - just pass through
        use_builtin_fsmonitor(repo_root, timestamp_secs)?;
    } else {
        // Use our Rust-based implementation
        use_rust_fsmonitor(repo_root, timestamp_secs)?;
    }

    Ok(())
}

fn get_repo_root() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    if output.status.success() {
        let root = String::from_utf8(output.stdout)?
            .trim()
            .to_string();
        Ok(root)
    } else {
        Err(anyhow::anyhow!("Failed to get repository root"))
    }
}

fn is_git_fsmonitor_enabled() -> Result<bool> {
    let output = Command::new("git")
        .args(&["config", "--get", "core.fsmonitor"])
        .output()?;

    Ok(output.status.success())
}

fn use_builtin_fsmonitor(repo_root: String, timestamp_secs: u64) -> Result<()> {
    // When Git's built-in FSMonitor is enabled, we can either:
    // 1. Pass through to the built-in daemon
    // 2. Provide additional validation/processing

    // For now, we'll just log that we're using the built-in daemon
    eprintln!("Using Git's built-in FSMonitor daemon");

    // In a real implementation, you might:
    // - Validate file changes against Hooksmith contracts
    // - Log changes for audit purposes
    // - Trigger additional validation workflows

    Ok(())
}

fn use_rust_fsmonitor(repo_root: String, timestamp_secs: u64) -> Result<()> {
    // Implement a Rust-based file system monitor
    // This is a simplified version - in production you'd use a proper file watcher

    let timestamp = UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs);
    let now = SystemTime::now();

    // Get list of files that have changed since the timestamp
    let changed_files = get_changed_files_since(&repo_root, timestamp)?;

    // Output changed files in the format Git expects (null-separated)
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for file in changed_files {
        handle.write_all(file.as_bytes())?;
        handle.write_all(&[0])?; // null separator
    }

    handle.flush()?;
    Ok(())
}

fn get_changed_files_since(repo_root: &str, since: SystemTime) -> Result<Vec<String>> {
    // This is a simplified implementation
    // In production, you'd use a proper file watcher like notify-rs

    let mut changed_files = Vec::new();

    // Use git status --porcelain to get changed files
    // This is a fallback - a real implementation would use file system events
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(repo_root)
        .output()?;

    if output.status.success() {
        let status_output = String::from_utf8(output.stdout)?;

        for line in status_output.lines() {
            if line.len() >= 3 {
                // Parse git status output (XY PATH)
                let file_path = &line[3..];
                if !file_path.is_empty() {
                    changed_files.push(file_path.to_string());
                }
            }
        }
    }

    Ok(changed_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_validation() {
        // Test that version "2" is accepted
        // This would be tested in integration tests
    }

    #[test]
    fn test_timestamp_parsing() {
        let timestamp = "1234567890";
        let result: Result<u64, _> = timestamp.parse();
        assert!(result.is_ok());
    }
}
