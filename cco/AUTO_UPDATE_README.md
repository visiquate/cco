# CCO Auto-Update System

Complete documentation for Claude Code Orchestra's secure, automatic update system.

## What's New

CCO now includes automatic security updates enabled by default. Updates are checked daily and installed automatically in the background, with no action required from you.

## Quick Start

### For Users
- **Updates are on by default** - No action needed
- **Your data is safe** - Only the binary is replaced
- **You can disable if desired** - Run: `cco config set updates.enabled false`
- **Check your status** - Run: `cco --version`

**Start here**: [User Guide](docs/AUTO_UPDATE_USER_GUIDE.md)

### For Administrators
- **Default behavior**: Auto-check daily, auto-install enabled
- **Control options**: Disable globally, require confirmation, or full manual
- **Monitor across teams**: View logs, track updates, audit changes
- **Deploy in stages**: Pilot → Beta → General rollout

**Start here**: [Admin Guide](docs/AUTO_UPDATE_ADMIN_GUIDE.md)

### For Security Teams
- **12 security fixes implemented**
- **HTTPS-only, checksum verification, secure temp files**
- **Threat model documented**
- **Compliance verified** (OWASP, CWE, NIST)

**Start here**: [Security Guide](docs/AUTO_UPDATE_SECURITY.md)

---

## Key Features

### Default Automatic Updates
- Daily checks for new releases
- Background installation of patches
- Zero user intervention
- Fully reversible with automatic rollback

### 12 Security Fixes
1. HTTPS-only connections
2. SHA256 checksum verification
3. Secure temporary file handling
4. Release tag validation
5. Binary verification before installation
6. Atomic binary replacement
7. Automatic backup creation
8. GitHub repository verification
9. Update channel isolation
10. Checksum file validation
11. Network retry logic with exponential backoff
12. Secure update logging

### Complete Control
- Disable auto-updates: `cco config set updates.enabled false`
- Require confirmation: `cco config set updates.auto_install false`
- Switch channels: `cco config set updates.channel beta`
- Manual updates: `cco update --yes`

### Enterprise-Ready
- Organization-wide configuration
- Centralized monitoring and logging
- Staged rollout strategies
- Compliance and audit trails
- Team management capabilities

---

## Documentation Overview

### For End Users
| Document | Purpose | Read Time |
|----------|---------|-----------|
| [Auto-Update User Guide](docs/AUTO_UPDATE_USER_GUIDE.md) | Complete user guide with examples | 30 min |
| [Auto-Update FAQ](docs/AUTO_UPDATE_FAQ.md) | Quick Q&A format | 10 min |
| [Auto-Update Migration Guide](docs/AUTO_UPDATE_MIGRATION_GUIDE.md) | For existing users | 15 min |

### For Administrators
| Document | Purpose | Read Time |
|----------|---------|-----------|
| [Auto-Update Admin Guide](docs/AUTO_UPDATE_ADMIN_GUIDE.md) | Deployment and management | 40 min |
| [Auto-Update Command Reference](docs/AUTO_UPDATE_COMMAND_REFERENCE.md) | All commands | Quick ref |

### For Security Teams
| Document | Purpose | Read Time |
|----------|---------|-----------|
| [Auto-Update Security](docs/AUTO_UPDATE_SECURITY.md) | Security features | 20 min |
| [Auto-Update Security Hardening](docs/AUTO_UPDATE_SECURITY_HARDENING.md) | Implementation details | 45 min |

### For Developers
| Document | Purpose | Read Time |
|----------|---------|-----------|
| [Auto-Update Architecture](docs/AUTO_UPDATE_ARCHITECTURE.md) | System design | 20 min |
| [Auto-Update Command Reference](docs/AUTO_UPDATE_COMMAND_REFERENCE.md) | API reference | Quick ref |

### For Troubleshooting
| Document | Purpose | Read Time |
|----------|---------|-----------|
| [Auto-Update Troubleshooting Advanced](docs/AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) | Complex issues | 60+ min |
| [Documentation Index](docs/AUTO_UPDATE_DOCUMENTATION_INDEX.md) | Navigation guide | 5 min |

---

## Common Tasks

### Check if updates are working
```bash
cco --version
tail -20 ~/.cco/logs/updates.log
```

### Disable auto-updates
```bash
cco config set updates.enabled false
```

### Require confirmation before installing
```bash
cco config set updates.auto_install false
```

### Manually check for updates
```bash
cco update --check
```

### Manually install updates
```bash
cco update --yes
```

### View your configuration
```bash
cco config show
```

### Deploy across your organization
See: [Admin Guide - Staged Rollout](docs/AUTO_UPDATE_ADMIN_GUIDE.md#staged-rollout-strategy)

---

## Security Highlights

### Every update is verified before installation
- Downloaded over HTTPS only (encrypted)
- SHA256 checksum verified
- Binary tested before replacing current version
- Previous version backed up automatically
- Automatic rollback if anything fails

### Your data is protected
- Configuration files preserved
- Credentials never accessed
- Only the binary is updated
- Logs all operations for auditing

### Designed for organizations
- GitHub repository verification
- Stable/beta channel support
- Secure temporary file handling
- Network retry with exponential backoff
- Comprehensive logging

**See**: [Security Guide](docs/AUTO_UPDATE_SECURITY.md) for all 12 fixes

---

## Default Configuration

```toml
[updates]
enabled = true              # Auto-updates are on
auto_install = true         # Automatic background installation
check_interval = "daily"    # Check every 24 hours
channel = "stable"          # Stable releases (no beta)
```

To change settings:
```bash
cco config set updates.enabled false        # Disable
cco config set updates.auto_install false   # Require confirmation
cco config set updates.check_interval weekly # Check weekly
cco config set updates.channel beta         # Try beta releases
```

---

## File Locations

- **Configuration**: `~/.config/cco/config.toml`
- **Update logs**: `~/.cco/logs/updates.log`
- **Binary location**: `~/.local/bin/cco`
- **Backup copy**: `~/.local/bin/cco.backup`
- **Temporary files**: `/tmp/cco-*` (automatically cleaned up)

---

## FAQ

### Q: Do I need to do anything for updates?
**A**: No. If you haven't disabled auto-updates, CCO automatically checks daily and installs patches in the background.

### Q: Will updates interrupt my work?
**A**: No. Updates happen in the background. Most take effect immediately; some may need a restart.

### Q: What if I don't want automatic updates?
**A**: Disable them:
```bash
cco config set updates.enabled false
```
You can update manually anytime with: `cco update --yes`

### Q: Are my credentials safe during updates?
**A**: Yes. Configuration and credentials are never accessed or modified. Only the binary is replaced.

### Q: Can I see what changed in each update?
**A**: Yes:
```bash
tail -50 ~/.cco/logs/updates.log  # Local logs
# Or view on GitHub: https://github.com/yourusername/cco/releases
```

### Q: What happens if an update fails?
**A**: The previous version is automatically restored. You can also manually restore:
```bash
mv ~/.local/bin/cco.backup ~/.local/bin/cco
cco --version
```

### Q: How do I deploy across multiple machines?
**A**: See: [Admin Guide - Staged Rollout](docs/AUTO_UPDATE_ADMIN_GUIDE.md#staged-rollout-strategy)

### Q: What security features are included?
**A**: 12 comprehensive security fixes including HTTPS-only, checksum verification, secure temp files, atomic installation, and more. See: [Security Guide](docs/AUTO_UPDATE_SECURITY.md)

---

## Navigation

### I'm a user who just installed CCO
→ Read: [Auto-Update User Guide](docs/AUTO_UPDATE_USER_GUIDE.md) (5 min overview)

### I'm migrating from manual updates
→ Read: [Auto-Update Migration Guide](docs/AUTO_UPDATE_MIGRATION_GUIDE.md)

### I'm deploying CCO across my organization
→ Read: [Auto-Update Admin Guide](docs/AUTO_UPDATE_ADMIN_GUIDE.md)

### I need to understand the security
→ Read: [Auto-Update Security](docs/AUTO_UPDATE_SECURITY.md)

### I have update issues
→ Read: [Auto-Update FAQ](docs/AUTO_UPDATE_FAQ.md) first, then [Troubleshooting Advanced](docs/AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) if needed

### I need a specific command
→ Read: [Auto-Update Command Reference](docs/AUTO_UPDATE_COMMAND_REFERENCE.md)

### I need the big picture
→ Read: [Documentation Index](docs/AUTO_UPDATE_DOCUMENTATION_INDEX.md)

---

## Complete Documentation List

All documentation is in the `docs/` directory:

1. **AUTO_UPDATE_USER_GUIDE.md** - Complete user guide (~30 pages)
2. **AUTO_UPDATE_FAQ.md** - Quick Q&A format (~15 pages)
3. **AUTO_UPDATE_ADMIN_GUIDE.md** - Administrator guide (~25 pages)
4. **AUTO_UPDATE_SECURITY.md** - Security features (~20 pages)
5. **AUTO_UPDATE_MIGRATION_GUIDE.md** - Migration from manual updates (~10 pages)
6. **AUTO_UPDATE_COMMAND_REFERENCE.md** - All commands (~20 pages)
7. **AUTO_UPDATE_ARCHITECTURE.md** - System design (~15 pages)
8. **AUTO_UPDATE_SECURITY_HARDENING.md** - Security implementation (~25 pages)
9. **AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md** - Advanced debugging (~30 pages)
10. **AUTO_UPDATE_DOCUMENTATION_INDEX.md** - Navigation guide (~10 pages)
11. **AUTO_UPDATE_DOCUMENTATION_SUMMARY.md** - Overview of all docs (~10 pages)

**Total**: ~170 pages of comprehensive documentation

---

## Support

### For Questions
Check the [FAQ](docs/AUTO_UPDATE_FAQ.md) first - it covers most common questions.

### For Configuration Help
See: [Command Reference](docs/AUTO_UPDATE_COMMAND_REFERENCE.md)

### For Deployment Help
See: [Admin Guide](docs/AUTO_UPDATE_ADMIN_GUIDE.md)

### For Troubleshooting
1. Check: [User Guide troubleshooting section](docs/AUTO_UPDATE_USER_GUIDE.md#troubleshooting)
2. If still stuck: [Advanced troubleshooting](docs/AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md)

### For Security Questions
See: [Security Guide](docs/AUTO_UPDATE_SECURITY.md)

---

## Key Points to Remember

✅ **Auto-updates are on by default** - No action needed
✅ **Updates are safe** - 12 security fixes, fully reversible
✅ **You have control** - Can disable or require confirmation
✅ **Your data is safe** - Only binary is replaced
✅ **Automatic backup** - Previous version preserved
✅ **Secure logging** - All operations logged for audit
✅ **Organization-ready** - Enterprise deployment supported

---

## Getting Started

### Right Now (1 minute)
```bash
cco --version                    # Check current version
cco config show                  # View current settings
```

### Today (5 minutes)
Read: [Auto-Update User Guide](docs/AUTO_UPDATE_USER_GUIDE.md#what-you-need-to-know)

### This Week (30 minutes)
Complete: [Auto-Update User Guide](docs/AUTO_UPDATE_USER_GUIDE.md)

### For Your Organization (2+ hours)
Complete: [Admin Guide](docs/AUTO_UPDATE_ADMIN_GUIDE.md) and plan deployment

---

## Version Information

- **Documentation Version**: 1.0 (November 2025)
- **CCO Version**: See `cco --version`
- **Auto-Update Supported**: Yes (enabled by default)
- **All 12 Security Fixes**: ✅ Implemented
- **Default Behavior**: ✅ Auto-enabled
- **Opt-Out Available**: ✅ Yes

---

## Quick Reference Card

```bash
# Check version
cco --version

# View update status
cco update --check
tail -20 ~/.cco/logs/updates.log

# Disable auto-updates
cco config set updates.enabled false

# Enable auto-updates
cco config set updates.enabled true

# Manual update
cco update --yes

# Require confirmation
cco config set updates.auto_install false

# View configuration
cco config show

# Try beta releases
cco config set updates.channel beta

# Restore previous version
mv ~/.local/bin/cco.backup ~/.local/bin/cco
```

---

**Documentation Created**: November 17, 2025
**Status**: Complete and Ready
**Coverage**: All topics documented
**Security Fixes**: All 12 documented
**Auto-Update Default**: Yes (enabled)
**Opt-Out Available**: Yes (easy)

Start with: [Auto-Update User Guide](docs/AUTO_UPDATE_USER_GUIDE.md)
