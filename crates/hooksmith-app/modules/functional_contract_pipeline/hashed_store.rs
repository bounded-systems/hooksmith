use crate::modules::functional_contract_pipeline::symbols::{ConcernSymbol, ContractSymbol};
use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Content-addressable store for concerns and contracts
#[derive(Debug, Clone)]
pub struct HashedStore {
    /// Concern snapshots indexed by hash
    pub concerns: HashMap<String, ConcernSnapshot>,
    /// Expected snapshots indexed by hash
    pub expectations: HashMap<String, ExpectedSnapshot>,
    /// Contract symbols indexed by hash
    pub contracts: HashMap<String, ContractSymbol>,
    /// Metadata about stored items
    pub metadata: HashMap<String, StoreMetadata>,
}

/// Metadata for stored items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetadata {
    /// When the item was stored
    pub stored_at: u64,
    /// The original concern symbol (for concerns)
    pub concern_symbol: Option<ConcernSymbol>,
    /// The original contract symbol (for contracts)
    pub contract_symbol: Option<ContractSymbol>,
    /// Additional metadata
    pub additional: HashMap<String, serde_json::Value>,
}

/// Hash computation utilities
pub struct HashComputer;

impl HashComputer {
    /// Compute hash of concern snapshot
    pub fn hash_concern(concern: &ConcernSnapshot) -> String {
        let mut hasher = Sha256::new();

        // Hash the concern symbol
        hasher.update(concern.symbol.name().as_bytes());

        // Hash the data
        let data_string = serde_json::to_string(&concern.data).unwrap_or_default();
        hasher.update(data_string.as_bytes());

        // Hash the timestamp
        hasher.update(concern.timestamp.as_bytes());

        // Hash the metadata
        let metadata_string = serde_json::to_string(&concern.metadata).unwrap_or_default();
        hasher.update(metadata_string.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Compute hash of expected snapshot
    pub fn hash_expectation(expectation: &ExpectedSnapshot) -> String {
        let mut hasher = Sha256::new();

        // Hash the concern symbol
        hasher.update(expectation.symbol.name().as_bytes());

        // Hash the expectation data
        let expectation_string =
            serde_json::to_string(&expectation.expectation).unwrap_or_default();
        hasher.update(expectation_string.as_bytes());

        // Hash the contract info
        hasher.update(expectation.contract.as_bytes());
        hasher.update(expectation.contract_version.as_bytes());

        // Hash the metadata
        let metadata_string = serde_json::to_string(&expectation.metadata).unwrap_or_default();
        hasher.update(metadata_string.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Compute hash of contract symbol
    pub fn hash_contract(contract: &ContractSymbol) -> String {
        let mut hasher = Sha256::new();
        hasher.update(contract.name().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute hash of data with salt
    pub fn hash_with_salt(data: &serde_json::Value, salt: &str) -> String {
        let mut hasher = Sha256::new();
        let data_string = serde_json::to_string(data).unwrap_or_default();
        hasher.update(data_string.as_bytes());
        hasher.update(salt.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl HashedStore {
    /// Create a new hashed store
    pub fn new() -> Self {
        Self {
            concerns: HashMap::new(),
            expectations: HashMap::new(),
            contracts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Store a concern snapshot
    pub fn store_concern(&mut self, concern: ConcernSnapshot) -> String {
        let hash = HashComputer::hash_concern(&concern);

        let metadata = StoreMetadata {
            stored_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            concern_symbol: Some(concern.symbol.clone()),
            contract_symbol: None,
            additional: HashMap::new(),
        };

        self.concerns.insert(hash.clone(), concern);
        self.metadata.insert(hash.clone(), metadata);

        hash
    }

    /// Store an expected snapshot
    pub fn store_expectation(&mut self, expectation: ExpectedSnapshot) -> String {
        let hash = HashComputer::hash_expectation(&expectation);

        let metadata = StoreMetadata {
            stored_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            concern_symbol: Some(expectation.symbol.clone()),
            contract_symbol: Some(ContractSymbol::new(&expectation.contract)),
            additional: HashMap::new(),
        };

        self.expectations.insert(hash.clone(), expectation);
        self.metadata.insert(hash.clone(), metadata);

        hash
    }

    /// Store a contract symbol
    pub fn store_contract(&mut self, contract: ContractSymbol) -> String {
        let hash = HashComputer::hash_contract(&contract);

        let metadata = StoreMetadata {
            stored_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            concern_symbol: None,
            contract_symbol: Some(contract.clone()),
            additional: HashMap::new(),
        };

        self.contracts.insert(hash.clone(), contract);
        self.metadata.insert(hash.clone(), metadata);

        hash
    }

    /// Get a concern snapshot by hash
    pub fn get_concern(&self, hash: &str) -> Option<&ConcernSnapshot> {
        self.concerns.get(hash)
    }

    /// Get an expected snapshot by hash
    pub fn get_expectation(&self, hash: &str) -> Option<&ExpectedSnapshot> {
        self.expectations.get(hash)
    }

    /// Get a contract symbol by hash
    pub fn get_contract(&self, hash: &str) -> Option<&ContractSymbol> {
        self.contracts.get(hash)
    }

    /// Get metadata by hash
    pub fn get_metadata(&self, hash: &str) -> Option<&StoreMetadata> {
        self.metadata.get(hash)
    }

    /// Check if a concern exists
    pub fn has_concern(&self, concern: &ConcernSnapshot) -> Option<String> {
        let hash = HashComputer::hash_concern(concern);
        if self.concerns.contains_key(&hash) {
            Some(hash)
        } else {
            None
        }
    }

    /// Check if an expectation exists
    pub fn has_expectation(&self, expectation: &ExpectedSnapshot) -> Option<String> {
        let hash = HashComputer::hash_expectation(expectation);
        if self.expectations.contains_key(&hash) {
            Some(hash)
        } else {
            None
        }
    }

    /// Check if a contract exists
    pub fn has_contract(&self, contract: &ContractSymbol) -> Option<String> {
        let hash = HashComputer::hash_contract(contract);
        if self.contracts.contains_key(&hash) {
            Some(hash)
        } else {
            None
        }
    }

    /// Get all concern hashes
    pub fn concern_hashes(&self) -> Vec<&String> {
        self.concerns.keys().collect()
    }

    /// Get all expectation hashes
    pub fn expectation_hashes(&self) -> Vec<&String> {
        self.expectations.keys().collect()
    }

    /// Get all contract hashes
    pub fn contract_hashes(&self) -> Vec<&String> {
        self.contracts.keys().collect()
    }

    /// Get concerns by symbol
    pub fn concerns_by_symbol(&self, symbol: &ConcernSymbol) -> Vec<&ConcernSnapshot> {
        self.concerns
            .values()
            .filter(|concern| concern.symbol == *symbol)
            .collect()
    }

    /// Get expectations by symbol
    pub fn expectations_by_symbol(&self, symbol: &ConcernSymbol) -> Vec<&ExpectedSnapshot> {
        self.expectations
            .values()
            .filter(|expectation| expectation.symbol == *symbol)
            .collect()
    }

    /// Get contracts by name pattern
    pub fn contracts_by_pattern(&self, pattern: &str) -> Vec<&ContractSymbol> {
        self.contracts
            .values()
            .filter(|contract| contract.name().contains(pattern))
            .collect()
    }

    /// Clear all stored data
    pub fn clear(&mut self) {
        self.concerns.clear();
        self.expectations.clear();
        self.contracts.clear();
        self.metadata.clear();
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            concern_count: self.concerns.len(),
            expectation_count: self.expectations.len(),
            contract_count: self.contracts.len(),
            metadata_count: self.metadata.len(),
        }
    }

    /// Find changed concerns between two stores
    pub fn find_changed_concerns(&self, other: &HashedStore) -> Vec<ConcernChange> {
        let mut changes = Vec::new();

        // Find added concerns
        for (hash, concern) in &other.concerns {
            if !self.concerns.contains_key(hash) {
                changes.push(ConcernChange::Added(concern.clone()));
            }
        }

        // Find removed concerns
        for (hash, concern) in &self.concerns {
            if !other.concerns.contains_key(hash) {
                changes.push(ConcernChange::Removed(concern.clone()));
            }
        }

        // Find modified concerns (same symbol, different hash)
        for (hash, concern) in &other.concerns {
            if let Some(existing) = self.concerns.values().find(|c| c.symbol == concern.symbol) {
                if HashComputer::hash_concern(existing) != *hash {
                    changes.push(ConcernChange::Modified {
                        old: existing.clone(),
                        new: concern.clone(),
                    });
                }
            }
        }

        changes
    }

    /// Find changed expectations between two stores
    pub fn find_changed_expectations(&self, other: &HashedStore) -> Vec<ExpectationChange> {
        let mut changes = Vec::new();

        // Find added expectations
        for (hash, expectation) in &other.expectations {
            if !self.expectations.contains_key(hash) {
                changes.push(ExpectationChange::Added(expectation.clone()));
            }
        }

        // Find removed expectations
        for (hash, expectation) in &self.expectations {
            if !other.expectations.contains_key(hash) {
                changes.push(ExpectationChange::Removed(expectation.clone()));
            }
        }

        // Find modified expectations (same symbol, different hash)
        for (hash, expectation) in &other.expectations {
            if let Some(existing) = self
                .expectations
                .values()
                .find(|e| e.symbol == expectation.symbol)
            {
                if HashComputer::hash_expectation(existing) != *hash {
                    changes.push(ExpectationChange::Modified {
                        old: existing.clone(),
                        new: expectation.clone(),
                    });
                }
            }
        }

        changes
    }
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    /// Number of concern snapshots
    pub concern_count: usize,
    /// Number of expected snapshots
    pub expectation_count: usize,
    /// Number of contract symbols
    pub contract_count: usize,
    /// Number of metadata entries
    pub metadata_count: usize,
}

/// Concern change types
#[derive(Debug, Clone)]
pub enum ConcernChange {
    /// Concern was added
    Added(ConcernSnapshot),
    /// Concern was removed
    Removed(ConcernSnapshot),
    /// Concern was modified
    Modified {
        /// Old concern snapshot
        old: ConcernSnapshot,
        /// New concern snapshot
        new: ConcernSnapshot,
    },
}

/// Expectation change types
#[derive(Debug, Clone)]
pub enum ExpectationChange {
    /// Expectation was added
    Added(ExpectedSnapshot),
    /// Expectation was removed
    Removed(ExpectedSnapshot),
    /// Expectation was modified
    Modified {
        /// Old expected snapshot
        old: ExpectedSnapshot,
        /// New expected snapshot
        new: ExpectedSnapshot,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::symbols::ConcernSymbol;

    #[test]
    fn test_hashed_store() {
        let mut store = HashedStore::new();

        // Create a test concern
        let concern = ConcernSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"exists": true}),
            HashMap::new(),
        );

        // Store the concern
        let hash = store.store_concern(concern.clone());
        assert!(!hash.is_empty());

        // Retrieve the concern
        let retrieved = store.get_concern(&hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().symbol, concern.symbol);
    }

    #[test]
    fn test_hash_consistency() {
        let concern1 = ConcernSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"exists": true}),
            HashMap::new(),
        );

        let concern2 = ConcernSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"exists": true}),
            HashMap::new(),
        );

        let hash1 = HashComputer::hash_concern(&concern1);
        let hash2 = HashComputer::hash_concern(&concern2);

        // Same data should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_store_stats() {
        let mut store = HashedStore::new();

        // Add some test data
        let concern = ConcernSnapshot::new(
            ConcernSymbol::Index,
            serde_json::json!({"exists": true}),
            HashMap::new(),
        );
        store.store_concern(concern);

        let stats = store.stats();
        assert_eq!(stats.concern_count, 1);
        assert_eq!(stats.expectation_count, 0);
        assert_eq!(stats.contract_count, 0);
    }
}
