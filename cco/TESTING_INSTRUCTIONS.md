# Testing Instructions - Critical Issues Verification

**Complete guide for executing the comprehensive test suite.**

---

## Before You Start

### Prerequisites Checklist
- [ ] Latest code pulled from git
- [ ] No CCO processes running: `pkill -f "cco " 2>/dev/null || true`
- [ ] Port 3000 is available: `lsof -i :3000 || echo "Port free"`
- [ ] Rust toolchain installed: `rustc --version`
- [ ] At least 1GB free disk space
- [ ] Terminal access available

### Verify Build
```bash
cd /Users/brent/git/cc-orchestra/cco

# Build the release binary
cargo build --release 2>&1 | tail -5

# Verify binary exists
ls -lh target/release/cco
```

Expected output:
```
   Compiling cco v2025.11.X
    Finished release [optimized] target(s) in Xs
-rwxr-xr-x cco
```

---

## Quick Test (TL;DR)

For the impatient:

```bash
# Run all tests in one command
/tmp/comprehensive_test_suite.sh
```

Expected result: `✓ ALL CRITICAL TESTS PASSED`

Expected time: 8-10 minutes

**Done.** The three critical issues are verified as fixed.

---

## Detailed Testing Instructions

### Option 1: Automated Full Test Suite (Recommended)

**What it does**: Runs all 11 critical tests automatically with color-coded output.

**How to run**:
```bash
/tmp/comprehensive_test_suite.sh
```

**What to expect**:
1. Script starts and shows the test plan (15 seconds)
2. Executes Test Suite 1: Shutdown (2 minutes)
3. Executes Test Suite 2: Logging (3 minutes)
4. Executes Test Suite 3: Terminal (2 minutes)
5. Executes Test Suite 4: Integration (1 minute)
6. Shows summary and results (30 seconds)

**Total time**: 8-10 minutes

**Output example**:
```
✓ PASS: Shutdown completed in 1245ms (target: < 2000ms)
✓ PASS: Shutdown message found
✓ PASS: Port 3000 released successfully
✓ PASS: Spam messages within acceptable limits (2 total)
✓ PASS: Debug flag correctly increases verbosity
✓ PASS: Health endpoint responds successfully
✓ PASS: Terminal endpoint is accessible
✓ PASS: Complete lifecycle test passed

Tests Executed: 11
Passed: 11
Failed: 0
Warnings: 0

✓ ALL CRITICAL TESTS PASSED
```

---

### Option 2: Manual Testing with Detailed Documentation

**What it does**: Tests each component manually with detailed verification.

**When to use**: When automated tests fail and you need to debug individual components.

**How to run**:

1. **Review the comprehensive test plan**:
   ```bash
   cat /Users/brent/git/cc-orchestra/cco/COMPREHENSIVE_TEST_PLAN.md
   ```

2. **Print the detailed checklist**:
   ```bash
   cat /Users/brent/git/cc-orchestra/cco/TEST_VERIFICATION_CHECKLIST.md
   ```

3. **Execute individual tests** following the checklist procedures

4. **Record results** in TEST_VERIFICATION_CHECKLIST.md

**Total time**: 30-45 minutes (more thorough, better documentation)

---

### Option 3: Quick Reference Guide

**What it does**: Fast manual tests without extensive documentation.

**When to use**: When you want quick verification without full documentation.

**How to use**:
```bash
# Print quick guide
cat /Users/brent/git/cc-orchestra/cco/QUICK_TEST_GUIDE.md

# Follow individual test procedures listed
```

**Total time**: 15-20 minutes

---

## Understanding Test Results

### All Tests PASS (Green Light)

```
✓ ALL CRITICAL TESTS PASSED
```

**What this means**: All three issues are fixed and verified.

**Next steps**:
1. Archive test logs: `mkdir -p /tmp/test_logs_$(date +%Y%m%d); mv /tmp/test_*.log /tmp/test_logs_$(date +%Y%m%d)/`
2. Record performance baseline (see below)
3. Update release notes
4. Prepare for deployment

---

### Some Tests FAIL (Red Alert)

```
✗ FAIL: Shutdown took 3500ms (expected < 2000ms)
✗ FAIL: Excessive spam messages detected (15 total, expected ≤ 6)
```

**What this means**: At least one issue is not fully fixed.

**Next steps**:
1. Check the specific test log: `tail -100 /tmp/test_X_X.log`
2. Use the debugging guide in COMPREHENSIVE_TEST_PLAN.md
3. Identify which component needs attention
4. Wait for fixes, then re-run tests

**Debugging example**:
```bash
# Shutdown test failed?
tail -50 /tmp/test_1_1.log | grep -E "ERROR|panic|timeout"

# Logging test failed?
grep -c "CCO_PROJECT_PATH" /tmp/test_2_1.log

# Terminal test failed?
curl -v http://127.0.0.1:3000/health
```

---

### Tests Pass with WARNINGS (Yellow)

```
⚠ WARNING: Graceful shutdown message not found in logs
```

**What this means**: Test passed but something expected wasn't found.

**Next steps**:
1. Review the warning message
2. Check if the warning is acceptable
3. If acceptable, proceed normally
4. If not acceptable, investigate like a FAIL

---

## Performance Recording

After all tests PASS, record these baseline metrics:

```bash
# Extract from test output
# Find lines like "Shutdown duration: XXXms"
# Spam count: X
# etc.
```

**Create a performance baseline file**:

```bash
cat > /tmp/test_baseline_$(date +%Y%m%d_%H%M%S).txt << 'EOF'
CCO Performance Baseline
=======================
Date: [today's date]
Build: [commit hash from: git log -1 --oneline]

ISSUE #1 - SHUTDOWN PERFORMANCE
  Target: < 2000ms
  Actual: ___ ms
  Status: PASS / FAIL

ISSUE #2 - TERMINAL FUNCTIONALITY
  Status: PASS / FAIL
  Notes: ___________

ISSUE #3 - LOGGING SPAM
  Target: ≤ 6 messages in 15 seconds
  Actual: ___ messages
  Status: PASS / FAIL

OVERALL RESULT: ALL PASS / SOME FAIL
EOF

cat /tmp/test_baseline_*.txt
```

---

## Cleanup After Testing

### Remove Test Logs
```bash
# Archive logs for later reference
mkdir -p /tmp/test_logs_archive
mv /tmp/test_*.log /tmp/test_logs_archive/

# Or delete immediately if not needed
rm /tmp/test_*.log 2>/dev/null || true
```

### Kill Any Remaining Processes
```bash
# Kill any CCO processes
pkill -9 -f "cargo run.*port 3000" 2>/dev/null || true
pkill -9 -f "cco " 2>/dev/null || true
sleep 1

# Verify port is free
lsof -i :3000 || echo "✓ Port 3000 free"

# Verify no processes
ps aux | grep -E "cargo|cco" | grep -v grep || echo "✓ No CCO processes"
```

---

## Troubleshooting Failed Tests

### If Shutdown Test Fails

**Symptom**: `Shutdown took 5000ms (expected < 2000ms)`

**Investigation**:
```bash
# Check what's happening during shutdown
tail -100 /tmp/test_1_1.log | tail -30

# Look for these error patterns
grep -i "error\|panic\|timeout\|stuck" /tmp/test_1_1.log

# Check what processes are still running
ps aux | grep cco | grep -v grep

# Manually test shutdown
timeout 5 cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3
kill -INT $PID
wait $PID
echo "Exit code: $?"
```

**Files to check**:
- `src/server.rs` - Shutdown handler
- `src/main.rs` - Signal handling
- `src/lib.rs` - Cleanup logic

---

### If Logging Test Fails

**Symptom**: `Excessive spam messages detected (25 total, expected ≤ 6)`

**Investigation**:
```bash
# Count exact spam messages
grep "CCO_PROJECT_PATH" /tmp/test_2_1.log | wc -l
grep "Current working" /tmp/test_2_1.log | wc -l
grep "Derived project" /tmp/test_2_1.log | wc -l

# Show when they appear
grep -n "CCO_PROJECT_PATH" /tmp/test_2_1.log

# Check if they're repeating every 5 seconds
grep "CCO_PROJECT_PATH" /tmp/test_2_1.log | head -5
```

**Files to check**:
- `src/server.rs` - Logging initialization
- `src/lib.rs` - Logger setup
- `cco/src/analytics.rs` - Analytics logging

---

### If Terminal Test Fails

**Symptom**: `Terminal endpoint is accessible - FAIL`

**Investigation**:
```bash
# Start server and test endpoints manually
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3

# Test health endpoint
curl -v http://127.0.0.1:3000/health

# Test terminal endpoint
curl -i -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: test" \
  http://127.0.0.1:3000/terminal

# Check logs for errors
kill -INT $PID
wait $PID
```

**Files to check**:
- `src/router.rs` - Route registration
- `src/server.rs` - Handler setup
- `src/terminal.rs` - Terminal implementation

---

## Advanced Testing

### Test Specific Component Only

**Test shutdown only**:
```bash
# Run just shutdown tests
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
export NO_BROWSER=1
timeout 5 cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3
time kill -INT $PID
wait $PID
```

**Test logging only**:
```bash
# Monitor logs for 15 seconds
timeout 16 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/logging.log &
sleep 15
pkill -f cargo
grep -c "CCO_PROJECT_PATH" /tmp/logging.log
```

**Test terminal only**:
```bash
# Start server and test endpoints
cargo run --release -- run --debug --port 3000 &
PID=$!
sleep 3
curl http://127.0.0.1:3000/health
kill -INT $PID
```

---

## Test Documentation Templates

### Test Execution Log
```
TEST EXECUTION LOG
==================

Date: [DATE]
Tester: [NAME]
Build: [COMMIT HASH]

Test Suite 1 - Shutdown:
  Test 1.1: ✓ PASS (duration: XXXms)
  Test 1.2: ✓ PASS
  Test 1.3: ✓ PASS
  Test 1.4: ✓ PASS
  Test 1.5: ✓ PASS

Test Suite 2 - Logging:
  Test 2.1: ✓ PASS (spam: X)
  Test 2.2: ✓ PASS
  Test 2.3: ✓ PASS
  Test 2.4: ✓ PASS

Test Suite 3 - Terminal:
  Test 3.1: ✓ PASS
  Test 3.2: ✓ PASS
  Test 3.3: ✓ PASS
  Test 3.4: ✓ PASS

Test Suite 4 - Integration:
  Test 4.1: ✓ PASS

OVERALL RESULT: ✓ ALL TESTS PASSED

Issues Found: None
Recommendations: Ready for release
```

---

## Test Execution Checklist

Before starting:
- [ ] Code is built: `cargo build --release`
- [ ] Processes killed: `pkill -f cco`
- [ ] Port is free: `lsof -i :3000 || echo "free"`
- [ ] 1GB+ disk space available

During testing:
- [ ] Running automated test: `/tmp/comprehensive_test_suite.sh`
- [ ] Monitoring output for failures
- [ ] Noting any warnings or issues
- [ ] Waiting for test completion

After testing:
- [ ] Reviewing test results
- [ ] Archiving logs if needed
- [ ] Recording performance baseline
- [ ] Documenting any issues found

---

## Support & Reference

### Quick Reference Links

**Documents**:
- Full test plan: `/Users/brent/git/cc-orchestra/cco/COMPREHENSIVE_TEST_PLAN.md`
- Quick guide: `/Users/brent/git/cc-orchestra/cco/QUICK_TEST_GUIDE.md`
- Checklist: `/Users/brent/git/cc-orchestra/cco/TEST_VERIFICATION_CHECKLIST.md`
- Summary: `/Users/brent/git/cc-orchestra/cco/TEST_DELIVERABLES_SUMMARY.md`

**Test Script**:
- Automated runner: `/tmp/comprehensive_test_suite.sh`

**Issues**:
1. Ctrl+C shutdown (Issue #1)
2. Terminal prompt (Issue #2)
3. Logging spam (Issue #3)

---

## Next Steps After Testing

### If All Tests PASS:
1. Verify performance baseline (all under target)
2. Archive test logs
3. Update CHANGELOG with fixes
4. Prepare release notes
5. Merge to production branch
6. Create release tag

### If Some Tests FAIL:
1. Review detailed failure information
2. Check component-specific logs
3. Identify root cause
4. File issue or wait for fixes
5. Re-run tests after fixes applied

---

## Questions or Issues?

Refer to the comprehensive documentation:
- **Detailed specs**: COMPREHENSIVE_TEST_PLAN.md
- **Quick answers**: QUICK_TEST_GUIDE.md
- **Full checklist**: TEST_VERIFICATION_CHECKLIST.md
- **Overview**: TEST_DELIVERABLES_SUMMARY.md

---

**Status**: Ready to execute
**Last Updated**: 2025-11-16
**Test Suite Version**: 1.0
