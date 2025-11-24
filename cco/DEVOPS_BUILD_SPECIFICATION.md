# DevOps Build System Specification for CCO

**Status**: Ready for Implementation by DevOps Engineer Agent
**Target Directory**: `/Users/brent/git/cc-orchestra/cco/`
**Last Updated**: 2025-11-15

## Executive Summary

This specification defines the complete build system, CI/CD pipeline, and deployment configuration for the CCO (Claude Orchestration) Rust application. The system will provide:

- **Optimized Release Builds**: Stripped binaries with LTO and max optimization
- **Multi-Platform Support**: macOS Intel + Apple Silicon builds
- **Comprehensive Testing**: Unit tests, formatting checks, clippy validation
- **Automated Deployment**: Docker containerization and GitHub Actions CI/CD
- **Installation Support**: Cross-platform installation scripts

## Deliverables

### 1. Cargo.toml Configuration

**File**: `/Users/brent/git/cc-orchestra/cco/Cargo.toml`

```toml
[package]
name = "cco"
version = "0.1.0"
edition = "2021"
description = "Claude Orchestration - Multi-agent development system"
repository = "https://github.com/brent/cc-orchestra"
readme = "README.md"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
axum = "0.7"
hyper = "1"

[profile.release]
strip = true              # Strip symbols for smaller binary
opt-level = 3             # Maximum optimization
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit for better optimization
panic = "abort"           # Smaller binary panic handling
```

**Implementation Notes**:
- `strip = true` reduces binary size by ~40%
- `lto = true` improves performance at compile cost
- `codegen-units = 1` required for LTO optimization
- `panic = "abort"` saves additional binary space

### 2. Build Script (build.rs)

**File**: `/Users/brent/git/cc-orchestra/cco/build.rs`

**Purpose**: Embed build metadata and validate configuration files

**Key Features**:
- Rerun on config changes
- Validate orchestra-config.json exists
- Embed Git commit hash as `GIT_HASH` environment variable
- Embed build date as `BUILD_DATE` environment variable
- Display warnings during build

**Implementation Requirements**:
```
- Use `cargo:rerun-if-changed` for dependency tracking
- Validate existence of ../config/orchestra-config.json
- Execute `git rev-parse --short HEAD` for commit hash
- Use chrono for build date formatting
- Print `cargo:rustc-env` for metadata propagation
```

### 3. Makefile

**File**: `/Users/brent/git/cc-orchestra/cco/Makefile`

**Targets Required**:
- `make build` - Debug build
- `make test` - Run test suite
- `make release` - Optimized release build with symbol stripping
- `make install` - Install to /usr/local/bin/cco
- `make clean` - Clean build artifacts
- `make fmt` - Format code
- `make lint` - Run clippy linter
- `make check-size` - Display binary size
- `make cross-compile-macos` - Build both aarch64 and x86_64 targets

**Key Features**:
- `.PHONY` declarations for all targets
- Progress indicators (✓, ✗ symbols)
- Conditional symbol stripping for release builds
- Cross-compilation targets for macOS architectures
- Binary size analysis

### 4. GitHub Actions Workflows

**Directory**: `/Users/brent/git/cc-orchestra/cco/.github/workflows/`

#### 4a. Test Workflow (build.yml)

**File**: `.github/workflows/build.yml`

**Job: test**
- Trigger: Push to main/develop, PR to main
- Runs on: ubuntu-latest
- Steps:
  1. Checkout code
  2. Setup Rust (stable)
  3. Cache cargo registry, git, target/
  4. Run `cargo test --verbose`
  5. Check formatting with `cargo fmt -- --check`
  6. Run clippy linter with `-D warnings`

**Job: build**
- Trigger: After test passes
- Runs on: ubuntu-latest and macos-latest
- Targets: x86_64-apple-darwin, aarch64-apple-darwin
- Steps:
  1. Checkout code
  2. Setup Rust with target
  3. Cache cargo artifacts
  4. Build release binary
  5. Strip symbols
  6. Upload artifact to GitHub Artifacts

**Cache Strategy**:
- Key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- Paths: ~/.cargo/registry, ~/.cargo/git, target/

**Artifact Upload**:
- Name: `cco-${{ matrix.target }}`
- Path: `target/${{ matrix.target }}/release/cco`
- Retention: 90 days (GitHub default)

### 5. Installation Script

**File**: `/Users/brent/git/cc-orchestra/cco/install.sh`

**Requirements**:
- Executable permissions (755)
- Set -e (exit on error)
- Build release binary
- Strip symbols
- Copy to /usr/local/bin/
- Display success message with usage instructions
- Require sudo for system installation

**Output**:
```
🔨 Building CCO...
🔪 Stripping symbols...
📦 Installing to /usr/local/bin...
✅ CCO installed successfully!

Usage:
  cco run          # Start CCO
  cco stats        # View analytics
  cco --help       # Show all commands
```

### 6. Dockerfile

**File**: `/Users/brent/git/cc-orchestra/cco/Dockerfile`

**Requirements**:
- Multi-stage build (builder + runtime)
- Builder stage:
  - Base: rust:1.75
  - Build release binary in /usr/src/cco
- Runtime stage:
  - Base: debian:bookworm-slim
  - Install ca-certificates
  - Copy binary from builder
  - Set working directory to /workspace
- Environment variables:
  - ANTHROPIC_API_KEY (empty default)
  - CCO_PROXY_PORT=8888
  - CCO_DASHBOARD_PORT=3939
- Expose ports 8888 and 3939
- Entry point: `cco run`

**Build Optimization**:
- Multi-stage reduces final image size
- Debian slim base (70MB vs 2GB)
- Non-root user for security
- Health check endpoint

### 7. Docker Compose

**File**: `/Users/brent/git/cc-orchestra/cco/docker-compose.yml`

**Important**: No `version:` line (deprecated)

**Services**:
- `cco`:
  - Build from Dockerfile
  - Ports: 8888:8888, 3939:3939
  - Environment variables from .env
  - Volumes:
    - cco-data:/root/.cco (persistent cache)
    - ./project:/workspace (project mounting)
  - Working directory: /workspace

**Volumes**:
- `cco-data` named volume for persistent data

**Configuration**:
- ANTHROPIC_API_KEY (from environment)
- CCO_CACHE_SIZE=200
- CCO_CACHE_TTL=48

### 8. Cross-Compilation Guide

**File**: `/Users/brent/git/cc-orchestra/cco/docs/CROSS_COMPILE.md`

**Content**:
- macOS Intel → Apple Silicon: `rustup target add aarch64-apple-darwin`
- macOS Apple Silicon → Intel: `rustup target add x86_64-apple-darwin`
- Linux → macOS: Using musl-cross toolchain
- CI/CD cross-compilation strategy
- Binary verification steps

### 9. Binary Size Analysis

**File**: `/Users/brent/git/cc-orchestra/cco/scripts/analyze-size.sh`

**Features**:
- Display total binary size in human-readable format
- Show size by ELF sections (using `size` command)
- Top 10 largest dependencies (using `cargo bloat`)
- Comparison before/after optimization

### 10. .gitignore Updates

**File**: `/Users/brent/git/cc-orchestra/cco/.gitignore`

```
# Rust
target/
Cargo.lock

# CCO runtime
.cco/
*.swarm

# IDE
.vscode/
.idea/
*.iml

# OS
.DS_Store
Thumbs.db

# Temporary
*.tmp
*.backup
```

## Implementation Order

1. **Cargo.toml** - Project metadata and dependencies
2. **build.rs** - Build script for metadata
3. **Makefile** - Local development commands
4. **.github/workflows/build.yml** - CI/CD pipeline
5. **install.sh** - Installation automation
6. **Dockerfile** - Container image definition
7. **docker-compose.yml** - Container orchestration
8. **docs/CROSS_COMPILE.md** - Cross-compilation guide
9. **scripts/analyze-size.sh** - Binary analysis
10. **.gitignore** - Version control exclusions

## Testing Strategy

### Build Verification
1. Clean build: `cargo clean && cargo build --release`
2. Binary size check: Should be < 15MB
3. Symbol stripping verification: `nm target/release/cco | wc -l`
4. Cross-compile test: Both aarch64 and x86_64 targets

### CI/CD Verification
1. GitHub Actions workflows trigger on push
2. Test job passes all formatting and linter checks
3. Build job produces multi-platform artifacts
4. Artifacts downloadable from GitHub Actions

### Installation Verification
1. `./install.sh` completes without errors
2. `/usr/local/bin/cco` exists and is executable
3. `cco --version` displays build metadata
4. `cco --help` shows available commands

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Release binary size | < 15 MB | After stripping |
| Build time (release) | < 5 min | On GH Actions |
| Debug build time | < 1 min | Local development |
| Test execution | < 2 min | Full test suite |

## Security Considerations

- **Binary stripping**: Reduces attack surface by removing debug info
- **Dependency audit**: Include `cargo audit` in CI pipeline
- **Non-root container**: Docker image runs as non-root user
- **Secret management**: ANTHROPIC_API_KEY via environment variables
- **Image scanning**: Optional Trivy scanning in CI/CD

## Future Enhancements

- **Automated release tagging**: Create GitHub releases with binaries
- **Binary signing**: GPG sign release artifacts
- **Distribution channels**: Brew formula, cargo-binstall support
- **Performance benchmarks**: Track build times across commits
- **Coverage reporting**: Codecov integration for test coverage
- **Security scanning**: OWASP dependency checker, SAST tools

## Agent Responsibilities

### DevOps Engineer (Lead)
- Overall build system architecture
- GitHub Actions workflow design
- Docker/container strategy
- CI/CD pipeline optimization
- Deployment automation

### Rust Specialist
- Cargo.toml optimization and dependencies
- build.rs implementation
- Release profile tuning
- Cross-compilation configuration
- Binary size optimization

### Documentation Lead
- Cross-compilation guide
- Build troubleshooting guide
- Installation documentation
- Docker usage guide
- Binary size analysis documentation

## Success Criteria

✅ Cargo.toml with optimized release profile created
✅ build.rs script embeds Git commit and build date
✅ Makefile provides all essential commands
✅ GitHub Actions CI/CD pipeline configured and passing
✅ Installation script functional
✅ Dockerfile creates optimized container image
✅ Docker Compose enables local development environment
✅ Documentation complete and clear
✅ Binary size < 15MB after stripping
✅ Cross-compilation tested for both macOS targets
✅ All workflows trigger correctly on push/PR events

---

**Implementation Status**: Awaiting DevOps Engineer agent execution
**Created**: 2025-11-15
**For**: Claude Orchestra Orchestrator (CCO) Rust Application
