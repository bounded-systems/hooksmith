# Hybrid Architecture Integration with Existing Orchestrator

## 🎯 **Overview**

This document explains how to integrate the hybrid WIT + native Rust architecture with Hooksmith's existing orchestrator system. The goal is to enhance the current orchestrator to support event-driven communication between WIT components and native handlers.

## ✅ **Current Architecture Analysis**

### **Existing Orchestrator System**
Your Hooksmith repository already has a sophisticated orchestrator system:

```
src/orchestrator/
├── mod.rs           # Main orchestrator coordination
├── components.rs    # Component handles and registry
├── config.rs        # Configuration management
├── router.rs        # Command routing
└── runtime.rs       # WASM runtime management
```

### **Existing WIT Components**
```
crates/components/
├── git-filter/      # Git object filtering
├── hook-builder/    # Hook building and compilation
├── validation-handler/ # Contract validation
└── worktree-runner/ # Worktree operations
```

### **Existing Event Bus**
```
crates/xtask/src/
├── event_bus.rs     # Core event bus infrastructure
└── wasm_event_bus.rs # WASM component event integration
```

## 🔧 **Integration Strategy**

### **Phase 1: Enhance Existing Orchestrator**

#### **1.1 Add Event-Driven Component Communication**
Extend the existing `ComponentHandle` to support event-driven communication:

```rust
// src/orchestrator/components.rs
impl ComponentHandle {
    /// Emit an event to the component
    pub async fn emit_event(&self, event: HooksmithEvent) -> Result<()> {
        // Use existing event bus infrastructure
        crate::xtask::event_bus::emit_event(event)
    }

    /// Subscribe to events from the component
    pub async fn subscribe_to_events(&self, event_types: Vec<String>) -> Result<EventSubscription> {
        // Use existing WASM event bus
        crate::xtask::wasm_event_bus::subscribe_to_component_events(&self.name, event_types).await
    }
}
```

#### **1.2 Add Native Handler Integration**
Extend the orchestrator to manage native handlers alongside WIT components:

```rust
// src/orchestrator/mod.rs
pub struct HooksmithOrchestrator {
    runtime: WasmRuntime,
    router: CommandRouter,
    config: OrchestratorConfig,
    components: HashMap<String, ComponentHandle>,
    // NEW: Native handlers
    native_handlers: HashMap<String, Box<dyn EventHandler>>,
    // NEW: Event bus integration
    event_bus: EventBusManager,
}
```

#### **1.3 Add Event Bus Manager**
Create a new module to manage event routing:

```rust
// src/orchestrator/event_bus.rs
pub struct EventBusManager {
    registry: EventRegistry,
    subscriptions: HashMap<String, EventSubscription>,
}

impl EventBusManager {
    /// Route events to appropriate handlers
    pub async fn route_event(&self, event: HooksmithEvent) -> Result<()> {
        let handler = self.registry.get_handler_for_event(&event.event)?;
        
        match handler.handler_type {
            HandlerType::Wit => {
                // Route to WIT component
                self.route_to_wit_component(event).await
            }
            HandlerType::Native => {
                // Route to native handler
                self.route_to_native_handler(event).await
            }
        }
    }
}
```

### **Phase 2: Update Existing Components**

#### **2.1 Enhance WIT Components for Events**
Update existing WIT components to handle events:

```rust
// crates/components/validation-handler/src/lib.rs
impl ValidationHandler {
    /// Handle validation events
    pub async fn handle_validation_event(&mut self, event: HooksmithEvent) -> Result<HooksmithEvent> {
        match event.event.as_str() {
            "validation_request" => {
                self.handle_validation_request(event).await
            }
            "contract_check_request" => {
                self.handle_contract_check_request(event).await
            }
            _ => Err(anyhow::anyhow!("Unknown event type: {}", event.event))
        }
    }
}
```

#### **2.2 Add Event Handlers to Existing Components**
Register event handlers in the component initialization:

```rust
// crates/components/validation-handler/src/lib.rs
pub fn init() -> Result<()> {
    // Register with WASM event bus
    wasm_event_bus::register_component_handler(
        "validation-handler",
        ValidationEventHandler::new()
    )?;
    
    Ok(())
}
```

### **Phase 3: Integrate Native Handlers**

#### **3.1 Register Native Handlers with Orchestrator**
Add native handlers to the orchestrator:

```rust
// src/orchestrator/mod.rs
impl HooksmithOrchestrator {
    /// Register a native handler
    pub fn register_native_handler(&mut self, name: String, handler: Box<dyn EventHandler>) {
        self.native_handlers.insert(name, handler);
    }

    /// Initialize default native handlers
    pub async fn init_default_handlers(&mut self) -> Result<()> {
        // File operations handler
        let file_handler = FileOperationsEventHandler::new(
            std::env::current_dir()?,
            self.config.session_id.clone()
        );
        self.register_native_handler("file-operations".to_string(), Box::new(file_handler));

        // Git operations handler
        let git_handler = GitOperationsEventHandler::new(
            std::env::current_dir()?,
            self.config.session_id.clone()
        );
        self.register_native_handler("git-operations".to_string(), Box::new(git_handler));

        Ok(())
    }
}
```

#### **3.2 Update CLI Commands to Use Event-Driven Approach**
Refactor existing CLI commands to use the event bus:

```rust
// src/commands/contract_validation.rs
pub async fn validate_contract_event_driven(
    contract_name: &str,
    file_path: &str,
    strict: bool,
    store_proof: bool,
) -> Result<()> {
    let orchestrator = HooksmithOrchestrator::new().await?;
    
    // Step 1: Read file using native handler
    let file_content = orchestrator.read_file_via_events(file_path).await?;
    
    // Step 2: Validate using WIT component
    let validation_result = orchestrator.validate_via_events(
        contract_name,
        &file_content,
        strict,
        store_proof
    ).await?;
    
    // Step 3: Store proof if needed
    if validation_result.valid && store_proof {
        orchestrator.store_proof_via_events(file_path, &validation_result).await?;
    }
    
    Ok(())
}
```

## 🔄 **Migration Path**

### **Step 1: Enhance Orchestrator (Week 1)**
- [ ] Add event bus manager to orchestrator
- [ ] Extend component handles for event communication
- [ ] Add native handler registration
- [ ] Update orchestrator configuration

### **Step 2: Update WIT Components (Week 2)**
- [ ] Add event handling to validation-handler
- [ ] Add event handling to hook-builder
- [ ] Add event handling to worktree-runner
- [ ] Add event handling to git-filter

### **Step 3: Add Native Handlers (Week 3)**
- [ ] Integrate file-operations handler
- [ ] Integrate git-operations handler
- [ ] Register handlers with orchestrator
- [ ] Test handler integration

### **Step 4: Update CLI Commands (Week 4)**
- [ ] Refactor contract validation command
- [ ] Refactor auto-push command
- [ ] Refactor git-filter command
- [ ] Add event-driven workflows

### **Step 5: Testing and Validation (Week 5)**
- [ ] Add integration tests
- [ ] Add performance benchmarks
- [ ] Validate all workflows
- [ ] Update documentation

## 🎯 **Benefits of This Integration**

### **Leverages Existing Infrastructure**
- ✅ **Uses existing orchestrator** - No need to rebuild from scratch
- ✅ **Uses existing event bus** - Extends current event infrastructure
- ✅ **Uses existing WIT components** - Enhances current components
- ✅ **Maintains compatibility** - Existing code continues to work

### **Enhances Current Capabilities**
- ✅ **Event-driven communication** - Adds event bus to existing components
- ✅ **Native handler integration** - Adds system operations to orchestrator
- ✅ **Better separation of concerns** - Clear WIT vs native boundaries
- ✅ **Improved scalability** - Event-driven architecture

### **Maintains Architecture Principles**
- ✅ **WIT-first approach** - Keeps existing WIT component structure
- ✅ **Hybrid architecture** - Adds native handlers for system operations
- ✅ **Event-driven integration** - Uses event bus for coordination
- ✅ **Incremental migration** - Gradual enhancement of existing system

## 🔧 **Implementation Details**

### **Event Registry Integration**
```rust
// src/orchestrator/event_registry.rs
pub struct EventRegistry {
    events: HashMap<String, EventDefinition>,
    handlers: HashMap<String, HandlerDefinition>,
}

impl EventRegistry {
    /// Load registry from config
    pub fn from_config(config: &OrchestratorConfig) -> Result<Self> {
        let registry_path = config.event_registry_path.clone();
        let registry_content = std::fs::read_to_string(registry_path)?;
        let registry: EventRegistryConfig = serde_json::from_str(&registry_content)?;
        
        Self::from_config(registry)
    }
}
```

### **Component Event Integration**
```rust
// src/orchestrator/components.rs
impl ComponentHandle {
    /// Handle events for this component
    pub async fn handle_event(&self, event: HooksmithEvent) -> Result<HooksmithEvent> {
        // Route to appropriate WIT component function
        match event.event.as_str() {
            "validation_request" => {
                self.call("handle_validation_request", event).await
            }
            "contract_check_request" => {
                self.call("handle_contract_check_request", event).await
            }
            _ => Err(anyhow::anyhow!("Unknown event type"))
        }
    }
}
```

### **Orchestrator Event Routing**
```rust
// src/orchestrator/mod.rs
impl HooksmithOrchestrator {
    /// Route an event to the appropriate handler
    pub async fn route_event(&self, event: HooksmithEvent) -> Result<()> {
        let handler_name = self.event_bus.get_handler_for_event(&event.event)?;
        
        if let Some(component) = self.components.get(&handler_name) {
            // Route to WIT component
            component.handle_event(event).await?;
        } else if let Some(handler) = self.native_handlers.get(&handler_name) {
            // Route to native handler
            handler.handle_event(&event)?;
        } else {
            return Err(anyhow::anyhow!("No handler found for event: {}", event.event));
        }
        
        Ok(())
    }
}
```

## 🎉 **Conclusion**

This integration approach:

1. **Leverages your existing infrastructure** - Uses the orchestrator, event bus, and WIT components you already have
2. **Enhances current capabilities** - Adds event-driven communication and native handlers
3. **Maintains architecture principles** - Keeps the WIT-first approach while adding hybrid capabilities
4. **Provides incremental migration** - Allows gradual enhancement without breaking existing functionality

The result is a seamless integration that enhances your current hybrid WIT-first monorepo with event-driven communication and clear separation between WIT components (pure computation) and native handlers (system operations). 
