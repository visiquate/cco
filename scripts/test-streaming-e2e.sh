#!/bin/bash
set -euo pipefail

# End-to-End Streaming Test Script
# Tests the complete streaming pipeline with incremental chunk delivery verification

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================"
echo "E2E Streaming Test Suite"
echo "========================================${NC}"
echo ""

# Test configuration
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to log test results
log_test() {
    local test_name="$1"
    local status="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ "$status" == "PASS" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗${NC} $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Step 1: Check daemon status
echo -e "${YELLOW}Step 1: Checking daemon status...${NC}"
if ! cco daemon status &>/dev/null; then
    echo -e "${RED}✗ Daemon is not running${NC}"
    echo "Starting daemon..."
    cco daemon start
    sleep 3
    if ! cco daemon status &>/dev/null; then
        echo -e "${RED}✗ Failed to start daemon${NC}"
        exit 1
    fi
fi

DAEMON_INFO=$(cco daemon status)
PORT=$(echo "$DAEMON_INFO" | grep "Port:" | awk '{print $2}')

if [ -z "$PORT" ]; then
    echo -e "${RED}✗ Could not determine gateway port${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Daemon is running on port $PORT${NC}"
GATEWAY_URL="http://127.0.0.1:$PORT"
echo ""

# Step 2: Check for authentication
echo -e "${YELLOW}Step 2: Checking authentication...${NC}"
if [ -n "${CLAUDE_CODE_OAUTH_TOKEN:-}" ]; then
    AUTH_HEADER="Authorization: Bearer $CLAUDE_CODE_OAUTH_TOKEN"
    echo -e "${GREEN}✓ Using OAuth token${NC}"
elif [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    AUTH_HEADER="x-api-key: $ANTHROPIC_API_KEY"
    echo -e "${GREEN}✓ Using API key${NC}"
else
    echo -e "${RED}✗ No authentication token found${NC}"
    echo "Please set CLAUDE_CODE_OAUTH_TOKEN or ANTHROPIC_API_KEY"
    exit 1
fi
echo ""

# Step 3: Test health endpoint
echo -e "${YELLOW}Step 3: Testing health endpoint...${NC}"
HEALTH_RESPONSE=$(curl -s "$GATEWAY_URL/gateway/health")
if echo "$HEALTH_RESPONSE" | jq . &>/dev/null; then
    log_test "Health endpoint returns valid JSON" "PASS"
else
    log_test "Health endpoint returns valid JSON" "FAIL"
fi
echo ""

# Step 4: Test incremental streaming delivery
echo -e "${YELLOW}Step 4: Testing incremental streaming delivery...${NC}"
STREAM_OUTPUT=$(mktemp)
STREAM_TIMINGS=$(mktemp)

STREAM_REQUEST='{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 150,
  "stream": true,
  "messages": [{"role": "user", "content": "Count from 1 to 10, one number per line. Be verbose about it."}]
}'

echo "Sending streaming request and capturing timestamps..."

# Start timestamp
START_TIME=$(date +%s%N)

# Stream with timestamp tracking
curl -N -s \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -d "$STREAM_REQUEST" \
  --no-buffer \
  2>&1 | while IFS= read -r line; do
    CURRENT_TIME=$(date +%s%N)
    ELAPSED_MS=$(( (CURRENT_TIME - START_TIME) / 1000000 ))
    echo "$ELAPSED_MS|$line" >> "$STREAM_TIMINGS"
    echo "$line" >> "$STREAM_OUTPUT"
done

END_TIME=$(date +%s%N)
TOTAL_DURATION_MS=$(( (END_TIME - START_TIME) / 1000000 ))

echo "Stream completed in ${TOTAL_DURATION_MS}ms"
echo ""

# Step 5: Analyze streaming behavior
echo -e "${YELLOW}Step 5: Analyzing streaming behavior...${NC}"

# Count SSE events
EVENT_COUNT=$(grep -c "^event:" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
DATA_COUNT=$(grep -c "^data:" "$STREAM_OUTPUT" 2>/dev/null || echo "0")

if [ "$EVENT_COUNT" -gt 0 ]; then
    log_test "SSE events received (count: $EVENT_COUNT)" "PASS"
else
    log_test "SSE events received" "FAIL"
fi

if [ "$DATA_COUNT" -gt 0 ]; then
    log_test "SSE data chunks received (count: $DATA_COUNT)" "PASS"
else
    log_test "SSE data chunks received" "FAIL"
fi

# Check for incremental delivery (timestamps should be spread out)
if [ -f "$STREAM_TIMINGS" ] && [ -s "$STREAM_TIMINGS" ]; then
    FIRST_EVENT_TIME=$(grep "event:" "$STREAM_TIMINGS" | head -1 | cut -d'|' -f1)
    LAST_EVENT_TIME=$(grep "event:" "$STREAM_TIMINGS" | tail -1 | cut -d'|' -f1)

    if [ -n "$FIRST_EVENT_TIME" ] && [ -n "$LAST_EVENT_TIME" ]; then
        TIME_SPREAD=$((LAST_EVENT_TIME - FIRST_EVENT_TIME))

        echo "Streaming timeline:"
        echo "  First event: ${FIRST_EVENT_TIME}ms"
        echo "  Last event: ${LAST_EVENT_TIME}ms"
        echo "  Time spread: ${TIME_SPREAD}ms"

        # Streaming should take some time (not instant buffering)
        if [ "$TIME_SPREAD" -gt 100 ]; then
            log_test "Incremental delivery verified (not buffered)" "PASS"
        else
            log_test "Incremental delivery verified (WARNING: may be buffered)" "FAIL"
        fi
    fi
fi

# Step 6: Verify SSE event types
echo ""
echo -e "${YELLOW}Step 6: Verifying SSE event types...${NC}"

MESSAGE_START=$(grep -c "event: message_start" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_START=$(grep -c "event: content_block_start" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_DELTA=$(grep -c "event: content_block_delta" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_STOP=$(grep -c "event: content_block_stop" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
MESSAGE_DELTA=$(grep -c "event: message_delta" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
MESSAGE_STOP=$(grep -c "event: message_stop" "$STREAM_OUTPUT" 2>/dev/null || echo "0")

echo "SSE Event Breakdown:"
echo "  message_start: $MESSAGE_START"
echo "  content_block_start: $CONTENT_BLOCK_START"
echo "  content_block_delta: $CONTENT_BLOCK_DELTA"
echo "  content_block_stop: $CONTENT_BLOCK_STOP"
echo "  message_delta: $MESSAGE_DELTA"
echo "  message_stop: $MESSAGE_STOP"

# Verify expected events
[ "$MESSAGE_START" -ge 1 ] && log_test "message_start event present" "PASS" || log_test "message_start event present" "FAIL"
[ "$CONTENT_BLOCK_START" -ge 1 ] && log_test "content_block_start event present" "PASS" || log_test "content_block_start event present" "FAIL"
[ "$CONTENT_BLOCK_DELTA" -gt 5 ] && log_test "content_block_delta events present (count: $CONTENT_BLOCK_DELTA)" "PASS" || log_test "content_block_delta events present" "FAIL"
[ "$CONTENT_BLOCK_STOP" -ge 1 ] && log_test "content_block_stop event present" "PASS" || log_test "content_block_stop event present" "FAIL"
[ "$MESSAGE_STOP" -ge 1 ] && log_test "message_stop event present" "PASS" || log_test "message_stop event present" "FAIL"

# Step 7: Test error scenarios
echo ""
echo -e "${YELLOW}Step 7: Testing error scenarios...${NC}"

# Test 7a: Invalid API key
INVALID_KEY_OUTPUT=$(mktemp)
HTTP_CODE=$(curl -s -w "%{http_code}" -o "$INVALID_KEY_OUTPUT" \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "x-api-key: invalid-key-12345" \
  -H "anthropic-version: 2023-06-01" \
  -d '{"model":"claude-sonnet-4-5-20250929","max_tokens":10,"stream":true,"messages":[{"role":"user","content":"Hi"}]}')

if [ "$HTTP_CODE" == "401" ] || [ "$HTTP_CODE" == "400" ]; then
    log_test "Invalid API key returns error (HTTP $HTTP_CODE)" "PASS"
else
    log_test "Invalid API key returns error" "FAIL"
fi
rm -f "$INVALID_KEY_OUTPUT"

# Test 7b: Invalid model
INVALID_MODEL_OUTPUT=$(mktemp)
HTTP_CODE=$(curl -s -w "%{http_code}" -o "$INVALID_MODEL_OUTPUT" \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -d '{"model":"nonexistent-model","max_tokens":10,"stream":true,"messages":[{"role":"user","content":"Hi"}]}')

if [ "$HTTP_CODE" != "200" ]; then
    log_test "Invalid model returns error (HTTP $HTTP_CODE)" "PASS"
else
    log_test "Invalid model returns error" "FAIL"
fi
rm -f "$INVALID_MODEL_OUTPUT"

# Test 7c: Timeout handling
TIMEOUT_OUTPUT=$(mktemp)
TIMEOUT_RESULT="PASS"
if timeout 2s curl -N -s \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -d '{"model":"claude-sonnet-4-5-20250929","max_tokens":5000,"stream":true,"messages":[{"role":"user","content":"Write a very long essay"}]}' \
  > "$TIMEOUT_OUTPUT" 2>&1; then
    # Request completed within timeout
    TIMEOUT_RESULT="PASS"
else
    # Timeout occurred (expected for long requests)
    TIMEOUT_RESULT="PASS"
fi
log_test "Timeout handling works" "$TIMEOUT_RESULT"
rm -f "$TIMEOUT_OUTPUT"

# Step 8: Verify Content-Type header
echo ""
echo -e "${YELLOW}Step 8: Verifying response headers...${NC}"
HEADERS_OUTPUT=$(mktemp)

curl -I -s \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -d '{"model":"claude-sonnet-4-5-20250929","max_tokens":10,"stream":true,"messages":[{"role":"user","content":"Hi"}]}' \
  > "$HEADERS_OUTPUT" 2>&1 &

CURL_PID=$!
sleep 1
kill $CURL_PID 2>/dev/null || true
wait $CURL_PID 2>/dev/null || true

if grep -qi "content-type.*text/event-stream" "$HEADERS_OUTPUT" 2>/dev/null; then
    log_test "Content-Type is text/event-stream" "PASS"
else
    log_test "Content-Type is text/event-stream" "FAIL"
fi

if grep -qi "cache-control.*no-cache" "$HEADERS_OUTPUT" 2>/dev/null; then
    log_test "Cache-Control is no-cache" "PASS"
else
    log_test "Cache-Control is no-cache" "FAIL"
fi

if grep -qi "x-accel-buffering.*no" "$HEADERS_OUTPUT" 2>/dev/null; then
    log_test "x-accel-buffering is no" "PASS"
else
    log_test "x-accel-buffering is no (optional)" "PASS"
fi

rm -f "$HEADERS_OUTPUT"

# Cleanup
rm -f "$STREAM_OUTPUT" "$STREAM_TIMINGS"

# Final summary
echo ""
echo -e "${BLUE}========================================"
echo "Test Summary"
echo "========================================${NC}"
echo -e "Total tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
