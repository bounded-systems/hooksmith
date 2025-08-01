use crate::error::FilterError;
use crate::state::FileState;
use std::collections::HashMap;

/// Represents a Git operation that can trigger hooks
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitOperation {
    /// Git add operation
    Add,
    /// Git checkout operation
    Checkout,
    /// Git diff operation
    Diff,
    /// Git show operation
    Show,
    /// Git log operation
    Log,
    /// Git merge operation
    Merge,
    /// Git rebase operation
    Rebase,
    /// Git archive operation
    Archive,
}

/// Represents a hook-like action that can be performed
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Run a clean/smudge filter
    RunFilterDriver {
        driver_name: String,
        operation: String, // "clean" or "smudge"
    },
    /// Normalize end-of-line characters
    NormalizeEol {
        target_eol: String, // "lf" or "crlf"
    },
    /// Run a custom diff driver
    RunDiffDriver { driver_name: String },
    /// Run a custom merge driver
    RunMergeDriver { driver_name: String },
    /// Enforce encoding
    EnforceEncoding { encoding: String },
    /// Skip file in archive
    SkipInArchive,
    /// Expand placeholders in archive
    ExpandPlaceholders,
    /// Validate safe ASCII characters
    ValidateSafeAscii,
    /// Custom action
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

impl HookAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            HookAction::RunFilterDriver {
                driver_name,
                operation,
            } => {
                format!("Run {} filter driver '{}'", operation, driver_name)
            }
            HookAction::NormalizeEol { target_eol } => {
                format!("Normalize EOL to {}", target_eol)
            }
            HookAction::RunDiffDriver { driver_name } => {
                format!("Run diff driver '{}'", driver_name)
            }
            HookAction::RunMergeDriver { driver_name } => {
                format!("Run merge driver '{}'", driver_name)
            }
            HookAction::EnforceEncoding { encoding } => {
                format!("Enforce encoding '{}'", encoding)
            }
            HookAction::SkipInArchive => "Skip file in archive".to_string(),
            HookAction::ExpandPlaceholders => "Expand placeholders in archive".to_string(),
            HookAction::ValidateSafeAscii => "Validate safe ASCII characters".to_string(),
            HookAction::Custom { name, .. } => {
                format!("Custom action '{}'", name)
            }
        }
    }
}

/// Maps Git attributes to hook actions
#[derive(Debug, Clone)]
pub struct ActionResolver {
    /// Configuration mapping attributes to actions
    attribute_actions: HashMap<String, Vec<HookAction>>,
    /// Whether actions are blocking (fail on error)
    blocking_actions: HashMap<String, bool>,
}

impl Default for ActionResolver {
    fn default() -> Self {
        let mut resolver = Self {
            attribute_actions: HashMap::new(),
            blocking_actions: HashMap::new(),
        };

        // Set up default attribute mappings
        resolver.setup_default_mappings();
        resolver
    }
}

impl ActionResolver {
    /// Create a new ActionResolver with custom configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set up default attribute to action mappings
    fn setup_default_mappings(&mut self) {
        // Filter driver actions
        self.attribute_actions.insert(
            "filter".to_string(),
            vec![HookAction::RunFilterDriver {
                driver_name: "{filter}".to_string(),
                operation: "clean".to_string(),
            }],
        );
        self.blocking_actions.insert("filter".to_string(), true);

        // Text/EOL actions
        self.attribute_actions.insert(
            "text".to_string(),
            vec![HookAction::NormalizeEol {
                target_eol: "lf".to_string(),
            }],
        );
        self.blocking_actions.insert("text".to_string(), false);

        self.attribute_actions.insert(
            "eol".to_string(),
            vec![HookAction::NormalizeEol {
                target_eol: "{eol}".to_string(),
            }],
        );
        self.blocking_actions.insert("eol".to_string(), false);

        // Diff driver actions
        self.attribute_actions.insert(
            "diff".to_string(),
            vec![HookAction::RunDiffDriver {
                driver_name: "{diff}".to_string(),
            }],
        );
        self.blocking_actions.insert("diff".to_string(), false);

        // Merge driver actions
        self.attribute_actions.insert(
            "merge".to_string(),
            vec![HookAction::RunMergeDriver {
                driver_name: "{merge}".to_string(),
            }],
        );
        self.blocking_actions.insert("merge".to_string(), true);

        // Encoding actions
        self.attribute_actions.insert(
            "working-tree-encoding".to_string(),
            vec![HookAction::EnforceEncoding {
                encoding: "{encoding}".to_string(),
            }],
        );
        self.blocking_actions
            .insert("working-tree-encoding".to_string(), true);

        // Export actions
        self.attribute_actions
            .insert("export-ignore".to_string(), vec![HookAction::SkipInArchive]);
        self.blocking_actions
            .insert("export-ignore".to_string(), false);

        self.attribute_actions.insert(
            "export-subst".to_string(),
            vec![HookAction::ExpandPlaceholders],
        );
        self.blocking_actions
            .insert("export-subst".to_string(), false);

        // Custom safe-ascii attribute
        self.attribute_actions.insert(
            "safe-ascii".to_string(),
            vec![HookAction::ValidateSafeAscii],
        );
        self.blocking_actions.insert("safe-ascii".to_string(), true);
    }

    /// Add a custom attribute mapping
    pub fn add_attribute_mapping(
        &mut self,
        attribute: &str,
        actions: Vec<HookAction>,
        blocking: bool,
    ) {
        self.attribute_actions
            .insert(attribute.to_string(), actions);
        self.blocking_actions
            .insert(attribute.to_string(), blocking);
    }

    /// Resolve actions for a file state and Git operation
    pub fn resolve_actions(
        &self,
        file_state: &FileState,
        operation: &GitOperation,
    ) -> Vec<HookAction> {
        let mut actions = Vec::new();

        // Check each attribute and add corresponding actions
        for (attribute, attribute_actions) in &self.attribute_actions {
            if file_state.is_enabled(attribute) {
                for action in attribute_actions {
                    let resolved_action = self.resolve_action_placeholders(action, file_state);
                    if self.should_run_action(&resolved_action, operation) {
                        actions.push(resolved_action);
                    }
                }
            }
        }

        actions
    }

    /// Check if an action should run for a given operation
    fn should_run_action(&self, action: &HookAction, operation: &GitOperation) -> bool {
        match (action, operation) {
            // Filter drivers run on add/checkout
            (HookAction::RunFilterDriver { operation: op, .. }, GitOperation::Add) => op == "clean",
            (HookAction::RunFilterDriver { operation: op, .. }, GitOperation::Checkout) => {
                op == "smudge"
            }

            // EOL normalization runs on add/checkout
            (HookAction::NormalizeEol { .. }, GitOperation::Add | GitOperation::Checkout) => true,

            // Diff drivers run on diff/show/log
            (
                HookAction::RunDiffDriver { .. },
                GitOperation::Diff | GitOperation::Show | GitOperation::Log,
            ) => true,

            // Merge drivers run on merge/rebase
            (HookAction::RunMergeDriver { .. }, GitOperation::Merge | GitOperation::Rebase) => true,

            // Encoding enforcement runs on add/checkout
            (HookAction::EnforceEncoding { .. }, GitOperation::Add | GitOperation::Checkout) => {
                true
            }

            // Export actions run on archive
            (HookAction::SkipInArchive | HookAction::ExpandPlaceholders, GitOperation::Archive) => {
                true
            }

            // Safe ASCII validation runs on add
            (HookAction::ValidateSafeAscii, GitOperation::Add) => true,

            // Custom actions run on all operations by default
            (HookAction::Custom { .. }, _) => true,

            // Default: don't run
            _ => false,
        }
    }

    /// Resolve placeholders in action parameters
    fn resolve_action_placeholders(
        &self,
        action: &HookAction,
        file_state: &FileState,
    ) -> HookAction {
        match action {
            HookAction::RunFilterDriver {
                driver_name,
                operation,
            } => {
                let resolved_driver = self.resolve_placeholder(driver_name, file_state);
                HookAction::RunFilterDriver {
                    driver_name: resolved_driver,
                    operation: operation.clone(),
                }
            }
            HookAction::NormalizeEol { target_eol } => {
                let resolved_eol = self.resolve_placeholder(target_eol, file_state);
                HookAction::NormalizeEol {
                    target_eol: resolved_eol,
                }
            }
            HookAction::RunDiffDriver { driver_name } => {
                let resolved_driver = self.resolve_placeholder(driver_name, file_state);
                HookAction::RunDiffDriver {
                    driver_name: resolved_driver,
                }
            }
            HookAction::RunMergeDriver { driver_name } => {
                let resolved_driver = self.resolve_placeholder(driver_name, file_state);
                HookAction::RunMergeDriver {
                    driver_name: resolved_driver,
                }
            }
            HookAction::EnforceEncoding { encoding } => {
                let resolved_encoding = self.resolve_placeholder(encoding, file_state);
                HookAction::EnforceEncoding {
                    encoding: resolved_encoding,
                }
            }
            action => action.clone(),
        }
    }

    /// Resolve a placeholder string using file state values
    fn resolve_placeholder(&self, placeholder: &str, file_state: &FileState) -> String {
        match placeholder {
            "{filter}" => file_state.get_filter_driver().unwrap_or("").to_string(),
            "{eol}" => file_state.get_eol().unwrap_or("lf").to_string(),
            "{diff}" => file_state.get_value("diff").unwrap_or("").to_string(),
            "{merge}" => file_state.get_value("merge").unwrap_or("").to_string(),
            "{encoding}" => file_state.get_encoding().unwrap_or("utf-8").to_string(),
            _ => placeholder.to_string(),
        }
    }

    /// Check if an attribute's actions are blocking
    pub fn is_blocking(&self, attribute: &str) -> bool {
        self.blocking_actions
            .get(attribute)
            .copied()
            .unwrap_or(false)
    }

    /// Get all attributes that have actions configured
    pub fn get_configured_attributes(&self) -> Vec<&String> {
        self.attribute_actions.keys().collect()
    }
}
