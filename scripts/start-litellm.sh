#!/bin/bash
# Start LiteLLM proxy with OAuth passthrough for Claude Code
#
# This script:
# 1. Checks for LiteLLM installation
# 2. Creates data directory for cost tracking DB
# 3. Starts LiteLLM with Anthropic passthrough enabled
#
# Usage:
#   ./scripts/start-litellm.sh
#
# Then configure Claude Code:
#   export ANTHROPIC_BASE_URL="http://localhost:4000/anthropic"

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$PROJECT_DIR/config/litellm_oauth.yaml"
DATA_DIR="$PROJECT_DIR/data"
PORT="${LITELLM_PORT:-4000}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting LiteLLM OAuth Proxy${NC}"
echo "================================"

# Check if LiteLLM is installed
if ! command -v litellm &> /dev/null; then
    echo -e "${YELLOW}LiteLLM not found. Installing...${NC}"
    pip install 'litellm[proxy]'
fi

# Check for OAuth token
if [ -z "$CLAUDE_CODE_OAUTH_TOKEN" ]; then
    echo -e "${RED}Warning: CLAUDE_CODE_OAUTH_TOKEN not set${NC}"
    echo "OAuth passthrough may not work without this token."
    echo ""
fi

# Create data directory for SQLite DB
mkdir -p "$DATA_DIR"

# Set a default master key if not set
if [ -z "$LITELLM_MASTER_KEY" ]; then
    export LITELLM_MASTER_KEY="sk-cco-$(openssl rand -hex 16)"
    echo -e "${YELLOW}Generated temporary LITELLM_MASTER_KEY${NC}"
fi

echo ""
echo "Configuration:"
echo "  Config: $CONFIG_FILE"
echo "  Port: $PORT"
echo "  Data: $DATA_DIR"
echo ""
echo -e "${GREEN}Starting LiteLLM...${NC}"
echo ""
echo "To use with Claude Code, run:"
echo "  export ANTHROPIC_BASE_URL=\"http://localhost:$PORT/anthropic\""
echo ""

# Start LiteLLM
cd "$PROJECT_DIR"
litellm --config "$CONFIG_FILE" --port "$PORT" --detailed_debug
