#!/bin/bash

###############################################################################
# Bearer Token Authentication Test Suite
# Tests all three authentication methods for coder.visiquate.com
###############################################################################

set -e

echo "=================================================="
echo "Bearer Token Authentication Test Suite"
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
FAILED_TESTS=0
PASSED_TESTS=0

# Helper function for test output
test_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED_TESTS++))
}

test_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED_TESTS++))
}

test_info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

###############################################################################
# Test 1: Server Connectivity (No Auth Required)
###############################################################################
echo "Test 1: Server Connectivity"
echo "----------------------------"
if curl -s https://coder.visiquate.com/api/tags > /dev/null 2>&1; then
    test_pass "Server is reachable at https://coder.visiquate.com"
else
    test_fail "Server is not reachable"
    exit 1
fi
echo ""

###############################################################################
# Test 2: Available Models
###############################################################################
echo "Test 2: Available Models"
echo "------------------------"
MODELS=$(curl -s https://coder.visiquate.com/api/tags | jq -r '.models[] | select(.name | contains("qwen")) | .name' 2>/dev/null | head -3)
if [ -n "$MODELS" ]; then
    test_pass "Models available:"
    echo "$MODELS" | sed 's/^/  - /'
else
    test_fail "No models found"
fi
echo ""

###############################################################################
# Test 3: Direct API Call with Bearer Token
###############################################################################
echo "Test 3: Direct API Call with Bearer Token"
echo "------------------------------------------"
RESPONSE=$(curl -s -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"model": "qwen-fast:latest", "prompt": "Say hello", "stream": false}')

if echo "$RESPONSE" | jq -e '.response' > /dev/null 2>&1; then
    test_pass "Direct API call with bearer token successful"
    RESPONSE_TEXT=$(echo "$RESPONSE" | jq -r '.response' | head -c 50)
    test_info "Response preview: $RESPONSE_TEXT..."
else
    test_fail "Direct API call failed"
    echo "$RESPONSE" | jq . 2>/dev/null || echo "$RESPONSE"
fi
echo ""

###############################################################################
# Test 4: Environment Variable Method
###############################################################################
echo "Test 4: Environment Variable Method"
echo "------------------------------------"
export CODER_LLM_TOKEN="$TOKEN"

# Unset credential manager encryption key to ensure env var is used
unset CREDENTIAL_ENCRYPTION_KEY

ROUTER_RESPONSE=$(node src/llm-router.js call-coding-llm "Write 2+2" 2>&1)

if echo "$ROUTER_RESPONSE" | grep -q '"text":'; then
    test_pass "Environment variable authentication works"
    # Extract just the text field
    ROUTER_TEXT=$(echo "$ROUTER_RESPONSE" | jq -r '.text' 2>/dev/null | head -c 50)
    test_info "Response preview: $ROUTER_TEXT..."
else
    test_fail "Environment variable authentication failed"
    echo "$ROUTER_RESPONSE"
fi
echo ""

###############################################################################
# Test 5: Routing Decision
###############################################################################
echo "Test 5: Routing Decision for Coding Agents"
echo "-------------------------------------------"
ROUTING=$(node src/llm-router.js route python-expert implement 2>&1)

if echo "$ROUTING" | grep -q "custom"; then
    test_pass "Coding tasks route to custom endpoint"
    ENDPOINT=$(echo "$ROUTING" | jq -r '.url' 2>/dev/null)
    test_info "Routes to: $ENDPOINT"
else
    test_fail "Routing decision incorrect"
    echo "$ROUTING"
fi
echo ""

###############################################################################
# Test 6: Router Configuration
###############################################################################
echo "Test 6: Router Configuration"
echo "----------------------------"
STATS=$(node src/llm-router.js stats 2>&1)

if echo "$STATS" | grep -q "https://coder.visiquate.com"; then
    test_pass "Router configuration is correct"
    ENABLED=$(echo "$STATS" | jq -r '.endpoints[0].enabled' 2>/dev/null)
    test_info "Custom endpoint enabled: $ENABLED"
else
    test_fail "Router configuration incorrect"
    echo "$STATS"
fi
echo ""

###############################################################################
# Test 7: Both Models Work
###############################################################################
echo "Test 7: Both Models (qwen-fast and qwen-quality-128k)"
echo "------------------------------------------------------"

# Test qwen-fast
test_info "Testing qwen-fast (7B model)..."
FAST_RESPONSE=$(curl -s -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"model": "qwen-fast:latest", "prompt": "Say hi", "stream": false}')

if echo "$FAST_RESPONSE" | jq -e '.response' > /dev/null 2>&1; then
    test_pass "qwen-fast model works with bearer token"
else
    test_fail "qwen-fast model failed"
fi

# Test qwen-quality-128k
test_info "Testing qwen-quality-128k (32B model)..."
QUALITY_RESPONSE=$(curl -s -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"model": "qwen-quality-128k:latest", "prompt": "Say hi", "stream": false}')

if echo "$QUALITY_RESPONSE" | jq -e '.response' > /dev/null 2>&1; then
    test_pass "qwen-quality-128k model works with bearer token"
else
    test_fail "qwen-quality-128k model failed"
fi
echo ""

###############################################################################
# Test 8: Credential Manager (Optional)
###############################################################################
echo "Test 8: Credential Manager Storage (Optional)"
echo "----------------------------------------------"
test_info "Testing credential manager storage..."

# Set encryption key for consistent encryption
export CREDENTIAL_ENCRYPTION_KEY="test-encryption-key-do-not-use-in-production"

# Try to store token
if node src/credential-manager.js store TEST_TOKEN "test-value" api-token 2>&1 | grep -q "stored securely"; then
    test_pass "Credential manager can store tokens"

    # Try to retrieve
    if node src/credential-manager.js retrieve TEST_TOKEN 2>&1 | grep -q "test-value"; then
        test_pass "Credential manager can retrieve tokens"
    else
        test_fail "Credential manager cannot retrieve tokens"
    fi

    # Cleanup
    rm -f /tmp/credentials.json
else
    test_fail "Credential manager cannot store tokens"
fi
echo ""

###############################################################################
# Summary
###############################################################################
echo "=================================================="
echo "Test Summary"
echo "=================================================="
echo -e "Total Tests: $((PASSED_TESTS + FAILED_TESTS))"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo ""
    echo -e "${RED}Some tests failed. Please review the output above.${NC}"
    exit 1
else
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo ""
    echo -e "${GREEN}All tests passed! Bearer token authentication is working correctly.${NC}"
fi
echo "=================================================="
