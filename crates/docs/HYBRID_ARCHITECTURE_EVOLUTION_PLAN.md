# Hybrid Architecture Evolution Plan: Event Bus Integration

## 🎯 **Overview**

Your repository already has a **comprehensive event bus system** that's perfectly positioned to serve as the integration layer for the hybrid WIT + native Rust architecture. This plan shows how to evolve the existing system to support the hybrid model.

## ✅ **What You Already Have**

### **Event Bus Infrastructure**
- ✅ **Core Event Bus** (`crates/xtask/src/event_bus.rs`)
  - In-memory broadcast channels
  - JSONL persistence
  - Event handlers and processors
  - Statistics and metrics

- ✅ **WASM Event Bus Host** (`crates/xtask/src/wasm_event_bus.rs`)
  - WASM component loading and registration
  - Event routing to WASM components
  - Component metadata management

- ✅ **WIT Event Bus Interface** (`wit/event-bus.wit`)
  - Language-agnostic event definitions
  - Component handler interfaces
  - Event subscription and routing

### **Event Types Already Defined**
```rust
// Current event structure
pub struct HooksmithEvent {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub event: String,
    pub hook: Option<String>,
    pub state: Option<String>,
    pub context: Value,
    pub error: Option<Value>,
    pub session_id: Option<String>,
    pub duration_ms: Option<u64>,
}
```

## 🔄 **Evolution Strategy**

### **Phase 1: Extend Event Types for Hybrid Architecture**

#### **1.1 Add System Operation Events**
```rust
// New event types for system operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    // File operations
    FileReadRequest { path: String, context: Value },
    FileReadResult { path: String, content: Option<String>, error: Option<String> },
    FileWriteRequest { path: String, content: String, context: Value },
    FileWriteResult { path: String, success: bool, error: Option<String> },
    
    // Git operations
    GitCommitRequest { message: String, files: Vec<String>, context: Value },
    GitCommitResult { hash: Option<String>, success: bool, error: Option<String> },
    GitPushRequest { branch: String, remote: String, context: Value },
    GitPushResult { success: bool, error: Option<String> },
    
    // Process operations
    ProcessExecuteRequest { command: String, args: Vec<String>, context: Value },
    ProcessExecuteResult { exit_code: i32, stdout: String, stderr: String },
    
    // Network operations
    NetworkRequest { url: String, method: String, context: Value },
    NetworkResult { status_code: u16, body: String, error: Option<String> },
}
```

#### **1.2 Add Pure Computation Events**
```rust
// New event types for pure computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputationEvent {
    // Validation events
    ValidationRequest { content: String, rules: Vec<String>, context: Value },
    ValidationResult { valid: bool, errors: Vec<String>, warnings: Vec<String> },
    
    // Contract events
    ContractCheckRequest { content: String, contract_name: String, context: Value },
    ContractCheckResult { valid: bool, violations: Vec<String> },
    
    // Checksum events
    ChecksumCalculateRequest { content: String, algorithm: String, context: Value },
    ChecksumCalculateResult { checksum: String, algorithm: String },
    
    // Policy evaluation events
    PolicyEvaluateRequest { content: String, policies: Vec<String>, context: Value },
    PolicyEvaluateResult { allowed: bool, reasons: Vec<String> },
}
```

### **Phase 2: Create Native System Operation Handlers**

#### **2.1 File System Handler**
```rust
// crates/file-operations/src/event_handler.rs
pub struct FileSystemEventHandler {
    base_path: PathBuf,
}

impl EventHandler for FileSystemEventHandler {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()> {
        match event.event.as_str() {
            "file_read_request" => self.handle_file_read(event),
            "file_write_request" => self.handle_file_write(event),
            _ => Ok(()),
        }
    }
    
    fn name(&self) -> &str { "file-system-handler" }
    
    fn should_handle(&self, event: &HooksmithEvent) -> bool {
        matches!(event.event.as_str(), "file_read_request" | "file_write_request")
    }
}

impl FileSystemEventHandler {
    fn handle_file_read(&self, event: &HooksmithEvent) -> Result<()> {
        let path = event.context["path"].as_str().unwrap_or("");
        let full_path = self.base_path.join(path);
        
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                let result_event = HooksmithEvent::new(
                    "file-system-handler".to_string(),
                    "file_read_result".to_string(),
                    json!({
                        "path": path,
                        "content": content,
                        "success": true
                    })
                );
                emit_event(result_event)?;
            }
            Err(e) => {
                let result_event = HooksmithEvent::new(
                    "file-system-handler".to_string(),
                    "file_read_result".to_string(),
                    json!({
                        "path": path,
                        "success": false,
                        "error": e.to_string()
                    })
                );
                emit_event(result_event)?;
            }
        }
        Ok(())
    }
}
```

#### **2.2 Git Operations Handler**
```rust
// crates/git-operations/src/event_handler.rs
pub struct GitOperationsEventHandler {
    repo_path: PathBuf,
}

impl EventHandler for GitOperationsEventHandler {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()> {
        match event.event.as_str() {
            "git_commit_request" => self.handle_git_commit(event),
            "git_push_request" => self.handle_git_push(event),
            _ => Ok(()),
        }
    }
    
    fn name(&self) -> &str { "git-operations-handler" }
    
    fn should_handle(&self, event: &HooksmithEvent) -> bool {
        matches!(event.event.as_str(), "git_commit_request" | "git_push_request")
    }
}

impl GitOperationsEventHandler {
    fn handle_git_commit(&self, event: &HooksmithEvent) -> Result<()> {
        let message = event.context["message"].as_str().unwrap_or("");
        let files = event.context["files"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|f| f.as_str())
            .collect::<Vec<_>>();
        
        // Execute git commit
        let output = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()?;
        
        if output.status.success() {
            let result_event = HooksmithEvent::new(
                "git-operations-handler".to_string(),
                "git_commit_result".to_string(),
                json!({
                    "success": true,
                    "message": message,
                    "files": files
                })
            );
            emit_event(result_event)?;
        } else {
            let result_event = HooksmithEvent::new(
                "git-operations-handler".to_string(),
                "git_commit_result".to_string(),
                json!({
                    "success": false,
                    "error": String::from_utf8_lossy(&output.stderr)
                })
            );
            emit_event(result_event)?;
        }
        
        Ok(())
    }
}
```

### **Phase 3: Create WIT Component Event Handlers**

#### **3.1 Validation Component Handler**
```rust
// crates/components/validation-handler/src/event_handler.rs
impl EventHandler for ValidationHandlerComponent {
    fn handle_event(&mut self, event: &HooksmithEvent) -> Result<()> {
        match event.event.as_str() {
            "validation_request" => self.handle_validation_request(event),
            "contract_check_request" => self.handle_contract_check(event),
            _ => Ok(()),
        }
    }
    
    fn name(&self) -> &str { "validation-handler" }
    
    fn should_handle(&self, event: &HooksmithEvent) -> bool {
        matches!(event.event.as_str(), "validation_request" | "contract_check_request")
    }
}

impl ValidationHandlerComponent {
    fn handle_validation_request(&self, event: &HooksmithEvent) -> Result<()> {
        let content = event.context["content"].as_str().unwrap_or("");
        let rules = event.context["rules"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| r.as_str())
            .collect::<Vec<_>>();
        
        // Pure computation - no I/O
        let validation_result = self.validate_content(content, rules)?;
        
        let result_event = HooksmithEvent::new(
            "validation-handler".to_string(),
            "validation_result".to_string(),
            json!({
                "valid": validation_result.success,
                "errors": validation_result.errors,
                "warnings": validation_result.warnings
            })
        );
        emit_event(result_event)?;
        
        Ok(())
    }
}
```

### **Phase 4: Update CLI Host for Event-Driven Orchestration**

#### **4.1 Event-Driven Command Implementation**
```rust
// src/commands/contract_validation.rs
pub async fn validate_contract(file_path: &str) -> Result<()> {
    // 1. Emit file read request
    let read_event = HooksmithEvent::new(
        "cli-contract-validation".to_string(),
        "file_read_request".to_string(),
        json!({
            "path": file_path,
            "context": "contract_validation"
        })
    );
    emit_event(read_event)?;
    
    // 2. Wait for file read result
    let mut subscriber = get_event_bus().unwrap().subscribe();
    while let Ok(event) = subscriber.recv().await {
        if event.event == "file_read_result" && 
           event.context["path"].as_str() == Some(file_path) {
            
            if let Some(content) = event.context["content"].as_str() {
                // 3. Emit validation request to WIT component
                let validation_event = HooksmithEvent::new(
                    "cli-contract-validation".to_string(),
                    "validation_request".to_string(),
                    json!({
                        "content": content,
                        "rules": ["contract_rules"],
                        "context": "contract_validation"
                    })
                );
                emit_event(validation_event)?;
                break;
            } else {
                return Err(anyhow::anyhow!("Failed to read file: {}", file_path));
            }
        }
    }
    
    // 4. Wait for validation result
    while let Ok(event) = subscriber.recv().await {
        if event.event == "validation_result" {
            let valid = event.context["valid"].as_bool().unwrap_or(false);
            if !valid {
                // 5. Emit file write request for report
                let write_event = HooksmithEvent::new(
                    "cli-contract-validation".to_string(),
                    "file_write_request".to_string(),
                    json!({
                        "path": "validation-report.json",
                        "content": serde_json::to_string(&event.context)?,
                        "context": "contract_validation"
                    })
                );
                emit_event(write_event)?;
            }
            break;
        }
    }
    
    Ok(())
}
```

### **Phase 5: Create Event Schemas and Registry**

#### **5.1 Event Schema Definitions**
```jsonc
// schemas/events/contract-validation.schema.jsonc
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Contract Validation Event Schema",
  "description": "Schema for contract validation events",
  
  "definitions": {
    "validation_request": {
      "type": "object",
      "properties": {
        "content": { "type": "string" },
        "rules": { "type": "array", "items": { "type": "string" } },
        "context": { "type": "string" }
      },
      "required": ["content", "rules"]
    },
    
    "validation_result": {
      "type": "object",
      "properties": {
        "valid": { "type": "boolean" },
        "errors": { "type": "array", "items": { "type": "string" } },
        "warnings": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["valid"]
    }
  }
}
```

#### **5.2 Event Registry**
```jsonc
// config/event-registry.jsonc
{
  "events": {
    "file_read_request": {
      "handler": "file-system-handler",
      "schema": "schemas/events/file-operations.schema.jsonc",
      "category": "system"
    },
    "file_read_result": {
      "handler": "event-bus",
      "schema": "schemas/events/file-operations.schema.jsonc",
      "category": "system"
    },
    "validation_request": {
      "handler": "validation-handler",
      "schema": "schemas/events/contract-validation.schema.jsonc",
      "category": "computation"
    },
    "validation_result": {
      "handler": "event-bus",
      "schema": "schemas/events/contract-validation.schema.jsonc",
      "category": "computation"
    },
    "git_commit_request": {
      "handler": "git-operations-handler",
      "schema": "schemas/events/git-operations.schema.jsonc",
      "category": "system"
    },
    "git_commit_result": {
      "handler": "event-bus",
      "schema": "schemas/events/git-operations.schema.jsonc",
      "category": "system"
    }
  },
  
  "handlers": {
    "file-system-handler": {
      "type": "native",
      "crate": "file-operations",
      "events": ["file_read_request", "file_write_request"]
    },
    "git-operations-handler": {
      "type": "native",
      "crate": "git-operations",
      "events": ["git_commit_request", "git_push_request"]
    },
    "validation-handler": {
      "type": "wit",
      "component": "validation-handler",
      "events": ["validation_request", "contract_check_request"]
    },
    "hook-builder": {
      "type": "wit",
      "component": "hook-builder",
      "events": ["build_request", "validate_source_request"]
    }
  }
}
```

## 🚀 **Implementation Timeline**

### **Week 1: Event Type Extensions**
- [ ] Extend `HooksmithEvent` with system operation events
- [ ] Add computation events for WIT components
- [ ] Update WIT event bus interface
- [ ] Create event schemas

### **Week 2: Native System Handlers**
- [ ] Create `crates/file-operations/` crate
- [ ] Create `crates/git-operations/` crate
- [ ] Implement event handlers for system operations
- [ ] Add handlers to event bus

### **Week 3: WIT Component Integration**
- [ ] Update existing WIT components to handle events
- [ ] Implement event-driven validation logic
- [ ] Add component event handlers to WASM event bus
- [ ] Test WIT component event handling

### **Week 4: CLI Orchestration**
- [ ] Refactor CLI commands to use event-driven approach
- [ ] Implement event waiting and response handling
- [ ] Add event-driven contract validation
- [ ] Test end-to-end event flow

### **Week 5: Registry and Validation**
- [ ] Create event registry system
- [ ] Add event schema validation
- [ ] Implement event routing based on registry
- [ ] Add comprehensive testing

## 🎉 **Benefits of This Approach**

### **Clear Separation of Concerns**
- ✅ **WIT components** handle only pure computation
- ✅ **Native handlers** handle all system operations
- ✅ **Event bus** provides clean integration layer

### **Language Agnostic**
- ✅ **WIT interfaces** allow components in any language
- ✅ **Event schemas** provide contract validation
- ✅ **Registry system** enables dynamic component loading

### **Scalable Architecture**
- ✅ **Event-driven** design supports complex workflows
- ✅ **Handler registration** enables plugin architecture
- ✅ **Event persistence** supports debugging and replay

### **Production Ready**
- ✅ **Existing infrastructure** already supports this
- ✅ **Comprehensive tooling** for validation and testing
- ✅ **CI/CD integration** already in place

## 🎯 **Next Steps**

1. **Start with Phase 1** - extend event types for hybrid architecture
2. **Create native system handlers** for file and Git operations
3. **Update WIT components** to handle computation events
4. **Refactor CLI** to use event-driven orchestration
5. **Add event registry** for dynamic component management

This approach leverages your existing event bus infrastructure while providing the clean separation between WIT components (pure computation) and native Rust (system operations) that the hybrid architecture requires. 
