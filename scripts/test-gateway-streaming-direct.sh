#!/bin/bash
# Test CCO Gateway streaming directly
# This tests if the gateway correctly proxies streaming responses

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}CCO Gateway Streaming Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Get gateway port from daemon.pid
if [ ! -f ~/.cco/daemon.pid ]; then
    echo -e "${RED}Error: daemon.pid not found. Is the daemon running?${NC}"
    echo "Start it with: cco daemon start"
    exit 1
fi

GATEWAY_PORT=$(jq -r '.gateway_port' ~/.cco/daemon.pid 2>/dev/null)
DAEMON_PORT=$(jq -r '.port' ~/.cco/daemon.pid 2>/dev/null)

if [ -z "$GATEWAY_PORT" ] || [ "$GATEWAY_PORT" == "null" ]; then
    echo -e "${RED}Error: gateway_port not found in daemon.pid${NC}"
    exit 1
fi

echo -e "${YELLOW}Gateway port: $GATEWAY_PORT${NC}"
echo -e "${YELLOW}Daemon port: $DAEMON_PORT${NC}"
echo ""

# Check gateway health
echo -e "${YELLOW}Step 1: Checking gateway health...${NC}"
HEALTH=$(curl -s "http://127.0.0.1:$GATEWAY_PORT/gateway/health" | jq -r '.status')
if [ "$HEALTH" == "healthy" ] || [ "$HEALTH" == "degraded" ]; then
    echo -e "${GREEN}✓ Gateway is $HEALTH${NC}"
else
    echo -e "${RED}✗ Gateway health check failed${NC}"
    curl -s "http://127.0.0.1:$GATEWAY_PORT/gateway/health" | jq .
    exit 1
fi
echo ""

# Check for OAuth token
if [ -z "${CLAUDE_CODE_OAUTH_TOKEN:-}" ]; then
    echo -e "${YELLOW}Note: CLAUDE_CODE_OAUTH_TOKEN not set${NC}"
    echo "Will use any available auth from the gateway's default provider"
    AUTH_HEADER=""
else
    echo -e "${GREEN}✓ OAuth token found${NC}"
    AUTH_HEADER="-H \"Authorization: Bearer $CLAUDE_CODE_OAUTH_TOKEN\""
fi
echo ""

# Test streaming request
echo -e "${YELLOW}Step 2: Testing streaming request...${NC}"
echo "Sending request to: http://127.0.0.1:$GATEWAY_PORT/v1/messages"
echo ""

# Create temp file for output
TEMP_OUTPUT=$(mktemp)
TEMP_RAW=$(mktemp)

# Make streaming request - collect both raw output and parsed events
echo -e "${BLUE}Streaming response:${NC}"
echo "---"

# Use curl with verbose output to see connection details
if [ -n "${CLAUDE_CODE_OAUTH_TOKEN:-}" ]; then
    curl -sN \
        -X POST "http://127.0.0.1:$GATEWAY_PORT/v1/messages" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $CLAUDE_CODE_OAUTH_TOKEN" \
        -H "anthropic-beta: interleaved-thinking-2025-05-14" \
        -d '{
            "model": "claude-sonnet-4-5-20250929",
            "max_tokens": 100,
            "stream": true,
            "messages": [{"role": "user", "content": "Say hello in exactly 5 words."}]
        }' \
        2>/dev/null | tee "$TEMP_RAW" | while IFS= read -r line; do
            echo "$line"
            # Extract text deltas from content_block_delta events
            if [[ "$line" == data:* ]]; then
                DATA="${line#data: }"
                TEXT=$(echo "$DATA" | jq -r '.delta.text // empty' 2>/dev/null)
                if [ -n "$TEXT" ]; then
                    echo -n "$TEXT" >> "$TEMP_OUTPUT"
                fi
            fi
        done
else
    curl -sN \
        -X POST "http://127.0.0.1:$GATEWAY_PORT/v1/messages" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-sonnet-4-5-20250929",
            "max_tokens": 100,
            "stream": true,
            "messages": [{"role": "user", "content": "Say hello in exactly 5 words."}]
        }' \
        2>/dev/null | tee "$TEMP_RAW"
fi

echo ""
echo "---"
echo ""

# Check if we got any response
if [ -s "$TEMP_RAW" ]; then
    echo -e "${GREEN}✓ Received streaming response${NC}"

    # Count events
    EVENT_COUNT=$(grep -c "^event:" "$TEMP_RAW" 2>/dev/null || echo 0)
    DATA_COUNT=$(grep -c "^data:" "$TEMP_RAW" 2>/dev/null || echo 0)
    DONE_EVENT=$(grep -c "\[DONE\]" "$TEMP_RAW" 2>/dev/null || echo 0)

    echo "  Events received: $EVENT_COUNT"
    echo "  Data lines: $DATA_COUNT"
    echo "  Received [DONE]: $DONE_EVENT"

    if [ "$DONE_EVENT" -gt 0 ]; then
        echo -e "${GREEN}✓ Stream completed normally ([DONE] received)${NC}"
    else
        echo -e "${YELLOW}⚠ Stream may have been truncated (no [DONE] received)${NC}"
    fi

    # Show extracted text
    if [ -s "$TEMP_OUTPUT" ]; then
        echo ""
        echo -e "${BLUE}Extracted text content:${NC}"
        echo "---"
        cat "$TEMP_OUTPUT"
        echo ""
        echo "---"
    fi
else
    echo -e "${RED}✗ No response received${NC}"
    echo "Check gateway logs at: ~/.cco/logs/daemon.log"
fi

# Cleanup
rm -f "$TEMP_OUTPUT" "$TEMP_RAW"

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Complete${NC}"
echo -e "${BLUE}========================================${NC}"
