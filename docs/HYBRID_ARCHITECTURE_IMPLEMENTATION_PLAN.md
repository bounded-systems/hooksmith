# Hybrid Architecture Implementation Plan

## 🎯 **Overview**

This document outlines the step-by-step implementation plan for evolving Hooksmith toward a hybrid WIT + native Rust architecture with an event bus as the integration layer. The goal is to achieve clear separation between pure computation (WIT components) and system operations (native Rust), while maintaining the existing functionality.

## ✅ **Current State Analysis**

### **What We Already Have**
- ✅ **Event Bus Infrastructure** (`crates/xtask/src/event_bus.rs`)
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

- ✅ **Existing WIT Components**
  - `validation-handler` - Contract validation
  - `hook-builder` - Hook building and compilation
  - `worktree-runner` - Worktree operations
  - `git-filter` - Git filtering operations

- ✅ **Native System Crates**
  - `xtask` - Build orchestration and CLI
  - `lefthook-rs` - Git hooks integration
  - `git-filter` - Git object filtering

## 🚀 **Implementation Phases**

### **Phase 1: Event Schema and Registry (Week 1)**

#### **1.1 Event Schemas** ✅ COMPLETED
- [x] `schemas/events/contract-validation.schema.jsonc`
- [x] `schemas/events/git-operations.schema.jsonc`
- [x] `schemas/events/file-operations.schema.jsonc`

#### **1.2 Event Registry** ✅ COMPLETED
- [x] `config/event-registry.jsonc` - Event routing configuration

#### **1.3 Schema Validation**
- [ ] Add schema validation to event bus
- [ ] Implement event schema loading from registry
- [ ] Add validation middleware for events

### **Phase 2: Native System Handlers (Week 2)**

#### **2.1 File Operations Handler** ✅ COMPLETED
- [x] `crates/file-operations/Cargo.toml`
- [x] `crates/file-operations/src/lib.rs`
- [x] `crates/file-operations/src/operations.rs`
- [x] `crates/file-operations/src/event_handler.rs`

#### **2.2 Git Operations Handler** ✅ COMPLETED
- [x] `crates/git-operations/Cargo.toml`
- [x] `crates/git-operations/src/lib.rs`
- [ ] `crates/git-operations/src/operations.rs`
- [ ] `crates/git-operations/src/event_handler.rs`

#### **2.3 Handler Registration**
- [ ] Update `xtask` to register native handlers
- [ ] Add handler discovery and loading
- [ ] Implement handler lifecycle management

### **Phase 3: WIT Component Event Integration (Week 3)**

#### **3.1 Update Existing WIT Components**
- [ ] Update `validation-handler` to handle events
- [ ] Update `hook-builder` to handle events
- [ ] Update `worktree-runner` to handle events

#### **3.2 Event-Driven Validation**
- [ ] Implement event-driven contract validation
- [ ] Add validation result event emission
- [ ] Update validation workflow

#### **3.3 Component Event Handlers**
- [ ] Add event handlers to WASM event bus
- [ ] Implement component event routing
- [ ] Add component event processing

### **Phase 4: CLI Orchestration Refactor (Week 4)**

#### **4.1 Event-Driven CLI Commands**
- [ ] Refactor `src/commands/contract_validation.rs`
- [ ] Implement event waiting and response handling
- [ ] Add event-driven contract validation

#### **4.2 Workflow Orchestration**
- [ ] Implement validation workflow
- [ ] Add Git operations workflow
- [ ] Create file operations workflow

#### **4.3 Event Response Handling**
- [ ] Add event response aggregation
- [ ] Implement timeout handling
- [ ] Add error recovery mechanisms

### **Phase 5: Registry and Validation System (Week 5)**

#### **5.1 Event Registry System**
- [ ] Create event registry loading
- [ ] Add dynamic handler registration
- [ ] Implement event routing based on registry

#### **5.2 Schema Validation**
- [ ] Add event schema validation
- [ ] Implement validation middleware
- [ ] Add validation error handling

#### **5.3 Testing and Validation**
- [ ] Add comprehensive testing
- [ ] Implement integration tests
- [ ] Add performance benchmarks

## 🔧 **Technical Implementation Details**

### **Event Flow Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Command   │───▶│   Event Bus     │───▶│ Native Handler  │
│                 │    │                 │    │ (File/Git Ops)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  WIT Component  │
                       │ (Validation)    │
                       └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Event Bus     │
                       │ (Result Events) │
                       └─────────────────┘
```

### **Event Types and Routing**

#### **System Operations (Native Handlers)**
- `file_read_request` → `file-operations-handler`
- `file_write_request` → `file-operations-handler`
- `git_commit_request` → `git-operations-handler`
- `git_push_request` → `git-operations-handler`

#### **Pure Computation (WIT Components)**
- `validation_request` → `validation-handler`
- `contract_check_request` → `validation-handler`
- `build_request` → `hook-builder`
- `validate_source_request` → `hook-builder`

### **Event Schema Structure**

```jsonc
{
  "event_type": "validation_request",
  "request_id": "uuid",
  "contract_name": "string",
  "file_path": "string",
  "validation_config": {
    "strict": "boolean",
    "store_proof": "boolean",
    "rules": ["string"]
  },
  "metadata": {
    "component": "string",
    "timestamp": "datetime",
    "session_id": "string"
  }
}
```

## 🎯 **Migration Strategy**

### **Incremental Migration**
1. **Start with new features** - Use hybrid architecture for new functionality
2. **Gradual refactoring** - Migrate existing commands one by one
3. **Backward compatibility** - Maintain existing APIs during transition
4. **Testing at each step** - Ensure functionality is preserved

### **Command Migration Order**
1. `contract validation` - Already has event bus integration
2. `auto-push` - Natural fit for event-driven workflow
3. `git-filter` - Leverage existing git-filter component
4. `hook-builder` - Use existing hook-builder component
5. Other commands - Migrate as needed

### **Component Migration Strategy**
1. **Keep existing WIT components** - No changes needed initially
2. **Add event handlers** - Implement event processing
3. **Update CLI commands** - Use event-driven approach
4. **Test thoroughly** - Ensure all functionality works

## 🧪 **Testing Strategy**

### **Unit Tests**
- [ ] Test individual event handlers
- [ ] Test event parsing and serialization
- [ ] Test schema validation
- [ ] Test error handling

### **Integration Tests**
- [ ] Test complete workflows
- [ ] Test event routing
- [ ] Test WIT component integration
- [ ] Test native handler integration

### **Performance Tests**
- [ ] Benchmark event processing
- [ ] Test concurrent event handling
- [ ] Measure memory usage
- [ ] Test scalability

## 📊 **Success Metrics**

### **Functional Metrics**
- [ ] All existing functionality preserved
- [ ] New event-driven features working
- [ ] WIT components properly integrated
- [ ] Native handlers functioning correctly

### **Performance Metrics**
- [ ] Event processing latency < 10ms
- [ ] Memory usage within acceptable limits
- [ ] No performance regression
- [ ] Scalability to 100+ concurrent events

### **Quality Metrics**
- [ ] 100% test coverage for new code
- [ ] All schemas validated
- [ ] Error handling comprehensive
- [ ] Documentation complete

## 🚨 **Risk Mitigation**

### **Technical Risks**
- **Event ordering issues** - Use request IDs and correlation
- **Memory leaks** - Implement proper cleanup
- **Performance degradation** - Benchmark and optimize
- **Schema evolution** - Version schemas properly

### **Migration Risks**
- **Breaking changes** - Maintain backward compatibility
- **Data loss** - Implement proper error handling
- **Rollback complexity** - Keep old code until stable
- **Testing gaps** - Comprehensive test coverage

## 📅 **Timeline**

### **Week 1: Foundation**
- [x] Event schemas and registry
- [ ] Schema validation system
- [ ] Basic event routing

### **Week 2: Native Handlers**
- [x] File operations handler
- [ ] Git operations handler
- [ ] Handler registration system

### **Week 3: WIT Integration**
- [ ] Update WIT components
- [ ] Event-driven validation
- [ ] Component event handlers

### **Week 4: CLI Refactor**
- [ ] Event-driven CLI commands
- [ ] Workflow orchestration
- [ ] Response handling

### **Week 5: Polish**
- [ ] Registry system
- [ ] Comprehensive testing
- [ ] Documentation and cleanup

## 🎉 **Expected Benefits**

### **Architecture Benefits**
- ✅ **Clear separation** between computation and system operations
- ✅ **Language agnostic** WIT components
- ✅ **Event-driven** design for scalability
- ✅ **Modular** and testable components

### **Development Benefits**
- ✅ **Easier testing** with event-driven architecture
- ✅ **Better error handling** with structured events
- ✅ **Improved debugging** with event logging
- ✅ **Extensible** design for new features

### **Operational Benefits**
- ✅ **Better monitoring** with event metrics
- ✅ **Easier troubleshooting** with event traces
- ✅ **Scalable** architecture for growth
- ✅ **Maintainable** codebase

## 🔄 **Next Steps**

1. **Complete Phase 1** - Finish schema validation system
2. **Implement Phase 2** - Complete Git operations handler
3. **Begin Phase 3** - Update WIT components for events
4. **Plan Phase 4** - Design CLI refactoring approach
5. **Prepare Phase 5** - Set up testing infrastructure

This implementation plan provides a clear roadmap for evolving Hooksmith toward the hybrid WIT + native Rust architecture while maintaining existing functionality and ensuring a smooth migration path. 
