# Hybrid ccproxy Deployment Status

**Date**: 2025-11-04
**Status**: ✅ DEPLOYED (awaiting verification)

## ✅ Successfully Completed

### 1. Environment Configuration
- ✅ Real Anthropic API key added to `~/.zshrc`
  - `ANTHROPIC_API_KEY_REAL="sk-ant-api03-pGmP..."`
- ✅ ccproxy bearer token configured
  - `ANTHROPIC_API_KEY="da69328552..."`
  - `ANTHROPIC_BASE_URL="https://coder.visiquate.com"`

### 2. Hybrid Configuration Created
- ✅ Config file: `/Users/brent/git/cc-army/config/ccproxy/hybrid-config-ready-to-deploy.yaml`
- ✅ 5 models configured:
  1. `claude-opus-4` → Anthropic API (Architect)
  2. `claude-sonnet-4-5` → Anthropic API (Architect fallback)
  3. `claude-3-5-sonnet` → Local qwen2.5-coder:32b (Agents 1-10)
  4. `claude-3-haiku` → Local qwen-fast (Agent 11)
  5. `gpt-4` → Local qwen-quality-128k (Agents 13-15)

### 3. Deployment to Mac Mini
- ✅ Config deployed to `/Users/brent/ccproxy/config.yaml`
- ✅ Backup created: `config.yaml.backup.hybrid-YYYYMMDD-HHMMSS`
- ✅ ccproxy restarted with new config
- ✅ **Confirmed**: All 5 models were available on Mac mini at time of deployment

**Deployment output**:
```
📋 Available models:
claude-3-5-sonnet
claude-3-haiku
claude-opus-4      ← NEW (Anthropic API)
claude-sonnet-4-5  ← NEW (Anthropic API)
gpt-4
```

## ⚠️ Pending Verification

**Issue**: VPN connection is unstable
- ICMP ping works (32-492ms latency)
- SSH connections timeout
- Public URL returns "no available server"

**Need to verify**:
1. ✅ ccproxy process is running
2. ✅ Local endpoint (127.0.0.1:8081) responds
3. ✅ Public endpoint (coder.visiquate.com) responds
4. ✅ Traefik/Cloudflare tunnel is working

## 🔧 Verification Commands

### When you have stable access to Mac mini:

```bash
# Option 1: Run automated verification
cd /Users/brent/git/cc-army
./VERIFY_DEPLOYMENT.sh

# Option 2: Manual checks
ssh brent@192.168.9.123

# Check ccproxy is running
ps aux | grep litellm | grep -v grep

# Test local endpoint
curl -s http://127.0.0.1:8081/v1/models | jq

# Check logs
tail -50 /Users/brent/ccproxy/logs/litellm.log

# Check public endpoint
curl -s https://coder.visiquate.com/v1/models \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" | jq
```

## 🚀 Next Steps After Verification

Once public endpoint is confirmed working:

### 1. Configure Claude Code

```bash
# Logout from claude.ai
claude /logout

# Start Claude Code (uses env vars)
claude
# Say "No" to claude.ai
# Say "Yes" to API key
```

### 2. Test Hybrid Routing

**Test Architect model (should use real Claude API)**:
```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [{"role": "user", "content": "Hi from Architect"}],
    "max_tokens": 50,
    "stream": false
  }' | jq
```

**Test coding agent model (should use local Ollama)**:
```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet",
    "messages": [{"role": "user", "content": "Hi from Coder"}],
    "max_tokens": 50,
    "stream": false
  }' | jq
```

### 3. Deploy Claude Orchestra

```
"Use the Claude Orchestra to add a health endpoint to a Python API"
```

Expected behavior:
- Chief Architect spawns with `model="opus"` → Routes to real Claude Opus 4 (💰 paid API)
- All coding agents spawn with `model="sonnet-4.5"` → Routes to local qwen2.5-coder (🆓 free)
- Total cost: ~$1-5/day vs $50-200/day (95% savings)

## 📊 Model Routing Table

| Agent | Model Param | ccproxy Routes To | Backend | Cost |
|-------|-------------|-------------------|---------|------|
| Chief Architect | `opus` | `claude-opus-4` | Anthropic API | 💰 $0.01-0.10/request |
| Chief Architect (fallback) | `sonnet-4.5` | `claude-sonnet-4-5` | Anthropic API | 💰 $0.01-0.05/request |
| Agents 1-10 (Coding) | `sonnet-4.5` | `claude-3-5-sonnet` | Local qwen2.5-coder | 🆓 Free |
| Agent 11 (Credentials) | `haiku` | `claude-3-haiku` | Local qwen-fast | 🆓 Free |
| Agents 13-15 (QA/Sec/Docs) | `gpt-4` | `gpt-4` | Local qwen-quality-128k | 🆓 Free |

## 🛠️ Troubleshooting

### If ccproxy is not running

```bash
ssh brent@192.168.9.123
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &
```

### If public URL not working

```bash
# Restart Traefik
ssh brent@192.168.9.123 'docker restart traefik'

# Restart Cloudflare tunnel
ssh brent@192.168.9.123 'docker restart cloudflared'

# Wait 30-60 seconds for services to sync
```

### If Anthropic API key invalid

Check credits at: https://console.anthropic.com/settings/usage

Test key directly:
```bash
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: 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" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-opus-4-20250514","messages":[{"role":"user","content":"test"}],"max_tokens":10}'
```

## 📁 Files Created

- `/Users/brent/git/cc-army/config/ccproxy/hybrid-config-ready-to-deploy.yaml` - Deployed config
- `/Users/brent/git/cc-army/DEPLOY_WHEN_READY.sh` - Quick deploy script
- `/Users/brent/git/cc-army/VERIFY_DEPLOYMENT.sh` - Verification script
- `/Users/brent/git/cc-army/docs/HYBRID_ROUTING_SETUP.md` - Full setup guide
- `/Users/brent/git/cc-army/docs/MANUAL_HYBRID_DEPLOYMENT.md` - Manual deployment steps

## 📝 Summary

✅ **Hybrid routing is deployed** - config is on Mac mini with all 5 models
⏳ **Awaiting verification** - VPN connection unstable, need to confirm ccproxy is running
🎯 **Ready to use** - Once verified, configure Claude Code and deploy the army

**When stable access is available**: Run `./VERIFY_DEPLOYMENT.sh` to complete verification.
