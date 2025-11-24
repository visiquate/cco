# API Cost Monitor Daemon - Architecture Document

## Executive Summary

This document defines the architecture for pivoting the CCO project from a WASM terminal web interface to a focused API cost monitoring daemon. The new system will be a lightweight, cross-platform background service that tracks and reports Anthropic API costs in real-time through a Terminal User Interface (TUI).

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                   API Cost Monitor Daemon                    │
│                                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐    │
│  │   Config   │  │    Data    │  │    TUI Dashboard   │    │
│  │  Manager   │  │   Engine   │  │   (ratatui/tui)    │    │
│  └─────┬──────┘  └──────┬─────┘  └──────────┬─────────┘    │
│        │                │                     │              │
│  ┌─────▼──────────────▼──────────────────────▼─────────┐   │
│  │              Core Cost Tracking Engine               │   │
│  │  ┌────────────┐  ┌───────────┐  ┌────────────────┐  │   │
│  │  │ API Parser │  │ Cost Calc │  │ Metrics Store  │  │   │
│  │  └────────────┘  └───────────┘  └────────────────┘  │   │
│  └───────────────────────┬──────────────────────────────┘   │
│                          │                                   │
│  ┌───────────────────────▼──────────────────────────────┐   │
│  │           CCO Proxy Log Integration                   │   │
│  │  ┌──────────────┐  ┌────────────────────────────┐    │   │
│  │  │ Log Watcher  │  │ API Response Parser       │    │   │
│  │  └──────────────┘  └────────────────────────────┘    │   │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### System Architecture

1. **Background Daemon Process**
   - Runs continuously in background
   - Minimal CPU/memory footprint (< 50MB RAM)
   - Cross-platform (macOS/Windows)
   - Single binary deployment

2. **Data Collection Layer**
   - Monitors CCO proxy logs in real-time
   - Parses API requests/responses
   - Extracts token counts and model usage
   - Calculates costs based on current pricing

3. **Metrics Engine**
   - Aggregates data by model tier
   - Maintains rolling history (last 25 calls)
   - Tracks cumulative costs and usage
   - Persists metrics for historical analysis

4. **TUI Dashboard**
   - Real-time cost display
   - Model tier breakdown
   - Recent call history
   - Status indicators (Idle/Active)

## Data Model Specification

### Core Data Structures

```rust
/// API call record with full token breakdown
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiCall {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub model_tier: ModelTier,
    pub model_name: String,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tokens_cache_write: u64,
    pub tokens_cache_read: u64,
    pub cost_usd: f64,
    pub source_file: Option<String>,
    pub agent_name: Option<String>,
    pub latency_ms: u64,
    pub status: CallStatus,
}

/// Model tier categorization
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModelTier {
    Opus,      // claude-opus-4-1-*
    Sonnet,    // claude-3-5-sonnet-*, claude-sonnet-4-5-*
    Haiku,     // claude-3-haiku-*, claude-haiku-4-5-*
    Custom(String), // Future models
}

/// Aggregated metrics by model tier
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TierMetrics {
    pub tier: ModelTier,
    pub total_cost: f64,
    pub cost_percentage: f32,
    pub call_count: u64,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tokens_cache_write: u64,
    pub tokens_cache_read: u64,
    pub avg_cost_per_call: f64,
    pub avg_latency_ms: u64,
}

/// Session metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionMetrics {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub total_cost: f64,
    pub total_calls: u64,
    pub avg_cost_per_call: f64,
    pub calls_per_minute: f64,
    pub is_active: bool,
    pub tier_breakdown: Vec<TierMetrics>,
    pub recent_calls: VecDeque<ApiCall>, // Last 25
}

/// Persistent storage schema
#[derive(Serialize, Deserialize)]
pub struct PersistentMetrics {
    pub lifetime_cost: f64,
    pub lifetime_calls: u64,
    pub daily_metrics: HashMap<NaiveDate, DailyMetrics>,
    pub monthly_metrics: HashMap<String, MonthlyMetrics>,
    pub model_statistics: HashMap<String, ModelStats>,
}
```

### Pricing Model

```rust
/// Cost calculation based on Anthropic pricing (as of Nov 2024)
pub struct PricingModel {
    pub opus: TokenPricing {
        input_per_million: 15.00,      // $15/M tokens
        output_per_million: 75.00,     // $75/M tokens
        cache_write_per_million: 3.75,  // $3.75/M tokens
        cache_read_per_million: 0.30,   // $0.30/M tokens
    },
    pub sonnet: TokenPricing {
        input_per_million: 3.00,       // $3/M tokens
        output_per_million: 15.00,     // $15/M tokens
        cache_write_per_million: 3.75,  // $3.75/M tokens
        cache_read_per_million: 0.30,   // $0.30/M tokens
    },
    pub haiku: TokenPricing {
        input_per_million: 0.25,       // $0.25/M tokens
        output_per_million: 1.25,      // $1.25/M tokens
        cache_write_per_million: 0.30,  // $0.30/M tokens
        cache_read_per_million: 0.03,   // $0.03/M tokens
    },
}
```

## TUI Dashboard Design

### Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│ Claude API Cost Monitor v2025.11.1    [Active] ↻ 0.5s      │
├─────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────────────────────────────┐   │
│ │ Live Monitor                                          │   │
│ │ Started: 2:30 PM PST | Elapsed: 2h 15m | Status: ●   │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                              │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ Cost Summary                                          │   │
│ │ Total: $12.47 | Calls: 847 | Avg: $0.015 | Rate: 6/m │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                              │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ By Model Tier                                         │   │
│ │ ┌─────────┬──────┬────┬──────┬────────────────────┐  │   │
│ │ │ Tier    │ Cost │ %  │Calls │ Tokens (I/O/CW/CR) │  │   │
│ │ ├─────────┼──────┼────┼──────┼────────────────────┤  │   │
│ │ │ Sonnet  │$8.32 │67% │ 234  │ 1.2M/450K/100K/50K │  │   │
│ │ │ Opus    │$3.15 │25% │  45  │ 200K/50K/20K/10K   │  │   │
│ │ │ Haiku   │$1.00 │ 8% │ 568  │ 3.5M/800K/200K/100K│  │   │
│ │ └─────────┴──────┴────┴──────┴────────────────────┘  │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                              │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ Recent API Calls (Last 25)                   ↑ 3/25  │   │
│ │ ┌────────────┬────────┬───────┬──────────────────┐  │   │
│ │ │ Time       │ Tier   │ Cost  │ Source           │  │   │
│ │ ├────────────┼────────┼───────┼──────────────────┤  │   │
│ │ │ 4:45:32 PM │ Sonnet │ $0.045│ architect.js     │  │   │
│ │ │ 4:45:28 PM │ Haiku  │ $0.002│ python-pro.js    │  │   │
│ │ │ 4:45:15 PM │ Haiku  │ $0.001│ docs-writer.js   │  │   │
│ │ └────────────┴────────┴───────┴──────────────────┘  │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                              │
│ [q]uit [r]eset [e]xport [h]elp      Auto-refresh: ON        │
└─────────────────────────────────────────────────────────────┘
```

### TUI Components

1. **Header Bar**
   - Version and status indicator
   - Refresh rate display
   - Connection status to CCO proxy

2. **Live Monitor Panel**
   - Session start time (with timezone)
   - Elapsed time counter
   - Active/Idle status indicator

3. **Cost Summary Panel**
   - Total cost for session
   - Total API calls
   - Average cost per call
   - Calls per minute rate

4. **Model Tier Breakdown Table**
   - Cost per tier with percentage
   - Call count per tier
   - Token usage (I/O/Cache-Write/Cache-Read)
   - Sorted by cost (descending)

5. **Recent Calls Table**
   - Scrollable list (last 25 calls)
   - Timestamp with timezone
   - Model tier
   - Cost per call
   - Source file/agent

6. **Footer Bar**
   - Keyboard shortcuts
   - Auto-refresh toggle

### TUI Implementation

```rust
// Using ratatui + crossterm for cross-platform TUI
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Table, Gauge},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct DashboardUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    metrics: Arc<RwLock<SessionMetrics>>,
    refresh_rate: Duration,
    is_active: bool,
}
```

## Metrics Persistence Strategy

### Storage Options Analysis

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| **SQLite** | - Full SQL queries<br>- ACID compliance<br>- Single file<br>- Good for analytics | - Overhead for simple metrics<br>- Schema migrations | **✓ SELECTED** |
| JSON Files | - Simple<br>- Human readable<br>- Easy backup | - No queries<br>- File locking issues<br>- Performance at scale | Not selected |
| Binary Format | - Compact<br>- Fast | - Not human readable<br>- Custom implementation | Not selected |

### SQLite Schema

```sql
-- Core metrics table
CREATE TABLE api_calls (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    model_tier TEXT NOT NULL,
    model_name TEXT NOT NULL,
    tokens_input INTEGER NOT NULL,
    tokens_output INTEGER NOT NULL,
    tokens_cache_write INTEGER DEFAULT 0,
    tokens_cache_read INTEGER DEFAULT 0,
    cost_usd REAL NOT NULL,
    source_file TEXT,
    agent_name TEXT,
    latency_ms INTEGER,
    status TEXT NOT NULL
);

-- Daily aggregates for performance
CREATE TABLE daily_metrics (
    date TEXT PRIMARY KEY,
    total_cost REAL NOT NULL,
    total_calls INTEGER NOT NULL,
    tier_breakdown TEXT NOT NULL -- JSON
);

-- Session tracking
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    total_cost REAL NOT NULL,
    total_calls INTEGER NOT NULL
);

-- Indexes for common queries
CREATE INDEX idx_api_calls_timestamp ON api_calls(timestamp);
CREATE INDEX idx_api_calls_model_tier ON api_calls(model_tier);
CREATE INDEX idx_api_calls_date ON api_calls(date(timestamp, 'unixepoch'));
```

### Data Retention Policy

- **Real-time data**: Last 25 calls in memory
- **Session data**: Current session in memory
- **Daily data**: 90 days in SQLite
- **Monthly aggregates**: Unlimited in SQLite
- **Automatic cleanup**: Daily at midnight local time

## Configuration Management

### Configuration File Structure

```toml
# ~/.cco/config.toml

[api]
# Anthropic API key (optional - can use env var)
anthropic_api_key = "${ANTHROPIC_API_KEY}"

[proxy]
# CCO proxy settings
proxy_port = 8888
log_file = "~/.cco/logs/api_calls.json"
log_rotation = "daily"

[monitoring]
# Cost monitoring settings
refresh_rate_ms = 500
auto_start = true
timezone = "auto"  # or "PST", "UTC", etc.

[persistence]
# Database settings
db_path = "~/.cco/metrics.db"
retention_days = 90
auto_cleanup = true

[display]
# TUI customization
theme = "dark"  # or "light"
show_cache_tokens = true
decimal_places = 3
currency = "USD"

[alerts]
# Cost alerts (future feature)
daily_limit = 100.0
session_limit = 50.0
alert_sound = false
```

### Environment Variables

```bash
# API configuration
export ANTHROPIC_API_KEY="sk-ant-..."
export CCO_CONFIG_PATH="~/.cco/config.toml"

# Proxy settings
export CCO_PROXY_PORT=8888
export CCO_LOG_LEVEL=info

# Display preferences
export CCO_THEME=dark
export TZ=America/Los_Angeles
```

## Cross-Platform Deployment

### Build Strategy

```toml
# Cargo.toml
[package]
name = "cco-monitor"
version = "2025.11.1"
edition = "2021"

[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# TUI
ratatui = "0.24"
crossterm = "0.27"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite"] }

# Utils
chrono = "0.4"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
anyhow = "1"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["wincon", "processenv"] }

[target.'cfg(unix)'.dependencies]
nix = "0.27"

[profile.release]
lto = true
opt-level = "z"
strip = true
```

### Platform-Specific Considerations

#### macOS
- Binary location: `/usr/local/bin/cco-monitor`
- Config location: `~/.cco/`
- LaunchAgent for auto-start
- Terminal.app and iTerm2 compatibility

#### Windows
- Binary location: `%LOCALAPPDATA%\cco\cco-monitor.exe`
- Config location: `%APPDATA%\cco\`
- Windows Service or Task Scheduler
- Windows Terminal and cmd.exe compatibility

### Installation Script

```bash
#!/bin/bash
# Universal installer

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

# Download appropriate binary
BINARY_URL="https://github.com/user/cco-monitor/releases/latest/download/cco-monitor-${OS}-${ARCH}"

# Install binary
case "$OS" in
    Darwin)
        sudo curl -L "$BINARY_URL" -o /usr/local/bin/cco-monitor
        sudo chmod +x /usr/local/bin/cco-monitor
        ;;
    Linux)
        sudo curl -L "$BINARY_URL" -o /usr/local/bin/cco-monitor
        sudo chmod +x /usr/local/bin/cco-monitor
        ;;
    MINGW*|MSYS*|CYGWIN*)
        curl -L "$BINARY_URL.exe" -o "$LOCALAPPDATA/cco/cco-monitor.exe"
        ;;
esac

# Create config directory
mkdir -p ~/.cco

# Initialize database
cco-monitor init
```

## Integration with CCO Proxy

### Log Monitoring Strategy

The daemon will monitor CCO proxy logs using:

1. **File Watcher**: Monitor log file for new entries
2. **JSON Parser**: Parse structured API call logs
3. **Event Stream**: Process logs as event stream

```rust
pub struct LogWatcher {
    log_path: PathBuf,
    last_position: u64,
    watcher: RecommendedWatcher,
}

impl LogWatcher {
    pub async fn watch(&mut self) -> Result<ApiCall> {
        // Watch for file changes
        // Parse new log entries
        // Convert to ApiCall records
        // Return for processing
    }
}
```

### Expected Log Format

```json
{
    "timestamp": "2024-11-17T10:30:00Z",
    "model": "claude-3-5-sonnet-20241022",
    "agent": "python-specialist",
    "source_file": "src/agent.js",
    "request": {
        "messages": [...],
        "max_tokens": 4096
    },
    "response": {
        "usage": {
            "input_tokens": 1500,
            "output_tokens": 450,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 200
        }
    },
    "latency_ms": 2340,
    "status": "success"
}
```

## Migration Plan

### Phase 1: Remove Terminal Components (Day 1)
1. Remove `wasm-terminal/` directory
2. Remove terminal-related files from `static/`
3. Remove terminal modules from `src/`
4. Clean up terminal tests
5. Update build scripts

### Phase 2: Implement Core Monitor (Days 2-3)
1. Create daemon skeleton
2. Implement log watcher
3. Add cost calculation
4. Set up SQLite persistence

### Phase 3: Build TUI Dashboard (Days 4-5)
1. Implement ratatui layout
2. Add real-time updates
3. Create keyboard navigation
4. Add export functionality

### Phase 4: Testing & Polish (Days 6-7)
1. Cross-platform testing
2. Performance optimization
3. Documentation update
4. Release preparation

## Performance Targets

- **Memory Usage**: < 50MB resident
- **CPU Usage**: < 1% idle, < 5% active
- **Startup Time**: < 100ms
- **Log Processing**: < 10ms per entry
- **TUI Refresh**: 60 FPS capability
- **Database Writes**: Batched every 1s

## Security Considerations

1. **API Key Protection**
   - Never log API keys
   - Store encrypted in config
   - Use OS keychain when available

2. **Log File Access**
   - Read-only access to CCO logs
   - No modification of proxy behavior
   - Respect file permissions

3. **Database Security**
   - Local SQLite only
   - No network access
   - Encrypted at rest (optional)

## Future Enhancements

### Version 2.1 (Q1 2025)
- Cost alerts and notifications
- Daily/monthly cost reports
- Export to CSV/JSON
- Historical trend analysis

### Version 2.2 (Q2 2025)
- Multi-project tracking
- Team cost allocation
- Budget management
- Slack/Discord integration

### Version 3.0 (Q3 2025)
- Web dashboard (optional)
- Cloud sync (optional)
- Multi-provider support (OpenAI, etc.)
- Cost optimization recommendations

## Conclusion

This architecture provides a focused, efficient solution for API cost monitoring while removing the complexity of the WASM terminal interface. The daemon will be lightweight, cross-platform, and provide real-time insights into API usage and costs through an intuitive TUI dashboard.

## Next Steps

1. **Immediate Actions**:
   - Chief Architect approval of this design
   - Spawn specialist agents for implementation
   - Begin Phase 1 (terminal removal)

2. **Agent Coordination**:
   - Rust Backend Developer: Core daemon implementation
   - TUI Developer: Dashboard interface
   - DevOps Engineer: Build and deployment
   - Technical Writer: Documentation updates
   - QA Engineer: Testing strategy

3. **Deliverables Timeline**:
   - Day 1: Terminal removal complete
   - Day 3: Core daemon functional
   - Day 5: TUI dashboard complete
   - Day 7: Production ready

---

*Document Version: 1.0*
*Date: November 17, 2024*
*Status: READY FOR REVIEW*