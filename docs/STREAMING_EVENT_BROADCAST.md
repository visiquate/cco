# Streaming Event Broadcast System

## Overview

The `StreamEventBroadcaster` provides real-time visibility into LLM streaming requests for the TUI and other monitoring tools. It uses Tokio's broadcast channel to efficiently deliver events to multiple subscribers.

## Architecture

```
┌─────────────────┐
│  LLM Gateway    │
│                 │
│  ┌───────────┐  │     ┌──────────────┐
│  │ Complete  │──┼────▶│  Broadcaster │──┬──▶ TUI Subscriber
│  │ Stream    │  │     │              │  │
│  └───────────┘  │     └──────────────┘  ├──▶ Metrics Subscriber
│                 │                        │
└─────────────────┘                        └──▶ Logging Subscriber
```

## File Location

**Implementation**: `/Users/brent/git/cc-orchestra/src/daemon/llm_gateway/sse_broadcast.rs`

## Event Types

The system broadcasts four types of events:

### 1. Started
Emitted when a streaming request begins.

```rust
TuiStreamEvent::Started {
    request_id: String,     // Unique request identifier
    model: String,          // Model being used (e.g., "claude-sonnet-4-5-20250929")
    agent_type: Option<String>, // Agent type if available
}
```

### 2. TextDelta
Emitted for each text chunk received from the stream.

```rust
TuiStreamEvent::TextDelta {
    request_id: String,     // Matches the Started event
    text: String,           // Incremental text content
}
```

### 3. Completed
Emitted when the stream completes successfully.

```rust
TuiStreamEvent::Completed {
    request_id: String,     // Matches the Started event
    input_tokens: u32,      // Total input tokens used
    output_tokens: u32,     // Total output tokens generated
    cost_usd: f64,          // Calculated cost in USD
}
```

### 4. Error
Emitted when a stream encounters an error.

```rust
TuiStreamEvent::Error {
    request_id: String,     // Matches the Started event
    message: String,        // Error description
}
```

## Integration with LlmGateway

The broadcaster is integrated into the `LlmGateway` struct:

```rust
pub struct LlmGateway {
    pub config: GatewayConfig,
    pub router: RoutingEngine,
    pub providers: ProviderRegistry,
    pub cost_tracker: Arc<CostTracker>,
    pub audit_logger: Arc<RwLock<AuditLogger>>,
    pub litellm_client: Option<LiteLLMClient>,
    pub stream_broadcaster: Arc<StreamEventBroadcaster>, // ← Added
}
```

The constructor initializes the broadcaster with default settings (1000 event capacity):
- `LlmGateway::new()` - Direct provider routing

## Usage Examples

### Subscribe to Events

```rust
use cco::daemon::llm_gateway::sse_broadcast::StreamEventBroadcaster;

// Get the broadcaster from the gateway
let broadcaster = gateway.stream_broadcaster.clone();

// Subscribe to events
let mut receiver = broadcaster.subscribe();

// Spawn a task to handle events
tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        match event {
            TuiStreamEvent::Started { request_id, model, agent_type } => {
                println!("🚀 Started: {} using {}", request_id, model);
            }
            TuiStreamEvent::TextDelta { request_id, text } => {
                print!("{}", text); // Stream text to console
            }
            TuiStreamEvent::Completed { request_id, input_tokens, output_tokens, cost_usd } => {
                println!("\n✅ Completed: {} ({} in, {} out, ${:.4})",
                    request_id, input_tokens, output_tokens, cost_usd);
            }
            TuiStreamEvent::Error { request_id, message } => {
                eprintln!("❌ Error: {} - {}", request_id, message);
            }
        }
    }
});
```

### Multiple Subscribers

The broadcaster supports multiple subscribers receiving the same events:

```rust
let broadcaster = gateway.stream_broadcaster.clone();

// TUI subscriber
let mut tui_receiver = broadcaster.subscribe();
tokio::spawn(async move {
    while let Ok(event) = tui_receiver.recv().await {
        update_tui_display(event);
    }
});

// Metrics subscriber
let mut metrics_receiver = broadcaster.subscribe();
tokio::spawn(async move {
    while let Ok(event) = metrics_receiver.recv().await {
        update_metrics(event);
    }
});

// Logging subscriber
let mut log_receiver = broadcaster.subscribe();
tokio::spawn(async move {
    while let Ok(event) = log_receiver.recv().await {
        log_streaming_event(event);
    }
});
```

### Sending Events (Internal Gateway Use)

Inside the gateway streaming logic:

```rust
// When stream starts
gateway.stream_broadcaster.send(TuiStreamEvent::Started {
    request_id: request_id.clone(),
    model: request.model.clone(),
    agent_type: request.agent_type.clone(),
});

// For each text chunk
gateway.stream_broadcaster.send(TuiStreamEvent::TextDelta {
    request_id: request_id.clone(),
    text: chunk_text,
});

// When completed
gateway.stream_broadcaster.send(TuiStreamEvent::Completed {
    request_id,
    input_tokens: usage.input_tokens,
    output_tokens: usage.output_tokens,
    cost_usd: calculated_cost,
});
```

## Event Serialization

All events are serializable to JSON for logging or transmission:

```json
// Started event
{
  "type": "started",
  "request_id": "req_abc123",
  "model": "claude-sonnet-4-5-20250929",
  "agent_type": "python-expert"
}

// TextDelta event
{
  "type": "text_delta",
  "request_id": "req_abc123",
  "text": "Hello, world!"
}

// Completed event
{
  "type": "completed",
  "request_id": "req_abc123",
  "input_tokens": 100,
  "output_tokens": 50,
  "cost_usd": 0.015
}

// Error event
{
  "type": "error",
  "request_id": "req_abc123",
  "message": "Connection timeout"
}
```

## Performance Characteristics

### Channel Capacity
- **Default**: 1000 events per subscriber
- **Configurable**: Can be adjusted via `StreamEventBroadcaster::new(capacity)`
- **Behavior**: If a subscriber falls behind and the buffer fills, older events are dropped (lagging receiver)

### Memory Overhead
- Each subscriber maintains its own buffer (copy-on-write semantics)
- Events are cloned per subscriber (relatively cheap due to small event size)
- No subscribers = no memory overhead (events are immediately dropped)

### Broadcasting Efficiency
- **Zero-copy** to sender (fire-and-forget)
- **Async-safe**: Non-blocking sends
- **Subscriber isolation**: Slow subscribers don't block fast ones

## Testing

The module includes comprehensive tests:

```bash
# Run all sse_broadcast tests
cargo test --lib sse_broadcast

# Tests covered:
# - Broadcaster creation and default capacity
# - Subscribe and receive events
# - Multiple subscribers receiving same events
# - Sending without subscribers (graceful handling)
# - Event serialization/deserialization
```

All tests pass successfully:
```
test daemon::llm_gateway::sse_broadcast::tests::test_broadcaster_creation ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_broadcaster_default ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_subscribe_and_receive ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_multiple_subscribers ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_send_without_subscribers ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_event_serialization ... ok
test daemon::llm_gateway::sse_broadcast::tests::test_event_deserialization ... ok
```

## Next Steps

### Integration with Gateway Streaming Handler

The next phase is to integrate the broadcaster into the actual streaming handlers:

1. **In `api.rs`**: Modify `handle_streaming_request()` to:
   - Send `Started` event when stream begins
   - Parse SSE events and send `TextDelta` for each text chunk
   - Send `Completed` or `Error` when stream ends

2. **In TUI**: Subscribe to events and display:
   - Active streams panel showing in-progress requests
   - Real-time text streaming (optional)
   - Completion metrics (tokens, cost, duration)

### Monitoring & Observability

The broadcaster enables:
- Real-time TUI dashboards showing active LLM requests
- Cost tracking as streams progress
- Error monitoring and alerting
- Performance profiling (latency per chunk)
- Request tracing across the system

## Configuration

Currently uses default configuration. Future enhancements may include:

```toml
[gateway.streaming]
broadcaster_capacity = 1000      # Events per subscriber
enable_tui_broadcast = true      # Toggle broadcasting
broadcast_text_deltas = true     # Include text content (can be verbose)
```

## Security Considerations

- **Text Content**: `TextDelta` events contain actual LLM output text
  - Consider privacy implications when broadcasting
  - May want to make text broadcasting optional
  - Sensitive requests should be flagged to skip broadcasting

- **Request Metadata**: `Started` events expose model and agent type
  - Generally safe for internal monitoring
  - Should not be exposed to external systems without filtering

## API Stability

This is an internal API for TUI/monitoring integration. The event structure is:
- **Stable**: Core event types (Started, TextDelta, Completed, Error)
- **Evolving**: May add new fields to events as needed
- **Versioned**: No breaking changes to existing fields

## References

- **Tokio Broadcast**: https://docs.rs/tokio/latest/tokio/sync/broadcast/
- **LLM Gateway Architecture**: `/Users/brent/git/cc-orchestra/docs/GATEWAY_STREAMING_VERIFICATION.md`
- **Streaming API**: `/Users/brent/git/cc-orchestra/src/daemon/llm_gateway/api.rs`
