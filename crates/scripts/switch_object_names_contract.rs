use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run --bin switch_object_names_contract <variant>");
        println!();
        println!("Available variants:");
        println!("  strict     - Strict compliance (original contract)");
        println!("  rust       - Rust workspace variant (recommended)");
        println!();
        println!("Examples:");
        println!("  cargo run --bin switch_object_names_contract rust");
        println!("  cargo run --bin switch_object_names_contract strict");
        std::process::exit(1);
    }

    let variant = &args[1];
    let contract_dir = Path::new("../contracts");

    let (source_file, description) = match variant.as_str() {
        "strict" => ("object-names@v1-strict.json", "Strict compliance contract"),
        "rust" => (
            "object-names@v1-rust-workspace-fixed.json",
            "Rust workspace contract",
        ),
        _ => {
            eprintln!("❌ Unknown variant: {}", variant);
            eprintln!("Available variants: strict, rust");
            std::process::exit(1);
        }
    };

    let source_path = contract_dir.join(source_file);
    let target_path = contract_dir.join("object-names@v1.json");

    if !source_path.exists() {
        eprintln!(
            "❌ Source contract file not found: {}",
            source_path.display()
        );
        std::process::exit(1);
    }

    // Backup current contract if it exists
    if target_path.exists() {
        let backup_path = contract_dir.join("object-names@v1.json.backup");
        fs::copy(&target_path, &backup_path)
            .context("Failed to create backup of current contract")?;
        println!("📋 Backed up current contract to {}", backup_path.display());
    }

    // Copy the selected variant
    fs::copy(&source_path, &target_path).context("Failed to copy contract file")?;

    println!("✅ Switched to {} contract", description);
    println!("📄 Active contract: {}", target_path.display());
    println!();
    println!("🔍 Next steps:");
    println!("  1. Run validation: cargo run --bin validate_object_names_contract");
    println!("  2. Fix any remaining issues");
    println!("  3. Commit the contract change");

    Ok(())
}
