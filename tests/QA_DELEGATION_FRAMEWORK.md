# QA Delegation Framework: Dashboard Acceptance Tests

**Document**: QA Engineer Delegation Framework
**Date**: November 17, 2025
**Status**: Ready for Agent Execution
**Recipient**: QA Engineer Agent

---

## Executive Summary

This framework delegates comprehensive dashboard acceptance testing to the QA Engineer agent. The work includes:

1. **Automated Test Execution**: Run the dashboard acceptance test script
2. **Manual Validation**: Verify results and investigate any failures
3. **Report Generation**: Document findings in standardized format
4. **Production Sign-Off**: Determine readiness for deployment

**Expected Duration**: 30-60 minutes
**Complexity**: Medium
**Risk Level**: Low (read-only, non-destructive tests)

---

## Delegation Details

### QA Engineer Responsibilities

As the designated QA Engineer, you are responsible for:

1. **Test Setup Verification**
   - Confirm server is running on `http://127.0.0.1:3000`
   - Verify all prerequisites installed (curl, jq, bash)
   - Document system environment

2. **Automated Test Execution**
   - Execute: `bash tests/dashboard-acceptance-tests.sh`
   - Capture full output
   - Document any warnings or anomalies

3. **Test Analysis**
   - Review each test result
   - Investigate any failures
   - Determine root causes
   - Collect diagnostic information

4. **Report Generation**
   - Document all findings
   - Generate comprehensive report
   - Provide production readiness assessment
   - Identify action items if needed

5. **Production Sign-Off**
   - Determine if system is "READY FOR PRODUCTION"
   - Document any critical issues
   - Provide recommendations
   - Sign off on readiness

---

## Test Framework Overview

### 8 Comprehensive Acceptance Tests

| # | Test | Purpose | Validates |
|---|------|---------|-----------|
| 1 | Dashboard Loads | Basic functionality | HTTP 200, valid HTML |
| 2 | No JSON Errors | Data parsing | No parse error messages |
| 3 | Dashboard.js Loads | Asset delivery | Script cache-busting |
| 4 | API Data Loads | Data endpoints | 3 critical APIs |
| 5 | WebSocket Terminal | Real-time comms | Upgrade handshake |
| 6 | SSE Stream | Live updates | Event streaming |
| 7 | Full Feature Flow | End-to-end | User workflow |
| 8 | Error Scenarios | Error handling | Graceful degradation |

### Test Execution

**Automated Script**:
```bash
bash tests/dashboard-acceptance-tests.sh
```

**Location**: `/Users/brent/git/cc-orchestra/tests/dashboard-acceptance-tests.sh`

**Documentation**: `/Users/brent/git/cc-orchestra/tests/DASHBOARD_ACCEPTANCE_CRITERIA.md`

---

## Step-by-Step Execution Guide

### Step 1: Verify Prerequisites

```bash
# Check server is running
curl -s http://127.0.0.1:3000/ > /dev/null && echo "Server OK" || echo "Server DOWN"

# Verify curl is installed
curl --version | head -1

# Verify jq is installed
jq --version

# Verify bash version
bash --version | head -1
```

**Expected Output**:
```
Server OK
curl 7.x.x (...)
jq-1.x
GNU bash, version 4.x.x
```

**If Prerequisites Missing**:
- Install jq: `brew install jq` (macOS) or `apt-get install jq` (Linux)
- Ensure curl is in PATH
- Use bash 4.0+

---

### Step 2: Run Automated Tests

```bash
# Change to repository root
cd /Users/brent/git/cc-orchestra

# Make script executable
chmod +x tests/dashboard-acceptance-tests.sh

# Run with full output
bash tests/dashboard-acceptance-tests.sh

# Optionally capture to file
bash tests/dashboard-acceptance-tests.sh | tee test-results/dashboard-acceptance-$(date +%Y%m%d-%H%M%S).txt
```

**Expected Duration**: 30-60 seconds

**Expected Result**: Summary with test counts and verdict

---

### Step 3: Analyze Results

```bash
# If using output capture, review the file
cat test-results/dashboard-acceptance-*.txt

# Key sections to verify:
# - Test Summary: Check PASSED and FAILED counts
# - Individual Results: Verify each test status
# - Overall Verdict: Check for "READY FOR PRODUCTION"
```

**Success Criteria**:
- Total Tests: 22+ (depending on implementation)
- Passed: All
- Failed: 0
- Verdict: "READY FOR PRODUCTION"

---

### Step 4: Investigate Failures (If Any)

If any tests fail, follow the failure investigation guide:

**For Test 1 (Dashboard Loads) Failure**:
```bash
curl -v http://127.0.0.1:3000/ 2>&1 | head -20
# Check HTTP status and HTML validity
```

**For Test 4 (API Data) Failure**:
```bash
curl -s http://127.0.0.1:3000/api/agents | jq .
curl -s http://127.0.0.1:3000/api/metrics/projects | jq .
curl -s http://127.0.0.1:3000/api/stats | jq .
# Verify JSON validity for each
```

**For Test 5 (WebSocket) Failure**:
```bash
curl -v -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
  http://127.0.0.1:3000/terminal 2>&1 | head -10
# Check for HTTP 101 or 400 response
```

**For Test 6 (SSE) Failure**:
```bash
timeout 5 curl -s -N http://127.0.0.1:3000/api/stream
# Verify data events with valid JSON
```

---

### Step 5: Generate Report

Create a comprehensive report with the following structure:

```markdown
# Dashboard Acceptance Test Report

**Date**: [Date/Time]
**Executed By**: [Agent Name]
**System**: [macOS/Linux version]
**Server**: http://127.0.0.1:3000

## Test Results Summary

Total Tests: X
Passed: X
Failed: X
Success Rate: X%

## Individual Test Results

### Test 1: Dashboard Loads Without Errors
- Status: [PASS/FAIL]
- HTTP Status: 200
- HTML Valid: Yes
- Notes: [Any observations]

### Test 2: No JSON Parse Errors
- Status: [PASS/FAIL]
- Error Message Found: No
- Notes: [Any observations]

### Test 3: Dashboard.js Loads Correctly
- Status: [PASS/FAIL]
- Cache-Bust Parameter: v=abc123def
- Notes: [Any observations]

### Test 4: API Data Loads
- /api/agents: [PASS/FAIL]
- /api/metrics/projects: [PASS/FAIL]
- /api/stats: [PASS/FAIL]
- Notes: [Any observations]

### Test 5: WebSocket Terminal Works
- Status: [PASS/FAIL]
- Response Code: 101/400
- Notes: [Any observations]

### Test 6: SSE Stream Works
- Status: [PASS/FAIL]
- Events Received: X
- JSON Valid: Yes
- Notes: [Any observations]

### Test 7: Full Feature Flow
- Status: [PASS/FAIL]
- Steps Completed: 4/4
- Duration: X.XXs
- Performance OK: Yes
- Notes: [Any observations]

### Test 8: Error Scenarios
- Invalid Endpoint: [PASS/FAIL]
- Malformed Request: [PASS/FAIL]
- Notes: [Any observations]

## Issues Found

[If any, list with:
- Issue ID
- Severity (Critical/High/Medium/Low)
- Description
- Steps to Reproduce
- Recommended Action
]

## Performance Analysis

- Dashboard Load: X.XXms
- API Response Time: X.XXms average
- Total Feature Flow: X.XXs
- Performance Rating: Excellent/Good/Acceptable/Poor

## Production Readiness Assessment

**Verdict**: READY FOR PRODUCTION / NEEDS FIXES / BLOCKED

**Critical Issues**: [List if any]
**Recommended Actions**: [List if any]

**Sign-Off**:
- Status: Ready/Not Ready
- Confidence: High/Medium/Low
- Date: [Date]
- QA Engineer: [Name]
```

---

## Coordination Protocol

### Before Starting

1. **Read full context**:
   - `/Users/brent/git/cc-orchestra/tests/DASHBOARD_ACCEPTANCE_CRITERIA.md`
   - This document

2. **Verify environment**:
   ```bash
   # Confirm server running
   curl http://127.0.0.1:3000/ > /dev/null && echo "OK" || echo "FAILED"
   ```

3. **Check prerequisites**:
   - curl available: `curl --version`
   - jq available: `jq --version`
   - bash 4.0+: `bash --version`

### During Execution

1. **Run tests**:
   ```bash
   bash tests/dashboard-acceptance-tests.sh
   ```

2. **Document results**:
   - Capture full output
   - Note any warnings
   - Record timing information

3. **Investigate failures**:
   - Use provided failure guides
   - Run diagnostic commands
   - Collect evidence

### After Completion

1. **Generate report**:
   - Create comprehensive report
   - Include all test results
   - Provide production assessment

2. **Communicate findings**:
   - "All tests PASSED - READY FOR PRODUCTION"
   - "Tests FAILED - Issues found requiring fixes"
   - Provide specific details

3. **Store results**:
   - Save report to: `/Users/brent/git/cc-orchestra/test-results/`
   - Filename: `dashboard-acceptance-{TIMESTAMP}.md`

---

## Success Criteria

The QA Engineer has successfully completed this task when:

✅ All 8 tests executed
✅ All tests passed (or failures documented)
✅ Performance analyzed
✅ Comprehensive report generated
✅ Production readiness determined
✅ No blockers remaining

---

## Escalation Path

If critical issues discovered:

1. **Document**: Describe issue clearly with reproduction steps
2. **Analyze**: Determine severity and impact
3. **Report**: Communicate findings to team
4. **Defer**: Mark as "NEEDS FIXES" until resolved
5. **Retest**: Execute tests again after fixes

---

## Additional Resources

- **Test Script**: `tests/dashboard-acceptance-tests.sh`
- **Acceptance Criteria**: `tests/DASHBOARD_ACCEPTANCE_CRITERIA.md`
- **Server Logs**: Check application logs for errors
- **API Documentation**: Refer to API spec in project docs
- **Failure Guide**: Section 8 of acceptance criteria document

---

## Knowledge Manager Integration

Before beginning, check stored knowledge:

```bash
# Search for relevant context
node src/knowledge-manager.js search "dashboard" 2>/dev/null
node src/knowledge-manager.js search "api" 2>/dev/null
```

During execution, store findings:

```bash
# Store test results
node src/knowledge-manager.js store "Dashboard tests executed: X passed, Y failed" \
  --type completion --agent qa-engineer 2>/dev/null
```

After completion, document:

```bash
# Store final assessment
node src/knowledge-manager.js store "Dashboard READY FOR PRODUCTION - all tests passed" \
  --type completion --agent qa-engineer 2>/dev/null
```

---

## Frequently Asked Questions

**Q: What if the server is not running?**
A: Start it with: `cargo run --bin cco -- server --port 3000` (or appropriate command)

**Q: What if tests timeout?**
A: Check network connectivity and server load. May indicate slow responses.

**Q: What if JSON parsing fails?**
A: Verify `jq` is installed and working: `jq --version`

**Q: Can I run individual tests?**
A: Yes, the script is modular. Source it and call individual test functions.

**Q: How do I capture test output?**
A: `bash tests/dashboard-acceptance-tests.sh | tee output.txt`

**Q: What if a test is flaky?**
A: Run multiple times to confirm consistency. Document if intermittent.

---

## Timeline

- **Execution**: 30-60 minutes
- **Analysis**: 15-30 minutes
- **Report Generation**: 10-15 minutes
- **Total**: 55-105 minutes

---

## Acceptance Sign-Off

**Orchestrator**: I am delegating comprehensive dashboard acceptance testing to the QA Engineer.

**Responsibility Transfer**:
- Test Execution: QA Engineer
- Result Analysis: QA Engineer
- Report Generation: QA Engineer
- Production Assessment: QA Engineer

**Expected Deliverable**: Comprehensive test report with production readiness determination

**Timeline**: As soon as QA Engineer can execute

---

**Status**: Ready for QA Engineer Execution
**Last Updated**: November 17, 2025
**Prepared By**: Orchestrator (Claude Code)
