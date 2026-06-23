use anyhow::{Context, Result};
use git2::{ObjectType, Repository, Tree, TreeWalkMode, TreeWalkResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitObject {
    pub sha: String,
    pub object_type: ObjectType,
    pub path: Option<String>,
    pub size: Option<usize>,
    pub tree_sha: Option<String>,
}

#[derive(Debug)]
pub struct ObjectGraph {
    pub root_commit_sha: String,
    pub root_tree_sha: String,
    pub objects: HashMap<String, GitObject>,
    pub tree_entries: HashMap<String, Vec<String>>,
    pub blob_paths: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TreeScope {
    pub tree_sha: String,
    pub path: String,
    pub entry_names: Vec<String>,
    pub sub_trees: Vec<String>,
    pub blobs: Vec<String>,
}

pub struct GitObjectWalker {
    repo: Repository,
}

impl GitObjectWalker {
    pub fn new(repo_path: &str) -> Result<Self> {
        let repo = Repository::open(repo_path).context("Failed to open Git repository")?;

        Ok(GitObjectWalker { repo })
    }

    pub fn walk_ref(&self, ref_name: &str) -> Result<ObjectGraph> {
        let commit = if let Ok(reference) = self.repo.find_reference(ref_name) {
            reference
                .peel_to_commit()
                .context("Failed to peel reference to commit")?
        } else {
            // Try to find by commit hash
            let oid = git2::Oid::from_str(ref_name)
                .context(format!("Failed to parse commit hash: {}", ref_name))?;
            self.repo
                .find_commit(oid)
                .context(format!("Failed to find commit: {}", ref_name))?
        };

        let tree = commit.tree().context("Failed to get commit tree")?;

        let mut object_graph = ObjectGraph {
            root_commit_sha: commit.id().to_string(),
            root_tree_sha: tree.id().to_string(),
            objects: HashMap::new(),
            tree_entries: HashMap::new(),
            blob_paths: HashMap::new(),
        };

        self.walk_tree_recursive(&tree, "", &mut object_graph)?;

        Ok(object_graph)
    }

    fn walk_tree_recursive(
        &self,
        tree: &Tree,
        path: &str,
        object_graph: &mut ObjectGraph,
    ) -> Result<()> {
        let tree_sha = tree.id().to_string();
        let mut entries = Vec::new();
        let mut sub_trees = Vec::new();
        let mut blobs = Vec::new();

        // Add tree object to graph
        object_graph.objects.insert(
            tree_sha.clone(),
            GitObject {
                sha: tree_sha.clone(),
                object_type: ObjectType::Tree,
                path: if path.is_empty() {
                    None
                } else {
                    Some(path.to_string())
                },
                size: None,
                tree_sha: None,
            },
        );

        // Walk tree entries
        tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            let entry_name = entry.name().unwrap_or("").to_string();
            let entry_path = if root.is_empty() {
                entry_name.clone()
            } else {
                format!("{}/{}", root, entry_name)
            };

            let full_path = if path.is_empty() {
                entry_path.clone()
            } else {
                format!("{}/{}", path, entry_path)
            };

            let entry_sha = entry.id().to_string();
            entries.push(entry_sha.clone());

            match entry.kind() {
                Some(ObjectType::Tree) => {
                    sub_trees.push(entry_sha.clone());

                    // Add tree object
                    object_graph.objects.insert(
                        entry_sha.clone(),
                        GitObject {
                            sha: entry_sha.clone(),
                            object_type: ObjectType::Tree,
                            path: Some(full_path.clone()),
                            size: None,
                            tree_sha: None,
                        },
                    );

                    // Recursively walk subtree
                    if let Ok(subtree) = self.repo.find_tree(entry.id()) {
                        if let Err(e) = self.walk_tree_recursive(&subtree, &full_path, object_graph)
                        {
                            eprintln!("Warning: Failed to walk subtree {}: {}", full_path, e);
                        }
                    }
                }
                Some(ObjectType::Blob) => {
                    blobs.push(entry_sha.clone());

                    // Add blob object
                    if let Ok(blob) = self.repo.find_blob(entry.id()) {
                        object_graph.objects.insert(
                            entry_sha.clone(),
                            GitObject {
                                sha: entry_sha.clone(),
                                object_type: ObjectType::Blob,
                                path: Some(full_path.clone()),
                                size: Some(blob.size()),
                                tree_sha: Some(tree_sha.clone()),
                            },
                        );
                        object_graph.blob_paths.insert(entry_sha, full_path);
                    }
                }
                _ => {
                    // Skip other object types (commits, tags, etc.)
                }
            }

            TreeWalkResult::Ok
        })?;

        // Store tree entries
        object_graph.tree_entries.insert(tree_sha, entries);

        Ok(())
    }

    pub fn get_tree_scope(&self, tree_sha: &str) -> Result<TreeScope> {
        let tree = self
            .repo
            .find_tree(git2::Oid::from_str(tree_sha)?)
            .context(format!("Failed to find tree: {}", tree_sha))?;

        let mut entry_names = Vec::new();
        let mut sub_trees = Vec::new();
        let mut blobs = Vec::new();

        for entry in tree.iter() {
            if let Some(name) = entry.name() {
                entry_names.push(name.to_string());
            }

            let entry_sha = entry.id().to_string();
            match entry.kind() {
                Some(ObjectType::Tree) => {
                    sub_trees.push(entry_sha);
                }
                Some(ObjectType::Blob) => {
                    blobs.push(entry_sha);
                }
                _ => {}
            }
        }

        Ok(TreeScope {
            tree_sha: tree_sha.to_string(),
            path: "".to_string(), // Root tree
            entry_names,
            sub_trees,
            blobs,
        })
    }

    pub fn get_tree_scope_for_path(&self, ref_name: &str, path: &str) -> Result<TreeScope> {
        let commit = if let Ok(reference) = self.repo.find_reference(ref_name) {
            reference
                .peel_to_commit()
                .context("Failed to peel reference to commit")?
        } else {
            // Try to find by commit hash
            let oid = git2::Oid::from_str(ref_name)
                .context(format!("Failed to parse commit hash: {}", ref_name))?;
            self.repo
                .find_commit(oid)
                .context(format!("Failed to find commit: {}", ref_name))?
        };

        let tree = commit.tree().context("Failed to get commit tree")?;

        // Find the tree at the specified path
        let target_tree = if path.is_empty() {
            tree
        } else {
            tree.get_path(Path::new(path))
                .context(format!("Failed to get tree at path: {}", path))?
                .to_object(&self.repo)?
                .peel_to_tree()?
        };

        let mut entry_names = Vec::new();
        let mut sub_trees = Vec::new();
        let mut blobs = Vec::new();

        for entry in target_tree.iter() {
            if let Some(name) = entry.name() {
                entry_names.push(name.to_string());
            }

            let entry_sha = entry.id().to_string();
            match entry.kind() {
                Some(ObjectType::Tree) => {
                    sub_trees.push(entry_sha);
                }
                Some(ObjectType::Blob) => {
                    blobs.push(entry_sha);
                }
                _ => {}
            }
        }

        Ok(TreeScope {
            tree_sha: target_tree.id().to_string(),
            path: path.to_string(),
            entry_names,
            sub_trees,
            blobs,
        })
    }

    pub fn get_changed_objects(&self, base_ref: &str, head_ref: &str) -> Result<Vec<GitObject>> {
        let base_graph = self.walk_ref(base_ref)?;
        let head_graph = self.walk_ref(head_ref)?;

        let mut changed_objects = Vec::new();

        // Find objects that exist in head but not in base, or have different content
        for (sha, head_obj) in &head_graph.objects {
            if let Some(base_obj) = base_graph.objects.get(sha) {
                // Object exists in both, check if content changed
                if head_obj.object_type != base_obj.object_type {
                    changed_objects.push(head_obj.clone());
                }
            } else {
                // New object in head
                changed_objects.push(head_obj.clone());
            }
        }

        // Find objects that were removed (exist in base but not in head)
        for (sha, base_obj) in &base_graph.objects {
            if !head_graph.objects.contains_key(sha) {
                changed_objects.push(base_obj.clone());
            }
        }

        Ok(changed_objects)
    }

    pub fn get_changed_trees(&self, base_ref: &str, head_ref: &str) -> Result<Vec<String>> {
        let changed_objects = self.get_changed_objects(base_ref, head_ref)?;

        let mut changed_trees = Vec::new();
        for obj in changed_objects {
            if obj.object_type == ObjectType::Tree {
                changed_trees.push(obj.sha);
            }
        }

        Ok(changed_trees)
    }

    pub fn get_changed_blobs(&self, base_ref: &str, head_ref: &str) -> Result<Vec<GitObject>> {
        let changed_objects = self.get_changed_objects(base_ref, head_ref)?;

        let mut changed_blobs = Vec::new();
        for obj in changed_objects {
            if obj.object_type == ObjectType::Blob {
                changed_blobs.push(obj);
            }
        }

        Ok(changed_blobs)
    }

    pub fn analyze_tree_stability(&self, ref_name: &str, path: &str) -> Result<Value> {
        let tree_scope = self.get_tree_scope_for_path(ref_name, path)?;

        // Analyze tree structure
        let total_entries = tree_scope.entry_names.len();
        let tree_count = tree_scope.sub_trees.len();
        let blob_count = tree_scope.blobs.len();

        // Calculate stability metrics
        let tree_ratio = if total_entries > 0 {
            tree_count as f64 / total_entries as f64
        } else {
            0.0
        };

        let blob_ratio = if total_entries > 0 {
            blob_count as f64 / total_entries as f64
        } else {
            0.0
        };

        // Determine stability level
        let stability_level = if tree_ratio > 0.7 {
            "high" // Mostly directories, stable structure
        } else if blob_ratio > 0.8 {
            "low" // Mostly files, likely to change
        } else {
            "medium" // Mixed structure
        };

        Ok(json!({
            "tree_sha": tree_scope.tree_sha,
            "path": tree_scope.path,
            "total_entries": total_entries,
            "tree_count": tree_count,
            "blob_count": blob_count,
            "tree_ratio": tree_ratio,
            "blob_ratio": blob_ratio,
            "stability_level": stability_level,
            "sub_trees": tree_scope.sub_trees,
            "blobs": tree_scope.blobs
        }))
    }

    pub fn generate_cache_key(
        &self,
        tree_sha: &str,
        contract_id: &str,
        fix_hash: &str,
    ) -> Result<String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}:{}", tree_sha, contract_id, fix_hash).as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn export_object_graph(&self, object_graph: &ObjectGraph) -> Result<Value> {
        let mut objects_json = json!({});

        for (sha, obj) in &object_graph.objects {
            objects_json[sha] = json!({
                "sha": obj.sha,
                "type": match obj.object_type {
                    ObjectType::Tree => "tree",
                    ObjectType::Blob => "blob",
                    ObjectType::Commit => "commit",
                    ObjectType::Tag => "tag",
                    _ => "unknown"
                },
                "path": obj.path,
                "size": obj.size,
                "tree_sha": obj.tree_sha
            });
        }

        Ok(json!({
            "root_commit_sha": object_graph.root_commit_sha,
            "root_tree_sha": object_graph.root_tree_sha,
            "objects": objects_json,
            "tree_entries": object_graph.tree_entries,
            "blob_paths": object_graph.blob_paths
        }))
    }
}

// Integration with existing Hooksmith pipeline
pub struct HooksmithObjectAnalyzer {
    walker: GitObjectWalker,
}

impl HooksmithObjectAnalyzer {
    pub fn new(repo_path: &str) -> Result<Self> {
        let walker = GitObjectWalker::new(repo_path)?;
        Ok(HooksmithObjectAnalyzer { walker })
    }

    pub fn analyze_ref_for_contracts(&self, ref_name: &str) -> Result<Value> {
        let object_graph = self.walker.walk_ref(ref_name)?;

        // Analyze root tree for object-names contract
        let root_stability = self.walker.analyze_tree_stability(ref_name, "")?;

        // Find changed scopes for contract validation
        let changed_trees = self.walker.get_changed_trees("HEAD~1", ref_name)?;

        let mut scope_analysis = Vec::new();
        for tree_sha in changed_trees {
            if let Ok(tree_scope) = self.walker.get_tree_scope(&tree_sha) {
                scope_analysis.push(json!({
                    "tree_sha": tree_scope.tree_sha,
                    "path": tree_scope.path,
                    "entry_count": tree_scope.entry_names.len(),
                    "sub_tree_count": tree_scope.sub_trees.len(),
                    "blob_count": tree_scope.blobs.len()
                }));
            }
        }

        Ok(json!({
            "ref": ref_name,
            "root_commit_sha": object_graph.root_commit_sha,
            "root_tree_sha": object_graph.root_tree_sha,
            "total_objects": object_graph.objects.len(),
            "root_stability": root_stability,
            "changed_scopes": scope_analysis,
            "object_graph": self.walker.export_object_graph(&object_graph)?
        }))
    }

    pub fn get_validation_scopes(&self, base_ref: &str, head_ref: &str) -> Result<Vec<Value>> {
        let changed_trees = self.walker.get_changed_trees(base_ref, head_ref)?;

        let mut scopes = Vec::new();

        // Always include root scope
        let root_scope = self.walker.get_tree_scope_for_path(head_ref, "")?;
        scopes.push(json!({
            "tree_sha": root_scope.tree_sha,
            "scope_type": "root",
            "contract_ids": ["object-names@v1"],
            "path": "",
            "entry_count": root_scope.entry_names.len()
        }));

        // Add changed subtree scopes
        for tree_sha in changed_trees {
            if let Ok(tree_scope) = self.walker.get_tree_scope(&tree_sha) {
                let contract_ids = self.map_path_to_contracts(&tree_scope.path);
                if !contract_ids.is_empty() {
                    scopes.push(json!({
                        "tree_sha": tree_scope.tree_sha,
                        "scope_type": format!("subtree:{}", tree_scope.path),
                        "contract_ids": contract_ids,
                        "path": tree_scope.path,
                        "entry_count": tree_scope.entry_names.len()
                    }));
                }
            }
        }

        Ok(scopes)
    }

    fn map_path_to_contracts(&self, path: &str) -> Vec<String> {
        match path {
            "crates" => vec!["crate-structure@v1".to_string()],
            "docs" => vec!["documentation@v1".to_string()],
            "src" => vec!["source-structure@v1".to_string()],
            "tests" => vec!["test-structure@v1".to_string()],
            "scripts" => vec!["script-structure@v1".to_string()],
            _ => vec![],
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --bin git_object_walker <command> [args...]");
        println!();
        println!("Commands:");
        println!("  walk <ref>                    - Walk object graph for ref");
        println!("  analyze <ref>                 - Analyze ref for contracts");
        println!("  scopes <base> <head>          - Get validation scopes");
        println!("  stability <ref> <path>        - Analyze tree stability");
        println!("  changed <base> <head>         - Get changed objects");
        std::process::exit(1);
    }

    let command = &args[1];
    let repo_path = "."; // Current directory

    let analyzer = HooksmithObjectAnalyzer::new(repo_path)?;

    match command.as_str() {
        "walk" => {
            if args.len() < 3 {
                eprintln!("Error: walk command requires a ref");
                std::process::exit(1);
            }
            let ref_name = &args[2];
            let object_graph = analyzer.walker.walk_ref(ref_name)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&analyzer.walker.export_object_graph(&object_graph)?)?
            );
        }
        "analyze" => {
            if args.len() < 3 {
                eprintln!("Error: analyze command requires a ref");
                std::process::exit(1);
            }
            let ref_name = &args[2];
            let analysis = analyzer.analyze_ref_for_contracts(ref_name)?;
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        "scopes" => {
            if args.len() < 4 {
                eprintln!("Error: scopes command requires base and head refs");
                std::process::exit(1);
            }
            let base_ref = &args[2];
            let head_ref = &args[3];
            let scopes = analyzer.get_validation_scopes(base_ref, head_ref)?;
            println!("{}", serde_json::to_string_pretty(&scopes)?);
        }
        "stability" => {
            if args.len() < 4 {
                eprintln!("Error: stability command requires ref and path");
                std::process::exit(1);
            }
            let ref_name = &args[2];
            let path = &args[3];
            let stability = analyzer.walker.analyze_tree_stability(ref_name, path)?;
            println!("{}", serde_json::to_string_pretty(&stability)?);
        }
        "changed" => {
            if args.len() < 4 {
                eprintln!("Error: changed command requires base and head refs");
                std::process::exit(1);
            }
            let base_ref = &args[2];
            let head_ref = &args[3];
            let changed_objects = analyzer.walker.get_changed_objects(base_ref, head_ref)?;
            println!("Changed objects: {}", changed_objects.len());
            for obj in changed_objects {
                println!(
                    "  {} ({:?}) - {}",
                    obj.sha,
                    obj.object_type,
                    obj.path.unwrap_or_default()
                );
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }

    Ok(())
}
