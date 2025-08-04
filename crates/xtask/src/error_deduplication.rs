use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;

/// Global storage for seen error hashes
static ERROR_HASHES: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Normalize error output by removing run-specific details
pub fn normalize_error(err: &str) -> String {
    err.lines()
        .filter(|line| {
            !line.contains("Finished `dev` profile")
                && !line.contains("Blocking waiting for file lock")
                && !line.contains("note:")
                && !line.contains("warning:")
                && !line.contains("Compiling")
                && !line.contains("Checking")
                && !line.contains("Running")
                && !line.trim().is_empty()
        })
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a SHA256 hash of normalized error content
pub fn hash_error(normalized: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Record an error and return whether it's new
pub fn record_error(err: &str) -> bool {
    let normalized = normalize_error(err);
    if normalized.is_empty() {
        return false;
    }

    let hash = hash_error(&normalized);
    let mut seen = ERROR_HASHES.lock().unwrap();

    if seen.insert(hash.clone()) {
        println!("🔴 New error detected:");
        println!("{normalized}");
        println!("Hash: {hash}");
        println!();
        true
    } else {
        println!("⚠️ Duplicate error detected (hash={})", &hash[..8]);
        false
    }
}

/// Get statistics about recorded errors
pub fn get_error_stats() -> (usize, usize) {
    let seen = ERROR_HASHES.lock().unwrap();
    (seen.len(), seen.len())
}

/// Clear all recorded error hashes
pub fn clear_error_history() {
    let mut seen = ERROR_HASHES.lock().unwrap();
    seen.clear();
    println!("🧹 Error history cleared");
}

/// Process command output and handle errors
pub fn process_command_output(output: &std::process::Output, command_name: &str) -> bool {
    if output.status.success() {
        return true;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut has_errors = false;

    if !stderr.is_empty() {
        has_errors = record_error(&stderr) || has_errors;
    }

    if !stdout.is_empty() {
        // Sometimes errors are printed to stdout
        has_errors = record_error(&stdout) || has_errors;
    }

    if has_errors {
        println!("❌ {command_name} failed");
    }

    has_errors
}
