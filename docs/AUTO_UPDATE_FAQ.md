# Auto-Update FAQ

Frequently asked questions about CCO's auto-update system.

## General Questions

### Q: How often are CCO updates released?

**A:** Release frequency varies based on needs:
- **Stable channel**: Typically weekly or bi-weekly with tested features
- **Beta channel**: Multiple times per week with experimental features
- **Hotfixes**: Emergency releases when critical bugs are found

Subscribe to releases at: https://github.com/brentley/cco-releases/releases

### Q: What time are updates released?

**A:** Releases happen on Tuesdays at 2 PM UTC. Version checks are most active immediately after, but new versions are available anytime.

### Q: How do I know what changed in a version?

**A:** When you run `cco update`, release notes are displayed before installation. You can also view:
- Full release notes online: https://github.com/brentley/cco-releases/releases
- Recent versions: `cco update --check`

### Q: Can I see version history?

**A:** All versions are available at:
https://github.com/brentley/cco-releases/releases

Filter by channel:
- **Stable**: Regular releases, fully tested
- **Beta**: Pre-release versions with new features

### Q: Is there a changelog?

**A:** Release notes are in the GitHub releases page. Each release includes:
- New features
- Bug fixes
- Performance improvements
- Breaking changes (if any)
- Migration guides (if needed)

## Configuration Questions

### Q: Where is the configuration stored?

**A:** Configuration is stored in your home directory:
- **macOS/Linux**: `~/.config/cco/config.toml`
- **Windows**: `%AppData%\cco\config.toml`

You can edit this file directly with any text editor.

### Q: Can I backup my configuration?

**A:** Yes, backup the config file:

```bash
# Backup
cp ~/.config/cco/config.toml ~/.config/cco/config.toml.backup

# Restore
cp ~/.config/cco/config.toml.backup ~/.config/cco/config.toml
```

The configuration is TOML format, so it's human-readable and versionable.

### Q: What happens to my settings when I update?

**A:** Your settings are never modified by updates. Configuration is stored separately and persists across all updates.

### Q: Can I share the config file across multiple machines?

**A:** Yes, the config file is portable:

```bash
# Copy to another machine
scp ~/.config/cco/config.toml user@remote:~/.config/cco/
```

Settings apply to that machine's CCO installation.

### Q: How do I reset to default configuration?

**A:** Remove the config file:

```bash
rm ~/.config/cco/config.toml

# Next run will recreate with defaults
cco version
```

Or manually set defaults:

```bash
cco config set updates.enabled true
cco config set updates.auto_install false
cco config set updates.check_interval daily
cco config set updates.channel stable
```

## Update Process Questions

### Q: Can updates interrupt my work?

**A:** It depends on how updates are configured:

- **Auto-install disabled** (default): Updates only when you run `cco update`
- **Auto-install enabled**: Updates run in background, but don't interrupt running services
- **Running daemon**: Service continues working during update, restarts afterward

If you're using CCO as a daemon, restart after update:
```bash
cco daemon restart
```

### Q: What happens to running services during update?

**A:** Running services are not interrupted:

1. Update process downloads and verifies new binary
2. Old binary is backed up
3. New binary replaces old one
4. New binary is tested
5. Daemon continues using old binary until restarted
6. Restart to use new version:
```bash
cco daemon restart
```

### Q: How long does an update take?

**A:** Typical timing:
- Check only: 2-5 seconds
- Download: 5-30 seconds (varies by network)
- Verification: 2-5 seconds
- Installation: 1-5 seconds
- **Total: 10-45 seconds**

You can continue working during download and installation.

### Q: Do I need internet for updates?

**A:** Yes, updates require:
- Outbound HTTPS (port 443) to GitHub
- ~50-100 MB bandwidth per update

If behind a proxy:
```bash
export HTTPS_PROXY="http://proxy:8080"
cco update --yes
```

### Q: What if my internet cuts out during update?

**A:** Safe behavior:
- **Before backup**: Nothing is changed, try again
- **Before installation**: Temp files are cleaned up, try again
- **During installation**: New binary is tested, rollback occurs if it fails
- **After installation**: You're left with previous working version

Just run `cco update` again.

### Q: Can I update multiple machines at once?

**A:** Yes, but stagger the updates to avoid issues:

```bash
# Update one at a time with 10-minute intervals
for server in prod-1 prod-2 prod-3; do
    ssh $server cco update --yes
    sleep 600  # Wait 10 minutes
done
```

Or configure scheduled updates:
```bash
# prod-1: 2 AM
# prod-2: 2:10 AM
# prod-3: 2:20 AM
```

### Q: Can I update in the background while working?

**A:** Yes, if auto-install is enabled:

```bash
cco config set updates.auto_install true
```

Then:
- Updates check in background
- Download happens if new version is available
- Installation happens automatically
- You see notification but don't have to do anything
- Restart daemon when convenient:
```bash
cco daemon restart
```

### Q: What's the difference between --check and --yes flags?

**A:**

| Flag | Behavior |
|------|----------|
| None | Check for updates, show release notes, prompt to confirm |
| `--check` | Check only, don't download or install |
| `--yes` | Check and install without prompting |
| `--check --yes` | Check only (--yes ignored with --check) |

**Examples:**

```bash
# Interactive (shows notes, asks confirmation)
cco update

# Check only
cco update --check

# Auto-install (good for scripts/cron)
cco update --yes

# Use specific channel
cco update --channel beta
cco update --yes --channel beta
```

## Channel Questions

### Q: What's the difference between stable and beta?

**A:**

| Aspect | Stable | Beta |
|--------|--------|------|
| Release frequency | Weekly or less | Multiple times/week |
| Testing | Fully tested | Minimal testing |
| New features | Settled and proven | Experimental |
| Reliability | Production-ready | May have issues |
| Recommended for | Production | Development/testing |

### Q: How do I switch channels?

**A:**

```bash
# Switch to stable (default)
cco config set updates.channel stable

# Switch to beta
cco config set updates.channel beta
```

Verify the change:
```bash
cco config get updates.channel
```

### Q: Can I use beta channel in production?

**A:** Not recommended. Beta releases:
- Are not fully tested
- May have breaking changes
- May have bugs

Use only for development/testing. Switch back to stable for production:
```bash
cco config set updates.channel stable
```

### Q: What's the difference between beta and latest?

**A:** There is no "latest" channel. Available channels:
- **stable**: Regular releases, recommended
- **beta**: Pre-releases, experimental

### Q: How do I report bugs in beta versions?

**A:** Report issues at:
https://github.com/brentley/cco-releases/issues

Include:
- CCO version: `cco version`
- Your system: `uname -a`
- Error message: Full output from failed command
- Steps to reproduce: How to trigger the bug

## Troubleshooting Questions

### Q: How do I fix "Update check never completes"?

**A:** The update check is hanging. Try:

1. **Check GitHub status**:
   - https://www.githubstatus.com/
   - Is GitHub having issues?

2. **Check your internet**:
   ```bash
   ping github.com
   curl -I https://api.github.com
   ```

3. **Disable auto-checks temporarily**:
   ```bash
   cco config set updates.enabled false
   ```

4. **Try manual check**:
   ```bash
   cco update --check
   ```

5. **Check for proxy issues**:
   ```bash
   # If behind corporate proxy
   export HTTPS_PROXY="http://proxy:8080"
   cco update --check
   ```

### Q: What does "Permission denied" mean?

**A:** The installation directory is not writable. Fix it:

```bash
# Create directory with proper permissions
mkdir -p ~/.local/bin
chmod 755 ~/.local/bin

# Ensure you own it
chown $(whoami) ~/.local/bin

# Try update again
cco update --yes
```

### Q: What does "Checksum verification failed" mean?

**A:** The downloaded file is corrupted. This is a safety feature!

```bash
# The update was NOT installed
# Try again - file will be re-downloaded
cco update --yes
```

If it keeps failing:
1. Check your internet connection
2. Try from a different network
3. Download manually from releases page

### Q: How do I recover from a failed update?

**A:** If the new binary didn't work, previous version is restored:

```bash
# Check which version is running
cco version

# If still on old version, all is good
# If on new version but failing, restore backup:
cp ~/.local/bin/cco.backup ~/.local/bin/cco
```

### Q: How do I report update issues?

**A:** Provide detailed information:

```bash
# System information
uname -a
uname -m  # Architecture

# Current version
cco version

# Update attempt with debug output
RUST_LOG=debug cco update --yes 2>&1 | tee update.log

# Available disk space
df -h ~/.local/bin
```

Report at: https://github.com/brentley/cco-releases/issues

## Version Questions

### Q: How do I check my current version?

**A:**

```bash
cco version
```

Output:
```
CCO version 2025.11.1
Build: Production
Rust: 1.75+

✅ You have the latest version
```

### Q: What's the version numbering scheme?

**A:** CCO uses date-based versioning: **YYYY.MM.N**

- **YYYY**: Year (e.g., 2025)
- **MM**: Month (1-12)
- **N**: Release counter for that month (starts at 1)

**Examples:**
```
2025.11.1    # First release in November 2025
2025.11.2    # Second release in November 2025
2025.12.1    # First release in December (counter resets)
2026.1.1     # First release in January (counter resets)
```

### Q: Can I have multiple versions installed?

**A:** No, CCO installs to a single location:
- **macOS/Linux**: `~/.local/bin/cco`
- **Windows**: `%AppData%\cco\cco.exe`

Only the latest version is active. If you need a previous version:

1. Download from releases page
2. Save with different name:
```bash
curl -L https://github.com/brentley/cco-releases/releases/download/v2025.11.1/... \
  -o ~/.local/bin/cco-2025.11.1
chmod +x ~/.local/bin/cco-2025.11.1

# Run specific version
~/.local/bin/cco-2025.11.1 version
```

### Q: How do I downgrade to a previous version?

**A:**

```bash
# Download specific version from releases page
RELEASE="v2025.11.1"
PLATFORM="darwin-arm64"  # or your platform

curl -L "https://github.com/brentley/cco-releases/releases/download/${RELEASE}/cco-${RELEASE}-${PLATFORM}.tar.gz" \
  -o /tmp/cco.tar.gz

# Extract and install
tar -xzf /tmp/cco.tar.gz -C /tmp
cp /tmp/cco ~/.local/bin/cco
chmod +x ~/.local/bin/cco

# Verify
cco version
```

## Production Questions

### Q: Should I enable auto-install in production?

**A:** Not recommended. Use manual updates instead:

```bash
# Disable auto-install
cco config set updates.auto_install false

# Schedule manual updates
0 2 * * 2 /usr/local/bin/cco update --yes
```

This gives you control over when updates happen.

### Q: How do I monitor update status in production?

**A:**

```bash
# Check current version
cco version

# Check last update time
cco config show | grep "Last update"

# Check if updates are enabled
cco config get updates.enabled
```

Collect this info from all servers:

```bash
for server in prod-1 prod-2 prod-3; do
    echo "=== $server ==="
    ssh $server cco version
    ssh $server cco config show | grep -E "Last|channel"
done
```

### Q: Can I schedule updates for specific times?

**A:** Yes, using cron (Linux/macOS):

```bash
# Edit crontab
crontab -e

# Add (Tuesday 2 AM UTC)
0 2 * * 2 /usr/local/bin/cco update --yes

# Or use absolute path
0 2 * * 2 /home/username/.local/bin/cco update --yes
```

### Q: Should I update in development or production first?

**A:** Update development first:

```
Development → Test → Staging → Production
    ↓          ↓         ↓           ↓
Update    Run tests   Verify    Schedule
          in staging  features  production
```

1. Update development/local
2. Test thoroughly
3. Verify in staging environment
4. Schedule production update

### Q: What's the rollback process?

**A:** Automatic rollback if new binary fails:

```
Update Check
  ├─ Backup current version
  ├─ Install new version
  ├─ Test new binary
  └─ If test fails:
     └─ Restore backup
     └─ Old version continues working
```

Manual rollback if needed:

```bash
# Check if backup exists
ls -la ~/.local/bin/cco*

# Restore
cp ~/.local/bin/cco.backup ~/.local/bin/cco

# Restart
cco daemon restart
```

### Q: How do I handle failed updates in production?

**A:** Steps:

1. **Don't panic** - Previous version is still there
2. **Check status**:
   ```bash
   cco version
   cco daemon status
   ```
3. **Restore if needed**:
   ```bash
   cp ~/.local/bin/cco.backup ~/.local/bin/cco
   cco daemon restart
   ```
4. **Investigate why it failed**:
   ```bash
   cco daemon logs | tail -100
   ```
5. **Report the issue** with logs and system info

## Support Questions

### Q: Where do I report bugs?

**A:** GitHub Issues:
https://github.com/brentley/cco-releases/issues

Include:
- CCO version
- System info (OS, architecture)
- Exact command you ran
- Full error message
- Steps to reproduce

### Q: Where do I find documentation?

**A:** Documentation is available at:
- [User Guide](./AUTO_UPDATE_USER_GUIDE.md)
- [Admin Guide](./AUTO_UPDATE_ADMIN_GUIDE.md)
- [Architecture](./AUTO_UPDATE_ARCHITECTURE.md)
- [GitHub Releases](https://github.com/brentley/cco-releases/releases)

### Q: How do I get help?

**A:** Resources:
1. Check this FAQ
2. Read the user guide
3. Check GitHub issues for similar problems
4. Open a new issue with details
5. Contact support at: support@example.com (placeholder)

### Q: Can I contribute updates?

**A:** Yes! The main repository accepts contributions:
https://github.com/brentley/cco

Submit pull requests with:
- Clear description of changes
- Tests for new features
- Documentation updates
- Commit messages following conventional commits

### Q: Where do I request features?

**A:** Feature requests at:
https://github.com/brentley/cco/discussions

Describe:
- What you want to do
- Why it would help
- How you'd like it to work

