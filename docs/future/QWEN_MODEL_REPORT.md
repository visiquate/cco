# Qwen 2.5 Coder Model Report

**STATUS: PLANNED - FUTURE LOCAL LLM DEPLOYMENT**

## Executive Summary

Based on future deployment plans, the Claude Orchestra **will use two Qwen 2.5 Coder models** for heavy coding tasks when ccproxy is deployed. The 32B model will be the high-quality option for complex work. **This is NOT currently in use - the system uses Claude API exclusively today.**

---

## Available Models

### 1. qwen-fast (7B Model)
- **Full Model Name**: `qwen2.5-coder:7b-instruct`
- **Parameters**: 7 billion
- **Context Window**: 32k tokens
- **Quantization**: Standard (likely Q4_K_M)
- **Speed**: ~50 tokens/second
- **Size**: ~4-5GB
- **Use Cases**:
  - Quick implementations
  - Simple functions
  - Bug fixes
  - Rapid prototyping
  - Documentation generation

### 2. qwen-quality-128k (32B Model) ⭐ HIGH QUALITY
- **Full Model Name**: `qwen2.5-coder:32b-instruct-q8` or `qwen2.5-coder:32b-instruct-128k`
- **Aliases**:
  - `qwen-quality-128k`
  - `qwen-quality`
  - `qwen2.5-coder:32b-instruct`
- **Parameters**: 32 billion
- **Context Window**: 128k tokens
- **Quantization**: Q8_0 (8-bit quantization)
- **Speed**: ~20 tokens/second
- **Size**: ~20-25GB
- **Use Cases**:
  - Complex algorithms
  - Full-stack features
  - Production-quality code
  - Architecture design
  - System integration
  - Security implementations

---

## Model Routing Configuration

### In `config/orchestra-config.json`

```json
{
  "llmRouting": {
    "enabled": true,
    "endpoints": {
      "coding": {
        "enabled": true,
        "type": "ollama",
        "url": "https://coder.visiquate.com",
        "defaultModel": "qwen2.5-coder:32b-instruct",
        "temperature": 0.7,
        "maxTokens": 4096,
        "headers": {},
        "additionalParams": {}
      }
    },
    "rules": {
      "architectureTasks": "claude",
      "codingTasks": "custom-if-enabled",
      "fallbackToClaude": true
    }
  }
}
```

**Key Points**:
- **Default model is the 32B version** for coding tasks
- Falls back to Claude for architecture tasks
- Falls back to Claude if Ollama is unavailable

---

## Access Methods

### 1. Direct Ollama API (Recommended for Testing)

```bash
# Test 32B model
curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:32b-instruct",
    "prompt": "Write a Python function to calculate fibonacci",
    "stream": false
  }'

# Test with alias
curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-quality-128k:latest",
    "prompt": "Write a Python function to calculate fibonacci",
    "stream": false
  }'
```

### 2. Via ccproxy (LiteLLM)

**Expected Configuration** (from CCPROXY_DEPLOYMENT_MISSION.md):

```yaml
model_list:
  # Fast 7B model
  - model_name: ollama/qwen-fast
    litellm_params:
      model: ollama/qwen2.5-coder:7b-instruct
      api_base: http://host.docker.internal:11434
      max_tokens: 32768
      temperature: 0.7

  # Quality 32B model
  - model_name: ollama/qwen-quality
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct-128k
      api_base: http://host.docker.internal:11434
      max_tokens: 131072
      temperature: 0.7
```

**Usage via ccproxy**:

```bash
# OpenAI-compatible format
curl -X POST "https://coder.visiquate.com/v1/chat/completions" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama/qwen-quality",
    "messages": [
      {"role": "user", "content": "Write a complex REST API with authentication"}
    ],
    "max_tokens": 4096
  }'
```

### 3. Via LLM Router (llm-router.js)

```bash
# Using the router CLI
node src/llm-router.js call-coding-llm "Implement OAuth2 flow" \
  --model qwen2.5-coder:32b-instruct
```

**Auto-detection logic** in `llm-router.js`:
```javascript
// Router automatically detects Ollama endpoints
const isOllama = endpoint.type === 'ollama' ||
                 endpoint.url?.includes('coder.visiquate.com') ||
                 endpoint.defaultModel?.includes('qwen') ||
                 endpoint.defaultModel?.includes('llama');

// Default model selection
model: options.model || endpoint.defaultModel || 'qwen2.5-coder:32b-instruct'
```

---

## Agent Assignments (32B Model)

The following agents are configured to use **Qwen 2.5 Coder 32B**:

### Coding Specialists
1. **Python Specialist** - FastAPI, Django, ML/AI implementation
2. **Swift/iOS Specialist** - SwiftUI, UIKit, Core Data
3. **Go Specialist** - Microservices, gRPC, concurrency
4. **Rust Specialist** - Systems programming, memory safety
5. **Flutter Specialist** - Cross-platform mobile apps

### Integration Specialists
6. **Salesforce API Specialist** - REST/SOAP API, SOQL
7. **Authentik API Specialist** - OAuth2/OIDC, SAML

### Support Agents
8. **Documentation Lead** - Code comments, API docs
9. **Credential Manager** - Secure credential implementation
10. **DevOps Engineer** - Docker, Kubernetes, CI/CD

**Total: 10 agents use 32B model** (out of 14 total agents)

---

## Model Selection Strategy

### Use qwen-fast (7B) for:
- Quick bug fixes
- Simple function implementations
- Documentation generation
- Code formatting
- Basic refactoring
- Prototyping

### Use qwen-quality-128k (32B) for:
- Complex algorithms
- Full-stack applications
- API integrations (Salesforce, Authentik)
- Security implementations
- Production code
- System architecture
- Multi-file refactoring
- Large context requirements (>32k tokens)

### Fallback to Claude for:
- High-level architecture decisions
- Strategic planning
- UX/UI design decisions
- Business logic design
- When Ollama is unavailable

---

## Performance Characteristics

### 32B Model Advantages
- **Superior code quality**: Better understanding of complex requirements
- **Larger context**: 128k tokens vs 32k
- **Better reasoning**: More sophisticated problem-solving
- **Production-ready**: Code requires fewer revisions

### 32B Model Trade-offs
- **Slower inference**: ~20 tokens/sec vs 50 tokens/sec (2.5x slower)
- **Higher memory**: ~20GB vs 4GB (5x more RAM)
- **Longer response time**: ~30-60 seconds for complex tasks

### Cost Comparison
- **Both models are FREE** (self-hosted on Mac mini)
- No API costs
- No token limits
- Full privacy

---

## Testing Status

### Current Availability
⚠️ **Unable to verify models remotely** - Server appears down or network unreachable

### Expected Models (from documentation)
- ✅ `qwen2.5-coder:7b-instruct` (qwen-fast)
- ✅ `qwen2.5-coder:32b-instruct` or `qwen2.5-coder:32b-instruct-q8` (qwen-quality-128k)

### Test Script Reference
See `/Users/brent/git/cc-orchestra/tests/test-bearer-auth.sh` lines 147-174 for model testing examples.

---

## Recommendations

### 1. Model Naming Consistency

**Problem**: Multiple names for the same model
- `qwen-quality-128k`
- `qwen-quality`
- `qwen2.5-coder:32b-instruct`
- `qwen2.5-coder:32b-instruct-q8`

**Recommendation**: Standardize on ONE name across all configurations:
```
qwen2.5-coder:32b-instruct
```

### 2. ccproxy Configuration

**Current config** (`docs/ccproxy/config.yaml`):
- References `ollama/llama2` (incorrect)
- Needs update to `qwen-fast` and `qwen-quality-128k`

**Action Required**:
```yaml
model_list:
  - model_name: claude-3-5-sonnet
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct  # Update this

  - model_name: gpt-4
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct  # Update this

  - model_name: ollama/qwen-fast
    litellm_params:
      model: ollama/qwen2.5-coder:7b-instruct

  - model_name: ollama/qwen-quality
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct
```

### 3. Default Routing

**For Claude Orchestra**:
- Heavy coding → `qwen2.5-coder:32b-instruct` (32B)
- Quick tasks → `qwen2.5-coder:7b-instruct` (7B)
- Architecture → Claude Opus 4.1

**Smart routing** (future enhancement):
```javascript
function selectModel(taskComplexity, contextSize) {
  if (contextSize > 32000) return 'qwen2.5-coder:32b-instruct';
  if (taskComplexity === 'high') return 'qwen2.5-coder:32b-instruct';
  if (taskComplexity === 'low') return 'qwen2.5-coder:7b-instruct';
  return 'qwen2.5-coder:32b-instruct'; // Default to quality
}
```

### 4. Documentation Updates

Update these files with consistent naming:
- `config/orchestra-config.json` ✅ (already correct)
- `docs/ccproxy/config.yaml` ❌ (needs update)
- `docs/ccproxy/TEST_REPORT.md` ❌ (references old model)
- `docs/ccproxy/QUICK_REFERENCE.md` ❌ (needs 32B examples)

---

## Verification Checklist

To verify the 32B model is available:

```bash
# 1. Check if models are pulled on Mac mini
ssh coder@192.168.1.101 "ollama list | grep qwen"

# Expected output:
# qwen2.5-coder:7b-instruct         ...  4.7GB
# qwen2.5-coder:32b-instruct       ...  20GB

# 2. Test direct access
curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen2.5-coder:32b-instruct", "prompt": "test", "stream": false}'

# 3. List all models
curl -s "https://coder.visiquate.com/api/tags" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  | jq '.models[] | select(.name | contains("qwen"))'
```

---

## Summary

### What You Asked For
✅ **32B high-quality model**: `qwen2.5-coder:32b-instruct` (or `-q8` variant)

### How to Access
1. **Direct Ollama API**: Use model name `qwen2.5-coder:32b-instruct`
2. **Via ccproxy**: Use alias `ollama/qwen-quality` (once configured)
3. **Via llm-router**: Automatically routed based on task type

### Current Status
- ⚠️ Server appears down or unreachable for remote testing
- ✅ Configuration files reference the correct 32B model
- ❌ ccproxy config needs updating from `llama2` to `qwen2.5-coder`

### Next Steps
1. Verify server accessibility
2. Update ccproxy configuration
3. Test both 7B and 32B models
4. Document performance differences
5. Create routing guidelines for agents

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
**Status**: Waiting for server accessibility to verify deployment
