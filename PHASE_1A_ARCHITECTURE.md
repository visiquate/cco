# PHASE 1A DETAILED ARCHITECTURE DOCUMENT

## Executive Summary
Phase 1a implements the core daemon skeleton with SSE client and metrics aggregation for the CCO Cost Monitor system. The daemon will be a separate Rust binary that connects to CCO's existing SSE endpoint at /api/stream.

## 1. DISCOVERED CCO INFRASTRUCTURE

### 1.1 Existing SSE Endpoint
- **Endpoint**: http://localhost:8080/api/stream
- **Event Type**: 'analytics'
- **Interval**: 5-second updates
- **Data Structure**:
  - project: {name, cost, tokens, calls, last_updated}
  - machine: {cpu, memory, uptime, process_count}
  - activity: Array of ActivityEvent objects
  - chart_data: Visualization data

### 1.2 ActivityEvent Structure (from analytics.rs)
```rust
pub struct ActivityEvent {
    pub timestamp: String,      // ISO 8601
    pub event_type: String,      // api_call, error, cache_hit, etc.
    pub agent_name: Option<String>,
    pub model: Option<String>,
    pub tokens: Option<u64>,
    pub latency_ms: Option<u64>,
    pub status: Option<String>,
    pub cost: Option<f64>,
}
```

## 2. PHASE 1A ARCHITECTURE

### 2.1 Project Structure
```
cco-cost-monitor/           # New crate alongside cco/
├── Cargo.toml
├── src/
│   ├── main.rs            # CLI entry point
│   ├── daemon/
│   │   ├── mod.rs         # Daemon orchestration
│   │   └── runtime.rs     # Tokio runtime management
│   ├── sse/
│   │   ├── mod.rs         # SSE module
│   │   ├── client.rs      # SSE client implementation
│   │   └── parser.rs      # Event parsing
│   ├── metrics/
│   │   ├── mod.rs         # Metrics module
│   │   ├── engine.rs      # Aggregation engine
│   │   └── types.rs       # Shared types
│   └── diagnostics/
│       ├── mod.rs         # Health/diagnostics
│       └── health.rs      # Health endpoint
```

### 2.2 Core Components

#### A. Daemon Runtime
- Tokio multi-threaded runtime
- Graceful shutdown handling (SIGTERM/SIGINT)
- PID file management at ~/.cco-monitor/daemon.pid
- Background/foreground modes

#### B. SSE Client
- Uses eventsource-client crate
- Connects to http://localhost:8080/api/stream
- Automatic reconnection with exponential backoff
- Parse 'analytics' events every 5 seconds

#### C. Metrics Engine
- In-memory aggregation using Arc<Mutex<MetricsState>>
- Model tier classification (opus/sonnet/haiku)
- Cost accumulation and projection
- Ring buffer for recent events (last 100)

#### D. Health Endpoint
- HTTP server on port 8081
- GET /health returns JSON status
- Includes: uptime, events_processed, connection_status

## 3. IMPLEMENTATION CHECKLIST

### 3.1 Core Daemon (Priority 1)
- [ ] Initialize Cargo project with dependencies
- [ ] Create main.rs with clap CLI parsing
- [ ] Implement daemon module with Tokio runtime
- [ ] Add signal handling for graceful shutdown
- [ ] Create PID file management
- [ ] Add logging with tracing subscriber

### 3.2 SSE Client (Priority 2)
- [ ] Implement SSE client with eventsource-client
- [ ] Parse SseStreamResponse JSON structure
- [ ] Handle connection errors and reconnection
- [ ] Extract ActivityEvent array from response
- [ ] Channel events to metrics engine via mpsc

### 3.3 Metrics Engine (Priority 3)
- [ ] Define MetricsState struct
- [ ] Implement aggregation logic
- [ ] Calculate costs per model tier
- [ ] Track hourly/daily accumulations
- [ ] Provide projection calculations

### 3.4 Health Diagnostics (Priority 4)
- [ ] Create mini HTTP server on 8081
- [ ] Implement /health endpoint
- [ ] Return JSON with daemon status
- [ ] Include metrics summary

## 4. DEPENDENCIES

### 4.1 External Crates Required
```toml
[dependencies]
tokio = { version = "1.42", features = ["full"] }
eventsource-client = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4.5", features = ["derive"] }
axum = "0.7"  # For health endpoint
```

### 4.2 No External Dependencies
- CCO proxy must be running locally
- No authentication required (localhost only)
- No internet connection needed

## 5. BLOCKING ISSUES & RISKS

### 5.1 Identified Blockers
- **NONE**: SSE endpoint exists and is functional
- **NONE**: Data structure is already well-defined
- **NONE**: No authentication needed for localhost

### 5.2 Potential Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| SSE connection drops | Medium | Exponential backoff reconnection |
| Memory growth | Low | Bounded buffers (100 events max) |
| CCO not running | High | Clear error message, retry logic |
| Port 8081 occupied | Low | Make port configurable via CLI |

## 6. TEST CASES

### 6.1 Unit Tests
- [ ] Daemon lifecycle (start/stop)
- [ ] PID file creation/cleanup
- [ ] Signal handler registration
- [ ] Metrics aggregation calculations
- [ ] Event parsing from JSON

### 6.2 Integration Tests
- [ ] SSE client connects to mock server
- [ ] Handles disconnection/reconnection
- [ ] Processes 100 events without memory growth
- [ ] Health endpoint returns valid JSON

### 6.3 Manual Validation
- [ ] Start daemon with CCO running
- [ ] Verify events received every 5 seconds
- [ ] Check health endpoint (curl http://localhost:8081/health)
- [ ] Graceful shutdown via Ctrl-C
- [ ] PID file cleaned up on exit

## 7. IMPLEMENTATION SEQUENCE

### Day 1: Project Setup & Daemon Core
1. Create new cco-cost-monitor crate
2. Set up dependencies in Cargo.toml
3. Implement basic daemon with signal handling
4. Add PID file management
5. Configure logging

### Day 2: SSE Client Implementation
1. Create SSE client module
2. Connect to CCO endpoint
3. Parse incoming events
4. Handle reconnection logic
5. Channel events to metrics

### Day 3: Metrics Engine
1. Define metrics data structures
2. Implement aggregation logic
3. Calculate model tier costs
4. Add projection calculations
5. Test with mock data

### Day 4: Health & Polish
1. Add health endpoint
2. Complete unit tests
3. Integration testing
4. Documentation
5. Manual validation

## 8. SUCCESS METRICS

Phase 1a is complete when:
- ✅ Daemon runs as background process
- ✅ Connects to CCO SSE endpoint
- ✅ Processes events without data loss
- ✅ Aggregates metrics in memory
- ✅ Health endpoint responds with status
- ✅ Graceful shutdown works
- ✅ All tests passing

## 9. DELIVERABLES

1. **cco-cost-monitor** binary
2. **Unit test suite** with >80% coverage
3. **Integration tests** for SSE client
4. **API documentation** (rustdoc)
5. **README.md** with usage instructions

---
Document Version: 1.0
Status: READY FOR IMPLEMENTATION
Next Step: Create cco-cost-monitor crate and begin Day 1 tasks