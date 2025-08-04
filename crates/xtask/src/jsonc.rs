use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// JSONC configuration manager for handling JSONC files with comments
pub struct JsoncManager {
    /// Base directory for config files
    config_dir: PathBuf,
    /// Schema validation rules
    schemas: HashMap<String, serde_json::Value>,
}

/// JSONC file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsoncFile {
    /// File path
    pub path: PathBuf,
    /// Parsed JSON content (comments stripped)
    pub content: serde_json::Value,
    /// Original content with comments
    pub original_content: String,
    /// File metadata
    pub metadata: JsoncMetadata,
}

/// JSONC file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsoncMetadata {
    /// File type (e.g., "cargo", "config", "template")
    pub file_type: String,
    /// Target format for conversion (e.g., "toml", "yaml", "json")
    pub target_format: Option<String>,
    /// Whether this file is a template
    pub is_template: bool,
    /// Template variables
    pub template_vars: Option<HashMap<String, String>>,
    /// Validation schema (if any)
    pub schema: Option<String>,
}

impl JsoncManager {
    /// Create a new JSONC manager
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
            schemas: HashMap::new(),
        }
    }

    /// Load a JSONC file
    pub fn load_file(&self, filename: &str) -> Result<JsoncFile> {
        let file_path = self.config_dir.join(filename);
        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read JSONC file: {}", file_path.display()))?;

        // Strip comments and parse JSON
        let json_content = self.strip_comments(&content)?;
        let parsed_content: serde_json::Value = serde_json::from_str(&json_content)
            .with_context(|| format!("Failed to parse JSONC content: {}", file_path.display()))?;

        // Extract metadata from comments
        let metadata = self.extract_metadata(&content)?;

        Ok(JsoncFile {
            path: file_path,
            content: parsed_content,
            original_content: content,
            metadata,
        })
    }

    /// Load all JSONC files in the config directory
    pub fn load_all(&self) -> Result<Vec<JsoncFile>> {
        let mut files = Vec::new();

        if !self.config_dir.exists() {
            return Ok(files);
        }

        for entry in std::fs::read_dir(&self.config_dir).with_context(|| {
            format!(
                "Failed to read config directory: {}",
                self.config_dir.display()
            )
        })? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonc") {
                if let Ok(file) = self.load_file(path.file_name().unwrap().to_str().unwrap()) {
                    files.push(file);
                }
            }
        }

        Ok(files)
    }

    /// Convert JSONC to TOML
    pub fn to_toml(&self, jsonc_file: &JsoncFile) -> Result<String> {
        let toml_value: toml::Value = toml::Value::try_from(jsonc_file.content.clone())
            .context("Failed to convert JSONC to TOML")?;

        toml::to_string_pretty(&toml_value).context("Failed to serialize TOML")
    }

    /// Convert JSONC to YAML
    pub fn to_yaml(&self, jsonc_file: &JsoncFile) -> Result<String> {
        serde_yaml::to_string(&jsonc_file.content).context("Failed to convert JSONC to YAML")
    }

    /// Convert JSONC to JSON
    pub fn to_json(&self, jsonc_file: &JsoncFile) -> Result<String> {
        serde_json::to_string_pretty(&jsonc_file.content).context("Failed to convert JSONC to JSON")
    }

    /// Process template variables in JSONC content
    pub fn process_template(
        &self,
        jsonc_file: &JsoncFile,
        vars: &HashMap<String, String>,
    ) -> Result<JsoncFile> {
        if !jsonc_file.metadata.is_template {
            return Ok(jsonc_file.clone());
        }

        let mut processed_content = jsonc_file.original_content.clone();

        // Replace template variables
        for (key, value) in vars {
            let placeholder = format!("${{{key}}}");
            processed_content = processed_content.replace(&placeholder, value);
        }

        // Re-parse the processed content
        let json_content = self.strip_comments(&processed_content)?;
        let parsed_content: serde_json::Value =
            serde_json::from_str(&json_content).with_context(|| {
                format!(
                    "Failed to parse processed JSONC content: {}",
                    jsonc_file.path.display()
                )
            })?;

        Ok(JsoncFile {
            path: jsonc_file.path.clone(),
            content: parsed_content,
            original_content: processed_content,
            metadata: jsonc_file.metadata.clone(),
        })
    }

    /// Validate JSONC content against a schema
    pub fn validate_schema(
        &self,
        jsonc_file: &JsoncFile,
        schema: &serde_json::Value,
    ) -> Result<()> {
        use jsonschema::JSONSchema;

        // Leak the schema to get a 'static reference for JSONSchema::compile
        let static_schema: &'static serde_json::Value = Box::leak(Box::new(schema.clone()));
        let compiled_schema =
            JSONSchema::compile(static_schema).context("Failed to compile JSON schema")?;

        compiled_schema
            .validate(&jsonc_file.content)
            .map_err(|errors| {
                let error_messages: Vec<String> = errors
                    .map(|e| format!("{}: {}", e.instance_path, e))
                    .collect();
                anyhow::anyhow!("JSONC validation failed: {}", error_messages.join(", "))
            })?;

        Ok(())
    }

    /// Write processed JSONC to output file
    pub fn write_output(
        &self,
        jsonc_file: &JsoncFile,
        output_path: &Path,
        format: &str,
    ) -> Result<()> {
        let content = match format {
            "toml" => self.to_toml(jsonc_file)?,
            "yaml" | "yml" => self.to_yaml(jsonc_file)?,
            "json" => self.to_json(jsonc_file)?,
            _ => anyhow::bail!("Unsupported output format: {}", format),
        };

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        std::fs::write(output_path, content)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        Ok(())
    }

    /// Strip comments from JSONC content
    fn strip_comments(&self, content: &str) -> Result<String> {
        let mut result = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();

        while i < chars.len() {
            let ch = chars[i];

            if escape_next {
                result.push(ch);
                escape_next = false;
                i += 1;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                result.push(ch);
                i += 1;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                result.push(ch);
                i += 1;
                continue;
            }

            if !in_string {
                // Check for single-line comment
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    // Skip to end of line
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    if i < chars.len() {
                        result.push('\n'); // Keep newline
                    }
                    continue;
                }

                // Check for multi-line comment
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                    // Skip to end of comment
                    i += 2; // Skip /*
                    while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                        i += 1;
                    }
                    if i + 1 < chars.len() {
                        i += 2; // Skip */
                    }
                    continue;
                }
            }

            result.push(ch);
            i += 1;
        }

        Ok(result)
    }

    /// Extract metadata from JSONC comments
    fn extract_metadata(&self, content: &str) -> Result<JsoncMetadata> {
        let mut metadata = JsoncMetadata {
            file_type: "config".to_string(),
            target_format: None,
            is_template: false,
            template_vars: None,
            schema: None,
        };

        for line in content.lines() {
            let line = line.trim();

            // Look for metadata comments
            if line.starts_with("//") {
                let comment = line[2..].trim();

                if comment.starts_with("@type:") {
                    metadata.file_type = comment[6..].trim().to_string();
                } else if comment.starts_with("@format:") {
                    metadata.target_format = Some(comment[8..].trim().to_string());
                } else if comment.starts_with("@template") {
                    metadata.is_template = true;
                } else if comment.starts_with("@schema:") {
                    metadata.schema = Some(comment[8..].trim().to_string());
                }
            }
        }

        Ok(metadata)
    }

    /// Create a sample JSONC file
    pub fn create_sample(&self, filename: &str, file_type: &str) -> Result<()> {
        let file_path = self.config_dir.join(filename);

        // Ensure config directory exists
        std::fs::create_dir_all(&self.config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                self.config_dir.display()
            )
        })?;

        let sample_content = match file_type {
            "cargo" => self.create_cargo_sample(),
            "config" => self.create_config_sample(),
            "template" => self.create_template_sample(),
            _ => anyhow::bail!("Unknown file type: {}", file_type),
        };

        std::fs::write(&file_path, sample_content)
            .with_context(|| format!("Failed to write sample file: {}", file_path.display()))?;

        Ok(())
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&serde_json::Value> {
        self.schemas.get(name)
    }

    /// Add a schema
    pub fn add_schema(&mut self, name: String, schema: serde_json::Value) {
        self.schemas.insert(name, schema);
    }

    fn create_cargo_sample(&self) -> String {
        r#"// @type: cargo
// @format: toml
{
    "package": {
        "name": "example",
        "version": "0.1.0",
        "edition": "2021"
    },
    "dependencies": {
        "serde": { "version": "1.0", "features": ["derive"] },
        "tokio": { "version": "1.0", "features": ["full"] }
    }
}"#
        .to_string()
    }

    fn create_config_sample(&self) -> String {
        r#"// @type: config
// @format: yaml
{
    "app": {
        "name": "example-app",
        "version": "1.0.0"
    },
    "features": {
        "debug": true,
        "verbose": false
    }
}"#
        .to_string()
    }

    fn create_template_sample(&self) -> String {
        r#"// @type: template
// @format: json
// @template
{
    "name": "${PROJECT_NAME}",
    "version": "${VERSION}",
    "description": "${DESCRIPTION}",
    "author": "${AUTHOR}"
}"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_strip_comments() {
        let manager = JsoncManager::new(PathBuf::from("."));
        let jsonc = r#"{
            // This is a comment
            "key": "value", /* another comment */
            "number": 42
        }"#;

        let stripped = manager.strip_comments(jsonc).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&stripped).unwrap();

        assert_eq!(parsed["key"], "value");
        assert_eq!(parsed["number"], 42);
    }

    #[test]
    fn test_extract_metadata() {
        let manager = JsoncManager::new(PathBuf::from("."));
        let content = r#"// @type: cargo
// @format: toml
// @template
{
    "name": "test"
}"#;

        let metadata = manager.extract_metadata(content).unwrap();
        assert_eq!(metadata.file_type, "cargo");
        assert_eq!(metadata.target_format, Some("toml".to_string()));
        assert!(metadata.is_template);
    }

    #[test]
    fn test_jsonc_to_toml() {
        let manager = JsoncManager::new(PathBuf::from("."));
        let jsonc_content = r#"{
            "package": {
                "name": "test",
                "version": "0.1.0"
            }
        }"#;

        let jsonc_file = JsoncFile {
            path: PathBuf::from("test.jsonc"),
            content: serde_json::from_str(jsonc_content).unwrap(),
            original_content: jsonc_content.to_string(),
            metadata: JsoncMetadata {
                file_type: "cargo".to_string(),
                target_format: Some("toml".to_string()),
                is_template: false,
                template_vars: None,
                schema: None,
            },
        };

        let toml_content = manager.to_toml(&jsonc_file).unwrap();
        assert!(toml_content.contains("[package]"));
        assert!(toml_content.contains("name = \"test\""));
    }
}
