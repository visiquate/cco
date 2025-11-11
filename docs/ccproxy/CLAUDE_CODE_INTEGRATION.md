# CCProxy Integration with Claude Code Army

This document explains how to configure Claude Code's orchestra-config.json to use the ccproxy deployment.

---

## Connection Details

**Base URL**: `https://coder.visiquate.com/v1`
**API Key**: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`
**Format**: OpenAI ChatCompletion API

---

## Available Model Names

Use these model names in Claude Code's army configuration:

| Model Name | Maps To | Context | Speed | Best For |
|------------|---------|---------|-------|----------|
| `claude-3-5-sonnet` | qwen-fast (7B) | 32k tokens | ~50 tok/s | Quick tasks, documentation |
| `gpt-4` | qwen-quality-128k (32B) | 128k tokens | ~8-10 tok/s | Complex coding, architecture |
| `ollama/qwen-fast` | qwen-fast (7B) | 32k tokens | ~50 tok/s | Direct access to 7B model |
| `ollama/qwen-quality` | qwen-quality-128k (32B) | 128k tokens | ~8-10 tok/s | Direct access to 32B model |

**Note**: Two distinct models are now available:
- **qwen-fast (7B)**: Lightweight, fast responses (~50 tokens/sec), ideal for quick tasks and documentation
- **qwen-quality-128k (32B)**: High-quality output, larger context (128k tokens), perfect for complex coding and architecture (~8-10 tokens/sec)

---

## Model Selection Guide

Choose the right model for your task based on complexity and requirements:

### qwen-fast (7B) - Quick Tasks
- **Model**: qwen2.5-coder:7b-instruct
- **Context**: 32k tokens
- **Speed**: ~50 tokens/second
- **Best For**: Documentation, simple fixes, quick queries, code reviews, support tasks
- **Aliases**: `claude-3-5-sonnet`, `ollama/qwen-fast`

### qwen-quality-128k (32B) - Heavy Lifting ⭐
- **Model**: qwen2.5-coder:32b-instruct
- **Context**: 128k tokens
- **Speed**: ~8-10 tokens/second
- **Best For**: Complex coding, API integrations, production code, architecture decisions, multi-file refactoring
- **Aliases**: `gpt-4`, `ollama/qwen-quality`

**Decision Guide**:
- Use **7B (qwen-fast)** when: Response speed matters, task is simple, context is under 32k tokens
- Use **32B (qwen-quality)** when: Code quality is critical, large context needed, complex problem solving required

---

## Recommended orchestra-config.json Configuration

Update `/Users/brent/git/cc-orchestra/config/orchestra-config.json` with these settings:

```json
{
  "chief-architect": {
    "role": "Chief Architect",
    "model": "opus-4.1",
    "fallbackModel": "gpt-4",
    "baseUrl": "https://coder.visiquate.com/v1",
    "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
  },
  "coding-specialists": {
    "python-expert": {
      "model": "gpt-4",
      "baseUrl": "https://coder.visiquate.com/v1",
      "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
    },
    "swift-expert": {
      "model": "gpt-4",
      "baseUrl": "https://coder.visiquate.com/v1",
      "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
    },
    "go-expert": {
      "model": "gpt-4",
      "baseUrl": "https://coder.visiquate.com/v1",
      "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
    }
  },
  "support-agents": {
    "documentation": {
      "model": "claude-3-5-sonnet",
      "baseUrl": "https://coder.visiquate.com/v1",
      "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
    },
    "qa-engineer": {
      "model": "claude-3-5-sonnet",
      "baseUrl": "https://coder.visiquate.com/v1",
      "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
    }
  }
}
```

---

## Model Aliases for Different Tasks

Select the right model for your task type to optimize quality and performance:

### Architecture & Design
- **Model**: `gpt-4` (32B, qwen-quality-128k)
- **Use**: Chief Architect, system design, high-level decisions, complex problem decomposition
- **Why**: 128k context allows reviewing entire architecture, superior reasoning

### General Coding
- **Model**: `gpt-4` (32B, qwen-quality-128k)
- **Use**: Python, Swift, Go, Rust, Flutter experts - production code implementation
- **Why**: Code quality critical, larger context for multi-file implementations

### Support Tasks
- **Model**: `claude-3-5-sonnet` (7B, qwen-fast)
- **Use**: Documentation, QA, Security audits, support functions
- **Why**: Speed acceptable for these tasks, same quality output, reduces cost

### Direct Model Access
- **Model**: `ollama/qwen-quality` (32B)
- **Use**: When you explicitly need the high-quality 32B model
- **Why**: Guaranteed 32B model selection, bypasses alias mapping

---

## Usage Example

When spawning agents via Claude Code's Task tool, select models appropriately:

```javascript
// Chief Architect with 32B model for architecture decisions
Task(
  "Chief Architect",
  "Design system architecture for REST API with authentication",
  "system-architect",
  "opus-4.1",  // Primary model
  {
    fallbackModel: "gpt-4",  // 32B qwen-quality for complex reasoning
    baseUrl: "https://coder.visiquate.com/v1",
    apiKey: "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
  }
)

// Python Expert using 32B model for high-quality code
Task(
  "Python Expert",
  "Implement FastAPI with JWT authentication and database integration",
  "python-expert",
  "gpt-4",  // 32B qwen-quality for production code
  {
    baseUrl: "https://coder.visiquate.com/v1",
    apiKey: "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
  }
)

// Documentation agent using 7B model for speed
Task(
  "Documentation Lead",
  "Document API endpoints and authentication flow",
  "coder",
  "claude-3-5-sonnet",  // 7B qwen-fast for quick documentation
  {
    baseUrl: "https://coder.visiquate.com/v1",
    apiKey: "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
  }
)
```

---

## Testing the Integration

### Quick Test with cURL
```bash
# Test as Claude-3.5-Sonnet
curl -X POST "https://coder.visiquate.com/v1/chat/completions" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 50
  }'

# Test as GPT-4
curl -X POST "https://coder.visiquate.com/v1/chat/completions" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 50
  }'
```

### List Available Models
```bash
curl -X GET "https://coder.visiquate.com/v1/models" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
```

Expected response:
```json
{
  "data": [
    {"id": "claude-3-5-sonnet", "object": "model", "backend": "qwen-fast (7B)"},
    {"id": "gpt-4", "object": "model", "backend": "qwen-quality-128k (32B)"},
    {"id": "ollama/qwen-fast", "object": "model", "backend": "qwen2.5-coder:7b-instruct"},
    {"id": "ollama/qwen-quality", "object": "model", "backend": "qwen2.5-coder:32b-instruct"}
  ]
}
```

---

## OpenAI SDK Integration

If Claude Code uses OpenAI SDK internally:

```python
from openai import OpenAI

client = OpenAI(
    base_url="https://coder.visiquate.com/v1",
    api_key="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
)

# Quick task with 7B model
response_fast = client.chat.completions.create(
    model="claude-3-5-sonnet",  # 7B qwen-fast
    messages=[
        {"role": "system", "content": "You are a helpful coding assistant."},
        {"role": "user", "content": "Write a Python hello world"}
    ],
    max_tokens=150
)

# Complex task with 32B model
response_quality = client.chat.completions.create(
    model="gpt-4",  # 32B qwen-quality-128k
    messages=[
        {"role": "system", "content": "You are an expert software architect."},
        {"role": "user", "content": "Design a scalable microservices architecture"}
    ],
    max_tokens=2000
)

# Direct model access
response_direct = client.chat.completions.create(
    model="ollama/qwen-quality",  # Explicit 32B model
    messages=[
        {"role": "user", "content": "Implement a FastAPI REST API"}
    ],
    max_tokens=3000
)

print(response_quality.choices[0].message.content)
```

---

## Performance Expectations

| Metric | qwen-fast (7B) | qwen-quality (32B) | Notes |
|--------|--------|--------|-------|
| Response Time | 0.4-0.8s | 6-10s | Start vs end of response |
| Token Throughput | ~50 tok/s | ~8-10 tok/s | Tokens/second |
| First Token Latency | <100ms | 200-400ms | Time to first response |
| Context Window | 32k tokens | 128k tokens | Max input size |
| Memory Usage | ~7GB | ~32GB | Per-model overhead |
| Best For | Quick tasks | Production code | Task complexity |
| Cost Factor | 1x | 4x | Relative processing cost |

---

## Limitations

1. **Model Speed Tradeoff**: 32B model is slower (~8-10 tok/s) than 7B (~50 tok/s)
2. **No Anthropic Format**: Must use OpenAI format (`/v1/chat/completions`)
3. **Local Only**: Depends on local Ollama instance availability
4. **Memory Intensive**: 32B model requires ~32GB VRAM, 7B requires ~7GB
5. **Sequential Inference**: Single Ollama instance processes requests sequentially

---

## Future Enhancements

### Add Model Variety
```bash
# Pull additional models
ollama pull llama2
ollama pull mistral
ollama pull codellama
ollama pull mixtral
```

### Update config.yaml
```yaml
model_list:
  - model_name: claude-3-5-sonnet
    litellm_params:
      model: ollama/qwen-fast  # Fast, general

  - model_name: gpt-4
    litellm_params:
      model: ollama/llama2  # Conversational

  - model_name: coding
    litellm_params:
      model: ollama/codellama  # Code-specialized

  - model_name: architecture
    litellm_params:
      model: ollama/mixtral  # Reasoning tasks
```

Then map army agents to specialized models:
- Chief Architect → `architecture` (mixtral)
- Coding Specialists → `coding` (codellama)
- Documentation → `gpt-4` (llama2)
- QA/Security → `claude-3-5-sonnet` (qwen-fast)

---

## Troubleshooting

### "Invalid bearer token"
**Issue**: Wrong API key
**Fix**: Use exact token: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`

### "Invalid model name"
**Issue**: Unsupported model
**Fix**: Use one of: `claude-3-5-sonnet`, `gpt-4`, `ollama/qwen-fast`, `ollama/qwen-quality`

### "Internal Server Error"
**Issue**: Using Anthropic `/v1/messages` endpoint
**Fix**: Use OpenAI `/v1/chat/completions` format

### Slow responses
**Issue**: Using 32B model (normal behavior)
**Context**: 32B model produces ~8-10 tokens/sec (vs 7B at ~50 tokens/sec)
**Fix**: Use `claude-3-5-sonnet` (7B) for quick responses, `gpt-4` (32B) for quality

**Issue**: Ollama service overloaded
**Check**: `ps aux | grep ollama` and restart if needed

### Out of memory (OOM) errors
**Issue**: 32B model requires ~32GB VRAM
**Fix**: Ensure sufficient VRAM available, or use 7B model (`claude-3-5-sonnet`)
**Check**: `nvidia-smi` (NVIDIA GPU) or `free -h` (system RAM)

---

## Security Considerations

- ✅ Bearer token required (acts as API key)
- ✅ HTTPS encrypted external traffic
- ✅ LiteLLM bound to localhost only
- ⚠️ Bearer token provides full access (keep secure)
- ⚠️ No rate limiting currently

**Recommendation**: Store token in environment variables, not in config files:

```bash
export CCPROXY_API_KEY="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
```

Then reference in config:
```json
{
  "apiKey": "${CCPROXY_API_KEY}"
}
```

---

## References

- [Full Test Report](./TEST_REPORT.md) - Complete analysis and test results
- [Quick Reference](./QUICK_REFERENCE.md) - API usage examples
- [Summary](./SUMMARY.md) - Executive summary
- [LiteLLM Config](./config.yaml) - Current configuration

---

**Last Updated**: 2025-11-04
**Status**: ✅ Ready for integration (both models operational)
**Models**:
- qwen-fast (7B) - Quick tasks, ~50 tok/s
- qwen-quality-128k (32B) - Production code, ~8-10 tok/s
**Performance**:
- 7B: A-grade (0.4-0.8s avg response)
- 32B: A-grade (6-10s avg response, superior quality)
