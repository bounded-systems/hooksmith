//! Configuration structures for Lefthook
//!
//! This module provides type-safe Rust structs that correspond to the Lefthook
//! configuration schema. These structs can be serialized to YAML to generate
//! `lefthook.yml` files programmatically.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main Lefthook configuration structure
///
/// This struct represents the root configuration for Lefthook, containing
/// all hook sections and global settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookConfig {
    /// Pre-commit hooks configuration
    #[serde(rename = "pre-commit")]
    pub pre_commit: Option<HookSection>,

    /// Pre-push hooks configuration
    #[serde(rename = "pre-push")]
    pub pre_push: Option<HookSection>,

    /// Commit-msg hooks configuration
    #[serde(rename = "commit-msg")]
    pub commit_msg: Option<HookSection>,

    /// Post-commit hooks configuration
    #[serde(rename = "post-commit")]
    pub post_commit: Option<HookSection>,

    /// Post-merge hooks configuration
    #[serde(rename = "post-merge")]
    pub post_merge: Option<HookSection>,

    /// Pre-rebase hooks configuration
    #[serde(rename = "pre-rebase")]
    pub pre_rebase: Option<HookSection>,

    /// Post-checkout hooks configuration
    #[serde(rename = "post-checkout")]
    pub post_checkout: Option<HookSection>,

    /// Applypatch-msg hooks configuration
    #[serde(rename = "applypatch-msg")]
    pub applypatch_msg: Option<HookSection>,

    /// Pre-applypatch hooks configuration
    #[serde(rename = "pre-applypatch")]
    pub pre_applypatch: Option<HookSection>,

    /// Post-applypatch hooks configuration
    #[serde(rename = "post-applypatch")]
    pub post_applypatch: Option<HookSection>,

    /// Pre-receive hooks configuration
    #[serde(rename = "pre-receive")]
    pub pre_receive: Option<HookSection>,

    /// Update hooks configuration
    pub update: Option<HookSection>,

    /// Post-receive hooks configuration
    #[serde(rename = "post-receive")]
    pub post_receive: Option<HookSection>,

    /// Post-update hooks configuration
    #[serde(rename = "post-update")]
    pub post_update: Option<HookSection>,

    /// Push-to-checkout hooks configuration
    #[serde(rename = "push-to-checkout")]
    pub push_to_checkout: Option<HookSection>,

    /// Pre-auto-gc hooks configuration
    #[serde(rename = "pre-auto-gc")]
    pub pre_auto_gc: Option<HookSection>,

    /// Post-rewrite hooks configuration
    #[serde(rename = "post-rewrite")]
    pub post_rewrite: Option<HookSection>,

    /// Sendemail-validate hooks configuration
    #[serde(rename = "sendemail-validate")]
    pub sendemail_validate: Option<HookSection>,

    /// Fsmonitor-watchman hooks configuration
    #[serde(rename = "fsmonitor-watchman")]
    pub fsmonitor_watchman: Option<HookSection>,

    /// P4-changelist hooks configuration
    #[serde(rename = "p4-changelist")]
    pub p4_changelist: Option<HookSection>,

    /// P4-prepare-changelist hooks configuration
    #[serde(rename = "p4-prepare-changelist")]
    pub p4_prepare_changelist: Option<HookSection>,

    /// P4-post-changelist hooks configuration
    #[serde(rename = "p4-post-changelist")]
    pub p4_post_changelist: Option<HookSection>,

    /// P4-pre-submit hooks configuration
    #[serde(rename = "p4-pre-submit")]
    pub p4_pre_submit: Option<HookSection>,

    /// Post-index-change hooks configuration
    #[serde(rename = "post-index-change")]
    pub post_index_change: Option<HookSection>,

    /// Global configuration settings
    pub config: Option<GlobalConfig>,
}

impl HookConfig {
    /// Write the configuration to a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the YAML file should be written
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the file was written successfully.
    pub async fn write_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let yaml =
            serde_yaml::to_string(self).context("Failed to serialize configuration to YAML")?;

        tokio::fs::write(path, yaml)
            .await
            .context("Failed to write configuration file")?;

        Ok(())
    }

    /// Read configuration from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML file to read
    ///
    /// # Returns
    ///
    /// Returns the parsed configuration.
    pub async fn read_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read configuration file")?;

        let config: Self =
            serde_yaml::from_str(&content).context("Failed to parse configuration YAML")?;

        Ok(config)
    }

    /// Add a pre-commit hook
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the hook
    /// * `job` - Job configuration
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_pre_commit_hook(&mut self, name: String, job: JobConfig) -> &mut Self {
        if self.pre_commit.is_none() {
            self.pre_commit = Some(HookSection::default());
        }

        if let Some(section) = &mut self.pre_commit {
            section.jobs.insert(name, job);
        }

        self
    }

    /// Add a pre-push hook
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the hook
    /// * `job` - Job configuration
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_pre_push_hook(&mut self, name: String, job: JobConfig) -> &mut Self {
        if self.pre_push.is_none() {
            self.pre_push = Some(HookSection::default());
        }

        if let Some(section) = &mut self.pre_push {
            section.jobs.insert(name, job);
        }

        self
    }

    /// Add a commit-msg hook
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the hook
    /// * `job` - Job configuration
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_commit_msg_hook(&mut self, name: String, job: JobConfig) -> &mut Self {
        if self.commit_msg.is_none() {
            self.commit_msg = Some(HookSection::default());
        }

        if let Some(section) = &mut self.commit_msg {
            section.jobs.insert(name, job);
        }

        self
    }
}

/// Configuration for a hook section (e.g., pre-commit, pre-push)
///
/// This struct represents the configuration for a specific Git hook type,
/// containing jobs and execution settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookSection {
    /// Whether to run jobs in parallel
    pub parallel: Option<bool>,

    /// Whether to stage fixed files
    #[serde(rename = "stage_fixed")]
    pub stage_fixed: Option<bool>,

    /// Whether to skip output
    #[serde(rename = "skip_output")]
    pub skip_output: Option<bool>,

    /// Whether to skip output for successful jobs
    #[serde(rename = "skip_output_success")]
    pub skip_output_success: Option<bool>,

    /// Whether to skip output for failed jobs
    #[serde(rename = "skip_output_fail")]
    pub skip_output_fail: Option<bool>,

    /// Whether to skip output for meta information
    #[serde(rename = "skip_output_meta")]
    pub skip_output_meta: Option<bool>,

    /// Whether to skip output for time information
    #[serde(rename = "skip_output_time")]
    pub skip_output_time: Option<bool>,

    /// Whether to skip output for summary information
    #[serde(rename = "skip_output_summary")]
    pub skip_output_summary: Option<bool>,

    /// Jobs to execute
    pub jobs: HashMap<String, JobConfig>,
}

impl HookSection {
    /// Create a new hook section with the given jobs
    ///
    /// # Arguments
    ///
    /// * `jobs` - Jobs to include in the section
    ///
    /// # Returns
    ///
    /// Returns a new `HookSection` instance.
    pub fn new(jobs: HashMap<String, JobConfig>) -> Self {
        Self {
            jobs,
            ..Default::default()
        }
    }

    /// Set whether jobs should run in parallel
    ///
    /// # Arguments
    ///
    /// * `parallel` - Whether to run jobs in parallel
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_parallel(&mut self, parallel: bool) -> &mut Self {
        self.parallel = Some(parallel);
        self
    }

    /// Set whether to stage fixed files
    ///
    /// # Arguments
    ///
    /// * `stage_fixed` - Whether to stage fixed files
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_stage_fixed(&mut self, stage_fixed: bool) -> &mut Self {
        self.stage_fixed = Some(stage_fixed);
        self
    }

    /// Add a job to the section
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the job
    /// * `job` - Job configuration
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_job(&mut self, name: String, job: JobConfig) -> &mut Self {
        self.jobs.insert(name, job);
        self
    }
}

/// Configuration for a single job/hook
///
/// This struct represents the configuration for a single job within a hook
/// section, containing the command to run and execution options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Command to run
    pub run: String,

    /// Files to run on (glob pattern)
    pub files: Option<String>,

    /// Glob patterns for files
    pub glob: Option<Vec<String>>,

    /// Whether to run in parallel
    pub parallel: Option<bool>,

    /// Whether to stage fixed files
    #[serde(rename = "stage_fixed")]
    pub stage_fixed: Option<bool>,

    /// Whether to skip output
    #[serde(rename = "skip_output")]
    pub skip_output: Option<bool>,

    /// Whether to skip output for successful jobs
    #[serde(rename = "skip_output_success")]
    pub skip_output_success: Option<bool>,

    /// Whether to skip output for failed jobs
    #[serde(rename = "skip_output_fail")]
    pub skip_output_fail: Option<bool>,

    /// Whether to skip output for meta information
    #[serde(rename = "skip_output_meta")]
    pub skip_output_meta: Option<bool>,

    /// Whether to skip output for time information
    #[serde(rename = "skip_output_time")]
    pub skip_output_time: Option<bool>,

    /// Whether to skip output for summary information
    #[serde(rename = "skip_output_summary")]
    pub skip_output_summary: Option<bool>,

    /// Environment variables
    pub env: Option<HashMap<String, String>>,

    /// Priority for job execution
    pub priority: Option<i32>,

    /// Fail text to display on failure
    #[serde(rename = "fail_text")]
    pub fail_text: Option<String>,

    /// Tags for the job
    pub tags: Option<Vec<String>>,

    /// Whether to run only on specific branches
    pub branches: Option<Vec<String>>,

    /// Whether to run only on specific remote branches
    #[serde(rename = "remote_branches")]
    pub remote_branches: Option<Vec<String>>,

    /// Whether to run only on specific local branches
    #[serde(rename = "local_branches")]
    pub local_branches: Option<Vec<String>>,

    /// Whether to run only on specific refs
    pub refs: Option<Vec<String>>,
}

impl JobConfig {
    /// Create a new job configuration
    ///
    /// # Arguments
    ///
    /// * `run` - Command to run
    ///
    /// # Returns
    ///
    /// Returns a new `JobConfig` instance.
    pub fn new(run: &str) -> Self {
        Self {
            run: run.to_string(),
            files: None,
            glob: None,
            parallel: None,
            stage_fixed: None,
            skip_output: None,
            skip_output_success: None,
            skip_output_fail: None,
            skip_output_meta: None,
            skip_output_time: None,
            skip_output_summary: None,
            env: None,
            priority: None,
            fail_text: None,
            tags: None,
            branches: None,
            remote_branches: None,
            local_branches: None,
            refs: None,
        }
    }

    /// Set the files pattern for the job
    ///
    /// # Arguments
    ///
    /// * `files` - Files pattern (glob)
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_files(&mut self, files: &str) -> &mut Self {
        self.files = Some(files.to_string());
        self
    }

    /// Set the glob patterns for the job
    ///
    /// # Arguments
    ///
    /// * `glob` - Glob patterns
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_glob(&mut self, glob: Vec<String>) -> &mut Self {
        self.glob = Some(glob);
        self
    }

    /// Set whether to run in parallel
    ///
    /// # Arguments
    ///
    /// * `parallel` - Whether to run in parallel
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_parallel(&mut self, parallel: bool) -> &mut Self {
        self.parallel = Some(parallel);
        self
    }

    /// Set whether to stage fixed files
    ///
    /// # Arguments
    ///
    /// * `stage_fixed` - Whether to stage fixed files
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_stage_fixed(&mut self, stage_fixed: bool) -> &mut Self {
        self.stage_fixed = Some(stage_fixed);
        self
    }

    /// Set environment variables
    ///
    /// # Arguments
    ///
    /// * `env` - Environment variables
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_env(&mut self, env: HashMap<String, String>) -> &mut Self {
        self.env = Some(env);
        self
    }

    /// Set priority for job execution
    ///
    /// # Arguments
    ///
    /// * `priority` - Priority value
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_priority(&mut self, priority: i32) -> &mut Self {
        self.priority = Some(priority);
        self
    }

    /// Set fail text
    ///
    /// # Arguments
    ///
    /// * `fail_text` - Text to display on failure
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_fail_text(&mut self, fail_text: &str) -> &mut Self {
        self.fail_text = Some(fail_text.to_string());
        self
    }

    /// Set tags for the job
    ///
    /// # Arguments
    ///
    /// * `tags` - Tags for the job
    ///
    /// # Returns
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn with_tags(&mut self, tags: Vec<String>) -> &mut Self {
        self.tags = Some(tags);
        self
    }
}

/// Global configuration settings
///
/// This struct represents global configuration settings that apply to all
/// hooks in the Lefthook configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    /// Whether to skip output
    #[serde(rename = "skip_output")]
    pub skip_output: Option<bool>,

    /// Whether to skip output for successful jobs
    #[serde(rename = "skip_output_success")]
    pub skip_output_success: Option<bool>,

    /// Whether to skip output for failed jobs
    #[serde(rename = "skip_output_fail")]
    pub skip_output_fail: Option<bool>,

    /// Whether to skip output for meta information
    #[serde(rename = "skip_output_meta")]
    pub skip_output_meta: Option<bool>,

    /// Whether to skip output for time information
    #[serde(rename = "skip_output_time")]
    pub skip_output_time: Option<bool>,

    /// Whether to skip output for summary information
    #[serde(rename = "skip_output_summary")]
    pub skip_output_summary: Option<bool>,

    /// Whether to show time information
    #[serde(rename = "show_time")]
    pub show_time: Option<bool>,

    /// Whether to show summary information
    #[serde(rename = "show_summary")]
    pub show_summary: Option<bool>,

    /// Whether to show meta information
    #[serde(rename = "show_meta")]
    pub show_meta: Option<bool>,

    /// Whether to show success information
    #[serde(rename = "show_success")]
    pub show_success: Option<bool>,

    /// Whether to show failed information
    #[serde(rename = "show_fail")]
    pub show_fail: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_config_creation() {
        let job = JobConfig::new("cargo test");
        assert_eq!(job.run, "cargo test");
        assert!(job.files.is_none());
        assert!(job.parallel.is_none());
    }

    #[test]
    fn test_job_config_builder() {
        let mut job = JobConfig::new("cargo fmt");
        job.with_files("*.rs");
        job.with_parallel(true);
        job.with_stage_fixed(true);

        assert_eq!(job.run, "cargo fmt");
        assert_eq!(job.files, Some("*.rs".to_string()));
        assert_eq!(job.parallel, Some(true));
        assert_eq!(job.stage_fixed, Some(true));
    }

    #[test]
    fn test_hook_section_creation() {
        let mut jobs = HashMap::new();
        jobs.insert("test".to_string(), JobConfig::new("cargo test"));

        let section = HookSection::new(jobs);
        assert_eq!(section.jobs.len(), 1);
        assert!(section.jobs.contains_key("test"));
    }

    #[test]
    fn test_hook_config_creation() {
        let mut config = HookConfig::default();

        config.add_pre_commit_hook("fmt".to_string(), JobConfig::new("cargo fmt -- --check"));

        assert!(config.pre_commit.is_some());
        let pre_commit = config.pre_commit.as_ref().unwrap();
        assert!(pre_commit.jobs.contains_key("fmt"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = HookConfig::default();
        config.add_pre_commit_hook("test".to_string(), JobConfig::new("cargo test"));

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("pre-commit"));
        assert!(yaml.contains("test"));
        assert!(yaml.contains("cargo test"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
pre-commit:
  parallel: true
  jobs:
    test:
      run: cargo test
      files: "*.rs"
"#;

        let config: HookConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.pre_commit.is_some());

        let pre_commit = config.pre_commit.unwrap();
        assert_eq!(pre_commit.parallel, Some(true));
        assert!(pre_commit.jobs.contains_key("test"));

        let job = &pre_commit.jobs["test"];
        assert_eq!(job.run, "cargo test");
        assert_eq!(job.files, Some("*.rs".to_string()));
    }
}
