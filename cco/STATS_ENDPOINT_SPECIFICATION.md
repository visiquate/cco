# `/api/stats` Endpoint Specification

## Overview

The TUI (Terminal User Interface) in `tui_app.rs` fetches stats from the `/api/stats` endpoint to display cost analysis, recent API calls, and activity information. This document specifies **exactly** what data structure the TUI expects.

---

## 1. Endpoint Details

### URL
```
GET /api/stats
```

### Server Location
`src/server.rs`, line 612: `async fn stats(State(state): State<Arc<ServerState>>) -> Result<Json<StatsResponse>, ServerError>`

### Response Type
```rust
Json<StatsResponse>
```

---

## 2. Complete JSON Schema

### Root Response Structure
```json
{
  "project": {
    "name": "string",
    "cost": "f64 (total cost in dollars)",
    "tokens": "u64 (total tokens)",
    "calls": "u64 (total API calls)",
    "last_updated": "string (ISO 8601 timestamp)",
    "token_breakdown": {
      "optional": {
        "Haiku": {
          "input_tokens": "u64",
          "output_tokens": "u64",
          "cache_read_tokens": "u64",
          "cache_write_tokens": "u64"
        },
        "Sonnet": {
          "input_tokens": "u64",
          "output_tokens": "u64",
          "cache_read_tokens": "u64",
          "cache_write_tokens": "u64"
        },
        "Opus": {
          "input_tokens": "u64",
          "output_tokens": "u64",
          "cache_read_tokens": "u64",
          "cache_write_tokens": "u64"
        }
      }
    }
  },
  "machine": {
    "cpu": "string (CPU description)",
    "memory": "string (memory info)",
    "uptime": "u64 (seconds)",
    "process_count": "u64"
  },
  "activity": [
    {
      "timestamp": "string (ISO 8601)",
      "event_type": "string (api_call, error, cache_hit, cache_miss, model_override)",
      "agent_name": "optional string",
      "model": "optional string (e.g., claude-sonnet-4-5, claude-opus-4, claude-haiku-4-5)",
      "tokens": "optional u64",
      "latency_ms": "optional u64",
      "status": "optional string (success, error, pending)",
      "cost": "optional f64 (calculated cost for this event)",
      "file_source": "optional string (added by TUI parsing)"
    }
  ],
  "chart_data": {
    "optional": {
      "cost_over_time": [
        {
          "date": "string (YYYY-MM-DD)",
          "cost": "f64"
        }
      ],
      "cost_by_project": [
        {
          "project": "string",
          "cost": "f64"
        }
      ],
      "model_distribution": [
        {
          "model": "string (e.g., claude-sonnet-4-5, claude-opus-4, claude-haiku-4-5)",
          "percentage": "f64 (0-100)"
        }
      ]
    }
  }
}
```

---

## 3. Rust Type Definitions

### Main Response Type
```rust
pub struct StatsResponse {
    pub project: ProjectInfo,
    pub machine: MachineInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<Vec<ActivityEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,
}
```

### ProjectInfo
```rust
pub struct ProjectInfo {
    pub name: String,
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
    pub last_updated: String,  // ISO 8601 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_breakdown: Option<HashMap<String, TokenBreakdownInfo>>,
}
```

### TokenBreakdownInfo
```rust
pub struct TokenBreakdownInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}
```

### MachineInfo
```rust
pub struct MachineInfo {
    pub cpu: String,
    pub memory: String,
    pub uptime: u64,        // uptime in seconds
    pub process_count: u64,
}
```

### ActivityEvent
```rust
pub struct ActivityEvent {
    pub timestamp: String,                // ISO 8601
    pub event_type: String,               // "api_call", "error", "cache_hit", "cache_miss", "model_override"
    pub agent_name: Option<String>,
    pub model: Option<String>,
    pub tokens: Option<u64>,
    pub latency_ms: Option<u64>,
    pub status: Option<String>,           // "success", "error", "pending"
    pub cost: Option<f64>,
    // Note: "file_source" is added during TUI parsing, not by the endpoint
}
```

### ChartData
```rust
pub struct ChartData {
    pub cost_over_time: Vec<ChartDataPoint>,
    pub cost_by_project: Vec<ProjectChartData>,
    pub model_distribution: Vec<ModelDistribution>,
}
```

### ChartDataPoint
```rust
pub struct ChartDataPoint {
    pub date: String,  // YYYY-MM-DD format
    pub cost: f64,
}
```

### ProjectChartData
```rust
pub struct ProjectChartData {
    pub project: String,
    pub cost: f64,
}
```

### ModelDistribution
```rust
pub struct ModelDistribution {
    pub model: String,    // e.g., "claude-sonnet-4-5", "claude-opus-4", "claude-haiku-4-5"
    pub percentage: f64,  // 0-100, rounded
}
```

---

## 4. How TUI Uses the Data

### Location of Usage
File: `/Users/brent/git/cc-orchestra/cco/src/tui_app.rs`

### Parsing Steps (lines 478-520)

1. **Fetch from daemon** (line 482):
   ```rust
   let stats_url = format!("{}/api/stats", self.client.base_url);
   let stats_response: Result<serde_json::Value, _> = self.client.get_with_retry(&stats_url).await;
   ```

2. **Parse cost by tier** (line 488):
   ```rust
   let cost_by_tier = self.parse_cost_by_tier(&stats);
   ```
   This function (lines 522-617):
   - Extracts `stats["project"]["cost"]`
   - Extracts `stats["project"]["calls"]`
   - Extracts `stats["project"]["tokens"]`
   - Parses `stats["chart_data"]["model_distribution"]` array
   - Calculates cost per model tier based on percentages
   - Extracts token stats per model

3. **Parse recent calls** (line 491):
   ```rust
   let recent_calls = self.parse_recent_calls(&stats);
   ```
   This function (lines 654-689):
   - Iterates through `stats["activity"]` array (up to 20 items)
   - Extracts fields: `model`, `cost`, `file_source`
   - Maps model name to tier: "Opus", "Sonnet", "Haiku"
   - Returns Vec<RecentCall> with: `{ tier, cost, file }`

4. **Determine activity status** (line 494):
   ```rust
   let is_active = !recent_calls.is_empty();
   ```

### CostByTier Structure Expected
```rust
pub struct CostByTier {
    // Sonnet tier breakdown
    pub sonnet_cost: f64,
    pub sonnet_pct: f64,           // percentage of total
    pub sonnet_calls: u64,
    pub sonnet_tokens: TokenStats,

    // Opus tier breakdown
    pub opus_cost: f64,
    pub opus_pct: f64,
    pub opus_calls: u64,
    pub opus_tokens: TokenStats,

    // Haiku tier breakdown
    pub haiku_cost: f64,
    pub haiku_pct: f64,
    pub haiku_calls: u64,
    pub haiku_tokens: TokenStats,

    // Totals
    pub total_cost: f64,
    pub total_calls: u64,
    pub total_tokens: TokenStats,
}

pub struct TokenStats {
    pub input: u64,
    pub output: u64,
    pub cache_write: u64,
    pub cache_read: u64,
}
```

### RecentCall Structure Expected
```rust
pub struct RecentCall {
    pub tier: String,    // "Opus", "Sonnet", "Haiku", or "Unknown"
    pub cost: f64,
    pub file: String,    // from activity["file_source"]
}
```

---

## 5. Data Source & Aggregation

The endpoint aggregates data from multiple sources:

1. **Claude History** (`~/.claude/metrics.json` or project JSONL files)
   - Total cost, tokens, calls
   - Model breakdown percentages
   - Per-model token statistics

2. **Analytics Engine** (`state.analytics`)
   - Recent activity events (last 20)
   - Real-time events with timestamps

3. **Persistence Layer** (Database if available)
   - Historical cost data (30 days)
   - Daily totals for chart

4. **Machine Info**
   - CPU, memory, uptime, process count

---

## 6. TUI Display Components

### Overall Summary Panel
Displays (loads from `~/.claude/metrics.json`):
```
Cost: $X.XXXXX  Tokens: XKM  Calls: X | Opus: Y% | Sonnet: Z% | Haiku: W%
```

### Project Summaries Panel
Shows top 5 projects (loads from `~/.claude/projects/*/claude.jsonl`):
```
Project Name               Cost         Tokens    Calls
[project]                  $X.XXXXX     XKM       X
```

### Cost Summary by Tier Table (from `/api/stats`)
```
Tier      Cost       %     Calls  Tokens (I/O/CW/CR)
Sonnet    $X.XX    XX.X%   X      I:X O:X CW:X
Opus      $X.XX    XX.X%   X      I:X O:X CW:X
Haiku     $X.XX    XX.X%   X      I:X O:X CW:X
─────────────────────────────────────────────────
TOTAL     $X.XX   100.0%   X      I:X O:X CW:X CR:X
```

### Recent API Calls List
```
Tier      Cost        File
Sonnet    $X.XXXX     [file_source]
Opus      $X.XXXX     [file_source]
Haiku     $X.XXXX     [file_source]
```

---

## 7. Refresh Frequency

### Main Event Loop (line 365)
```rust
if crossterm::event::poll(Duration::from_millis(200))? {
```
- Polls for keyboard input every **200ms**

### Update State Call (line 374)
```rust
self.update_state().await?;
```
- Called after every render
- Currently doesn't trigger data refresh (commented out at line 1233)
- Hooks panel has its own throttling

### Current Behavior
- Initial load at startup
- Manual refresh on daemon restart (r key)
- No automatic periodic refresh of stats
- System is ready for timer-based refresh (~5-10 second intervals recommended)

---

## 8. Example JSON Response

### Minimal Valid Response
```json
{
  "project": {
    "name": "Claude Orchestra",
    "cost": 1.23456,
    "tokens": 50000,
    "calls": 100,
    "last_updated": "2025-11-26T12:34:56Z"
  },
  "machine": {
    "cpu": "Apple Silicon (M1 Max)",
    "memory": "16 GB",
    "uptime": 3600,
    "process_count": 42
  },
  "activity": [
    {
      "timestamp": "2025-11-26T12:30:00Z",
      "event_type": "api_call",
      "agent_name": "TDD Coding Agent",
      "model": "claude-sonnet-4-5",
      "tokens": 5000,
      "latency_ms": 1234,
      "status": "success",
      "cost": 0.05,
      "file_source": "src/main.rs"
    }
  ],
  "chart_data": {
    "cost_over_time": [
      {
        "date": "2025-11-26",
        "cost": 0.12345
      }
    ],
    "cost_by_project": [
      {
        "project": "Claude Orchestra",
        "cost": 1.23456
      }
    ],
    "model_distribution": [
      {
        "model": "claude-sonnet-4-5",
        "percentage": 50.0
      },
      {
        "model": "claude-opus-4",
        "percentage": 30.0
      },
      {
        "model": "claude-haiku-4-5",
        "percentage": 20.0
      }
    ]
  }
}
```

### Full Example with Token Breakdown
```json
{
  "project": {
    "name": "Claude Orchestra",
    "cost": 5.67890,
    "tokens": 250000,
    "calls": 500,
    "last_updated": "2025-11-26T13:00:00Z",
    "token_breakdown": {
      "Sonnet": {
        "input_tokens": 75000,
        "output_tokens": 50000,
        "cache_read_tokens": 10000,
        "cache_write_tokens": 5000
      },
      "Opus": {
        "input_tokens": 45000,
        "output_tokens": 30000,
        "cache_read_tokens": 5000,
        "cache_write_tokens": 2000
      },
      "Haiku": {
        "input_tokens": 30000,
        "output_tokens": 20000,
        "cache_read_tokens": 3000,
        "cache_write_tokens": 1000
      }
    }
  },
  "machine": {
    "cpu": "Apple Silicon (M1 Max)",
    "memory": "16 GB / 32 GB",
    "uptime": 7200,
    "process_count": 48
  },
  "activity": [
    {
      "timestamp": "2025-11-26T12:58:30Z",
      "event_type": "api_call",
      "agent_name": "Python Expert",
      "model": "claude-opus-4",
      "tokens": 8500,
      "latency_ms": 2100,
      "status": "success",
      "cost": 0.085,
      "file_source": "src/api/handlers.py"
    },
    {
      "timestamp": "2025-11-26T12:57:45Z",
      "event_type": "api_call",
      "agent_name": "TDD Coding Agent",
      "model": "claude-sonnet-4-5",
      "tokens": 5200,
      "latency_ms": 1234,
      "status": "success",
      "cost": 0.052,
      "file_source": "src/main.rs"
    },
    {
      "timestamp": "2025-11-26T12:56:00Z",
      "event_type": "cache_hit",
      "model": "claude-haiku-4-5",
      "tokens": 2000,
      "status": "success",
      "cost": 0.0002
    }
  ],
  "chart_data": {
    "cost_over_time": [
      {"date": "2025-11-24", "cost": 0.45},
      {"date": "2025-11-25", "cost": 0.78},
      {"date": "2025-11-26", "cost": 1.23}
    ],
    "cost_by_project": [
      {"project": "Claude Orchestra", "cost": 5.67890}
    ],
    "model_distribution": [
      {"model": "claude-sonnet-4-5", "percentage": 45.0},
      {"model": "claude-opus-4", "percentage": 35.0},
      {"model": "claude-haiku-4-5", "percentage": 20.0}
    ]
  }
}
```

---

## 9. Critical Implementation Notes

### For Server Implementers

1. **Always include `project` and `machine`** - These are required
2. **Optional fields**: `activity` and `chart_data` can be null
3. **Model names must normalize**: Use exact model IDs with date suffixes (e.g., `claude-sonnet-4-5-20250929`)
4. **Token breakdown keys**: Must be "Haiku", "Sonnet", "Opus" (title case) for TUI parsing
5. **Cost calculations**: All decimals preserved (no rounding at endpoint level)
6. **Timestamps**: Use ISO 8601 format (UTC)
7. **Percentages**: Round to nearest 0.1% in model_distribution
8. **Activity array**: Limit to 20 most recent events; newer events first
9. **Cache tokens**: Include cache_read and cache_write separately from input/output
10. **File source**: ActivityEvent should have this field for TUI display

### For TUI Improvements

1. Refresh stats every 5-10 seconds (currently disabled)
2. Parse model names case-insensitively (already done)
3. Handle missing fields gracefully (already done with Option types)
4. Display "Unknown" for unrecognized model names
5. Support both old format (project JSONL) and new format (~/.claude/metrics.json)

---

## 10. Error Handling

### 404 Response
If `/api/stats` returns 404, TUI treats as "no data yet":
- Shows default empty CostByTier
- Shows empty recent calls list
- System appears inactive

### 5xx Response
Triggers retry logic (up to 3 retries with exponential backoff)

### Malformed JSON
Returns parse error, moves to Error state

---

## 11. Testing Checklist

- [ ] Endpoint returns valid StatsResponse JSON
- [ ] `project.cost` is accurate f64
- [ ] `project.tokens` sums all tiers correctly
- [ ] `project.calls` represents total API calls
- [ ] `activity` array has newest items first
- [ ] `chart_data.model_distribution` percentages sum to ~100%
- [ ] Token breakdown keys are "Haiku", "Sonnet", "Opus"
- [ ] All timestamps are ISO 8601 UTC
- [ ] File_source field exists in activity events
- [ ] Response time < 100ms from cached data
- [ ] Optional fields serialize correctly when null
- [ ] Cache tokens properly separated from input/output

