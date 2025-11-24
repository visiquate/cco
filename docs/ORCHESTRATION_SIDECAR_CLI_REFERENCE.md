# Orchestration Sidecar CLI Reference

**Version**: 1.0.0
**Date**: November 2025
**Audience**: Operators and developers using the CLI

## Table of Contents

1. [Overview](#overview)
2. [Server Commands](#server-commands)
3. [Context Commands](#context-commands)
4. [Results Commands](#results-commands)
5. [Event Commands](#event-commands)
6. [Agent Commands](#agent-commands)
7. [Configuration](#configuration)
8. [Environment Variables](#environment-variables)

---

## Overview

The orchestration sidecar provides CLI commands for server management, context operations, result storage, event publishing, and agent spawning.

### Command Structure

```bash
cco <command> <subcommand> [options]
```

### Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--help, -h` | Show help message | - |
| `--version, -v` | Show version | - |
| `--config <path>` | Configuration file | `~/.cco/config.json` |
| `--log-level <level>` | Log level (error\|warn\|info\|debug) | `info` |
| `--no-color` | Disable colored output | `false` |

---

## Server Commands

### `cco orchestration-server`

Start the orchestration sidecar server.

#### Usage

```bash
cco orchestration-server [OPTIONS]
```

#### Options

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `--port <PORT>` | integer | HTTP port to listen on | `3001` |
| `--host <HOST>` | string | Host address to bind | `127.0.0.1` |
| `--storage <PATH>` | string | Storage directory path | `/tmp/cco-sidecar` |
| `--cache-size-mb <SIZE>` | integer | Context cache size (MB) | `1024` |
| `--jwt-secret <SECRET>` | string | JWT signing secret | auto-generated |
| `--log-level <LEVEL>` | string | Log level | `info` |
| `--daemon` | flag | Run as background daemon | `false` |
| `--pid-file <PATH>` | string | PID file path (daemon mode) | `/var/run/cco-sidecar.pid` |

#### Examples

```bash
# Start with defaults
cco orchestration-server

# Start on custom port
cco orchestration-server --port 4000

# Start with larger cache
cco orchestration-server --cache-size-mb 2048

# Start as daemon
cco orchestration-server --daemon --pid-file /var/run/cco.pid

# Start with debug logging
cco orchestration-server --log-level debug

# Full configuration
cco orchestration-server \
  --port 3001 \
  --host 0.0.0.0 \
  --storage /var/lib/cco-sidecar \
  --cache-size-mb 2048 \
  --log-level info \
  --daemon
```

#### Output

```
Orchestration Sidecar v1.0.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Configuration:
  Host:         127.0.0.1
  Port:         3001
  Storage:      /tmp/cco-sidecar
  Cache Size:   1024 MB
  Log Level:    info

Starting server...
✓ Storage initialized
✓ Event bus started
✓ Context cache ready
✓ Server listening on http://127.0.0.1:3001

Ready to coordinate 119 agents
```

### `cco orchestration-server stop`

Stop the orchestration sidecar server.

#### Usage

```bash
cco orchestration-server stop [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--pid-file <PATH>` | PID file path | `/var/run/cco-sidecar.pid` |
| `--force` | Force kill if graceful shutdown fails | `false` |
| `--timeout <SECONDS>` | Graceful shutdown timeout | `30` |

#### Examples

```bash
# Graceful shutdown
cco orchestration-server stop

# Force shutdown if needed
cco orchestration-server stop --force

# Custom PID file
cco orchestration-server stop --pid-file /custom/path.pid
```

### `cco orchestration-server restart`

Restart the orchestration sidecar server.

#### Usage

```bash
cco orchestration-server restart [OPTIONS]
```

#### Examples

```bash
# Restart with current configuration
cco orchestration-server restart

# Restart with new configuration
cco orchestration-server restart --port 4000 --cache-size-mb 2048
```

### `cco orchestration-server status`

Check server status.

#### Usage

```bash
cco orchestration-server status
```

#### Output

```
Orchestration Sidecar Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Server:         Running
PID:            12345
Uptime:         2h 15m 30s
Port:           3001
Health:         Healthy

Agents:
  Active:       5
  Completed:    12
  Failed:       0

Storage:
  Cache:        45 entries (256 MB)
  Results:      12 stored (12.5 MB)
  Events:       156 in queue

Performance:
  Req/sec:      15.5
  Avg latency:  45ms
  P99 latency:  120ms
```

---

## Context Commands

### `cco context get`

Retrieve context for an agent.

#### Usage

```bash
cco context get <ISSUE_ID> <AGENT_TYPE> [OPTIONS]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ISSUE_ID>` | Yes | Issue/task identifier |
| `<AGENT_TYPE>` | Yes | Agent type |

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output <PATH>` | Save to file | stdout |
| `--format <FORMAT>` | Output format (json\|yaml\|text) | `json` |
| `--jwt-token <TOKEN>` | JWT authentication token | from env |
| `--sidecar-url <URL>` | Sidecar URL | `http://localhost:3001` |

#### Examples

```bash
# Get context for Python specialist
cco context get issue-123 python-specialist

# Save to file
cco context get issue-123 python-specialist --output context.json

# YAML format
cco context get issue-123 python-specialist --format yaml

# Custom sidecar URL
cco context get issue-123 python-specialist --sidecar-url http://remote:3001
```

#### Output

```json
{
  "issue_id": "issue-123",
  "agent_type": "python-specialist",
  "context": {
    "project_structure": {...},
    "relevant_files": [...],
    "git_context": {...},
    "metadata": {...}
  },
  "file_count": 12,
  "total_size_bytes": 102400
}
```

### `cco context refresh`

Refresh cached context.

#### Usage

```bash
cco context refresh [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--issue <ISSUE_ID>` | Specific issue (all if not provided) |
| `--agent-type <TYPE>` | Specific agent type (all if not provided) |
| `--force` | Force refresh even if cache is valid |

#### Examples

```bash
# Refresh all cached context
cco context refresh

# Refresh for specific issue
cco context refresh --issue issue-123

# Refresh for specific agent type
cco context refresh --agent-type python-specialist

# Force refresh
cco context refresh --issue issue-123 --force
```

### `cco context clear`

Clear cached context.

#### Usage

```bash
cco context clear [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--issue <ISSUE_ID>` | Specific issue (all if not provided) |
| `--agent-type <TYPE>` | Specific agent type (all if not provided) |
| `--all` | Clear entire cache |

#### Examples

```bash
# Clear cache for specific issue
cco context clear --issue issue-123

# Clear cache for agent type
cco context clear --agent-type python-specialist

# Clear entire cache
cco context clear --all
```

---

## Results Commands

### `cco results store`

Store agent results.

#### Usage

```bash
cco results store [OPTIONS]
```

#### Options

| Option | Required | Description |
|--------|----------|-------------|
| `--agent <ID>` | Yes | Agent ID |
| `--agent-type <TYPE>` | Yes | Agent type |
| `--issue <ID>` | Yes | Issue ID |
| `--project <ID>` | Yes | Project ID |
| `--status <STATUS>` | Yes | Status (success\|failed\|partial) |
| `--files-created <FILES>` | No | Comma-separated file paths |
| `--files-modified <FILES>` | No | Comma-separated file paths |
| `--decisions <TEXT>` | No | Semicolon-separated decisions |
| `--metrics <JSON>` | No | Metrics JSON |
| `--input <FILE>` | No | Read from JSON file |

#### Examples

```bash
# Store simple result
cco results store \
  --agent python-specialist-uuid \
  --agent-type python-specialist \
  --issue issue-123 \
  --project project-abc \
  --status success \
  --files-created "src/api.py,tests/test_api.py" \
  --decisions "Implemented REST API;Added tests"

# Store from JSON file
cco results store --input result.json

# With metrics
cco results store \
  --agent python-specialist-uuid \
  --agent-type python-specialist \
  --issue issue-123 \
  --project project-abc \
  --status success \
  --metrics '{"test_coverage": 95.0, "execution_time_ms": 4500}'
```

#### Input File Format (result.json)

```json
{
  "agent_id": "python-specialist-uuid",
  "agent_type": "python-specialist",
  "issue_id": "issue-123",
  "project_id": "project-abc",
  "result": {
    "status": "success",
    "files_created": ["src/api.py"],
    "decisions": ["Implemented REST API"],
    "metrics": {"test_coverage": 95.0}
  }
}
```

### `cco results list`

List stored results.

#### Usage

```bash
cco results list [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--issue <ISSUE_ID>` | Filter by issue | all |
| `--agent-type <TYPE>` | Filter by agent type | all |
| `--project <PROJECT_ID>` | Filter by project | all |
| `--format <FORMAT>` | Output format (table\|json\|csv) | `table` |
| `--limit <N>` | Limit results | `50` |

#### Examples

```bash
# List all results
cco results list

# Results for specific issue
cco results list --issue issue-123

# Results from Python specialists
cco results list --agent-type python-specialist

# JSON format
cco results list --issue issue-123 --format json

# CSV export
cco results list --format csv > results.csv
```

#### Output (Table Format)

```
Results for issue-123
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Agent Type           Status    Files Created    Decisions    Timestamp
──────────────────────────────────────────────────────────────────────
python-specialist    success   2                3            10:05:00
test-engineer        success   1                2            10:10:00
security-auditor     success   0                1            10:15:00
documentation-expert success   1                1            10:20:00

Total: 4 results
```

### `cco results get`

Get specific agent result.

#### Usage

```bash
cco results get <ISSUE_ID> <AGENT_TYPE> [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output <PATH>` | Save to file | stdout |
| `--format <FORMAT>` | Output format (json\|yaml\|text) | `json` |

#### Examples

```bash
# Get result
cco results get issue-123 python-specialist

# Save to file
cco results get issue-123 python-specialist --output result.json

# YAML format
cco results get issue-123 python-specialist --format yaml
```

---

## Event Commands

### `cco events publish`

Publish an event.

#### Usage

```bash
cco events publish [OPTIONS]
```

#### Options

| Option | Required | Description |
|--------|----------|-------------|
| `--type <TYPE>` | Yes | Event type |
| `--topic <TOPIC>` | Yes | Event topic |
| `--data <JSON>` | No | Event data (JSON) |
| `--correlation-id <ID>` | No | Correlation ID |
| `--ttl <SECONDS>` | No | Time-to-live (default: 86400) |
| `--input <FILE>` | No | Read from JSON file |

#### Examples

```bash
# Simple event
cco events publish \
  --type agent_completed \
  --topic implementation \
  --data '{"issue_id": "issue-123", "status": "success"}'

# With correlation
cco events publish \
  --type review_addressed \
  --topic implementation \
  --correlation-id review-session-123 \
  --data '{"changes": ["Added error handling"]}'

# From file
cco events publish --input event.json

# Custom TTL
cco events publish \
  --type custom_event \
  --topic testing \
  --ttl 3600 \
  --data '{"test": "data"}'
```

### `cco events subscribe`

Subscribe to events (long polling).

#### Usage

```bash
cco events subscribe <EVENT_TYPE> [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--timeout <MS>` | Polling timeout (milliseconds) | `30000` |
| `--filter <EXPR>` | Filter expression | none |
| `--format <FORMAT>` | Output format (json\|text) | `text` |
| `--continuous` | Keep subscribing | `false` |
| `--limit <N>` | Max events to receive | unlimited |

#### Examples

```bash
# Wait for single event
cco events subscribe agent_completed

# With timeout
cco events subscribe agent_completed --timeout 60000

# With filter
cco events subscribe agent_completed --filter "issue_id:issue-123"

# Continuous subscription
cco events subscribe agent_completed --continuous

# Limit events
cco events subscribe agent_completed --continuous --limit 10

# JSON output
cco events subscribe agent_completed --format json
```

#### Output (Text Format)

```
Waiting for events (timeout: 30s)...

[10:06:00] Event: agent_completed
  Publisher: python-specialist-uuid
  Topic:     implementation
  Data:      {"issue_id": "issue-123", "status": "success"}

[10:06:05] Event: agent_completed
  Publisher: test-engineer-uuid
  Topic:     testing
  Data:      {"issue_id": "issue-123", "coverage": 95.0}

Received 2 events
```

### `cco events list`

List recent events.

#### Usage

```bash
cco events list [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--topic <TOPIC>` | Filter by topic | all |
| `--type <TYPE>` | Filter by event type | all |
| `--limit <N>` | Number of events | `20` |
| `--since <TIME>` | Events since (ISO 8601) | none |
| `--format <FORMAT>` | Output format (table\|json\|csv) | `table` |

#### Examples

```bash
# List recent events
cco events list

# Implementation events
cco events list --topic implementation

# Specific event type
cco events list --type agent_completed

# Last 100 events
cco events list --limit 100

# Since timestamp
cco events list --since "2025-11-18T10:00:00Z"

# JSON format
cco events list --format json
```

#### Output

```
Recent Events (last 20)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Time     Type                  Topic           Publisher
──────────────────────────────────────────────────────────────
10:06:00 agent_completed       implementation  python-specialist
10:06:05 agent_completed       testing         test-engineer
10:06:10 agent_completed       security        security-auditor
10:06:15 deployment_complete   deployment      devops-engineer

Total: 4 events
```

---

## Agent Commands

### `cco agent spawn`

Spawn a new agent.

#### Usage

```bash
cco agent spawn [OPTIONS]
```

#### Options

| Option | Required | Description |
|--------|----------|-------------|
| `--type <TYPE>` | Yes | Agent type |
| `--issue <ID>` | Yes | Issue ID |
| `--task <DESCRIPTION>` | Yes | Task description |
| `--project <ID>` | No | Project ID (from context if not provided) |
| `--priority <LEVEL>` | No | Priority (low\|normal\|high) |
| `--env <KEY=VALUE>` | No | Environment variables (repeatable) |

#### Examples

```bash
# Spawn Python specialist
cco agent spawn \
  --type python-specialist \
  --issue issue-123 \
  --task "Implement JWT authentication"

# With priority
cco agent spawn \
  --type security-auditor \
  --issue issue-123 \
  --task "Audit new authentication code" \
  --priority high

# With environment variables
cco agent spawn \
  --type python-specialist \
  --issue issue-123 \
  --task "Implement feature X" \
  --env PYTHON_VERSION=3.11 \
  --env API_BASE_URL=http://localhost:8000

# Specify project
cco agent spawn \
  --type python-specialist \
  --issue issue-123 \
  --task "Fix bug" \
  --project project-abc
```

#### Output

```
Spawning agent...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Agent ID:     python-specialist-abc123
Agent Type:   python-specialist
Issue:        issue-123
Task:         Implement JWT authentication
Priority:     normal

✓ JWT token generated
✓ Context prepared
✓ Agent process spawned (PID: 12345)
✓ Webhook configured

Agent running at PID 12345
Monitor at: http://localhost:3001/api/agents/python-specialist-abc123/status
```

### `cco agent list`

List active agents.

#### Usage

```bash
cco agent list [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--status <STATUS>` | Filter by status (active\|completed\|failed) | all |
| `--type <TYPE>` | Filter by agent type | all |
| `--issue <ID>` | Filter by issue | all |
| `--format <FORMAT>` | Output format (table\|json\|csv) | `table` |

#### Examples

```bash
# List all agents
cco agent list

# Active agents only
cco agent list --status active

# Python specialists
cco agent list --type python-specialist

# Agents for issue
cco agent list --issue issue-123

# JSON format
cco agent list --format json
```

#### Output

```
Active Agents
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Agent Type           Issue       Status    PID     Uptime
──────────────────────────────────────────────────────────────
python-specialist    issue-123   active    12345   5m 30s
test-engineer        issue-123   active    12346   2m 15s
security-auditor     issue-124   active    12347   1m 45s

Total: 3 active agents
```

### `cco agent kill`

Stop an agent.

#### Usage

```bash
cco agent kill <AGENT_ID> [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--force` | Force kill (SIGKILL) | `false` |
| `--timeout <SECONDS>` | Graceful shutdown timeout | `30` |

#### Examples

```bash
# Graceful shutdown
cco agent kill python-specialist-abc123

# Force kill
cco agent kill python-specialist-abc123 --force

# Custom timeout
cco agent kill python-specialist-abc123 --timeout 60
```

---

## Configuration

### Configuration File

Default location: `~/.cco/config.json`

```json
{
  "sidecar": {
    "port": 3001,
    "host": "127.0.0.1",
    "storage_path": "/tmp/cco-sidecar",
    "cache_size_mb": 1024,
    "log_level": "info"
  },
  "agents": {
    "default_timeout_seconds": 3600,
    "max_concurrent": 10,
    "rate_limits": {
      "coding_specialists": 60,
      "documentation": 30
    }
  },
  "events": {
    "retention_seconds": 86400,
    "max_queue_size": 10000
  }
}
```

### Override Configuration

```bash
# Use custom config file
cco --config /path/to/config.json orchestration-server

# Override specific settings
cco orchestration-server --port 4000 --cache-size-mb 2048
```

---

## Environment Variables

### Server Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `CCO_SIDECAR_PORT` | HTTP port | `3001` |
| `CCO_SIDECAR_HOST` | Bind address | `127.0.0.1` |
| `CCO_SIDECAR_STORAGE` | Storage path | `/tmp/cco-sidecar` |
| `CCO_SIDECAR_CACHE_SIZE_MB` | Cache size (MB) | `1024` |
| `CCO_SIDECAR_JWT_SECRET` | JWT secret | auto-generated |
| `CCO_SIDECAR_LOG_LEVEL` | Log level | `info` |

### Agent Configuration

| Variable | Description |
|----------|-------------|
| `AGENT_ID` | Unique agent identifier |
| `AGENT_TYPE` | Agent type |
| `ISSUE_ID` | Issue/task identifier |
| `PROJECT_ID` | Project identifier |
| `SIDECAR_URL` | Sidecar API URL |
| `JWT_TOKEN` | Authentication token |

### Examples

```bash
# Start with environment variables
export CCO_SIDECAR_PORT=4000
export CCO_SIDECAR_CACHE_SIZE_MB=2048
cco orchestration-server

# Agent environment (auto-injected)
echo $AGENT_ID        # python-specialist-abc123
echo $SIDECAR_URL     # http://localhost:3001/api
echo $JWT_TOKEN       # eyJhbGciOiJIUzI1NiIs...
```

---

## See Also

- [Quick Start Guide](ORCHESTRATION_SIDECAR_QUICKSTART.md)
- [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
- [Agent Integration](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md)
- [Event System](ORCHESTRATION_SIDECAR_EVENTS.md)
- [Troubleshooting](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)
