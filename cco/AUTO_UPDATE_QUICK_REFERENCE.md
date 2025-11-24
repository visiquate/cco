# Auto-Update Quick Reference

## Default Behavior

CCO **automatically updates itself** by default without requiring user confirmation.

## Quick Commands

```bash
# View current update configuration
cco config show

# Force update check and install
cco update

# Check for updates without installing
cco update --check

# Update with confirmation prompt (opt-out of auto-install)
cco update --prompt

# Use beta channel
cco update --channel beta
```

## Disable Auto-Updates

### Permanently (Config File)
```bash
# Disable all auto-updates
cco config set updates.enabled false

# Disable auto-install (require confirmation)
cco config set updates.auto_install false

# Never check for updates
cco config set updates.check_interval never
```

### Temporarily (Environment Variable)
```bash
# Disable for single command
CCO_AUTO_UPDATE=false cco run

# Disable globally (add to ~/.bashrc or ~/.zshrc)
export CCO_AUTO_UPDATE=false
```

## Configuration Options

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `updates.enabled` | `true`/`false` | `true` | Enable auto-updates |
| `updates.auto_install` | `true`/`false` | `true` | Auto-install without prompt |
| `updates.check_interval` | `daily`/`weekly`/`never` | `daily` | Update check frequency |
| `updates.channel` | `stable`/`beta` | `stable` | Update channel |

## Environment Variables

| Variable | Values | Description |
|----------|--------|-------------|
| `CCO_AUTO_UPDATE` | `true`/`false` | Enable/disable auto-updates |
| `CCO_AUTO_UPDATE_CHANNEL` | `stable`/`beta` | Override update channel |
| `CCO_AUTO_UPDATE_INTERVAL` | `daily`/`weekly`/`never` | Override check interval |

**Priority:** Environment variables > Config file > Defaults

## Update Logs

**Location:** `~/.cco/logs/updates.log`

**View logs:**
```bash
# Show log location and size
cco config show

# View logs directly
tail -f ~/.cco/logs/updates.log

# View last 50 lines
tail -n 50 ~/.cco/logs/updates.log
```

**Log rotation:**
- Automatically rotates when >10MB
- Keeps last 30 days of logs

## Troubleshooting

### Updates not working?

1. **Check configuration:**
   ```bash
   cco config show
   ```

2. **Check logs:**
   ```bash
   tail ~/.cco/logs/updates.log
   ```

3. **Force manual update:**
   ```bash
   cco update
   ```

### Disable updates temporarily

```bash
CCO_AUTO_UPDATE=false cco run
```

### Check current version

```bash
cco --version
```

## Update Process

1. **Background check** (on daemon startup)
2. **Download** new binary from GitHub
3. **Verify** checksum and binary
4. **Install** atomically with backup
5. **Log** all activities
6. **Notify** user (restart required)

**Note:** Updates do NOT automatically restart the daemon. You must manually restart:
```bash
cco daemon restart
```

## Safety Features

- Checksum verification (SHA256)
- Binary verification before/after install
- Automatic rollback on failure
- Atomic replacement (Unix)
- Backup creation before update

## Examples

### Check if updates are enabled
```bash
cco config show | grep "Enabled:"
```

### View effective configuration (with environment overrides)
```bash
cco config show
```

### Disable auto-install but keep checking
```bash
cco config set updates.auto_install false
```

### Change to weekly checks
```bash
cco config set updates.check_interval weekly
```

### Switch to beta channel
```bash
cco config set updates.channel beta
```

### Temporarily use beta channel for one update
```bash
CCO_AUTO_UPDATE_CHANNEL=beta cco update
```

## Summary

- **Default:** Auto-updates enabled, auto-install enabled, daily checks, stable channel
- **Opt-out:** Config file, environment variables, or `--prompt` flag
- **Logs:** `~/.cco/logs/updates.log` (rotates at 10MB, keeps 30 days)
- **Safety:** Checksum verification, binary verification, rollback on failure
