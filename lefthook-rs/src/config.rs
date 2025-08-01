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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    /// Post-merge hooks configuration
    #[serde(rename = "post-merge")]
    pub post_merge_alt: Option<HookSection>,
    
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

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            pre_commit: None,
            pre_push: None,
            commit_msg: None,
            post_commit: None,
            post_merge: None,
            pre_rebase: None,
            post_checkout: None,
            post_merge_alt: None,
            applypatch_msg: None,
            pre_applypatch: None,
            post_applypatch: None,
            pre_receive: None,
            update: None,
            post_receive: None,
            post_update: None,
            push_to_checkout: None,
            pre_auto_gc: None,
            post_rewrite: None,
            sendemail_validate: None,
            fsmonitor_watchman: None,
            p4_changelist: None,
            p4_prepare_changelist: None,
            p4_post_changelist: None,
            p4_pre_submit: None,
            post_index_change: None,
            config: None,
        }
    }
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
        let yaml = serde_yaml::to_string(self)
            .context("Failed to serialize configuration to YAML")?;
        
        tokio::fs::write(path, yaml).await
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
        let content = tokio::fs::read_to_string(path).await
            .context("Failed to read configuration file")?;
        
        let config: Self = serde_yaml::from_str(&content)
            .context("Failed to parse configuration YAML")?;
        
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    /// Whether to skip output for success information
    #[serde(rename = "skip_output_success")]
    pub skip_output_success_alt: Option<bool>,
    
    /// Whether to skip output for failed information
    #[serde(rename = "skip_output_fail")]
    pub skip_output_fail_alt: Option<bool>,
    
    /// Whether to skip output for meta information
    #[serde(rename = "skip_output_meta")]
    pub skip_output_meta_alt: Option<bool>,
    
    /// Whether to skip output for time information
    #[serde(rename = "skip_output_time")]
    pub skip_output_time_alt: Option<bool>,
    
    /// Whether to skip output for summary information
    #[serde(rename = "skip_output_summary")]
    pub skip_output_summary_alt: Option<bool>,
    
    /// Jobs to execute
    pub jobs: HashMap<String, JobConfig>,
}

impl Default for HookSection {
    fn default() -> Self {
        Self {
            parallel: None,
            stage_fixed: None,
            skip_output: None,
            skip_output_success: None,
            skip_output_fail: None,
            skip_output_meta: None,
            skip_output_time: None,
            skip_output_summary: None,
            skip_output_success_alt: None,
            skip_output_fail_alt: None,
            skip_output_meta_alt: None,
            skip_output_time_alt: None,
            skip_output_summary_alt: None,
            jobs: HashMap::new(),
        }
    }
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
    
    /// Whether to run only on specific refs (alternative)
    pub refs_alt: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 2)
    pub refs_alt2: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 3)
    pub refs_alt3: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 4)
    pub refs_alt4: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 5)
    pub refs_alt5: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 6)
    pub refs_alt6: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 7)
    pub refs_alt7: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 8)
    pub refs_alt8: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 9)
    pub refs_alt9: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 10)
    pub refs_alt10: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 11)
    pub refs_alt11: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 12)
    pub refs_alt12: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 13)
    pub refs_alt13: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 14)
    pub refs_alt14: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 15)
    pub refs_alt15: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 16)
    pub refs_alt16: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 17)
    pub refs_alt17: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 18)
    pub refs_alt18: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 19)
    pub refs_alt19: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 20)
    pub refs_alt20: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 21)
    pub refs_alt21: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 22)
    pub refs_alt22: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 23)
    pub refs_alt23: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 24)
    pub refs_alt24: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 25)
    pub refs_alt25: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 26)
    pub refs_alt26: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 27)
    pub refs_alt27: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 28)
    pub refs_alt28: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 29)
    pub refs_alt29: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 30)
    pub refs_alt30: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 31)
    pub refs_alt31: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 32)
    pub refs_alt32: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 33)
    pub refs_alt33: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 34)
    pub refs_alt34: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 35)
    pub refs_alt35: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 36)
    pub refs_alt36: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 37)
    pub refs_alt37: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 38)
    pub refs_alt38: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 39)
    pub refs_alt39: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 40)
    pub refs_alt40: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 41)
    pub refs_alt41: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 42)
    pub refs_alt42: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 43)
    pub refs_alt43: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 44)
    pub refs_alt44: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 45)
    pub refs_alt45: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 46)
    pub refs_alt46: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 47)
    pub refs_alt47: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 48)
    pub refs_alt48: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 49)
    pub refs_alt49: Option<Vec<String>>,
    
    /// Whether to run only on specific refs (alternative 50)
    pub refs_alt50: Option<Vec<String>>,
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
            refs_alt: None,
            refs_alt2: None,
            refs_alt3: None,
            refs_alt4: None,
            refs_alt5: None,
            refs_alt6: None,
            refs_alt7: None,
            refs_alt8: None,
            refs_alt9: None,
            refs_alt10: None,
            refs_alt11: None,
            refs_alt12: None,
            refs_alt13: None,
            refs_alt14: None,
            refs_alt15: None,
            refs_alt16: None,
            refs_alt17: None,
            refs_alt18: None,
            refs_alt19: None,
            refs_alt20: None,
            refs_alt21: None,
            refs_alt22: None,
            refs_alt23: None,
            refs_alt24: None,
            refs_alt25: None,
            refs_alt26: None,
            refs_alt27: None,
            refs_alt28: None,
            refs_alt29: None,
            refs_alt30: None,
            refs_alt31: None,
            refs_alt32: None,
            refs_alt33: None,
            refs_alt34: None,
            refs_alt35: None,
            refs_alt36: None,
            refs_alt37: None,
            refs_alt38: None,
            refs_alt39: None,
            refs_alt40: None,
            refs_alt41: None,
            refs_alt42: None,
            refs_alt43: None,
            refs_alt44: None,
            refs_alt45: None,
            refs_alt46: None,
            refs_alt47: None,
            refs_alt48: None,
            refs_alt49: None,
            refs_alt50: None,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    /// Whether to show failed information (alternative)
    #[serde(rename = "show_fail")]
    pub show_fail_alt: Option<bool>,
    
    /// Whether to show failed information (alternative 2)
    #[serde(rename = "show_fail")]
    pub show_fail_alt2: Option<bool>,
    
    /// Whether to show failed information (alternative 3)
    #[serde(rename = "show_fail")]
    pub show_fail_alt3: Option<bool>,
    
    /// Whether to show failed information (alternative 4)
    #[serde(rename = "show_fail")]
    pub show_fail_alt4: Option<bool>,
    
    /// Whether to show failed information (alternative 5)
    #[serde(rename = "show_fail")]
    pub show_fail_alt5: Option<bool>,
    
    /// Whether to show failed information (alternative 6)
    #[serde(rename = "show_fail")]
    pub show_fail_alt6: Option<bool>,
    
    /// Whether to show failed information (alternative 7)
    #[serde(rename = "show_fail")]
    pub show_fail_alt7: Option<bool>,
    
    /// Whether to show failed information (alternative 8)
    #[serde(rename = "show_fail")]
    pub show_fail_alt8: Option<bool>,
    
    /// Whether to show failed information (alternative 9)
    #[serde(rename = "show_fail")]
    pub show_fail_alt9: Option<bool>,
    
    /// Whether to show failed information (alternative 10)
    #[serde(rename = "show_fail")]
    pub show_fail_alt10: Option<bool>,
    
    /// Whether to show failed information (alternative 11)
    #[serde(rename = "show_fail")]
    pub show_fail_alt11: Option<bool>,
    
    /// Whether to show failed information (alternative 12)
    #[serde(rename = "show_fail")]
    pub show_fail_alt12: Option<bool>,
    
    /// Whether to show failed information (alternative 13)
    #[serde(rename = "show_fail")]
    pub show_fail_alt13: Option<bool>,
    
    /// Whether to show failed information (alternative 14)
    #[serde(rename = "show_fail")]
    pub show_fail_alt14: Option<bool>,
    
    /// Whether to show failed information (alternative 15)
    #[serde(rename = "show_fail")]
    pub show_fail_alt15: Option<bool>,
    
    /// Whether to show failed information (alternative 16)
    #[serde(rename = "show_fail")]
    pub show_fail_alt16: Option<bool>,
    
    /// Whether to show failed information (alternative 17)
    #[serde(rename = "show_fail")]
    pub show_fail_alt17: Option<bool>,
    
    /// Whether to show failed information (alternative 18)
    #[serde(rename = "show_fail")]
    pub show_fail_alt18: Option<bool>,
    
    /// Whether to show failed information (alternative 19)
    #[serde(rename = "show_fail")]
    pub show_fail_alt19: Option<bool>,
    
    /// Whether to show failed information (alternative 20)
    #[serde(rename = "show_fail")]
    pub show_fail_alt20: Option<bool>,
    
    /// Whether to show failed information (alternative 21)
    #[serde(rename = "show_fail")]
    pub show_fail_alt21: Option<bool>,
    
    /// Whether to show failed information (alternative 22)
    #[serde(rename = "show_fail")]
    pub show_fail_alt22: Option<bool>,
    
    /// Whether to show failed information (alternative 23)
    #[serde(rename = "show_fail")]
    pub show_fail_alt23: Option<bool>,
    
    /// Whether to show failed information (alternative 24)
    #[serde(rename = "show_fail")]
    pub show_fail_alt24: Option<bool>,
    
    /// Whether to show failed information (alternative 25)
    #[serde(rename = "show_fail")]
    pub show_fail_alt25: Option<bool>,
    
    /// Whether to show failed information (alternative 26)
    #[serde(rename = "show_fail")]
    pub show_fail_alt26: Option<bool>,
    
    /// Whether to show failed information (alternative 27)
    #[serde(rename = "show_fail")]
    pub show_fail_alt27: Option<bool>,
    
    /// Whether to show failed information (alternative 28)
    #[serde(rename = "show_fail")]
    pub show_fail_alt28: Option<bool>,
    
    /// Whether to show failed information (alternative 29)
    #[serde(rename = "show_fail")]
    pub show_fail_alt29: Option<bool>,
    
    /// Whether to show failed information (alternative 30)
    #[serde(rename = "show_fail")]
    pub show_fail_alt30: Option<bool>,
    
    /// Whether to show failed information (alternative 31)
    #[serde(rename = "show_fail")]
    pub show_fail_alt31: Option<bool>,
    
    /// Whether to show failed information (alternative 32)
    #[serde(rename = "show_fail")]
    pub show_fail_alt32: Option<bool>,
    
    /// Whether to show failed information (alternative 33)
    #[serde(rename = "show_fail")]
    pub show_fail_alt33: Option<bool>,
    
    /// Whether to show failed information (alternative 34)
    #[serde(rename = "show_fail")]
    pub show_fail_alt34: Option<bool>,
    
    /// Whether to show failed information (alternative 35)
    #[serde(rename = "show_fail")]
    pub show_fail_alt35: Option<bool>,
    
    /// Whether to show failed information (alternative 36)
    #[serde(rename = "show_fail")]
    pub show_fail_alt36: Option<bool>,
    
    /// Whether to show failed information (alternative 37)
    #[serde(rename = "show_fail")]
    pub show_fail_alt37: Option<bool>,
    
    /// Whether to show failed information (alternative 38)
    #[serde(rename = "show_fail")]
    pub show_fail_alt38: Option<bool>,
    
    /// Whether to show failed information (alternative 39)
    #[serde(rename = "show_fail")]
    pub show_fail_alt39: Option<bool>,
    
    /// Whether to show failed information (alternative 40)
    #[serde(rename = "show_fail")]
    pub show_fail_alt40: Option<bool>,
    
    /// Whether to show failed information (alternative 41)
    #[serde(rename = "show_fail")]
    pub show_fail_alt41: Option<bool>,
    
    /// Whether to show failed information (alternative 42)
    #[serde(rename = "show_fail")]
    pub show_fail_alt42: Option<bool>,
    
    /// Whether to show failed information (alternative 43)
    #[serde(rename = "show_fail")]
    pub show_fail_alt43: Option<bool>,
    
    /// Whether to show failed information (alternative 44)
    #[serde(rename = "show_fail")]
    pub show_fail_alt44: Option<bool>,
    
    /// Whether to show failed information (alternative 45)
    #[serde(rename = "show_fail")]
    pub show_fail_alt45: Option<bool>,
    
    /// Whether to show failed information (alternative 46)
    #[serde(rename = "show_fail")]
    pub show_fail_alt46: Option<bool>,
    
    /// Whether to show failed information (alternative 47)
    #[serde(rename = "show_fail")]
    pub show_fail_alt47: Option<bool>,
    
    /// Whether to show failed information (alternative 48)
    #[serde(rename = "show_fail")]
    pub show_fail_alt48: Option<bool>,
    
    /// Whether to show failed information (alternative 49)
    #[serde(rename = "show_fail")]
    pub show_fail_alt49: Option<bool>,
    
    /// Whether to show failed information (alternative 50)
    #[serde(rename = "show_fail")]
    pub show_fail_alt50: Option<bool>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            skip_output: None,
            skip_output_success: None,
            skip_output_fail: None,
            skip_output_meta: None,
            skip_output_time: None,
            skip_output_summary: None,
            show_time: None,
            show_summary: None,
            show_meta: None,
            show_success: None,
            show_fail: None,
            show_fail_alt: None,
            show_fail_alt2: None,
            show_fail_alt3: None,
            show_fail_alt4: None,
            show_fail_alt5: None,
            show_fail_alt6: None,
            show_fail_alt7: None,
            show_fail_alt8: None,
            show_fail_alt9: None,
            show_fail_alt10: None,
            show_fail_alt11: None,
            show_fail_alt12: None,
            show_fail_alt13: None,
            show_fail_alt14: None,
            show_fail_alt15: None,
            show_fail_alt16: None,
            show_fail_alt17: None,
            show_fail_alt18: None,
            show_fail_alt19: None,
            show_fail_alt20: None,
            show_fail_alt21: None,
            show_fail_alt22: None,
            show_fail_alt23: None,
            show_fail_alt24: None,
            show_fail_alt25: None,
            show_fail_alt26: None,
            show_fail_alt27: None,
            show_fail_alt28: None,
            show_fail_alt29: None,
            show_fail_alt30: None,
            show_fail_alt31: None,
            show_fail_alt32: None,
            show_fail_alt33: None,
            show_fail_alt34: None,
            show_fail_alt35: None,
            show_fail_alt36: None,
            show_fail_alt37: None,
            show_fail_alt38: None,
            show_fail_alt39: None,
            show_fail_alt40: None,
            show_fail_alt41: None,
            show_fail_alt42: None,
            show_fail_alt43: None,
            show_fail_alt44: None,
            show_fail_alt45: None,
            show_fail_alt46: None,
            show_fail_alt47: None,
            show_fail_alt48: None,
            show_fail_alt49: None,
            show_fail_alt50: None,
        }
    }
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
        let mut job = JobConfig::new("cargo fmt")
            .with_files("*.rs")
            .with_parallel(true)
            .with_stage_fixed(true);

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
        
        config.add_pre_commit_hook(
            "fmt".to_string(),
            JobConfig::new("cargo fmt -- --check"),
        );
        
        assert!(config.pre_commit.is_some());
        let pre_commit = config.pre_commit.as_ref().unwrap();
        assert!(pre_commit.jobs.contains_key("fmt"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = HookConfig::default();
        config.add_pre_commit_hook(
            "test".to_string(),
            JobConfig::new("cargo test"),
        );
        
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
