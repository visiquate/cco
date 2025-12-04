# Hybrid Routing Design: LLM Gateway Architecture

## Executive Summary

**Problem**: Claude Code's native OAuth authentication doesn't work through the gateway because the gateway intercepts traffic, and streaming breaks due to buffering.

**Solution**: **Option D - Gateway with Intelligent Passthrough** (RECOMMENDED)

The gateway receives all traffic but passes Anthropic requests through transparently without modifying auth headers, while routing non-Anthropic requests to configured providers.

## Current Architecture Analysis

### How Traffic Flows Today

```
Claude Code Process
    ↓ (spawns subagents via Task tool)
    ↓
    ├─ Main orchestrator (you) - uses ANTHROPIC_BASE_URL env var
    │  ↓
    │  → Gateway (port from PID file)
    │     ↓
    │     ├─ Anthropic requests → Direct to api.anthropic.com
    │     ├─ Azure agents → Azure OpenAI
    │     └─ DeepSeek agents → DeepSeek API
    │
    └─ Spawned agents (Task tool) - inherit same ANTHROPIC_BASE_URL
       ↓
       → Gateway (same port)
          ↓
          ├─ Anthropic requests → Direct to api.anthropic.com
          ├─ Azure agents → Azure OpenAI
          └─ DeepSeek agents → DeepSeek API
```

### Current Components

1. **launcher.rs** (lines 403-450)
   - Sets `ANTHROPIC_BASE_URL` to gateway port (preferred)
   - Falls back to `HTTPS_PROXY` for legacy proxy
   - Gateway URL: `http://127.0.0.1:{gateway_port}`

2. **LLM Gateway** (unified routing system)
   - Runs on random OS-assigned port (stored in PID file)
   - Routes based on agent type and model tier
   - Provides cost tracking and audit logging
   - **Already supports streaming via SSE** (api.rs lines 75-138)

3. **Anthropic Provider** (anthropic.rs)
   - **Already supports auth passthrough** (lines 113-128)
   - Checks for `client_auth` parameter first
   - Falls back to configured API key
   - **Already supports streaming** (lines 203-273)

4. **Routing Engine** (router.rs)
   - Routes by agent type (chief-architect → anthropic)
   - Routes by model tier (opus/sonnet/haiku → anthropic)
   - Infers agent type from prompts if not specified

### Key Insight: Most Infrastructure Already Exists!

The gateway **already has everything we need**:
- ✅ Auth passthrough (client_auth parameter)
- ✅ Streaming support (SSE forwarding)
- ✅ Intelligent routing (agent type + model tier)
- ✅ Cost tracking and audit logging

**The only issue**: Claude Code doesn't pass auth headers when making requests to the gateway because it treats it as a trusted local service.

## Option Analysis

### Option A: Don't Set ANTHROPIC_BASE_URL (REJECTED)

**How it would work:**
- Remove gateway URL from launcher.rs (lines 426-432)
- Claude Code connects directly to api.anthropic.com
- Azure/DeepSeek agents have no routing mechanism

**Problems:**
- ❌ No way to route Azure/DeepSeek agents
- ❌ Loses all cost tracking
- ❌ Loses all audit logging
- ❌ Gateway becomes useless

**Verdict**: Not viable - destroys the entire gateway system.

---

### Option B: Use HTTP_PROXY with Legacy Proxy (REJECTED)

**How it would work:**
- Set `HTTPS_PROXY` instead of `ANTHROPIC_BASE_URL`
- Old proxy (proxy.rs) intercepts all HTTPS traffic
- Routes based on agent type detection

**Problems:**
- ❌ Streaming still broken (buffering issue remains)
- ❌ More complex - two routing systems (proxy + gateway)
- ❌ HTTP proxy has limited visibility into request metadata
- ❌ Can't access agent_type from Claude Code's Task tool

**Verdict**: Same core problem - streaming doesn't work through HTTP proxy.

---

### Option C: Custom Agent Routing via Task Tool Metadata (REJECTED)

**How it would work:**
- Modify Claude Code's Task tool to inject different ANTHROPIC_BASE_URL per agent
- Chief architect gets no URL (direct to Anthropic)
- Azure/DeepSeek agents get gateway URL

**Problems:**
- ❌ Requires modifying Claude Code (not possible)
- ❌ Claude Code controls Task tool implementation
- ❌ We don't control how agents are spawned
- ❌ Overly complex - different env vars per agent

**Verdict**: Not feasible - requires changes to Claude Code itself.

---

### Option D: Gateway with Intelligent Passthrough (RECOMMENDED)

**How it works:**
1. **All traffic goes through gateway** (ANTHROPIC_BASE_URL set)
2. **Gateway extracts auth from client request headers**
3. **Gateway inspects request** to determine routing:
   - Agent type in request body
   - Model name in request body
   - System prompt patterns (fallback)
4. **Smart routing**:
   - **Anthropic requests**: Pass through with client's auth headers
   - **Azure requests**: Use configured Azure credentials
   - **DeepSeek requests**: Use configured DeepSeek credentials

**Why this works:**

1. **Auth Passthrough Already Exists**
   ```rust
   // anthropic.rs lines 113-128
   if let Some(auth) = client_auth {
       // Passthrough client's auth header
       if auth.to_lowercase().starts_with("bearer ") {
           request_builder = request_builder.header("Authorization", auth);
       } else {
           request_builder = request_builder.header("x-api-key", auth);
       }
   }
   ```

2. **Streaming Already Works**
   ```rust
   // api.rs lines 75-138 - SSE streaming support
   if request.stream {
       return handle_streaming_request(gateway, request, auth_header, beta_header).await;
   }
   ```

3. **Routing Already Implemented**
   ```rust
   // router.rs - determines provider based on:
   // 1. Agent type rules (chief-architect → anthropic)
   // 2. Model tier rules (opus/sonnet/haiku → anthropic)
   // 3. Inferred agent type from prompt
   ```

**The Missing Piece**: Claude Code doesn't send auth headers to local gateway.

---

## Recommended Solution: Environment Variable Passthrough

### The Core Issue

Claude Code has these credentials:
- `ANTHROPIC_API_KEY` (optional)
- `CLAUDE_CODE_OAUTH_TOKEN` (always present when logged in)

When Claude Code makes requests to `api.anthropic.com` directly:
- ✅ It includes auth headers automatically

When Claude Code makes requests to `http://127.0.0.1:PORT` (gateway):
- ❌ It treats it as a trusted local service
- ❌ No auth headers sent

### The Solution

Make the gateway read Claude Code's credentials from **environment variables** when client auth is missing:

```rust
// In anthropic.rs, complete() method

async fn complete(
    &self,
    request: CompletionRequest,
    client_auth: Option<String>,
    client_beta: Option<String>,
) -> Result<(CompletionResponse, RequestMetrics)> {
    // ... existing code ...

    // Build request with auth
    let mut request_builder = self
        .client
        .post(self.api_url())
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json");

    // AUTH PRIORITY:
    // 1. Client passthrough (if provided)
    // 2. Environment variable fallback
    // 3. Configured API key (last resort)

    if let Some(auth) = client_auth {
        // Passthrough client's auth header
        if auth.to_lowercase().starts_with("bearer ") {
            request_builder = request_builder.header("Authorization", auth);
        } else {
            request_builder = request_builder.header("x-api-key", auth);
        }
        tracing::debug!("Using client passthrough authentication");
    } else if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
        // Claude Code OAuth token (most common scenario)
        request_builder = request_builder.header("Authorization", format!("Bearer {}", oauth_token));
        tracing::debug!("Using CLAUDE_CODE_OAUTH_TOKEN from environment");
    } else if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        // Standard API key from environment
        request_builder = request_builder.header("x-api-key", api_key);
        tracing::debug!("Using ANTHROPIC_API_KEY from environment");
    } else if self.api_key.starts_with("sk-ant-oat") {
        // OAuth token from config
        request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
        tracing::debug!("Using configured OAuth token");
    } else {
        // Standard API key from config
        request_builder = request_builder.header("x-api-key", &self.api_key);
        tracing::debug!("Using configured API key");
    }

    // ... rest of method ...
}
```

### Why This Works

1. **Transparent to Claude Code**
   - No changes needed to Claude Code
   - No changes needed to how agents are spawned
   - Works with existing Task tool implementation

2. **Preserves Gateway Benefits**
   - ✅ Cost tracking for all requests
   - ✅ Audit logging for all requests
   - ✅ Multi-provider routing (Azure, DeepSeek)
   - ✅ Streaming works (SSE passthrough)

3. **Secure**
   - Credentials only accessible within daemon process
   - No credentials stored in gateway config
   - Environment variables inherited from parent process

4. **Simple Implementation**
   - Only 10-15 lines of code
   - All in one place (anthropic.rs)
   - No architectural changes needed

## Implementation Plan

### Phase 1: Add Environment Variable Fallback (5 minutes)

**File**: `/Users/brent/git/cc-orchestra/src/daemon/llm_gateway/providers/anthropic.rs`

**Changes**:
1. Modify `complete()` method (lines 82-199)
2. Modify `complete_stream()` method (lines 203-273)

**Code Changes**:

```rust
// In complete() method, replace lines 113-128 with:

if let Some(auth) = client_auth {
    // Priority 1: Client passthrough
    if auth.to_lowercase().starts_with("bearer ") {
        request_builder = request_builder.header("Authorization", auth);
    } else {
        request_builder = request_builder.header("x-api-key", auth);
    }
    tracing::debug!("Using client passthrough authentication");
} else if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
    // Priority 2: Claude Code OAuth token from environment
    request_builder = request_builder.header("Authorization", format!("Bearer {}", oauth_token));
    tracing::debug!("Using CLAUDE_CODE_OAUTH_TOKEN from environment");
} else if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
    // Priority 3: Standard API key from environment
    request_builder = request_builder.header("x-api-key", api_key);
    tracing::debug!("Using ANTHROPIC_API_KEY from environment");
} else if self.api_key.starts_with("sk-ant-oat") {
    // Priority 4: OAuth token from config
    request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
    tracing::debug!("Using configured OAuth token");
} else if !self.api_key.is_empty() {
    // Priority 5: Standard API key from config
    request_builder = request_builder.header("x-api-key", &self.api_key);
    tracing::debug!("Using configured API key");
} else {
    // No credentials available
    return Err(anyhow!("No Anthropic API credentials available (checked client_auth, CLAUDE_CODE_OAUTH_TOKEN, ANTHROPIC_API_KEY, and config)"));
}

// Apply same logic to complete_stream() method (lines 224-246)
```

### Phase 2: Test with Claude Code (10 minutes)

**Test Cases**:

1. **Main orchestrator makes request**
   - Gateway receives request
   - No client_auth header (Claude Code doesn't send to localhost)
   - Gateway reads CLAUDE_CODE_OAUTH_TOKEN
   - Passes to api.anthropic.com
   - ✅ Streaming works (SSE passthrough)

2. **Spawned agent makes request**
   - Agent inherits ANTHROPIC_BASE_URL
   - Same flow as main orchestrator
   - ✅ Same credentials work

3. **Azure agent makes request**
   - Gateway routes to Azure provider
   - Uses configured Azure credentials
   - ✅ Separate credential system

4. **Cost tracking works**
   - All requests logged
   - Metrics calculated
   - ✅ Audit trail maintained

### Phase 3: Documentation (5 minutes)

Update these files:
- `docs/HYBRID_ROUTING_DESIGN.md` (this file)
- `config/orchestra-config.json` (add comments)
- `README.md` (mention auth behavior)

## Configuration Example

```json
{
  "llmGateway": {
    "routing": {
      "defaultProvider": "anthropic",
      "agentRules": {
        "chief-architect": "anthropic",
        "code-reviewer": "azure",
        "python-specialist": "deepseek"
      },
      "modelTierRules": {
        "opus": "anthropic",
        "sonnet": "anthropic",
        "haiku": "anthropic"
      }
    },
    "providers": {
      "anthropic": {
        "enabled": true,
        "type": "anthropic",
        "baseUrl": "https://api.anthropic.com",
        "apiKeyRef": "env:ANTHROPIC_API_KEY",
        "comment": "Auth priority: 1) client passthrough, 2) CLAUDE_CODE_OAUTH_TOKEN env var, 3) ANTHROPIC_API_KEY env var, 4) configured apiKeyRef"
      },
      "azure": {
        "enabled": true,
        "type": "azure",
        "baseUrl": "https://cco-resource.openai.azure.com",
        "apiKeyRef": "env:AZURE_OPENAI_API_KEY",
        "deployment": "gpt-5-1-mini"
      },
      "deepseek": {
        "enabled": true,
        "type": "deepseek",
        "baseUrl": "https://cco-resource.cognitiveservices.azure.com",
        "apiKeyRef": "env:DEEPSEEK_API_KEY"
      }
    }
  }
}
```

## Benefits of This Approach

### 1. Zero Changes to Claude Code
- No modifications to Task tool
- No modifications to agent spawning
- Works with current Claude Code implementation

### 2. Full Gateway Functionality
- ✅ Cost tracking for all requests
- ✅ Audit logging with request/response bodies
- ✅ Multi-provider routing (Azure, DeepSeek, Ollama)
- ✅ Intelligent routing (agent type + model tier)
- ✅ Streaming support (SSE passthrough)

### 3. Security
- Credentials never leave daemon process
- Environment variable inheritance (secure)
- No credential storage in configs (already using env refs)

### 4. Simplicity
- Single routing system (gateway only)
- No HTTP proxy complexity
- Clear auth priority chain
- Easy to debug (structured logging)

### 5. Future-Proof
- Works with OAuth tokens (current)
- Works with API keys (backward compatible)
- Works with client passthrough (if Claude Code adds it)
- Extensible to other auth methods

## How Agent Spawning Works

Claude Code's Task tool implementation:
```javascript
// When spawning an agent via Task tool:
Task({
  name: "Python Specialist",
  instructions: "Implement authentication",
  agentType: "python-specialist",
  model: "opus"  // or "sonnet" or "haiku"
})

// Claude Code spawns subprocess with:
// - All environment variables inherited (including ANTHROPIC_BASE_URL)
// - ANTHROPIC_API_KEY or CLAUDE_CODE_OAUTH_TOKEN inherited
// - Subprocess makes API requests to ANTHROPIC_BASE_URL
// - Gateway receives request with agent_type and model in request body
// - Gateway routes based on agent_type and model tier
```

**Key Point**: We don't control agent spawning, but we don't need to! The gateway receives all the metadata it needs in the request body.

## Routing Logic Flow

```
Request arrives at gateway
    ↓
Extract metadata from request body:
    - agent_type: "python-specialist"
    - model: "claude-sonnet-4-5"
    - system prompt text (for inference)
    ↓
Routing engine determines provider:
    1. Check agent_type rules
       - "python-specialist" → deepseek
    2. Check model_tier rules
       - "sonnet" → anthropic (if no agent rule)
    3. Infer from system prompt
       - Contains "Chief Architect" → anthropic
    4. Default provider
       - anthropic
    ↓
Get provider and execute:
    - Anthropic provider: Use env var auth
    - Azure provider: Use configured auth
    - DeepSeek provider: Use configured auth
    ↓
Return response with metadata:
    - cost_usd (calculated)
    - latency_ms (measured)
    - provider (logged)
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_auth_priority_client_passthrough() {
        // Set env vars
        std::env::set_var("CLAUDE_CODE_OAUTH_TOKEN", "oauth-token");
        std::env::set_var("ANTHROPIC_API_KEY", "api-key");

        // Create provider with config API key
        let provider = create_test_provider("config-key").await;

        // Test: client_auth takes priority
        let (response, _) = provider.complete(
            test_request(),
            Some("client-auth-key".to_string()),
            None
        ).await.unwrap();

        // Verify client-auth-key was used
        assert!(was_used("client-auth-key"));
    }

    #[tokio::test]
    async fn test_auth_priority_oauth_env() {
        // Set CLAUDE_CODE_OAUTH_TOKEN
        std::env::set_var("CLAUDE_CODE_OAUTH_TOKEN", "oauth-token");

        // No client_auth provided
        let (response, _) = provider.complete(
            test_request(),
            None,
            None
        ).await.unwrap();

        // Verify oauth-token was used
        assert!(was_used("oauth-token"));
    }
}
```

### Integration Tests

1. **Full Flow Test**
   ```bash
   # Start daemon with gateway
   cargo run -- daemon start

   # Verify gateway port
   cat ~/.cco/daemon.pid

   # Test request via gateway
   curl -X POST http://localhost:PORT/v1/messages \
     -H "Content-Type: application/json" \
     -d '{
       "model": "claude-sonnet-4-5",
       "max_tokens": 100,
       "messages": [{"role": "user", "content": "Hello"}],
       "agent_type": "python-specialist"
     }'

   # Check routing
   # - python-specialist should route to deepseek
   # - Response should include provider="deepseek"
   ```

2. **Claude Code Integration**
   ```bash
   # Launch Claude Code via cco
   cco

   # Ask orchestrator to spawn agents
   # "Spawn Python specialist and Code reviewer agents"

   # Check gateway logs
   tail -f ~/.cco/logs/daemon.log | grep "Routing request"

   # Should see:
   # - python-specialist → deepseek
   # - code-reviewer → azure
   # - chief-architect → anthropic
   ```

## Rollback Plan

If the gateway approach doesn't work, we have a clean rollback:

```bash
# Set environment variable to force legacy proxy mode
export CCO_USE_LEGACY_PROXY=1

# Restart daemon
cco daemon restart

# This activates the HTTPS_PROXY fallback in launcher.rs (lines 405-423)
```

The legacy proxy code path is already implemented and tested. It has the streaming issue, but it's a known state.

## Summary

**Recommended Approach**: Option D - Gateway with Environment Variable Fallback

**Implementation**:
- Modify anthropic.rs to check environment variables when client_auth is missing
- Priority: client_auth → CLAUDE_CODE_OAUTH_TOKEN → ANTHROPIC_API_KEY → config

**Benefits**:
- ✅ Zero changes to Claude Code
- ✅ Full gateway functionality (cost tracking, audit, routing)
- ✅ Streaming works (SSE passthrough)
- ✅ Simple implementation (10-15 lines of code)
- ✅ Secure (env var inheritance)

**Time Estimate**:
- Implementation: 5 minutes
- Testing: 10 minutes
- Documentation: 5 minutes
- **Total: 20 minutes**

**Risk**: Low - falls back to existing behavior if env vars not set

**Next Steps**:
1. Implement auth fallback in anthropic.rs
2. Test with Claude Code
3. Verify streaming works
4. Document behavior in README
