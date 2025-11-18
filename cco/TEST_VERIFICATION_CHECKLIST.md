# Test Verification Checklist - Critical Issues

Comprehensive checklist for validating all three critical fixes.

---

## PRE-TESTING VERIFICATION

### Environment Setup
- [ ] Fresh clone/pull of latest changes
- [ ] Cargo build succeeds: `cargo build --release 2>&1 | tail -5`
- [ ] No existing cco processes running: `pkill -9 cco; sleep 1`
- [ ] Port 3000 available: `lsof -i :3000 || echo "Port free"`
- [ ] At least 1GB free disk space: `df -h | grep /Volumes`
- [ ] Terminal/shell access available
- [ ] curl command available: `which curl`

### Build Verification
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
```
- [ ] Build completes successfully
- [ ] Binary exists: `ls -lh target/release/cco`
- [ ] Binary is executable: `./target/release/cco --version`

### Latest Changes
```bash
git log --oneline -5
```
- [ ] Most recent commit includes critical fixes
- [ ] Commit message mentions: "shutdown", "logging", or "terminal"
- [ ] All relevant files modified (server.rs, logger config, etc.)

---

## TEST SUITE 1: CTRL+C SHUTDOWN PERFORMANCE

### Test 1.1: Basic Shutdown Response Time

**Description**: Measure time from Ctrl+C to complete shutdown

**Procedure**:
```bash
cd /Users/brent/git/cc-orchestra/cco
export NO_BROWSER=1

# Start with timing
START=$(date +%s%N)
timeout 8 cargo run --release -- run --debug --port 3000 &
PID=$!

# Wait for startup
sleep 3

# Send Ctrl+C
kill -INT $PID

# Record shutdown
wait $PID
END=$(date +%s%N)

DURATION_MS=$(((END - START - 3000000000) / 1000000))
echo "Shutdown duration: ${DURATION_MS}ms"
```

**Acceptance Criteria**:
- [ ] Process exits cleanly
- [ ] Shutdown completes within 2000ms (2 seconds)
- [ ] Exit code is 0 or 130 (SIGINT)
- [ ] No error messages in output
- [ ] No "panic" or "thread" error messages

**Evidence Collection**:
- [ ] Record actual shutdown time in milliseconds
- [ ] Screenshot or log of output showing "shut down gracefully"
- [ ] Verify no error stack traces

**Pass/Fail**: ______ (PASS / FAIL)

**Notes**: ___________________________________________________________

---

### Test 1.2: Graceful Shutdown Message

**Description**: Verify "Server shut down gracefully" message appears

**Procedure**:
```bash
cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_1_2.log &
PID=$!
sleep 3
kill -INT $PID
wait $PID

# Check for message
grep -i "shut down\|shutting down" /tmp/test_1_2.log
```

**Acceptance Criteria**:
- [ ] At least one line contains "shut down" or "shutting down"
- [ ] Message appears after Ctrl+C is sent
- [ ] Message indicates graceful termination (not error)
- [ ] Timestamp is reasonable

**Message Examples** (one should be present):
- [ ] "Server shut down gracefully"
- [ ] "Shutting down gracefully"
- [ ] "Graceful shutdown initiated"
- [ ] "Shutdown complete"

**Pass/Fail**: ______ (PASS / FAIL)

**Exact Message Found**: ___________________________________________________________

---

### Test 1.3: Port Release Verification

**Description**: Verify port 3000 is released immediately after shutdown

**Procedure**:
```bash
# Start server
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Verify port is in use
lsof -i :3000 | grep cco
echo "Port in use: OK"

# Shutdown
kill -INT $PID
wait $PID

# Wait 1 second
sleep 1

# Verify port is free
lsof -i :3000 && echo "FAIL: Port still in use" || echo "PASS: Port released"
```

**Acceptance Criteria**:
- [ ] Before shutdown: `lsof -i :3000` shows cco listening
- [ ] After shutdown (1s wait): `lsof -i :3000` returns no results
- [ ] Port is available for new connection immediately
- [ ] No "Address already in use" errors on restart

**Pass/Fail**: ______ (PASS / FAIL)

**Notes**: ___________________________________________________________

---

### Test 1.4: No Zombie Processes

**Description**: Verify no zombie or hung processes remain after shutdown

**Procedure**:
```bash
# Start server
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Verify running
ps -p $PID && echo "Running"

# Shutdown
kill -INT $PID

# Watch process exit
for i in {1..20}; do
    if ! ps -p $PID > /dev/null 2>&1; then
        echo "Process exited after $((i * 100))ms"
        break
    fi
    sleep 0.1
done

# Final check
ps aux | grep -E 'cargo|cco' | grep -v grep
```

**Acceptance Criteria**:
- [ ] Process exits within 200ms of Ctrl+C
- [ ] `ps aux | grep cco` shows no remaining processes
- [ ] No `<defunct>` or `<zombie>` entries
- [ ] Parent process is not in zombie state

**Pass/Fail**: ______ (PASS / FAIL)

**Process Exit Time**: ___________________________________________________________

---

### Test 1.5: Multiple Rapid Shutdowns (Stress Test)

**Description**: Verify multiple Ctrl+C presses don't cause issues

**Procedure**:
```bash
for i in {1..3}; do
    echo "=== Iteration $i ==="
    cargo run --release -- run --debug --port 3000 &
    PID=$!
    sleep 3

    # Send multiple Ctrl+C signals
    kill -INT $PID
    sleep 0.2
    kill -INT $PID 2>/dev/null || true

    # Wait for exit
    wait $PID 2>/dev/null
    echo "Exit code: $?"

    sleep 1
done
```

**Acceptance Criteria**:
- [ ] All 3 iterations complete successfully
- [ ] No hangs on second Ctrl+C
- [ ] All exit codes are 0 or 130
- [ ] Port is free between iterations
- [ ] No error messages about signal handling

**Pass/Fail**: ______ (PASS / FAIL)

**Notes**: ___________________________________________________________

---

## TEST SUITE 2: LOGGING SPAM REDUCTION

### Test 2.1: Spam Message Count (15-second run)

**Description**: Count repeating "CCO_PROJECT_PATH" type messages over 15 seconds

**Procedure**:
```bash
export NO_BROWSER=1

echo "Starting 15-second logging analysis..."
timeout 16 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/spam_test.log &
sleep 15
pkill -f "cargo run.*port"
sleep 1

# Analyze
echo "=== SPAM MESSAGE ANALYSIS ==="
echo "CCO_PROJECT_PATH: $(grep -c 'CCO_PROJECT_PATH' /tmp/spam_test.log || echo 0)"
echo "Current working directory: $(grep -c 'Current working' /tmp/spam_test.log || echo 0)"
echo "Derived project path: $(grep -c 'Derived project' /tmp/spam_test.log || echo 0)"
echo "Total: $(($(grep -c 'CCO_PROJECT_PATH' /tmp/spam_test.log || echo 0) + $(grep -c 'Current working' /tmp/spam_test.log || echo 0) + $(grep -c 'Derived project' /tmp/spam_test.log || echo 0)))"
```

**Acceptance Criteria**:
- [ ] "CCO_PROJECT_PATH" appears ≤ 2 times (startup only)
- [ ] "Current working directory" appears ≤ 2 times (startup only)
- [ ] "Derived project path" appears ≤ 2 times (startup only)
- [ ] Total spam messages ≤ 6 across entire 15 seconds
- [ ] Messages appear in first 5 seconds only (startup phase)

**Measured Values**:
- [ ] CCO_PROJECT_PATH: _____ occurrences
- [ ] Current working directory: _____ occurrences
- [ ] Derived project path: _____ occurrences
- [ ] Total: _____ occurrences

**Expected**: ≤ 6 total
**Actual**: _____
**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 2.2: Spam Pattern Timing Analysis

**Description**: Verify spam messages don't repeat every 5 seconds

**Procedure**:
```bash
# Extract timing of spam messages
grep -n "CCO_PROJECT_PATH\|Current working\|Derived project" /tmp/spam_test.log | \
    awk -F: '{print NR": Line "$2}' | head -10

# Calculate intervals between occurrences
# If pattern repeats every ~5 seconds, this should show ~5 second gaps
```

**Acceptance Criteria**:
- [ ] Messages do NOT appear at regular 5-second intervals
- [ ] Gaps between messages are irregular or non-existent
- [ ] Messages only appear near the beginning of log
- [ ] No repeating pattern of messages throughout 15 seconds

**Observed Pattern**:
___________________________________________________________
___________________________________________________________

**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 2.3: Log Level Verification (Debug Mode)

**Description**: Verify --debug flag increases verbosity appropriately

**Procedure**:
```bash
# WITH debug flag
timeout 6 cargo run --release -- run --debug --port 3000 2>&1 > /tmp/debug_on.log &
sleep 5
pkill -f cargo

# WITHOUT debug flag
timeout 6 cargo run --release -- run --port 3000 2>&1 > /tmp/debug_off.log &
sleep 5
pkill -f cargo

echo "Debug ON - Log levels:"
echo "  TRACE: $(grep -c ' TRACE ' /tmp/debug_on.log || echo 0)"
echo "  DEBUG: $(grep -c ' DEBUG ' /tmp/debug_on.log || echo 0)"
echo "  INFO:  $(grep -c ' INFO ' /tmp/debug_on.log || echo 0)"

echo ""
echo "Debug OFF - Log levels:"
echo "  TRACE: $(grep -c ' TRACE ' /tmp/debug_off.log || echo 0)"
echo "  DEBUG: $(grep -c ' DEBUG ' /tmp/debug_off.log || echo 0)"
echo "  INFO:  $(grep -c ' INFO ' /tmp/debug_off.log || echo 0)"
```

**Acceptance Criteria**:
- [ ] WITH --debug: Higher DEBUG and TRACE message counts
- [ ] WITHOUT --debug: Primarily INFO level messages
- [ ] Spam patterns don't increase with --debug
- [ ] Log levels respond correctly to RUST_LOG environment variable

**Measured Values**:
| Level | With --debug | Without --debug |
|-------|--------------|-----------------|
| TRACE | _____ | _____ |
| DEBUG | _____ | _____ |
| INFO  | _____ | _____ |

**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 2.4: Long-running Stability (60-second observation)

**Description**: Verify no log spam or degradation over 60 seconds

**Procedure**:
```bash
export NO_BROWSER=1

echo "Starting 60-second stability test..."
START=$(date +%s)

timeout 61 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/stability_60s.log &
wait $?

END=$(date +%s)
DURATION=$((END - START))

echo "=== 60-SECOND RESULTS ==="
echo "Duration: ${DURATION}s"
echo "Log file size: $(du -h /tmp/stability_60s.log | cut -f1)"
echo "Spam messages: $(grep -c 'CCO_PROJECT_PATH\|Current working\|Derived project' /tmp/stability_60s.log || echo 0)"

# Calculate rate
TOTAL_SPAM=$(grep -c 'CCO_PROJECT_PATH\|Current working\|Derived project' /tmp/stability_60s.log || echo 0)
RATE=$((TOTAL_SPAM * 60 / DURATION))
echo "Spam rate: ~${RATE} per minute"
```

**Acceptance Criteria**:
- [ ] Server runs for full 60 seconds without crashing
- [ ] Log file size is reasonable (< 500KB)
- [ ] Spam message count ≤ 12 (max 2 per minute)
- [ ] No exponential growth in log size
- [ ] No recurring error patterns

**Measured Values**:
- [ ] Duration: _____ seconds
- [ ] Log file size: _____
- [ ] Total spam messages: _____
- [ ] Spam rate: _____ per minute

**Pass/Fail**: ______ (PASS / FAIL)

---

## TEST SUITE 3: TERMINAL FUNCTIONALITY

### Test 3.1: Health Endpoint Availability

**Description**: Verify /health endpoint responds during normal operation

**Procedure**:
```bash
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Test endpoint
curl -s http://127.0.0.1:3000/health | head -20

# Shutdown
kill -INT $PID
wait $PID
```

**Acceptance Criteria**:
- [ ] HTTP 200 status code returned
- [ ] Response contains JSON with system health info
- [ ] Response time < 100ms
- [ ] No error messages in response

**Response Sample**:
___________________________________________________________
___________________________________________________________

**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 3.2: Terminal Endpoint Accessibility

**Description**: Verify /terminal endpoint is accessible and responds correctly

**Procedure**:
```bash
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Test WebSocket upgrade
curl -i -N -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: test-key" \
  -H "Sec-WebSocket-Version: 13" \
  http://127.0.0.1:3000/terminal 2>&1 | head -15

# Shutdown
kill -INT $PID
wait $PID
```

**Acceptance Criteria**:
- [ ] Endpoint does not return 404
- [ ] Response includes upgrade headers or proper WebSocket response
- [ ] No 500 or 503 errors
- [ ] Error responses are graceful (not panic)

**Response Headers**:
___________________________________________________________

**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 3.3: Terminal in Browser UI (Manual)

**Description**: Visual verification of terminal in web interface

**Procedure** (Manual):
1. Start server: `cargo run --release -- run --port 3000`
2. Browser should auto-open to http://127.0.0.1:3000
3. Look for Terminal tab or section
4. Verify prompt is visible
5. Try typing in input field

**Acceptance Criteria**:
- [ ] Terminal tab/section is visible in UI
- [ ] Prompt displays correctly (not empty or broken)
- [ ] Input field is active and accepts text
- [ ] No JavaScript errors in browser console (F12 → Console)
- [ ] UI layout is not broken or misaligned

**Observations**:
___________________________________________________________
___________________________________________________________

**Pass/Fail**: ______ (PASS / FAIL)

---

### Test 3.4: No Broken Terminal Routes

**Description**: Verify terminal functionality doesn't break on startup

**Procedure**:
```bash
# Check logs for terminal-related errors
timeout 5 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/terminal_test.log &
sleep 3
pkill -f cargo
sleep 1

# Look for errors
echo "Checking for terminal errors..."
grep -i "terminal\|error\|failed\|panic" /tmp/terminal_test.log | grep -v "ok\|test\|ready"
```

**Acceptance Criteria**:
- [ ] No error messages containing "terminal"
- [ ] No panic or thread errors
- [ ] No "route not found" for /terminal
- [ ] All systems start successfully

**Errors Found**: _____ (count)

**Pass/Fail**: ______ (PASS / FAIL)

---

## TEST SUITE 4: INTEGRATION TEST

### Test 4.1: Complete Server Lifecycle

**Description**: End-to-end verification of start → operation → shutdown

**Procedure**:
```bash
export NO_BROWSER=1

echo "=== INTEGRATION TEST ==="
START=$(date +%s%N)

# 1. Start server
echo "1. Starting server..."
cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/integration.log &
PID=$!

sleep 3

# 2. Verify startup
if ps -p $PID > /dev/null; then
    echo "   ✓ Server running"
else
    echo "   ✗ Server failed"
    exit 1
fi

# 3. Test health
if curl -s http://127.0.0.1:3000/health > /dev/null; then
    echo "   ✓ Health endpoint works"
fi

# 4. Test terminal
if curl -s -I http://127.0.0.1:3000/terminal > /dev/null; then
    echo "   ✓ Terminal endpoint works"
fi

# 5. Shutdown
echo "2. Initiating shutdown..."
kill -INT $PID
wait $PID
END=$(date +%s%N)

TOTAL_MS=$(((END - START) / 1000000))
echo "   Total duration: ${TOTAL_MS}ms"

# 6. Verify results
echo "3. Verifying shutdown..."
if grep -i "shut down" /tmp/integration.log; then
    echo "   ✓ Graceful shutdown logged"
fi

if ! lsof -i :3000 > /dev/null 2>&1; then
    echo "   ✓ Port released"
fi
```

**Acceptance Criteria**:
- [ ] Server starts successfully
- [ ] No startup errors in logs
- [ ] Health endpoint responds
- [ ] Terminal endpoint accessible
- [ ] Graceful shutdown message logged
- [ ] Total operation time reasonable (< 15 seconds)
- [ ] Port released immediately
- [ ] No zombie processes

**Results**:
- [ ] Server startup: PASS / FAIL
- [ ] Health endpoint: PASS / FAIL
- [ ] Terminal endpoint: PASS / FAIL
- [ ] Graceful shutdown: PASS / FAIL
- [ ] Port released: PASS / FAIL

**Overall Pass/Fail**: ______ (PASS / FAIL)

---

### Test 4.2: Error Recovery

**Description**: Verify graceful handling of errors doesn't break shutdown

**Procedure**:
```bash
# Kill server ungracefully during operation
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Send SIGTERM instead of SIGINT
echo "Sending SIGTERM..."
kill -TERM $PID
wait $PID

# Check exit code
echo "Exit code: $?"

# Verify cleanup
sleep 1
lsof -i :3000 || echo "Port free"
ps aux | grep cco | grep -v grep || echo "No processes"
```

**Acceptance Criteria**:
- [ ] Process terminates gracefully
- [ ] No error messages
- [ ] Port is released
- [ ] No zombie processes

**Pass/Fail**: ______ (PASS / FAIL)

---

## FINAL SUMMARY

### Overall Test Results

**Total Test Cases**: 15+

**Individual Test Results**:
- [ ] Test 1.1 (Shutdown time): PASS / FAIL
- [ ] Test 1.2 (Shutdown message): PASS / FAIL
- [ ] Test 1.3 (Port release): PASS / FAIL
- [ ] Test 1.4 (No zombies): PASS / FAIL
- [ ] Test 1.5 (Multiple shutdowns): PASS / FAIL
- [ ] Test 2.1 (Spam count): PASS / FAIL
- [ ] Test 2.2 (Spam pattern): PASS / FAIL
- [ ] Test 2.3 (Log levels): PASS / FAIL
- [ ] Test 2.4 (Stability): PASS / FAIL
- [ ] Test 3.1 (Health endpoint): PASS / FAIL
- [ ] Test 3.2 (Terminal endpoint): PASS / FAIL
- [ ] Test 3.3 (Browser UI): PASS / FAIL
- [ ] Test 3.4 (No errors): PASS / FAIL
- [ ] Test 4.1 (Lifecycle): PASS / FAIL
- [ ] Test 4.2 (Error recovery): PASS / FAIL

**Overall Status**:
```
[  ] All tests passed - Issues RESOLVED ✓
[  ] Some tests failed - Issues NOT resolved ✗
[  ] Warnings present - Review needed ⚠
```

### Critical Fixes Verification

**Issue #1 - Shutdown Performance**:
- Target: < 2 seconds
- Actual: _____ ms
- Status: [  ] PASS [  ] FAIL

**Issue #2 - Terminal Prompt**:
- Status: [  ] PASS [  ] FAIL
- Observations: ___________________________________________________________

**Issue #3 - Logging Spam**:
- Target: ≤ 6 messages in 15 seconds
- Actual: _____ messages
- Status: [  ] PASS [  ] FAIL

### Tester Information

**Tester Name**: ___________________________________

**Test Date**: ___________________________________

**Build Commit**: ___________________________________

**Test Environment**:
- OS: ___________________________________
- Rust Version: ___________________________________
- ccoreference: ___________________________________

### Comments and Notes

___________________________________________________________
___________________________________________________________
___________________________________________________________

---

**This checklist must be FULLY COMPLETED before declaring fixes as verified.**

**Keep a copy of this completed checklist in the project records.**
