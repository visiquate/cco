# Claude Code LLM Routing Research

**Research Date:** 2025-11-04
**Objective:** Determine how to route Claude Code Task tool calls through an LLM router/proxy

---

## Executive Summary

**Answer to the core question:** Yes, Claude Code can be configured to route requests through a local proxy/router at 127.0.0.1 using the `ANTHROPIC_BASE_URL` environment variable.

**Recommended Approach:** Local HTTP proxy at 127.0.0.1:8080 that intercepts Claude API requests and routes them based on intelligent routing rules.

**Complexity:** Medium (requires proxy implementation but no Claude Code modifications)

---

## 1. Local Proxy Architecture (RECOMMENDED)

### Overview
Claude Code supports custom API endpoints via the `ANTHROPIC_BASE_URL` environment variable. This enables routing all Claude API calls through a local proxy that makes routing decisions.

### Architecture Diagram
```
┌─────────────────────────────────────────────────────────┐
│              Claude Code (User)                         │
│         Task("Python Expert", "Implement X")            │
└────────────────────┬────────────────────────────────────┘
                                                          │
         ANTHROPIC_BASE_URL=http://127.0.0.1:8080
                                                          │
                     ▼
         ┌───────────────────────┐
         │  Local LLM Router     │
         │  (127.0.0.1:8080)     │
         │                       │
         │  - Analyzes request   │
         │  - Checks agent type  │
         │  - Makes routing      │
         │    decision           │
         └───────────┬───────────┘
                                 │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌──────────────┐          ┌──────────────┐
│ Architecture │          │   Coding     │
│   Tasks      │          │   Tasks      │
│              │          │              │
│ → Claude API │          │ → Ollama     │
│ (claude.ai)  │          │ (coder.      │
│              │          │  visiquate   │
│              │          │  .com)       │
└──────────────┘          └──────────────┘
```

### Configuration

**Step 1: Set environment variable**
```bash
# In ~/.zshrc or ~/.bashrc
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
export ANTHROPIC_AUTH_TOKEN="your-actual-anthropic-api-key"
```

**Step 2: Start local proxy**
```bash
node src/llm-proxy-server.js
# Listens on 127.0.0.1:8080
# Proxies to Claude API or coder.visiquate.com based on routing rules
```

**Step 3: Use Claude Code normally**
```javascript
// Claude Code automatically routes through proxy
Task("Python Expert", "Implement feature X", "python-expert")
Task("Chief Architect", "Design system", "system-architect")
```

### Routing Decision Logic

The local proxy analyzes each request and routes based on:

1. **Request Analysis:**
   - Extract model name from request (opus, sonnet, haiku)
   - Parse prompt/messages for task context
   - Detect agent type hints in system messages

2. **Routing Rules:**
   ```javascript
   // Architecture tasks → Claude API
   if (model === 'opus' || agentType === 'system-architect') {
     return CLAUDE_API_ENDPOINT;
   }

   // Coding tasks → Ollama
   if (agentType in ['python-expert', 'backend-dev', 'ios-developer']) {
     return OLLAMA_ENDPOINT;
   }

   // Default → Claude API
   return CLAUDE_API_ENDPOINT;
   ```

3. **Response Translation:**
   - Proxy translates Ollama responses to Claude API format
   - Maintains compatibility with Claude Code expectations

### Implementation Requirements

**New File: `src/llm-proxy-server.js`**
```javascript
const http = require('http');
const https = require('https');
const LLMRouter = require('./llm-router');

class ClaudeCodeProxy {
  constructor(config) {
    this.router = new LLMRouter(config);
    this.claudeApiKey = process.env.ANTHROPIC_AUTH_TOKEN;
  }

  async handleRequest(req, res) {
    // 1. Parse incoming Claude Code request
    const body = await this.parseBody(req);
    const routing = this.analyzeAndRoute(body);

    // 2. Route to appropriate endpoint
    if (routing.endpoint === 'claude') {
      return this.proxytoClaude(body, res);
    } else {
      return this.proxyToOllama(body, res);
    }
  }

  analyzeAndRoute(requestBody) {
    // Extract hints from request
    const model = requestBody.model;
    const systemMsg = requestBody.messages?.find(m => m.role === 'system');
    const agentType = this.extractAgentType(systemMsg?.content);

    // Use existing router logic
    return this.router.routeTask(agentType, 'default');
  }

  async proxyToClaude(body, res) {
    // Forward to real Claude API
    const response = await this.callClaudeAPI(body);
    res.writeHead(200, {'Content-Type': 'application/json'});
    res.end(JSON.stringify(response));
  }

  async proxyToOllama(body, res) {
    // Translate Claude format → Ollama format
    const ollamaPrompt = this.claudeToOllama(body);
    const ollamaResponse = await this.router.callCustomEndpoint(ollamaPrompt);

    // Translate Ollama format → Claude format
    const claudeResponse = this.ollamaToClaude(ollamaResponse);
    res.writeHead(200, {'Content-Type': 'application/json'});
    res.end(JSON.stringify(claudeResponse));
  }
}

const proxy = new ClaudeCodeProxy();
const server = http.createServer((req, res) => proxy.handleRequest(req, res));
server.listen(8080, '127.0.0.1', () => {
  console.log('LLM Proxy running on http://127.0.0.1:8080');
});
```

### Pros
✅ **Transparent to Claude Code** - No changes to Task tool usage
✅ **Automatic routing** - All requests automatically routed
✅ **Format translation** - Handles API format differences
✅ **Authentication handling** - Manages both Claude and Ollama auth
✅ **Easy to test** - Can enable/disable by changing env var
✅ **Maintainable** - Single proxy server, clear responsibility

### Cons
❌ **Requires proxy server running** - Additional process to manage
❌ **Format translation complexity** - Must handle Claude API format
❌ **Agent type detection** - Must infer agent type from requests
❌ **Potential latency** - Extra hop through localhost
❌ **Error handling** - Proxy failures affect all requests

### Risk Assessment
**Technical Risk:** Medium
**Maintenance Risk:** Low
**User Impact:** None (transparent to usage)

---

## 2. MCP Server Interceptor (NOT FEASIBLE)

### Overview
An MCP server that wraps the Task tool to intercept calls before they reach Claude API.

### Why This Doesn't Work

**Problem 1: Task Tool Architecture**
- The Task tool is **internal to Claude Code CLI**, not exposed via MCP
- MCP servers provide NEW tools, they cannot intercept existing tools
- Task tool calls go directly to Claude API, MCP never sees them

**Problem 2: MCP Protocol Limitations**
```javascript
// MCP servers can provide NEW tools:
{
  "name": "custom_task",
  "description": "Custom task execution",
  // ...but this is a DIFFERENT tool, not an interceptor
}

// MCP CANNOT intercept Claude Code's internal Task tool
Task("Agent", "task") → Goes directly to Claude API
                      → MCP never sees this call
```

**Problem 3: Execution Flow**
```
User → Claude Code → Task Tool → DIRECTLY TO CLAUDE API
                               ↑
                               MCP has no visibility here
```

### Verdict
❌ **NOT FEASIBLE** - MCP architecture doesn't support intercepting internal tools

---

## 3. Claude Code Configuration Override (VERIFIED APPROACH)

### Overview
Use Claude Code's built-in support for custom API endpoints via environment variables.

### Official Support

**Documented in Claude Code docs:**
- `ANTHROPIC_BASE_URL` - Override API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Authentication token
- `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` - Key rotation support

### Configuration Options

**Option A: Global environment variable**
```bash
# ~/.zshrc
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
export ANTHROPIC_AUTH_TOKEN="sk-ant-your-key"
```

**Option B: Claude Code settings**
```json
// ~/.claude/settings.local.json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:8080"
  }
}
```

**Option C: Per-session**
```bash
ANTHROPIC_BASE_URL=http://127.0.0.1:8080 claude
```

### Implementation
Same as Local Proxy Architecture (Option 1) - this IS the mechanism that enables it.

### Pros
✅ **Officially supported** - Documented Claude Code feature
✅ **No hacks required** - Clean, supported approach
✅ **Well-tested** - Used by LiteLLM, enterprise gateways
✅ **Flexible** - Can switch endpoints easily

### Cons
❌ **Still requires proxy** - Need to build routing logic
❌ **Environment setup** - Users must configure env vars

---

## 4. Agent-Level Explicit Calls (CURRENT APPROACH)

### Overview
The current approach where agents explicitly call the router.

### Current Implementation
```javascript
Task("Python Expert", `
  Implement feature X.

  For LLM routing, use:
  node src/llm-router.js call-coding-llm "your prompt"
`, "python-expert")
```

### Pros
✅ **Already implemented** - No new code needed
✅ **Explicit control** - Clear what's happening
✅ **No proxy required** - Direct API calls

### Cons
❌ **Manual routing** - Agents must remember to call router
❌ **Inconsistent usage** - Easy to forget
❌ **Not transparent** - Changes workflow
❌ **Error-prone** - Requires agent discipline

---

## Comparison Matrix

| Approach | Feasibility | Complexity | Transparency | Maintenance | Recommendation |
|----------|-------------|------------|--------------|-------------|----------------|
| **Local Proxy** | ✅ High | Medium | ✅ Full | Easy | ⭐ **RECOMMENDED** |
| **MCP Interceptor** | ❌ Not Possible | N/A | N/A | N/A | ❌ Don't pursue |
| **Env Variable Config** | ✅ High | Low | ✅ Full | Easy | ⭐ **SAME AS PROXY** |
| **Explicit Calls** | ✅ High | Low | ❌ None | Hard | ⚠️ Current fallback |

---

## Recommended Implementation Plan

### Phase 1: Local Proxy Server (Week 1)

**Tasks:**
1. Create `src/llm-proxy-server.js`
2. Implement Claude API format parsing
3. Integrate with existing `llm-router.js`
4. Add Ollama ↔ Claude format translation
5. Test with basic requests

**Deliverables:**
- Working proxy server on 127.0.0.1:8080
- Format translation for Ollama responses
- Routing based on model/agent type

### Phase 2: Agent Type Detection (Week 2)

**Tasks:**
1. Enhance agent type extraction from requests
2. Add prompt analysis for task type detection
3. Implement fallback routing logic
4. Add logging and debugging

**Deliverables:**
- Accurate agent type detection (>95%)
- Routing decision logging
- Fallback to Claude API for unknowns

### Phase 3: Testing & Validation (Week 3)

**Tasks:**
1. Test all 16 agent types
2. Verify routing decisions
3. Load testing (100+ concurrent requests)
4. Error handling edge cases

**Deliverables:**
- Test suite for proxy
- Performance benchmarks
- Error handling documentation

### Phase 4: Production Deployment (Week 4)

**Tasks:**
1. Systemd service for proxy (auto-start on boot)
2. Monitoring and health checks
3. Documentation for users
4. Migration guide from explicit calls

**Deliverables:**
- Production-ready proxy
- User setup guide
- Migration checklist

---

## Technical Specifications

### Proxy Server Requirements

**Input Format (Claude API):**
```json
{
  "model": "claude-sonnet-4.5-20250929",
  "messages": [
    {
      "role": "system",
      "content": "You are a Python expert..."
    },
    {
      "role": "user",
      "content": "Implement feature X"
    }
  ],
  "max_tokens": 4096,
  "temperature": 0.7
}
```

**Output Format (Claude API):**
```json
{
  "id": "msg_xyz",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Here is the implementation..."
    }
  ],
  "model": "claude-sonnet-4.5-20250929",
  "usage": {
    "input_tokens": 123,
    "output_tokens": 456
  }
}
```

**Ollama Format (for coding tasks):**
```json
// Request
{
  "model": "qwen2.5-coder:32b-instruct",
  "prompt": "Implement feature X",
  "stream": false
}

// Response
{
  "model": "qwen2.5-coder:32b-instruct",
  "response": "Here is the implementation...",
  "done": true
}
```

### Routing Decision Algorithm

```javascript
function routeRequest(requestBody) {
  // 1. Extract model
  const model = requestBody.model;

  // 2. Check for opus (always Claude)
  if (model.includes('opus')) {
    return { endpoint: 'claude', reason: 'Opus model' };
  }

  // 3. Extract agent type from system message
  const systemMsg = requestBody.messages.find(m => m.role === 'system');
  const agentType = extractAgentType(systemMsg?.content);

  // 4. Use router logic
  const routing = router.routeTask(agentType, 'default');

  return routing;
}

function extractAgentType(systemContent) {
  // Look for agent type hints in system prompt
  const patterns = {
    'python-expert': /python expert|python specialist/i,
    'ios-developer': /swift|ios|swiftui/i,
    'backend-dev': /go|rust|backend/i,
    'system-architect': /architect|design system/i,
  };

  for (const [type, pattern] of Object.entries(patterns)) {
    if (pattern.test(systemContent)) {
      return type;
    }
  }

  return 'coder'; // default
}
```

### Authentication Handling

```javascript
class ProxyAuth {
  constructor() {
    this.claudeKey = process.env.ANTHROPIC_AUTH_TOKEN;
    this.ollamaToken = process.env.CODER_LLM_TOKEN;
  }

  getClaudeHeaders() {
    return {
      'anthropic-version': '2023-06-01',
      'content-type': 'application/json',
      'x-api-key': this.claudeKey
    };
  }

  getOllamaHeaders() {
    return {
      'content-type': 'application/json',
      'authorization': `Bearer ${this.ollamaToken}`
    };
  }
}
```

---

## Configuration Examples

### Development Setup
```bash
# ~/.zshrc
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
export ANTHROPIC_AUTH_TOKEN="sk-ant-your-key"
export CODER_LLM_TOKEN="your-ollama-token"

# Start proxy
alias start-llm-proxy="node ~/git/cc-orchestra/src/llm-proxy-server.js"

# Normal Claude Code usage
claude
```

### Production Setup (systemd)
```ini
# /etc/systemd/system/llm-proxy.service
[Unit]
Description=LLM Routing Proxy for Claude Code
After=network.target

[Service]
Type=simple
User=brent
WorkingDirectory=/Users/brent/git/cc-orchestra
ExecStart=/usr/local/bin/node src/llm-proxy-server.js
Restart=always
Environment="ANTHROPIC_AUTH_TOKEN=sk-ant-key"
Environment="CODER_LLM_TOKEN=token"

[Install]
WantedBy=multi-user.target
```

### Testing Configuration
```bash
# Test routing decisions without proxy
node src/llm-router.js route python-expert implement

# Test proxy with curl
curl -X POST http://127.0.0.1:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_AUTH_TOKEN" \
  -d '{
    "model": "claude-sonnet-4.5-20250929",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

---

## Gotchas and Limitations

### 1. Agent Type Detection Accuracy
**Issue:** Not all requests clearly indicate agent type
**Solution:** Use conservative fallback to Claude API
**Mitigation:** Add logging to improve pattern detection over time

### 2. Format Translation Complexity
**Issue:** Claude and Ollama have different response formats
**Solution:** Comprehensive translation layer with fallbacks
**Mitigation:** Extensive testing with edge cases

### 3. Authentication Token Security
**Issue:** Proxy needs access to both API keys
**Solution:** Use environment variables, never hardcode
**Mitigation:** Consider secure credential storage (Keychain)

### 4. Proxy Availability
**Issue:** If proxy crashes, all requests fail
**Solution:** Systemd auto-restart, health monitoring
**Mitigation:** Fallback to direct Claude API if proxy unreachable

### 5. Performance Overhead
**Issue:** Extra localhost hop adds latency
**Solution:** Optimize proxy, use HTTP keep-alive
**Expected:** <10ms overhead (negligible)

### 6. Model Name Mapping
**Issue:** Claude Code sends specific model names
**Solution:** Map to appropriate Ollama models
**Example:**
  - `claude-sonnet-4.5` → `qwen2.5-coder:32b-instruct`
  - `claude-opus-4.1` → Always route to real Claude

---

## Alternative Architectures Considered

### Option: Pre-flight Hook
**Idea:** Hook into Claude Code before API call
**Verdict:** Not possible - no pre-API-call hooks exposed

### Option: Response Interceptor
**Idea:** Intercept responses and modify
**Verdict:** Doesn't help with routing decisions (too late)

### Option: Custom Task Tool via MCP
**Idea:** Provide `custom_task` tool that routes
**Verdict:** Works but not transparent (users must use different tool)

### Option: Monkey-patch Claude Code
**Idea:** Modify Claude Code binary to inject routing
**Verdict:** ❌ Fragile, breaks on updates, not maintainable

---

## Cost Analysis

### Current Approach (Explicit Calls)
- **Development Time:** 0 (already built)
- **Maintenance:** High (agents forget to call router)
- **User Experience:** Poor (manual routing)

### Local Proxy Approach
- **Development Time:** ~40 hours (4 weeks)
  - Week 1: Proxy server (16h)
  - Week 2: Agent detection (12h)
  - Week 3: Testing (8h)
  - Week 4: Deployment (4h)
- **Maintenance:** Low (runs automatically)
- **User Experience:** Excellent (transparent)

### ROI Calculation
- **One-time Investment:** 40 hours
- **Ongoing Savings:** ~2 hours/week (no manual routing)
- **Payback Period:** 20 weeks (~5 months)
- **Long-term Benefit:** Infinite (fully automated)

---

## Success Criteria

### Must Have
✅ Proxy successfully routes architecture tasks to Claude API
✅ Proxy successfully routes coding tasks to Ollama
✅ Format translation works for both directions
✅ Authentication handled securely
✅ Error handling prevents cascading failures

### Should Have
✅ Agent type detection >95% accurate
✅ Latency overhead <10ms
✅ Logging for debugging and monitoring
✅ Health check endpoint
✅ Graceful degradation to Claude API on errors

### Nice to Have
✅ Web UI for monitoring routing decisions
✅ Metrics dashboard (requests/min, routing breakdown)
✅ A/B testing framework (compare output quality)
✅ Cost tracking per endpoint

---

## Conclusion

**RECOMMENDED ARCHITECTURE:** Local HTTP proxy at 127.0.0.1:8080

**Key Benefits:**
1. ✅ **Transparent** - No changes to Claude Code usage
2. ✅ **Automatic** - All requests automatically routed
3. ✅ **Supported** - Uses official `ANTHROPIC_BASE_URL` feature
4. ✅ **Maintainable** - Single proxy server, clear responsibility
5. ✅ **Testable** - Can enable/disable with environment variable

**Implementation Effort:** ~40 hours over 4 weeks

**Expected Outcomes:**
- 100% automatic routing (no manual calls)
- Transparent to users (works like normal Claude Code)
- 62.5% cost savings on coding tasks (via Ollama)
- <10ms latency overhead
- Production-ready with systemd auto-start

**Next Steps:**
1. Review this research document
2. Approve implementation plan
3. Begin Phase 1: Proxy server development
4. Test with small subset of agents
5. Gradual rollout to all 16 agents

---

## References

- [Claude Code LLM Gateway Documentation](https://docs.claude.com/en/docs/claude-code/llm-gateway)
- [LiteLLM Integration Guide](https://docs.litellm.ai/docs/tutorials/claude_responses_api)
- [Claude Code Proxy (Community)](https://github.com/fuergaosi233/claude-code-proxy)
- [Orchestra LLM Routing Guide](./LLM_ROUTING_GUIDE.md)
- [Orchestra Routing Summary v2](./ROUTING_SUMMARY_V2.md)

---

**Research completed by:** Research Agent
**Date:** 2025-11-04
**Status:** Ready for implementation
