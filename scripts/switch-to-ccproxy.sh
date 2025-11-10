#!/bin/bash

# Switch Claude Code to use ccproxy (local Ollama)
# Use this when hitting claude.ai Max plan usage limits

echo "🔄 Switching Claude Code to ccproxy (local Ollama)"
echo ""

# Check if ccproxy bearer token is set
if [ -z "$CCPROXY_BEARER_TOKEN" ]; then
    echo "⚠️  WARNING: CCPROXY_BEARER_TOKEN not set in .zshrc"
    echo ""
    echo "Add to ~/.zshrc:"
    echo "  export CCPROXY_BEARER_TOKEN=\"your-ccproxy-bearer-token\""
    echo ""
    exit 1
fi

# Set environment variables for current session
export ANTHROPIC_API_KEY="$CCPROXY_BEARER_TOKEN"
export ANTHROPIC_BASE_URL="https://coder.visiquate.com"

echo "✅ Environment configured:"
echo "  ANTHROPIC_API_KEY: $ANTHROPIC_API_KEY"
echo "  ANTHROPIC_BASE_URL: $ANTHROPIC_BASE_URL"
echo ""
echo "🎯 Next steps:"
echo "  1. claude /logout"
echo "  2. claude (say 'No' to claude.ai, 'Yes' to API key)"
echo ""
echo "📝 To make permanent, add to ~/.zshrc:"
echo "  export ANTHROPIC_API_KEY=\"\$CCPROXY_BEARER_TOKEN\""
echo "  export ANTHROPIC_BASE_URL=\"https://coder.visiquate.com\""
echo ""
