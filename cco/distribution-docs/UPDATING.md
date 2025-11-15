# CCO Update Guide

Complete guide to updating and version management for CCO (Claude Code Orchestra).

## Table of Contents

- [Update Overview](#update-overview)
- [Automatic Updates](#automatic-updates)
- [Manual Updates](#manual-updates)
- [Update Channels](#update-channels)
- [Rollback and Recovery](#rollback-and-recovery)
- [Version Management](#version-management)
- [Update Configuration](#update-configuration)
- [Troubleshooting Updates](#troubleshooting-updates)

## Update Overview

CCO includes a built-in update system that:

- **Checks for updates** automatically (configurable interval)
- **Notifies you** when updates are available
- **Installs updates** automatically (opt-in)
- **Verifies integrity** with SHA256 checksums
- **Backs up** previous versions for rollback
- **Supports multiple channels** (stable, beta, nightly)

### Update Process

```
1. Check for updates → version-manifest.json
2. Compare versions → Current vs Latest
3. Download binary → GitHub Releases
4. Verify checksum → SHA256 validation
5. Backup old version → ~/.local/bin/cco.backup
6. Atomic replace → Install new version
7. Notify user → Update complete
```

## Automatic Updates

### Enabling Automatic Updates

```bash
# Enable automatic updates
cco config set updates.auto_install true

# Configure check interval
cco config set updates.check_interval daily  # daily, weekly, never

# Select update channel
cco config set updates.channel stable  # stable, beta, nightly
```

### How Automatic Updates Work

1. **Background Check**: CCO checks for updates on startup (if check interval elapsed)
2. **Download**: If update available and auto_install enabled, downloads in background
3. **Verification**: Verifies SHA256 checksum
4. **Installation**: Atomically replaces binary
5. **Notification**: Shows brief notification about the update

**Example**:
```
CCO updated to v0.3.0
Previous version backed up to ~/.local/bin/cco.backup
Restart CCO to use the new version
```

### Configuration

```toml
# ~/.config/cco/config.toml
[updates]
enabled = true
auto_install = true           # Enable automatic installation
check_interval = "daily"      # Check frequency
channel = "stable"            # Update channel
notify_on_update = true       # Show notification
last_check = "2025-11-15T10:00:00Z"  # Auto-updated
```

### Disabling Automatic Updates

```bash
# Disable automatic updates
cco config set updates.auto_install false

# Or disable checks entirely
cco config set updates.enabled false
```

## Manual Updates

### Checking for Updates

```bash
# Check if update is available
cco update --check

# Check with verbose output
cco update --check --verbose

# Check specific channel
cco update --check --channel beta
```

**Example Output**:
```
Current version: 0.2.0
Latest version:  0.3.0
Update available: Yes

Release notes: https://github.com/brentley/cco-releases/releases/tag/v0.3.0

To install:
  cco update --install
```

### Installing Updates

```bash
# Install latest version (interactive)
cco update --install

# Install without confirmation
cco update --install --yes

# Install from specific channel
cco update --install --channel beta

# Install specific version
cco update --install --version 0.3.0
```

**Installation Process**:
```
Checking for updates...
Found update: 0.2.0 → 0.3.0

Downloading: cco-v0.3.0-darwin-arm64.tar.gz
Progress: [████████████████████] 100% (2.9 MB)

Verifying checksum...
Checksum verified: OK

Backing up current version...
Backup saved: ~/.local/bin/cco.backup

Installing new version...
Installation complete!

CCO updated to v0.3.0
Run 'cco --version' to verify
```

### Installation Script Method

You can also use the installation script to upgrade:

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex
```

The installer detects existing installations and upgrades automatically.

## Update Channels

CCO supports three update channels:

### Stable Channel (Recommended)

Production-ready releases that have been thoroughly tested.

```bash
cco config set updates.channel stable
```

**Characteristics**:
- Released every 2-4 weeks
- Full test coverage
- Security reviewed
- Production-ready
- Long-term support

**Example versions**: `0.2.0`, `0.3.0`, `1.0.0`

### Beta Channel

Pre-release versions for early adopters and testing.

```bash
cco config set updates.channel beta
```

**Characteristics**:
- Released weekly
- Feature complete
- Tested but may have bugs
- Good for testing new features
- May have breaking changes

**Example versions**: `0.3.0-beta.1`, `0.3.0-beta.2`, `0.4.0-beta.1`

### Nightly Channel

Latest development builds for bleeding-edge features.

```bash
cco config set updates.channel nightly
```

**Characteristics**:
- Built daily from main branch
- Latest features
- Unstable, may have bugs
- For developers and testing only
- Breaking changes possible

**Example versions**: `0.4.0-nightly.20251115`, `0.4.0-nightly.20251116`

### Switching Channels

```bash
# Switch to beta channel
cco config set updates.channel beta
cco update --check

# Switch back to stable
cco config set updates.channel stable
cco update --check
```

**Note**: Switching from stable to beta/nightly will upgrade. Switching back to stable may require rollback if no new stable version exists.

## Rollback and Recovery

### Viewing Backups

```bash
# List available backups
cco update --list-backups

# Show backup details
ls -lh ~/.local/bin/cco.backup*
```

**Example Output**:
```
Available backups:
  0.2.0  ~/.local/bin/cco.backup       (3.1 MB)  2025-11-15 10:00
  0.1.0  ~/.local/bin/cco.backup.old   (2.9 MB)  2025-11-01 14:30
```

### Rolling Back

```bash
# Rollback to previous version
cco update --rollback

# Rollback to specific version
cco update --rollback --version 0.2.0

# Force rollback (skip confirmation)
cco update --rollback --yes
```

**Rollback Process**:
```
Current version: 0.3.0
Rollback to: 0.2.0

Backing up current version...
Restoring previous version...
Rollback complete!

CCO reverted to v0.2.0
Run 'cco --version' to verify
```

### Manual Rollback

If the automatic rollback fails:

```bash
# Restore from backup manually
cp ~/.local/bin/cco.backup ~/.local/bin/cco
chmod +x ~/.local/bin/cco

# Verify
cco --version
```

### Emergency Recovery

If CCO becomes unusable after an update:

```bash
# Option 1: Manual rollback (above)

# Option 2: Reinstall from scratch
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Option 3: Install specific version
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | \
  bash -s -- --version 0.2.0
```

## Version Management

### Checking Current Version

```bash
# Show version
cco --version

# Show detailed version info
cco version --verbose
```

**Example Output**:
```
CCO (Claude Code Orchestra) 0.2.0
Platform: darwin-arm64
Build date: 2025-11-15
Commit: abc1234
Update channel: stable
```

### Version Constraints

You can set version constraints to prevent unwanted updates:

```toml
# ~/.config/cco/config.toml
[updates]
minimum_version = "0.2.0"     # Don't downgrade below this
skip_versions = ["0.2.5"]     # Skip problematic versions
```

```bash
# Set minimum version
cco config set updates.minimum_version 0.2.0

# Skip specific version
cco config set updates.skip_versions '["0.2.5"]'
```

### Version Pinning

To stay on a specific version:

```bash
# Disable updates
cco config set updates.enabled false

# Or keep checks enabled but disable auto-install
cco config set updates.auto_install false
```

### Release Notes

```bash
# View release notes for current version
cco version --release-notes

# View release notes for specific version
cco version --release-notes --version 0.3.0

# Or visit GitHub
open https://github.com/brentley/cco-releases/releases/tag/v0.3.0
```

## Update Configuration

### Configuration Options

```toml
# ~/.config/cco/config.toml
[updates]
# Enable/disable update system
enabled = true

# Automatic installation
auto_install = false          # Require explicit opt-in

# Check frequency
check_interval = "daily"      # daily, weekly, never

# Update channel
channel = "stable"            # stable, beta, nightly

# Notifications
notify_on_update = true       # Show notification when updated

# Security
verify_signatures = true      # Verify GPG signatures (future)

# Rollback
keep_backups = true           # Keep backup of previous version
backup_count = 3              # Number of backups to keep

# Version constraints
minimum_version = "0.2.0"     # Don't downgrade below this
skip_versions = []            # Skip specific versions

# Internal (auto-updated)
last_check = "2025-11-15T10:00:00Z"
last_update = "2025-11-15T10:00:00Z"
```

### Environment Variables

```bash
# Override update channel
export CCO_UPDATE_CHANNEL="beta"

# Disable updates
export CCO_UPDATE_ENABLED="false"

# Force update check on startup
export CCO_UPDATE_CHECK_NOW="true"
```

### Command-Line Overrides

```bash
# Check beta channel regardless of config
cco update --check --channel beta

# Force update check
cco update --check --force

# Install without auto-update enabled
cco update --install --force
```

## Update Schedule

### Automatic Check Schedule

CCO checks for updates based on `check_interval`:

| Interval | Frequency | Use Case |
|----------|-----------|----------|
| `daily` | Every 24 hours | Development, stay current |
| `weekly` | Every 7 days | Production, stability |
| `never` | Manual only | Controlled environments |

### Check on Startup

CCO checks for updates on startup if:
1. Update checking is enabled (`updates.enabled = true`)
2. Check interval has elapsed since last check
3. Not in the middle of a request

The check is non-blocking and happens in the background.

### Force Update Check

```bash
# Force immediate check (ignores interval)
cco update --check --force

# Force update on next startup
cco config set updates.last_check "2000-01-01T00:00:00Z"
```

## Troubleshooting Updates

### Update Check Fails

**Problem**: `cco update --check` fails with network error.

**Solutions**:
```bash
# 1. Check network connectivity
curl -I https://github.com

# 2. Check GitHub API access
curl https://api.github.com/repos/brentley/cco-releases/releases/latest

# 3. Use proxy if behind firewall
export HTTPS_PROXY="http://proxy.company.com:8080"
cco update --check

# 4. Check DNS resolution
nslookup github.com
```

### Download Fails

**Problem**: Update download fails partway through.

**Solutions**:
```bash
# 1. Retry the update
cco update --install --retry

# 2. Clear download cache
rm -rf ~/.cache/cco/downloads

# 3. Manual download and install
curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.3.0/cco-v0.3.0-darwin-arm64.tar.gz
tar -xzf cco.tar.gz
mv cco ~/.local/bin/cco
chmod +x ~/.local/bin/cco
```

### Checksum Verification Fails

**Problem**: Downloaded file fails checksum verification.

**Solutions**:
```bash
# 1. Clear cache and retry
rm -rf ~/.cache/cco/downloads
cco update --install

# 2. Download fresh checksums
curl -L -o checksums.sha256 https://github.com/brentley/cco-releases/releases/download/v0.3.0/checksums.sha256

# 3. Verify manually
sha256sum -c checksums.sha256 --ignore-missing
```

### Update Installed But Version Unchanged

**Problem**: Update installed successfully but `cco --version` shows old version.

**Solutions**:
```bash
# 1. Restart CCO (if running)
pkill cco
cco --version

# 2. Check which binary is being used
which cco
# Should be ~/.local/bin/cco

# 3. Check PATH
echo $PATH | grep ".local/bin"

# 4. Verify binary was actually updated
ls -l ~/.local/bin/cco
stat ~/.local/bin/cco  # Check modification time
```

### Permission Denied During Update

**Problem**: Update fails with permission error.

**Solutions**:
```bash
# 1. Check file permissions
ls -l ~/.local/bin/cco

# 2. Fix permissions
chmod +x ~/.local/bin/cco

# 3. Check directory permissions
ls -ld ~/.local/bin
chmod 755 ~/.local/bin

# 4. Reinstall to fix permissions
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

### Rollback Not Working

**Problem**: Rollback fails or backup not found.

**Solutions**:
```bash
# 1. Check if backup exists
ls -lh ~/.local/bin/cco.backup*

# 2. If backup missing, reinstall specific version
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | \
  bash -s -- --version 0.2.0

# 3. If backup corrupted, download fresh
curl -L -o cco https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz
tar -xzf cco.tar.gz -O cco > ~/.local/bin/cco
chmod +x ~/.local/bin/cco
```

## Update Best Practices

### For Development

```bash
# Stay on latest stable
cco config set updates.channel stable
cco config set updates.auto_install true
cco config set updates.check_interval daily
```

### For Production

```bash
# Manual updates only
cco config set updates.channel stable
cco config set updates.auto_install false
cco config set updates.check_interval weekly
cco config set updates.notify_on_update true
```

### For Testing

```bash
# Get early access to new features
cco config set updates.channel beta
cco config set updates.auto_install true
cco config set updates.check_interval daily
```

### For Enterprise

```bash
# Controlled updates with version pinning
cco config set updates.enabled false  # Disable auto-updates
# Manual updates through deployment pipeline
# Test in staging before production
# Coordinated rollout across team
```

## Migration Guides

### Upgrading from 0.1.x to 0.2.x

```bash
# 1. Backup configuration
cp ~/.config/cco/config.toml ~/.config/cco/config.toml.backup

# 2. Install 0.2.x
cco update --install --version 0.2.0

# 3. Review configuration changes
cco config show --effective

# 4. Test
cco proxy --test-config
```

No breaking changes in 0.2.x - configuration is backward compatible.

## Next Steps

- [CONFIGURATION.md](CONFIGURATION.md) - Update configuration options
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - More troubleshooting help
- [CHANGELOG.md](../CHANGELOG.md) - See what's new in each version

---

Last updated: 2025-11-15
