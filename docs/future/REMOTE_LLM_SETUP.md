# Remote LLM Integration - coder.visiquate.com

**Complete guide to integrating Claude Orchestra with the remote Ollama LLM server**

## Table of Contents
1. [Overview](#overview)
2. [Available Models](#available-models)
3. [Model Selection Strategy](#model-selection-strategy)
4. [LLM Router Architecture](#llm-router-architecture)
5. [Integration with Claude Orchestra](#integration-with-claude-orchestra)
6. [Testing & Validation](#testing--validation)
7. [Network Requirements](#network-requirements)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Claude Orchestra uses **intelligent LLM routing** to optimize cost and performance:
- **Strategic tasks** (architecture, design, quality assurance) → **Claude API** (Opus 4.1 / Sonnet 4.5)
- **Implementation tasks** (coding, documentation, DevOps) → **coder.visiquate.com** (Qwen models)

This hybrid approach provides:
- ✅ **Cost savings**: ~62.5% of agents use free Ollama endpoint
- ✅ **High quality**: Claude handles complex reasoning
- ✅ **Fast coding**: Qwen 2.5 Coder excels at implementation
- ✅ **Automatic routing**: No manual intervention needed

**Current Distribution**:
- **6 agents** (37.5%) → Claude API
- **10 agents** (62.5%) → coder.visiquate.com

---

## Available Models

The remote server at `https://coder.visiquate.com` hosts two primary models optimized for different use cases:

### 1. **qwen-fast** (Fast & Efficient)

**Model Details**:
```json
{
  "name": "qwen-fast:latest",
  "family": "qwen2",
  "parameter_size": "7.6B",
  "quantization": "Q4_K_M",
  "context_length": "32k tokens",
  "size": "4.68 GB",
  "base_model": "qwen2.5-coder:7b-instruct"
}
```

**Best For**:
- Simple coding tasks
- Quick prototypes
- Code formatting and linting
- Documentation generation
- Simple API integrations
- Credential management
- Basic DevOps scripts

**Performance Characteristics**:
- ⚡ **Speed**: Very fast (< 2s response time)
- 💾 **Memory**: Low resource usage
- 🎯 **Quality**: Good for straightforward tasks
- 📊 **Context**: 32k tokens sufficient for most files

**When to Use**:
- Single-file changes
- Standard CRUD operations
- Following established patterns
- Quick iterations
- Documentation updates

---

### 2. **qwen-quality-128k** (High Quality)

**Model Details**:
```json
{
  "name": "qwen-quality-128k:latest",
  "family": "qwen2",
  "parameter_size": "32.8B",
  "quantization": "Q8_0",
  "context_length": "128k tokens",
  "size": "34.82 GB",
  "base_model": "qwen2.5-coder:32b-instruct-q8_0"
}
```

**Best For**:
- Complex algorithms
- Multi-file refactoring
- Advanced architecture implementation
- Performance optimization
- Complex API integrations (Salesforce, Authentik)
- Security-critical code
- Full-stack features

**Performance Characteristics**:
- 🎯 **Quality**: Excellent reasoning and code quality
- 📊 **Context**: 128k tokens for large codebases
- 🧠 **Capability**: Handles complex logic
- ⏱️ **Speed**: Moderate (3-8s response time)
- 💾 **Memory**: Higher resource usage

**When to Use**:
- Multi-component features
- Complex business logic
- Large-scale refactoring
- Performance-critical code
- Security implementations
- Advanced integrations

---

## Model Selection Strategy

### Default Routing by Agent Type

The LLM router **automatically selects** the appropriate model based on agent type:

| Agent | Model | Reason |
|-------|-------|--------|
| Python Specialist | qwen-quality-128k | Complex frameworks (FastAPI, Django, ML) |
| Swift/iOS Specialist | qwen-quality-128k | SwiftUI, UIKit, Core Data complexity |
| Go Specialist | qwen-quality-128k | Concurrency, microservices architecture |
| Rust Specialist | qwen-quality-128k | Memory safety, systems programming |
| Flutter Specialist | qwen-quality-128k | Cross-platform state management |
| Salesforce API Specialist | qwen-quality-128k | Complex OAuth, SOQL, API integration |
| Authentik API Specialist | qwen-quality-128k | OAuth2/OIDC, SAML complexity |
| Documentation Lead | qwen-fast | Code comments, docstrings |
| Credential Manager | qwen-fast | Simple credential operations |
| DevOps Engineer | qwen-quality-128k | Docker, Kubernetes, IaC complexity |

### Manual Override

You can manually specify a model when needed:

```bash
# Call specific model via CLI
node src/llm-router.js call-coding-llm "task description" --model qwen-fast

# Or for high quality
node src/llm-router.js call-coding-llm "complex task" --model qwen-quality-128k
```

### Token Budget Considerations

**qwen-fast** (32k context):
- ✅ Single files < 1000 lines
- ✅ Simple multi-file tasks
- ❌ Large codebase analysis
- ❌ Complex multi-file refactoring

**qwen-quality-128k** (128k context):
- ✅ Entire project analysis
- ✅ Large-scale refactoring
- ✅ Multi-component features
- ✅ Comprehensive documentation

### Performance Trade-offs

```
Speed:     qwen-fast (2s) >> qwen-quality-128k (5s)
Quality:   qwen-quality-128k >> qwen-fast
Context:   qwen-quality-128k (128k) >> qwen-fast (32k)
Cost:      Both FREE via coder.visiquate.com
```

**Rule of Thumb**:
- Start with `qwen-fast` for simple tasks
- Upgrade to `qwen-quality-128k` when you need:
  - More context (> 32k tokens)
  - Better reasoning
  - Complex logic
  - Critical code

---

## LLM Router Architecture

### How llm-router.js Works

The router intelligently routes tasks based on agent type and task description:

```javascript
// File: src/llm-router.js

class LLMRouter {
  routeTask(agentType, taskType) {
    // Architecture/Planning → Claude API
    if (this.isArchitectureTask(agentType, taskType)) {
      return { endpoint: 'claude', useClaudeCode: true };
    }

    // Coding/Implementation → coder.visiquate.com
    if (this.isCodingTask(agentType, taskType)) {
      return {
        endpoint: 'custom',
        url: 'https://coder.visiquate.com',
        useClaudeCode: false
      };
    }

    // Default → Claude API
    return { endpoint: 'claude', useClaudeCode: true };
  }
}
```

### Architecture Task Detection

**Always routes to Claude**:
- Agent types: `system-architect`, `architecture`, `specification`, `planner`
- Task keywords: `design`, `architecture`, `planning`, `specification`, `requirements`, `coordination`

**Examples**:
- "Design the system architecture"
- "Create specification document"
- "Plan the feature roadmap"
- "Coordinate team activities"

### Coding Task Detection

**Routes to coder.visiquate.com**:
- Agent types: `python-expert`, `ios-developer`, `backend-dev`, `mobile-developer`, `coder`, `frontend-dev`, `deployment-engineer`
- Task keywords: `implement`, `code`, `develop`, `build`, `write code`, `programming`

**Examples**:
- "Implement REST API endpoints"
- "Build Flutter mobile app"
- "Write Python data pipeline"
- "Deploy with Docker"

### Configuration

The router loads configuration from `config/orchestra-config.json`:

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

### API Endpoint Format

**Ollama API** (detected automatically):
```bash
POST https://coder.visiquate.com/api/generate
Content-Type: application/json

{
  "model": "qwen2.5-coder:32b-instruct",
  "prompt": "Write a Python function to...",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "num_predict": 4096
  }
}
```

**Response Format**:
```json
{
  "model": "qwen2.5-coder:32b-instruct",
  "created_at": "2025-01-16T10:30:00Z",
  "response": "def example():\n    pass",
  "done": true,
  "context": [...],
  "total_duration": 5000000000,
  "load_duration": 1000000000,
  "prompt_eval_count": 50,
  "eval_count": 100
}
```

### Request/Response Flow

```
┌─────────────────┐
│  Claude Orchestra│
│  (Orchestrator) │
└────────┬────────┘
                  │
         v
┌─────────────────┐
│  LLM Router     │
│  (Decision)     │
└────┬───────┬────┘
     │            │
     │       └─────────────────────┐
     v                             v
┌─────────────┐          ┌─────────────────┐
│ Claude API  │          │ coder.visiquate │
│ (Strategy)  │          │ (Implementation)│
└─────────────┘          └─────────────────┘
     │                                     │
     └─────────────┬───────────────┘
                   v
           ┌───────────────┐
           │  Result       │
           │  (Combined)   │
           └───────────────┘
```

---

## Integration with Claude Orchestra

### How Coding Agents Use Remote LLM

When you spawn agents using Claude Code's Task tool, the routing happens automatically:

```javascript
// User request: "Build a Python API with authentication"

// Step 1: Claude Code spawns agents
Task("Chief Architect", "Design API architecture", "system-architect", "opus")
// ^ Routes to Claude API (Opus 4.1)

Task("Python Expert", "Implement FastAPI endpoints", "python-expert", "sonnet")
// ^ Routes to coder.visiquate.com (qwen-quality-128k)

Task("Security Auditor", "Review authentication", "security-auditor", "sonnet")
// ^ Routes to Claude API (Sonnet 4.5)

Task("Documentation Lead", "Document API", "coder", "haiku")
// ^ Routes to coder.visiquate.com (qwen-fast)
```

### Which Agents Benefit Most

**Maximum Benefit** (cost savings + quality):
1. **Python Specialist** - 80% of tasks are implementation
2. **Go Specialist** - Heavy coding workload
3. **Rust Specialist** - Systems programming
4. **Flutter Specialist** - Mobile development
5. **DevOps Engineer** - Infrastructure as code

**Moderate Benefit**:
6. **Swift/iOS Specialist** - UI and data layer
7. **Salesforce API Specialist** - Integration code
8. **Authentik API Specialist** - Auth implementation
9. **Documentation Lead** - Code-level docs

**Minimal Benefit**:
10. **Credential Manager** - Simple operations

### Token Savings with Remote LLM

**Estimated savings per project**:

| Project Size | Claude-Only Cost | With Remote LLM | Savings |
|--------------|------------------|-----------------|---------|
| Small (< 1k lines) | ~$2.00 | ~$0.80 | **60%** |
| Medium (1-10k lines) | ~$8.00 | ~$3.00 | **62.5%** |
| Large (10k+ lines) | ~$25.00 | ~$9.50 | **62%** |

**Why 62.5% savings?**
- 10 out of 16 agents (62.5%) use free Ollama endpoint
- Strategic agents (37.5%) still use Claude for quality
- Net result: ~62.5% reduction in API costs

### Fallback Strategies

The router implements automatic fallbacks:

```javascript
// Fallback chain:
1. Try coder.visiquate.com (if enabled and reachable)
2. If connection fails → Use Claude API
3. If Claude API fails → Return error

// Configuration:
{
  "llmRouting": {
    "rules": {
      "fallbackToClaude": true  // Enable automatic fallback
    }
  }
}
```

**Fallback Scenarios**:
- ❌ Network connectivity issues
- ❌ Server maintenance
- ❌ Rate limiting (unlikely with Ollama)
- ❌ Model unavailable

**Monitoring**:
```bash
# Check router status
node src/llm-router.js stats

# Test specific routing
node src/llm-router.js route python-expert implement
```

---

## Testing & Validation

### Test Connection to coder.visiquate.com

**1. Basic Connectivity Test**:
```bash
# Check server is reachable (no auth required for tags endpoint)
curl -s https://coder.visiquate.com/api/tags | jq '.models[] | {name, parameter_size, quantization}'
```

**1b. Test Bearer Token Authentication**:
```bash
# Set bearer token
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# Test with authentication
curl -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $CODER_LLM_TOKEN" \
  -d '{"model": "qwen-fast:latest", "prompt": "Write hello world", "stream": false}' \
  | jq -r '.response'
```

**Expected Output**:
```json
{
  "name": "qwen-fast:latest",
  "parameter_size": "7.6B",
  "quantization": "Q4_K_M"
}
{
  "name": "qwen-quality-128k:latest",
  "parameter_size": "32.8B",
  "quantization": "Q8_0"
}
```

**2. Verify Model Availability**:
```bash
# List all available models
curl -s https://coder.visiquate.com/api/tags | jq '.models[] | .name'
```

**Expected Models**:
- `qwen-fast:latest`
- `qwen-quality-128k:latest`
- `qwen2.5-coder:32b-instruct`
- `qwen2.5-coder:32b-instruct-q8_0`
- `qwen2.5-coder:7b-instruct`

**3. Test LLM Router**:
```bash
# Check routing configuration
node src/llm-router.js stats
```

**Expected Output**:
```json
{
  "endpoints": [
    {
      "name": "coding",
      "enabled": true,
      "url": "https://coder.visiquate.com"
    }
  ],
  "architectureTasks": "Always route to Claude",
  "codingTasks": "Route to https://coder.visiquate.com"
}
```

**4. Test Routing Decisions**:
```bash
# Test architecture task (should route to Claude)
node src/llm-router.js route system-architect design

# Test coding task (should route to custom LLM)
node src/llm-router.js route python-expert implement
```

### Example Requests for Each Model

**qwen-fast (Simple Task)**:
```bash
# Via LLM Router
node src/llm-router.js call-coding-llm "Write a Python function to validate email addresses"

# Direct API call
curl -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-fast:latest",
    "prompt": "Write a Python function to validate email addresses",
    "stream": false,
    "options": {
      "temperature": 0.7,
      "num_predict": 500
    }
  }'
```

**qwen-quality-128k (Complex Task)**:
```bash
# Via LLM Router
node src/llm-router.js call-coding-llm "Implement a FastAPI application with JWT authentication and PostgreSQL integration" --model qwen-quality-128k

# Direct API call
curl -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-quality-128k:latest",
    "prompt": "Implement a FastAPI application with JWT authentication and PostgreSQL integration",
    "stream": false,
    "options": {
      "temperature": 0.7,
      "num_predict": 4096
    }
  }'
```

### Performance Benchmarks

**Test Setup**:
```bash
# Create benchmark script
cat > benchmark.sh << 'EOF'
#!/bin/bash
echo "Testing qwen-fast..."
time node src/llm-router.js call-coding-llm "Write a hello world in Python" --model qwen-fast

echo -e "\nTesting qwen-quality-128k..."
time node src/llm-router.js call-coding-llm "Write a REST API endpoint with error handling" --model qwen-quality-128k
EOF

chmod +x benchmark.sh
./benchmark.sh
```

**Expected Performance** (approximate):
```
qwen-fast:
- Simple task: 1-3 seconds
- Response time: ~2s average
- Token generation: ~50 tokens/sec

qwen-quality-128k:
- Complex task: 3-8 seconds
- Response time: ~5s average
- Token generation: ~30 tokens/sec
```

**Quality Comparison**:
```bash
# Test same prompt on both models
PROMPT="Implement a Python function with error handling, logging, and type hints"

echo "=== qwen-fast ===" > comparison.txt
node src/llm-router.js call-coding-llm "$PROMPT" --model qwen-fast >> comparison.txt

echo -e "\n=== qwen-quality-128k ===" >> comparison.txt
node src/llm-router.js call-coding-llm "$PROMPT" --model qwen-quality-128k >> comparison.txt

cat comparison.txt
```

---

## Network Requirements

### Firewall/VPN Requirements

**Connection Details**:
- **Protocol**: HTTPS (TLS 1.2+)
- **Port**: 443
- **Hostname**: `coder.visiquate.com`
- **IP**: (resolves via DNS)

**Firewall Rules**:
```bash
# Allow outbound HTTPS to coder.visiquate.com
sudo iptables -A OUTPUT -p tcp --dport 443 -d $(dig +short coder.visiquate.com) -j ACCEPT

# Or for all HTTPS (recommended)
sudo iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT
```

**Corporate Network**:
- ✅ No special VPN required (public endpoint)
- ✅ Standard HTTPS (port 443) should work
- ⚠️ Some corporate proxies may block Ollama API
- 💡 Contact IT if connection fails

**Proxy Configuration**:
```bash
# Set proxy for Node.js
export HTTPS_PROXY=http://proxy.company.com:8080
export NO_PROXY=localhost,127.0.0.1

# Test connection through proxy
node src/llm-router.js call-coding-llm "test" --verbose
```

### Authentication Mechanism

**Current Setup**:
- ✅ **Bearer token authentication** enabled
- 🔒 Token required for API access
- 🔐 Token stored securely via credential manager or environment variable

**Authentication Methods** (in priority order):

1. **Environment Variable** (Recommended for development):
```bash
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c
node src/llm-router.js call-coding-llm "your prompt"
```

2. **Credential Manager** (Recommended for production):
```bash
# Store token securely (encrypted)
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token

# Token will be automatically retrieved when making API calls
node src/llm-router.js call-coding-llm "your prompt"
```

3. **Configuration File** (Alternative):
```javascript
// config/orchestra-config.json
{
  "llmRouting": {
    "endpoints": {
      "coding": {
        "url": "https://coder.visiquate.com",
        "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
      }
    }
  }
}
```

**Security Best Practices**:
- ⚠️ Never commit tokens to version control
- ✅ Use environment variables for local development
- ✅ Use credential manager for shared/production environments
- ✅ Set `CREDENTIAL_ENCRYPTION_KEY` environment variable for production encryption
- ✅ Rotate tokens regularly (tracked via credential manager)

### Rate Limiting Considerations

**Current Limits**:
- ✅ **No rate limits** on coder.visiquate.com
- ✅ Self-hosted Ollama (unlimited requests)
- ⚠️ Server capacity is the only limit

**Best Practices**:
```javascript
// Implement exponential backoff for failures
async function callWithRetry(prompt, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await router.callCustomEndpoint(prompt);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(Math.pow(2, i) * 1000); // 1s, 2s, 4s
    }
  }
}
```

**Monitor Server Load**:
```bash
# Check server health
curl -s https://coder.visiquate.com/api/tags | jq '.models | length'

# If response is slow or fails:
# 1. Check server resources
# 2. Consider fallback to Claude
# 3. Report to system admin
```

### Error Handling

The router implements comprehensive error handling:

```javascript
// src/llm-router.js
try {
  const response = await router.callCustomEndpoint(prompt);
  return response;
} catch (error) {
  if (error.code === 'ECONNREFUSED') {
    console.error('Server unavailable, falling back to Claude');
    return { endpoint: 'claude', fallback: true };
  }

  if (error.code === 'ETIMEDOUT') {
    console.error('Request timeout, retrying...');
    // Retry logic
  }

  throw error; // Re-throw if unrecoverable
}
```

---

## Troubleshooting

### Common Issues & Solutions

**1. Connection Refused**
```bash
Error: Request failed: connect ECONNREFUSED
```

**Solution**:
```bash
# Check if server is running
curl -I https://coder.visiquate.com

# Check DNS resolution
dig +short coder.visiquate.com

# Check network connectivity
ping coder.visiquate.com

# Verify firewall rules
sudo iptables -L OUTPUT -n | grep 443
```

**2. Model Not Found**
```bash
Error: model 'qwen-quality-128k:latest' not found
```

**Solution**:
```bash
# List available models
curl -s https://coder.visiquate.com/api/tags | jq '.models[] | .name'

# Use correct model name
node src/llm-router.js call-coding-llm "test" --model qwen2.5-coder:32b-instruct
```

**3. Timeout Errors**
```bash
Error: Request failed: socket hang up
```

**Solution**:
```bash
# Increase timeout in config
# config/orchestra-config.json
{
  "llmRouting": {
    "endpoints": {
      "coding": {
        "timeout": 30000  // 30 seconds
      }
    }
  }
}

# Or use smaller prompt/lower maxTokens
node src/llm-router.js call-coding-llm "short task" --max-tokens 500
```

**4. Invalid JSON Response**
```bash
Error: Failed to parse response: Unexpected token < in JSON
```

**Solution**:
```bash
# Check if you're hitting an error page
curl -s https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen-fast:latest", "prompt": "test"}' | head -20

# Verify API endpoint is correct
curl -s https://coder.visiquate.com/api/tags
```

**5. Router Not Using Custom Endpoint**
```bash
# All tasks routing to Claude despite configuration
```

**Solution**:
```bash
# Check if routing is enabled
node src/llm-router.js stats

# Verify config
cat config/orchestra-config.json | jq '.llmRouting'

# Enable if disabled
{
  "llmRouting": {
    "enabled": true,  // Must be true
    "endpoints": {
      "coding": {
        "enabled": true  // Must be true
      }
    }
  }
}
```

### Debugging Tools

**Enable Verbose Logging**:
```bash
# Set debug environment variable
export DEBUG=llm-router:*

# Run with verbose output
node src/llm-router.js call-coding-llm "test" --verbose
```

**Test End-to-End Flow**:
```bash
# Create comprehensive test script
cat > test-llm-integration.sh << 'EOF'
#!/bin/bash
set -e

echo "=== Testing LLM Integration ==="

echo -e "\n1. Testing server connectivity..."
curl -s https://coder.visiquate.com/api/tags > /dev/null && echo "✓ Server reachable" || echo "✗ Server unreachable"

echo -e "\n2. Checking available models..."
curl -s https://coder.visiquate.com/api/tags | jq -r '.models[] | "  - \(.name) (\(.details.parameter_size))"'

echo -e "\n3. Testing router configuration..."
node src/llm-router.js stats | jq -r '"Routing: \(.codingTasks)"'

echo -e "\n4. Testing routing decision..."
node src/llm-router.js route python-expert implement | jq -r '"Routes to: \(.endpoint) (\(.url // "N/A"))"'

echo -e "\n5. Testing actual LLM call..."
echo "  Calling qwen-fast with simple prompt..."
node src/llm-router.js call-coding-llm "print hello world" --model qwen-fast | jq -r '.text' | head -5

echo -e "\n✓ All tests passed!"
EOF

chmod +x test-llm-integration.sh
./test-llm-integration.sh
```

**Monitor Network Requests**:
```bash
# Use verbose curl to see full request/response
curl -v https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-fast:latest",
    "prompt": "test",
    "stream": false
  }' 2>&1 | grep -E '^(>|<|HTTP)'
```

### Getting Help

**Check System Status**:
```bash
# Router status
node src/llm-router.js stats

# Agent routing
cat docs/ROUTING_SUMMARY_V2.md

# Configuration
cat config/orchestra-config.json | jq '.llmRouting'
```

**Log Files** (if implemented):
```bash
# Check for logs
ls -lah logs/llm-router-*.log

# Tail recent errors
tail -f logs/llm-router.log | grep ERROR
```

**Contact Information**:
- **System Admin**: Contact VisiQuate DevOps for server issues
- **Documentation**: See `docs/LLM_ROUTING_GUIDE.md` for more details
- **Configuration**: Check `config/orchestra-config.json` for settings

---

## Summary

**Key Takeaways**:

1. ✅ **Two models available**:
   - `qwen-fast` (7B, 32k context) - Fast & efficient
   - `qwen-quality-128k` (32B, 128k context) - High quality

2. ✅ **Automatic routing**:
   - Architecture → Claude API
   - Coding → coder.visiquate.com
   - No manual intervention needed

3. ✅ **62.5% cost savings**:
   - 10 out of 16 agents use free Ollama endpoint
   - Strategic agents still use Claude for quality

4. ✅ **Simple testing**:
   - `curl https://coder.visiquate.com/api/tags` - Check connectivity
   - `node src/llm-router.js stats` - Verify configuration
   - `node src/llm-router.js call-coding-llm "test"` - Test end-to-end

5. ✅ **Fallback support**:
   - Automatic fallback to Claude if Ollama unavailable
   - No service disruption

**Next Steps**:
1. Test connectivity: `curl https://coder.visiquate.com/api/tags`
2. Verify routing: `node src/llm-router.js stats`
3. Try a simple task: `node src/llm-router.js call-coding-llm "hello world"`
4. Build something: Spawn agents and watch automatic routing!

---

**Version**: 1.0.0
**Last Updated**: 2025-01-16
**Maintained By**: VisiQuate DevOps Team
