# Auto-Update Administrator Guide

## Overview

This guide is for system administrators, IT departments, and organization managers deploying CCO across multiple machines. It covers default behavior, organizational policies, monitoring, troubleshooting, and staged rollout strategies.

## Default Behavior

### What's Enabled By Default

When CCO is installed on user machines, auto-update is configured with these defaults:

```toml
[updates]
enabled = true
auto_install = true
check_interval = "daily"
channel = "stable"
```

This means:

- Daily automatic checks for updates
- Automatic background installation of patches
- No user interaction required
- Only stable releases (no beta/RC)
- All operations logged to ~/.cco/logs/updates.log

### User Impact

With default settings:

- **Network usage**: ~1-2 MB per day for checks, ~10-50 MB when updates available
- **CPU impact**: Minimal (background processes)
- **Disk usage**: ~5-10 MB for temporary files during updates
- **Interruptions**: None (background operation)

## Organizational Configuration

### Disable Auto-Updates Organization-Wide

For organizations that want to manage updates manually:

#### Option 1: Configuration File Distribution

Create a default config file:

```toml
# cco-default-config.toml
[updates]
enabled = false
auto_install = false
check_interval = "never"
channel = "stable"
```

Deploy during installation:

```bash
# Installation script
cco install
cp cco-default-config.toml ~/.config/cco/config.toml
```

#### Option 2: Environment Variable Default

Set default for all users:

```bash
# In /etc/environment or similar
export CCO_UPDATES_ENABLED=false
export CCO_UPDATES_CHECK_INTERVAL=never
```

#### Option 3: Managed Policy (Linux/macOS)

Create policy file:

```bash
# /etc/cco/policy.toml
[updates]
enabled = false
auto_install = false
# Users cannot override if policy file is read-only
```

### Configure Staged Rollout

For large organizations, deploy updates in phases:

#### Phase 1: Beta Channel (Day 1)

```bash
# 10% of machines on beta
# Give early access to updates
cco config set updates.channel beta
cco config set updates.auto_install true
```

#### Phase 2: Stable Channel (Day 3)

```bash
# After 2 days of beta testing
# 50% of machines switch to stable
cco config set updates.channel stable
```

#### Phase 3: Full Rollout (Day 5)

```bash
# After 4 days of stable testing
# All machines on stable channel
```

### Configure Check Intervals

Adjust frequency based on organizational needs:

```bash
# Aggressive security (hourly)
# For high-risk environments
cco config set updates.check_interval hourly

# Standard (daily) - default
cco config set updates.check_interval daily

# Conservative (weekly)
# For stable environments
cco config set updates.check_interval weekly

# Never automatic (manual only)
# For locked-down environments
cco config set updates.check_interval never
```

### Require Confirmation Before Install

Allow checks but require manual installation:

```bash
# Check automatically, prompt before installing
cco config set updates.auto_install false

# Users will see:
# "Update available: 2025.11.3. Install now? (y/n)"
```

## Monitoring Updates

### Monitor Update Status Organization-Wide

Collect update status from all machines:

```bash
#!/bin/bash
# collect-update-status.sh

for machine in $MACHINES; do
    ssh user@$machine "cat ~/.cco/logs/updates.log" >> all-updates.log
done

# Analyze
grep "successfully installed" all-updates.log | wc -l  # Count successful updates
grep "ERROR" all-updates.log | wc -l                   # Count failures
tail -1 all-updates.log | grep "$(date +%Y-%m-%d)"    # Today's updates
```

### Monitor Log File

Check individual machine logs:

```bash
# View recent updates
tail -50 ~/.cco/logs/updates.log

# Monitor in real-time
tail -f ~/.cco/logs/updates.log

# Search for errors
grep "ERROR\|WARN" ~/.cco/logs/updates.log

# Analyze patterns
grep "Downloaded" ~/.cco/logs/updates.log | wc -l  # How many downloads
grep "successfully installed" ~/.cco/logs/updates.log | wc -l  # Successes
```

### Understanding Log Entries

Each successful update creates detailed logs:

```
[2025-11-17 14:32:15] Check started for stable channel
    ↑ Initial check triggered

[2025-11-17 14:32:18] Update available: 2025.11.3 (current: 2025.11.2)
    ↑ New version found

[2025-11-17 14:32:45] Downloaded 12.5 MB from GitHub
    ↑ Binary retrieved successfully

[2025-11-17 14:32:46] Verified checksum: SHA256 match
    ↑ Security check passed

[2025-11-17 14:32:47] Created backup: ~/.local/bin/cco.backup
    ↑ Rollback protection enabled

[2025-11-17 14:32:48] Verified new binary works
    ↑ Functionality verification passed

[2025-11-17 14:32:49] Successfully installed 2025.11.3
    ↑ Installation complete

[2025-11-17 14:32:50] Cleanup: Removed temporary files
    ↑ Temporary files cleaned up
```

### Detect Failed Updates

```bash
# Find failed updates in logs
grep "ERROR" ~/.cco/logs/updates.log

# Common error patterns:
grep "Checksum verification failed" ~/.cco/logs/updates.log
grep "Permission denied" ~/.cco/logs/updates.log
grep "Network error\|Connection failed" ~/.cco/logs/updates.log
grep "Verification of new binary failed" ~/.cco/logs/updates.log
```

## Troubleshooting Failed Updates

### Checksum Verification Failed

**Cause**: Downloaded file corrupted or network issue

**Solution**:

```bash
# Option 1: Wait and retry (usually resolves network issues)
# Auto-update will retry on next scheduled check

# Option 2: Force immediate retry
ssh user@machine "cco update --yes"

# Option 3: Check network quality
ping github.com
curl -I https://api.github.com

# Option 4: Check logs for pattern
grep "Checksum verification failed" ~/.cco/logs/updates.log | wc -l
```

### Permission Denied During Update

**Cause**: CCO binary location not writable

**Solution**:

```bash
# Check permissions
ls -la ~/.local/bin/cco
# Should be: -rwxr-xr-x (owner can write)

# Fix permissions (on affected machine)
chmod 755 ~/.local/bin/
chmod 755 ~/.local/bin/cco

# Or reinstall to user directory
cd ~/
wget https://github.com/yourusername/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz
tar xzf cco-v2025.11.3-linux-x86_64.tar.gz
mv cco ~/.local/bin/cco
```

### Verification of New Binary Failed

**Cause**: Downloaded binary is corrupted or incompatible

**Solution**:

```bash
# Check for platform compatibility
uname -a  # Get system info
cco --version  # Verify current works

# Check logs for details
tail -100 ~/.cco/logs/updates.log

# Possible platform mismatch:
# - Downloaded ARM64 binary on x86_64 system
# - Downloaded Linux binary on macOS
# - Downloaded 32-bit on 64-bit system

# Manual fix:
# 1. Download correct binary manually
# 2. Reinstall to ~/.local/bin/cco
```

### Network Error - No Internet Connectivity

**Cause**: Machine is offline or cannot reach GitHub

**Solution**:

```bash
# Verify connectivity
ping github.com  # Can you reach GitHub?
curl -I https://api.github.com  # Can you connect?

# For offline machines:
# Option 1: Wait for connectivity restoration
#           Auto-update retries on next scheduled check

# Option 2: Manual update when online
#           ssh user@machine "cco update --yes"

# Option 3: Disable auto-updates for offline machines
#           cco config set updates.enabled false
#           Update manually when online
```

## Staged Rollout Strategy

### Example: Enterprise Deployment

Manage updates across your organization systematically:

#### Week 1: Pilot Group (5 machines)

```bash
# Canaries: Internal engineering team
# Purpose: Early feedback on new releases

cco config set updates.channel beta
cco config set updates.auto_install true
cco config set updates.check_interval daily

# Monitor closely
tail -f ~/.cco/logs/updates.log
```

#### Week 2: Beta Group (25 machines)

```bash
# After 1 week of successful pilot testing
# Expand to wider beta testing

cco config set updates.channel beta
cco config set updates.auto_install false  # Manual confirmation

# Users see prompts for updates
# IT reviews logs daily
```

#### Week 3: General Rollout (100+ machines)

```bash
# After 2 weeks of testing
# General release to all machines

cco config set updates.channel stable
cco config set updates.auto_install true

# Full automation
# Automated monitoring
```

### Monitoring Checklist

```
Daily:
☐ Check error count in logs
☐ Verify successful update count
☐ Monitor network usage patterns
☐ Review user support tickets

Weekly:
☐ Analyze update performance metrics
☐ Identify problematic machines
☐ Plan remediation for failures
☐ Communicate status to stakeholders

Monthly:
☐ Comprehensive audit of all machines
☐ Review update distribution patterns
☐ Assess version coverage
☐ Plan next month's strategy
```

## Rollback Strategy

### Automatic Rollback (Automatic)

If an update fails, auto-update automatically rolls back:

```bash
# User typically sees:
# [ERROR] Verification of new binary failed
# [INFO] Rolling back to previous version
# [INFO] Previous version restored
```

### Manual Rollback (If Needed)

```bash
# Restore from backup
mv ~/.local/bin/cco.backup ~/.local/bin/cco

# Verify it works
cco --version

# Report the issue
# Include: ~/.cco/logs/updates.log
```

### Organizational Rollback

For organization-wide rollback:

```bash
# 1. Disable auto-updates
for machine in $MACHINES; do
    ssh user@$machine "cco config set updates.enabled false"
done

# 2. Restore previous version on all machines
for machine in $MACHINES; do
    ssh user@$machine "mv ~/.local/bin/cco.backup ~/.local/bin/cco"
done

# 3. Verify restoration
for machine in $MACHINES; do
    ssh user@$machine "cco --version"
done

# 4. Investigate root cause
# 5. Fix and re-enable when ready
```

## Performance Considerations

### Network Impact

```
Check operation: ~50 KB/request
Download operation: ~12-50 MB (one-time per update)
Daily recurring: ~50 KB (if no new version)

Optimization:
- Checks happen daily (not hourly)
- Downloads only for new versions
- Minimal bandwidth for stable environments
```

### CPU Impact

```
Background operations have minimal impact:
- Negligible during check phase
- Parallel with other work during download
- Brief CPU spike during verification (~1-2 seconds)

No impact on:
- User interactive work
- Development environments
- Production services
```

### Disk Space

```
Required disk space:
- Temporary files: 5-10 MB during update
- Backup copy: ~15-30 MB (binary size)
- Logs: ~10-50 KB per month

Total: ~25-50 MB for a complete setup
```

## Compliance and Auditing

### Audit Trail

All updates are logged for compliance:

```bash
# Export update audit trail
grep "successfully installed" ~/.cco/logs/updates.log > audit.log

# Format: [timestamp] Version installed
# Example: [2025-11-17 14:32:49] Successfully installed 2025.11.3
```

### Compliance Reporting

For compliance and auditing:

```bash
# Generate update report
cat > update-report.sh << 'EOF'
#!/bin/bash
echo "CCO Update Report: $(date)"
echo "================="

for machine in $MACHINES; do
    echo ""
    echo "Machine: $machine"
    version=$(ssh user@$machine "cco --version" 2>/dev/null || echo "ERROR")
    echo "Current version: $version"

    updates=$(ssh user@$machine "grep 'successfully installed' ~/.cco/logs/updates.log 2>/dev/null | wc -l" || echo "0")
    echo "Updates installed: $updates"

    errors=$(ssh user@$machine "grep 'ERROR' ~/.cco/logs/updates.log 2>/dev/null | wc -l" || echo "0")
    echo "Update errors: $errors"
done
EOF

chmod +x update-report.sh
./update-report.sh
```

## Best Practices

### For Administrators

1. **Start with conservative settings**
   - Begin with `check_interval = "weekly"`
   - Require manual confirmation (`auto_install = false`)
   - Observe for 2-4 weeks

2. **Monitor closely**
   - Review logs daily
   - Track error patterns
   - Watch for version coverage

3. **Communicate with users**
   - Announce major updates
   - Explain security patches
   - Provide support for issues

4. **Plan staged rollouts**
   - Pilot with small group
   - Beta test with larger group
   - Full rollout after validation

5. **Have rollback plan**
   - Know how to restore backups
   - Test rollback procedure
   - Document recovery steps

### For Users Under Management

If your organization manages updates:

1. **Don't disable updates** without approval
   - Updates provide security patches
   - Discuss concerns with IT

2. **Report issues early**
   - Document problems in logs
   - Contact IT support quickly
   - Provide system information

3. **Test in non-critical environments first**
   - If updates are optional
   - Ensure compatibility
   - Verify functionality

4. **Keep logs for auditing**
   - Don't delete ~/.cco/logs/updates.log
   - Archive for compliance
   - Use for troubleshooting

## Deployment Checklist

Use this checklist for organization-wide deployment:

```
Preparation Phase:
☐ Read this guide completely
☐ Test on pilot machines
☐ Define update policy
☐ Plan rollout schedule

Rollout Phase:
☐ Deploy to pilot group (5-10 machines)
☐ Monitor for 1 week
☐ Address any issues
☐ Deploy to beta group (25-50 machines)
☐ Monitor for 1 week
☐ Deploy to all machines

Post-Deployment Phase:
☐ Establish monitoring process
☐ Define SLAs for updates
☐ Create escalation procedures
☐ Document custom policies
☐ Train support staff
☐ Schedule regular reviews

Ongoing Phase:
☐ Daily log review
☐ Weekly error analysis
☐ Monthly comprehensive audit
☐ Quarterly policy review
☐ Annual security assessment
```

## Support and Escalation

### When to Contact Support

- Consistent checksum verification failures
- Recurring permission errors
- More than 10% of machines failing
- Unknown error messages in logs
- Security vulnerability concerns

### Support Information to Provide

```bash
# Gather this information for support:
echo "=== System Information ==="
uname -a
cco --version

echo "=== Configuration ==="
cco config show

echo "=== Recent Logs ==="
tail -100 ~/.cco/logs/updates.log

echo "=== Network Status ==="
ping -c 4 github.com
curl -I https://api.github.com

echo "=== File Permissions ==="
ls -la ~/.local/bin/cco*
ls -la ~/.config/cco/
ls -la ~/.cco/
```

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md) - End-user documentation
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md) - Security features
- [Auto-Update Architecture](AUTO_UPDATE_ARCHITECTURE.md) - Technical details
- [Auto-Update Command Reference](AUTO_UPDATE_COMMAND_REFERENCE.md) - All commands
- [Auto-Update Troubleshooting (Advanced)](AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) - Advanced debugging
