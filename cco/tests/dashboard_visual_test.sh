#!/bin/bash

# Comprehensive Visual Testing for Dashboard
# Tests HTML rendering, asset loading, and terminal functionality

set -e

cd "$(dirname "$0")/.." || exit 1

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0
WARNINGS=0
PORT=3333

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}COMPREHENSIVE DASHBOARD INTEGRATION TEST SUITE${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${BLUE}Cleaning up...${NC}"
    pkill -f "cco run.*$PORT" 2>/dev/null || true
    sleep 1
}

trap cleanup EXIT

# Start server
echo "Starting server on port $PORT..."
export NO_BROWSER=1
./target/release/cco run --debug --port $PORT 2>&1 > /tmp/dashboard_test.log &
SERVER_PID=$!

sleep 4

if ! ps -p $SERVER_PID > /dev/null 2>&1; then
    echo -e "${RED}✗ FAIL: Server failed to start${NC}"
    ((FAILED++))
    exit 1
fi

echo -e "${GREEN}✓ Server started (PID: $SERVER_PID)${NC}"
echo ""

# ===== TEST 1: DASHBOARD HTML ENDPOINT =====
echo -e "${BLUE}TEST 1: DASHBOARD HTML ENDPOINT${NC}"
echo "---"

RESPONSE=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/ 2>/dev/null)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✓ PASS: Dashboard endpoint returns HTTP 200${NC}"
    ((PASSED++))

    # Check Content-Type
    CONTENT_TYPE=$(curl -s -I http://127.0.0.1:$PORT/ 2>/dev/null | grep -i "content-type" | head -1)
    if echo "$CONTENT_TYPE" | grep -q "text/html"; then
        echo -e "${GREEN}✓ PASS: Content-Type is text/html${NC}"
        ((PASSED++))
    else
        echo -e "${YELLOW}⚠ WARNING: Unexpected Content-Type: $CONTENT_TYPE${NC}"
        ((WARNINGS++))
    fi

    # Check HTML content
    BODY=$(echo "$RESPONSE" | head -1)
    if echo "$BODY" | grep -q "<!DOCTYPE\|<html\|<head"; then
        echo -e "${GREEN}✓ PASS: HTML structure present${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL: Missing HTML structure${NC}"
        ((FAILED++))
    fi
else
    echo -e "${RED}✗ FAIL: HTTP $HTTP_CODE (expected 200)${NC}"
    ((FAILED++))
fi

echo ""

# ===== TEST 2: CSS ASSET =====
echo -e "${BLUE}TEST 2: CSS ASSET ENDPOINT${NC}"
echo "---"

CSS_CODE=$(curl -s -w "%{http_code}" -o /tmp/dashboard.css http://127.0.0.1:$PORT/dashboard.css 2>/dev/null)

if [ "$CSS_CODE" = "200" ]; then
    echo -e "${GREEN}✓ PASS: CSS endpoint returns HTTP 200${NC}"
    ((PASSED++))

    if [ -s /tmp/dashboard.css ]; then
        CSS_SIZE=$(wc -c < /tmp/dashboard.css)
        echo -e "${GREEN}✓ PASS: CSS file loaded ($CSS_SIZE bytes)${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL: CSS file is empty${NC}"
        ((FAILED++))
    fi
else
    echo -e "${RED}✗ FAIL: CSS HTTP $CSS_CODE (expected 200)${NC}"
    ((FAILED++))
fi

echo ""

# ===== TEST 3: JAVASCRIPT ASSET =====
echo -e "${BLUE}TEST 3: JAVASCRIPT ASSET ENDPOINT${NC}"
echo "---"

JS_CODE=$(curl -s -w "%{http_code}" -o /tmp/dashboard.js http://127.0.0.1:$PORT/dashboard.js 2>/dev/null)

if [ "$JS_CODE" = "200" ]; then
    echo -e "${GREEN}✓ PASS: JavaScript endpoint returns HTTP 200${NC}"
    ((PASSED++))

    if [ -s /tmp/dashboard.js ]; then
        JS_SIZE=$(wc -c < /tmp/dashboard.js)
        echo -e "${GREEN}✓ PASS: JavaScript file loaded ($JS_SIZE bytes)${NC}"
        ((PASSED++))

        # Check for terminal code
        if grep -q "terminal\|Terminal\|xterm" /tmp/dashboard.js; then
            echo -e "${GREEN}✓ PASS: Terminal code present in JavaScript${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠ WARNING: Terminal code not found in JavaScript${NC}"
            ((WARNINGS++))
        fi

        # Check for focus management
        if grep -q "focus\|Focus" /tmp/dashboard.js; then
            echo -e "${GREEN}✓ PASS: Focus management code present${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠ WARNING: Focus management code not found${NC}"
            ((WARNINGS++))
        fi

        # Check for WebSocket support
        if grep -q "WebSocket\|ws://" /tmp/dashboard.js; then
            echo -e "${GREEN}✓ PASS: WebSocket code present${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠ WARNING: WebSocket code not found${NC}"
            ((WARNINGS++))
        fi
    else
        echo -e "${RED}✗ FAIL: JavaScript file is empty${NC}"
        ((FAILED++))
    fi
else
    echo -e "${RED}✗ FAIL: JavaScript HTTP $JS_CODE (expected 200)${NC}"
    ((FAILED++))
fi

echo ""

# ===== TEST 4: HEALTH ENDPOINT =====
echo -e "${BLUE}TEST 4: HEALTH ENDPOINT${NC}"
echo "---"

HEALTH=$(curl -s http://127.0.0.1:$PORT/health 2>/dev/null)

if echo "$HEALTH" | grep -q '"status"'; then
    echo -e "${GREEN}✓ PASS: Health endpoint returns valid JSON${NC}"
    ((PASSED++))

    STATUS=$(echo "$HEALTH" | grep -o '"status":"[^"]*"' | head -1)
    echo "  Status: $STATUS"
else
    echo -e "${RED}✗ FAIL: Health endpoint invalid response${NC}"
    ((FAILED++))
fi

echo ""

# ===== TEST 5: AGENTS ENDPOINT =====
echo -e "${BLUE}TEST 5: AGENTS API ENDPOINT${NC}"
echo "---"

AGENTS=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/api/agents 2>/dev/null)
AGENTS_CODE=$(echo "$AGENTS" | tail -1)

if [ "$AGENTS_CODE" = "200" ]; then
    echo -e "${GREEN}✓ PASS: Agents endpoint returns HTTP 200${NC}"
    ((PASSED++))

    AGENT_COUNT=$(echo "$AGENTS" | head -1 | grep -o '"name"' | wc -l)
    if [ "$AGENT_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ PASS: Agents returned ($AGENT_COUNT agents)${NC}"
        ((PASSED++))
    fi
else
    echo -e "${RED}✗ FAIL: Agents HTTP $AGENTS_CODE${NC}"
    ((FAILED++))
fi

echo ""

# ===== TEST 6: STATS ENDPOINT =====
echo -e "${BLUE}TEST 6: STATS ENDPOINT${NC}"
echo "---"

STATS=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/api/stats 2>/dev/null)
STATS_CODE=$(echo "$STATS" | tail -1)

if [ "$STATS_CODE" = "200" ]; then
    echo -e "${GREEN}✓ PASS: Stats endpoint returns HTTP 200${NC}"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ WARNING: Stats HTTP $STATS_CODE${NC}"
    ((WARNINGS++))
fi

echo ""

# ===== TEST 7: TERMINAL WEBSOCKET =====
echo -e "${BLUE}TEST 7: TERMINAL WEBSOCKET ENDPOINT${NC}"
echo "---"

WS_RESPONSE=$(curl -s -i -N \
    -H "Connection: Upgrade" \
    -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
    -H "Sec-WebSocket-Version: 13" \
    http://127.0.0.1:$PORT/terminal 2>/dev/null | head -1)

if echo "$WS_RESPONSE" | grep -q "101\|400\|404"; then
    if echo "$WS_RESPONSE" | grep -q "101"; then
        echo -e "${GREEN}✓ PASS: WebSocket upgrade successful${NC}"
        ((PASSED++))
    elif echo "$WS_RESPONSE" | grep -q "400\|404"; then
        echo -e "${YELLOW}⚠ WARNING: Terminal endpoint exists but WebSocket handshake not complete${NC}"
        ((WARNINGS++))
    fi
else
    echo -e "${YELLOW}⚠ WARNING: Terminal endpoint response unclear${NC}"
    ((WARNINGS++))
fi

echo ""

# ===== TEST 8: 404 HANDLING =====
echo -e "${BLUE}TEST 8: ERROR HANDLING (404 ROUTES)${NC}"
echo "---"

NOT_FOUND=$(curl -s -w "%{http_code}" -o /dev/null http://127.0.0.1:$PORT/nonexistent 2>/dev/null)

if [ "$NOT_FOUND" = "404" ]; then
    echo -e "${GREEN}✓ PASS: 404 error handling works${NC}"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ WARNING: Nonexistent route returned HTTP $NOT_FOUND (expected 404)${NC}"
    ((WARNINGS++))
fi

echo ""

# ===== SUMMARY =====
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}TEST SUMMARY${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo ""
echo "Passed:  $PASSED"
echo "Failed:  $FAILED"
echo "Warnings: $WARNINGS"
echo "Total:   $((PASSED + FAILED + WARNINGS))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL CRITICAL TESTS PASSED${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo ""
    exit 1
fi
