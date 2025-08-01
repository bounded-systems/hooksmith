//! Utility functions for worktree CLI

/// Print success message
pub fn print_success(message: &str) {
    println!("✅ {}", message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("❌ {}", message);
}

/// Print info message
pub fn print_info(message: &str) {
    println!("ℹ️  {}", message);
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("⚠️  {}", message);
}

/// Print worktree-specific message
pub fn print_worktree(message: &str) {
    println!("🌳 {}", message);
}

/// Print safety-specific message
pub fn print_safety(message: &str) {
    println!("��️  {}", message);
} 
