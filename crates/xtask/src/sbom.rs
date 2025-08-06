//! SBOM Generation Module
//!
//! This module provides functionality to generate Software Bill of Materials (SBOM)
//! in multiple formats including CycloneDX, SPDX, and raw Cargo metadata.

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
use tokio::fs;

/// SBOM generation formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    /// CycloneDX format (JSON)
    CycloneDx,
    /// SPDX format (JSON)
    Spdx,
    /// Raw Cargo metadata (JSON)
    CargoMetadata,
}

/// SBOM generator that supports multiple formats
pub struct SbomGenerator {
    output_dir: String,
}

impl SbomGenerator {
    /// Create a new SBOM generator
    pub fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
        }
    }

    /// Generate SBOM in the specified format
    pub async fn generate_sbom(&self, format: SbomFormat) -> Result<String> {
        match format {
            SbomFormat::CycloneDx => self.generate_cyclonedx_sbom().await,
            SbomFormat::Spdx => self.generate_spdx_sbom().await,
            SbomFormat::CargoMetadata => self.generate_cargo_metadata_sbom().await,
        }
    }

    /// Generate CycloneDX SBOM
    async fn generate_cyclonedx_sbom(&self) -> Result<String> {
        // Check if cyclonedx-bom is installed
        if !self.is_command_available("cyclonedx-bom").await? {
            return Err(anyhow!(
                "cyclonedx-bom not found. Install with: cargo install cyclonedx-bom"
            ));
        }

        let output_path = format!("{}/sbom.cyclonedx.json", self.output_dir);
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir).await?;

        // Run cyclonedx-bom
        let output = Command::new("cyclonedx-bom")
            .args(["-o", &output_path, "--format", "json"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to generate CycloneDX SBOM: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Read the generated file
        let content = fs::read_to_string(&output_path).await?;
        Ok(content)
    }

    /// Generate SPDX SBOM using protobom
    async fn generate_spdx_sbom(&self) -> Result<String> {
        // Check if protobom is installed
        if !self.is_command_available("protobom").await? {
            return Err(anyhow!(
                "protobom not found. Install with: go install github.com/github/protobom/cmd/protobom@latest"
            ));
        }

        let output_path = format!("{}/sbom.spdx.json", self.output_dir);
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir).await?;

        // Run protobom
        let output = Command::new("protobom")
            .args(["-f", "spdx", "-o", &output_path])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to generate SPDX SBOM: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Read the generated file
        let content = fs::read_to_string(&output_path).await?;
        Ok(content)
    }

    /// Generate raw Cargo metadata SBOM
    async fn generate_cargo_metadata_sbom(&self) -> Result<String> {
        let output_path = format!("{}/sbom.cargo-metadata.json", self.output_dir);
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir).await?;

        // Run cargo metadata
        let output = Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to generate Cargo metadata SBOM: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse and format the metadata
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        
        // Write to file
        let formatted_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&output_path, &formatted_json).await?;

        Ok(formatted_json)
    }

    /// Check if a command is available
    async fn is_command_available(&self, command: &str) -> Result<bool> {
        let output = Command::new("which")
            .arg(command)
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Generate all SBOM formats
    pub async fn generate_all_sboms(&self) -> Result<()> {
        println!("🔍 Generating SBOMs in multiple formats...");

        // Create output directory
        fs::create_dir_all(&self.output_dir).await?;

        // Generate each format
        let formats = [
            (SbomFormat::CargoMetadata, "Cargo Metadata"),
            (SbomFormat::CycloneDx, "CycloneDX"),
            (SbomFormat::Spdx, "SPDX"),
        ];

        for (format, name) in formats {
            match self.generate_sbom(format).await {
                Ok(_) => println!("✅ Generated {} SBOM", name),
                Err(e) => println!("⚠️  Failed to generate {} SBOM: {}", name, e),
            }
        }

        println!("📁 SBOMs saved to: {}", self.output_dir);
        Ok(())
    }

    /// Validate SBOM files
    pub async fn validate_sboms(&self) -> Result<()> {
        println!("🔍 Validating SBOM files...");

        let files = [
            "sbom.cargo-metadata.json",
            "sbom.cyclonedx.json",
            "sbom.spdx.json",
        ];

        for file in &files {
            let file_path = format!("{}/{}", self.output_dir, file);
            if Path::new(&file_path).exists() {
                match fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        match serde_json::from_str::<Value>(&content) {
                            Ok(_) => println!("✅ {} is valid JSON", file),
                            Err(e) => println!("❌ {} has invalid JSON: {}", file, e),
                        }
                    }
                    Err(e) => println!("❌ Failed to read {}: {}", file, e),
                }
            } else {
                println!("⚠️  {} not found", file);
            }
        }

        Ok(())
    }

    /// Generate SBOM report with statistics
    pub async fn generate_sbom_report(&self) -> Result<String> {
        let mut report = String::new();
        report.push_str("# SBOM Generation Report\n\n");

        // Check for each SBOM file
        let files = [
            ("sbom.cargo-metadata.json", "Cargo Metadata"),
            ("sbom.cyclonedx.json", "CycloneDX"),
            ("sbom.spdx.json", "SPDX"),
        ];

        for (file, name) in &files {
            let file_path = format!("{}/{}", self.output_dir, file);
            if Path::new(&file_path).exists() {
                let metadata = fs::metadata(&file_path).await?;
                let size_kb = metadata.len() as f64 / 1024.0;
                report.push_str(&format!("## {}\n", name));
                report.push_str(&format!("- File: `{}\`\n", file));
                report.push_str(&format!("- Size: {:.2} KB\n", size_kb));
                report.push_str(&format!("- Status: ✅ Generated\n\n"));
            } else {
                report.push_str(&format!("## {}\n", name));
                report.push_str(&format!("- File: `{}\`\n", file));
                report.push_str("Status: ❌ Not generated\n\n");
            }
        }

        // Add generation instructions
        report.push_str("## Generation Instructions\n\n");
        report.push_str("### Prerequisites\n\n");
        report.push_str("```bash\n");
        report.push_str("# Install CycloneDX generator\n");
        report.push_str("cargo install cyclonedx-bom\n\n");
        report.push_str("# Install SPDX generator (requires Go)\n");
        report.push_str("go install github.com/github/protobom/cmd/protobom@latest\n");
        report.push_str("```\n\n");
        report.push_str("### Generate SBOMs\n\n");
        report.push_str("```bash\n");
        report.push_str("# Generate all SBOM formats\n");
        report.push_str("cargo xtask sbom generate\n\n");
        report.push_str("# Generate specific format\n");
        report.push_str("cargo xtask sbom generate --format cyclonedx\n");
        report.push_str("cargo xtask sbom generate --format spdx\n");
        report.push_str("cargo xtask sbom generate --format cargo-metadata\n");
        report.push_str("```\n");

        Ok(report)
    }
}

/// SBOM command handler
pub async fn handle_sbom_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("Usage: cargo xtask sbom <command> [options]");
        println!("\nCommands:");
        println!("  generate    Generate SBOM files");
        println!("  validate    Validate existing SBOM files");
        println!("  report      Generate SBOM report");
        return Ok(());
    }

    let command = &args[0];
    let generator = SbomGenerator::new(".devcontracts/sbom");

    match command.as_str() {
        "generate" => {
            if args.len() > 1 && args[1] == "--format" {
                if args.len() < 3 {
                    println!("Error: --format requires a format argument");
                    return Ok(());
                }
                let format = match args[2].as_str() {
                    "cyclonedx" => SbomFormat::CycloneDx,
                    "spdx" => SbomFormat::Spdx,
                    "cargo-metadata" => SbomFormat::CargoMetadata,
                    _ => {
                        println!("Error: Unknown format. Use: cyclonedx, spdx, or cargo-metadata");
                        return Ok(());
                    }
                };
                generator.generate_sbom(format).await?;
                println!("✅ Generated SBOM in {:?} format", format);
            } else {
                generator.generate_all_sboms().await?;
            }
        }
        "validate" => {
            generator.validate_sboms().await?;
        }
        "report" => {
            let report = generator.generate_sbom_report().await?;
            println!("{}", report);
        }
        _ => {
            println!("Error: Unknown command '{}'", command);
            println!("Use: generate, validate, or report");
        }
    }

    Ok(())
} 
