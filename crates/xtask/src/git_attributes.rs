use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Git attribute entry with pattern and attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAttribute {
    /// File pattern (e.g., "*.rs", "docs/**")
    pub pattern: String,
    /// Attributes map (e.g., {"text": true, "diff": "contract_json"})
    pub attributes: HashMap<String, String>,
    /// Comments associated with this attribute
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<String>,
    /// Line number in original file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
}

/// Complete Git attributes structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAttributes {
    /// All attribute entries
    pub attributes: Vec<GitAttribute>,
    /// Global comments at the top of the file
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_comments: Vec<String>,
    /// File metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<GitAttributesMetadata>,
}

/// Metadata about the .gitattributes file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAttributesMetadata {
    /// Whether the file is generated
    pub generated: bool,
    /// Generator tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    /// Checksum if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Generation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
}

/// Categories for organizing Git attributes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttributeCategory {
    /// Text/binary handling
    TextHandling,
    /// Diff drivers
    DiffDriver,
    /// Merge strategies
    MergeStrategy,
    /// Export filtering
    ExportFilter,
    /// Linguist overrides (GitHub)
    Linguist,
    /// Custom attributes
    Custom,
    /// Other/unclassified
    Other,
}

impl GitAttributes {
    /// Parse a .gitattributes file into structured format
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    /// Parse Git attributes content from string
    pub fn from_string(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut attributes = Vec::new();
        let mut global_comments = Vec::new();
        let mut current_comments = Vec::new();
        let mut metadata = None;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Handle comments
            if trimmed.starts_with('#') {
                let comment = trimmed[1..].trim().to_string();

                // Check for metadata comments
                if comment.contains("@generated") || comment.contains("@checksum") {
                    metadata = Some(GitAttributesMetadata {
                        generated: true,
                        generator: comment.split("by").nth(1).map(|s| s.trim().to_string()),
                        checksum: comment
                            .split("@checksum:")
                            .nth(1)
                            .map(|s| s.trim().to_string()),
                        generated_at: None, // Could parse from comment if available
                    });
                }

                if attributes.is_empty() {
                    global_comments.push(comment);
                } else {
                    current_comments.push(comment);
                }
                continue;
            }

            // Handle attribute lines
            if !trimmed.starts_with('#') && !trimmed.is_empty() {
                if let Some((pattern, attrs)) = Self::parse_attribute_line(trimmed) {
                    let mut attributes_map = HashMap::new();

                    for attr in attrs {
                        if let Some((key, value)) = Self::parse_attribute(&attr) {
                            attributes_map.insert(key, value);
                        }
                    }

                    attributes.push(GitAttribute {
                        pattern,
                        attributes: attributes_map,
                        comments: current_comments.clone(),
                        line_number: Some(line_num + 1),
                    });
                    current_comments.clear();
                }
            }
        }

        Ok(GitAttributes {
            attributes,
            global_comments,
            metadata,
        })
    }

    /// Parse an attribute line into pattern and attributes
    fn parse_attribute_line(line: &str) -> Option<(String, Vec<String>)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let pattern = parts[0].to_string();
        let attributes = parts[1..].to_vec();

        Some((
            pattern,
            attributes.into_iter().map(|s| s.to_string()).collect(),
        ))
    }

    /// Parse a single attribute into key-value pair
    fn parse_attribute(attr: &str) -> Option<(String, String)> {
        if attr.contains('=') {
            let parts: Vec<&str> = attr.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        } else {
            // Boolean attribute (e.g., "text", "binary")
            Some((attr.to_string(), "true".to_string()))
        }
    }

    /// Convert to JSONC format with proper structure and documentation
    pub fn to_jsonc(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut jsonc = serde_json::to_string_pretty(&self)?;

        // Add schema and documentation
        let schema = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Attributes Configuration",
  "description": "Structured Git attributes converted from .gitattributes to JSONC format",
  "type": "object",
  "properties": {
    "attributes": {
      "type": "array",
      "description": "Git attribute entries",
      "items": {
        "type": "object",
        "properties": {
          "pattern": {
            "type": "string",
            "description": "File pattern (e.g., *.rs, docs/**)"
          },
          "attributes": {
            "type": "object",
            "description": "Attribute key-value pairs"
          },
          "comments": {
            "type": "array",
            "description": "Comments associated with this attribute",
            "items": {
              "type": "string"
            }
          },
          "line_number": {
            "type": "integer",
            "description": "Line number in original file"
          }
        }
      }
    },
    "global_comments": {
      "type": "array",
      "description": "Global comments at the top of the file",
      "items": {
        "type": "string"
      }
    },
    "metadata": {
      "type": "object",
      "description": "File metadata",
      "properties": {
        "generated": {
          "type": "boolean",
          "description": "Whether the file is generated"
        },
        "generator": {
          "type": "string",
          "description": "Generator tool name"
        },
        "checksum": {
          "type": "string",
          "description": "File checksum"
        },
        "generated_at": {
          "type": "string",
          "description": "Generation timestamp"
        }
      }
    }
  }
}"#;

        // Replace the opening brace with schema and documentation
        jsonc = jsonc.replacen("{", &format!("{}{{", schema), 1);

        Ok(jsonc)
    }

    /// Categorize attributes by their purpose
    pub fn categorize_attributes(&self) -> HashMap<AttributeCategory, Vec<&GitAttribute>> {
        let mut categories = HashMap::new();

        for attr in &self.attributes {
            let category =
                if attr.attributes.contains_key("text") || attr.attributes.contains_key("binary") {
                    AttributeCategory::TextHandling
                } else if attr.attributes.contains_key("diff") {
                    AttributeCategory::DiffDriver
                } else if attr.attributes.contains_key("merge") {
                    AttributeCategory::MergeStrategy
                } else if attr.attributes.contains_key("export-ignore")
                    || attr.attributes.contains_key("export-subst")
                {
                    AttributeCategory::ExportFilter
                } else if attr.attributes.contains_key("linguist") {
                    AttributeCategory::Linguist
                } else if attr.attributes.contains_key("custom")
                    || attr.attributes.contains_key("contract")
                {
                    AttributeCategory::Custom
                } else {
                    AttributeCategory::Other
                };

            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(attr);
        }

        categories
    }

    /// Generate a summary of the attributes
    pub fn generate_summary(&self) -> String {
        let categories = self.categorize_attributes();
        let mut summary = String::new();

        summary.push_str("# Git Attributes Summary\n\n");

        if let Some(metadata) = &self.metadata {
            summary.push_str(&format!("## File Metadata\n"));
            summary.push_str(&format!("- Generated: {}\n", metadata.generated));
            if let Some(generator) = &metadata.generator {
                summary.push_str(&format!("- Generator: {}\n", generator));
            }
            if let Some(checksum) = &metadata.checksum {
                summary.push_str(&format!("- Checksum: {}\n", checksum));
            }
            summary.push_str("\n");
        }

        summary.push_str(&format!(
            "## Total Attributes: {}\n\n",
            self.attributes.len()
        ));

        for (category, attrs) in &categories {
            if !attrs.is_empty() {
                summary.push_str(&format!("### {:?} ({} entries)\n", category, attrs.len()));
                for attr in attrs {
                    summary.push_str(&format!(
                        "- **{}**: {} attributes\n",
                        attr.pattern,
                        attr.attributes.len()
                    ));
                }
                summary.push_str("\n");
            }
        }

        summary
    }

    /// Validate the attributes structure
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for attr in &self.attributes {
            // Check for valid patterns
            if attr.pattern.is_empty() {
                errors.push(format!(
                    "Empty pattern at line {}",
                    attr.line_number.unwrap_or(0)
                ));
            }

            // Check for valid attributes
            for (key, _value) in &attr.attributes {
                if key.is_empty() {
                    errors.push(format!(
                        "Empty attribute key for pattern {} at line {}",
                        attr.pattern,
                        attr.line_number.unwrap_or(0)
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Export to different formats
    pub fn export(&self, format: &str) -> Result<String, Box<dyn std::error::Error>> {
        match format {
            "jsonc" => self.to_jsonc(),
            "json" => Ok(serde_json::to_string_pretty(&self)?),
            "yaml" => {
                let yaml = serde_yaml::to_string(&self)?;
                Ok(yaml)
            }
            "toml" => {
                let toml = toml::to_string_pretty(&self)?;
                Ok(toml)
            }
            "gitattributes" => self.to_gitattributes(),
            _ => Err("Unsupported format".into()),
        }
    }

    /// Convert back to .gitattributes format
    pub fn to_gitattributes(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();

        // Add global comments
        for comment in &self.global_comments {
            output.push_str(&format!("# {}\n", comment));
        }

        if !self.global_comments.is_empty() {
            output.push('\n');
        }

        // Add attributes
        for attr in &self.attributes {
            // Add comments
            for comment in &attr.comments {
                output.push_str(&format!("# {}\n", comment));
            }

            // Add pattern and attributes
            let mut attr_parts = vec![attr.pattern.clone()];
            for (key, value) in &attr.attributes {
                if value == "true" {
                    attr_parts.push(key.clone());
                } else {
                    attr_parts.push(format!("{}={}", key, value));
                }
            }

            output.push_str(&format!("{}\n", attr_parts.join(" ")));
        }

        Ok(output)
    }

    /// Find attributes that match a specific file path
    pub fn find_matching_attributes(&self, file_path: &str) -> Vec<&GitAttribute> {
        let mut matches = Vec::new();

        for attr in &self.attributes {
            if Self::pattern_matches(&attr.pattern, file_path) {
                matches.push(attr);
            }
        }

        matches
    }

    /// Check if a pattern matches a file path
    fn pattern_matches(pattern: &str, file_path: &str) -> bool {
        // Simple glob pattern matching
        // This is a basic implementation - could be enhanced with proper glob matching
        if pattern.contains("**") {
            // Recursive pattern — check before single `*` since `**` also
            // contains `*`.
            let pattern_parts: Vec<&str> = pattern.split("**").collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                file_path.starts_with(prefix) && file_path.ends_with(suffix)
            } else {
                false
            }
        } else if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                file_path.starts_with(prefix) && file_path.ends_with(suffix)
            } else {
                false
            }
        } else {
            // Exact match
            file_path == pattern
        }
    }
}

/// Command-line interface for Git attributes management
pub struct GitAttributesManager;

impl GitAttributesManager {
    /// Convert .gitattributes to JSONC
    pub fn convert_to_jsonc<P: AsRef<Path>>(
        input_path: P,
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = GitAttributes::from_file(input_path)?;

        // Validate the attributes
        if let Err(errors) = attributes.validate() {
            eprintln!("Validation warnings:");
            for error in errors {
                eprintln!("  - {}", error);
            }
        }

        // Generate JSONC
        let jsonc = attributes.to_jsonc()?;
        fs::write(output_path, jsonc)?;

        println!("✅ Git attributes converted to JSONC successfully");
        Ok(())
    }

    /// Generate a comprehensive attributes template
    pub fn generate_template<P: AsRef<Path>>(
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = GitAttributes {
            global_comments: vec![
                "Git Attributes Template".to_string(),
                "This template shows common Git attribute patterns".to_string(),
            ],
            metadata: Some(GitAttributesMetadata {
                generated: true,
                generator: Some("git-attributes-manager".to_string()),
                checksum: None,
                generated_at: None,
            }),
            attributes: vec![
                // Text handling
                GitAttribute {
                    pattern: "*.sh".to_string(),
                    attributes: HashMap::from([
                        ("text".to_string(), "true".to_string()),
                        ("eol".to_string(), "lf".to_string()),
                    ]),
                    comments: vec!["Shell scripts with LF line endings".to_string()],
                    line_number: None,
                },
                // Binary files
                GitAttribute {
                    pattern: "*.jpg".to_string(),
                    attributes: HashMap::from([
                        ("binary".to_string(), "true".to_string()),
                        ("-diff".to_string(), "true".to_string()),
                        ("-merge".to_string(), "true".to_string()),
                    ]),
                    comments: vec!["Binary image files".to_string()],
                    line_number: None,
                },
                // Custom diff driver
                GitAttribute {
                    pattern: "*.json".to_string(),
                    attributes: HashMap::from([("diff".to_string(), "json".to_string())]),
                    comments: vec!["JSON files with custom diff driver".to_string()],
                    line_number: None,
                },
                // Merge strategy
                GitAttribute {
                    pattern: "*.lock".to_string(),
                    attributes: HashMap::from([("merge".to_string(), "union".to_string())]),
                    comments: vec!["Lock files with union merge strategy".to_string()],
                    line_number: None,
                },
                // Export filtering
                GitAttribute {
                    pattern: "secret.key".to_string(),
                    attributes: HashMap::from([("export-ignore".to_string(), "true".to_string())]),
                    comments: vec!["Exclude from git archive".to_string()],
                    line_number: None,
                },
                // Linguist overrides
                GitAttribute {
                    pattern: "*.ts".to_string(),
                    attributes: HashMap::from([(
                        "linguist-language".to_string(),
                        "TypeScript".to_string(),
                    )]),
                    comments: vec!["TypeScript files for GitHub".to_string()],
                    line_number: None,
                },
            ],
        };

        let jsonc = template.to_jsonc()?;
        fs::write(output_path, jsonc)?;

        println!("✅ Git attributes template generated successfully");
        Ok(())
    }

    /// Analyze current Git attributes
    pub fn analyze<P: AsRef<Path>>(input_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = GitAttributes::from_file(input_path)?;

        println!("{}", attributes.generate_summary());

        let categories = attributes.categorize_attributes();
        println!("## Detailed Analysis\n");

        for (category, attrs) in &categories {
            if !attrs.is_empty() {
                println!("### {:?} ({} entries)", category, attrs.len());
                for attr in attrs {
                    println!("- **{}**:", attr.pattern);
                    for (key, value) in &attr.attributes {
                        println!("  {} = {}", key, value);
                    }
                    println!();
                }
            }
        }

        Ok(())
    }

    /// Test attribute matching
    pub fn test_matching<P: AsRef<Path>>(
        input_path: P,
        test_files: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = GitAttributes::from_file(input_path)?;

        println!("## Attribute Matching Test\n");

        for file_path in test_files {
            let matches = attributes.find_matching_attributes(&file_path);
            println!("**{}**:", file_path);
            if matches.is_empty() {
                println!("  No matching attributes");
            } else {
                for attr in matches {
                    println!("  - Pattern: {}", attr.pattern);
                    for (key, value) in &attr.attributes {
                        println!("    {} = {}", key, value);
                    }
                }
            }
            println!();
        }

        Ok(())
    }
}

/// Parse Git attributes file
pub fn parse_git_attributes(_input: &str) -> anyhow::Result<serde_json::Value> {
    // Stub implementation
    Ok(serde_json::json!({}))
}

/// Convert Git attributes to JSONC format
pub fn convert_to_jsonc(_attributes: &serde_json::Value) -> anyhow::Result<String> {
    // Stub implementation
    Ok("{}".to_string())
}

/// Load Git attributes schema
pub fn load_schema() -> anyhow::Result<serde_json::Value> {
    // Stub implementation
    Ok(serde_json::json!({}))
}

/// Validate JSONC against schema
pub fn validate_jsonc(_output: &str, _schema: &serde_json::Value) -> anyhow::Result<bool> {
    // Stub implementation
    Ok(true)
}

/// Generate comprehensive template
pub fn generate_comprehensive_template() -> anyhow::Result<String> {
    // Stub implementation
    Ok("{}".to_string())
}

/// Generate basic template
pub fn generate_template() -> anyhow::Result<String> {
    // Stub implementation
    Ok("{}".to_string())
}

/// Analyze Git attributes
pub fn analyze_attributes(
    _attributes: &serde_json::Value,
    _detailed: bool,
) -> anyhow::Result<serde_json::Value> {
    // Stub implementation
    Ok(serde_json::json!({}))
}

/// Summarize analysis
pub fn summarize_analysis(_analysis: &serde_json::Value) -> anyhow::Result<String> {
    // Stub implementation
    Ok("Analysis summary".to_string())
}

/// Convert to gitattributes format
pub fn convert_to_gitattributes(_attributes: &serde_json::Value) -> anyhow::Result<String> {
    // Stub implementation
    Ok("* text=auto".to_string())
}

/// Validate attributes structure
pub fn validate_attributes(
    _attributes: &serde_json::Value,
    _schema: &serde_json::Value,
) -> anyhow::Result<bool> {
    // Stub implementation
    Ok(true)
}

/// Match attributes for files
pub fn match_attributes(
    _attributes: &serde_json::Value,
    _file: &str,
) -> anyhow::Result<Vec<String>> {
    // Stub implementation
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_attributes() {
        let content = r#"
*.sh text eol=lf
*.jpg binary -diff -merge
*.json diff=json
"#;

        let attributes = GitAttributes::from_string(content).unwrap();
        assert_eq!(attributes.attributes.len(), 3);
        assert_eq!(attributes.attributes[0].pattern, "*.sh");
        assert_eq!(attributes.attributes[1].pattern, "*.jpg");
        assert_eq!(attributes.attributes[2].pattern, "*.json");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# Global comment
*.sh text eol=lf
# Shell scripts
*.bat text eol=crlf
"#;

        let attributes = GitAttributes::from_string(content).unwrap();
        assert_eq!(attributes.global_comments.len(), 1);
        // "# Shell scripts" precedes *.bat (attributes[1]); the leading
        // "# Global comment" is captured as a global comment, so *.sh
        // (attributes[0]) has none.
        assert_eq!(attributes.attributes[1].comments.len(), 1);
    }

    #[test]
    fn test_pattern_matching() {
        assert!(GitAttributes::pattern_matches("*.rs", "src/main.rs"));
        assert!(GitAttributes::pattern_matches("docs/**", "docs/api.md"));
        assert!(GitAttributes::pattern_matches("*.sh", "script.sh"));
        assert!(!GitAttributes::pattern_matches("*.rs", "src/main.py"));
    }
}
