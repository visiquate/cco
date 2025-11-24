# Dashboard Acceptance Test Criteria

**Document**: Dashboard Acceptance Testing Framework
**Created**: November 17, 2025
**Purpose**: Define comprehensive acceptance criteria for the CCO dashboard and provide automated testing framework
**Status**: Ready for QA Execution

---

## Overview

This document defines 8 comprehensive acceptance tests for the Claude Code Orchestrator (CCO) dashboard. These tests validate:

1. Dashboard loading and HTML validity
2. Absence of JSON parsing errors
3. Proper script loading with cache-busting
4. API endpoint functionality
5. WebSocket terminal connectivity
6. Server-Sent Events (SSE) stream operation
7. Complete user feature flow
8. Error handling and graceful degradation

---

## Prerequisites

Before running tests, ensure:

- **Server**: Running on `http://127.0.0.1:3000`
- **Tools**:
  - `curl` (for HTTP requests)
  - `jq` (for JSON validation)
  - `bash` 4.0+
- **Network**: Local TCP connectivity to port 3000
- **Duration**: ~30-60 seconds for full test suite

---

## Detailed Test Specifications

### Test 1: Dashboard Loads Without Errors

**Purpose**: Verify basic dashboard availability and HTML integrity

**Acceptance Criteria**:
- HTTP response status is 200 (OK)
- Response contains valid HTML structure
- HTML includes `<body>` tag
- HTML includes `<script>` tags
- No error messages present in HTML

**Test Steps**:
```bash
curl -s -w "\n%{http_code}" http://127.0.0.1:3000/
```

**Validation**:
```
HTTP/1.1 200 OK
[Valid HTML content with <body> and <script>]
[No error messages, exceptions, or failures]
```

**Pass Criteria**: All 4 sub-checks pass (HTTP 200, valid HTML, contains body, contains scripts, no errors)

**Failure Examples**:
- HTTP 500 error
- Malformed HTML
- Missing `<body>` or `<script>` tags
- Error messages like "Fatal Error" or "Exception"

---

### Test 2: No JSON Parse Errors

**Purpose**: Verify that JSON parsing is working correctly and error messages don't indicate parse failures

**Acceptance Criteria**:
- Dashboard HTML does NOT contain error message: "Failed to load data: JSON parse error"
- All API responses return valid JSON
- No JSON parse errors in browser console simulation

**Test Steps**:
```bash
curl -s http://127.0.0.1:3000/ | grep -q "Failed to load data: JSON parse error"
echo $?  # Should return 1 (not found)
```

**Validation**:
- Error message should NOT appear anywhere
- All API endpoints return parseable JSON

**Pass Criteria**: Error message not found in dashboard HTML

**Failure Examples**:
- Error message present in HTML
- API returns non-JSON content (e.g., HTML error pages)
- Malformed JSON responses

---

### Test 3: Dashboard.js Loads Correctly

**Purpose**: Verify the main dashboard JavaScript file loads with proper cache-busting

**Acceptance Criteria**:
- Dashboard HTML includes `<script>` tag referencing `dashboard.js`
- Script tag includes cache-bust query parameter `?v=`
- Cache-bust parameter contains non-empty version hash
- Script tag format is correct

**Test Steps**:
```bash
curl -s http://127.0.0.1:3000/ | grep -o 'src="[^"]*dashboard\.js[^"]*"'
```

**Expected Output**:
```
src="dashboard.js?v=abc123def456"
```

**Pass Criteria**: Script tag found with `?v=<version>` parameter

**Failure Examples**:
- Script tag missing
- No cache-bust parameter
- Incorrect script path
- Empty version parameter

---

### Test 4: API Data Loads

**Purpose**: Verify all critical API endpoints return valid JSON responses

**Acceptance Criteria**:
- `/api/agents` endpoint returns HTTP 200 with valid JSON
- `/api/metrics/projects` endpoint returns HTTP 200 with valid JSON
- `/api/stats` endpoint returns HTTP 200 with valid JSON
- All endpoints respond within reasonable time (<2 seconds)

**Test Steps**:
```bash
# Test each endpoint
curl -s -w "\n%{http_code}" http://127.0.0.1:3000/api/agents
curl -s -w "\n%{http_code}" http://127.0.0.1:3000/api/metrics/projects
curl -s -w "\n%{http_code}" http://127.0.0.1:3000/api/stats
```

**Expected Output**:
```
HTTP/1.1 200 OK
[Valid JSON object/array]
```

**Pass Criteria**: All 3 endpoints return HTTP 200 with valid JSON

**Failure Examples**:
- HTTP 404 (endpoint not found)
- HTTP 500 (server error)
- Non-JSON response
- Malformed JSON

---

### Test 5: WebSocket Terminal Works

**Purpose**: Verify WebSocket upgrade capability for terminal feature

**Acceptance Criteria**:
- WebSocket upgrade request to `/terminal` is recognized
- Server responds with HTTP 101 (Upgrade) or HTTP 400 (validation error)
- Connection upgrade mechanism is functional

**Test Steps**:
```bash
curl -s -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
  -H "Sec-WebSocket-Version: 13" \
  http://127.0.0.1:3000/terminal | head -1
```

**Expected Output**:
```
HTTP/1.1 101 Switching Protocols
or
HTTP/1.1 400 Bad Request
```

**Pass Criteria**: HTTP 101 (successful upgrade) or HTTP 400 (validation of upgrade request)

**Failure Examples**:
- HTTP 404 (endpoint not found)
- HTTP 500 (server error)
- Timeout or no response
- Connection refused

---

### Test 6: SSE Stream Works

**Purpose**: Verify Server-Sent Events (SSE) stream functionality for real-time updates

**Acceptance Criteria**:
- `/api/stream` endpoint returns HTTP 200
- Response includes data events in SSE format (`data: {...}`)
- Event data contains valid JSON
- Stream continues for duration of connection

**Test Steps**:
```bash
timeout 5 curl -s -N http://127.0.0.1:3000/api/stream | head -20
```

**Expected Output**:
```
data: {"event":"type","timestamp":"..."}
data: {"status":"active"}
...
```

**Pass Criteria**:
- At least one `data:` event received
- All data is valid JSON
- Events continue without errors

**Failure Examples**:
- No response or timeout
- Malformed SSE format
- Invalid JSON in event data
- Connection closes prematurely

---

### Test 7: Full Feature Flow

**Purpose**: Verify complete user interaction flow from dashboard load through API calls

**Acceptance Criteria**:
- User can load dashboard: HTTP 200
- Dashboard can fetch initial stats: HTTP 200, valid JSON
- Dashboard can fetch agent list: HTTP 200, valid JSON
- Dashboard can fetch project metrics: HTTP 200, valid JSON
- Complete flow finishes in under 2 seconds
- All responses are valid JSON (parseable)

**Test Steps**:
1. Load dashboard: `GET http://127.0.0.1:3000/`
2. Fetch stats: `GET http://127.0.0.1:3000/api/stats`
3. Fetch agents: `GET http://127.0.0.1:3000/api/agents`
4. Fetch metrics: `GET http://127.0.0.1:3000/api/metrics/projects`
5. Measure total time

**Acceptance Thresholds**:
- All steps succeed (HTTP 200)
- All responses valid JSON
- Total time < 2 seconds

**Failure Examples**:
- Any step fails with HTTP error
- Any response is invalid JSON
- Total time exceeds 2 seconds
- Connection timeouts

---

### Test 8: Error Scenarios

**Purpose**: Verify graceful error handling for invalid requests

**Acceptance Criteria**:
- Invalid endpoint returns appropriate error status (404 or 500)
- Malformed POST request handled gracefully
- Server doesn't crash or hang
- Error responses are meaningful

**Test Steps**:

**Invalid Endpoint**:
```bash
curl -s -w "\n%{http_code}" http://127.0.0.1:3000/api/invalid
```

**Malformed POST**:
```bash
curl -s -w "\n%{http_code}" -X POST http://127.0.0.1:3000/api/agents
```

**Expected Output**:
```
HTTP/1.1 404 Not Found
or
HTTP/1.1 400 Bad Request
or
HTTP/1.1 405 Method Not Allowed
```

**Pass Criteria**:
- Invalid endpoint returns 404 or 500
- Malformed POST returns 400, 404, or 405
- No server crashes
- Responses are consistent

**Failure Examples**:
- Unexpected HTTP status codes
- Server crash or hang
- Missing error information
- Inconsistent error handling

---

## Test Execution

### Using the Automated Test Script

```bash
# Make script executable
chmod +x tests/dashboard-acceptance-tests.sh

# Run all tests
bash tests/dashboard-acceptance-tests.sh
```

### Expected Output

```
================================================================================
Dashboard Acceptance Test Suite
================================================================================

Target: http://127.0.0.1:3000
Time: [date and time]

[TEST] Test 1: Dashboard Loads Without Errors
[PASS] HTTP status is 200
[PASS] HTML contains <body> tag
[PASS] HTML contains <script> tags
[PASS] No error messages in HTML

[TEST] Test 2: No JSON Parse Errors
[PASS] No JSON parse error message found

[TEST] Test 3: Dashboard.js Loads Correctly
[PASS] dashboard.js script tag found
[PASS] Cache-bust parameter found: v=abc123

[TEST] Test 4: API Data Loads
[PASS] /api/agents returns HTTP 200
[PASS] /api/agents returns valid JSON
[PASS] /api/metrics/projects returns HTTP 200
[PASS] /api/metrics/projects returns valid JSON
[PASS] /api/stats returns HTTP 200
[PASS] /api/stats returns valid JSON

[TEST] Test 5: WebSocket Terminal Works
[PASS] WebSocket upgrade successful (HTTP 101)

[TEST] Test 6: SSE Stream Works
[PASS] SSE stream returns data events
[PASS] SSE stream produced 10 data events
[PASS] SSE event data is valid JSON

[TEST] Test 7: Full Feature Flow
[PASS] Dashboard loaded
[PASS] API stats loaded
[PASS] API agents loaded
[PASS] API metrics loaded
[PASS] Full feature flow completed: 4/4 steps, 1.23s
[PASS] Performance acceptable (< 2 seconds)

[TEST] Test 8: Error Scenarios
[PASS] Invalid endpoint handled gracefully (HTTP 404)
[PASS] Malformed request handled gracefully (HTTP 400)

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

## Acceptance Criteria Summary

### Pass Criteria (All Must Pass)

| Test | Criteria | Status |
|------|----------|--------|
| 1. Dashboard Loads | HTTP 200, valid HTML, no errors | PASS |
| 2. No JSON Errors | Error message not present | PASS |
| 3. Dashboard.js Loads | Script tag with v= parameter | PASS |
| 4. API Data Loads | 3 endpoints return 200 + valid JSON | PASS |
| 5. WebSocket Terminal | HTTP 101 or 400 response | PASS |
| 6. SSE Stream | Data events with valid JSON | PASS |
| 7. Full Feature Flow | 4 steps, < 2 seconds, valid JSON | PASS |
| 8. Error Scenarios | Graceful handling of invalid requests | PASS |

### Production Readiness

Dashboard is **READY FOR PRODUCTION** when:
- ✅ All 8 tests pass
- ✅ No critical issues identified
- ✅ Performance acceptable (< 2 seconds)
- ✅ Error handling graceful

Dashboard requires **FIXES** if:
- ❌ Any test fails
- ❌ Critical issues identified
- ❌ Performance degraded
- ❌ Error handling inadequate

---

## Failure Investigation Guide

### If Test 1 Fails (Dashboard Loads)

**Check**:
- Server running: `curl http://127.0.0.1:3000/`
- Server logs for errors: `journalctl -u cco` or application logs
- HTML generation: Check template files in `cco/static/`
- HTTP status codes: Use `curl -v`

**Common Causes**:
- Server crashed or not running
- Template files missing or corrupted
- Port 3000 in use by another process

---

### If Test 4 Fails (API Data Loads)

**Check**:
- Each endpoint individually:
  ```bash
  curl -v http://127.0.0.1:3000/api/agents
  curl -v http://127.0.0.1:3000/api/metrics/projects
  curl -v http://127.0.0.1:3000/api/stats
  ```
- API endpoint definitions in `cco/src/server.rs`
- Data formatting in API handlers
- JSON validity with `jq`: `curl -s http://127.0.0.1:3000/api/agents | jq .`

**Common Causes**:
- Endpoints not implemented
- Data serialization issues
- Database/cache connectivity problems

---

### If Test 5 Fails (WebSocket Terminal)

**Check**:
- WebSocket endpoint enabled in server
- Correct upgrade headers
- Port 3000 accessible and not blocked

**Common Causes**:
- WebSocket endpoint not implemented
- Header validation failing
- Firewall blocking WebSocket upgrade

---

### If Test 6 Fails (SSE Stream)

**Check**:
- SSE endpoint at `/api/stream`
- Server supports persistent connections
- Event data format: `data: {json}\n\n`

**Common Causes**:
- SSE endpoint not implemented
- Stream not persisting beyond first event
- Data format incorrect

---

### If Test 7 Fails (Full Feature Flow)

**Check**:
- Individual endpoints (Test 4)
- Network latency: `ping 127.0.0.1`
- Server response times
- Dashboard JS loading correctly (Test 3)

**Common Causes**:
- Slow API responses
- Network connectivity issues
- Large response payloads

---

## QA Execution Checklist

Before declaring tests complete:

- [ ] All 8 tests executed
- [ ] Test output documented
- [ ] All tests passed
- [ ] No critical issues found
- [ ] Performance acceptable
- [ ] Error scenarios validated
- [ ] Test execution time recorded
- [ ] System logs reviewed for warnings
- [ ] Report generated and reviewed
- [ ] Production readiness determined

---

## Sign-Off

**Test Execution Date**: _______________

**Executed By**: _______________

**Status**:

- [ ] READY FOR PRODUCTION
- [ ] NEEDS FIXES
- [ ] BLOCKED - Requires Investigation

**Critical Issues**:
```
[List any critical issues found]
```

**Notes**:
```
[Additional observations or recommendations]
```

---

## Appendix: Manual Test Procedures

If automated testing is unavailable, follow these manual steps:

### Test 1: Manual Dashboard Load
```bash
curl -s http://127.0.0.1:3000/ | head -50
# Verify HTTP 200 status and HTML presence
```

### Test 4: Manual API Check
```bash
# Check each endpoint
echo "=== /api/agents ==="
curl -s http://127.0.0.1:3000/api/agents | jq .

echo "=== /api/metrics/projects ==="
curl -s http://127.0.0.1:3000/api/metrics/projects | jq .

echo "=== /api/stats ==="
curl -s http://127.0.0.1:3000/api/stats | jq .
```

### Test 7: Manual Performance Check
```bash
# Time the complete flow
time {
  curl -s http://127.0.0.1:3000/ > /dev/null
  curl -s http://127.0.0.1:3000/api/stats > /dev/null
  curl -s http://127.0.0.1:3000/api/agents > /dev/null
  curl -s http://127.0.0.1:3000/api/metrics/projects > /dev/null
}
```

---

**Document Version**: 1.0
**Last Updated**: November 17, 2025
**Status**: Active - Ready for QA Execution
