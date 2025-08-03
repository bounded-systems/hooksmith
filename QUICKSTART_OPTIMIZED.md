# Hooksmith Quick Start - Optimized Workflow

## 🚀 Quick Start (Recommended)

This guide shows you how to get started with Hooksmith using the **optimized build workflow** for maximum performance and developer productivity.

### 1. Clone and Setup

```bash
git clone https://github.com/your-username/hooksmith.git
cd hooksmith

# Complete optimized setup (recommended)
./scripts/setup-default.sh
```

This single command will:
- ✅ Install all optimization tools (sccache, cargo-hakari, cargo-nextest)
- ✅ Configure environment variables for optimized builds
- ✅ Set up shell profile integration
- ✅ Create VS Code settings for optimization
- ✅ Add git hooks for optimization checks
- ✅ Create a Makefile with optimized tasks
- ✅ Set up direnv integration

### 2. Start Developing

```bash
# Fast development cycle with all optimizations
make dev

# Or use individual optimized commands
make build    # Fast development build
make test     # Parallel test execution
make check    # Fast checking
make dev      # Complete development cycle
```

### 3. Available Optimized Commands

#### 🚀 Fast Development
```bash
cargo dev-fast      # Fast development builds
cargo test-parallel # Parallel test execution
cargo check-fast    # Fast checking
./scripts/dev-cycle.sh  # Complete development workflow
```

#### 📊 Performance Monitoring
```bash
make stats          # Show build performance statistics
./scripts/build-stats.sh  # Detailed performance analysis
```

#### 🛠️ Project Tasks
```bash
make help           # Show all available commands
make clean          # Clean all build artifacts
make docs           # Generate documentation
make validate       # Validate project configuration
make ci             # Run CI build locally
```

## 📈 Performance Benefits

With the optimized workflow, you'll experience:

- **30-70% faster rebuilds** with sccache build caching
- **2-4x faster tests** with cargo-nextest parallel execution
- **Up to 50% faster builds** with cargo-hakari workspace optimization
- **20-30% faster linking** with LLD/zld/mold linkers
- **Up to 50% faster compilation** with parallel compilation frontend (nightly)

## 🔧 IDE Integration

### VS Code
The setup automatically configures VS Code with:
- Rust Analyzer optimization settings
- sccache integration for faster builds
- Optimized cargo commands

### Other IDEs
The optimization environment is automatically loaded when you enter the project directory (via direnv or shell profile integration).

## 📋 Development Workflow

### Daily Development
```bash
# 1. Start development cycle
make dev

# 2. Make changes to code

# 3. Fast check
make check

# 4. Run tests
make test

# 5. Monitor performance
make stats
```

### Before Committing
```bash
# Run full validation
make validate

# Check performance
make stats

# Commit with optimization checks
git commit -m "Your commit message"
```

## 🚨 Troubleshooting

### Optimization Not Working?
```bash
# Re-run setup
./scripts/setup-default.sh

# Check optimization status
./scripts/build-stats.sh

# Verify tools are installed
cargo dev-fast --help
cargo test-parallel --help
```

### Performance Issues?
```bash
# Check cache status
sccache --show-stats

# Monitor build times
cargo build --timings

# Analyze workspace
cargo tree --all-features --depth 1
```

### Missing Tools?
```bash
# Install optimization tools
./scripts/optimize-build.sh

# Set up environment
source scripts/setup-env.sh
```

## 📚 Next Steps

1. **Explore the project structure** - Check out the components and WASM integration
2. **Read the full documentation** - See `README.md` for comprehensive details
3. **Try the CLI** - Run `hooksmith --help` to see available commands
4. **Monitor performance** - Use `make stats` to track optimization effectiveness
5. **Contribute** - The optimized workflow makes development faster and more enjoyable

## 🎯 Success Metrics

You'll know the optimized workflow is working when:
- ✅ `cargo dev-fast` works and is faster than `cargo run`
- ✅ `cargo test-parallel` runs tests significantly faster
- ✅ `make stats` shows sccache cache hits
- ✅ Build times improve over time as cache warms up
- ✅ VS Code/IDE builds are noticeably faster

## 💡 Pro Tips

- **Use `make dev`** for the complete development cycle
- **Monitor cache hit rates** with `make stats`
- **Keep tools updated** - optimization tools improve regularly
- **Share cache** - Consider setting up a shared sccache server for teams
- **Use nightly** - Enable nightly toolchain for parallel compilation frontend

## 🍎 macOS-Specific Optimizations

For optimal performance on Apple Silicon:

1. **Run the dedicated optimization script**:
   ```bash
   ./scripts/macos-optimize.sh
   ```

2. **Complete the manual setup**:
   - Open System Settings → Privacy & Security → Developer Tools
   - Find 'Terminal' and check the box
   - Restart your terminal application

3. **Benefits**:
   - 70% reduction in debug build compile time
   - Faster iterative binary execution
   - Reduced Gatekeeper overhead (safe and granular)
   - LLD linker configured for optimal Apple Silicon performance

**Security Note**: This only affects Terminal and its child processes. GUI apps remain protected by Gatekeeper.

---

**Ready to start?** Run `./scripts/setup-default.sh` and experience the fastest Rust development workflow! 
