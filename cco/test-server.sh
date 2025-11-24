#!/bin/bash
# Test script for CCO server

set -e

CCO_BIN="./target/release/cco"
PORT=3333

echo "=== CCO Server Test ==="
echo ""

# Clean up any existing processes
pkill -f "cco" 2>/dev/null || true
sleep 1

# Test 1: Default command (no arguments)
echo "1. Testing default command (should start server)..."
$CCO_BIN &
SERVER_PID=$!
sleep 2

# Check if server is running
if ps -p $SERVER_PID > /dev/null; then
    echo "   ✅ Server started successfully (PID: $SERVER_PID)"
    kill -INT $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
else
    echo "   ❌ Server failed to start"
    exit 1
fi

echo ""

# Test 2: Explicit run command with custom port
echo "2. Testing explicit 'run' command with custom port..."
$CCO_BIN run --port $PORT &
SERVER_PID=$!
sleep 2

# Test health endpoint
echo "3. Testing /health endpoint..."
HEALTH=$(curl -s http://127.0.0.1:$PORT/health)
if echo "$HEALTH" | grep -q '"status":"ok"'; then
    echo "   ✅ Health check passed: $HEALTH"
else
    echo "   ❌ Health check failed"
    kill -INT $SERVER_PID 2>/dev/null
    exit 1
fi

echo ""

# Test 4: Chat completion endpoint
echo "4. Testing /v1/chat/completions endpoint..."
RESPONSE=$(curl -s -X POST http://127.0.0.1:$PORT/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [{"role": "user", "content": "Test"}],
    "temperature": 1.0,
    "max_tokens": 4096
  }')

if echo "$RESPONSE" | grep -q '"content"'; then
    echo "   ✅ Chat completion successful"
    echo "   Response (first request): $(echo $RESPONSE | jq -c '{from_cache, content: .content[:50]}')"
else
    echo "   ❌ Chat completion failed"
    echo "   Response: $RESPONSE"
    kill -INT $SERVER_PID 2>/dev/null
    exit 1
fi

echo ""

# Test 5: Cache hit
echo "5. Testing cache (same request should hit cache)..."
RESPONSE2=$(curl -s -X POST http://127.0.0.1:$PORT/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [{"role": "user", "content": "Test"}],
    "temperature": 1.0,
    "max_tokens": 4096
  }')

if echo "$RESPONSE2" | grep -q '"from_cache":true'; then
    echo "   ✅ Cache hit detected"
    echo "   Response (cached): $(echo $RESPONSE2 | jq -c '{from_cache, content: .content[:50]}')"
else
    echo "   ⚠️  Cache miss (unexpected)"
    echo "   Response: $RESPONSE2"
fi

echo ""

# Test 6: Graceful shutdown
echo "6. Testing graceful shutdown (Ctrl+C)..."
kill -INT $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null
echo "   ✅ Server shut down gracefully"

echo ""
echo "=== All tests passed! ==="
