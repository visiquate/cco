# LLM Gateway LiteLLM Integration

## Overview

The LLM Gateway has been refactored to optionally route requests through LiteLLM instead of calling providers directly. This provides a flexible architecture where the gateway can work with or without LiteLLM.

## Architecture

### Before (Direct Provider Mode)
```
Request → Gateway API → Router → Provider (Anthropic/Azure/DeepSeek/Ollama) → LLM
```

### After (LiteLLM Mode)
```
Request → Gateway API → Router → LiteLLM Client → LiteLLM Proxy → LLM
```

### Hybrid Mode
The gateway supports **both modes simultaneously**:
- If `LiteLLMClient` is configured, ALL requests route through LiteLLM
- If `LiteLLMClient` is NOT configured, requests use direct providers (backwards compatible)

## Key Design Decisions

### 1. Optional LiteLLM Client
- `LlmGateway` has an `Option<LiteLLMClient>` field
- Two constructors:
  - `new()` - Creates gateway with direct providers (existing behavior)
  - `new_with_litellm()` - Creates gateway with LiteLLM client

### 2. Preserved Routing Logic
- The Rust-based routing engine (`RoutingEngine`) is **still used** for agent-type routing decisions
- Routing rules (agent types, model tiers, fallback chains) remain in Rust
- LiteLLM handles the actual LLM provider communication

### 3. Preserved Metrics & Audit
- Cost tracking continues to work (using gateway's `CostTracker`)
- Audit logging continues to work (using gateway's `AuditLogger`)
- Request metrics are calculated at the gateway level
- Provider field is set to "litellm" when using LiteLLM mode

### 4. Backwards Compatibility
- Existing code using `LlmGateway::new()` continues to work unchanged
- Direct provider implementations are NOT deleted
- Migration to LiteLLM is opt-in

## Changes Made

### `/Users/brent/git/cc-orchestra/src/daemon/llm_gateway/mod.rs`

**Added:**
- `use self::litellm_client::LiteLLMClient;`
- `litellm_client: Option<LiteLLMClient>` field to `LlmGateway` struct
- `new_with_litellm(config: GatewayConfig, litellm_url: &str)` constructor

**Modified:**
- `complete()` method now checks if LiteLLM client is available
- If LiteLLM is available, routes through `litellm.complete()`
- If not, falls back to direct provider via `provider.complete()`
- Provider name in metrics/audit is "litellm" when using LiteLLM

### `/Users/brent/git/cc-orchestra/src/daemon/llm_gateway/api.rs`

**Modified:**
- `handle_streaming_request()` now checks if LiteLLM client is available
- If LiteLLM is available, routes through `litellm.complete_stream()`
- If not, falls back to direct provider streaming
- Streaming responses work identically in both modes (SSE passthrough)

## Usage Examples

### Using LiteLLM (New)

```rust
use cco::daemon::llm_gateway::{LlmGateway, config::load_from_orchestra_config};

// Load config
let gateway_config = load_from_orchestra_config(None)?;

// Create gateway with LiteLLM
let gateway = LlmGateway::new_with_litellm(
    gateway_config,
    "http://localhost:4000"  // LiteLLM proxy URL
).await?;

// Make requests (automatically routed through LiteLLM)
let response = gateway.complete(request, auth, beta).await?;
```

### Using Direct Providers (Existing)

```rust
use cco::daemon::llm_gateway::{LlmGateway, config::load_from_orchestra_config};

// Load config
let gateway_config = load_from_orchestra_config(None)?;

// Create gateway with direct providers
let gateway = LlmGateway::new(gateway_config).await?;

// Make requests (routed to providers directly)
let response = gateway.complete(request, auth, beta).await?;
```

## Authentication Passthrough

The gateway correctly passes through authentication headers to LiteLLM:
- `Authorization` header or `x-api-key` header → passed as `client_auth`
- `anthropic-beta` header → passed as `client_beta`

LiteLLM receives these headers and forwards them to the actual provider.

## Cost Tracking

Cost tracking works in both modes:

### LiteLLM Mode
1. LiteLLM returns usage info in the response
2. Gateway calculates cost using `calculate_cost()` in `litellm_client.rs`
3. Gateway records metrics via `cost_tracker.record()`
4. Provider field is "litellm"

### Direct Provider Mode
1. Provider returns usage info in the response
2. Provider calculates cost using its own pricing
3. Gateway records metrics via `cost_tracker.record()`
4. Provider field is actual provider name (e.g., "anthropic")

## Streaming Support

Both modes support streaming (SSE):

### LiteLLM Mode
1. Gateway calls `litellm_client.complete_stream()`
2. LiteLLM client sends POST to `/v1/messages` with `stream: true`
3. Returns `ByteStream` of SSE events
4. Gateway wraps in axum `Body::from_stream()`
5. Client receives SSE events in Anthropic format

### Direct Provider Mode
1. Gateway calls `provider.complete_stream()`
2. Provider makes streaming request to LLM
3. Returns `ByteStream` of SSE events
4. Gateway wraps in axum `Body::from_stream()`
5. Client receives SSE events in Anthropic format

## Testing

### Manual Testing (LiteLLM Mode)

1. Start LiteLLM proxy:
```bash
litellm --config config/litellm_config.yaml --port 4000
```

2. Start gateway with LiteLLM:
```bash
# Modify server.rs to use new_with_litellm() instead of new()
cargo run -- daemon start
```

3. Make request:
```bash
curl http://localhost:GATEWAY_PORT/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 100,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### Manual Testing (Direct Provider Mode)

1. Start gateway (existing behavior):
```bash
cargo run -- daemon start
```

2. Make same request as above - should work identically

## Migration Path

To migrate from direct providers to LiteLLM:

1. Set up LiteLLM proxy with your provider credentials
2. Change gateway instantiation:
   ```rust
   // From:
   let gateway = LlmGateway::new(config).await?;

   // To:
   let gateway = LlmGateway::new_with_litellm(config, "http://localhost:4000").await?;
   ```
3. No other code changes needed
4. Cost tracking, audit logging, and routing all work the same

## Benefits of LiteLLM Integration

1. **Centralized Provider Management**: Add/remove providers in LiteLLM config
2. **Advanced Routing**: LiteLLM provides load balancing, retries, fallbacks
3. **Additional Providers**: Easy to add OpenAI, Cohere, etc. via LiteLLM
4. **Cost Tracking**: LiteLLM provides additional cost tracking layer
5. **Monitoring**: LiteLLM dashboard for request monitoring
6. **Caching**: LiteLLM supports response caching

## Preserved Gateway Value

Even with LiteLLM, the Rust gateway still provides:

1. **Agent-Type Routing**: Intelligent routing based on task type
2. **Cost Attribution**: Per-agent, per-project cost tracking
3. **Audit Logging**: Detailed request/response logging with SQLite
4. **API Compatibility**: Anthropic-compatible API surface
5. **Integration**: Seamless integration with cco orchestrator

## Future Enhancements

1. **Auto-Discovery**: Detect if LiteLLM is running and auto-configure
2. **Fallback**: If LiteLLM is down, fall back to direct providers
3. **Metrics Comparison**: Compare costs/latency between LiteLLM and direct
4. **Configuration**: Add `litellm_url` to gateway config file
