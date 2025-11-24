# CCO Architecture Overview (User Guide)

**Audience:** CCO users who want to understand how the system works

This document explains CCO's architecture from a user perspective, focusing on components you interact with and how they work together.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow](#data-flow)
4. [FUSE VFS Explained](#fuse-vfs-explained)
5. [Security Model](#security-model)
6. [Performance Characteristics](#performance-characteristics)

---

## System Overview

CCO consists of three main components that work together to provide orchestration for Claude Code:

```
┌─────────────────────────────────────────────────────┐
│  CLI (cco)                                          │
│  Your entry point for all CCO operations            │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│  Daemon (Background Service)                        │
│  - HTTP API Server (port 3000)                      │
│  - FUSE VFS (/var/run/cco/)                         │
│  - Agent coordination                               │
│  - Metrics collection                               │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│  Claude Code (with Orchestration)                   │
│  - Reads agent definitions from VFS                 │
│  - Coordinates 119 specialized agents               │
│  - Reports metrics back to daemon                   │
└─────────────────────────────────────────────────────┘
```

---

## Component Architecture

### 1. CLI (Command Line Interface)

**What it is:** The `cco` command you type in your terminal

**What it does:**
- Launches Claude Code with orchestration support
- Manages daemon lifecycle (start/stop/restart)
- Displays monitoring dashboard (TUI)
- Provides access to configuration and logs

**How you use it:**
```bash
cco                  # Launch Claude Code
cco tui              # Launch monitoring dashboard
cco daemon start     # Start daemon
cco daemon status    # Check daemon status
```

**How it works:**
1. Checks if daemon is running (auto-starts if needed)
2. Verifies VFS is mounted and healthy
3. Sets environment variables (`ORCHESTRATOR_*`)
4. Launches Claude Code in your current directory

### 2. Daemon (Background Service)

**What it is:** A long-running background process that provides orchestration services

**What it does:**
- Serves HTTP API on port 3000 (default)
- Mounts FUSE VFS at `/var/run/cco/`
- Routes requests to appropriate agents
- Collects metrics and analytics
- Caches API responses

**When it runs:**
- Automatically starts when you run `cco` (if not already running)
- Continues running in background
- Survives across multiple Claude Code sessions
- Can be configured to start on boot (optional)

**How it works:**
```
Daemon Components:
├─ HTTP Server (Axum)
│  ├─ /health endpoint
│  ├─ /api/project/stats
│  ├─ /api/machine/stats
│  └─ /api/cache/stats
│
├─ FUSE VFS
│  ├─ Mount at /var/run/cco/
│  ├─ Serve sealed agent files
│  └─ Provide health status
│
├─ Request Router
│  ├─ Pattern matching
│  ├─ Agent selection
│  └─ Load balancing
│
└─ Metrics Engine
   ├─ Request tracking
   ├─ Cost calculation
   └─ Cache statistics
```

### 3. FUSE VFS (Virtual Filesystem)

**What it is:** A virtual filesystem mounted at `/var/run/cco/` that looks like regular files but is served by the daemon

**Why it exists:** To provide a secure, read-only view of agent definitions and orchestration rules without storing them on disk

**What's in it:**
```
/var/run/cco/
├── agents.sealed              # 119 agent definitions (encrypted)
├── orchestrator.sealed         # Orchestration rules (encrypted)
├── hooks.sealed                # Pre/post-compaction hooks (encrypted)
├── .manifest                   # Version metadata (plaintext JSON)
├── health                      # VFS health check (plaintext: "OK")
└── README.txt                  # VFS explanation (plaintext)
```

**How it works:**
- Files appear as regular filesystem entries
- Read operations are intercepted by daemon
- Daemon unseals and decrypts on-the-fly
- No sensitive data stored on disk
- Files disappear when daemon stops

**User experience:**
```bash
# VFS looks like regular files
$ ls -la /var/run/cco/
-r--r--r-- 1 user user 123456 Nov 17 10:00 agents.sealed
-r--r--r-- 1 user user   1234 Nov 17 10:00 orchestrator.sealed

# Can read with normal tools
$ cat /var/run/cco/health
OK

# But files are served by daemon, not stored on disk
$ cco daemon stop
$ ls /var/run/cco/
ls: /var/run/cco/: No such file or directory
```

### 4. Claude Code Integration

**What it is:** Claude Code running with orchestration enabled

**How it detects CCO:**
1. Checks `ORCHESTRATOR_ENABLED=true` environment variable
2. Reads agent definitions from `ORCHESTRATOR_AGENTS` path
3. Unseals and parses agent configuration
4. Generates dynamic `CLAUDE.md` in memory
5. Uses `ORCHESTRATOR_API_URL` for agent communication

**What changes:**
- Claude Code coordinates with 119 specialized agents
- Requests routed through daemon for caching/routing
- Metrics reported back to daemon
- Agent definitions dynamically loaded from VFS

---

## Data Flow

### Launching Claude Code

```
User types: cco
    │
    ├─→ CLI checks daemon status
    │   ├─→ If running: proceed
    │   └─→ If not running: auto-start daemon
    │
    ├─→ CLI verifies VFS mounted
    │   └─→ Check /var/run/cco/health == "OK"
    │
    ├─→ CLI sets environment variables
    │   ├─→ ORCHESTRATOR_ENABLED=true
    │   ├─→ ORCHESTRATOR_VFS_MOUNT=/var/run/cco
    │   ├─→ ORCHESTRATOR_AGENTS=/var/run/cco/agents.sealed
    │   ├─→ ORCHESTRATOR_RULES=/var/run/cco/orchestrator.sealed
    │   └─→ ORCHESTRATOR_API_URL=http://localhost:3000
    │
    └─→ CLI launches Claude Code
        └─→ Claude Code reads VFS for agent definitions
            └─→ Orchestration enabled!
```

### API Request Flow

```
User asks question in Claude Code
    │
    ├─→ Claude Code determines agent needed
    │   └─→ Example: "Chief Architect" for design decisions
    │
    ├─→ Request sent to daemon API
    │   └─→ POST http://localhost:3000/api/agent/invoke
    │       {
    │         "agent": "Chief Architect",
    │         "task": "Design authentication flow",
    │         "context": {...}
    │       }
    │
    ├─→ Daemon checks cache
    │   ├─→ Cache hit: return cached response (instant)
    │   └─→ Cache miss: proceed to routing
    │
    ├─→ Daemon routes to Claude API
    │   ├─→ Model: claude-sonnet-4.5 (per agent config)
    │   ├─→ Prompt: augmented with agent instructions
    │   └─→ Request sent to api.anthropic.com
    │
    ├─→ Daemon receives response
    │   ├─→ Cache the response
    │   ├─→ Record metrics (cost, tokens, latency)
    │   └─→ Return to Claude Code
    │
    └─→ Claude Code displays response to user
        └─→ User sees agent's answer
```

### Monitoring Flow

```
User runs: cco tui
    │
    ├─→ TUI connects to daemon API
    │   └─→ GET http://localhost:3000/api/project/stats
    │
    ├─→ Daemon queries metrics database
    │   ├─→ Current session metrics
    │   ├─→ Cache statistics
    │   └─→ Recent activity
    │
    ├─→ TUI displays real-time dashboard
    │   ├─→ Cost: $45.67
    │   ├─→ Tokens: 123,456
    │   ├─→ Cache hit rate: 73%
    │   └─→ Recent requests
    │
    └─→ Auto-refresh every 5 seconds
        └─→ User sees live updates
```

---

## FUSE VFS Explained

### What is FUSE?

**FUSE (Filesystem in Userspace)** allows programs to create virtual filesystems without kernel code.

**CCO uses FUSE to:**
- Present agent definitions as files
- Encrypt sensitive data
- Provide read-only access
- Auto-unmount when daemon stops

### Why use FUSE VFS?

**Security:**
- Agent definitions never stored unencrypted on disk
- Files are generated on-the-fly when read
- Automatic cleanup when daemon stops

**Simplicity:**
- Claude Code reads files like normal
- No special unsealing logic needed
- Standard file operations work

**Flexibility:**
- Easy to update agent definitions (restart daemon)
- Version metadata in `.manifest`
- Health checks via `/health` file

### VFS File Details

#### agents.sealed

**Content:** Encrypted JSON with 119 agent definitions

**Structure (after unsealing):**
```json
{
  "leadership": [
    {
      "name": "Chief Architect",
      "type": "system-architect",
      "model": "opus-4.1",
      "capabilities": ["architecture", "design", "coordination"],
      "description": "Strategic decision-making and system design"
    }
  ],
  "codingAgents": [...],
  "testingAgents": [...],
  ...
}
```

**How it's used:**
- Claude Code reads this file on startup
- Parses agent definitions
- Generates dynamic `CLAUDE.md`
- Enables agent coordination

#### orchestrator.sealed

**Content:** Encrypted orchestration rules and configuration

**Structure (after unsealing):**
```json
{
  "rules": {
    "taskRouting": {
      "architecture": "Chief Architect",
      "implementation": ["TDD Coding Agent", "Language Specialists"],
      "testing": ["Test Engineer", "QA Engineer"],
      "security": ["Security Auditor", "Security Engineer"]
    },
    "coordination": {
      "parallelExecution": true,
      "maxConcurrentAgents": 10,
      "knowledgeSharing": true
    }
  }
}
```

**How it's used:**
- Daemon uses rules to route tasks
- Claude Code uses for agent selection
- Knowledge Manager uses for coordination

#### .manifest

**Content:** Plaintext JSON with version metadata

**Structure:**
```json
{
  "version": "2025.11.17",
  "buildCommit": "abc123def456",
  "buildDate": "2025-11-17T10:30:00Z",
  "agentCount": 119,
  "checksums": {
    "agents.sealed": "sha256:...",
    "orchestrator.sealed": "sha256:...",
    "hooks.sealed": "sha256:..."
  }
}
```

**How it's used:**
- Version verification
- Integrity checks
- Debugging and diagnostics

#### health

**Content:** Plaintext "OK" or error message

**How it's used:**
```bash
# CLI checks health before launching
$ cat /var/run/cco/health
OK

# If unhealthy:
$ cat /var/run/cco/health
ERROR: VFS mount failed

# Restart daemon to fix:
$ cco daemon restart
```

---

## Security Model

### Threat Model

CCO protects against:
- ✅ **Disk theft**: Agent definitions encrypted, keys in memory only
- ✅ **File tampering**: Checksums in `.manifest` verify integrity
- ✅ **Unauthorized access**: VFS read-only, daemon controls access
- ✅ **Credential leakage**: API keys never logged or persisted

CCO does NOT protect against:
- ❌ **Memory dumps**: Keys exist in daemon memory
- ❌ **Root access**: Root user can read all files
- ❌ **Network sniffing**: Use HTTPS in production

### Encryption Details

**Sealed Files (agents.sealed, etc.):**
- **Algorithm**: AES-256-GCM
- **Key derivation**: HMAC-SHA256
- **Compression**: Gzip before encryption
- **Authentication**: GCM tag prevents tampering

**Unsealing Process:**
1. Read encrypted file from VFS
2. Verify GCM authentication tag
3. Decrypt using AES-256-GCM
4. Decompress with gzip
5. Parse JSON
6. Return to caller

**Key Management:**
- Master key generated at daemon startup
- Stored in daemon memory only
- Never written to disk
- Lost when daemon stops (intentional)

### File Permissions

```bash
# VFS files are read-only
$ ls -la /var/run/cco/
-r--r--r-- 1 user user 123456 Nov 17 10:00 agents.sealed
-r--r--r-- 1 user user   1234 Nov 17 10:00 orchestrator.sealed

# Attempting to write fails
$ echo "bad" > /var/run/cco/agents.sealed
-bash: /var/run/cco/agents.sealed: Read-only file system
```

---

## Performance Characteristics

### Latency

| Operation | Latency | Notes |
|-----------|---------|-------|
| `cco` launch (daemon running) | <1s | Fast path |
| `cco` launch (daemon not running) | 3-4s | Auto-start overhead |
| VFS file read | <5ms | In-memory unsealing |
| Daemon API call | <10ms | Local HTTP |
| Cache hit | <5ms | In-memory cache |
| Cache miss | 500-2000ms | Depends on Claude API |

### Memory Usage

| Component | Memory | Notes |
|-----------|--------|-------|
| Daemon (idle) | ~50MB | Base memory |
| Daemon (active) | ~200MB | With cache |
| VFS | ~10MB | Agent definitions in memory |
| TUI | ~20MB | Dashboard UI |
| Claude Code | Varies | Independent process |

### Disk Usage

| Component | Disk | Notes |
|-----------|------|-------|
| Binary | ~15MB | Single executable |
| Database | ~100MB | Analytics (grows over time) |
| Logs | ~50MB | Rotated daily |
| VFS | **0MB** | Virtual filesystem (no disk storage) |

### Network Usage

| Operation | Bandwidth | Notes |
|-----------|-----------|-------|
| Agent API call | ~1-5KB | Request to daemon |
| Claude API call | ~10-100KB | Depends on prompt size |
| Dashboard refresh | ~1KB | Metrics update |
| Cache enabled | -50-90% | Reduces API calls |

### CPU Usage

| Component | CPU | Notes |
|-----------|-----|-------|
| Daemon (idle) | <1% | Minimal background |
| Daemon (active) | 5-10% | Processing requests |
| VFS unsealing | <1% | Fast AES-GCM |
| TUI rendering | 2-5% | Dashboard updates |

---

## Summary

**Key takeaways:**

1. **Three components:** CLI, Daemon, Claude Code
2. **FUSE VFS:** Virtual filesystem for secure agent storage
3. **Automatic setup:** CLI handles daemon startup and environment
4. **Security:** Encryption at rest, read-only VFS, integrity checks
5. **Performance:** Fast local caching, minimal overhead

**For users:**
- Just run `cco` - everything else is automatic
- Monitor with `cco tui` in separate terminal
- VFS at `/var/run/cco/` is managed by daemon
- Daemon persists across sessions for fast startup

**For troubleshooting:**
- Check daemon: `cco daemon status`
- Check VFS: `cat /var/run/cco/health`
- Check logs: `cco daemon logs`
- Restart: `cco daemon restart`
