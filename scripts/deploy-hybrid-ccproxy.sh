#!/bin/bash

# Deploy Hybrid ccproxy Configuration
# Routes Architect to Claude API, all other agents to local Ollama

set -e

echo "🚀 Deploying Hybrid ccproxy Configuration"
echo ""

# Check if real Anthropic API key is available
if [ -z "$ANTHROPIC_API_KEY_REAL" ]; then
    echo "❌ ERROR: ANTHROPIC_API_KEY_REAL environment variable not set"
    echo ""
    echo "You need your real Anthropic API key for the Architect to use Claude API."
    echo "Add to your .zshrc:"
    echo ""
    echo "  export ANTHROPIC_API_KEY_REAL=\"sk-ant-api03-xxxxx\""
    echo ""
    exit 1
fi

# SSH details
SSH_HOST="brent@192.168.9.123"
REMOTE_CONFIG_PATH="/Users/brent/ccproxy/config.yaml"
LOCAL_CONFIG_PATH="/Users/brent/git/cc-army/config/ccproxy/ccproxy-config-hybrid.yaml"

echo "📋 Configuration:"
echo "  Local config: $LOCAL_CONFIG_PATH"
echo "  Remote config: $REMOTE_CONFIG_PATH"
echo "  SSH target: $SSH_HOST"
echo ""

# Backup current config
echo "💾 Creating backup of current config..."
ssh $SSH_HOST "cp $REMOTE_CONFIG_PATH ${REMOTE_CONFIG_PATH}.backup.hybrid-\$(date +%Y%m%d-%H%M%S)"

# Deploy new config (with environment variable substitution)
echo "📦 Deploying hybrid config..."
envsubst < "$LOCAL_CONFIG_PATH" | ssh $SSH_HOST "cat > $REMOTE_CONFIG_PATH"

# Restart ccproxy
echo "🔄 Restarting ccproxy..."
ssh $SSH_HOST "pkill -f litellm || true"
sleep 2
ssh $SSH_HOST "cd /Users/brent/ccproxy && source venv/bin/activate && nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &"

echo "⏳ Waiting for ccproxy to start..."
sleep 5

# Verify deployment
echo "✅ Verifying deployment..."
ssh $SSH_HOST 'curl -s http://127.0.0.1:8081/v1/models | jq'

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📝 Available models:"
echo "  - claude-opus-4       → Anthropic API (Architect)"
echo "  - claude-sonnet-4-5   → Anthropic API (Architect fallback)"
echo "  - claude-3-5-sonnet   → Local Ollama qwen2.5-coder (Agents 1-10)"
echo "  - claude-3-haiku      → Local Ollama qwen-fast (Agent 11)"
echo "  - gpt-4               → Local Ollama qwen-quality-128k (Agents 13-15)"
echo ""
echo "🎯 Next steps:"
echo "  1. Update .zshrc with ccproxy settings (see instructions below)"
echo "  2. Logout from claude.ai: claude /logout"
echo "  3. Start Claude Code: claude"
echo ""
