# Hybrid Routing Setup Guide

## Overview

This guide configures ccproxy to route **Chief Architect** to real Claude API while routing all **coding agents** to local Ollama models.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│            Claude Code Orchestrator                     │
│         ANTHROPIC_BASE_URL=ccproxy                      │
│         ANTHROPIC_API_KEY=ccproxy-bearer-token          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │  ccproxy (Smart Router)    │
        │  https://coder.visiquate.com│
        └────────┬───────────────────┬┘
                 │                   │
     ┌───────────▼──────┐     ┌─────▼──────────────┐
     │  Anthropic API   │     │  Local Ollama      │
     │  (claude.api.com)│     │  (Mac mini)        │
     └──────────────────┘     └────────────────────┘
             │                          │
    ┌────────▼─────────┐      ┌────────▼────────────┐
    │ Chief Architect  │      │ All Coding Agents   │
    │ - opus-4         │      │ - qwen2.5-coder     │
    │ - sonnet-4.5     │      │ - qwen-fast         │
    └──────────────────┘      │ - qwen-quality-128k │
                              └─────────────────────┘
```

## Model Routing

| Model Name | Routes To | Used By | Cost |
|------------|-----------|---------|------|
| `claude-opus-4` | Anthropic API | Chief Architect | 💰 Paid |
| `claude-sonnet-4-5` | Anthropic API | Architect fallback | 💰 Paid |
| `claude-3-5-sonnet` | Local qwen2.5-coder | Agents 1-10 (Coding) | 🆓 Free |
| `claude-3-haiku` | Local qwen-fast | Agent 11 (Credentials) | 🆓 Free |
| `gpt-4` | Local qwen-quality-128k | Agents 13-15 (QA/Sec/Docs) | 🆓 Free |

## Prerequisites

1. **Real Anthropic API Key**: Get from https://console.anthropic.com
2. **ccproxy Bearer Token**: Your ccproxy authentication token
3. **Local Network Access**: SSH to Mac mini (192.168.9.123)

## Step-by-Step Setup

### 1. Store Real Anthropic API Key

Add to your `~/.zshrc`:

```bash
# Real Anthropic API key (for Architect via ccproxy pass-through)
export ANTHROPIC_API_KEY_REAL="sk-ant-api03-xxxxx"  # Replace with your real key

# ccproxy settings (for Claude Code)
export ANTHROPIC_API_KEY="your-ccproxy-bearer-token"  # ccproxy auth token
export ANTHROPIC_BASE_URL="https://coder.visiquate.com"
```

Reload your shell:
```bash
source ~/.zshrc
```

### 2. Deploy Hybrid ccproxy Configuration

```bash
cd /Users/brent/git/cc-army
./scripts/deploy-hybrid-ccproxy.sh
```

This will:
- ✅ Backup current ccproxy config
- ✅ Deploy hybrid routing config
- ✅ Inject your real Anthropic API key into config
- ✅ Restart ccproxy
- ✅ Verify all 5 models are available

### 3. Configure Claude Code

Logout from claude.ai:
```bash
claude /logout
```

When you run `claude` again:
1. Say **"No"** when asked to login to claude.ai
2. Say **"Yes"** when asked about API key
3. It will use ANTHROPIC_API_KEY (ccproxy token) and ANTHROPIC_BASE_URL

### 4. Verify Setup

Test the routing:

```bash
# Test Architect model (should route to Anthropic API)
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-opus-4","messages":[{"role":"user","content":"Hi from Architect"}],"max_tokens":100,"stream":false}' | jq

# Test coding agent model (should route to local Ollama)
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-3-5-sonnet","messages":[{"role":"user","content":"Hi from Coder"}],"max_tokens":100,"stream":false}' | jq
```

## Usage

Once configured, Claude Code will automatically route:

```javascript
// Chief Architect spawned with model="opus" or model="sonnet-4.5"
Task("Chief Architect", "Design system architecture", "system-architect", "opus")
// ↑ Routes to Anthropic API (💰 costs money)

// Coding agents spawned with model="sonnet-4.5"
Task("Python Expert", "Implement API", "python-expert", "sonnet-4.5")
// ↑ Maps to "claude-3-5-sonnet" → Routes to local qwen2.5-coder (🆓 free)

// Lightweight agent with model="haiku"
Task("Credential Manager", "Store secrets", "coder", "haiku")
// ↑ Maps to "claude-3-haiku" → Routes to local qwen-fast (🆓 free)

// Quality agents with model="gpt-4"
Task("QA Engineer", "Write tests", "test-automator", "gpt-4")
// ↑ Routes to local qwen-quality-128k (🆓 free)
```

## Cost Optimization

- **Architect only**: ~$0.01-0.10 per request (strategic decisions, infrequent)
- **All coding agents**: $0.00 (local Ollama models)
- **Estimated cost**: $1-5 per day (vs $50-200 for all Claude API)

## Troubleshooting

### Auth Conflict Error

If you see:
```
⚠Auth conflict: Both a token (claude.ai) and an API key (ANTHROPIC_API_KEY) are set
```

**Solution:**
```bash
claude /logout
unset ANTHROPIC_API_KEY_REAL  # Temporarily
claude  # Say "Yes" to API key
```

### ccproxy Returns 401 Unauthorized

**Problem**: Bearer token incorrect

**Solution:**
```bash
# Check your token
echo $ANTHROPIC_API_KEY

# Update .zshrc with correct token
# Reload shell
source ~/.zshrc
```

### Architect Requests Fail

**Problem**: Real Anthropic API key not injected or invalid

**Solution:**
```bash
# Verify real API key is set
echo $ANTHROPIC_API_KEY_REAL

# Redeploy config
cd /Users/brent/git/cc-army
./scripts/deploy-hybrid-ccproxy.sh
```

### Coding Agent Requests Fail

**Problem**: Ollama models not loaded

**Solution:**
```bash
# SSH to Mac mini
ssh brent@192.168.9.123

# Check Ollama status
curl -s http://localhost:11434/api/tags | jq

# Verify models exist
ollama list | grep -E 'qwen2.5-coder|qwen-fast|qwen-quality'
```

## Rollback

To revert to local-only routing:

```bash
ssh brent@192.168.9.123

# Find latest backup
ls -lt /Users/brent/ccproxy/config.yaml.backup* | head -1

# Restore it
cp /Users/brent/ccproxy/config.yaml.backup.YYYYMMDD-HHMMSS /Users/brent/ccproxy/config.yaml

# Restart ccproxy
pkill -f litellm
cd /Users/brent/ccproxy
source venv/bin/activate
nohup litellm --config config.yaml --port 8081 --host 127.0.0.1 > logs/litellm.log 2>&1 &
```

## Monitoring

Check ccproxy logs for routing:

```bash
ssh brent@192.168.9.123
tail -f /Users/brent/ccproxy/logs/litellm.log | grep -E 'anthropic|ollama'
```

You should see:
- `anthropic/claude-opus-4` requests for Architect
- `ollama/qwen2.5-coder` requests for coding agents

## Security Notes

1. **API Key Storage**: Real Anthropic API key is stored in ccproxy config on Mac mini
2. **Bearer Token**: ccproxy bearer token is stored in .zshrc
3. **TLS**: All communication uses HTTPS (Cloudflare tunnel)
4. **Access**: Only you can access ccproxy (bearer token auth)

## Next Steps

Once setup is complete:
1. ✅ Test with a simple army task: "Add a health endpoint to a Python API"
2. ✅ Verify Architect uses real Claude API (check ccproxy logs)
3. ✅ Verify coding agents use local Ollama (check ccproxy logs)
4. ✅ Monitor costs via Anthropic console (should be minimal)
5. ✅ Update orchestra-config.json if needed

## Summary

✅ **Architect**: Real Claude API (Opus 4.1 → Sonnet 4.5) for strategic decisions
✅ **All coding agents**: Local Ollama (qwen models) for implementation
✅ **Cost**: ~95% reduction vs all-Claude-API
✅ **Speed**: Same (local models are fast)
✅ **Quality**: Architect gets best model, coders get highly capable qwen2.5-coder
