#!/bin/bash

#############################################################################
# COMPREHENSIVE SMOKE TEST - BOTH MODES
# Tests both MODE 1 (explicit server) and MODE 2 (TUI/daemon)
# Execution: ./tests/smoke_test_all_modes.sh
#############################################################################

set -e

# Configuration
BINARY="/Users/brent/.cargo/bin/cco"
MODE1_PORT=3101
MODE2_PORT=3000
HOST="127.0.0.1"
MAX_RETRIES=15
RETRY_DELAY=0.5

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
MODE1_PASSED=0
MODE1_FAILED=0
MODE2_PASSED=0
MODE2_FAILED=0

# Timing
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

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

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

cleanup_port() {
    local port=$1
    log_info "Cleaning up processes on port $port..."

    # Kill any process using the port
    if lsof -i :$port > /dev/null 2>&1; then
        local pids=$(lsof -i :$port | grep LISTEN | awk '{print $2}' | sort -u)
        for pid in $pids; do
            if [ ! -z "$pid" ]; then
                log_info "  Killing process $pid on port $port..."
                kill -9 $pid 2>/dev/null || true
            fi
        done
    fi

    sleep 1
}

wait_for_health() {
    local port=$1
    local max_attempts=${2:-$MAX_RETRIES}
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s "http://${HOST}:${port}/health" > /dev/null 2>&1; then
            return 0
        fi
        ((attempt++))
        sleep $RETRY_DELAY
    done
    return 1
}

port_is_released() {
    local port=$1
    # Check if port is still in use
    ! lsof -i :$port > /dev/null 2>&1
}

wait_for_port_release() {
    local port=$1
    local max_attempts=40  # 40 * 0.5s = 20 seconds
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if port_is_released $port; then
            return 0
        fi
        ((attempt++))
        sleep $RETRY_DELAY
    done
    return 1
}

test_health_endpoint() {
    local port=$1
    local mode=$2

    log_test "Testing /health endpoint on port $port ($mode)"
    ((TOTAL_TESTS++))

    local health_response
    health_response=$(curl -s "http://${HOST}:${port}/health")

    local status
    status=$(echo "$health_response" | jq -r '.status' 2>/dev/null)

    if [ "$status" = "ok" ]; then
        log_pass "Health endpoint returned ok status ($mode)"
        return 0
    else
        log_fail "Health endpoint failed. Status: $status Response: $health_response ($mode)"
        return 1
    fi
}

test_agents_endpoint() {
    local port=$1
    local mode=$2

    log_test "Testing /api/agents endpoint on port $port ($mode)"
    ((TOTAL_TESTS++))

    local agents
    agents=$(curl -s "http://${HOST}:${port}/api/agents")

    # Check if it's valid JSON array
    local is_array
    is_array=$(echo "$agents" | jq 'type' 2>/dev/null || echo "error")

    if [ "$is_array" = '"array"' ]; then
        local agent_count
        agent_count=$(echo "$agents" | jq 'length' 2>/dev/null || echo 0)
        log_pass "Agents endpoint returns valid JSON array with $agent_count agents ($mode)"
        return 0
    else
        log_fail "Agents endpoint did not return JSON array. Response: $agents ($mode)"
        return 1
    fi
}

test_stats_endpoint() {
    local port=$1
    local mode=$2

    log_test "Testing /api/stats endpoint on port $port ($mode)"
    ((TOTAL_TESTS++))

    local stats
    stats=$(curl -s "http://${HOST}:${port}/api/stats")

    # Check if it's valid JSON
    local is_json
    is_json=$(echo "$stats" | jq 'type' 2>/dev/null || echo "error")

    if [ "$is_json" != "error" ]; then
        log_pass "Stats endpoint returns valid JSON ($mode)"
        return 0
    else
        log_fail "Stats endpoint returned invalid JSON. Response: $stats ($mode)"
        return 1
    fi
}

test_metrics_endpoint() {
    local port=$1
    local mode=$2

    log_test "Testing /api/metrics/projects endpoint on port $port ($mode)"
    ((TOTAL_TESTS++))

    local metrics
    metrics=$(curl -s "http://${HOST}:${port}/api/metrics/projects")

    # Check if it's valid JSON
    local is_json
    is_json=$(echo "$metrics" | jq 'type' 2>/dev/null || echo "error")

    if [ "$is_json" != "error" ]; then
        log_pass "Metrics endpoint returns valid JSON ($mode)"
        return 0
    else
        log_fail "Metrics endpoint returned invalid JSON. Response: $metrics ($mode)"
        return 1
    fi
}

#############################################################################
# MODE 1: EXPLICIT SERVER MODE
#############################################################################

test_mode_1_explicit_server() {
    log_header "MODE 1: EXPLICIT SERVER (cco run --debug --port $MODE1_PORT)"

    MODE1_PASSED=0
    MODE1_FAILED=0

    # Clean up any existing process
    cleanup_port $MODE1_PORT

    # Start server
    log_info "Starting cco run --debug --port $MODE1_PORT..."
    NO_BROWSER=1 "${BINARY}" run --debug --port $MODE1_PORT > /tmp/cco_mode1.log 2>&1 &
    SERVER_PID=$!

    log_info "Server PID: $SERVER_PID"

    # Wait for server to be healthy
    if wait_for_health $MODE1_PORT; then
        log_pass "Server started and health check passed (MODE 1)"
        ((MODE1_PASSED++))
    else
        log_fail "Server failed to start or reach health status (MODE 1)"
        ((MODE1_FAILED++))
        cat /tmp/cco_mode1.log
        return 1
    fi

    sleep 1

    # Run endpoint tests
    test_health_endpoint $MODE1_PORT "MODE 1" && ((MODE1_PASSED++)) || ((MODE1_FAILED++))
    test_agents_endpoint $MODE1_PORT "MODE 1" && ((MODE1_PASSED++)) || ((MODE1_FAILED++))
    test_stats_endpoint $MODE1_PORT "MODE 1" && ((MODE1_PASSED++)) || ((MODE1_FAILED++))
    test_metrics_endpoint $MODE1_PORT "MODE 1" && ((MODE1_PASSED++)) || ((MODE1_FAILED++))

    # Test shutdown
    log_test "Testing server shutdown (MODE 1)"
    ((TOTAL_TESTS++))

    log_info "Sending SIGINT to server..."
    SHUTDOWN_START=$(date +%s)
    kill -INT $SERVER_PID 2>/dev/null || true
    sleep 1

    # Check if process is still running
    if ps -p $SERVER_PID > /dev/null 2>&1; then
        log_info "Process still running, sending SIGTERM..."
        kill -TERM $SERVER_PID 2>/dev/null || true
        sleep 1
    fi

    # Force kill if needed
    if ps -p $SERVER_PID > /dev/null 2>&1; then
        log_info "Process still running, force killing..."
        kill -9 $SERVER_PID 2>/dev/null || true
    fi

    SHUTDOWN_END=$(date +%s)
    SHUTDOWN_TIME=$((SHUTDOWN_END - SHUTDOWN_START))

    # Check port is released
    if wait_for_port_release $MODE1_PORT; then
        log_pass "Port $MODE1_PORT released after shutdown (${SHUTDOWN_TIME}s) (MODE 1)"
        ((MODE1_PASSED++))
    else
        log_fail "Port $MODE1_PORT still in use after shutdown (MODE 1)"
        ((MODE1_FAILED++))
        lsof -i :$MODE1_PORT || true
    fi

    # Give it a final cleanup
    cleanup_port $MODE1_PORT
}

#############################################################################
# MODE 2: TUI/DAEMON MODE
#############################################################################

test_mode_2_tui_daemon() {
    log_header "MODE 2: TUI/DAEMON MODE (cco with no arguments)"

    MODE2_PASSED=0
    MODE2_FAILED=0

    # Clean up any existing process
    cleanup_port $MODE2_PORT

    # Start daemon (will fall back to server mode on port 3000)
    log_info "Starting cco (will start daemon on port $MODE2_PORT)..."

    # The TUI mode will try to start a TUI, fail (headless environment), and fall back to daemon
    NO_BROWSER=1 timeout 3 "${BINARY}" > /tmp/cco_mode2.log 2>&1 &
    DAEMON_PID=$!

    # Give it time to start (TUI will fail, then daemon starts)
    sleep 2

    # Wait for daemon to be healthy on port 3000
    if wait_for_health $MODE2_PORT 20; then
        log_pass "Daemon started and health check passed (MODE 2)"
        ((MODE2_PASSED++))
    else
        log_fail "Daemon failed to start or reach health status (MODE 2)"
        ((MODE2_FAILED++))
        cat /tmp/cco_mode2.log

        # Try to kill any hanging process
        pkill -f "cco$" || true
        return 1
    fi

    sleep 1

    # Run endpoint tests
    test_health_endpoint $MODE2_PORT "MODE 2" && ((MODE2_PASSED++)) || ((MODE2_FAILED++))
    test_agents_endpoint $MODE2_PORT "MODE 2" && ((MODE2_PASSED++)) || ((MODE2_FAILED++))
    test_stats_endpoint $MODE2_PORT "MODE 2" && ((MODE2_PASSED++)) || ((MODE2_FAILED++))
    test_metrics_endpoint $MODE2_PORT "MODE 2" && ((MODE2_PASSED++)) || ((MODE2_FAILED++))

    # Test shutdown
    log_test "Testing daemon shutdown (MODE 2)"
    ((TOTAL_TESTS++))

    log_info "Killing daemon process..."
    SHUTDOWN_START=$(date +%s)

    # Kill any cco process
    pkill -f "cco$" || true
    sleep 1

    # Kill any remaining process on the port
    if lsof -i :$MODE2_PORT > /dev/null 2>&1; then
        local pids=$(lsof -i :$MODE2_PORT | grep LISTEN | awk '{print $2}' | sort -u)
        for pid in $pids; do
            if [ ! -z "$pid" ]; then
                kill -9 $pid 2>/dev/null || true
            fi
        done
    fi

    SHUTDOWN_END=$(date +%s)
    SHUTDOWN_TIME=$((SHUTDOWN_END - SHUTDOWN_START))

    # Check port is released
    if wait_for_port_release $MODE2_PORT; then
        log_pass "Port $MODE2_PORT released after shutdown (${SHUTDOWN_TIME}s) (MODE 2)"
        ((MODE2_PASSED++))
    else
        log_fail "Port $MODE2_PORT still in use after shutdown (MODE 2)"
        ((MODE2_FAILED++))
        lsof -i :$MODE2_PORT || true
    fi

    # Give it a final cleanup
    cleanup_port $MODE2_PORT
}

#############################################################################
# MAIN EXECUTION
#############################################################################

main() {
    log_header "COMPREHENSIVE SMOKE TEST - ALL MODES"

    # Check if binary exists
    if [ ! -f "$BINARY" ]; then
        log_fail "CCO binary not found at $BINARY"
        exit 1
    fi

    log_info "Binary: $BINARY"
    log_info "Timestamp: $TIMESTAMP"

    # Run MODE 1
    test_mode_1_explicit_server || true

    # Run MODE 2
    test_mode_2_tui_daemon || true

    # Final cleanup
    cleanup_port $MODE1_PORT
    cleanup_port $MODE2_PORT

    #############################################################################
    # RESULTS SUMMARY
    #############################################################################

    log_header "RESULTS SUMMARY"

    echo "MODE 1 (Explicit Server):"
    echo "  Passed: $MODE1_PASSED"
    echo "  Failed: $MODE1_FAILED"
    echo ""

    echo "MODE 2 (TUI/Daemon):"
    echo "  Passed: $MODE2_PASSED"
    echo "  Failed: $MODE2_FAILED"
    echo ""

    echo "Overall:"
    echo "  Total Tests: $TOTAL_TESTS"
    echo "  Total Passed: $PASSED_TESTS"
    echo "  Total Failed: $FAILED_TESTS"
    echo ""

    # Determine exit code
    if [ $FAILED_TESTS -eq 0 ] && [ $MODE1_FAILED -eq 0 ] && [ $MODE2_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ ALL SMOKE TESTS PASSED${NC}"
        echo ""
        exit 0
    else
        echo -e "${RED}✗ SOME SMOKE TESTS FAILED${NC}"
        if [ $MODE1_FAILED -gt 0 ]; then
            echo "  - MODE 1 (Explicit Server) had $MODE1_FAILED failures"
        fi
        if [ $MODE2_FAILED -gt 0 ]; then
            echo "  - MODE 2 (TUI/Daemon) had $MODE2_FAILED failures"
        fi
        echo ""
        exit 1
    fi
}

# Run main
main "$@"
