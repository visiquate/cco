# `/api/stats` Parsing Flow - Visual Guide

## Overview: How TUI Processes the Response

```
┌─────────────────────────────────────────────────────────────┐
│  TUI calls /api/stats (line 482)                            │
│  get_with_retry(&stats_url) → serde_json::Value            │
└────────────┬────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│  Parse to AppState::Connected (lines 485-514)              │
└────────────┬────────────────────────────────────────────────┘
             │
        ┌────┴────────────────────────┬─────────────────────┐
        │                             │                     │
        ▼                             ▼                     ▼
   ┌─────────────┐            ┌──────────────┐      ┌────────────┐
   │ parse_cost  │            │ parse_recent │      │ is_active  │
   │ _by_tier    │            │   _calls     │      │ detection  │
   │ (lines      │            │ (lines 654)  │      │ (line 494) │
   │ 522-617)    │            │              │      │            │
   └─────────────┘            └──────────────┘      └────────────┘
        │                             │                     │
        ▼                             ▼                     ▼
   CostByTier            Vec<RecentCall>          bool (has_activity)
   ├─ sonnet_cost        ├─ [0] RecentCall
   ├─ sonnet_pct         │   ├─ tier: "Sonnet"   Load from ~/.claude/
   ├─ sonnet_calls       │   ├─ cost: 0.05       metrics.json
   ├─ sonnet_tokens      │   └─ file: "main.rs"  (OverallSummary)
   ├─ opus_cost          └─ [1] RecentCall
   ├─ opus_pct               ├─ tier: "Opus"     Load project JSONL
   ├─ opus_calls             ├─ cost: 0.15       summaries from
   ├─ opus_tokens            └─ file: "db.py"    ~/.claude/projects/
   ├─ haiku_cost                                 */claude.jsonl
   ├─ haiku_pct
   ├─ haiku_calls
   ├─ haiku_tokens
   ├─ total_cost
   ├─ total_calls
   └─ total_tokens
```

---

## Step 1: parse_cost_by_tier() - Lines 522-617

### Input JSON Path
```json
{
  "project": {
    "cost": 100.0,
    "calls": 500,
    "tokens": 250000
  },
  "chart_data": {
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 30.0 },
      { "model": "claude-haiku-4-5", "percentage": 20.0 }
    ]
  }
}
```

### Parsing Algorithm

```rust
fn parse_cost_by_tier(&self, stats: &serde_json::Value) -> CostByTier {
    // STEP 1: Extract totals from stats["project"]
    let total_cost = stats["project"]["cost"].as_f64()
                     // ↓ Result: 100.0
    let total_calls = stats["project"]["calls"].as_u64()
                     // ↓ Result: 500
    let total_tokens = stats["project"]["tokens"].as_u64()
                     // ↓ Result: 250000

    // STEP 2: Initialize tier accumulators
    let mut sonnet_cost = 0.0;
    let mut opus_cost = 0.0;
    let mut haiku_cost = 0.0;
    let mut sonnet_calls = 0;
    let mut opus_calls = 0;
    let mut haiku_calls = 0;

    // STEP 3: Iterate model_distribution array
    // FOR EACH entry in stats["chart_data"]["model_distribution"]
    for model_item in model_distribution {
        let model_name = model_item["model"].as_str()
                        // ↓ Example: "claude-sonnet-4-5"
        let percentage = model_item["percentage"].as_f64()
                        // ↓ Example: 50.0

        // STEP 4: Calculate cost for this model
        let cost = (total_cost * percentage) / 100.0
                  // ↓ For Sonnet: (100.0 * 50.0) / 100.0 = 50.0
        let calls = ((total_calls as f64 * percentage) / 100.0) as u64
                   // ↓ For Sonnet: (500 * 50) / 100 = 250

        // STEP 5: Distribute to tier bucket
        if model_name.to_lowercase().contains("sonnet") {
            sonnet_cost += cost        // ↓ 0.0 + 50.0 = 50.0
            sonnet_calls += calls      // ↓ 0 + 250 = 250
        } else if model_name.to_lowercase().contains("opus") {
            opus_cost += cost          // ↓ 0.0 + 30.0 = 30.0
            opus_calls += calls        // ↓ 0 + 150 = 150
        } else if model_name.to_lowercase().contains("haiku") {
            haiku_cost += cost         // ↓ 0.0 + 20.0 = 20.0
            haiku_calls += calls       // ↓ 0 + 100 = 100
        }
    }
    // After loop: sonnet: (50.0, 250), opus: (30.0, 150), haiku: (20.0, 100)

    // STEP 6: Calculate percentages
    let total_calculated = sonnet_cost + opus_cost + haiku_cost
                          // ↓ 50.0 + 30.0 + 20.0 = 100.0
    if total_calculated > 0.0 {
        let sonnet_pct = (sonnet_cost / total_calculated) * 100.0
                        // ↓ (50.0 / 100.0) * 100.0 = 50.0%
        let opus_pct = (opus_cost / total_calculated) * 100.0
                      // ↓ (30.0 / 100.0) * 100.0 = 30.0%
        let haiku_pct = (haiku_cost / total_calculated) * 100.0
                       // ↓ (20.0 / 100.0) * 100.0 = 20.0%
    }

    // STEP 7: Extract token stats per model (lines 585-586)
    let (sonnet_tokens, opus_tokens, haiku_tokens) =
        self.extract_token_stats_per_model(stats, total_tokens)
        // ↓ Returns TokenStats for each tier

    // STEP 8: Build total token stats
    let total_token_stats = TokenStats {
        input: (total_tokens as f64 * 0.6) as u64,
               // ↓ 250000 * 0.6 = 150000
        output: (total_tokens as f64 * 0.4) as u64,
                // ↓ 250000 * 0.4 = 100000
        cache_write: 0,
        cache_read: 0,
    }

    // STEP 9: Return CostByTier struct
    CostByTier {
        sonnet_cost: 50.0,
        sonnet_pct: 50.0,
        sonnet_calls: 250,
        sonnet_tokens: TokenStats { ... },
        opus_cost: 30.0,
        opus_pct: 30.0,
        opus_calls: 150,
        opus_tokens: TokenStats { ... },
        haiku_cost: 20.0,
        haiku_pct: 20.0,
        haiku_calls: 100,
        haiku_tokens: TokenStats { ... },
        total_cost: 100.0,
        total_calls: 500,
        total_tokens: TokenStats { input: 150000, output: 100000, ... },
    }
}
```

### Output: CostByTier
```rust
CostByTier {
    // Sonnet (50% of costs)
    sonnet_cost: 50.0,
    sonnet_pct: 50.0,
    sonnet_calls: 250,
    sonnet_tokens: TokenStats { input: 75000, output: 50000, cache_write: 0, cache_read: 0 },

    // Opus (30% of costs)
    opus_cost: 30.0,
    opus_pct: 30.0,
    opus_calls: 150,
    opus_tokens: TokenStats { input: 45000, output: 30000, cache_write: 0, cache_read: 0 },

    // Haiku (20% of costs)
    haiku_cost: 20.0,
    haiku_pct: 20.0,
    haiku_calls: 100,
    haiku_tokens: TokenStats { input: 30000, output: 20000, cache_write: 0, cache_read: 0 },

    // Totals
    total_cost: 100.0,
    total_calls: 500,
    total_tokens: TokenStats { input: 150000, output: 100000, cache_write: 0, cache_read: 0 },
}
```

---

## Step 2: parse_recent_calls() - Lines 654-689

### Input JSON Path
```json
{
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
    },
    {
      "timestamp": "2025-11-26T12:57:00Z",
      "event_type": "cache_hit",
      "model": "claude-haiku-4-5",
      "cost": 0.002,
      "file_source": "cache",
      "status": "success"
    }
  ]
}
```

### Parsing Algorithm

```rust
fn parse_recent_calls(&self, stats: &serde_json::Value) -> Vec<RecentCall> {
    let mut calls = Vec::new();

    // STEP 1: Get activity array from stats
    if let Some(activity) = stats["activity"].as_array() {
        // STEP 2: Take first 20 items (lines 659)
        for event in activity.iter().take(20) {
            // FOR event index 0 (Sonnet call):

            // STEP 3: Extract model and cost
            if let (Some(model), Some(cost)) = (
                event["model"].as_str(),
                event["cost"].as_f64(),
            ) {
                // model = "claude-sonnet-4-5"
                // cost = 0.05

                // STEP 4: Map model name to tier (case-insensitive)
                let tier = if model.contains("opus") {
                    "Opus"
                } else if model.contains("sonnet") {
                    "Sonnet"               // ← MATCH!
                } else if model.contains("haiku") {
                    "Haiku"
                } else {
                    "Unknown"
                };
                // tier = "Sonnet"

                // STEP 5: Extract file_source
                let file = event["file_source"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                // file = "src/main.rs"

                // STEP 6: Create RecentCall
                calls.push(RecentCall {
                    tier: tier.to_string(),      // "Sonnet"
                    cost,                        // 0.05
                    file,                        // "src/main.rs"
                });
            }

            // FOR event index 1 (Opus call):
            // Repeats same process...
            // Result: RecentCall { tier: "Opus", cost: 0.15, file: "src/database.py" }

            // FOR event index 2 (Haiku cache_hit):
            // Repeats same process...
            // Result: RecentCall { tier: "Haiku", cost: 0.002, file: "cache" }
        }
    }

    calls
}
```

### Output: Vec<RecentCall>
```rust
vec![
    RecentCall {
        tier: "Sonnet".to_string(),
        cost: 0.05,
        file: "src/main.rs".to_string(),
    },
    RecentCall {
        tier: "Opus".to_string(),
        cost: 0.15,
        file: "src/database.py".to_string(),
    },
    RecentCall {
        tier: "Haiku".to_string(),
        cost: 0.002,
        file: "cache".to_string(),
    },
]
```

### Rendered in TUI
```
Tier      Cost        File
Sonnet    $0.0500     src/main.rs
Opus      $0.1500     src/database.py
Haiku     $0.0020     cache
```

---

## Step 3: Display Rendering

### Cost Summary Table (from CostByTier)

```rust
fn render_cost_summary(f: &mut Frame, cost: &CostByTier, area: Rect) {
    // Line: Sonnet    ${:>8.2}  {:>4.1}%  {:>6}
    //       Sonnet    $   50.00   50.0%    250

    // Line: Opus      ${:>8.2}  {:>4.1}%  {:>6}
    //       Opus      $   30.00   30.0%    150

    // Line: Haiku     ${:>8.2}  {:>4.1}%  {:>6}
    //       Haiku     $   20.00   20.0%    100

    // Line: TOTAL     ${:>8.2}   100.0%  {:>6}
    //       TOTAL     $  100.00  100.0%    500
}
```

### Recent Calls List (from Vec<RecentCall>)

```rust
fn render_recent_calls_dynamic(f: &mut Frame, calls: &[RecentCall], area: Rect) {
    // For each RecentCall in calls:
    // Format: "{:<8} ${:>7.4}  {}"
    //         "Sonnet  $0.0500  src/main.rs"
    //         "Opus    $0.1500  src/database.py"
    //         "Haiku   $0.0020  cache"
}
```

---

## Critical Parsing Rules

### 1. Model Name Matching
```
Input: "claude-sonnet-4-5-20250929"
       ↓ .contains("sonnet") = true
       ↓ .to_lowercase().contains("sonnet") = true
Output: Tier = "Sonnet" ✓

Input: "Claude-Opus-4"
       ↓ .to_lowercase() = "claude-opus-4"
       ↓ .contains("opus") = true
Output: Tier = "Opus" ✓

Input: "haiku"
       ↓ .to_lowercase().contains("haiku") = true
Output: Tier = "Haiku" ✓

Input: "unknown-model"
       ↓ None of the above match
Output: Tier = "Unknown" ✓
```

### 2. Percentage Calculations
```
Total Cost: 100.0
Sonnet Percentage: 50.0
Sonnet Cost: (100.0 * 50.0) / 100.0 = 50.0 ✓

Total Calls: 500
Sonnet Percentage: 50.0
Sonnet Calls: ((500 * 50.0) / 100.0) as u64 = 250 ✓

Total Tokens: 250000
Sonnet Percentage: 50.0
Sonnet Tokens: ((250000 * 50.0) / 100.0) as u64 = 125000
├─ Input: 125000 * 0.6 = 75000
└─ Output: 125000 * 0.4 = 50000 ✓
```

### 3. Activity Event Limits
```
Activity array in response:  [e0, e1, e2, e3, e4, ..., e500]
Applied filter:               .take(20)
Result in RecentCall list:    [e0, e1, e2, ..., e19] ✓
```

### 4. Missing Field Handling
```
activity[0] = {
  "model": "claude-sonnet-4-5",
  "cost": 0.05,
  // Missing file_source!
}

Parsing:
event["file_source"].as_str().unwrap_or("unknown").to_string()
Result: "unknown" ✓ (No crash, graceful fallback)
```

---

## Data Flow Diagram

```
JSON Response
    ↓
    ├─→ Load health()              → HealthResponse
    │   ├─ status
    │   ├─ version
    │   ├─ uptime_seconds
    │   └─ port
    │
    ├─→ Load stats()                → StatsResponse
    │   ├─ project
    │   │   ├─ cost ────────────────────┐
    │   │   ├─ tokens ─────────────────┐│
    │   │   ├─ calls ──────────────────┼┼─→ parse_cost_by_tier()
    │   │   └─ token_breakdown         ││   → CostByTier
    │   │                              ││
    │   ├─ chart_data                  ││
    │   │   └─ model_distribution ─────┼┴───→ [Calculate tier costs]
    │   │       ├─ model              │
    │   │       └─ percentage         │
    │   │                             │
    │   └─ activity ─────────────────┬┴───→ parse_recent_calls()
    │       ├─ model                 │     → Vec<RecentCall>
    │       ├─ cost                  │
    │       └─ file_source           │
    │
    ├─→ Load overall_metrics()        → OverallSummary
    │   (from ~/.claude/metrics.json)
    │
    └─→ Load project_summaries()      → Vec<ProjectSummary>
        (from ~/.claude/projects/*/claude.jsonl)

All data collected → AppState::Connected
                  → render() displays to terminal
```

---

## Example: Full Request/Response Cycle

### 1. TUI Makes Request
```rust
let stats_url = format!("{}/api/stats", self.client.base_url);
// → "http://localhost:3000/api/stats"

let stats_response: Result<serde_json::Value, _> =
    self.client.get_with_retry(&stats_url).await;
// HTTP GET request sent
// Timeout: 5 seconds
// Retries: Up to 3 with exponential backoff
```

### 2. Server Responds with JSON
```json
HTTP/1.1 200 OK
Content-Type: application/json

{
  "project": {
    "name": "Claude Orchestra",
    "cost": 123.456,
    "tokens": 500000,
    "calls": 1000,
    "last_updated": "2025-11-26T13:00:00Z"
  },
  "machine": {...},
  "activity": [...20 events...],
  "chart_data": {...}
}
```

### 3. TUI Parses Response
```rust
// Parse cost by tier
let cost_by_tier = self.parse_cost_by_tier(&stats);
// Iterates model_distribution, calculates per-tier costs

// Parse recent calls
let recent_calls = self.parse_recent_calls(&stats);
// Takes first 20 activity events, maps to RecentCall

// Determine if active
let is_active = !recent_calls.is_empty();
// true if there are recent calls
```

### 4. Update UI State
```rust
self.state = AppState::Connected {
    cost_by_tier,        // For cost summary table
    recent_calls,        // For recent calls list
    health,              // For header/status
    is_active,           // For status message
    overall_summary,     // Loaded from ~/.claude/metrics.json
    project_summaries,   // Loaded from ~/.claude/projects/*/claude.jsonl
};
```

### 5. Render to Terminal
```
┌─────────────────────────────────────────────────────────────┐
│ Claude Code Orchestra  | v2025.11.2 | Port: 3000 | Uptime:  │
├─────────────────────────────────────────────────────────────┤
│ Overall Summary                                             │
│ Cost: $0.12346  Tokens: 500K  Calls: 1000 | Op:35% Sn:50% │
├─────────────────────────────────────────────────────────────┤
│ Cost Summary by Tier                                        │
│ Sonnet    $ 61.73    50.0%    500    I:300K O:200K CW:10K  │
│ Opus      $ 41.21    35.0%    350    I:200K O:150K CW:5K   │
│ Haiku     $ 20.51    15.0%    150    I:100K O:50K CW:2K    │
│ ────────────────────────────────────────────────────────── │
│ TOTAL     $123.45   100.0%   1000    I:600K O:400K CW:17K  │
├─────────────────────────────────────────────────────────────┤
│ Recent API Calls (3 of 5)                                   │
│ Sonnet   $0.0500     src/main.rs                            │
│ Opus     $0.1500     src/database.py                        │
│ Haiku    $0.0020     cache                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Performance Characteristics

### Response Time Breakdown
```
Request time:        < 1ms (network)
Server processing:   < 10ms (from cache) to 100ms (without cache)
JSON serialization:  < 5ms
TUI parsing:         < 1ms
Total roundtrip:     10-110ms typical
```

### Cached Metrics
- `/api/stats` uses in-memory cache (MetricsCache)
- Background aggregation task updates every 5-10 seconds
- First request may take longer (builds cache)
- Subsequent requests: <10ms

### No Automatic Refresh Currently
- TUI loads stats once at startup
- Updates on manual daemon restart (r key)
- Ready for 5-10 second polling (code supports it, just not enabled)

