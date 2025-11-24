# CCO User Guide

Complete guide for using Claude Code Orchestrator (CCO) with the enhanced CLI.

---

## Table of Contents

1. [What is CCO?](#what-is-cco)
2. [Basic Workflows](#basic-workflows)
3. [Command Reference](#command-reference)
4. [Advanced Usage](#advanced-usage)
5. [Troubleshooting](#troubleshooting)

---

## What is CCO?

CCO (Claude Code Orchestrator) is a multi-agent development system that enhances Claude Code with:

- **119 Specialized Agents**: Architecture, coding, testing, security, DevOps
- **Temp-Based Settings**: Encrypted configuration files in OS temp directory
- **Daemon-Based Routing**: Intelligent request routing and orchestration
- **Real-Time Monitoring**: TUI dashboard for metrics and agent activity
- **Cost Tracking**: Monitor API usage and savings

### How It Works

```
┌─────────────────────────────────────────────────────┐
│  cco (CLI)                                          │
│  ├─ Ensures daemon is running                      │
│  ├─ Verifies settings files exist                  │
│  ├─ Sets ORCHESTRATOR_* environment variables      │
│  └─ Launches Claude Code in current directory      │
└─────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────┐
│  CCO Daemon (Background)                            │
│  ├─ HTTP API Server (port 3000)                     │
│  ├─ Temp Files ($TMPDIR/.cco-*)                     │
│  │   ├─ .cco-orchestrator-settings (encrypted)     │
│  │   ├─ .cco-agents-sealed (encrypted)             │
│  │   ├─ .cco-rules-sealed (encrypted)              │
│  │   └─ .cco-hooks-sealed (encrypted)              │
│  └─ Agent communication & metrics                   │
└─────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────┐
│  Claude Code (with Orchestration)                   │
│  ├─ Reads settings from temp files                 │
│  ├─ Decrypts agent definitions                     │
│  ├─ Coordinates 119 specialized agents             │
│  └─ Reports metrics to daemon                      │
└─────────────────────────────────────────────────────┘
```

---

## Basic Workflows

### Workflow 1: Single Development Session

```bash
# Start fresh
$ cco daemon start

# Launch Claude Code
$ cd ~/my-project
$ cco

# Now develop with Claude Code
# The daemon runs in background
```

**What happens:**
1. Check if daemon is running (auto-starts if not)
2. Verify settings files exist in temp directory
3. Set `ORCHESTRATOR_*` environment variables
4. Launch Claude Code in current directory

### Workflow 2: Development + Monitoring

```bash
# Terminal 1: Development
$ cd ~/my-project
$ cco                    # Launches Claude Code

# Terminal 2: Monitoring (in parallel)
$ cco tui               # Shows dashboard with agent activity
```

**Benefits:**
- Real-time visibility into agent coordination
- Cost tracking and API usage metrics
- Cache hit rates and savings

### Workflow 3: Multiple Projects

```bash
# Each project gets same daemon + VFS
$ cd ~/project-1
$ cco                   # Launches Claude Code for project 1

# In another terminal
$ cd ~/project-2
$ cco                   # Launches Claude Code for project 2
# Both share the same daemon for routing/orchestration
```

**Benefits:**
- Single daemon serves all projects
- Consistent orchestration rules
- Shared cache and metrics

---

## Command Reference

### Core Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `cco` | Launch Claude Code with orchestration | `cco` |
| `cco tui` | Launch monitoring dashboard | `cco tui` |
| `cco daemon start` | Start daemon (usually not needed) | `cco daemon start` |
| `cco daemon stop` | Stop daemon | `cco daemon stop` |
| `cco daemon restart` | Restart daemon | `cco daemon restart` |
| `cco daemon status` | Check daemon status | `cco daemon status` |
| `cco daemon logs` | View daemon logs | `cco daemon logs --follow` |
| `cco version` | Show version | `cco version` |
| `cco update` | Check for updates | `cco update` |

### Starting CCO

#### Launch Claude Code (Primary Use)

```bash
# Navigate to project
cd ~/my-awesome-project

# Launch Claude Code with orchestration
cco

# Expected output:
# ✅ Daemon is running
# ✅ VFS mounted and healthy
# ✅ Orchestration environment configured
# 🚀 Launching Claude Code with orchestration support...
#    Working directory: /Users/you/my-awesome-project
#    VFS mount: /var/run/cco/
```

#### Launch TUI Dashboard

```bash
# In another terminal, monitor activity
cco tui

# Expected output:
# ✅ Daemon is running
# 🎯 Launching TUI dashboard...
#
# (TUI interface shows real-time metrics)
```

### Daemon Management

#### Start Daemon

```bash
# Start daemon (auto-starts if not running)
cco daemon start

# Start with custom port
cco daemon start --port 3001

# Start with custom cache settings
cco daemon start --cache-size 2000 --cache-ttl 7200
```

#### Stop Daemon

```bash
# Gracefully stop daemon
cco daemon stop

# Force stop if not responding
cco daemon stop --force
```

#### Restart Daemon

```bash
# Restart daemon (preserves configuration)
cco daemon restart

# Restart with new settings
cco daemon restart --port 3001
```

#### Check Status

```bash
# Check if daemon is running
cco daemon status

# Output:
# Daemon is running (PID: 12345)
# Uptime: 2 hours 15 minutes
# VFS: /var/run/cco (mounted)
# Dashboard: http://localhost:3000
```

#### View Logs

```bash
# View recent logs
cco daemon logs

# Follow logs in real-time
cco daemon logs --follow

# Filter by level
cco daemon logs --level error

# View last N lines
cco daemon logs --tail 50
```

### Configuration

```bash
# Custom Configuration
# Start on custom port with auto-open
cco daemon start --port 9000
# Dashboard opens at http://localhost:9000

# Start with verbose logging
cco daemon start --log-level debug

# Start with custom database path
cco daemon start --db-path /var/lib/cco/analytics.db

# Bind to all interfaces (for team servers)
cco daemon start --host 0.0.0.0 --port 3000
```

### Command Line Options

```
USAGE:
    cco-proxy [OPTIONS]

OPTIONS:
    --port <PORT>              Port to listen on (default: 8000)
    --db-path <PATH>          SQLite database path (default: ./cco.db)
    --config <PATH>           Config directory (default: ./config)
    --cache-size <MB>         Cache size in MB (default: 500)
    --cache-ttl <SECS>        Cache TTL in seconds (default: 3600)
    --log-level <LEVEL>       Log level: debug, info, warn, error (default: info)
    --workers <N>             Worker threads (default: CPU count)
    --bind <ADDR>             Bind address (default: 127.0.0.1)
    -h, --help                Print help
    -v, --version             Print version
```

### Environment Variables

```bash
# API Keys (required)
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."           # Optional
export GROQ_API_KEY="..."                # Optional
export LOCAL_API_KEY="..."               # Optional for local endpoints

# Proxy Configuration
export CCO_PORT=8000                     # Listen port
export CCO_LOG_LEVEL=info                # Log verbosity
export CCO_DB_PATH=/var/lib/cco.db       # Database location
export CCO_CACHE_SIZE=500                # MB
export CCO_CACHE_TTL=3600                # seconds
export CCO_BIND_ADDRESS=0.0.0.0          # Bind to all interfaces
export CCO_WORKERS=8                     # Thread count

# Feature Flags
export CCO_ENABLE_CACHE=true             # Enable/disable caching
export CCO_ENABLE_ANALYTICS=true         # Enable/disable analytics
export CCO_ENABLE_DASHBOARD=true         # Enable/disable dashboard
```

---

## Advanced Usage

### Pass Arguments to Claude Code

```bash
# Get Claude Code help
cco --help                    # Claude Code help (not CCO help)

# Analyze a file
cco analyze src/main.rs       # Analyze file

# Refactor code
cco refactor legacy.py --target modern

# Any Claude Code argument works
cco explain code.py
cco test integration_tests.rs
```

### Environment Variables

CCO automatically sets these when launching Claude Code:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
ORCHESTRATOR_API_URL=http://localhost:3000
```

**Note:** `$TMPDIR` is the OS temp directory:
- macOS: `/var/folders/xx/xxx/T/`
- Windows: `C:\Users\[user]\AppData\Local\Temp\`
- Linux: `/tmp/`

**Manual override** (advanced):

```bash
# Override API URL (for remote daemon)
export ORCHESTRATOR_API_URL=http://remote-server:3000
cco

# Disable orchestration temporarily
export ORCHESTRATOR_ENABLED=false
cco
```

### Multiple Daemons (Advanced)

Normally you run **one daemon per machine**. But you can run isolated daemons per project:

```bash
# Project 1: Use port 3000
cd ~/project-1
cco daemon start --port 3000
export ORCHESTRATOR_API_URL=http://localhost:3000
cco

# Project 2: Use port 3001
cd ~/project-2
cco daemon start --port 3001
export ORCHESTRATOR_API_URL=http://localhost:3001
cco
```

**Note:** This is **rarely needed**. One daemon can serve all projects.

### Remote Daemon Setup

For team development with shared daemon:

```bash
# Server: Start daemon (accessible to team)
ssh team-server
cco daemon start --host 0.0.0.0 --port 3000

# Developer workstation: Point to remote daemon
export ORCHESTRATOR_API_URL=http://team-server:3000
cco

# All developers share same daemon, VFS, and metrics
```

---

## Using CCO with Python

### Basic Example

```python
import anthropic

# Point to CCO instead of Claude API
client = anthropic.Anthropic(
    api_key="sk-ant-...",
    base_url="http://localhost:8000"
)

# Everything works the same
response = client.messages.create(
    model="claude-opus-4",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "Explain quantum computing"}
    ]
)

print(response.content[0].text)
```

### First Request vs Cached Request

```python
import anthropic
import time

client = anthropic.Anthropic(
    api_key="sk-ant-...",
    base_url="http://localhost:8000"
)

prompt = "Explain the theory of relativity in simple terms"

# First request - hits Claude API
start = time.time()
response1 = client.messages.create(
    model="claude-opus-4",
    max_tokens=1024,
    messages=[{"role": "user", "content": prompt}]
)
print(f"First request: {time.time() - start:.2f}s")  # ~1.5s

# Second request - served from cache
start = time.time()
response2 = client.messages.create(
    model="claude-opus-4",
    max_tokens=1024,
    messages=[{"role": "user", "content": prompt}]
)
print(f"Cached request: {time.time() - start:.2f}s")  # ~0.01s (100x faster!)

print(f"Identical responses: {response1.content == response2.content}")  # True
```

## Web Dashboard

### Accessing the Dashboard

The CCO dashboard provides real-time analytics, cost tracking, and cache management through a modern web interface.

#### Auto-Open on Startup

When you start CCO, the dashboard automatically opens in your default browser:

```bash
./cco-proxy
# Dashboard automatically opens at http://localhost:3000
```

#### Manual Access

If the dashboard doesn't auto-open:

```bash
# Option 1: Using open command (macOS/Linux)
open http://localhost:3000

# Option 2: Using xdg-open (Linux)
xdg-open http://localhost:3000

# Option 3: Paste into browser manually
# Navigate to: http://localhost:3000
```

### Dashboard Tabs

**Tab 1: Current Project**
- Real-time cost metrics with 24-hour trends
- Token usage (input and output)
- API call count and response time averages
- Cache hit rate and savings amount
- Recent activity timeline

**Tab 2: Machine-Wide Analytics**
- Total cost across all projects on this machine
- Number of active projects
- Cost breakdown by project
- Cost breakdown by model
- Usage distribution charts
- Projects table with activity timestamps

### Auto-Refresh Behavior

All dashboard data refreshes automatically every 5 seconds:
- Metrics update with new API requests
- Cache statistics refresh in real-time
- Charts update as data changes
- No manual refresh needed
- Connection status shows if data is live

### Dashboard Features

- **Real-time Metrics**: Live updates without page refresh
- **Cost Tracking**: See exactly what each model costs
- **Cache Statistics**: Monitor hit rate and savings
- **Project Isolation**: View metrics per project
- **Export Capability**: Download analytics as JSON or CSV

## Dashboard API

Access analytics via HTTP endpoints at `http://localhost:3000`

### Project Stats Endpoint

```bash
# Get current project metrics
curl http://localhost:8000/api/project/stats

# Response:
{
  "cost": 45.67,
  "costTrend": {"value": 5.2, "period": "24h"},
  "tokens": 123456,
  "tokensTrend": {"value": -2.1, "period": "24h"},
  "calls": 89,
  "callsTrend": {"value": 12.3, "period": "24h"},
  "avgTime": 245,
  "timeTrend": {"value": -8.5, "period": "24h"}
}
```

### Machine-Wide Stats Endpoint

```bash
# Get stats for all projects on this machine
curl http://localhost:8000/api/machine/stats

# Response:
{
  "totalCost": 1234.56,
  "activeProjects": 7,
  "totalCalls": 45678,
  "totalTokens": 12345678,
  "projects": [
    {
      "name": "project-a",
      "calls": 1234,
      "inputTokens": 50000,
      "outputTokens": 30000,
      "cost": 123.45,
      "lastActivity": "2024-11-15T10:30:00Z"
    }
  ],
  "models": [
    {
      "name": "claude-opus-4",
      "calls": 500,
      "inputTokens": 100000,
      "outputTokens": 50000,
      "cost": 456.78
    }
  ],
  "chartData": {
    "costOverTime": [...],
    "costByProject": [...],
    "modelDistribution": [...]
  }
}
```

### Cache Management

```bash
# View cache statistics
curl http://localhost:8000/api/cache/stats

# Response:
{
  "hits": 1234,
  "misses": 456,
  "hitRate": 0.73,
  "size": 256,
  "maxSize": 500,
  "entries": 89,
  "savedCost": 145.67
}

# Clear cache
curl -X POST http://localhost:8000/api/cache/clear

# Response:
{
  "status": "ok",
  "cleared": 89,
  "freedMemory": "256MB"
}

# View top cached prompts
curl http://localhost:8000/api/cache/top-prompts?limit=10

# Response:
[
  {
    "hash": "abc123...",
    "hits": 42,
    "savedCost": 12.34,
    "preview": "Explain the theory of..."
  }
]
```

### Health Check

```bash
# Check if CCO is running
curl http://localhost:8000/health

# Response:
{
  "status": "ok",
  "uptime": 3600,
  "database": "ok",
  "cache": "ok"
}
```

## Configuration Files

### model-routing.json

Define routing rules for different models.

```json
{
  "routes": [
    {
      "pattern": "^claude-",
      "provider": "anthropic",
      "endpoint": "https://api.anthropic.com/v1",
      "api_key_env": "ANTHROPIC_API_KEY",
      "timeout_ms": 60000,
      "max_retries": 3
    },
    {
      "pattern": "^gpt-",
      "provider": "openai",
      "endpoint": "https://api.openai.com",
      "api_key_env": "OPENAI_API_KEY",
      "timeout_ms": 60000,
      "max_retries": 3
    },
    {
      "pattern": "^ollama/",
      "provider": "ollama",
      "endpoint": "http://localhost:11434",
      "api_key_env": null,
      "timeout_ms": 120000,
      "max_retries": 2
    },
    {
      "pattern": "^local/",
      "provider": "openai",
      "endpoint": "http://localhost:8001",
      "api_key_env": "LOCAL_API_KEY",
      "timeout_ms": 120000,
      "max_retries": 2
    }
  ],
  "fallback_chain": {
    "claude-opus-4": ["claude-sonnet-3.5", "gpt-4"],
    "claude-sonnet-3.5": ["claude-haiku", "gpt-4-turbo"],
    "gpt-4": ["gpt-4-turbo", "claude-sonnet-3.5"]
  }
}
```

**Key Options:**
- `pattern`: Regex to match model names
- `provider`: anthropic, openai, ollama, openai (for local)
- `endpoint`: API base URL
- `timeout_ms`: Request timeout in milliseconds
- `max_retries`: Retry attempts on failure

### model-pricing.json

Define pricing and cost tracking.

```json
{
  "pricing": {
    "claude-opus-4": {
      "input": 15.0,
      "output": 75.0,
      "cache_read": 1.5,
      "cache_write": 18.75
    },
    "claude-sonnet-3.5": {
      "input": 3.0,
      "output": 15.0,
      "cache_read": 0.3,
      "cache_write": 3.75
    },
    "claude-haiku": {
      "input": 0.25,
      "output": 1.25,
      "cache_read": 0.05,
      "cache_write": 0.30
    },
    "gpt-4": {
      "input": 30.0,
      "output": 60.0
    },
    "ollama/llama3-70b": {
      "input": 0.0,
      "output": 0.0,
      "savings_comparison": "claude-sonnet-3.5"
    }
  }
}
```

**Price Format:**
- `input`: Cost per 1M input tokens (in cents)
- `output`: Cost per 1M output tokens (in cents)
- `cache_read`: Cost per 1M cache read tokens (optional)
- `cache_write`: Cost per 1M cache write tokens (optional)
- `savings_comparison`: Compare free models to this model

## Real-Time Streaming

### Server-Sent Events (SSE)

```bash
# Connect to analytics stream
curl -N http://localhost:8000/api/stream

# Continuous output:
event: analytics
data: {"cost": 45.67, "calls": 89, "tokens": 123456}

event: analytics
data: {"cost": 45.68, "calls": 90, "tokens": 123567}
```

## Data Export

### Export Analytics

```bash
# Export last 7 days as JSON
curl "http://localhost:8000/api/export/analytics?days=7" \
  > analytics.json

# Export as CSV
curl "http://localhost:8000/api/export/csv?days=30" \
  > analytics.csv

# Export specific project
curl "http://localhost:8000/api/export/project/my-project?format=json" \
  > project-analytics.json
```

### Export Format

```json
{
  "exportDate": "2024-11-15T10:30:00Z",
  "period": "2024-11-08T10:30:00Z to 2024-11-15T10:30:00Z",
  "summary": {
    "totalCost": 1234.56,
    "totalCalls": 45678,
    "totalTokens": 12345678,
    "cacheSavings": 345.67
  },
  "byModel": [...],
  "byProject": [...],
  "hourlyTrend": [...]
}
```

## Monitoring & Logging

### View Live Logs

```bash
# Follow logs in real-time
tail -f /var/log/cco/proxy.log

# Filter by level
grep "ERROR" /var/log/cco/proxy.log

# Search for specific request
grep "request-id-123" /var/log/cco/proxy.log
```

### Log Levels

```bash
# Debug (verbose, all requests)
./cco-proxy --log-level debug

# Info (key events)
./cco-proxy --log-level info

# Warn (potential issues)
./cco-proxy --log-level warn

# Error (only problems)
./cco-proxy --log-level error
```

### Prometheus Metrics

```bash
# Get metrics in Prometheus format
curl http://localhost:8000/metrics

# Response:
# HELP cco_requests_total Total API requests
# TYPE cco_requests_total counter
cco_requests_total{provider="anthropic",model="claude-opus-4"} 1234
cco_requests_total{provider="ollama",model="llama3-70b"} 567

# HELP cco_cache_hits_total Cache hit count
# TYPE cco_cache_hits_total counter
cco_cache_hits_total 4321

# HELP cco_cost_total Total cost in cents
# TYPE cco_cost_total counter
cco_cost_total 123456
```

## Performance Tuning

### Cache Configuration

```bash
# Increase cache size for larger workloads
./cco-proxy --cache-size 2000  # 2GB

# Adjust cache TTL (time-to-live)
./cco-proxy --cache-ttl 7200   # 2 hours

# Balance cache size and memory usage
export CCO_CACHE_SIZE=1000     # 1GB max
export CCO_CACHE_TTL=3600      # 1 hour TTL
```

### Database Optimization

```bash
# Compact database to reduce size
curl -X POST http://localhost:8000/api/database/compact

# Analyze query performance
curl http://localhost:8000/api/database/analyze

# Backup database
cp cco.db cco.db.backup
```

### Worker Threads

```bash
# Increase workers for high throughput
./cco-proxy --workers 16

# Check current performance
curl http://localhost:8000/metrics | grep worker
```

## Scheduled Maintenance

### Daily Cache Maintenance

```bash
# Run automatically at midnight
0 0 * * * /usr/local/bin/cco-maintenance --task cache-cleanup

# Or manually
curl -X POST http://localhost:8000/api/maintenance/cache-cleanup
```

### Weekly Database Optimization

```bash
# Run every Sunday at 2 AM
0 2 * * 0 /usr/local/bin/cco-maintenance --task database-vacuum

# Or manually
curl -X POST http://localhost:8000/api/maintenance/database-vacuum
```

### Monthly Analytics Archive

```bash
# Run first day of month at 1 AM
0 1 1 * * /usr/local/bin/cco-maintenance --task archive-analytics

# Specify retention period (keep 90 days, archive rest)
./cco-proxy --archive-retention 90
```

## Common Workflows

### Setup for Development Team

```bash
# 1. Start CCO on shared server
ssh team-server
./cco-proxy --port 8000 --bind 0.0.0.0

# 2. Team members point their clients
export ANTHROPIC_API_KEY="sk-ant-..."
export LLM_ENDPOINT="http://team-server:8000"

# 3. Monitor team usage
curl http://team-server:8000/api/machine/stats
```

### Setup with Docker Compose

```yaml
version: '3'
services:
  cco-proxy:
    image: cco-proxy:latest
    ports:
      - "8000:8000"
    environment:
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
      CCO_LOG_LEVEL: info
      CCO_CACHE_SIZE: 1000
    volumes:
      - cco-data:/var/lib/cco
      - ./config:/etc/cco
    restart: unless-stopped

volumes:
  cco-data:
```

Then run:
```bash
docker-compose up -d
export ANTHROPIC_API_KEY="sk-ant-..."
docker-compose up
```

### Monitor Cost in Real-Time

```bash
# Watch costs update every 2 seconds
watch -n 2 'curl -s http://localhost:8000/api/machine/stats | jq .totalCost'
```

## Troubleshooting Commands

```bash
# Test connection
curl -v http://localhost:8000/health

# Check cache status
curl http://localhost:8000/api/cache/stats

# View recent errors
curl http://localhost:8000/api/logs?level=error&limit=20

# Check database integrity
curl -X POST http://localhost:8000/api/database/check

# Reset analytics (careful!)
curl -X POST http://localhost:8000/api/analytics/reset
```

## Advanced: Custom Middleware

### Add Authentication

```bash
# Protect CCO with API key
./cco-proxy --auth-enabled --auth-key "secret123"

# Client usage
curl -H "X-API-Key: secret123" http://localhost:8000/api/stats
```

### Add Request Filtering

```bash
# Block certain models
./cco-proxy --blocked-models "gpt-4,claude-opus"

# Enforce minimum quality (models above threshold only)
./cco-proxy --min-model-quality "advanced"
```

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for more help.
