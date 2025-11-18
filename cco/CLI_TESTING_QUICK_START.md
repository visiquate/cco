# CLI Enhancements Testing Quick Start

**For**: Test Engineer, QA, Security Auditor
**Status**: Ready for testing after macFUSE installation

---

## Prerequisites

### 1. Install macFUSE

```bash
# Install via Homebrew
brew install --cask macfuse

# Follow prompts:
# 1. Enter sudo password
# 2. Go to System Settings → Privacy & Security
# 3. Click "Allow" for macFUSE kernel extension
# 4. Reboot if required
```

### 2. Verify Installation

```bash
# Check if macFUSE is installed
ls /Library/Filesystems/macfuse.fs/

# Expected output: Contents directory exists
```

### 3. Build CCO

```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
cargo test
```

---

## Manual Testing Checklist

### Test 1: Basic Launch (Happy Path)

```bash
# Prerequisites: Daemon NOT running
cco daemon stop

# Test: Launch Claude Code
cd /tmp
cco

# Expected behavior:
# 1. ⚙️  Starting daemon...
# 2. ✅ Daemon started
# 3. ✅ VFS mounted and healthy
# 4. ✅ Orchestration environment configured
# 5. 🚀 Launching Claude Code with orchestration support...
# 6. Claude Code starts (or shows "not found" error if not installed)

# Verify environment variables are set (check Claude Code process)
ps aux | grep claude
# (Claude Code process should have ORCHESTRATOR_* env vars)
```

**Expected Time**: < 3 seconds for daemon auto-start

---

### Test 2: Launch with Daemon Already Running

```bash
# Prerequisites: Daemon running
cco daemon start

# Test: Launch Claude Code
cd ~/projects/test
cco

# Expected behavior:
# 1. ✅ Daemon is running (instant check)
# 2. ✅ VFS mounted and healthy
# 3. ✅ Orchestration environment configured
# 4. 🚀 Launching Claude Code...

# Verify: No daemon startup delay
```

**Expected Time**: < 500ms (no daemon startup)

---

### Test 3: TUI Launch

```bash
# Prerequisites: None
cco tui

# If daemon not running:
# 1. ⚠️  Daemon is not running
# 2. Start daemon now? [Y/n]
#    (Press Y or Enter)
# 3. ⚙️  Starting daemon...
# 4. ✅ Daemon started successfully
# 5. 🎯 Launching TUI dashboard...
# 6. TUI appears

# If daemon running:
# 1. ✅ Daemon is running
# 2. 🎯 Launching TUI dashboard...
# 3. TUI appears
```

**Expected Time**: < 3 seconds with auto-start, < 500ms without

---

### Test 4: Pass-Through Arguments

```bash
# Test: Claude Code --help
cco --help

# Expected: Claude Code help message (not CCO help)
# Verify: Daemon auto-starts if needed
# Verify: Environment variables set

# Test: Claude Code analysis
cco analyze /tmp/test.txt

# Expected: Claude Code runs analysis
# Verify: Current directory preserved
```

---

### Test 5: Error Scenarios

#### 5a. Daemon Start Failure (Port Conflict)

```bash
# Block port 3000
nc -l 3000 &
NC_PID=$!

# Try to launch
cco

# Expected error:
# ⚙️  Starting daemon...
# ❌ Failed to start daemon: Address already in use (os error 48)

# Cleanup
kill $NC_PID
```

#### 5b. VFS Not Mounted

```bash
# This test requires manual daemon modification to disable VFS
# Skip for now - tested in integration tests
```

#### 5c. Claude Code Not Found

```bash
# Temporarily rename Claude Code binary
which claude && sudo mv $(which claude) $(which claude).bak

# Try to launch
cco

# Expected error:
# ✅ Daemon is running
# ✅ VFS mounted and healthy
# ✅ Orchestration environment configured
# ❌ Claude Code executable not found in PATH
#    Please install Claude Code first:
#    https://claude.ai/code

# Restore Claude Code
sudo mv $(which claude).bak $(which claude)
```

---

### Test 6: Multiple Sessions

```bash
# Terminal 1: Launch Claude Code
cd ~/project1
cco

# Terminal 2: Launch TUI
cco tui

# Terminal 3: Launch another Claude Code session
cd ~/project2
cco

# Verify:
# 1. All three sessions run simultaneously
# 2. TUI shows activity from both Claude Code sessions
# 3. Each Claude Code session has correct CWD
# 4. Daemon serves all sessions
```

---

### Test 7: Environment Variables

```bash
# Launch Claude Code in background
cd /tmp
cco &
CCO_PID=$!

# Find Claude Code process
sleep 2
CLAUDE_PID=$(pgrep -P $CCO_PID claude || echo "")

if [ -n "$CLAUDE_PID" ]; then
    # Check environment variables
    ps eww $CLAUDE_PID | tr ' ' '\n' | grep ORCHESTRATOR

    # Expected output (7 variables):
    # ORCHESTRATOR_ENABLED=true
    # ORCHESTRATOR_VFS_MOUNT=/var/run/cco
    # ORCHESTRATOR_AGENTS=/var/run/cco/agents.sealed
    # ORCHESTRATOR_RULES=/var/run/cco/orchestrator.sealed
    # ORCHESTRATOR_HOOKS=/var/run/cco/hooks.sealed
    # ORCHESTRATOR_MANIFEST=/var/run/cco/.manifest
    # ORCHESTRATOR_API_URL=http://localhost:3000
fi

# Cleanup
kill $CCO_PID 2>/dev/null
```

---

### Test 8: VFS Health Checks

```bash
# Start daemon
cco daemon start

# Verify VFS is mounted
ls /var/run/cco/

# Expected files:
# agents.sealed
# orchestrator.sealed
# hooks.sealed
# .manifest
# health
# README.txt

# Check health file
cat /var/run/cco/health
# Expected: OK

# Check manifest
cat /var/run/cco/.manifest
# Expected: Valid JSON with version metadata
```

---

### Test 9: Daemon Auto-Start Performance

```bash
# Stop daemon
cco daemon stop

# Time the auto-start
time cco --version  # Claude Code --version (fast exit)

# Expected:
# real    0m2.5s  (< 3 seconds including daemon startup)
# user    0m0.1s
# sys     0m0.05s
```

---

### Test 10: Current Directory Preservation

```bash
# Create test directories
mkdir -p /tmp/test-cco/{dir1,dir2,dir3}

# Test 1: Launch from dir1
cd /tmp/test-cco/dir1
cco --help 2>&1 | head -1

# Verify: Claude Code runs in /tmp/test-cco/dir1

# Test 2: Launch from dir2
cd /tmp/test-cco/dir2
cco --version 2>&1 | head -1

# Verify: Claude Code runs in /tmp/test-cco/dir2

# Cleanup
rm -rf /tmp/test-cco
```

---

## Automated Test Commands

### Unit Tests

```bash
# Test launcher module
cargo test --lib launcher

# Test TUI module
cargo test --lib tui

# Test all commands
cargo test --lib commands
```

### Integration Tests

```bash
# Test full launcher workflow
cargo test --test launcher_integration

# Test TUI workflow
cargo test --test tui_integration

# Test daemon lifecycle
cargo test --test daemon_lifecycle
```

### All Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_ensure_daemon_running
```

---

## Security Testing

### Environment Variable Injection Test

```bash
# Verify no sensitive data in env vars
cco &
CCO_PID=$!
sleep 2

# Check Claude Code process environment
CLAUDE_PID=$(pgrep -P $CCO_PID claude || echo "")
if [ -n "$CLAUDE_PID" ]; then
    ps eww $CLAUDE_PID | tr ' ' '\n' | grep -E '(PASSWORD|SECRET|KEY|TOKEN)'
fi

# Expected: No sensitive environment variables
# Only ORCHESTRATOR_* variables should be present

kill $CCO_PID 2>/dev/null
```

### VFS Security Test

```bash
# Verify VFS files are read-only
ls -la /var/run/cco/

# Expected permissions:
# -r--r--r-- agents.sealed       (444)
# -r--r--r-- orchestrator.sealed (444)
# -r--r--r-- hooks.sealed        (444)
# -r--r--r-- .manifest           (444)
# -r--r--r-- health              (444)

# Verify cannot modify files
echo "test" >> /var/run/cco/health 2>&1 | grep -i permission

# Expected: Permission denied
```

### PATH Injection Test

```bash
# Create fake claude in current directory
cat > ./claude << 'EOF'
#!/bin/bash
echo "MALICIOUS CLAUDE EXECUTED"
env | grep ORCHESTRATOR
EOF
chmod +x ./claude

# Try to launch (should find real claude in /usr/local/bin or similar)
PATH=".:$PATH" cco

# Expected: Real Claude Code launches, not fake one
# OR: Error if real Claude Code not in standard PATH
```

---

## Performance Benchmarks

### Daemon Auto-Start

```bash
# Target: < 3 seconds
for i in {1..5}; do
    cco daemon stop
    time (cco --version >/dev/null 2>&1)
done

# Expected average: ~2.5 seconds
```

### VFS Health Check

```bash
# Target: < 100ms
for i in {1..10}; do
    time (cat /var/run/cco/health >/dev/null 2>&1)
done

# Expected average: < 10ms
```

### Pass-Through Argument Performance

```bash
# Target: No significant overhead
time cco --version
time claude --version

# Difference should be < 100ms (just daemon check overhead)
```

---

## Test Coverage Goals

- **Unit Tests**: > 90% line coverage
- **Integration Tests**: All 11+ error scenarios
- **E2E Tests**: Happy path + 5 common error paths
- **Security Tests**: Environment, VFS, PATH injection
- **Performance Tests**: 3 critical metrics

---

## Reporting Issues

### Issue Template

```markdown
## Environment
- OS: macOS [version]
- CCO Version: [git hash or version]
- macFUSE Version: [version]

## Test Case
- Test Name: [e.g., Test 5a - Daemon Start Failure]
- Command: `cco [args]`

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happened]

## Reproduction Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Logs
```bash
# Daemon logs
cco daemon logs

# VFS status
ls -la /var/run/cco/

# Environment
env | grep ORCHESTRATOR
```
```

---

**Last Updated**: 2025-11-17
**Next Update**: After Phase 4 test implementation
