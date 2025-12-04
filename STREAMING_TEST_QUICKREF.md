# LLM Gateway Streaming - Quick Reference

**Gateway Port**: 54548 (check with `cco daemon status`)

---

## Quick Test Commands

### 1. Check Daemon
```bash
cco daemon status
```

### 2. Test Health
```bash
curl http://127.0.0.1:54548/gateway/health | jq
```

### 3. Test Non-Streaming
```bash
curl -X POST http://127.0.0.1:54548/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 50,
    "stream": false,
    "messages": [{"role": "user", "content": "Say hello in 3 words"}]
  }' | jq
```

### 4. Test Streaming
```bash
curl -N -X POST http://127.0.0.1:54548/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 100,
    "stream": true,
    "messages": [{"role": "user", "content": "Count from 1 to 5"}]
  }'
```

### 5. Check Metrics
```bash
curl http://127.0.0.1:54548/gateway/metrics | jq
```

---

## Automated Tests

```bash
# Basic test (requires ANTHROPIC_API_KEY in env)
export ANTHROPIC_API_KEY=your-key
./scripts/test-gateway-streaming.sh

# Credential-aware test
./scripts/test-gateway-streaming-with-creds.sh
```

---

## Expected SSE Output

```
event: message_start
data: {"type":"message_start",...}

event: content_block_start
data: {"type":"content_block_start",...}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":"1"}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":"\n2"}}

event: content_block_stop
data: {"type":"content_block_stop",...}

event: message_stop
data: {"type":"message_stop"}
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Daemon not running | `cco daemon start` |
| 401 Unauthorized | Check `$ANTHROPIC_API_KEY` |
| Connection refused | Verify port with `cco daemon status` |
| No SSE events | Check `stream: true` in request |
| Buffered streaming | Use `curl -N` flag |

---

## Files

| File | Description |
|------|-------------|
| `scripts/test-gateway-streaming.sh` | Automated test |
| `docs/MANUAL_STREAMING_TEST.md` | Manual testing guide |
| `docs/GATEWAY_STREAMING_VERIFICATION.md` | Architecture review |
| `TEST_RESULTS_SUMMARY.md` | Test results summary |

---

## Status

✅ **Architecture Verified** - Ready for live testing
⏳ **Live Testing** - Pending API key availability
