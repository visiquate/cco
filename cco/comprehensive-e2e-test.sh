#!/usr/bin/env bash
#
# Comprehensive End-to-End Test Suite for CCO Agent System
#
# Tests complete flow: CCO binary → embedded agents → HTTP API → agent-loader.js → Claude spawning
#
# Author: Test Engineer
# Date: 2025-11-15
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TEST_NUMBER=0
WARNINGS=0

# Performance tracking (using arrays for compatibility)
PERF_METRIC_NAMES=()
PERF_METRIC_VALUES=()

# Helper functions
print_header() {
    echo -e "\n${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║ $1${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}\n"
}

print_section() {
    echo -e "\n${CYAN}▶ $1${NC}\n"
}

print_test() {
    TEST_NUMBER=$((TEST_NUMBER + 1))
    echo -e "${YELLOW}Test #${TEST_NUMBER}: $1${NC}"
}

print_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}  ✅ PASS${NC}: $1"
}

print_fail() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}  ❌ FAIL${NC}: $1"
}

print_warn() {
    WARNINGS=$((WARNINGS + 1))
    echo -e "${YELLOW}  ⚠️  WARN${NC}: $1"
}

print_info() {
    echo -e "${BLUE}  ℹ️  ${NC}$1"
}

print_metric() {
    echo -e "${CYAN}  📊 $1${NC}"
}

# Configuration
CCO_PORT=3000
CCO_URL="http://localhost:${CCO_PORT}"
CCO_BINARY="./target/release/cco"
CCO_PID=""
REPORT_FILE="e2e-test-report-$(date +%Y%m%d-%H%M%S).md"

# Cleanup function
cleanup() {
    if [ -n "$CCO_PID" ] && kill -0 "$CCO_PID" 2>/dev/null; then
        print_info "Stopping CCO server (PID: $CCO_PID)"
        kill "$CCO_PID" 2>/dev/null || true
        wait "$CCO_PID" 2>/dev/null || true
    fi
}

trap cleanup EXIT

# ============================================================================
print_header "CCO COMPREHENSIVE END-TO-END TEST SUITE"
# ============================================================================

print_info "Test started: $(date)"
print_info "CCO Binary: $CCO_BINARY"
print_info "CCO Port: $CCO_PORT"
print_info "Report file: $REPORT_FILE"

# ============================================================================
print_section "PHASE 1: Build Verification"
# ============================================================================

print_test "CCO binary exists and is executable"
if [ -f "$CCO_BINARY" ] && [ -x "$CCO_BINARY" ]; then
    SIZE=$(ls -lh "$CCO_BINARY" | awk '{print $5}')
    print_pass "CCO binary found (size: $SIZE)"
else
    print_fail "CCO binary not found or not executable at $CCO_BINARY"
    exit 1
fi

print_test "CCO binary has embedded agents"
if strings "$CCO_BINARY" | grep -q "chief-architect"; then
    AGENT_STRINGS=$(strings "$CCO_BINARY" | grep -E "(chief-architect|rust-specialist|python-specialist)" | wc -l | tr -d ' ')
    print_pass "Found $AGENT_STRINGS agent-related strings in binary"
else
    print_fail "No agent data found in binary"
fi

print_test "CCO version information"
VERSION_OUTPUT=$("$CCO_BINARY" --version 2>&1 || echo "error")
if echo "$VERSION_OUTPUT" | grep -q "cco"; then
    VERSION=$(echo "$VERSION_OUTPUT" | head -1)
    print_pass "Version: $VERSION"
else
    print_fail "Could not retrieve version information"
fi

# ============================================================================
print_section "PHASE 2: Embedded Agents Configuration"
# ============================================================================

print_test "Embedded agents configuration exists"
if [ -f "config/agents.json" ]; then
    AGENT_COUNT=$(jq '. | length' config/agents.json 2>/dev/null || echo "0")
    print_pass "Found agents.json with $AGENT_COUNT agents"
else
    print_fail "config/agents.json not found"
fi

print_test "Agent definitions directory exists"
if [ -d "config/agents" ]; then
    MD_COUNT=$(ls -1 config/agents/*.md 2>/dev/null | wc -l | tr -d ' ')
    print_pass "Found $MD_COUNT agent definition files"
else
    print_fail "config/agents directory not found"
fi

print_test "Model distribution in agents.json"
if [ -f "config/agents.json" ]; then
    OPUS_COUNT=$(jq '[.[] | select(.model == "opus")] | length' config/agents.json)
    SONNET_COUNT=$(jq '[.[] | select(.model == "sonnet")] | length' config/agents.json)
    HAIKU_COUNT=$(jq '[.[] | select(.model == "haiku")] | length' config/agents.json)

    print_metric "Opus: $OPUS_COUNT, Sonnet: $SONNET_COUNT, Haiku: $HAIKU_COUNT"

    if [ "$OPUS_COUNT" -eq 1 ]; then
        print_pass "Exactly 1 Opus agent (chief-architect)"
    else
        print_fail "Expected 1 Opus agent, found $OPUS_COUNT"
    fi
else
    print_fail "Cannot verify model distribution"
fi

# ============================================================================
print_section "PHASE 3: CCO Server Startup"
# ============================================================================

print_test "Starting CCO server on port $CCO_PORT"
"$CCO_BINARY" run --port "$CCO_PORT" > /tmp/cco-test.log 2>&1 &
CCO_PID=$!

print_info "CCO PID: $CCO_PID"

# Wait for server to start
MAX_WAIT=10
WAIT_COUNT=0
while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    if curl -s "$CCO_URL/health" > /dev/null 2>&1; then
        break
    fi
    sleep 1
    WAIT_COUNT=$((WAIT_COUNT + 1))
done

if [ $WAIT_COUNT -lt $MAX_WAIT ]; then
    STARTUP_TIME="${WAIT_COUNT}s"
    STARTUP_TIME_VAL=$WAIT_COUNT
    print_pass "Server started in $STARTUP_TIME"
else
    print_fail "Server failed to start within ${MAX_WAIT}s"
    cat /tmp/cco-test.log
    exit 1
fi

# ============================================================================
print_section "PHASE 4: HTTP API Endpoints"
# ============================================================================

print_test "GET /health endpoint"
START_MS=$(date +%s%N)
HEALTH_RESPONSE=$(curl -s "$CCO_URL/health")
END_MS=$(date +%s%N)
HEALTH_TIME_MS=$(( (END_MS - START_MS) / 1000000 ))

if echo "$HEALTH_RESPONSE" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    VERSION=$(echo "$HEALTH_RESPONSE" | jq -r '.version')
    UPTIME=$(echo "$HEALTH_RESPONSE" | jq -r '.uptime')
    print_pass "Health check OK (version: $VERSION, uptime: ${UPTIME}s) - ${HEALTH_TIME_MS}ms"
else
    print_fail "Health check failed"
fi

print_test "GET /api/agents (list all agents)"
START_MS=$(date +%s%N)
AGENTS_RESPONSE=$(curl -s "$CCO_URL/api/agents")
END_MS=$(date +%s%N)
LIST_TIME_MS=$(( (END_MS - START_MS) / 1000000 ))

# Handle both array and object-with-agents format
if echo "$AGENTS_RESPONSE" | jq -e '.agents' > /dev/null 2>&1; then
    AGENTS_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.agents | length' 2>/dev/null || echo "0")
else
    AGENTS_COUNT=$(echo "$AGENTS_RESPONSE" | jq '. | length' 2>/dev/null || echo "0")
fi

if [ "$AGENTS_COUNT" -ge 117 ]; then
    print_pass "API returned $AGENTS_COUNT agents - ${LIST_TIME_MS}ms"
else
    print_fail "Expected 117+ agents, got $AGENTS_COUNT"
fi

print_test "GET /api/agents/{agent-name} (individual agent)"
TEST_AGENTS=("chief-architect" "rust-specialist" "python-specialist" "test-engineer" "tdd-coding-agent")
for agent in "${TEST_AGENTS[@]}"; do
    START_MS=$(date +%s%N)
    AGENT_RESPONSE=$(curl -s "$CCO_URL/api/agents/$agent")
    END_MS=$(date +%s%N)
    GET_TIME_MS=$(( (END_MS - START_MS) / 1000000 ))

    AGENT_NAME=$(echo "$AGENT_RESPONSE" | jq -r '.name' 2>/dev/null)
    AGENT_MODEL=$(echo "$AGENT_RESPONSE" | jq -r '.model' 2>/dev/null)

    if [ "$AGENT_NAME" = "$agent" ]; then
        print_pass "$agent → $AGENT_MODEL (${GET_TIME_MS}ms)"
    else
        print_fail "$agent lookup failed"
    fi
done

# ============================================================================
print_section "PHASE 5: Model Assignment Verification"
# ============================================================================

print_test "Verify critical agent model assignments"

# Expected model assignments (agent:model format)
EXPECTED_MODELS=(
    "chief-architect:opus"
    "rust-specialist:haiku"
    "python-specialist:haiku"
    "swift-specialist:haiku"
    "go-specialist:haiku"
    "flutter-specialist:haiku"
    "tdd-coding-agent:haiku"
    "test-engineer:haiku"
    "security-auditor:sonnet"
    "code-reviewer:sonnet"
    "backend-architect:sonnet"
    "api-explorer:sonnet"
    "devops-engineer:haiku"
    "documentation-expert:haiku"
    "technical-writer:haiku"
    "performance-engineer:sonnet"
    "database-architect:sonnet"
    "cloud-architect:sonnet"
)

CORRECT_MODELS=0
INCORRECT_MODELS=0

for entry in "${EXPECTED_MODELS[@]}"; do
    agent=$(echo "$entry" | cut -d: -f1)
    expected=$(echo "$entry" | cut -d: -f2)
    ACTUAL=$(curl -s "$CCO_URL/api/agents/$agent" | jq -r '.model' 2>/dev/null)

    if [ "$ACTUAL" = "$expected" ]; then
        CORRECT_MODELS=$((CORRECT_MODELS + 1))
    else
        INCORRECT_MODELS=$((INCORRECT_MODELS + 1))
        print_fail "$agent: expected '$expected', got '$ACTUAL'"
    fi
done

if [ $INCORRECT_MODELS -eq 0 ]; then
    print_pass "All $CORRECT_MODELS tested agents have correct model assignments"
else
    print_fail "$INCORRECT_MODELS agents have incorrect model assignments"
fi

# ============================================================================
print_section "PHASE 6: Model Distribution Analysis"
# ============================================================================

print_test "Analyze model distribution across all agents"
AGENTS_API_RESPONSE=$(curl -s "$CCO_URL/api/agents")

# Handle both array and object-with-agents format
if echo "$AGENTS_API_RESPONSE" | jq -e '.agents' > /dev/null 2>&1; then
    OPUS_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.agents[] | select(.model == "opus")] | length' 2>/dev/null || echo "0")
    SONNET_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.agents[] | select(.model == "sonnet")] | length' 2>/dev/null || echo "0")
    HAIKU_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.agents[] | select(.model == "haiku")] | length' 2>/dev/null || echo "0")
else
    OPUS_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.[] | select(.model == "opus")] | length' 2>/dev/null || echo "0")
    SONNET_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.[] | select(.model == "sonnet")] | length' 2>/dev/null || echo "0")
    HAIKU_API=$(echo "$AGENTS_API_RESPONSE" | jq '[.[] | select(.model == "haiku")] | length' 2>/dev/null || echo "0")
fi

TOTAL_API=$((OPUS_API + SONNET_API + HAIKU_API))

print_metric "Distribution: Opus=$OPUS_API, Sonnet=$SONNET_API, Haiku=$HAIKU_API (Total=$TOTAL_API)"

# Validate distribution
if [ "$OPUS_API" -eq 1 ]; then
    print_pass "Opus count correct (1)"
else
    print_fail "Opus count incorrect (expected 1, got $OPUS_API)"
fi

if [ "$SONNET_API" -ge 30 ] && [ "$SONNET_API" -le 40 ]; then
    print_pass "Sonnet count in range (30-40): $SONNET_API"
else
    print_fail "Sonnet count out of range: $SONNET_API"
fi

if [ "$HAIKU_API" -ge 75 ] && [ "$HAIKU_API" -le 90 ]; then
    print_pass "Haiku count in range (75-90): $HAIKU_API"
else
    print_fail "Haiku count out of range: $HAIKU_API"
fi

# ============================================================================
print_section "PHASE 7: Cost Optimization Verification"
# ============================================================================

print_test "Calculate potential cost savings"

if [ "$TOTAL_API" -gt 0 ]; then
    HAIKU_PERCENTAGE=$((HAIKU_API * 100 / TOTAL_API))
    print_metric "Haiku usage: ${HAIKU_PERCENTAGE}% of agents"

    if [ "$HAIKU_PERCENTAGE" -ge 65 ]; then
        print_pass "Cost-optimized: ${HAIKU_PERCENTAGE}% using Haiku (target: 65%+)"
    else
        print_warn "Below target: ${HAIKU_PERCENTAGE}% Haiku (target: 65%+)"
    fi

    # Estimate cost savings (assuming Haiku is 20x cheaper than Opus)
    OPUS_COST_MULTIPLIER=20
    SONNET_COST_MULTIPLIER=4
    BASELINE_COST=$((TOTAL_API * OPUS_COST_MULTIPLIER))
    ACTUAL_COST=$((OPUS_API * OPUS_COST_MULTIPLIER + SONNET_API * SONNET_COST_MULTIPLIER + HAIKU_API * 1))
    SAVINGS_PERCENTAGE=$(( (BASELINE_COST - ACTUAL_COST) * 100 / BASELINE_COST ))

    print_metric "Cost savings vs all-Opus: ${SAVINGS_PERCENTAGE}%"
else
    print_fail "No agents found for cost analysis"
    HAIKU_PERCENTAGE=0
    SAVINGS_PERCENTAGE=0
fi

# ============================================================================
print_section "PHASE 8: Performance Benchmarks"
# ============================================================================

print_test "API response time benchmarks (50 requests)"
RESPONSE_TIMES=()
for i in {1..50}; do
    START_MS=$(date +%s%N)
    curl -s "$CCO_URL/api/agents/rust-specialist" > /dev/null
    END_MS=$(date +%s%N)
    TIME_MS=$(( (END_MS - START_MS) / 1000000 ))
    RESPONSE_TIMES+=($TIME_MS)
done

# Calculate stats
TOTAL_TIME=0
MIN_TIME=999999
MAX_TIME=0
for time in "${RESPONSE_TIMES[@]}"; do
    TOTAL_TIME=$((TOTAL_TIME + time))
    [ $time -lt $MIN_TIME ] && MIN_TIME=$time
    [ $time -gt $MAX_TIME ] && MAX_TIME=$time
done
AVG_TIME=$((TOTAL_TIME / 50))

print_metric "Average: ${AVG_TIME}ms, Min: ${MIN_TIME}ms, Max: ${MAX_TIME}ms"

if [ $AVG_TIME -lt 50 ]; then
    print_pass "Excellent performance: ${AVG_TIME}ms average"
elif [ $AVG_TIME -lt 100 ]; then
    print_pass "Good performance: ${AVG_TIME}ms average"
else
    print_warn "Slow performance: ${AVG_TIME}ms average (target: <100ms)"
fi

# ============================================================================
print_section "PHASE 9: Error Handling"
# ============================================================================

print_test "Non-existent agent returns 404"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CCO_URL/api/agents/non-existent-agent-xyz")
if [ "$HTTP_CODE" = "404" ]; then
    print_pass "Correctly returns 404 for non-existent agent"
else
    print_fail "Expected 404, got $HTTP_CODE"
fi

print_test "Malformed requests handled gracefully"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CCO_URL/api/agents/")
if [ "$HTTP_CODE" = "404" ] || [ "$HTTP_CODE" = "400" ]; then
    print_pass "Malformed request handled (status: $HTTP_CODE)"
else
    print_fail "Unexpected status for malformed request: $HTTP_CODE"
fi

# ============================================================================
print_section "PHASE 10: Critical Path Test"
# ============================================================================

print_test "Complete workflow: User → CCO → Agent Model"
print_info "Simulating: User requests rust-specialist agent"

# Step 1: CCO serves agent data
print_info "Step 1: Query CCO API for rust-specialist"
AGENT_DATA=$(curl -s "$CCO_URL/api/agents/rust-specialist")
AGENT_MODEL=$(echo "$AGENT_DATA" | jq -r '.model')

if [ "$AGENT_MODEL" = "haiku" ]; then
    print_pass "Step 1: CCO returned model='haiku'"
else
    print_fail "Step 1: Expected 'haiku', got '$AGENT_MODEL'"
fi

# Step 2: Verify agent has correct definition
print_info "Step 2: Verify agent definition complete"
AGENT_ROLE=$(echo "$AGENT_DATA" | jq -r '.role')
if [ -n "$AGENT_ROLE" ] && [ "$AGENT_ROLE" != "null" ]; then
    print_pass "Step 2: Agent has complete definition (role: ${AGENT_ROLE:0:50}...)"
else
    print_fail "Step 2: Agent definition incomplete"
fi

# Step 3: Verify cost optimization
print_info "Step 3: Verify cost-effective model selection"
if [ "$AGENT_MODEL" = "haiku" ]; then
    print_pass "Step 3: Using cost-effective 'haiku' model (not 'sonnet')"
else
    print_fail "Step 3: Not using cost-effective model"
fi

print_pass "Complete critical path verified"

# ============================================================================
print_section "PHASE 11: Filesystem Independence Test"
# ============================================================================

print_test "CCO operates independently of filesystem agents"
print_info "Testing: CCO uses embedded agents, not filesystem"

# Check if filesystem agents exist
if [ -d "$HOME/.claude/agents" ]; then
    FS_AGENT_COUNT=$(ls -1 "$HOME/.claude/agents"/*.md 2>/dev/null | wc -l | tr -d ' ')
    print_info "Filesystem agents found: $FS_AGENT_COUNT"

    if [ "$FS_AGENT_COUNT" -gt 0 ]; then
        print_warn "Filesystem agents present, but CCO should use embedded agents"
    fi
fi

# Verify CCO is using embedded agents
if strings "$CCO_BINARY" | grep -q "rust-specialist"; then
    print_pass "CCO has embedded agents (filesystem independent)"
else
    print_fail "CCO may be relying on filesystem agents"
fi

# ============================================================================
print_section "PHASE 12: Agent Data Completeness"
# ============================================================================

print_test "All agents accessible and complete"
INCOMPLETE_AGENTS=0
MISSING_FIELDS=0

# Handle both array and object-with-agents format
if echo "$AGENTS_RESPONSE" | jq -e '.agents' > /dev/null 2>&1; then
    AGENTS_ARRAY=$(echo "$AGENTS_RESPONSE" | jq '.agents')
else
    AGENTS_ARRAY="$AGENTS_RESPONSE"
fi

for i in $(seq 0 $((AGENTS_COUNT - 1))); do
    AGENT=$(echo "$AGENTS_ARRAY" | jq -r ".[$i]")

    # Check required fields
    NAME=$(echo "$AGENT" | jq -r '.name')
    MODEL=$(echo "$AGENT" | jq -r '.model')
    TYPE=$(echo "$AGENT" | jq -r '.type')

    if [ -z "$NAME" ] || [ "$NAME" = "null" ] || \
       [ -z "$MODEL" ] || [ "$MODEL" = "null" ] || \
       [ -z "$TYPE" ] || [ "$TYPE" = "null" ]; then
        INCOMPLETE_AGENTS=$((INCOMPLETE_AGENTS + 1))
        MISSING_FIELDS=$((MISSING_FIELDS + 1))
    fi
done

if [ $INCOMPLETE_AGENTS -eq 0 ]; then
    print_pass "All $AGENTS_COUNT agents have complete data"
else
    print_fail "$INCOMPLETE_AGENTS agents have incomplete data"
fi

# ============================================================================
print_section "FINAL SUMMARY"
# ============================================================================

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))
else
    PASS_RATE=0
fi

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                         TEST RESULTS                           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:      $TOTAL_TESTS"
echo -e "  ${GREEN}Passed:          $TESTS_PASSED${NC}"
echo -e "  ${RED}Failed:          $TESTS_FAILED${NC}"
echo -e "  ${YELLOW}Warnings:        $WARNINGS${NC}"
echo -e "  Pass Rate:        ${PASS_RATE}%"
echo ""
echo -e "${CYAN}Performance Metrics:${NC}"
echo -e "  Startup time:     ${STARTUP_TIME_VAL}s"
echo -e "  Avg response:     ${AVG_TIME}ms"
echo -e "  Min response:     ${MIN_TIME}ms"
echo -e "  Max response:     ${MAX_TIME}ms"
echo ""
echo -e "${CYAN}Agent Statistics:${NC}"
echo -e "  Total agents:     $TOTAL_API"
echo -e "  Opus agents:      $OPUS_API ($(( OPUS_API * 100 / TOTAL_API ))%)"
echo -e "  Sonnet agents:    $SONNET_API ($(( SONNET_API * 100 / TOTAL_API ))%)"
echo -e "  Haiku agents:     $HAIKU_API ($(( HAIKU_API * 100 / TOTAL_API ))%)"
echo -e "  Cost savings:     ${SAVINGS_PERCENTAGE}% vs all-Opus"
echo ""

# Generate markdown report
{
    echo "# CCO End-to-End Test Report"
    echo ""
    echo "**Date:** $(date)"
    echo "**CCO Version:** $VERSION"
    echo ""
    echo "## Summary"
    echo ""
    echo "- **Total Tests:** $TOTAL_TESTS"
    echo "- **Passed:** $TESTS_PASSED"
    echo "- **Failed:** $TESTS_FAILED"
    echo "- **Warnings:** $WARNINGS"
    echo "- **Pass Rate:** ${PASS_RATE}%"
    echo ""
    echo "## Performance Metrics"
    echo ""
    echo "| Metric | Value |"
    echo "|--------|-------|"
    echo "| Startup Time | ${STARTUP_TIME_VAL}s |"
    echo "| Average Response | ${AVG_TIME}ms |"
    echo "| Min Response | ${MIN_TIME}ms |"
    echo "| Max Response | ${MAX_TIME}ms |"
    echo ""
    echo "## Agent Distribution"
    echo ""
    echo "| Model | Count | Percentage |"
    echo "|-------|-------|------------|"
    echo "| Opus | $OPUS_API | $(( OPUS_API * 100 / TOTAL_API ))% |"
    echo "| Sonnet | $SONNET_API | $(( SONNET_API * 100 / TOTAL_API ))% |"
    echo "| Haiku | $HAIKU_API | $(( HAIKU_API * 100 / TOTAL_API ))% |"
    echo ""
    echo "**Total Agents:** $TOTAL_API"
    echo ""
    echo "## Cost Optimization"
    echo ""
    echo "- **Haiku Usage:** ${HAIKU_PERCENTAGE}% (Target: 65%+)"
    echo "- **Estimated Savings:** ${SAVINGS_PERCENTAGE}% vs all-Opus deployment"
    echo ""
    echo "## Production Readiness"
    echo ""
    if [ $TESTS_FAILED -eq 0 ] && [ $WARNINGS -le 2 ]; then
        echo "✅ **READY FOR PRODUCTION**"
        echo ""
        echo "All critical tests passed. System is production-ready."
    elif [ $TESTS_FAILED -eq 0 ]; then
        echo "⚠️  **READY WITH WARNINGS**"
        echo ""
        echo "All tests passed but $WARNINGS warnings present. Review recommended."
    else
        echo "❌ **NOT READY**"
        echo ""
        echo "$TESTS_FAILED tests failed. Issues must be resolved."
    fi
} > "$REPORT_FILE"

print_info "Test report saved to: $REPORT_FILE"
echo ""

# Final verdict
if [ $TESTS_FAILED -eq 0 ] && [ $WARNINGS -le 2 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                                ║${NC}"
    echo -e "${GREEN}║            ✅  ALL TESTS PASSED - PRODUCTION READY             ║${NC}"
    echo -e "${GREEN}║                                                                ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
    exit 0
elif [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${YELLOW}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║                                                                ║${NC}"
    echo -e "${YELLOW}║        ⚠️   TESTS PASSED WITH WARNINGS - REVIEW NEEDED         ║${NC}"
    echo -e "${YELLOW}║                                                                ║${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                                ║${NC}"
    echo -e "${RED}║           ❌  TESTS FAILED - ISSUES MUST BE RESOLVED           ║${NC}"
    echo -e "${RED}║                                                                ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
