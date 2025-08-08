//! Core Repair Planning Module
//! 
//! This module provides the foundational components for the repair planning system:
//! - RepairPlan: Structured repair plans with constraint-safe semantics
//! - Fixer trait: Composable fixer implementations
//! - RepairAction: JSON-serializable action definitions
//! - Validation: Graph invariants and plan validation
//! - Mermaid export: Visualization support

use crate::modules::functional_contract_pipeline::symbols::ConcernSymbol;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use thiserror::Error;

/// Error types for repair planning operations
#[derive(Error, Debug)]
pub enum RepairError {
    /// Invalid repair action detected
    #[error("Invalid repair action: {0}")]
    InvalidAction(String),
    /// Circular dependency detected in repair plan
    #[error("Circular dependency detected in repair plan")]
    CircularDependency,
    /// Invalid fixer ID provided
    #[error("Invalid fixer ID: {0}")]
    InvalidFixer(String),
    /// Plan validation failed
    #[error("Plan validation failed: {0}")]
    ValidationFailed(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for repair operations
pub type RepairResult<T> = Result<T, RepairError>;

/// Represents a specific repair action to be executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepairAction {
    /// Unique identifier for this action
    pub id: String,
    /// ID of the fixer that will execute this action
    pub fixer_id: String,
    /// Type of action to perform
    pub action_type: ActionType,
    /// Target path for the action
    pub path: String,
    /// Action-specific parameters
    pub params: HashMap<String, serde_json::Value>,
    /// Whether this action is required (vs optional)
    pub required: bool,
    /// Priority level (lower = higher priority)
    pub priority: u32,
    /// IDs of actions this depends on
    pub dependencies: Vec<String>,
}

/// Types of repair actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Edit a file at specific line/column
    Edit { 
        /// Line number to edit
        line: u32, 
        /// Column number to edit
        column: u32, 
        /// Content to insert
        content: String 
    },
    /// Replace entire file content
    Replace { 
        /// New file content
        content: String 
    },
    /// Delete a file
    Delete,
    /// Reorder lines in a file
    ReorderLines { 
        /// Strategy for reordering (e.g., "alphabetical")
        strategy: String 
    },
    /// Run a command
    RunCommand { 
        /// Command to run
        command: String, 
        /// Command arguments
        args: Vec<String> 
    },
    /// Apply a patch
    ApplyPatch { 
        /// Patch content to apply
        patch: String 
    },
    /// Create a new file
    Create { 
        /// File content to create
        content: String 
    },
    /// Move/rename a file
    Move { 
        /// New path for the file
        new_path: String 
    },
}

/// Result of executing a repair action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// ID of the action that was executed
    pub action_id: String,
    /// Whether the action succeeded
    pub success: bool,
    /// Messages from the fixer
    pub messages: Vec<String>,
    /// Diff showing what changed
    pub diff: Option<String>,
    /// New hash of the modified object
    pub new_hash: Option<String>,
}

/// Complete repair plan for a failing concern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    /// Unique identifier for this plan
    pub id: String,
    /// The concern being repaired
    pub concern: ConcernSymbol,
    /// Contract that was violated
    pub contract: String,
    /// Violation details
    pub violation: Violation,
    /// Root cause analysis
    pub root_cause: RootCause,
    /// Dispatcher that created this plan
    pub dispatcher: String,
    /// Ordered list of repair actions
    pub actions: Vec<RepairAction>,
    /// Whether the plan is complete and ready for execution
    pub is_complete: bool,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when plan was created
    pub timestamp: String,
}

/// Represents a contract violation that needs repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The concern that violated the contract
    pub concern: ConcernSymbol,
    /// Contract that was violated
    pub contract: String,
    /// Human-readable violation message
    pub message: String,
    /// Location of the violation (file:line:column)
    pub location: String,
    /// Severity level
    pub severity: ViolationSeverity,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationSeverity {
    /// Error severity level
    Error,
    /// Warning severity level
    Warning,
    /// Info severity level
    Info,
}

/// Result of root cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// Primary cause of the violation
    pub primary_cause: String,
    /// Contributing factors
    pub factors: Vec<String>,
    /// Categories of fixes that could resolve this
    pub fix_categories: Vec<FixCategory>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// Categories of fixes that can be applied
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FixCategory {
    /// Formatting fixes (indentation, spacing, etc.)
    Formatting,
    /// Linting fixes (code quality, style violations)
    Linting,
    /// Structural fixes (file organization, naming)
    Structural,
    /// Configuration fixes (settings, options)
    Configuration,
    /// Content fixes (actual file content changes)
    Content,
    /// Tool-specific fixes (external tool integration)
    ToolSpecific,
}

/// Trait for fixer implementations
pub trait Fixer: Send + Sync {
    /// Unique identifier for this fixer
    fn id(&self) -> &'static str;
    
    /// Plan a repair action for the given violation
    fn plan(&self, violation: &Violation, root_cause: &RootCause) -> RepairResult<Option<RepairAction>>;
    
    /// Execute a repair action
    fn execute(&self, action: &RepairAction) -> RepairResult<ActionResult>;
    
    /// Check if this fixer can handle the given violation
    fn can_handle(&self, violation: &Violation) -> bool {
        // Default implementation - override in specific fixers
        true
    }
}

/// Validates a repair plan for correctness
pub struct PlanValidator;

impl PlanValidator {
    /// Validates a repair plan for structural correctness
    pub fn validate(plan: &RepairPlan) -> RepairResult<()> {
        // Check for circular dependencies
        Self::check_circular_dependencies(plan)?;
        
        // Check for invalid action references
        Self::check_action_references(plan)?;
        
        // Check for required actions
        Self::check_required_actions(plan)?;
        
        Ok(())
    }
    
    /// Checks for circular dependencies in the action graph
    fn check_circular_dependencies(plan: &RepairPlan) -> RepairResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut action_map = HashMap::new();
        
        // Build action map
        for action in &plan.actions {
            action_map.insert(action.id.as_str(), action);
        }
        
        // DFS to detect cycles
        for action in &plan.actions {
            if !visited.contains(&action.id) {
                if Self::has_cycle(&action.id, &action_map, &mut visited, &mut rec_stack)? {
                    return Err(RepairError::CircularDependency);
                }
            }
        }
        
        Ok(())
    }
    
    /// DFS helper to detect cycles
    fn has_cycle(
        action_id: &str,
        action_map: &HashMap<&str, &RepairAction>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> RepairResult<bool> {
        visited.insert(action_id.to_string());
        rec_stack.insert(action_id.to_string());
        
        if let Some(action) = action_map.get(action_id) {
            for dep_id in &action.dependencies {
                if !visited.contains(dep_id) {
                    if Self::has_cycle(dep_id, action_map, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dep_id) {
                    return Ok(true);
                }
            }
        }
        
        rec_stack.remove(action_id);
        Ok(false)
    }
    
    /// Checks that all action references are valid
    fn check_action_references(plan: &RepairPlan) -> RepairResult<()> {
        let action_ids: HashSet<_> = plan.actions.iter().map(|a| &a.id).collect();
        
        for action in &plan.actions {
            for dep_id in &action.dependencies {
                if !action_ids.contains(dep_id) {
                    return Err(RepairError::InvalidAction(
                        format!("Action {} depends on non-existent action {}", action.id, dep_id)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Checks that required actions are present
    fn check_required_actions(plan: &RepairPlan) -> RepairResult<()> {
        for action in &plan.actions {
            if action.required && action.fixer_id.is_empty() {
                return Err(RepairError::InvalidAction(
                    format!("Required action {} has no fixer", action.id)
                ));
            }
        }
        
        Ok(())
    }
}

/// Exports repair plans to Mermaid diagrams
pub struct MermaidExporter;

impl MermaidExporter {
    /// Converts a repair plan to a Mermaid flowchart
    pub fn export_plan(plan: &RepairPlan) -> String {
        let mut mermaid = String::new();
        mermaid.push_str("flowchart TD\n");
        
        // Add violation node
        mermaid.push_str(&format!("    Violation[\"Violation: {}\"]\n", 
            plan.violation.message.replace("\"", "\\\"")));
        
        // Add root cause node
        mermaid.push_str(&format!("    RootCause[\"Root Cause: {}\"]\n", 
            plan.root_cause.primary_cause.replace("\"", "\\\"")));
        
        // Add dispatcher node
        mermaid.push_str(&format!("    Dispatcher[\"Dispatcher: {}\"]\n", plan.dispatcher));
        
        // Add action nodes
        for action in &plan.actions {
            let node_id = action.id.replace("-", "_");
            mermaid.push_str(&format!("    {node_id}[\"{}: {}\"]\n", 
                action.fixer_id, action.action_type.to_string()));
        }
        
        // Add edges
        mermaid.push_str("    Violation --> RootCause\n");
        mermaid.push_str("    RootCause --> Dispatcher\n");
        
        for action in &plan.actions {
            let node_id = action.id.replace("-", "_");
            mermaid.push_str(&format!("    Dispatcher --> {node_id}\n"));
            
            // Add dependency edges
            for dep_id in &action.dependencies {
                let dep_node_id = dep_id.replace("-", "_");
                mermaid.push_str(&format!("    {dep_node_id} --> {node_id}\n"));
            }
        }
        
        mermaid
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Edit { line, column, .. } => write!(f, "Edit at {}:{}", line, column),
            ActionType::Replace { .. } => write!(f, "Replace content"),
            ActionType::Delete => write!(f, "Delete file"),
            ActionType::ReorderLines { strategy } => write!(f, "Reorder lines ({})", strategy),
            ActionType::RunCommand { command, .. } => write!(f, "Run: {}", command),
            ActionType::ApplyPatch { .. } => write!(f, "Apply patch"),
            ActionType::Create { .. } => write!(f, "Create file"),
            ActionType::Move { new_path } => write!(f, "Move to {}", new_path),
        }
    }
}

/// Example fixer implementations

/// Fixer that replaces root-level wildcard patterns in .gitignore
pub struct ReplaceRootStarFixer;

impl Fixer for ReplaceRootStarFixer {
    fn id(&self) -> &'static str {
        "fixer.replace-root-star"
    }
    
    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> RepairResult<Option<RepairAction>> {
        if violation.message.contains("wildcard") && violation.message.contains(".gitignore") {
            Ok(Some(RepairAction {
                id: "replace-root-star".to_string(),
                fixer_id: self.id().to_string(),
                action_type: ActionType::Edit {
                    line: 1,
                    column: 1,
                    content: "/*\n".to_string(),
                },
                path: ".gitignore".to_string(),
                params: HashMap::new(),
                required: true,
                priority: 1,
                dependencies: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }
    
    fn execute(&self, action: &RepairAction) -> RepairResult<ActionResult> {
        // Implementation would read the file, make the edit, and return result
        Ok(ActionResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Replaced root wildcard pattern".to_string()],
            diff: Some("/*\n".to_string()),
            new_hash: Some("abc123".to_string()),
        })
    }
    
    fn can_handle(&self, _violation: &Violation) -> bool {
        true
    }
}

/// Fixer that reorders lines in .gitignore files
pub struct LintIgnoreOrderFixer;

impl Fixer for LintIgnoreOrderFixer {
    fn id(&self) -> &'static str {
        "fixer.lint-ignore-order"
    }
    
    fn plan(&self, violation: &Violation, _root_cause: &RootCause) -> RepairResult<Option<RepairAction>> {
        if violation.message.contains(".gitignore") && violation.message.contains("order") {
            Ok(Some(RepairAction {
                id: "reorder-ignore-lines".to_string(),
                fixer_id: self.id().to_string(),
                action_type: ActionType::ReorderLines {
                    strategy: "alphabetical".to_string(),
                },
                path: ".gitignore".to_string(),
                params: HashMap::new(),
                required: false,
                priority: 2,
                dependencies: vec!["replace-root-star".to_string()],
            }))
        } else {
            Ok(None)
        }
    }
    
    fn execute(&self, action: &RepairAction) -> RepairResult<ActionResult> {
        Ok(ActionResult {
            action_id: action.id.clone(),
            success: true,
            messages: vec!["Reordered .gitignore lines alphabetically".to_string()],
            diff: Some("+ *.log\n+ *.tmp\n- *.tmp\n- *.log\n".to_string()),
            new_hash: Some("def456".to_string()),
        })
    }
    
    fn can_handle(&self, _violation: &Violation) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repair_action_serialization() {
        let action = RepairAction {
            id: "test-action".to_string(),
            fixer_id: "test-fixer".to_string(),
            action_type: ActionType::Edit {
                line: 1,
                column: 1,
                content: "test".to_string(),
            },
            path: "test.txt".to_string(),
            params: HashMap::new(),
            required: true,
            priority: 1,
            dependencies: Vec::new(),
        };
        
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: RepairAction = serde_json::from_str(&json).unwrap();
        
        assert_eq!(action, deserialized);
    }
    
    #[test]
    fn test_plan_validation() {
        let plan = RepairPlan {
            id: "test-plan".to_string(),
            concern: ConcernSymbol::TreeFile,
            contract: "test-contract".to_string(),
            violation: Violation {
                concern: ConcernSymbol::TreeFile,
                contract: "test-contract".to_string(),
                message: "Test violation".to_string(),
                location: "test.txt:1".to_string(),
                severity: ViolationSeverity::Error,
            },
            root_cause: RootCause {
                primary_cause: "Test cause".to_string(),
                factors: Vec::new(),
                fix_categories: vec![FixCategory::Formatting],
                confidence: 0.8,
            },
            dispatcher: "test-dispatcher".to_string(),
            actions: vec![
                RepairAction {
                    id: "action1".to_string(),
                    fixer_id: "fixer1".to_string(),
                    action_type: ActionType::Edit {
                        line: 1,
                        column: 1,
                        content: "test".to_string(),
                    },
                    path: "test.txt".to_string(),
                    params: HashMap::new(),
                    required: true,
                    priority: 1,
                    dependencies: Vec::new(),
                },
            ],
            is_complete: true,
            metadata: HashMap::new(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        
        assert!(PlanValidator::validate(&plan).is_ok());
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let plan = RepairPlan {
            id: "test-plan".to_string(),
            concern: ConcernSymbol::TreeFile,
            contract: "test-contract".to_string(),
            violation: Violation {
                concern: ConcernSymbol::TreeFile,
                contract: "test-contract".to_string(),
                message: "Test violation".to_string(),
                location: "test.txt:1".to_string(),
                severity: ViolationSeverity::Error,
            },
            root_cause: RootCause {
                primary_cause: "Test cause".to_string(),
                factors: Vec::new(),
                fix_categories: vec![FixCategory::Formatting],
                confidence: 0.8,
            },
            dispatcher: "test-dispatcher".to_string(),
            actions: vec![
                RepairAction {
                    id: "action1".to_string(),
                    fixer_id: "fixer1".to_string(),
                    action_type: ActionType::Edit {
                        line: 1,
                        column: 1,
                        content: "test".to_string(),
                    },
                    path: "test.txt".to_string(),
                    params: HashMap::new(),
                    required: true,
                    priority: 1,
                    dependencies: vec!["action2".to_string()],
                },
                RepairAction {
                    id: "action2".to_string(),
                    fixer_id: "fixer2".to_string(),
                    action_type: ActionType::Edit {
                        line: 2,
                        column: 1,
                        content: "test2".to_string(),
                    },
                    path: "test.txt".to_string(),
                    params: HashMap::new(),
                    required: true,
                    priority: 2,
                    dependencies: vec!["action1".to_string()],
                },
            ],
            is_complete: true,
            metadata: HashMap::new(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        
        assert!(PlanValidator::validate(&plan).is_err());
    }
    
    #[test]
    fn test_mermaid_export() {
        let plan = RepairPlan {
            id: "test-plan".to_string(),
            concern: ConcernSymbol::TreeFile,
            contract: "test-contract".to_string(),
            violation: Violation {
                concern: ConcernSymbol::TreeFile,
                contract: "test-contract".to_string(),
                message: "Test violation".to_string(),
                location: "test.txt:1".to_string(),
                severity: ViolationSeverity::Error,
            },
            root_cause: RootCause {
                primary_cause: "Test cause".to_string(),
                factors: Vec::new(),
                fix_categories: vec![FixCategory::Formatting],
                confidence: 0.8,
            },
            dispatcher: "test-dispatcher".to_string(),
            actions: vec![
                RepairAction {
                    id: "test-action".to_string(),
                    fixer_id: "test-fixer".to_string(),
                    action_type: ActionType::Edit {
                        line: 1,
                        column: 1,
                        content: "test".to_string(),
                    },
                    path: "test.txt".to_string(),
                    params: HashMap::new(),
                    required: true,
                    priority: 1,
                    dependencies: Vec::new(),
                },
            ],
            is_complete: true,
            metadata: HashMap::new(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let mermaid = MermaidExporter::export_plan(&plan);
        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("Violation"));
        assert!(mermaid.contains("RootCause"));
        assert!(mermaid.contains("Dispatcher"));
        assert!(mermaid.contains("test_action"));
    }
}
