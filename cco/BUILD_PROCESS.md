# CCO Build Process Documentation

## Overview

The CCO build process uses Rust's build script system (`build.rs`) to embed agent definitions, validate configurations, and set version information at compile time.

## Build Phases

### Phase 1: Initialization

When you run `cargo build` or `cargo build --release`:

1. Cargo reads `Cargo.toml`
2. Locates `build.rs` in project root
3. Invokes build script before compilation

### Phase 2: Build Script Execution (build.rs)

The `build.rs` script handles several tasks:

#### 2.1 File Watch Setup

```rust
println!("cargo:rerun-if-changed=../config/");
println!("cargo:rerun-if-changed=../config/orchestra-config.json");
```

**What this does:**
- Tells Cargo to rerun build script when files change
- Watches entire `../config/` directory
- Watches `orchestra-config.json` specifically
- Enables incremental builds during development

**Behavior:**
- If you modify `cco/config/agents/chief-architect.md`
- Next build will recompile with new content
- No manual cache clearing needed

#### 2.2 Version Information

```rust
// Get git commit hash
let git_hash = get_git_hash();
println!("cargo:rustc-env=GIT_HASH={}", git_hash);

// Get build timestamp
let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
println!("cargo:rustc-env=BUILD_DATE={}", build_date);

// Get or use default version
let version = env::var("CCO_VERSION")
    .unwrap_or_else(|_| "2025.11.2".to_string());
println!("cargo:rustc-env=CCO_VERSION={}", version);
```

**What this does:**
- Extracts current git commit: `git rev-parse --short HEAD`
- Records build timestamp: Current date/time
- Uses `CCO_VERSION` environment variable if set
- Falls back to default if environment variable missing

**Usage:**
```bash
# Use default version (2025.11.2)
cargo build --release

# Override version for release
CCO_VERSION=2025.11.3 cargo build --release

# Check version in built binary
./target/release/cco --version
```

**Runtime Access:**
```rust
// In source code - these become compile-time constants
let version = env!("CCO_VERSION");      // "2025.11.2"
let git_hash = env!("GIT_HASH");        // "a3f1b2c"
let build_date = env!("BUILD_DATE");    // "2025-11-15 14:30:45"
```

#### 2.3 Configuration Validation

```rust
fn validate_configs() {
    let config_paths = vec!["../config/orchestra-config.json"];

    for config_file in config_paths {
        let path = Path::new(config_file);
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Validate JSON structure
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                        eprintln!("Invalid JSON in {}: {}", config_file, e);
                        panic!("Config validation failed for {}", config_file);
                    }
                    println!("cargo:warning=Validated config: {}", config_file);
                }
                Err(e) => {
                    eprintln!("Failed to read {}: {}", config_file, e);
                    panic!("Config file read failed: {}", config_file);
                }
            }
        }
    }
}
```

**What this does:**
- Reads `orchestra-config.json`
- Parses as JSON to validate syntax
- Fails build if JSON is invalid
- Provides clear error messages

**Error Examples:**

```bash
# Missing trailing comma
cargo build
# Error: Invalid JSON in ../config/orchestra-config.json:
#   trailing comma at line 42 column 5

# Wrong file path
cargo build
# Error: Failed to read ../config/orchestra-config.json:
#   No such file or directory

# Both errors halt build process
```

#### 2.4 Compiler Configuration

```rust
// Enable position-independent code (for shared libraries)
println!("cargo:rustc-link-arg=-fPIC");
```

**What this does:**
- Sets `-fPIC` flag for compiler
- Enables position-independent executable
- Important for some deployment scenarios

### Phase 3: Rust Compilation

Cargo compiles Rust source with:

1. **Environment variables** set by build.rs
2. **Config files** validated
3. **Dependency compilation** (tokio, axum, serde, etc.)
4. **Binary generation** at `target/debug/cco` or `target/release/cco`

### Phase 4: Artifact Generation

```
Output locations:
Debug:   target/debug/cco
Release: target/release/cco
```

**Debug build:**
- `cargo build` (no release flag)
- Larger binary size
- Slower execution
- Includes debug symbols
- Use for development

**Release build:**
- `cargo build --release`
- Smaller binary (with debug symbols for diagnostics)
- Optimized performance
- Use for distribution

## Build Triggers and Incremental Compilation

### When Build Script Re-runs

The build script re-executes when:

1. **Config files change**
   - Any file in `../config/` directory
   - `../config/orchestra-config.json` specifically

2. **Source code changes**
   - Any `.rs` file in `src/`

3. **Cargo.toml changes**
   - Dependencies, features, metadata

4. **Manual rebuild**
   - `cargo clean` then `cargo build`

### When Build Script Doesn't Re-run

The build script skips if:

- No watched files changed
- No source code changed
- Binary already up-to-date

```bash
# First build - full compilation
cargo build --release
# Compiling serde v1.0...
# Compiling cco v0.0.0...
# Finished...

# No changes - skips build script
cargo build --release
# Finished `release` profile [optimized] target(s) in 0.01s
```

## Development Workflow

### Adding a New Agent

1. **Create agent definition** in `cco/config/agents/new-agent.md`:
   ```markdown
   ---
   name: new-agent
   model: sonnet
   description: A new agent for custom tasks
   tools: Read, Write, Edit, Bash
   ---

   ## Description
   This agent handles...
   ```

2. **Rebuild**:
   ```bash
   cargo build --release
   ```

3. **Test**:
   ```bash
   ./target/release/cco run --port 3000
   curl http://localhost:3000/api/agents/new-agent
   ```

### Modifying an Existing Agent

1. **Edit agent file** (e.g., `chief-architect.md`)

2. **Rebuild**:
   ```bash
   cargo build --release
   ```

3. **Verify changes**:
   ```bash
   ./target/release/cco run --port 3000
   curl http://localhost:3000/api/agents/chief-architect
   ```

### Setting Version for Release

```bash
# Release version 2025.11.3
CCO_VERSION=2025.11.3 cargo build --release

# Check version
./target/release/cco --version
# cco 2025.11.3

# Or create release tag
git tag v2025.11.3
git push origin v2025.11.3
```

## Build Configuration File Structure

### Cargo.toml Build Dependencies

```toml
[build-dependencies]
include_dir = "0.7"           # Directory inclusion utilities
chrono = { version = "0.4", features = ["serde"] }  # Timestamp
serde_json = "1.0"            # JSON parsing
```

These are only needed during build, not in final binary.

### Build Script Location

```
cco/
├── build.rs                    # Build script (top level)
├── Cargo.toml                  # Package definition
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── agents_config.rs
│   └── ...
└── config/
    ├── agents/                 # Agent definitions
    │   ├── chief-architect.md
    │   ├── python-specialist.md
    │   └── ... (119 more)
    └── orchestra-config.json   # Orchestration config
```

## Troubleshooting Build Issues

### Issue: Build Fails with "Config file not found"

**Error:**
```
Warning: Config file not found at "/path/to/config/orchestra-config.json"
```

**Solution:**
1. Check file exists: `ls cco/config/orchestra-config.json`
2. Verify path is relative from `build.rs`
3. Ensure not in `.gitignore`

### Issue: Build Fails with "Invalid JSON"

**Error:**
```
Invalid JSON in ../config/orchestra-config.json:
trailing comma at line 42 column 5
```

**Solution:**
1. Check for syntax errors: `jq . cco/config/orchestra-config.json`
2. Look for trailing commas, missing quotes
3. Use online JSON validator for detailed errors
4. Fix and rebuild: `cargo build --release`

### Issue: Version Not Updated

**Problem:**
```bash
./target/release/cco --version
# Still shows old version
```

**Solution:**
1. Set environment variable explicitly:
   ```bash
   CCO_VERSION=2025.11.3 cargo build --release
   ```

2. Or clean and rebuild:
   ```bash
   cargo clean
   cargo build --release
   ```

### Issue: Changes Not Reflected After Build

**Problem:**
Agent file was modified but binary shows old definition.

**Solution:**
1. Force rebuild:
   ```bash
   cargo clean
   cargo build --release
   ```

2. Or touch config file to trigger recompile:
   ```bash
   touch cco/config/
   cargo build --release
   ```

## Environment Variables

### CCO_VERSION

Set version for release builds:

```bash
CCO_VERSION=2025.11.3 cargo build --release
```

**Default:** `2025.11.2` (from build.rs)

**Format:** `YYYY.MM.N` (date-based versioning)

### RUST_LOG

Enable debug logging during build:

```bash
RUST_LOG=debug cargo build --release
```

**Options:**
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Info and above (default)
- `debug` - All messages
- `trace` - Very verbose

## Build Performance

### Build Times (Approximate)

**First build:**
- Clean build from scratch: 2-5 minutes
- Includes dependency compilation

**Incremental build:**
- After file changes: 10-30 seconds
- Recompiles only changed modules

**Release build optimization:**
- `cargo build --release` with optimizations: 5-10 minutes
- Larger binaries, but better performance

### Parallel Compilation

Cargo uses multiple CPU cores by default:

```bash
# Use specific number of cores
cargo build --release -j 4

# Use all cores
cargo build --release -j $(nproc)
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build CCO

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: cargo build --release --manifest-path cco/Cargo.toml
        env:
          CCO_VERSION: ${{ github.ref_name }}

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: cco-linux-x86_64
          path: cco/target/release/cco
```

## See Also

- [EMBEDDING_ARCHITECTURE.md](EMBEDDING_ARCHITECTURE.md) - System architecture overview
- [EMBEDDING_IMPLEMENTATION.md](EMBEDDING_IMPLEMENTATION.md) - Code implementation details
- [DEPLOYMENT_EMBEDDING.md](DEPLOYMENT_EMBEDDING.md) - Deployment guide
- [EMBEDDING_TROUBLESHOOTING.md](EMBEDDING_TROUBLESHOOTING.md) - Troubleshooting guide
