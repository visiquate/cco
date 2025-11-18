# Orchestration Sidecar Quick Start Guide

**Version**: 1.0.0
**Date**: November 2025
**Audience**: Developers and operators getting started with the orchestration sidecar

## What is the Orchestration Sidecar?

The Orchestration Sidecar is an autonomous HTTP server that coordinates the 119 agents in the Claude Orchestra system. It runs alongside the CCO daemon on port 3001 and provides three critical services:

1. **Context Injection** - Gathers and delivers relevant project context to each agent
2. **Event Coordination** - Enables agents to communicate via pub-sub messaging
3. **Result Storage** - Persists agent outputs for review and handoff

Think of it as the **nervous system** that allows 119 specialized agents to work together autonomously on complex development tasks.

## Why Agents Need It

Without the sidecar, agents would:
- **Lack context** about what other agents have done
- **Work in isolation** without coordination
- **Duplicate effort** by not sharing results
- **Require manual coordination** from the orchestrator

With the sidecar, agents automatically:
- **Receive relevant context** based on their role
- **Coordinate via events** (e.g., "architecture defined", "tests passing")
- **Share results** that other agents can build upon
- **Work autonomously** for hours without human intervention

## System Architecture at a Glance

```
┌─────────────────────┐
│   Claude Code       │  Spawns agents, monitors progress
│  (Orchestrator)     │
└──────────┬──────────┘
           │ HTTP API
           ▼
┌─────────────────────┐
│ Orchestration       │  Port 3001 - Context, Events, Results
│    Sidecar          │
└──────────┬──────────┘
           │ Coordinates
           ▼
┌─────────────────────┐
│   119 Agents        │  Python Specialist, Security Auditor, etc.
│   Working in        │
│    Parallel         │
└─────────────────────┘
```

## Prerequisites

Before launching the sidecar, ensure you have:

- **CCO installed** - The sidecar is part of the CCO binary
- **Port 3001 available** - The sidecar listens on `http://localhost:3001`
- **Project directory** - A Git repository for agents to work on
- **Rust 1.70+** - If building from source

## How to Launch the Sidecar

### Method 1: Standalone Launch (Development)

```bash
# Launch the orchestration sidecar
cco orchestration-server

# Server starts on http://localhost:3001
# Output:
# Orchestration Sidecar v1.0.0
# Listening on http://127.0.0.1:3001
# Ready to coordinate 119 agents
```

### Method 2: With CCO Daemon (Production)

The sidecar automatically starts when you launch the CCO daemon:

```bash
# Start the daemon (includes sidecar)
cco daemon start

# Verify sidecar is running
curl http://localhost:3001/health

# Expected output:
# {
#   "status": "healthy",
#   "service": "orchestration-sidecar",
#   "version": "1.0.0",
#   "uptime_seconds": 42
# }
```

### Method 3: Docker Deployment

```bash
# Docker run with sidecar enabled
docker run -p 3000:3000 -p 3001:3001 \
  -v $(pwd):/workspace \
  ghcr.io/visiquate/cco:latest \
  cco daemon start
```

## First Agent Example

Let's walk through spawning your first agent and seeing how it interacts with the sidecar.

### Step 1: Start the Sidecar

```bash
# Terminal 1: Launch sidecar
cco orchestration-server
```

### Step 2: Spawn an Agent (Python Specialist)

```bash
# Terminal 2: Spawn Python Specialist for issue-123
cco agent spawn \
  --type python-specialist \
  --issue issue-123 \
  --task "Add JWT authentication to the API"
```

### Step 3: Watch the Agent Work

The agent automatically:

1. **Requests context** from the sidecar:
   ```
   GET http://localhost:3001/api/context/issue-123/python-specialist
   ```

2. **Receives relevant files**:
   - All `.py` files
   - Test files
   - `requirements.txt`
   - Previous architectural decisions

3. **Implements the feature** based on context

4. **Stores results** back to the sidecar:
   ```
   POST http://localhost:3001/api/results
   {
     "agent_type": "python-specialist",
     "files_created": ["src/auth.py", "tests/test_auth.py"],
     "decisions": ["Implemented JWT with PyJWT library"]
   }
   ```

5. **Publishes completion event**:
   ```
   POST http://localhost:3001/api/events/implementation
   {
     "event_type": "implementation_complete",
     "next_phase": "testing"
   }
   ```

### Step 4: Spawn a Test Engineer (Automatic)

The sidecar automatically suggests the next agent:

```bash
# The sidecar responds with:
{
  "next_agents": ["test-engineer", "security-auditor"]
}

# Claude Code spawns them automatically
```

### Step 5: View Results

```bash
# Check stored results
cco results list --issue issue-123

# Output:
# issue-123:
#   - python-specialist: JWT authentication implemented
#   - test-engineer: Tests passing (95% coverage)
#   - security-auditor: No vulnerabilities found
```

## Verifying the Sidecar is Running

### Health Check

```bash
curl http://localhost:3001/health
```

**Healthy Response:**
```json
{
  "status": "healthy",
  "service": "orchestration-sidecar",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "checks": {
    "storage": "healthy",
    "event_bus": "healthy",
    "memory_usage_mb": 150,
    "active_agents": 5,
    "event_queue_size": 12
  }
}
```

### System Status

```bash
curl http://localhost:3001/status
```

**Response:**
```json
{
  "agents": {
    "active": 5,
    "completed": 12,
    "failed": 0,
    "by_type": {
      "python-specialist": 2,
      "test-engineer": 1,
      "security-auditor": 1,
      "documentation-expert": 1
    }
  },
  "storage": {
    "context_cache_entries": 45,
    "results_stored": 12,
    "total_size_mb": 25.5
  },
  "events": {
    "total_published": 156,
    "active_subscriptions": 8,
    "queue_depth": 3
  }
}
```

## Configuration Options

### Environment Variables

```bash
# Set custom port (default: 3001)
export CCO_SIDECAR_PORT=4000

# Set storage directory (default: /tmp/cco-sidecar)
export CCO_SIDECAR_STORAGE=/var/lib/cco-sidecar

# Set context cache size (default: 1GB)
export CCO_SIDECAR_CACHE_SIZE_MB=2048

# Set JWT secret (auto-generated if not set)
export CCO_SIDECAR_JWT_SECRET=your-secret-key

# Set log level (default: info)
export CCO_SIDECAR_LOG_LEVEL=debug
```

### Command Line Flags

```bash
# Launch with custom configuration
cco orchestration-server \
  --port 4000 \
  --storage /custom/path \
  --cache-size-mb 2048 \
  --log-level debug
```

## Troubleshooting Basics

### Issue: Sidecar won't start

```bash
# Check if port 3001 is already in use
lsof -i :3001

# If blocked, kill the process or use a different port
cco orchestration-server --port 4000
```

### Issue: Agents can't connect

```bash
# Verify sidecar is listening
curl http://localhost:3001/health

# Check firewall rules
sudo ufw status
sudo ufw allow 3001

# Verify network connectivity
ping localhost
```

### Issue: Context is missing

```bash
# Check context cache status
curl http://localhost:3001/status | jq '.storage'

# Clear context cache
curl -X DELETE http://localhost:3001/api/cache/context/issue-123

# Force context refresh
cco context refresh --issue issue-123
```

### Issue: Events not delivered

```bash
# Check event queue depth
curl http://localhost:3001/status | jq '.events.queue_depth'

# If queue is full (> 10,000), restart sidecar
cco orchestration-server restart

# View event log
cco events list --topic implementation --limit 20
```

### Issue: High memory usage

```bash
# Check memory usage
curl http://localhost:3001/health | jq '.checks.memory_usage_mb'

# If > 1GB, reduce cache size
export CCO_SIDECAR_CACHE_SIZE_MB=512
cco orchestration-server restart

# Clear all caches
curl -X DELETE http://localhost:3001/api/cache/all
```

## Common Commands

### Launch and Monitor

```bash
# Start sidecar
cco orchestration-server

# Monitor logs
tail -f /tmp/cco-sidecar/logs/orchestration.log

# Watch agent activity
watch -n 1 'curl -s http://localhost:3001/status | jq ".agents"'
```

### Context Management

```bash
# Get context for an agent
cco context get issue-123 python-specialist

# Refresh context cache
cco context refresh --issue issue-123

# Clear context cache
cco context clear --issue issue-123
```

### Event Management

```bash
# Publish an event
cco events publish --type custom_event --topic testing

# Subscribe to events
cco events subscribe --topic implementation --timeout 30

# List recent events
cco events list --limit 20
```

### Results Management

```bash
# Store agent results
cco results store --agent python-specialist --issue issue-123

# List all results for an issue
cco results list --issue issue-123

# Get specific agent result
cco results get --agent python-specialist --issue issue-123
```

## Next Steps

Now that you have the sidecar running and understand the basics:

1. **Read the API Reference** - Learn all 8 HTTP endpoints
   → [ORCHESTRATION_SIDECAR_API_REFERENCE.md](ORCHESTRATION_SIDECAR_API_REFERENCE.md)

2. **Integrate with Agents** - Build agents that use the sidecar
   → [ORCHESTRATION_SIDECAR_AGENT_GUIDE.md](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md)

3. **Learn the Event System** - Coordinate multi-agent workflows
   → [ORCHESTRATION_SIDECAR_EVENTS.md](ORCHESTRATION_SIDECAR_EVENTS.md)

4. **Explore CLI Tools** - Master the command-line interface
   → [ORCHESTRATION_SIDECAR_CLI_REFERENCE.md](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md)

5. **Advanced Topics** - Architecture, scaling, and performance
   → [ORCHESTRATION_SIDECAR_ADVANCED.md](ORCHESTRATION_SIDECAR_ADVANCED.md)

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Documentation Index**: [ORCHESTRATION_SIDECAR_INDEX.md](ORCHESTRATION_SIDECAR_INDEX.md)
- **FAQ**: [ORCHESTRATION_SIDECAR_FAQ.md](ORCHESTRATION_SIDECAR_FAQ.md)
- **Troubleshooting**: [ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)

---

**Congratulations!** You now have a running orchestration sidecar and understand how agents use it to coordinate autonomously. The sidecar is the key technology that enables 119 agents to work together seamlessly on complex development tasks.
