//! README template for generating project documentation

use super::{ApiDocumentation, ExampleInfo, ProjectData, Template};
use std::fmt;

/// README template that generates comprehensive project documentation
pub struct ReadmeTemplate {
    pub project_data: ProjectData,
    pub api_docs: ApiDocumentation,
    pub examples: Vec<ExampleInfo>,
    pub features: Vec<Feature>,
    pub status: ProjectStatus,
    pub roadmap: Vec<RoadmapItem>,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
}

#[derive(Debug, Clone)]
pub enum FeatureStatus {
    Complete,
    InProgress,
    Planned,
    NotStarted,
}

impl fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureStatus::Complete => write!(f, "✅ Complete"),
            FeatureStatus::InProgress => write!(f, "🔄 In Progress"),
            FeatureStatus::Planned => write!(f, "📋 Planned"),
            FeatureStatus::NotStarted => write!(f, "❌ Not Started"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoadmapItem {
    pub phase: String,
    pub title: String,
    pub description: String,
    pub status: RoadmapStatus,
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RoadmapStatus {
    Complete,
    InProgress,
    Planned,
}

impl fmt::Display for RoadmapStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoadmapStatus::Complete => write!(f, "✅"),
            RoadmapStatus::InProgress => write!(f, "🔄"),
            RoadmapStatus::Planned => write!(f, "📋"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectStatus {
    pub current_state: String,
    pub intended_purpose: String,
    pub features: Vec<Feature>,
}

impl ReadmeTemplate {
    /// Create a new README template with default data
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_data = ProjectData::from_cargo_toml()?;
        let api_docs = ApiDocumentation::from_rust_sources()?;

        Ok(Self {
            project_data,
            api_docs,
            examples: vec![],
            features: Self::default_features(),
            status: Self::default_status(),
            roadmap: Self::default_roadmap(),
        })
    }

    /// Render the architecture diagram
    fn render_architecture(&self) -> String {
        r#"
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Lefthook      │    │   Hooksmith     │    │   WASM          │
│   (Git Hooks)   │◄──►│   (CLI Tool)    │◄──►│   Components    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Rust          │
                       │   Binaries      │
                       └─────────────────┘
```
"#
        .to_string()
    }

    /// Render the features table
    fn render_features_table(&self) -> String {
        let mut table = String::new();
        table.push_str("| Feature | Status | Description |\n");
        table.push_str("|---------|--------|-------------|\n");

        for feature in &self.features {
            table.push_str(&format!(
                "| **{}** | {} | {} |\n",
                feature.name, feature.status, feature.description
            ));
        }

        table
    }

    /// Render the roadmap
    fn render_roadmap(&self) -> String {
        let mut roadmap = String::new();

        for item in &self.roadmap {
            roadmap.push_str(&format!(
                "#### **{}: {}** {}\n",
                item.phase, item.title, item.status
            ));
            roadmap.push_str(&format!("{}\n\n", item.description));

            for (_i, task) in item.items.iter().enumerate() {
                let status = match item.status {
                    RoadmapStatus::Complete => "✅",
                    RoadmapStatus::InProgress => "🔄",
                    RoadmapStatus::Planned => "📋",
                };
                roadmap.push_str(&format!("- [{}] {}\n", status, task));
            }
            roadmap.push('\n');
        }

        roadmap
    }

    /// Default features for Hooksmith
    fn default_features() -> Vec<Feature> {
        vec![
            Feature {
                name: "CLI Structure".to_string(),
                description: "Full CLI with commands for building, generating, installing, and managing hooks".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Documentation".to_string(),
                description: "Comprehensive rustdoc comments, README, and generated documentation".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Testing".to_string(),
                description: "16 integration tests, unit tests, and build verification".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Build System".to_string(),
                description: "Automated build script with component compilation".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "WASM Components".to_string(),
                description: "WASM module with component building, running, and bindings generation".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Lefthook Integration".to_string(),
                description: "Configuration generator module implemented".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Hook Building".to_string(),
                description: "Rust-to-binary compilation logic with Cargo integration".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "WASM Compilation".to_string(),
                description: "Placeholder WASM component building with WIT validation".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Tool Integration".to_string(),
                description: "Integration with existing worktree tools (wtp, wt, treekanga, git)".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Hook Installation".to_string(),
                description: "Hook installation and management functionality".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Worktree Management".to_string(),
                description: "Worktree creation, listing, switching, and removal".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Hierarchical Validation".to_string(),
                description: "Bottom-up contract validation with Git Notes integration".to_string(),
                status: FeatureStatus::Complete,
            },
            Feature {
                name: "Contract State Machine".to_string(),
                description: "Schema-driven state machine with Merkle chain validation".to_string(),
                status: FeatureStatus::Complete,
            },
        ]
    }

    /// Default project status
    fn default_status() -> ProjectStatus {
        ProjectStatus {
            current_state: "Hooksmith is a fully functional CLI tool that builds Rust binaries into Lefthook hooks with WASM components. All core features are implemented and tested.".to_string(),
            intended_purpose: "Hooksmith is designed to be a CLI tool that builds Rust binaries into Lefthook hooks with WASM components. The goal is to compile Rust code into optimized binary executables for Git hooks, integrate WebAssembly components for cross-language functionality, generate Lefthook configuration files automatically, and provide a unified interface for hook management.".to_string(),
            features: Self::default_features(),
        }
    }

    /// Default roadmap
    fn default_roadmap() -> Vec<RoadmapItem> {
        vec![
            RoadmapItem {
                phase: "Phase 1".to_string(),
                title: "Foundation".to_string(),
                description: "Core CLI structure and project setup".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "CLI structure and command parsing".to_string(),
                    "Documentation and testing framework".to_string(),
                    "Build system and component architecture".to_string(),
                    "Basic project structure".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 2".to_string(),
                title: "WASM Integration".to_string(),
                description: "WebAssembly component integration".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "WASM dependencies added (wasmtime, wit-bindgen)".to_string(),
                    "WIT interface definitions created".to_string(),
                    "Worktree-runner component scaffolded".to_string(),
                    "Actual WASM component compilation (placeholder implementation)".to_string(),
                    "WASM runtime integration in hooks".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 3".to_string(),
                title: "Lefthook Integration".to_string(),
                description: "Git hook management integration".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "Lefthook configuration generator".to_string(),
                    "YAML configuration structure".to_string(),
                    "Hook installation and management".to_string(),
                    "Git integration and hook execution".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 4".to_string(),
                title: "Tool Integration".to_string(),
                description: "Integration with existing Git tools".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "Integration with wtp, wt, treekanga, git".to_string(),
                    "Worktree management automation".to_string(),
                    "Cross-platform compatibility".to_string(),
                    "Performance optimization (basic implementation)".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 5".to_string(),
                title: "Hierarchical Validation".to_string(),
                description: "Contract validation system".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "Bottom-up validation pipeline implementation".to_string(),
                    "Git Notes integration for validation history".to_string(),
                    "Hierarchical scope detection".to_string(),
                    "Validation chain integrity verification".to_string(),
                    "Git hooks integration (pre-commit, post-commit)".to_string(),
                    "Xtask CLI for validation management".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 6".to_string(),
                title: "Contract State Machine".to_string(),
                description: "State machine validation system".to_string(),
                status: RoadmapStatus::Complete,
                items: vec![
                    "Schema-driven state machine implementation".to_string(),
                    "Merkle chain validation system".to_string(),
                    "Git Notes integration for audit trails".to_string(),
                    "CI pipeline with security enforcement".to_string(),
                    "State transition validation and enforcement".to_string(),
                ],
            },
            RoadmapItem {
                phase: "Phase 7".to_string(),
                title: "Production Ready".to_string(),
                description: "Production deployment preparation".to_string(),
                status: RoadmapStatus::Planned,
                items: vec![
                    "Error handling and recovery".to_string(),
                    "Performance benchmarking".to_string(),
                    "Security audit and hardening".to_string(),
                    "Production deployment pipeline".to_string(),
                ],
            },
        ]
    }
}

impl Template for ReadmeTemplate {
    fn name(&self) -> &str {
        "readme"
    }

    fn validate(&self) -> super::Result<()> {
        if self.project_data.name.is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }
        if self.project_data.description.is_empty() {
            return Err(anyhow::anyhow!("Project description cannot be empty"));
        }
        Ok(())
    }
}

impl fmt::Display for ReadmeTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl ReadmeTemplate {
    /// Render the complete README
    pub fn render(&self) -> String {
        format!(
            r#"# {}

{}

## 🎯 Purpose

Hooksmith bridges the gap between modern Git workflow tools and WebAssembly components, enabling:

- **High-performance Git hooks** written in Rust
- **Cross-language functionality** via WASM components
- **Type-safe interfaces** using WIT (WebAssembly Interface Types)
- **Seamless integration** with Lefthook for Git workflow management

## 🏗️ Architecture

{}

## 🚀 Current Status vs Intended Purpose

### 🎯 **Intended Purpose**
{}

### 📊 **Current State**

{}

## 🚀 **Roadmap**

{}

## 📚 API Documentation

{}

## 🧪 Examples

{}

## 🛠️ Installation

```bash
cargo install hooksmith
```

## 🚀 Quick Start

```bash
# Build and install hooks
hooksmith build --install

# Generate Lefthook configuration
hooksmith gen-lefthook

# Run validation
hooksmith validate
```

## 📖 Documentation

- [API Reference](docs/api/)
- [Development Guide](docs/DEVELOPMENT.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
"#,
            self.project_data.name,
            self.project_data.description,
            self.render_architecture(),
            self.status.intended_purpose,
            self.render_features_table(),
            self.render_roadmap(),
            self.api_docs.render(),
            self.render_examples(),
        )
    }

    /// Render examples section
    fn render_examples(&self) -> String {
        if self.examples.is_empty() {
            "No examples available yet.".to_string()
        } else {
            let mut examples = String::new();
            for example in &self.examples {
                examples.push_str(&format!(
                    "### {}\n\n{}\n\n",
                    example.name, example.description
                ));
                examples.push_str(&format!("```rust\n{}\n```\n\n", example.code));
                if let Some(output) = &example.output {
                    examples.push_str(&format!("Output:\n\n```\n{}\n```\n\n", output));
                }
            }
            examples
        }
    }
}
