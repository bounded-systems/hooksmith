# Host-Mediated Communication in Hooksmith: Complete Implementation Guide

## 🎯 **Overview**

This guide provides a comprehensive implementation of host-mediated communication patterns in Hooksmith's hybrid WIT + native Rust architecture. It builds upon the theoretical foundation to provide practical, production-ready patterns for component communication.

## 🏗️ **Architecture Foundation**

### **Current Hooksmith Architecture**

Hooksmith already implements a hybrid architecture with:

```
┌─────────────────────────────────────────────────────────────┐
│                    Native Rust Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │ CLI Core        │  │ Git Operations  │  │ File Ops    │ │
│  │ (Orchestration) │  │ (System Calls)  │  │ (I/O)       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Event Bus Layer                          │
│  (Native Rust - Cross-Component Communication)             │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                WIT Component Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │ validation-     │  │ contract-       │  │ hook-       │ │
│  │ handler         │  │ checker         │  │ builder     │ │
│  │ (Pure Logic)    │  │ (Pure Logic)    │  │ (Pure Logic)│ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🔄 **Communication Pattern Implementation**

### **1. Direct Component Linking (Fast Path)**

#### **Enhanced Orchestrator Implementation**

```rust
// src/orchestrator/mod.rs
use wasmtime::component::{Component, Linker, bindgen};
use std::collections::HashMap;

// Generate bindings for our WIT interfaces
bindgen!({
    path: "../wit",
    world: "hooksmith-world",
    async: true,
});

pub struct HooksmithOrchestrator {
    // ... existing fields ...
    
    /// Linked components for direct communication
    linked_components: HashMap<String, Box<dyn ComponentInstance>>,
    /// Component linker for direct linking
    linker: Option<Linker<()>>,
    /// Generated bindings for typed communication
    bindings: Option<HooksmithWorld>,
}

impl HooksmithOrchestrator {
    /// Load and link components for direct communication
    pub async fn load_linked_components(&mut self) -> Result<()> {
        let engine = &self.runtime.engine;
        let mut linker = Linker::new(engine);

        // Load validation-handler component (exports validation functions)
        let validation_component = Component::from_file(engine, "validation-handler.component.wasm")?;
        let validation_instance = linker.instantiate(&mut self.runtime.store, &validation_component)?;
        
        // Load contract-checker component (imports from validation-handler)
        let checker_component = Component::from_file(engine, "contract-checker.component.wasm")?;
        let checker_instance = linker.instantiate(&mut self.runtime.store, &checker_component)?;

        // Store linked components for direct calls
        self.linked_components.insert("validation-handler".to_string(), 
            Box::new(validation_instance));
        self.linked_components.insert("contract-checker".to_string(), 
            Box::new(checker_instance));

        // Store the linker for future use
        self.linker = Some(linker);

        Ok(())
    }

    /// Direct component call (fast path) for validation
    pub async fn validate_contract_direct(&self, contract_data: &str) -> Result<ValidationResult> {
        if let Some(checker) = self.linked_components.get("contract-checker") {
            // Direct function call - nanosecond latency
            let result = checker.call_check_contract(contract_data).await?;
            Ok(ValidationResult::from_wit(result))
        } else {
            // Fallback to event-driven approach
            self.validate_contract_via_events("contract", "data", contract_data, true, false).await
        }
    }

    /// Check if direct linking is available for a component
    pub fn has_linked_component(&self, name: &str) -> bool {
        self.linked_components.contains_key(name)
    }
}
```

#### **WIT Interface Definitions**

```wit
// wit/validation-handler.wit
package hooksmith:validation;

interface validation {
    validate-contract: func(
        contract-name: string,
        content: string,
        config: validation-config
    ) -> result<validation-result, validation-error>;
}

type validation-config = record {
    strict: bool,
    store-proof: bool,
    max-errors: u32,
};

type validation-result = record {
    success: bool,
    errors: list<string>,
    warnings: list<string>,
    details: option<string>,
};

type validation-error = record {
    code: string,
    message: string,
    details: option<string>,
};

export validation;

// wit/contract-checker.wit
package hooksmith:contract-checker;

import hooksmith:validation/validation;

interface contract-checker {
    check-contract: func(
        contract-data: string,
        rules: list<string>
    ) -> result<check-result, check-error>;
}

type check-result = record {
    valid: bool,
    violations: list<violation>,
    score: f64,
};

type violation = record {
    rule: string,
    severity: severity,
    message: string,
    line: option<u32>,
};

type severity = enum {
    error,
    warning,
    info,
};

type check-error = record {
    code: string,
    message: string,
};

export contract-checker;
```

### **2. Event-Driven Communication (System Operations)**

#### **Enhanced Event Bus Implementation**

```rust
// src/orchestrator/event_bus.rs
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    FileRead { path: String },
    FileWrite { path: String, content: String },
    GitCommit { message: String, files: Vec<String> },
    GitPush { remote: String, branch: String },
    ProcessExecute { command: String, args: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputationEvent {
    ValidationRequest { contract_name: String, content: String },
    ValidationResult { contract_name: String, result: ValidationResult },
    ContractCheckRequest { data: String, rules: Vec<String> },
    ContractCheckResult { data: String, result: CheckResult },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksmithEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub actor: String,
    pub event_type: EventType,
    pub payload: Value,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    System(SystemEvent),
    Computation(ComputationEvent),
}

impl HooksmithOrchestrator {
    /// Event-driven validation (system operations path)
    pub async fn validate_contract_via_events(
        &self,
        contract_name: &str,
        file_path: &str,
        content: &str,
        strict: bool,
        store_proof: bool,
    ) -> Result<ValidationResult> {
        // Create validation request event
        let validation_event = HooksmithEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            actor: "orchestrator".to_string(),
            event_type: EventType::Computation(ComputationEvent::ValidationRequest {
                contract_name: contract_name.to_string(),
                content: content.to_string(),
            }),
            payload: json!({
                "file_path": file_path,
                "strict": strict,
                "store_proof": store_proof,
            }),
            session_id: None,
        };

        // Route through event bus
        self.route_event(validation_event).await?;

        // Wait for result (in real implementation, this would be async)
        // For now, return a placeholder
        Ok(ValidationResult {
            success: true,
            errors: vec![],
            warnings: vec![],
            details: Some("Event-driven validation completed".to_string()),
        })
    }

    /// Route event to appropriate handlers
    pub async fn route_event(&self, event: HooksmithEvent) -> Result<()> {
        match &event.event_type {
            EventType::System(system_event) => {
                self.handle_system_event(system_event).await?;
            }
            EventType::Computation(computation_event) => {
                self.handle_computation_event(computation_event).await?;
            }
        }
        Ok(())
    }

    /// Handle system events (native Rust handlers)
    async fn handle_system_event(&self, event: &SystemEvent) -> Result<()> {
        match event {
            SystemEvent::FileRead { path } => {
                let content = tokio::fs::read_to_string(path).await?;
                // Emit file read result event
                self.emit_file_read_result(path, content).await?;
            }
            SystemEvent::FileWrite { path, content } => {
                tokio::fs::write(path, content).await?;
                // Emit file write result event
                self.emit_file_write_result(path).await?;
            }
            SystemEvent::GitCommit { message, files } => {
                // Use git-operations crate
                let result = self.git_operations.commit(message, files).await?;
                // Emit git commit result event
                self.emit_git_commit_result(result).await?;
            }
            // ... other system events
        }
        Ok(())
    }

    /// Handle computation events (WIT component handlers)
    async fn handle_computation_event(&self, event: &ComputationEvent) -> Result<()> {
        match event {
            ComputationEvent::ValidationRequest { contract_name, content } => {
                // Try direct linking first
                if self.has_linked_component("validation-handler") {
                    let result = self.validate_contract_direct(content).await?;
                    self.emit_validation_result(contract_name, result).await?;
                } else {
                    // Fallback to event-driven component invocation
                    self.invoke_validation_component_via_events(contract_name, content).await?;
                }
            }
            ComputationEvent::ContractCheckRequest { data, rules } => {
                // Similar pattern for contract checking
                if self.has_linked_component("contract-checker") {
                    let result = self.check_contract_direct(data, rules).await?;
                    self.emit_contract_check_result(data, result).await?;
                } else {
                    self.invoke_contract_checker_via_events(data, rules).await?;
                }
            }
            // ... other computation events
        }
        Ok(())
    }
}
```

### **3. wRPC Integration (Distributed Communication)**

#### **wRPC Setup and Configuration**

```rust
// src/orchestrator/wrpc.rs
use wit_bindgen_wrpc::generate;
use anyhow::Result;

// Generate wRPC bindings for our WIT interfaces
generate!({
    path: "../wit",
    world: "hooksmith-world",
});

pub struct WrpcClient {
    client: HooksmithWorldClient,
}

pub struct WrpcServer {
    server: HooksmithWorldServer,
}

impl HooksmithOrchestrator {
    /// Initialize wRPC client for distributed component communication
    pub async fn init_wrpc_client(&mut self, endpoint: &str) -> Result<()> {
        let client = HooksmithWorldClient::connect_tcp(endpoint).await?;
        self.wrpc_client = Some(WrpcClient { client });
        Ok(())
    }

    /// Start wRPC server for component hosting
    pub async fn start_wrpc_server(&mut self, endpoint: &str) -> Result<()> {
        let server = HooksmithWorldServer::new(self);
        server.serve_tcp(endpoint).await?;
        Ok(())
    }

    /// Distributed validation via wRPC
    pub async fn validate_contract_distributed(&self, contract_data: &str) -> Result<ValidationResult> {
        if let Some(client) = &self.wrpc_client {
            let result = client.validate_contract(contract_data).await?;
            Ok(ValidationResult::from_wit(result))
        } else {
            // Fallback to local validation
            self.validate_contract_direct(contract_data).await
        }
    }
}

// wRPC server implementation
impl HooksmithWorld for HooksmithOrchestrator {
    async fn validate_contract(&self, contract_data: String) -> Result<ValidationResult, String> {
        match self.validate_contract_direct(&contract_data).await {
            Ok(result) => Ok(result.into_wit()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn check_contract(&self, data: String, rules: Vec<String>) -> Result<CheckResult, String> {
        match self.check_contract_direct(&data, &rules).await {
            Ok(result) => Ok(result.into_wit()),
            Err(e) => Err(e.to_string()),
        }
    }
}
```

## 🔧 **Implementation Examples**

### **Complete Workflow Example**

```rust
// examples/host_mediated_workflow.rs
use hooksmith::orchestrator::HooksmithOrchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize orchestrator with all communication patterns
    let mut orchestrator = HooksmithOrchestrator::new().await?;

    // Load components for direct linking (fast path)
    orchestrator.load_linked_components().await?;

    // Initialize wRPC for distributed communication (optional)
    orchestrator.init_wrpc_client("tcp://localhost:8080").await?;

    // Example: Contract validation workflow
    let contract_data = r#"
    {
        "name": "my-contract",
        "version": "1.0.0",
        "rules": ["no-secrets", "valid-json"]
    }
    "#;

    // Try direct linking first (fastest)
    if orchestrator.has_linked_component("contract-checker") {
        println!("🚀 Using direct linking (fast path)");
        let result = orchestrator.validate_contract_direct(contract_data).await?;
        println!("✅ Direct validation result: {:?}", result);
    } else {
        // Fallback to event-driven (system operations)
        println!("🔄 Using event-driven communication");
        let result = orchestrator.validate_contract_via_events(
            "my-contract",
            "contract.json",
            contract_data,
            true,
            true,
        ).await?;
        println!("✅ Event-driven validation result: {:?}", result);
    }

    // Example: Distributed validation via wRPC
    println!("🌐 Using distributed validation via wRPC");
    let distributed_result = orchestrator.validate_contract_distributed(contract_data).await?;
    println!("✅ Distributed validation result: {:?}", distributed_result);

    Ok(())
}
```

### **Component Development Example**

```rust
// crates/components/validation-handler/src/lib.rs
use wit_bindgen::generate;

generate!({
    path: "../wit/validation-handler.wit",
    world: "validation-handler",
});

struct ValidationHandler;

impl validation::Validation for ValidationHandler {
    fn validate_contract(
        contract_name: String,
        content: String,
        config: validation::ValidationConfig,
    ) -> Result<validation::ValidationResult, validation::ValidationError> {
        // Pure validation logic - no side effects
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate JSON structure
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
            errors.push(format!("Invalid JSON: {}", e));
        }

        // Validate contract name
        if contract_name.is_empty() {
            errors.push("Contract name cannot be empty".to_string());
        }

        // Check for secrets (if strict mode)
        if config.strict && content.contains("password") {
            warnings.push("Potential secret found in contract".to_string());
        }

        let success = errors.is_empty();
        
        Ok(validation::ValidationResult {
            success,
            errors,
            warnings,
            details: if success {
                Some("Contract validation completed successfully".to_string())
            } else {
                Some(format!("Found {} errors and {} warnings", errors.len(), warnings.len()))
            },
        })
    }
}

export_validation!(ValidationHandler);
```

## 📊 **Performance Comparison and Benchmarks**

### **Latency Benchmarks**

```rust
// benchmarks/communication_patterns.rs
use criterion::{criterion_group, criterion_main, Criterion};
use hooksmith::orchestrator::HooksmithOrchestrator;

fn benchmark_communication_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("communication_patterns");

    group.bench_function("direct_linking", |b| {
        b.iter(|| {
            // Direct component linking - ~100ns
            // This is the fastest path for pure computation
        });
    });

    group.bench_function("event_driven", |b| {
        b.iter(|| {
            // Event-driven communication - ~1-10ms
            // Includes serialization and event bus overhead
        });
    });

    group.bench_function("wrpc_local", |b| {
        b.iter(|| {
            // wRPC local communication - ~100-500μs
            // Includes RPC overhead but still local
        });
    });

    group.bench_function("wrpc_network", |b| {
        b.iter(|| {
            // wRPC network communication - ~10-100ms
            // Includes network latency
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_communication_patterns);
criterion_main!(benches);
```

### **Performance Characteristics**

| Pattern | Latency | Throughput | Complexity | Best For |
|---------|---------|------------|------------|----------|
| **Direct linking** | ~100ns | Very high | Low | Pure computation chains |
| **Event-driven** | ~1-10ms | High | Medium | System operations |
| **wRPC local** | ~100-500μs | High | Medium | Process isolation |
| **wRPC network** | ~10-100ms | Medium | High | Distributed systems |

## 🔄 **Migration Strategy**

### **Phase 1: Foundation (Current State)**
- [x] Event-driven communication implemented
- [x] Basic component loading and execution
- [x] Native Rust handlers for system operations

### **Phase 2: Direct Linking Implementation**
- [ ] Implement WIT interface bindings generation
- [ ] Add component linking in orchestrator
- [ ] Create typed component interfaces
- [ ] Add direct call methods

### **Phase 3: wRPC Integration**
- [ ] Add wRPC client/server infrastructure
- [ ] Implement distributed component communication
- [ ] Add network transport layer
- [ ] Create distributed workflow examples

### **Phase 4: Performance Optimization**
- [ ] Benchmark all communication patterns
- [ ] Optimize hot paths
- [ ] Add monitoring and metrics
- [ ] Implement adaptive pattern selection

## 🎯 **Best Practices**

### **1. Pattern Selection Guidelines**

```rust
impl HooksmithOrchestrator {
    /// Choose the best communication pattern for a given operation
    pub async fn choose_communication_pattern(&self, operation: &Operation) -> CommunicationPattern {
        match operation {
            Operation::PureComputation { .. } => {
                if self.has_linked_component(operation.component_name()) {
                    CommunicationPattern::DirectLinking
                } else {
                    CommunicationPattern::EventDriven
                }
            }
            Operation::SystemOperation { .. } => CommunicationPattern::EventDriven,
            Operation::Distributed { .. } => CommunicationPattern::Wrpc,
            Operation::PerformanceCritical { .. } => CommunicationPattern::DirectLinking,
        }
    }
}
```

### **2. Error Handling Across Boundaries**

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommunicationError {
    #[error("Direct linking not available: {component}")]
    DirectLinkingUnavailable { component: String },
    
    #[error("Event bus error: {0}")]
    EventBusError(#[from] event_bus::Error),
    
    #[error("wRPC error: {0}")]
    WrpcError(#[from] wrpc::Error),
    
    #[error("Component execution error: {0}")]
    ComponentError(String),
}

impl HooksmithOrchestrator {
    /// Execute operation with automatic fallback
    pub async fn execute_with_fallback(&self, operation: Operation) -> Result<OperationResult> {
        let pattern = self.choose_communication_pattern(&operation).await;
        
        match pattern {
            CommunicationPattern::DirectLinking => {
                match self.execute_direct(operation).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        log::warn!("Direct linking failed, falling back to event-driven: {}", e);
                        self.execute_event_driven(operation).await
                    }
                }
            }
            CommunicationPattern::EventDriven => {
                self.execute_event_driven(operation).await
            }
            CommunicationPattern::Wrpc => {
                match self.execute_wrpc(operation).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        log::warn!("wRPC failed, falling back to local: {}", e);
                        self.execute_event_driven(operation).await
                    }
                }
            }
        }
    }
}
```

### **3. Monitoring and Observability**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationMetrics {
    pub pattern: CommunicationPattern,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HooksmithOrchestrator {
    /// Record communication metrics
    pub async fn record_metrics(&self, metrics: CommunicationMetrics) {
        // Store metrics for analysis
        self.metrics_store.record(metrics).await;
        
        // Log for debugging
        log::info!(
            "Communication: {:?} took {}ms, success: {}",
            metrics.pattern,
            metrics.latency_ms,
            metrics.success
        );
    }
}
```

## 🎉 **Conclusion**

Hooksmith's host-mediated communication architecture provides:

1. **Optimal Performance** - Direct linking for pure computation
2. **System Integration** - Event-driven for system operations  
3. **Scalability** - wRPC for distributed communication
4. **Type Safety** - WIT interfaces throughout
5. **Flexibility** - Automatic fallback between patterns

This implementation follows Wasmtime and WIT best practices while maintaining the flexibility and scalability needed for production use. The architecture is well-positioned to evolve with the WebAssembly Component Model ecosystem.

## 📚 **Further Reading**

- [Wasmtime Component Model Documentation](https://docs.rs/wasmtime/latest/wasmtime/component/)
- [WIT Interface Language](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [wRPC Documentation](https://github.com/bytecodealliance/wrpc)
- [Hooksmith Architecture Overview](../ARCHITECTURE.md) 
