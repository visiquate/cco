#!/bin/bash

# Verify and fix ccproxy hybrid deployment

echo "🔍 Verifying ccproxy hybrid deployment"
echo ""

# Test connectivity
if ! ssh -o ConnectTimeout=5 brent@192.168.9.123 "echo 'Connected'" 2>/dev/null; then
    echo "❌ Cannot reach Mac mini - check VPN connection"
    exit 1
fi

echo "✅ Mac mini accessible"
echo ""

# Check ccproxy status
echo "📡 Checking ccproxy status..."
if ssh brent@192.168.9.123 'ps aux | grep litellm | grep -v grep' > /dev/null 2>&1; then
    echo "✅ ccproxy is running"

    # Test local endpoint
    echo ""
    echo "🧪 Testing local endpoint..."
    ssh brent@192.168.9.123 'curl -s http://127.0.0.1:8081/v1/models | jq -r ".data[].id" | sort'
else
    echo "❌ ccproxy is NOT running - restarting..."
    ssh brent@192.168.9.123 << 'ENDSSH'
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &
sleep 5
ENDSSH
    echo "✅ ccproxy restarted"
fi

echo ""
echo "🌐 Testing public endpoint..."
RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" https://coder.visiquate.com/v1/models \
    -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c")

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Public endpoint working!"
    echo "$RESPONSE" | grep -v "HTTP_CODE:" | jq -r '.data[].id' | sort
else
    echo "❌ Public endpoint returning: HTTP $HTTP_CODE"
    echo "$RESPONSE" | grep -v "HTTP_CODE:"
    echo ""
    echo "🔧 Possible fixes:"
    echo "  1. Restart Traefik: ssh brent@192.168.9.123 'docker restart traefik'"
    echo "  2. Restart Cloudflare tunnel: ssh brent@192.168.9.123 'docker restart cloudflared'"
    echo "  3. Wait 30-60 seconds for services to sync"
fi

echo ""
echo "📋 ccproxy logs (last 20 lines):"
ssh brent@192.168.9.123 'tail -20 /Users/brent/ccproxy/logs/litellm.log'
