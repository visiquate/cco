#!/bin/bash

# Quick Hybrid ccproxy Deployment Script
# Run this when Mac mini is accessible

set -e

echo "🚀 Hybrid ccproxy Quick Deploy"
echo ""

# Test connectivity first
echo "📡 Testing Mac mini connectivity..."
if ! ssh -o ConnectTimeout=5 brent@192.168.9.123 "echo 'Connected'" 2>/dev/null; then
    echo "❌ Cannot reach Mac mini at 192.168.9.123"
    echo ""
    echo "Options:"
    echo "  1. Ensure VPN is connected"
    echo "  2. Check if Mac mini is powered on"
    echo "  3. Try physical access to Mac mini"
    echo "  4. Use macOS Screen Sharing (Finder → Go → Network)"
    echo ""
    exit 1
fi

echo "✅ Mac mini accessible"
echo ""

# Copy config
echo "📦 Copying hybrid config to Mac mini..."
scp /Users/brent/git/cc-orchestra/config/ccproxy/hybrid-config-ready-to-deploy.yaml brent@192.168.9.123:/tmp/

# Deploy via SSH
echo "🔧 Deploying config..."
ssh brent@192.168.9.123 << 'ENDSSH'
# Backup current config
cp /Users/brent/ccproxy/config.yaml /Users/brent/ccproxy/config.yaml.backup.hybrid-$(date +%Y%m%d-%H%M%S)

# Deploy new config
mv /tmp/hybrid-config-ready-to-deploy.yaml /Users/brent/ccproxy/config.yaml

# Restart ccproxy
pkill -f litellm || true
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &

# Wait for startup
sleep 5

# Verify
echo ""
echo "📋 Available models:"
curl -s http://127.0.0.1:8081/v1/models | jq -r '.data[].id' | sort
ENDSSH

echo ""
echo "✅ Deployment complete!"
echo ""
echo "🧪 Testing from your Mac..."
curl -s https://coder.visiquate.com/v1/models \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" | jq -r '.data[].id' | sort

echo ""
echo "📝 Next steps:"
echo "  1. claude /logout"
echo "  2. claude (say 'No' to claude.ai, 'Yes' to API key)"
echo "  3. Test with: 'Use the Claude Orchestra to add a health endpoint'"
echo ""
