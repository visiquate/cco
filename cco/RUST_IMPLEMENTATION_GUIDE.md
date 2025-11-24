# CCO Rust Implementation Guide

## Overview
This guide provides step-by-step instructions for implementing cache savings analytics and multi-model routing in the CCO proxy.

## Project Structure

```
cco/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── moka_cache.rs       # Moka cache implementation
│   │   ├── metrics.rs           # Cache metrics collection
│   │   └── savings.rs           # Cost savings calculations
│   ├── router/
│   │   ├── mod.rs
│   │   ├── config.rs            # Router configuration
│   │   ├── clients/
│   │   │   ├── mod.rs
│   │   │   ├── anthropic.rs    # Anthropic client
│   │   │   ├── openai.rs       # OpenAI-compatible client
│   │   │   ├── ollama.rs       # Ollama client
│   │   │   └── traits.rs       # LLMClient trait
│   │   ├── fallback.rs         # Fallback logic
│   │   └── pricing.rs          # Cost calculation
│   ├── analytics/
│   │   ├── mod.rs
│   │   ├── database.rs         # SQLite operations
│   │   ├── queries.rs          # Analytics queries
│   │   └── aggregation.rs     # Metrics aggregation
│   └── api/
│       ├── mod.rs
│       ├── handlers.rs         # HTTP handlers
│       └── dashboard.rs        # Dashboard endpoints
├── config/
│   ├── model-routing.json      # Routing rules
│   └── model-pricing.json      # Pricing configuration
└── migrations/
    └── 001_cache_metrics.sql   # Database migrations
```

## Dependencies (Cargo.toml)

```toml
[package]
name = "cco-proxy"
version = "0.2.0"
edition = "2021"

[dependencies]
# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Caching
moka = { version = "0.12", features = ["future"] }

# HTTP client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.13"

# Hashing
sha2 = "0.10"
hex = "0.4"

# Regex for routing
regex = "1.10"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Metrics
prometheus = "0.13"

[build-dependencies]
# Embed configs at compile time
include_dir = "2.0"
```

## Step 1: Cache Implementation

### src/cache/moka_cache.rs

```rust
use moka::future::Cache;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct MokaCache {
    cache: Arc<Cache<String, CachedResponse>>,
    metrics: Arc<CacheMetrics>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

impl MokaCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    pub fn generate_key(
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(temperature.to_le_bytes());
        hasher.update(max_tokens.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let result = self.cache.get(key).await;

        if result.is_some() {
            self.metrics.record_hit().await;
        } else {
            self.metrics.record_miss().await;
        }

        result
    }

    pub async fn insert(&self, key: String, response: CachedResponse) {
        self.cache.insert(key, response).await;
    }

    pub async fn get_metrics(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot().await
    }
}
```

### src/cache/metrics.rs

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    total_savings_cents: AtomicU64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_savings_cents: AtomicU64::new(0),
        }
    }

    pub async fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn record_savings(&self, savings_cents: u64) {
        self.total_savings_cents.fetch_add(savings_cents, Ordering::Relaxed);
    }

    pub async fn snapshot(&self) -> CacheMetricsSnapshot {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        CacheMetricsSnapshot {
            hits,
            misses,
            hit_rate: if total > 0 {
                (hits as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            total_savings: self.total_savings_cents.load(Ordering::Relaxed) as f64 / 100.0,
        }
    }
}

pub struct CacheMetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub total_savings: f64,
}
```

### src/cache/savings.rs

```rust
use crate::router::pricing::{ModelPricing, get_model_pricing};

pub struct SavingsCalculator;

impl SavingsCalculator {
    pub fn calculate_cache_savings(
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> CacheSavings {
        let pricing = get_model_pricing(model);

        // Calculate what it would have cost
        let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;
        let would_be_cost = input_cost + output_cost;

        CacheSavings {
            actual_cost: 0.0,  // Served from cache
            would_be_cost,
            savings: would_be_cost,
            cache_type: CacheType::Moka,
        }
    }

    pub fn calculate_claude_cache_savings(
        model: &str,
        cached_tokens: u32,
        new_tokens: u32,
        output_tokens: u32,
    ) -> CacheSavings {
        let pricing = get_model_pricing(model);

        let cache_read_cost = (cached_tokens as f64 / 1_000_000.0) * pricing.cache_read;
        let new_input_cost = (new_tokens as f64 / 1_000_000.0) * pricing.input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;

        let actual_cost = cache_read_cost + new_input_cost + output_cost;

        let total_input = cached_tokens + new_tokens;
        let would_be_cost = (total_input as f64 / 1_000_000.0) * pricing.input + output_cost;

        CacheSavings {
            actual_cost,
            would_be_cost,
            savings: would_be_cost - actual_cost,
            cache_type: CacheType::ClaudeCache,
        }
    }
}

pub struct CacheSavings {
    pub actual_cost: f64,
    pub would_be_cost: f64,
    pub savings: f64,
    pub cache_type: CacheType,
}

pub enum CacheType {
    None,
    Moka,
    ClaudeCache,
}
```

## Step 2: Router Implementation

### src/router/config.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouterConfig {
    pub routes: Vec<RouteRule>,
    pub fallback_chain: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteRule {
    pub pattern: String,
    pub provider: ProviderType,
    pub endpoint: String,
    pub api_key_env: Option<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Ollama,
    LocalAI,
    VLLM,
    TGI,
    Custom(String),
}

impl RouterConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Load embedded config at compile time
        let config_str = include_str!("../../config/model-routing.json");
        Ok(serde_json::from_str(config_str)?)
    }
}
```

### src/router/mod.rs

```rust
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use async_trait::async_trait;

use crate::cache::MokaCache;
use crate::router::config::{RouterConfig, RouteRule, ProviderType};
use crate::router::clients::{LLMClient, AnthropicClient, OpenAIClient, OllamaClient};

pub struct ModelRouter {
    config: RouterConfig,
    routes: Vec<CompiledRoute>,
    clients: HashMap<ProviderType, Arc<dyn LLMClient>>,
    cache: MokaCache,
    fallback_manager: FallbackManager,
}

struct CompiledRoute {
    pattern: Regex,
    rule: RouteRule,
}

impl ModelRouter {
    pub fn new(cache: MokaCache) -> anyhow::Result<Self> {
        let config = RouterConfig::load()?;

        // Compile regex patterns
        let routes = config.routes
            .iter()
            .map(|rule| {
                Ok(CompiledRoute {
                    pattern: Regex::new(&rule.pattern)?,
                    rule: rule.clone(),
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Initialize clients
        let mut clients = HashMap::new();
        clients.insert(
            ProviderType::Anthropic,
            Arc::new(AnthropicClient::new()) as Arc<dyn LLMClient>,
        );
        clients.insert(
            ProviderType::OpenAI,
            Arc::new(OpenAIClient::new()) as Arc<dyn LLMClient>,
        );
        clients.insert(
            ProviderType::Ollama,
            Arc::new(OllamaClient::new()) as Arc<dyn LLMClient>,
        );

        let fallback_manager = FallbackManager::new(config.fallback_chain.clone());

        Ok(Self {
            config,
            routes,
            clients,
            cache,
            fallback_manager,
        })
    }

    pub async fn route_request(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        // Check cache first
        let cache_key = MokaCache::generate_key(
            &request.model,
            &request.messages.last().unwrap().content,
            request.temperature.unwrap_or(1.0),
            request.max_tokens.unwrap_or(4096),
        );

        if let Some(cached) = self.cache.get(&cache_key).await {
            // Calculate savings
            let savings = SavingsCalculator::calculate_cache_savings(
                &request.model,
                cached.input_tokens,
                cached.output_tokens,
            );

            // Record in analytics
            self.record_cache_hit(
                &request,
                &cached,
                savings,
            ).await?;

            return Ok(ChatResponse::from_cached(cached));
        }

        // Find matching route
        let route = self.find_route(&request.model)?;

        // Get client for provider
        let client = self.clients
            .get(&route.rule.provider)
            .ok_or_else(|| anyhow::anyhow!("No client for provider"))?;

        // Execute with fallback support
        let response = self.fallback_manager
            .execute_with_fallback(
                &request.model,
                |model| {
                    let mut req = request.clone();
                    req.model = model;
                    client.chat_completion(&req, &route.rule)
                },
            )
            .await?;

        // Cache the response
        self.cache.insert(
            cache_key,
            CachedResponse::from_response(&response),
        ).await;

        // Record in analytics
        self.record_api_call(&request, &response).await?;

        Ok(response)
    }

    fn find_route(&self, model: &str) -> anyhow::Result<&CompiledRoute> {
        self.routes
            .iter()
            .find(|r| r.pattern.is_match(model))
            .ok_or_else(|| anyhow::anyhow!("No route found for model: {}", model))
    }
}
```

## Step 3: Database Integration

### migrations/001_cache_metrics.sql

```sql
-- Cache tracking extensions
ALTER TABLE api_calls ADD COLUMN cache_hit BOOLEAN DEFAULT 0;
ALTER TABLE api_calls ADD COLUMN would_be_cost REAL DEFAULT 0.0;
ALTER TABLE api_calls ADD COLUMN savings REAL DEFAULT 0.0;
ALTER TABLE api_calls ADD COLUMN cache_type TEXT;

-- Cache metrics table
CREATE TABLE IF NOT EXISTS cache_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    period_type TEXT NOT NULL,
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

-- Indexes
CREATE INDEX idx_cache_metrics_period ON cache_metrics(period_type, period_start);
CREATE INDEX idx_cache_metrics_project ON cache_metrics(project_id);
CREATE INDEX idx_api_calls_cache ON api_calls(cache_hit, timestamp);
```

### src/analytics/database.rs

```rust
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use chrono::{DateTime, Utc};

pub struct AnalyticsDb {
    pool: SqlitePool,
}

impl AnalyticsDb {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn record_cache_hit(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        savings: f64,
        project_id: i64,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO api_calls (
                model, input_tokens, output_tokens,
                cache_hit, would_be_cost, savings,
                actual_cost, cache_type, project_id
            )
            VALUES (?, ?, ?, 1, ?, ?, 0.0, 'moka', ?)
            "#,
            model,
            input_tokens,
            output_tokens,
            savings,
            savings,
            project_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_api_call(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        cost: f64,
        project_id: i64,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO api_calls (
                model, input_tokens, output_tokens,
                cache_hit, actual_cost, would_be_cost,
                savings, project_id
            )
            VALUES (?, ?, ?, 0, ?, ?, 0.0, ?)
            "#,
            model,
            input_tokens,
            output_tokens,
            cost,
            cost,
            project_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_cache_metrics(
        &self,
        project_id: Option<i64>,
        period: &str,
    ) -> anyhow::Result<CacheMetrics> {
        let query = if let Some(pid) = project_id {
            sqlx::query_as!(
                CacheMetrics,
                r#"
                SELECT
                    COUNT(*) as total_requests,
                    SUM(cache_hit) as cache_hits,
                    SUM(1 - cache_hit) as cache_misses,
                    AVG(cache_hit) * 100 as hit_rate,
                    SUM(savings) as total_savings
                FROM api_calls
                WHERE project_id = ?
                AND timestamp > datetime('now', ?)
                "#,
                pid,
                period
            )
        } else {
            sqlx::query_as!(
                CacheMetrics,
                r#"
                SELECT
                    COUNT(*) as total_requests,
                    SUM(cache_hit) as cache_hits,
                    SUM(1 - cache_hit) as cache_misses,
                    AVG(cache_hit) * 100 as hit_rate,
                    SUM(savings) as total_savings
                FROM api_calls
                WHERE timestamp > datetime('now', ?)
                "#,
                period
            )
        };

        Ok(query.fetch_one(&self.pool).await?)
    }
}
```

## Step 4: API Handlers

### src/api/handlers.rs

```rust
use axum::{
    extract::{Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::router::ModelRouter;
use crate::analytics::AnalyticsDb;

#[derive(Clone)]
pub struct AppState {
    pub router: Arc<ModelRouter>,
    pub db: Arc<AnalyticsDb>,
}

#[derive(Deserialize)]
pub struct MetricsQuery {
    project_id: Option<i64>,
    period: Option<String>,
}

#[derive(Serialize)]
pub struct MetricsResponse {
    pub cache_hit_rate: f64,
    pub total_savings: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub would_be_cost: f64,
    pub actual_cost: f64,
}

pub async fn get_cache_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsQuery>,
) -> Result<Json<MetricsResponse>, StatusCode> {
    let period = params.period.as_deref().unwrap_or("-30 days");

    let metrics = state.db
        .get_cache_metrics(params.project_id, period)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MetricsResponse {
        cache_hit_rate: metrics.hit_rate,
        total_savings: metrics.total_savings,
        cache_hits: metrics.cache_hits as u64,
        cache_misses: metrics.cache_misses as u64,
        would_be_cost: metrics.would_be_cost,
        actual_cost: metrics.actual_cost,
    }))
}

pub async fn chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let response = state.router
        .route_request(request)
        .await
        .map_err(|e| {
            tracing::error!("Chat completion error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(response))
}
```

## Step 5: Main Application

### src/main.rs

```rust
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod router;
mod analytics;
mod api;

use crate::cache::MokaCache;
use crate::router::ModelRouter;
use crate::analytics::AnalyticsDb;
use crate::api::{handlers, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize cache (1GB, 1 hour TTL)
    let cache = MokaCache::new(1_073_741_824, 3600);

    // Initialize router
    let router = Arc::new(ModelRouter::new(cache)?);

    // Initialize database
    let db = Arc::new(AnalyticsDb::new("sqlite://analytics.db").await?);

    // Create app state
    let state = AppState { router, db };

    // Build routes
    let app = Router::new()
        .route("/v1/chat/completions", post(handlers::chat_completion))
        .route("/metrics/cache", get(handlers::get_cache_metrics))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    tracing::info!("CCO Proxy listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
```

## Testing Strategy

### tests/cache_tests.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit_savings() {
        let cache = MokaCache::new(1024 * 1024, 60);

        let response = CachedResponse {
            content: "Test response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cached_at: chrono::Utc::now(),
        };

        let key = MokaCache::generate_key(
            "claude-opus-4",
            "test prompt",
            1.0,
            4096,
        );

        cache.insert(key.clone(), response.clone()).await;

        let cached = cache.get(&key).await;
        assert!(cached.is_some());

        let savings = SavingsCalculator::calculate_cache_savings(
            "claude-opus-4",
            1000,
            500,
        );

        assert_eq!(savings.actual_cost, 0.0);
        assert!(savings.savings > 0.0);
    }

    #[tokio::test]
    async fn test_model_routing() {
        let cache = MokaCache::new(1024 * 1024, 60);
        let router = ModelRouter::new(cache).unwrap();

        let route = router.find_route("claude-opus-4");
        assert!(route.is_ok());

        let route = router.find_route("gpt-4");
        assert!(route.is_ok());

        let route = router.find_route("ollama/llama3");
        assert!(route.is_ok());
    }
}
```

## Deployment Configuration

### docker-compose.yml

```yaml
services:
  cco-proxy:
    build: .
    ports:
      - "3000:3000"
    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - DATABASE_URL=sqlite:///data/analytics.db
      - RUST_LOG=info
    volumes:
      - ./data:/data
      - ./config:/config
    restart: unless-stopped
```

### Dockerfile

```dockerfile
FROM rust:1.75 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cco-proxy /usr/local/bin/
COPY --from=builder /app/config /config
COPY --from=builder /app/migrations /migrations

EXPOSE 3000

CMD ["cco-proxy"]
```

## Performance Optimization

1. **Cache Key Strategy**:
   - Use SHA256 for consistent hashing
   - Include all request parameters that affect response

2. **Memory Management**:
   - Set appropriate cache size limits
   - Use LRU eviction policy
   - Monitor memory usage

3. **Database Optimization**:
   - Use prepared statements
   - Batch inserts for metrics
   - Regular VACUUM operations

4. **Concurrent Request Handling**:
   - Use Arc for shared state
   - Async/await for non-blocking I/O
   - Connection pooling for database

## Monitoring & Observability

1. **Metrics to Track**:
   - Cache hit rate
   - Response times
   - Cost savings
   - Error rates
   - Model usage distribution

2. **Logging**:
   - Structured logging with tracing
   - Request/response correlation IDs
   - Error context preservation

3. **Alerts**:
   - Low cache hit rate (< 50%)
   - High error rate (> 1%)
   - Unusual cost patterns
   - Memory pressure