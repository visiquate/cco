# Auto-Update User Guide

Learn how to keep CCO updated with the latest features, bug fixes, and performance improvements.

## Overview

CCO includes a built-in auto-update system that checks for new versions and notifies you when updates are available. Updates are released regularly using a date-based versioning scheme (YYYY.MM.N format), making it easy to identify when each version was released.

### What Gets Updated?

When you update CCO, you receive:
- New features and improvements
- Bug fixes and security patches
- Performance enhancements
- Updated documentation

### Supported Platforms

Auto-updates work on:
- macOS (Intel and Apple Silicon)
- Linux (x86_64 and ARM64)
- Windows (x86_64) - limited support

## Quick Start

### Check for Updates

**Without installing:**
```bash
cco update --check
```

Output:
```
→ Checking for updates...
→ Current version: 2025.11.1
→ Latest version: 2025.11.2
ℹ️  New version available: v2025.11.2
Run 'cco update' to install
```

**Or see what's new in the latest version:**
```bash
cco update
```

You'll see the release notes before being prompted to install.

### Install an Update

**Interactive (prompts for confirmation):**
```bash
cco update
```

The system will show release notes and ask:
```
Update now? [Y/n]:
```

**Automatic (no confirmation):**
```bash
cco update --yes
```

## Configuration Options

### View Current Settings

```bash
cco config show
```

Output:
```
Update Configuration:
  Enabled: true
  Auto-install: false
  Check interval: daily
  Channel: stable
  Last check: 2025-11-17 14:32:15 UTC
  Last update: 2025-11-10 09:22:03 UTC
```

### Enable/Disable Auto-Update Checks

**Enable updates (default):**
```bash
cco config set updates.enabled true
```

**Disable updates:**
```bash
cco config set updates.enabled false
```

When disabled, CCO will not check for updates in the background.

### Configure Check Frequency

**Check daily (default):**
```bash
cco config set updates.check_interval daily
```

**Check weekly:**
```bash
cco config set updates.check_interval weekly
```

**Never check automatically:**
```bash
cco config set updates.check_interval never
```

With `never`, you must manually run `cco update` to check.

### Enable Auto-Install

**Disable auto-install (default):**
```bash
cco config set updates.auto_install false
```

When disabled, updates are downloaded and installed only when you run `cco update` or respond "yes" to the update prompt.

**Enable auto-install:**
```bash
cco config set updates.auto_install true
```

When enabled, updates are installed automatically in the background without prompting. This is useful in production environments where you want to stay current with minimal manual intervention.

### Update Channel

Only the stable channel is supported for automated updates. Beta/prerelease updates are not distributed via auto-update.

## What Happens During an Update

### Update Process Overview

```
1. Check GitHub for latest stable release
2. Compare versions (date-based: YYYY.MM.N)
3. Download platform archive + checksums.txt
4. Verify SHA256 checksum from checksums.txt
5. Extract archive
6. Backup current version
7. Install new version
8. Restart services (if needed)
```

> Note: If GitHub API rate limits are hit during metadata fetch, you can provide a `GITHUB_TOKEN` in the environment to raise limits. Artifact downloads remain unauthenticated.


### Time Required

- Check only: 2-5 seconds
- Download: 5-30 seconds (depends on file size and connection)
- Installation: 2-5 seconds
- Total: 10-40 seconds

### Service Continuity

The update process is safe and doesn't interrupt running services:

1. **Backup created**: Current version is backed up before replacement
2. **Atomic installation**: New binary replaces old one immediately
3. **Verification**: New binary is tested before confirming success
4. **Rollback available**: If verification fails, previous version is restored

### After Update

After a successful update, CCO logs will show:
```
✅ Successfully updated to v2025.11.2
Restart CCO to use the new version.
```

**When using daemon mode:**
```bash
cco daemon restart
```

**When running directly:**
- Stop the current instance (Ctrl+C)
- Start it again: `cco run`

## Configuration File

Settings are stored in `~/.config/cco/config.toml`:

```toml
[updates]
enabled = true               # Enable/disable update checks
auto_install = false         # Auto-install without prompt
check_interval = "daily"     # daily/weekly/never
channel = "stable"           # stable/beta
last_check = "2025-11-17T14:32:15Z"  # ISO timestamp
last_update = "2025-11-10T09:22:03Z" # ISO timestamp
```

### Configuration Path

- **macOS/Linux**: `~/.config/cco/config.toml`
- **Windows**: `%AppData%\cco\config.toml`

The file is created automatically on first use.

## Command Reference

| Command | Purpose |
|---------|---------|
| `cco update` | Check for and install updates (interactive) |
| `cco update --check` | Check only, don't install |
| `cco update --yes` | Check and install without prompting |
| `cco update --channel beta` | Check/install from beta channel |
| `cco config show` | View all update settings |
| `cco config set updates.enabled true` | Enable update checks |
| `cco config set updates.check_interval daily` | Set check frequency |
| `cco config set updates.auto_install true` | Enable auto-install |
| `cco config set updates.channel stable` | Switch to stable channel |
| `cco config get updates.enabled` | Get specific setting |
| `cco version` | Show current version + check for updates |

## Troubleshooting

### "Update check never completes"

**Symptoms**: `cco update` hangs or times out

**Solutions**:
1. Check internet connection: `ping github.com`
2. Check GitHub status: https://www.githubstatus.com/
3. Try again in a few minutes
4. Use `--check` flag to see what's happening: `cco update --check`

**Manual workaround**:
```bash
# Disable auto-checks
cco config set updates.enabled false

# Download and install manually from:
# https://github.com/brentley/cco-releases/releases
```

### "Permission denied when updating"

**Symptoms**: `Failed to install new binary: Permission denied`

**Cause**: The installation directory (`~/.local/bin/`) is not writable

**Solutions**:

1. Check directory permissions:
```bash
ls -la ~/.local/bin/
```

2. Create directory with correct permissions:
```bash
mkdir -p ~/.local/bin
chmod 755 ~/.local/bin
```

3. Ensure you own the directory:
```bash
chown $(whoami) ~/.local/bin
```

4. Try update again:
```bash
cco update --yes
```

### "Checksum mismatch error"

**Symptoms**: `Checksum verification failed! Update aborted.`

**Cause**: Downloaded binary was corrupted or modified

**Solutions**:

1. The system already prevents installation - this is working correctly
2. Try the update again - files will be re-downloaded:
```bash
cco update --yes
```

3. Check your internet connection for dropped packets
4. Try from a different network

### "New binary verification failed"

**Symptoms**:
```
⚠️  New binary verification failed, rolling back...
Update failed, rolled back to previous version
```

**Cause**: New binary doesn't work on your system

**Solutions**:

1. Check that your platform is supported:
```bash
# Shows your system info
uname -a
```

2. Supported combinations:
   - macOS: arm64 (Apple Silicon) or x86_64 (Intel)
   - Linux: x86_64 or aarch64
   - Windows: x86_64

3. Report the issue with your system info:
   - Include output from: `uname -a`
   - Include CCO version: `cco version`
   - Include error details from the update output

### "Version comparison issues"

**Symptoms**: Says newer version is available when you already have it

**Cause**: Version format parsing error

**Versions use YYYY.MM.N format:**
- `2025.11.1` - first release in November 2025
- `2025.11.2` - second release in November 2025
- `2025.12.1` - resets to 1 in new month
- `2026.1.1` - resets to 1 in new year

**Example comparison:**
```
2025.11.1 < 2025.11.2  ✓ Correct
2025.11.2 < 2025.12.1  ✓ Correct
2025.12.1 < 2026.1.1   ✓ Correct
```

## Background Update Checks

CCO runs update checks in the background:

- **When**: When you start `cco run` or use any command
- **Frequency**: Based on your `check_interval` setting
- **Non-blocking**: Doesn't slow down CCO startup
- **Silent**: Doesn't show output unless new version is found

### Sample Output When Update Available

When CCO starts and a newer version is found:
```
🚀 Starting Claude Code Orchestra 2025.11.1...

ℹ️  New version available: 2025.11.2 (current: 2025.11.1)
   Run 'cco update' to upgrade
```

### Disable Background Checks

```bash
cco config set updates.enabled false
```

Then check only when you want:
```bash
cco update --check
```

## Keeping Multiple Versions

The auto-update system doesn't keep old versions by default. If you need to revert:

### Automatic Backup

If an update fails during installation, the previous version is automatically restored from `~/.local/bin/cco.backup`.

### Manual Revert

You can download and install any previous version:

1. Visit https://github.com/brentley/cco-releases/releases
2. Download the version you want
3. Extract and place in `~/.local/bin/cco`
4. Make it executable: `chmod +x ~/.local/bin/cco`

## Production Recommendations

### For Production Servers

1. **Stable channel only** (already default):
```bash
cco config set updates.channel stable
```

2. **Manual updates recommended**:
```bash
cco config set updates.check_interval never
cco config set updates.auto_install false
```

Then check on a schedule:
```bash
# Weekly update check (run from cron)
cco update --yes --channel stable
```

3. **Sample crontab entry** (Tuesday 2 AM UTC):
```
0 2 * * 2 /home/user/.local/bin/cco update --yes
```

### For Development Machines

1. **Enable auto-checks** (default):
```bash
cco config set updates.enabled true
cco config set updates.check_interval daily
```

2. **Choose your channel**:
```bash
# For latest stable features
cco config set updates.channel stable

# For testing new features
cco config set updates.channel beta
```

3. **Optional: Enable auto-install for convenience**:
```bash
cco config set updates.auto_install true
```

## FAQ

**Q: Is it safe to update while CCO is running?**

A: Safe, but requires restart. The update process:
1. Downloads to temporary directory
2. Backs up current version
3. Replaces binary
4. Tests new binary
5. Restarts are handled automatically in daemon mode

**Q: What if update fails halfway?**

A: The previous version is restored automatically. You can safely run `cco update` again.

**Q: Can I schedule updates?**

A: Yes, use cron (Linux/macOS) or Task Scheduler (Windows):
```bash
# Check daily at 3 AM
0 3 * * * /usr/local/bin/cco update --yes
```

**Q: Will updates change my configuration?**

A: No. All your settings in `~/.config/cco/config.toml` are preserved.

**Q: How do I know what changed in a version?**

A: Run `cco update` to see release notes, or visit:
https://github.com/brentley/cco-releases/releases

**Q: Can I disable updates completely?**

A: Yes:
```bash
cco config set updates.enabled false
```

Then manually manage updates using the releases page.

## See Also

- [Administrator Guide](./AUTO_UPDATE_ADMIN_GUIDE.md) - For production deployments
- [Versioning Guide](./VERSIONING.md) - How version numbers work
- [Getting Started](./README.md) - Installation and setup

