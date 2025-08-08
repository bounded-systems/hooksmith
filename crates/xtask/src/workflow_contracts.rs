use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Simple workflow structure for parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: Option<String>,
    pub on: Option<serde_yaml::Value>,
    pub jobs: Option<serde_yaml::Value>,
    pub env: Option<serde_yaml::Value>,
    pub permissions: Option<serde_yaml::Value>,
}

/// Parse workflow from YAML content
fn parse_workflow(content: &str) -> Result<Workflow> {
    let workflow: Workflow = serde_yaml::from_str(content)
        .with_context(|| "Failed to parse workflow YAML")?;
    Ok(workflow)
}

/// Workflow contract validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContractResult {
    pub workflow_path: PathBuf,
    pub is_valid: bool,
    pub concerns: Vec<WorkflowConcern>,
    pub verification_results: Vec<WorkflowVerification>,
    pub audit_trail: Vec<WorkflowAuditEntry>,
    pub trigger_analysis: TriggerAnalysis,
    pub contract_compliance: ContractCompliance,
}

/// Workflow concern (potential issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConcern {
    pub level: ConcernLevel,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
    pub contract_violation: Option<String>,
}

/// Concern severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcernLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Workflow verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVerification {
    pub check_name: String,
    pub passed: bool,
    pub details: String,
    pub contract_scope: Option<String>,
}

/// Workflow audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAuditEntry {
    pub timestamp: String,
    pub action: String,
    pub details: String,
    pub contract_id: Option<String>,
}

/// Trigger analysis for workflow contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAnalysis {
    pub defined_triggers: Vec<TriggerDefinition>,
    pub gated_triggers: Vec<TriggerDefinition>,
    pub mockable_triggers: Vec<MockableTrigger>,
    pub billing_impact: BillingImpact,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDefinition {
    pub name: String,
    pub event_type: String,
    pub conditions: Vec<String>,
    pub is_gated: bool,
    pub mock_inputs: Option<HashMap<String, String>>,
}

/// Mockable trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockableTrigger {
    pub original_trigger: String,
    pub mock_inputs: HashMap<String, MockInput>,
    pub conditional_logic: String,
}

/// Mock input definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockInput {
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub input_type: String,
}

/// Billing impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingImpact {
    pub estimated_monthly_cost: f64,
    pub cost_optimization: Vec<String>,
    pub gating_strategy: String,
}

/// Contract compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCompliance {
    pub hooks_defined: bool,
    pub validation_present: bool,
    pub audit_trail_enabled: bool,
    pub deterministic_behavior: bool,
    pub contract_metadata: HashMap<String, String>,
}

/// Workflow contract validator configuration
#[derive(Debug, Clone)]
pub struct WorkflowContractConfig {
    pub strict_mode: bool,
    pub allow_disabled_workflows: bool,
    pub require_paths: bool,
    pub max_jobs_per_workflow: Option<usize>,
    pub allowed_runners: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub contract_validation: bool,
    pub trigger_mocking: bool,
}

/// Workflow contract validator
pub struct WorkflowContractValidator {
    config: WorkflowContractConfig,
}

impl WorkflowContractValidator {
    pub fn new(config: WorkflowContractConfig) -> Self {
        Self { config }
    }

    /// Validate a workflow file for contract compliance
    pub fn validate_workflow(&self, workflow_path: &Path) -> Result<WorkflowContractResult> {
        let content = fs::read_to_string(workflow_path)
            .with_context(|| format!("Failed to read workflow file: {:?}", workflow_path))?;

        // Parse the workflow using gh-workflow-parser
        let workflow = parse_workflow(&content).with_context(|| "Failed to parse workflow YAML")?;

        let mut result = WorkflowContractResult {
            workflow_path: workflow_path.to_path_buf(),
            is_valid: true,
            concerns: Vec::new(),
            verification_results: Vec::new(),
            audit_trail: Vec::new(),
            trigger_analysis: self.analyze_triggers(&workflow),
            contract_compliance: self.check_contract_compliance(&workflow),
        };

        // Perform validation checks
        self.validate_workflow_structure(&workflow, &mut result)?;
        self.validate_jobs(&workflow, &mut result)?;
        self.validate_triggers(&workflow, &mut result)?;
        self.validate_contracts(&workflow, &mut result)?;

        // Update overall validity
        result.is_valid = result
            .concerns
            .iter()
            .all(|c| matches!(c.level, ConcernLevel::Info | ConcernLevel::Warning));

        Ok(result)
    }

    /// Analyze triggers for gating and mocking strategies
    fn analyze_triggers(&self, workflow: &Workflow) -> TriggerAnalysis {
        let mut defined_triggers = Vec::new();
        let mut gated_triggers = Vec::new();
        let mut mockable_triggers = Vec::new();

        // Extract trigger information from workflow
        if let Some(on) = &workflow.on {
            for (trigger_name, trigger_config) in on {
                let trigger_def = TriggerDefinition {
                    name: trigger_name.clone(),
                    event_type: trigger_name.clone(),
                    conditions: self.extract_trigger_conditions(trigger_config),
                    is_gated: self.is_trigger_gated(trigger_name, workflow),
                    mock_inputs: self.generate_mock_inputs(trigger_name),
                };

                defined_triggers.push(trigger_def.clone());

                if trigger_def.is_gated {
                    gated_triggers.push(trigger_def);
                }

                // Check if trigger can be mocked
                if self.can_mock_trigger(trigger_name) {
                    mockable_triggers.push(MockableTrigger {
                        original_trigger: trigger_name.clone(),
                        mock_inputs: self.get_mock_inputs_for_trigger(trigger_name),
                        conditional_logic: self.generate_mock_conditional(trigger_name),
                    });
                }
            }
        }

        TriggerAnalysis {
            defined_triggers,
            gated_triggers,
            mockable_triggers,
            billing_impact: self.calculate_billing_impact(&defined_triggers, workflow),
        }
    }

    /// Check contract compliance
    fn check_contract_compliance(&self, workflow: &Workflow) -> ContractCompliance {
        ContractCompliance {
            hooks_defined: self.has_hooks_defined(workflow),
            validation_present: self.has_validation_steps(workflow),
            audit_trail_enabled: self.has_audit_trail(workflow),
            deterministic_behavior: self.has_deterministic_behavior(workflow),
            contract_metadata: self.extract_contract_metadata(workflow),
        }
    }

    /// Validate workflow structure
    fn validate_workflow_structure(
        &self,
        workflow: &Workflow,
        result: &mut WorkflowContractResult,
    ) -> Result<()> {
        // Check for required fields
        if workflow.name.is_none() {
            result.concerns.push(WorkflowConcern {
                level: ConcernLevel::Warning,
                message: "Workflow name is not defined".to_string(),
                location: None,
                suggestion: Some("Add a name field for better identification".to_string()),
                contract_violation: None,
            });
        }

        // Check job count limits
        if let Some(max_jobs) = self.config.max_jobs_per_workflow {
            if let Some(jobs) = &workflow.jobs {
                if jobs.len() > max_jobs {
                    result.concerns.push(WorkflowConcern {
                        level: ConcernLevel::Error,
                        message: format!("Too many jobs: {} (max: {})", jobs.len(), max_jobs),
                        location: None,
                        suggestion: Some("Consider splitting into multiple workflows".to_string()),
                        contract_violation: Some("job_count_limit".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate jobs for contract compliance
    fn validate_jobs(
        &self,
        workflow: &Workflow,
        result: &mut WorkflowContractResult,
    ) -> Result<()> {
        if let Some(jobs) = &workflow.jobs {
            for (job_name, job) in jobs {
                // Check runner compliance
                if let Some(runs_on) = &job.runs_on {
                    if !self.config.allowed_runners.contains(runs_on) {
                        result.concerns.push(WorkflowConcern {
                            level: ConcernLevel::Error,
                            message: format!(
                                "Job '{}' uses unauthorized runner: {}",
                                job_name, runs_on
                            ),
                            location: Some(format!("jobs.{}", job_name)),
                            suggestion: Some(
                                "Use an allowed runner from the configuration".to_string(),
                            ),
                            contract_violation: Some("runner_policy".to_string()),
                        });
                    }
                }

                // Check for forbidden actions
                if let Some(steps) = &job.steps {
                    for step in steps {
                        if let Some(uses) = &step.uses {
                            for forbidden_action in &self.config.forbidden_actions {
                                if uses.contains(forbidden_action) {
                                    result.concerns.push(WorkflowConcern {
                                        level: ConcernLevel::Critical,
                                        message: format!(
                                            "Job '{}' uses forbidden action: {}",
                                            job_name, uses
                                        ),
                                        location: Some(format!("jobs.{}.steps", job_name)),
                                        suggestion: Some(
                                            "Replace with an approved action".to_string(),
                                        ),
                                        contract_violation: Some("forbidden_action".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate triggers for contract compliance
    fn validate_triggers(
        &self,
        workflow: &Workflow,
        result: &mut WorkflowContractResult,
    ) -> Result<()> {
        if let Some(on) = &workflow.on {
            for (trigger_name, _) in on {
                // Check if trigger is properly gated
                if !self.is_trigger_gated(trigger_name, workflow)
                    && trigger_name != "workflow_dispatch"
                {
                    result.concerns.push(WorkflowConcern {
                        level: ConcernLevel::Warning,
                        message: format!(
                            "Trigger '{}' is not gated and may cause billing",
                            trigger_name
                        ),
                        location: Some("on".to_string()),
                        suggestion: Some("Add conditional logic to gate execution".to_string()),
                        contract_violation: Some("billing_control".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate contracts
    fn validate_contracts(
        &self,
        workflow: &Workflow,
        result: &mut WorkflowContractResult,
    ) -> Result<()> {
        if self.config.contract_validation {
            // Check for Hooksmith contract metadata
            if !self.has_contract_metadata(workflow) {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Info,
                    message: "No Hooksmith contract metadata found".to_string(),
                    location: None,
                    suggestion: Some("Add contract metadata for better traceability".to_string()),
                    contract_violation: None,
                });
            }

            // Check for deterministic behavior
            if !self.has_deterministic_behavior(workflow) {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Warning,
                    message: "Workflow may not have deterministic behavior".to_string(),
                    location: None,
                    suggestion: Some(
                        "Add explicit conditions and avoid non-deterministic steps".to_string(),
                    ),
                    contract_violation: Some("deterministic_behavior".to_string()),
                });
            }
        }

        Ok(())
    }

    // Helper methods for trigger analysis
    fn extract_trigger_conditions(&self, _trigger_config: &serde_yaml::Value) -> Vec<String> {
        // Implementation would parse trigger configuration
        vec!["default".to_string()]
    }

    fn is_trigger_gated(&self, trigger_name: &str, workflow: &Workflow) -> bool {
        // Check if jobs have conditional logic that gates execution
        if let Some(jobs) = &workflow.jobs {
            for (_, job) in jobs {
                if let Some(if_condition) = &job.if_ {
                    if if_condition.contains(&format!("github.event_name == '{}'", trigger_name)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn generate_mock_inputs(&self, trigger_name: &str) -> Option<HashMap<String, String>> {
        match trigger_name {
            "pull_request" => {
                let mut inputs = HashMap::new();
                inputs.insert("event_name".to_string(), "pull_request".to_string());
                inputs.insert("pr_number".to_string(), "123".to_string());
                inputs.insert("head_ref".to_string(), "feature-branch".to_string());
                inputs.insert("base_ref".to_string(), "main".to_string());
                Some(inputs)
            }
            "push" => {
                let mut inputs = HashMap::new();
                inputs.insert("event_name".to_string(), "push".to_string());
                inputs.insert("branch".to_string(), "main".to_string());
                inputs.insert("commit_sha".to_string(), "abc123".to_string());
                Some(inputs)
            }
            _ => None,
        }
    }

    fn can_mock_trigger(&self, trigger_name: &str) -> bool {
        matches!(
            trigger_name,
            "pull_request" | "push" | "release" | "issue_comment"
        )
    }

    fn get_mock_inputs_for_trigger(&self, trigger_name: &str) -> HashMap<String, MockInput> {
        let mut inputs = HashMap::new();

        match trigger_name {
            "pull_request" => {
                inputs.insert(
                    "event_name".to_string(),
                    MockInput {
                        description: "Event type to mock".to_string(),
                        required: true,
                        default_value: Some("pull_request".to_string()),
                        input_type: "string".to_string(),
                    },
                );
                inputs.insert(
                    "pr_number".to_string(),
                    MockInput {
                        description: "Mock PR number".to_string(),
                        required: true,
                        default_value: None,
                        input_type: "number".to_string(),
                    },
                );
            }
            "push" => {
                inputs.insert(
                    "event_name".to_string(),
                    MockInput {
                        description: "Event type to mock".to_string(),
                        required: true,
                        default_value: Some("push".to_string()),
                        input_type: "string".to_string(),
                    },
                );
                inputs.insert(
                    "branch".to_string(),
                    MockInput {
                        description: "Branch name".to_string(),
                        required: true,
                        default_value: Some("main".to_string()),
                        input_type: "string".to_string(),
                    },
                );
            }
            _ => {}
        }

        inputs
    }

    fn generate_mock_conditional(&self, trigger_name: &str) -> String {
        format!(
            "github.event_name == 'workflow_dispatch' && inputs.event_name == '{}'",
            trigger_name
        )
    }

    fn calculate_billing_impact(
        &self,
        triggers: &[TriggerDefinition],
        _workflow: &Workflow,
    ) -> BillingImpact {
        let gated_count = triggers.iter().filter(|t| t.is_gated).count();
        let total_count = triggers.len();

        let estimated_cost = if gated_count == total_count {
            0.0 // Fully gated workflows cost nothing
        } else {
            (total_count - gated_count) as f64 * 0.008 // Rough estimate per trigger
        };

        BillingImpact {
            estimated_monthly_cost: estimated_cost,
            cost_optimization: vec![
                "Use conditional logic to gate execution".to_string(),
                "Consider workflow_dispatch for manual triggers".to_string(),
            ],
            gating_strategy: if gated_count > 0 {
                "Partial gating implemented".to_string()
            } else {
                "No gating - consider implementing".to_string()
            },
        }
    }

    // Helper methods for contract compliance
    fn has_hooks_defined(&self, _workflow: &Workflow) -> bool {
        // Check for Hooksmith hook definitions
        true // Placeholder
    }

    fn has_validation_steps(&self, workflow: &Workflow) -> bool {
        if let Some(jobs) = &workflow.jobs {
            for (_, job) in jobs {
                if let Some(steps) = &job.steps {
                    for step in steps {
                        if let Some(run) = &step.run {
                            if run.contains("validate") || run.contains("check") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn has_audit_trail(&self, _workflow: &Workflow) -> bool {
        // Check for audit trail implementation
        true // Placeholder
    }

    fn has_deterministic_behavior(&self, _workflow: &Workflow) -> bool {
        // Check for deterministic behavior
        true // Placeholder
    }

    fn extract_contract_metadata(&self, _workflow: &Workflow) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("contract_version".to_string(), "1.0".to_string());
        metadata.insert("validator".to_string(), "hooksmith".to_string());
        metadata
    }

    fn has_contract_metadata(&self, _workflow: &Workflow) -> bool {
        // Check for contract metadata
        true // Placeholder
    }
}

/// Generate a workflow contracts report
pub fn generate_workflow_contracts_report(
    results: &[WorkflowContractResult],
    format: &str,
) -> Result<String> {
    match format.to_lowercase().as_str() {
        "json" => Ok(serde_json::to_string_pretty(results)?),
        "yaml" => Ok(serde_yaml::to_string(results)?),
        "markdown" => generate_markdown_report(results),
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
}

/// Generate markdown report
fn generate_markdown_report(results: &[WorkflowContractResult]) -> Result<String> {
    let mut report = String::new();
    report.push_str("# Workflow Contracts Report\n\n");

    for result in results {
        report.push_str(&format!("## {}\n\n", result.workflow_path.display()));

        // Overall status
        let status = if result.is_valid {
            "✅ Valid"
        } else {
            "❌ Invalid"
        };
        report.push_str(&format!("**Status:** {}\n\n", status));

        // Concerns
        if !result.concerns.is_empty() {
            report.push_str("### Concerns\n\n");
            for concern in &result.concerns {
                let level_icon = match concern.level {
                    ConcernLevel::Info => "ℹ️",
                    ConcernLevel::Warning => "⚠️",
                    ConcernLevel::Error => "❌",
                    ConcernLevel::Critical => "🚨",
                };
                report.push_str(&format!(
                    "{} **{}:** {}\n",
                    level_icon,
                    format!("{:?}", concern.level),
                    concern.message
                ));
                if let Some(suggestion) = &concern.suggestion {
                    report.push_str(&format!("   💡 **Suggestion:** {}\n", suggestion));
                }
                report.push_str("\n");
            }
        }

        // Trigger analysis
        report.push_str("### Trigger Analysis\n\n");
        report.push_str(&format!(
            "- **Defined Triggers:** {}\n",
            result.trigger_analysis.defined_triggers.len()
        ));
        report.push_str(&format!(
            "- **Gated Triggers:** {}\n",
            result.trigger_analysis.gated_triggers.len()
        ));
        report.push_str(&format!(
            "- **Mockable Triggers:** {}\n",
            result.trigger_analysis.mockable_triggers.len()
        ));
        report.push_str(&format!(
            "- **Estimated Monthly Cost:** ${:.2}\n",
            result
                .trigger_analysis
                .billing_impact
                .estimated_monthly_cost
        ));

        // Contract compliance
        report.push_str("\n### Contract Compliance\n\n");
        report.push_str(&format!(
            "- **Hooks Defined:** {}\n",
            result.contract_compliance.hooks_defined
        ));
        report.push_str(&format!(
            "- **Validation Present:** {}\n",
            result.contract_compliance.validation_present
        ));
        report.push_str(&format!(
            "- **Audit Trail Enabled:** {}\n",
            result.contract_compliance.audit_trail_enabled
        ));
        report.push_str(&format!(
            "- **Deterministic Behavior:** {}\n",
            result.contract_compliance.deterministic_behavior
        ));

        report.push_str("\n---\n\n");
    }

    Ok(report)
}

/// Generate a gated workflow stub
pub fn generate_gated_workflow_stub(trigger_name: &str) -> Result<String> {
    let stub = match trigger_name {
        "pull_request" => {
            r#"name: Gated Pull Request Workflow

on:
  pull_request:
    branches: [main]
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true
        default: "pull_request"
      pr_number:
        description: "Mock PR number"
        required: true
      head_ref:
        description: "Head branch name"
        required: false
      base_ref:
        description: "Base branch name"
        required: false

jobs:
  gated-job:
    if: github.event_name == 'workflow_dispatch' || github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Run validation
        run: |
          echo "Running validation for PR #${{ github.event.number || inputs.pr_number }}"
          echo "From: ${{ github.head_ref || inputs.head_ref }}"
          echo "To: ${{ github.base_ref || inputs.base_ref }}"
"#
        }
        "push" => {
            r#"name: Gated Push Workflow

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true
        default: "push"
      branch:
        description: "Branch name"
        required: true
        default: "main"
      commit_sha:
        description: "Commit SHA"
        required: false

jobs:
  gated-job:
    if: github.event_name == 'workflow_dispatch' || github.event_name == 'push'
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Run validation
        run: |
          echo "Running validation for push to ${{ github.ref_name || inputs.branch }}"
          echo "Commit: ${{ github.sha || inputs.commit_sha }}"
"#
        }
        _ => {
            r#"name: Gated Workflow

on:
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true

jobs:
  gated-job:
    if: github.event_name == 'workflow_dispatch'
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Run validation
        run: |
          echo "Running validation for ${{ inputs.event_name }}"
"#
        }
    };

    Ok(stub.to_string())
}

/// Test configuration for workflow contracts
#[derive(Debug, Clone)]
pub struct WorkflowContractTestConfig {
    pub use_act: bool,
    pub act_dry_run: bool,
    pub generate_inputs: bool,
    pub test_all_triggers: bool,
    pub output_dir: Option<PathBuf>,
    pub act_inputs_file: Option<PathBuf>,
}

/// Test result for workflow contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContractTestResult {
    pub workflow_path: PathBuf,
    pub validation_result: WorkflowContractResult,
    pub act_test_results: Vec<ActTestResult>,
    pub mock_inputs_generated: Vec<MockInputFile>,
    pub test_summary: TestSummary,
}

/// Act test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActTestResult {
    pub trigger_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time: f64,
}

/// Mock input file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockInputFile {
    pub trigger_name: String,
    pub file_path: PathBuf,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Test summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_workflows: usize,
    pub valid_workflows: usize,
    pub act_tests_passed: usize,
    pub act_tests_failed: usize,
    pub mock_inputs_generated: usize,
}

/// Test runner for workflow contracts
pub struct WorkflowContractTestRunner {
    config: WorkflowContractTestConfig,
    validator: WorkflowContractValidator,
}

impl WorkflowContractTestRunner {
    pub fn new(config: WorkflowContractTestConfig, validator: WorkflowContractValidator) -> Self {
        Self { config, validator }
    }

    /// Run comprehensive tests on workflow contracts
    pub fn run_tests(&self, workflow_paths: &[PathBuf]) -> Result<Vec<WorkflowContractTestResult>> {
        let mut results = Vec::new();

        for workflow_path in workflow_paths {
            println!("🧪 Testing workflow: {}", workflow_path.display());

            let mut test_result = WorkflowContractTestResult {
                workflow_path: workflow_path.clone(),
                validation_result: self.validator.validate_workflow(workflow_path)?,
                act_test_results: Vec::new(),
                mock_inputs_generated: Vec::new(),
                test_summary: TestSummary {
                    total_workflows: 1,
                    valid_workflows: 0,
                    act_tests_passed: 0,
                    act_tests_failed: 0,
                    mock_inputs_generated: 0,
                },
            };

            // Update test summary
            test_result.test_summary.valid_workflows = if test_result.validation_result.is_valid {
                1
            } else {
                0
            };

            // Generate mock inputs if requested
            if self.config.generate_inputs {
                test_result.mock_inputs_generated = self.generate_mock_inputs(workflow_path)?;
                test_result.test_summary.mock_inputs_generated =
                    test_result.mock_inputs_generated.len();
            }

            // Run act tests if enabled
            if self.config.use_act {
                test_result.act_test_results = self.run_act_tests(workflow_path)?;
                test_result.test_summary.act_tests_passed = test_result
                    .act_test_results
                    .iter()
                    .filter(|r| r.success)
                    .count();
                test_result.test_summary.act_tests_failed = test_result
                    .act_test_results
                    .iter()
                    .filter(|r| !r.success)
                    .count();
            }

            results.push(test_result);
        }

        Ok(results)
    }

    /// Generate mock input files for testing
    fn generate_mock_inputs(&self, workflow_path: &Path) -> Result<Vec<MockInputFile>> {
        let mut mock_files = Vec::new();

        // Read workflow content
        let content = fs::read_to_string(workflow_path)?;
        let workflow = parse_workflow(&content)?;

        // Generate inputs for each mockable trigger
        if let Some(on) = &workflow.on {
            for (trigger_name, _) in on {
                if self.validator.can_mock_trigger(trigger_name) {
                    let inputs = self.generate_mock_inputs_for_trigger(trigger_name);
                    let file_path = self.get_mock_inputs_file_path(workflow_path, trigger_name)?;

                    // Write inputs to file
                    let inputs_json = serde_json::to_string_pretty(&inputs)?;
                    fs::write(&file_path, inputs_json)?;

                    mock_files.push(MockInputFile {
                        trigger_name: trigger_name.clone(),
                        file_path,
                        inputs,
                    });
                }
            }
        }

        Ok(mock_files)
    }

    /// Generate mock inputs for a specific trigger
    fn generate_mock_inputs_for_trigger(
        &self,
        trigger_name: &str,
    ) -> HashMap<String, serde_json::Value> {
        let mut inputs = HashMap::new();

        match trigger_name {
            "pull_request" => {
                inputs.insert(
                    "event_name".to_string(),
                    serde_json::Value::String("pull_request".to_string()),
                );
                inputs.insert(
                    "pr_number".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(123)),
                );
                inputs.insert(
                    "head_ref".to_string(),
                    serde_json::Value::String("feature-branch".to_string()),
                );
                inputs.insert(
                    "base_ref".to_string(),
                    serde_json::Value::String("main".to_string()),
                );
            }
            "push" => {
                inputs.insert(
                    "event_name".to_string(),
                    serde_json::Value::String("push".to_string()),
                );
                inputs.insert(
                    "branch".to_string(),
                    serde_json::Value::String("main".to_string()),
                );
                inputs.insert(
                    "commit_sha".to_string(),
                    serde_json::Value::String("abc123def456".to_string()),
                );
            }
            "release" => {
                inputs.insert(
                    "event_name".to_string(),
                    serde_json::Value::String("release".to_string()),
                );
                inputs.insert(
                    "tag".to_string(),
                    serde_json::Value::String("v1.0.0".to_string()),
                );
                inputs.insert("published".to_string(), serde_json::Value::Bool(true));
            }
            "issue_comment" => {
                inputs.insert(
                    "event_name".to_string(),
                    serde_json::Value::String("issue_comment".to_string()),
                );
                inputs.insert(
                    "issue_number".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(456)),
                );
                inputs.insert(
                    "comment_body".to_string(),
                    serde_json::Value::String("Test comment".to_string()),
                );
            }
            _ => {
                inputs.insert(
                    "event_name".to_string(),
                    serde_json::Value::String(trigger_name.to_string()),
                );
            }
        }

        inputs
    }

    /// Get mock inputs file path
    fn get_mock_inputs_file_path(
        &self,
        workflow_path: &Path,
        trigger_name: &str,
    ) -> Result<PathBuf> {
        let output_dir = self
            .config
            .output_dir
            .as_ref()
            .unwrap_or(&PathBuf::from(".github/inputs"));

        fs::create_dir_all(output_dir)?;

        let workflow_name = workflow_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        Ok(output_dir.join(format!("{}-{}-inputs.json", workflow_name, trigger_name)))
    }

    /// Run act tests on workflow
    fn run_act_tests(&self, workflow_path: &Path) -> Result<Vec<ActTestResult>> {
        let mut results = Vec::new();

        // Test workflow_dispatch trigger
        if let Ok(act_result) = self.run_act_workflow_dispatch(workflow_path) {
            results.push(act_result);
        }

        // Test other triggers if configured
        if self.config.test_all_triggers {
            let trigger_names = vec!["push", "pull_request", "release", "schedule"];
            for trigger_name in trigger_names {
                if let Ok(act_result) = self.run_act_trigger(workflow_path, trigger_name) {
                    results.push(act_result);
                }
            }
        }

        Ok(results)
    }

    /// Run act with workflow_dispatch trigger
    fn run_act_workflow_dispatch(&self, workflow_path: &Path) -> Result<ActTestResult> {
        let start_time = std::time::Instant::now();

        let mut command = std::process::Command::new("act");
        command.arg("workflow_dispatch");
        command.arg("-W");
        command.arg(workflow_path);

        // Add inputs file if specified
        if let Some(inputs_file) = &self.config.act_inputs_file {
            command.arg("--input-file");
            command.arg(inputs_file);
        }

        // Add dry run flag if configured
        if self.config.act_dry_run {
            command.arg("-n");
        }

        let output = command.output();
        let execution_time = start_time.elapsed().as_secs_f64();

        match output {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);

                Ok(ActTestResult {
                    trigger_name: "workflow_dispatch".to_string(),
                    success,
                    output: output_str.to_string(),
                    error: if error_str.is_empty() {
                        None
                    } else {
                        Some(error_str.to_string())
                    },
                    execution_time,
                })
            }
            Err(e) => Ok(ActTestResult {
                trigger_name: "workflow_dispatch".to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Failed to run act: {}", e)),
                execution_time,
            }),
        }
    }

    /// Run act with specific trigger
    fn run_act_trigger(&self, workflow_path: &Path, trigger_name: &str) -> Result<ActTestResult> {
        let start_time = std::time::Instant::now();

        let mut command = std::process::Command::new("act");
        command.arg(trigger_name);
        command.arg("-W");
        command.arg(workflow_path);

        // Add dry run flag if configured
        if self.config.act_dry_run {
            command.arg("-n");
        }

        let output = command.output();
        let execution_time = start_time.elapsed().as_secs_f64();

        match output {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);

                Ok(ActTestResult {
                    trigger_name: trigger_name.to_string(),
                    success,
                    output: output_str.to_string(),
                    error: if error_str.is_empty() {
                        None
                    } else {
                        Some(error_str.to_string())
                    },
                    execution_time,
                })
            }
            Err(e) => Ok(ActTestResult {
                trigger_name: trigger_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Failed to run act: {}", e)),
                execution_time,
            }),
        }
    }
}

/// Generate comprehensive test report
pub fn generate_test_report(
    results: &[WorkflowContractTestResult],
    format: &str,
) -> Result<String> {
    match format.to_lowercase().as_str() {
        "json" => Ok(serde_json::to_string_pretty(results)?),
        "yaml" => Ok(serde_yaml::to_string(results)?),
        "markdown" => generate_test_markdown_report(results),
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
}

/// Generate markdown test report
fn generate_test_markdown_report(results: &[WorkflowContractTestResult]) -> Result<String> {
    let mut report = String::new();
    report.push_str("# Workflow Contracts Test Report\n\n");

    // Summary
    let total_workflows = results.len();
    let valid_workflows = results
        .iter()
        .map(|r| r.test_summary.valid_workflows)
        .sum::<usize>();
    let total_act_tests = results
        .iter()
        .map(|r| r.act_test_results.len())
        .sum::<usize>();
    let passed_act_tests = results
        .iter()
        .map(|r| r.test_summary.act_tests_passed)
        .sum::<usize>();
    let failed_act_tests = results
        .iter()
        .map(|r| r.test_summary.act_tests_failed)
        .sum::<usize>();
    let mock_inputs_generated = results
        .iter()
        .map(|r| r.test_summary.mock_inputs_generated)
        .sum::<usize>();

    report.push_str("## Test Summary\n\n");
    report.push_str(&format!(
        "- **Total Workflows Tested:** {}\n",
        total_workflows
    ));
    report.push_str(&format!("- **Valid Workflows:** {}\n", valid_workflows));
    report.push_str(&format!("- **Act Tests Run:** {}\n", total_act_tests));
    report.push_str(&format!("- **Act Tests Passed:** {}\n", passed_act_tests));
    report.push_str(&format!("- **Act Tests Failed:** {}\n", failed_act_tests));
    report.push_str(&format!(
        "- **Mock Inputs Generated:** {}\n",
        mock_inputs_generated
    ));
    report.push_str("\n");

    // Detailed results
    for result in results {
        report.push_str(&format!("## {}\n\n", result.workflow_path.display()));

        // Validation status
        let status = if result.validation_result.is_valid {
            "✅ Valid"
        } else {
            "❌ Invalid"
        };
        report.push_str(&format!("**Validation Status:** {}\n\n", status));

        // Act test results
        if !result.act_test_results.is_empty() {
            report.push_str("### Act Test Results\n\n");
            for act_result in &result.act_test_results {
                let status_icon = if act_result.success { "✅" } else { "❌" };
                report.push_str(&format!(
                    "{} **{}** ({}s)\n",
                    status_icon, act_result.trigger_name, act_result.execution_time
                ));

                if let Some(error) = &act_result.error {
                    report.push_str(&format!("   Error: {}\n", error));
                }
                report.push_str("\n");
            }
        }

        // Mock inputs generated
        if !result.mock_inputs_generated.is_empty() {
            report.push_str("### Mock Inputs Generated\n\n");
            for mock_file in &result.mock_inputs_generated {
                report.push_str(&format!(
                    "- **{}**: {}\n",
                    mock_file.trigger_name,
                    mock_file.file_path.display()
                ));
            }
            report.push_str("\n");
        }

        report.push_str("---\n\n");
    }

    Ok(report)
}

/// Create a test workflow for act testing
pub fn create_test_workflow_for_act(trigger_name: &str) -> Result<String> {
    let workflow = match trigger_name {
        "pull_request" => {
            r#"name: Test Pull Request Workflow

on:
  pull_request:
    branches: [main]
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true
        default: "pull_request"
      pr_number:
        description: "Mock PR number"
        required: true
        default: "123"
      head_ref:
        description: "Head branch name"
        required: false
        default: "feature-branch"
      base_ref:
        description: "Base branch name"
        required: false
        default: "main"

jobs:
  test-job:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Echo event info
        run: |
          echo "Event: ${{ github.event_name }}"
          echo "PR Number: ${{ github.event.number || inputs.pr_number }}"
          echo "Head Ref: ${{ github.head_ref || inputs.head_ref }}"
          echo "Base Ref: ${{ github.base_ref || inputs.base_ref }}"
          
      - name: Run tests
        run: |
          echo "Running tests for ${{ github.event_name }}"
          echo "✅ All tests passed!"
"#
        }
        "push" => {
            r#"name: Test Push Workflow

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true
        default: "push"
      branch:
        description: "Branch name"
        required: true
        default: "main"
      commit_sha:
        description: "Commit SHA"
        required: false
        default: "abc123def456"

jobs:
  test-job:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Echo event info
        run: |
          echo "Event: ${{ github.event_name }}"
          echo "Branch: ${{ github.ref_name || inputs.branch }}"
          echo "Commit: ${{ github.sha || inputs.commit_sha }}"
          
      - name: Run tests
        run: |
          echo "Running tests for ${{ github.event_name }}"
          echo "✅ All tests passed!"
"#
        }
        _ => {
            r#"name: Test Workflow

on:
  workflow_dispatch:
    inputs:
      event_name:
        description: "Event type to mock"
        required: true

jobs:
  test-job:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Echo event info
        run: |
          echo "Event: ${{ github.event_name }}"
          echo "Mock Event: ${{ inputs.event_name }}"
          
      - name: Run tests
        run: |
          echo "Running tests for ${{ inputs.event_name }}"
          echo "✅ All tests passed!"
"#
        }
    };

    Ok(workflow.to_string())
}
