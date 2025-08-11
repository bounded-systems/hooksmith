# Hooksmith Attestation Integration Plan

## 🎯 **Overview**

This document outlines how to enhance Hooksmith's existing four-actor pipeline with attestation capabilities, aligning it with proven supply chain security frameworks like **in-toto**, **SLSA**, and **OPA**. The goal is to transform Hooksmith into a comprehensive attestation and policy enforcement system for Git operations.

## 🏗️ **Current Architecture Assessment**

### **Existing Strengths**
- ✅ **Four-Actor Pipeline**: Researcher → Reporter → Mandator → Auditor
- ✅ **Content-Addressed Storage**: Git object database with SHA-256 hashing
- ✅ **SARIF Integration**: Structured validation results
- ✅ **Event Bus System**: Real-time event routing and processing
- ✅ **WASM Component Architecture**: Sandboxed execution environment
- ✅ **Contract Validation**: Declarative policy enforcement

### **Alignment Opportunities**
- 🔄 **in-toto Layout**: Convert pipeline outputs to in-toto attestations
- 🔄 **SLSA Provenance**: Add build-level attestation for Git operations
- 🔄 **OPA Policy Engine**: Expose contracts as OPA policies
- 🔄 **Protobuf Streaming**: Enhance data flows with typed serialization

## 📋 **Integration Roadmap**

### **Phase 1: Attestation Foundation (Week 1-2)**

#### **1.1 Enhanced Event Structure with Attestation**
```rust
// Enhanced HooksmithEvent with attestation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestedEvent {
    // Existing fields
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub event: String,
    pub context: Value,
    
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
    pub build_config: Value,
    pub metadata: Metadata,
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub key_id: String,
    pub signature: String,
    pub algorithm: String,
}
```

#### **1.2 in-toto Layout Integration**
```rust
// in-toto layout for Hooksmith pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntotoLayout {
    pub version: String,
    pub expires: String,
    pub readme: String,
    pub keys: HashMap<String, Key>,
    pub steps: HashMap<String, Step>,
    pub inspect: Vec<Inspection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub threshold: u32,
    pub pubkeys: Vec<String>,
    pub expected_materials: Vec<MaterialRule>,
    pub expected_products: Vec<ProductRule>,
}

// Convert existing pipeline actors to in-toto steps
impl From<ObjectNamesResearcher> for Step {
    fn from(researcher: ObjectNamesResearcher) -> Self {
        Step {
            threshold: 1,
            pubkeys: vec!["researcher-key".to_string()],
            expected_materials: vec![
                MaterialRule::Allow {
                    pattern: "MATERIALS".to_string(),
                }
            ],
            expected_products: vec![
                ProductRule::Create {
                    pattern: "analysis-*.json".to_string(),
                }
            ],
        }
    }
}
```

### **Phase 2: SLSA Provenance Integration (Week 3-4)**

#### **2.1 Provenance Chain Enhancement**
```rust
// Enhanced provenance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub build_id: String,
    pub build_type: String,
    pub builder: Builder,
    pub invocation: Invocation,
    pub materials: Vec<Material>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    pub id: String,
    pub version: String,
    pub builder_dependencies: Vec<BuilderDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub config_source: ConfigSource,
    pub parameters: Value,
    pub environment: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub uri: String,
    pub digest: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}
```

#### **2.2 Git-Specific Provenance**
```rust
// Git operation provenance
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
pub struct FileChange {
    pub path: String,
    pub mode: String,
    pub sha: String,
    pub size: u64,
    pub operation: ChangeOperation, // Add, Modify, Delete
}
```

### **Phase 3: OPA Policy Integration (Week 5-6)**

#### **3.1 Contract to OPA Policy Conversion**
```rust
// OPA policy generator
pub struct OpaPolicyGenerator {
    contract_definitions: HashMap<String, ContractDefinition>,
}

impl OpaPolicyGenerator {
    pub fn generate_policy(&self, contract_name: &str) -> Result<String> {
        let contract = self.contract_definitions.get(contract_name)
            .ok_or_else(|| anyhow!("Contract not found: {}", contract_name))?;
        
        // Convert contract rules to Rego policies
        let rego_policy = self.convert_contract_to_rego(contract)?;
        
        Ok(rego_policy)
    }
    
    fn convert_contract_to_rego(&self, contract: &ContractDefinition) -> Result<String> {
        let mut rego_rules = String::new();
        
        for rule in &contract.rules {
            match rule.rule_type {
                RuleType::Pattern => {
                    rego_rules.push_str(&self.generate_pattern_rule(rule)?);
                }
                RuleType::JsonSchema => {
                    rego_rules.push_str(&self.generate_schema_rule(rule)?);
                }
                RuleType::FileSize => {
                    rego_rules.push_str(&self.generate_filesize_rule(rule)?);
                }
                _ => {}
            }
        }
        
        Ok(rego_rules)
    }
}
```

#### **3.2 OPA Integration Service**
```rust
// OPA service integration
pub struct OpaService {
    client: reqwest::Client,
    base_url: String,
}

impl OpaService {
    pub async fn evaluate_policy(
        &self,
        policy_name: &str,
        input: Value,
    ) -> Result<OpaResult> {
        let response = self.client
            .post(&format!("{}/v1/data/{}", self.base_url, policy_name))
            .json(&json!({ "input": input }))
            .send()
            .await?;
        
        let result: OpaResult = response.json().await?;
        Ok(result)
    }
}
```

### **Phase 4: Protobuf Streaming Enhancement (Week 7-8)**

#### **4.1 Protobuf Schema Definitions**
```protobuf
// proto/attestation.proto
syntax = "proto3";

package hooksmith.attestation;

message AttestationEvent {
  string id = 1;
  string timestamp = 2;
  string actor = 3;
  string event_type = 4;
  Attestation attestation = 5;
  DigitalSignature signature = 6;
  ProvenanceChain provenance = 7;
}

message Attestation {
  string statement_type = 1;
  repeated Subject subjects = 2;
  string predicate_type = 3;
  Predicate predicate = 4;
}

message Subject {
  string name = 1;
  map<string, string> digests = 2;
}

message Predicate {
  string build_type = 1;
  Builder builder = 2;
  Invocation invocation = 3;
  bytes build_config = 4;
  Metadata metadata = 5;
  repeated Material materials = 6;
}

message DigitalSignature {
  string key_id = 1;
  bytes signature = 2;
  string algorithm = 3;
}
```

#### **4.2 High-Performance Streaming**
```rust
// Protobuf-based event streaming
pub struct ProtobufEventStream {
    writer: tokio::io::BufWriter<tokio::net::TcpStream>,
    schema_registry: SchemaRegistry,
}

impl ProtobufEventStream {
    pub async fn emit_attestation(&mut self, event: AttestationEvent) -> Result<()> {
        // Serialize to protobuf
        let bytes = event.encode_to_vec();
        
        // Write length-prefixed message
        let length = bytes.len() as u32;
        self.writer.write_all(&length.to_be_bytes()).await?;
        self.writer.write_all(&bytes).await?;
        self.writer.flush().await?;
        
        Ok(())
    }
}
```

## 🔧 **Implementation Strategy**

### **Step 1: Attestation Event Bus Enhancement**
```rust
// Enhanced event bus with attestation
pub struct AttestationEventBus {
    inner: EventBus,
    key_manager: KeyManager,
    signature_service: SignatureService,
}

impl AttestationEventBus {
    pub async fn emit_attested_event(
        &mut self,
        event: HooksmithEvent,
        sign: bool,
    ) -> Result<AttestedEvent> {
        // Create attestation
        let attestation = self.create_attestation(&event).await?;
        
        // Sign if requested
        let signature = if sign {
            Some(self.signature_service.sign(&attestation).await?)
        } else {
            None
        };
        
        // Create attested event
        let attested_event = AttestedEvent {
            id: event.id,
            ts: event.ts,
            actor: event.actor,
            event: event.event,
            context: event.context,
            attestation,
            signature,
            provenance: self.build_provenance_chain(&event).await?,
        };
        
        // Emit to inner bus
        self.inner.emit(event)?;
        
        Ok(attested_event)
    }
}
```

### **Step 2: Pipeline Actor Attestation**
```rust
// Enhanced pipeline actors with attestation
impl ObjectNamesResearcher {
    pub async fn analyze_tree_with_attestation(
        &self,
        repo: &Repository,
        object: &GitObject,
        event_bus: &mut AttestationEventBus,
    ) -> Result<(Analysis, AttestedEvent)> {
        // Perform analysis
        let analysis = self.analyze_tree(repo, object)?;
        
        // Create attestation event
        let event = HooksmithEvent {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now(),
            actor: "object-names-researcher".to_string(),
            event: "analysis_completed".to_string(),
            context: json!({
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
        let attested_event = event_bus.emit_attested_event(event, true).await?;
        
        Ok((analysis, attested_event))
    }
}
```

### **Step 3: in-toto Layout Generation**
```rust
// in-toto layout generator
pub struct IntotoLayoutGenerator {
    pipeline_config: PipelineConfig,
    key_registry: KeyRegistry,
}

impl IntotoLayoutGenerator {
    pub fn generate_layout(&self) -> Result<IntotoLayout> {
        let mut steps = HashMap::new();
        
        // Add researcher step
        steps.insert("researcher".to_string(), Step {
            threshold: 1,
            pubkeys: vec![self.key_registry.get_key("researcher")?.id.clone()],
            expected_materials: vec![
                MaterialRule::Allow {
                    pattern: "MATERIALS".to_string(),
                }
            ],
            expected_products: vec![
                ProductRule::Create {
                    pattern: "analysis-*.json".to_string(),
                }
            ],
        });
        
        // Add reporter step
        steps.insert("reporter".to_string(), Step {
            threshold: 1,
            pubkeys: vec![self.key_registry.get_key("reporter")?.id.clone()],
            expected_materials: vec![
                MaterialRule::Match {
                    pattern: "analysis-*.json".to_string(),
                }
            ],
            expected_products: vec![
                ProductRule::Create {
                    pattern: "report-*.json".to_string(),
                }
            ],
        });
        
        // Add mandator step
        steps.insert("mandator".to_string(), Step {
            threshold: 1,
            pubkeys: vec![self.key_registry.get_key("mandator")?.id.clone()],
            expected_materials: vec![
                MaterialRule::Match {
                    pattern: "contract-*.json".to_string(),
                }
            ],
            expected_products: vec![
                ProductRule::Create {
                    pattern: "mandate-*.json".to_string(),
                }
            ],
        });
        
        // Add auditor step
        steps.insert("auditor".to_string(), Step {
            threshold: 1,
            pubkeys: vec![self.key_registry.get_key("auditor")?.id.clone()],
            expected_materials: vec![
                MaterialRule::Match {
                    pattern: "report-*.json".to_string(),
                },
                MaterialRule::Match {
                    pattern: "mandate-*.json".to_string(),
                }
            ],
            expected_products: vec![
                ProductRule::Create {
                    pattern: "verdict-*.json".to_string(),
                }
            ],
        });
        
        Ok(IntotoLayout {
            version: "0.1".to_string(),
            expires: (Utc::now() + Duration::days(30)).to_rfc3339(),
            readme: "Hooksmith four-actor pipeline attestation layout".to_string(),
            keys: self.key_registry.get_all_keys()?,
            steps,
            inspect: vec![],
        })
    }
}
```

## 🎯 **Benefits of Integration**

### **1. Supply Chain Security Alignment**
- **Immutable Audit Trail**: Every Git operation becomes an attested event
- **Provenance Tracking**: Complete chain of custody for code changes
- **Tamper Evidence**: Cryptographic signatures prevent unauthorized modifications

### **2. Policy-as-Code Integration**
- **OPA Compatibility**: Contracts become enforceable policies
- **Declarative Rules**: Clear, auditable policy definitions
- **CI/CD Integration**: Seamless integration with existing pipelines

### **3. Performance Improvements**
- **Protobuf Streaming**: Efficient, typed data serialization
- **Content-Addressed Caching**: Deterministic, cacheable operations
- **Parallel Processing**: Concurrent attestation generation

### **4. Ecosystem Compatibility**
- **in-toto Tools**: Compatible with existing supply chain tools
- **SLSA Frameworks**: Aligns with Google's SLSA framework
- **SARIF Integration**: Maintains existing tooling compatibility

## 🚀 **Next Steps**

1. **Implement Attestation Event Bus** (Week 1)
2. **Add Digital Signature Support** (Week 2)
3. **Create in-toto Layout Generator** (Week 3)
4. **Integrate OPA Policy Engine** (Week 4)
5. **Add Protobuf Streaming** (Week 5)
6. **Create SLSA Provenance Chain** (Week 6)
7. **Build Integration Examples** (Week 7)
8. **Documentation and Testing** (Week 8)

This integration will transform Hooksmith from a Git validation tool into a comprehensive attestation and policy enforcement system that aligns with industry best practices for supply chain security.
