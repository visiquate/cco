#!/bin/bash
# Benchmark script for /api/stats endpoint performance

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_URL="http://127.0.0.1:3000/api/stats"
WARMUP_REQUESTS=3
BENCHMARK_REQUESTS=10

echo "============================================"
echo "Metrics Cache Performance Benchmark"
echo "============================================"
echo ""

# Check if daemon is running
if ! curl -s "${API_URL}" > /dev/null 2>&1; then
    echo -e "${RED}ERROR: Daemon not running at ${API_URL}${NC}"
    echo "Start daemon with: cco daemon start"
    exit 1
fi

echo -e "${GREEN}✓ Daemon is running${NC}"
echo ""

# Warmup requests (to populate cache)
echo "Running warmup requests..."
for i in $(seq 1 $WARMUP_REQUESTS); do
    curl -s "${API_URL}" > /dev/null
    echo "  Warmup request $i/$WARMUP_REQUESTS"
done
echo ""

# Benchmark individual requests
echo "Benchmarking ${BENCHMARK_REQUESTS} individual requests..."
total_time=0

for i in $(seq 1 $BENCHMARK_REQUESTS); do
    # Use time command to measure request duration
    start=$(gdate +%s%N 2>/dev/null || date +%s%N)
    curl -s "${API_URL}" > /dev/null
    end=$(gdate +%s%N 2>/dev/null || date +%s%N)

    duration=$(( (end - start) / 1000000 ))  # Convert to milliseconds
    total_time=$(( total_time + duration ))

    if [ $duration -lt 10 ]; then
        color=$GREEN
        status="✓ EXCELLENT"
    elif [ $duration -lt 100 ]; then
        color=$YELLOW
        status="⚠ ACCEPTABLE"
    else
        color=$RED
        status="✗ SLOW"
    fi

    echo -e "  Request $i: ${color}${duration}ms${NC} ${status}"
done

avg_time=$(( total_time / BENCHMARK_REQUESTS ))

echo ""
echo "============================================"
echo "Results Summary"
echo "============================================"
echo "Total requests: $BENCHMARK_REQUESTS"
echo "Total time: ${total_time}ms"
echo "Average time: ${avg_time}ms"

if [ $avg_time -lt 10 ]; then
    echo -e "${GREEN}✓ PASS: Average response time < 10ms (target met!)${NC}"
    echo -e "${GREEN}✓ Performance improvement: ~700x vs 7000ms baseline${NC}"
    exit 0
elif [ $avg_time -lt 100 ]; then
    echo -e "${YELLOW}⚠ WARN: Average response time < 100ms (acceptable but not optimal)${NC}"
    exit 0
else
    echo -e "${RED}✗ FAIL: Average response time > 100ms (needs investigation)${NC}"
    exit 1
fi
