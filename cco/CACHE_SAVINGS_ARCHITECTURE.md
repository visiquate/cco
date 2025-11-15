# Cache Savings Analytics Architecture for CCO

## Overview
This document defines the architecture for tracking and displaying cache savings in the CCO proxy system.

## Core Concept
Track Moka cache hits as cost savings by calculating what would have been charged if the request went to Claude API.

## Database Schema Extensions

### SQLite Schema Updates
```sql
-- Extend api_calls table for cache tracking
ALTER TABLE api_calls ADD COLUMN cache_hit BOOLEAN DEFAULT 0;
ALTER TABLE api_calls ADD COLUMN would_be_cost REAL DEFAULT 0.0;
ALTER TABLE api_calls ADD COLUMN savings REAL DEFAULT 0.0;
ALTER TABLE api_calls ADD COLUMN cache_type TEXT; -- 'moka', 'claude_cache_read', 'claude_cache_write'

-- New cache metrics table for aggregated statistics
CREATE TABLE cache_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    period_type TEXT NOT NULL, -- 'hourly', 'daily', 'weekly', 'monthly'
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    total_requests INTEGER DEFAULT 0,
    cache_hits INTEGER DEFAULT 0,
    cache_misses INTEGER DEFAULT 0,
    hit_rate REAL DEFAULT 0.0,
    total_savings REAL DEFAULT 0.0,
    saved_input_tokens INTEGER DEFAULT 0,
    saved_output_tokens INTEGER DEFAULT 0,
    project_id INTEGER,
    model TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id),
    UNIQUE(period_type, period_start, project_id, model)
);

-- Index for efficient querying
CREATE INDEX idx_cache_metrics_period ON cache_metrics(period_type, period_start);
CREATE INDEX idx_cache_metrics_project ON cache_metrics(project_id);
CREATE INDEX idx_api_calls_cache ON api_calls(cache_hit, timestamp);
```

## Pricing Calculations

### Claude Cache Pricing Structure
```rust
pub struct CachePricing {
    pub cache_write: f64,  // Same as input tokens
    pub cache_read: f64,   // 90% discount for Opus/Sonnet, 80% for Haiku
}

pub struct ModelPricing {
    pub input: f64,        // Per 1M tokens
    pub output: f64,       // Per 1M tokens
    pub cache: CachePricing,
}

impl ModelPricing {
    pub fn claude_opus_4() -> Self {
        Self {
            input: 15.0,
            output: 75.0,
            cache: CachePricing {
                cache_write: 18.75,  // 1.25x input
                cache_read: 1.5,     // 90% discount
            },
        }
    }

    pub fn claude_sonnet_3_5() -> Self {
        Self {
            input: 3.0,
            output: 15.0,
            cache: CachePricing {
                cache_write: 3.75,   // 1.25x input
                cache_read: 0.3,     // 90% discount
            },
        }
    }

    pub fn claude_haiku() -> Self {
        Self {
            input: 0.25,
            output: 1.25,
            cache: CachePricing {
                cache_write: 0.30,   // 1.2x input
                cache_read: 0.05,    // 80% discount
            },
        }
    }
}
```

### Savings Calculation Logic
```rust
pub fn calculate_cache_savings(
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
    cache_hit: bool,
) -> CacheSavings {
    let pricing = get_model_pricing(model);

    if cache_hit {
        // Moka cache hit - saved entire API call cost
        let would_be_input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input;
        let would_be_output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;
        let would_be_total = would_be_input_cost + would_be_output_cost;

        CacheSavings {
            actual_cost: 0.0,  // Served from Moka
            would_be_cost: would_be_total,
            savings: would_be_total,
            cache_type: CacheType::Moka,
        }
    } else {
        // No cache hit - regular API cost
        CacheSavings {
            actual_cost: calculate_regular_cost(model, input_tokens, output_tokens),
            would_be_cost: calculate_regular_cost(model, input_tokens, output_tokens),
            savings: 0.0,
            cache_type: CacheType::None,
        }
    }
}

// For Claude's native cache (future enhancement)
pub fn calculate_claude_cache_savings(
    model: &str,
    cached_tokens: u32,
    new_tokens: u32,
    output_tokens: u32,
) -> CacheSavings {
    let pricing = get_model_pricing(model);

    // Claude cache read cost (90% or 80% discount)
    let cache_read_cost = (cached_tokens as f64 / 1_000_000.0) * pricing.cache.cache_read;
    let new_input_cost = (new_tokens as f64 / 1_000_000.0) * pricing.input;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;

    let actual_cost = cache_read_cost + new_input_cost + output_cost;

    // What it would cost without cache
    let total_input = cached_tokens + new_tokens;
    let would_be_cost = (total_input as f64 / 1_000_000.0) * pricing.input + output_cost;

    CacheSavings {
        actual_cost,
        would_be_cost,
        savings: would_be_cost - actual_cost,
        cache_type: CacheType::ClaudeCache,
    }
}
```

## Analytics Queries

### Core Analytics Queries
```sql
-- Total savings from all cache types
SELECT
    SUM(savings) as total_savings,
    COUNT(*) as total_cached_requests,
    AVG(cache_hit) * 100 as cache_hit_rate
FROM api_calls
WHERE timestamp > datetime('now', '-30 days');

-- Savings by cache type
SELECT
    cache_type,
    COUNT(*) as hits,
    SUM(savings) as total_savings,
    AVG(savings) as avg_savings_per_hit
FROM api_calls
WHERE cache_hit = 1
GROUP BY cache_type;

-- Project-level cache performance
SELECT
    p.name as project,
    COUNT(CASE WHEN ac.cache_hit = 1 THEN 1 END) as cache_hits,
    COUNT(*) as total_requests,
    COUNT(CASE WHEN ac.cache_hit = 1 THEN 1 END) * 100.0 / COUNT(*) as hit_rate,
    SUM(ac.savings) as project_savings,
    SUM(ac.actual_cost) as actual_cost,
    SUM(ac.would_be_cost) as would_be_cost
FROM api_calls ac
JOIN conversations c ON ac.conversation_id = c.id
JOIN projects p ON c.project_id = p.id
GROUP BY p.name
ORDER BY project_savings DESC;

-- Hourly savings trend
SELECT
    strftime('%Y-%m-%d %H:00', timestamp) as hour,
    COUNT(CASE WHEN cache_hit = 1 THEN 1 END) as cache_hits,
    SUM(savings) as hourly_savings,
    AVG(cache_hit) * 100 as hit_rate
FROM api_calls
WHERE timestamp > datetime('now', '-7 days')
GROUP BY hour
ORDER BY hour DESC;

-- Model-level cache efficiency
SELECT
    model,
    COUNT(*) as total_requests,
    COUNT(CASE WHEN cache_hit = 1 THEN 1 END) as cache_hits,
    SUM(savings) as model_savings,
    AVG(input_tokens + output_tokens) as avg_tokens_per_request
FROM api_calls
GROUP BY model
ORDER BY model_savings DESC;

-- Top cached prompts (for cache warming optimization)
SELECT
    prompt_hash,
    COUNT(*) as reuse_count,
    SUM(savings) as prompt_savings,
    MAX(input_tokens) as tokens,
    SUBSTR(prompt, 1, 100) as prompt_preview
FROM api_calls
WHERE cache_hit = 1
GROUP BY prompt_hash
ORDER BY reuse_count DESC
LIMIT 20;
```

## Dashboard Integration

### Tab 1: Current Project Metrics
```
┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐
│ Cache Hit Rate     │  │ Cache Savings      │  │ Efficiency         │
│      67%           │  │    $45.32          │  │   3.2x faster      │
│ ████████░░         │  │  27% reduction     │  │  152 cache hits    │
└────────────────────┘  └────────────────────┘  └────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ Cost Breakdown                                                        │
├─────────────────────────────────────────────────────────────────────┤
│ Actual API Costs:     $120.45                                       │
│ Moka Cache Savings:    $45.32  ████████                             │
│ Would-Be Cost:        $165.77                                       │
│                                                                      │
│ Tokens Saved: 3.2M input / 1.8M output                              │
└─────────────────────────────────────────────────────────────────────┘
```

### Tab 2: Machine-Wide Analytics
```
┌─────────────────────────────────────────────────────────────────────┐
│ Total Savings Overview                                                │
├─────────────────────────────────────────────────────────────────────┤
│ All-Time Savings:     $1,234.56                                      │
│ This Month:            $456.78                                       │
│ This Week:             $89.12                                        │
│ Today:                 $12.34                                        │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ Model-Level Cache Performance                                         │
├─────────────────────────────────────────────────────────────────────┤
│ Model              │ Requests │ Cache Hits │ Hit Rate │ Savings     │
├────────────────────┼──────────┼────────────┼──────────┼─────────────┤
│ claude-opus-4      │    250   │     175    │   70%    │  $234.56    │
│ claude-sonnet-3.5  │   1,890  │   1,323    │   70%    │  $567.89    │
│ claude-haiku       │   5,432  │   3,802    │   70%    │  $123.45    │
│ [Moka Cache]       │   7,572  │   5,300    │   70%    │  $925.90    │
└────────────────────┴──────────┴────────────┴──────────┴─────────────┘
```

## Implementation Notes

1. **Cache Key Generation**: Use SHA256 hash of (model + prompt + temperature + max_tokens)
2. **TTL Strategy**:
   - Default: 1 hour for all cached responses
   - Configurable per model or per project
3. **Cache Size Limits**:
   - Default: 1GB memory limit
   - LRU eviction policy
4. **Metrics Collection**:
   - Real-time updates on each request
   - Hourly aggregation job for cache_metrics table
5. **Cost Calculation Precision**:
   - Store costs in cents (integer) to avoid floating point issues
   - Convert to dollars only for display

## Future Enhancements

1. **Cache Warming**: Pre-populate cache with frequently used prompts
2. **Smart Invalidation**: Detect when cached responses may be stale
3. **Distributed Cache**: Redis support for multi-instance deployments
4. **Cache Analytics API**: REST endpoints for detailed cache metrics
5. **Predictive Caching**: ML model to predict which prompts to cache