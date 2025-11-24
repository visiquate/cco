#!/usr/bin/env bash
#
# Agent Definitions System - End-to-End Verification Script
#
# Tests the complete flow from CCO startup through Claude Code agent spawning
# with correct model assignments.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TEST_NUMBER=0

# Helper functions
print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_test() {
    TEST_NUMBER=$((TEST_NUMBER + 1))
    echo -e "\n${YELLOW}Test #${TEST_NUMBER}: $1${NC}"
}

print_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✅ PASS${NC}: $1"
}

print_fail() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}❌ FAIL${NC}: $1"
}

print_info() {
    echo -e "${BLUE}ℹ️  ${NC}$1"
}

# Test configuration
CCO_PORT=3000
CCO_URL="http://localhost:${CCO_PORT}"

print_header "Agent Definitions System - End-to-End Verification"

# ============================================================================
# SECTION 1: System Setup Verification
# ============================================================================
print_header "SECTION 1: System Setup Verification"

# Test 1.1: Check agent definition files exist
print_test "Agent definition files exist"
AGENT_DIR="$HOME/.claude/agents"
AGENT_COUNT=$(ls -1 "$AGENT_DIR"/*.md 2>/dev/null | wc -l)
if [ "$AGENT_COUNT" -ge 117 ]; then
    print_pass "Found $AGENT_COUNT agent definition files"
else
    print_fail "Expected 117+ agents, found $AGENT_COUNT"
fi

# Test 1.2: Check CCO binary exists
print_test "CCO binary exists and is accessible"
if command -v cco >/dev/null 2>&1; then
    CCO_VERSION=$(cco --version 2>&1 | head -1 || echo "unknown")
    print_pass "CCO binary found: $CCO_VERSION"
else
    print_fail "CCO binary not found in PATH"
fi

# Test 1.3: Check CCO is running
print_test "CCO server is running on port $CCO_PORT"
if pgrep -f "cco.*$CCO_PORT" > /dev/null; then
    print_pass "CCO process found"
else
    print_fail "CCO is not running on port $CCO_PORT"
fi

# ============================================================================
# SECTION 2: HTTP API Endpoint Verification
# ============================================================================
print_header "SECTION 2: HTTP API Endpoint Verification"

# Test 2.1: Health check endpoint
print_test "GET /health endpoint"
HEALTH_RESPONSE=$(curl -s "$CCO_URL/health")
if echo "$HEALTH_RESPONSE" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    VERSION=$(echo "$HEALTH_RESPONSE" | jq -r '.version')
    UPTIME=$(echo "$HEALTH_RESPONSE" | jq -r '.uptime')
    print_pass "Health check successful (version: $VERSION, uptime: ${UPTIME}s)"
else
    print_fail "Health check failed or returned invalid JSON"
fi

# Test 2.2: List all agents endpoint
print_test "GET /api/agents endpoint"
AGENTS_RESPONSE=$(curl -s "$CCO_URL/api/agents")
AGENTS_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.agents | length' 2>/dev/null || echo "0")
if [ "$AGENTS_COUNT" -ge 117 ]; then
    print_pass "API returned $AGENTS_COUNT agents"
else
    print_fail "Expected 117+ agents, API returned $AGENTS_COUNT"
fi

# Test 2.3: Get specific agent endpoint
print_test "GET /api/agents/{agent-name} endpoint"
RUST_AGENT=$(curl -s "$CCO_URL/api/agents/rust-specialist")
RUST_NAME=$(echo "$RUST_AGENT" | jq -r '.name' 2>/dev/null)
if [ "$RUST_NAME" = "rust-specialist" ]; then
    print_pass "Successfully retrieved rust-specialist agent"
else
    print_fail "Failed to retrieve rust-specialist agent"
fi

# ============================================================================
# SECTION 3: Agent Model Assignment Verification
# ============================================================================
print_header "SECTION 3: Agent Model Assignment Verification"

# Test agents organized by model tier (space-separated: "agent:model")
TEST_AGENTS="
chief-architect:opus
rust-specialist:haiku
test-engineer:haiku
python-specialist:haiku
tdd-coding-agent:haiku
security-auditor:sonnet
code-reviewer:sonnet
backend-architect:sonnet
api-explorer:sonnet
devops-engineer:haiku
frontend-developer:haiku
flutter-specialist:haiku
go-specialist:haiku
swift-specialist:haiku
documentation-expert:haiku
technical-writer:haiku
performance-engineer:sonnet
database-architect:sonnet
cloud-architect:sonnet
terraform-specialist:sonnet
"

for agent_entry in $TEST_AGENTS; do
    agent_name=$(echo "$agent_entry" | cut -d: -f1)
    expected_model=$(echo "$agent_entry" | cut -d: -f2)

    print_test "Agent '$agent_name' model assignment"

    AGENT_DATA=$(curl -s "$CCO_URL/api/agents/$agent_name")
    ACTUAL_MODEL=$(echo "$AGENT_DATA" | jq -r '.model' 2>/dev/null)

    if [ "$ACTUAL_MODEL" = "$expected_model" ]; then
        print_pass "$agent_name → $ACTUAL_MODEL (expected: $expected_model)"
    else
        print_fail "$agent_name → $ACTUAL_MODEL (expected: $expected_model)"
    fi
done

# ============================================================================
# SECTION 4: Model Distribution Verification
# ============================================================================
print_header "SECTION 4: Model Distribution Verification"

print_test "Model distribution across all agents"
OPUS_COUNT=$(curl -s "$CCO_URL/api/agents" | jq '[.agents[] | select(.model == "opus")] | length')
SONNET_COUNT=$(curl -s "$CCO_URL/api/agents" | jq '[.agents[] | select(.model == "sonnet")] | length')
HAIKU_COUNT=$(curl -s "$CCO_URL/api/agents" | jq '[.agents[] | select(.model == "haiku")] | length')

print_info "Opus agents: $OPUS_COUNT"
print_info "Sonnet agents: $SONNET_COUNT"
print_info "Haiku agents: $HAIKU_COUNT"

if [ "$OPUS_COUNT" -eq 1 ]; then
    print_pass "Exactly 1 Opus agent (chief-architect)"
else
    print_fail "Expected 1 Opus agent, found $OPUS_COUNT"
fi

if [ "$SONNET_COUNT" -ge 30 ] && [ "$SONNET_COUNT" -le 40 ]; then
    print_pass "Sonnet agent count in expected range: $SONNET_COUNT"
else
    print_fail "Sonnet agent count out of range: $SONNET_COUNT (expected 30-40)"
fi

if [ "$HAIKU_COUNT" -ge 75 ] && [ "$HAIKU_COUNT" -le 90 ]; then
    print_pass "Haiku agent count in expected range: $HAIKU_COUNT"
else
    print_fail "Haiku agent count out of range: $HAIKU_COUNT (expected 75-90)"
fi

# ============================================================================
# SECTION 5: Agent-Loader.js Integration
# ============================================================================
print_header "SECTION 5: Agent-Loader.js Integration"

print_test "agent-loader.js can read agent definitions"
cd ~/.claude
LOADER_OUTPUT=$(node -e "
const { getAgentModel } = require('./agent-loader.js');
console.log(JSON.stringify({
    'rust-specialist': getAgentModel('rust-specialist'),
    'chief-architect': getAgentModel('chief-architect'),
    'test-engineer': getAgentModel('test-engineer')
}));
" 2>&1 | grep -v "^✅" | tail -1)

if echo "$LOADER_OUTPUT" | jq . > /dev/null 2>&1; then
    LOADER_RUST=$(echo "$LOADER_OUTPUT" | jq -r '."rust-specialist"')
    LOADER_ARCHITECT=$(echo "$LOADER_OUTPUT" | jq -r '."chief-architect"')
    LOADER_TESTER=$(echo "$LOADER_OUTPUT" | jq -r '."test-engineer"')

    if [ "$LOADER_RUST" = "haiku" ] && [ "$LOADER_ARCHITECT" = "opus" ] && [ "$LOADER_TESTER" = "haiku" ]; then
        print_pass "agent-loader.js correctly reads models from definitions"
    else
        print_fail "agent-loader.js returned incorrect models: rust=$LOADER_RUST architect=$LOADER_ARCHITECT tester=$LOADER_TESTER"
    fi
else
    print_fail "agent-loader.js failed to return valid JSON"
fi

# ============================================================================
# SECTION 6: Data Consistency Verification
# ============================================================================
print_header "SECTION 6: Data Consistency Verification"

print_test "API matches file system agent count"
API_COUNT=$(curl -s "$CCO_URL/api/agents" | jq '.agents | length')
FILE_COUNT=$(ls -1 "$HOME/.claude/agents"/*.md 2>/dev/null | wc -l | tr -d ' ')

if [ "$API_COUNT" -eq "$FILE_COUNT" ]; then
    print_pass "Agent counts match: API=$API_COUNT, Files=$FILE_COUNT"
else
    print_fail "Agent count mismatch: API=$API_COUNT, Files=$FILE_COUNT"
fi

# ============================================================================
# SECTION 7: Performance Verification
# ============================================================================
print_header "SECTION 7: Performance Verification"

print_test "API response time < 100ms"
START_TIME=$(date +%s%N)
curl -s "$CCO_URL/api/agents" > /dev/null
END_TIME=$(date +%s%N)
RESPONSE_TIME_MS=$(( (END_TIME - START_TIME) / 1000000 ))

if [ "$RESPONSE_TIME_MS" -lt 100 ]; then
    print_pass "API responded in ${RESPONSE_TIME_MS}ms"
else
    print_fail "API too slow: ${RESPONSE_TIME_MS}ms (expected < 100ms)"
fi

# ============================================================================
# SECTION 8: Error Handling Verification
# ============================================================================
print_header "SECTION 8: Error Handling Verification"

print_test "Non-existent agent returns 404"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CCO_URL/api/agents/non-existent-agent")
if [ "$HTTP_CODE" -eq 404 ]; then
    print_pass "Correctly returns 404 for non-existent agent"
else
    print_fail "Expected 404, got $HTTP_CODE"
fi

# ============================================================================
# Final Summary
# ============================================================================
print_header "VERIFICATION SUMMARY"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))

echo ""
echo "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo "Pass Rate: ${PASS_RATE}%"
echo ""

if [ "$TESTS_FAILED" -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ ALL TESTS PASSED - System Ready for Deployment${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ SOME TESTS FAILED - Review Issues Above${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
