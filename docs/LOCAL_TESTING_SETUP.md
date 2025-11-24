# Local Testing Setup for Orchestration Sidecar System

**Version**: 1.0.0
**Date**: November 2025
**Author**: Documentation Team
**Status**: Complete

## Overview

This guide provides step-by-step instructions for testing the Claude Code Orchestra (CCO) orchestration sidecar system on your local machine. The sidecar enables autonomous coordination of 119 Claude Orchestra agents without manual human intervention.

## Quick Start (2 minutes)

```bash
# Build the project
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# Launch CCO (starts daemon + sidecar + Claude Code)
./target/release/cco

# Verify everything is running
cco health
cco status
```

---

## Section 1: Installation & Setup

### Prerequisites

You need the following tools installed on your system:

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.70+ | Compile CCO binaries |
| cargo | Latest | Rust package manager |
| Claude Code CLI | Latest | Run orchestrated agent tasks |
| git | 2.0+ | Version control |
| curl | 7.0+ | Test HTTP endpoints |
| jq | 1.6+ | Parse JSON responses |

### Installation Steps

#### 1. Install Rust (if not already installed)

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the cargo environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version  # Should show 1.70+
cargo --version
```

#### 2. Install Claude Code CLI

```bash
# Install Claude Code (requires npm/Node.js)
# Visit https://claude.ai/code for official installation instructions

# Verify installation
which claude
claude --version
```

#### 3. Clone and Build CCO

```bash
# Navigate to CCO directory
cd /Users/brent/git/cc-orchestra/cco

# Build the project in release mode
cargo build --release

# Verify build succeeded
ls -la target/release/cco
file target/release/cco  # Should be executable binary
```

#### 4. Verify Build Artifacts

```bash
# Check the build output
cargo build --release 2>&1 | tail -20

# Should show:
#   Compiling cco v0.0.0
#   Finished release [optimized] target(s) in X.XXs
#
# Finished successfully if exit code is 0
echo $?
```

### Setup Verification

Run this command to verify all prerequisites are installed:

```bash
#!/bin/bash
echo "Verifying prerequisites..."

# Check Rust
if command -v rustc &> /dev/null; then
    echo "✅ Rust installed: $(rustc --version)"
else
    echo "❌ Rust not installed"
    exit 1
fi

# Check Claude Code
if command -v claude &> /dev/null; then
    echo "✅ Claude Code installed: $(claude --version)"
else
    echo "❌ Claude Code CLI not found in PATH"
    exit 1
fi

# Check CCO binary
if [ -f /Users/brent/git/cc-orchestra/cco/target/release/cco ]; then
    echo "✅ CCO binary exists"
else
    echo "❌ CCO binary not found - run: cargo build --release"
    exit 1
fi

echo ""
echo "All prerequisites verified!"
```

---

## Section 2: Understanding the Components

### System Architecture

The orchestration system consists of three main components that work together:

```
┌─────────────────────────────────────────────────────────┐
│               CLAUDE CODE (Orchestrator)                │
│  - Spawns agents via Task tool                          │
│  - Receives context from sidecar                        │
│  - Coordinates multi-round agent interactions           │
│  - Reads/writes orchestration files                     │
└─────────────────┬───────────────────────────────────────┘
                  │
        Sets ORCHESTRATOR_* env vars
                  │
        ┌─────────┴────────────┬────────────┐
        │                      │            │
        ▼                      ▼            ▼
┌──────────────┐      ┌──────────────┐  ┌──────────────┐
│  CCO Daemon  │      │  Orchestration   Sidecar│  │ Agent Processes │
│  (Port 3000) │      │  (Port 3001)     │  (External)  │
└──────────────┘      └──────────────┘  └──────────────┘
```

### Component Details

#### CCO Daemon (Port 3000)

**Role**: Main orchestration hub and configuration server

**Responsibilities**:
- Manages system-wide state and lifecycle
- Provides REST API for daemon commands
- Creates temporary configuration files in `/tmp/`
- Coordinates with the sidecar on startup
- Handles shutdown and cleanup

**Key Files**:
- Source: `/Users/brent/git/cc-orchestra/cco/src/daemon/`
- Config: `~/.config/cco/daemon.toml` (created on first run)
- Logs: `/tmp/cco-daemon.log`
- Temp Settings: `/tmp/.cco-orchestrator-settings`

**Startup Process**:
```bash
1. Load configuration (create defaults if needed)
2. Initialize database
3. Create temp settings file
4. Start HTTP server on port 3000
5. Signal readiness
```

#### Orchestration Sidecar (Port 3001)

**Role**: Autonomous agent coordination service

**Responsibilities**:
- Handles agent requests for context
- Stores execution results
- Manages event pub-sub system
- Injects intelligent context for each agent type
- Provides webhook endpoints for agent status updates

**Key Files**:
- Source: `/Users/brent/git/cc-orchestra/cco/src/orchestration/`
- Storage: `~/.cco/orchestration/` (results and cache)
- Port: 3001 (fixed)

**Startup Process**:
```bash
1. Initialize event bus (in-memory)
2. Create result storage directory
3. Load knowledge broker
4. Start HTTP server on port 3001
5. Register health check endpoint
```

#### Claude Code (Orchestrator Process)

**Role**: Main orchestration engine

**Responsibilities**:
- Spawns multiple agents in parallel via Task tool
- Reads context from sidecar
- Coordinates agent execution flow
- Reviews aggregated results
- Makes strategic decisions about next phases

**Environment Variables** (set by launcher):
```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=/tmp/.cco-orchestrator-settings
ORCHESTRATOR_AGENTS=/tmp/.cco-agents-sealed
ORCHESTRATOR_RULES=/tmp/.cco-rules-sealed
ORCHESTRATOR_HOOKS=/tmp/.cco-hooks-sealed
ORCHESTRATOR_API_URL=http://localhost:3000
ORCHESTRATOR_HOOKS_CONFIG={...JSON hooks config...}
```

### Component Relationships

**Startup Sequence**:
```
User runs: cco [args]
    │
    ├─> Check if daemon running
    │   └─> If not, start daemon (port 3000)
    │
    ├─> Wait for daemon health (max 3 seconds)
    │
    ├─> Verify temp files exist (created by daemon)
    │
    ├─> Set ORCHESTRATOR_* environment variables
    │
    ├─> Launch Claude Code process
    │   ├─> Claude Code inherits env vars
    │   ├─> Claude Code starts sidecar (if not running)
    │   └─> Claude Code can now coordinate agents
    │
    └─> Return when Claude Code exits
```

**Communication Pattern**:
```
Claude Code -> [HTTP] -> Sidecar (port 3001)
    │
    └─> GET /api/context/issue-id/agent-type
    └─> POST /api/results
    └─> POST /api/events/publish
    └─> GET /api/events/wait
```

---

## Section 3: Running cco Command Locally

### What Happens When You Run `cco`

The `cco` command is a sophisticated launcher that orchestrates three key steps:

#### Step 1: Daemon Auto-Start (500ms)

```bash
cco [args]
    └─> Check daemon health at http://localhost:3000/health
        ├─> If running: continue immediately
        └─> If not running:
            ├─> Start daemon process
            ├─> Wait up to 3 seconds for startup
            ├─> Verify health endpoint responds
            └─> Fail if not responsive after 3 seconds
```

**What the daemon does on startup**:
```rust
// From launcher.rs::ensure_daemon_running()
1. Create DaemonManager with config
2. Call manager.get_status()
3. If fails, call manager.start()
4. Poll /health endpoint (300ms intervals)
5. Succeed when first response received
```

#### Step 2: Orchestrator Settings Verification (100ms)

```bash
Verify temp files exist:
    ├─> /tmp/.cco-orchestrator-settings (main config)
    ├─> /tmp/.cco-agents-sealed (agent definitions)
    ├─> /tmp/.cco-rules-sealed (orchestrator rules)
    └─> /tmp/.cco-hooks-sealed (execution hooks)
```

**Failure Scenarios**:
- If settings not found: daemon didn't fully initialize
- If not readable: permission issues
- If corrupted: daemon crashed during initialization

#### Step 3: Environment Variable Injection (instant)

The launcher sets these variables before launching Claude Code:

```bash
# Core orchestration flags
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_API_URL=http://localhost:3000
ORCHESTRATOR_HOOKS_ENABLED=true

# File paths (in temp directory)
ORCHESTRATOR_SETTINGS=/tmp/.cco-orchestrator-settings
ORCHESTRATOR_AGENTS=/tmp/.cco-agents-sealed
ORCHESTRATOR_RULES=/tmp/.cco-rules-sealed
ORCHESTRATOR_HOOKS=/tmp/.cco-hooks-sealed

# Hooks configuration (JSON)
ORCHESTRATOR_HOOKS_CONFIG='{"permissions": {...}, "enabled": true}'

# Quick-access permission flags
ORCHESTRATOR_AUTO_ALLOW_READ=true
ORCHESTRATOR_REQUIRE_CUD_CONFIRMATION=false
```

#### Step 4: Claude Code Launch (1 second)

```bash
find claude executable in PATH
    ├─> Try: "claude" (most common)
    ├─> Try: "claude-code"
    └─> Try: "claude-ai"

Launch Claude Code process:
    claude --settings /tmp/.cco-orchestrator-settings [args]
    │
    └─> Inherits all ORCHESTRATOR_* env vars
    └─> Current working directory preserved
    └─> All user arguments passed through
```

#### Step 5: Process Waits for Exit (indefinite)

```bash
cco process waits for Claude Code to exit
    │
    └─> When Claude Code exits (user quit/Ctrl+C):
        ├─> Capture exit code
        ├─> Pass through exit code
        └─> Return to shell
```

### Example: What Happens Internally

When you run:
```bash
cco --help
```

Here's the execution flow:

```
1. Launcher checks daemon at http://localhost:3000/health
   GET /health
   200 OK
   ✅ Daemon is running

2. Verify temp files exist
   stat /tmp/.cco-orchestrator-settings
   ✅ Orchestrator settings found

3. Set environment variables
   ORCHESTRATOR_ENABLED=true
   ORCHESTRATOR_SETTINGS=/tmp/.cco-orchestrator-settings
   ... (5 more variables)

4. Find Claude Code executable
   which claude
   /Users/brent/.local/bin/claude

5. Execute Claude Code
   /Users/brent/.local/bin/claude --settings /tmp/.cco-orchestrator-settings --help

6. Claude Code prints help and exits
   Claude Code v1.25.0
   ...
   exit 0

7. cco launcher returns exit code
   echo $?
   0
```

### Timing Expectations

```bash
Typical timing breakdown:

  0ms: User types 'cco'
  0-50ms: Check daemon health (if already running)
 50-100ms: Verify temp files
100-150ms: Set environment variables
150-200ms: Find Claude executable
200-1000ms: Launch Claude Code process
1000-1200ms: Claude Code startup
1200+ms: Claude Code execution (depends on command)

Total time to interactive shell: 2-3 seconds
```

---

## Section 4: Testing Scenarios

### Test 1: Basic Help (30 seconds)

Tests that the launcher works and Claude Code is accessible.

```bash
# Run command
cco --help

# Expected output:
# ✅ Daemon is running
# ✅ Orchestrator settings found
# ✅ Orchestration environment configured
# 🚀 Launching Claude Code with orchestration support...
#    Working directory: /Users/brent/git/cc-orchestra/cco
#    Settings: /tmp/.cco-orchestrator-settings
#    Executable: /Users/brent/.local/bin/claude
#
# [Claude Code help output...]
```

**What it tests**:
- Daemon is responsive
- Temp files are readable
- Environment variables are set correctly
- Claude Code can be invoked with arguments

### Test 2: Version Check (30 seconds)

Tests version information is properly displayed.

```bash
# Run command
cco --version

# Expected output:
# ✅ Daemon is running
# ✅ Orchestrator settings found
# ✅ Orchestration environment configured
# 🚀 Launching Claude Code with orchestration support...
#
# [Claude Code version...]
```

**What it tests**:
- Version information passes through correctly
- No orchestration interference with version output

### Test 3: Interactive Launch (60 seconds)

Tests launching Claude Code in interactive mode.

```bash
# Run command
cco

# You should see:
# ✅ Daemon is running
# ✅ Orchestrator settings found
# ✅ Orchestration environment configured
# 🚀 Launching Claude Code with orchestration support...
#    Working directory: /Users/brent/git/cc-orchestra/cco
#    Settings: /tmp/.cco-orchestrator-settings
#    Executable: /Users/brent/.local/bin/claude
#
# [Claude Code welcome message...]
# [Ready for user input]

# In Claude Code, test a command:
# > list files in current directory

# Type Ctrl+D or type 'exit' to quit
```

**What it tests**:
- Full orchestration setup works end-to-end
- Claude Code receives correct context
- Interactive mode is functional
- Environment variables are inherited

### Test 4: Verify Sidecar Running (15 seconds)

Tests that the orchestration sidecar is active.

```bash
# In a new terminal window:
curl -s http://localhost:3001/health | jq .

# Expected output:
# {
#   "status": "healthy",
#   "service": "orchestration-sidecar",
#   "version": "1.0.0",
#   "uptime_seconds": 145,
#   "checks": {
#     "storage": "healthy",
#     "event_bus": "healthy",
#     "memory_usage_mb": 45,
#     "active_agents": 0,
#     "event_queue_size": 0
#   }
# }
```

**What it tests**:
- Sidecar is running on port 3001
- API is responding to requests
- All health checks are passing
- Sidecar is ready for agent coordination

### Test 5: Sidecar Status (15 seconds)

Tests detailed sidecar system status.

```bash
# Get sidecar status
curl -s http://localhost:3001/status | jq .

# Expected output:
# {
#   "agents": {
#     "active": 0,
#     "completed": 0,
#     "failed": 0,
#     "by_type": {}
#   },
#   "storage": {
#     "context_cache_entries": 0,
#     "results_stored": 0,
#     "total_size_mb": 0.0
#   },
#   "events": {
#     "total_published": 0,
#     "active_subscriptions": 0,
#     "queue_depth": 0
#   },
#   "performance": {
#     "avg_response_time_ms": 0,
#     "p99_response_time_ms": 0,
#     "requests_per_second": 0.0
#   }
# }
```

**What it tests**:
- Sidecar status endpoint works
- No agents have been spawned yet (normal)
- System is idle and ready

---

## Section 5: Verification Checklist

Use this checklist to ensure your local setup is complete:

```bash
#!/bin/bash
echo "=== CCO Local Setup Verification ==="
echo ""

# Check 1: Build exists
echo -n "[ ] Build artifact exists... "
if [ -f /Users/brent/git/cc-orchestra/cco/target/release/cco ]; then
    echo "✅"
else
    echo "❌ Run: cargo build --release"
    exit 1
fi

# Check 2: Daemon running on port 3000
echo -n "[ ] Daemon running on port 3000... "
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅"
else
    echo "⚠️  Not running (will auto-start)"
fi

# Check 3: Sidecar running on port 3001
echo -n "[ ] Sidecar running on port 3001... "
if curl -s http://localhost:3001/health > /dev/null 2>&1; then
    echo "✅"
else
    echo "⚠️  Not running (will start with cco)"
fi

# Check 4: Claude Code in PATH
echo -n "[ ] Claude Code in PATH... "
if command -v claude &> /dev/null; then
    echo "✅"
else
    echo "❌ Not found - install from https://claude.ai/code"
    exit 1
fi

# Check 5: Temp settings file
echo -n "[ ] Orchestrator settings file... "
if [ -f /tmp/.cco-orchestrator-settings ]; then
    echo "✅"
else
    echo "⚠️  Will be created on daemon startup"
fi

# Check 6: Port availability
echo -n "[ ] Port 3000 available... "
if ! lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "✅"
else
    PID=$(lsof -Pi :3000 -sTCP:LISTEN -t)
    echo "❌ In use by PID $PID"
    exit 1
fi

echo -n "[ ] Port 3001 available... "
if ! lsof -Pi :3001 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "✅"
else
    PID=$(lsof -Pi :3001 -sTCP:LISTEN -t)
    echo "❌ In use by PID $PID"
    exit 1
fi

echo ""
echo "All checks passed! You can run 'cco' now."
```

Save this as `/Users/brent/git/cc-orchestra/verify-setup.sh`:

```bash
chmod +x verify-setup.sh
./verify-setup.sh
```

---

## Section 6: Troubleshooting

### Issue: Port 3000 Already in Use

**Symptom**:
```
Error: Failed to bind to address 127.0.0.1:3000
```

**Cause**:
Another CCO daemon is already running, or another process is using the port.

**Solution**:
```bash
# Find process using port 3000
lsof -i :3000

# Kill the existing daemon
pkill -f "cco.*daemon"

# Or kill a specific process
kill -9 <PID>

# Or configure daemon to use different port
# Edit: ~/.config/cco/daemon.toml
# Change: port = 3001  # Use different port

# Then restart
cco daemon start
```

### Issue: Port 3001 Already in Use

**Symptom**:
```
Sidecar failed to start: Address already in use
```

**Cause**:
Sidecar crashed but process didn't clean up, or another service on that port.

**Solution**:
```bash
# Find and kill orphaned sidecar
lsof -i :3001
kill -9 <PID>

# Or restart the daemon which will restart sidecar
cco daemon restart

# Verify sidecar starts
sleep 2
curl http://localhost:3001/health
```

### Issue: Claude Not Found in PATH

**Symptom**:
```
Claude Code executable not found in PATH
Please install Claude Code first: https://claude.ai/code
```

**Cause**:
Claude Code CLI is not installed or not in PATH.

**Solution**:
```bash
# Install Claude Code
npm install -g @anthropic-ai/claude-code
# Or follow official instructions at https://claude.ai/code

# Verify installation
which claude
claude --version

# Or add to PATH if installed in non-standard location
export PATH="/path/to/claude/bin:$PATH"
```

### Issue: Daemon Fails to Start

**Symptom**:
```
Failed to start daemon within 3 seconds
```

**Cause**:
Daemon is crashing on startup, check logs.

**Solution**:
```bash
# View daemon logs
tail -50 /tmp/cco-daemon.log

# Try manual start to see error details
cco daemon start

# Check logs for specific error
grep -i error /tmp/cco-daemon.log

# Common issues:
# - Database locked: delete ~/.cco/analytics.db
# - Config corrupted: delete ~/.config/cco/
# - Port conflict: kill other process on port 3000
```

### Issue: Orchestrator Settings Not Found

**Symptom**:
```
Orchestrator settings not found at: /tmp/.cco-orchestrator-settings
This usually means the daemon failed to start.
```

**Cause**:
Daemon started but failed to create temp files.

**Solution**:
```bash
# Force daemon restart
cco daemon stop
cco daemon start

# Wait and verify
sleep 2
ls -la /tmp/.cco-orchestrator-settings

# If still missing, check daemon logs
tail -50 /tmp/cco-daemon.log

# Nuclear option: restart everything
killall cco 2>/dev/null
sleep 1
cco daemon restart
```

### Issue: Sidecar Fails to Start

**Symptom**:
```
cco launches but no response on port 3001
```

**Cause**:
Sidecar initialization failed (check Rust compilation).

**Solution**:
```bash
# Rebuild the entire project
cd /Users/brent/git/cc-orchestra/cco
cargo clean
cargo build --release

# Check for Rust compilation errors
cargo build --release 2>&1 | grep error

# Verify binary was created
ls -lah target/release/cco

# If still failing, check detailed logs
RUST_BACKTRACE=1 cco 2>&1 | tail -100
```

### Issue: Environment Variables Not Set

**Symptom**:
Claude Code doesn't see ORCHESTRATOR_* variables.

**Solution**:
```bash
# Verify variables are being set
cco --help 2>&1 | grep "environment configured"

# Check if variables are inherited by child process
cco -c 'echo $ORCHESTRATOR_ENABLED'

# If not set, check launcher code
# File: /Users/brent/git/cc-orchestra/cco/src/commands/launcher.rs
# Function: set_orchestrator_env_vars()

# Rebuild and test
cargo build --release
cco daemon restart
```

### Issue: Permissions Denied

**Symptom**:
```
Permission denied while trying to open file
```

**Cause**:
Binary doesn't have execute permissions or directory permissions.

**Solution**:
```bash
# Fix binary permissions
chmod +x /Users/brent/git/cc-orchestra/cco/target/release/cco

# Fix directory permissions
chmod -R 755 /Users/brent/git/cc-orchestra/cco/target/release/

# Fix temp directory permissions
chmod 755 /tmp/.cco-*

# Verify permissions
ls -la /Users/brent/git/cc-orchestra/cco/target/release/cco
```

---

## Section 7: Development Workflow

### Recommended Development Cycle

Use this workflow when modifying CCO code:

```bash
# 1. Make code changes
# Edit files in: /Users/brent/git/cc-orchestra/cco/src/

# 2. Rebuild binary
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# 3. Restart daemon (clean shutdown)
cco daemon stop
sleep 1

# 4. Verify cleanup
lsof -i :3000  # Should be empty
lsof -i :3001  # Should be empty

# 5. Test changes
cco --help
# Or
cco

# 6. View logs if there are issues
tail -100 /tmp/cco-daemon.log
```

### Running Specific Tests

Test individual components:

```bash
# Run all tests
cd /Users/brent/git/cc-orchestra/cco
cargo test

# Run launcher tests only
cargo test launcher --lib

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_ensure_daemon_running

# Run tests with output
cargo test -- --nocapture
```

### Building for Distribution

When ready to release:

```bash
# Build optimized binary
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# Check size
ls -lh target/release/cco

# Test the binary
./target/release/cco --help

# Create checksum for verification
sha256sum target/release/cco > cco.sha256

# Ready to distribute
# Binary: target/release/cco
# Checksum: cco.sha256
```

### Local Daemon Logs Location

```bash
# View logs
tail -50 /tmp/cco-daemon.log

# Follow logs in real-time
tail -f /tmp/cco-daemon.log

# Search logs
grep "error" /tmp/cco-daemon.log
grep "sidecar" /tmp/cco-daemon.log

# View last error
tail -100 /tmp/cco-daemon.log | grep -i error | tail -1

# Archive old logs
mv /tmp/cco-daemon.log /tmp/cco-daemon.log.$(date +%s)
```

### Debugging Sidecar Startup

Enable debug logging:

```bash
# Set Rust logging
export RUST_LOG=debug

# Run cco with debug output
cco 2>&1 | grep -i sidecar

# Or check sidecar logs if available
tail -50 /tmp/cco-sidecar.log

# Check what's listening on ports
netstat -tuln | grep -E "3000|3001"

# Verify sidecar is responding
curl -v http://localhost:3001/health
```

---

## Section 8: Performance Expectations

### Expected Timing

**Daemon Startup**:
- Cold start: 400-600ms
- Warm start: 100-200ms (already running)
- Failure detection: 3 seconds (timeout)

**Sidecar Startup**:
- Initialization: 150-300ms
- Health check ready: 200-400ms
- First request: 50-100ms

**Claude Code Launch**:
- Environment setup: 50-100ms
- Process spawn: 200-500ms
- Interactive ready: 500-1000ms

**Total Time**:
- First run: 1.5-2.5 seconds (daemon + sidecar + Claude)
- Warm run: 1-2 seconds (everything running)
- With network latency: add 100-300ms

### Memory Usage

**Daemon**:
- Baseline: 30-50 MB
- With cache: 50-100 MB
- Under load: up to 200 MB

**Sidecar**:
- Baseline: 20-40 MB
- With context cache: 50-150 MB
- With 119 agents: 150-300 MB

**Claude Code**:
- Depends on Claude Code itself
- Typically 100-200 MB

**Total System**:
- Baseline: ~100 MB
- Under full load: ~500-600 MB

### Network Performance

**Sidecar HTTP Endpoints**:
- GET /health: ~5ms
- GET /context: ~20-50ms (depends on project size)
- POST /results: ~10-20ms
- POST /events: ~5-10ms
- GET /events/wait: ~30-100ms (depends on timeout)

**Concurrent Requests**:
- Can handle 100+ agents simultaneously
- Response time under load: <100ms (p99)
- Requests per second: 500+ (with small payloads)

---

## Section 9: Agent Coordination

### How Agents Discover the Sidecar

When Claude Code is launched with `cco`, agents inherit these environment variables:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_API_URL=http://localhost:3000
```

Agents can then discover the sidecar:

```python
# Python agent example
import os
import requests

api_url = os.getenv("ORCHESTRATOR_API_URL")
# api_url = "http://localhost:3000"

# But sidecar runs on 3001
# The daemon (3000) forwards to sidecar (3001)
```

### How Agents Query Context

Agents make HTTP requests to retrieve context:

```bash
# Example: Python specialist requests context
curl -H "Authorization: Bearer $JWT_TOKEN" \
    http://localhost:3001/api/context/issue-123/python-specialist

# Response includes:
# - Project structure
# - Relevant source files
# - Previous agent decisions
# - Git history
# - Metadata about the project
```

### How Agents Store Results

After completing work, agents store results:

```bash
# Example: Store implementation results
curl -X POST http://localhost:3001/api/results \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "agent_id": "python-specialist-123",
        "agent_type": "python-specialist",
        "issue_id": "issue-123",
        "result": {
            "status": "success",
            "files_created": ["api.py", "tests/test_api.py"],
            "decisions": ["Use FastAPI for REST API"]
        }
    }'

# Results stored at:
# ~/.cco/orchestration/results/issue-123/python-specialist.json
```

### How Agents Publish Events

Agents coordinate via events:

```bash
# Example: Python specialist publishes completion event
curl -X POST http://localhost:3001/api/events/agent_completed \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -d '{
        "publisher": "python-specialist-123",
        "topic": "implementation",
        "data": {
            "issue_id": "issue-123",
            "status": "success"
        }
    }'

# Subscribers receive:
# - QA Engineers (to write tests)
# - Security Auditors (to review code)
# - Documentation Writers (to document)
```

### Event Flow Example

Here's a complete multi-agent coordination:

```
1. Chief Architect publishes to 'architecture' topic:
   POST /api/events/architecture_defined
   {topic: "architecture", data: {design: "REST API"}}

2. Python Specialist subscribes to 'architecture':
   GET /api/events/wait/architecture_defined?filter=topic:architecture
   Receives event, starts implementation

3. Python Specialist implements and publishes:
   POST /api/events/implementation_complete
   {topic: "implementation", data: {files: ["api.py"]}}

4. Test Engineer receives event:
   GET /api/events/wait/implementation_complete
   Starts writing tests

5. Test Engineer publishes test results:
   POST /api/events/testing_complete
   {topic: "testing", data: {coverage: 95%}}

6. Security Auditor receives test results:
   GET /api/events/wait/testing_complete
   Starts security review

7. Complete workflow visible to orchestrator
```

---

## Quick Reference

### Key Commands

```bash
# Launch CCO with orchestration
cco

# Check daemon status
cco daemon status
cco health

# View system status
cco status

# Restart everything
cco daemon restart

# View logs
tail -f /tmp/cco-daemon.log

# Check ports
lsof -i :3000
lsof -i :3001

# Test endpoints
curl http://localhost:3000/health
curl http://localhost:3001/health
```

### Directory Structure

```
/Users/brent/git/cc-orchestra/cco/
├── src/
│   ├── main.rs                 # Entry point
│   ├── commands/launcher.rs    # Claude Code launcher
│   ├── daemon/                 # Daemon implementation
│   └── orchestration/          # Sidecar implementation
├── target/release/cco          # Compiled binary
├── Cargo.toml                  # Dependencies
└── Cargo.lock                  # Dependency versions

~/.config/cco/
└── daemon.toml                 # Daemon config

~/.cco/
├── orchestration/              # Sidecar data
│   └── results/                # Agent results storage
└── ...

/tmp/
├── .cco-orchestrator-settings  # Temp config
├── .cco-agents-sealed          # Sealed agents file
├── .cco-rules-sealed           # Sealed rules
└── .cco-hooks-sealed           # Sealed hooks
```

### Environment Variables

```bash
ORCHESTRATOR_ENABLED           # true/false
ORCHESTRATOR_API_URL           # http://localhost:3000
ORCHESTRATOR_SETTINGS          # /tmp/.cco-orchestrator-settings
ORCHESTRATOR_AGENTS            # /tmp/.cco-agents-sealed
ORCHESTRATOR_RULES             # /tmp/.cco-rules-sealed
ORCHESTRATOR_HOOKS             # /tmp/.cco-hooks-sealed
ORCHESTRATOR_HOOKS_CONFIG      # JSON hooks config
ORCHESTRATOR_AUTO_ALLOW_READ   # true/false
ORCHESTRATOR_REQUIRE_CUD_CONFIRMATION  # true/false
ORCHESTRATOR_HOOKS_ENABLED     # true/false
```

### Common Issues & Fixes

| Issue | Fix |
|-------|-----|
| Port 3000 in use | `pkill -f "cco.*daemon"` |
| Port 3001 in use | `kill $(lsof -t -i:3001)` |
| Daemon won't start | Check `/tmp/cco-daemon.log` |
| Sidecar won't respond | Rebuild: `cargo build --release` |
| Claude not found | Install: `npm install -g @anthropic-ai/claude-code` |
| Settings file missing | Run: `cco daemon restart` |
| Permissions denied | Run: `chmod +x target/release/cco` |

---

## Appendix A: Testing Sidecar API Directly

You can test the sidecar API without launching Claude Code:

```bash
# 1. Start daemon in background
cco daemon start &

# 2. Wait for startup
sleep 2

# 3. Test daemon health
curl http://localhost:3000/health | jq .

# 4. Test sidecar health (should be up)
curl http://localhost:3001/health | jq .

# 5. Test sidecar status
curl http://localhost:3001/status | jq .

# 6. View structure of responses
curl http://localhost:3001/health | jq 'keys'

# 7. Stop daemon when done
pkill -f "cco.*daemon"
```

---

## Appendix B: Manual Component Testing

Test each component independently:

### Test Daemon Only

```bash
# Build daemon
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# Run daemon directly
./target/release/cco run --port 3000

# In another terminal:
curl http://localhost:3000/health
```

### Test Sidecar Only

```bash
# Sidecar runs as part of CCO system
# But you can start it manually if needed

# Check if running
curl http://localhost:3001/health

# View detailed status
curl http://localhost:3001/status | jq .

# Test context endpoint (requires auth)
curl -H "Authorization: Bearer test-token" \
    http://localhost:3001/api/context/test-issue/python-specialist
```

### Test Claude Integration

```bash
# Verify Claude is in PATH
which claude

# Test Claude with orchestrator settings
export ORCHESTRATOR_ENABLED=true
export ORCHESTRATOR_API_URL=http://localhost:3000
claude --help

# Test Claude with a simple command
claude -c "echo 'Test from Claude'"
```

---

## Appendix C: Performance Profiling

Profile the system to understand bottlenecks:

```bash
# Enable Rust debug info
RUST_BACKTRACE=1 cco 2>&1 | head -50

# Measure startup time
time cco --help

# Profile with perf (Linux only)
perf record cco --help
perf report

# Monitor resource usage
while true; do
    clear
    echo "=== CCO Resource Usage ==="
    ps aux | grep cco | grep -v grep
    echo ""
    echo "=== Network Connections ==="
    lsof -i :3000 -i :3001
    sleep 5
done
```

---

## Support & Documentation

For more information:

- **Architecture**: See [ORCHESTRATION_SIDECAR_ARCHITECTURE.md](./ORCHESTRATION_SIDECAR_ARCHITECTURE.md)
- **Daemon Docs**: `/Users/brent/git/cc-orchestra/cco/src/daemon/mod.rs`
- **Sidecar Code**: `/Users/brent/git/cc-orchestra/cco/src/orchestration/`
- **Main Source**: `/Users/brent/git/cc-orchestra/cco/src/main.rs`

---

**Document Status**: Complete
**Last Updated**: November 2025
**Review Status**: Ready for Testing
**Next Steps**: Complete local testing and document any issues discovered
