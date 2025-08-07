use std::path::Path;
use std::process;

// Add serde dependencies for build script

fn main() {
    println!("cargo:rerun-if-changed=.hooksmith/hooks/");
    println!("cargo:rerun-if-changed=target/release/");
    
    // Skip validation during build process to avoid circular dependency
    if std::env::var("SKIP_HOOK_VALIDATION").is_ok() {
        return;
    }
    
    // Validate all static hooks at build time
    if let Err(e) = validate_static_hooks() {
        eprintln!("❌ Static hook validation failed: {}", e);
        process::exit(1);
    }
    
    println!("✅ All static hooks validated successfully");
}

fn validate_static_hooks() -> Result<(), Box<dyn std::error::Error>> {
    let hooks_dir = Path::new(".hooksmith/hooks");
    
    if !hooks_dir.exists() {
        println!("⚠️  No .hooksmith/hooks directory found - skipping validation");
        return Ok(());
    }
    
    let mut total_hooks = 0;
    let mut valid_hooks = 0;
    let mut errors = Vec::new();
    
    // Walk through all scope directories
    for scope_entry in std::fs::read_dir(hooks_dir)? {
        let scope_entry = scope_entry?;
        let scope_path = scope_entry.path();
        
        if scope_path.is_dir() {
            let _scope_name = scope_path.file_name().unwrap().to_string_lossy();
            
            // Walk through all hook files in this scope
            for hook_entry in std::fs::read_dir(&scope_path)? {
                let hook_entry = hook_entry?;
                let hook_path = hook_entry.path();
                
                if hook_path.is_file() && hook_path.extension().map_or(false, |ext| ext == "jsonc") {
                    total_hooks += 1;
                    
                    match validate_single_hook(&hook_path) {
                        Ok(_) => {
                            valid_hooks += 1;
                            println!("✅ Validated: {}", hook_path.display());
                        }
                        Err(e) => {
                            errors.push(format!("{}: {}", hook_path.display(), e));
                        }
                    }
                }
            }
        }
    }
    
    // Report results
    println!("📊 Static Hook Validation Summary:");
    println!("   Total hooks found: {}", total_hooks);
    println!("   Valid hooks: {}", valid_hooks);
    println!("   Invalid hooks: {}", total_hooks - valid_hooks);
    
    if !errors.is_empty() {
        println!("\n❌ Validation errors:");
        for error in &errors {
            println!("   - {}", error);
        }
        return Err(format!("{} hook(s) failed validation", errors.len()).into());
    }
    
    Ok(())
}

fn validate_single_hook(hook_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read the hook definition file
    let content = std::fs::read_to_string(hook_path)?;
    
    // Simple JSON parsing without serde (for build script compatibility)
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Extract required fields
    let name = json["name"].as_str().ok_or("Missing 'name' field")?;
    let scope = json["scope"].as_str().ok_or("Missing 'scope' field")?;
    let bin = json["bin"].as_str().ok_or("Missing 'bin' field")?;
    
    // Validate name format
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(format!("Invalid hook name '{}': must contain only alphanumeric characters, underscores, and hyphens", name).into());
    }
    
    // Validate scope
    let valid_scopes = ["git", "github", "fsmonitor", "reference", "email", "patch"];
    if !valid_scopes.contains(&scope) {
        return Err(format!("Invalid scope '{}': must be one of {:?}", scope, valid_scopes).into());
    }
    
    // Validate concerns (Git-native only)
    let concerns = json["concerns"].as_array().ok_or("Missing 'concerns' field")?;
    let valid_concerns = [
        "blob", "tree", "commit", "tag", "ref", "note", "attr", "index", "stash", "worktree", "remote",
        "branch", "head", "reflog",
        "config-user", "config-core", "config-branch", "config-remote", "config-init", "config-color",
        "config-alias", "config-diff", "config-merge", "config-gpg", "config-commit", "config-pull",
        "config-push", "config-rebase", "config-fetch", "config-status", "config-tar", "config-rerere",
        "config-advice", "config-interactive", "config-submodule", "config-filter", "config-include",
        "config-credential", "config-http", "config-url", "config-safe", "config-notes", "config-gc",
        "config-maintenance", "config-pager", "config-worktree"
    ];
    for concern in concerns {
        let concern_str = concern.as_str().ok_or("Invalid concern format")?;
        if !valid_concerns.contains(&concern_str) {
            return Err(format!("Invalid concern '{}': must be one of {:?}", concern_str, valid_concerns).into());
        }
    }
    
    // Check for duplicate concerns
    let mut concern_strings: Vec<&str> = concerns.iter()
        .filter_map(|c| c.as_str())
        .collect();
    concern_strings.sort();
    concern_strings.dedup();
    if concern_strings.len() != concerns.len() {
        return Err(format!("Duplicate concerns found in hook '{}'", name).into());
    }
    
    // Check if binary exists in target/release/
    let binary_path = Path::new("target/release").join(bin);
    if !binary_path.exists() {
        return Err(format!("Binary '{}' not found in target/release/", bin).into());
    }
    
    if !binary_path.is_file() {
        return Err(format!("Binary '{}' is not a file", bin).into());
    }
    
    Ok(())
}
