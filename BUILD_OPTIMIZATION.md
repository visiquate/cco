# Build Optimization Guide

## Problem
Full rebuilds with `cargo clean` take 20+ minutes because all dependencies must recompile.

## Solution: Incremental Builds

### Quick Reference

```bash
# Fast incremental build (only rebuilds changed code)
cargo build --release

# Fast incremental install (uses cached dependencies)
cargo install --path . --force

# Check without building (fastest validation)
cargo check

# Development build (much faster, but less optimized)
cargo build
```

### When to Use Each Command

**Daily Development:**
```bash
# After editing code - fastest feedback (5-10s)
cargo check

# Build and test changes (10-30s depending on changes)
cargo build

# Install updated binary (30s-2min depending on changes)
cargo install --path . --force
```

**Release Builds:**
```bash
# Only when needed (after editing code, NOT from scratch)
cargo build --release

# Install release binary (reuses cached dependencies)
cargo install --path . --force
```

**NEVER RUN (unless absolutely necessary):**
```bash
# ❌ Wipes all dependency caches - forces 20+ min rebuild
cargo clean && cargo build --release
```

## Why Incremental Builds Work

Cargo tracks:
- Which source files changed
- Which dependencies need recompiling
- Git commit hash changes (via build.rs)

**Result**: Only your code recompiles, not 200+ dependencies!

## Build Times Comparison

| Command | First Run | After Code Edit | Why |
|---------|-----------|----------------|-----|
| `cargo clean && cargo build --release` | 20-30 min | 20-30 min | Rebuilds EVERYTHING |
| `cargo build --release` | 20-30 min | 30s-2min | Only rebuilds YOUR code |
| `cargo build` | 5-10 min | 10-30s | Dev mode, less optimization |
| `cargo check` | 3-5 min | 5-10s | Type-checking only |

## When cargo clean IS Needed

Only run `cargo clean` when:
1. **Dependency version changes** - After editing Cargo.toml dependencies
2. **Build.rs changes** - After modifying build scripts
3. **Corrupted cache** - Rare cases of unexplained build errors
4. **Switching branches with major changes** - If incremental build acts weird

**In 95% of cases, you DON'T need cargo clean!**

## Advanced: sccache for Even Faster Builds

Install sccache to cache compiled artifacts across projects:

```bash
# Install sccache
brew install sccache

# Configure cargo to use it
export RUSTC_WRAPPER=sccache

# Add to ~/.zshrc or ~/.bash_profile for permanent use
echo 'export RUSTC_WRAPPER=sccache' >> ~/.zshrc

# Check cache statistics
sccache --show-stats
```

**Benefits**:
- Shares compiled dependencies across projects
- Even faster rebuilds after `cargo clean`
- Persistent cache across sessions

## Workflow Examples

### Example 1: Fix a Bug
```bash
# Edit src/daemon/hooks/llm/model.rs
vim src/daemon/hooks/llm/model.rs

# Quick validation (5-10s)
cargo check

# Build and install (30s-1min)
cargo build --release
cargo install --path . --force

# Test it
cco --version
```

### Example 2: Add New Feature
```bash
# Edit multiple files
vim src/orchestration/server.rs
vim src/orchestration/context_injector.rs

# Quick check (10-15s)
cargo check

# Build incrementally (1-2min)
cargo build --release

# Install
cargo install --path . --force

# Verify
cco --version
```

### Example 3: Update Dependencies
```bash
# Edit Cargo.toml
vim Cargo.toml

# NOW clean is appropriate
cargo clean

# Full rebuild (20-30 min, but necessary)
cargo build --release
cargo install --path . --force
```

## Git Hash Updates

The git hash is embedded automatically via build.rs:
- No manual intervention needed
- Updates on every build (incremental or full)
- Hash reflects current `git rev-parse --short HEAD`

**You don't need cargo clean to update the git hash!**

## Troubleshooting

**Problem**: "Build seems to take too long after a small change"
**Solution**:
```bash
# Check what's recompiling
cargo build --release -vv | grep "Running"

# If many dependencies recompile, might need clean
cargo clean
cargo build --release
```

**Problem**: "Binary has wrong git hash"
**Solution**:
```bash
# Check current hash
git rev-parse --short HEAD

# Rebuild (incremental is fine)
cargo build --release
cargo install --path . --force

# Verify
cco --version
```

**Problem**: "Builds are consistently slow"
**Solution**:
```bash
# Install sccache for persistent caching
brew install sccache
export RUSTC_WRAPPER=sccache

# Rebuild once to populate cache
cargo build --release

# Future builds will be much faster
```

## Summary

✅ **DO**:
- Use `cargo build --release` for normal release builds
- Use `cargo check` for quick validation
- Use `cargo install --path . --force` for installing changes
- Let incremental compilation do its job

❌ **DON'T**:
- Run `cargo clean` before every build
- Rebuild dependencies unnecessarily
- Wait 20+ minutes when 30 seconds would suffice

**Remember**: Cargo is smart. Trust incremental compilation!
