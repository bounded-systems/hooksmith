# Wasmtime Component Communication in Hooksmith

## 🎯 **Overview**

This document explains how Hooksmith's hybrid WIT + native Rust architecture leverages different Wasmtime component communication patterns for optimal performance, security, and maintainability.

## 🔄 **Communication Patterns**

### **1. Direct Component Linking (Recommended for Pure Computation)**

When multiple WIT components need to communicate for pure computation tasks, we use Wasmtime's component linking for direct, type-safe communication.

#### **Example: Validation Chain**
```rust
// Link validation-handler and contract-checker components
let engine = Engine::new(&config)?;
let mut linker = Linker::new(&engine);

// Load validation-handler component (exports validation functions)
let validation_component = Component::from_file(&engine, "validation-handler.component.wasm")?;
linker.instantiate(&mut store, &validation_component)?;

// Load contract-checker component (imports from validation-handler)
let checker_component = Component::from_file(&engine, "contract-checker.component.wasm")?;
let checker = ContractChecker::new(&mut store, &checker_component)?;

// Direct function call - validation-handler.validate() called internally
let result = checker.check_contract(contract_data).await?;
```

#### **Benefits**
- ✅ **Nanosecond latency** - Direct function calls
- ✅ **Type-safe** - WIT interface guarantees
- ✅ **Zero-copy** - Canonical ABI for efficient data transfer
- ✅ **No serialization** - Direct structured data passing

### **2. Event-Driven Communication (For System Operations)**

When components need to interact with system resources or other components across different domains, we use the event bus.

#### **Example: File Validation Workflow**
```rust
// 1. Native handler reads file
let file_content = orchestrator.read_file_via_events("contract.json").await?;

// 2. WIT component validates (pure computation)
let validation_result = orchestrator.validate_contract_via_events(
    "my_contract",
    "contract.json", 
    &file_content,
    true,
    true
).await?;

// 3. Native handler stores result
if validation_result.success {
    orchestrator.store_proof_via_events("contract.json", &validation_result).await?;
}
```

#### **Benefits**
- ✅ **Loose coupling** - Components don't need to know about each other
- ✅ **System integration** - Native handlers for file I/O, Git, etc.
- ✅ **Scalable** - Event-driven architecture
- ✅ **Auditable** - All interactions logged

### **3. Shared Memory (For Performance-Critical Operations)**

For highly performance-sensitive algorithms that need shared state, we can use Wasmtime's SharedMemory.

#### **Example: Large Data Processing**
```rust
// Create shared memory for large dataset processing
let shared_mem = SharedMemory::new(&engine, MemoryType::shared(1024, 10240))?;

// Pass to multiple components that need to process the same data
let component_a = Component::from_file(&engine, "data-processor-a.component.wasm")?;
let instance_a = linker.instantiate(&mut store, &component_a)?;
instance_a.exports().set_shared_memory(shared_mem.clone())?;

let component_b = Component::from_file(&engine, "data-processor-b.component.wasm")?;
let instance_b = linker.instantiate(&mut store, &component_b)?;
instance_b.exports().set_shared_memory(shared_mem)?;
```

#### **Benefits**
- ✅ **High performance** - Direct memory access
- ✅ **Memory efficient** - Shared buffers
- ✅ **Concurrent processing** - Multiple components can work on same data

#### **Cautions**
- ⚠️ **Complex** - Requires careful synchronization
- ⚠️ **Unsafe** - Manual memory management
- ⚠️ **Limited scope** - Only within same process

## 🏗️ **Architecture Integration**

### **Component Composition Strategy**

Our hybrid architecture uses a layered approach:

```
┌─────────────────────────────────────────────────────────────┐
│                    Event Bus Layer                          │
│  (Native Rust - System Operations & Orchestration)         │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                WIT Component Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │ validation-     │  │ contract-       │  │ hook-       │ │
│  │ handler         │◄─┤ checker         │  │ builder     │ │
│  │ (exports)       │  │ (imports)       │  │ (standalone)│ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│  (Direct Linking)     (Direct Linking)     (Event-Driven)  │
└─────────────────────────────────────────────────────────────┘
```

### **Communication Pattern Selection**

| Scenario | Pattern | Example | Benefits |
|----------|---------|---------|----------|
| **Pure computation chain** | Direct linking | validation → contract-checker | Fast, type-safe |
| **System operations** | Event-driven | file-read → validation → git-commit | Loose coupling |
| **Large data processing** | Shared memory | data-processor-a ↔ data-processor-b | High performance |
| **Cross-domain operations** | Event-driven | validation → notification → logging | Scalable |

## 🔧 **Implementation Examples**

### **Enhanced Orchestrator with Component Linking**

```rust
// src/orchestrator/mod.rs
impl HooksmithOrchestrator {
    /// Load and link components for direct communication
    pub async fn load_linked_components(&mut self) -> Result<()> {
        let engine = &self.runtime.engine;
        let mut linker = Linker::new(engine);

        // Load validation-handler (exports validation functions)
        let validation_component = Component::from_file(engine, "validation-handler.component.wasm")?;
        linker.instantiate(&mut self.runtime.store, &validation_component)?;

        // Load contract-checker (imports from validation-handler)
        let checker_component = Component::from_file(engine, "contract-checker.component.wasm")?;
        let checker = ContractChecker::new(&mut self.runtime.store, &checker_component)?;

        // Store linked component for direct calls
        self.linked_components.insert("contract-checker".to_string(), checker);

        Ok(())
    }

    /// Direct component call (fast path)
    pub async fn validate_contract_direct(&self, contract_data: &str) -> Result<ValidationResult> {
        if let Some(checker) = self.linked_components.get("contract-checker") {
            // Direct function call - nanosecond latency
            checker.check_contract(contract_data).await
        } else {
            // Fallback to event-driven approach
            self.validate_contract_via_events("contract", "data", contract_data, true, false).await
        }
    }
}
```

### **Component Interface Definitions**

```wit
// wit/validation-handler.wit
package hooksmith:validation;

interface validation {
    validate-contract: func(
        contract-name: string,
        content: string,
        config: validation-config
    ) -> validation-result;
}

export validation;

// wit/contract-checker.wit
package hooksmith:contract-checker;

import hooksmith:validation/validation;

interface contract-checker {
    check-contract: func(
        contract-data: string,
        rules: list<string>
    ) -> check-result;
}

export contract-checker;
```

### **Event-Driven Fallback**

```rust
// When direct linking isn't available or appropriate
impl HooksmithOrchestrator {
    /// Event-driven validation (fallback path)
    pub async fn validate_contract_via_events(
        &self,
        contract_name: &str,
        file_path: &str,
        content: &str,
        strict: bool,
        store_proof: bool,
    ) -> Result<ValidationResult> {
        // Create validation request event
        let validation_event = HooksmithEvent::new(
            "orchestrator".to_string(),
            "validation_request".to_string(),
            json!({
                "contract_name": contract_name,
                "file_path": file_path,
                "content": content,
                "validation_config": {
                    "strict": strict,
                    "store_proof": store_proof
                }
            }),
        );

        // Route through event bus
        self.route_event(validation_event).await?;

        // Wait for result (in real implementation)
        Ok(ValidationResult { /* ... */ })
    }
}
```

## 🎯 **Best Practices**

### **1. Choose the Right Pattern**

- **Direct linking** for pure computation chains
- **Event-driven** for system operations and cross-domain communication
- **Shared memory** only for performance-critical, same-process operations

### **2. Component Design**

- **Keep WIT components pure** - No side effects, no shared state
- **Use WIT interfaces** for type-safe communication
- **Export clear APIs** - Well-defined function signatures

### **3. Performance Optimization**

- **Profile communication patterns** - Measure latency and throughput
- **Use direct linking** for hot paths
- **Batch operations** when possible
- **Cache component instances** - Avoid repeated instantiation

### **4. Security Considerations**

- **Isolate components** - Use separate stores when needed
- **Validate inputs** - Check data at boundaries
- **Limit shared memory** - Use sparingly and carefully
- **Audit event flows** - Log all cross-component communication

## 🔄 **Migration Strategy**

### **Phase 1: Identify Communication Patterns**
- [ ] Audit existing component interactions
- [ ] Categorize by communication type (pure computation vs system operations)
- [ ] Identify performance bottlenecks

### **Phase 2: Implement Direct Linking**
- [ ] Define WIT interfaces for component chains
- [ ] Implement component linking in orchestrator
- [ ] Add direct call methods

### **Phase 3: Optimize Event-Driven Communication**
- [ ] Refine event schemas
- [ ] Implement async result handling
- [ ] Add event correlation

### **Phase 4: Performance Tuning**
- [ ] Benchmark different patterns
- [ ] Optimize hot paths
- [ ] Add monitoring and metrics

## 📊 **Performance Comparison**

| Pattern | Latency | Throughput | Complexity | Use Case |
|---------|---------|------------|------------|----------|
| **Direct linking** | ~100ns | Very high | Low | Pure computation chains |
| **Event-driven** | ~1-10ms | High | Medium | System operations |
| **Shared memory** | ~10-100ns | Very high | High | Performance-critical data processing |

## 🎉 **Conclusion**

Hooksmith's hybrid architecture leverages the right Wasmtime communication patterns for each use case:

1. **Direct linking** for fast, type-safe pure computation
2. **Event-driven** for scalable system operations
3. **Shared memory** for performance-critical operations

This approach provides:
- ✅ **Optimal performance** - Right tool for each job
- ✅ **Type safety** - WIT interfaces throughout
- ✅ **Scalability** - Event-driven for system operations
- ✅ **Maintainability** - Clear separation of concerns

The architecture is well-positioned to take advantage of Wasmtime's component model while maintaining the flexibility and scalability of event-driven communication. 
