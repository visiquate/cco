# Manual LLM Gateway Streaming Test

This document provides manual testing steps to verify the LLM Gateway's streaming functionality.

## Prerequisites

1. **Daemon must be running**:
   ```bash
   cco daemon status
   ```
   Note the port number (e.g., 54548)

2. **API key available**:
   - Set `ANTHROPIC_API_KEY` environment variable, OR
   - Store in credential system: `cco credentials store ANTHROPIC_API_KEY your-key`

## Quick Test (Automated)

Run the automated test script:

```bash
# With environment variable
export ANTHROPIC_API_KEY=your-key
./scripts/test-gateway-streaming.sh

# OR with credential system
./scripts/test-gateway-streaming-with-creds.sh
```

## Manual Testing Steps

### 1. Check Gateway Health

```bash
PORT=54548  # Replace with your actual port
curl http://127.0.0.1:$PORT/gateway/health | jq
```

Expected response:
```json
{
  "status": "healthy",
  "providers": {
    "anthropic": true
  },
  "routing": {
    "default_provider": "anthropic",
    "fallback_chain": ["anthropic", "azure", "deepseek", "ollama"]
  },
  "audit": {
    "enabled": true,
    "log_request_bodies": true,
    "log_response_bodies": true
  }
}
```

### 2. Test Non-Streaming Request

```bash
curl -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 50,
    "stream": false,
    "messages": [{"role": "user", "content": "Say hello in exactly 3 words"}]
  }' | jq
```

Expected response:
```json
{
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Hello there friend."
    }
  ],
  "model": "claude-sonnet-4-5-20250929",
  "usage": {
    "input_tokens": 15,
    "output_tokens": 8
  }
}
```

### 3. Test Streaming Request (SSE)

```bash
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
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

Expected output (SSE format):
```
event: message_start
data: {"type":"message_start","message":{"id":"msg_...","type":"message","role":"assistant",...}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"1"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"\n2"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"\n3"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"\n4"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"\n5"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":13}}

event: message_stop
data: {"type":"message_stop"}
```

### 4. Verify Streaming Performance

**Key indicators of proper streaming:**

1. **Content-Type header** should be `text/event-stream`
   ```bash
   curl -I -X POST http://127.0.0.1:$PORT/v1/messages \
     -H "Content-Type: application/json" \
     -H "x-api-key: $ANTHROPIC_API_KEY" \
     -H "anthropic-version: 2023-06-01" \
     -d '{"model":"claude-sonnet-4-5-20250929","max_tokens":50,"stream":true,"messages":[{"role":"user","content":"Hi"}]}'
   ```

2. **Incremental delivery** - Events should arrive progressively, not all at once
   - Use `-N` flag with curl to disable buffering
   - Watch for incremental output in real-time

3. **Proper SSE format** - Each event should have `event:` and `data:` lines

### 5. Check Metrics

```bash
curl http://127.0.0.1:$PORT/gateway/metrics | jq
```

Expected response:
```json
{
  "summary": {
    "total_requests": 3,
    "total_cost_usd": 0.0012,
    "total_input_tokens": 50,
    "total_output_tokens": 45,
    "total_cache_write_tokens": 0,
    "total_cache_read_tokens": 0,
    "avg_latency_ms": 1234.5
  },
  "by_provider": {
    "anthropic": 0.0012
  },
  "by_model": {
    "claude-sonnet-4-5-20250929": 0.0012
  },
  "recent": [
    {
      "request_id": "req_...",
      "timestamp": "2025-12-02T10:30:00Z",
      "provider": "anthropic",
      "model": "claude-sonnet-4-5-20250929",
      "input_tokens": 15,
      "output_tokens": 8,
      "cost_usd": 0.0004,
      "latency_ms": 1234
    }
  ]
}
```

### 6. Check Audit Logs

```bash
curl http://127.0.0.1:$PORT/gateway/audit?limit=5 | jq
```

Or via CLI:
```bash
cco daemon logs
```

## Troubleshooting

### Issue: "Daemon is not running"
**Solution**: Start the daemon
```bash
cco daemon start
```

### Issue: "401 Unauthorized"
**Solution**: Check API key
```bash
echo $ANTHROPIC_API_KEY
# OR
cco credentials retrieve ANTHROPIC_API_KEY
```

### Issue: "Connection refused"
**Solution**: Verify daemon port
```bash
cco daemon status
```

### Issue: No SSE events in streaming response
**Solution**:
1. Check that `stream: true` is in request body
2. Verify `Content-Type: text/event-stream` in response headers
3. Check daemon logs for errors: `cco daemon logs`

### Issue: Buffered streaming (all events arrive at once)
**Solution**:
1. Use `curl -N` to disable curl's buffering
2. Check for `x-accel-buffering: no` header in response (prevents nginx buffering)
3. Verify the provider implementation uses `reqwest::Bytes` streaming

## Advanced Testing

### Test with Different Models

```bash
# Opus (most expensive, highest quality)
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-opus-4-20250514",
    "max_tokens": 50,
    "stream": true,
    "messages": [{"role": "user", "content": "Explain quantum entanglement"}]
  }'

# Haiku (fastest, cheapest)
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-haiku-4-20250514",
    "max_tokens": 50,
    "stream": true,
    "messages": [{"role": "user", "content": "What is 2+2?"}]
  }'
```

### Test Error Handling

```bash
# Invalid API key
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: invalid-key" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 50,
    "stream": true,
    "messages": [{"role": "user", "content": "Hi"}]
  }'

# Invalid model
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "nonexistent-model",
    "max_tokens": 50,
    "stream": true,
    "messages": [{"role": "user", "content": "Hi"}]
  }'
```

### Test with Agent Types

```bash
# Route to specific agent type
curl -N -X POST http://127.0.0.1:$PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 50,
    "stream": true,
    "agent_type": "code-reviewer",
    "messages": [{"role": "user", "content": "Review this code: def add(a,b): return a+b"}]
  }'
```

## Success Criteria

The LLM Gateway streaming is working correctly if:

- ✅ Health endpoint returns `"status": "healthy"`
- ✅ Non-streaming requests return valid JSON responses
- ✅ Streaming requests have `Content-Type: text/event-stream`
- ✅ SSE events arrive incrementally (not buffered)
- ✅ SSE events follow Anthropic's format (message_start, content_block_delta, etc.)
- ✅ Metrics track request counts and costs
- ✅ Audit logs capture request/response data
- ✅ Error responses are properly formatted

## Next Steps

After verifying streaming works:

1. **Integration testing** - Test with actual Claude Code Task tool
2. **Load testing** - Test multiple concurrent streaming requests
3. **Provider fallback** - Test fallback when primary provider fails
4. **Cost tracking** - Verify cost calculations are accurate
5. **Security** - Test authentication and authorization
