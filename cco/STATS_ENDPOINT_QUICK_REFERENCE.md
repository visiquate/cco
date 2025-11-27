# `/api/stats` Endpoint - Quick Reference

## TL;DR - What TUI Expects

The TUI parses the `/api/stats` response as **raw JSON** (serde_json::Value), not a typed struct.

### Critical Parsing Paths

```
stats["project"]["cost"]                          → Total project cost (f64)
stats["project"]["calls"]                         → Total API calls (u64)
stats["project"]["tokens"]                        → Total tokens (u64)
stats["chart_data"]["model_distribution"]         → Array of model usage
  └─ [].model                                     → Model name (e.g., "claude-sonnet-4-5")
  └─ [].percentage                                → Usage percentage (f64)
stats["activity"]                                 → Array of recent events (up to 20)
  └─ [].model                                     → Model used (e.g., "claude-opus-4")
  └─ [].cost                                      → Cost for this event (f64)
  └─ [].file_source                               → Source file/agent
```

---

## How TUI Uses Each Field

### 1. Cost Breakdown by Model Tier

```rust
// From parse_cost_by_tier() - lines 522-617

// Gets total from project
let total_cost = stats["project"]["cost"].as_f64() → f64

// Parses model_distribution to calculate per-tier costs
for model_item in stats["chart_data"]["model_distribution"] {
    let model_name = model_item["model"].as_str()           // e.g., "claude-sonnet-4-5"
    let percentage = model_item["percentage"].as_f64()      // e.g., 45.5

    // Maps model to tier and calculates cost
    if model_name.contains("sonnet") {
        sonnet_cost += (total_cost * percentage) / 100.0    // ← TUI does the calculation!
    } else if model_name.contains("opus") {
        opus_cost += (total_cost * percentage) / 100.0
    } else if model_name.contains("haiku") {
        haiku_cost += (total_cost * percentage) / 100.0
    }
}
```

**What this means:**
- TUI calculates per-tier costs from percentages
- Must provide accurate model_distribution array
- Percentages should sum to ~100%
- Model names are case-insensitive in TUI matching

### 2. Recent API Calls

```rust
// From parse_recent_calls() - lines 654-689

for event in stats["activity"].as_array() {
    let model = event["model"].as_str()           // e.g., "claude-opus-4"
    let cost = event["cost"].as_f64()             // API call cost
    let file = event["file_source"].as_str()      // File or agent name

    // Maps model to tier name
    let tier = if model.contains("opus") {
        "Opus"
    } else if model.contains("sonnet") {
        "Sonnet"
    } else if model.contains("haiku") {
        "Haiku"
    } else {
        "Unknown"
    }

    recent_calls.push(RecentCall { tier, cost, file })
}
```

**What this means:**
- TUI takes first 20 activity events
- Extracts: model, cost, file_source
- **CRITICAL**: Must have "file_source" field, not just "file"
- Model matching is case-insensitive (lowercase comparison)
- Missing fields default to "unknown" or skip the entry

---

## Actual Response Structure (From Source Code)

### Serialized as StatsResponse
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
    pub cost: f64,              // CRITICAL: Used by TUI
    pub tokens: u64,            // CRITICAL: Used by TUI
    pub calls: u64,             // CRITICAL: Used by TUI
    pub last_updated: String,
    pub token_breakdown: Option<HashMap<String, TokenBreakdownInfo>>,
}
```

### ChartData
```rust
pub struct ChartData {
    pub cost_over_time: Vec<ChartDataPoint>,
    pub cost_by_project: Vec<ProjectChartData>,
    pub model_distribution: Vec<ModelDistribution>,  // CRITICAL
}

pub struct ModelDistribution {
    pub model: String,      // CRITICAL: Model name like "claude-sonnet-4-5"
    pub percentage: f64,    // CRITICAL: Percentage of usage
}
```

### ActivityEvent
```rust
pub struct ActivityEvent {
    pub timestamp: String,
    pub event_type: String,
    pub agent_name: Option<String>,
    pub model: Option<String>,         // CRITICAL: Used by TUI
    pub tokens: Option<u64>,
    pub latency_ms: Option<u64>,
    pub status: Option<String>,
    pub cost: Option<f64>,             // CRITICAL: Used by TUI
    // file_source is NOT in ActivityEvent struct, but TUI expects it!
    // May need to add it or merge agent_name into display
}
```

---

## Common Implementation Mistakes

### ❌ Wrong: Missing file_source in activity
```json
{
  "activity": [
    {
      "model": "claude-opus-4",
      "cost": 0.05
      // Missing file_source!
    }
  ]
}
```

**Result**: TUI displays "unknown" for all recent calls

### ❌ Wrong: Wrong model names
```json
{
  "model_distribution": [
    { "model": "opus-4", "percentage": 50 },     // Should be "claude-opus-4"
    { "model": "sonnet", "percentage": 30 }      // Should be "claude-sonnet-4-5"
  ]
}
```

**Result**: TUI can't match models, shows "Unknown" tier

### ❌ Wrong: Percentages don't calculate correctly
```json
{
  "project": { "cost": 100.0 },
  "model_distribution": [
    { "model": "claude-sonnet-4-5", "percentage": 40 },
    { "model": "claude-opus-4", "percentage": 40 }
    // Missing 20%! TUI calculates Haiku cost as 0
  ]
}
```

### ✅ Correct Response Structure
```json
{
  "project": {
    "name": "Claude Orchestra",
    "cost": 123.456,
    "tokens": 500000,
    "calls": 1000,
    "last_updated": "2025-11-26T13:00:00Z",
    "token_breakdown": {
      "Sonnet": {
        "input_tokens": 200000,
        "output_tokens": 150000,
        "cache_read_tokens": 50000,
        "cache_write_tokens": 10000
      },
      "Opus": {
        "input_tokens": 100000,
        "output_tokens": 80000,
        "cache_read_tokens": 20000,
        "cache_write_tokens": 5000
      },
      "Haiku": {
        "input_tokens": 50000,
        "output_tokens": 30000,
        "cache_read_tokens": 10000,
        "cache_write_tokens": 2000
      }
    }
  },
  "machine": {
    "cpu": "Apple Silicon (M1 Max)",
    "memory": "16 GB",
    "uptime": 3600,
    "process_count": 50
  },
  "activity": [
    {
      "timestamp": "2025-11-26T12:59:00Z",
      "event_type": "api_call",
      "model": "claude-sonnet-4-5",
      "cost": 0.05,
      "file_source": "src/main.rs",
      "status": "success"
    },
    {
      "timestamp": "2025-11-26T12:58:00Z",
      "event_type": "api_call",
      "model": "claude-opus-4",
      "cost": 0.15,
      "file_source": "src/database.py",
      "status": "success"
    }
  ],
  "chart_data": {
    "cost_over_time": [
      { "date": "2025-11-26", "cost": 45.67 }
    ],
    "cost_by_project": [
      { "project": "Claude Orchestra", "cost": 123.456 }
    ],
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 35.0 },
      { "model": "claude-haiku-4-5", "percentage": 15.0 }
    ]
  }
}
```

---

## Display Output from Example

Given the correct response above, TUI displays:

### Cost Summary by Tier
```
Tier      Cost       %     Calls  Tokens (I/O/CW/CR)
Sonnet    $ 61.73   50.0%    500   I:200K O:150K CW:10K
Opus      $ 41.21   35.0%    350   I:100K O:80K CW:5K
Haiku     $ 18.51   15.0%    150   I:50K O:30K CW:2K
──────────────────────────────────────────────────────
TOTAL     $123.45  100.0%   1000   I:350K O:260K CW:17K CR:80K
```

### Recent API Calls
```
Tier      Cost        File
Sonnet    $0.0500     src/main.rs
Opus      $0.1500     src/database.py
```

---

## Key Implementation Checklist

### Response Structure
- [x] `project.cost` as f64
- [x] `project.tokens` as u64
- [x] `project.calls` as u64
- [x] `chart_data.model_distribution[]` with `model` and `percentage`
- [x] `activity[]` with `model`, `cost`, `file_source`

### Model Names (Critical for TUI parsing)
- [x] Use full model IDs: `claude-sonnet-4-5`, `claude-opus-4`, `claude-haiku-4-5`
- [x] Can include date suffix: `claude-sonnet-4-5-20250929` (TUI strips it)
- [x] Must be lowercase (TUI does case-insensitive matching)

### Token Breakdown (for display)
- [x] Use tier names: `"Sonnet"`, `"Opus"`, `"Haiku"` (title case)
- [x] Include: `input_tokens`, `output_tokens`, `cache_read_tokens`, `cache_write_tokens`
- [x] All as u64

### Recent Activity (for recent calls list)
- [x] Array of up to 20 events
- [x] Newest events first
- [x] Required fields: `model`, `cost`, `file_source`
- [x] Optional: `timestamp`, `status`, `event_type`, etc.

---

## Response Time Requirements

- **Cached**: < 10ms (from metrics_cache)
- **Uncached**: < 100ms target
- **Timeout**: TUI default 5 second timeout per request

---

## Testing the Endpoint

### Curl Test
```bash
curl -s http://localhost:3000/api/stats | jq '.'
```

### Check Critical Fields
```bash
curl -s http://localhost:3000/api/stats | jq '{
  project_cost: .project.cost,
  project_tokens: .project.tokens,
  project_calls: .project.calls,
  model_count: (.chart_data.model_distribution | length),
  activity_count: (.activity | length),
  recent_model: .activity[0].model,
  recent_cost: .activity[0].cost,
  recent_file: .activity[0].file_source
}'
```

### Expected Output
```json
{
  "project_cost": 123.456,
  "project_tokens": 500000,
  "project_calls": 1000,
  "model_count": 3,
  "activity_count": 2,
  "recent_model": "claude-sonnet-4-5",
  "recent_cost": 0.05,
  "recent_file": "src/main.rs"
}
```

