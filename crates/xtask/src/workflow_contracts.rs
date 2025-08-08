use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Workflow contract validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContractResult {
    pub workflow_path: PathBuf,
    pub is_valid: bool,
    pub concerns: Vec<WorkflowConcern>,
    pub verification_results: Vec<WorkflowVerification>,
    pub audit_trail: Vec<WorkflowAuditEntry>,
}

/// Workflow concern (potential issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConcern {
    pub level: ConcernLevel,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
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
    pub name: String,
    pub passed: bool,
    pub details: String,
    pub contract_metadata: HashMap<String, serde_json::Value>,
}

/// Workflow audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub details: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow contract validator
pub struct WorkflowContractValidator {
    config: WorkflowContractConfig,
}

/// Workflow contract configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContractConfig {
    pub strict_mode: bool,
    pub allow_disabled_workflows: bool,
    pub require_paths: bool,
    pub max_jobs_per_workflow: Option<usize>,
    pub allowed_runners: Vec<String>,
    pub forbidden_actions: Vec<String>,
}

impl Default for WorkflowContractConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_disabled_workflows: true,
            require_paths: false,
            max_jobs_per_workflow: Some(10),
            allowed_runners: vec![
                "ubuntu-latest".to_string(),
                "ubuntu-22.04".to_string(),
                "ubuntu-20.04".to_string(),
            ],
            forbidden_actions: vec![
                "actions/checkout@v1".to_string(),
                "actions/checkout@v2".to_string(),
            ],
        }
    }
}

impl WorkflowContractValidator {
    /// Create a new validator with configuration
    pub fn new(config: WorkflowContractConfig) -> Self {
        Self { config }
    }

    /// Validate a workflow file
    pub fn validate_workflow<P: AsRef<Path>>(
        &self,
        workflow_path: P,
    ) -> Result<WorkflowContractResult> {
        let workflow_path = workflow_path.as_ref();
        let content = fs::read_to_string(workflow_path).with_context(|| {
            format!("Failed to read workflow file: {}", workflow_path.display())
        })?;

        let mut result = WorkflowContractResult {
            workflow_path: workflow_path.to_path_buf(),
            is_valid: true,
            concerns: Vec::new(),
            verification_results: Vec::new(),
            audit_trail: Vec::new(),
        };

        // Parse workflow using gh-workflow-parser
        self.parse_and_validate_workflow(&content, &mut result)?;

        // Run contract validations
        self.validate_contracts(&content, &mut result)?;

        // Audit the workflow
        self.audit_workflow(&content, &mut result)?;

        // Determine overall validity
        result.is_valid = result
            .concerns
            .iter()
            .all(|c| matches!(c.level, ConcernLevel::Info | ConcernLevel::Warning));

        Ok(result)
    }

    /// Parse and validate workflow structure
    fn parse_and_validate_workflow(
        &self,
        content: &str,
        result: &mut WorkflowContractResult,
    ) -> Result<()> {
        // Try to parse with gh-workflow-parser
        match gh_workflow_parser::parse_workflow(content) {
            Ok(workflow) => {
                result.audit_trail.push(WorkflowAuditEntry {
                    timestamp: chrono::Utc::now(),
                    action: "parse_workflow".to_string(),
                    details: "Successfully parsed workflow structure".to_string(),
                    metadata: HashMap::new(),
                });

                // Extract workflow metadata
                self.extract_workflow_metadata(&workflow, result);
            }
            Err(e) => {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Error,
                    message: format!("Failed to parse workflow: {}", e),
                    location: None,
                    suggestion: Some("Check YAML syntax and workflow structure".to_string()),
                });
                result.is_valid = false;
            }
        }

        Ok(())
    }

    /// Extract workflow metadata for contract validation
    fn extract_workflow_metadata(
        &self,
        workflow: &gh_workflow_parser::Workflow,
        result: &mut WorkflowContractResult,
    ) {
        // Extract triggers
        if let Some(on) = &workflow.on {
            self.validate_triggers(on, result);
        }

        // Extract jobs
        if let Some(jobs) = &workflow.jobs {
            self.validate_jobs(jobs, result);
        }

        // Extract permissions
        if let Some(permissions) = &workflow.permissions {
            self.validate_permissions(permissions, result);
        }
    }

    /// Validate workflow triggers
    fn validate_triggers(
        &self,
        triggers: &gh_workflow_parser::Triggers,
        result: &mut WorkflowContractResult,
    ) {
        // Check for disabled workflows (only workflow_dispatch)
        if triggers.workflow_dispatch.is_some() && triggers.len() == 1 {
            result.verification_results.push(WorkflowVerification {
                name: "disabled_workflow".to_string(),
                passed: true,
                details: "Workflow is disabled (manual trigger only)".to_string(),
                contract_metadata: HashMap::new(),
            });
        }

        // Check for potentially expensive triggers
        if triggers.push.is_some() || triggers.pull_request.is_some() {
            result.concerns.push(WorkflowConcern {
                level: ConcernLevel::Warning,
                message: "Workflow triggers on push/pull_request - may incur costs".to_string(),
                location: Some("on:".to_string()),
                suggestion: Some(
                    "Consider using workflow_dispatch only for development".to_string(),
                ),
            });
        }

        // Validate schedule triggers
        if let Some(schedule) = &triggers.schedule {
            for cron in schedule {
                if !self.is_valid_cron(cron) {
                    result.concerns.push(WorkflowConcern {
                        level: ConcernLevel::Error,
                        message: format!("Invalid cron expression: {}", cron),
                        location: Some("on.schedule:".to_string()),
                        suggestion: Some("Use valid cron syntax (e.g., '0 0 * * *')".to_string()),
                    });
                }
            }
        }
    }

    /// Validate workflow jobs
    fn validate_jobs(
        &self,
        jobs: &HashMap<String, gh_workflow_parser::Job>,
        result: &mut WorkflowContractResult,
    ) {
        // Check job count
        if let Some(max_jobs) = self.config.max_jobs_per_workflow {
            if jobs.len() > max_jobs {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Warning,
                    message: format!("Too many jobs: {} (max: {})", jobs.len(), max_jobs),
                    location: Some("jobs:".to_string()),
                    suggestion: Some(
                        "Consider consolidating jobs or splitting workflow".to_string(),
                    ),
                });
            }
        }

        for (job_name, job) in jobs {
            self.validate_job(job_name, job, result);
        }
    }

    /// Validate individual job
    fn validate_job(
        &self,
        job_name: &str,
        job: &gh_workflow_parser::Job,
        result: &mut WorkflowContractResult,
    ) {
        // Validate runner
        if let Some(runs_on) = &job.runs_on {
            if !self.config.allowed_runners.contains(runs_on) {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Warning,
                    message: format!("Job '{}' uses non-standard runner: {}", job_name, runs_on),
                    location: Some(format!("jobs.{}.runs-on:", job_name)),
                    suggestion: Some("Consider using ubuntu-latest for consistency".to_string()),
                });
            }
        }

        // Validate steps
        if let Some(steps) = &job.steps {
            for (step_idx, step) in steps.iter().enumerate() {
                self.validate_step(job_name, step_idx, step, result);
            }
        }

        // Check for paths (if required)
        if self.config.require_paths && job.paths.is_none() {
            result.concerns.push(WorkflowConcern {
                level: ConcernLevel::Info,
                message: format!("Job '{}' has no path restrictions", job_name),
                location: Some(format!("jobs.{}.paths:", job_name)),
                suggestion: Some("Consider adding paths to limit job execution scope".to_string()),
            });
        }
    }

    /// Validate individual step
    fn validate_step(
        &self,
        job_name: &str,
        step_idx: usize,
        step: &gh_workflow_parser::Step,
        result: &mut WorkflowContractResult,
    ) {
        // Check for forbidden actions
        if let Some(uses) = &step.uses {
            if self.config.forbidden_actions.contains(uses) {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Error,
                    message: format!(
                        "Job '{}' step {} uses forbidden action: {}",
                        job_name,
                        step_idx + 1,
                        uses
                    ),
                    location: Some(format!("jobs.{}.steps[{}].uses:", job_name, step_idx)),
                    suggestion: Some("Use a newer version of the action".to_string()),
                });
            }
        }

        // Check for shell injection risks
        if let Some(run) = &step.run {
            if run.contains("${{") && !run.contains("github.") {
                result.concerns.push(WorkflowConcern {
                    level: ConcernLevel::Warning,
                    message: format!(
                        "Job '{}' step {} may have shell injection risk",
                        job_name,
                        step_idx + 1
                    ),
                    location: Some(format!("jobs.{}.steps[{}].run:", job_name, step_idx)),
                    suggestion: Some("Sanitize inputs and use proper quoting".to_string()),
                });
            }
        }
    }

    /// Validate permissions
    fn validate_permissions(
        &self,
        permissions: &HashMap<String, String>,
        result: &mut WorkflowContractResult,
    ) {
        // Check for overly broad permissions
        if permissions.contains_key("contents") && permissions["contents"] == "write" {
            result.concerns.push(WorkflowConcern {
                level: ConcernLevel::Warning,
                message: "Workflow has write access to repository contents".to_string(),
                location: Some("permissions:".to_string()),
                suggestion: Some("Use minimal required permissions for security".to_string()),
            });
        }

        if permissions.contains_key("actions") {
            result.concerns.push(WorkflowConcern {
                level: ConcernLevel::Critical,
                message: "Workflow has access to GitHub Actions - security risk".to_string(),
                location: Some("permissions:".to_string()),
                suggestion: Some(
                    "Remove actions permission unless absolutely necessary".to_string(),
                ),
            });
        }
    }

    /// Validate contracts against workflow
    fn validate_contracts(&self, content: &str, result: &mut WorkflowContractResult) -> Result<()> {
        // Contract: Workflow should be deterministic
        result.verification_results.push(WorkflowVerification {
            name: "deterministic_workflow".to_string(),
            passed: !content.contains("${{") || content.contains("github."),
            details: "Workflow uses deterministic expressions".to_string(),
            contract_metadata: HashMap::new(),
        });

        // Contract: Workflow should be auditable
        result.verification_results.push(WorkflowVerification {
            name: "auditable_workflow".to_string(),
            passed: content.contains("name:") && content.contains("jobs:"),
            details: "Workflow has required metadata for auditing".to_string(),
            contract_metadata: HashMap::new(),
        });

        // Contract: Workflow should be Git-native
        result.verification_results.push(WorkflowVerification {
            name: "git_native_workflow".to_string(),
            passed: content.contains("actions/checkout") || content.contains("git"),
            details: "Workflow integrates with Git operations".to_string(),
            contract_metadata: HashMap::new(),
        });

        Ok(())
    }

    /// Audit workflow for compliance
    fn audit_workflow(&self, content: &str, result: &mut WorkflowContractResult) -> Result<()> {
        // Audit: Check for security best practices
        if content.contains("GITHUB_TOKEN") && !content.contains("permissions:") {
            result.audit_trail.push(WorkflowAuditEntry {
                timestamp: chrono::Utc::now(),
                action: "security_audit".to_string(),
                details: "Workflow uses GITHUB_TOKEN without explicit permissions".to_string(),
                metadata: HashMap::new(),
            });
        }

        // Audit: Check for cost optimization
        if content.contains("runs-on:") && !content.contains("ubuntu-latest") {
            result.audit_trail.push(WorkflowAuditEntry {
                timestamp: chrono::Utc::now(),
                action: "cost_audit".to_string(),
                details: "Workflow uses non-standard runner - may affect costs".to_string(),
                metadata: HashMap::new(),
            });
        }

        // Audit: Check for maintainability
        if content.lines().count() > 200 {
            result.audit_trail.push(WorkflowAuditEntry {
                timestamp: chrono::Utc::now(),
                action: "maintainability_audit".to_string(),
                details: "Workflow is large - consider splitting into reusable workflows"
                    .to_string(),
                metadata: HashMap::new(),
            });
        }

        Ok(())
    }

    /// Validate cron expression
    fn is_valid_cron(&self, cron: &str) -> bool {
        // Basic cron validation (5 or 6 fields)
        let parts: Vec<&str> = cron.split_whitespace().collect();
        parts.len() == 5 || parts.len() == 6
    }

    /// Generate a disabled workflow stub
    pub fn generate_disabled_workflow_stub(&self, name: &str) -> String {
        format!(
            r#"# Hooksmith Disabled Workflow Stub
# Generated by: WorkflowContractValidator
# Purpose: Safe workflow stub that only runs manually

name: {name}
on:
  workflow_dispatch:  # Manual trigger only

jobs:
  noop:
    runs-on: ubuntu-latest
    steps:
      - name: No Operation
        run: echo "This workflow is disabled and only runs when manually triggered"
      - name: Hooksmith Contract Validation
        run: echo "Workflow contracts: deterministic, auditable, git-native"
"#
        )
    }

    /// Validate multiple workflows in a directory
    pub fn validate_workflows<P: AsRef<Path>>(
        &self,
        workflows_dir: P,
    ) -> Result<Vec<WorkflowContractResult>> {
        let workflows_dir = workflows_dir.as_ref();
        let mut results = Vec::new();

        if workflows_dir.is_dir() {
            for entry in fs::read_dir(workflows_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("yml")
                    || path.extension().and_then(|s| s.to_str()) == Some("yaml")
                {
                    match self.validate_workflow(&path) {
                        Ok(result) => results.push(result),
                        Err(e) => {
                            eprintln!("Failed to validate {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Generate workflow contracts report
pub fn generate_workflow_contracts_report(results: &[WorkflowContractResult]) -> String {
    let mut report = String::new();
    report.push_str("# Hooksmith Workflow Contracts Report\n\n");

    let total_workflows = results.len();
    let valid_workflows = results.iter().filter(|r| r.is_valid).count();
    let total_concerns = results.iter().map(|r| r.concerns.len()).sum::<usize>();

    report.push_str(&format!("## Summary\n"));
    report.push_str(&format!("- Total workflows: {}\n", total_workflows));
    report.push_str(&format!("- Valid workflows: {}\n", valid_workflows));
    report.push_str(&format!("- Total concerns: {}\n\n", total_concerns));

    for result in results {
        report.push_str(&format!("## {}\n", result.workflow_path.display()));
        report.push_str(&format!(
            "Status: {}\n\n",
            if result.is_valid {
                "✅ Valid"
            } else {
                "❌ Invalid"
            }
        ));

        if !result.concerns.is_empty() {
            report.push_str("### Concerns\n");
            for concern in &result.concerns {
                report.push_str(&format!(
                    "- **{}**: {}\n",
                    match concern.level {
                        ConcernLevel::Info => "ℹ️ Info",
                        ConcernLevel::Warning => "⚠️ Warning",
                        ConcernLevel::Error => "❌ Error",
                        ConcernLevel::Critical => "🚨 Critical",
                    },
                    concern.message
                ));
                if let Some(suggestion) = &concern.suggestion {
                    report.push_str(&format!("  - Suggestion: {}\n", suggestion));
                }
            }
            report.push_str("\n");
        }

        if !result.verification_results.is_empty() {
            report.push_str("### Contract Verifications\n");
            for verification in &result.verification_results {
                report.push_str(&format!(
                    "- **{}**: {} ({})\n",
                    verification.name,
                    if verification.passed {
                        "✅ Passed"
                    } else {
                        "❌ Failed"
                    },
                    verification.details
                ));
            }
            report.push_str("\n");
        }
    }

    report
}
