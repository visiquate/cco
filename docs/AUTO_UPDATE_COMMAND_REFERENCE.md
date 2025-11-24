# Auto-Update Command Reference

Complete reference for all CCO auto-update commands.

## Command Overview

The auto-update system is controlled through two main command groups:

```
cco update   - Check for and install updates
cco config   - Manage update configuration
```

## Update Commands

### cco update

Check for updates and optionally install them.

**Basic syntax:**
```bash
cco update [OPTIONS]
```

**Interactive mode** (default):

```bash
cco update
```

**What it does:**
1. Checks for new versions on GitHub
2. Compares with your current version
3. Shows release notes if newer version is available
4. Prompts: "Update now? [Y/n]:"
5. Installs if you confirm, otherwise exits

**Example output:**
```
→ Checking for updates...
→ Current version: 2025.11.1
→ Latest version: 2025.11.2

What's new in v2025.11.2:
  - Fix: Improved error handling
  - Feature: New dashboard metrics
  - Perf: 15% faster startup

Update now? [Y/n]: y

→ Downloading CCO v2025.11.2...
→ Verifying checksum...
  ✓ Checksum verified
→ Extracting archive...
→ Backing up current version...
→ Installing update...
✅ Successfully updated to v2025.11.2

Restart CCO to use the new version.
```

### cco update --check

Check for updates without installing.

**Syntax:**
```bash
cco update --check
```

**What it does:**
1. Checks GitHub for latest version
2. Compares with current version
3. Prints result
4. Exits (doesn't install)

**Example output when update available:**
```
→ Checking for updates...
→ Current version: 2025.11.1
→ Latest version: 2025.11.2
ℹ️  New version available: v2025.11.2
Run 'cco update' to install
```

**Example output when up to date:**
```
→ Checking for updates...
→ Current version: 2025.11.2
→ Latest version: 2025.11.2
✅ You are running the latest version
```

### cco update --yes

Check for updates and install without prompting.

**Syntax:**
```bash
cco update --yes
```

**What it does:**
1. Checks GitHub for latest version
2. If newer version exists: downloads and installs automatically
3. Skips the "Update now?" confirmation prompt
4. Shows progress messages

**Useful for:**
- Automated scripts
- Cron jobs
- Unattended servers
- CI/CD pipelines

**Example output:**
```
→ Checking for updates...
→ Current version: 2025.11.1
→ Latest version: 2025.11.2

What's new in v2025.11.2:
  - Fix: Critical security update
  - Perf: 20% memory reduction

→ Downloading CCO v2025.11.2...
→ Verifying checksum...
  ✓ Checksum verified
→ Extracting archive...
→ Backing up current version...
→ Installing update...
✅ Successfully updated to v2025.11.2

Restart CCO to use the new version.
```

### cco update --channel stable

Check for updates from the stable channel.

**Syntax:**
```bash
cco update [--channel stable] [--check] [--yes]
```

**What it does:**
1. Overrides configured channel temporarily
2. Checks stable channel for latest release
3. Proceeds with check/install as specified

**Channels:**
- `stable` (default) - Fully tested releases
- `beta` - Pre-release versions with new features

**Example:**
```bash
# Check stable channel
cco update --channel stable --check

# Install from stable
cco update --channel stable --yes
```

### cco update --channel beta

Check for updates from the beta channel.

**Syntax:**
```bash
cco update --channel beta [--check] [--yes]
```

**What it does:**
1. Checks beta/prerelease versions
2. Finds latest beta release
3. Proceeds with check/install as specified

**Warning:** Beta releases may be unstable. Use only for testing.

**Example:**
```bash
# Check available beta versions
cco update --channel beta --check

# Install latest beta
cco update --channel beta --yes
```

## Configuration Commands

### cco config show

Display all update configuration.

**Syntax:**
```bash
cco config show
```

**Output:**
```
Update Configuration:
  Enabled: true
  Auto-install: false
  Check interval: daily
  Channel: stable
  Last check: 2025-11-17 14:32:15 UTC
  Last update: 2025-11-10 09:22:03 UTC
```

**What to look for:**
- `Enabled: false` = Updates are disabled, must manually check
- `Auto-install: true` = Updates install automatically
- `Check interval: never` = Only manual updates via `cco update`
- `Last check: Never` = Never checked for updates yet

### cco config set KEY VALUE

Set a configuration value.

**Syntax:**
```bash
cco config set <key> <value>
```

**Valid keys:**
- `updates.enabled` (boolean)
- `updates.auto_install` (boolean)
- `updates.check_interval` (string)
- `updates.channel` (string)

**Valid values by key:**

| Key | Values | Example |
|-----|--------|---------|
| `updates.enabled` | `true`, `false` | `cco config set updates.enabled true` |
| `updates.auto_install` | `true`, `false` | `cco config set updates.auto_install false` |
| `updates.check_interval` | `daily`, `weekly`, `never` | `cco config set updates.check_interval daily` |
| `updates.channel` | `stable`, `beta` | `cco config set updates.channel stable` |

**Examples:**

```bash
# Enable update checks
cco config set updates.enabled true

# Disable automatic installations
cco config set updates.auto_install false

# Check for updates weekly instead of daily
cco config set updates.check_interval weekly

# Never check automatically (manual only)
cco config set updates.check_interval never

# Use beta channel
cco config set updates.channel beta

# Switch back to stable
cco config set updates.channel stable
```

**Output on success:**
```
✅ Configuration updated: updates.enabled = true
```

**Output on error:**
```
Invalid interval: monthly. Use: daily, weekly, never
Invalid channel: experimental. Use: stable, beta
Invalid boolean value: maybe
```

### cco config get KEY

Get a specific configuration value.

**Syntax:**
```bash
cco config get <key>
```

**Example:**
```bash
cco config get updates.enabled
# Output: updates.enabled = true

cco config get updates.channel
# Output: updates.channel = stable

cco config get updates.last_check
# Output: updates.last_check = 2025-11-17T14:32:15Z
```

**Useful for:**
- Scripting: Extract value for use in automation
- Verification: Confirm setting was applied
- Monitoring: Check configuration across fleet

**Parsing output in scripts:**

```bash
# Extract just the value
ENABLED=$(cco config get updates.enabled | awk '{print $NF}')

if [ "$ENABLED" = "true" ]; then
    echo "Updates are enabled"
fi

# Get channel
CHANNEL=$(cco config get updates.channel | awk '{print $NF}')

case $CHANNEL in
    stable) echo "Using stable releases" ;;
    beta) echo "Using beta releases" ;;
esac
```

## Version Commands

### cco version

Show current version and check for updates.

**Syntax:**
```bash
cco version
```

**Output:**
```
CCO version 2025.11.1
Build: Production
Rust: 1.75+

✅ You have the latest version
```

**Or if update is available:**
```
CCO version 2025.11.1
Build: Production
Rust: 1.75+

⚠️  New version available: 2025.11.2 (current: 2025.11.1)
   Run 'cco update' to upgrade
```

**Useful for:**
- Verifying installation worked
- Checking for updates in one command
- Getting version for bug reports

## Advanced Usage

### Update in Scripts

**Safe update with error handling:**

```bash
#!/bin/bash
set -e

echo "Checking for updates..."
if cco update --check 2>&1 | grep -q "New version available"; then
    echo "Update available, installing..."
    cco update --yes

    # Restart daemon if running
    if cco daemon status > /dev/null 2>&1; then
        echo "Restarting daemon..."
        cco daemon restart
    fi

    echo "✅ Update complete"
else
    echo "No updates available"
fi
```

**Scheduled updates with cron:**

```bash
# Edit crontab
crontab -e

# Update every Tuesday at 2 AM UTC
0 2 * * 2 /usr/local/bin/cco update --yes 2>&1 | logger -t cco-update
```

**Monitor updates across fleet:**

```bash
#!/bin/bash
for server in prod-{1,2,3}; do
    echo "=== $server ==="
    ssh $server 'cco version && cco config get updates.last_check'
    echo
done
```

### Conditional Updates

**Update only on specific days:**

```bash
#!/bin/bash
WEEKDAY=$(date +%A)

case $WEEKDAY in
    Tuesday)
        echo "Update day, checking..."
        cco update --yes
        ;;
    *)
        echo "Not update day"
        ;;
esac
```

**Update only if version is old:**

```bash
#!/bin/bash
CURRENT=$(cco version | head -1 | awk '{print $3}')
THRESHOLD="2025.11.1"

# Simple string comparison (works for YYYY.MM.N format)
if [ "$CURRENT" \< "$THRESHOLD" ]; then
    echo "Version is old, updating..."
    cco update --yes
else
    echo "Version is recent enough"
fi
```

### Extract Update Status

**Get version for monitoring:**

```bash
#!/bin/bash
# Output: version=2025.11.1
echo "version=$(cco version | head -1 | awk '{print $3}')"
```

**Get last update time:**

```bash
#!/bin/bash
# Output: last_update=2025-11-10T09:22:03Z
cco config get updates.last_update | awk '{print "last_update=" $3}'
```

**Get update channel:**

```bash
#!/bin/bash
# Output: channel=stable
cco config get updates.channel | awk '{print "channel=" $3}'
```

**Get enablement status:**

```bash
#!/bin/bash
# Output: enabled=true
cco config get updates.enabled | awk '{print "enabled=" $3}'
```

## Common Command Combinations

### Setup for Development (Auto-Updates)

```bash
cco config set updates.enabled true
cco config set updates.check_interval daily
cco config set updates.channel stable
cco config show
```

### Setup for Production (Manual Updates)

```bash
cco config set updates.enabled false
cco config set updates.auto_install false
cco config set updates.check_interval never
cco config set updates.channel stable
cco config show
```

### Switch to Beta Channel (Testing)

```bash
cco config set updates.channel beta
cco update --check
cco update --yes
```

### Switch Back to Stable

```bash
cco config set updates.channel stable
cco update --yes
```

### Disable All Update Checks

```bash
cco config set updates.enabled false
cco config set updates.check_interval never
cco config show
```

### Re-enable Updates

```bash
cco config set updates.enabled true
cco config set updates.check_interval daily
cco config show
```

## Exit Codes

All commands return standard exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Network error |
| 5 | Update failed |

**Example usage in scripts:**

```bash
if cco update --yes; then
    echo "Update successful"
else
    EXIT_CODE=$?
    echo "Update failed with code $EXIT_CODE"
    exit $EXIT_CODE
fi
```

## Troubleshooting Command Issues

### Command hangs (no output for long time)

```bash
# Check if GitHub is accessible
timeout 5 curl -I https://api.github.com

# Check your internet
ping github.com

# Check proxy settings
echo $HTTPS_PROXY
```

### "Unknown configuration key" error

```bash
# Valid keys only
cco config set updates.enabled true     # ✓
cco config set updates.auto_install true # ✓
cco config set update.enabled true      # ✗ (typo)
cco config set updates.foo true         # ✗ (invalid)
```

### "Invalid boolean value" error

```bash
# Valid values
cco config set updates.enabled true     # ✓
cco config set updates.enabled false    # ✓
cco config set updates.enabled maybe    # ✗
cco config set updates.enabled 1        # ✗
```

### "Invalid interval" error

```bash
# Valid intervals
cco config set updates.check_interval daily   # ✓
cco config set updates.check_interval weekly  # ✓
cco config set updates.check_interval never   # ✓
cco config set updates.check_interval monthly # ✗
```

### "Invalid channel" error

```bash
# Valid channels
cco config set updates.channel stable   # ✓
cco config set updates.channel beta     # ✓
cco config set updates.channel latest   # ✗
cco config set updates.channel dev      # ✗
```

## Command Help

Get help for any command:

```bash
# Show main help
cco --help

# Show update command help
cco update --help

# Show config command help
cco config --help
```

