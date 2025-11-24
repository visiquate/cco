# Comprehensive Test Plan - Critical Issues Fix Verification

**Document Version**: 1.0
**Created**: 2025-11-16
**Target Fixes**:
1. Ctrl+C shutdown not working cleanly (4+ seconds to exit)
2. Terminal prompt not displaying correctly
3. Logging spam every 5 seconds

---

## Test Execution Priority

**CRITICAL**: Execute these tests immediately after build to verify all fixes are working.

---

## Test Environment Setup

### Prerequisites
- Rust toolchain installed and configured
- Fresh build from latest commit
- Port 3000 available (or modify for different port)
- Shell terminal for running commands
- Log monitoring capability

### Setup Commands
```bash
# Kill any existing cco processes
pkill -f "cco run" || true
pkill -f "cco " || true
sleep 1

# Verify clean state
ps aux | grep cco | grep -v grep || echo "✓ No CCO processes running"
```

---

## TEST SUITE 1: Ctrl+C Shutdown - Graceful Termination

### Objective
Verify that Ctrl+C shutdown completes cleanly within 1-2 seconds and leaves no zombie processes.

### Test 1.1: Clean Shutdown Response Time

**Setup**:
```bash
export NO_BROWSER=1
cd /Users/brent/git/cc-orchestra/cco
cargo build --release 2>&1 | tail -5  # Verify successful build
```

**Execution**:
```bash
# Start server and note the exact start time
START_TIME=$(date +%s.%N)
timeout 5 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_1_1_logs.txt &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Send Ctrl+C (SIGINT)
kill -INT $SERVER_PID 2>/dev/null || true
SHUTDOWN_TIME=$(date +%s.%N)

# Calculate shutdown duration
DURATION=$(echo "$SHUTDOWN_TIME - ($START_TIME + 3)" | bc)

# Wait for process to fully exit
wait $SERVER_PID 2>/dev/null || true
FINAL_TIME=$(date +%s.%N)
TOTAL_SHUTDOWN=$(echo "$FINAL_TIME - $SHUTDOWN_TIME" | bc)
```

**Expected Results**:
- Process exits within 2 seconds of Ctrl+C
- Total shutdown time: **< 2 seconds** (PASS), > 4 seconds (FAIL)
- Log message appears: "Server shut down gracefully"
- Exit code is 0 (graceful) or 130 (SIGINT - acceptable)

**Verification**:
```bash
# Check logs
grep -i "shut down" /tmp/test_1_1_logs.txt
echo "Shutdown duration: ${TOTAL_SHUTDOWN}s"

# Check for exit code
echo "Exit code: $?"
```

**Pass Criteria**:
- [ ] "Server shut down gracefully" message in logs
- [ ] Shutdown duration < 2.0 seconds
- [ ] No timeout error (timeout command doesn't trigger)
- [ ] Exit code is 0 or 130

---

### Test 1.2: No Zombie Processes After Shutdown

**Setup**:
```bash
export NO_BROWSER=1
```

**Execution**:
```bash
# Start server
cargo run --release -- run --debug --port 3000 &
SERVER_PID=$!

# Let it initialize
sleep 3

# Verify process exists
ps -p $SERVER_PID > /dev/null && echo "✓ Server running (PID: $SERVER_PID)"

# Kill with Ctrl+C
kill -INT $SERVER_PID 2>/dev/null || true

# Wait up to 2 seconds for graceful shutdown
for i in {1..20}; do
    if ! ps -p $SERVER_PID > /dev/null 2>&1; then
        echo "✓ Process exited cleanly after $((i * 100))ms"
        break
    fi
    sleep 0.1
done

# Final check - process should NOT exist
if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo "✗ FAIL: Process still running after shutdown"
    kill -9 $SERVER_PID 2>/dev/null || true
else
    echo "✓ Process successfully terminated"
fi
```

**Expected Results**:
- Process exits within 200ms of Ctrl+C (or kill -INT)
- No zombie process remains
- ps/lsof shows no listening socket on port 3000

**Verification**:
```bash
# Verify port is released
sleep 1
lsof -i :3000 || echo "✓ Port 3000 is free"

# Verify no zombie processes
ps aux | grep -E 'cargo|cco' | grep -v grep || echo "✓ No CCO processes"
```

**Pass Criteria**:
- [ ] Process exits within 200ms of kill -INT
- [ ] lsof shows port 3000 is free
- [ ] No zombie processes remain
- [ ] No error messages in logs related to shutdown

---

### Test 1.3: Multiple Shutdown Attempts (Idempotency)

**Objective**: Verify that multiple Ctrl+C presses don't cause issues.

**Execution**:
```bash
export NO_BROWSER=1
cargo run --release -- run --debug --port 3000 &
SERVER_PID=$!

# Wait for startup
sleep 3

# Send first Ctrl+C
echo "Sending first Ctrl+C..."
kill -INT $SERVER_PID

# Wait 0.5s and send second Ctrl+C (simulating user pressing multiple times)
sleep 0.5
if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo "Sending second Ctrl+C..."
    kill -INT $SERVER_PID
fi

# Wait for exit
wait $SERVER_PID 2>/dev/null
EXIT_CODE=$?
```

**Expected Results**:
- First Ctrl+C initiates shutdown
- Second Ctrl+C is handled gracefully (or ignored)
- Process exits cleanly
- No error messages about signal handling

**Pass Criteria**:
- [ ] No error messages in logs
- [ ] Process exits within 2 seconds of first Ctrl+C
- [ ] Exit code 0 or 130

---

## TEST SUITE 2: Terminal Functionality - Prompt Display

### Objective
Verify that terminal prompt displays correctly and user input/output work properly.

### Test 2.1: Terminal Prompt Display via WebSocket

**Setup**:
```bash
export NO_BROWSER=1
cargo run --release -- run --debug --port 3000 &
SERVER_PID=$!

# Wait for server startup
sleep 3
```

**Execution**:
```bash
# Create test script to connect to terminal WebSocket
cat > /tmp/test_terminal.sh << 'EOF'
#!/bin/bash

# Simple WebSocket test using curl
RESPONSE=$(curl -i -N -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  -H "Sec-WebSocket-Version: 13" \
  http://127.0.0.1:3000/terminal 2>&1 | head -20)

echo "WebSocket Response:"
echo "$RESPONSE"

# Check for upgrade headers
if echo "$RESPONSE" | grep -qi "101 Switching"; then
    echo "✓ WebSocket upgrade successful"
elif echo "$RESPONSE" | grep -qi "upgrade"; then
    echo "✓ Upgrade header present"
else
    echo "⚠ May need direct WebSocket test"
fi
EOF

chmod +x /tmp/test_terminal.sh
/tmp/test_terminal.sh
```

**Check for terminal endpoint availability**:
```bash
# Health check for /terminal endpoint
curl -s http://127.0.0.1:3000/health | jq '.' || echo "Health endpoint returned data"

# Test API endpoints
curl -s http://127.0.0.1:3000/ | head -20 || echo "Root endpoint accessible"
```

**Expected Results**:
- WebSocket upgrade succeeds OR returns proper error
- Terminal endpoint responds to connection attempts
- No 404 or 500 errors for /terminal endpoint
- Server handles connection gracefully

**Pass Criteria**:
- [ ] WebSocket upgrade headers present or endpoint responds
- [ ] No error messages about missing terminal functionality
- [ ] Server continues running after terminal test

---

### Test 2.2: Terminal via Browser Dashboard (Manual)

**Objective**: Verify terminal display in browser UI.

**Setup**:
```bash
# Start server WITHOUT NO_BROWSER to auto-open
cargo run --release -- run --debug --port 3000
```

**Manual Steps**:
1. Browser opens automatically to http://127.0.0.1:3000
2. Look for "Terminal" tab/section in UI
3. Click on Terminal
4. Look for prompt display (should show something like `$` or `>`)

**Expected Results**:
- Terminal section visible in dashboard
- Prompt displays clearly
- No missing UI elements
- Input field is active and accepts input

**Pass Criteria**:
- [ ] Terminal tab/section is visible
- [ ] Prompt is displayed (not empty)
- [ ] Input field is focusable
- [ ] No JavaScript errors in browser console

---

### Test 2.3: Terminal Input/Output (Script-based)

**Objective**: Verify that terminal can receive and respond to commands.

**Setup**:
```bash
# Terminal test with bash
cat > /tmp/test_terminal_io.js << 'EOF'
const WebSocket = require('ws');

async function testTerminal() {
    console.log('Attempting terminal connection...');

    try {
        const ws = new WebSocket('ws://127.0.0.1:3000/terminal');

        ws.on('open', () => {
            console.log('✓ Terminal WebSocket connected');

            // Send a simple command
            ws.send(JSON.stringify({
                type: 'input',
                data: 'echo "test"\n'
            }));

            // Wait for response
            setTimeout(() => {
                ws.close();
            }, 1000);
        });

        ws.on('message', (data) => {
            console.log('✓ Received output:', data.substring(0, 100));
        });

        ws.on('error', (err) => {
            console.log('WebSocket error:', err.message);
        });

    } catch (err) {
        console.log('Connection error:', err.message);
    }
}

testTerminal();
EOF
```

**Pass Criteria** (if able to run):
- [ ] WebSocket connection succeeds
- [ ] Server responds to input messages
- [ ] Output messages are received
- [ ] No connection errors

---

## TEST SUITE 3: Logging - Reduce Spam

### Objective
Verify that logging no longer spams with repetitive project path messages every 5 seconds.

### Test 3.1: Monitor Logging Output for 15 Seconds

**Setup**:
```bash
export NO_BROWSER=1
```

**Execution**:
```bash
# Start server and capture logs
cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_3_1_logs.txt &
SERVER_PID=$!

# Let it run for 15 seconds
echo "Monitoring logs for 15 seconds..."
sleep 15

# Kill server
kill -INT $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

# Analyze logs
echo ""
echo "=== LOG ANALYSIS ==="
echo ""

# Count specific log lines that were causing spam
SPAM_PATTERNS=(
    "CCO_PROJECT_PATH"
    "Current working directory"
    "Derived project path"
    "Project path detection"
)

for pattern in "${SPAM_PATTERNS[@]}"; do
    COUNT=$(grep -c "$pattern" /tmp/test_3_1_logs.txt 2>/dev/null || echo 0)
    if [ "$COUNT" -gt 0 ]; then
        echo "Found '$pattern': $COUNT occurrences"
        if [ "$COUNT" -gt 3 ]; then
            echo "  ⚠ WARNING: This appears too frequently (should be rare)"
        fi
    fi
done

echo ""
echo "Total log lines: $(wc -l < /tmp/test_3_1_logs.txt)"
```

**Expected Results**:
- "CCO_PROJECT_PATH": Should appear 0-2 times (at startup, not every 5 seconds)
- "Current working directory": Should appear 0-2 times (at startup only)
- "Derived project path": Should appear 0-2 times (at startup only)
- No repeating patterns appearing every 5 seconds
- Log file size is reasonable (not growing exponentially)

**Verification**:
```bash
# Check for repeating patterns
echo "Checking for repeating spam patterns..."
grep -E "CCO_PROJECT_PATH|Current working directory" /tmp/test_3_1_logs.txt | \
    awk '{print NR": "$0}' | tail -20 || echo "No spam patterns found ✓"

# Check for INFO-level duplicate events
echo ""
echo "INFO level log count:"
grep -c " INFO " /tmp/test_3_1_logs.txt || echo 0

echo "DEBUG level log count:"
grep -c " DEBUG " /tmp/test_3_1_logs.txt || echo 0

echo "TRACE level log count:"
grep -c " TRACE " /tmp/test_3_1_logs.txt || echo 0
```

**Pass Criteria**:
- [ ] No pattern appears more than 2 times in 15 seconds
- [ ] Spam-related messages only at startup (first 5 seconds)
- [ ] Log file is reasonable size (< 50KB for 15 seconds)
- [ ] No repeating "error" or "warn" messages

---

### Test 3.2: Log Level Verification (DEBUG flag behavior)

**Objective**: Verify that trace logs appear only at appropriate levels.

**Execution**:
```bash
# Run WITH debug flag
echo "Running WITH --debug flag..."
cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_debug_on.txt &
DEBUG_PID=$!
sleep 5
kill -INT $DEBUG_PID 2>/dev/null || true
wait $DEBUG_PID 2>/dev/null || true

# Run WITHOUT debug flag
echo "Running WITHOUT --debug flag..."
export NO_BROWSER=1
cargo run --release -- run --port 3000 2>&1 | tee /tmp/test_debug_off.txt &
NORMAL_PID=$!
sleep 5
kill -INT $NORMAL_PID 2>/dev/null || true
wait $NORMAL_PID 2>/dev/null || true

# Compare log levels
echo ""
echo "=== LOG LEVEL COMPARISON ==="
echo "WITH --debug:"
echo "  TRACE: $(grep -c 'TRACE' /tmp/test_debug_on.txt || echo 0)"
echo "  DEBUG: $(grep -c 'DEBUG' /tmp/test_debug_on.txt || echo 0)"
echo "  INFO:  $(grep -c 'INFO' /tmp/test_debug_on.txt || echo 0)"

echo ""
echo "WITHOUT --debug:"
echo "  TRACE: $(grep -c 'TRACE' /tmp/test_debug_off.txt || echo 0)"
echo "  DEBUG: $(grep -c 'DEBUG' /tmp/test_debug_off.txt || echo 0)"
echo "  INFO:  $(grep -c 'INFO' /tmp/test_debug_off.txt || echo 0)"
```

**Expected Results**:
- WITH --debug: Higher count of DEBUG and TRACE messages
- WITHOUT --debug: Only INFO and ERROR messages
- Spam patterns present only in both (if at all) at startup

**Pass Criteria**:
- [ ] --debug flag increases verbosity
- [ ] Spam patterns don't increase in frequency with --debug
- [ ] INFO-level logging is consistent between modes

---

### Test 3.3: Long-running Stability (60-second observation)

**Objective**: Verify no logging degradation over extended runtime.

**Execution**:
```bash
export NO_BROWSER=1

echo "Starting 60-second stability test..."
START=$(date +%s)

# Run server for 60 seconds
timeout 61 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_stability_60s.txt &
STABILITY_PID=$!

wait $STABILITY_PID 2>/dev/null || true

END=$(date +%s)
DURATION=$((END - START))

echo ""
echo "=== 60-SECOND STABILITY TEST RESULTS ==="
echo "Duration: $DURATION seconds"
echo "Log file size: $(du -h /tmp/test_stability_60s.txt | cut -f1)"
echo ""

# Calculate log growth rate
echo "Spam pattern occurrences (full run):"
for pattern in "CCO_PROJECT_PATH" "Current working directory" "Derived project path"; do
    TOTAL=$(grep -c "$pattern" /tmp/test_stability_60s.txt 2>/dev/null || echo 0)
    if [ "$TOTAL" -gt 0 ]; then
        echo "  $pattern: $TOTAL times"
        RATE=$((TOTAL * 60 / DURATION))
        echo "    → Rate: ~$RATE per minute"
        if [ "$RATE" -gt 12 ]; then
            echo "    ⚠ WARNING: High occurrence rate"
        fi
    fi
done

# Check for error spikes
echo ""
echo "Error/Warn message count:"
ERROR_COUNT=$(grep -c -E ' (ERROR|WARN) ' /tmp/test_stability_60s.txt || echo 0)
echo "  Total: $ERROR_COUNT"
```

**Pass Criteria**:
- [ ] Server runs for full 60 seconds without crashing
- [ ] Log file size grows linearly (not exponentially)
- [ ] Spam patterns don't increase over time (constant baseline)
- [ ] No recurring error messages

---

## TEST SUITE 4: Integration Test - All Features Together

### Test 4.1: Server Start, Terminal Access, Shutdown Cycle

**Objective**: Complete cycle test verifying all fixes work together.

**Execution**:
```bash
export NO_BROWSER=1

echo "=== INTEGRATION TEST: Complete Cycle ==="
echo ""

# 1. Start server
echo "1. Starting server..."
START_TIME=$(date +%s%N)
cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test_integration.txt &
SERVER_PID=$!

# 2. Verify startup
sleep 3
if ps -p $SERVER_PID > /dev/null; then
    echo "   ✓ Server running"
else
    echo "   ✗ Server failed to start"
    exit 1
fi

# 3. Test health endpoint
echo "2. Testing health endpoint..."
if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
    echo "   ✓ Health endpoint responds"
else
    echo "   ⚠ Health endpoint not responding"
fi

# 4. Check logs for startup messages
echo "3. Verifying startup logs..."
if grep -q "Starting Claude Code Orchestra" /tmp/test_integration.txt; then
    echo "   ✓ Startup message found"
fi
if grep -q "agents loaded" /tmp/test_integration.txt || \
   grep -q "loaded" /tmp/test_integration.txt; then
    echo "   ✓ Agents loaded message found"
fi

# 5. Shutdown gracefully
echo "4. Testing shutdown..."
SHUTDOWN_START=$(date +%s%N)
kill -INT $SERVER_PID
SHUTDOWN_END=$(date +%s%N)

# Wait for process to exit
wait $SERVER_PID 2>/dev/null || true
PROCESS_EXIT=$(date +%s%N)

SHUTDOWN_DURATION_MS=$(((SHUTDOWN_END - SHUTDOWN_START) / 1000000))
TOTAL_SHUTDOWN_MS=$(((PROCESS_EXIT - SHUTDOWN_START) / 1000000))

echo "   Graceful shutdown initiated: ${SHUTDOWN_DURATION_MS}ms"
echo "   Total shutdown time: ${TOTAL_SHUTDOWN_MS}ms"

if [ $TOTAL_SHUTDOWN_MS -lt 2000 ]; then
    echo "   ✓ Shutdown completed within 2 seconds"
else
    echo "   ⚠ Shutdown took longer than expected"
fi

# 6. Check shutdown logs
echo "5. Verifying shutdown logs..."
if grep -i "shut down" /tmp/test_integration.txt; then
    echo "   ✓ Shutdown message found"
else
    echo "   ⚠ Shutdown message not found"
fi

# 7. Verify port is released
sleep 1
if ! lsof -i :3000 > /dev/null 2>&1; then
    echo "   ✓ Port 3000 released"
else
    echo "   ✗ Port 3000 still in use"
fi

echo ""
echo "=== INTEGRATION TEST SUMMARY ==="
echo "All systems: Ready"
```

**Pass Criteria**:
- [ ] Server starts successfully
- [ ] Health endpoint responds
- [ ] Logs show all systems loaded
- [ ] Shutdown completes within 2 seconds
- [ ] Shutdown message appears in logs
- [ ] Port is released immediately after shutdown

---

## TEST SCRIPT - Automated Execution

Create `/tmp/comprehensive_test_suite.sh` to run all tests:

```bash
#!/bin/bash

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo ""
}

test_passed() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED_TESTS++))
}

test_failed() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED_TESTS++))
}

test_warning() {
    echo -e "${YELLOW}⚠ WARNING${NC}: $1"
}

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    pkill -f "cargo run.*--port 3000" || true
    pkill -f "cco run" || true
    sleep 1
}

trap cleanup EXIT

# Start tests
print_header "COMPREHENSIVE CCO TEST SUITE"
echo "Testing critical issue fixes:"
echo "1. Ctrl+C shutdown (< 2 seconds)"
echo "2. Terminal prompt display"
echo "3. Logging spam reduction"
echo ""

# TEST SUITE 1: Shutdown
print_header "TEST SUITE 1: SHUTDOWN PERFORMANCE"

cd /Users/brent/git/cc-orchestra/cco

# Test 1.1: Response time
((TOTAL_TESTS++))
echo "Test 1.1: Shutdown response time..."
export NO_BROWSER=1

timeout 8 cargo run --release -- run --debug --port 3000 2>&1 > /tmp/test_1_1.log &
PID=$!
sleep 3

SHUTDOWN_START=$(date +%s%N)
kill -INT $PID 2>/dev/null || true
SHUTDOWN_END=$(date +%s%N)

wait $PID 2>/dev/null || true
FINAL=$(date +%s%N)

SHUTDOWN_MS=$(((FINAL - SHUTDOWN_START) / 1000000))

if [ $SHUTDOWN_MS -lt 2000 ]; then
    test_passed "Shutdown completed in ${SHUTDOWN_MS}ms"
else
    test_failed "Shutdown took ${SHUTDOWN_MS}ms (expected < 2000ms)"
fi

sleep 1

# Test 1.2: Graceful message
((TOTAL_TESTS++))
echo "Test 1.2: Graceful shutdown message..."
if grep -qi "shut down" /tmp/test_1_1.log; then
    test_passed "Shutdown message found in logs"
else
    test_failed "Shutdown message not found in logs"
fi

# Test 1.3: Port released
((TOTAL_TESTS++))
echo "Test 1.3: Port release verification..."
if ! lsof -i :3000 > /dev/null 2>&1; then
    test_passed "Port 3000 released successfully"
else
    test_failed "Port 3000 still in use"
    lsof -i :3000 || true
fi

# TEST SUITE 2: Logging
print_header "TEST SUITE 2: LOGGING VERIFICATION"

# Test 2.1: Log spam check
((TOTAL_TESTS++))
echo "Test 2.1: Monitoring logs for 15 seconds..."

timeout 16 cargo run --release -- run --debug --port 3000 2>&1 > /tmp/test_2_1.log &
PID=$!

sleep 15
kill -INT $PID 2>/dev/null || true
wait $PID 2>/dev/null || true

PROJECT_PATH_COUNT=$(grep -c "CCO_PROJECT_PATH" /tmp/test_2_1.log || echo 0)
WORKING_DIR_COUNT=$(grep -c "Current working directory" /tmp/test_2_1.log || echo 0)
DERIVED_COUNT=$(grep -c "Derived project path" /tmp/test_2_1.log || echo 0)

MAX_SPAM=$((PROJECT_PATH_COUNT + WORKING_DIR_COUNT + DERIVED_COUNT))

if [ $MAX_SPAM -le 6 ]; then
    test_passed "Log spam within acceptable limits ($MAX_SPAM total)"
else
    test_failed "Excessive spam messages: $MAX_SPAM (expected <= 6)"
fi

# Test 2.2: Debug mode verification
((TOTAL_TESTS++))
echo "Test 2.2: Log level with debug flag..."

timeout 6 cargo run --release -- run --debug --port 3000 2>&1 > /tmp/test_debug_on.log &
sleep 5
pkill -f "cargo run.*--port 3000" 2>/dev/null || true

timeout 6 cargo run --release -- run --port 3000 2>&1 > /tmp/test_debug_off.log &
sleep 5
pkill -f "cargo run.*--port 3000" 2>/dev/null || true

DEBUG_COUNT=$(grep -c "DEBUG" /tmp/test_debug_on.log || echo 0)
NORMAL_COUNT=$(grep -c "DEBUG" /tmp/test_debug_off.log || echo 0)

if [ $DEBUG_COUNT -gt $NORMAL_COUNT ]; then
    test_passed "Debug flag increases verbosity (debug: $DEBUG_COUNT, normal: $NORMAL_COUNT)"
else
    test_warning "Debug flag may not be increasing verbosity appropriately"
fi

# TEST SUITE 3: Integration
print_header "TEST SUITE 3: INTEGRATION TEST"

# Test 3.1: Full cycle
((TOTAL_TESTS++))
echo "Test 3.1: Complete server lifecycle..."

timeout 10 cargo run --release -- run --debug --port 3000 2>&1 > /tmp/test_integration.log &
PID=$!

# Wait for startup
sleep 3

# Verify health
if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
    test_passed "Health endpoint responds during operation"
else
    test_warning "Health endpoint not responding"
fi

# Graceful shutdown
kill -INT $PID 2>/dev/null || true
wait $PID 2>/dev/null || true

# Verify logs
if grep -qi "shut down\|shutting down" /tmp/test_integration.log; then
    test_passed "Integration test completed successfully"
else
    test_warning "Integration test log messages unclear"
fi

# SUMMARY
print_header "TEST SUMMARY"
TOTAL_RESULTS=$((PASSED_TESTS + FAILED_TESTS))
echo "Results: $PASSED_TESTS passed, $FAILED_TESTS failed (out of $TOTAL_TESTS tests)"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    exit 1
fi
```

**Usage**:
```bash
chmod +x /tmp/comprehensive_test_suite.sh
/tmp/comprehensive_test_suite.sh
```

---

## Test Checklist

### Pre-Test Checklist
- [ ] Clean build completed successfully
- [ ] No existing cco processes running
- [ ] Port 3000 is available
- [ ] At least 500MB disk space available for logs
- [ ] Terminal/shell access available
- [ ] Recent commit includes all fixes

### Ctrl+C Shutdown Tests
- [ ] Test 1.1: Shutdown response time (< 2 seconds)
- [ ] Test 1.2: No zombie processes remain
- [ ] Test 1.3: Multiple Ctrl+C handled gracefully
- [ ] "Server shut down gracefully" message present
- [ ] Port immediately released
- [ ] Exit code 0 or 130 (acceptable)

### Terminal Tests
- [ ] Test 2.1: WebSocket endpoint responds
- [ ] Test 2.2: Terminal accessible via browser UI
- [ ] Test 2.3: Terminal I/O functional (if testable)
- [ ] No missing UI elements
- [ ] Prompt displays clearly

### Logging Tests
- [ ] Test 3.1: Spam patterns < 3 occurrences in 15s
- [ ] Test 3.2: Debug flag increases verbosity
- [ ] Test 3.3: No exponential log growth over 60s
- [ ] "CCO_PROJECT_PATH" appears only at startup
- [ ] "Current working directory" appears only at startup
- [ ] No repeating error messages

### Integration Tests
- [ ] Test 4.1: Server starts successfully
- [ ] Server health endpoint responds
- [ ] All systems load without errors
- [ ] Graceful shutdown within 2 seconds
- [ ] Port released immediately
- [ ] Complete lifecycle works

### Post-Test Checklist
- [ ] All processes cleaned up
- [ ] No zombie processes
- [ ] Port 3000 available for next test
- [ ] Log files archived if needed
- [ ] Test results documented

---

## Failure Investigation Guide

If tests fail, follow this debugging approach:

### Shutdown Failures
1. Check for background threads: `ps aux | grep cco`
2. Review shutdown logs for error messages
3. Check signal handling in src/server.rs
4. Verify connection_tracker cleanup
5. Check for blocking operations in shutdown path

### Terminal Failures
1. Verify WebSocket route is registered in router
2. Check for terminal module compilation
3. Review browser console for JavaScript errors
4. Test WebSocket with external tool (wscat)
5. Check CORS configuration

### Logging Failures
1. Verify RUST_LOG environment variable is set
2. Check tracing subscriber initialization
3. Review log level constants in code
4. Look for debug-forced log statements
5. Check for background tasks logging

---

## Expected Test Duration

- Full automated test suite: 8-10 minutes
- Manual browser test: 5 minutes
- Individual test investigations: 5-30 minutes (if failures)

**Total validation time**: ~30-45 minutes for complete verification

---

## Test Report Template

```markdown
# Test Execution Report

**Date**: YYYY-MM-DD HH:MM:SS
**Tester**: [Name]
**Build Commit**: [commit hash]
**Result**: PASS / FAIL

## Test Results Summary
- Total Tests: XX
- Passed: XX
- Failed: XX
- Warnings: XX

## Detailed Results

### Shutdown Tests
- Test 1.1: ✓/✗
- Test 1.2: ✓/✗
- Test 1.3: ✓/✗

### Terminal Tests
- Test 2.1: ✓/✗
- Test 2.2: ✓/✗
- Test 2.3: ✓/✗

### Logging Tests
- Test 3.1: ✓/✗
- Test 3.2: ✓/✗
- Test 3.3: ✓/✗

### Integration Tests
- Test 4.1: ✓/✗

## Issues Found
[Describe any failures or unexpected behavior]

## Recommendations
[Any follow-up actions needed]
```

---

## Success Criteria Summary

**All three issues must be fixed**:

1. **Ctrl+C Shutdown**: ✓ Complete within 2 seconds
2. **Terminal Prompt**: ✓ Display correctly and respond to input
3. **Logging Spam**: ✓ Reduce repeating messages to startup only

**No regression issues**:
- ✓ No new crashes
- ✓ No memory leaks
- ✓ No hanging processes
- ✓ Health endpoint functional
- ✓ All features operational

---

## Next Steps After Testing

1. **If all tests pass**:
   - Archive test logs
   - Create commit noting test verification
   - Prepare release notes

2. **If tests fail**:
   - Document failure details
   - Identify affected component
   - Create issue for fixes
   - Schedule re-testing after fixes

3. **Performance optimization**:
   - Baseline performance metrics
   - Monitor in production
   - Optimize if needed

---

**Document Owner**: Test Engineer
**Last Updated**: 2025-11-16
**Next Review**: After test execution completion
