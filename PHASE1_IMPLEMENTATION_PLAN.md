# Phase 1 Implementation Plan: Daemon-Based Cost Monitoring System

## Executive Summary

This document outlines the comprehensive implementation plan for Phase 1 of the daemon-based API cost monitoring system for Claude Code Orchestra (CCO). Phase 1 delivers a production-ready, self-contained binary that provides real-time cost tracking and monitoring through a native daemon service with TUI dashboard.

## 1. Architecture Overview

### 1.1 System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CCO Proxy (Existing)                     │
│                 Emits cost events via SSE                   │
└────────────────────┬────────────────────────────────────────┘
                     │ SSE Events
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Cost Monitor Daemon (Rust)                 │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Event Listener│  │Metrics Engine│  │   SQLite     │     │
│  │  (SSE Client)│──▶│ (Aggregation)│──▶│ Persistence │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         │                   │                 │             │
│         └───────────────────┼─────────────────┘             │
│                             ▼                               │
│                    ┌──────────────┐                        │
│                    │ TUI Dashboard│                        │
│                    │  (Ratatui)   │                        │
│                    └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
                    ┌──────────────┐
                    │   Terminal   │
                    │   Display    │
                    └──────────────┘
```

### 1.2 Event Flow

1. **Event Reception**: SSE client connects to CCO proxy at `http://localhost:8080/api/sse/analytics`
2. **Event Parsing**: JSON events parsed into structured metrics
3. **Real-time Processing**: Metrics engine aggregates data in memory
4. **Persistence**: Periodic flush to SQLite database (every 5 seconds)
5. **UI Updates**: TUI refreshes display at 100ms intervals for smooth visualization
6. **Recovery**: On restart, load historical data from SQLite

### 1.3 Data Flow

```json
// SSE Event from CCO Proxy
{
  "type": "api_request",
  "timestamp": "2025-11-17T12:00:00Z",
  "model": "claude-3-5-sonnet-20250929",
  "input_tokens": 1500,
  "output_tokens": 800,
  "cost": 0.00315,
  "cache_read_tokens": 0,
  "cache_write_tokens": 0,
  "provider": "anthropic",
  "status": "success",
  "latency_ms": 1250
}
```

## 2. Technology Stack

### 2.1 Core Dependencies

```toml
[dependencies]
# Async Runtime
tokio = { version = "1.42", features = ["full"] }

# TUI Framework
ratatui = "0.29"
crossterm = "0.28"

# Database
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "chrono"] }

# SSE Client
eventsource-client = "0.13"
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# Platform-specific daemon management
[target.'cfg(target_os = "macos")'.dependencies]
launchd = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows-service = "0.7"
```

### 2.2 Build Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## 3. File Structure

```
cco-cost-monitor/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs                 # Entry point, CLI handling
│   ├── daemon/
│   │   ├── mod.rs              # Daemon lifecycle management
│   │   ├── service.rs          # Platform-specific service impl
│   │   └── config.rs           # Daemon configuration
│   ├── events/
│   │   ├── mod.rs              # Event processing module
│   │   ├── listener.rs         # SSE client implementation
│   │   ├── parser.rs           # Event parsing and validation
│   │   └── types.rs            # Event data structures
│   ├── metrics/
│   │   ├── mod.rs              # Metrics engine module
│   │   ├── aggregator.rs       # Real-time aggregation
│   │   ├── calculator.rs       # Cost calculations
│   │   └── cache.rs            # In-memory metrics cache
│   ├── database/
│   │   ├── mod.rs              # Database module
│   │   ├── schema.rs           # SQLite schema definitions
│   │   ├── repository.rs       # Data access layer
│   │   └── migrations.rs       # Database migrations
│   ├── tui/
│   │   ├── mod.rs              # TUI module
│   │   ├── app.rs              # Application state
│   │   ├── ui.rs               # UI rendering
│   │   ├── widgets/
│   │   │   ├── mod.rs          # Custom widgets
│   │   │   ├── live_monitor.rs # Live API call monitor
│   │   │   ├── cost_summary.rs # Cost summary widget
│   │   │   ├── model_chart.rs  # Model tier breakdown chart
│   │   │   └── call_list.rs    # Recent calls list
│   │   └── input.rs            # Keyboard input handling
│   └── utils/
│       ├── mod.rs              # Utility functions
│       ├── error.rs            # Custom error types
│       └── formatting.rs       # Display formatters
├── tests/
│   ├── integration/
│   │   ├── daemon_test.rs
│   │   ├── metrics_test.rs
│   │   └── database_test.rs
│   └── fixtures/
│       └── sample_events.json
└── scripts/
    ├── install.sh              # macOS/Linux installation
    └── install.ps1             # Windows installation
```

## 4. SQLite Database Schema

### 4.1 Core Tables

```sql
-- API metrics table
CREATE TABLE api_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL,
    model TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'anthropic',
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER DEFAULT 0,
    cache_write_tokens INTEGER DEFAULT 0,
    cost_usd REAL NOT NULL,
    latency_ms INTEGER,
    status TEXT NOT NULL,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Model tier aggregations (hourly)
CREATE TABLE hourly_aggregations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hour_timestamp DATETIME NOT NULL,
    model_tier TEXT NOT NULL, -- 'opus', 'sonnet', 'haiku'
    total_requests INTEGER NOT NULL,
    total_input_tokens INTEGER NOT NULL,
    total_output_tokens INTEGER NOT NULL,
    total_cache_tokens INTEGER NOT NULL,
    total_cost_usd REAL NOT NULL,
    avg_latency_ms INTEGER,
    error_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(hour_timestamp, model_tier)
);

-- Daily summaries
CREATE TABLE daily_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL UNIQUE,
    total_requests INTEGER NOT NULL,
    total_cost_usd REAL NOT NULL,
    opus_cost_usd REAL DEFAULT 0,
    sonnet_cost_usd REAL DEFAULT 0,
    haiku_cost_usd REAL DEFAULT 0,
    cache_savings_usd REAL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Session tracking
CREATE TABLE monitoring_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    started_at DATETIME NOT NULL,
    ended_at DATETIME,
    total_events INTEGER DEFAULT 0,
    total_cost_usd REAL DEFAULT 0,
    daemon_version TEXT NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_api_metrics_timestamp ON api_metrics(timestamp);
CREATE INDEX idx_api_metrics_model ON api_metrics(model);
CREATE INDEX idx_hourly_aggregations_lookup ON hourly_aggregations(hour_timestamp, model_tier);
CREATE INDEX idx_daily_summaries_date ON daily_summaries(date);
```

### 4.2 Data Retention Policy

- **Raw metrics**: 30 days (configurable)
- **Hourly aggregations**: 90 days
- **Daily summaries**: Indefinite
- **Auto-vacuum**: Daily at 3 AM local time

## 5. Implementation Phases

### Phase 1a: Core Daemon & Metrics Engine (Week 1)

**Components**:
- Basic daemon lifecycle (start/stop/status)
- SSE event listener connecting to CCO proxy
- Event parser and validator
- In-memory metrics aggregation
- Basic logging infrastructure

**Deliverables**:
- `daemon/` module with service management
- `events/` module with SSE client
- `metrics/` module with aggregator
- Unit tests for core components

**Success Criteria**:
- Daemon starts and connects to CCO proxy
- Successfully receives and parses SSE events
- Aggregates metrics in memory
- Graceful shutdown on SIGTERM/SIGINT

### Phase 1b: SQLite Persistence & Recovery (Week 2)

**Components**:
- SQLite database initialization
- Schema creation and migrations
- Repository pattern for data access
- Periodic flush from memory to disk
- Recovery on startup

**Deliverables**:
- `database/` module with SQLx integration
- Migration system for schema updates
- Data access layer with async operations
- Integration tests for persistence

**Success Criteria**:
- Database created on first run
- Metrics persisted every 5 seconds
- Historical data loaded on startup
- No data loss on daemon restart

### Phase 1c: TUI Dashboard & Real-time Updates (Week 3)

**Components**:
- Ratatui TUI framework setup
- Four-panel layout implementation
- Real-time data binding
- Keyboard navigation
- Color-coded model tiers

**Deliverables**:
- `tui/` module with dashboard implementation
- Custom widgets for each panel
- Responsive layout system
- Accessibility features (high contrast mode)

**Dashboard Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ CCO Cost Monitor v1.0.0        [Q]uit [R]efresh [H]elp      │
├─────────────────────────────────────────────────────────────┤
│ Live Monitor                 │ Cost Summary                 │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━ │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ Status: ● Connected          │ Today:        $12.45         │
│ Events/sec: 3.2              │ This Hour:    $2.31          │
│ Current Model: Sonnet 3.5    │ Last 5 min:   $0.87          │
│ Last Update: 12:34:56        │                              │
│                              │ Projections:                 │
│ Current Request:             │ Hour:  $2.75                 │
│ Input:  1,234 tokens         │ Day:   $14.20                │
│ Output: 567 tokens           │ Month: $426.00               │
│ Cost:   $0.0234              │                              │
├──────────────────────────────┼──────────────────────────────┤
│ Model Tier Breakdown         │ Recent API Calls (25)        │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━ │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                              │ 12:34:56 Sonnet  $0.023 ✓   │
│ Opus 4.1    ████  15% $1.87 │ 12:34:52 Haiku   $0.002 ✓   │
│ Sonnet 3.5  ████████  40%   │ 12:34:48 Sonnet  $0.018 ✓   │
│             $4.98            │ 12:34:45 Opus    $0.087 ✓   │
│ Haiku 3.5   █████████ 45%   │ 12:34:41 Haiku   $0.001 ✓   │
│             $5.60            │ [More calls below...]        │
│                              │                              │
│ Cache Savings: $3.21 (26%)   │ Total: 142 calls             │
└──────────────────────────────┴──────────────────────────────┘
```

**Success Criteria**:
- TUI renders all four panels correctly
- Updates in real-time (< 100ms latency)
- Keyboard navigation works
- No screen flicker or tearing

### Phase 1d: Cross-Platform Daemon Management (Week 4)

**Components**:
- macOS launchd integration
- Windows Service implementation
- Linux systemd support (bonus)
- Installation scripts
- Auto-start configuration

**Deliverables**:
- Platform-specific service files
- Installation/uninstallation scripts
- Service management commands
- Documentation for each platform

**Platform Support Matrix**:

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Binary | ✓ | ✓ | ✓ |
| Auto-start | launchd | Service | systemd |
| Install script | shell | PowerShell | shell |
| Status command | ✓ | ✓ | ✓ |
| Logs | Console.app | Event Viewer | journalctl |

**Success Criteria**:
- Single binary per platform
- Daemon auto-starts on boot
- Clean install/uninstall process
- Platform-native logging

## 6. Testing Strategy

### 6.1 Unit Testing

- **Coverage Target**: 80% minimum
- **Framework**: Built-in Rust testing
- **Mocking**: mockall for external dependencies

**Test Categories**:
- Event parsing and validation
- Metrics calculations
- Database operations
- TUI widget rendering
- Daemon lifecycle

### 6.2 Integration Testing

- **SSE Event Stream**: Mock CCO proxy server
- **Database**: In-memory SQLite for tests
- **TUI**: Headless rendering tests
- **End-to-end**: Full daemon lifecycle

### 6.3 Performance Testing

- **Event Processing**: 1000+ events/second
- **Memory Usage**: < 50MB baseline
- **CPU Usage**: < 5% idle, < 20% active
- **Startup Time**: < 500ms
- **Database Queries**: < 10ms p99

### 6.4 Platform Testing

- **macOS**: 12.0+ (Monterey and later)
- **Windows**: Windows 10 1809+, Windows 11
- **Linux**: Ubuntu 20.04+, RHEL 8+

## 7. Success Criteria

### 7.1 Functional Requirements

- ✅ Daemon runs as background service
- ✅ Connects to CCO proxy automatically
- ✅ Processes all SSE events without loss
- ✅ Persists metrics to SQLite
- ✅ TUI displays real-time information
- ✅ Survives CCO proxy restarts
- ✅ Recovers from network interruptions

### 7.2 Performance Requirements

- ✅ < 100ms UI update latency
- ✅ < 50MB memory footprint
- ✅ < 5% CPU usage at idle
- ✅ Handles 1000+ events/second
- ✅ Database < 100MB for 30 days data

### 7.3 Quality Requirements

- ✅ Zero crashes in 24-hour test
- ✅ No memory leaks
- ✅ Clean shutdown without data loss
- ✅ 80% test coverage
- ✅ Cross-platform compatibility

## 8. Timeline Estimates

### Overall Timeline: 4 Weeks

| Phase | Duration | Start | End | Status |
|-------|----------|-------|-----|--------|
| Phase 1a: Core Daemon | 1 week | Week 1 | Week 1 | Pending |
| Phase 1b: SQLite | 1 week | Week 2 | Week 2 | Pending |
| Phase 1c: TUI Dashboard | 1 week | Week 3 | Week 3 | Pending |
| Phase 1d: Platform Support | 1 week | Week 4 | Week 4 | Pending |
| Testing & Polish | Continuous | Week 1 | Week 4 | Pending |

### Critical Path

1. **Week 1**: Core daemon must work before persistence
2. **Week 2**: Database required before TUI can show history
3. **Week 3**: TUI development in parallel with testing
4. **Week 4**: Platform packaging after core is stable

## 9. Dependencies and Blockers

### 9.1 Dependencies

- **CCO Proxy**: Must be running and emitting SSE events
- **Rust Toolchain**: 1.75+ for latest async features
- **SQLite**: System library or bundled build
- **Terminal**: Supporting ANSI escape codes

### 9.2 Potential Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| SSE connection issues | High | Implement exponential backoff retry |
| SQLite locking | Medium | Use WAL mode, connection pooling |
| TUI rendering bugs | Medium | Extensive testing on different terminals |
| Platform service APIs | High | Early platform-specific prototypes |
| Binary size (static linking) | Low | Use dynamic linking where acceptable |

### 9.3 External Dependencies

- No external API dependencies (local only)
- No internet connection required
- No authentication needed (localhost only)
- No cloud services

## 10. Next Steps

### Immediate Actions (Phase 1a Start)

1. **Project Setup**
   - Initialize Rust project with Cargo
   - Set up dependency management
   - Configure CI/CD pipeline
   - Create development environment

2. **Core Development**
   - Implement basic daemon structure
   - Create SSE client for CCO proxy
   - Build event parser
   - Set up metrics aggregator

3. **Testing Infrastructure**
   - Set up test framework
   - Create mock SSE server
   - Write initial unit tests
   - Set up coverage reporting

4. **Documentation**
   - API documentation with rustdoc
   - Development setup guide
   - Architecture decision records
   - Progress tracking

### Team Coordination

**Agent Assignments for Phase 1a**:

1. **Rust Specialist**: Implement core daemon and service lifecycle
2. **Backend Architect**: Design event processing pipeline
3. **DevOps Engineer**: Set up build and CI/CD
4. **Test Engineer**: Create test infrastructure
5. **Technical Writer**: Document architecture and APIs
6. **Security Auditor**: Review daemon security model

### Deliverable Checklist

- [ ] Daemon executable with basic lifecycle
- [ ] SSE client connecting to CCO proxy
- [ ] Event parsing and validation
- [ ] In-memory metrics aggregation
- [ ] Unit test suite (> 80% coverage)
- [ ] API documentation
- [ ] Development setup guide
- [ ] Phase 1a completion report

## 11. Risk Management

### Technical Risks

1. **SSE Stability**: Implement circuit breaker pattern
2. **Memory Growth**: Bounded channels and cleanup
3. **Database Corruption**: WAL mode + backups
4. **TUI Compatibility**: Test on multiple terminals

### Schedule Risks

1. **Platform APIs**: Start platform work early
2. **Testing Time**: Automate from day one
3. **Integration Issues**: Daily integration tests

## 12. Definition of Done

### Phase 1 Complete When:

- ✅ All four phases delivered
- ✅ Cross-platform binaries built
- ✅ TUI shows all required information
- ✅ 30-day data retention working
- ✅ Installation scripts tested
- ✅ Documentation complete
- ✅ All tests passing (> 80% coverage)
- ✅ 24-hour stability test passed
- ✅ Code reviewed and approved
- ✅ Deployment package ready

---

## Appendices

### A. Cost Calculation Formulas

```rust
// Model pricing as of Nov 2024
const OPUS_INPUT: f64 = 15.0 / 1_000_000.0;  // per token
const OPUS_OUTPUT: f64 = 75.0 / 1_000_000.0;
const SONNET_INPUT: f64 = 3.0 / 1_000_000.0;
const SONNET_OUTPUT: f64 = 15.0 / 1_000_000.0;
const HAIKU_INPUT: f64 = 0.25 / 1_000_000.0;
const HAIKU_OUTPUT: f64 = 1.25 / 1_000_000.0;

// Cache pricing (90% discount)
const CACHE_READ_DISCOUNT: f64 = 0.1;  // Pay 10% of input price
const CACHE_WRITE_COST: f64 = 0.25;    // 25% premium for writing
```

### B. Terminal Requirements

- **Minimum Size**: 80x24 characters
- **Color Support**: 256 colors preferred, 16 minimum
- **Unicode**: UTF-8 support for charts
- **Input**: Standard keyboard input

### C. Configuration File

```toml
# ~/.cco-monitor/config.toml
[daemon]
port = 8081
update_interval_ms = 100

[database]
path = "~/.cco-monitor/metrics.db"
retention_days = 30

[cco]
proxy_url = "http://localhost:8080"
sse_endpoint = "/api/sse/analytics"
reconnect_delay_ms = 5000

[ui]
theme = "dark"
refresh_rate_ms = 100
show_cache_savings = true
```

---

**Document Version**: 1.0.0
**Last Updated**: November 17, 2025
**Author**: Chief Architect
**Status**: Ready for Implementation