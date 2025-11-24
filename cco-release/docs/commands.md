# Command Reference

Complete reference for all CCO commands and options.

## Table of Contents

1. [Basic Commands](#basic-commands)
2. [Daemon Management](#daemon-management)
3. [Configuration](#configuration)
4. [Monitoring & Logs](#monitoring--logs)
5. [Advanced Options](#advanced-options)
6. [Environment Variables](#environment-variables)
7. [Quick Reference Table](#quick-reference-table)

---

## Basic Commands

### Launch Claude Code

```bash
cco
```

Launch Claude Code with orchestration support in current directory. The daemon auto-starts if not running.

**Example:**
```bash
cd ~/my-project
cco
```

**What happens:**
1. Checks if daemon is running (auto-starts if not)
2. Verifies settings files exist
3. Sets ORCHESTRATOR_* environment variables
4. Launches Claude Code in current directory

### Launch TUI Dashboard

```bash
cco tui
```

Launch monitoring dashboard with real-time metrics.

**Features:**
- Real-time cost metrics
- Token usage tracking
- API call count
- Cache hit rate
- Recent activity log

### Show Version

```bash
cco version
```

Display current CCO version in format `YYYY.MM.N+commithash`.

**Output example:**
```
CCO version 2025.11.17+e37b7c0
```

### Check Health

```bash
cco health
```

Check if daemon is running and healthy.

**Output:**
- Status: ok/error
- Uptime
- Database status
- Cache status

### Update CCO

```bash
cco update
cco update --check
cco update --yes
```

**Options:**
- `--check`: Only check for updates, don't install
- `--yes`: Auto-confirm installation

---

## Daemon Management

### Start Daemon

```bash
cco daemon start
cco daemon start --port 3001
cco daemon start --cache-size 2000 --cache-ttl 7200
```

Start the CCO daemon process.

**Options:**
- `--port PORT`: Listen port (default: 3000)
- `--host HOST`: Bind address (default: 127.0.0.1)
- `--cache-size MB`: Cache size in MB (default: 500)
- `--cache-ttl SECS`: Cache TTL in seconds (default: 3600)
- `--log-level LEVEL`: Log level: debug, info, warn, error
- `--workers N`: Worker threads (default: CPU count)
- `--db-path PATH`: Database file path

**Examples:**

Team server setup:
```bash
cco daemon start --host 0.0.0.0 --port 3000
```

Custom cache settings:
```bash
cco daemon start --cache-size 2000 --cache-ttl 7200
```

Debug mode:
```bash
cco daemon start --log-level debug
```

### Stop Daemon

```bash
cco daemon stop
cco daemon stop --force
```

Gracefully stop the daemon.

**Options:**
- `--force`: Force stop if not responding

### Restart Daemon

```bash
cco daemon restart
cco daemon restart --port 3001
```

Stop and start daemon with new settings.

### Check Daemon Status

```bash
cco daemon status
cco daemon status --verbose
```

Show daemon status and information.

**Output example:**
```
Daemon is running (PID: 12345)
Uptime: 2 hours 15 minutes
VFS: /var/run/cco (mounted)
Dashboard: http://localhost:3000
Cache: 256 MB / 500 MB
```

### View Daemon Logs

```bash
cco daemon logs
cco daemon logs --follow
cco daemon logs --level error
cco daemon logs --tail 50
cco daemon logs --lines 100
```

View daemon activity logs.

**Options:**
- `--follow`: Follow logs in real-time (Ctrl+C to exit)
- `--level LEVEL`: Filter by level (debug, info, warn, error)
- `--tail N`: Show last N lines
- `--lines N`: Show specific number of lines

### Install as System Service

```bash
cco daemon install
cco daemon uninstall
```

Install or uninstall as system service.

**macOS (launchd):**
```bash
cco daemon install
cco daemon enable   # Enable auto-start on boot
sudo launchctl load ~/Library/LaunchAgents/com.visiquate.cco.plist
```

**Linux (systemd):**
```bash
cco daemon install
sudo systemctl enable cco
sudo systemctl start cco
```

---

## Configuration

### View Configuration

```bash
cco config
```

Show current configuration.

### Set Configuration Value

```bash
cco config set <key> <value>

# Examples:
cco config set auto_update enabled
cco config set metrics_enabled true
cco config set cache_size 1000
cco config set cache_ttl 7200
```

### Reset Configuration

```bash
cco config reset
```

Reset to default configuration.

---

## Monitoring & Logs

### View Analytics

```bash
# Current project metrics
curl http://localhost:3000/api/project/stats

# Machine-wide analytics
curl http://localhost:3000/api/machine/stats

# Cache statistics
curl http://localhost:3000/api/cache/stats
```

### Clear Cache

```bash
# Via API
curl -X POST http://localhost:3000/api/cache/clear

# Output:
# {
#   "status": "ok",
#   "cleared": 89,
#   "freedMemory": "256MB"
# }
```

### Export Analytics

```bash
# Export last 7 days as JSON
curl "http://localhost:3000/api/export/analytics?days=7" > analytics.json

# Export as CSV
curl "http://localhost:3000/api/export/csv?days=30" > analytics.csv

# Export specific project
curl "http://localhost:3000/api/export/project/my-project?format=json" > project.json
```

---

## Advanced Options

### Pass Arguments to Claude Code

Any argument not recognized by CCO is passed to Claude Code:

```bash
cco --help              # Claude Code help
cco analyze src/main.rs  # Analyze file
cco refactor legacy.py   # Refactor code
cco explain code.py      # Explain code
cco test integration.rs  # Run tests
```

### Custom Port

```bash
cco daemon start --port 9000

# Update environment variable
export ORCHESTRATOR_API_URL=http://localhost:9000
```

### Team Server Setup

```bash
# Server: Start daemon (accessible to team)
ssh team-server
cco daemon start --host 0.0.0.0 --port 3000

# Developer: Point to remote daemon
export ORCHESTRATOR_API_URL=http://team-server:3000
cco
```

### Multiple Projects

```bash
# Project 1
cd ~/project-1
cco  # Uses same daemon

# Project 2
cd ~/project-2
cco  # Uses same daemon
```

All projects share the same daemon, cache, and metrics.

---

## Environment Variables

### Auto-Set by CCO

When you run `cco`, these are automatically set:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
ORCHESTRATOR_API_URL=http://localhost:3000
```

### User-Set Variables

```bash
# Required: Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: OpenAI API key
export OPENAI_API_KEY="sk-..."

# Optional: Override daemon URL
export ORCHESTRATOR_API_URL=http://remote-server:3000

# Optional: Disable orchestration
export ORCHESTRATOR_ENABLED=false

# Optional: Custom port
export CCO_PORT=8000

# Optional: Log level
export CCO_LOG_LEVEL=debug

# Optional: Cache settings
export CCO_CACHE_SIZE=1000      # MB
export CCO_CACHE_TTL=3600       # seconds

# Optional: Worker threads
export CCO_WORKERS=8
```

### Temp Directory Locations

The `$TMPDIR` variable points to OS temp directory:

**macOS:**
```
/var/folders/xx/xxx/T/
```

**Linux:**
```
/tmp/
```

**Windows (WSL2):**
```
C:\Users\[user]\AppData\Local\Temp\
```

---

## Quick Reference Table

| Command | Purpose | Example |
|---------|---------|---------|
| `cco` | Launch Claude Code | `cco` |
| `cco tui` | Launch dashboard | `cco tui` |
| `cco daemon start` | Start daemon | `cco daemon start` |
| `cco daemon stop` | Stop daemon | `cco daemon stop` |
| `cco daemon restart` | Restart daemon | `cco daemon restart` |
| `cco daemon status` | Check status | `cco daemon status` |
| `cco daemon logs` | View logs | `cco daemon logs --follow` |
| `cco daemon install` | Install as service | `cco daemon install` |
| `cco version` | Show version | `cco version` |
| `cco update` | Check updates | `cco update --check` |
| `cco health` | Health check | `cco health` |
| `cco config` | Show config | `cco config` |
| `cco config set` | Set config value | `cco config set cache_size 1000` |

---

## Troubleshooting Commands

### Verify Installation

```bash
which cco              # Find CCO location
cco version            # Show version
cco health             # Check health
```

### Check Daemon

```bash
cco daemon status      # Check if running
cco daemon logs        # View logs
ps aux | grep cco      # Find process
```

### Verify Settings

```bash
# Check temp directory
ls -la $TMPDIR/.cco-*

# Check environment variables
env | grep ORCHESTRATOR

# Check API key
echo $ANTHROPIC_API_KEY | grep -c "sk-ant"
```

### Network Diagnostics

```bash
# Check if daemon is listening
lsof -i :3000

# Test connection
curl http://localhost:3000/health

# Test API
curl http://localhost:3000/api/project/stats
```

### Clear Cache

```bash
# Via CLI
curl -X POST http://localhost:3000/api/cache/clear

# Via daemon restart (clears in-memory cache)
cco daemon restart
```

---

## Getting Help

### Print Help

```bash
cco --help             # CCO help
cco daemon --help      # Daemon command help
cco update --help      # Update command help
```

### Enable Debug Logging

```bash
cco daemon restart --log-level debug
cco daemon logs --follow
```

### Check Known Issues

See [Troubleshooting Guide](./troubleshooting.md) for detailed solutions.

---

## Daily Workflow Cheat Sheet

```bash
# First time setup
curl -fsSL https://cco.visiquate.com/install.sh | sh
export ANTHROPIC_API_KEY="sk-ant-..."
cco daemon start

# Daily development
cd ~/my-project
cco                  # Terminal 1: Development
cco tui              # Terminal 2: Monitoring

# Team server setup
cco daemon start --host 0.0.0.0 --port 3000
export ORCHESTRATOR_API_URL=http://server:3000

# Maintenance
cco update --check   # Check for updates
cco daemon logs      # View logs
cco daemon restart   # Restart daemon
```
