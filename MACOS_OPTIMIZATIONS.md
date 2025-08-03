# macOS Optimizations for Hooksmith

## 🍎 Apple Silicon Best Practices (August 2025)

This document outlines the current best practices for optimizing Rust builds on macOS, specifically for Apple Silicon (M1/M2/M3) systems.

## ✅ Current Best Practices

### 1. **LLD Linker (Recommended)**

**Why LLD?**
- ✅ **Actively maintained** by LLVM team
- ✅ **Native Apple Silicon support** 
- ✅ **Frequently updated** alongside Rust's LLVM backend
- ✅ **Default for nightly builds** on macOS
- ✅ **No compatibility issues** with Xcode 16.4+

**Configuration**:
```toml
# .cargo/config.toml
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "link-arg=-fuse-ld=lld",
]

[target.x86_64-apple-darwin]
rustflags = [
    "-C", "link-arg=-fuse-ld=lld",
]
```

### 2. **macOS-Specific Profile Optimizations**

**Debug Builds**:
```toml
# Cargo.toml
[profile.dev]
split-debuginfo = "unpacked"
```

**Benefits**:
- **70% reduction** in debug build compile time
- Avoids containerized debug info overhead
- Faster iterative development

### 3. **System-Level Optimizations**

**Enable Developer Mode** (Safe and Granular):
```bash
spctl developer-mode enable-terminal
```

**Add Terminal to Developer Tools**:
1. Go to **System Settings → Privacy & Security → Developer Tools**
2. Find 'Terminal' in the list and check the box
3. Restart your terminal application
4. This enables faster execution of development tools without global Gatekeeper changes

**Security Benefits**:
- ✅ Only affects Terminal and its child processes
- ✅ GUI apps launched from Finder remain protected
- ✅ System-wide Gatekeeper protection stays active
- ✅ Can be completely reverted with `spctl developer-mode disable-terminal`

## ❌ What to Avoid

### **ZLD Linker (Deprecated)**
- ❌ **Archived in February 2023**
- ❌ **No longer maintained**
- ❌ **Fails on Xcode 16.4+**
- ❌ **Compatibility issues** with modern toolchains

### **Mold Linker**
- ❌ **Linux-first** - macOS support is experimental
- ❌ **Incompatible** with rustc's `cc` driver on macOS
- ❌ **Not recommended** for production use

## 🚀 Performance Benefits

### **Build Performance**
- **30-50% faster link times** with LLD
- **70% reduction** in debug build compile time
- **Faster iterative builds** with split-debuginfo optimization

### **Development Experience**
- **Reduced Gatekeeper overhead** with developer mode
- **Faster binary execution** for local development
- **Better IDE integration** with optimized toolchain

## 🔧 Implementation in Hooksmith

### **Automatic Configuration**

The Hooksmith project automatically configures these optimizations:

1. **LLD Linker**: Configured in `.cargo/config.toml`
2. **Profile Optimizations**: Set in `Cargo.toml`
3. **Setup Script**: `./scripts/setup-default.sh` enables developer mode

### **Automatic Setup**

Use the dedicated macOS optimization script:

```bash
./scripts/macos-optimize.sh
```

This script provides:
- ✅ Safety checks and validation
- ✅ Comprehensive security information
- ✅ Step-by-step setup guidance
- ✅ Performance monitoring instructions

### **Manual Setup**

If you need to configure manually:

```bash
# 1. Enable developer mode (safe and granular)
spctl developer-mode enable-terminal

# 2. Add terminal to Developer Tools
# (System Settings → Privacy & Security → Developer Tools)

# 3. Verify configuration
cargo build --verbose
# Should show: "Running `rustc ... -C link-arg=-fuse-ld=lld ...`"
```

## 📊 Performance Monitoring

### **Check Linker Usage**
```bash
# Verify LLD is being used
cargo build --verbose 2>&1 | grep "link-arg=-fuse-ld=lld"
```

### **Monitor Build Times**
```bash
# Generate timing data
cargo build --timings

# View in browser
open target/cargo-timings.html
```

### **Check Debug Info**
```bash
# Verify split-debuginfo is working
cargo build --profile dev
# Should be faster than without the optimization
```

## 🔄 Migration from ZLD

If you were previously using ZLD:

1. **Remove ZLD**:
   ```bash
   brew uninstall zld
   ```

2. **Update Configuration**:
   - LLD is already configured in Hooksmith
   - No additional setup needed

3. **Verify Performance**:
   ```bash
   make stats
   # Should show improved build times
   ```

## 🎯 Success Metrics

You'll know the optimizations are working when:

- ✅ **Build times improve** by 30-50%
- ✅ **Debug builds are 70% faster**
- ✅ **No Xcode compatibility issues**
- ✅ **LLD linker is being used** (visible in verbose output)
- ✅ **Developer mode is enabled** (faster binary execution)

## 📚 References

- **LLD Documentation**: [LLVM Linker](https://lld.llvm.org/)
- **Rust Performance**: [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- **Apple Developer**: [Developer Tools](https://developer.apple.com/)

## 🚨 Troubleshooting

### **LLD Not Being Used**
```bash
# Check if LLD is available
which lld

# Verify configuration
cargo build --verbose 2>&1 | grep "link-arg"
```

### **Developer Mode Issues**
```bash
# Check developer mode status
spctl --status

# Re-enable if needed
sudo spctl developer-mode enable-terminal
```

### **Performance Issues**
```bash
# Check build statistics
make stats

# Monitor cache usage
sccache --show-stats
```

---

**The Hooksmith project is configured with the latest macOS optimization best practices for maximum performance on Apple Silicon systems.** 
