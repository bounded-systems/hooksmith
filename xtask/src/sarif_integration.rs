use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as TokioCommand;

use crate::structured_logging::StructuredEvent;

/// SARIF result structure for conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    pub rule_id: String,
    pub level: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub details: Option<Value>,
    pub tool: String,
    pub category: Option<String>,
}

/// SARIF document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDocument {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

/// SARIF run structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifRunResult>,
}

/// SARIF tool structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    pub driver: SarifToolComponent,
}

/// SARIF tool component structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifToolComponent {
    pub name: String,
    pub version: String,
}

/// SARIF run result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRunResult {
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
}

/// SARIF message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    pub text: String,
}

/// SARIF location structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

/// SARIF physical location structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

/// SARIF artifact location structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

/// SARIF region structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
    pub start_line: u32,
    pub start_column: u32,
}

/// CodeQL configuration
#[derive(Debug, Clone)]
pub struct CodeQLConfig {
    /// Path to CodeQL CLI
    pub cli_path: Option<String>,
    /// Database directory
    pub db_dir: PathBuf,
    /// Query suite to run
    pub query_suite: String,
    /// Language to analyze (cpp for Rust)
    pub language: String,
    /// Build command
    pub build_command: Vec<String>,
    /// Additional arguments
    pub additional_args: Vec<String>,
}

impl Default for CodeQLConfig {
    fn default() -> Self {
        Self {
            cli_path: None,
            db_dir: PathBuf::from("codeql-db"),
            query_suite: "codeql-cpp-queries:Security-and-quality.qls".to_string(),
            language: "cpp".to_string(),
            build_command: vec!["cargo".to_string(), "build".to_string()],
            additional_args: vec![],
        }
    }
}

/// SARIF integration for Hooksmith validation pipeline
pub struct SarifIntegration {
    config: Option<CodeQLConfig>,
}

impl SarifIntegration {
    /// Create a new SARIF integration instance
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Set CodeQL configuration
    pub fn with_config(mut self, config: CodeQLConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Convert JSONL events to SARIF format
    pub fn jsonl_to_sarif(&self, input: &Path) -> Result<String> {
        let file = std::fs::File::open(input)
            .context(format!("Failed to open input file: {}", input.display()))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            if !line.trim().is_empty() {
                let event: StructuredEvent = serde_json::from_str(&line)
                    .context(format!("Failed to parse JSONL line: {line}"))?;
                events.push(event);
            }
        }

        // Convert events to SARIF format
        let sarif_results: Vec<SarifRunResult> = events
            .iter()
            .filter_map(|event| {
                if event.action == "validation" {
                    let rule_id = event
                        .details
                        .as_ref()
                        .and_then(|d| d.get("rule_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let level = match event.level.as_str() {
                        "error" => "error",
                        "warning" => "warning",
                        "info" => "note",
                        _ => "note",
                    }
                    .to_string();

                    let locations = if let (Some(file), Some(line), Some(column)) = (
                        event
                            .details
                            .as_ref()
                            .and_then(|d| d.get("file"))
                            .and_then(|v| v.as_str()),
                        event
                            .details
                            .as_ref()
                            .and_then(|d| d.get("line"))
                            .and_then(|v| v.as_u64()),
                        event
                            .details
                            .as_ref()
                            .and_then(|d| d.get("column"))
                            .and_then(|v| v.as_u64()),
                    ) {
                        vec![SarifLocation {
                            physical_location: SarifPhysicalLocation {
                                artifact_location: SarifArtifactLocation {
                                    uri: file.to_string(),
                                },
                                region: SarifRegion {
                                    start_line: line as u32,
                                    start_column: column as u32,
                                },
                            },
                        }]
                    } else {
                        vec![]
                    };

                    Some(SarifRunResult {
                        rule_id,
                        level,
                        message: SarifMessage {
                            text: event.message.clone(),
                        },
                        locations,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Create SARIF document
        let sarif_doc = SarifDocument {
            schema: "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json"
                .to_string(),
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifToolComponent {
                        name: "Hooksmith".to_string(),
                        version: "0.1.0".to_string(),
                    },
                },
                results: sarif_results,
            }],
        };

        Ok(serde_json::to_string_pretty(&sarif_doc)?)
    }

    /// Convert SARIF to JSONL events
    pub fn sarif_to_jsonl(&self, input: &Path) -> Result<Vec<StructuredEvent>> {
        let sarif_content = std::fs::read_to_string(input)
            .context(format!("Failed to read SARIF file: {}", input.display()))?;

        let sarif_doc: SarifDocument =
            serde_json::from_str(&sarif_content).context("Failed to parse SARIF JSON")?;

        let mut events = Vec::new();
        let timestamp = Utc::now();

        for run in sarif_doc.runs {
            let tool_name = run.tool.driver.name;

            for result in run.results {
                let mut details = serde_json::Map::new();
                details.insert("rule_id".to_string(), Value::String(result.rule_id.clone()));
                details.insert("tool".to_string(), Value::String(tool_name.clone()));

                if let Some(location) = result.locations.first() {
                    details.insert(
                        "file".to_string(),
                        Value::String(location.physical_location.artifact_location.uri.clone()),
                    );
                    details.insert(
                        "line".to_string(),
                        Value::Number(serde_json::Number::from(
                            location.physical_location.region.start_line,
                        )),
                    );
                    details.insert(
                        "column".to_string(),
                        Value::Number(serde_json::Number::from(
                            location.physical_location.region.start_column,
                        )),
                    );
                }

                let event = StructuredEvent {
                    timestamp: timestamp.to_rfc3339(),
                    level: result.level,
                    action: "validation".to_string(),
                    message: result.message.text,
                    details: Some(Value::Object(details)),
                    file: None,
                    line: None,
                    metadata: None,
                };

                events.push(event);
            }
        }

        Ok(events)
    }

    /// Validate SARIF file
    pub fn validate_sarif(&self, file: &Path) -> Result<bool> {
        let content = std::fs::read_to_string(file)
            .context(format!("Failed to read SARIF file: {}", file.display()))?;

        // Basic validation - check if it's valid JSON and has required fields
        let sarif_doc: SarifDocument =
            serde_json::from_str(&content).context("Failed to parse SARIF JSON")?;

        // Check required fields
        if sarif_doc.schema.is_empty() || sarif_doc.version.is_empty() || sarif_doc.runs.is_empty()
        {
            return Ok(false);
        }

        Ok(true)
    }

    /// Run CodeQL analysis
    pub async fn run_codeql_analysis(&self) -> Result<Vec<StructuredEvent>> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CodeQL configuration not set"))?;

        // Check if CodeQL CLI is available
        let codeql_cmd = self.get_codeql_command()?;

        // Create database directory
        if !config.db_dir.exists() {
            std::fs::create_dir_all(&config.db_dir)
                .context("Failed to create CodeQL database directory")?;
        }

        // Create database
        self.create_codeql_database(config).await?;

        // Run analysis
        let sarif_output = self.run_codeql_analysis_command(config).await?;

        // Convert SARIF to events
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(&temp_file, sarif_output)?;

        let events = self.sarif_to_jsonl(temp_file.path())?;

        Ok(events)
    }

    /// Create CodeQL database
    async fn create_codeql_database(&self, config: &CodeQLConfig) -> Result<()> {
        let codeql_cmd = self.get_codeql_command()?;

        let mut cmd = TokioCommand::new(&codeql_cmd);
        cmd.args([
            "database",
            "create",
            "--language",
            &config.language,
            "--command",
            &config.build_command.join(" "),
            &config.db_dir.to_string_lossy(),
        ]);

        let output = cmd
            .output()
            .await
            .context("Failed to execute CodeQL database create command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("CodeQL database creation failed: {}", stderr);
        }

        Ok(())
    }

    /// Run CodeQL analysis command
    async fn run_codeql_analysis_command(&self, config: &CodeQLConfig) -> Result<String> {
        let codeql_cmd = self.get_codeql_command()?;

        let mut cmd = TokioCommand::new(&codeql_cmd);
        cmd.args([
            "database",
            "analyze",
            &config.db_dir.to_string_lossy(),
            &config.query_suite,
            "--format=sarif-latest",
        ]);

        let output = cmd
            .output()
            .await
            .context("Failed to execute CodeQL analysis command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("CodeQL analysis failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Get CodeQL command path
    fn get_codeql_command(&self) -> Result<String> {
        if let Some(config) = &self.config {
            if let Some(cli_path) = &config.cli_path {
                return Ok(cli_path.clone());
            }
        }

        // Try to find codeql in PATH
        let output = Command::new("which").arg("codeql").output();

        match output {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(path)
            }
            _ => {
                // Try common installation paths
                let common_paths = [
                    "/usr/local/bin/codeql",
                    "/opt/homebrew/bin/codeql",
                    "/usr/bin/codeql",
                ];

                for path in &common_paths {
                    if Path::new(path).exists() {
                        return Ok(path.to_string());
                    }
                }

                anyhow::bail!("CodeQL CLI not found. Install with: brew install codeql")
            }
        }
    }

    /// Merge multiple SARIF files
    pub fn merge_sarif_files(&self, files: &[PathBuf]) -> Result<String> {
        let mut all_runs = Vec::new();

        for file_path in files {
            let content = std::fs::read_to_string(file_path).context(format!(
                "Failed to read SARIF file: {}",
                file_path.display()
            ))?;

            let sarif_doc: SarifDocument = serde_json::from_str(&content).context(format!(
                "Failed to parse SARIF file: {}",
                file_path.display()
            ))?;

            all_runs.extend(sarif_doc.runs);
        }

        let merged_doc = SarifDocument {
            schema: "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json"
                .to_string(),
            version: "2.1.0".to_string(),
            runs: all_runs,
        };

        Ok(serde_json::to_string_pretty(&merged_doc)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_codeql_config_default() {
        let config = CodeQLConfig::default();
        assert_eq!(config.language, "cpp");
        assert_eq!(
            config.query_suite,
            "codeql-cpp-queries:Security-and-quality.qls"
        );
    }

    #[test]
    fn test_sarif_result_creation() {
        let result = SarifRunResult {
            rule_id: "test-rule".to_string(),
            level: "error".to_string(),
            message: SarifMessage {
                text: "Test message".to_string(),
            },
            locations: vec![],
        };

        assert_eq!(result.rule_id, "test-rule");
        assert_eq!(result.level, "error");
        assert_eq!(result.message.text, "Test message");
    }

    #[tokio::test]
    async fn test_sarif_integration_creation() {
        // Removed StructuredLogger - not implemented
        let integration = SarifIntegration::new();

        assert!(integration.config.is_none());
    }
}
