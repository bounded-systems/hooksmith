//! Tree fix cache module for caching fix plans
#![allow(missing_docs)]

use git2::{Oid, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Fix plan structure for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlan {
    pub id: String,
    pub description: String,
    pub steps: Vec<FixStep>,
}

impl FixPlan {
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: String::new(),
            steps: Vec::new(),
        }
    }
}

/// Individual fix step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStep {
    pub action: String,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Cache key combining tree SHA and fix plan hash for memoization
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreeFixCacheKey {
    pub tree_sha: String,
    pub fix_hash: String,
    pub contract_scope: String,
}

/// Cached fix plan result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFixPlan {
    pub fix_plan: FixPlan,
    pub tree_sha: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub cache_hit_count: u64,
    pub is_valid: bool,
}

/// Statistics for cache performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeFixCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub total_requests: u64,
    pub cache_size: usize,
}

/// Tree-aware fix plan cache with SHA validation and statistics
pub struct TreeFixCache {
    cache: Arc<RwLock<HashMap<TreeFixCacheKey, CachedFixPlan>>>,
    stats: Arc<RwLock<TreeFixCacheStats>>,
    repo: Arc<Repository>,
}

impl TreeFixCache {
    pub fn new(repo: Repository) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(TreeFixCacheStats {
                hits: 0,
                misses: 0,
                invalidations: 0,
                total_requests: 0,
                cache_size: 0,
            })),
            repo: Arc::new(repo),
        }
    }

    /// Get a cached fix plan if available and valid
    pub fn get_fix_plan(
        &self,
        tree_sha: &str,
        fix_hash: &str,
        contract_scope: &str,
    ) -> Option<FixPlan> {
        let key = TreeFixCacheKey {
            tree_sha: tree_sha.to_string(),
            fix_hash: fix_hash.to_string(),
            contract_scope: contract_scope.to_string(),
        };

        let mut stats = self.stats.write().unwrap();
        stats.total_requests += 1;

        if let Some(cached) = self.cache.read().unwrap().get(&key).cloned() {
            if self.validate_cached_plan(&cached) {
                stats.hits += 1;
                // Update cache hit count
                if let Some(mut_cached) = self.cache.write().unwrap().get_mut(&key) {
                    mut_cached.cache_hit_count += 1;
                }
                return Some(cached.fix_plan);
            } else {
                stats.invalidations += 1;
                // Remove invalid cache entry
                self.cache.write().unwrap().remove(&key);
            }
        }

        stats.misses += 1;
        None
    }

    /// Cache a fix plan with tree SHA validation
    pub fn cache_fix_plan(
        &self,
        tree_sha: &str,
        fix_hash: &str,
        contract_scope: &str,
        fix_plan: FixPlan,
    ) -> Result<(), String> {
        // Validate that the tree SHA exists in Git
        if !self.validate_tree_sha(tree_sha)? {
            return Err(format!(
                "Tree SHA {} does not exist in repository",
                tree_sha
            ));
        }

        let key = TreeFixCacheKey {
            tree_sha: tree_sha.to_string(),
            fix_hash: fix_hash.to_string(),
            contract_scope: contract_scope.to_string(),
        };

        let cached_plan = CachedFixPlan {
            fix_plan,
            tree_sha: tree_sha.to_string(),
            created_at: chrono::Utc::now(),
            cache_hit_count: 0,
            is_valid: true,
        };

        let mut cache = self.cache.write().unwrap();
        cache.insert(key, cached_plan);

        let mut stats = self.stats.write().unwrap();
        stats.cache_size = cache.len();

        Ok(())
    }

    /// Validate that a cached fix plan is still valid
    fn validate_cached_plan(&self, cached: &CachedFixPlan) -> bool {
        // Check if tree SHA still exists
        if !self.validate_tree_sha(&cached.tree_sha).unwrap_or(false) {
            return false;
        }

        // Check if cache entry is not too old (configurable TTL)
        let max_age = chrono::Duration::hours(24);
        if chrono::Utc::now() - cached.created_at > max_age {
            return false;
        }

        true
    }

    /// Validate that a tree SHA exists in the Git repository
    fn validate_tree_sha(&self, tree_sha: &str) -> Result<bool, String> {
        match Oid::from_str(tree_sha) {
            Ok(oid) => match self.repo.find_tree(oid) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            },
            Err(_) => Err(format!("Invalid tree SHA format: {}", tree_sha)),
        }
    }

    /// Normalize input paths for consistent cache keys
    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        // Convert to absolute path and normalize separators
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.repo.path().parent().unwrap().join(path)
        }
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> TreeFixCacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        let mut stats = self.stats.write().unwrap();
        stats.cache_size = 0;
        stats.invalidations += 1;
    }

    /// Remove cache entries for a specific tree SHA
    pub fn invalidate_tree(&self, tree_sha: &str) {
        let mut cache = self.cache.write().unwrap();
        let keys_to_remove: Vec<_> = cache
            .keys()
            .filter(|key| key.tree_sha == tree_sha)
            .cloned()
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
        }

        let mut stats = self.stats.write().unwrap();
        stats.cache_size = cache.len();
        stats.invalidations += 1;
    }

    /// Export cache statistics to JSON
    pub fn export_stats(&self) -> serde_json::Value {
        let stats = self.get_stats();
        serde_json::to_value(stats).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use tempfile::TempDir;

    fn create_test_repo() -> (Repository, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create a test tree
        let signature = git2::Signature::now("test", "test@example.com").unwrap();
        let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let commit_id = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();

        (repo, temp_dir)
    }

    #[test]
    fn test_cache_operations() {
        let (repo, _temp_dir) = create_test_repo();
        let cache = TreeFixCache::new(repo);

        let tree_sha = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"; // Empty tree
        let fix_hash = "test_fix_hash";
        let contract_scope = "test_scope";

        // Test cache miss
        assert!(cache
            .get_fix_plan(tree_sha, fix_hash, contract_scope)
            .is_none());

        // Test cache hit after insertion
        let fix_plan = FixPlan::new("test_plan".to_string());
        cache
            .cache_fix_plan(tree_sha, fix_hash, contract_scope, fix_plan.clone())
            .unwrap();

        let cached = cache.get_fix_plan(tree_sha, fix_hash, contract_scope);
        assert!(cached.is_some());

        // Test stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests, 2);
    }

    #[test]
    fn test_invalid_tree_sha() {
        let (repo, _temp_dir) = create_test_repo();
        let cache = TreeFixCache::new(repo);

        let invalid_sha = "invalid_sha";
        let fix_hash = "test_fix_hash";
        let contract_scope = "test_scope";
        let fix_plan = FixPlan::new("test_plan".to_string());

        // Should fail to cache with invalid tree SHA
        let result = cache.cache_fix_plan(invalid_sha, fix_hash, contract_scope, fix_plan);
        assert!(result.is_err());
    }
}
