# Auto-Update Quick Start Guide

## For Users

### Check for Updates

```bash
# Check if update is available
cco update --check

# Install available update (interactive prompt)
cco update

# Install without prompting
cco update --yes

# Check beta channel
cco update --channel beta
```

### Configure Auto-Update

```bash
# View current configuration
cco config show

# Enable/disable update checks
cco config set updates.enabled true
cco config set updates.enabled false

# Change check interval
cco config set updates.check_interval daily   # Check daily
cco config set updates.check_interval weekly  # Check weekly
cco config set updates.check_interval never   # Never check

# Enable/disable auto-install
cco config set updates.auto_install true   # Automatic background updates
cco config set updates.auto_install false  # Manual updates only

# Change update channel
cco config set updates.channel stable  # Stable releases only
cco config set updates.channel beta    # Beta/RC releases
```

## For Developers

### Using the Auto-Update API

```rust
use cco::auto_update::AutoUpdateManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create update manager
    let mut manager = AutoUpdateManager::new()?;

    // Check for updates
    if let Some(release) = manager.check_for_updates().await? {
        println!("Update available: {}", release.version);
        println!("Download size: {} bytes", release.size);

        // Perform update
        manager.perform_update(false).await?;
    } else {
        println!("Already up to date");
    }

    Ok(())
}
```

### Background Update Checking

```rust
use cco::auto_update;

// Spawn background check (non-blocking)
auto_update::check_for_updates_async();

// This returns immediately, check runs in background
```

### Manual Download and Installation

```rust
use cco::auto_update::{AutoUpdateManager, github};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut manager = AutoUpdateManager::new()?;

    // Fetch specific version
    let release = github::fetch_release_by_version("2025.11.2").await?;

    // Download and verify
    let binary_path = manager.download_binary(&release).await?;

    // Install
    manager.replace_binary(&binary_path).await?;

    Ok(())
}
```

## Configuration File

Location: `~/.config/cco/config.toml`

```toml
[updates]
enabled = true           # Enable update checks
auto_install = false     # Automatic installation
check_interval = "daily" # daily, weekly, or never
channel = "stable"       # stable or beta

# Timestamps (managed automatically)
last_check = 2025-11-17T12:00:00Z
last_update = 2025-11-15T10:30:00Z
```

## Supported Platforms

- **macOS**: arm64 (Apple Silicon), x86_64 (Intel)
- **Linux**: x86_64 (AMD64), aarch64 (ARM64)
- **Windows**: x86_64 (AMD64)

## Update Process

1. **Check**: Query GitHub API for latest release
2. **Download**: Download platform-specific binary archive
3. **Verify**: SHA256 checksum validation
4. **Extract**: Unpack from tar.gz or zip
5. **Test**: Verify new binary works
6. **Backup**: Save current binary
7. **Install**: Atomic replacement
8. **Cleanup**: Remove temporary files

## Error Handling

All operations include automatic rollback on failure:

- Checksum mismatch → Delete and abort
- Download failure → Retry on next interval
- Installation failure → Restore from backup
- Verification failure → Keep current version

## Security

- HTTPS-only connections
- SHA256 checksum verification
- Binary verification before/after installation
- Atomic replacement operations
- Automatic rollback on any failure

## Testing

```bash
# Run auto-update tests
cargo test --lib auto_update

# Run in release mode
cargo test --lib auto_update --release
```

## Troubleshooting

### Update Check Not Working

```bash
# Check configuration
cco config show

# Ensure updates are enabled
cco config set updates.enabled true

# Force check (ignores interval)
cco update --check
```

### Permission Denied During Update

```bash
# Option 1: Install to user directory
mv ~/.local/bin/cco ~/.local/bin/cco.bak
cco update

# Option 2: Run with sudo (if installed system-wide)
sudo cco update --yes
```

### Checksum Verification Failed

This indicates a corrupted download or potential security issue. The update will abort automatically. Try again:

```bash
cco update --yes
```

If the problem persists, report it as a security issue.

## Files and Directories

### Runtime Files

- `~/.config/cco/config.toml` - Configuration
- `~/.local/bin/cco` - Installed binary (Unix)
- `~/.local/bin/cco.backup` - Backup copy
- `/tmp/cco-update-{version}/` - Temporary downloads

### Source Files

- `cco/src/auto_update/mod.rs` - Main module
- `cco/src/auto_update/github.rs` - GitHub API
- `cco/src/auto_update/updater.rs` - Update logic

## GitHub Release Requirements

Each release must include:

```
cco-v{VERSION}-darwin-arm64.tar.gz
cco-v{VERSION}-darwin-x86_64.tar.gz
cco-v{VERSION}-linux-x86_64.tar.gz
cco-v{VERSION}-linux-aarch64.tar.gz
cco-v{VERSION}-windows-x86_64.zip
checksums.sha256
```

## Example Release Workflow

```bash
# Build for all platforms
./build-all-platforms.sh

# Generate checksums
sha256sum cco-v*.tar.gz cco-v*.zip > checksums.sha256

# Create GitHub release
gh release create v2025.11.2 \
  --title "CCO 2025.11.2" \
  --notes-file RELEASE_NOTES.md \
  cco-v2025.11.2-*.tar.gz \
  cco-v2025.11.2-*.zip \
  checksums.sha256
```

## Version Format

CCO uses date-based versioning: `YYYY.MM.N`

- `YYYY`: Year (e.g., 2025)
- `MM`: Month (1-12)
- `N`: Release counter for that month (starts at 1)

Examples:
- `2025.11.1` - First release in November 2025
- `2025.11.2` - Second release in November 2025
- `2025.12.1` - First release in December 2025 (counter resets)

## API Documentation

See `/Users/brent/git/cc-orchestra/cco/AUTO_UPDATE_IMPLEMENTATION.md` for complete API reference and implementation details.
