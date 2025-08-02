//! Examples template for generating example documentation

use super::{Template, ExampleInfo};
use std::fmt;

/// Examples template for generating example documentation
pub struct ExamplesTemplate {
    pub examples: Vec<ExampleInfo>,
    pub title: String,
    pub description: String,
}

impl ExamplesTemplate {
    /// Create a new examples template
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            examples: vec![],
            title: title.to_string(),
            description: description.to_string(),
        }
    }
    
    /// Add an example to the template
    pub fn add_example(&mut self, example: ExampleInfo) {
        self.examples.push(example);
    }
    
    /// Render the examples as Markdown
    pub fn render(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# {}\n\n{}\n\n", self.title, self.description));
        
        for (i, example) in self.examples.iter().enumerate() {
            output.push_str(&format!("## Example {}: {}\n\n", i + 1, example.name));
            output.push_str(&format!("{}\n\n", example.description));
            output.push_str(&format!("```rust\n{}\n```\n\n", example.code));
            
            if let Some(output_text) = &example.output {
                output.push_str(&format!("Output:\n\n```\n{}\n```\n\n", output_text));
            }
        }
        
        output
    }
}

impl Template for ExamplesTemplate {
    fn name(&self) -> &str {
        "examples"
    }
    
    fn validate(&self) -> super::Result<()> {
        if self.title.is_empty() {
            return Err(anyhow::anyhow!("Examples title cannot be empty"));
        }
        if self.description.is_empty() {
            return Err(anyhow::anyhow!("Examples description cannot be empty"));
        }
        Ok(())
    }
}

impl fmt::Display for ExamplesTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
} 
