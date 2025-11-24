#!/bin/bash

###############################################################################
# Dashboard Acceptance Test Suite
#
# This script runs comprehensive acceptance tests for the CCO dashboard.
# It validates all API endpoints, WebSocket connectivity, and SSE streams.
#
# Prerequisites:
#   - Server running on http://127.0.0.1:3000
#   - curl and jq installed
#   - bash 4.0+
#
# Usage:
#   bash tests/dashboard-acceptance-tests.sh
###############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Server configuration
SERVER_URL="http://127.0.0.1:3000"
TIMEOUT=5

###############################################################################
# Utility Functions
###############################################################################

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_fail() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}[FAIL]${NC} $1"
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}================================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================================================${NC}"
}

validate_json() {
    local json_string="$1"
    if echo "$json_string" | jq . > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

check_status_code() {
    local status_code="$1"
    local expected="$2"

    if [ "$status_code" = "$expected" ]; then
        return 0
    else
        return 1
    fi
}

###############################################################################
# Test 1: Dashboard Loads Without Errors
###############################################################################

test_dashboard_loads() {
    log_section "Test 1: Dashboard Loads Without Errors"

    local response=$(curl -s -w "\n%{http_code}" "$SERVER_URL/")
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    # Check HTTP status
    if check_status_code "$http_code" "200"; then
        log_pass "HTTP status is 200"
    else
        log_fail "HTTP status is $http_code (expected 200)"
        return 1
    fi

    # Check for valid HTML
    if echo "$body" | grep -q "<body"; then
        log_pass "HTML contains <body> tag"
    else
        log_fail "HTML does not contain <body> tag"
        return 1
    fi

    # Check for script tags
    if echo "$body" | grep -q "<script"; then
        log_pass "HTML contains <script> tags"
    else
        log_fail "HTML does not contain <script> tags"
        return 1
    fi

    # Check for error messages
    if echo "$body" | grep -qi "error\|failed\|exception"; then
        log_fail "HTML contains error messages"
        return 1
    else
        log_pass "No error messages in HTML"
    fi

    return 0
}

###############################################################################
# Test 2: No JSON Parse Errors
###############################################################################

test_no_json_errors() {
    log_section "Test 2: No JSON Parse Errors"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    local response=$(curl -s "$SERVER_URL/")

    # Check for parse error message
    if echo "$response" | grep -q "Failed to load data: JSON parse error"; then
        log_fail "Found JSON parse error message in dashboard"
        return 1
    else
        log_pass "No JSON parse error message found"
    fi

    return 0
}

###############################################################################
# Test 3: Dashboard.js Loads Correctly
###############################################################################

test_dashboard_js_loads() {
    log_section "Test 3: Dashboard.js Loads Correctly"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    local response=$(curl -s "$SERVER_URL/")

    # Check for dashboard.js script tag
    if echo "$response" | grep -q "dashboard.js"; then
        log_pass "dashboard.js script tag found"
    else
        log_fail "dashboard.js script tag not found"
        return 1
    fi

    # Check for cache-bust parameter
    local script_tag=$(echo "$response" | grep -o 'src="[^"]*dashboard\.js[^"]*"' | head -1)

    if echo "$script_tag" | grep -q "?v="; then
        local version=$(echo "$script_tag" | grep -o "v=[^&\"]*" | cut -d= -f2)
        log_pass "Cache-bust parameter found: v=$version"
    else
        log_fail "Cache-bust parameter not found in dashboard.js script tag"
        log_info "Script tag: $script_tag"
        return 1
    fi

    return 0
}

###############################################################################
# Test 4: API Data Loads
###############################################################################

test_api_endpoints() {
    log_section "Test 4: API Data Loads"

    local endpoints=(
        "/api/agents"
        "/api/metrics/projects"
        "/api/stats"
    )

    local all_passed=true

    for endpoint in "${endpoints[@]}"; do
        TESTS_TOTAL=$((TESTS_TOTAL + 1))

        local response=$(curl -s -w "\n%{http_code}" "$SERVER_URL$endpoint")
        local http_code=$(echo "$response" | tail -n1)
        local body=$(echo "$response" | sed '$d')

        # Check HTTP status
        if [ "$http_code" = "200" ]; then
            log_pass "$endpoint returns HTTP 200"
        else
            log_fail "$endpoint returns HTTP $http_code (expected 200)"
            all_passed=false
            continue
        fi

        # Validate JSON
        if validate_json "$body"; then
            log_pass "$endpoint returns valid JSON"
        else
            log_fail "$endpoint does not return valid JSON"
            log_info "Response: ${body:0:200}..."
            all_passed=false
        fi
    done

    if [ "$all_passed" = true ]; then
        return 0
    else
        return 1
    fi
}

###############################################################################
# Test 5: WebSocket Terminal Works
###############################################################################

test_websocket_terminal() {
    log_section "Test 5: WebSocket Terminal Works"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    # Test WebSocket upgrade request
    local response=$(curl -s -i -N \
        -H "Connection: Upgrade" \
        -H "Upgrade: websocket" \
        -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
        -H "Sec-WebSocket-Version: 13" \
        "$SERVER_URL/terminal" 2>/dev/null | head -1)

    # Check for upgrade response (101) or error validation (400)
    if echo "$response" | grep -q "101\|400"; then
        local code=$(echo "$response" | grep -o "HTTP/[^ ]* [0-9]*" | awk '{print $2}')
        if [ "$code" = "101" ]; then
            log_pass "WebSocket upgrade successful (HTTP 101)"
        else
            log_pass "WebSocket endpoint reachable with validation (HTTP $code)"
        fi
        return 0
    else
        log_fail "WebSocket endpoint not responding correctly"
        log_info "Response: $response"
        return 1
    fi
}

###############################################################################
# Test 6: SSE Stream Works
###############################################################################

test_sse_stream() {
    log_section "Test 6: SSE Stream Works"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    # Test SSE stream with timeout
    local response=$(timeout $TIMEOUT curl -s -N "$SERVER_URL/api/stream" 2>/dev/null || true)

    if [ -z "$response" ]; then
        log_fail "SSE stream returned no data"
        return 1
    fi

    # Check for data: format
    if echo "$response" | grep -q "^data:"; then
        log_pass "SSE stream returns data events"
    else
        log_fail "SSE stream does not return data events"
        log_info "Response: ${response:0:200}..."
        return 1
    fi

    # Count events
    local event_count=$(echo "$response" | grep -c "^data:" || true)
    log_pass "SSE stream produced $event_count data events"

    # Validate at least one JSON object
    local first_event=$(echo "$response" | grep "^data:" | head -1 | sed 's/^data: //')

    if [ -n "$first_event" ]; then
        if validate_json "$first_event"; then
            log_pass "SSE event data is valid JSON"
            return 0
        else
            log_fail "SSE event data is not valid JSON"
            log_info "Event: $first_event"
            return 1
        fi
    fi

    return 0
}

###############################################################################
# Test 7: Full Feature Flow
###############################################################################

test_full_feature_flow() {
    log_section "Test 7: Full Feature Flow"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    local start_time=$(date +%s%N)
    local step_count=0

    # Step 1: Load dashboard
    log_info "Step 1: Loading dashboard..."
    local dashboard=$(curl -s -w "\n%{http_code}" "$SERVER_URL/")
    local http_code=$(echo "$dashboard" | tail -n1)

    if [ "$http_code" != "200" ]; then
        log_fail "Dashboard load failed (HTTP $http_code)"
        return 1
    fi
    step_count=$((step_count + 1))
    log_pass "Dashboard loaded"

    # Step 2: Fetch /api/stats
    log_info "Step 2: Fetching /api/stats..."
    local stats=$(curl -s -w "\n%{http_code}" "$SERVER_URL/api/stats")
    local http_code=$(echo "$stats" | tail -n1)
    local body=$(echo "$stats" | sed '$d')

    if [ "$http_code" != "200" ] || ! validate_json "$body"; then
        log_fail "/api/stats request failed (HTTP $http_code)"
        return 1
    fi
    step_count=$((step_count + 1))
    log_pass "API stats loaded"

    # Step 3: Fetch /api/agents
    log_info "Step 3: Fetching /api/agents..."
    local agents=$(curl -s -w "\n%{http_code}" "$SERVER_URL/api/agents")
    local http_code=$(echo "$agents" | tail -n1)
    local body=$(echo "$agents" | sed '$d')

    if [ "$http_code" != "200" ] || ! validate_json "$body"; then
        log_fail "/api/agents request failed (HTTP $http_code)"
        return 1
    fi
    step_count=$((step_count + 1))
    log_pass "API agents loaded"

    # Step 4: Fetch /api/metrics/projects
    log_info "Step 4: Fetching /api/metrics/projects..."
    local metrics=$(curl -s -w "\n%{http_code}" "$SERVER_URL/api/metrics/projects")
    local http_code=$(echo "$metrics" | tail -n1)
    local body=$(echo "$metrics" | sed '$d')

    if [ "$http_code" != "200" ] || ! validate_json "$body"; then
        log_fail "/api/metrics/projects request failed (HTTP $http_code)"
        return 1
    fi
    step_count=$((step_count + 1))
    log_pass "API metrics loaded"

    local end_time=$(date +%s%N)
    local elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    local elapsed_seconds=$(echo "scale=2; $elapsed_ms / 1000" | bc)

    log_pass "Full feature flow completed: $step_count/4 steps, ${elapsed_seconds}s"

    if (( $(echo "$elapsed_ms < 2000" | bc -l) )); then
        log_pass "Performance acceptable (< 2 seconds)"
    else
        log_fail "Performance slow (${elapsed_seconds}s > 2 seconds)"
        return 1
    fi

    return 0
}

###############################################################################
# Test 8: Error Scenarios
###############################################################################

test_error_scenarios() {
    log_section "Test 8: Error Scenarios"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    # Test invalid endpoint
    log_info "Testing invalid endpoint..."
    local invalid=$(curl -s -w "\n%{http_code}" "$SERVER_URL/api/invalid")
    local http_code=$(echo "$invalid" | tail -n1)

    if [ "$http_code" = "404" ] || [ "$http_code" = "500" ]; then
        log_pass "Invalid endpoint handled gracefully (HTTP $http_code)"
    else
        log_fail "Invalid endpoint returned unexpected status (HTTP $http_code)"
    fi

    # Test malformed POST request
    log_info "Testing malformed POST request..."
    local malformed=$(curl -s -w "\n%{http_code}" -X POST "$SERVER_URL/api/agents")
    local http_code=$(echo "$malformed" | tail -n1)

    if [ "$http_code" = "400" ] || [ "$http_code" = "404" ] || [ "$http_code" = "405" ]; then
        log_pass "Malformed request handled gracefully (HTTP $http_code)"
        return 0
    else
        log_fail "Malformed request returned unexpected status (HTTP $http_code)"
        return 1
    fi
}

###############################################################################
# Main Execution
###############################################################################

main() {
    echo ""
    echo "================================================================================"
    echo "Dashboard Acceptance Test Suite"
    echo "================================================================================"
    echo ""
    echo "Target: $SERVER_URL"
    echo "Time: $(date)"
    echo ""

    # Check if server is running
    if ! curl -s "$SERVER_URL/" > /dev/null 2>&1; then
        echo -e "${RED}[FATAL] Server is not responding at $SERVER_URL${NC}"
        echo "Please ensure the server is running:"
        echo "  - Start: cargo run --bin cco -- server --port 3000"
        echo "  - Test: curl http://127.0.0.1:3000/"
        exit 1
    fi

    # Run all tests
    test_dashboard_loads || true
    test_no_json_errors || true
    test_dashboard_js_loads || true
    test_api_endpoints || true
    test_websocket_terminal || true
    test_sse_stream || true
    test_full_feature_flow || true
    test_error_scenarios || true

    # Print summary
    log_section "Test Summary"

    echo ""
    echo "Total Tests:  $TESTS_TOTAL"
    echo -e "Passed:       ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed:       ${RED}$TESTS_FAILED${NC}"
    echo ""

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}================================================================================${NC}"
        echo -e "${GREEN}OVERALL VERDICT: READY FOR PRODUCTION${NC}"
        echo -e "${GREEN}================================================================================${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}================================================================================${NC}"
        echo -e "${RED}OVERALL VERDICT: NEEDS FIXES${NC}"
        echo -e "${RED}================================================================================${NC}"
        echo ""
        return 1
    fi
}

# Run main function
main
exit $?
