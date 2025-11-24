# Auto-Update Migration Guide

## For Existing CCO Users

This guide helps you transition to the new auto-update system with auto-install enabled by default.

## What's Changing

### Before (Old Behavior)

Previously, CCO had manual updates only:

```bash
# Users had to manually check and install
cco update --check
cco update --yes
```

### After (New Behavior)

Now, CCO automatically checks and installs updates:

- **Automatic daily checks** - CCO checks for updates every 24 hours
- **Automatic installation** - Critical patches install in the background
- **Zero intervention** - No user action needed
- **Rollback protection** - Previous version preserved in case of problems

## What Gets Updated

### Auto-Updated (When New Version Available)

- CCO executable program
- Built-in agents and models
- Core functionality and APIs
- Security patches
- Performance improvements

### NOT Updated (Preserved)

- Configuration files
- Credentials and API keys
- Project files and code
- User settings
- Cached data

## Getting Started

### Option 1: Default (Recommended)

Accept the new automatic updates. They're enabled by default:

```bash
# Nothing to do!
# Updates will happen automatically starting tomorrow
```

**With defaults:**
- Updates check daily
- Auto-install enabled
- Stable releases only
- Fully automatic operation

### Option 2: Keep Manual Control

If you prefer to control when updates happen:

```bash
# Disable auto-install but allow checking
cco config set updates.auto_install false

# You'll see prompts when updates are available:
# "Update available: 2025.11.3. Install now? (y/n)"
```

### Option 3: Completely Manual

If you want to manage all updates manually:

```bash
# Disable automatic checks entirely
cco config set updates.enabled false
cco config set updates.check_interval never

# Update only when you're ready
cco update --yes
```

## Understanding the New Experience

### What You'll See (With Auto-Install Enabled)

**Nothing special.** Updates happen silently in the background.

To verify updates are working:

```bash
# Check current version
cco --version

# View update history
tail -20 ~/.cco/logs/updates.log
```

### What You'll See (With Auto-Install Disabled)

When an update is available, you'll see a prompt:

```
Update available: 2025.11.3 (you have 2025.11.2)
Continue with update? (y/n)
```

Press `y` to install now or `n` to skip.

### What You'll See (With Checks Disabled)

Nothing happens unless you manually check:

```bash
cco update --check   # Only when you run this
cco update --yes     # Only when you run this
```

## Monitoring Updates

### Check Your Current Configuration

```bash
cco config show

# Look for updates section:
# [updates]
# enabled = true
# auto_install = true
# check_interval = "daily"
# channel = "stable"
```

### View Update History

```bash
# See what's been updated
tail -50 ~/.cco/logs/updates.log

# Search for successful updates
grep "successfully installed" ~/.cco/logs/updates.log

# Find any errors
grep "ERROR" ~/.cco/logs/updates.log
```

### Check Last Update Time

```bash
# When was last check?
grep "Check started" ~/.cco/logs/updates.log | tail -1

# When was last successful install?
grep "successfully installed" ~/.cco/logs/updates.log | tail -1
```

## Troubleshooting Migration Issues

### "I haven't seen any updates"

**Problem:** Updates aren't happening as expected

**Solution:**

```bash
# Check if updates are enabled
cco config show | grep enabled
# Should show: enabled = true

# Check last check time
tail -5 ~/.cco/logs/updates.log
# Should have recent entries

# Force a check
cco update --check
# Should show current version or available update

# Check internet connectivity
ping github.com
```

### "I don't want auto-update"

**Solution:**

```bash
# Option 1: Disable auto-install but keep checking
cco config set updates.auto_install false

# Option 2: Disable everything
cco config set updates.enabled false

# Verify it's disabled
cco config show
```

### "Updates are failing"

**Solution:**

```bash
# View error logs
grep "ERROR" ~/.cco/logs/updates.log

# Common errors:

# 1. Permission denied
# Fix: chmod 755 ~/.local/bin/

# 2. Checksum verification failed
# Fix: Wait and retry (network issue usually)
# cco update --yes

# 3. Network error
# Fix: Check internet connection
# ping github.com

# 4. Disk full
# Fix: Free up disk space
# du -sh ~/.local/
```

### "Backup file keeps growing"

**Problem:** `~/.local/bin/cco.backup` exists from previous updates

**Solution:**

```bash
# This is normal - it's kept for rollback
# It's fine to delete if you want to save space
rm ~/.local/bin/cco.backup

# A new backup will be created on next update
```

## Migration Checklist

Use this to ensure smooth migration:

```
[ ] Read this migration guide
[ ] Check current version: cco --version
[ ] View current config: cco config show
[ ] Choose your preferred update behavior
    [ ] Automatic (recommended) - no action needed
    [ ] Manual confirmation - cco config set updates.auto_install false
    [ ] Manual only - cco config set updates.enabled false
[ ] Monitor logs for first 24 hours
    [ ] Check: tail -20 ~/.cco/logs/updates.log
[ ] Report any issues
    [ ] Note: System information, error logs, commands tried
```

## Frequently Asked Migration Questions

### Q: Will auto-updates happen while I'm working?

**A:** Updates happen in the background and won't interrupt your work. Most take effect immediately; some may need a restart.

### Q: What if I don't want updates?

**A:** You can disable them:
```bash
cco config set updates.enabled false
```

### Q: Can I schedule when updates happen?

**A:** Currently no, but it's planned for a future release. For now:
- Daily check happens at a consistent time
- You can disable auto-install and choose when to install

### Q: Will my configuration be preserved?

**A:** Yes, absolutely. Only the CCO binary is updated.

### Q: Can I see what changed in each update?

**A:** Yes:
```bash
tail -50 ~/.cco/logs/updates.log
# Or view GitHub releases:
https://github.com/yourusername/cco/releases
```

### Q: What if an update breaks something?

**A:** The previous version is automatically backed up:
```bash
mv ~/.local/bin/cco.backup ~/.local/bin/cco
cco --version  # Should work again
```

### Q: Do I need to manually update to get security patches?

**A:** No. With auto-update enabled, you automatically get patches.

### Q: Can I choose to get only security updates, not feature updates?

**A:** Not yet. All releases include both. This is planned for a future release.

### Q: What happens if internet is offline during an update check?

**A:** Nothing - the check is skipped. When you reconnect, the next check will happen and any pending updates will install.

### Q: Can I see all versions available?

**A:** Yes, on GitHub:
```bash
# View all releases
https://github.com/yourusername/cco/releases

# Or via API
curl https://api.github.com/repos/yourusername/cco/releases
```

## For Organized Teams

If you manage CCO deployments across multiple machines:

### Staged Rollout (Recommended)

1. **Week 1: Pilot Group (5-10 machines)**
   - Enable auto-update with monitoring
   - Check logs daily
   - Report issues early

2. **Week 2: Beta Group (25-50 machines)**
   - After successful pilot
   - Require manual confirmation
   - Monitor and adjust

3. **Week 3: Full Rollout (all machines)**
   - After successful beta testing
   - Full auto-install enabled
   - Automated monitoring

### Team Configuration

Distribute this configuration file:

```toml
# cco-config-team.toml
[updates]
enabled = true
auto_install = false  # Require confirmation initially
check_interval = "daily"
channel = "stable"
```

Apply across team:

```bash
# For each machine:
ssh user@machine "cco config import ~/cco-config-team.toml"
```

### Team Monitoring

```bash
#!/bin/bash
# monitor-updates.sh

for machine in $MACHINES; do
    echo "=== $machine ==="

    # Current version
    ssh user@machine "cco --version"

    # Update status
    ssh user@machine "tail -1 ~/.cco/logs/updates.log"

    # Error count
    ssh user@machine "grep 'ERROR' ~/.cco/logs/updates.log | wc -l"
done
```

## Reverting to Manual Updates

If you need to go back to manual updates:

```bash
# Disable all automatic updates
cco config set updates.enabled false

# Manually update when needed
cco update --yes

# Re-enable when ready
cco config set updates.enabled true
```

## Support and Help

If you encounter issues during migration:

1. **Check the logs**: `cat ~/.cco/logs/updates.log`
2. **Read the FAQ**: See AUTO_UPDATE_FAQ.md
3. **Try troubleshooting**: See AUTO_UPDATE_USER_GUIDE.md troubleshooting section
4. **Report issues**: GitHub issues with logs and system info

## Summary

**The new auto-update system:**

- ✅ Automatically checks for updates daily
- ✅ Automatically installs critical patches
- ✅ Preserves your configuration and credentials
- ✅ Keeps previous version as backup
- ✅ Logs all operations for your review
- ✅ Can be disabled if you prefer manual updates

**You can:**

- Accept automatic updates (recommended)
- Require confirmation before installing
- Disable automatic checks and update manually
- Switch channels between stable and beta
- Override with environment variables

**Your data is:**

- Safe - only binary is replaced
- Backed up - previous version preserved
- Logged - all operations recorded
- Recoverable - easy rollback if needed

**Next Steps:**

1. Choose your preferred update behavior
2. Monitor the first 24 hours
3. Report any issues
4. Enjoy automatic security updates!

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md) - Complete user guide
- [Auto-Update FAQ](AUTO_UPDATE_FAQ.md) - Common questions
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md) - Security features
- [Auto-Update Administrator Guide](AUTO_UPDATE_ADMIN_GUIDE.md) - For teams/IT
