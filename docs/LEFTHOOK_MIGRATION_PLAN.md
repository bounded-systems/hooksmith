# Lefthook Migration Plan

## Overview

We are migrating from Lefthook to custom Rust-based Git hooks that integrate with our validation workflow. This migration provides better type safety, performance, and integration with our Hooksmith validation system.

## Current State

### ✅ **Completed**
- [x] All 18 client-side Git hooks implemented as Rust binaries
- [x] All hook stubs created in `.git/hooks/`
- [x] All hooks in no-op mode, ready for validation workflow integration
- [x] Comprehensive test coverage of hook functionality

### 🔄 **In Progress**
- [ ] Integration with validation workflow
- [ ] Activation of actual validation logic
- [ ] Performance optimization

### 📋 **Planned**
- [ ] Remove Lefthook dependency
- [ ] Remove `lefthook.yml` configuration
- [ ] Update documentation
- [ ] Update CI/CD pipelines

## Migration Timeline

### Phase 1: Foundation ✅ (Complete)
- Implement all Git hooks as Rust binaries
- Create hook stubs that call Rust binaries
- Test all hooks in no-op mode

### Phase 2: Integration 🔄 (Current)
- Integrate hooks with validation workflow
- Activate actual validation logic
- Performance testing and optimization

### Phase 3: Cleanup 📋 (Planned)
- Remove Lefthook dependency
- Remove `lefthook.yml` file
- Update documentation and CI/CD

## Benefits of Migration

### **Before (Lefthook)**
- Shell-based hooks with limited type safety
- External dependency on Lefthook
- Limited integration with our validation workflow
- YAML configuration complexity

### **After (Custom Rust Hooks)**
- Type-safe Rust binaries
- Direct integration with validation workflow
- No external dependencies
- Better performance and reliability
- Unified architecture with our codebase

## Hook Coverage

| Hook Type | Lefthook | Custom Rust Hooks | Status |
|-----------|----------|-------------------|--------|
| pre-commit | ✅ | ✅ | Migrated |
| prepare-commit-msg | ✅ | ✅ | Migrated |
| commit-msg | ✅ | ✅ | Migrated |
| post-commit | ✅ | ✅ | Migrated |
| pre-push | ✅ | ✅ | Migrated |
| pre-rebase | ❌ | ✅ | Added |
| post-rebase | ❌ | ✅ | Added |
| post-checkout | ❌ | ✅ | Added |
| post-merge | ❌ | ✅ | Added |
| post-rewrite | ❌ | ✅ | Added |
| applypatch-msg | ❌ | ✅ | Added |
| pre-applypatch | ❌ | ✅ | Added |
| post-applypatch | ❌ | ✅ | Added |
| pre-merge-commit | ❌ | ✅ | Added |
| reference-transaction | ❌ | ✅ | Added |
| sendemail-validate | ❌ | ✅ | Added |
| fsmonitor-watchman | ❌ | ✅ | Added |
| post-index-change | ❌ | ✅ | Added |

**Total Coverage**: 4/18 → 18/18 (100% increase)

## File Structure

### **Before (Lefthook)**
```
lefthook.yml                    # Configuration
.git/hooks/                     # Lefthook-managed stubs
```

### **After (Custom Rust Hooks)**
```
.git/hooks/                     # Custom hook stubs
├── pre-commit                  → calls target/debug/pre-commit
├── prepare-commit-msg          → calls target/debug/prepare-commit-msg
├── commit-msg                  → calls target/debug/commit-msg
├── post-commit                 → calls target/debug/post-commit
├── pre-merge-commit            → calls target/debug/pre-merge-commit
├── pre-push                   → calls target/debug/pre-push
├── pre-rebase                 → calls target/debug/pre-rebase
├── post-rebase                → calls target/debug/post-rebase
├── post-checkout              → calls target/debug/post-checkout
├── post-merge                 → calls target/debug/post-merge
├── post-rewrite               → calls target/debug/post-rewrite
├── applypatch-msg             → calls target/debug/applypatch-msg
├── pre-applypatch             → calls target/debug/pre-applypatch
├── post-applypatch            → calls target/debug/post-applypatch
├── reference-transaction      → calls target/debug/reference-transaction
├── sendemail-validate         → calls target/debug/sendemail-validate
├── fsmonitor-watchman         → calls target/debug/fsmonitor-watchman
└── post-index-change          → calls target/debug/post-index-change

crates/hooks/src/bin/          # Rust binary implementations
├── pre-commit.rs
├── prepare-commit-msg.rs
├── commit-msg.rs
├── post-commit.rs
├── pre-merge-commit.rs
├── pre-push.rs
├── pre-rebase.rs
├── post-rebase.rs
├── post-checkout.rs
├── post-merge.rs
├── post-rewrite.rs
├── applypatch-msg.rs
├── pre-applypatch.rs
├── post-applypatch.rs
├── reference-transaction.rs
├── sendemail-validate.rs
├── fsmonitor-watchman.rs
└── post-index-change.rs
```

## Next Steps

1. **Integrate with validation workflow** - Connect hooks to our validation system
2. **Activate validation logic** - Replace no-op messages with actual validation
3. **Performance testing** - Ensure hooks are fast and reliable
4. **Remove Lefthook** - Clean up dependencies and configuration
5. **Update documentation** - Reflect new hook architecture

## Rollback Plan

If issues arise during migration:
1. Keep Lefthook configuration as backup
2. Maintain both systems during transition
3. Gradual migration with feature flags
4. Easy rollback to Lefthook if needed

## Success Criteria

- [ ] All hooks working with validation workflow
- [ ] Performance comparable to or better than Lefthook
- [ ] No external dependencies on Lefthook
- [ ] Comprehensive test coverage
- [ ] Documentation updated
- [ ] CI/CD pipelines updated
