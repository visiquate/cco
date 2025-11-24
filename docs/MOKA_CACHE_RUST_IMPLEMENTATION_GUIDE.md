# Moka Cache Implementation Guide for Rust Specialist

## Overview

This guide provides step-by-step instructions for implementing the Moka in-memory cache layer in the CCO proxy. The implementation will intercept requests, check for cached responses, and significantly reduce API costs.

## Project Structure

```
cco-proxy/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── proxy.rs           # Main proxy handler
│   ├── cache/
│   │   ├── mod.rs         # Cache module interface
│   │   ├── config.rs      # Cache configuration
│   │   ├── store.rs       # Moka cache implementation
│   │   ├── key.rs         # Cache key generation
│   │   └── metrics.rs     # Metrics collection
│   ├── claude/
│   │   ├── mod.rs         # Claude API client
│   │   └── types.rs       # Request/response types
│   └── analytics/
│       ├── mod.rs         # Analytics engine
│       └── dashboard.rs   # Dashboard API
├── tests/
│   └── cache_tests.rs
└── config.toml
```

## Step 1: Add Dependencies

Update `Cargo.toml`:

```toml
[package]
name = "cco-proxy"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Caching
moka = { version = "0.12", features = ["future", "sync"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Cryptography for cache keys
sha2 = "0.10"
hex = "0.4"

# HTTP client for Claude API
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Metrics
prometheus = "0.13"

# Configuration
config = "0.13"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Time handling
chrono = "0.4"

# CLI
clap = { version = "4.4", features = ["derive"] }

[dev-dependencies]
# Testing utilities
mockito = "1.2"
proptest = "1.4"
criterion = "0.5"

[[bench]]
name = "cache_benchmark"
harness = false
```

## Step 2: Cache Configuration Module

Create `src/cache/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable or disable caching
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum number of cache entries
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,

    /// Maximum memory usage in bytes
    #[serde(default = "default_max_memory_bytes")]
    pub max_memory_bytes: u64,

    /// Time-to-live in seconds
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,

    /// Time-to-idle in seconds
    #[serde(default = "default_tti_secs")]
    pub tti_secs: u64,

    /// Enable metrics collection
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,

    /// Cache warming on startup
    #[serde(default)]
    pub warm_on_startup: bool,

    /// Persistence path for cache snapshots
    pub persistence_path: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 10_000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ttl_secs: 86_400,                     // 24 hours
            tti_secs: 3_600,                      // 1 hour
            metrics_enabled: true,
            warm_on_startup: false,
            persistence_path: None,
        }
    }
}

// Default functions for serde
fn default_enabled() -> bool { true }
fn default_max_entries() -> u64 { 10_000 }
fn default_max_memory_bytes() -> u64 { 100 * 1024 * 1024 }
fn default_ttl_secs() -> u64 { 86_400 }
fn default_tti_secs() -> u64 { 3_600 }
fn default_metrics_enabled() -> bool { true }

impl CacheConfig {
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_secs)
    }

    pub fn tti(&self) -> Duration {
        Duration::from_secs(self.tti_secs)
    }
}
```

## Step 3: Cache Key Generation

Create `src/cache/key.rs`:

```rust
use sha2::{Sha256, Digest};
use serde_json::Value;

/// Generate a deterministic cache key from request parameters
pub fn generate_cache_key(
    model: &str,
    messages: &[Message],
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    system: Option<&str>,
    cache_control: Option<&Value>,
) -> String {
    let mut hasher = Sha256::new();

    // Hash model
    hasher.update(model.as_bytes());

    // Hash temperature (normalized to 2 decimal places)
    let temp = temperature.unwrap_or(1.0);
    hasher.update(format!("{:.2}", temp).as_bytes());

    // Hash max_tokens
    if let Some(tokens) = max_tokens {
        hasher.update(tokens.to_le_bytes());
    }

    // Hash system prompt
    if let Some(system_prompt) = system {
        hasher.update(b"system:");
        hasher.update(system_prompt.as_bytes());
    }

    // Hash messages in order
    for message in messages {
        hasher.update(message.role.as_bytes());
        hasher.update(b":");
        hasher.update(message.content.as_bytes());
        hasher.update(b"|");
    }

    // Hash cache control if present (for prompt caching awareness)
    if let Some(cc) = cache_control {
        hasher.update(b"cache_control:");
        hasher.update(cc.to_string().as_bytes());
    }

    // Generate hex string
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }
        ];

        let key1 = generate_cache_key(
            "claude-3-opus",
            &messages,
            Some(0.7),
            Some(1000),
            None,
            None,
        );

        let key2 = generate_cache_key(
            "claude-3-opus",
            &messages,
            Some(0.7),
            Some(1000),
            None,
            None,
        );

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_params() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }
        ];

        let key1 = generate_cache_key(
            "claude-3-opus",
            &messages,
            Some(0.7),
            Some(1000),
            None,
            None,
        );

        let key2 = generate_cache_key(
            "claude-3-opus",
            &messages,
            Some(0.8), // Different temperature
            Some(1000),
            None,
            None,
        );

        assert_ne!(key1, key2);
    }
}
```

## Step 4: Moka Cache Store Implementation

Create `src/cache/store.rs`:

```rust
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::config::CacheConfig;
use super::metrics::CacheMetrics;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub request_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl CachedResponse {
    /// Estimate memory size in bytes
    pub fn size_bytes(&self) -> u64 {
        (self.content.len() +
         self.model.len() +
         self.request_hash.len() +
         64) as u64 // 64 bytes for metadata
    }
}

pub struct MokaCache {
    cache: Cache<String, Arc<CachedResponse>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl MokaCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_entries)
            .weigher(|_key: &String, value: &Arc<CachedResponse>| -> u32 {
                (value.size_bytes() / 1024) as u32 // Weight in KB
            })
            .time_to_live(config.ttl())
            .time_to_idle(config.tti())
            .build();

        Self {
            cache,
            config: config.clone(),
            metrics: Arc::new(RwLock::new(CacheMetrics::new(config.metrics_enabled))),
        }
    }

    /// Get a cached response
    pub async fn get(&self, key: &str) -> Option<Arc<CachedResponse>> {
        let start = Instant::now();
        let result = self.cache.get(key).await;

        // Update metrics
        if self.config.metrics_enabled {
            let mut metrics = self.metrics.write().await;
            if result.is_some() {
                metrics.record_hit(start.elapsed());
            } else {
                metrics.record_miss();
            }
        }

        result
    }

    /// Store a response in cache
    pub async fn put(&self, key: String, response: CachedResponse) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let size = response.size_bytes();
        let response_arc = Arc::new(response);

        self.cache.insert(key.clone(), response_arc.clone()).await;

        // Update metrics
        if self.config.metrics_enabled {
            let mut metrics = self.metrics.write().await;
            metrics.record_store(size);
        }

        tracing::debug!("Cached response with key: {} (size: {} bytes)", key, size);

        Ok(())
    }

    /// Invalidate a specific entry
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await;

        if self.config.metrics_enabled {
            let mut metrics = self.metrics.write().await;
            metrics.record_eviction();
        }

        Ok(())
    }

    /// Clear the entire cache
    pub async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;

        if self.config.metrics_enabled {
            let mut metrics = self.metrics.write().await;
            metrics.reset();
        }

        tracing::info!("Cache cleared");
        Ok(())
    }

    /// Get current cache statistics
    pub async fn stats(&self) -> CacheStats {
        let metrics = self.metrics.read().await;

        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
            hits: metrics.hits,
            misses: metrics.misses,
            hit_rate: metrics.hit_rate(),
            stores: metrics.stores,
            evictions: metrics.evictions,
        }
    }

    /// Export cache for persistence
    pub async fn export(&self) -> Result<Vec<(String, CachedResponse)>> {
        // Note: Moka doesn't provide direct iteration, so we'd need to track keys separately
        // This is a simplified version
        Ok(vec![])
    }

    /// Import cache from persistence
    pub async fn import(&self, entries: Vec<(String, CachedResponse)>) -> Result<()> {
        for (key, value) in entries {
            self.cache.insert(key, Arc::new(value)).await;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub stores: u64,
    pub evictions: u64,
}
```

## Step 5: Metrics Implementation

Create `src/cache/metrics.rs`:

```rust
use std::time::Duration;
use prometheus::{Counter, Gauge, Histogram, HistogramOpts};
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE_HITS: Counter = Counter::new(
        "cco_cache_hits_total",
        "Total number of cache hits"
    ).unwrap();

    static ref CACHE_MISSES: Counter = Counter::new(
        "cco_cache_misses_total",
        "Total number of cache misses"
    ).unwrap();

    static ref CACHE_STORES: Counter = Counter::new(
        "cco_cache_stores_total",
        "Total number of cache stores"
    ).unwrap();

    static ref CACHE_EVICTIONS: Counter = Counter::new(
        "cco_cache_evictions_total",
        "Total number of cache evictions"
    ).unwrap();

    static ref CACHE_SIZE_BYTES: Gauge = Gauge::new(
        "cco_cache_size_bytes",
        "Current cache size in bytes"
    ).unwrap();

    static ref CACHE_HIT_LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "cco_cache_hit_latency_seconds",
            "Cache hit latency in seconds"
        )
    ).unwrap();

    static ref COST_SAVED_DOLLARS: Gauge = Gauge::new(
        "cco_cost_saved_dollars_total",
        "Total cost saved in dollars"
    ).unwrap();
}

pub struct CacheMetrics {
    pub enabled: bool,
    pub hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub evictions: u64,
    total_size_bytes: u64,
    total_cost_saved_cents: u64,
}

impl CacheMetrics {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            hits: 0,
            misses: 0,
            stores: 0,
            evictions: 0,
            total_size_bytes: 0,
            total_cost_saved_cents: 0,
        }
    }

    pub fn record_hit(&mut self, latency: Duration) {
        self.hits += 1;

        if self.enabled {
            CACHE_HITS.inc();
            CACHE_HIT_LATENCY.observe(latency.as_secs_f64());

            // Calculate cost saved (example: $0.30 per 1M tokens for Opus cache reads)
            // Assuming average request is 1000 tokens
            let cost_saved_cents = 3; // $0.03 saved
            self.total_cost_saved_cents += cost_saved_cents;
            COST_SAVED_DOLLARS.set((self.total_cost_saved_cents as f64) / 100.0);
        }
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;

        if self.enabled {
            CACHE_MISSES.inc();
        }
    }

    pub fn record_store(&mut self, size_bytes: u64) {
        self.stores += 1;
        self.total_size_bytes += size_bytes;

        if self.enabled {
            CACHE_STORES.inc();
            CACHE_SIZE_BYTES.set(self.total_size_bytes as f64);
        }
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;

        if self.enabled {
            CACHE_EVICTIONS.inc();
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64)
        }
    }

    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.stores = 0;
        self.evictions = 0;
        self.total_size_bytes = 0;
        self.total_cost_saved_cents = 0;

        if self.enabled {
            // Reset Prometheus metrics
            CACHE_SIZE_BYTES.set(0.0);
            COST_SAVED_DOLLARS.set(0.0);
        }
    }
}
```

## Step 6: Integration with Proxy Handler

Create/update `src/proxy.rs`:

```rust
use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::cache::{MokaCache, CachedResponse, generate_cache_key};
use crate::claude::ClaudeClient;

pub struct ProxyState {
    pub cache: Arc<MokaCache>,
    pub claude_client: Arc<ClaudeClient>,
}

/// Main request handler with cache integration
pub async fn handle_claude_request(
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<ClaudeRequest>,
) -> Result<Response, ProxyError> {
    // Generate cache key from request
    let cache_key = generate_cache_key(
        &request.model,
        &request.messages,
        request.temperature,
        request.max_tokens,
        request.system.as_deref(),
        request.cache_control.as_ref(),
    );

    tracing::debug!("Generated cache key: {}", cache_key);

    // Check cache first
    if let Some(cached) = state.cache.get(&cache_key).await {
        tracing::info!("Cache HIT for key: {}", cache_key);

        // Return cached response with cache indicator
        let response = json!({
            "id": format!("cached-{}", cached.request_hash),
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": cached.content
            }],
            "model": cached.model,
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": cached.usage.prompt_tokens,
                "output_tokens": cached.usage.completion_tokens,
                "cache_read_tokens": cached.usage.total_tokens, // All tokens from cache
                "cache_creation_tokens": 0,
            },
            "_cache_metadata": {
                "cache_hit": true,
                "cached_at": cached.cached_at,
                "cache_key": cache_key,
            }
        });

        return Ok((StatusCode::OK, Json(response)).into_response());
    }

    tracing::info!("Cache MISS for key: {}", cache_key);

    // Forward to Claude API with prompt caching enabled
    let mut claude_request = request.clone();

    // Ensure prompt caching is enabled in the request
    if claude_request.cache_control.is_none() {
        claude_request.cache_control = Some(json!({
            "type": "ephemeral"
        }));
    }

    let claude_response = state.claude_client
        .send_request(claude_request)
        .await
        .map_err(|e| ProxyError::ClaudeApiError(e.to_string()))?;

    // Extract content for caching
    let content = extract_content_from_response(&claude_response)?;
    let usage = extract_usage_from_response(&claude_response)?;

    // Store in cache for future use
    let cached_response = CachedResponse {
        content: content.clone(),
        model: request.model.clone(),
        usage,
        cached_at: chrono::Utc::now(),
        request_hash: cache_key.clone(),
    };

    if let Err(e) = state.cache.put(cache_key.clone(), cached_response).await {
        tracing::warn!("Failed to cache response: {}", e);
    }

    // Add cache metadata to response
    let mut response_value = claude_response;
    response_value["_cache_metadata"] = json!({
        "cache_hit": false,
        "cache_key": cache_key,
        "cached_for_future": true,
    });

    Ok((StatusCode::OK, Json(response_value)).into_response())
}

/// Health check endpoint with cache stats
pub async fn health_check(
    State(state): State<Arc<ProxyState>>,
) -> Result<Json<Value>, ProxyError> {
    let cache_stats = state.cache.stats().await;

    Ok(Json(json!({
        "status": "healthy",
        "cache": {
            "enabled": true,
            "entries": cache_stats.entry_count,
            "hit_rate": format!("{:.2}%", cache_stats.hit_rate * 100.0),
            "total_hits": cache_stats.hits,
            "total_misses": cache_stats.misses,
        }
    })))
}

/// Cache management endpoints
pub async fn clear_cache(
    State(state): State<Arc<ProxyState>>,
) -> Result<Json<Value>, ProxyError> {
    state.cache.clear().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Cache cleared"
    })))
}

pub async fn cache_stats(
    State(state): State<Arc<ProxyState>>,
) -> Result<Json<Value>, ProxyError> {
    let stats = state.cache.stats().await;

    Ok(Json(json!({
        "entries": stats.entry_count,
        "memory_kb": stats.weighted_size,
        "hits": stats.hits,
        "misses": stats.misses,
        "hit_rate": stats.hit_rate,
        "stores": stats.stores,
        "evictions": stats.evictions,
    })))
}

// Helper functions
fn extract_content_from_response(response: &Value) -> Result<String, ProxyError> {
    response["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ProxyError::InvalidResponse("Missing content in response".to_string()))
}

fn extract_usage_from_response(response: &Value) -> Result<Usage, ProxyError> {
    let usage = &response["usage"];

    Ok(Usage {
        prompt_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as u32,
        completion_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as u32,
        total_tokens: (usage["input_tokens"].as_u64().unwrap_or(0) +
                      usage["output_tokens"].as_u64().unwrap_or(0)) as u32,
    })
}

// Request/Response types
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub system: Option<String>,
    pub cache_control: Option<Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Claude API error: {0}")]
    ClaudeApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Cache error: {0}")]
    CacheError(#[from] anyhow::Error),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::ClaudeApiError(msg) => (StatusCode::BAD_GATEWAY, msg),
            ProxyError::InvalidResponse(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ProxyError::CacheError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

## Step 7: Main Application Setup

Create/update `src/main.rs`:

```rust
use axum::{
    routing::{get, post, delete},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod claude;
mod proxy;
mod analytics;

use cache::{MokaCache, CacheConfig};
use claude::ClaudeClient;
use proxy::{ProxyState, handle_claude_request, health_check, clear_cache, cache_stats};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable cache
    #[clap(long, default_value = "true")]
    cache_enabled: bool,

    /// Maximum cache entries
    #[clap(long, default_value = "10000")]
    cache_max_entries: u64,

    /// Maximum cache memory in MB
    #[clap(long, default_value = "100")]
    cache_max_memory_mb: u64,

    /// Cache TTL in hours
    #[clap(long, default_value = "24")]
    cache_ttl_hours: u64,

    /// Cache TTI in hours
    #[clap(long, default_value = "1")]
    cache_tti_hours: u64,

    /// Port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,

    /// Config file path
    #[clap(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cco_proxy=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let args = Args::parse();

    // Build cache configuration
    let cache_config = CacheConfig {
        enabled: args.cache_enabled,
        max_entries: args.cache_max_entries,
        max_memory_bytes: args.cache_max_memory_mb * 1024 * 1024,
        ttl_secs: args.cache_ttl_hours * 3600,
        tti_secs: args.cache_tti_hours * 3600,
        metrics_enabled: true,
        warm_on_startup: false,
        persistence_path: None,
    };

    // Initialize cache
    let cache = Arc::new(MokaCache::new(cache_config));
    tracing::info!("Cache initialized with {} max entries", args.cache_max_entries);

    // Initialize Claude client
    let claude_client = Arc::new(ClaudeClient::new()?);

    // Create application state
    let state = Arc::new(ProxyState {
        cache,
        claude_client,
    });

    // Build router
    let app = Router::new()
        // Main proxy endpoint
        .route("/v1/messages", post(handle_claude_request))

        // Health and metrics
        .route("/health", get(health_check))
        .route("/cache/stats", get(cache_stats))
        .route("/cache/clear", delete(clear_cache))

        // Prometheus metrics
        .route("/metrics", get(prometheus_metrics))

        // Add state
        .with_state(state)

        // Add CORS
        .layer(CorsLayer::permissive());

    // Start server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Starting CCO proxy on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn prometheus_metrics() -> String {
    use prometheus::Encoder;

    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

## Step 8: Testing

Create `tests/cache_tests.rs`:

```rust
use cco_proxy::cache::{MokaCache, CacheConfig, CachedResponse, generate_cache_key};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cache_hit_miss() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        ..Default::default()
    };

    let cache = MokaCache::new(config);

    let key = "test_key";
    let response = CachedResponse {
        content: "Test response".to_string(),
        model: "claude-3-opus".to_string(),
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
        cached_at: chrono::Utc::now(),
        request_hash: key.to_string(),
    };

    // Test miss
    assert!(cache.get(key).await.is_none());

    // Store
    cache.put(key.to_string(), response.clone()).await.unwrap();

    // Test hit
    let cached = cache.get(key).await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().content, "Test response");

    // Check stats
    let stats = cache.stats().await;
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.stores, 1);
}

#[tokio::test]
async fn test_cache_ttl() {
    let config = CacheConfig {
        enabled: true,
        ttl_secs: 1, // 1 second TTL
        ..Default::default()
    };

    let cache = MokaCache::new(config);

    let key = "ttl_test";
    let response = CachedResponse {
        content: "TTL test".to_string(),
        ..Default::default()
    };

    cache.put(key.to_string(), response).await.unwrap();

    // Should hit immediately
    assert!(cache.get(key).await.is_some());

    // Wait for TTL expiration
    sleep(Duration::from_secs(2)).await;

    // Should miss after TTL
    assert!(cache.get(key).await.is_none());
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = MokaCache::new(CacheConfig::default());

    // Add multiple entries
    for i in 0..10 {
        let key = format!("key_{}", i);
        let response = CachedResponse {
            content: format!("Response {}", i),
            ..Default::default()
        };
        cache.put(key, response).await.unwrap();
    }

    let stats = cache.stats().await;
    assert_eq!(stats.entry_count, 10);

    // Clear cache
    cache.clear().await.unwrap();

    let stats = cache.stats().await;
    assert_eq!(stats.entry_count, 0);
}
```

## Step 9: Benchmarks

Create `benches/cache_benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cco_proxy::cache::generate_cache_key;

fn benchmark_key_generation(c: &mut Criterion) {
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "This is a test message for benchmarking cache key generation".to_string(),
        }
    ];

    c.bench_function("generate_cache_key", |b| {
        b.iter(|| {
            generate_cache_key(
                black_box("claude-3-opus"),
                black_box(&messages),
                black_box(Some(0.7)),
                black_box(Some(1000)),
                black_box(None),
                black_box(None),
            )
        });
    });
}

criterion_group!(benches, benchmark_key_generation);
criterion_main!(benches);
```

## Step 10: Deployment Script

Create `deploy.sh`:

```bash
#!/bin/bash

# Build release version
cargo build --release

# Run with production settings
RUST_LOG=info \
ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
./target/release/cco-proxy \
    --cache-enabled true \
    --cache-max-entries 50000 \
    --cache-max-memory-mb 500 \
    --cache-ttl-hours 48 \
    --cache-tti-hours 4 \
    --port 8080
```

## Testing the Implementation

1. **Unit Test Suite**:
```bash
cargo test
```

2. **Integration Test**:
```bash
# Start the proxy
cargo run -- --cache-enabled true

# Test cache hit/miss
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'

# Check cache stats
curl http://localhost:8080/cache/stats

# View Prometheus metrics
curl http://localhost:8080/metrics
```

3. **Benchmark**:
```bash
cargo bench
```

## Monitoring Dashboard Integration

Add to your existing dashboard:

```javascript
// Dashboard API calls
async function getCacheStats() {
    const response = await fetch('/cache/stats');
    return response.json();
}

// Display cache metrics
function updateDashboard(stats) {
    document.getElementById('hit-rate').textContent =
        `${(stats.hit_rate * 100).toFixed(2)}%`;
    document.getElementById('cache-entries').textContent =
        stats.entries;
    document.getElementById('memory-usage').textContent =
        `${(stats.memory_kb / 1024).toFixed(2)} MB`;
}
```

## Performance Optimization Tips

1. **Use Arc for shared cached responses** to avoid cloning large strings
2. **Implement batch warming** for frequently used prompts
3. **Use tokio::spawn** for async cache operations to avoid blocking
4. **Consider using `FutureExt::now_or_never()` for non-blocking cache checks**
5. **Implement cache compression** for large responses (using `flate2` or `zstd`)

## Common Issues and Solutions

| Issue | Solution |
|-------|----------|
| High memory usage | Reduce `max_entries` or `max_memory_bytes` |
| Low hit rate | Increase TTL/TTI, analyze key generation |
| Slow cache operations | Check for lock contention, use Arc |
| Cache not persisting | Implement snapshot/restore functionality |

## Next Steps

1. **Add cache persistence** using `bincode` or `rmp-serde`
2. **Implement cache warming** from historical data
3. **Add distributed caching** using Redis backend
4. **Create admin UI** for cache management
5. **Implement smart eviction** based on token cost

This implementation provides a solid foundation for the Moka cache integration with room for future enhancements based on production usage patterns.