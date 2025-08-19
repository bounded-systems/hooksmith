# Hooksmith Pipeline Architecture

This document provides a visual overview of the complete Hooksmith validation pipeline, combining the actor chain with strategic principles for a unified understanding of the system.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           HOOKSMITH VALIDATION PIPELINE                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │    HOOK     │───▶│   CONCERN   │───▶│  CONTRACT   │───▶│  VERIFIER   │     │
│  │ Entry Point │    │ Scope+Class │    │ Rule Set    │    │ Executes    │     │
│  │ Git/CI      │    │ Problem     │    │ Valid State │    │ Checks      │     │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │                   │         │
│         ▼                   ▼                   ▼                   ▼         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  AUDITOR    │    │INVESTIGATOR │    │ DISPATCHER  │    │   PLANNER   │     │
│  │ SARIF Log   │    │Root Cause   │    │ Route to    │    │ Fix Strategy│     │
│  │ Actionable  │    │Diagnosis    │    │ Fixer Set   │    │ DAG of      │     │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │                   │         │
│         ▼                   ▼                   ▼                   ▼         │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                           FIXERS                                        │   │
│  │                    Stateless Repair Tools                              │   │
│  │                                                                         │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │   │
│  │  │ File Mover  │  │Dir Creator  │  │Schema Fixer │  │License Fixer│   │   │
│  │  │ Move Files  │  │Create Dirs  │  │Fix JSON     │  │Add Headers  │   │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│         │                                                                       │
│         ▼                                                                       │
│  ┌─────────────┐                                                               │
│  │   REPEAT    │◀──────────────────────────────────────────────────────────────┘
│  │ Iterate     │
│  │ Until Pass  │
│  └─────────────┘
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 🔄 Pipeline Flow

### Phase 1: Pre-Validation
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    HOOK     │───▶│   CONCERN   │───▶│  CONTRACT   │
│ Entry Point │    │ Scope+Class │    │ Rule Set    │
│ Git/CI      │    │ Problem     │    │ Valid State │
└─────────────┘    └─────────────┘    └─────────────┘
```

**Hook**: Git hook or CI trigger initiates validation
**Concern**: Identifies scope (e.g., root tree) + problem class (e.g., object-names)
**Contract**: Declarative rule set defining valid state

### Phase 2: Validation
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  VERIFIER   │───▶│  AUDITOR    │───▶│INVESTIGATOR │
│ Executes    │    │ SARIF Log   │    │Root Cause   │
│ Checks      │    │ Actionable  │    │Diagnosis    │
└─────────────┘    └─────────────┘    └─────────────┘
```

**Verifier**: Executes contract checks against scope
**Auditor**: Decides if violations are actionable, emits SARIF
**Investigator**: Traces root cause of failing concerns

### Phase 3: Orchestration
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ DISPATCHER  │───▶│   PLANNER   │───▶│   FIXERS    │
│ Route to    │    │ Fix Strategy│    │Stateless    │
│ Fixer Set   │    │ DAG of      │    │Repair Tools │
└─────────────┘    └─────────────┘    └─────────────┘
```

**Dispatcher**: Routes concern to correct fixer set
**Planner**: Builds fix strategy as DAG of fixers
**Fixers**: Stateless tools that execute repairs

### Phase 4: Iteration
```
┌─────────────┐
│   REPEAT    │◀─── Loop until passing state
│ Iterate     │
│ Until Pass  │
└─────────────┘
```

## 🎯 Strategic Foundation

### 1. Tree-Aware Caching Strategy
```
┌─────────────────────────────────────────────────────────────────┐
│                    CACHE KEY STRATEGY                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  cache_key = hash({                                            │
│    tree_sha,          // SHA-stable tree scope                 │
│    contract_id,       // e.g., object-names@1.0.0              │
│    contract_rev,      // Schema hash or version                │
│    fix_hash,          // Tool+config fingerprint               │
│  })                                                             │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Tree SHA    │    │ Contract ID │    │ Fix Hash    │         │
│  │ Stable Key  │    │ Version     │    │ Tool Config │         │
│  │ Survives    │    │ Pinned      │    │ Fingerprint │         │
│  │ Rebase      │    │ Schema      │    │ Cache Key   │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Volatility-Driven Decomposition
```
┌─────────────────────────────────────────────────────────────────┐
│                VOLATILITY ANALYSIS                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │tree_object_ │    │tree_split_  │    │crate_       │         │
│  │stability_   │───▶│planner.rs   │───▶│contract_    │         │
│  │auditor      │    │Suggested    │    │mapper.rs    │         │
│  │High Churn   │    │Splits       │    │Crate        │         │
│  │Detection    │    │Reduce       │    │Boundaries   │         │
│  └─────────────┘    │Conflicts    │    └─────────────┘         │
│                     └─────────────┘                            │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ High Churn  │    │ Suggested   │    │ Crate       │         │
│  │ Trees       │───▶│ Crate       │───▶│ Contract    │         │
│  │ Flagged     │    │ Splits      │    │ Mapping     │         │
│  │ For Split   │    │ Reduce      │    │ Enforced    │         │
│  └─────────────┘    │ Conflicts   │    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Performance Guardrails
```
┌─────────────────────────────────────────────────────────────────┐
│                    EXECUTION ORDER                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. STRUCTURE (Fast & Cheap)                                   │
│     ┌─────────────┐    ┌─────────────┐                         │
│     │ Object Names│    │ Tree        │                         │
│     │ Contract    │    │ Stability   │                         │
│     │ Root Tree   │    │ Analysis    │                         │
│     └─────────────┘    └─────────────┘                         │
│                                                                 │
│  2. SEMANTICS (Medium Cost)                                    │
│     ┌─────────────┐    ┌─────────────┐                         │
│     │ Schema      │    │ License     │                         │
│     │ Validation  │    │ Headers     │                         │
│     │ JSON Files  │    │ Check       │                         │
│     └─────────────┘    └─────────────┘                         │
│                                                                 │
│  3. HEAVY ANALYSIS (If Needed)                                 │
│     ┌─────────────┐    ┌─────────────┐                         │
│     │ Pack/Delta  │    │ SBOM        │                         │
│     │ Analysis    │    │ Generation  │                         │
│     │ Deep Audit  │    │ LFS Audit   │                         │
│     └─────────────┘    └─────────────┘                         │
└─────────────────────────────────────────────────────────────────┘
```

## 🔧 Implementation Components

### Core Tools
```
┌─────────────────────────────────────────────────────────────────┐
│                    IMPLEMENTATION STACK                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │tree_fix_    │    │crate_       │    │tree_object_ │         │
│  │cache.rs     │    │contract_    │    │stability_   │         │
│  │Tree-Aware   │    │mapper.rs    │    │auditor      │         │
│  │Caching      │    │Crate        │    │Volatility   │         │
│  │TTL + Stats  │    │Boundaries   │    │Detection    │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │tree_split_  │    │contract_    │    │validate_    │         │
│  │planner.rs   │    │validation_  │    │object_names │         │
│  │Suggested    │    │pipeline.rs  │    │_contract.rs │         │
│  │Splits       │    │Full Pipeline│    │Basic        │         │
│  │Reduce       │    │Caching      │    │Validation   │         │
│  │Conflicts    │    │SARIF        │    │Object Names │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Contract Variants
```
┌─────────────────────────────────────────────────────────────────┐
│                    CONTRACT VARIANTS                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Strict      │    │ Rust        │    │ Custom      │         │
│  │ Compliance  │    │ Workspace   │    │ Project     │         │
│  │ Clean       │    │ Friendly    │    │ Specific    │         │
│  │ Organized   │    │ Scripts     │    │ Rules       │         │
│  │ Structure   │    │ At Root     │    │ Tailored    │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 Integration Points

### GitHub Actions Matrix
```
┌─────────────────────────────────────────────────────────────────┐
│                    CI/CD INTEGRATION                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Detect      │    │ Validate    │    │ Generate    │         │
│  │ Scopes      │───▶│ Contracts   │───▶│ Report      │         │
│  │ Matrix      │    │ Parallel    │    │ SARIF       │         │
│  │ Strategy    │    │ Execution   │    │ PR Comments │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Cache       │    │ Artifact    │    │ GitHub      │         │
│  │ Persistence │    │ Sharing     │    │ Integration │         │
│  │ Between     │    │ Between     │    │ Annotations │         │
│  │ Jobs        │    │ Matrix      │    │ Code        │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Git Hooks
```
┌─────────────────────────────────────────────────────────────────┐
│                    GIT HOOK INTEGRATION                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Pre-Commit  │    │ Pre-Push    │    │ Pre-Receive │         │
│  │ Local       │    │ Branch      │    │ Server      │         │
│  │ Validation  │    │ Validation  │    │ Protection  │         │
│  │ Fast Check  │    │ Full        │    │ Block       │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

## 📊 Performance Characteristics

### Cache Hit Rates
```
┌─────────────────────────────────────────────────────────────────┐
│                    CACHE PERFORMANCE                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Stable      │    │ Active      │    │ Major       │         │
│  │ Trees       │    │ Development │    │ Refactoring │         │
│  │ 90%+ Hit    │    │ 60-80% Hit  │    │ 20-40% Hit  │         │
│  │ Rate        │    │ Rate        │    │ Rate        │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Execution Times
```
┌─────────────────────────────────────────────────────────────────┐
│                    EXECUTION TIMING                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Cache Hit   │    │ Fresh       │    │ Full        │         │
│  │ <5ms        │    │ Validation  │    │ Pipeline    │         │
│  │ Instant     │    │ 150-200ms   │    │ 500ms-2s    │         │
│  │ Return      │    │ Typical     │    │ Complex     │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Strategic Principles

### 1. Minimize Contract Recomputation
- Use SHA-stable crates and trees as caching units
- Contract validation keyed by tree_sha + contract_id + fix_hash

### 2. Tree-Aware Caching
- tree_fix_cache.rs memoizes results per tree scope
- TTL + tree-specific invalidation to avoid stale hits

### 3. Volatility-Driven Decomposition
- Use tree_object_stability_auditor to flag high-churn trees/blobs
- Feed into tree_split_planner.rs for suggested crate/scope splits

### 4. Avoid Git Submodules
- Keep everything in Cargo workspace crates for ecosystem benefit
- Modularity via crate boundaries, not nested repos

### 5. Map Contracts to Crate Boundaries
- crate_contract_mapper.rs enforces contracts within stable, isolated crate scopes
- Detect and warn about cross-crate dependency violations

### 6. Conflict + Churn Minimization
- Break up unstable trees to reduce merge conflicts and delta compression decay
- Ensure fix plan cache keys survive squash/rebase by scoping to tree/blobs, not commits

### 7. Merge/Rewrite Safety
- Contracts run on synthetic merge tree (PR ⊕ origin/main), so result matches post-merge state
- Tree-scoped keys survive commit rewrites unless content changes

### 8. Performance Guardrails
- Fail fast: cheap structural checks before heavy semantic checks
- Skip heavy analysis for volatile scopes unless gating is required

## 🔮 Future Extensions

### Planned Enhancements
```
┌─────────────────────────────────────────────────────────────────┐
│                    FUTURE ROADMAP                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Contract    │    │ Visual      │    │ Integration │         │
│  │ Packs       │    │ Reports     │    │ APIs        │         │
│  │ Predefined  │    │ Tree        │    │ REST        │         │
│  │ Rule Sets   │    │ Structure   │    │ Endpoints   │         │
│  │ Common      │    │ Diagrams    │    │ External    │         │
│  │ Projects    │    │ Interactive │    │ Validation  │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Custom      │    │ Auto-       │    │ IDE         │         │
│  │ Rules       │    │ Remediation │    │ Plugins     │         │
│  │ Repository  │    │ Automatic   │    │ Real-time   │         │
│  │ Specific    │    │ Fixes       │    │ Validation  │         │
│  │ Overrides   │    │ Safe        │    │ Editor      │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

This architecture provides a comprehensive, scalable foundation for contract validation that integrates seamlessly with modern development workflows while maintaining performance and reliability.
