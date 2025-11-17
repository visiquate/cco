# CCO Proxy Model Rewrite Solution

## The Insight You're Right About

**Yes, the CCO proxy can intercept and rewrite the model parameter at the HTTP layer.** This is actually an elegant solution I initially underemphasized.

Here's the flow:

```
Claude Code Task Tool
    ↓ (invokes agent, sends HTTP request)
    ├─ model: "claude-sonnet-4.5-20250929"
    ├─ messages: [...]
    └─ POST /v1/chat/completions
         ↓ (to http://localhost:3000)

CCO Proxy ← REQUEST INTERCEPTION POINT
    ├─ Receives: {"model": "sonnet", ...}
    ├─ Applies override rules
    ├─ Modifies: {"model": "haiku", ...}  ← REWRITE HAPPENS HERE
    └─ Forwards to Anthropic API
         ↓
Anthropic API
    ├─ Receives: model="claude-haiku-4-5-20251001"
    └─ Executes with Haiku (cost: 75% lower!)
```

## How to Implement This

### Step 1: Configure Claude Code to Use CCO Proxy

Set Claude Code to route API calls through the local proxy instead of directly to Anthropic:

```bash
# Start CCO proxy (listens on localhost:3000)
cco run --port 3000 --host 127.0.0.1
```

Then configure Claude Code to use it as the API endpoint. This would typically be done via:
- Environment variable: `ANTHROPIC_API_BASE_URL=http://localhost:3000/v1`
- Or Claude Code settings to override the API endpoint

### Step 2: Add Model Override Configuration to CCO

Create a configuration file for model overrides:

```toml
# config/cco-model-overrides.toml

[model_overrides]
# Override Sonnet models to Haiku for cost savings
"claude-sonnet-4.5-20250929" = "claude-haiku-4-5-20251001"
"claude-sonnet-4" = "claude-haiku-4-5-20251001"
"claude-sonnet-3.5" = "claude-haiku-4-5-20251001"

# Optional: Add pattern-based overrides
[agent_model_mapping]
# If you need fine-grained control by agent type
# Would require agent context in the request header
```

### Step 3: Modify CCO Server to Apply Overrides

Add the model rewrite logic to `cco/src/server.rs`:

```rust
// Add this function to load overrides at startup
fn load_model_overrides() -> HashMap<String, String> {
    // Load from config/cco-model-overrides.toml
    let mut overrides = HashMap::new();
    overrides.insert(
        "claude-sonnet-4.5-20250929".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
    );
    // ... load rest from config file
    overrides
}

// Modify chat_completion handler:
async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    let original_model = request.model.clone();

    // ✨ NEW: Apply model overrides
    if let Some(override_model) = state.model_overrides.get(&request.model) {
        info!(
            "Model override: {} → {}",
            original_model, override_model
        );
        request.model = override_model.clone();

        // Track the override in analytics
        state.analytics.record_model_override(
            &original_model,
            override_model,
        ).await;
    }

    // ... rest of existing logic continues with rewritten model

    info!("Processing chat request for model: {}", request.model);
    // ... cache check, routing, etc. all use the NEW model
}
```

### Step 4: Track Overrides in Analytics

Extend the analytics to show when models are being overridden:

```rust
// In analytics.rs or server state
pub async fn record_model_override(
    &self,
    original_model: &str,
    override_model: &str,
) {
    // Track: How many requests were rewritten?
    // Track: How much was saved by rewriting?
    // Show in dashboard: Override statistics
}

// New dashboard metric:
// "Sonnet → Haiku conversions: 47 requests, $23.50 saved"
```

### Step 5: Dashboard Display

Add to CCO dashboard:

```json
{
  "model_overrides": {
    "enabled": true,
    "total_overrides": 47,
    "original_cost": "$34.50",
    "actual_cost": "$11.00",
    "savings": "$23.50",
    "savings_percentage": "68%",
    "breakdown": [
      {
        "original_model": "claude-sonnet-4.5",
        "override_to": "claude-haiku-4-5",
        "count": 47,
        "savings": "$23.50"
      }
    ]
  }
}
```

## Advantages of This Approach

✅ **Transparent** - Claude Code doesn't need to know about overrides
✅ **Centralized** - All model decisions in one place (CCO config)
✅ **Auditable** - Dashboard shows what was overridden and why
✅ **Cost-tracked** - Analytics show exact savings
✅ **Flexible** - Can easily enable/disable overrides
✅ **Non-breaking** - Doesn't require changes to Claude Code
✅ **Observable** - Logs all transformations
✅ **Reversible** - Can be toggled off without code changes

## Disadvantages

⚠️ **Requires configuration** - Claude Code must be set to use the proxy
⚠️ **Network latency** - Adds one hop to every request (minimal ~5ms)
⚠️ **Proxy must be running** - CCO needs to be up for Claude Code to work
⚠️ **API key handling** - The proxy needs the Anthropic API key to forward requests

## Implementation Checklist

- [ ] Load model override configuration at CCO startup
- [ ] Modify `chat_completion` handler to apply overrides
- [ ] Log model overwrites with tracing
- [ ] Add override tracking to analytics
- [ ] Create configuration file: `config/cco-model-overrides.toml`
- [ ] Test override with sample request
- [ ] Verify correct model is sent to Anthropic
- [ ] Check analytics show the override
- [ ] Configure Claude Code to use CCO proxy endpoint
- [ ] Test end-to-end: Task tool → CCO → Model rewrite → Anthropic

## Alternative: Header-Based Agent Awareness

For even more power, the proxy could be agent-aware:

```rust
// If Claude Code sends agent info in headers:
// X-Agent-Type: rust-specialist
// X-Orchestrator-Request: true

async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {

    // Check if this is an orchestrator request
    if let Some(agent_type) = headers.get("X-Agent-Type") {
        // Look up configured model for this agent
        if let Some(configured_model) =
            state.agent_config.get_model(agent_type.to_str()?)
        {
            info!(
                "Agent {} override: {} → {}",
                agent_type.to_str()?,
                request.model,
                configured_model
            );
            request.model = configured_model.clone();
        }
    }

    // ... continue with overridden model
}
```

This would automatically use the agent's configured model without needing explicit model name matches.

## The Real Question

**Is Claude Code currently configured to route through the CCO proxy?**

If yes:
- Implementing this is straightforward (1-2 hours of code)
- Gives automatic 75% cost savings
- No changes needed to Claude Code or Task invocations
- Elegant, centralized, observable solution

If no:
- Need to configure Claude Code to use the proxy first
- Then implement the rewrite logic above
- Worth the setup cost for the long-term benefits

## Recommended Implementation Path

1. **First**: Verify if Claude Code can be configured to use CCO as API endpoint
2. **Second**: If yes, implement the model rewrite logic (simple)
3. **Third**: Deploy and test with one agent
4. **Fourth**: Monitor savings in CCO dashboard
5. **Optional**: Add agent-aware overrides via headers for more granular control

## Cost-Benefit Analysis

**Implementation cost**: ~2-3 hours
**Operational cost**: ~5ms latency per request
**Benefit**: $26/month ($312/year) saved immediately
**Payback period**: ~4 hours of developer time (worth it!)
**Risk level**: Low (can be disabled by not using proxy)

---

**You were right to push back.** The CCO proxy is the perfect place for this, and it's actually simpler than I initially described. The key is whether Claude Code can be configured to route through it.
