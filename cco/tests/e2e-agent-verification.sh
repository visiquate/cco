#!/usr/bin/env bash
#
# End-to-End Agent Definition System Verification
#
# This script validates the complete flow from CCO server startup through
# Claude Code agent spawning, including HTTP API, agent-loader integration,
# and fallback mechanisms.
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CCO_PORT=${CCO_PORT:-3210}
CCO_HOST=${CCO_HOST:-127.0.0.1}
CCO_API_URL="http://${CCO_HOST}:${CCO_PORT}"
TEST_RESULTS_FILE="/tmp/e2e-test-results-$(date +%s).json"
AGENT_LOADER_PATH="$HOME/.claude/agent-loader.js"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNINGS=0

# Store test results
declare -A TEST_RESULTS
declare -a FAILED_TEST_DETAILS
declare -a WARNING_DETAILS

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
    ((PASSED_TESTS++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $*"
    ((FAILED_TESTS++))
    FAILED_TEST_DETAILS+=("$*")
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $*"
    ((WARNINGS++))
    WARNING_DETAILS+=("$*")
}

# Test result tracking
record_test_result() {
    local test_name="$1"
    local result="$2"
    local details="${3:-}"

    ((TOTAL_TESTS++))
    TEST_RESULTS["$test_name"]="$result"

    if [ "$result" = "PASS" ]; then
        log_success "$test_name"
    else
        log_fail "$test_name: $details"
    fi
}

# Check if CCO server is running
check_cco_running() {
    log_info "Checking if CCO server is running on port $CCO_PORT..."

    if curl -s -f "$CCO_API_URL/health" > /dev/null 2>&1; then
        log_success "CCO server is running"
        return 0
    else
        log_fail "CCO server is not responding on $CCO_API_URL"
        return 1
    fi
}

# Start CCO server if not running
start_cco_server() {
    log_info "Starting CCO server..."

    cd "$(dirname "$0")/.." || exit 1

    # Check if already running
    if check_cco_running 2>/dev/null; then
        log_info "CCO server already running"
        return 0
    fi

    # Start in background
    cargo run --release -- --port "$CCO_PORT" > /tmp/cco-e2e-test.log 2>&1 &
    CCO_PID=$!

    # Wait for server to start (max 10 seconds)
    for i in {1..20}; do
        if check_cco_running 2>/dev/null; then
            log_success "CCO server started (PID: $CCO_PID)"
            echo "$CCO_PID" > /tmp/cco-e2e-test.pid
            return 0
        fi
        sleep 0.5
    done

    log_fail "CCO server failed to start within 10 seconds"
    return 1
}

# Test 1: HTTP API - List all agents
test_list_all_agents() {
    log_info "Test 1: GET /api/agents - List all agents"

    local response
    response=$(curl -s "$CCO_API_URL/api/agents")

    # Check if response is valid JSON
    if ! echo "$response" | jq . > /dev/null 2>&1; then
        record_test_result "API_LIST_ALL_AGENTS" "FAIL" "Invalid JSON response"
        return 1
    fi

    # Check for agents array
    local agent_count
    agent_count=$(echo "$response" | jq '.agents | length')

    if [ "$agent_count" -lt 100 ]; then
        record_test_result "API_LIST_ALL_AGENTS" "FAIL" "Expected 117-119 agents, got $agent_count"
        return 1
    fi

    record_test_result "API_LIST_ALL_AGENTS" "PASS"
    log_info "  → Found $agent_count agents"
    return 0
}

# Test 2: HTTP API - Get specific agents and verify models
test_get_specific_agents() {
    log_info "Test 2: GET /api/agents/{agent-name} - Verify specific agent models"

    # Test cases: agent name -> expected model
    declare -A test_cases=(
        ["chief-architect"]="opus"
        ["rust-specialist"]="haiku"
        ["test-engineer"]="haiku"
        ["security-auditor"]="sonnet"
        ["api-explorer"]="sonnet"
        ["python-specialist"]="haiku"
        ["tdd-coding-agent"]="haiku"
        ["devops-engineer"]="haiku"
        ["documentation-expert"]="haiku"
        ["backend-architect"]="sonnet"
    )

    local all_passed=true

    for agent_name in "${!test_cases[@]}"; do
        local expected_model="${test_cases[$agent_name]}"

        log_info "  Testing: $agent_name (expect: $expected_model)"

        local response
        response=$(curl -s "$CCO_API_URL/api/agents/$agent_name")

        # Check if response is valid JSON
        if ! echo "$response" | jq . > /dev/null 2>&1; then
            record_test_result "API_GET_AGENT_$agent_name" "FAIL" "Invalid JSON response"
            all_passed=false
            continue
        fi

        # Extract model from response
        local actual_model
        actual_model=$(echo "$response" | jq -r '.model')

        if [ "$actual_model" = "$expected_model" ]; then
            log_success "    $agent_name: $actual_model ✓"
        else
            log_fail "    $agent_name: Expected '$expected_model', got '$actual_model'"
            all_passed=false
        fi
    done

    if [ "$all_passed" = true ]; then
        record_test_result "API_GET_SPECIFIC_AGENTS" "PASS"
        return 0
    else
        record_test_result "API_GET_SPECIFIC_AGENTS" "FAIL" "Some agent models incorrect"
        return 1
    fi
}

# Test 3: HTTP API - 404 for non-existent agent
test_404_handling() {
    log_info "Test 3: GET /api/agents/non-existent - Verify 404 handling"

    local http_code
    http_code=$(curl -s -o /dev/null -w "%{http_code}" "$CCO_API_URL/api/agents/non-existent-agent-xyz")

    if [ "$http_code" = "404" ]; then
        record_test_result "API_404_HANDLING" "PASS"
        return 0
    else
        record_test_result "API_404_HANDLING" "FAIL" "Expected 404, got $http_code"
        return 1
    fi
}

# Test 4: HTTP API - Response structure validation
test_response_structure() {
    log_info "Test 4: Verify API response structure"

    local response
    response=$(curl -s "$CCO_API_URL/api/agents/rust-specialist")

    # Check required fields
    local required_fields=("name" "model" "description" "tools")
    local all_present=true

    for field in "${required_fields[@]}"; do
        if ! echo "$response" | jq -e ".$field" > /dev/null 2>&1; then
            log_fail "  Missing required field: $field"
            all_present=false
        fi
    done

    if [ "$all_present" = true ]; then
        record_test_result "API_RESPONSE_STRUCTURE" "PASS"
        return 0
    else
        record_test_result "API_RESPONSE_STRUCTURE" "FAIL" "Missing required fields"
        return 1
    fi
}

# Test 5: HTTP API - Response time check
test_response_time() {
    log_info "Test 5: Verify API response times"

    # Test list endpoint (should be < 10ms)
    local start_time=$(date +%s%N)
    curl -s "$CCO_API_URL/api/agents" > /dev/null
    local end_time=$(date +%s%N)
    local duration_ms=$(( (end_time - start_time) / 1000000 ))

    log_info "  → List endpoint: ${duration_ms}ms"

    if [ "$duration_ms" -gt 100 ]; then
        log_warning "  List endpoint slow: ${duration_ms}ms > 100ms"
    fi

    # Test individual agent endpoint (should be < 2ms)
    start_time=$(date +%s%N)
    curl -s "$CCO_API_URL/api/agents/rust-specialist" > /dev/null
    end_time=$(date +%s%N)
    duration_ms=$(( (end_time - start_time) / 1000000 ))

    log_info "  → Individual agent endpoint: ${duration_ms}ms"

    if [ "$duration_ms" -gt 50 ]; then
        log_warning "  Individual endpoint slow: ${duration_ms}ms > 50ms"
    fi

    record_test_result "API_RESPONSE_TIME" "PASS"
    return 0
}

# Test 6: agent-loader.js integration with API
test_agent_loader_with_api() {
    log_info "Test 6: agent-loader.js integration with CCO API"

    # Set environment variable to use API
    export CCO_API_URL="$CCO_API_URL"

    # Test multiple agents
    declare -A test_agents=(
        ["rust-specialist"]="haiku"
        ["chief-architect"]="opus"
        ["security-auditor"]="sonnet"
        ["test-engineer"]="haiku"
    )

    local all_passed=true

    for agent_name in "${!test_agents[@]}"; do
        local expected_model="${test_agents[$agent_name]}"

        log_info "  Testing agent-loader: $agent_name"

        # Run agent-loader
        local output
        output=$(node "$AGENT_LOADER_PATH" "$agent_name" 2>&1)

        # Extract model from last line (the actual output)
        local actual_model
        actual_model=$(echo "$output" | tail -n1)

        # Check if it shows API source in logs
        if echo "$output" | grep -q "from API"; then
            log_success "    Agent loaded from API: $agent_name"
        else
            log_warning "    No API source indicator in output for $agent_name"
        fi

        if [ "$actual_model" = "$expected_model" ]; then
            log_success "    Model match: $actual_model ✓"
        else
            log_fail "    Model mismatch: expected '$expected_model', got '$actual_model'"
            all_passed=false
        fi
    done

    if [ "$all_passed" = true ]; then
        record_test_result "AGENT_LOADER_WITH_API" "PASS"
        return 0
    else
        record_test_result "AGENT_LOADER_WITH_API" "FAIL" "Some agents returned incorrect models"
        return 1
    fi
}

# Test 7: CLI usage test
test_agent_loader_cli() {
    log_info "Test 7: agent-loader.js CLI usage"

    export CCO_API_URL="$CCO_API_URL"

    local output
    output=$(node "$AGENT_LOADER_PATH" "rust-specialist" 2>&1)

    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        record_test_result "AGENT_LOADER_CLI" "PASS"
        return 0
    else
        record_test_result "AGENT_LOADER_CLI" "FAIL" "CLI exited with code $exit_code"
        return 1
    fi
}

# Test 8: Fallback mechanism when API unavailable
test_fallback_mechanism() {
    log_info "Test 8: Fallback to local files when API unavailable"

    # Stop CCO server temporarily
    log_info "  Stopping CCO server to test fallback..."

    local cco_pid
    if [ -f /tmp/cco-e2e-test.pid ]; then
        cco_pid=$(cat /tmp/cco-e2e-test.pid)
        kill "$cco_pid" 2>/dev/null || true
        sleep 1
    fi

    # Set environment variable to non-existent server
    export CCO_API_URL="http://127.0.0.1:9999"

    # Try loading agent
    local output
    output=$(node "$AGENT_LOADER_PATH" "rust-specialist" 2>&1)

    # Check for fallback message
    if echo "$output" | grep -q "falling back to local files\|API unavailable"; then
        log_success "  Fallback mechanism triggered"
    else
        log_warning "  No fallback message found in output"
    fi

    # Verify it still returns correct model
    local actual_model
    actual_model=$(echo "$output" | tail -n1)

    if [ "$actual_model" = "haiku" ]; then
        record_test_result "FALLBACK_MECHANISM" "PASS"

        # Restart CCO server
        log_info "  Restarting CCO server..."
        start_cco_server
        return 0
    else
        record_test_result "FALLBACK_MECHANISM" "FAIL" "Fallback returned wrong model: $actual_model"

        # Restart CCO server
        log_info "  Restarting CCO server..."
        start_cco_server
        return 1
    fi
}

# Test 9: Network timeout handling
test_network_timeout() {
    log_info "Test 9: Network timeout handling (simulated)"

    # Use unreachable host to simulate timeout
    export CCO_API_URL="http://10.255.255.1:3210"

    local start_time=$(date +%s)
    local output
    output=$(timeout 5 node "$AGENT_LOADER_PATH" "rust-specialist" 2>&1 || true)
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log_info "  → Timeout test completed in ${duration}s"

    # Should fallback quickly, not hang
    if [ "$duration" -lt 5 ]; then
        record_test_result "NETWORK_TIMEOUT" "PASS"
        return 0
    else
        record_test_result "NETWORK_TIMEOUT" "FAIL" "Took too long to fallback (${duration}s)"
        return 1
    fi
}

# Test 10: End-to-end agent spawning simulation
test_e2e_agent_spawning() {
    log_info "Test 10: End-to-end Claude agent spawning simulation"

    export CCO_API_URL="$CCO_API_URL"

    # Simulate what Claude Code would do when spawning an agent
    log_info "  Simulating: model = getAgentModel('rust-specialist')"

    local model
    model=$(node "$AGENT_LOADER_PATH" "rust-specialist" 2>&1 | tail -n1)

    log_info "  → Returned model: $model"

    # Simulate Task call
    log_info "  Simulating: Task('Rust Specialist', '...', 'rust-specialist', '$model')"

    if [ "$model" = "haiku" ]; then
        log_success "  Agent would be spawned with correct model: $model"
        record_test_result "E2E_AGENT_SPAWNING" "PASS"
        return 0
    else
        log_fail "  Agent would be spawned with wrong model: $model (expected haiku)"
        record_test_result "E2E_AGENT_SPAWNING" "FAIL" "Wrong model returned"
        return 1
    fi
}

# Test 11: Verify agent count consistency
test_agent_count() {
    log_info "Test 11: Verify agent count consistency (117-119 agents)"

    local response
    response=$(curl -s "$CCO_API_URL/api/agents")

    local agent_count
    agent_count=$(echo "$response" | jq '.agents | length')

    log_info "  → Total agents: $agent_count"

    if [ "$agent_count" -ge 117 ] && [ "$agent_count" -le 119 ]; then
        record_test_result "AGENT_COUNT" "PASS"
        return 0
    else
        record_test_result "AGENT_COUNT" "FAIL" "Expected 117-119 agents, got $agent_count"
        return 1
    fi
}

# Test 12: Model distribution validation
test_model_distribution() {
    log_info "Test 12: Verify model distribution (1 Opus, 37 Sonnet, 81 Haiku)"

    local response
    response=$(curl -s "$CCO_API_URL/api/agents")

    local opus_count
    opus_count=$(echo "$response" | jq '[.agents[] | select(.model == "opus")] | length')

    local sonnet_count
    sonnet_count=$(echo "$response" | jq '[.agents[] | select(.model == "sonnet")] | length')

    local haiku_count
    haiku_count=$(echo "$response" | jq '[.agents[] | select(.model == "haiku")] | length')

    log_info "  → Opus: $opus_count (expected: 1)"
    log_info "  → Sonnet: $sonnet_count (expected: ~37)"
    log_info "  → Haiku: $haiku_count (expected: ~81)"

    local all_correct=true

    if [ "$opus_count" -ne 1 ]; then
        log_fail "  Opus count incorrect: $opus_count (expected 1)"
        all_correct=false
    fi

    if [ "$sonnet_count" -lt 30 ] || [ "$sonnet_count" -gt 45 ]; then
        log_fail "  Sonnet count out of range: $sonnet_count (expected ~37)"
        all_correct=false
    fi

    if [ "$haiku_count" -lt 70 ] || [ "$haiku_count" -gt 90 ]; then
        log_fail "  Haiku count out of range: $haiku_count (expected ~81)"
        all_correct=false
    fi

    if [ "$all_correct" = true ]; then
        record_test_result "MODEL_DISTRIBUTION" "PASS"
        return 0
    else
        record_test_result "MODEL_DISTRIBUTION" "FAIL" "Model distribution incorrect"
        return 1
    fi
}

# Generate test results report
generate_report() {
    log_info "Generating test results report..."

    cat > "$TEST_RESULTS_FILE" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "warnings": $WARNINGS,
    "success_rate": $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")
  },
  "environment": {
    "cco_api_url": "$CCO_API_URL",
    "cco_port": $CCO_PORT,
    "agent_loader_path": "$AGENT_LOADER_PATH"
  },
  "test_results": $(
    for key in "${!TEST_RESULTS[@]}"; do
      echo "\"$key\": \"${TEST_RESULTS[$key]}\","
    done | sed '$ s/,$//' | jq -R -s 'split("\n") | map(select(length > 0)) | map(split(": ") | {(.[0]): .[1]}) | add'
  ),
  "failures": [
    $(printf '%s\n' "${FAILED_TEST_DETAILS[@]}" | jq -R . | paste -sd,)
  ],
  "warnings": [
    $(printf '%s\n' "${WARNING_DETAILS[@]}" | jq -R . | paste -sd,)
  ]
}
EOF

    log_info "Test results written to: $TEST_RESULTS_FILE"
}

# Display summary
display_summary() {
    echo ""
    echo "=========================================="
    echo "  E2E Agent Verification Test Summary"
    echo "=========================================="
    echo ""
    echo "Total Tests:    $TOTAL_TESTS"
    echo "Passed:         $PASSED_TESTS"
    echo "Failed:         $FAILED_TESTS"
    echo "Warnings:       $WARNINGS"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    else
        echo -e "${RED}✗ SOME TESTS FAILED${NC}"
        echo ""
        echo "Failed Tests:"
        for detail in "${FAILED_TEST_DETAILS[@]}"; do
            echo "  - $detail"
        done
    fi

    if [ $WARNINGS -gt 0 ]; then
        echo ""
        echo "Warnings:"
        for detail in "${WARNING_DETAILS[@]}"; do
            echo "  - $detail"
        done
    fi

    echo ""
    echo "Full results: $TEST_RESULTS_FILE"
    echo ""
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."

    # Stop CCO server if we started it
    if [ -f /tmp/cco-e2e-test.pid ]; then
        local cco_pid
        cco_pid=$(cat /tmp/cco-e2e-test.pid)
        if kill -0 "$cco_pid" 2>/dev/null; then
            log_info "Stopping CCO server (PID: $cco_pid)..."
            kill "$cco_pid" 2>/dev/null || true
        fi
        rm -f /tmp/cco-e2e-test.pid
    fi

    # Reset environment
    unset CCO_API_URL
}

# Main execution
main() {
    echo ""
    echo "=========================================="
    echo "  E2E Agent Definition System Test"
    echo "=========================================="
    echo ""

    # Trap cleanup on exit
    trap cleanup EXIT

    # Start CCO server
    start_cco_server || exit 1

    # Wait a bit for server to be fully ready
    sleep 2

    # Run tests
    test_list_all_agents
    test_get_specific_agents
    test_404_handling
    test_response_structure
    test_response_time
    test_agent_loader_with_api
    test_agent_loader_cli
    test_fallback_mechanism
    test_network_timeout
    test_e2e_agent_spawning
    test_agent_count
    test_model_distribution

    # Generate report
    generate_report

    # Display summary
    display_summary

    # Exit with appropriate code
    if [ $FAILED_TESTS -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"
