use clap::Parser;
use git2::Repository;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hooksmith-mandate")]
#[command(about = "Contract mandate compiler for Hooksmith pipeline")]
struct Cli {
    /// Contract reference
    #[arg(long)]
    contract_ref: String,

    /// Contract version
    #[arg(long, default_value = "1.0.0")]
    version: String,

    /// Selector hash for caching
    #[arg(long)]
    selector_hash: String,
}

/// Input object descriptor from NDJSON
#[derive(Debug, Deserialize)]
struct ObjectDescriptor {
    oid: String,
    kind: String,
    logical_path: Option<String>,
    parent_tree_oid: Option<String>,
    size: usize,
}

/// Mandate result for NDJSON output
#[derive(Debug, Serialize)]
struct MandateResult {
    object_oid: String,
    contract_name: String,
    contract_oid: String,
    version: String,
    mandate_blob_oid: String,
    cache_key: String,
    object_selector: String,
    logical_path: Option<String>,
    metadata: serde_json::Value,
}

/// Contract specification
#[derive(Debug, Deserialize)]
struct Contract {
    name: String,
    version: String,
    spec: ContractSpec,
}

#[derive(Debug, Deserialize)]
struct ContractSpec {
    git: GitSpec,
}

#[derive(Debug, Deserialize)]
struct GitSpec {
    tree: TreeSpec,
}

#[derive(Debug, Deserialize)]
struct TreeSpec {
    objects: ObjectsSpec,
}

#[derive(Debug, Deserialize)]
struct ObjectsSpec {
    names: NamesSpec,
}

#[derive(Debug, Deserialize)]
struct NamesSpec {
    required: Vec<String>,
    allowed: Vec<String>,
    rejected: Vec<String>,
    ignored: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repo = Repository::open(".")?;

    // Load contract
    let contract = load_contract(&repo, &cli.contract_ref)?;

    // Read NDJSON from stdin
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Output NDJSON to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let object: ObjectDescriptor = serde_json::from_str(&line)?;

        // Compute object selector
        let object_selector = compute_object_selector(&object);

        // Check cache first
        let cache_key = compute_cache_key(
            &cli.version,
            &cli.contract_ref,
            &object_selector,
            &object.logical_path,
        );

        if let Some(cached_oid) = get_cached_mandate(&repo, &cli.contract_ref, &object_selector)? {
            let result = MandateResult {
                object_oid: object.oid,
                contract_name: contract.name.clone(),
                contract_oid: cli.contract_ref.clone(),
                version: cli.version.clone(),
                mandate_blob_oid: cached_oid,
                cache_key,
                object_selector,
                logical_path: object.logical_path.clone(),
                metadata: json!({"cached": true}),
            };
            let json = serde_json::to_string(&result)?;
            writeln!(handle, "{}", json)?;
            continue;
        }

        // Create mandate
        let mandate_data = create_mandate(&contract, &object)?;

        // Store mandate blob
        let mandate_blob_oid = store_mandate_blob(&repo, &mandate_data)?;

        let result = MandateResult {
            object_oid: object.oid,
            contract_name: contract.name.clone(),
            contract_oid: cli.contract_ref.clone(),
            version: cli.version.clone(),
            mandate_blob_oid,
            cache_key,
            object_selector,
            logical_path: object.logical_path.clone(),
            metadata: json!({"cached": false}),
        };

        let json = serde_json::to_string(&result)?;
        writeln!(handle, "{}", json)?;
    }

    Ok(())
}

fn load_contract(
    repo: &Repository,
    contract_ref: &str,
) -> Result<Contract, Box<dyn std::error::Error>> {
    // For now, load from contracts/object-names@v1.json
    // TODO: Load from refs/hooksmith/contracts/<name>@vX
    let contract_path = "contracts/object-names@v1.json";
    let contract_content = std::fs::read_to_string(contract_path)?;
    let contract: Contract = serde_json::from_str(&contract_content)?;
    Ok(contract)
}

fn create_mandate(
    contract: &Contract,
    object: &ObjectDescriptor,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Determine if this is a root tree
    let is_root = object.logical_path.as_ref().map_or(true, |p| p.is_empty());

    let expectation = if is_root {
        // Root tree expectations
        json!({
            "type": "root_tree",
            "required_entries": contract.spec.git.tree.objects.names.required,
            "allowed_entries": contract.spec.git.tree.objects.names.allowed,
            "rejected_entries": contract.spec.git.tree.objects.names.rejected,
            "ignored_entries": contract.spec.git.tree.objects.names.ignored,
        })
    } else {
        // Non-root tree expectations (can be more permissive)
        json!({
            "type": "sub_tree",
            "allowed_entries": contract.spec.git.tree.objects.names.allowed,
            "ignored_entries": contract.spec.git.tree.objects.names.ignored,
        })
    };

    Ok(expectation)
}

fn compute_object_selector(object: &ObjectDescriptor) -> String {
    let mut hasher = Sha256::new();
    hasher.update(object.kind.as_bytes());
    if let Some(path) = &object.logical_path {
        hasher.update(path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn compute_cache_key(
    version: &str,
    contract_oid: &str,
    selector: &str,
    logical_path: &Option<String>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("mandate@{}", version).as_bytes());
    hasher.update(contract_oid.as_bytes());
    hasher.update(selector.as_bytes());
    if let Some(path) = logical_path {
        hasher.update(path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn get_cached_mandate(
    repo: &Repository,
    contract_oid: &str,
    selector: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/mandate/
    // For now, return None (no cache hit)
    Ok(None)
}

fn store_mandate_blob(
    repo: &Repository,
    mandate_data: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to canonical JSON (sorted keys, no whitespace)
    let json_string = serde_json::to_string(mandate_data)?;

    // Store as blob
    let blob_oid = repo.blob(json_string.as_bytes())?;

    Ok(blob_oid.to_string())
}
