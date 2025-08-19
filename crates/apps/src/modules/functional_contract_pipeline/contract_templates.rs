use crate::modules::functional_contract_pipeline::symbols::{
    ConcernSymbol, ContractSymbol, RuleSeverity, RuleType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contract template for reusable contract definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplate {
    /// Template name/identifier
    pub name: String,
    /// Template description
    pub description: String,
    /// Template version
    pub version: String,
    /// Template parameters
    pub parameters: Vec<TemplateParameter>,
    /// Template rules
    pub rules: Vec<TemplateRule>,
    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether parameter is required
    pub required: bool,
    /// Default value (if any)
    pub default: Option<serde_json::Value>,
}

/// Parameter types for contract templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Boolean parameter
    Boolean,
    /// Number parameter
    Number,
    /// Array parameter
    Array,
    /// Object parameter
    Object,
    /// Concern symbol parameter
    ConcernSymbol,
    /// Rule severity parameter
    RuleSeverity,
}

/// Template rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule type
    pub rule_type: RuleType,
    /// Rule parameters (parameterized)
    pub parameters: HashMap<String, serde_json::Value>,
    /// Rule severity
    pub severity: RuleSeverity,
    /// Whether rule is required
    pub required: bool,
}

/// Contract template registry
#[derive(Debug, Clone)]
pub struct ContractTemplateRegistry {
    /// Registered templates
    pub templates: HashMap<String, ContractTemplate>,
}

impl ContractTemplateRegistry {
    /// Create a new template registry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register_template(&mut self, template: ContractTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&ContractTemplate> {
        self.templates.get(name)
    }

    /// List all template names
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Instantiate a template with parameters
    pub fn instantiate_template(
        &self,
        template_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<ContractSymbol>, TemplateError> {
        let template = self
            .get_template(template_name)
            .ok_or(TemplateError::TemplateNotFound(template_name.to_string()))?;

        // Validate parameters
        self.validate_parameters(template, &parameters)?;

        // Generate contract symbols from template rules
        let mut contracts = Vec::new();
        for rule in &template.rules {
            let contract_name = self.generate_contract_name(template, rule, &parameters);
            contracts.push(ContractSymbol::new(&contract_name));
        }

        Ok(contracts)
    }

    /// Validate template parameters
    fn validate_parameters(
        &self,
        template: &ContractTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<(), TemplateError> {
        for param in &template.parameters {
            if param.required {
                if !parameters.contains_key(&param.name) {
                    return Err(TemplateError::MissingRequiredParameter(param.name.clone()));
                }
            }
        }

        // Validate parameter types
        for (name, value) in parameters {
            if let Some(param) = template.parameters.iter().find(|p| &p.name == name) {
                if !self.validate_parameter_type(param, value) {
                    return Err(TemplateError::InvalidParameterType(
                        name.clone(),
                        format!("{:?}", param.param_type),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate parameter type
    fn validate_parameter_type(
        &self,
        param: &TemplateParameter,
        value: &serde_json::Value,
    ) -> bool {
        match param.param_type {
            ParameterType::String => value.is_string(),
            ParameterType::Boolean => value.is_boolean(),
            ParameterType::Number => value.is_number(),
            ParameterType::Array => value.is_array(),
            ParameterType::Object => value.is_object(),
            ParameterType::ConcernSymbol => {
                if let Some(s) = value.as_str() {
                    // Basic validation - could be more sophisticated
                    !s.is_empty()
                } else {
                    false
                }
            }
            ParameterType::RuleSeverity => {
                if let Some(s) = value.as_str() {
                    matches!(s, "info" | "warning" | "error" | "critical")
                } else {
                    false
                }
            }
        }
    }

    /// Generate contract name from template and parameters
    fn generate_contract_name(
        &self,
        template: &ContractTemplate,
        rule: &TemplateRule,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut name = format!("{}-{}", template.name, rule.name);

        // Add parameter values to name for uniqueness
        for (param_name, value) in parameters {
            if let Some(s) = value.as_str() {
                name.push_str(&format!("-{}={}", param_name, s));
            }
        }

        name
    }
}

/// Errors that can occur during template operations
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingRequiredParameter(String),
    /// Invalid parameter type
    #[error("Invalid parameter type: expected {1}, got {0}")]
    InvalidParameterType(String, String),
    /// Template instantiation failed
    #[error("Template instantiation failed: {0}")]
    InstantiationFailed(String),
}

/// Predefined contract templates
pub mod predefined {
    use super::*;

    /// Create a template for file mode validation
    pub fn file_mode_template() -> ContractTemplate {
        ContractTemplate {
            name: "file-mode".to_string(),
            description: "Validate file modes for specific patterns".to_string(),
            version: "1.0.0".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "pattern".to_string(),
                    description: "File pattern to match".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                TemplateParameter {
                    name: "executable".to_string(),
                    description: "Whether files should be executable".to_string(),
                    param_type: ParameterType::Boolean,
                    required: true,
                    default: None,
                },
                TemplateParameter {
                    name: "severity".to_string(),
                    description: "Rule severity".to_string(),
                    param_type: ParameterType::RuleSeverity,
                    required: false,
                    default: Some(serde_json::Value::String("error".to_string())),
                },
            ],
            rules: vec![TemplateRule {
                name: "mode-validation".to_string(),
                description: "Validate file mode matches expected value".to_string(),
                rule_type: RuleType::Custom,
                parameters: HashMap::new(),
                severity: RuleSeverity::Error,
                required: true,
            }],
            metadata: HashMap::new(),
        }
    }

    /// Create a template for line ending validation
    pub fn line_ending_template() -> ContractTemplate {
        ContractTemplate {
            name: "line-ending".to_string(),
            description: "Validate line ending normalization".to_string(),
            version: "1.0.0".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "pattern".to_string(),
                    description: "File pattern to match".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                TemplateParameter {
                    name: "normalization".to_string(),
                    description: "Expected line ending normalization".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
            ],
            rules: vec![TemplateRule {
                name: "line-ending-validation".to_string(),
                description: "Validate line ending normalization".to_string(),
                rule_type: RuleType::Custom,
                parameters: HashMap::new(),
                severity: RuleSeverity::Warning,
                required: true,
            }],
            metadata: HashMap::new(),
        }
    }

    /// Create a template for file size validation
    pub fn file_size_template() -> ContractTemplate {
        ContractTemplate {
            name: "file-size".to_string(),
            description: "Validate file size limits".to_string(),
            version: "1.0.0".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "pattern".to_string(),
                    description: "File pattern to match".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                TemplateParameter {
                    name: "max_size".to_string(),
                    description: "Maximum file size in bytes".to_string(),
                    param_type: ParameterType::Number,
                    required: true,
                    default: None,
                },
            ],
            rules: vec![TemplateRule {
                name: "size-validation".to_string(),
                description: "Validate file size is within limits".to_string(),
                rule_type: RuleType::FileSize,
                parameters: HashMap::new(),
                severity: RuleSeverity::Error,
                required: true,
            }],
            metadata: HashMap::new(),
        }
    }

    /// Create a template for file extension validation
    pub fn file_extension_template() -> ContractTemplate {
        ContractTemplate {
            name: "file-extension".to_string(),
            description: "Validate file extensions".to_string(),
            version: "1.0.0".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "allowed_extensions".to_string(),
                    description: "List of allowed file extensions".to_string(),
                    param_type: ParameterType::Array,
                    required: true,
                    default: None,
                },
                TemplateParameter {
                    name: "severity".to_string(),
                    description: "Rule severity".to_string(),
                    param_type: ParameterType::RuleSeverity,
                    required: false,
                    default: Some(serde_json::Value::String("warning".to_string())),
                },
            ],
            rules: vec![TemplateRule {
                name: "extension-validation".to_string(),
                description: "Validate file has allowed extension".to_string(),
                rule_type: RuleType::FileExtension,
                parameters: HashMap::new(),
                severity: RuleSeverity::Warning,
                required: true,
            }],
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_registry() {
        let mut registry = ContractTemplateRegistry::new();
        let template = predefined::file_mode_template();

        registry.register_template(template);
        assert_eq!(registry.list_templates().len(), 1);
        assert!(registry.get_template("file-mode").is_some());
    }

    #[test]
    fn test_template_instantiation() {
        let mut registry = ContractTemplateRegistry::new();
        let template = predefined::file_mode_template();
        registry.register_template(template);

        let mut parameters = HashMap::new();
        parameters.insert(
            "pattern".to_string(),
            serde_json::Value::String("*.rs".to_string()),
        );
        parameters.insert("executable".to_string(), serde_json::Value::Bool(false));

        let contracts = registry
            .instantiate_template("file-mode", parameters)
            .unwrap();
        assert_eq!(contracts.len(), 1);
    }

    #[test]
    fn test_template_validation() {
        let mut registry = ContractTemplateRegistry::new();
        let template = predefined::file_mode_template();
        registry.register_template(template);

        // Missing required parameter
        let parameters = HashMap::new();
        let result = registry.instantiate_template("file-mode", parameters);
        assert!(result.is_err());
    }
}
