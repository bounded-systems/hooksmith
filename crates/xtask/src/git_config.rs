use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Git configuration section with subsections and key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfigSection {
    /// Section name (e.g., "core", "remote", "branch")
    pub name: String,
    /// Subsection name (e.g., "origin" for [remote "origin"])
    pub subsection: Option<String>,
    /// Key-value pairs in this section
    pub values: HashMap<String, String>,
    /// Comments associated with this section
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<String>,
}

/// Complete Git configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// All configuration sections
    pub sections: Vec<GitConfigSection>,
    /// Global comments at the top of the file
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_comments: Vec<String>,
}

/// Categories for organizing Git configuration
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash)]
pub enum ConfigCategory {
    /// User identity and commit behavior
    Identity,
    /// Remotes, branches, and syncing
    Remote,
    /// Behavior customization and safety
    Behavior,
    /// Aliases and custom commands
    Alias,
    /// Tooling integration and custom sections
    Tooling,
    /// Other/unclassified sections
    Other,
}

impl GitConfig {
    /// Parse a .git/config file into structured format
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    /// Parse Git config content from string
    pub fn from_string(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sections = Vec::new();
        let mut global_comments = Vec::new();
        let mut current_section: Option<GitConfigSection> = None;
        let mut current_comments = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Handle comments
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                let comment = trimmed[1..].trim().to_string();
                if current_section.is_none() {
                    global_comments.push(comment);
                } else {
                    current_comments.push(comment);
                }
                continue;
            }

            // Handle section headers [section] or [section "subsection"]
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Save previous section if exists
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }

                let section_content = &trimmed[1..trimmed.len() - 1];
                let parts: Vec<&str> = section_content.splitn(2, ' ').collect();

                let (name, subsection) = match parts.as_slice() {
                    [name] => (name.to_string(), None),
                    [name, subsection] => {
                        let subsection = subsection.trim_matches('"').to_string();
                        (name.to_string(), Some(subsection))
                    }
                    _ => return Err("Invalid section format".into()),
                };

                current_section = Some(GitConfigSection {
                    name,
                    subsection,
                    values: HashMap::new(),
                    comments: current_comments.clone(),
                });
                current_comments.clear();
                continue;
            }

            // Handle key-value pairs
            if let Some(section) = &mut current_section {
                if let Some((key, value)) = Self::parse_key_value(trimmed) {
                    section.values.insert(key, value);
                }
            }
        }

        // Add the last section
        if let Some(section) = current_section {
            sections.push(section);
        }

        Ok(GitConfig {
            sections,
            global_comments,
        })
    }

    /// Parse a key-value pair from a line
    fn parse_key_value(line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            Some((key, value))
        } else {
            None
        }
    }

    /// Convert to JSONC format with proper structure and documentation
    pub fn to_jsonc(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut jsonc = serde_json::to_string_pretty(&self)?;

        // Add schema and documentation
        let schema = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Configuration",
  "description": "Structured Git configuration converted from .git/config to JSONC format",
  "type": "object",
  "properties": {
    "sections": {
      "type": "array",
      "description": "Git configuration sections",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Section name (e.g., core, remote, branch, alias)"
          },
          "subsection": {
            "type": "string",
            "description": "Subsection name (e.g., origin for [remote \"origin\"])"
          },
          "values": {
            "type": "object",
            "description": "Key-value pairs in this section"
          },
          "comments": {
            "type": "array",
            "description": "Comments associated with this section",
            "items": {
              "type": "string"
            }
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
    }
  }
}"#;

        // Replace the opening brace with schema and documentation
        jsonc = jsonc.replacen("{", &format!("{}{{", schema), 1);

        Ok(jsonc)
    }

    /// Categorize sections by their purpose
    pub fn categorize_sections(&self) -> HashMap<ConfigCategory, Vec<&GitConfigSection>> {
        let mut categories = HashMap::new();

        for section in &self.sections {
            let category = match section.name.as_str() {
                "user" | "commit" | "gpg" => ConfigCategory::Identity,
                "remote" | "branch" | "push" | "pull" => ConfigCategory::Remote,
                "core" | "merge" | "rebase" | "diff" | "clean" | "init" => ConfigCategory::Behavior,
                "alias" => ConfigCategory::Alias,
                "vscode" | "github" | "xtask" | "cursor" => ConfigCategory::Tooling,
                _ => ConfigCategory::Other,
            };

            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(section);
        }

        categories
    }

    /// Generate a summary of the configuration
    pub fn generate_summary(&self) -> String {
        let categories = self.categorize_sections();
        let mut summary = String::new();

        summary.push_str("# Git Configuration Summary\n\n");

        for (category, sections) in &categories {
            if !sections.is_empty() {
                summary.push_str(&format!("## {:?}\n", category));
                for section in sections {
                    summary.push_str(&format!(
                        "- **{}**: {} keys\n",
                        section.name,
                        section.values.len()
                    ));
                }
                summary.push_str("\n");
            }
        }

        summary
    }

    /// Validate the configuration structure
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for section in &self.sections {
            // Check for required fields in specific sections
            match section.name.as_str() {
                "remote" => {
                    if section.subsection.is_none() {
                        errors.push(format!("Remote section missing subsection"));
                    }
                    if !section.values.contains_key("url") {
                        errors.push(format!(
                            "Remote section '{}' missing url",
                            section
                                .subsection
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        ));
                    }
                }
                "branch" => {
                    if section.subsection.is_none() {
                        errors.push(format!("Branch section missing subsection"));
                    }
                }
                _ => {}
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
            _ => Err("Unsupported format".into()),
        }
    }
}

/// Command-line interface for Git config management
pub struct GitConfigManager;

impl GitConfigManager {
    /// Convert .git/config to JSONC
    pub fn convert_to_jsonc<P: AsRef<Path>>(
        config_path: P,
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = GitConfig::from_file(config_path)?;

        // Validate the configuration
        if let Err(errors) = config.validate() {
            eprintln!("Validation warnings:");
            for error in errors {
                eprintln!("  - {}", error);
            }
        }

        // Generate JSONC
        let jsonc = config.to_jsonc()?;
        fs::write(output_path, jsonc)?;

        println!("✅ Git config converted to JSONC successfully");
        Ok(())
    }

    /// Generate a comprehensive configuration template
    pub fn generate_template<P: AsRef<Path>>(
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = GitConfig {
            global_comments: vec![
                "Git Configuration Template".to_string(),
                "This template shows all possible Git configuration categories".to_string(),
            ],
            sections: vec![
                // Identity section
                GitConfigSection {
                    name: "user".to_string(),
                    subsection: None,
                    values: HashMap::from([
                        ("name".to_string(), "Your Name".to_string()),
                        ("email".to_string(), "your.email@example.com".to_string()),
                    ]),
                    comments: vec!["User identity for commits".to_string()],
                },
                // Core behavior
                GitConfigSection {
                    name: "core".to_string(),
                    subsection: None,
                    values: HashMap::from([
                        ("editor".to_string(), "nvim".to_string()),
                        ("filemode".to_string(), "true".to_string()),
                        ("autocrlf".to_string(), "input".to_string()),
                        ("ignorecase".to_string(), "true".to_string()),
                    ]),
                    comments: vec!["Core Git behavior settings".to_string()],
                },
                // Remote configuration
                GitConfigSection {
                    name: "remote".to_string(),
                    subsection: Some("origin".to_string()),
                    values: HashMap::from([
                        (
                            "url".to_string(),
                            "git@github.com:user/repo.git".to_string(),
                        ),
                        (
                            "fetch".to_string(),
                            "+refs/heads/*:refs/remotes/origin/*".to_string(),
                        ),
                    ]),
                    comments: vec!["Remote repository configuration".to_string()],
                },
                // Branch tracking
                GitConfigSection {
                    name: "branch".to_string(),
                    subsection: Some("main".to_string()),
                    values: HashMap::from([
                        ("remote".to_string(), "origin".to_string()),
                        ("merge".to_string(), "refs/heads/main".to_string()),
                    ]),
                    comments: vec!["Branch tracking configuration".to_string()],
                },
                // Aliases
                GitConfigSection {
                    name: "alias".to_string(),
                    subsection: None,
                    values: HashMap::from([
                        ("co".to_string(), "checkout".to_string()),
                        ("st".to_string(), "status".to_string()),
                        ("ci".to_string(), "commit".to_string()),
                        (
                            "l".to_string(),
                            "log --oneline --graph --decorate".to_string(),
                        ),
                    ]),
                    comments: vec!["Git command aliases".to_string()],
                },
                // Tooling integration
                GitConfigSection {
                    name: "vscode".to_string(),
                    subsection: None,
                    values: HashMap::from([("merge-base".to_string(), "origin/main".to_string())]),
                    comments: vec!["VS Code integration settings".to_string()],
                },
            ],
        };

        let jsonc = template.to_jsonc()?;
        fs::write(output_path, jsonc)?;

        println!("✅ Git config template generated successfully");
        Ok(())
    }

    /// Analyze current Git configuration
    pub fn analyze<P: AsRef<Path>>(config_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config = GitConfig::from_file(config_path)?;

        println!("{}", config.generate_summary());

        let categories = config.categorize_sections();
        println!("## Detailed Analysis\n");

        for (category, sections) in &categories {
            if !sections.is_empty() {
                println!("### {:?} ({} sections)", category, sections.len());
                for section in sections {
                    println!("- **{}**:", section.name);
                    if let Some(subsection) = &section.subsection {
                        println!("  Subsection: {}", subsection);
                    }
                    for (key, value) in &section.values {
                        println!("  {} = {}", key, value);
                    }
                    println!();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config_content = r#"
[core]
    filemode = true
    bare = false
[remote "origin"]
    url = git@github.com:user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
"#;

        let config = GitConfig::from_string(config_content).unwrap();
        assert_eq!(config.sections.len(), 2);
        assert_eq!(config.sections[0].name, "core");
        assert_eq!(config.sections[1].name, "remote");
        assert_eq!(config.sections[1].subsection, Some("origin".to_string()));
    }

    #[test]
    fn test_parse_with_comments() {
        let config_content = r#"
# Global comment
[core]
    # Section comment
    filemode = true
[alias]
    co = checkout
"#;

        let config = GitConfig::from_string(config_content).unwrap();
        assert_eq!(config.global_comments.len(), 1);
        assert_eq!(config.sections[0].comments.len(), 1);
    }
}
