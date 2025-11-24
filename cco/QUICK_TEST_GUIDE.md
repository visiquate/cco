# Quick Test Guide - Critical Issues Verification

Fast reference for running tests on the three critical fixes.

## TL;DR - Run All Tests

```bash
/tmp/comprehensive_test_suite.sh
```

Expected result: **✓ ALL CRITICAL TESTS PASSED**

Expected duration: 8-10 minutes

---

## What's Being Tested

### Issue #1: Ctrl+C Shutdown Takes 4+ Seconds
**Fix Target**: Shutdown should complete in < 2 seconds
**Tests**:
- Shutdown response time measurement
- Graceful shutdown message
- Port release verification

**Expected Logs**:
```
[INFO] Server shut down gracefully
```

### Issue #2: Terminal Prompt Not Displaying
**Fix Target**: Terminal prompt displays correctly
**Tests**:
- Health endpoint responds
- Terminal endpoint accessible
- No 404/500 errors for /terminal

**Expected**: WebSocket upgrade succeeds or returns proper error

### Issue #3: Logging Spam Every 5 Seconds
**Fix Target**: Reduce repeating "CCO_PROJECT_PATH" messages
**Tests**:
- Count "CCO_PROJECT_PATH" in 15-second run
- Verify not appearing every 5 seconds
- Check debug flag effect on verbosity

**Expected**: 0-2 occurrences in 15 seconds (startup only)

---

## Individual Tests (Manual)

### Test 1: Shutdown Time

```bash
cd /Users/brent/git/cc-orchestra/cco
export NO_BROWSER=1

# Start server
time cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Send Ctrl+C and measure
kill -INT $PID
wait $PID

# Expected: < 2 seconds, "Server shut down gracefully" in output
```

### Test 2: Check Logs for Spam

```bash
cd /Users/brent/git/cc-orchestra/cco
export NO_BROWSER=1

# Run for 15 seconds
timeout 16 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/logs.txt &
sleep 15
pkill -f "cargo run"

# Count spam
grep -c "CCO_PROJECT_PATH" /tmp/logs.txt || echo "0"
# Expected: 0-2 occurrences
```

### Test 3: Terminal Access

```bash
cd /Users/brent/git/cc-orchestra/cco
export NO_BROWSER=1

# Start server
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Test health endpoint
curl -s http://127.0.0.1:3000/health | jq '.'

# Kill server
kill -INT $PID
```

---

## Test File Locations

| File | Purpose |
|------|---------|
| `/tmp/comprehensive_test_suite.sh` | Automated test runner (8-10 min) |
| `/Users/brent/git/cc-orchestra/cco/COMPREHENSIVE_TEST_PLAN.md` | Detailed test scenarios & checklist |
| `/tmp/test_*.log` | Individual test logs |

---

## Expected Test Results

### Passing Test (All Green)

```
════════════════════════════════════════════════════════
COMPREHENSIVE CCO TEST SUITE - Issue Verification
════════════════════════════════════════════════════════

✓ PASS: Shutdown completed in 1245ms (target: < 2000ms)
✓ PASS: Shutdown message found: 'Server shut down gracefully'
✓ PASS: Port 3000 released successfully
✓ PASS: Process exited with acceptable status
✓ PASS: Spam messages within acceptable limits (2 total, target: ≤ 6)
✓ PASS: Spam messages don't repeat every 5 seconds (2 occurrences)
✓ PASS: Debug flag correctly increases verbosity
✓ PASS: Health endpoint responds successfully
✓ PASS: Terminal endpoint is accessible
✓ PASS: Complete lifecycle test passed (8500ms total)
✓ PASS: Shutdown completion verified in logs

Tests Executed: 11
Passed: 11
Failed: 0
Warnings: 0

════════════════════════════════════════════════════════
✓ ALL CRITICAL TESTS PASSED
════════════════════════════════════════════════════════
```

### Failing Test (Red Alert)

Any FAIL indicates the issue is not fully fixed. Check the detailed logs in `/tmp/test_*.log` for error messages.

---

## Debugging Failed Tests

### If Shutdown Test Fails

```bash
# Check the shutdown logs for errors
tail -50 /tmp/test_1_1.log | grep -E "ERROR|panic|failed"

# Monitor system processes
ps aux | grep cco | grep -v grep

# Check if port is stuck
lsof -i :3000
```

### If Logging Test Fails

```bash
# Count all occurrences of spam pattern
grep "CCO_PROJECT_PATH\|Current working\|Derived project" /tmp/test_2_1.log

# Show timing of spam (should be only at start)
grep -n "CCO_PROJECT_PATH" /tmp/test_2_1.log
```

### If Terminal Test Fails

```bash
# Check if health endpoint works
curl -v http://127.0.0.1:3000/health

# Check if server is running
ps aux | grep cargo | grep 3000

# Check logs for startup errors
tail -100 /tmp/test_3_1.log | grep -i "error\|failed"
```

---

## Performance Baseline

After fixes are verified, these are expected performance values:

| Metric | Target | Actual |
|--------|--------|--------|
| Shutdown time | < 2 seconds | ___ |
| Spam messages (15s) | ≤ 6 occurrences | ___ |
| Health endpoint response | < 100ms | ___ |
| Memory usage | < 50MB | ___ |
| CPU during idle | < 2% | ___ |

---

## Testing Checklist

Before declaring fixes complete:

- [ ] Test 1.1: Shutdown < 2 seconds
- [ ] Test 1.2: No zombie processes
- [ ] Test 1.3: Port released immediately
- [ ] Test 2.1: Spam messages ≤ 6 in 15s
- [ ] Test 2.2: Spam only at startup
- [ ] Test 2.3: Debug flag works correctly
- [ ] Test 3.1: Health endpoint responds
- [ ] Test 3.2: Terminal endpoint accessible
- [ ] Test 4.1: Full lifecycle works
- [ ] Test 4.2: Shutdown message in logs

---

## Quick Cleanup After Testing

```bash
# Kill any remaining processes
pkill -f "cargo run" 2>/dev/null || true
pkill -f "cco " 2>/dev/null || true

# Archive logs if needed
mkdir -p /tmp/test_logs
mv /tmp/test_*.log /tmp/test_logs/ 2>/dev/null || true

# Verify clean state
ps aux | grep cco | grep -v grep || echo "✓ Clean"
lsof -i :3000 || echo "✓ Port 3000 free"
```

---

## Report Test Results

After running tests, provide this summary:

```
Test Execution Report
====================
Date: [date and time]
Build Commit: [git log -1 --oneline]
Result: [PASS/FAIL]

Summary:
- Shutdown tests: [PASS/FAIL]
- Logging tests: [PASS/FAIL]
- Terminal tests: [PASS/FAIL]
- Integration test: [PASS/FAIL]

Issues (if any):
[List any failures or unexpected behavior]

Performance:
- Shutdown duration: [X]ms
- Spam message count: [X]
- Health endpoint response: [X]ms
```

---

## Still Failing?

1. Check the detailed test plan: `/Users/brent/git/cc-orchestra/cco/COMPREHENSIVE_TEST_PLAN.md`
2. Review individual test logs in `/tmp/test_*.log`
3. Check commit: `git log -1 --format="%H %s"`
4. Verify build: `cargo build --release 2>&1 | tail`
5. Check for issues: Look at recent commits for what was changed

---

## Next Steps

**After tests pass**:
1. Archive test logs
2. Document baseline performance
3. Update release notes with fixes
4. Prepare for production deployment

**If tests still fail**:
1. Review the fix implementation
2. Check for missed edge cases
3. Run individual tests for more details
4. Create issue for additional investigation

---

**Last Updated**: 2025-11-16
**Test Suite Version**: 1.0
**Target**: CCO v2025.11.x with critical fixes
