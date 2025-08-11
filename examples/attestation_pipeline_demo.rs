use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

// Enhanced event structure with attestation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestedEvent {
    // Existing fields
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub event: String,
    pub context: serde_json::Value,
    
    // New attestation fields
    pub attestation: Attestation,
    pub signature: Option<DigitalSignature>,
    pub provenance: ProvenanceChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub statement_type: String, // "https://in-toto.io/Statement/v0.1"
    pub subject: Vec<Subject>,
    pub predicate_type: String, // "https://slsa.dev/provenance/v0.2"
    pub predicate: Predicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: HashMap<String, String>, // sha256, sha512, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub build_type: String,
    pub builder: Builder,
    pub invocation: Invocation,
    pub build_config: serde_json::Value,
    pub metadata: Metadata,
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    pub id: String,
    pub version: String,
    pub builder_dependencies: Vec<BuilderDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderDependency {
    pub uri: String,
    pub digest: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub config_source: ConfigSource,
    pub parameters: serde_json::Value,
    pub environment: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSource {
    pub uri: String,
    pub digest: HashMap<String, String>,
    pub entry_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub build_invocation_id: String,
    pub build_started_on: String,
    pub build_finished_on: String,
    pub completeness: Completeness,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completeness {
    pub parameters: bool,
    pub environment: bool,
    pub materials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub uri: String,
    pub digest: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub key_id: String,
    pub signature: String,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub build_id: String,
    pub build_type: String,
    pub builder: Builder,
    pub invocation: Invocation,
    pub materials: Vec<Material>,
    pub metadata: Metadata,
}

// Git-specific provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitProvenance {
    pub repository: String,
    pub commit_sha: String,
    pub tree_sha: String,
    pub parent_commits: Vec<String>,
    pub author: GitAuthor,
    pub committer: GitCommitter,
    pub message: String,
    pub files_changed: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAuthor {
    pub name: String,
    pub email: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitter {
    pub name: String,
    pub email: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub mode: String,
    pub sha: String,
    pub size: u64,
    pub operation: ChangeOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeOperation {
    Add,
    Modify,
    Delete,
}

// Enhanced event bus with attestation capabilities
pub struct AttestationEventBus {
    key_manager: KeyManager,
    signature_service: SignatureService,
}

impl AttestationEventBus {
    pub fn new() -> Self {
        Self {
            key_manager: KeyManager::new(),
            signature_service: SignatureService::new(),
        }
    }

    pub async fn emit_attested_event(
        &mut self,
        event: &HooksmithEvent,
        sign: bool,
    ) -> Result<AttestedEvent, Box<dyn std::error::Error>> {
        // Create attestation
        let attestation = self.create_attestation(event).await?;
        
        // Sign if requested
        let signature = if sign {
            Some(self.signature_service.sign(&attestation).await?)
        } else {
            None
        };
        
        // Create attested event
        let attested_event = AttestedEvent {
            id: event.id.clone(),
            ts: event.ts,
            actor: event.actor.clone(),
            event: event.event.clone(),
            context: event.context.clone(),
            attestation,
            signature,
            provenance: self.build_provenance_chain(event).await?,
        };
        
        Ok(attested_event)
    }

    async fn create_attestation(&self, event: &HooksmithEvent) -> Result<Attestation, Box<dyn std::error::Error>> {
        // Create subject from event context
        let subject = Subject {
            name: format!("hooksmith:{}", event.id),
            digest: self.compute_digest(&event.context),
        };

        // Create predicate based on event type
        let predicate = match event.event.as_str() {
            "analysis_completed" => self.create_analysis_predicate(event).await?,
            "report_created" => self.create_report_predicate(event).await?,
            "mandate_created" => self.create_mandate_predicate(event).await?,
            "audit_completed" => self.create_audit_predicate(event).await?,
            _ => self.create_generic_predicate(event).await?,
        };

        Ok(Attestation {
            statement_type: "https://in-toto.io/Statement/v0.1".to_string(),
            subject: vec![subject],
            predicate_type: "https://slsa.dev/provenance/v0.2".to_string(),
            predicate,
        })
    }

    async fn create_analysis_predicate(&self, event: &HooksmithEvent) -> Result<Predicate, Box<dyn std::error::Error>> {
        let context = &event.context;
        
        Ok(Predicate {
            build_type: "https://hooksmith.dev/analysis/v1".to_string(),
            builder: Builder {
                id: "hooksmith-researcher".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "git://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "analyze_tree".to_string(),
                },
                parameters: context.clone(),
                environment: serde_json::json!({
                    "tool": "object-names-researcher",
                    "version": "1.0.0",
                }),
            },
            build_config: context.clone(),
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![
                Material {
                    uri: format!("git://{}", context["object_oid"].as_str().unwrap_or("")),
                    digest: HashMap::new(),
                    annotations: HashMap::new(),
                }
            ],
        })
    }

    async fn create_report_predicate(&self, event: &HooksmithEvent) -> Result<Predicate, Box<dyn std::error::Error>> {
        Ok(Predicate {
            build_type: "https://hooksmith.dev/report/v1".to_string(),
            builder: Builder {
                id: "hooksmith-reporter".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "git://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "create_report".to_string(),
                },
                parameters: event.context.clone(),
                environment: serde_json::json!({
                    "tool": "object-names-reporter",
                    "version": "1.0.0",
                }),
            },
            build_config: event.context.clone(),
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![
                Material {
                    uri: "analysis-*.json".to_string(),
                    digest: HashMap::new(),
                    annotations: HashMap::new(),
                }
            ],
        })
    }

    async fn create_mandate_predicate(&self, event: &HooksmithEvent) -> Result<Predicate, Box<dyn std::error::Error>> {
        Ok(Predicate {
            build_type: "https://hooksmith.dev/mandate/v1".to_string(),
            builder: Builder {
                id: "hooksmith-mandator".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "contract://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "create_mandate".to_string(),
                },
                parameters: event.context.clone(),
                environment: serde_json::json!({
                    "tool": "object-names-mandator",
                    "version": "1.0.0",
                }),
            },
            build_config: event.context.clone(),
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![
                Material {
                    uri: "contract-*.json".to_string(),
                    digest: HashMap::new(),
                    annotations: HashMap::new(),
                }
            ],
        })
    }

    async fn create_audit_predicate(&self, event: &HooksmithEvent) -> Result<Predicate, Box<dyn std::error::Error>> {
        Ok(Predicate {
            build_type: "https://hooksmith.dev/audit/v1".to_string(),
            builder: Builder {
                id: "hooksmith-auditor".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "audit://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "audit".to_string(),
                },
                parameters: event.context.clone(),
                environment: serde_json::json!({
                    "tool": "object-names-auditor",
                    "version": "1.0.0",
                }),
            },
            build_config: event.context.clone(),
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![
                Material {
                    uri: "report-*.json".to_string(),
                    digest: HashMap::new(),
                    annotations: HashMap::new(),
                },
                Material {
                    uri: "mandate-*.json".to_string(),
                    digest: HashMap::new(),
                    annotations: HashMap::new(),
                }
            ],
        })
    }

    async fn create_generic_predicate(&self, event: &HooksmithEvent) -> Result<Predicate, Box<dyn std::error::Error>> {
        Ok(Predicate {
            build_type: "https://hooksmith.dev/generic/v1".to_string(),
            builder: Builder {
                id: "hooksmith-generic".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "generic://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "process".to_string(),
                },
                parameters: event.context.clone(),
                environment: serde_json::json!({
                    "tool": "hooksmith-generic",
                    "version": "1.0.0",
                }),
            },
            build_config: event.context.clone(),
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![],
        })
    }

    async fn build_provenance_chain(&self, event: &HooksmithEvent) -> Result<ProvenanceChain, Box<dyn std::error::Error>> {
        Ok(ProvenanceChain {
            build_id: event.id.clone(),
            build_type: "https://hooksmith.dev/provenance/v1".to_string(),
            builder: Builder {
                id: "hooksmith-provenance".to_string(),
                version: "1.0.0".to_string(),
                builder_dependencies: vec![],
            },
            invocation: Invocation {
                config_source: ConfigSource {
                    uri: "provenance://".to_string(),
                    digest: HashMap::new(),
                    entry_point: "build_chain".to_string(),
                },
                parameters: event.context.clone(),
                environment: serde_json::json!({
                    "tool": "hooksmith-provenance",
                    "version": "1.0.0",
                }),
            },
            materials: vec![],
            metadata: Metadata {
                build_invocation_id: event.id.clone(),
                build_started_on: event.ts.to_rfc3339(),
                build_finished_on: Utc::now().to_rfc3339(),
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
                reproducible: true,
            },
        })
    }

    fn compute_digest(&self, data: &serde_json::Value) -> HashMap<String, String> {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(data).unwrap().as_bytes());
        let result = format!("{:x}", hasher.finalize());
        
        let mut digest = HashMap::new();
        digest.insert("sha256".to_string(), result);
        digest
    }
}

// Mock services for demonstration
pub struct KeyManager {
    keys: HashMap<String, String>,
}

impl KeyManager {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        keys.insert("researcher".to_string(), "researcher-key-id".to_string());
        keys.insert("reporter".to_string(), "reporter-key-id".to_string());
        keys.insert("mandator".to_string(), "mandator-key-id".to_string());
        keys.insert("auditor".to_string(), "auditor-key-id".to_string());
        Self { keys }
    }
}

pub struct SignatureService {
    key_manager: KeyManager,
}

impl SignatureService {
    pub fn new() -> Self {
        Self {
            key_manager: KeyManager::new(),
        }
    }

    pub async fn sign(&self, attestation: &Attestation) -> Result<DigitalSignature, Box<dyn std::error::Error>> {
        // Mock signature generation
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(attestation)?.as_bytes());
        let signature = format!("{:x}", hasher.finalize());
        
        Ok(DigitalSignature {
            key_id: "mock-key-id".to_string(),
            signature,
            algorithm: "sha256".to_string(),
        })
    }
}

// Original HooksmithEvent structure (from existing codebase)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksmithEvent {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub event: String,
    pub hook: Option<String>,
    pub state: Option<String>,
    pub context: serde_json::Value,
    pub error: Option<serde_json::Value>,
    pub session_id: Option<String>,
    pub duration_ms: Option<u64>,
}

// Enhanced pipeline actors with attestation
pub struct AttestedObjectNamesResearcher {
    inner: ObjectNamesResearcher,
    event_bus: AttestationEventBus,
}

impl AttestedObjectNamesResearcher {
    pub fn new() -> Self {
        Self {
            inner: ObjectNamesResearcher::new(),
            event_bus: AttestationEventBus::new(),
        }
    }

    pub async fn analyze_tree_with_attestation(
        &mut self,
        repo: &Repository,
        object: &GitObject,
    ) -> Result<(Analysis, AttestedEvent), Box<dyn std::error::Error>> {
        // Perform analysis using existing logic
        let analysis = self.inner.analyze_tree(repo, object)?;
        
        // Create attestation event
        let event = HooksmithEvent {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now(),
            actor: "object-names-researcher".to_string(),
            event: "analysis_completed".to_string(),
            context: serde_json::json!({
                "object_oid": object.oid,
                "analysis_oid": analysis.cache_key,
                "tool_fingerprint": analysis.tool_fingerprint,
            }),
            hook: None,
            state: None,
            error: None,
            session_id: None,
            duration_ms: None,
        };
        
        // Emit attested event
        let attested_event = self.event_bus.emit_attested_event(&event, true).await?;
        
        Ok((analysis, attested_event))
    }
}

// Mock structures for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitObject {
    pub oid: String,
    pub kind: String,
    pub logical_path: Option<PathBuf>,
    pub parent_tree_oid: Option<String>,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub tool_fingerprint: ToolFingerprint,
    pub object_oid: String,
    pub analysis_data: serde_json::Value,
    pub cache_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFingerprint {
    pub name: String,
    pub version: String,
    pub config_hash: String,
}

pub struct Repository;

pub struct ObjectNamesResearcher;

impl ObjectNamesResearcher {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_tree(&self, _repo: &Repository, _object: &GitObject) -> Result<Analysis, Box<dyn std::error::Error>> {
        // Mock analysis
        Ok(Analysis {
            tool_fingerprint: ToolFingerprint {
                name: "object-names-researcher".to_string(),
                version: "1.0.0".to_string(),
                config_hash: "config-hash".to_string(),
            },
            object_oid: "mock-oid".to_string(),
            analysis_data: serde_json::json!({
                "entry_count": 10,
                "file_types": ["rs", "md", "toml"],
            }),
            cache_key: "analysis-cache-key".to_string(),
        })
    }
}

// Demo function showing the complete attested pipeline
pub async fn demo_attested_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Attested Pipeline Demo");
    
    // Create repository and object
    let repo = Repository;
    let object = GitObject {
        oid: "abc123".to_string(),
        kind: "tree".to_string(),
        logical_path: Some(PathBuf::from("src")),
        parent_tree_oid: Some("parent123".to_string()),
        size: 1024,
    };
    
    // Create attested researcher
    let mut researcher = AttestedObjectNamesResearcher::new();
    
    // Perform analysis with attestation
    let (analysis, attested_event) = researcher.analyze_tree_with_attestation(&repo, &object).await?;
    
    println!("✅ Analysis completed with attestation");
    println!("📊 Analysis cache key: {}", analysis.cache_key);
    println!("🔐 Attestation ID: {}", attested_event.id);
    println!("📝 Attestation statement type: {}", attested_event.attestation.statement_type);
    println!("🔑 Digital signature: {:?}", attested_event.signature);
    
    // Print attestation details
    println!("\n📋 Attestation Details:");
    println!("  Statement Type: {}", attested_event.attestation.statement_type);
    println!("  Predicate Type: {}", attested_event.attestation.predicate_type);
    println!("  Build Type: {}", attested_event.attestation.predicate.build_type);
    println!("  Builder ID: {}", attested_event.attestation.predicate.builder.id);
    println!("  Build Invocation ID: {}", attested_event.attestation.predicate.metadata.build_invocation_id);
    
    // Print provenance chain
    println!("\n🔗 Provenance Chain:");
    println!("  Build ID: {}", attested_event.provenance.build_id);
    println!("  Build Type: {}", attested_event.provenance.build_type);
    println!("  Builder ID: {}", attested_event.provenance.builder.id);
    println!("  Reproducible: {}", attested_event.provenance.metadata.reproducible);
    
    println!("\n🎉 Attested Pipeline Demo Completed Successfully!");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    demo_attested_pipeline().await
}
