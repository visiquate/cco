# Building CCO

## Quick Build

```bash
cargo build --release
```

The binary will be at: `target/release/cco`

## Custom Version Build

To build with a specific version number:

```bash
# Set the version before building
CCO_VERSION=202511-2 cargo build --release

# Verify the version
./target/release/cco --version
```

## Version Format

CCO uses date-based versioning: `YYYYMM-N`

- `YYYYMM`: Release year and month
- `N`: Incremental version number within that month

**Examples:**
- `202511-1`: First release in November 2025
- `202511-2`: Second release in November 2025
- `202512-1`: First release in December 2025

## Default Version

If `CCO_VERSION` is not set, the default version is defined in `build.rs`:

```rust
let version = env::var("CCO_VERSION")
    .unwrap_or_else(|_| "202511-1".to_string());
```

## GitHub Release Workflow

When creating a GitHub release, set the tag to match the version format:

```bash
# Tag format: YYYYMM-N (no 'v' prefix)
git tag 202511-1
git push origin 202511-1

# Or with 'v' prefix (will be stripped during version comparison)
git tag v202511-1
git push origin v202511-1
```

## Release Asset Naming

For GitHub releases, name assets as:

- macOS ARM64: `cco-202511-1-darwin-arm64.tar.gz`
- macOS x86_64: `cco-202511-1-darwin-x86_64.tar.gz`
- Linux x86_64: `cco-202511-1-linux-x86_64.tar.gz`
- Linux ARM64: `cco-202511-1-linux-aarch64.tar.gz`

## Updating the Version

To release a new version:

1. Update `build.rs` default version if needed
2. Update `Cargo.toml` metadata section
3. Update `README.md` version badge
4. Build with the new version:
   ```bash
   CCO_VERSION=202512-1 cargo build --release
   ```
5. Test the binary:
   ```bash
   ./target/release/cco --version
   ./target/release/cco version
   ```
6. Create GitHub release with matching tag

## Running Tests

```bash
# Run all tests
cargo test

# Run version-specific tests
cargo test version

# Run with output
cargo test version -- --nocapture
```

## Development Build

For development (with debug symbols):

```bash
cargo build
./target/debug/cco --version
```

## Installation

After building, install to `~/.local/bin`:

```bash
./target/release/cco install
```

Or manually:

```bash
cp target/release/cco ~/.local/bin/
chmod +x ~/.local/bin/cco
```

## Troubleshooting

### Version shows as "202511-1" instead of custom version

Make sure to rebuild completely:

```bash
cargo clean
CCO_VERSION=202512-1 cargo build --release
```

### Cargo.toml shows version "0.0.0"

This is expected. Cargo requires semantic versioning for packages, but we use date-based versioning for display. The actual version is in the `CCO_VERSION` environment variable and `package.metadata.display_version`.
