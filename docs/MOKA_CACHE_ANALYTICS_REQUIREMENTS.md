# Moka Cache Analytics Requirements

## Executive Summary

This document defines the analytics and monitoring requirements for the Moka cache integration in the CCO proxy. The analytics system will track cache performance, cost savings, and system health to optimize caching strategies and demonstrate ROI.

## Analytics Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Data Collection Layer                     │
├───────────────────────────────┬─────────────────────────────┤
│         Moka Cache            │        Proxy Handler        │
│  • Hit/Miss Events            │  • Request Metadata         │
│  • Eviction Events            │  • Response Times           │
│  • Memory Usage               │  • Token Counts             │
│  • Entry Statistics           │  • Model Types              │
└───────────────┬───────────────┴──────────────┬──────────────┘
                │                               │
                ▼                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Metrics Aggregation                       │
│  • Real-time counters (Prometheus)                          │
│  • Time-series data (InfluxDB/TimescaleDB)                 │
│  • Cost calculations                                        │
│  • Performance statistics                                   │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Analytics Endpoints                       │
├─────────────────────────────────────────────────────────────┤
│  /metrics         - Prometheus metrics export               │
│  /cache/stats     - Real-time cache statistics             │
│  /cache/analytics - Detailed analytics dashboard           │
│  /cache/report    - Cost/performance reports               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Visualization Layer                       │
├─────────────────────────────────────────────────────────────┤
│  • Real-time Dashboard (Web UI)                             │
│  • Grafana Integration                                      │
│  • Cost Reports (Daily/Weekly/Monthly)                      │
│  • Alert System                                             │
└─────────────────────────────────────────────────────────────┘
```

## Core Metrics

### 1. Performance Metrics

#### Cache Hit Rate
- **Metric**: `cache_hit_rate`
- **Type**: Gauge (0.0 - 1.0)
- **Calculation**: `hits / (hits + misses)`
- **Target**: >0.70 (70% hit rate)
- **Alert Threshold**: <0.30 (investigation needed)

#### Response Time Distribution
- **Metrics**:
  - `cache_response_time_ms` - Time for cache hits
  - `api_response_time_ms` - Time for API calls
  - `total_response_time_ms` - End-to-end time
- **Type**: Histogram with buckets [0.5, 1, 5, 10, 50, 100, 500, 1000]
- **Target**: Cache <1ms, API <200ms

#### Throughput
- **Metrics**:
  - `requests_per_second` - Total request rate
  - `cache_hits_per_second` - Cache hit rate
  - `cache_misses_per_second` - Cache miss rate
- **Type**: Counter with rate calculation
- **Target**: Handle >1000 req/s with cache

### 2. Cost Metrics

#### Token Savings
- **Metrics**:
  - `tokens_saved_total` - Cumulative tokens saved
  - `tokens_saved_per_model` - Breakdown by model
- **Calculation**:
```
tokens_saved = cache_hits * average_tokens_per_request
```

#### Cost Savings
- **Metrics**:
  - `cost_saved_usd_total` - Total savings in USD
  - `cost_saved_per_hour` - Hourly savings rate
  - `cost_saved_per_model` - Breakdown by model tier
- **Calculation**:
```
Model Pricing (per 1M tokens):
- Opus: $15.00 input, $75.00 output, $1.875 cache write, $0.1875 cache read
- Sonnet: $3.00 input, $15.00 output, $0.375 cache write, $0.0375 cache read
- Haiku: $0.25 input, $1.25 output, $0.03 cache write, $0.003 cache read

Cache Hit Savings = (input_cost + output_cost) - cache_read_cost
```

#### ROI Metrics
- **Metrics**:
  - `cache_roi_percentage` - Return on investment
  - `payback_period_days` - Time to recoup cache costs
- **Calculation**:
```
ROI = (cost_saved - infrastructure_cost) / infrastructure_cost * 100
```

### 3. Resource Metrics

#### Memory Usage
- **Metrics**:
  - `cache_memory_used_bytes` - Current memory usage
  - `cache_memory_limit_bytes` - Configured limit
  - `cache_memory_utilization` - Percentage used
- **Type**: Gauge
- **Alert Threshold**: >90% utilization

#### Entry Statistics
- **Metrics**:
  - `cache_entries_total` - Current entry count
  - `cache_entries_by_model` - Breakdown by model
  - `cache_entry_size_avg_bytes` - Average entry size
- **Type**: Gauge

#### Eviction Metrics
- **Metrics**:
  - `cache_evictions_total` - Total evictions
  - `cache_eviction_rate` - Evictions per minute
  - `cache_eviction_reason` - TTL vs size vs TTI
- **Type**: Counter with labels

### 4. Quality Metrics

#### Cache Effectiveness Score
- **Metric**: `cache_effectiveness_score`
- **Type**: Gauge (0-100)
- **Calculation**:
```
effectiveness = (
    hit_rate * 0.4 +
    cost_savings_rate * 0.3 +
    performance_gain * 0.2 +
    memory_efficiency * 0.1
) * 100
```

#### Error Rates
- **Metrics**:
  - `cache_errors_total` - Cache operation errors
  - `cache_corruption_detected` - Data integrity issues
- **Type**: Counter
- **Alert Threshold**: Any corruption detected

## Data Collection Implementation

### Event Tracking

```rust
pub enum CacheEvent {
    Hit {
        key: String,
        latency_ms: f64,
        size_bytes: u64,
        model: String,
    },
    Miss {
        key: String,
        model: String,
    },
    Store {
        key: String,
        size_bytes: u64,
        model: String,
    },
    Eviction {
        key: String,
        reason: EvictionReason,
        age_seconds: u64,
    },
}

pub struct EventCollector {
    events: Arc<Mutex<Vec<CacheEvent>>>,
    metrics: Arc<Metrics>,
}

impl EventCollector {
    pub async fn record(&self, event: CacheEvent) {
        // Update Prometheus metrics
        match event {
            CacheEvent::Hit { latency_ms, model, .. } => {
                self.metrics.cache_hits.inc();
                self.metrics.hit_latency.observe(latency_ms);
                self.metrics.hits_by_model.with_label_values(&[&model]).inc();
            }
            CacheEvent::Miss { model, .. } => {
                self.metrics.cache_misses.inc();
                self.metrics.misses_by_model.with_label_values(&[&model]).inc();
            }
            // ... other events
        }

        // Store for detailed analytics
        self.events.lock().await.push(event);
    }
}
```

### Cost Calculation Engine

```rust
pub struct CostCalculator {
    model_pricing: HashMap<String, ModelPricing>,
}

#[derive(Clone)]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_write_per_million: f64,
    pub cache_read_per_million: f64,
}

impl CostCalculator {
    pub fn calculate_savings(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> f64 {
        let pricing = &self.model_pricing[model];

        let full_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million +
                       (output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;

        let cache_cost = ((input_tokens + output_tokens) as f64 / 1_000_000.0) *
                        pricing.cache_read_per_million;

        full_cost - cache_cost
    }

    pub fn init_default() -> Self {
        let mut pricing = HashMap::new();

        // Claude Opus
        pricing.insert("claude-3-opus".to_string(), ModelPricing {
            input_per_million: 15.00,
            output_per_million: 75.00,
            cache_write_per_million: 1.875,
            cache_read_per_million: 0.1875,
        });

        // Claude Sonnet
        pricing.insert("claude-3-5-sonnet".to_string(), ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_write_per_million: 0.375,
            cache_read_per_million: 0.0375,
        });

        // Claude Haiku
        pricing.insert("claude-3-haiku".to_string(), ModelPricing {
            input_per_million: 0.25,
            output_per_million: 1.25,
            cache_write_per_million: 0.03,
            cache_read_per_million: 0.003,
        });

        Self { model_pricing: pricing }
    }
}
```

## Analytics API Endpoints

### 1. Real-time Statistics
**Endpoint**: `GET /cache/stats`

```json
{
  "current": {
    "entries": 4523,
    "memory_mb": 45.2,
    "hit_rate": 0.783,
    "avg_entry_size_kb": 10.2
  },
  "cumulative": {
    "total_hits": 152340,
    "total_misses": 42195,
    "total_evictions": 8923,
    "total_errors": 0
  },
  "performance": {
    "avg_hit_latency_ms": 0.8,
    "avg_miss_latency_ms": 152.3,
    "p95_hit_latency_ms": 1.2,
    "p95_miss_latency_ms": 245.7
  }
}
```

### 2. Cost Analytics
**Endpoint**: `GET /cache/analytics/cost`

```json
{
  "period": "24h",
  "savings": {
    "total_usd": 127.45,
    "by_model": {
      "claude-3-opus": 89.23,
      "claude-3-5-sonnet": 35.67,
      "claude-3-haiku": 2.55
    },
    "by_hour": [
      {"hour": "2024-01-15T00:00:00Z", "saved_usd": 5.23},
      {"hour": "2024-01-15T01:00:00Z", "saved_usd": 4.87}
    ]
  },
  "tokens": {
    "total_saved": 2834567,
    "by_model": {
      "claude-3-opus": 892345,
      "claude-3-5-sonnet": 1567234,
      "claude-3-haiku": 374988
    }
  },
  "roi": {
    "percentage": 2845.0,
    "break_even_hours": 0.35
  }
}
```

### 3. Performance Analytics
**Endpoint**: `GET /cache/analytics/performance`

```json
{
  "latency": {
    "cache_hits": {
      "p50": 0.5,
      "p95": 1.2,
      "p99": 2.3,
      "max": 5.6
    },
    "cache_misses": {
      "p50": 145.2,
      "p95": 245.7,
      "p99": 423.1,
      "max": 892.3
    }
  },
  "throughput": {
    "requests_per_second": 234.5,
    "hits_per_second": 183.4,
    "misses_per_second": 51.1
  },
  "speedup": {
    "average": 190.4,
    "median": 185.2
  }
}
```

### 4. Top Queries
**Endpoint**: `GET /cache/analytics/top`

```json
{
  "period": "24h",
  "top_hits": [
    {
      "query_hash": "a3f2b8c9...",
      "preview": "Explain Python async/await",
      "hits": 247,
      "model": "claude-3-5-sonnet",
      "saved_usd": 4.45
    }
  ],
  "top_misses": [
    {
      "query_pattern": "Code review for...",
      "misses": 89,
      "potential_savings_usd": 12.34
    }
  ],
  "recommendations": [
    {
      "action": "Pre-warm common queries",
      "potential_savings_usd": 34.56
    }
  ]
}
```

## Dashboard Requirements

### Real-time Dashboard Components

1. **Key Metrics Panel**
```
┌────────────────────────────────────────────┐
│             KEY METRICS                     │
├──────────────┬─────────────┬────────────────┤
│  Hit Rate    │ Cost Saved  │ Avg Response  │
│   78.3%      │  $127.45    │    0.8ms      │
│   ▲ 3.2%     │   Today     │  (vs 152ms)   │
└──────────────┴─────────────┴────────────────┘
```

2. **Time Series Charts**
- Hit rate over time (line chart)
- Cost savings accumulation (area chart)
- Response time comparison (dual-axis)
- Request volume (stacked bar)

3. **Resource Utilization**
```
Memory Usage:     ████████░░ 82% (82MB/100MB)
Entry Count:      ████░░░░░░ 45% (4,523/10,000)
Eviction Rate:    ██░░░░░░░░ 12 per minute
```

4. **Model Breakdown**
```
Model Distribution:
Opus:    ████░░░░░░ 35% ($89.23 saved)
Sonnet:  ██████░░░░ 52% ($35.67 saved)
Haiku:   ██░░░░░░░░ 13% ($2.55 saved)
```

### Alert Configuration

```yaml
alerts:
  - name: LowCacheHitRate
    condition: cache_hit_rate < 0.30
    severity: warning
    message: "Cache hit rate below 30% - investigate query patterns"

  - name: HighMemoryUsage
    condition: cache_memory_utilization > 0.90
    severity: critical
    message: "Cache memory usage above 90% - consider increasing limit"

  - name: HighEvictionRate
    condition: cache_eviction_rate > 100
    severity: warning
    message: "High eviction rate detected - cache may be undersized"

  - name: CacheErrors
    condition: cache_errors_total > 0
    severity: critical
    message: "Cache errors detected - immediate investigation required"
```

## Reporting Requirements

### Daily Report
Generated at: 00:00 UTC

```markdown
# CCO Cache Daily Report - 2024-01-15

## Executive Summary
- **Total Savings**: $342.67
- **Hit Rate**: 76.4%
- **Requests Served from Cache**: 45,234
- **Average Response Time Improvement**: 189x

## Performance Highlights
- Peak hit rate: 89.2% (14:00-15:00 UTC)
- Lowest hit rate: 45.3% (03:00-04:00 UTC)
- Total tokens saved: 7.2M

## Cost Analysis
| Model | Requests | Hits | Savings |
|-------|----------|------|---------|
| Opus | 12,345 | 9,234 | $234.56 |
| Sonnet | 23,456 | 18,234 | $98.76 |
| Haiku | 9,533 | 7,234 | $9.35 |

## Recommendations
1. Pre-warm cache with top 100 queries (potential +$45/day savings)
2. Increase TTL for stable queries (potential +12% hit rate)
3. Consider memory upgrade for peak hours
```

### Weekly Analytics Report
Generated: Mondays at 00:00 UTC

Includes:
- Week-over-week comparisons
- Usage patterns and trends
- Cost optimization opportunities
- Cache efficiency analysis
- Capacity planning recommendations

### Monthly Executive Report
Generated: First day of month

Includes:
- Total cost savings and ROI
- Infrastructure cost analysis
- Performance improvements
- Strategic recommendations
- Capacity forecasts

## Integration Requirements

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "CCO Cache Analytics",
    "panels": [
      {
        "title": "Cache Hit Rate",
        "type": "graph",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "rate(cco_cache_hits_total[5m]) / (rate(cco_cache_hits_total[5m]) + rate(cco_cache_misses_total[5m]))"
          }
        ]
      },
      {
        "title": "Cost Savings Rate",
        "type": "stat",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "rate(cco_cost_saved_dollars_total[1h])"
          }
        ]
      }
    ]
  }
}
```

### Prometheus Scrape Configuration

```yaml
scrape_configs:
  - job_name: 'cco-proxy'
    static_configs:
      - targets: ['localhost:8080']
    scrape_interval: 15s
    metrics_path: '/metrics'
```

## Data Retention Policy

| Data Type | Retention Period | Storage Location |
|-----------|------------------|------------------|
| Raw metrics | 7 days | Prometheus |
| Aggregated metrics | 90 days | TimescaleDB |
| Cost reports | 1 year | S3/Archive |
| Performance data | 30 days | InfluxDB |
| Cache entries | TTL-based | In-memory |

## Privacy and Compliance

1. **No PII in cache keys** - Use hashing for all identifiers
2. **Audit logging** - Track all cache management operations
3. **Data anonymization** - Strip sensitive data from analytics
4. **Compliance flags** - Support for GDPR/CCPA cache exclusions
5. **Encryption** - Optional encryption for cached values

## Success Metrics

### Phase 1 (Month 1)
- Achieve 50% cache hit rate
- Save $1,000 in API costs
- Reduce p95 latency by 100ms

### Phase 2 (Month 2)
- Achieve 70% cache hit rate
- Save $5,000 in API costs
- Handle 1000 req/s with <1ms cache response

### Phase 3 (Month 3)
- Achieve 80% cache hit rate
- Save $10,000+ in API costs
- Full production deployment

## Implementation Priority

1. **Core Metrics** (Week 1)
   - Hit/miss tracking
   - Basic cost calculation
   - Prometheus integration

2. **Analytics API** (Week 2)
   - Statistics endpoint
   - Cost analytics
   - Performance metrics

3. **Dashboard** (Week 3)
   - Web UI
   - Grafana integration
   - Alert system

4. **Reporting** (Week 4)
   - Daily reports
   - Weekly analytics
   - Executive dashboard

## Conclusion

The analytics system will provide comprehensive visibility into cache performance, enabling data-driven optimization and demonstrating clear ROI. With proper monitoring and analytics, the Moka cache can achieve 70%+ hit rates and save thousands of dollars per month in API costs while improving response times by 100-200x.