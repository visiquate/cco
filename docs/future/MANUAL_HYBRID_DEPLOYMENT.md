# Manual Hybrid Deployment Guide

## Status

**Mac mini (192.168.9.123) is currently offline or unreachable.**

Your hybrid configuration is ready to deploy when you have access to the Mac mini.

## Quick Deployment (When Mac Mini Accessible)

### Option A: Using the Pre-Built Config

```bash
# 1. Access the Mac mini (physical access, VNC, or working SSH)
# Then run:

# Copy config from this repo
scp /Users/brent/git/cc-orchestra/config/ccproxy/hybrid-config-ready-to-deploy.yaml brent@192.168.9.123:/tmp/

# SSH to Mac mini
ssh brent@192.168.9.123

# Backup current config
cp /Users/brent/ccproxy/config.yaml /Users/brent/ccproxy/config.yaml.backup.hybrid-$(date +%Y%m%d-%H%M%S)

# Deploy new config
mv /tmp/hybrid-config-ready-to-deploy.yaml /Users/brent/ccproxy/config.yaml

# Restart ccproxy
pkill -f litellm
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &

# Verify (wait 5 seconds first)
sleep 5
curl -s http://127.0.0.1:8081/v1/models | jq
```

### Option B: Direct File Edit (If Physical Access)

```bash
# 1. Access Mac mini physically or via Screen Sharing

# 2. Open Terminal on Mac mini

# 3. Backup config
cd /Users/brent/ccproxy
cp config.yaml config.yaml.backup.hybrid-$(date +%Y%m%d-%H%M%S)

# 4. Edit config
nano config.yaml

# 5. Add these two model entries at the TOP of model_list:

  - model_name: claude-opus-4
    litellm_params:
      model: anthropic/claude-opus-4-20250514
      api_key: $ANTHROPIC_API_KEY  # Set via environment variable
      max_tokens: 200000
      stream: true

  - model_name: claude-sonnet-4-5
    litellm_params:
      model: anthropic/claude-sonnet-4-20250514
      api_key: $ANTHROPIC_API_KEY  # Set via environment variable
      max_tokens: 200000
      stream: true

# 6. Save (Ctrl+O, Enter, Ctrl+X)

# 7. Restart ccproxy
pkill -f litellm
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &

# 8. Verify
sleep 5
curl -s http://127.0.0.1:8081/v1/models | jq
```

## Expected Output After Deployment

```json
{
  "data": [
    {"id": "claude-opus-4"},
    {"id": "claude-sonnet-4-5"},
    {"id": "claude-3-5-sonnet"},
    {"id": "claude-3-haiku"},
    {"id": "gpt-4"}
  ]
}
```

## Verification Steps

### 1. Check Models Endpoint (Local)

```bash
# On Mac mini
curl -s http://127.0.0.1:8081/v1/models | jq
```

### 2. Check Models Endpoint (Remote)

```bash
# From your Mac
curl -s https://coder.visiquate.com/v1/models -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" | jq
```

### 3. Test Architect Model (Should Use Real Claude API)

```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [{"role": "user", "content": "Say hi from Architect"}],
    "max_tokens": 100,
    "stream": false
  }' | jq -r '.choices[0].message.content'
```

Expected: Response from real Claude Opus 4

### 4. Test Coding Agent Model (Should Use Local Ollama)

```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet",
    "messages": [{"role": "user", "content": "Say hi from Coder"}],
    "max_tokens": 100,
    "stream": false
  }' | jq -r '.choices[0].message.content'
```

Expected: Response from local qwen2.5-coder

### 5. Monitor ccproxy Logs

```bash
# On Mac mini
tail -f /Users/brent/ccproxy/logs/litellm.log | grep -E 'anthropic|ollama|model'
```

Watch for:
- `anthropic/claude-opus-4` requests (Architect)
- `ollama/qwen2.5-coder` requests (Coding agents)

## Troubleshooting

### Mac Mini Not Responding

**Problem**: `ssh: connect to host 192.168.9.123 port 22: Operation timed out`

**Solutions**:
1. **Physical Access**: Use Mac mini directly with keyboard/mouse
2. **Screen Sharing**: Use macOS Screen Sharing (Finder → Go → Network)
3. **VPN**: Ensure VPN is connected and routes to local network
4. **Wake on LAN**: Mac mini might be sleeping

### ccproxy Not Running

**Problem**: `no available server` from Traefik

**Solution**:
```bash
# On Mac mini
ps aux | grep litellm

# If not running, start it
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &
```

### Anthropic API Key Invalid

**Problem**: `401 Unauthorized` when testing opus-4 model

**Solution**:
```bash
# Verify API key has credits
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: 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" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-opus-4-20250514","messages":[{"role":"user","content":"hi"}],"max_tokens":10}'

# Check response - should not be 401
```

## After Successful Deployment

### Configure Claude Code (One-Time)

```bash
# On your Mac
claude /logout
claude  # Say "No" to claude.ai, "Yes" to API key
```

### Test the Orchestra

```bash
# Start Claude Code
claude

# Test message
"Use the Claude Orchestra to add a /health endpoint to a Python API"
```

Expected behavior:
- Chief Architect spawns with `model="opus"` → Routes to real Claude Opus 4
- Coding agents spawn with `model="sonnet-4.5"` → Routes to local qwen2.5-coder
- All 15 agents coordinate via MCP and hooks

### Monitor Costs

Check Anthropic console: https://console.anthropic.com/settings/usage

Expected costs:
- **Architect**: ~$0.01-0.10 per request (strategic decisions only)
- **All other agents**: $0.00 (local Ollama)
- **Daily total**: ~$1-5 (vs $50-200 for all-API)

## Files Ready for Deployment

- **Config**: `/Users/brent/git/cc-orchestra/config/ccproxy/hybrid-config-ready-to-deploy.yaml`
- **Deploy Script**: `/Users/brent/git/cc-orchestra/scripts/deploy-hybrid-ccproxy.sh` (for SSH)
- **Environment**: API key already added to `~/.zshrc`

## Summary

✅ **Your .zshrc is configured** with:
- `ANTHROPIC_API_KEY_REAL` (for ccproxy pass-through to Anthropic)
- `ANTHROPIC_API_KEY` (ccproxy bearer token)
- `ANTHROPIC_BASE_URL` (points to ccproxy)

✅ **Hybrid config is ready** at:
- `/Users/brent/git/cc-orchestra/config/ccproxy/hybrid-config-ready-to-deploy.yaml`

⏳ **Waiting for Mac mini access** to deploy

Once Mac mini is accessible, run Option A or B above to deploy in ~2 minutes.
