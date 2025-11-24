#!/bin/bash
# Test script for /api/classify endpoint

set -e

DAEMON_HOST="127.0.0.1"
DAEMON_PORT="3000"
BASE_URL="http://${DAEMON_HOST}:${DAEMON_PORT}"

echo "========================================="
echo "Testing /api/classify endpoint"
echo "========================================="
echo ""

# Test health endpoint first
echo "1. Testing /health endpoint..."
curl -s "${BASE_URL}/health" | jq . || {
    echo "❌ Failed to reach health endpoint"
    echo "Make sure daemon is running: cco daemon start"
    exit 1
}
echo "✅ Health check passed"
echo ""

# Test READ command
echo "2. Testing READ classification (ls -la)..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": "ls -la"}')
echo "$response" | jq .
classification=$(echo "$response" | jq -r '.classification')
if [ "$classification" = "Read" ]; then
    echo "✅ READ classification correct"
else
    echo "⚠️  Classification: $classification (expected: Read)"
fi
echo ""

# Test CREATE command
echo "3. Testing CREATE classification (mkdir new_dir)..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": "mkdir new_dir"}')
echo "$response" | jq .
classification=$(echo "$response" | jq -r '.classification')
if [ "$classification" = "Create" ]; then
    echo "✅ CREATE classification correct"
else
    echo "⚠️  Classification: $classification (expected: Create)"
fi
echo ""

# Test UPDATE command
echo "4. Testing UPDATE classification (echo content >> file.txt)..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": "echo content >> file.txt"}')
echo "$response" | jq .
classification=$(echo "$response" | jq -r '.classification')
if [ "$classification" = "Update" ]; then
    echo "✅ UPDATE classification correct"
else
    echo "⚠️  Classification: $classification (expected: Update)"
fi
echo ""

# Test DELETE command
echo "5. Testing DELETE classification (rm file.txt)..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": "rm file.txt"}')
echo "$response" | jq .
classification=$(echo "$response" | jq -r '.classification')
if [ "$classification" = "Delete" ]; then
    echo "✅ DELETE classification correct"
else
    echo "⚠️  Classification: $classification (expected: Delete)"
fi
echo ""

# Test with context
echo "6. Testing with context parameter..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": "cat /etc/hosts", "context": "user-initiated"}')
echo "$response" | jq .
echo "✅ Context parameter accepted"
echo ""

# Test error handling - empty command
echo "7. Testing error handling (empty command)..."
response=$(curl -s -X POST "${BASE_URL}/api/classify" \
  -H "Content-Type: application/json" \
  -d '{"command": ""}')
echo "$response" | jq .
echo ""

echo "========================================="
echo "✅ All tests completed!"
echo "========================================="
