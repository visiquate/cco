#!/bin/bash

###############################################################################
# Qwen Model Discovery & Verification Script
# Purpose: Identify all available Qwen models on coder.visiquate.com
# Date: 2025-11-04
###############################################################################

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
ENDPOINT="https://coder.visiquate.com"

echo "=================================================="
echo "Qwen Model Discovery & Verification"
echo "=================================================="
echo ""

###############################################################################
# Step 1: Check Server Connectivity
###############################################################################
echo -e "${BLUE}[1/5] Checking server connectivity...${NC}"
if curl -s -f "$ENDPOINT/api/tags" -H "Authorization: Bearer $BEARER_TOKEN" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server is online and accessible${NC}"
else
    echo -e "${RED}✗ Server is offline or unreachable${NC}"
    echo "Please verify:"
    echo "  1. Server is running on Mac mini"
    echo "  2. Ollama service is running: systemctl status ollama"
    echo "  3. Traefik is running and routing correctly"
    exit 1
fi
echo ""

###############################################################################
# Step 2: List All Available Models
###############################################################################
echo -e "${BLUE}[2/5] Listing all Qwen models...${NC}"

MODELS=$(curl -s -H "Authorization: Bearer $BEARER_TOKEN" \
  "$ENDPOINT/api/tags" | jq -r '.models[]? | select(.name | contains("qwen")) | .name' 2>/dev/null)

if [ -z "$MODELS" ]; then
    echo -e "${RED}✗ No Qwen models found or error querying API${NC}"
    exit 1
fi

echo -e "${GREEN}Found Qwen models:${NC}"
echo "$MODELS" | while read -r model; do
    echo "  • $model"
done
echo ""

###############################################################################
# Step 3: Get Detailed Model Information
###############################################################################
echo -e "${BLUE}[3/5] Getting detailed model information...${NC}"
echo ""

declare -A MODEL_INFO

echo "$MODELS" | while read -r model; do
    echo -e "${YELLOW}Model: $model${NC}"

    # Get model details
    DETAILS=$(curl -s -H "Authorization: Bearer $BEARER_TOKEN" \
      -X POST "$ENDPOINT/api/show" \
      -d "{\"name\": \"$model\"}" 2>/dev/null)

    # Extract size
    SIZE=$(echo "$DETAILS" | jq -r '.model_info.size // "unknown"' 2>/dev/null)
    if [ "$SIZE" != "null" ] && [ "$SIZE" != "unknown" ]; then
        SIZE_GB=$(echo "scale=2; $SIZE / 1024 / 1024 / 1024" | bc 2>/dev/null || echo "unknown")
        echo "  Size: ${SIZE_GB}GB"
    fi

    # Extract parameters
    PARAMS=$(echo "$DETAILS" | jq -r '.model_info.parameter_count // "unknown"' 2>/dev/null)
    if [ "$PARAMS" != "null" ] && [ "$PARAMS" != "unknown" ]; then
        PARAMS_B=$(echo "scale=1; $PARAMS / 1000000000" | bc 2>/dev/null || echo "unknown")
        echo "  Parameters: ${PARAMS_B}B"
    fi

    # Extract quantization
    QUANT=$(echo "$DETAILS" | jq -r '.details.quantization_level // .model_info.quantization_level // "unknown"' 2>/dev/null)
    echo "  Quantization: $QUANT"

    echo ""
done

###############################################################################
# Step 4: Identify The Three Models
###############################################################################
echo -e "${BLUE}[4/5] Identifying the three models...${NC}"
echo ""

HAS_7B=false
HAS_32B=false
HAS_128K=false

if echo "$MODELS" | grep -q "7b-instruct"; then
    echo -e "${GREEN}✓ Found: qwen-fast (7B)${NC}"
    echo "$MODELS" | grep "7b-instruct"
    HAS_7B=true
    echo ""
fi

if echo "$MODELS" | grep -q "32b-instruct" | grep -v "128k"; then
    echo -e "${GREEN}✓ Found: qwen-quality (32B)${NC}"
    echo "$MODELS" | grep "32b-instruct" | grep -v "128k"
    HAS_32B=true
    echo ""
fi

if echo "$MODELS" | grep -q "128k"; then
    echo -e "${GREEN}✓ Found: qwen-latest (32B-128k) ⭐ THE 3RD MODEL${NC}"
    echo "$MODELS" | grep "128k"
    HAS_128K=true
    echo ""
elif echo "$MODELS" | grep -q "q8"; then
    echo -e "${GREEN}✓ Found: qwen-quality-q8 (32B-Q8) ⭐ POSSIBLE 3RD MODEL${NC}"
    echo "$MODELS" | grep "q8"
    echo ""
fi

# Check for any other variants
OTHER=$(echo "$MODELS" | grep -v "7b-instruct" | grep -v "32b-instruct" | grep -v "128k" | grep -v "q8" || true)
if [ ! -z "$OTHER" ]; then
    echo -e "${YELLOW}⚠ Additional models found:${NC}"
    echo "$OTHER"
    echo ""
fi

###############################################################################
# Step 5: Test Each Model
###############################################################################
echo -e "${BLUE}[5/5] Testing models with sample prompts...${NC}"
echo ""

# Define test models
TEST_MODELS=(
    "qwen2.5-coder:7b-instruct"
    "qwen2.5-coder:32b-instruct"
    "qwen2.5-coder:32b-instruct-128k"
    "qwen2.5-coder:32b-instruct-q8"
)

for model in "${TEST_MODELS[@]}"; do
    # Skip if model doesn't exist
    if ! echo "$MODELS" | grep -q "$model"; then
        continue
    fi

    echo -e "${YELLOW}Testing: $model${NC}"

    # Test with simple prompt
    START_TIME=$(date +%s.%N)

    RESPONSE=$(curl -s -X POST "$ENDPOINT/api/generate" \
        -H "Authorization: Bearer $BEARER_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$model\",
            \"prompt\": \"Write a one-line Python function to add two numbers\",
            \"stream\": false
        }" 2>&1)

    END_TIME=$(date +%s.%N)
    ELAPSED=$(echo "$END_TIME - $START_TIME" | bc 2>/dev/null || echo "unknown")

    # Check if successful
    if echo "$RESPONSE" | jq -e '.response' > /dev/null 2>&1; then
        echo -e "  ${GREEN}✓ Working${NC}"
        echo "  Response time: ${ELAPSED}s"

        # Show first line of response
        FIRST_LINE=$(echo "$RESPONSE" | jq -r '.response' | head -1)
        echo "  Sample: $FIRST_LINE"
    else
        echo -e "  ${RED}✗ Failed${NC}"
        ERROR=$(echo "$RESPONSE" | jq -r '.error // "Unknown error"' 2>/dev/null || echo "Parse error")
        echo "  Error: $ERROR"
    fi

    echo ""
done

###############################################################################
# Summary
###############################################################################
echo "=================================================="
echo "SUMMARY"
echo "=================================================="
echo ""

if [ "$HAS_7B" = true ] && [ "$HAS_32B" = true ]; then
    echo -e "${GREEN}✓ Core models available:${NC}"
    echo "  1. qwen-fast (7B) - Speed"
    echo "  2. qwen-quality (32B) - Quality"
fi

if [ "$HAS_128K" = true ]; then
    echo -e "${GREEN}✓ THE 3RD MODEL FOUND:${NC}"
    echo "  3. qwen2.5-coder:32b-instruct-128k ⭐"
    echo "     - Extended 128k context window"
    echo "     - Best for large projects"
    echo "     - Newest/most capable"
elif [ ! -z "$OTHER" ]; then
    echo -e "${YELLOW}⚠ Possible 3rd model:${NC}"
    echo "$OTHER"
    echo "  (Not 128k variant, but may be newer)"
else
    echo -e "${YELLOW}⚠ 3rd model not found${NC}"
    echo "  Only 7B and 32B base models available"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Update config.yaml with correct model names"
echo "  2. See: docs/ccproxy/RECOMMENDED_CONFIG_UPDATE.yaml"
echo "  3. Use 128k model as default for complex coding"
echo ""

echo "=================================================="
echo "Verification complete!"
echo "=================================================="
