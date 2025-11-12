# ccproxy Hybrid Routing Table

**STATUS: PLANNED - FUTURE HYBRID ROUTING CONFIGURATION**

## Future Configuration (When ccproxy is deployed)

This table shows the **intended routing configuration** when ccproxy hybrid routing is implemented. Currently, all requests use Claude API directly.

Last updated: 2025-11-04

| Model Alias | Backend | Provider | Cost | Use Case |
|-------------|---------|----------|------|----------|
| `claude-opus-4` | claude-opus-4-20250514 | Anthropic API | 💰 $0.015-0.075/1M tokens | **Chief Architect ONLY** |
| `claude-sonnet-4-5` | qwen-quality-128k:latest | Local Ollama | 🆓 **FREE** | **Normal Claude Code conversations** |
| `claude-3-5-sonnet` | qwen2.5-coder:32b-instruct | Local Ollama | 🆓 **FREE** | Coding agents (1-10) |
| `claude-3-haiku` | qwen-fast:latest | Local Ollama | 🆓 **FREE** | Credential Manager (11) |
| `gpt-4` | qwen-quality-128k:latest | Local Ollama | 🆓 **FREE** | QA/Security/Docs (13-15) |

## Cost Breakdown

### Paid Services (Anthropic API)
Only **Chief Architect** uses paid API:
- **Input**: $15 per 1M tokens
- **Output**: $75 per 1M tokens
- **Average request**: ~$0.01-0.02

**Expected daily cost**: $0.02-0.10 (for architect only)

### Free Services (Local Ollama)
Everything else is **100% FREE**:
- Normal Claude Code conversations
- All 14 coding agents
- QA, Security, Documentation agents

**Expected daily savings**: $50-200 vs all-Anthropic

## Savings Calculation

### Before (All Anthropic API)
- Normal conversations: $1-5/day
- Chief Architect: $0.02-0.10/day
- Coding agents: $30-100/day
- QA/Sec/Docs: $10-50/day
- **Total**: $41.02-155.10/day

### After (Hybrid Routing)
- Normal conversations: **$0** (qwen-quality)
- Chief Architect: $0.02-0.10/day (Anthropic)
- Coding agents: **$0** (qwen2.5-coder)
- QA/Sec/Docs: **$0** (qwen-quality)
- **Total**: $0.02-0.10/day

### Result
**Savings**: $41-155/day (99.5%+ reduction!)

## Model Specifications

### Anthropic API (Paid)

**claude-opus-4-20250514**:
- Context: ~200k tokens
- Output: Up to 32,000 tokens
- Quality: Highest (strategic decisions)
- Speed: Moderate
- Use: Chief Architect only

### Local Ollama (Free)

**qwen-quality-128k:latest** (32B params):
- Context: 128k tokens
- Output: Up to 131,072 tokens
- Quality: Very high (reasoning-focused)
- Speed: Fast (on Mac mini M2)
- Use: Normal conversations, QA, Security, Docs

**qwen2.5-coder:32b-instruct** (32B params):
- Context: 32k tokens
- Output: Up to 32,768 tokens
- Quality: High (coding-optimized)
- Speed: Very fast
- Use: All coding agents (TDD, Python, Swift, Go, Rust, Flutter, APIs, DevOps)

**qwen-fast:latest** (7B params):
- Context: 32k tokens
- Output: Up to 32,768 tokens
- Quality: Good (lightweight)
- Speed: Ultra-fast
- Use: Credential Manager only

## Memory Management

### Phase 1: Normal Operation
- **Loaded**: qwen-quality-128k (35GB) OR qwen2.5-coder (20GB) + qwen-fast (5GB)
- **Strategy**: Models auto-swap based on request pattern

### Phase 2: Orchestra Deployment
- **Initial**: qwen2.5-coder + qwen-fast loaded (25GB)
- **Later**: qwen-quality-128k loads, qwen2.5-coder unloads (35GB)

### Optimization
- Health checks **disabled** to prevent thrashing
- Models stay loaded until memory pressure
- Ollama handles auto-unload intelligently

## Configuration File

Location: `/Users/brent/ccproxy/config.yaml`

Key settings:
```yaml
general_settings:
  health_check_interval: 0  # Disabled
  health_check_timeout: 0   # Disabled

router_settings:
  disable_cooldowns: true
  allowed_fails: 999999
  cooldown_time: 0
```

## Monitoring

### Real-time Dashboard
```
https://coder.visiquate.com/dashboard/
```

### API Endpoint
```bash
curl -s https://coder.visiquate.com/dashboard/api/stats | jq
```

### Logs
```bash
ssh brent@192.168.9.123 'tail -f /Users/brent/ccproxy/logs/litellm.log'
```

## Testing Routing

### Test Normal Conversation (Should be FREE)
```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_tokens": 50,
    "stream": false
  }' | jq
```

Check logs - should see `ollama/qwen-quality-128k:latest`

### Test Chief Architect (Should be PAID)
```bash
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_tokens": 50,
    "stream": false
  }' | jq
```

Check logs - should see `anthropic/claude-opus-4-20250514`

## Related Documentation

- [Quick Start Guide](../QUICK_START_HYBRID.md)
- [Cost Tracking Dashboard](COST_TRACKING_DASHBOARD.md)
- [Orchestra Roster (TDD)](ORCHESTRA_ROSTER_TDD.md)
- [Deployment Status](../DEPLOYMENT_STATUS.md)
