# TUI Streaming Implementation Documentation

## Overview

This document describes the implementation for displaying real-time LLM streaming events in the TUI (Terminal User Interface). The system allows the TUI to subscribe to and display active streaming requests with live text previews.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          LLM Gateway                             │
│  ┌────────────────────────────────────────────────────────┐    │
│  │         StreamEventBroadcaster (Tokio Broadcast)       │    │
│  │  - Started: { request_id, model, agent_type }         │    │
│  │  - TextDelta: { request_id, text }                    │    │
│  │  - Completed: { request_id, tokens, cost }            │    │
│  │  - Error: { request_id, message }                     │    │
│  └────────────────────────────────────────────────────────┘    │
│                           │                                      │
│                           │ broadcasts events                    │
│                           ▼                                      │
│  ┌────────────────────────────────────────────────────────┐    │
│  │         /api/streams/subscribe (SSE Endpoint)          │    │
│  │  - HTTP SSE stream (text/event-stream)                │    │
│  │  - JSON-serialized TuiStreamEvent messages            │    │
│  └────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                           │
                           │ SSE connection
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                          TUI Application                         │
│  ┌────────────────────────────────────────────────────────┐    │
│  │              Event Processing Loop                      │    │
│  │  - Subscribes to SSE endpoint on startup               │    │
│  │  - Processes TuiStreamEvent messages                   │    │
│  │  - Updates active_streams HashMap                      │    │
│  └────────────────────────────────────────────────────────┘    │
│                           │                                      │
│                           │ updates                              │
│                           ▼                                      │
│  ┌────────────────────────────────────────────────────────┐    │
│  │         Active Streams Display Panel                    │    │
│  │  [LIVE 5s] claude-sonnet | agent: python | Hello...   │    │
│  │  [LIVE 12s] claude-haiku | agent: test | Testing...    │    │
│  └────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. StreamEventBroadcaster (`src/daemon/llm_gateway/sse_broadcast.rs`)

**Purpose**: Broadcast streaming events to multiple subscribers using Tokio's broadcast channel.

**Events**:
- `Started`: New stream initiated with metadata
- `TextDelta`: Incremental text content from the stream
- `Completed`: Stream finished successfully with token usage and cost
- `Error`: Stream encountered an error

**Key Methods**:
- `new(capacity: usize)`: Create broadcaster with buffer capacity
- `subscribe()`: Get a receiver for events
- `send(event: TuiStreamEvent)`: Broadcast event to all subscribers

### 2. ActiveStream Tracking (`src/tui_app.rs`)

**Data Structure**:
```rust
pub struct ActiveStream {
    pub request_id: String,
    pub model: String,
    pub agent_type: Option<String>,
    pub accumulated_text: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}
```

**Storage**: `HashMap<String, ActiveStream>` in `AppState::Connected`

**Event Processing**:
```rust
fn handle_stream_event(&mut self, event: TuiStreamEvent) {
    match event {
        Started { request_id, model, agent_type } => {
            // Create new ActiveStream entry
        }
        TextDelta { request_id, text } => {
            // Append text to accumulated_text
        }
        Completed { request_id, .. } | Error { request_id, .. } => {
            // Remove stream from active_streams
        }
    }
}
```

### 3. Rendering (`src/tui_app.rs`)

**Display Panel**:
- Title: "Active Streams (N live)"
- Green border (indicates live activity)
- Shows up to 5 most recent streams
- Each line: `[LIVE {duration}] {model} | agent: {agent_type} | {text_preview}`

**Layout Integration**:
- Dynamic height: `3 + min(stream_count, 5)` lines
- Positioned between Overall Summary and Project Summaries
- Automatically hidden when no active streams

**Rendering Logic** (`render_active_streams`):
1. Sort streams by start_time (most recent first)
2. Calculate elapsed duration
3. Truncate model name if > 22 chars
4. Show first 60 chars of accumulated text
5. Display in green with bold styling

## Implementation Status

### ✅ Completed

1. **Data Structures**
   - `ActiveStream` struct for tracking stream state
   - `active_streams` HashMap in `AppState::Connected`
   - Event processing logic in `handle_stream_event()`

2. **Rendering**
   - `render_active_streams()` method
   - Dynamic layout with conditional display
   - Duration calculation and text preview formatting

3. **State Management**
   - Preserve `active_streams` across state updates
   - Handle time range cycling without losing streams
   - Support for multiple concurrent streams

4. **Event Types**
   - `TuiStreamEvent` enum with all event types
   - Serialization/deserialization support
   - Integration with `StreamEventBroadcaster`

### ⏳ TODO: Wire Up SSE Connection

The TUI currently has all the infrastructure to display streams, but needs to be connected to the daemon's SSE endpoint.

**Required Steps**:

1. **Add SSE Endpoint to Daemon** (`src/daemon/llm_gateway/api.rs`):
   ```rust
   pub async fn subscribe_to_streams(
       State(gateway): State<GatewayState>,
   ) -> impl IntoResponse {
       let receiver = gateway.stream_broadcaster.subscribe();

       let stream = async_stream::stream! {
           let mut rx = receiver;
           while let Ok(event) = rx.recv().await {
               let json = serde_json::to_string(&event).unwrap();
               yield Ok::<_, std::convert::Infallible>(
                   axum::response::sse::Event::default()
                       .event("stream_event")
                       .data(json)
               );
           }
       };

       axum::response::Sse::new(stream)
   }
   ```

2. **Add Route** (`src/daemon/llm_gateway/api.rs`):
   ```rust
   .route("/api/streams/subscribe", get(subscribe_to_streams))
   ```

3. **TUI Subscription** (`src/tui_app.rs` in `run()` method):
   ```rust
   // Spawn SSE subscriber task
   let (stream_tx, mut stream_rx) = mpsc::channel::<TuiStreamEvent>(100);
   let base_url = self.client.base_url.clone();

   tokio::spawn(async move {
       let url = format!("{}/api/streams/subscribe", base_url);

       loop {
           match reqwest::Client::new()
               .get(&url)
               .send()
               .await
           {
               Ok(response) => {
                   let mut stream = response.bytes_stream();

                   while let Some(chunk) = stream.next().await {
                       if let Ok(bytes) = chunk {
                           if let Ok(text) = std::str::from_utf8(&bytes) {
                               // Parse SSE format
                               for line in text.lines() {
                                   if line.starts_with("data: ") {
                                       let json = &line[6..];
                                       if let Ok(event) = serde_json::from_str(json) {
                                           let _ = stream_tx.send(event).await;
                                       }
                                   }
                               }
                           }
                       }
                   }
               }
               Err(_) => {
                   // Retry after delay
                   tokio::time::sleep(Duration::from_secs(5)).await;
               }
           }
       }
   });
   ```

4. **Process Stream Events in Main Loop** (`src/tui_app.rs`):
   ```rust
   // In the main event loop, add:
   match stream_rx.try_recv() {
       Ok(event) => {
           self.handle_stream_event(event);
       }
       Err(TryRecvError::Empty) => {
           // No events, continue
       }
       Err(TryRecvError::Disconnected) => {
           // Reconnect needed
       }
   }
   ```

## Event Flow Example

### Scenario: Python Expert Agent Generates Code

```
1. User requests code generation
   └─> LLM Gateway receives CompletionRequest
       └─> Broadcasts: Started {
           request_id: "req_123",
           model: "claude-sonnet-4-5",
           agent_type: Some("python-expert")
       }

2. TUI receives Started event
   └─> Creates ActiveStream entry
       └─> Display shows: [LIVE 0s] claude-sonnet-4-5 | agent: python-expert |

3. Stream generates text "def calculate..."
   └─> Gateway broadcasts: TextDelta {
           request_id: "req_123",
           text: "def calculate"
       }

4. TUI receives TextDelta
   └─> Appends to accumulated_text
       └─> Display updates: [LIVE 2s] claude-sonnet-4-5 | agent: python-expert | def calculate...

5. Stream continues with more deltas...
   └─> TUI continuously updates display with growing text preview

6. Stream completes
   └─> Gateway broadcasts: Completed {
           request_id: "req_123",
           input_tokens: 1000,
           output_tokens: 500,
           cost_usd: 0.015
       }

7. TUI receives Completed
   └─> Removes stream from active_streams
       └─> Display panel shrinks or hides if no other active streams
```

## Performance Considerations

### Memory Management
- **Text Accumulation**: Only stores full text for active streams (max 5 displayed)
- **Auto-Cleanup**: Streams removed on completion/error
- **Preview Truncation**: Only first 60 chars displayed (full text still stored)

### Broadcast Channel
- **Capacity**: Default 1000 events
- **Lagging Receivers**: Old events dropped if TUI falls behind
- **Multiple Subscribers**: Supports future additions (monitoring tools, etc.)

### Rendering Efficiency
- **Conditional Display**: Panel hidden when no active streams
- **Dynamic Height**: Only allocates space for actual stream count
- **Update Frequency**: Tied to TUI refresh rate (200ms polling)

## Testing

### Manual Testing Scenario

1. **Start TUI**: `cco tui`
2. **Trigger Streaming Request**: In another terminal:
   ```bash
   curl -X POST http://localhost:3030/v1/messages \
     -H "Content-Type: application/json" \
     -d '{
       "model": "claude-sonnet-4-5-20250929",
       "messages": [{"role": "user", "content": "Write a Python function"}],
       "max_tokens": 100,
       "stream": true,
       "agent_type": "python-expert"
     }'
   ```
3. **Observe TUI**: Should show active stream panel with live text updates
4. **Verify Cleanup**: Panel should disappear when stream completes

### Unit Tests

Key test scenarios:
- `handle_stream_event()` correctly updates active_streams
- Multiple concurrent streams tracked properly
- Text accumulation works correctly
- Cleanup on completion/error
- Rendering with various stream counts

## Error Handling

### Stream Connection Loss
- TUI continues to function (streams just won't update)
- Auto-reconnect logic retries connection
- Stale streams timeout after 5 minutes

### Malformed Events
- Invalid JSON: Log warning, skip event
- Unknown event type: Ignore gracefully
- Missing required fields: Skip event

### Display Overflow
- Model names truncated to 22 chars
- Text preview limited to 60 chars
- Maximum 5 streams shown (most recent)

## Future Enhancements

1. **Stream Filters**: Filter by agent_type or model
2. **Detailed View**: Key binding to expand stream details
3. **History**: Show recently completed streams
4. **Metrics**: Track average stream duration, token rate
5. **Notifications**: Sound/visual alert for new streams
6. **Export**: Save stream text to file

## Dependencies

### Rust Crates
- `tokio::sync::broadcast`: Event broadcasting
- `serde_json`: Event serialization
- `chrono`: Timestamp handling
- `ratatui`: TUI rendering
- `reqwest`: (TODO) HTTP SSE client

### Internal Modules
- `daemon::llm_gateway`: Streaming infrastructure
- `api_client`: Daemon API communication
- `tui_app`: Display and event handling

## References

- SSE Broadcast System: `src/daemon/llm_gateway/sse_broadcast.rs`
- TUI Application: `src/tui_app.rs`
- LLM Gateway: `src/daemon/llm_gateway/mod.rs`
- Gateway API: `src/daemon/llm_gateway/api.rs`
