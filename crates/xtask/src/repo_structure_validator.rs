use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoStructureSchema {
    pub workspace: WorkspaceConfig,
    pub component_crates: ComponentCratesConfig,
    pub cli_crates: CliCratesConfig,
    pub documentation: DocumentationConfig,
    pub generated_sources: GeneratedSourcesConfig,
    pub examples: ExamplesConfig,
    pub tests: TestsConfig,
    pub hooks: HooksConfig,
    pub scripts: ScriptsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub required_files: Vec<String>,
    pub required_directories: Vec<String>,
    pub naming_patterns: NamingPatternsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamingPatternsConfig {
    pub config_files: Vec<String>,
    pub schema_files: Vec<String>,
    pub generated_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentCratesConfig {
    pub location: String,
    pub required_files: Vec<String>,
    pub required_directories: Vec<String>,
    pub optional_files: Vec<String>,
    pub cargo_toml_requirements: CargoTomlRequirements,
    pub wit_requirements: WitRequirements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoTomlRequirements {
    pub required_metadata: Vec<String>,
    pub component_metadata: ComponentMetadataConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentMetadataConfig {
    pub wit: Vec<String>,
    pub bindings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WitRequirements {
    pub location: String,
    pub naming: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliCratesConfig {
    pub locations: Vec<String>,
    pub required_files: Vec<String>,
    pub optional_files: Vec<String>,
    pub forbidden_directories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub required_files: Vec<String>,
    pub component_docs: ComponentDocsConfig,
    pub design_docs: DesignDocsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentDocsConfig {
    pub location: String,
    pub required_files: Vec<String>,
    pub naming: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DesignDocsConfig {
    pub location: String,
    pub required_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedSourcesConfig {
    pub location: String,
    pub registry_file: String,
    pub checksum_file: String,
    pub allowed_extensions: Vec<String>,
    pub validation: ValidationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub must_be_tracked: bool,
    pub must_have_checksums: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExamplesConfig {
    pub location: String,
    pub required_extensions: Vec<String>,
    pub naming: String,
    pub compilation: CompilationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilationConfig {
    pub must_compile: bool,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestsConfig {
    pub location: String,
    pub allowed_extensions: Vec<String>,
    pub naming: TestNamingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestNamingConfig {
    pub integration_tests: String,
    pub unit_tests: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HooksConfig {
    pub location: String,
    pub required_extensions: Vec<String>,
    pub naming: String,
    pub executable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptsConfig {
    pub location: String,
    pub allowed_extensions: Vec<String>,
    pub naming: ScriptNamingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptNamingConfig {
    pub simple_scripts: String,
    pub full_binaries: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub path: String,
    pub message: String,
    pub category: String,
}

pub struct RepoStructureValidator {
    schema: RepoStructureSchema,
    workspace_root: PathBuf,
}

impl RepoStructureValidator {
    pub fn new(workspace_root: PathBuf) -> Result<Self> {
        let schema_path = workspace_root.join("schemas/repo-structure.schema.jsonc");
        let schema_content =
            fs::read_to_string(&schema_path).context("Failed to read repo structure schema")?;

        // Parse JSONC (remove comments)
        let schema_content = Self::remove_jsonc_comments(&schema_content);
        let schema: RepoStructureSchema = serde_json::from_str(&schema_content)
            .context("Failed to parse repo structure schema")?;

        Ok(Self {
            schema,
            workspace_root,
        })
    }

    fn remove_jsonc_comments(content: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut escaped = false;
        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();

        while i < chars.len() {
            let ch = chars[i];

            if escaped {
                result.push(ch);
                escaped = false;
                i += 1;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                result.push(ch);
                i += 1;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                result.push(ch);
                i += 1;
                continue;
            }

            if !in_string {
                // Check for single-line comment
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    // Skip to end of line
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    if i < chars.len() {
                        result.push('\n');
                    }
                    continue;
                }

                // Check for multi-line comment
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                    // Skip to end of comment
                    i += 2;
                    while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                        i += 1;
                    }
                    if i + 1 < chars.len() {
                        i += 2;
                    }
                    continue;
                }
            }

            result.push(ch);
            i += 1;
        }

        result
    }

    pub fn validate(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Validate workspace structure
        self.validate_workspace(&mut result)?;

        // Validate component crates
        self.validate_component_crates(&mut result)?;

        // Validate CLI crates
        self.validate_cli_crates(&mut result)?;

        // Validate documentation
        self.validate_documentation(&mut result)?;

        // Validate generated sources
        self.validate_generated_sources(&mut result)?;

        // Validate examples
        self.validate_examples(&mut result)?;

        // Validate tests
        self.validate_tests(&mut result)?;

        // Validate hooks
        self.validate_hooks(&mut result)?;

        // Validate scripts
        self.validate_scripts(&mut result)?;

        result.valid = result.errors.is_empty();
        Ok(result)
    }

    fn validate_workspace(&self, result: &mut ValidationResult) -> Result<()> {
        // Check required files
        for file in &self.schema.workspace.required_files {
            let file_path = self.workspace_root.join(file);
            if !file_path.exists() {
                result.errors.push(ValidationError {
                    path: file.clone(),
                    message: format!("Required workspace file not found: {}", file),
                    category: "workspace".to_string(),
                });
            }
        }

        // Check required directories
        for dir in &self.schema.workspace.required_directories {
            let dir_path = self.workspace_root.join(dir);
            if !dir_path.exists() || !dir_path.is_dir() {
                result.errors.push(ValidationError {
                    path: dir.clone(),
                    message: format!("Required workspace directory not found: {}", dir),
                    category: "workspace".to_string(),
                });
            }
        }

        // Check naming patterns
        self.validate_naming_patterns(result)?;

        Ok(())
    }

    fn validate_naming_patterns(&self, result: &mut ValidationResult) -> Result<()> {
        // Check config files are only in config/ directory
        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative_path = entry.path().strip_prefix(&self.workspace_root)?;
            let path_str = relative_path.to_string_lossy();

            // Check if file matches config pattern but is not in config/
            for pattern in &self.schema.workspace.naming_patterns.config_files {
                if Self::matches_pattern(&path_str, pattern) && !path_str.starts_with("config/") {
                    result.warnings.push(ValidationWarning {
                        path: path_str.to_string(),
                        message: format!(
                            "Config file found outside config/ directory: {}",
                            path_str
                        ),
                        category: "naming_patterns".to_string(),
                    });
                }
            }

            // Check if file matches schema pattern but is not in schemas/
            for pattern in &self.schema.workspace.naming_patterns.schema_files {
                if Self::matches_pattern(&path_str, pattern) && !path_str.starts_with("schemas/") {
                    result.warnings.push(ValidationWarning {
                        path: path_str.to_string(),
                        message: format!(
                            "Schema file found outside schemas/ directory: {}",
                            path_str
                        ),
                        category: "naming_patterns".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn matches_pattern(path: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        if pattern.contains('*') {
            let regex_pattern = pattern.replace("*", ".*");
            regex::Regex::new(&regex_pattern).map_or(false, |re| re.is_match(path))
        } else {
            path.ends_with(pattern)
        }
    }

    fn validate_component_crates(&self, result: &mut ValidationResult) -> Result<()> {
        let components_dir = self
            .workspace_root
            .join(&self.schema.component_crates.location);

        if !components_dir.exists() {
            result.errors.push(ValidationError {
                path: self.schema.component_crates.location.clone(),
                message: "Component crates directory not found".to_string(),
                category: "component_crates".to_string(),
            });
            return Ok(());
        }

        for entry in fs::read_dir(&components_dir)? {
            let entry = entry?;
            let crate_path = entry.path();

            if !crate_path.is_dir() {
                continue;
            }

            let crate_name = crate_path.file_name().unwrap().to_string_lossy();

            // Check required files
            for file in &self.schema.component_crates.required_files {
                let file_path = crate_path.join(file);
                if !file_path.exists() {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", crate_name, file),
                        message: format!("Required file not found in component crate: {}", file),
                        category: "component_crates".to_string(),
                    });
                }
            }

            // Check required directories
            for dir in &self.schema.component_crates.required_directories {
                let dir_path = crate_path.join(dir);
                if !dir_path.exists() || !dir_path.is_dir() {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", crate_name, dir),
                        message: format!(
                            "Required directory not found in component crate: {}",
                            dir
                        ),
                        category: "component_crates".to_string(),
                    });
                }
            }

            // Check Cargo.toml requirements
            self.validate_component_cargo_toml(&crate_path, &crate_name, result)?;

            // Check WIT requirements
            self.validate_component_wit(&crate_path, &crate_name, result)?;
        }

        Ok(())
    }

    fn validate_component_cargo_toml(
        &self,
        crate_path: &Path,
        crate_name: &str,
        result: &mut ValidationResult,
    ) -> Result<()> {
        let cargo_toml_path = crate_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(()); // Already reported as missing required file
        }

        let cargo_content = fs::read_to_string(&cargo_toml_path)?;

        // Simple check for required metadata sections
        for metadata in &self
            .schema
            .component_crates
            .cargo_toml_requirements
            .required_metadata
        {
            if !cargo_content.contains(&format!("[{}]", metadata)) {
                result.errors.push(ValidationError {
                    path: format!("{}/Cargo.toml", crate_name),
                    message: format!("Missing required metadata section: {}", metadata),
                    category: "component_crates".to_string(),
                });
            }
        }
        Ok(())
    }

    fn validate_component_wit(
        &self,
        crate_path: &Path,
        crate_name: &str,
        result: &mut ValidationResult,
    ) -> Result<()> {
        let wit_dir = crate_path.join(&self.schema.component_crates.wit_requirements.location);

        if !wit_dir.exists() {
            if self.schema.component_crates.wit_requirements.required {
                result.errors.push(ValidationError {
                    path: format!(
                        "{}/{}",
                        crate_name, self.schema.component_crates.wit_requirements.location
                    ),
                    message: "WIT directory not found in component crate".to_string(),
                    category: "component_crates".to_string(),
                });
            }
            return Ok(());
        }

        // Check for WIT files
        let mut found_wit_files = false;
        for entry in fs::read_dir(&wit_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "wit") {
                found_wit_files = true;
                break;
            }
        }

        if !found_wit_files {
            result.warnings.push(ValidationWarning {
                path: format!(
                    "{}/{}",
                    crate_name, self.schema.component_crates.wit_requirements.location
                ),
                message: "No WIT files found in WIT directory".to_string(),
                category: "component_crates".to_string(),
            });
        }
        Ok(())
    }

    fn validate_cli_crates(&self, result: &mut ValidationResult) -> Result<()> {
        for location in &self.schema.cli_crates.locations {
            let cli_path = self.workspace_root.join(location);

            if !cli_path.exists() {
                continue; // CLI crates are optional
            }

            // Check required files
            for file in &self.schema.cli_crates.required_files {
                let file_path = cli_path.join(file);
                if !file_path.exists() {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", location, file),
                        message: format!("Required file not found in CLI crate: {}", file),
                        category: "cli_crates".to_string(),
                    });
                }
            }

            // Check forbidden directories
            for dir in &self.schema.cli_crates.forbidden_directories {
                let dir_path = cli_path.join(dir);
                if dir_path.exists() && dir_path.is_dir() {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", location, dir),
                        message: format!("Forbidden directory found in CLI crate: {}", dir),
                        category: "cli_crates".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_documentation(&self, result: &mut ValidationResult) -> Result<()> {
        // Check required documentation files
        for file in &self.schema.documentation.required_files {
            let file_path = self.workspace_root.join(file);
            if !file_path.exists() {
                result.errors.push(ValidationError {
                    path: file.clone(),
                    message: format!("Required documentation file not found: {}", file),
                    category: "documentation".to_string(),
                });
            }
        }

        // Check component documentation
        let component_docs_dir = self
            .workspace_root
            .join(&self.schema.documentation.component_docs.location);
        if component_docs_dir.exists() {
            for entry in fs::read_dir(&component_docs_dir)? {
                let entry = entry?;
                let doc_path = entry.path();

                if !doc_path.is_dir() {
                    continue;
                }

                let component_name = doc_path.file_name().unwrap().to_string_lossy();

                for file in &self.schema.documentation.component_docs.required_files {
                    let file_path = doc_path.join(file);
                    if !file_path.exists() {
                        result.warnings.push(ValidationWarning {
                            path: format!(
                                "{}/{}/{}",
                                self.schema.documentation.component_docs.location,
                                component_name,
                                file
                            ),
                            message: format!("Component documentation file not found: {}", file),
                            category: "documentation".to_string(),
                        });
                    }
                }
            }
        }

        // Check design documentation
        let design_docs_dir = self
            .workspace_root
            .join(&self.schema.documentation.design_docs.location);
        if design_docs_dir.exists() {
            for file in &self.schema.documentation.design_docs.required_files {
                let file_path = design_docs_dir.join(file);
                if !file_path.exists() {
                    result.warnings.push(ValidationWarning {
                        path: format!(
                            "{}/{}",
                            self.schema.documentation.design_docs.location, file
                        ),
                        message: format!("Design documentation file not found: {}", file),
                        category: "documentation".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_generated_sources(&self, result: &mut ValidationResult) -> Result<()> {
        let generated_dir = self
            .workspace_root
            .join(&self.schema.generated_sources.location);

        if !generated_dir.exists() {
            return Ok(()); // Generated sources directory is optional
        }

        // Check registry file exists
        let registry_path = self
            .workspace_root
            .join(&self.schema.generated_sources.registry_file);
        if !registry_path.exists() && self.schema.generated_sources.validation.must_be_tracked {
            result.errors.push(ValidationError {
                path: self.schema.generated_sources.registry_file.clone(),
                message: "Generated sources registry file not found".to_string(),
                category: "generated_sources".to_string(),
            });
        }

        // Check checksum file exists
        let checksum_path = self
            .workspace_root
            .join(&self.schema.generated_sources.checksum_file);
        if !checksum_path.exists() && self.schema.generated_sources.validation.must_have_checksums {
            result.errors.push(ValidationError {
                path: self.schema.generated_sources.checksum_file.clone(),
                message: "Generated sources checksum file not found".to_string(),
                category: "generated_sources".to_string(),
            });
        }

        // Check file extensions
        for entry in WalkDir::new(&generated_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative_path = entry.path().strip_prefix(&self.workspace_root)?;
            let path_str = relative_path.to_string_lossy();

            if let Some(extension) = entry.path().extension() {
                let ext_str = format!(".{}", extension.to_string_lossy());
                if !self
                    .schema
                    .generated_sources
                    .allowed_extensions
                    .contains(&ext_str)
                {
                    result.warnings.push(ValidationWarning {
                        path: path_str.to_string(),
                        message: format!(
                            "Unexpected file extension in generated sources: {}",
                            ext_str
                        ),
                        category: "generated_sources".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_examples(&self, result: &mut ValidationResult) -> Result<()> {
        let examples_dir = self.workspace_root.join(&self.schema.examples.location);

        if !examples_dir.exists() {
            return Ok(()); // Examples directory is optional
        }

        for entry in fs::read_dir(&examples_dir)? {
            let entry = entry?;
            let example_path = entry.path();

            if !example_path.is_file() {
                continue;
            }

            let example_name = example_path.file_name().unwrap().to_string_lossy();

            // Check file extension
            if let Some(extension) = example_path.extension() {
                let ext_str = format!(".{}", extension.to_string_lossy());
                if !self.schema.examples.required_extensions.contains(&ext_str) {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", self.schema.examples.location, example_name),
                        message: format!("Invalid file extension for example: {}", ext_str),
                        category: "examples".to_string(),
                    });
                }
            }

            // Check naming pattern
            if !Self::matches_pattern(&example_name, &self.schema.examples.naming) {
                result.warnings.push(ValidationWarning {
                    path: format!("{}/{}", self.schema.examples.location, example_name),
                    message: format!(
                        "Example file does not match naming pattern: {}",
                        self.schema.examples.naming
                    ),
                    category: "examples".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_tests(&self, result: &mut ValidationResult) -> Result<()> {
        let tests_dir = self.workspace_root.join(&self.schema.tests.location);

        if !tests_dir.exists() {
            return Ok(()); // Tests directory is optional
        }

        for entry in fs::read_dir(&tests_dir)? {
            let entry = entry?;
            let test_path = entry.path();

            if !test_path.is_file() {
                continue;
            }

            let test_name = test_path.file_name().unwrap().to_string_lossy();

            // Check file extension
            if let Some(extension) = test_path.extension() {
                let ext_str = format!(".{}", extension.to_string_lossy());
                if !self.schema.tests.allowed_extensions.contains(&ext_str) {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", self.schema.tests.location, test_name),
                        message: format!("Invalid file extension for test: {}", ext_str),
                        category: "tests".to_string(),
                    });
                }
            }

            // Check naming patterns
            let matches_integration =
                Self::matches_pattern(&test_name, &self.schema.tests.naming.integration_tests);
            let matches_unit =
                Self::matches_pattern(&test_name, &self.schema.tests.naming.unit_tests);

            if !matches_integration && !matches_unit {
                result.warnings.push(ValidationWarning {
                    path: format!("{}/{}", self.schema.tests.location, test_name),
                    message: "Test file does not match expected naming patterns".to_string(),
                    category: "tests".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_hooks(&self, result: &mut ValidationResult) -> Result<()> {
        let hooks_dir = self.workspace_root.join(&self.schema.hooks.location);

        if !hooks_dir.exists() {
            return Ok(()); // Hooks directory is optional
        }

        for entry in fs::read_dir(&hooks_dir)? {
            let entry = entry?;
            let hook_path = entry.path();

            if !hook_path.is_file() {
                continue;
            }

            let hook_name = hook_path.file_name().unwrap().to_string_lossy();

            // Check file extension
            if let Some(extension) = hook_path.extension() {
                let ext_str = format!(".{}", extension.to_string_lossy());
                if !self.schema.hooks.required_extensions.contains(&ext_str) {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", self.schema.hooks.location, hook_name),
                        message: format!("Invalid file extension for hook: {}", ext_str),
                        category: "hooks".to_string(),
                    });
                }
            }

            // Check naming pattern
            if !Self::matches_pattern(&hook_name, &self.schema.hooks.naming) {
                result.warnings.push(ValidationWarning {
                    path: format!("{}/{}", self.schema.hooks.location, hook_name),
                    message: format!(
                        "Hook file does not match naming pattern: {}",
                        self.schema.hooks.naming
                    ),
                    category: "hooks".to_string(),
                });
            }

            // Check if executable (if required)
            if self.schema.hooks.executable {
                let metadata = fs::metadata(&hook_path)?;
                if !Self::is_executable(&metadata) {
                    result.warnings.push(ValidationWarning {
                        path: format!("{}/{}", self.schema.hooks.location, hook_name),
                        message: "Hook file should be executable".to_string(),
                        category: "hooks".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_scripts(&self, result: &mut ValidationResult) -> Result<()> {
        let scripts_dir = self.workspace_root.join(&self.schema.scripts.location);

        if !scripts_dir.exists() {
            return Ok(()); // Scripts directory is optional
        }

        for entry in fs::read_dir(&scripts_dir)? {
            let entry = entry?;
            let script_path = entry.path();

            if !script_path.is_file() {
                continue;
            }

            let script_name = script_path.file_name().unwrap().to_string_lossy();

            // Check file extension
            if let Some(extension) = script_path.extension() {
                let ext_str = format!(".{}", extension.to_string_lossy());
                if !self.schema.scripts.allowed_extensions.contains(&ext_str) {
                    result.errors.push(ValidationError {
                        path: format!("{}/{}", self.schema.scripts.location, script_name),
                        message: format!("Invalid file extension for script: {}", ext_str),
                        category: "scripts".to_string(),
                    });
                }
            }

            // Check naming patterns
            let matches_simple =
                Self::matches_pattern(&script_name, &self.schema.scripts.naming.simple_scripts);
            let matches_full =
                Self::matches_pattern(&script_name, &self.schema.scripts.naming.full_binaries);

            if !matches_simple && !matches_full {
                result.warnings.push(ValidationWarning {
                    path: format!("{}/{}", self.schema.scripts.location, script_name),
                    message: "Script file does not match expected naming patterns".to_string(),
                    category: "scripts".to_string(),
                });
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn is_executable(metadata: &fs::Metadata) -> bool {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    fn is_executable(_metadata: &fs::Metadata) -> bool {
        true // Assume executable on non-Unix systems
    }
}

pub fn validate_repo_structure(workspace_root: PathBuf) -> Result<ValidationResult> {
    let validator = RepoStructureValidator::new(workspace_root)?;
    validator.validate()
}
