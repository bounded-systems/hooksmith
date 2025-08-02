# Hooksmith Architecture: WASM Components with Orchestrators

## 🎯 Overview

Hooksmith is being refactored from a monolithic CLI into a **WASM component-based architecture** with **orchestrators** for loose coupling and high flexibility.

## 🏗️ New Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hooksmith Orchestrator                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   CLI Router    │  │  WASM Runtime   │  │  Config Mgmt    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    WASM Component Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Hook Builder    │  │ Worktree Mgmt   │  │ Git Filter      │ │
│  │ Component       │  │ Component       │  │ Component       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Lefthook Gen    │  │ Validation      │  │ Schema Mgmt     │ │
│  │ Component       │  │ Component       │  │ Component       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    External Tool Layer                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Lefthook        │  │ Git Tools       │  │ Build Tools     │ │
│  │ (Git Hooks)     │  │ (wtp, wt, etc.) │  │ (cargo, etc.)   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 🔧 Component Architecture

### 1. **Orchestrator Layer**
- **CLI Router**: Command parsing and routing to appropriate components
- **WASM Runtime**: Manages WASM component lifecycle and communication
- **Config Management**: Centralized configuration and state management

### 2. **WASM Components**
Each component is a self-contained WASM module with a well-defined interface:

#### **Hook Builder Component**
- **Purpose**: Compile Rust code into binary executables for Git hooks
- **Interface**: `hook-builder.wit`
- **Responsibilities**:
  - Rust compilation pipeline
  - Binary optimization
  - Hook metadata generation

#### **Worktree Management Component**
- **Purpose**: Manage Git worktrees using various tools
- **Interface**: `worktree-manager.wit`
- **Responsibilities**:
  - Tool detection (wtp, wt, treekanga, git)
  - Worktree operations (create, list, switch, remove)
  - Tool orchestration

#### **Git Filter Component**
- **Purpose**: Process Git objects and apply filters
- **Interface**: `git-filter.wit`
- **Responsibilities**:
  - Blob processing
  - Tree filtering
  - Contract validation

#### **Lefthook Generator Component**
- **Purpose**: Generate Lefthook configuration files
- **Interface**: `lefthook-generator.wit`
- **Responsibilities**:
  - YAML configuration generation
  - Schema validation
  - Hook integration

#### **Validation Component**
- **Purpose**: Validate configurations and hooks
- **Interface**: `validation.wit`
- **Responsibilities**:
  - Configuration validation
  - Hook validation
  - Schema compliance checking

#### **Schema Management Component**
- **Purpose**: Manage WIT schemas and interfaces
- **Interface**: `schema-manager.wit`
- **Responsibilities**:
  - WIT interface generation
  - Schema versioning
  - Interface compatibility

## 🔄 Communication Flow

### Component-to-Component Communication
```
Component A ──WIT Interface──► Component B
     │                              │
     └─── Shared Data Types ────────┘
```

### Orchestrator-to-Component Communication
```
Orchestrator ──WASM Runtime──► Component
     │                              │
     └─── Config & State ───────────┘
```

## 📦 Component Structure

Each component follows this structure:
```
components/
├── hook-builder/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── builder.rs
│   │   └── compiler.rs
│   └── wit/
│       └── hook-builder.wit
├── worktree-manager/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── manager.rs
│   │   └── tools/
│   └── wit/
│       └── worktree-manager.wit
└── ...
```

## 🎯 Benefits of This Architecture

### 1. **Loose Coupling**
- Components are independent and can be developed/tested separately
- Changes to one component don't affect others
- Easy to add/remove components

### 2. **High Flexibility**
- Components can be swapped out or replaced
- Different implementations can coexist
- Easy to extend with new functionality

### 3. **Better Testing**
- Each component can be tested in isolation
- Mock components for testing
- Component-level integration tests

### 4. **Performance**
- Components can be optimized independently
- Lazy loading of components
- Parallel execution where possible

### 5. **Maintainability**
- Clear separation of concerns
- Well-defined interfaces
- Easier to understand and modify

## 🚀 Migration Plan

### Phase 1: Foundation
1. Create orchestrator framework
2. Define WIT interfaces for all components
3. Set up WASM runtime integration

### Phase 2: Component Migration
1. Migrate existing worktree-runner to new architecture
2. Create hook-builder component
3. Create lefthook-generator component

### Phase 3: Integration
1. Update CLI to use orchestrator
2. Implement component communication
3. Add configuration management

### Phase 4: Enhancement
1. Add new components (validation, schema management)
2. Optimize performance
3. Add advanced features

## 🔧 Implementation Details

### WIT Interface Example
```wit
package hooksmith:hook-builder;

interface hook-builder {
  build-hook: func(config: build-config) -> result<build-result, string>;
  validate-source: func(source-path: string) -> result<validation-result, string>;
  optimize-binary: func(binary-path: string) -> result<optimization-result, string>;
}

record build-config {
  source-path: string,
  output-path: string,
  target-triple: string,
  optimization-level: u8,
}
```

### Orchestrator Example
```rust
pub struct HooksmithOrchestrator {
    wasm_runtime: WasmRuntime,
    components: HashMap<String, ComponentHandle>,
    config: Config,
}

impl HooksmithOrchestrator {
    pub async fn build_hook(&self, config: BuildConfig) -> Result<BuildResult> {
        let hook_builder = self.get_component("hook-builder")?;
        hook_builder.call("build-hook", config).await
    }
}
```

This architecture provides the foundation for a truly modular, maintainable, and extensible Hooksmith system. 
