#!/bin/bash

# Run terminal tests one at a time to avoid PTY resource conflicts

TESTS=(
  "test_terminal_session_spawn"
  "test_terminal_session_clone"
  "test_shell_detection"
  "test_multiple_concurrent_sessions"
  "test_terminal_write_echo_input"
  "test_terminal_write_multiple"
  "test_terminal_read_non_blocking"
  "test_terminal_read_partial_buffer"
  "test_terminal_large_input"
  "test_terminal_write_control_chars"
  "test_terminal_resize"
  "test_terminal_is_running"
  "test_terminal_rapid_status_checks"
  "test_terminal_close_idempotent"
  "test_terminal_concurrent_read_write"
  "test_terminal_lock_contention"
  "test_terminal_cleanup_after_intensive_ops"
  "test_terminal_write_to_closed_session"
  "test_terminal_empty_write"
  "test_terminal_rapid_open_close"
  "test_spawn_latency"
  "test_write_latency"
  "test_read_latency"
  "test_close_latency"
)

PASSED=0
FAILED=0
SKIPPED=0

echo "Running Terminal Integration Tests..."
echo "====================================="

for test in "${TESTS[@]}"; do
  echo -n "Running $test ... "

  if /Users/brent/.cargo/bin/cargo test --test terminal_fast "$test" -- --nocapture 2>&1 | grep -q "test result: ok"; then
    echo "✓ PASS"
    ((PASSED++))
  else
    echo "✗ FAIL"
    ((FAILED++))
  fi

  # Small delay to let PTY resources clean up
  sleep 0.5
done

echo ""
echo "====================================="
echo "Test Results:"
echo "  Passed:  $PASSED"
echo "  Failed:  $FAILED"
echo "  Total:   $((PASSED + FAILED))"
echo ""

if [ $FAILED -eq 0 ]; then
  echo "✓ All tests passed!"
  exit 0
else
  echo "✗ Some tests failed"
  exit 1
fi
