//! API documentation template

use super::{ModuleInfo, Template};
use std::fmt;

/// API documentation template
pub struct ApiTemplate {
    pub modules: Vec<ModuleInfo>,
    pub title: String,
    pub description: String,
}

impl ApiTemplate {
    /// Create a new API template
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            modules: vec![],
            title: title.to_string(),
            description: description.to_string(),
        }
    }

    /// Add a module to the API documentation
    #[allow(dead_code)]
    pub fn add_module(&mut self, module: ModuleInfo) {
        self.modules.push(module);
    }

    /// Render the API documentation as Markdown
    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n{}\n\n", self.title, self.description));

        for module in &self.modules {
            output.push_str(&format!("## {}\n\n{}\n\n", module.name, module.description));

            for item in &module.public_items {
                output.push_str(&format!("### {}\n\n{}\n\n", item.name, item.description));
                if let Some(sig) = &item.signature {
                    output.push_str(&format!("```rust\n{sig}\n```\n\n"));
                }
            }
        }

        output
    }
}

impl Template for ApiTemplate {
    fn name(&self) -> &str {
        "api"
    }

    fn validate(&self) -> super::Result<()> {
        if self.title.is_empty() {
            return Err(anyhow::anyhow!("API title cannot be empty"));
        }
        if self.description.is_empty() {
            return Err(anyhow::anyhow!("API description cannot be empty"));
        }
        Ok(())
    }
}

impl fmt::Display for ApiTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}
