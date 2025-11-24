# Auto-Update Command Reference

## Overview

Complete reference for all auto-update related commands and configuration options for CCO (Claude Code Orchestra).

---

## Update Commands

### `cco update`

Check for and install updates interactively.

**Syntax**:
```bash
cco update [OPTIONS]
```

**Options**:
- `--yes` - Install without prompting
- `--channel <CHANNEL>` - Check specific channel (stable/beta)
- `--version <VERSION>` - Install specific version (not recommended)
- `--check` - Only check, don't install
- `--dry-run` - Show what would be done, don't do it
- `--verbose` - Show detailed output
- `--show-changes` - Show what changed in the update

**Examples**:

```bash
# Interactive update (prompts before installing)
cco update

# Auto-install without prompting
cco update --yes

# Check what's available without installing
cco update --check

# Try beta releases
cco update --channel beta

# Verbose output for debugging
cco update --verbose

# See what would happen without doing it
cco update --dry-run

# Show what changed in an update
cco update --show-changes 2025.11.3
```

**Output**:
```
Checking for updates... stable channel
Update available: 2025.11.3 (you have 2025.11.2)
Download size: 12.5 MB
Continue with update? (y/n)
```

---

### `cco update --check`

Check for updates without installing them.

**Syntax**:
```bash
cco update --check [OPTIONS]
```

**Options**:
- `--channel <CHANNEL>` - Check specific channel (stable/beta)
- `--verbose` - Show detailed output

**Examples**:

```bash
# Check for stable updates
cco update --check

# Check for beta updates
cco update --check --channel beta

# Verbose output
cco update --check --verbose
```

**Output**:
```
Checking for updates... stable channel
Already up to date! You have 2025.11.2 (latest: 2025.11.2)
```

Or:

```
Checking for updates... stable channel
Update available: 2025.11.3 (you have 2025.11.2)
Download size: 12.5 MB
Changes: security patches, bug fixes, new features
```

---

## Configuration Commands

### `cco config show`

Display current configuration.

**Syntax**:
```bash
cco config show [SECTION]
```

**Examples**:

```bash
# Show all configuration
cco config show

# Show only updates section
cco config show updates
```

**Output**:
```
[updates]
enabled = true
auto_install = true
check_interval = "daily"
channel = "stable"
last_check = 2025-11-17T14:32:15Z
last_update = 2025-11-17T14:32:49Z
```

---

### `cco config set`

Set configuration values.

**Syntax**:
```bash
cco config set <KEY> <VALUE>
```

**Update Configuration Keys**:

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `updates.enabled` | true/false | true | Enable auto-update checks |
| `updates.auto_install` | true/false | true | Auto-install updates |
| `updates.check_interval` | daily/weekly/never | daily | How often to check |
| `updates.channel` | stable/beta | stable | Release channel |

**Examples**:

```bash
# Disable auto-updates
cco config set updates.enabled false

# Allow checking but require confirmation
cco config set updates.auto_install false

# Check weekly instead of daily
cco config set updates.check_interval weekly

# Switch to beta channel
cco config set updates.channel beta

# Check only on demand (never automatically)
cco config set updates.check_interval never

# Re-enable auto-updates
cco config set updates.enabled true
```

---

### `cco config reset`

Reset configuration to defaults.

**Syntax**:
```bash
cco config reset [SECTION]
```

**Examples**:

```bash
# Reset only update configuration
cco config reset updates

# Reset all configuration
cco config reset
```

---

### `cco config export`

Export configuration to a file.

**Syntax**:
```bash
cco config export [FILE]
```

**Examples**:

```bash
# Export to standard location
cco config export

# Export to specific file
cco config export ~/my-cco-config.toml
```

**Output**:
```toml
[updates]
enabled = true
auto_install = true
check_interval = "daily"
channel = "stable"
```

---

### `cco config import`

Import configuration from a file.

**Syntax**:
```bash
cco config import <FILE>
```

**Examples**:

```bash
cco config import ~/my-cco-config.toml
```

---

## Log Viewing Commands

### View Update Logs

**Syntax**:
```bash
cat ~/.cco/logs/updates.log
```

**Examples**:

```bash
# View all logs
cat ~/.cco/logs/updates.log

# View recent updates (last 50 lines)
tail -50 ~/.cco/logs/updates.log

# Monitor in real-time
tail -f ~/.cco/logs/updates.log

# Search for errors
grep "ERROR" ~/.cco/logs/updates.log

# Count successful updates
grep "successfully installed" ~/.cco/logs/updates.log | wc -l

# Find failed updates
grep "ERROR\|WARN" ~/.cco/logs/updates.log
```

**Log Entry Format**:
```
[TIMESTAMP] [LEVEL] MESSAGE
[2025-11-17 14:32:15] [INFO] Check started for stable channel
[2025-11-17 14:32:46] [INFO] Verified checksum: SHA256 match
[2025-11-17 14:32:49] [INFO] Successfully installed 2025.11.3
```

---

## Version Commands

### `cco --version`

Show current CCO version.

**Syntax**:
```bash
cco --version
```

**Output**:
```
cco 2025.11.2 (commit: abc123def456)
```

---

## Environment Variable Overrides

Override configuration using environment variables:

### `CCO_UPDATES_ENABLED`

Enable/disable updates.

```bash
# Disable updates for this session
export CCO_UPDATES_ENABLED=false
cco

# Enable updates
export CCO_UPDATES_ENABLED=true
cco

# Unset (use config file value)
unset CCO_UPDATES_ENABLED
```

### `CCO_UPDATES_AUTO_INSTALL`

Control auto-installation.

```bash
# Auto-install in this session
export CCO_UPDATES_AUTO_INSTALL=true
cco

# Require confirmation
export CCO_UPDATES_AUTO_INSTALL=false
cco
```

### `CCO_UPDATES_CHECK_INTERVAL`

Set check interval.

```bash
# Check daily
export CCO_UPDATES_CHECK_INTERVAL=daily
cco

# Check weekly
export CCO_UPDATES_CHECK_INTERVAL=weekly
cco

# Never check
export CCO_UPDATES_CHECK_INTERVAL=never
cco
```

### `CCO_UPDATES_CHANNEL`

Set update channel.

```bash
# Use stable releases
export CCO_UPDATES_CHANNEL=stable
cco

# Use beta releases
export CCO_UPDATES_CHANNEL=beta
cco
```

---

## Manual Update Procedures

### Download and Install Manually

```bash
# Find latest release
LATEST=$(curl -s https://api.github.com/repos/yourusername/cco/releases/latest | jq -r '.tag_name')

# Detect your platform
OS=$(uname -s | tr A-Z a-z)
ARCH=$(uname -m)
case $ARCH in
    x86_64) ARCH=x86_64 ;;
    arm64) ARCH=aarch64 ;;
esac

# Download
URL="https://github.com/yourusername/cco/releases/download/${LATEST}/cco-${LATEST}-${OS}-${ARCH}.tar.gz"
wget "$URL"

# Extract and install
tar xzf "cco-${LATEST}-${OS}-${ARCH}.tar.gz"
mv cco ~/.local/bin/cco

# Verify
cco --version
```

### Verify Downloaded Binary

```bash
# Download binary and checksums
VERSION=2025.11.3
URL="https://github.com/yourusername/cco/releases/download/v${VERSION}"

wget "${URL}/cco-v${VERSION}-linux-x86_64.tar.gz"
wget "${URL}/checksums.sha256"

# Verify
sha256sum -c checksums.sha256
# Should show: OK

# Extract and verify
tar xzf "cco-v${VERSION}-linux-x86_64.tar.gz"
./cco --version
```

---

## Troubleshooting Commands

### Check Update Status

```bash
# View configuration
cco config show

# Check for updates
cco update --check

# View logs
tail -50 ~/.cco/logs/updates.log

# Check permissions
ls -la ~/.local/bin/cco
```

### Verify Binary Works

```bash
# Check version
cco --version

# Check configuration
cco config show

# Check connectivity
cco update --check
```

### Check Internet Connectivity

```bash
# Verify GitHub access
curl -I https://api.github.com

# Test DNS
nslookup github.com

# Test ping
ping github.com
```

### Restore Previous Version

```bash
# Restore from backup
mv ~/.local/bin/cco.backup ~/.local/bin/cco

# Verify it works
cco --version

# Check logs
tail -10 ~/.cco/logs/updates.log
```

---

## Configuration File

Location: `~/.config/cco/config.toml`

**Example Configuration**:
```toml
[updates]
enabled = true
auto_install = true
check_interval = "daily"
channel = "stable"
last_check = 2025-11-17T14:32:15Z
last_update = 2025-11-17T14:32:49Z
```

**Manual Editing** (not recommended, use `cco config set` instead):

```bash
# View config file
cat ~/.config/cco/config.toml

# Edit with text editor
nano ~/.config/cco/config.toml

# Validate after editing
cco config show  # Will report errors if invalid
```

---

## Log File Locations

| File | Purpose |
|------|---------|
| `~/.cco/logs/updates.log` | Update operation logs |
| `~/.config/cco/config.toml` | Configuration file |
| `~/.local/bin/cco` | Installed binary |
| `~/.local/bin/cco.backup` | Previous version backup |

---

## Binary Installation Locations

### Unix (macOS/Linux)

```
~/.local/bin/cco              # Installed binary
~/.local/bin/cco.backup       # Backup from last update
/usr/local/bin/cco            # System-wide (if installed there)
```

### Windows

```
%APPDATA%\Local\cco\cco.exe               # User directory
C:\Program Files\cco\cco.exe              # System-wide
```

---

## Quick Reference

### Common Tasks

**Check for updates without installing:**
```bash
cco update --check
```

**Manually install latest update:**
```bash
cco update --yes
```

**Disable automatic updates:**
```bash
cco config set updates.enabled false
```

**View update history:**
```bash
tail -50 ~/.cco/logs/updates.log
```

**Switch to beta releases:**
```bash
cco config set updates.channel beta
```

**Restore previous version:**
```bash
mv ~/.local/bin/cco.backup ~/.local/bin/cco
cco --version
```

---

## Help and Documentation

```bash
# Show help for update command
cco update --help

# Show help for config command
cco config --help

# View user guide
# See: docs/AUTO_UPDATE_USER_GUIDE.md

# View security documentation
# See: docs/AUTO_UPDATE_SECURITY.md

# View architecture details
# See: docs/AUTO_UPDATE_ARCHITECTURE.md
```

---

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md)
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md)
- [Auto-Update Administrator Guide](AUTO_UPDATE_ADMIN_GUIDE.md)
- [Auto-Update Architecture](AUTO_UPDATE_ARCHITECTURE.md)
- [Auto-Update FAQ](AUTO_UPDATE_FAQ.md)
