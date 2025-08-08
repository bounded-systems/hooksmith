use std::collections::HashMap;

pub mod git_inspector;
pub mod git_query;
pub mod git_snapshot;

#[derive(Debug, Clone)]
pub struct TreeRuleSet {
    pub allowed_root_dirs: Vec<String>,
    pub forbidden_dirs: Vec<String>,
    pub required_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileRuleSet {
    pub forbidden_root_extensions: Vec<String>,
    pub allowed_extensions_by_dir: HashMap<String, Vec<String>>,
    pub forbidden_files: Vec<String>,
    pub required_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub rule: String,
    pub path: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl TreeRuleSet {
    pub fn default() -> Self {
        Self {
            allowed_root_dirs: vec![
                "src".to_string(),
                "docs".to_string(),
                "crates".to_string(),
                "examples".to_string(),
                "tests".to_string(),
                "scripts".to_string(),
                "config".to_string(),
                "schemas".to_string(),
                "hooks".to_string(),
                "target".to_string(),
                ".github".to_string(),
                ".git".to_string(),
                "wit".to_string(),
                "gen".to_string(),
                "generated-sources".to_string(),
                "worktree-lifecycle".to_string(),
                ".cargo".to_string(),
                ".trunk".to_string(),
                ".wb".to_string(),
                ".workbloom".to_string(),
                "test-enhanced-gen-files".to_string(),
            ],
            forbidden_dirs: vec![
                "node_modules".to_string(),
                "vendor".to_string(),
                "tmp".to_string(),
                "temp".to_string(),
            ],
            required_dirs: vec!["src".to_string()],
        }
    }
}

impl FileRuleSet {
    pub fn default() -> Self {
        let mut allowed_extensions_by_dir = HashMap::new();
        allowed_extensions_by_dir.insert("src".to_string(), vec![".rs".to_string()]);
        allowed_extensions_by_dir.insert(
            "docs".to_string(),
            vec![".md".to_string(), ".txt".to_string()],
        );
        allowed_extensions_by_dir.insert(
            "examples".to_string(),
            vec![".rs".to_string(), ".md".to_string()],
        );
        allowed_extensions_by_dir.insert("tests".to_string(), vec![".rs".to_string()]);
        allowed_extensions_by_dir.insert(
            "scripts".to_string(),
            vec![".rs".to_string(), ".sh".to_string()],
        );
        allowed_extensions_by_dir.insert(
            "config".to_string(),
            vec![
                ".toml".to_string(),
                ".yml".to_string(),
                ".yaml".to_string(),
                ".json".to_string(),
                ".jsonc".to_string(),
            ],
        );
        allowed_extensions_by_dir.insert(
            "schemas".to_string(),
            vec![".json".to_string(), ".jsonc".to_string()],
        );

        Self {
            forbidden_root_extensions: vec![
                ".md".to_string(),
                ".toml".to_string(),
                ".rs".to_string(),
            ],
            allowed_extensions_by_dir,
            forbidden_files: vec![
                "Cargo.lock".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            required_files: vec!["Cargo.toml".to_string(), "README.md".to_string()],
        }
    }
}

/// Validate directory structure from HEAD commit tree
///
/// This function validates the directory structure as it exists in the HEAD commit,
/// not including any uncommitted changes in the working directory.
pub fn validate_tree_commit(paths: &[String], rules: &TreeRuleSet) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Get root directories from paths
    let mut root_dirs = std::collections::HashSet::new();
    for path in paths {
        if let Some(first_component) = path.split('/').next() {
            root_dirs.insert(first_component.to_string());
        }
    }

    // Check allowed root directories
    for root_dir in &root_dirs {
        if !rules.allowed_root_dirs.contains(root_dir) {
            violations.push(Violation {
                rule: "allowed_root_dirs".to_string(),
                path: root_dir.clone(),
                message: format!("Root directory '{}' is not in allowed list", root_dir),
                suggestion: Some(format!(
                    "Add '{}' to allowed_root_dirs or remove it",
                    root_dir
                )),
            });
        }
    }

    // Check forbidden directories
    for forbidden_dir in &rules.forbidden_dirs {
        if root_dirs.contains(forbidden_dir) {
            violations.push(Violation {
                rule: "forbidden_dirs".to_string(),
                path: forbidden_dir.clone(),
                message: format!("Forbidden directory '{}' found in root", forbidden_dir),
                suggestion: Some(format!("Remove '{}' directory", forbidden_dir)),
            });
        }
    }

    // Check required directories
    for required_dir in &rules.required_dirs {
        if !root_dirs.contains(required_dir) {
            violations.push(Violation {
                rule: "required_dirs".to_string(),
                path: required_dir.clone(),
                message: format!("Required directory '{}' not found", required_dir),
                suggestion: Some(format!("Create '{}' directory", required_dir)),
            });
        }
    }

    violations
}

/// Validate file structure from Git index (tracked files)
///
/// This function validates the file structure as it exists in the Git index,
/// including staged files but not unstaged changes or untracked files.
pub fn validate_files_index(paths: &[String], rules: &FileRuleSet) -> Vec<Violation> {
    let mut violations = Vec::new();

    for path in paths {
        // Check forbidden root extensions
        if let Some(ext) = get_file_extension(path) {
            if rules.forbidden_root_extensions.contains(&ext) {
                // Check if file is in root directory
                if get_directory(path).map_or(true, |dir| dir == ".") {
                    violations.push(Violation {
                        rule: "forbidden_root_extensions".to_string(),
                        path: path.clone(),
                        message: format!("File with forbidden extension '{}' found in root", ext),
                        suggestion: Some(format!("Move '{}' to appropriate subdirectory", path)),
                    });
                }
            }
        }

        // Check allowed extensions by directory
        if let Some(dir) = get_directory(path) {
            if let Some(allowed_extensions) = rules.allowed_extensions_by_dir.get(&dir) {
                if let Some(ext) = get_file_extension(path) {
                    if !allowed_extensions.contains(&ext) {
                        violations.push(Violation {
                            rule: "allowed_extensions_by_dir".to_string(),
                            path: path.clone(),
                            message: format!(
                                "File with extension '{}' not allowed in directory '{}'",
                                ext, dir
                            ),
                            suggestion: Some(format!(
                                "Move '{}' to directory that allows '{}' extension",
                                path, ext
                            )),
                        });
                    }
                }
            }
        }

        // Check forbidden files
        if rules.forbidden_files.contains(path) {
            violations.push(Violation {
                rule: "forbidden_files".to_string(),
                path: path.clone(),
                message: format!("Forbidden file '{}' found", path),
                suggestion: Some(format!("Remove '{}' file", path)),
            });
        }
    }

    // Check required files
    for required_file in &rules.required_files {
        if !paths.contains(required_file) {
            violations.push(Violation {
                rule: "required_files".to_string(),
                path: required_file.clone(),
                message: format!("Required file '{}' not found", required_file),
                suggestion: Some(format!("Create '{}' file", required_file)),
            });
        }
    }

    violations
}

fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| format!(".{}", s))
}

fn get_directory(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| {
            if s.is_empty() {
                ".".to_string()
            } else {
                s.to_string()
            }
        })
}

/// Compatibility function - validates tree structure (alias for validate_tree_commit)
pub fn validate_tree(paths: &[String], rules: &TreeRuleSet) -> Vec<Violation> {
    validate_tree_commit(paths, rules)
}

/// Compatibility function - validates file structure (alias for validate_files_index)
pub fn validate_files(paths: &[String], rules: &FileRuleSet) -> Vec<Violation> {
    validate_files_index(paths, rules)
}
