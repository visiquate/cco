# `/api/stats` Endpoint - Test Cases

## Overview
Test cases to validate the `/api/stats` endpoint returns correct data structure for TUI consumption.

---

## Test Case 1: Minimum Valid Response

### Setup
- No activity events
- Single model (Sonnet only)
- No token breakdown

### Expected JSON
```json
{
  "project": {
    "name": "Test Project",
    "cost": 1.0,
    "tokens": 1000,
    "calls": 10,
    "last_updated": "2025-11-26T12:00:00Z"
  },
  "machine": {
    "cpu": "Test CPU",
    "memory": "8 GB",
    "uptime": 1800,
    "process_count": 10
  }
}
```

### TUI Behavior
- [x] CostByTier loads with all zeros (no model_distribution)
- [x] RecentCall list is empty (no activity)
- [x] is_active = false
- [x] No crash on missing optional fields

---

## Test Case 2: Full Featured Response

### Setup
- All three model tiers present
- 20 activity events
- Full token breakdown
- Chart data included

### Expected JSON
```json
{
  "project": {
    "name": "Full Test",
    "cost": 100.0,
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
    "uptime": 7200,
    "process_count": 50
  },
  "activity": [
    {
      "timestamp": "2025-11-26T13:00:00Z",
      "event_type": "api_call",
      "agent_name": "Python Expert",
      "model": "claude-sonnet-4-5",
      "tokens": 5000,
      "latency_ms": 1234,
      "status": "success",
      "cost": 0.05,
      "file_source": "src/main.py"
    },
    {
      "timestamp": "2025-11-26T12:59:00Z",
      "event_type": "api_call",
      "agent_name": "Architect",
      "model": "claude-opus-4",
      "tokens": 8500,
      "latency_ms": 2100,
      "status": "success",
      "cost": 0.15,
      "file_source": "src/architecture.py"
    },
    {
      "timestamp": "2025-11-26T12:58:00Z",
      "event_type": "cache_hit",
      "model": "claude-haiku-4-5",
      "tokens": 2000,
      "status": "success",
      "cost": 0.002,
      "file_source": "cache"
    }
  ],
  "chart_data": {
    "cost_over_time": [
      { "date": "2025-11-24", "cost": 30.0 },
      { "date": "2025-11-25", "cost": 35.0 },
      { "date": "2025-11-26", "cost": 35.0 }
    ],
    "cost_by_project": [
      { "project": "Full Test", "cost": 100.0 }
    ],
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 50.0 },
      { "model": "claude-opus-4", "percentage": 30.0 },
      { "model": "claude-haiku-4-5", "percentage": 20.0 }
    ]
  }
}
```

### TUI Behavior Expectations

#### CostByTier Output
```
TUI Calculation:
  Sonnet: (100.0 * 50.0) / 100.0 = $50.00 (50.0%)
  Opus:   (100.0 * 30.0) / 100.0 = $30.00 (30.0%)
  Haiku:  (100.0 * 20.0) / 100.0 = $20.00 (20.0%)
  Total:                           $100.00

Calls Distribution:
  Sonnet: (1000 * 50) / 100 = 500
  Opus:   (1000 * 30) / 100 = 300
  Haiku:  (1000 * 20) / 100 = 200
  Total:                      1000

Token Distribution (60/40 split):
  Sonnet: input=200000, output=150000
  Opus:   input=100000, output=80000
  Haiku:  input=50000,  output=30000
```

#### RecentCall Output
```rust
vec![
  RecentCall { tier: "Sonnet", cost: 0.05, file: "src/main.py" },
  RecentCall { tier: "Opus", cost: 0.15, file: "src/architecture.py" },
  RecentCall { tier: "Haiku", cost: 0.002, file: "cache" },
]
```

#### Rendered Display
```
Tier      Cost       %     Calls  Tokens (I/O/CW/CR)
Sonnet    $ 50.00   50.0%    500   I:200K O:150K CW:10K

Opus      $ 30.00   30.0%    300   I:100K O:80K CW:5K

Haiku     $ 20.00   20.0%    200   I:50K O:30K CW:2K
──────────────────────────────────────────────────────
TOTAL     $100.00  100.0%   1000   I:350K O:260K CW:17K CR:80K

Recent API Calls (3 of 3)
Sonnet   $0.0500     src/main.py
Opus     $0.1500     src/architecture.py
Haiku    $0.0020     cache
```

---

## Test Case 3: Edge Case - Very Large Numbers

### Setup
- Large costs and token counts
- Many activity events (20+, should limit to 20)

### Expected JSON Fragment
```json
{
  "project": {
    "name": "Large Project",
    "cost": 9999.99,
    "tokens": 99999999,
    "calls": 999999,
    "last_updated": "2025-11-26T13:00:00Z"
  },
  "activity": [
    // Should have exactly 20 items (TUI takes first 20)
  ]
}
```

### TUI Behavior
- [x] Formats tokens with abbreviations: "100M" instead of "100000000"
- [x] Formats costs with decimals: "$9999.99"
- [x] Displays "Recent API Calls (20 of 21)" if 21 events provided
- [x] Limits display to available height

### Expected Calculations
```
Token formatting in display:
  99999999 → "100.00M"
  999999   → "1000.0K"
  50000    → "50.0K"

Cost formatting:
  9999.99  → "$9999.99"
  0.00001  → "$0.00001"
```

---

## Test Case 4: Edge Case - Model Names Variations

### Setup
- Model names with different formats
- Case sensitivity testing
- Old vs new model name formats

### Expected JSON Fragment
```json
{
  "activity": [
    {
      "model": "claude-sonnet-4-5-20250929",
      "cost": 0.05,
      "file_source": "test1.py"
    },
    {
      "model": "Claude-Opus-4",
      "cost": 0.10,
      "file_source": "test2.py"
    },
    {
      "model": "HAIKU",
      "cost": 0.01,
      "file_source": "test3.py"
    },
    {
      "model": "unknown-model-999",
      "cost": 0.02,
      "file_source": "test4.py"
    }
  ],
  "chart_data": {
    "model_distribution": [
      { "model": "claude-sonnet-4-5-20250929", "percentage": 50.0 },
      { "model": "Claude-Opus-4", "percentage": 30.0 },
      { "model": "HAIKU", "percentage": 15.0 },
      { "model": "unknown-model-999", "percentage": 5.0 }
    ]
  }
}
```

### TUI Behavior (Model Matching)
```
Input Model                     Matching Logic              Output Tier
─────────────────────────────────────────────────────────────────────
claude-sonnet-4-5-20250929      .contains("sonnet")        "Sonnet" ✓
Claude-Opus-4                   .to_lowercase().contains   "Opus" ✓
HAIKU                           .to_lowercase().contains   "Haiku" ✓
unknown-model-999               None match                 "Unknown" ✓
```

### Expected Recent Calls Display
```
Tier      Cost        File
Sonnet    $0.0500     test1.py
Opus      $0.1000     test2.py
Haiku     $0.0100     test3.py
Unknown   $0.0200     test4.py
```

### Expected Cost Summary Display
```
Tier      Cost       %     Calls
Sonnet    $ 50.00   50.0%    500
Opus      $ 30.00   30.0%    300
Haiku     $ 15.00   15.0%    150
Unknown   $  5.00    5.0%     50
────────────────────────────────
TOTAL     $100.00  100.0%   1000
```

---

## Test Case 5: Edge Case - Zero/Empty Values

### Setup
- Zero cost project
- No calls or tokens
- Empty activity array
- No model distribution

### Expected JSON
```json
{
  "project": {
    "name": "Empty Project",
    "cost": 0.0,
    "tokens": 0,
    "calls": 0,
    "last_updated": "2025-11-26T13:00:00Z"
  },
  "machine": {
    "cpu": "Unknown",
    "memory": "0 GB",
    "uptime": 0,
    "process_count": 0
  },
  "activity": [],
  "chart_data": {
    "cost_over_time": [],
    "cost_by_project": [],
    "model_distribution": []
  }
}
```

### TUI Behavior
- [x] CostByTier all zeros: sonnet_cost=0, opus_cost=0, haiku_cost=0
- [x] RecentCall list empty
- [x] is_active = false
- [x] Displays "Recent API Calls (None)"
- [x] Displays all percentages as 0.0%
- [x] No crash on division by zero (handled with check)

### Expected Display
```
Tier      Cost       %     Calls
Sonnet    $  0.00    0.0%      0
Opus      $  0.00    0.0%      0
Haiku     $  0.00    0.0%      0
──────────────────────────────
TOTAL     $  0.00    0.0%      0

Recent API Calls (None)
```

---

## Test Case 6: Edge Case - Missing Optional Fields

### Setup
- Activity events without file_source
- Missing agent_name
- Missing chart_data entirely
- No token_breakdown

### Expected JSON
```json
{
  "project": {
    "name": "Sparse Data",
    "cost": 50.0,
    "tokens": 100000,
    "calls": 100,
    "last_updated": "2025-11-26T13:00:00Z"
  },
  "machine": {
    "cpu": "Test",
    "memory": "8 GB",
    "uptime": 3600,
    "process_count": 20
  },
  "activity": [
    {
      "timestamp": "2025-11-26T12:00:00Z",
      "event_type": "api_call",
      "model": "claude-sonnet-4-5",
      "cost": 0.05
      // Missing: file_source, agent_name, status, etc.
    }
  ]
}
```

### TUI Behavior
- [x] Missing file_source defaults to "unknown"
- [x] Missing optional chart_data handled gracefully
- [x] No crash on Option::None fields
- [x] CostByTier defaults to all zeros (no model_distribution)

### Expected Output
```
Recent API Calls (1 of 1)
Sonnet   $0.0500     unknown

Cost Summary
Tier      Cost       %     Calls
Sonnet    $  0.00    0.0%      0
Opus      $  0.00    0.0%      0
Haiku     $  0.00    0.0%      0
──────────────────────────────
TOTAL     $  0.00    0.0%      0
```

---

## Test Case 7: Cache Hit Scenario

### Setup
- Activity with event_type="cache_hit"
- No agent_name (cache hit, not agent)
- Lower cost (cache read)

### Expected JSON Fragment
```json
{
  "activity": [
    {
      "timestamp": "2025-11-26T12:50:00Z",
      "event_type": "cache_hit",
      "model": "claude-sonnet-4-5",
      "tokens": 1000,
      "status": "success",
      "cost": 0.0001,
      "file_source": "cache"
    }
  ]
}
```

### TUI Behavior
- [x] Parses successfully (event_type is optional for parsing)
- [x] Tier matches correctly: "Sonnet"
- [x] Cost and file_source extracted
- [x] Displays in recent calls list

### Expected Output
```
Recent API Calls (1 of 1)
Sonnet   $0.0001     cache
```

---

## Test Case 8: Percentage Edge Case - Rounding

### Setup
- Model percentages that require rounding
- Unequal distribution

### Expected JSON
```json
{
  "project": {
    "cost": 100.0,
    "tokens": 1000,
    "calls": 100
  },
  "chart_data": {
    "model_distribution": [
      { "model": "claude-sonnet-4-5", "percentage": 33.333 },
      { "model": "claude-opus-4", "percentage": 33.333 },
      { "model": "claude-haiku-4-5", "percentage": 33.334 }
    ]
  }
}
```

### TUI Calculation
```
Sonnet: (100.0 * 33.333) / 100.0 = $33.333
Opus:   (100.0 * 33.333) / 100.0 = $33.333
Haiku:  (100.0 * 33.334) / 100.0 = $33.334
Total:                             $100.00 ✓

Call Distribution:
Sonnet: (100 * 33.333) / 100.0 = 33
Opus:   (100 * 33.333) / 100.0 = 33
Haiku:  (100 * 33.334) / 100.0 = 33
Total:                           99 (rounding)
```

### TUI Display (with rounding)
```
Tier      Cost       %        Calls
Sonnet    $ 33.33   33.3%      33
Opus      $ 33.33   33.3%      33
Haiku     $ 33.33   33.3%      33
─────────────────────────────────
TOTAL     $100.00  100.0%     99
```

---

## Test Case 9: Response Time Test

### Setup
- Measure actual response time
- Verify it meets TUI timeout

### Requirements
- [x] Response time: < 100ms (typical)
- [x] Cached response: < 10ms (from metrics_cache)
- [x] TUI timeout: 5 seconds (should never hit)

### Curl Test
```bash
time curl -s http://localhost:3000/api/stats | jq . > /dev/null
```

Expected output:
```
real    0m0.015s    ← < 100ms ✓
user    0m0.005s
sys     0m0.003s
```

---

## Test Case 10: Concurrent Requests

### Setup
- Multiple TUI instances or repeated rapid requests
- Verify no race conditions
- Check cache thread safety

### Test Script
```bash
#!/bin/bash
for i in {1..10}; do
  curl -s http://localhost:3000/api/stats | \
    jq '.project.cost' &
done
wait
```

### Requirements
- [x] All requests succeed
- [x] No corrupted JSON responses
- [x] Cache doesn't cause conflicts
- [x] Response times consistent

---

## Manual Testing Checklist

### Before Deployment

- [ ] Response has valid JSON structure
- [ ] `project.cost` is accurate decimal
- [ ] `project.tokens` is accurate count
- [ ] `project.calls` is accurate count
- [ ] Model names in distribution are recognized
- [ ] Percentages sum to ~100%
- [ ] Activity array limited to 20 items
- [ ] file_source field exists in activity
- [ ] Token breakdown uses "Sonnet", "Opus", "Haiku" (title case)
- [ ] Response time < 100ms
- [ ] Optional fields handle None gracefully
- [ ] All timestamps are ISO 8601 UTC
- [ ] No null values where not expected

### TUI Rendering Verification

- [ ] Cost summary table displays correctly
- [ ] All three tiers show (Sonnet, Opus, Haiku)
- [ ] Recent calls list shows tier names
- [ ] File sources display correctly
- [ ] Token abbreviations work (K, M)
- [ ] Cost formatting shows correct decimals
- [ ] Percentage calculations are accurate
- [ ] No visual glitches or text cutoff

### Edge Case Verification

- [ ] Empty project (zero cost)
- [ ] Large numbers format correctly
- [ ] Model name variations parse correctly
- [ ] Missing optional fields handled
- [ ] Cache hit events display
- [ ] Rounding doesn't break calculations
- [ ] Concurrent requests work

---

## Debugging Tips

### If CostByTier is all zeros:
```bash
# Check model_distribution exists and has correct format
curl -s http://localhost:3000/api/stats | \
  jq '.chart_data.model_distribution'

# Should show:
# [
#   { "model": "claude-sonnet-4-5", "percentage": 50 },
#   { "model": "claude-opus-4", "percentage": 30 },
#   { "model": "claude-haiku-4-5", "percentage": 20 }
# ]
```

### If RecentCall list is empty:
```bash
# Check activity array exists and has events
curl -s http://localhost:3000/api/stats | \
  jq '.activity | length'

# Should be > 0

# Check file_source field exists
curl -s http://localhost:3000/api/stats | \
  jq '.activity[0]'

# Should include: model, cost, file_source
```

### If model matching fails:
```bash
# Check model names in activity and distribution
curl -s http://localhost:3000/api/stats | \
  jq '[.activity[0].model, (.chart_data.model_distribution[0].model // empty)]'

# Ensure they contain: sonnet, opus, or haiku (case-insensitive)
```

### If percentages don't add up:
```bash
# Sum all percentages
curl -s http://localhost:3000/api/stats | \
  jq '[.chart_data.model_distribution[].percentage] | add'

# Should be approximately 100.0
```

