# Moka Local Cache Architecture for CCO Proxy

## Executive Summary

This document describes the integration of **Moka** (https://github.com/moka-rs/moka), a high-performance in-memory cache for Rust, into the Claude Code Orchestrator (CCO) proxy system. The cache layer will dramatically reduce API costs and improve response times by caching prompt-response pairs locally.

## Architecture Overview

### System Architecture with Moka Cache

```
┌─────────────────────────────────────────────────────────────┐
│                    User Request Flow                         │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  Claude Code Client                          │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                     CCO Proxy                                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                Request Handler                       │    │
│  └──────────────────┬───────────────────────────────────┘    │
│                     │                                         │
│                     ▼                                         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Cache Key Generator                     │    │
│  │         (SHA256 hash of prompt content)             │    │
│  └──────────────────┬───────────────────────────────────┘    │
│                     │                                         │
│                     ▼                                         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 Moka Cache Layer                     │    │
│  │  ┌────────────────────────────────────────────┐     │    │
│  │  │         Check Local Cache                   │     │    │
│  │  └────────────┬──────────┬────────────────────┘     │    │
│  │               │          │                           │    │
│  │          [HIT]│          │[MISS]                    │    │
│  │               │          │                           │    │
│  │               ▼          ▼                           │    │
│  │  ┌────────────────┐  ┌────────────────────────┐    │    │
│  │  │ Return Cached  │  │  Forward to Claude    │    │    │
│  │  │   Response     │  │  API with Caching     │    │    │
│  │  └────────────────┘  └───────────┬────────────┘    │    │
│  │                                   │                  │    │
│  │                                   ▼                  │    │
│  │                      ┌────────────────────────┐     │    │
│  │                      │  Store in Moka Cache   │     │    │
│  │                      └────────────────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Analytics & Metrics                     │    │
│  │  • Cache hit/miss rates                             │    │
│  │  • Cost savings tracking                            │    │
│  │  • Response time comparison                         │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│            Claude API (with prompt caching)                  │
└───────────────────────────────────────────────────────────────┘
```

## Design Rationale

### Why Local Caching?

1. **Dramatic Cost Reduction**
   - Cache hits cost $0 (only RAM usage)
   - Claude's cache read tokens: $0.30/1M (Opus), $0.03/1M (Sonnet), $0.002/1M (Haiku)
   - Potential savings: 30-70% cost reduction on repeated prompts
   - Particularly beneficial for development iterations and testing

2. **Performance Improvements**
   - Zero network latency for cache hits (<1ms response time)
   - Reduced API round-trip time (saves 50-200ms per request)
   - Lower CPU usage (no TLS handshake, no JSON serialization for network)
   - Improved developer experience with instant responses for cached content

3. **Resilience and Control**
   - Continue serving cached responses during API outages
   - Full control over cache lifetime and eviction policies
   - Independent of Claude's cache availability and pricing changes
   - Custom invalidation strategies based on business logic

4. **Development Efficiency**
   - Rapid iteration during development (instant responses for repeated queries)
   - Reduced quota consumption during testing
   - Ability to replay previous interactions without API calls
   - Cost-effective training and onboarding scenarios

### Cost Analysis

#### Current Costs (Without Local Cache)
```
Daily Usage Example (Single Developer):
- 500 requests/day average
- 2,000 tokens/request average
- Total: 1M tokens/day

Opus:   $30.00/day (write) + $0.30/day (cache read) = $30.30
Sonnet: $3.00/day (write) + $0.03/day (cache read) = $3.03
Haiku:  $0.25/day (write) + $0.002/day (cache read) = $0.252
```

#### With Moka Cache (70% Cache Hit Rate)
```
Same Usage Pattern:
- 150 requests/day to API (30% miss rate)
- 350 requests/day from cache (70% hit rate)

Opus:   $9.00/day (30% of writes) = 70% savings ($21.30/day saved)
Sonnet: $0.90/day (30% of writes) = 70% savings ($2.13/day saved)
Haiku:  $0.075/day (30% of writes) = 70% savings ($0.177/day saved)

Monthly Savings:
- Opus: ~$639/month
- Sonnet: ~$64/month
- Haiku: ~$5/month
```

#### Memory Cost
```
Cache Size Estimation:
- Average prompt: 2KB
- Average response: 4KB
- Total per entry: 6KB
- 10,000 cached entries: ~60MB RAM

Cost: Negligible (modern systems have GB of available RAM)
```

### Trade-offs

#### Advantages
- Massive cost savings (30-70% reduction)
- Sub-millisecond response times for cached content
- Complete control over caching strategy
- Resilience to API outages
- Enhanced development experience

#### Disadvantages
- Memory usage increases (10-100MB typical, up to 1GB max)
- Cache not shared across CCO instances
- Cache invalidation complexity
- Cold start penalty (empty cache on restart)
- Potential for serving stale content if not properly managed

## Cache Strategy Specification

### Cache Key Generation

```rust
use sha2::{Sha256, Digest};

fn generate_cache_key(
    model: &str,
    messages: &Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
    system_prompt: Option<&str>
) -> String {
    let mut hasher = Sha256::new();

    // Hash all relevant parameters that affect response
    hasher.update(model.as_bytes());
    hasher.update(temperature.to_le_bytes());

    if let Some(tokens) = max_tokens {
        hasher.update(tokens.to_le_bytes());
    }

    if let Some(system) = system_prompt {
        hasher.update(system.as_bytes());
    }

    // Hash message content in order
    for message in messages {
        hasher.update(message.role.as_bytes());
        hasher.update(message.content.as_bytes());
    }

    // Return hex-encoded hash
    format!("{:x}", hasher.finalize())
}
```

### Cache Configuration

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct CacheConfig {
    // Maximum number of entries
    pub max_entries: u64,           // Default: 10,000

    // Maximum memory usage in bytes
    pub max_memory_bytes: u64,      // Default: 100 MB

    // Time-to-live for entries
    pub ttl: Duration,              // Default: 24 hours

    // Time-to-idle (evict if not accessed)
    pub tti: Duration,              // Default: 1 hour

    // Enable metrics collection
    pub enable_metrics: bool,       // Default: true

    // Cache warming on startup
    pub warm_cache_on_start: bool,  // Default: false
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            ttl: Duration::from_secs(86_400),     // 24 hours
            tti: Duration::from_secs(3_600),      // 1 hour
            enable_metrics: true,
            warm_cache_on_start: false,
        }
    }
}
```

### Eviction Policy

Moka uses a sophisticated eviction algorithm combining:

1. **Window-TinyLFU** (W-TinyLFU)
   - Frequency-based eviction with recency consideration
   - Better hit rates than pure LRU or LFU
   - Adapts to changing access patterns

2. **Size-based eviction**
   - Tracks memory usage of cached values
   - Evicts entries when memory limit reached
   - Weights larger entries for earlier eviction

3. **Time-based eviction**
   - TTL (Time-to-Live): Absolute expiration
   - TTI (Time-to-Idle): Expires if not accessed
   - Configurable per-cache or per-entry

### Cache Operations

```rust
pub struct MokaCache {
    cache: Cache<String, CachedResponse>,
    metrics: Arc<Mutex<CacheMetrics>>,
}

impl MokaCache {
    /// Check cache for existing response
    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let result = self.cache.get(key).await;

        if self.metrics.is_some() {
            if result.is_some() {
                self.metrics.record_hit();
            } else {
                self.metrics.record_miss();
            }
        }

        result
    }

    /// Store response in cache
    pub async fn put(&self, key: String, response: CachedResponse) {
        let size = response.estimated_size_bytes();
        self.cache.insert(key, response).await;

        if let Some(metrics) = &self.metrics {
            metrics.record_store(size);
        }
    }

    /// Invalidate specific cache entry
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// Clear entire cache
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
            hit_rate: self.metrics.hit_rate(),
            miss_rate: self.metrics.miss_rate(),
            // ... other metrics
        }
    }
}
```

## Integration Points

### 1. Proxy Server Integration

The Moka cache integrates at the request handler level in the proxy:

```rust
// In proxy.rs
async fn handle_request(
    req: Request<Body>,
    cache: Arc<MokaCache>,
    claude_client: Arc<ClaudeClient>,
) -> Result<Response<Body>, Error> {
    // Extract and parse request
    let api_request = parse_request(req).await?;

    // Generate cache key
    let cache_key = generate_cache_key(&api_request);

    // Check cache
    if let Some(cached) = cache.get(&cache_key).await {
        // Cache hit - return immediately
        return Ok(build_response(cached));
    }

    // Cache miss - forward to Claude API
    let response = claude_client.send(api_request).await?;

    // Store in cache for future use
    cache.put(cache_key, response.clone()).await;

    Ok(build_response(response))
}
```

### 2. Analytics Engine Integration

Track and report cache performance metrics:

```rust
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    stores: AtomicU64,
    evictions: AtomicU64,
    total_bytes_saved: AtomicU64,
    total_time_saved_ms: AtomicU64,
    cost_saved_cents: AtomicU64,
}

impl CacheMetrics {
    pub fn record_hit(&self, request_size: u64) {
        self.hits.fetch_add(1, Ordering::Relaxed);

        // Calculate savings
        let tokens = request_size / 4; // Rough token estimate
        let cost_saved = calculate_cost_saved(tokens);
        self.cost_saved_cents.fetch_add(cost_saved, Ordering::Relaxed);
    }

    pub fn get_summary(&self) -> MetricsSummary {
        let total_requests = self.hits.load() + self.misses.load();
        let hit_rate = (self.hits.load() as f64) / (total_requests as f64);

        MetricsSummary {
            hit_rate,
            total_cost_saved: self.cost_saved_cents.load() as f64 / 100.0,
            avg_response_time_cached: 0.5, // ms
            avg_response_time_api: 150.0,  // ms
            // ... other metrics
        }
    }
}
```

### 3. Dashboard Integration

Display real-time cache statistics:

```rust
// API endpoint for dashboard
async fn cache_stats_endpoint(cache: Arc<MokaCache>) -> impl Reply {
    let stats = cache.stats().await;
    let metrics = cache.metrics().await;

    json(&CacheDashboard {
        current_entries: stats.entry_count,
        memory_used_mb: stats.weighted_size / (1024 * 1024),
        hit_rate_percent: metrics.hit_rate * 100.0,
        total_savings_usd: metrics.total_cost_saved,
        avg_cache_response_ms: 0.5,
        avg_api_response_ms: 150.0,
        cache_efficiency: calculate_efficiency(&stats, &metrics),
    })
}
```

## Configuration Parameters

### CLI Configuration

```bash
# Start CCO proxy with cache configuration
cco-proxy \
  --cache-enabled true \
  --cache-max-entries 10000 \
  --cache-max-memory-mb 100 \
  --cache-ttl-hours 24 \
  --cache-tti-hours 1 \
  --cache-metrics true

# Cache management commands
cco-proxy cache stats          # Show cache statistics
cco-proxy cache clear          # Clear all cached entries
cco-proxy cache invalidate <key>  # Invalidate specific entry
cco-proxy cache export         # Export cache to file
cco-proxy cache import <file>  # Import cache from file
```

### Environment Variables

```bash
# Cache configuration via environment
export CCO_CACHE_ENABLED=true
export CCO_CACHE_MAX_ENTRIES=10000
export CCO_CACHE_MAX_MEMORY_MB=100
export CCO_CACHE_TTL_HOURS=24
export CCO_CACHE_TTI_HOURS=1
export CCO_CACHE_WARM_ON_START=false
export CCO_CACHE_PERSISTENCE_PATH=/var/cache/cco
```

### Configuration File

```toml
# config.toml
[cache]
enabled = true
max_entries = 10000
max_memory_mb = 100
ttl_hours = 24
tti_hours = 1
warm_on_start = false
persistence_path = "/var/cache/cco"

[cache.metrics]
enabled = true
export_interval_seconds = 60
export_format = "prometheus"  # or "json"
```

## Implementation Guide for Rust Specialist

### Phase 1: Core Cache Implementation

1. **Add Moka dependency**
```toml
[dependencies]
moka = { version = "0.12", features = ["future", "sync"] }
sha2 = "0.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

2. **Create cache module** (`src/cache/mod.rs`)
```rust
mod config;
mod metrics;
mod key_generator;

pub use config::CacheConfig;
pub use metrics::CacheMetrics;

use moka::future::Cache;
use std::sync::Arc;

pub struct MokaCache {
    inner: Cache<String, CachedResponse>,
    config: CacheConfig,
    metrics: Arc<CacheMetrics>,
}
```

3. **Implement cache key generation** (`src/cache/key_generator.rs`)
   - Use SHA256 for consistent hashing
   - Include all parameters that affect response
   - Ensure deterministic ordering

4. **Integrate with proxy handler** (`src/proxy.rs`)
   - Add cache check before API call
   - Store responses after successful API calls
   - Handle cache errors gracefully

### Phase 2: Metrics and Analytics

1. **Implement metrics collection**
   - Track hit/miss rates
   - Calculate cost savings
   - Measure response time improvements
   - Monitor memory usage

2. **Add Prometheus exporter**
```rust
use prometheus::{Counter, Gauge, Histogram};

lazy_static! {
    static ref CACHE_HITS: Counter = Counter::new("cco_cache_hits_total", "Total cache hits").unwrap();
    static ref CACHE_MISSES: Counter = Counter::new("cco_cache_misses_total", "Total cache misses").unwrap();
    static ref CACHE_SIZE_BYTES: Gauge = Gauge::new("cco_cache_size_bytes", "Cache size in bytes").unwrap();
    static ref CACHE_RESPONSE_TIME: Histogram = Histogram::new("cco_cache_response_time_seconds", "Cache response time").unwrap();
}
```

3. **Create dashboard API endpoints**
   - `/cache/stats` - Current statistics
   - `/cache/metrics` - Prometheus metrics
   - `/cache/efficiency` - Cost/performance analysis

### Phase 3: Advanced Features

1. **Cache persistence**
   - Serialize cache to disk on shutdown
   - Restore cache on startup
   - Periodic snapshots for recovery

2. **Cache warming**
   - Load frequently used prompts on startup
   - Pre-populate from historical data
   - Background refresh of popular entries

3. **Smart invalidation**
   - Time-based invalidation for dynamic content
   - Manual invalidation via API
   - Pattern-based bulk invalidation

### Phase 4: Testing

1. **Unit tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = MokaCache::new(CacheConfig::default());
        let key = "test_key";
        let response = CachedResponse { /* ... */ };

        cache.put(key.to_string(), response.clone()).await;
        let cached = cache.get(key).await;

        assert_eq!(cached, Some(response));
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        // Test TTL, TTI, and size-based eviction
    }
}
```

2. **Integration tests**
   - End-to-end proxy tests with cache
   - Performance benchmarks
   - Memory usage tests

3. **Load testing**
   - Simulate high request volumes
   - Measure cache effectiveness
   - Identify bottlenecks

## Analytics Requirements

### Core Metrics to Track

1. **Performance Metrics**
   - Cache hit rate (%)
   - Cache miss rate (%)
   - Average response time (cache vs API)
   - P50, P95, P99 latencies
   - Requests per second

2. **Cost Metrics**
   - Tokens saved (by model tier)
   - Dollar amount saved per hour/day/month
   - Cost per request (cached vs uncached)
   - ROI of cache implementation

3. **Resource Metrics**
   - Memory usage (current/peak)
   - Entry count
   - Eviction rate
   - Cache turnover rate

4. **Quality Metrics**
   - Cache effectiveness score
   - Stale content served (if any)
   - Error rates
   - Cache availability

### Reporting Dashboard

```
┌─────────────────────────────────────────────┐
│          CCO Cache Analytics                │
├─────────────────────────────────────────────┤
│                                             │
│  Hit Rate:     ████████░░ 78.3%           │
│  Cost Saved:   $127.45 today              │
│  Avg Response: 0.8ms (cached) vs 152ms    │
│  Memory Used:  45.2 MB / 100 MB           │
│                                             │
│  Hourly Savings Trend                      │
│  $20 ┤     ╭─╮                           │
│  $15 ┤   ╭─╯ ╰─╮                         │
│  $10 ┤ ╭─╯     ╰─╮                       │
│   $5 ┤─╯         ╰─────                  │
│      └─────────────────────────────        │
│        6am   12pm   6pm   12am            │
│                                             │
│  Top Cached Queries                        │
│  1. "Explain Python async" (247 hits)     │
│  2. "React hooks tutorial" (189 hits)     │
│  3. "Rust ownership guide" (156 hits)     │
│                                             │
└─────────────────────────────────────────────┘
```

### Export Formats

1. **Prometheus Metrics**
```prometheus
# HELP cco_cache_hits_total Total number of cache hits
# TYPE cco_cache_hits_total counter
cco_cache_hits_total 15234

# HELP cco_cache_cost_saved_dollars Total cost saved in dollars
# TYPE cco_cache_cost_saved_dollars gauge
cco_cache_cost_saved_dollars 127.45

# HELP cco_cache_hit_rate Current cache hit rate
# TYPE cco_cache_hit_rate gauge
cco_cache_hit_rate 0.783
```

2. **JSON API**
```json
{
  "cache_stats": {
    "hit_rate": 0.783,
    "miss_rate": 0.217,
    "total_requests": 19453,
    "total_hits": 15234,
    "total_misses": 4219
  },
  "cost_analysis": {
    "total_saved_usd": 127.45,
    "saved_today_usd": 42.15,
    "projected_monthly_savings_usd": 1265.50
  },
  "performance": {
    "avg_cache_response_ms": 0.8,
    "avg_api_response_ms": 152,
    "speedup_factor": 190
  }
}
```

## Migration Path

### Phase 1: Development Environment (Week 1)
1. Implement basic Moka cache
2. Add cache key generation
3. Integrate with proxy handler
4. Basic metrics collection

### Phase 2: Testing & Optimization (Week 2)
1. Comprehensive testing suite
2. Performance benchmarking
3. Memory usage optimization
4. Configuration tuning

### Phase 3: Production Rollout (Week 3)
1. Canary deployment (10% traffic)
2. Monitor metrics and performance
3. Gradual rollout to 100%
4. Documentation and training

### Phase 4: Advanced Features (Week 4+)
1. Cache persistence
2. Advanced analytics
3. Cache warming strategies
4. Multi-instance coordination

## Security Considerations

1. **Cache Key Security**
   - Use cryptographic hashing (SHA256)
   - Include user context in key if needed
   - Prevent cache poisoning attacks

2. **Data Privacy**
   - Respect user privacy settings
   - Option to disable caching per request
   - Automatic PII detection and exclusion

3. **Access Control**
   - Admin-only cache management endpoints
   - Audit logging for cache operations
   - Rate limiting on cache endpoints

## Conclusion

The Moka cache integration provides a powerful, cost-effective enhancement to the CCO proxy system. With potential savings of 30-70% on API costs and sub-millisecond response times for cached content, this architecture delivers immediate value while maintaining system flexibility and control.

The implementation is straightforward, leveraging Rust's performance and Moka's battle-tested caching algorithms to create a robust, production-ready solution that scales with usage patterns and adapts to changing requirements.

## References

- [Moka Documentation](https://github.com/moka-rs/moka)
- [Window-TinyLFU Paper](https://arxiv.org/pdf/1512.00727.pdf)
- [Claude API Pricing](https://www.anthropic.com/api#pricing)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)