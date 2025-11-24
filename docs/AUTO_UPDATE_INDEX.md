# Auto-Update Feature Documentation Index

Complete documentation for CCO's built-in auto-update system.

## Quick Navigation

**For different roles:**
- [User Guide](#user-guide) - I want to keep CCO updated
- [Administrator Guide](#administrator-guide) - I manage CCO in production
- [Architecture](#architecture) - I want to understand how it works
- [FAQ](#faq) - I have a specific question
- [Command Reference](#command-reference) - I need command syntax

## Documentation Files

### User Guide
**File:** `AUTO_UPDATE_USER_GUIDE.md`
**For:** End users and developers

Learn how to:
- Check for updates
- Install updates interactively or automatically
- Configure update behavior
- Troubleshoot common issues
- Use CCO in production

**Key sections:**
- Quick start (2-minute setup)
- Configuration options (enable/disable, intervals, channels)
- What happens during an update
- Service continuity during updates
- Background update checks
- Production recommendations
- FAQ section

**Start here if you want to:** Keep CCO updated and working smoothly

### Administrator Guide
**File:** `AUTO_UPDATE_ADMIN_GUIDE.md`
**For:** System administrators and DevOps engineers

Learn how to:
- Deploy CCO with controlled updates
- Manage updates across multiple instances
- Monitor update status
- Handle failed updates
- Implement organizational policies
- Set up automated update schedules
- Implement compliance and audit trails

**Key sections:**
- Deployment strategies (manual, staged, automatic)
- Multi-instance management and fleet updates
- Monitoring and status tracking
- Failed update recovery
- Network and connectivity
- Configuration management
- Audit and compliance
- Performance considerations
- Automation examples (cron, systemd, Kubernetes)

**Start here if you want to:** Manage CCO updates across many servers

### Architecture
**File:** `AUTO_UPDATE_ARCHITECTURE.md`
**For:** Developers and technical teams

Learn how:
- The auto-update system is designed
- Each module works (auto_update.rs, update.rs, version.rs)
- Configuration is stored and managed
- Update checking and installation processes work
- Version comparison works
- Error handling is implemented
- Security is maintained
- Performance characteristics
- Debugging and testing

**Key sections:**
- System architecture diagram
- Core modules overview
- Configuration storage
- Update flow (background, manual, interactive, auto-install)
- Installation process (download, verify, backup, install, test)
- GitHub integration
- Version comparison strategy
- Error handling
- Security considerations
- Performance characteristics
- Future enhancements

**Start here if you want to:** Understand or modify the system

### FAQ
**File:** `AUTO_UPDATE_FAQ.md`
**For:** Anyone with specific questions

Common questions and answers about:
- General update information (frequency, timing, what's included)
- Configuration (storage, backups, defaults)
- Update process (interruption, downtime, timeouts)
- Channels (stable vs. beta, switching)
- Troubleshooting (common errors and fixes)
- Version management (numbering, history, downgrades)
- Production usage (scheduling, monitoring, rollbacks)
- Support and reporting

**Start here if you want to:** Find the answer to a specific question

### Command Reference
**File:** `AUTO_UPDATE_COMMAND_REFERENCE.md`
**For:** Anyone using CCO commands

Complete reference for:
- Update commands (`cco update`, `--check`, `--yes`, `--channel`)
- Configuration commands (`cco config show`, `set`, `get`)
- Version command (`cco version`)
- Advanced usage (scripts, automation, conditional updates)
- Exit codes
- Command help

**Start here if you want to:** Know the exact syntax for a command

## Feature Overview

### What is Auto-Update?

CCO includes a built-in system that:

1. **Checks for updates automatically** (daily by default)
2. **Notifies you** when new versions are available
3. **Allows manual or automatic installation** based on your preference
4. **Supports different update channels** (stable, beta)
5. **Handles failures gracefully** with automatic rollback
6. **Works in the background** without interrupting your work

### Key Capabilities

- **Transparent**: Works like the official update system
- **Configurable**: Control frequency, channels, and auto-install
- **Safe**: Backups current version, verifies new binary, rolls back on failure
- **Fast**: Download and install in 10-45 seconds
- **Secure**: SHA256 checksums, HTTPS only, no automatic execution
- **Production-ready**: Supports scheduled updates, fleet management, audit trails

## Version Format

CCO uses date-based versioning: **YYYY.MM.N**

```
2025.11.1    First release in November 2025
2025.11.2    Second release in November 2025
2025.12.1    First release in December (resets to 1)
2026.1.1     First release in January (resets to 1)
```

## Update Channels

**Stable** (recommended for production)
- Fully tested releases
- Weekly or less frequent
- Recommended for all users

**Beta** (testing/development only)
- Pre-release versions
- Multiple times per week
- May have bugs or breaking changes
- Use only for testing new features

## Configuration

Updates are configured in: `~/.config/cco/config.toml`

```toml
[updates]
enabled = true                  # Enable/disable checks
auto_install = false            # Auto-install without prompt
check_interval = "daily"        # daily/weekly/never
channel = "stable"              # stable/beta
last_check = "2025-11-17T..."   # Last check timestamp
last_update = "2025-11-10T..."  # Last update timestamp
```

## Command Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `cco update` | Check and install (interactive) | `cco update` |
| `cco update --check` | Check only | `cco update --check` |
| `cco update --yes` | Check and install (no prompt) | `cco update --yes` |
| `cco update --channel beta` | Check beta channel | `cco update --channel beta` |
| `cco config show` | View all settings | `cco config show` |
| `cco config set KEY VALUE` | Change a setting | `cco config set updates.enabled true` |
| `cco config get KEY` | View one setting | `cco config get updates.channel` |
| `cco version` | Show version and check for updates | `cco version` |

## Typical Workflows

### As a Developer (Auto-Updates)

```bash
# Default setup - checks daily, notifies when new version
cco config set updates.enabled true
cco config set updates.check_interval daily
cco config set updates.channel stable

# Check for updates anytime
cco update --check

# Install when you see a notification
cco update
```

### In Production (Manual Updates)

```bash
# Disable automatic checks
cco config set updates.enabled false
cco config set updates.check_interval never

# Schedule weekly updates
0 2 * * 2 /usr/local/bin/cco update --yes

# Monitor across your fleet
for server in prod-{1,2,3}; do
    ssh $server cco version
done
```

### Testing Beta Features (Beta Channel)

```bash
# Switch to beta channel
cco config set updates.channel beta

# Check for beta releases
cco update --check

# Install beta version
cco update --yes

# Switch back to stable when done
cco config set updates.channel stable
```

## Troubleshooting Quick Links

| Issue | Guide |
|-------|-------|
| "Update check never completes" | [User Guide - Troubleshooting](AUTO_UPDATE_USER_GUIDE.md#troubleshooting) |
| "Permission denied when updating" | [User Guide - Troubleshooting](AUTO_UPDATE_USER_GUIDE.md#permission-denied-when-updating) |
| "Checksum mismatch error" | [User Guide - Troubleshooting](AUTO_UPDATE_USER_GUIDE.md#checksum-mismatch-error) |
| "New binary verification failed" | [User Guide - Troubleshooting](AUTO_UPDATE_USER_GUIDE.md#new-binary-verification-failed) |
| How to roll back | [Admin Guide - Handling Failed Updates](AUTO_UPDATE_ADMIN_GUIDE.md#handling-failed-updates) |
| How to update a fleet | [Admin Guide - Multi-Instance Management](AUTO_UPDATE_ADMIN_GUIDE.md#multi-instance-management) |
| How to schedule updates | [Admin Guide - Automation Examples](AUTO_UPDATE_ADMIN_GUIDE.md#automation-examples) |

## Feature Details

### Background Update Checks

- **When**: Every time you start CCO (unless disabled)
- **Frequency**: Based on `check_interval` setting (daily/weekly/never)
- **Behavior**: Non-blocking, runs in background
- **Timeout**: 10 seconds
- **Output**: Silent unless new version available

### Update Installation

- **Download time**: 5-30 seconds
- **Verification**: SHA256 checksum verification
- **Atomic**: Replace binary atomically, rollback on failure
- **Backup**: Current version backed up to `.backup`
- **Test**: New binary is executed and tested before confirming

### Service Continuity

- **Non-disruptive**: Update process doesn't interrupt running services
- **Automatic rollback**: If new binary fails, previous version restored
- **Restart required**: Daemon needs restart to use new version

## Security Features

- **HTTPS only**: All GitHub communication encrypted
- **Checksum verification**: SHA256 hashing prevents tampering
- **Atomic operations**: Binary replacement is atomic or near-atomic
- **Automatic rollback**: Failed installations are rolled back
- **No automatic execution**: Updates require explicit user action or config
- **Backup preservation**: Previous version available if rollback needed

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Check only | 2-5s | Async, non-blocking |
| Download | 5-30s | Depends on network/file size |
| Verify | 2-5s | SHA256 checksum |
| Extract | 1-3s | Archive expansion |
| Install | 1-5s | Binary replacement + test |
| **Total** | **10-45s** | User can continue working |

## Support Resources

- **Bug Reports**: https://github.com/brentley/cco-releases/issues
- **Releases**: https://github.com/brentley/cco-releases/releases
- **Community**: https://github.com/brentley/cco/discussions

## Related Documentation

- [CCO README](../README.md) - Main project documentation
- [Installation Guide](./INSTALL_UPDATE_IMPLEMENTATION.md) - Installation details
- [Versioning Guide](./VERSIONING.md) - Version numbering scheme
- [Production Checklist](./PRODUCTION_CHECKLIST.md) - Production readiness

## Documentation Structure

```
docs/
├── AUTO_UPDATE_INDEX.md              # This file (navigation hub)
├── AUTO_UPDATE_USER_GUIDE.md         # For end users
├── AUTO_UPDATE_ADMIN_GUIDE.md        # For administrators
├── AUTO_UPDATE_ARCHITECTURE.md       # Technical details
├── AUTO_UPDATE_FAQ.md                # Common questions
├── AUTO_UPDATE_COMMAND_REFERENCE.md  # Command syntax
├── INSTALL_UPDATE_IMPLEMENTATION.md  # Implementation details
└── VERSIONING.md                     # Version numbering scheme
```

## How to Use This Documentation

1. **Finding information:**
   - Use the index above to find relevant documentation
   - Search within each document for specific topics
   - Check the Table of Contents in each document

2. **Common tasks:**
   - Setup auto-updates: [User Guide - Quick Start](AUTO_UPDATE_USER_GUIDE.md#quick-start)
   - Configure update behavior: [User Guide - Configuration Options](AUTO_UPDATE_USER_GUIDE.md#configuration-options)
   - Deploy in production: [Admin Guide - Deployment Strategies](AUTO_UPDATE_ADMIN_GUIDE.md#deployment-strategies)
   - Fix a problem: [FAQ](AUTO_UPDATE_FAQ.md) or [User Guide - Troubleshooting](AUTO_UPDATE_USER_GUIDE.md#troubleshooting)

3. **Deep dives:**
   - Understanding the system: [Architecture](AUTO_UPDATE_ARCHITECTURE.md)
   - Managing at scale: [Admin Guide](AUTO_UPDATE_ADMIN_GUIDE.md)
   - Specific question: [FAQ](AUTO_UPDATE_FAQ.md)

## Document Maintenance

These documents were created to cover:
- User workflows and setup
- Administrator tasks and deployment
- Technical architecture and implementation
- Common questions and troubleshooting
- Command reference and usage

Last updated: 2025-11-17

