# Cache Savings Analytics Documentation

## Overview

The CCO Proxy implements a sophisticated cache savings analytics system that tracks and calculates cost savings from both local (Moka) and Claude's native cache mechanisms. This document explains how savings are calculated, tracked, and displayed.

## Cache Types

### 1. Moka Local Cache
- **Type**: In-memory LRU cache
- **Location**: CCO Proxy process memory
- **Cost**: $0.00 (served locally)
- **Savings**: 100% of API cost

### 2. Claude Cache Read
- **Type**: Anthropic's server-side cache
- **Discount**: 90% for Opus/Sonnet, 80% for Haiku
- **Cost**: Reduced token pricing
- **Savings**: Difference between regular and cached pricing

### 3. Claude Cache Write
- **Type**: Creating cache entries on Anthropic's servers
- **Cost**: 25% premium over regular input tokens
- **Benefit**: Future reads at discounted rate

## Savings Calculation Methodology

### Formula for Moka Cache Hit

```
would_be_cost = (input_tokens / 1M) × input_price + (output_tokens / 1M) × output_price
actual_cost = $0.00
savings = would_be_cost
```

### Formula for Claude Cache Read

```
cache_read_cost = (cached_tokens / 1M) × cache_read_price
new_input_cost = (new_tokens / 1M) × input_price
output_cost = (output_tokens / 1M) × output_price

actual_cost = cache_read_cost + new_input_cost + output_cost
would_be_cost = ((cached_tokens + new_tokens) / 1M) × input_price + output_cost
savings = would_be_cost - actual_cost
```

### Pricing Table

| Model | Input ($/1M) | Output ($/1M) | Cache Read ($/1M) | Cache Write ($/1M) |
|-------|-------------|--------------|------------------|-------------------|
| Claude Opus 4 | $15.00 | $75.00 | $1.50 (90% off) | $18.75 (25% premium) |
| Claude Sonnet 3.5 | $3.00 | $15.00 | $0.30 (90% off) | $3.75 (25% premium) |
| Claude Haiku | $0.25 | $1.25 | $0.05 (80% off) | $0.30 (20% premium) |

## Example Calculations

### Example 1: Moka Cache Hit

**Request:**
- Model: claude-opus-4
- Input: 10,000 tokens
- Output: 2,000 tokens
- Served from: Moka cache

**Calculation:**
```
would_be_input = (10,000 / 1,000,000) × $15.00 = $0.15
would_be_output = (2,000 / 1,000,000) × $75.00 = $0.15
would_be_total = $0.30

actual_cost = $0.00 (served from local cache)
savings = $0.30
```

### Example 2: Claude Cache Read

**Request:**
- Model: claude-sonnet-3.5
- Cached tokens: 8,000
- New tokens: 2,000
- Output: 1,500 tokens

**Calculation:**
```
cache_read = (8,000 / 1,000,000) × $0.30 = $0.0024
new_input = (2,000 / 1,000,000) × $3.00 = $0.006
output = (1,500 / 1,000,000) × $15.00 = $0.0225

actual_cost = $0.0024 + $0.006 + $0.0225 = $0.0309

would_be_input = (10,000 / 1,000,000) × $3.00 = $0.03
would_be_output = $0.0225
would_be_total = $0.0525

savings = $0.0525 - $0.0309 = $0.0216
```

## Database Schema

### api_calls Table Extensions
```sql
cache_hit BOOLEAN DEFAULT 0       -- Whether request was served from cache
would_be_cost REAL DEFAULT 0.0    -- Cost if not cached
savings REAL DEFAULT 0.0          -- Amount saved
cache_type TEXT                   -- 'moka', 'claude_cache_read', etc.
```

### cache_metrics Table
```sql
period_type TEXT         -- 'hourly', 'daily', 'weekly', 'monthly'
period_start TIMESTAMP   -- Start of period
cache_hits INTEGER       -- Number of cache hits
hit_rate REAL           -- Percentage of requests cached
total_savings REAL      -- Total savings in period
```

## Analytics Queries

### Total Savings Query
```sql
SELECT
    SUM(savings) as total_savings,
    COUNT(CASE WHEN cache_hit = 1 THEN 1 END) as cache_hits,
    AVG(cache_hit) * 100 as hit_rate
FROM api_calls
WHERE timestamp > datetime('now', '-30 days');
```

### Savings by Model
```sql
SELECT
    model,
    SUM(savings) as model_savings,
    COUNT(CASE WHEN cache_hit = 1 THEN 1 END) as hits
FROM api_calls
GROUP BY model
ORDER BY model_savings DESC;
```

### Hourly Savings Trend
```sql
SELECT
    strftime('%Y-%m-%d %H:00', timestamp) as hour,
    SUM(savings) as hourly_savings,
    AVG(cache_hit) * 100 as hit_rate
FROM api_calls
WHERE timestamp > datetime('now', '-7 days')
GROUP BY hour;
```

## Dashboard Metrics

### Key Performance Indicators

1. **Cache Hit Rate**
   - Formula: `(cache_hits / total_requests) × 100`
   - Target: > 60%
   - Display: Percentage gauge

2. **Total Savings**
   - Formula: `SUM(savings)`
   - Display: Dollar amount
   - Breakdown by time period

3. **Efficiency Factor**
   - Formula: `would_be_cost / actual_cost`
   - Display: Multiplier (e.g., "3.2x")
   - Shows cost reduction factor

4. **Tokens Saved**
   - Formula: `SUM(input_tokens + output_tokens)` for cached requests
   - Display: Raw count
   - Useful for capacity planning

### Dashboard Cards

#### Current Session
```
┌─────────────────────────┐
│ Cache Performance       │
├─────────────────────────┤
│ Hit Rate:    67%        │
│ Savings:     $45.32     │
│ Requests:    1,234      │
│ Cached:      827        │
└─────────────────────────┘
```

#### Cost Comparison
```
┌─────────────────────────┐
│ Cost Analysis           │
├─────────────────────────┤
│ Would-Be:    $165.77    │
│ Actual:      $120.45    │
│ Saved:       $45.32     │
│ Reduction:   27%        │
└─────────────────────────┘
```

## Implementation Details

### Cache Key Generation

Cache keys are generated using SHA256 hash of:
- Model name
- Full prompt text
- Temperature setting
- Max tokens setting

```rust
let mut hasher = Sha256::new();
hasher.update(model.as_bytes());
hasher.update(prompt.as_bytes());
hasher.update(temperature.to_le_bytes());
hasher.update(max_tokens.to_le_bytes());
let key = hex::encode(hasher.finalize());
```

### TTL Strategy

Different TTL values based on model and use case:

| Model Type | Default TTL | Rationale |
|------------|------------|-----------|
| claude-opus | 1 hour | Expensive, cache aggressively |
| claude-sonnet | 45 minutes | Balance cost/freshness |
| claude-haiku | 30 minutes | Cheap, shorter cache |
| Self-hosted | 2 hours | No API cost |

### Memory Management

- **Max Size**: 1GB default
- **Eviction**: LRU (Least Recently Used)
- **Monitoring**: Alert at 80% capacity
- **Cleanup**: Automatic expired entry removal

## REST API Endpoints

### GET /metrics/cache

Returns current cache metrics.

**Query Parameters:**
- `project_id` (optional): Filter by project
- `period` (optional): Time period (default: -30 days)

**Response:**
```json
{
  "cache_hit_rate": 67.5,
  "total_savings": 234.56,
  "cache_hits": 1234,
  "cache_misses": 567,
  "would_be_cost": 456.78,
  "actual_cost": 222.22
}
```

### GET /metrics/cache/models

Returns cache performance by model.

**Response:**
```json
{
  "models": [
    {
      "name": "claude-opus-4",
      "hit_rate": 72.3,
      "savings": 123.45,
      "requests": 456
    }
  ]
}
```

### GET /metrics/cache/trend

Returns cache savings over time.

**Query Parameters:**
- `interval`: hourly, daily, weekly
- `period`: Time range (e.g., -7 days)

**Response:**
```json
{
  "trend": [
    {
      "timestamp": "2025-01-15T10:00:00Z",
      "savings": 12.34,
      "hit_rate": 65.2,
      "requests": 123
    }
  ]
}
```

## Best Practices

### 1. Cache Warming
Pre-populate cache with frequently used prompts:
```rust
// Load common prompts at startup
for prompt in load_common_prompts() {
    let response = fetch_from_api(prompt).await?;
    cache.insert(generate_key(prompt), response).await;
}
```

### 2. Cache Invalidation
Implement smart invalidation for stale data:
```rust
// Invalidate cache for specific patterns
cache.invalidate_pattern("weather_*").await;
cache.invalidate_older_than(Duration::hours(24)).await;
```

### 3. Monitoring Alerts

Set up alerts for:
- Hit rate < 50% (ineffective caching)
- Memory usage > 80% (capacity issues)
- Savings trend declining (usage pattern change)

### 4. A/B Testing

Compare cache strategies:
- Different TTL values
- Various cache sizes
- Key generation methods

## Troubleshooting

### Low Hit Rate

**Symptoms**: Cache hit rate < 40%

**Possible Causes**:
1. Unique prompts each time
2. TTL too short
3. Cache size too small

**Solutions**:
1. Normalize prompts before hashing
2. Increase TTL for stable content
3. Increase cache capacity

### High Memory Usage

**Symptoms**: Memory > 80% capacity

**Possible Causes**:
1. Large cached responses
2. TTL too long
3. No size limits on responses

**Solutions**:
1. Compress cached data
2. Reduce TTL
3. Limit max response size

### Incorrect Savings

**Symptoms**: Savings calculations seem wrong

**Possible Causes**:
1. Wrong pricing data
2. Token counting errors
3. Cache type misidentification

**Solutions**:
1. Verify pricing configuration
2. Check token counting logic
3. Audit cache type detection

## Future Enhancements

1. **Predictive Caching**: Use ML to predict which prompts to pre-cache
2. **Distributed Cache**: Redis for multi-instance deployments
3. **Smart TTL**: Dynamic TTL based on content type
4. **Compression**: Compress cached responses to save memory
5. **Cache Tiers**: Hot/warm/cold cache levels