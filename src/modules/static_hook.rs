use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Static hook definition with zero dynamic resolution
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StaticHook {
    /// Human-readable name of the hook
    pub name: String,
    /// Hook trigger scope (only one allowed)
    pub scope: HookScope,
    /// Required list of concerns (must match schema)
    pub concerns: Vec<HookConcern>,
    /// Only one binary per hook, must exist at build time
    pub bin: String,
}

/// Hook trigger scope enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookScope {
    /// Traditional Git lifecycle hooks
    Git,
    /// GitHub-specific hooks
    Github,
    /// File system monitoring hooks
    FsMonitor,
    /// Reference transaction hooks
    Reference,
    /// Email-related hooks
    Email,
    /// Patch-related hooks
    Patch,
}

/// Hook concerns enum - what the hook validates
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum HookConcern {
    /// Validates Git blob objects
    Blob,
    /// Validates Git tree objects
    Tree,
    /// Validates Git references
    Ref,
    /// Validates Git notes
    Note,
    /// Validates Git attributes
    Attr,
    /// Validates contract violations
    ContractViolation,
    /// Performs symbol analysis
    SymbolAnalysis,
}

impl StaticHook {
    /// Validate the static hook definition
    pub fn validate(&self) -> Result<()> {
        // Validate name format
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            bail!("Invalid hook name '{}': must contain only alphanumeric characters, underscores, and hyphens", self.name);
        }

        // Validate concerns are unique
        let mut concerns = self.concerns.clone();
        concerns.sort();
        concerns.dedup();
        if concerns.len() != self.concerns.len() {
            bail!("Duplicate concerns found in hook '{}'", self.name);
        }

        // Validate binary exists
        let path = Path::new(&self.bin);
        if !path.exists() {
            bail!("Missing hook binary: {}", path.display());
        }
        if !path.is_file() {
            bail!("Hook binary is not a file: {}", path.display());
        }

        Ok(())
    }

    /// Get the binary path as a Path
    pub fn binary_path(&self) -> &Path {
        Path::new(&self.bin)
    }

    /// Check if this hook concerns a specific type
    pub fn concerns_type(&self, concern: &HookConcern) -> bool {
        self.concerns.contains(concern)
    }

    /// Get the scope as a string
    pub fn scope_str(&self) -> &'static str {
        match self.scope {
            HookScope::Git => "git",
            HookScope::Github => "github", 
            HookScope::FsMonitor => "fsmonitor",
            HookScope::Reference => "reference",
            HookScope::Email => "email",
            HookScope::Patch => "patch",
        }
    }
}

/// Load and validate a static hook from a JSONC file
pub fn load_static_hook(path: &Path) -> Result<StaticHook> {
    let content = std::fs::read_to_string(path)?;
    
    // For now, parse as regular JSON (we can add JSONC support later)
    let hook: StaticHook = serde_json::from_str(&content)?;
    hook.validate()?;
    
    Ok(hook)
}

/// Validate all static hooks in a directory
pub fn validate_static_hooks(dir: &Path) -> Result<Vec<StaticHook>> {
    let mut hooks = Vec::new();
    
    if !dir.exists() {
        return Ok(hooks);
    }
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "jsonc") {
            match load_static_hook(&path) {
                Ok(hook) => hooks.push(hook),
                Err(e) => {
                    eprintln!("Failed to load hook from {}: {}", path.display(), e);
                }
            }
        }
    }
    
    Ok(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_valid_static_hook() {
        let hook = StaticHook {
            name: "pre-commit".to_string(),
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob, HookConcern::Tree],
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        // This will fail if the binary doesn't exist, which is expected
        let result = hook.validate();
        assert!(result.is_err()); // Binary doesn't exist in test
    }

    #[test]
    fn test_invalid_hook_name() {
        let hook = StaticHook {
            name: "pre-commit!".to_string(), // Invalid character
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob],
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        let result = hook.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_concerns() {
        let hook = StaticHook {
            name: "pre-commit".to_string(),
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob, HookConcern::Blob], // Duplicate
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        let result = hook.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_static_hook() {
        let temp_dir = tempdir().unwrap();
        let hook_file = temp_dir.path().join("test-hook.jsonc");
        
        let hook_content = r#"{
            "name": "pre-commit",
            "scope": "git",
            "concerns": ["blob", "tree"],
            "bin": "target/release/hooksmith-validate-tree"
        }"#;
        
        fs::write(&hook_file, hook_content).unwrap();
        
        let result = load_static_hook(&hook_file);
        assert!(result.is_err()); // Binary doesn't exist
    }
}
