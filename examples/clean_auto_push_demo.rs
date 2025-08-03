//! Clean Auto-Push Demo
//!
//! This example demonstrates the clean auto-push workflow that provides:
//! - Clean, user-friendly output
//! - Porcelain git push output parsing
//! - Comprehensive logging to files
//! - No confusing double push messages
//!
//! Usage:
//!   cargo run -p xtask -- clean-auto-push -m "feat: example commit"
//!   cargo run -p xtask -- clean-auto-push --verbose --watchdog --interval 60

// Demo script - no actual Command usage needed

fn main() {
    println!("🚀 Clean Auto-Push Demo");
    println!();
    println!("This demo shows the clean auto-push workflow that prevents confusing output.");
    println!();

    // Example 1: Basic clean auto-push
    println!("📦 Example 1: Basic clean auto-push");
    println!("   cargo run -p xtask -- clean-auto-push -m \"feat: example commit\"");
    println!();

    // Example 2: Verbose with logging
    println!("📦 Example 2: Verbose with logging");
    println!("   cargo run -p xtask -- clean-auto-push --verbose --log-file logs/my-push.log");
    println!();

    // Example 3: Watchdog mode
    println!("📦 Example 3: Watchdog mode");
    println!("   cargo run -p xtask -- clean-auto-push --watchdog --interval 60 --skip-validation");
    println!();

    // Example 4: Force push (use with caution)
    println!("📦 Example 4: Force push (use with caution)");
    println!("   cargo run -p xtask -- clean-auto-push --force -m \"feat: force update\"");
    println!();

    println!("✨ Key Benefits:");
    println!("   • Clean, single-line status messages");
    println!("   • No confusing git push output");
    println!("   • Comprehensive logging to .hooksmith/logs/");
    println!("   • Parsed porcelain output for better error messages");
    println!("   • Duration tracking and commit hash display");
    println!();

    println!("🔧 Try running one of the examples above!");
}
