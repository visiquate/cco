# Comprehensive Smoke Test - All Modes

This smoke test validates both modes of CCO operation to ensure critical functionality works correctly.

## Test Coverage

### Mode 1: Explicit Server Mode
Tests the explicit server startup with `cco run --debug --port XXXX`

**Tests:**
- Server starts successfully
- `/health` endpoint returns 200 + valid JSON
- `/api/agents` returns valid JSON array
- `/api/stats` returns valid JSON
- `/api/metrics/projects` returns valid JSON
- Server shuts down cleanly within 2 seconds
- Port is released after shutdown

**Port:** 3101

### Mode 2: TUI/Daemon Mode (Critical - Previously Missed)
Tests the TUI/daemon startup when running `cco` with no arguments

**Tests:**
- Process starts successfully
- Falls back to daemon mode on port 3000 (when TUI fails in headless environment)
- `/health` endpoint accessible at `http://127.0.0.1:3000/health`
- `/api/agents` endpoint accessible
- `/api/stats` endpoint accessible
- `/api/metrics/projects` endpoint accessible
- Daemon shuts down cleanly within 2 seconds
- Port is released after shutdown

**Port:** 3000

## Running the Smoke Test

### Quick Start
```bash
./tests/smoke_test_all_modes.sh
```

### Output
- Shows real-time progress for each test
- Color-coded results (green=pass, red=fail, blue=info)
- Summary at the end with per-mode breakdown
- Exit code 0 on success, 1 on failure

### Example Output
```
═══════════════════════════════════════
MODE 1: EXPLICIT SERVER (cco run --debug --port 3101)
═══════════════════════════════════════

[INFO] Starting cco run --debug --port 3101...
[INFO] Server PID: 12345
✓ PASS: Server started and health check passed (MODE 1)
[TEST] Testing /health endpoint on port 3101 (MODE 1)
✓ PASS: Health endpoint returned ok status (MODE 1)
[TEST] Testing /api/agents endpoint on port 3101 (MODE 1)
✓ PASS: Agents endpoint returns valid JSON array with 119 agents (MODE 1)
...

═══════════════════════════════════════
RESULTS SUMMARY
═══════════════════════════════════════

MODE 1 (Explicit Server):
  Passed: 6
  Failed: 0

MODE 2 (TUI/Daemon):
  Passed: 6
  Failed: 0

Overall:
  Total Tests: 12
  Total Passed: 12
  Total Failed: 0

✓ ALL SMOKE TESTS PASSED
```

## Integration with CI/CD

This script is designed for GitHub Actions and other CI/CD systems:

```yaml
- name: Run Smoke Tests
  run: ./cco/tests/smoke_test_all_modes.sh
```

**Exit Codes:**
- `0` - All tests passed, safe to deploy
- `1` - One or more tests failed, deployment should be blocked

## Key Features

1. **Comprehensive Port Cleanup**
   - Automatically kills any existing processes on test ports
   - Verifies port release after shutdown
   - Handles hung processes with force kill

2. **Robust Waiting**
   - Retries health checks up to 15 times (7.5 seconds)
   - Uses 0.5s intervals for responsive testing
   - Graceful degradation with timeouts

3. **Both Modes Validated**
   - MODE 1: Direct server control (explicit flags)
   - MODE 2: Default daemon behavior (addresses previous failure)

4. **Detailed Diagnostics**
   - Logs saved to `/tmp/cco_mode*.log`
   - Shows HTTP responses on failure
   - Process info (PID, port, timing)

5. **Fast Execution**
   - MODE 1: ~15 seconds
   - MODE 2: ~20 seconds
   - Total: ~35 seconds

## Testing Checklist

Before considering the system ready for deployment:

- [ ] Run smoke test: `./tests/smoke_test_all_modes.sh`
- [ ] All 12+ tests pass
- [ ] Both MODE 1 and MODE 2 show 0 failures
- [ ] No errors in `/tmp/cco_mode*.log`
- [ ] Exit code is 0
- [ ] Ports 3000 and 3101 are released after test completes

## Troubleshooting

### Port Still in Use
```bash
# Check what's using the port
lsof -i :3000
lsof -i :3101

# Force kill process
kill -9 <PID>
```

### Timeout Issues
- Increase `MAX_RETRIES` at the top of the script
- Check available system resources
- Review `/tmp/cco_mode*.log` for startup errors

### Mode 2 (TUI/Daemon) Failures
- Most common cause: TUI initialization taking longer than expected
- Try increasing timeout in `wait_for_health` calls
- Verify `cco` binary is built with daemon fallback support

## Files

- **Script:** `/Users/brent/git/cc-orchestra/cco/tests/smoke_test_all_modes.sh`
- **This Doc:** `/Users/brent/git/cc-orchestra/cco/tests/SMOKE_TEST_README.md`
- **Logs:** `/tmp/cco_mode1.log` and `/tmp/cco_mode2.log`

## Related Tests

- E2E Acceptance Tests: `./tests/e2e_acceptance_tests.sh`
- Dashboard Tests: `./tests/dashboard_visual_test.sh`
