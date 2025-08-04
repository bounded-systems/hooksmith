use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::Value;

#[derive(Debug)]
struct ComponentStatus {
    name: String,
    path: String,
    category: String,
    status: String,
    build_status: BuildStatus,
    version: Option<String>,
    errors: Vec<String>,
}

#[derive(Debug)]
enum BuildStatus {
    Success,
    Failed,
    NotBuilt,
    Building,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Hooksmith Component Status Report");
    println!("=====================================");
    println!("Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!();

    // Read component registry
    let registry_path = Path::new("config/component-registry.jsonc");
    if !registry_path.exists() {
        println!("❌ Component registry not found at: {}", registry_path.display());
        return Ok(());
    }

    let registry_content = std::fs::read_to_string(registry_path)?;
    let registry: Value = serde_json::from_str(&registry_content)?;

    // Check WIT Components
    println!("📦 WIT Components");
    println!("----------------");
    let wit_components = registry["wit_components"].as_array().unwrap_or(&vec![]);
    let mut wit_statuses = Vec::new();

    for component in wit_components {
        let name = component["name"].as_str().unwrap_or("unknown");
        let path = component["path"].as_str().unwrap_or("");
        let category = component["category"].as_str().unwrap_or("unknown");
        let status = component["status"].as_str().unwrap_or("unknown");

        let component_path = Path::new(path);
        let cargo_toml_path = component_path.join("Cargo.toml");
        let wit_path = component_path.join("wit").join(format!("{}.wit", name));

        let mut errors = Vec::new();
        let mut build_status = BuildStatus::NotBuilt;

        // Check if directory exists
        if !component_path.exists() {
            errors.push("Directory not found".to_string());
        } else {
            // Check if Cargo.toml exists
            if !cargo_toml_path.exists() {
                errors.push("Cargo.toml not found".to_string());
            }

            // Check if WIT file exists
            if !wit_path.exists() {
                errors.push("WIT file not found".to_string());
            }

            // Try to get version
            let version = get_crate_version(&cargo_toml_path).ok();

            // Try to check if it builds (simple check)
            if cargo_toml_path.exists() {
                match Command::new("cargo")
                    .args(["check", "--manifest-path", cargo_toml_path.to_str().unwrap()])
                    .output() {
                    Ok(output) => {
                        if output.status.success() {
                            build_status = BuildStatus::Success;
                        } else {
                            build_status = BuildStatus::Failed;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            errors.push(format!("Build failed: {}", stderr.lines().next().unwrap_or("Unknown error")));
                        }
                    }
                    Err(e) => {
                        build_status = BuildStatus::Failed;
                        errors.push(format!("Build error: {}", e));
                    }
                }
            }

            let status_icon = match build_status {
                BuildStatus::Success => "✅",
                BuildStatus::Failed => "❌",
                BuildStatus::NotBuilt => "⏸️",
                BuildStatus::Building => "🔄",
            };

            let version_str = get_crate_version(&cargo_toml_path).unwrap_or_else(|_| "—".to_string());
            
            println!("{:<20} {:<12} {:<10} {:<8} {:<8}",
                     name, category, status, status_icon, version_str);

            wit_statuses.push(ComponentStatus {
                name: name.to_string(),
                path: path.to_string(),
                category: category.to_string(),
                status: status.to_string(),
                build_status,
                version: get_crate_version(&cargo_toml_path).ok(),
                errors,
            });
        }
    }

    println!();

    // Check Native Crates
    println!("🦀 Native Crates");
    println!("---------------");
    let native_crates = registry["native_crates"].as_array().unwrap_or(&vec![]);
    let mut native_statuses = Vec::new();

    for crate_item in native_crates {
        let name = crate_item["name"].as_str().unwrap_or("unknown");
        let path = crate_item["path"].as_str().unwrap_or("");
        let category = crate_item["category"].as_str().unwrap_or("unknown");
        let status = crate_item["status"].as_str().unwrap_or("unknown");

        let crate_path = Path::new(path);
        let cargo_toml_path = crate_path.join("Cargo.toml");

        let mut errors = Vec::new();
        let mut build_status = BuildStatus::NotBuilt;

        // Check if directory exists
        if !crate_path.exists() {
            errors.push("Directory not found".to_string());
        } else {
            // Check if Cargo.toml exists
            if !cargo_toml_path.exists() {
                errors.push("Cargo.toml not found".to_string());
            }

            // Try to get version
            let version = get_crate_version(&cargo_toml_path).ok();

            // Try to check if it builds
            if cargo_toml_path.exists() {
                match Command::new("cargo")
                    .args(["check", "--manifest-path", cargo_toml_path.to_str().unwrap()])
                    .output() {
                    Ok(output) => {
                        if output.status.success() {
                            build_status = BuildStatus::Success;
                        } else {
                            build_status = BuildStatus::Failed;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            errors.push(format!("Build failed: {}", stderr.lines().next().unwrap_or("Unknown error")));
                        }
                    }
                    Err(e) => {
                        build_status = BuildStatus::Failed;
                        errors.push(format!("Build error: {}", e));
                    }
                }
            }

            let status_icon = match build_status {
                BuildStatus::Success => "✅",
                BuildStatus::Failed => "❌",
                BuildStatus::NotBuilt => "⏸️",
                BuildStatus::Building => "🔄",
            };

            let version_str = get_crate_version(&cargo_toml_path).unwrap_or_else(|_| "—".to_string());
            
            println!("{:<20} {:<12} {:<10} {:<8} {:<8}",
                     name, category, status, status_icon, version_str);

            native_statuses.push(ComponentStatus {
                name: name.to_string(),
                path: path.to_string(),
                category: category.to_string(),
                status: status.to_string(),
                build_status,
                version: get_crate_version(&cargo_toml_path).ok(),
                errors,
            });
        }
    }

    println!();

    // Summary
    println!("📊 Summary");
    println!("----------");
    let total_components = wit_statuses.len() + native_statuses.len();
    let successful_builds = wit_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Success)).count() +
                           native_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Success)).count();
    let failed_builds = wit_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Failed)).count() +
                       native_statuses.iter().filter(|s| matches!(s.build_status, BuildStatus::Failed)).count();

    println!("Total components: {}", total_components);
    println!("✅ Successful builds: {}", successful_builds);
    println!("❌ Failed builds: {}", failed_builds);
    
    let success_rate = if total_components > 0 {
        (successful_builds as f64 / total_components as f64) * 100.0
    } else {
        0.0
    };
    println!("📈 Success rate: {:.1}%", success_rate);

    Ok(())
}

fn get_crate_version(cargo_toml_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(cargo_toml_path)?;
    
    // Simple version extraction from Cargo.toml
    for line in content.lines() {
        if line.trim().starts_with("version = ") {
            let version = line.split('"').nth(1).unwrap_or("unknown");
            return Ok(version.to_string());
        }
    }
    
    Err("Version not found".into())
} 
