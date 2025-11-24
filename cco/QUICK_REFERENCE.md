# CCO Quick Reference Guide

**One-page reference for all common CCO operations**

---

## Installation

```bash
# Install CCO
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Verify installation
cco version

# Configure API key
export ANTHROPIC_API_KEY="sk-ant-..."
echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.zshrc
```

---

## Essential Commands

| Command | What it does |
|---------|--------------|
| `cco` | Launch Claude Code with orchestration |
| `cco tui` | Launch TUI monitoring dashboard |
| `cco daemon start` | Start daemon manually |
| `cco daemon stop` | Stop daemon |
| `cco daemon restart` | Restart daemon |
| `cco daemon status` | Check if daemon is running |
| `cco daemon logs` | View daemon logs |
| `cco version` | Show version |
| `cco update` | Check/install updates |

---

## Daily Workflow

### Development Session

```bash
# Terminal 1: Development
cd ~/my-project
cco                    # Launches Claude Code

# Terminal 2: Monitoring (optional)
cco tui                # Shows real-time metrics
```

### First Time Setup

```bash
# 1. Install CCO
curl -fsSL https://cco.visiquate.com/install.sh | sh

# 2. Set API key
export ANTHROPIC_API_KEY="sk-ant-..."

# 3. Start daemon
cco daemon start

# 4. Verify temp files
ls -la $TMPDIR/.cco-*  # Should show encrypted files

# 5. Launch Claude Code
cd ~/test-project
cco
```

---

## Troubleshooting Quick Fixes

### Daemon Won't Start

```bash
# Check what's using port 3000
lsof -i :3000

# Use different port
cco daemon start --port 3001

# Or kill conflicting process
kill <PID>
cco daemon start
```

### Temp Files Not Found

```bash
# Restart daemon
cco daemon restart

# Verify files created
ls -la $TMPDIR/.cco-*  # Should show encrypted files

# Check temp directory permissions
chmod 755 $TMPDIR
```

### Claude Code Not Found

```bash
# Install Claude Code
# https://claude.ai/code

# Add to PATH
echo 'export PATH="$PATH:/path/to/claude-code"' >> ~/.zshrc
source ~/.zshrc

# Verify
which claude-code
```

### Launch is Slow

```bash
# Normal on first run (daemon auto-starts)
# First run: 3-4 seconds
# Subsequent runs: <1 second

# Pre-start daemon to avoid delay
cco daemon start
```

---

## Environment Variables

**Auto-set by CCO:**

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
ORCHESTRATOR_API_URL=http://localhost:3000
```

**Verify (from within Claude Code):**
```bash
env | grep ORCHESTRATOR
```

**Platform-specific temp directories:**
- macOS: `/var/folders/xx/xxx/T/`
- Windows: `C:\Users\[user]\AppData\Local\Temp\`
- Linux: `/tmp/`

---

## Temp File Structure

```
$TMPDIR/
├── .cco-orchestrator-settings  # Main settings (encrypted)
├── .cco-agents-sealed          # 119 agent definitions (encrypted)
├── .cco-rules-sealed           # Orchestration rules (encrypted)
└── .cco-hooks-sealed           # Pre/post-compaction hooks (encrypted)
```

**Quick checks:**
```bash
# List temp files
ls -la $TMPDIR/.cco-*

# Verify files exist
test -f $TMPDIR/.cco-orchestrator-settings && echo "Settings found"

# Check file permissions (should be -rw-------)
ls -l $TMPDIR/.cco-*
```

---

## Advanced Usage

### Pass Arguments to Claude Code

```bash
cco --help              # Claude Code help
cco analyze src/main.rs  # Analyze file
cco refactor legacy.py   # Refactor code
```

### Custom Daemon Configuration

```bash
# Start with custom port
cco daemon start --port 3001

# Start with larger cache
cco daemon start --cache-size 2000

# Start with debug logging
cco daemon start --log-level debug

# Bind to all interfaces (team server)
cco daemon start --host 0.0.0.0 --port 3000
```

### Multiple Projects

```bash
# Same daemon serves all projects
cd ~/project-1
cco  # Launches Claude Code for project 1

cd ~/project-2
cco  # Launches Claude Code for project 2
# Both share same daemon, VFS, and cache
```

### Remote Daemon

```bash
# Server
ssh team-server
cco daemon start --host 0.0.0.0 --port 3000

# Client
export ORCHESTRATOR_API_URL=http://team-server:3000
cco
```

---

## Daemon Management

### Check Status

```bash
# Quick status
cco daemon status

# Detailed info
cco daemon status --verbose

# Output example:
# Daemon is running (PID: 12345)
# Uptime: 2 hours 15 minutes
# VFS: /var/run/cco (mounted)
# Dashboard: http://localhost:3000
```

### View Logs

```bash
# Recent logs
cco daemon logs

# Follow logs in real-time
cco daemon logs --follow

# Filter by level
cco daemon logs --level error

# Last N lines
cco daemon logs --tail 50
```

### Restart

```bash
# Graceful restart (preserves config)
cco daemon restart

# Restart with new settings
cco daemon restart --port 3001
```

---

## Monitoring

### TUI Dashboard

```bash
# Launch dashboard
cco tui

# Shows:
# - Real-time cost metrics
# - Token usage
# - API call count
# - Cache hit rate
# - Recent activity
```

### Web Dashboard

```bash
# Auto-opens in browser after daemon start
cco daemon start

# Or open manually
open http://localhost:3000
```

### API Endpoints

```bash
# Project stats
curl http://localhost:3000/api/project/stats

# Machine-wide stats
curl http://localhost:3000/api/machine/stats

# Cache statistics
curl http://localhost:3000/api/cache/stats

# Health check
curl http://localhost:3000/health
```

---

## Common Scenarios

### Scenario: First-time user

```bash
# Install
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Configure
export ANTHROPIC_API_KEY="sk-ant-..."

# Test
cd ~/test-project
cco  # Daemon auto-starts, Claude Code launches
```

### Scenario: Daily development

```bash
# Terminal 1
cd ~/project
cco  # Daemon auto-starts if needed

# Terminal 2 (optional monitoring)
cco tui
```

### Scenario: Team development

```bash
# Server setup (one-time)
ssh dev-server
cco daemon start --host 0.0.0.0

# Developer setup (each developer)
export ORCHESTRATOR_API_URL=http://dev-server:3000
cco
```

### Scenario: Daemon crashed

```bash
# Check status
cco daemon status

# View logs for errors
cco daemon logs | grep -i error

# Restart
cco daemon restart

# Verify temp files recreated
ls -la $TMPDIR/.cco-*  # Should show encrypted files
```

### Scenario: Port conflict

```bash
# Find conflicting process
lsof -i :3000

# Kill it
kill <PID>

# Or use different port
cco daemon start --port 3001
export ORCHESTRATOR_API_URL=http://localhost:3001
```

---

## Migration from Old CLI

| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `cco` | `cco tui` | TUI now requires explicit subcommand |
| `cco run` | `cco` | Primary use is now launching Claude Code |
| `cco dashboard` | `cco tui` | Renamed for clarity |
| `cco daemon start` | `cco daemon start` | Same (unchanged) |
| N/A | `cco [args]` | New: pass args to Claude Code |

---

## Performance Tips

### Optimize Cache

```bash
# Increase cache size (default 500MB)
cco daemon start --cache-size 2000  # 2GB

# Increase cache TTL (default 3600s = 1 hour)
cco daemon start --cache-ttl 7200  # 2 hours
```

### Pre-start Daemon

```bash
# Avoid 3-4 second delay on first `cco`
cco daemon start

# Now `cco` is instant
cco  # <1 second
```

### Monitor Cache Hit Rate

```bash
# Check cache effectiveness
curl http://localhost:3000/api/cache/stats | jq '.hitRate'

# Should be >50% for good savings
# 70-90% is excellent
```

---

## Security Best Practices

1. **Never commit API keys to git**
   ```bash
   # Use environment variables
   export ANTHROPIC_API_KEY="sk-ant-..."

   # Or .env file (gitignored)
   echo 'ANTHROPIC_API_KEY="sk-ant-..."' > .env
   echo '.env' >> .gitignore
   ```

2. **Use HTTPS in production**
   ```bash
   # For remote daemons, use reverse proxy with SSL
   # nginx, Caddy, or similar
   ```

3. **Restrict daemon access**
   ```bash
   # Bind to localhost only (default)
   cco daemon start --host 127.0.0.1

   # Or specific interface for team use
   cco daemon start --host 192.168.1.100
   ```

4. **Regular updates**
   ```bash
   # Check for updates weekly
   cco update --check

   # Install updates
   cco update
   ```

---

## Getting Help

### Documentation

- [README.md](./README.md) - Project overview
- [INSTALLATION.md](./INSTALLATION.md) - Installation guide
- [USAGE.md](./USAGE.md) - Complete command reference
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues
- [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) - For existing users

### Debugging

```bash
# Enable debug logging
cco daemon restart --log-level debug

# View logs
cco daemon logs --follow

# Check health
cco daemon status
cat /var/run/cco/health

# Verify environment
env | grep ORCHESTRATOR
```

### Common Issues

| Issue | Quick Fix |
|-------|-----------|
| Daemon won't start | `lsof -i :3000` → kill process → retry |
| VFS not mounted | `cco daemon restart` |
| Claude Code not found | Install from https://claude.ai/code |
| Slow first launch | Normal (daemon auto-start), 3-4 sec |
| Variables not set | Use `cco` not `claude-code` directly |

---

## Key Concepts

**Daemon:** Background service that provides orchestration
**Temp Files:** Encrypted configuration files in OS temp directory
**Sealed files:** Encrypted agent configurations
**Settings:** Main orchestration configuration file
**Orchestration:** Coordination of 119 specialized agents
**Automatic cleanup:** Temp files removed when daemon stops

**Remember:**
- One daemon per machine (shared across projects)
- Temp files auto-created when daemon starts
- Environment variables auto-set by `cco` command
- Both `cco` and `cco tui` can run simultaneously
- Temp files auto-cleaned when daemon stops

---

## Summary

**Simplest workflow:**
```bash
cco  # That's it! Everything else is automatic.
```

**What happens automatically:**
1. Daemon starts (if not running)
2. Temp files created in OS temp directory
3. Environment variables set
4. Claude Code launches with orchestration

**When things go wrong:**
```bash
cco daemon restart && ls -la $TMPDIR/.cco-*
# Should show encrypted files
```

**For monitoring:**
```bash
cco tui  # In separate terminal
```
