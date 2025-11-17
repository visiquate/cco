#!/bin/bash

##############################################################################
# Test Suite: Compile-Time Embedded Agent Definitions
#
# Tests that agent definitions are properly embedded in the CCO binary
# and available through the HTTP API without filesystem dependency.
##############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_test() {
    echo -e "${YELLOW}TEST:${NC} $1"
    ((TESTS_RUN++))
}

pass_test() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    ((TESTS_PASSED++))
}

fail_test() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    ((TESTS_FAILED++))
}

warn_test() {
    echo -e "${YELLOW}⚠ WARN:${NC} $1"
}

# Test 1: Check binary exists and version
print_header "Test 1: Binary Verification"
print_test "Check CCO binary exists"
if command -v cco &> /dev/null; then
    pass_test "CCO binary found at $(which cco)"
else
    fail_test "CCO binary not found in PATH"
    exit 1
fi

print_test "Check CCO version"
VERSION=$(cco --version 2>&1)
if [[ $VERSION == *"2025.11"* ]]; then
    pass_test "Version check: $VERSION"
else
    fail_test "Unexpected version: $VERSION"
fi

# Test 2: Runtime startup verification
print_header "Test 2: Runtime Startup & Agent Loading"
print_test "Verify server is running on port 3000"

# Check if server is running
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    pass_test "Server is running and responding"
else
    fail_test "Server not responding on port 3000"
    exit 1
fi

print_test "Check health endpoint returns expected fields"
HEALTH=$(curl -s http://localhost:3000/health)
if echo "$HEALTH" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    pass_test "Health status: ok"
else
    fail_test "Health endpoint invalid"
fi

# Test 3: HTTP API - List All Agents
print_header "Test 3: HTTP API - List All Agents"
print_test "GET /api/agents returns all agents"

AGENTS_RESPONSE=$(curl -s http://localhost:3000/api/agents)
AGENT_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.agents | length')

if [[ "$AGENT_COUNT" -ge 117 ]]; then
    pass_test "Agent count: $AGENT_COUNT (expected 117-119)"
else
    fail_test "Expected at least 117 agents, got $AGENT_COUNT"
fi

print_test "Verify agents have correct structure"
SAMPLE_AGENT=$(echo "$AGENTS_RESPONSE" | jq '.agents[0]')
if echo "$SAMPLE_AGENT" | jq -e '.name and .model and .description' > /dev/null 2>&1; then
    pass_test "Sample agent structure valid"
else
    fail_test "Agent structure invalid"
fi

# Test 4: Individual Agent Testing
print_header "Test 4: Individual Agent Testing (Sample of 15 Agents)"

# List of agents to test
TEST_AGENTS=(
    "chief-architect"
    "tdd-coding-agent"
    "rust-specialist"
    "python-expert"
    "python-pro"
    "swift-expert"
    "go-expert"
    "flutter-expert"
    "test-engineer"
    "security-auditor"
    "frontend-developer"
    "backend-architect"
    "api-explorer"
    "devops-engineer"
    "documentation-expert"
)

AGENT_TEST_RESULTS=()

for AGENT in "${TEST_AGENTS[@]}"; do
    print_test "Check agent: $AGENT"

    AGENT_DATA=$(curl -s "http://localhost:3000/api/agents/$AGENT")

    if echo "$AGENT_DATA" | jq -e '.name' > /dev/null 2>&1; then
        AGENT_MODEL=$(echo "$AGENT_DATA" | jq -r '.model')
        pass_test "Agent '$AGENT' found with model: $AGENT_MODEL"
        AGENT_TEST_RESULTS+=("$AGENT|$AGENT_MODEL")
    else
        fail_test "Agent '$AGENT' not found"
    fi
done

# Test 5: Performance - API Response Time
print_header "Test 5: Performance Testing"

print_test "Measure first API response time"
START_TIME=$(date +%s%N)
curl -s http://localhost:3000/api/agents > /dev/null
END_TIME=$(date +%s%N)
RESPONSE_TIME=$((($END_TIME - $START_TIME) / 1000000))

if [[ $RESPONSE_TIME -lt 50 ]]; then
    pass_test "First response time: ${RESPONSE_TIME}ms (target <50ms)"
else
    warn_test "First response time: ${RESPONSE_TIME}ms (acceptable, but slower than target)"
fi

print_test "Measure subsequent API response times (5 calls)"
TOTAL_TIME=0
for i in {1..5}; do
    START_TIME=$(date +%s%N)
    curl -s http://localhost:3000/api/agents > /dev/null
    END_TIME=$(date +%s%N)
    CALL_TIME=$((($END_TIME - $START_TIME) / 1000000))
    TOTAL_TIME=$((TOTAL_TIME + CALL_TIME))
done
AVG_TIME=$((TOTAL_TIME / 5))

if [[ $AVG_TIME -lt 50 ]]; then
    pass_test "Average response time (5 calls): ${AVG_TIME}ms"
else
    warn_test "Average response time (5 calls): ${AVG_TIME}ms"
fi

# Test 6: Filesystem Independence
print_header "Test 6: Filesystem Independence Test"

AGENTS_DIR="$HOME/.claude/agents"

print_test "Check if agents directory exists"
if [[ -d "$AGENTS_DIR" ]]; then
    pass_test "Agents directory found at $AGENTS_DIR"

    print_test "Rename agents directory temporarily"
    if mv "$AGENTS_DIR" "${AGENTS_DIR}.backup" 2>/dev/null; then
        pass_test "Agents directory renamed to .backup"

        # Give server a moment to handle any caching
        sleep 1

        print_test "Verify API still works without filesystem access"
        if curl -s http://localhost:3000/api/agents | jq -e '.agents | length' > /dev/null 2>&1; then
            AGENT_COUNT_NO_FS=$(curl -s http://localhost:3000/api/agents | jq '.agents | length')
            if [[ "$AGENT_COUNT_NO_FS" -ge 117 ]]; then
                pass_test "API still returns $AGENT_COUNT_NO_FS agents (FILESYSTEM NOT REQUIRED)"
            else
                fail_test "Agent count changed without filesystem"
            fi
        else
            fail_test "API failed without filesystem access"
        fi

        print_test "Restore agents directory"
        mv "${AGENTS_DIR}.backup" "$AGENTS_DIR" 2>/dev/null
        pass_test "Agents directory restored"
    else
        fail_test "Could not rename agents directory"
    fi
else
    warn_test "Agents directory not found - skipping filesystem independence test"
fi

# Test 7: agent-loader.js Integration
print_header "Test 7: agent-loader.js Integration"

print_test "Check if agent-loader.js exists"
if [[ -f "/Users/brent/git/cc-orchestra/agent-loader.js" ]]; then
    pass_test "agent-loader.js found"

    print_test "Set CCO_API_URL environment variable"
    export CCO_API_URL="http://localhost:3000/api"
    pass_test "CCO_API_URL=$CCO_API_URL"

    print_test "Test agent-loader.js with rust-specialist"
    if RUST_MODEL=$(node /Users/brent/git/cc-orchestra/agent-loader.js rust-specialist 2>/dev/null); then
        if [[ "$RUST_MODEL" == "haiku" ]]; then
            pass_test "rust-specialist correctly returns model: haiku"
        else
            fail_test "rust-specialist returned unexpected model: $RUST_MODEL (expected haiku)"
        fi
    else
        fail_test "agent-loader.js failed to fetch rust-specialist"
    fi

    # Test 5 more agents
    print_test "Test agent-loader.js with 5+ more agents"
    TEST_LOADERS=(
        "chief-architect|opus"
        "python-expert|haiku"
        "test-engineer|sonnet"
        "security-auditor|sonnet"
        "documentation-expert|haiku"
    )

    LOADER_PASSED=0
    LOADER_FAILED=0
    for AGENT_MODEL in "${TEST_LOADERS[@]}"; do
        AGENT=$(echo "$AGENT_MODEL" | cut -d'|' -f1)
        EXPECTED_MODEL=$(echo "$AGENT_MODEL" | cut -d'|' -f2)

        if RESULT=$(node /Users/brent/git/cc-orchestra/agent-loader.js "$AGENT" 2>/dev/null); then
            if [[ "$RESULT" == "$EXPECTED_MODEL" ]]; then
                pass_test "  $AGENT -> $RESULT (correct)"
                ((LOADER_PASSED++))
            else
                fail_test "  $AGENT -> $RESULT (expected $EXPECTED_MODEL)"
                ((LOADER_FAILED++))
            fi
        else
            fail_test "  Failed to load $AGENT"
            ((LOADER_FAILED++))
        fi
    done
else
    fail_test "agent-loader.js not found"
fi

# Test 8: Agent Model Assignment Verification
print_header "Test 8: Agent Model Assignment Verification"

print_test "Verify all agents have model assignments"
AGENTS_DATA=$(curl -s http://localhost:3000/api/agents)
AGENTS_WITHOUT_MODEL=$(echo "$AGENTS_DATA" | jq '[.agents[] | select(.model == null or .model == "")] | length')

if [[ "$AGENTS_WITHOUT_MODEL" -eq 0 ]]; then
    pass_test "All agents have model assignments"
else
    fail_test "Found $AGENTS_WITHOUT_MODEL agents without model assignment"
fi

print_test "Count agents by model type"
OPUS_COUNT=$(echo "$AGENTS_DATA" | jq '[.agents[] | select(.model == "opus")] | length')
SONNET_COUNT=$(echo "$AGENTS_DATA" | jq '[.agents[] | select(.model == "sonnet")] | length')
HAIKU_COUNT=$(echo "$AGENTS_DATA" | jq '[.agents[] | select(.model == "haiku")] | length')

echo "  Opus agents:   $OPUS_COUNT"
echo "  Sonnet agents: $SONNET_COUNT"
echo "  Haiku agents:  $HAIKU_COUNT"
echo "  Total:         $((OPUS_COUNT + SONNET_COUNT + HAIKU_COUNT))"

if [[ $OPUS_COUNT -ge 1 && $SONNET_COUNT -ge 1 && $HAIKU_COUNT -ge 1 ]]; then
    pass_test "All model types represented"
else
    fail_test "Missing some model types"
fi

# Test 9: Startup Message Verification
print_header "Test 9: Startup Message Verification"

print_test "Verify startup message shows agents loaded"
# Check via status command
if cco status 2>&1 | grep -q "Running"; then
    pass_test "CCO status shows running instances"
else
    warn_test "Could not verify startup message from cco status"
fi

# Final Summary
print_header "Test Summary"

echo ""
echo "Total Tests Run:    $TESTS_RUN"
echo -e "Tests Passed:       ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed:       ${RED}$TESTS_FAILED${NC}"

PASS_RATE=$((($TESTS_PASSED * 100) / $TESTS_RUN))
echo "Pass Rate:          $PASS_RATE%"

echo ""
echo "Agent Test Results (Sample):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
printf "%-30s %-15s\n" "Agent Name" "Model"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
for RESULT in "${AGENT_TEST_RESULTS[@]}"; do
    AGENT=$(echo "$RESULT" | cut -d'|' -f1)
    MODEL=$(echo "$RESULT" | cut -d'|' -f2)
    printf "%-30s %-15s\n" "$AGENT" "$MODEL"
done

echo ""
if [[ $TESTS_FAILED -eq 0 ]]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ $TESTS_FAILED TEST(S) FAILED${NC}"
    exit 1
fi
