# Framework Alignment Guide

## 🎯 **Quick Reference: Hooksmith ↔ Industry Frameworks**

This guide shows how Hooksmith's attestation integration aligns with specific supply chain security frameworks and tools.

## 📋 **Framework Mappings**

### **1. in-toto Framework**
| in-toto Concept | Hooksmith Implementation | Status |
|----------------|-------------------------|---------|
| **Layout** | `IntotoLayout` with four-actor steps | ✅ Implemented |
| **Steps** | Researcher, Reporter, Mandator, Auditor | ✅ Mapped |
| **Materials** | Git objects, contracts, analysis results | ✅ Defined |
| **Products** | Analysis, reports, mandates, verdicts | ✅ Defined |
| **Keys** | Actor-specific signing keys | ✅ Planned |
| **Verification** | `in-toto-verify` compatible output | ✅ Planned |

**Example in-toto Layout:**
```json
{
  "version": "0.1",
  "expires": "2024-12-31T23:59:59Z",
  "steps": {
    "researcher": {
      "threshold": 1,
      "pubkeys": ["researcher-key"],
      "expected_materials": [{"allow": "MATERIALS"}],
      "expected_products": [{"create": "analysis-*.json"}]
    },
    "reporter": {
      "threshold": 1,
      "pubkeys": ["reporter-key"],
      "expected_materials": [{"match": "analysis-*.json"}],
      "expected_products": [{"create": "report-*.json"}]
    }
  }
}
```

### **2. SLSA Framework**
| SLSA Concept | Hooksmith Implementation | Status |
|-------------|-------------------------|---------|
| **Provenance** | `GitProvenance` for commits | ✅ Implemented |
| **Builder** | Hooksmith actor identities | ✅ Defined |
| **Materials** | Git trees, blobs, contracts | ✅ Mapped |
| **Invocation** | Actor execution context | ✅ Defined |
| **Metadata** | Build timing, completeness | ✅ Planned |
| **Verification** | SLSA-compatible attestations | ✅ Planned |

**Example SLSA Provenance:**
```json
{
  "buildType": "https://hooksmith.dev/git-validation/v1",
  "builder": {
    "id": "hooksmith-researcher",
    "version": "1.0.0"
  },
  "invocation": {
    "configSource": {
      "uri": "git://github.com/org/repo",
      "entryPoint": "analyze_tree"
    },
    "parameters": {"object_oid": "abc123"},
    "environment": {"tool": "object-names-researcher"}
  },
  "materials": [{
    "uri": "git://abc123",
    "digest": {"sha256": "def456..."}
  }]
}
```

### **3. OPA (Open Policy Agent)**
| OPA Concept | Hooksmith Implementation | Status |
|------------|-------------------------|---------|
| **Policy** | Generated from contracts | ✅ Planned |
| **Data** | Git tree analysis results | ✅ Mapped |
| **Input** | File lists, metadata | ✅ Defined |
| **Rules** | Contract validation rules | ✅ Convertible |
| **Evaluation** | OPA-compatible API | ✅ Planned |
| **Output** | Allow/deny decisions | ✅ Compatible |

**Example OPA Policy:**
```rego
package hooksmith.object_names

# Generated from Hooksmith contract
allow {
    # Required files check
    required_files := ["README.md", "Cargo.toml"]
    every file in required_files {
        input.files[_] == file
    }
    
    # Rejected patterns check
    rejected_patterns := ["*.tmp", "*.log"]
    not any pattern in rejected_patterns {
        glob.match(pattern, [], input.files[_])
    }
}
```

### **4. Supply Chain Provenance Systems**
| System Concept | Hooksmith Implementation | Status |
|---------------|-------------------------|---------|
| **Audit Trail** | Immutable event chain | ✅ Implemented |
| **Content Addressing** | Git SHA-256 hashing | ✅ Existing |
| **Tamper Evidence** | Cryptographic signatures | ✅ Planned |
| **Verification** | Attestation validation | ✅ Planned |
| **Compliance** | Policy enforcement | ✅ Existing |

### **5. Policy-as-Code Enforcement**
| Policy Concept | Hooksmith Implementation | Status |
|---------------|-------------------------|---------|
| **Declarative Rules** | Contract definitions | ✅ Existing |
| **CI/CD Integration** | Git hooks, SARIF | ✅ Existing |
| **Validation** | Contract checking | ✅ Existing |
| **Attestation** | Policy compliance proof | ✅ Planned |
| **Audit** | Policy violation tracking | ✅ Existing |

## 🔧 **Integration Examples**

### **1. VeriPass/Tilkal Style Audit Trail**
```rust
// Immutable audit chain with content addressing
let audit_chain = vec![
    AttestedEvent {
        id: "event-1",
        attestation: Attestation {
            statement_type: "https://in-toto.io/Statement/v0.1",
            subject: vec![Subject {
                name: "hooksmith:commit-abc123",
                digest: HashMap::from([("sha256".to_string(), "def456...".to_string())]),
            }],
            predicate: Predicate { /* SLSA provenance */ },
        },
        signature: Some(DigitalSignature { /* cryptographic proof */ }),
    },
    // ... chain continues
];
```

### **2. ForensiBlock Style Merkle Roots**
```rust
// Merkle tree of attestations
let merkle_tree = MerkleTree::from_attestations(&attestations);
let root_hash = merkle_tree.root_hash();

// Store root in blockchain or immutable storage
store_merkle_root(root_hash, timestamp);
```

### **3. Cerbos Style Protobuf Streaming**
```protobuf
// High-performance policy evaluation
message PolicyEvaluationRequest {
    string policy_name = 1;
    bytes input_data = 2;
    string context = 3;
}

message PolicyEvaluationResponse {
    bool allowed = 1;
    repeated string violations = 2;
    bytes attestation = 3;
}
```

## 🚀 **Implementation Roadmap**

### **Phase 1: Foundation (Weeks 1-2)**
- [ ] **in-toto Layout Generation**: Convert pipeline to in-toto steps
- [ ] **SLSA Provenance**: Add Git-specific provenance tracking
- [ ] **Digital Signatures**: Implement cryptographic signing

### **Phase 2: Policy Engine (Weeks 3-4)**
- [ ] **OPA Integration**: Convert contracts to Rego policies
- [ ] **Policy Evaluation**: Add OPA-compatible API
- [ ] **Policy Management**: Version and update policies

### **Phase 3: Streaming & Performance (Weeks 5-6)**
- [ ] **Protobuf Schemas**: Define efficient serialization
- [ ] **High-Performance Streaming**: Async event processing
- [ ] **Caching Optimization**: Content-addressed caching

### **Phase 4: Ecosystem Integration (Weeks 7-8)**
- [ ] **Tool Compatibility**: in-toto-verify, slsa-verify
- [ ] **CI/CD Integration**: GitHub Actions, GitLab CI
- [ ] **Documentation**: Framework-specific guides

## 🎯 **Benefits by Framework**

### **in-toto Benefits**
- ✅ **Step-by-step attestation** of pipeline execution
- ✅ **Material and product tracking** for audit trails
- ✅ **Key-based verification** for trust establishment
- ✅ **Tool ecosystem compatibility** with existing in-toto tools

### **SLSA Benefits**
- ✅ **Build-level provenance** for Git operations
- ✅ **Builder identity** for actor accountability
- ✅ **Invocation tracking** for execution context
- ✅ **Framework compliance** with Google's SLSA

### **OPA Benefits**
- ✅ **Policy-as-code** for declarative rules
- ✅ **Multi-language support** across programming languages
- ✅ **CI/CD integration** with existing pipelines
- ✅ **Performance optimization** with efficient evaluation

### **Supply Chain Benefits**
- ✅ **Immutable audit trails** for compliance
- ✅ **Tamper evidence** for security
- ✅ **Content addressing** for integrity
- ✅ **Verification tools** for trust

## 🔗 **Tool Integration**

### **Existing Tools That Work**
- ✅ **in-toto-verify**: Verify attestation chains
- ✅ **slsa-verify**: Verify SLSA provenance
- ✅ **opa eval**: Evaluate Rego policies
- ✅ **sarif-tools**: Process validation results

### **New Tools to Build**
- 🔄 **hooksmith-attest**: Generate attestations
- 🔄 **hooksmith-verify**: Verify attestations
- 🔄 **hooksmith-policy**: Manage policies
- 🔄 **hooksmith-provenance**: Track provenance

This alignment guide shows how Hooksmith's attestation integration transforms it into a **comprehensive supply chain security platform** that works seamlessly with existing industry tools and frameworks.
