#!/bin/bash

# Fix public endpoint by restarting Traefik and Cloudflare tunnel
# Run this when you have stable access to Mac mini

echo "🔧 Fixing public endpoint (coder.visiquate.com)"
echo ""

# Test connectivity
if ! ssh -o ConnectTimeout=5 brent@192.168.9.123 "echo 'Connected'" 2>/dev/null; then
    echo "❌ Cannot reach Mac mini"
    echo ""
    echo "Try:"
    echo "  1. Check VPN connection is stable"
    echo "  2. Physical access to Mac mini"
    echo "  3. macOS Screen Sharing"
    exit 1
fi

echo "✅ Mac mini accessible"
echo ""

# Restart services
echo "🔄 Restarting Traefik..."
ssh brent@192.168.9.123 'docker restart traefik'

echo "🔄 Restarting Cloudflare tunnel..."
ssh brent@192.168.9.123 'docker restart cloudflared'

echo "⏳ Waiting 30 seconds for services to sync..."
sleep 30

# Test public endpoint
echo ""
echo "🧪 Testing public endpoint..."
RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" https://coder.visiquate.com/v1/models \
    -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c")

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Public endpoint WORKING!"
    echo ""
    echo "Available models:"
    echo "$RESPONSE" | grep -v "HTTP_CODE:" | jq -r '.data[].id' | sort
    echo ""
    echo "🎉 Hybrid routing is fully operational!"
    echo ""
    echo "Next steps:"
    echo "  1. claude /logout"
    echo "  2. claude (say 'No' to claude.ai, 'Yes' to API key)"
    echo "  3. Test: 'Use the Claude Army to add a health endpoint'"
else
    echo "❌ Public endpoint still not working (HTTP $HTTP_CODE)"
    echo ""
    echo "$RESPONSE" | grep -v "HTTP_CODE:"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check Docker containers: ssh brent@192.168.9.123 'docker ps'"
    echo "  2. Check Traefik logs: ssh brent@192.168.9.123 'docker logs traefik --tail 50'"
    echo "  3. Check Cloudflare logs: ssh brent@192.168.9.123 'docker logs cloudflared --tail 50'"
    echo "  4. Check ccproxy logs: ssh brent@192.168.9.123 'tail -50 /Users/brent/ccproxy/logs/litellm.log'"
fi
