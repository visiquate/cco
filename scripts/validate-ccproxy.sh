#!/bin/bash

###############################################################################
# CCProxy Validation Script
# Tests the complete routing chain: Traefik → LiteLLM → Ollama
# Usage: ./validate-ccproxy.sh
###############################################################################

set -e

# Configuration
BASE_URL="https://coder.visiquate.com"
BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
MODELS=("claude-3-5-sonnet" "gpt-4" "ollama/qwen-fast")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

###############################################################################
# Helper Functions
###############################################################################

print_header() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
}

print_test() {
    echo -e "${YELLOW}Testing: $1${NC}"
}

pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    ((PASSED_TESTS++))
    ((TOTAL_TESTS++))
}

fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    ((FAILED_TESTS++))
    ((TOTAL_TESTS++))
}

warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

info() {
    echo -e "${BLUE}ℹ️  INFO${NC}: $1"
}

###############################################################################
# Test Functions
###############################################################################

test_model_list() {
    print_test "GET /v1/models endpoint"

    local response=$(curl -s -w "\n%{http_code}" -X GET \
        "${BASE_URL}/v1/models" \
        -H "Authorization: Bearer ${BEARER_TOKEN}")

    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 200 ]; then
        local model_count=$(echo "$body" | jq -r '.data | length')
        pass "Models endpoint returned $model_count models (HTTP $status)"
        info "Available models: $(echo "$body" | jq -r '.data[].id' | tr '\n' ', ' | sed 's/,$//')"
    else
        fail "Models endpoint returned HTTP $status"
        echo "$body"
    fi
}

test_auth_valid() {
    print_test "Valid bearer token authentication"

    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${BASE_URL}/v1/chat/completions" \
        -H "Authorization: Bearer ${BEARER_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "Say OK"}],
            "max_tokens": 5
        }')

    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 200 ]; then
        pass "Valid token accepted (HTTP $status)"
    else
        fail "Valid token rejected (HTTP $status)"
        echo "$body"
    fi
}

test_auth_invalid() {
    print_test "Invalid bearer token rejection"

    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${BASE_URL}/v1/chat/completions" \
        -H "Authorization: Bearer invalid_token_12345" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 5
        }')

    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 403 ]; then
        pass "Invalid token rejected with 403"
    else
        fail "Invalid token should return 403, got $status"
        echo "$body"
    fi
}

test_auth_missing() {
    print_test "Missing bearer token rejection"

    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${BASE_URL}/v1/chat/completions" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 5
        }')

    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 401 ]; then
        pass "Missing token rejected with 401"
    else
        fail "Missing token should return 401, got $status"
        echo "$body"
    fi
}

test_model_alias() {
    local model=$1
    print_test "Model alias: $model"

    local start_time=$(date +%s.%N)

    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${BASE_URL}/v1/chat/completions" \
        -H "Authorization: Bearer ${BEARER_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"${model}\",
            \"messages\": [{\"role\": \"user\", \"content\": \"Say test\"}],
            \"max_tokens\": 10
        }")

    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)

    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 200 ]; then
        local actual_model=$(echo "$body" | jq -r '.model')
        local content=$(echo "$body" | jq -r '.choices[0].message.content')
        pass "Model $model works (actual: $actual_model, ${duration}s)"
        info "Response: $content"
    else
        fail "Model $model failed (HTTP $status)"
        echo "$body"
    fi
}

test_response_quality() {
    print_test "Response quality and formatting"

    local response=$(curl -s -X POST \
        "${BASE_URL}/v1/chat/completions" \
        -H "Authorization: Bearer ${BEARER_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-3-5-sonnet",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is 2+2? Answer with just the number."}
            ],
            "max_tokens": 10
        }')

    local content=$(echo "$response" | jq -r '.choices[0].message.content')
    local tokens=$(echo "$response" | jq -r '.usage.total_tokens')

    if [[ "$content" =~ "4" ]]; then
        pass "Response quality good (got: '$content', tokens: $tokens)"
    else
        warn "Response may be unexpected: '$content'"
    fi
}

test_performance() {
    print_test "Performance consistency (5 requests)"

    local times=()
    local total_time=0

    for i in {1..5}; do
        local start_time=$(date +%s.%N)

        curl -s -o /dev/null -X POST \
            "${BASE_URL}/v1/chat/completions" \
            -H "Authorization: Bearer ${BEARER_TOKEN}" \
            -H "Content-Type: application/json" \
            -d "{
                \"model\": \"ollama/qwen-fast\",
                \"messages\": [{\"role\": \"user\", \"content\": \"Test $i\"}],
                \"max_tokens\": 10
            }"

        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc)
        times+=("$duration")
        total_time=$(echo "$total_time + $duration" | bc)

        echo -e "  Request $i: ${duration}s"
    done

    local avg_time=$(echo "scale=3; $total_time / 5" | bc)

    if (( $(echo "$avg_time < 1.0" | bc -l) )); then
        pass "Performance good (avg: ${avg_time}s)"
    else
        warn "Performance slower than expected (avg: ${avg_time}s)"
    fi
}

test_anthropic_endpoint() {
    print_test "Anthropic /v1/messages endpoint (expected to fail)"

    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${BASE_URL}/v1/messages" \
        -H "Authorization: Bearer ${BEARER_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 10
        }')

    local status=$(echo "$response" | tail -n 1)

    if [ "$status" -eq 500 ]; then
        warn "Anthropic endpoint not supported (expected, HTTP 500)"
        info "Use /v1/chat/completions instead"
    else
        info "Anthropic endpoint returned HTTP $status"
    fi
}

###############################################################################
# Main Test Suite
###############################################################################

main() {
    print_header "CCProxy Validation Suite"
    echo "Base URL: $BASE_URL"
    echo "Testing: Authentication, Models, Performance"
    echo ""

    # Authentication tests
    print_header "Authentication Tests"
    test_auth_valid
    test_auth_invalid
    test_auth_missing
    echo ""

    # Model tests
    print_header "Model Tests"
    test_model_list
    for model in "${MODELS[@]}"; do
        test_model_alias "$model"
    done
    echo ""

    # Functionality tests
    print_header "Functionality Tests"
    test_response_quality
    echo ""

    # Performance tests
    print_header "Performance Tests"
    test_performance
    echo ""

    # Known limitations
    print_header "Known Limitations"
    test_anthropic_endpoint
    echo ""

    # Summary
    print_header "Test Summary"
    echo -e "Total Tests:  ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed:       ${PASSED_TESTS}${NC}"
    echo -e "${RED}Failed:       ${FAILED_TESTS}${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
        echo -e "${GREEN}CCProxy is READY FOR USE${NC}"
        exit 0
    else
        echo -e "${RED}❌ SOME TESTS FAILED${NC}"
        echo -e "${YELLOW}Review failures above${NC}"
        exit 1
    fi
}

# Run tests
main
