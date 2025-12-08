# Auto-Update Feature Summary

One-page overview of CCO's auto-update system for quick reference.

## What Is Auto-Update?

CCO includes a built-in system that automatically checks for new versions, notifies you when updates are available, and helps you install them with a single command. Updates are safe, non-disruptive, and can be fully automated or completely manual depending on your preference.

## Key Features

| Feature | Description |
|---------|-------------|
| **Automatic Checks** | Checks for updates daily in the background (or on your schedule) |
| **Non-Disruptive** | Update process doesn't interrupt running services |
| **Safe Installation** | Backs up current version, tests new binary, rolls back on failure |
| **Flexible** | Choose automatic or manual installation (stable channel only) |
| **Fast** | Download and install in 10-45 seconds |
| **Production-Ready** | Supports fleet management, scheduled updates, compliance tracking |

## Quick Start (2 Minutes)

### Check for Updates
```bash
cco update --check
```

### Install an Update
```bash
cco update
```
Shows release notes, asks for confirmation, then installs.

Or without confirmation:
```bash
cco update --yes
```

### View Configuration
```bash
cco config show
```

## Default Configuration

```
Enabled: true              (Check for updates automatically)
Auto-install: false        (Don't install without asking)
Check interval: daily      (Check once per day)
Channel: stable            (Use tested releases only)
```

This means CCO will check for updates once daily and notify you when a new version is available. You then decide when to install it.

## Customizing Updates

### Disable Auto-Checks (Manual Only)
```bash
cco config set updates.check_interval never
```

### Auto-Install Updates
```bash
cco config set updates.auto_install true
```

### Check Weekly Instead of Daily
```bash
cco config set updates.check_interval weekly
```

### Channel
Only the stable channel is supported for automated updates.

## Version Numbers

CCO uses date-based versioning: **YYYY.MM.N**

```
2025.11.1    November 2025, first release
2025.11.2    November 2025, second release
2025.12.1    December 2025, resets to 1
2026.1.1     January 2026, resets to 1
```

## Update Channels

**Stable** (default)
- Fully tested releases
- Recommended for production
- Weekly or less frequent

**Beta**
- Not currently available via auto-update

## How Updates Work

```
1. Check GitHub for latest release
2. Compare version numbers
3. If newer available, notify user
4. User runs: cco update
5. Download platform archive + checksums.txt from GitHub Releases
6. Verify SHA256 checksum
7. Backup current version
8. Extract and install new binary
9. Test new binary works
10. Success or automatic rollback
```

**Timeline:**
- Check: 2-5 seconds
- Download: 5-30 seconds
- Installation: 10-15 seconds
- **Total: 10-45 seconds**

## Where Configuration Is Stored

```
~/.config/cco/config.toml     (macOS/Linux)
%AppData%\cco\config.toml     (Windows)
```

Edit this file directly with any text editor if you prefer.

## Recommended Setups

### For Development Machines

```bash
# Automatic, friendly updates
cco config set updates.enabled true
cco config set updates.check_interval daily
cco config set updates.auto_install false    # You decide when
cco config set updates.channel stable
```

### For Production Servers

```bash
# Manual, scheduled updates only
cco config set updates.enabled false         # No auto-checks
cco config set updates.check_interval never  # Only manual
cco config set updates.auto_install false
cco config set updates.channel stable

# Then schedule via cron
# 0 2 * * 2 /usr/local/bin/cco update --yes
```

### For Testing New Features

```bash
# Beta channel, but don't auto-install
cco config set updates.channel beta
cco config set updates.auto_install false

# Check when you want to test
cco update --check

# Install when ready
cco update --yes
```

## Common Commands

```bash
# Check for updates (no install)
cco update --check

# Check and install (with confirmation)
cco update

# Check and install (automatic)
cco update --yes

# Check beta channel
cco update --channel beta

# Show all configuration
cco config show

# Get specific setting
cco config get updates.channel

# Change a setting
cco config set updates.check_interval daily

# Show current version
cco version
```

## Troubleshooting

### "Update check never completes"
1. Check GitHub status: https://www.githubstatus.com/
2. Check internet: `ping github.com`
3. Disable auto-checks: `cco config set updates.enabled false`

### "Permission denied when updating"
1. Create directory: `mkdir -p ~/.local/bin`
2. Fix permissions: `chmod 755 ~/.local/bin`
3. Ensure you own it: `chown $(whoami) ~/.local/bin`

### "Checksum verification failed"
1. This is a safety feature—file was corrupted
2. Try the update again (files will be re-downloaded)
3. Check your internet connection

### Failed Update/Rollback
1. Previous version is automatically restored from backup
2. Verify: `cco version`
3. If still broken: `cp ~/.local/bin/cco.backup ~/.local/bin/cco`

## What Gets Updated

When you update CCO, you get:
- New features
- Bug fixes
- Performance improvements
- Security patches
- Updated documentation

Your configuration, data, and running services are never affected.

## Security

- **HTTPS only**: All communication encrypted
- **Checksum verified**: SHA256 hashing validates integrity
- **Atomic installation**: Replace is atomic, rollback on failure
- **Backup available**: Previous version saved for recovery
- **No auto-execute**: You control when updates install

## Performance

- **Non-blocking**: Checks happen in background
- **Timeout**: 10 seconds per check
- **Fast install**: 10-45 seconds total
- **Bandwidth**: ~50-100 MB per update
- **Storage**: ~100 MB temporary, ~50 MB for binary

## Support & Reporting

**Report bugs or request features:**
https://github.com/brentley/cco-releases/issues

**View releases:**
https://github.com/brentley/cco-releases/releases

**Include when reporting:**
- Your version: `cco version`
- Your system: `uname -a`
- The error message
- Steps to reproduce

## Full Documentation

This is a quick reference. For complete information, see:

- **User Guide**: Getting started, configuration, troubleshooting
- **Administrator Guide**: Production deployments, fleet management, automation
- **Architecture**: How the system works internally
- **FAQ**: Common questions and answers
- **Command Reference**: Detailed command syntax
- **Index**: Navigation hub for all documentation

## Key Takeaways

1. **Updates are automatic by default** — CCO checks daily and notifies you
2. **You control when to install** — Confirmation required by default
3. **Updates are safe** — Automatic backup and rollback if anything fails
4. **No interruptions** — Running services continue working during updates
5. **Configurable** — Adjust frequency, channel, and auto-install to your needs
6. **Production-ready** — Supports all deployment scenarios

## Getting Started

```bash
# See what's configured
cco config show

# Check for updates now
cco update --check

# Install latest version
cco update

# Or install without confirming
cco update --yes

# Switch channels if you want
cco config set updates.channel stable  # Stable (default)
cco config set updates.channel beta    # Beta (testing)

# That's it! Updates are ready to go
```

## See Also

- [Complete User Guide](AUTO_UPDATE_USER_GUIDE.md)
- [Administrator Guide for Production](AUTO_UPDATE_ADMIN_GUIDE.md)
- [Detailed Architecture](AUTO_UPDATE_ARCHITECTURE.md)
- [Frequently Asked Questions](AUTO_UPDATE_FAQ.md)
- [Command Reference](AUTO_UPDATE_COMMAND_REFERENCE.md)
- [Documentation Index](AUTO_UPDATE_INDEX.md)

