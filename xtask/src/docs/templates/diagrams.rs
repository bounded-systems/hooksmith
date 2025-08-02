//! Diagrams template for generating Mermaid diagrams from Rust structs

use super::Template;
use std::fmt;

/// Git state for state diagrams
#[derive(Debug, Clone)]
pub struct GitState {
    pub name: String,
    pub description: String,
    pub color: Option<String>,
}

/// State transition for state diagrams
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub label: String,
    pub condition: Option<String>,
}

/// Git state machine diagram
pub struct GitStateMachine {
    pub states: Vec<GitState>,
    pub transitions: Vec<StateTransition>,
    pub title: String,
    pub description: String,
}

impl GitStateMachine {
    /// Create a new Git state machine
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            states: vec![],
            transitions: vec![],
            title: title.to_string(),
            description: description.to_string(),
        }
    }
    
    /// Add a state to the machine
    pub fn add_state(&mut self, state: GitState) {
        self.states.push(state);
    }
    
    /// Add a transition to the machine
    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }
    
    /// Render the state machine as Mermaid
    pub fn render_mermaid(&self) -> String {
        let mut output = String::new();
        
        output.push_str("```mermaid\n");
        output.push_str("stateDiagram-v2\n");
        
        // Render states
        for state in &self.states {
            if let Some(color) = &state.color {
                output.push_str(&format!("    {} : {}\n", state.name, state.description));
                output.push_str(&format!("    note right of {} : {}\n", state.name, color));
            } else {
                output.push_str(&format!("    {} : {}\n", state.name, state.description));
            }
        }

        // Render transitions
        for transition in &self.transitions {
            if let Some(condition) = &transition.condition {
                output.push_str(&format!("    {} --> {} : {}\n", transition.from, transition.to, transition.label));
                output.push_str(&format!("    note right of {} : {}\n", transition.to, condition));
            } else {
                output.push_str(&format!("    {} --> {} : {}\n", transition.from, transition.to, transition.label));
            }
        }
        
        output.push_str("```\n");
        output
    }
    
    /// Create a default Git file state machine
    pub fn default_git_file_states() -> Self {
        let mut machine = Self::new(
            "Git File State Machine",
            "State machine showing the various states a file can be in during Git operations"
        );
        
        // Add states
        machine.add_state(GitState {
            name: "Untracked".to_string(),
            description: "File exists but is not tracked by Git".to_string(),
            color: Some("red".to_string()),
        });
        
        machine.add_state(GitState {
            name: "Staged".to_string(),
            description: "File is staged for commit".to_string(),
            color: Some("green".to_string()),
        });
        
        machine.add_state(GitState {
            name: "Committed".to_string(),
            description: "File is committed to the repository".to_string(),
            color: Some("blue".to_string()),
        });
        
        machine.add_state(GitState {
            name: "Modified".to_string(),
            description: "File has been modified since last commit".to_string(),
            color: Some("orange".to_string()),
        });
        
        // Add transitions
        machine.add_transition(StateTransition {
            from: "Untracked".to_string(),
            to: "Staged".to_string(),
            label: "git add".to_string(),
            condition: None,
        });
        
        machine.add_transition(StateTransition {
            from: "Staged".to_string(),
            to: "Committed".to_string(),
            label: "git commit".to_string(),
            condition: None,
        });
        
        machine.add_transition(StateTransition {
            from: "Committed".to_string(),
            to: "Modified".to_string(),
            label: "Edit file".to_string(),
            condition: None,
        });
        
        machine.add_transition(StateTransition {
            from: "Modified".to_string(),
            to: "Staged".to_string(),
            label: "git add".to_string(),
            condition: None,
        });
        
        machine
    }
}

impl Template for GitStateMachine {
    fn name(&self) -> &str {
        "git_state_machine"
    }
    
    fn validate(&self) -> super::Result<()> {
        if self.title.is_empty() {
            return Err(anyhow::anyhow!("Diagram title cannot be empty"));
        }
        if self.description.is_empty() {
            return Err(anyhow::anyhow!("Diagram description cannot be empty"));
        }
        if self.states.is_empty() {
            return Err(anyhow::anyhow!("State machine must have at least one state"));
        }
        Ok(())
    }
    
    fn render(&self) -> String {
        format!(
            "# {}\n\n{}\n\n{}",
            self.title,
            self.description,
            self.render_mermaid()
        )
    }
}

impl fmt::Display for GitStateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Flow diagram for Git workflows
pub struct GitWorkflowDiagram {
    pub steps: Vec<WorkflowStep>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub description: String,
    pub action: String,
}

impl GitWorkflowDiagram {
    /// Create a new workflow diagram
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            steps: vec![],
            title: title.to_string(),
            description: description.to_string(),
        }
    }
    
    /// Add a step to the workflow
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }
    
    /// Render the workflow as Mermaid
    pub fn render_mermaid(&self) -> String {
        let mut output = String::new();
        
        output.push_str("```mermaid\n");
        output.push_str("flowchart TD\n");
        
        for (i, step) in self.steps.iter().enumerate() {
            let node_id = format!("step{}", i);
            output.push_str(&format!("    {}[{}]\n", node_id, step.name));
            
            if i > 0 {
                let prev_id = format!("step{}", i - 1);
                output.push_str(&format!("    {} --> {}\n", prev_id, node_id));
            }
        }
        
        output.push_str("```\n");
        output
    }
    
    /// Create a default Git commit workflow
    pub fn default_commit_workflow() -> Self {
        let mut workflow = Self::new(
            "Git Commit Workflow",
            "Standard workflow for committing changes to Git"
        );
        
        workflow.add_step(WorkflowStep {
            name: "Edit Files".to_string(),
            description: "Make changes to files".to_string(),
            action: "Edit".to_string(),
        });
        
        workflow.add_step(WorkflowStep {
            name: "Stage Changes".to_string(),
            description: "Add files to staging area".to_string(),
            action: "git add".to_string(),
        });
        
        workflow.add_step(WorkflowStep {
            name: "Commit Changes".to_string(),
            description: "Create a commit with staged changes".to_string(),
            action: "git commit".to_string(),
        });
        
        workflow.add_step(WorkflowStep {
            name: "Push Changes".to_string(),
            description: "Push commits to remote repository".to_string(),
            action: "git push".to_string(),
        });
        
        workflow
    }
}

impl Template for GitWorkflowDiagram {
    fn name(&self) -> &str {
        "git_workflow"
    }
    
    fn validate(&self) -> super::Result<()> {
        if self.title.is_empty() {
            return Err(anyhow::anyhow!("Workflow title cannot be empty"));
        }
        if self.description.is_empty() {
            return Err(anyhow::anyhow!("Workflow description cannot be empty"));
        }
        if self.steps.is_empty() {
            return Err(anyhow::anyhow!("Workflow must have at least one step"));
        }
        Ok(())
    }
    
    fn render(&self) -> String {
        format!(
            "# {}\n\n{}\n\n{}",
            self.title,
            self.description,
            self.render_mermaid()
        )
    }
}

impl fmt::Display for GitWorkflowDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
} 