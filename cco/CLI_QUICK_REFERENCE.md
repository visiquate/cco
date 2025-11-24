# CCO CLI Quick Reference

**After macFUSE installation and Phase 4 completion**

---

## 🚀 Basic Commands

```bash
# Launch Claude Code with orchestration
cco

# Launch TUI monitoring dashboard
cco tui

# Start daemon manually
cco daemon start

# Stop daemon
cco daemon stop

# Check daemon status
cco daemon status

# View daemon logs
cco daemon logs
```

---

## 🎯 Pass-Through Arguments

```bash
# Claude Code help
cco --help

# Analyze a file
cco analyze src/main.rs

# Any Claude Code command
cco [claude-code-arguments]
```

---

## 📋 Command Routing

| You Type | What Runs |
|----------|-----------|
| `cco` | Claude Code launcher |
| `cco tui` | TUI dashboard |
| `cco daemon [...]` | Daemon management |
| `cco server [...]` | Server management |
| `cco version` | CCO version |
| `cco --help` | CCO help |
| `cco [other]` | Claude Code with args |

---

## ⚡ Quick Workflows

### Developer Workflow

```bash
# Terminal 1: Development
cd ~/my-project
cco

# Terminal 2: Monitoring
cco tui
```

### Daemon Management

```bash
# Start
cco daemon start

# Stop
cco daemon stop

# Restart
cco daemon restart

# Status
cco daemon status

# Logs (follow)
cco daemon logs --follow

# Install as service
cco daemon install
cco daemon enable
```

---

## 🔍 Debugging

```bash
# Check daemon status
cco daemon status

# View logs
cco daemon logs --lines 100

# Check VFS health
cat /var/run/cco/health
# Should output: OK

# List VFS files
ls -la /var/run/cco/

# Check if daemon is listening
lsof -i :3000

# View daemon process
ps aux | grep cco
```

---

## 🌍 Environment Variables

These are **automatically set** when you run `cco`:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_VFS_MOUNT=/var/run/cco
ORCHESTRATOR_AGENTS=/var/run/cco/agents.sealed
ORCHESTRATOR_RULES=/var/run/cco/orchestrator.sealed
ORCHESTRATOR_HOOKS=/var/run/cco/hooks.sealed
ORCHESTRATOR_MANIFEST=/var/run/cco/.manifest
ORCHESTRATOR_API_URL=http://localhost:3000
```

---

## ❌ Common Errors

### "Daemon not running"

```bash
# Auto-starts automatically
cco

# Or manually start
cco daemon start
```

### "VFS not mounted"

```bash
# Restart daemon
cco daemon restart

# Check VFS
cat /var/run/cco/health
```

### "Claude Code not found"

```bash
# Install Claude Code
# Visit: https://claude.ai/code

# Verify in PATH
which claude
```

### "Port 3000 already in use"

```bash
# Find conflicting process
lsof -i :3000

# Kill it or use different port
cco daemon start --port 3001
```

---

## 📊 Performance

- **Daemon auto-start**: < 3 seconds
- **Claude Code launch** (daemon running): < 500ms
- **VFS health check**: < 100ms
- **TUI startup**: < 500ms

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run launcher tests
cargo test launcher

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_launch_claude_code
```

---

## 📚 Documentation

- `CLI_ENHANCEMENTS_SUMMARY.md` - This implementation
- `CLI_ENHANCEMENTS_STATUS.md` - Detailed status
- `CLI_TESTING_QUICK_START.md` - Test procedures
- `docs/FUSE_VFS_CLI_ENHANCEMENTS.md` - Full spec

---

## 🔐 Security

- VFS files are **read-only** (permissions: 444)
- Environment variables contain **no secrets**
- Daemon binds to **localhost only** (127.0.0.1)
- FUSE mount is **user-scoped** (/var/run/cco)

---

**Version**: Phase 1-3 Complete
**Last Updated**: 2025-11-17
