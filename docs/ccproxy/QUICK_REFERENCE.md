# CCProxy Quick Reference Guide

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

This guide describes the planned ccproxy infrastructure. The Claude Orchestra **currently uses direct Anthropic Claude API**, not ccproxy.

---

## Planned Connection Details (For Future Deployment)

**URL**: `https://coder.visiquate.com`
**Bearer Token**: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`
**Endpoint**: `/v1/chat/completions` (OpenAI format)

---

## Available Models

| Model Alias | Actual Model | Use Case |
|-------------|--------------|----------|
| `claude-3-5-sonnet` | `ollama/qwen-fast` | General purpose |
| `gpt-4` | `ollama/qwen-fast` | General purpose |
| `ollama/qwen-fast` | `ollama/qwen-fast` | Direct access |

> **Note**: All aliases currently map to `qwen-fast`. See TEST_REPORT.md for configuration recommendations.

---

## Quick Start Examples

### cURL
```bash
curl -X POST "https://coder.visiquate.com/v1/chat/completions" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'
```

### Python (requests)
```python
import requests

response = requests.post(
    "https://coder.visiquate.com/v1/chat/completions",
    headers={
        "Authorization": "Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c",
        "Content-Type": "application/json"
    },
    json={
        "model": "claude-3-5-sonnet",
        "messages": [{"role": "user", "content": "Hello!"}],
        "max_tokens": 100
    }
)
print(response.json()["choices"][0]["message"]["content"])
```

### Python (OpenAI SDK)
```python
from openai import OpenAI

client = OpenAI(
    base_url="https://coder.visiquate.com/v1",
    api_key="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
)

response = client.chat.completions.create(
    model="claude-3-5-sonnet",
    messages=[{"role": "user", "content": "Hello!"}],
    max_tokens=100
)
print(response.choices[0].message.content)
```

### JavaScript/Node
```javascript
const response = await fetch("https://coder.visiquate.com/v1/chat/completions", {
  method: "POST",
  headers: {
    "Authorization": "Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c",
    "Content-Type": "application/json"
  },
  body: JSON.stringify({
    model: "claude-3-5-sonnet",
    messages: [{ role: "user", content: "Hello!" }],
    max_tokens: 100
  })
});

const data = await response.json();
console.log(data.choices[0].message.content);
```

---

## List Available Models

```bash
curl -X GET "https://coder.visiquate.com/v1/models" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
```

**Response**:
```json
{
  "data": [
    {"id": "claude-3-5-sonnet", "object": "model", "owned_by": "openai"},
    {"id": "gpt-4", "object": "model", "owned_by": "openai"},
    {"id": "ollama/qwen-fast", "object": "model", "owned_by": "openai"}
  ]
}
```

---

## Common Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `model` | string | Yes | Model name or alias |
| `messages` | array | Yes | Conversation history |
| `max_tokens` | integer | No | Max response length (default: unlimited) |
| `temperature` | float | No | Randomness (0-2, default: 1) |
| `top_p` | float | No | Nucleus sampling (0-1) |
| `stream` | boolean | No | Enable streaming (default: false) |
| `stop` | array | No | Stop sequences |

---

## Response Format

```json
{
  "id": "chatcmpl-xxx",
  "created": 1762273316,
  "model": "ollama/qwen-fast",
  "object": "chat.completion",
  "choices": [{
    "finish_reason": "stop",
    "index": 0,
    "message": {
      "content": "Response text here",
      "role": "assistant"
    }
  }],
  "usage": {
    "completion_tokens": 10,
    "prompt_tokens": 20,
    "total_tokens": 30
  }
}
```

---

## Error Responses

### 401 Unauthorized
```json
{"error": "Missing or invalid Authorization header"}
```
**Fix**: Include bearer token in `Authorization` header

### 403 Forbidden
```json
{"error": "Invalid bearer token"}
```
**Fix**: Use correct bearer token

### 400 Bad Request
```json
{
  "error": {
    "message": "400: {'error': 'completion: Invalid model name...'}",
    "type": "None",
    "code": "400"
  }
}
```
**Fix**: Use valid model name from `/v1/models` endpoint

---

## Performance

- **Average Response Time**: 0.48s
- **Token Throughput**: ~100 tokens/sec
- **Uptime**: Check `/health` endpoint (requires auth)

---

## Troubleshooting

### "Missing or invalid Authorization header"
- Ensure `Authorization: Bearer <token>` header is present
- Check token is exactly: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`

### "Invalid model name"
- Use one of: `claude-3-5-sonnet`, `gpt-4`, `ollama/qwen-fast`
- List available models: `GET /v1/models`

### "Internal Server Error"
- Use `/v1/chat/completions` endpoint (NOT `/v1/messages`)
- Ensure request body is valid JSON
- Check model name is correct

### Slow responses
- Normal: 0.4-0.8s for typical queries
- Larger requests may take longer
- Check Ollama service status if consistently slow

---

## Architecture

```
Client Request
    ↓ HTTPS
[Traefik] (coder.visiquate.com)
    ↓ Bearer token validation
[LiteLLM] (localhost:8081)
    ↓ Model routing
[Ollama] (localhost:11434)
    ↓
Response (OpenAI format)
```

---

## Security Notes

- ✅ HTTPS encrypted external traffic
- ✅ Bearer token required for all requests
- ✅ LiteLLM bound to localhost only
- ✅ Traefik handles external access control
- ⚠️ Bearer token provides full access (keep secure)

---

## Links

- **Full Test Report**: [TEST_REPORT.md](./TEST_REPORT.md)
- **Configuration**: [config.yaml](./config.yaml)
- **Traefik Config**: [traefik.yml](./traefik.yml)

---

**Last Updated**: 2025-11-04
