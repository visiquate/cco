# `/api/stats` Endpoint - Implementation Guide

## Quick Start

You need to implement or verify the `/api/stats` endpoint returns a JSON response that the TUI can parse.

### Files You Need to Read First
1. **STATS_ENDPOINT_SPECIFICATION.md** - Complete formal specification
2. **STATS_ENDPOINT_QUICK_REFERENCE.md** - TL;DR version
3. **STATS_PARSING_FLOW.md** - Visual data flow
4. **STATS_ENDPOINT_TEST_CASES.md** - Test scenarios

---

## What the TUI Expects (30-Second Version)

### JSON Structure
```json
{
  "project": {
    "cost": 100.0,           // Required: total cost
    "tokens": 500000,        // Required: total tokens
    "calls": 1000,           // Required: total API calls
    "name": "string",        // Optional
    "last_updated": "ISO8601"
  },
  "machine": {
    "cpu": "string",
    "memory": "string",
    "uptime": 3600,
    "process_count": 50
  },
  "activity": [              // Optional: up to 20 recent events
    {
      "model": "claude-sonnet-4-5",    // CRITICAL
      "cost": 0.05,                     // CRITICAL
      "file_source": "src/main.py"      // CRITICAL
    }
  ],
  "chart_data": {            // Optional
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 30.0 },
      { "model": "claude-haiku-4-5", "percentage": 20.0 }
    ]
  }
}
```

### TUI Parsing
1. **Cost per tier** = `project.cost * model_distribution[i].percentage / 100.0`
2. **Recent calls** = First 20 from `activity` array, extract `model`, `cost`, `file_source`
3. **Model tier mapping** = `model.contains("sonnet")` → "Sonnet", etc. (case-insensitive)

---

## Current Implementation Status

### File Locations
- **Endpoint Handler**: `/Users/brent/git/cc-orchestra/cco/src/server.rs`, line 612
- **TUI Parser**: `/Users/brent/git/cc-orchestra/cco/src/tui_app.rs`, lines 478-689
- **Response Struct**: `src/server.rs`, lines 225-232

### Current Behavior
```rust
// Handler signature (server.rs:612)
async fn stats(State(state): State<Arc<ServerState>>) ->
    Result<Json<StatsResponse>, ServerError>

// Returns StatsResponse
pub struct StatsResponse {
    pub project: ProjectInfo,
    pub machine: MachineInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<Vec<ActivityEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,
}
```

### Data Sources
1. **Claude metrics** - `~/.claude/metrics.json` or project JSONL files
2. **Activity** - Real-time events from analytics engine
3. **Chart data** - 30-day history from database (or mock if unavailable)
4. **Machine info** - System stats

---

## Critical Implementation Requirements

### MUST HAVE
1. **project.cost** (f64) - Total project cost in dollars
2. **project.tokens** (u64) - Total tokens used
3. **project.calls** (u64) - Total API calls
4. **machine** object - System information
5. **activity** array with:
   - `model` field (e.g., "claude-sonnet-4-5")
   - `cost` field (f64)
   - `file_source` field (string)
6. **chart_data.model_distribution** with:
   - `model` field (recognized model names)
   - `percentage` field (0-100, sum ≈ 100%)

### SHOULD HAVE
- Token breakdown by tier ("Sonnet", "Opus", "Haiku")
- Cost history (30 days)
- Multiple activity events (up to 20)

### OPTIONAL
- Cost by project
- Agent names in activity
- Event types
- Latency metrics

### MUST NOT DO
- Return null for required fields
- Use unrecognized model names
- Have percentages not sum to ~100%
- Forget "file_source" in activity events

---

## Model Name Reference

The TUI matches model names case-insensitively:

```
Recognized Model Names → Tier
───────────────────────────────────
claude-sonnet-4-5              Sonnet
claude-sonnet-4-5-20250929     Sonnet
claude-3-5-sonnet              Sonnet
SONNET                         Sonnet
sonnet (any case)              Sonnet
───────────────────────────────────
claude-opus-4                  Opus
claude-opus-4-1                Opus
OPUS (any case)                Opus
───────────────────────────────────
claude-haiku-4-5               Haiku
claude-3-5-haiku               Haiku
HAIKU (any case)               Haiku
───────────────────────────────────
anything-else                  Unknown
```

**Rule**: TUI does `.to_lowercase().contains("sonnet|opus|haiku")`

---

## Cost Calculation Example

Given:
```json
{
  "project": { "cost": 100.0, "calls": 1000 },
  "chart_data": {
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 30.0 },
      { "model": "claude-haiku-4-5", "percentage": 20.0 }
    ]
  }
}
```

TUI calculates:
```
Sonnet:
  cost = (100.0 * 50.0) / 100.0 = $50.00
  calls = (1000 * 50) / 100 = 500

Opus:
  cost = (100.0 * 30.0) / 100.0 = $30.00
  calls = (1000 * 30) / 100 = 300

Haiku:
  cost = (100.0 * 20.0) / 100.0 = $20.00
  calls = (1000 * 20) / 100 = 200
```

Then displays:
```
Tier      Cost       %     Calls
Sonnet    $ 50.00   50.0%    500
Opus      $ 30.00   30.0%    300
Haiku     $ 20.00   20.0%    200
──────────────────────────────
TOTAL     $100.00  100.0%   1000
```

---

## Common Implementation Mistakes

### ❌ WRONG 1: Missing file_source
```json
{
  "activity": [
    { "model": "claude-sonnet-4-5", "cost": 0.05 }
    // Missing file_source!
  ]
}
```
**Result**: TUI shows "unknown" for all files

---

### ❌ WRONG 2: Wrong model names in distribution
```json
{
  "model_distribution": [
    { "model": "sonnet", "percentage": 50 },     // Wrong!
    { "model": "Opus 4", "percentage": 30 }      // Wrong!
  ]
}
```
**Result**: TUI can't match, tier percentages all 0

---

### ❌ WRONG 3: Percentages don't add up
```json
{
  "model_distribution": [
    { "model": "claude-sonnet-4-5", "percentage": 40 },
    { "model": "claude-opus-4", "percentage": 30 }
    // Missing 30%! Haiku will be 0
  ]
}
```

---

### ✅ CORRECT
```json
{
  "project": {
    "name": "My Project",
    "cost": 123.45,
    "tokens": 500000,
    "calls": 1000,
    "last_updated": "2025-11-26T13:00:00Z"
  },
  "machine": {
    "cpu": "Apple Silicon",
    "memory": "16 GB",
    "uptime": 3600,
    "process_count": 50
  },
  "activity": [
    {
      "timestamp": "2025-11-26T13:00:00Z",
      "event_type": "api_call",
      "model": "claude-sonnet-4-5",
      "cost": 0.05,
      "file_source": "src/main.py",
      "status": "success"
    }
  ],
  "chart_data": {
    "cost_over_time": [
      { "date": "2025-11-26", "cost": 45.67 }
    ],
    "cost_by_project": [
      { "project": "My Project", "cost": 123.45 }
    ],
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 30.0 },
      { "model": "claude-haiku-4-5", "percentage": 20.0 }
    ]
  }
}
```

---

## Testing Your Implementation

### Quick Curl Test
```bash
curl -s http://localhost:3000/api/stats | jq '.'
```

### Verify Critical Fields
```bash
curl -s http://localhost:3000/api/stats | jq '{
  cost: .project.cost,
  tokens: .project.tokens,
  calls: .project.calls,
  models: [.chart_data.model_distribution[].model],
  percentages: [.chart_data.model_distribution[].percentage],
  recent_file: .activity[0].file_source
}'
```

### Expected Output
```json
{
  "cost": 123.45,
  "tokens": 500000,
  "calls": 1000,
  "models": [
    "claude-sonnet-4-5",
    "claude-opus-4",
    "claude-haiku-4-5"
  ],
  "percentages": [50, 30, 20],
  "recent_file": "src/main.py"
}
```

### Sum Percentages to Verify
```bash
curl -s http://localhost:3000/api/stats | \
  jq '[.chart_data.model_distribution[].percentage] | add'
# Should output: 100
```

---

## Response Time Requirements

| Scenario | Requirement |
|----------|-------------|
| Cached response | < 10ms |
| Uncached response | < 100ms |
| TUI timeout | 5 seconds |

If response takes > 5 seconds, TUI times out and shows error.

---

## Refresh Behavior

### Current Behavior
- TUI loads stats once at startup
- Updates on daemon restart (manual 'r' key)
- No automatic periodic refresh (code ready for it)

### Future Enhancement
- Could add 5-10 second polling interval
- Already supports it in code (see `update_state()`)

### No Breaking Changes
- Response can be cached indefinitely
- TUI doesn't expect real-time updates
- Can optimize aggressively

---

## Rust Type Definitions

All types are in `src/server.rs`:

```rust
// Main response (line 225)
pub struct StatsResponse {
    pub project: ProjectInfo,
    pub machine: MachineInfo,
    pub activity: Option<Vec<ActivityEvent>>,
    pub chart_data: Option<ChartData>,
}

// Project info (line 920)
pub struct ProjectInfo {
    pub name: String,
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
    pub last_updated: String,
    pub token_breakdown: Option<HashMap<String, TokenBreakdownInfo>>,
}

// Activity event (from analytics.rs)
pub struct ActivityEvent {
    pub timestamp: String,
    pub event_type: String,
    pub agent_name: Option<String>,
    pub model: Option<String>,      // CRITICAL
    pub tokens: Option<u64>,
    pub latency_ms: Option<u64>,
    pub status: Option<String>,
    pub cost: Option<f64>,          // CRITICAL
    // Note: file_source not in struct but expected by TUI
}

// Chart data (line 901)
pub struct ChartData {
    pub cost_over_time: Vec<ChartDataPoint>,
    pub cost_by_project: Vec<ProjectChartData>,
    pub model_distribution: Vec<ModelDistribution>,
}

// Model distribution (line 895)
pub struct ModelDistribution {
    pub model: String,      // CRITICAL
    pub percentage: f64,    // CRITICAL (0-100)
}
```

---

## Implementation Checklist

### Response Structure
- [ ] `project.cost` is f64
- [ ] `project.tokens` is u64
- [ ] `project.calls` is u64
- [ ] `machine` object complete
- [ ] All timestamps ISO 8601

### Activity Array
- [ ] Has `model` field
- [ ] Has `cost` field
- [ ] Has `file_source` field
- [ ] Limited to 20 events max
- [ ] Newest events first

### Model Distribution
- [ ] `model` field uses recognized model names
- [ ] `percentage` field is 0-100
- [ ] Percentages sum to ~100%
- [ ] At least one entry

### Token Breakdown (if provided)
- [ ] Uses keys: "Sonnet", "Opus", "Haiku"
- [ ] Has: input_tokens, output_tokens, cache_read_tokens, cache_write_tokens
- [ ] All u64 values

### Edge Cases
- [ ] Handles zero cost (no crash)
- [ ] Handles empty activity (no crash)
- [ ] Handles missing optional fields gracefully
- [ ] Handles large numbers (formatting works)

### Performance
- [ ] Response time < 100ms
- [ ] JSON serialization works
- [ ] No timeout issues

### Testing
- [ ] Curl test returns valid JSON
- [ ] All critical fields present
- [ ] Model names recognized
- [ ] Percentages calculate correctly
- [ ] Concurrent requests work

---

## Debugging Guide

### If TUI shows all zeros in cost summary:
1. Check `chart_data.model_distribution` exists
2. Verify model names contain: sonnet, opus, or haiku
3. Check percentages sum to ~100%

### If recent calls are empty:
1. Verify `activity` array has items
2. Check `file_source` field exists
3. Ensure `model` and `cost` fields present

### If display is broken:
1. Check for null values in required fields
2. Verify JSON is valid (`jq` should parse it)
3. Check token counts are u64, not strings

### If response is slow:
1. Check if using cache
2. Profile database query time
3. Consider caching strategy

---

## Next Steps

1. **Read Full Spec**: Start with `STATS_ENDPOINT_SPECIFICATION.md`
2. **Understand Flow**: Review `STATS_PARSING_FLOW.md` for detailed parsing
3. **Test Cases**: Run through `STATS_ENDPOINT_TEST_CASES.md` scenarios
4. **Implement**: Build/verify endpoint returns correct JSON
5. **Test**: Use curl to validate response
6. **Deploy**: Verify TUI displays correctly

---

## Support Files

- **STATS_ENDPOINT_SPECIFICATION.md** - Formal detailed spec (50+ KB)
- **STATS_ENDPOINT_QUICK_REFERENCE.md** - Quick lookup (20 KB)
- **STATS_PARSING_FLOW.md** - Visual data flow with examples (30 KB)
- **STATS_ENDPOINT_TEST_CASES.md** - 10 test scenarios (25 KB)
- **STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md** - This file

Total documentation: ~145 KB of detailed specifications, examples, and test cases.

---

## Key Takeaway

The TUI expects the `/api/stats` endpoint to return a **JSON structure with specific required fields**:

```
✅ Required:
  - project.cost, project.tokens, project.calls
  - machine info
  - activity[].model, activity[].cost, activity[].file_source
  - chart_data.model_distribution[].model, .percentage

✅ Strongly Recommended:
  - Token breakdown by tier
  - Cost history
  - Proper timestamps

✅ TUI Does:
  - Calculates per-tier costs using percentages
  - Matches model names case-insensitively
  - Takes first 20 activity events
  - Formats large numbers with abbreviations

✅ TUI Doesn't Do:
  - Parse multiple responses
  - Handle missing critical fields gracefully (crashes)
  - Retry failed requests at app level
```

Go build it! All the info you need is in the specification files.

