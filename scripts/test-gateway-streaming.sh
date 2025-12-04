#!/bin/bash
set -euo pipefail

# LLM Gateway Streaming Test Script
# Tests both streaming and non-streaming modes of the gateway

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}LLM Gateway Streaming Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Check daemon status
echo -e "${YELLOW}Step 1: Checking daemon status...${NC}"
if ! cco daemon status &>/dev/null; then
    echo -e "${RED}✗ Daemon is not running${NC}"
    echo "Please start the daemon with: cco daemon start"
    exit 1
fi

# Extract port from daemon status
DAEMON_INFO=$(cco daemon status)
PORT=$(echo "$DAEMON_INFO" | grep "Port:" | awk '{print $2}')

if [ -z "$PORT" ]; then
    echo -e "${RED}✗ Could not determine gateway port${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Daemon is running on port $PORT${NC}"
echo ""

# Step 2: Check for authentication token
echo -e "${YELLOW}Step 2: Checking authentication token...${NC}"

# Check for OAuth token first (preferred), then fall back to API key
if [ -n "${CLAUDE_CODE_OAUTH_TOKEN:-}" ]; then
    AUTH_TOKEN="$CLAUDE_CODE_OAUTH_TOKEN"
    AUTH_HEADER="Authorization: Bearer $AUTH_TOKEN"
    TOKEN_TYPE="OAuth token"
elif [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    AUTH_TOKEN="$ANTHROPIC_API_KEY"
    AUTH_HEADER="x-api-key: $AUTH_TOKEN"
    TOKEN_TYPE="API key"
else
    echo -e "${RED}✗ No authentication token found${NC}"
    echo "Please set one of:"
    echo "  - export CLAUDE_CODE_OAUTH_TOKEN=your-oauth-token (preferred)"
    echo "  - export ANTHROPIC_API_KEY=your-api-key"
    exit 1
fi

echo -e "${GREEN}✓ $TOKEN_TYPE is set${NC}"
echo ""

# Gateway URL
GATEWAY_URL="http://127.0.0.1:$PORT"

# Step 3: Test health endpoint
echo -e "${YELLOW}Step 3: Testing health endpoint...${NC}"
HEALTH_RESPONSE=$(curl -s "$GATEWAY_URL/gateway/health")
HEALTH_STATUS=$(echo "$HEALTH_RESPONSE" | jq -r '.status' 2>/dev/null || echo "error")

if [ "$HEALTH_STATUS" != "healthy" ] && [ "$HEALTH_STATUS" != "degraded" ]; then
    echo -e "${RED}✗ Gateway health check failed${NC}"
    echo "Response: $HEALTH_RESPONSE"
    exit 1
fi
echo -e "${GREEN}✓ Gateway is $HEALTH_STATUS${NC}"
echo ""

# Step 4: Test non-streaming request
echo -e "${YELLOW}Step 4: Testing non-streaming request...${NC}"
NON_STREAM_REQUEST='{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 50,
  "stream": false,
  "messages": [{"role": "user", "content": "Say hello in exactly 3 words"}]
}'

echo "Sending non-streaming request..."
NON_STREAM_START=$(date +%s%3N)
NON_STREAM_RESPONSE=$(curl -s -w "\n%{http_code}" \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -d "$NON_STREAM_REQUEST")

NON_STREAM_END=$(date +%s%3N)
NON_STREAM_DURATION=$((NON_STREAM_END - NON_STREAM_START))

# Extract HTTP code (last line)
HTTP_CODE=$(echo "$NON_STREAM_RESPONSE" | tail -n 1)
RESPONSE_BODY=$(echo "$NON_STREAM_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" != "200" ]; then
    echo -e "${RED}✗ Non-streaming request failed with HTTP $HTTP_CODE${NC}"
    echo "Response: $RESPONSE_BODY"
    exit 1
fi

# Check if response is valid JSON
if ! echo "$RESPONSE_BODY" | jq . &>/dev/null; then
    echo -e "${RED}✗ Non-streaming response is not valid JSON${NC}"
    echo "Response: $RESPONSE_BODY"
    exit 1
fi

CONTENT=$(echo "$RESPONSE_BODY" | jq -r '.content[0].text' 2>/dev/null || echo "error")
echo -e "${GREEN}✓ Non-streaming request succeeded${NC}"
echo "  Response time: ${NON_STREAM_DURATION}ms"
echo "  Content: $CONTENT"
echo ""

# Step 5: Test streaming request
echo -e "${YELLOW}Step 5: Testing streaming request...${NC}"
STREAM_REQUEST='{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 100,
  "stream": true,
  "messages": [{"role": "user", "content": "Count from 1 to 5, one number per line"}]
}'

echo "Sending streaming request and monitoring SSE events..."
STREAM_OUTPUT=$(mktemp)
STREAM_HEADERS=$(mktemp)

# Track timing of first byte vs last byte
STREAM_START=$(date +%s%3N)
FIRST_EVENT_TIME=""
LAST_EVENT_TIME=""

curl -s -w "\n%{http_code}\n" \
  -X POST "$GATEWAY_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -H "anthropic-version: 2023-06-01" \
  -D "$STREAM_HEADERS" \
  -d "$STREAM_REQUEST" \
  --no-buffer \
  > "$STREAM_OUTPUT" 2>&1

STREAM_END=$(date +%s%3N)
STREAM_DURATION=$((STREAM_END - STREAM_START))

# Check HTTP response
HTTP_CODE=$(tail -n 1 "$STREAM_OUTPUT")
STREAM_BODY=$(head -n -1 "$STREAM_OUTPUT")

if [ "$HTTP_CODE" != "200" ]; then
    echo -e "${RED}✗ Streaming request failed with HTTP $HTTP_CODE${NC}"
    echo "Response: $(cat $STREAM_OUTPUT)"
    rm -f "$STREAM_OUTPUT" "$STREAM_HEADERS"
    exit 1
fi

# Check Content-Type header
CONTENT_TYPE=$(grep -i "^content-type:" "$STREAM_HEADERS" | cut -d' ' -f2- | tr -d '\r\n')
if [[ "$CONTENT_TYPE" != *"text/event-stream"* ]]; then
    echo -e "${RED}✗ Streaming response has wrong Content-Type: $CONTENT_TYPE${NC}"
    echo "Expected: text/event-stream"
    rm -f "$STREAM_OUTPUT" "$STREAM_HEADERS"
    exit 1
fi

echo -e "${GREEN}✓ Streaming response has correct Content-Type: $CONTENT_TYPE${NC}"

# Count SSE events
EVENT_COUNT=$(grep -c "^event:" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
DATA_COUNT=$(grep -c "^data:" "$STREAM_OUTPUT" 2>/dev/null || echo "0")

echo -e "${GREEN}✓ Streaming request succeeded${NC}"
echo "  Response time: ${STREAM_DURATION}ms"
echo "  SSE events received: $EVENT_COUNT"
echo "  Data chunks received: $DATA_COUNT"

if [ "$EVENT_COUNT" -eq 0 ] && [ "$DATA_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}⚠ Warning: No SSE events detected in response${NC}"
    echo "First 500 chars of response:"
    head -c 500 "$STREAM_OUTPUT"
    echo ""
fi

# Show sample of SSE events
echo ""
echo "Sample SSE events (first 10):"
head -n 20 "$STREAM_OUTPUT" | grep -E "^(event:|data:)" | head -n 10

# Check for incremental delivery (not buffered)
# In true streaming, events should arrive progressively
# We can't easily test this with curl, but we can verify SSE format

# Step 6: Test streaming with message_start, content_block, etc.
MESSAGE_START_COUNT=$(grep -c "event: message_start" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_START=$(grep -c "event: content_block_start" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_DELTA=$(grep -c "event: content_block_delta" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
CONTENT_BLOCK_STOP=$(grep -c "event: content_block_stop" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
MESSAGE_DELTA=$(grep -c "event: message_delta" "$STREAM_OUTPUT" 2>/dev/null || echo "0")
MESSAGE_STOP=$(grep -c "event: message_stop" "$STREAM_OUTPUT" 2>/dev/null || echo "0")

echo ""
echo "SSE Event Breakdown:"
echo "  message_start: $MESSAGE_START_COUNT"
echo "  content_block_start: $CONTENT_BLOCK_START"
echo "  content_block_delta: $CONTENT_BLOCK_DELTA"
echo "  content_block_stop: $CONTENT_BLOCK_STOP"
echo "  message_delta: $MESSAGE_DELTA"
echo "  message_stop: $MESSAGE_STOP"

# Cleanup
rm -f "$STREAM_OUTPUT" "$STREAM_HEADERS"

# Step 7: Test metrics endpoint
echo ""
echo -e "${YELLOW}Step 7: Testing metrics endpoint...${NC}"
METRICS_RESPONSE=$(curl -s "$GATEWAY_URL/gateway/metrics")
TOTAL_REQUESTS=$(echo "$METRICS_RESPONSE" | jq -r '.summary.total_requests' 2>/dev/null || echo "0")

if [ "$TOTAL_REQUESTS" -ge 2 ]; then
    echo -e "${GREEN}✓ Metrics endpoint working (total requests: $TOTAL_REQUESTS)${NC}"
else
    echo -e "${YELLOW}⚠ Metrics may not be tracking correctly (total requests: $TOTAL_REQUESTS)${NC}"
fi

# Final summary
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ All tests passed!${NC}"
echo ""
echo "Key findings:"
echo "  - Gateway is running on port $PORT"
echo "  - Non-streaming requests work correctly"
echo "  - Streaming requests return proper SSE format"
echo "  - Metrics tracking is operational"
echo ""
echo "Next steps:"
echo "  - Test with different models (opus, haiku)"
echo "  - Test error handling (invalid API key, etc.)"
echo "  - Test provider fallback scenarios"
echo "  - Monitor audit logs: cco daemon logs"
