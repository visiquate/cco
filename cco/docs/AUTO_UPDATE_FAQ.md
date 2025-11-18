# Auto-Update FAQ (Frequently Asked Questions)

## General Questions

### Do I need to do anything for updates?

**No.** Auto-updates are enabled by default and happen automatically. You don't need to take any action—CCO will check daily and install updates in the background.

### Will updates interrupt my work?

**No.** Updates happen in the background and don't interrupt your work. Most updates take effect immediately. If a restart is needed, you'll be notified.

### Can I see what changed in each update?

**Yes.** Check the update logs:

```bash
# View update history
tail -50 ~/.cco/logs/updates.log

# View release notes on GitHub
https://github.com/yourusername/cco/releases

# View what changed in a specific version
cco update --show-changes 2025.11.3
```

### How often does CCO check for updates?

**Daily by default.** You can adjust the frequency:

```bash
cco config set updates.check_interval daily    # Every day (default)
cco config set updates.check_interval weekly   # Every week
cco config set updates.check_interval never    # Never check
```

### What if my internet is offline?

Updates are skipped if you're offline. When you reconnect, CCO will check and install any pending updates on the next scheduled check.

### Can updates be scheduled for a specific time?

**Not yet.** Currently, checks happen daily at a consistent time. Custom scheduling is planned for a future release.

---

## Configuration and Control

### How do I disable auto-updates?

If you prefer manual updates:

```bash
# Disable all update checks
cco config set updates.enabled false

# Verify it's disabled
cco config show
```

### Can I be notified about updates without auto-installing?

**Yes.** Allow checking but require manual confirmation:

```bash
# Still check daily, but prompt before installing
cco config set updates.auto_install false

# When an update is available, you'll be prompted:
# "Update available: 2025.11.3. Install now? (y/n)"
```

### Can I disable updates just for today?

**Yes, temporarily:**

```bash
# Disable for this session
export CCO_UPDATES_ENABLED=false
cco

# Re-enable for next session
unset CCO_UPDATES_ENABLED
```

### Can I switch between stable and beta updates?

**Yes.** By default you get stable releases:

```bash
# Stay with stable releases (recommended)
cco config set updates.channel stable

# Try beta/pre-release updates
cco config set updates.channel beta
```

### How do I view my current configuration?

```bash
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

---

## Manual Updates

### How do I manually check for updates?

```bash
cco update --check
# Output: CCO 2025.11.3 is available (you have 2025.11.2)
```

### How do I manually install updates?

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

### What's my current version?

```bash
cco --version
# Output: cco 2025.11.2 (commit: abc123)
```

---

## Security and Safety

### Are my credentials safe during updates?

**Yes, absolutely.** Configuration files and credentials are never modified or accessed during updates. Only the CCO binary is replaced.

### How do I know updates are safe?

**CCO implements 12 security features:**

1. HTTPS-only connections (no unencrypted downloads)
2. SHA256 checksum verification
3. Secure temporary file handling
4. Release tag validation
5. Binary verification before installation
6. Atomic replacement operations
7. Automatic backup creation
8. GitHub repository verification
9. Update channel isolation
10. Checksum file validation
11. Network retry logic
12. Secure update logging

See [Auto-Update Security](AUTO_UPDATE_SECURITY.md) for details.

### What if update verification fails?

If verification fails, the update is aborted:

```bash
# Example error:
[ERROR] Checksum verification failed! Update aborted for security.
```

**This is good—it means the security check worked.** Try again:

```bash
cco update --yes
```

### How are credentials protected during updates?

- Configuration files remain untouched
- Environment variables with credentials are not read
- Credentials are never stored in temporary files
- API keys are not logged
- Credential storage is never modified

### Can I manually verify downloaded binaries?

**Yes.** Download and verify manually:

```bash
VERSION=2025.11.3
URL=https://github.com/yourusername/cco/releases/download/v${VERSION}

# Download binary and checksums
wget ${URL}/cco-v${VERSION}-linux-x86_64.tar.gz
wget ${URL}/checksums.sha256

# Verify
sha256sum -c checksums.sha256
# Should show: OK
```

---

## Data and Files

### What gets updated?

**Updated:**
- CCO executable program
- Built-in agents and models
- Core functionality and APIs
- Performance improvements
- Security patches

**Not changed:**
- Configuration files (~/.config/cco/config.toml)
- Credentials and API keys
- Local cache and databases
- Project files and code
- User settings

### Will updates affect my projects or files?

**No.** Updates only replace the CCO binary. Your projects, code, and data are never touched.

### What happens to my configuration file?

Configuration files are preserved during updates:

```bash
# Before update: ~/.config/cco/config.toml (saved)
# During update: only binary replaced
# After update: ~/.config/cco/config.toml (unchanged)
```

### Where are update logs stored?

```bash
# Log file location
~/.cco/logs/updates.log

# View logs
cat ~/.cco/logs/updates.log

# View recent entries
tail -50 ~/.cco/logs/updates.log
```

---

## Troubleshooting

### Updates aren't installing

**Problem:** You haven't seen any updates for a while

**Solutions:**

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

### Permission denied error

**Problem:** Update fails with "Permission denied"

**Solutions:**

```bash
# Option 1: Check directory permissions
ls -la ~/.local/bin/ | grep cco

# Option 2: Fix permissions
chmod 755 ~/.local/bin/
chmod 755 ~/.local/bin/cco

# Option 3: Reinstall to user directory
cd ~/ && rm -f ~/.local/bin/cco
wget https://github.com/yourusername/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz
tar xzf cco-v2025.11.3-linux-x86_64.tar.gz -C ~/.local/bin/
```

### Checksum verification failed

**Problem:** "Checksum verification failed! Update aborted."

**Explanation:** The downloaded file doesn't match the expected hash.

**Solution:**

```bash
# Wait and retry - network issues often resolve
cco update --yes

# If it persists, check your internet
ping github.com
curl -I https://api.github.com

# Report if it continues persistently
# Visit: https://github.com/yourusername/cco/security/advisories
```

### Rollback after failed update

If an update causes problems:

```bash
# Restore the previous version
mv ~/.local/bin/cco.backup ~/.local/bin/cco

# Verify it works
cco --version

# Report the issue
```

### Update checking seems slow

**Problem:** `cco update --check` takes longer than expected

**Explanation:**
- First check may download release information
- Network conditions affect speed
- GitHub API rate limits may apply

**Solution:**

```bash
# Checks are cached, subsequent checks are faster
# Run again to see cached response

# If internet is limited, disable checks
cco config set updates.check_interval never

# Or increase interval
cco config set updates.check_interval weekly
```

### Can't see the update in the logs

**Problem:** No update entries in logs

**Solutions:**

```bash
# Check if log file exists
ls -la ~/.cco/logs/updates.log

# Force a check
cco update --check

# View logs
tail -20 ~/.cco/logs/updates.log

# If still empty, updates may be disabled
cco config show | grep enabled
```

---

## Network and Environment

### What if I'm on a restricted network?

You can disable update checks:

```bash
cco config set updates.enabled false
```

Then update manually when you have internet access.

### How much bandwidth does auto-update use?

```
Daily check: ~50 KB (if no update available)
When update available: ~10-50 MB download
   (depends on platform and binary size)

Optimization: Downloads only happen when newer version is available
```

### Can I use a proxy for updates?

**Currently:** No explicit proxy support (uses system network settings)

**Workaround:** System proxy settings are respected:

```bash
# On Linux/macOS, set system proxy
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
cco update --check
```

---

## Version and Compatibility

### What version format does CCO use?

CCO uses date-based versioning: **YYYY.MM.N**

- **YYYY**: Year
- **MM**: Month (1-12)
- **N**: Release counter for that month (resets each month)

Examples:
- `2025.11.1` - First release in November 2025
- `2025.11.2` - Second release in November 2025
- `2025.12.1` - First release in December 2025

### Can I downgrade to an older version?

**No.** Auto-update prevents downgrades—only newer versions can be installed. This is a security feature.

If you need a previous version, download it manually from GitHub releases.

### Can I skip an update and wait for the next one?

**Yes.** By default, you can manage updates with:

```bash
# Disable for now
export CCO_UPDATES_ENABLED=false

# Re-enable later
unset CCO_UPDATES_ENABLED

# Or use configuration
cco config set updates.enabled false
cco config set updates.enabled true  # When ready
```

### What platforms are supported?

- **macOS**: arm64 (Apple Silicon), x86_64 (Intel)
- **Linux**: x86_64 (AMD64), aarch64 (ARM64)
- **Windows**: x86_64 (AMD64)

Auto-update detects your platform and downloads the correct binary.

---

## Beta and Testing

### What's the difference between stable and beta?

**Stable:**
- Fully tested releases
- Recommended for all users
- Low risk of issues

**Beta:**
- Pre-release versions
- More recent features
- May contain experimental code
- Helpful for early feedback

### Should I use beta updates?

**Beta is recommended for:**
- Power users who want latest features
- Early feedback for new functionality
- Testing in non-critical environments

**Stable is recommended for:**
- Production environments
- Critical work
- Most users

### How do I switch to beta?

```bash
cco config set updates.channel beta
cco update --check  # Will show beta versions
```

### Can I switch back to stable?

**Yes:**

```bash
cco config set updates.channel stable
cco update --check  # Will only show stable releases
```

---

## Advanced Questions

### How does auto-update handle concurrent instances?

CCO uses file locking to ensure only one update happens at a time. If one instance is updating, others wait safely.

### What happens if CCO crashes during update?

**Safe design prevents problems:**
- Backup created before replacement
- Atomic operations ensure consistency
- On failure, previous version is restored

### Can I manually edit the configuration file?

**Yes,** but use commands when possible:

```bash
# Recommended (safe)
cco config set updates.enabled false

# Also works (but less safe)
nano ~/.config/cco/config.toml
```

### How do I export/import configuration?

**For advanced users:**

```bash
# Export current configuration
cco config export > cco-config.toml

# Import on another machine
cco config import cco-config.toml
```

### What logging levels are available?

By default, updates are logged with:
- **INFO**: Normal operations
- **WARN**: Recoverable issues
- **ERROR**: Update failures

View full logs:

```bash
cat ~/.cco/logs/updates.log
```

---

## When to Contact Support

Contact support if you experience:

1. Consistent checksum verification failures
2. Recurring permission errors
3. Update checks never completing
4. Unknown error messages
5. Security concerns

**Include when reporting:**
- Your CCO version: `cco --version`
- Your platform: `uname -a`
- Recent logs: `tail -100 ~/.cco/logs/updates.log`
- Error messages

---

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md) - Complete user guide
- [Auto-Update Security](AUTO_UPDATE_SECURITY.md) - Security features and fixes
- [Auto-Update Administrator Guide](AUTO_UPDATE_ADMIN_GUIDE.md) - Admin/IT documentation
- [Auto-Update Architecture](AUTO_UPDATE_ARCHITECTURE.md) - Technical architecture
- [Auto-Update Troubleshooting (Advanced)](AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) - Advanced debugging
- [Auto-Update Command Reference](AUTO_UPDATE_COMMAND_REFERENCE.md) - All commands

---

**Can't find your answer?** Check the full documentation or open an issue on GitHub.
