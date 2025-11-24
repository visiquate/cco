# CCO Build System - Quick Start

## One-Minute Setup

```bash
cd /Users/brent/git/cc-orchestra/cco

# Install to system
./install.sh

# Or use make
make install
```

## Daily Commands

```bash
# Format code
make fmt

# Run linter
make clippy

# Run tests
make test

# Build debug
make build

# Build release
make release

# Check everything
make check-all
```

## Development Workflow

```bash
# 1. Make changes
# 2. Format and lint
make fmt clippy

# 3. Run tests
make test

# 4. Build and verify
make build

# 5. Full quality check before commit
make check-all
```

## Installation Options

### System-wide (requires sudo if needed)
```bash
make install
# Binary at: /usr/local/bin/cco
```

### User directory (no sudo)
```bash
make INSTALL_DIR=~/.local/bin install
# Add to PATH: export PATH="$HOME/.local/bin:$PATH"
```

### Custom directory
```bash
INSTALL_DIR=/opt/bin ./install.sh
```

## Build Artifacts

### Debug binary
```bash
make build
# Output: target/debug/cco
```

### Release binary
```bash
make release
# Output: target/release/cco (optimized, stripped)
```

## Continuous Integration

The GitHub Actions workflow runs automatically on:
- Push to `main` or `develop`
- Pull requests to `main`

Checks:
- Code formatting
- Clippy linting
- All tests (unit + integration)
- Multi-platform builds (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows)
- Dependency security scan
- SARIF code analysis

## Help

```bash
make help
```

## Environment Variables

```bash
# Custom install directory
INSTALL_DIR=/opt/bin make install

# Additional cargo flags
CARGO_FLAGS="--verbose" make build

# Both together
INSTALL_DIR=/opt/bin CARGO_FLAGS="--verbose" make release
```

## Troubleshooting

### Build fails
```bash
make clean
make build
```

### Tests fail
```bash
make test-verbose  # See detailed output
```

### Install denied
```bash
sudo make install
# Or use ~/.local/bin
make INSTALL_DIR=~/.local/bin install
```

## What Gets Built

### build.rs (72 lines)
- Embeds git hash and build date
- Validates config files
- Sets environment variables

### Makefile (145 lines)
- 20+ build targets
- Multi-OS support
- Parallel test execution

### install.sh (222 lines)
- Prerequisite checking
- Smart error handling
- Colorized output

### GitHub Actions (.github/workflows/build.yml)
- 6 parallel jobs
- 5 platform builds
- Security scanning
- SARIF analysis

### .gitignore (48 lines)
- Standard Rust patterns
- CCO-specific excludes
- Secrets protection

---

**Total: 653 lines of production-grade build infrastructure**

For detailed documentation, see `BUILD_SYSTEM.md`
