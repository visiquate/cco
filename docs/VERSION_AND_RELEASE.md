# Versioning and Release Process

CCO uses a date-based versioning system combined with automatic git hash tracking for complete build traceability. This document explains the versioning format, how to build releases, and how the auto-update system works.

## Versioning Format

### Version String Structure

CCO uses the format: **`YYYY.MM.DD+<git-hash>`**

- **YYYY** - Four-digit year (e.g., 2025)
- **MM** - Month (1-12, no zero-padding required)
- **DD** - Day of month (1-31)
- **git-hash** - 7-character short git commit hash (automatic, set at build time)

### Examples

```
2025.11.18+ff0d033     # Released on Nov 18, 2025, from commit ff0d033
2025.12.1+a1b2c3d      # Released on Dec 1, 2025, from commit a1b2c3d
2026.1.15+9e8f7a6     # Released on Jan 15, 2026, from commit 9e8f7a6
```

### Version Semantics

The version format emphasizes **when** a release was created and **exactly which commit** was built, rather than feature/breaking change semantics. This makes it easy to:
- Identify exactly when a release was created
- Trace back to the exact git commit
- Compare versions by date (2025.11.18 < 2025.12.1)

## Building CCO

### Build Requirements

The `build.rs` script requires the `VERSION_DATE` environment variable to be set before building. This ensures every build has a date associated with it.

### Building a Release

To build a release, set the `VERSION_DATE` environment variable before running `cargo build`:

```bash
# Build with today's date
VERSION_DATE=$(date +%Y.%-m.%-d) cargo build --release

# Or manually specify the release date
VERSION_DATE=2025.11.18 cargo build --release

# Verify the version
./target/release/cco version
# Output: CCO 2025.11.18+ff0d033
```

### VERSION_DATE Format Requirements

- **Required**: Must be set, build fails without it
- **Format**: `YYYY.MM.DD` (e.g., `2025.11.18`)
- **No zero-padding**: Month and day don't need to be zero-padded (but they can be)
  - Valid: `2025.1.5`, `2025.11.18`, `2025.12.31`
  - Invalid: `2025-11-18` (wrong separator), `2025.11` (missing day)

### Build Failure Examples

```bash
# This will FAIL - VERSION_DATE not set
cargo build --release
# ERROR: VERSION_DATE environment variable is required!
# Usage: VERSION_DATE=2025.11.18 cargo build --release
# Format: YYYY.MM.DD (e.g., 2025.11.18)

# This will FAIL - wrong format
VERSION_DATE=2025-11-18 cargo build --release
# ERROR: VERSION_DATE format invalid: 2025-11-18
# Expected format: YYYY.MM.DD (e.g., 2025.11.18)

# This will FAIL - incomplete date
VERSION_DATE=2025.11 cargo build --release
# ERROR: VERSION_DATE format invalid: 2025.11
# Expected format: YYYY.MM.DD (e.g., 2025.11.18)
```

### Git Hash Automatic Capture

The git commit hash is captured automatically by `build.rs` at compile time:

```rust
// In build.rs
let git_hash = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .ok()
    .and_then(|output| String::from_utf8(output.stdout).ok())
    .unwrap_or_else(|| "unknown".to_string())
    .trim()
    .to_string();

// Final version: VERSION_DATE+git_hash
let version = format!("{}+{}", base_version, git_hash);
```

**Key points:**
- Hash is captured from the current git HEAD
- 7-character short hash (standard git format)
- If git command fails, uses "unknown" as fallback
- Hash is part of the binary metadata, no additional action needed

## Development Workflow

### Local Development Builds

For local development, you'll need to set a VERSION_DATE when building:

```bash
# Option 1: Use today's date
VERSION_DATE=$(date +%Y.%-m.%-d) cargo build --release

# Option 2: Use a fixed dev date
VERSION_DATE=2025.11.18 cargo build --release

# Option 3: Use a dev build script
./scripts/build-dev.sh
```

### Creating a Dev Build Script

If you frequently build locally, create a helper script at `scripts/build-dev.sh`:

```bash
#!/bin/bash
# Build script for local development

set -e

# Use today's date for version
VERSION_DATE=$(date +%Y.%-m.%-d)

echo "Building CCO $VERSION_DATE..."
VERSION_DATE="$VERSION_DATE" cargo build --release

echo "✅ Build complete!"
./target/release/cco version
```

Make it executable:
```bash
chmod +x scripts/build-dev.sh
```

Then build with:
```bash
./scripts/build-dev.sh
```

## Release Process

### Step 1: Determine Release Date

Decide on the release date (typically today, or a specific date):

```bash
# Get today's date
RELEASE_DATE=$(date +%Y.%-m.%-d)
echo "Release date: $RELEASE_DATE"

# Or specify manually
RELEASE_DATE=2025.11.18
```

### Step 2: Build with Release Date

Build the binary with the release date:

```bash
# Set the VERSION_DATE and build
VERSION_DATE=$RELEASE_DATE cargo build --release

# The git hash is automatically captured from current HEAD
# Binary is at: ./target/release/cco
```

### Step 3: Verify Version

Confirm the binary has the correct version:

```bash
./target/release/cco version
# Expected output: CCO 2025.11.18+ff0d033 (or similar)
```

### Step 4: Test the Build

Run basic tests to ensure the build is functional:

```bash
# Quick health check
./target/release/cco daemon status

# Run tests (optional but recommended)
cargo test --release
```

### Step 5: Create Release Tag

Create a git tag matching the release version:

```bash
# Tag format: v<version>
git tag -a "v2025.11.18+ff0d033" -m "Release 2025.11.18"

# Or simpler (without git hash in tag):
git tag -a "v2025.11.18" -m "Release 2025.11.18"

# Push tag to remote
git push origin "v2025.11.18"
```

### Step 6: Create GitHub Release

1. Go to GitHub repository releases page
2. Click "New Release"
3. Select the tag you just created
4. Fill in release title and notes
5. Publish the release

**Release Title**: `CCO 2025.11.18`

**Release Notes**: Include:
- New features
- Bug fixes
- Breaking changes (if any)
- Installation instructions

### Complete Release Workflow Example

```bash
# 1. Ensure you're on the correct branch and up-to-date
git checkout main
git pull origin main

# 2. Determine release date
RELEASE_DATE=$(date +%Y.%-m.%-d)
echo "Releasing CCO $RELEASE_DATE"

# 3. Build with release date
VERSION_DATE=$RELEASE_DATE cargo build --release

# 4. Verify version
./target/release/cco version
# Output: CCO 2025.11.18+ff0d033

# 5. Run tests (optional)
cargo test --release

# 6. Get git hash from binary (should match)
GIT_HASH=$(git rev-parse --short HEAD)
FULL_VERSION="$RELEASE_DATE+$GIT_HASH"
echo "Full version: $FULL_VERSION"

# 7. Create and push tag
git tag -a "v$RELEASE_DATE" -m "Release $RELEASE_DATE"
git push origin "v$RELEASE_DATE"

# 8. Create GitHub release (via web UI or gh CLI)
gh release create "v$RELEASE_DATE" \
  "./target/release/cco" \
  --title "CCO $RELEASE_DATE" \
  --notes "See CHANGELOG.md for details"
```

## Daemon Auto-Update

The CCO daemon automatically detects when the CLI binary has a newer version and performs a graceful auto-update.

### How Auto-Update Works

1. **Trigger**: Every `cco` command checks if daemon version < CLI version
2. **Detection**: Compares version strings using semantic comparison (year, month, day)
3. **Message**: If update needed, user sees: `⚠️  Daemon version older than cco binary, auto-updating...`
4. **Process**:
   - Stops current daemon gracefully
   - Replaces daemon binary with new version
   - Restarts daemon automatically
   - User's command continues normally
5. **Failure Handling**: If restart fails, user gets helpful error message with troubleshooting steps

### Version Comparison Logic

Versions are compared component-by-component:

```rust
// Compare year first
match self.year.cmp(&other.year) {
    // If years equal, compare month
    Ordering::Equal => match self.month.cmp(&other.month) {
        // If months equal, compare day
        Ordering::Equal => self.day.cmp(&other.day)
        other => other,
    }
    other => other,
}
```

**Examples:**
- `2025.11.18` > `2025.11.17` (newer)
- `2025.12.1` > `2025.11.30` (newer month)
- `2026.1.1` > `2025.12.31` (newer year)
- `2025.11.18` = `2025.11.18` (equal, regardless of git hash)

### Git Hash Comparison

**Important**: The git hash is NOT used in version comparisons. Two binaries built from different commits on the same date are considered the same version.

This means:
- Version `2025.11.18+abc123` equals `2025.11.18+def456`
- Auto-update won't trigger between them
- Both have the same semantic version number

## Troubleshooting

### Build fails with "VERSION_DATE not set"

**Symptom:**
```
ERROR: VERSION_DATE environment variable is required!
Usage: VERSION_DATE=2025.11.18 cargo build --release
```

**Solution**: Set the VERSION_DATE environment variable before building:

```bash
# Option 1: Inline
VERSION_DATE=2025.11.18 cargo build --release

# Option 2: Export then build
export VERSION_DATE=2025.11.18
cargo build --release
unset VERSION_DATE

# Option 3: Use today's date
VERSION_DATE=$(date +%Y.%-m.%-d) cargo build --release
```

### Build fails with "VERSION_DATE format invalid"

**Symptom:**
```
ERROR: VERSION_DATE format invalid: 2025-11-18
Expected format: YYYY.MM.DD (e.g., 2025.11.18)
```

**Solution**: Use the correct date format with dots, not dashes:

```bash
# ✓ Correct
VERSION_DATE=2025.11.18 cargo build --release

# ✗ Wrong (dashes instead of dots)
VERSION_DATE=2025-11-18 cargo build --release

# ✗ Wrong (too few components)
VERSION_DATE=2025.11 cargo build --release
```

### Daemon not auto-updating

**Symptom**: User updates CCO binary but daemon continues to use old version.

**Diagnosis**:
1. Check if daemon is still running: `cco daemon status`
2. Verify version mismatch: Compare `cco version` with `cco daemon status`
3. Check daemon logs: `cco daemon logs`

**Solutions**:
1. Restart daemon manually: `cco daemon restart`
2. Force daemon upgrade: Build new daemon and restart
3. Check permissions: Ensure daemon process can be restarted
4. Review logs: Look for error messages in daemon logs

### Version string doesn't match expected format

**Symptom**: `cco version` shows unexpected format

**Solutions**:
1. Verify build completed successfully
2. Check that you set VERSION_DATE: `echo $VERSION_DATE`
3. Confirm git is available: `git --version`
4. Rebuild from scratch: `cargo clean && VERSION_DATE=$(date +%Y.%-m.%-d) cargo build --release`

## Configuration Files

### Version Metadata

Version information is stored in `/Users/brent/git/cc-orchestra/cco/src/version.rs`:

```rust
pub struct DateVersion {
    year: u32,
    month: u32,
    day: u32,
    git_hash: Option<String>,
}
```

This struct:
- Parses version strings in `YYYY.MM.DD+<hash>` format
- Implements `Ord` for proper version comparison
- Provides methods to access components
- Ignores git hash in comparisons

### Build Configuration

Build-time version setup is in `/Users/brent/git/cc-orchestra/cco/build.rs`:

```rust
// Set version from VERSION_DATE env var
let base_version = env::var("VERSION_DATE").unwrap_or_else(|_| {
    eprintln!("ERROR: VERSION_DATE environment variable is required!");
    std::process::exit(1);
});

// Append git hash
let version = format!("{}+{}", base_version, git_hash);

// Make available to binary as CCO_VERSION constant
println!("cargo:rustc-env=CCO_VERSION={}", version);
```

## See Also

- [VERSION.rs](/Users/brent/git/cc-orchestra/cco/src/version.rs) - Version struct and parsing
- [build.rs](/Users/brent/git/cc-orchestra/cco/build.rs) - Build-time version setup
- [launcher.rs](/Users/brent/git/cc-orchestra/cco/src/commands/launcher.rs) - Daemon auto-update logic
- [CHANGELOG.md](/Users/brent/git/cc-orchestra/CHANGELOG.md) - Release history

## Quick Reference

```bash
# Build commands
VERSION_DATE=$(date +%Y.%-m.%-d) cargo build --release
VERSION_DATE=2025.11.18 cargo build --release

# Verify version
./target/release/cco version

# Create release tag
git tag -a "v2025.11.18" -m "Release 2025.11.18"
git push origin "v2025.11.18"

# Check daemon version
cco daemon status

# Force daemon restart (triggers auto-update)
cco daemon restart
```

---

**Last Updated**: November 18, 2025
**Relevant Files**:
- `/Users/brent/git/cc-orchestra/cco/src/version.rs`
- `/Users/brent/git/cc-orchestra/cco/build.rs`
- `/Users/brent/git/cc-orchestra/cco/src/commands/launcher.rs`
