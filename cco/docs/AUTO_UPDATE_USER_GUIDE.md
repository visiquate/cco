# Auto-Update User Guide

## Overview

CCO (Claude Code Orchestra) now includes automatic security updates enabled by default. Your installation will automatically check for and install security patches daily, requiring no action from you. This guide explains how the auto-update system works and how to manage it if needed.

## What You Need to Know

### Auto-Updates Are On By Default

When you install CCO, automatic updates are enabled. This means:

- **Daily checks**: CCO checks for updates every 24 hours
- **Automatic installation**: Critical security patches install automatically in the background
- **Zero interruption**: Updates complete without stopping your work
- **Restart optional**: Most updates take effect immediately; some require a restart

### Why Auto-Updates Matter

Auto-updates keep your CCO installation secure by:

1. **Delivering security patches immediately** - No waiting for the next manual update
2. **Protecting your credentials** - Patches prevent unauthorized access to your API keys
3. **Ensuring platform stability** - Bug fixes improve reliability
4. **Maintaining compatibility** - Staying current with API changes

## Understanding the Update Process

### What Happens During an Auto-Update

The update process is completely safe and follows these steps:

```
1. CHECK (Daily)
   ↓ Connects to GitHub repository
   ↓ Compares your version with latest
   ↓ Returns to sleep if up-to-date

2. DOWNLOAD (If update available)
   ↓ Retrieves platform-specific binary
   ↓ Creates temporary file

3. VERIFY (Before installing)
   ↓ Validates SHA256 checksum
   ↓ Confirms file integrity
   ↓ Aborts if verification fails

4. BACKUP (Safety first)
   ↓ Creates backup copy of current version
   ↓ Stored as ~/.local/bin/cco.backup

5. INSTALL (Atomic operation)
   ↓ Replaces binary with new version
   ↓ Preserves all configuration files
   ↓ No downtime required

6. TEST (Verify success)
   ↓ Runs version check on new binary
   ↓ Confirms it works properly

7. CLEANUP (Remove temporary files)
   ↓ Deletes download directory
   ↓ Removes temporary files
```

### Important Safety Features

All auto-updates include these protections:

- **Checksum verification**: Every binary is verified before installation
- **Atomic operations**: Binary replacement is atomic (all-or-nothing)
- **Automatic rollback**: If anything fails, previous version is restored
- **No data loss**: Configuration and credentials remain unchanged
- **Secure connections**: All downloads use HTTPS

## Checking Update Status

### View Your Current Version

```bash
cco --version
# Output: cco 2025.11.2 (commit: abc123)
```

### Check for Updates Manually

```bash
# Check if an update is available (non-intrusive)
cco update --check
# Output: CCO 2025.11.3 is available (you have 2025.11.2)
```

### See Update History

View the update log to understand what's been installed:

```bash
# View recent updates
tail -50 ~/.cco/logs/updates.log

# View all updates with details
cat ~/.cco/logs/updates.log

# Search for specific updates
grep "successfully installed" ~/.cco/logs/updates.log
```

### Understand Log Entries

Each update creates a log entry like:

```
[2025-11-17 14:32:15] Check started for stable channel
[2025-11-17 14:32:18] Update available: 2025.11.3 (current: 2025.11.2)
[2025-11-17 14:32:45] Downloaded 12.5 MB from GitHub
[2025-11-17 14:32:46] Verified checksum: SHA256 match
[2025-11-17 14:32:47] Created backup: ~/.local/bin/cco.backup
[2025-11-17 14:32:47] Installing new version...
[2025-11-17 14:32:48] Verified new binary works
[2025-11-17 14:32:49] Successfully installed 2025.11.3
[2025-11-17 14:32:50] Cleanup: Removed temporary files
```

## Managing Auto-Updates

### Disable Auto-Updates (Optional)

If you prefer to update manually, you can disable automatic updates:

```bash
# Disable all update checks
cco config set updates.enabled false

# Verify it's disabled
cco config show
# Should show: updates.enabled = false
```

### Allow Checking But Require Confirmation

If you want to be notified about updates but install them manually:

```bash
# Still check daily, but prompt before installing
cco config set updates.auto_install false

# When an update is available, you'll be prompted:
# "Update available: 2025.11.3. Install now? (y/n)"
```

### Change Check Frequency

Adjust how often CCO checks for updates:

```bash
# Check daily (default)
cco config set updates.check_interval daily

# Check weekly
cco config set updates.check_interval weekly

# Never check automatically (manual only)
cco config set updates.check_interval never
```

### Switch Update Channels

By default, CCO gets stable releases. Optionally try beta releases:

```bash
# Stay with stable releases (default)
cco config set updates.channel stable

# Get beta/pre-release updates (may be less stable)
cco config set updates.channel beta
```

### View Your Configuration

```bash
# See all update settings
cco config show

# Look for the [updates] section:
# [updates]
# enabled = true
# auto_install = true
# check_interval = "daily"
# channel = "stable"
# last_check = 2025-11-17T14:32:15Z
# last_update = 2025-11-17T14:32:49Z
```

## Manual Updates

If you prefer to update manually, use these commands:

### Check Without Installing

```bash
# See what's available
cco update --check
```

### Install Latest Update Immediately

```bash
# Interactive - prompts before installing
cco update

# Automatic - installs without prompting
cco update --yes

# Install from beta channel
cco update --channel beta

# Install specific version
cco update --version 2025.11.3
```

## Environment Variable Overrides

You can override configuration using environment variables:

```bash
# Disable updates via environment
export CCO_UPDATES_ENABLED=false
cco

# Force auto-install via environment
export CCO_UPDATES_AUTO_INSTALL=true
cco

# Override check interval
export CCO_UPDATES_CHECK_INTERVAL=weekly
cco

# Override channel
export CCO_UPDATES_CHANNEL=beta
cco
```

## Troubleshooting

### Updates Not Installing

**Problem**: You haven't seen any updates for a while

**Solutions**:

```bash
# Check if updates are enabled
cco config set updates.enabled true

# Check last update time
tail -10 ~/.cco/logs/updates.log

# Force an immediate check
cco update --check

# Manually install latest
cco update --yes
```

### Permission Denied Error

**Problem**: Update fails with "Permission denied"

**Solutions**:

```bash
# Option 1: Check directory permissions
ls -la ~/.local/bin/ | grep cco

# Option 2: Run with sudo (if installed system-wide)
sudo cco update --yes

# Option 3: Reinstall to user directory
wget https://github.com/yourusername/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz
tar xzf cco-v2025.11.3-linux-x86_64.tar.gz
mv cco ~/.local/bin/cco
```

### Checksum Verification Failed

**Problem**: "Checksum verification failed! Update aborted."

**Explanation**: The downloaded file doesn't match the expected hash. This can indicate:
- Corrupted download
- Network interruption
- Potential security issue (rare)

**Solution**:

```bash
# Wait and retry - this often resolves network issues
cco update --yes

# If it persists, check your internet connection
ping github.com

# Report as security issue if it continues
# Visit: https://github.com/yourusername/cco/security/advisories
```

### Rollback After Failed Update

If for any reason an update causes problems:

```bash
# Restore the previous version
mv ~/.local/bin/cco.backup ~/.local/bin/cco

# Verify it works
cco --version

# Report the issue so we can fix it
```

### Update Checking Seems Slow

**Problem**: `cco update --check` takes longer than expected

**Explanation**:
- First check may download release information
- Network conditions affect speed
- GitHub API rate limits may apply

**Solution**:

```bash
# Check are cached, subsequent checks are faster
# Run again to see cached response

# If internet is limited, disable checks
cco config set updates.check_interval never

# Or increase interval
cco config set updates.check_interval weekly
```

## Frequently Asked Questions

### Do I need to do anything to get updates?

No. If you haven't disabled auto-updates, CCO automatically checks daily and installs critical patches in the background.

### Will updates interrupt my work?

No. Updates happen in the background. Most updates take effect immediately without restart. If a restart is needed, you'll be notified and can choose when to restart.

### Can I see what changed in each update?

Yes. View update details:

```bash
# See update history with versions
grep "successfully installed" ~/.cco/logs/updates.log

# View release notes on GitHub
https://github.com/yourusername/cco/releases

# Check specific version changes
cco update --show-changes 2025.11.3
```

### What if my internet is offline?

Updates are skipped if offline. When you reconnect, CCO will check and install any pending updates on the next scheduled check.

### Can updates be scheduled for a specific time?

Currently, checks happen daily but at a consistent time. Custom scheduling is planned for a future release.

### Are my credentials safe during updates?

Yes. Configuration files and credentials are never modified or accessed during updates. Only the CCO binary is replaced.

### Can I disable updates just for today?

Yes, temporarily:

```bash
# Disable for this session
export CCO_UPDATES_ENABLED=false
cco

# Re-enable for next session
unset CCO_UPDATES_ENABLED
```

### What if I'm on a restricted network?

You can disable update checks:

```bash
cco config set updates.enabled false
```

Then update manually when you have internet access.

### How do I update if I don't have my configuration file?

Updates don't depend on configuration. Even if your config is lost:

```bash
# Manual update still works
cco update --yes

# Configuration will be recreated with defaults on next run
```

### Are beta updates safer than stable?

Beta updates are tested but may contain newer experimental features. Stable updates are the most tested and recommended for production use.

## What Gets Updated

### Binary Updates

- CCO executable program
- All built-in agents and models
- Core functionality and APIs
- Performance improvements
- Security patches

### What Does NOT Change

- Configuration files (~/.config/cco/config.toml)
- Credentials and API keys
- Local cache and databases
- Project files and code
- User settings

## Contact Support

If you encounter issues with auto-updates:

1. **Check the logs**: `cat ~/.cco/logs/updates.log`
2. **Review this guide**: Look for similar issues above
3. **Search GitHub Issues**: https://github.com/yourusername/cco/issues
4. **Report new issues**: https://github.com/yourusername/cco/issues/new

Include in your report:
- Your CCO version: `cco --version`
- Your platform: `uname -a`
- Recent log entries: `tail -30 ~/.cco/logs/updates.log`
- Any error messages you received

## Advanced Configuration

For advanced users managing multiple installations:

```bash
# See all available configuration options
cco config help

# Export current configuration
cco config export > cco-config.toml

# Import configuration on another machine
cco config import cco-config.toml

# Check configuration without making changes
cco config validate
```

## Summary

- **Auto-updates are on by default** - No action needed
- **Updates are safe** - Fully verified before installation
- **You can disable them** - Set `updates.enabled = false` if preferred
- **Check logs anytime** - View history in `~/.cco/logs/updates.log`
- **Manual updates work too** - Use `cco update` command
- **Your data is safe** - Only the binary changes, not your files

For more information, see:
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md) - Security features detailed
- [Auto-Update Architecture](AUTO_UPDATE_ARCHITECTURE.md) - How it works internally
- [Auto-Update Troubleshooting (Advanced)](AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) - Deep debugging
