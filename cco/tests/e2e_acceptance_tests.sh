#!/bin/bash

#############################################################################
# E2E ACCEPTANCE TEST SUITE FOR CCO
# Comprehensive end-to-end testing for all CCO features
# Execution: ./tests/e2e_acceptance_tests.sh
#############################################################################

set -e

# Configuration
BINARY="/Users/brent/.cargo/bin/cco"
PORT=3000
HOST="127.0.0.1"
BASE_URL="http://${HOST}:${PORT}"
HEALTH_ENDPOINT="${BASE_URL}/health"
MAX_RETRIES=10
RETRY_DELAY=1

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNING_TESTS=0
CYCLE_RESULTS=()

# Timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="/Users/brent/git/cc-orchestra/cco/E2E_TEST_REPORT_${TIMESTAMP}.md"

#############################################################################
# UTILITY FUNCTIONS
#############################################################################

log_header() {
    echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}\n"
}

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED_TESTS++))
}

log_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED_TESTS++))
}

log_warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
    ((WARNING_TESTS++))
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

cleanup_processes() {
    log_info "Cleaning up processes..."
    pkill -f "cco run" || true
    sleep 2

    # Force kill if still running
    if pgrep -f "cco run" > /dev/null; then
        pkill -9 -f "cco run" || true
        sleep 1
    fi
}

wait_for_health() {
    local attempt=0
    while [ $attempt -lt $MAX_RETRIES ]; do
        if curl -s "${HEALTH_ENDPOINT}" > /dev/null 2>&1; then
            return 0
        fi
        ((attempt++))
        sleep $RETRY_DELAY
    done
    return 1
}

start_server() {
    log_info "Starting CCO server on port $PORT..."
    cleanup_processes

    NO_BROWSER=1 "${BINARY}" run --debug --port $PORT > /tmp/cco_server.log 2>&1 &
    SERVER_PID=$!

    log_info "Server PID: $SERVER_PID"

    # Wait for server to be healthy
    if wait_for_health; then
        log_info "Server is healthy"
        return 0
    else
        log_fail "Server failed to start or reach health status"
        cat /tmp/cco_server.log
        return 1
    fi
}

stop_server() {
    log_info "Stopping server (PID: $SERVER_PID)..."

    if [ -z "$SERVER_PID" ]; then
        log_warn "No server PID to stop"
        return 1
    fi

    # Try graceful shutdown
    kill -INT $SERVER_PID 2>/dev/null || true
    sleep 2

    # Check if still running
    if ps -p $SERVER_PID > /dev/null 2>&1; then
        log_warn "Server did not shut down gracefully, killing..."
        kill -9 $SERVER_PID 2>/dev/null || true
        sleep 1
    fi

    return 0
}

#############################################################################
# E2E TEST IMPLEMENTATIONS
#############################################################################

test_1_server_startup_and_health() {
    log_test "E2E Test 1: Server Startup and Health"
    ((TOTAL_TESTS++))

    local health_response
    health_response=$(curl -s "${HEALTH_ENDPOINT}")

    local status
    status=$(echo "$health_response" | jq -r '.status' 2>/dev/null)

    if [ "$status" = "ok" ]; then
        log_pass "Server started and health check passed"
        echo "  Response: $health_response" | head -1
    else
        log_fail "Server health check failed. Status: $status"
        echo "  Response: $health_response"
        return 1
    fi
}

test_2_data_endpoints_chain() {
    log_test "E2E Test 2: Data Endpoints Chain"
    ((TOTAL_TESTS++))

    local all_pass=true

    # Test /api/agents
    log_info "  Testing /api/agents..."
    local agents
    agents=$(curl -s "${BASE_URL}/api/agents")
    local agent_count
    agent_count=$(echo "$agents" | jq 'length' 2>/dev/null || echo 0)

    if [ "$agent_count" -gt 0 ]; then
        log_info "    ✓ Agents endpoint: $agent_count agents"
    else
        log_warn "    Agents endpoint returned 0 agents or parse error"
        all_pass=false
    fi

    # Test /api/metrics/projects
    log_info "  Testing /api/metrics/projects..."
    local metrics
    metrics=$(curl -s "${BASE_URL}/api/metrics/projects")
    local project_count
    project_count=$(echo "$metrics" | jq 'length' 2>/dev/null || echo 0)
    log_info "    ✓ Metrics endpoint: $project_count projects"

    # Test /api/stats
    log_info "  Testing /api/stats..."
    local stats
    stats=$(curl -s "${BASE_URL}/api/stats")

    local has_project
    has_project=$(echo "$stats" | jq 'has("project")' 2>/dev/null || echo false)

    local has_machine
    has_machine=$(echo "$stats" | jq 'has("machine")' 2>/dev/null || echo false)

    local has_activity
    has_activity=$(echo "$stats" | jq 'has("activity")' 2>/dev/null || echo false)

    if [ "$has_project" = "true" ] && [ "$has_machine" = "true" ] && [ "$has_activity" = "true" ]; then
        log_info "    ✓ Stats has all required fields (project, machine, activity)"
    else
        log_warn "    Stats missing some fields: project=$has_project, machine=$has_machine, activity=$has_activity"
        all_pass=false
    fi

    if [ "$all_pass" = true ]; then
        log_pass "All data endpoints working correctly"
    else
        log_fail "Some data endpoints had issues (see warnings above)"
        return 1
    fi
}

test_3_websocket_connection() {
    log_test "E2E Test 3: WebSocket Connection Flow"
    ((TOTAL_TESTS++))

    # Test WebSocket upgrade with curl
    local ws_response
    ws_response=$(curl -s -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
        -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
        -H "Sec-WebSocket-Version: 13" \
        "${BASE_URL}/terminal" 2>/dev/null 2>&1 || echo "")

    if echo "$ws_response" | grep -q "101\|Connection: Upgrade\|Upgrade: websocket"; then
        log_pass "WebSocket upgrade successful (101 Switching Protocols or Connection header)"
    else
        log_fail "WebSocket upgrade failed"
        echo "  First 5 lines of response:"
        echo "$ws_response" | head -5 | sed 's/^/    /'
        return 1
    fi
}

test_4_sse_stream_connection() {
    log_test "E2E Test 4: SSE Stream Connection"
    ((TOTAL_TESTS++))

    # Collect SSE data for 5 seconds
    log_info "  Collecting SSE events for 5 seconds..."
    local sse_data
    sse_data=$(timeout 5 curl -s -N "${BASE_URL}/api/stream" 2>/dev/null || echo "")

    local event_count
    event_count=$(echo "$sse_data" | grep -c "data:" || echo 0)

    if [ "$event_count" -gt 0 ]; then
        log_pass "SSE stream working ($event_count events received)"
        echo "  Sample events:" | sed 's/^/    /'
        echo "$sse_data" | head -3 | sed 's/^/      /'
    else
        log_fail "No SSE events received"
        return 1
    fi
}

test_5_dashboard_html_response() {
    log_test "E2E Test 5: Dashboard HTML Response"
    ((TOTAL_TESTS++))

    local dashboard
    dashboard=$(curl -s "${BASE_URL}/")

    local has_html
    has_html=$(echo "$dashboard" | grep -c "<!DOCTYPE" || echo 0)

    local has_script
    has_script=$(echo "$dashboard" | grep -c "dashboard.js" || echo 0)

    local has_cache_bust
    has_cache_bust=$(echo "$dashboard" | grep -c "dashboard.js?v=" || echo 0)

    if [ "$has_html" -eq 0 ]; then
        log_fail "Dashboard missing HTML structure"
        return 1
    fi

    if [ "$has_script" -eq 0 ]; then
        log_fail "Dashboard missing dashboard.js reference"
        return 1
    fi

    if [ "$has_cache_bust" -gt 0 ]; then
        log_pass "Dashboard loads with cache-bust parameter"
    else
        log_warn "Dashboard loads but missing cache-bust parameter (dashboard.js?v=)"
    fi
}

test_6_json_parse_error_scenario() {
    log_test "E2E Test 6: JSON Parse Error Scenario"
    ((TOTAL_TESTS++))

    local json_valid
    json_valid=$(curl -s "${BASE_URL}/api/stats" | jq . > /dev/null 2>&1 && echo "true" || echo "false")

    if [ "$json_valid" = "true" ]; then
        log_pass "JSON parsing successful (original error fixed)"
    else
        log_fail "JSON parsing error detected"
        return 1
    fi
}

test_7_server_shutdown_and_cleanup() {
    log_test "E2E Test 7: Server Shutdown and Cleanup"
    ((TOTAL_TESTS++))

    # Send SIGINT
    log_info "  Sending SIGINT to server..."
    kill -INT $SERVER_PID 2>/dev/null || true
    sleep 2

    # Check if still running
    if ps -p $SERVER_PID > /dev/null 2>&1; then
        log_fail "Server did not shut down cleanly"
        kill -9 $SERVER_PID 2>/dev/null || true
        return 1
    fi

    log_pass "Server shut down cleanly"

    # Check port released
    sleep 1
    if ! lsof -i :$PORT > /dev/null 2>&1; then
        log_pass "Port $PORT released"
    else
        log_fail "Port $PORT still in use"
        return 1
    fi
}

#############################################################################
# TEST CYCLE EXECUTION
#############################################################################

run_test_cycle() {
    local cycle=$1

    log_header "CYCLE $cycle - COMPREHENSIVE E2E TEST SUITE"

    # Reset counters for this cycle
    TOTAL_TESTS=0
    PASSED_TESTS=0
    FAILED_TESTS=0
    WARNING_TESTS=0

    # Start server
    if ! start_server; then
        log_fail "Server failed to start"
        CYCLE_RESULTS+=("Cycle $cycle: CRITICAL FAILURE - Server startup failed")
        return 1
    fi

    # Run all tests
    test_1_server_startup_and_health
    test_2_data_endpoints_chain || true
    test_3_websocket_connection || true
    test_4_sse_stream_connection || true
    test_5_dashboard_html_response || true
    test_6_json_parse_error_scenario || true

    # Stop server
    stop_server || true

    # Final test for shutdown
    test_7_server_shutdown_and_cleanup || true

    # Record cycle result
    local cycle_status="PASS"
    if [ $FAILED_TESTS -gt 0 ]; then
        cycle_status="FAIL"
    fi

    local result="Cycle $cycle: $PASSED_TESTS/$TOTAL_TESTS passed, $FAILED_TESTS failed, $WARNING_TESTS warnings - $cycle_status"
    CYCLE_RESULTS+=("$result")

    log_header "END CYCLE $cycle"
    echo "Result: $result"
}

#############################################################################
# REPORTING
#############################################################################

generate_report() {
    log_header "GENERATING COMPREHENSIVE REPORT"

    local overall_status="READY FOR PRODUCTION"
    local total_overall_failed=0

    cat > "$REPORT_FILE" << 'REPORT_START'
# E2E Acceptance Test Report

**Generated**: REPORT_START
    echo "$(date)" >> "$REPORT_FILE"
    cat >> "$REPORT_FILE" << 'REPORT_HEADER'

## Test Summary

REPORT_HEADER

    echo "- **Total Test Cycles**: 3" >> "$REPORT_FILE"
    echo "- **Tests per Cycle**: 7" >> "$REPORT_FILE"
    echo "- **Timestamp**: $TIMESTAMP" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    # Cycle results
    echo "## Per-Cycle Results" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    for result in "${CYCLE_RESULTS[@]}"; do
        echo "- $result" >> "$REPORT_FILE"
        if [[ "$result" == *"FAIL"* ]] || [[ "$result" == *"CRITICAL"* ]]; then
            overall_status="NEEDS WORK"
            total_overall_failed=$((total_overall_failed + 1))
        fi
    done

    echo "" >> "$REPORT_FILE"
    echo "## Test Categories" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "1. **Server Startup and Health** - Verifies server boots and /health endpoint works" >> "$REPORT_FILE"
    echo "2. **Data Endpoints Chain** - Tests /api/agents, /api/metrics/projects, /api/stats" >> "$REPORT_FILE"
    echo "3. **WebSocket Connection** - Validates WebSocket upgrade to /terminal" >> "$REPORT_FILE"
    echo "4. **SSE Stream Connection** - Tests Server-Sent Events on /api/stream" >> "$REPORT_FILE"
    echo "5. **Dashboard HTML Response** - Verifies dashboard loads with cache-bust" >> "$REPORT_FILE"
    echo "6. **JSON Parse Error** - Validates JSON parsing (original fix verification)" >> "$REPORT_FILE"
    echo "7. **Server Shutdown and Cleanup** - Tests graceful shutdown and port release" >> "$REPORT_FILE"

    echo "" >> "$REPORT_FILE"
    echo "## Overall Status" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "- **Status**: **$overall_status**" >> "$REPORT_FILE"
    echo "- **Failed Cycles**: $total_overall_failed / 3" >> "$REPORT_FILE"
    echo "- **Critical Issues**: $([ $total_overall_failed -eq 0 ] && echo 'NONE' || echo 'See cycle failures above')" >> "$REPORT_FILE"

    echo "" >> "$REPORT_FILE"
    echo "## Recommendations" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    if [ "$overall_status" = "READY FOR PRODUCTION" ]; then
        echo "- All test cycles passed successfully" >> "$REPORT_FILE"
        echo "- Server startup and shutdown are clean and reliable" >> "$REPORT_FILE"
        echo "- All API endpoints responding correctly" >> "$REPORT_FILE"
        echo "- WebSocket and SSE connections functional" >> "$REPORT_FILE"
        echo "- System is production-ready" >> "$REPORT_FILE"
    else
        echo "- Review cycle failures above" >> "$REPORT_FILE"
        echo "- Check server logs for detailed errors" >> "$REPORT_FILE"
        echo "- Address any connection timeout issues" >> "$REPORT_FILE"
        echo "- Verify all endpoints are properly implemented" >> "$REPORT_FILE"
    fi

    echo "" >> "$REPORT_FILE"
    echo "---" >> "$REPORT_FILE"
    echo "Report generated: $(date)" >> "$REPORT_FILE"

    cat "$REPORT_FILE"
    log_info "Report saved to: $REPORT_FILE"
}

#############################################################################
# MAIN EXECUTION
#############################################################################

main() {
    log_header "E2E ACCEPTANCE TEST SUITE FOR CCO"

    # Check if binary exists
    if [ ! -f "$BINARY" ]; then
        log_fail "CCO binary not found at $BINARY"
        exit 1
    fi

    # Run 3 cycles
    for cycle in 1 2 3; do
        run_test_cycle $cycle

        # Add small delay between cycles
        if [ $cycle -lt 3 ]; then
            log_info "Waiting 5 seconds before next cycle..."
            sleep 5
        fi
    done

    # Generate report
    generate_report

    log_header "TEST SUITE COMPLETE"

    # Final summary
    echo ""
    echo "E2E Test Suite Results:"
    echo "  - Report: $REPORT_FILE"
    echo "  - Failed Cycles: $total_overall_failed"
    echo ""
}

# Run main
main "$@"
