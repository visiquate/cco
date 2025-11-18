# Dashboard Testing - Quick Start Guide

**Status**: Ready for Execution
**Time Required**: 60 minutes
**Complexity**: Medium

---

## TL;DR - Run This Now

```bash
# Navigate to project root
cd /Users/brent/git/cc-orchestra

# Make test script executable
chmod +x tests/dashboard-acceptance-tests.sh

# Run comprehensive tests
bash tests/dashboard-acceptance-tests.sh
```

**Expected Result**:
- `READY FOR PRODUCTION` (all tests pass)
- or `NEEDS FIXES` (issues found - see details)

---

## Quick Reference Files

| File | Purpose | Action |
|------|---------|--------|
| `tests/dashboard-acceptance-tests.sh` | Run this file | `bash tests/dashboard-acceptance-tests.sh` |
| `tests/DASHBOARD_ACCEPTANCE_CRITERIA.md` | Reference this | Read if tests fail |
| `tests/QA_DELEGATION_FRAMEWORK.md` | Follow this | Detailed execution guide |
| `ORCHESTRATOR_DASHBOARD_TESTING_SUMMARY.md` | Understand context | Optional background |

---

## Prerequisites (1 minute)

```bash
# Verify server running
curl http://127.0.0.1:3000/ > /dev/null && echo "OK" || echo "ERROR"

# Verify jq installed
jq --version || echo "Install: brew install jq"

# Verify curl available
curl --version | head -1

# Verify bash version
bash --version | head -1
```

---

## Execute Tests (30-60 seconds)

```bash
cd /Users/brent/git/cc-orchestra
bash tests/dashboard-acceptance-tests.sh
```

### Output Will Look Like:

```
================================================================================
Dashboard Acceptance Test Suite
================================================================================

[TEST] Test 1: Dashboard Loads Without Errors
[PASS] HTTP status is 200
[PASS] HTML contains <body> tag
[PASS] HTML contains <script> tags
[PASS] No error messages in HTML

[TEST] Test 2: No JSON Parse Errors
[PASS] No JSON parse error message found

... (more tests) ...

================================================================================
Test Summary
================================================================================

Total Tests:  22
Passed:       22
Failed:       0

================================================================================
OVERALL VERDICT: READY FOR PRODUCTION
================================================================================
```

---

## Interpret Results

### Success (All Tests Pass)
```
VERDICT: READY FOR PRODUCTION
Action: System is acceptable for production deployment
```

### Failure (Any Test Fails)
```
VERDICT: NEEDS FIXES
Action: Review failed tests and follow investigation guide
```

---

## If Tests Fail - Quick Diagnostics

### Test 1 (Dashboard Loads) Failed?
```bash
curl -v http://127.0.0.1:3000/ 2>&1 | head -20
```

### Test 4 (API Endpoints) Failed?
```bash
curl -s http://127.0.0.1:3000/api/agents | jq .
curl -s http://127.0.0.1:3000/api/stats | jq .
curl -s http://127.0.0.1:3000/api/metrics/projects | jq .
```

### Test 5 (WebSocket) Failed?
```bash
curl -v -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
  http://127.0.0.1:3000/terminal 2>&1 | head -10
```

### Test 6 (SSE Stream) Failed?
```bash
timeout 5 curl -s -N http://127.0.0.1:3000/api/stream | head -20
```

---

## Save Results (Optional)

```bash
# Capture test output to file
bash tests/dashboard-acceptance-tests.sh | tee test-results/dashboard-$(date +%Y%m%d-%H%M%S).txt

# View saved results
cat test-results/dashboard-*.txt
```

---

## 8 Tests Explained (30 seconds each)

| # | Test | What It Does | Pass If |
|---|------|--------------|---------|
| 1 | Dashboard Loads | Requests home page | HTTP 200 + HTML valid |
| 2 | No JSON Errors | Checks for parse errors | Error message not found |
| 3 | Dashboard.js | Verifies script loading | Cache-bust param present |
| 4 | API Data | Requests 3 APIs | All return 200 + JSON |
| 5 | WebSocket | Requests upgrade | HTTP 101 or 400 response |
| 6 | SSE Stream | Tests live events | Data events with JSON |
| 7 | Full Flow | Complete workflow | 4 steps in < 2s |
| 8 | Error Handling | Tests bad requests | Handled gracefully |

---

## Production Readiness Checklist

Before deploying, verify:

- [ ] All 8 test categories passed
- [ ] No critical issues found
- [ ] Performance acceptable (< 2 seconds)
- [ ] Error scenarios handled
- [ ] Test report generated
- [ ] QA sign-off obtained

---

## Common Issues & Fixes

### Server Not Running
```bash
# Start server (example command)
cargo run --bin cco -- server --port 3000
# or
docker compose up
# or
./start-server.sh
```

### jq Not Found
```bash
# macOS
brew install jq

# Linux (Ubuntu/Debian)
sudo apt-get install jq

# Verify
jq --version
```

### Tests Timeout
```bash
# Check if server is slow
curl -w "Time: %{time_total}s\n" http://127.0.0.1:3000/

# Check server logs for errors
tail -50 /var/log/cco.log
# or
docker logs cco  # if using Docker
```

### Tests Intermittently Fail (Flaky)
```bash
# Run tests multiple times
for i in {1..3}; do bash tests/dashboard-acceptance-tests.sh; done
```

---

## Next Steps After Testing

### If READY FOR PRODUCTION
1. Generate final report
2. Archive test results
3. Proceed with deployment
4. Monitor production

### If NEEDS FIXES
1. Review failures (see details in output)
2. Fix identified issues
3. Re-run tests
4. Iterate until all pass

---

## File Locations Reference

```
/Users/brent/git/cc-orchestra/
├── tests/
│   ├── dashboard-acceptance-tests.sh       ← RUN THIS
│   ├── DASHBOARD_ACCEPTANCE_CRITERIA.md    ← READ IF TESTS FAIL
│   └── QA_DELEGATION_FRAMEWORK.md          ← DETAILED GUIDE
├── ORCHESTRATOR_DASHBOARD_TESTING_SUMMARY.md  ← BACKGROUND
└── DASHBOARD_TESTING_QUICK_START.md        ← THIS FILE
```

---

## 60-Second Summary

1. **Setup** (5 sec): Verify prerequisites
2. **Execute** (30 sec): Run `bash tests/dashboard-acceptance-tests.sh`
3. **Review** (15 sec): Check results
4. **Decide** (10 sec): READY or NEEDS FIXES

---

## Help & Support

- **Test Details**: Read `tests/DASHBOARD_ACCEPTANCE_CRITERIA.md`
- **Full Instructions**: Read `tests/QA_DELEGATION_FRAMEWORK.md`
- **Background**: Read `ORCHESTRATOR_DASHBOARD_TESTING_SUMMARY.md`
- **Server**: Ensure port 3000 is running

---

## Version

- **Created**: November 17, 2025
- **Status**: Ready for Use
- **Execution Model**: Automated bash script with manual fallback

---

**GET STARTED**: `bash tests/dashboard-acceptance-tests.sh`
