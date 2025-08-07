use std::path::Path;
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=.hooksmith/hooks/");
    println!("cargo:rerun-if-changed=target/release/");
    
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
            let scope_name = scope_path.file_name().unwrap().to_string_lossy();
            
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
    // Read and parse the hook definition
    let content = std::fs::read_to_string(hook_path)?;
    let hook: StaticHook = serde_json::from_str(&content)?;
    
    // Validate the hook structure
    hook.validate()?;
    
    // Check if binary exists in target/release/
    let binary_path = Path::new("target/release").join(&hook.bin);
    if !binary_path.exists() {
        return Err(format!("Binary '{}' not found in target/release/", hook.bin).into());
    }
    
    if !binary_path.is_file() {
        return Err(format!("Binary '{}' is not a file", hook.bin).into());
    }
    
    Ok(())
}

// Static hook definition (simplified for build script)
#[derive(serde::Deserialize)]
struct StaticHook {
    name: String,
    scope: String,
    concerns: Vec<String>,
    bin: String,
}

impl StaticHook {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate name format
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(format!("Invalid hook name '{}': must contain only alphanumeric characters, underscores, and hyphens", self.name).into());
        }
        
        // Validate scope
        let valid_scopes = ["git", "github", "fsmonitor", "reference", "email", "patch"];
        if !valid_scopes.contains(&self.scope.as_str()) {
            return Err(format!("Invalid scope '{}': must be one of {:?}", self.scope, valid_scopes).into());
        }
        
        // Validate concerns
        let valid_concerns = ["blob", "tree", "ref", "note", "attr", "contract-violation", "symbol-analysis"];
        for concern in &self.concerns {
            if !valid_concerns.contains(&concern.as_str()) {
                return Err(format!("Invalid concern '{}': must be one of {:?}", concern, valid_concerns).into());
            }
        }
        
        // Check for duplicate concerns
        let mut concerns = self.concerns.clone();
        concerns.sort();
        concerns.dedup();
        if concerns.len() != self.concerns.len() {
            return Err(format!("Duplicate concerns found in hook '{}'", self.name).into());
        }
        
        Ok(())
    }
}
