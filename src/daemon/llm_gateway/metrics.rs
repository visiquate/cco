//! Cost tracking and metrics
//!
//! Tracks costs per request, agent, model, and provider with
//! atomic counters for thread-safe aggregation.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::config::ModelPricing;
use super::{RequestMetrics, Usage};

/// Maximum number of recent requests to keep in memory
const MAX_RECENT_REQUESTS: usize = 1000;

/// Cost tracker with atomic counters and per-dimension breakdowns
pub struct CostTracker {
    // Global counters (atomic for lock-free access)
    total_requests: AtomicU64,
    total_cost_micros: AtomicU64, // Store as microdollars for precision
    total_input_tokens: AtomicU64,
    total_output_tokens: AtomicU64,

    // Per-dimension breakdowns
    by_agent: RwLock<HashMap<String, AgentMetrics>>,
    by_provider: RwLock<HashMap<String, ProviderMetrics>>,
    by_model: RwLock<HashMap<String, ModelMetrics>>,
    by_project: RwLock<HashMap<String, ProjectMetrics>>,

    // Recent requests ring buffer (for TUI display)
    recent_requests: RwLock<VecDeque<RequestMetrics>>,

    // Pricing table
    pricing: PricingTable,
}

impl CostTracker {
    /// Create a new cost tracker with default pricing
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_cost_micros: AtomicU64::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
            by_agent: RwLock::new(HashMap::new()),
            by_provider: RwLock::new(HashMap::new()),
            by_model: RwLock::new(HashMap::new()),
            by_project: RwLock::new(HashMap::new()),
            recent_requests: RwLock::new(VecDeque::with_capacity(MAX_RECENT_REQUESTS)),
            pricing: PricingTable::default(),
        }
    }

    /// Record a completed request
    pub fn record(&self, metrics: &RequestMetrics) {
        // Update atomic counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_cost_micros
            .fetch_add((metrics.cost_usd * 1_000_000.0) as u64, Ordering::Relaxed);
        self.total_input_tokens
            .fetch_add(metrics.input_tokens as u64, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(metrics.output_tokens as u64, Ordering::Relaxed);

        // Update by-agent metrics
        if let Some(agent) = &metrics.agent_type {
            let mut by_agent = self.by_agent.write();
            let entry = by_agent
                .entry(agent.clone())
                .or_insert_with(AgentMetrics::new);
            entry.record(metrics);
        }

        // Update by-provider metrics
        {
            let mut by_provider = self.by_provider.write();
            let entry = by_provider
                .entry(metrics.provider.clone())
                .or_insert_with(ProviderMetrics::new);
            entry.record(metrics);
        }

        // Update by-model metrics
        {
            let mut by_model = self.by_model.write();
            let entry = by_model
                .entry(metrics.model.clone())
                .or_insert_with(ModelMetrics::new);
            entry.record(metrics);
        }

        // Update by-project metrics
        if let Some(project) = &metrics.project_id {
            let mut by_project = self.by_project.write();
            let entry = by_project
                .entry(project.clone())
                .or_insert_with(ProjectMetrics::new);
            entry.record(metrics);
        }

        // Add to recent requests
        {
            let mut recent = self.recent_requests.write();
            if recent.len() >= MAX_RECENT_REQUESTS {
                recent.pop_front();
            }
            recent.push_back(metrics.clone());
        }
    }

    /// Calculate cost for a usage
    pub fn calculate_cost(&self, model: &str, usage: &Usage) -> f64 {
        self.pricing.calculate_cost(model, usage)
    }

    /// Get total metrics snapshot
    pub fn get_totals(&self) -> TotalMetrics {
        TotalMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_cost_usd: self.total_cost_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            total_input_tokens: self.total_input_tokens.load(Ordering::Relaxed),
            total_output_tokens: self.total_output_tokens.load(Ordering::Relaxed),
        }
    }

    /// Get metrics by agent
    pub fn get_by_agent(&self) -> HashMap<String, AgentMetrics> {
        self.by_agent.read().clone()
    }

    /// Get metrics by provider
    pub fn get_by_provider(&self) -> HashMap<String, ProviderMetrics> {
        self.by_provider.read().clone()
    }

    /// Get metrics by model
    pub fn get_by_model(&self) -> HashMap<String, ModelMetrics> {
        self.by_model.read().clone()
    }

    /// Get metrics by project
    pub fn get_by_project(&self) -> HashMap<String, ProjectMetrics> {
        self.by_project.read().clone()
    }

    /// Get recent requests (for TUI)
    pub fn get_recent_requests(&self, limit: usize) -> Vec<RequestMetrics> {
        let recent = self.recent_requests.read();
        recent.iter().rev().take(limit).cloned().collect()
    }

    /// Get full metrics snapshot
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp: Utc::now(),
            totals: self.get_totals(),
            by_agent: self.get_by_agent(),
            by_provider: self.get_by_provider(),
            by_model: self.get_by_model(),
            by_project: self.get_by_project(),
        }
    }

    /// Get summary for API
    pub fn summary(&self) -> CostSummary {
        let totals = self.get_totals();

        // Calculate average latency from recent requests
        let recent = self.recent_requests.read();
        let avg_latency_ms = if recent.is_empty() {
            0.0
        } else {
            let sum: u64 = recent.iter().map(|r| r.latency_ms).sum();
            sum as f64 / recent.len() as f64
        };

        // Calculate cache tokens from recent requests (not tracked globally)
        let (total_cache_write, total_cache_read) = recent.iter().fold((0u64, 0u64), |acc, r| {
            (
                acc.0 + r.cache_write_tokens as u64,
                acc.1 + r.cache_read_tokens as u64,
            )
        });

        CostSummary {
            total_requests: totals.total_requests,
            total_cost_usd: totals.total_cost_usd,
            total_input_tokens: totals.total_input_tokens,
            total_output_tokens: totals.total_output_tokens,
            total_cache_write_tokens: total_cache_write,
            total_cache_read_tokens: total_cache_read,
            avg_latency_ms,
        }
    }

    /// Get cost by agent type
    pub fn by_agent(&self) -> HashMap<String, f64> {
        self.by_agent
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.cost_usd))
            .collect()
    }

    /// Get cost by provider
    pub fn by_provider(&self) -> HashMap<String, f64> {
        self.by_provider
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.cost_usd))
            .collect()
    }

    /// Get cost by model
    pub fn by_model(&self) -> HashMap<String, f64> {
        self.by_model
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.cost_usd))
            .collect()
    }

    /// Get cost by project
    pub fn by_project(&self) -> HashMap<String, f64> {
        self.by_project
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.cost_usd))
            .collect()
    }

    /// Get recent requests for display
    pub fn recent_requests(&self, limit: usize) -> Vec<RequestMetrics> {
        self.get_recent_requests(limit)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.total_cost_micros.store(0, Ordering::Relaxed);
        self.total_input_tokens.store(0, Ordering::Relaxed);
        self.total_output_tokens.store(0, Ordering::Relaxed);

        self.by_agent.write().clear();
        self.by_provider.write().clear();
        self.by_model.write().clear();
        self.by_project.write().clear();
        self.recent_requests.write().clear();
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Metrics Types
// ============================================================================

/// Total metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalMetrics {
    pub total_requests: u64,
    pub total_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

/// Cost summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_requests: u64,
    pub total_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub avg_latency_ms: f64,
}

/// Per-agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub requests: u64,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub avg_latency_ms: f64,
    pub last_request: Option<DateTime<Utc>>,
}

impl AgentMetrics {
    fn new() -> Self {
        Self {
            requests: 0,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            avg_latency_ms: 0.0,
            last_request: None,
        }
    }

    fn record(&mut self, metrics: &RequestMetrics) {
        self.requests += 1;
        self.cost_usd += metrics.cost_usd;
        self.input_tokens += metrics.input_tokens as u64;
        self.output_tokens += metrics.output_tokens as u64;
        // Running average
        self.avg_latency_ms = self.avg_latency_ms
            + (metrics.latency_ms as f64 - self.avg_latency_ms) / self.requests as f64;
        self.last_request = Some(metrics.timestamp);
    }
}

/// Per-provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub requests: u64,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub avg_latency_ms: f64,
    pub error_count: u64,
    pub last_request: Option<DateTime<Utc>>,
}

impl ProviderMetrics {
    fn new() -> Self {
        Self {
            requests: 0,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            avg_latency_ms: 0.0,
            error_count: 0,
            last_request: None,
        }
    }

    fn record(&mut self, metrics: &RequestMetrics) {
        self.requests += 1;
        self.cost_usd += metrics.cost_usd;
        self.input_tokens += metrics.input_tokens as u64;
        self.output_tokens += metrics.output_tokens as u64;
        self.avg_latency_ms = self.avg_latency_ms
            + (metrics.latency_ms as f64 - self.avg_latency_ms) / self.requests as f64;
        self.last_request = Some(metrics.timestamp);
    }
}

/// Per-model metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub requests: u64,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
    pub avg_latency_ms: f64,
}

impl ModelMetrics {
    fn new() -> Self {
        Self {
            requests: 0,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            avg_latency_ms: 0.0,
        }
    }

    fn record(&mut self, metrics: &RequestMetrics) {
        self.requests += 1;
        self.cost_usd += metrics.cost_usd;
        self.input_tokens += metrics.input_tokens as u64;
        self.output_tokens += metrics.output_tokens as u64;
        self.cache_write_tokens += metrics.cache_write_tokens as u64;
        self.cache_read_tokens += metrics.cache_read_tokens as u64;
        self.avg_latency_ms = self.avg_latency_ms
            + (metrics.latency_ms as f64 - self.avg_latency_ms) / self.requests as f64;
    }
}

/// Per-project metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub requests: u64,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl ProjectMetrics {
    fn new() -> Self {
        Self {
            requests: 0,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    fn record(&mut self, metrics: &RequestMetrics) {
        self.requests += 1;
        self.cost_usd += metrics.cost_usd;
        self.input_tokens += metrics.input_tokens as u64;
        self.output_tokens += metrics.output_tokens as u64;
    }
}

/// Full metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub totals: TotalMetrics,
    pub by_agent: HashMap<String, AgentMetrics>,
    pub by_provider: HashMap<String, ProviderMetrics>,
    pub by_model: HashMap<String, ModelMetrics>,
    pub by_project: HashMap<String, ProjectMetrics>,
}

// ============================================================================
// Pricing Table
// ============================================================================

/// Pricing table for cost calculation
pub struct PricingTable {
    prices: HashMap<String, ModelPricing>,
}

impl PricingTable {
    /// Calculate cost for a usage
    pub fn calculate_cost(&self, model: &str, usage: &Usage) -> f64 {
        let pricing = self.get_pricing(model);

        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;

        let cache_write_cost = usage
            .cache_creation_input_tokens
            .map(|t| (t as f64 / 1_000_000.0) * pricing.cache_write_per_million.unwrap_or(0.0))
            .unwrap_or(0.0);

        let cache_read_cost = usage
            .cache_read_input_tokens
            .map(|t| (t as f64 / 1_000_000.0) * pricing.cache_read_per_million.unwrap_or(0.0))
            .unwrap_or(0.0);

        input_cost + output_cost + cache_write_cost + cache_read_cost
    }

    /// Get pricing for a model (with fallback to default)
    fn get_pricing(&self, model: &str) -> &ModelPricing {
        // Try exact match first
        if let Some(pricing) = self.prices.get(model) {
            return pricing;
        }

        // Try prefix matching (e.g., "claude-opus-4-1" matches "claude-opus")
        for (key, pricing) in &self.prices {
            if model.starts_with(key) || key.starts_with(model) {
                return pricing;
            }
        }

        // Try tier matching
        let model_lower = model.to_lowercase();
        if model_lower.contains("opus") {
            if let Some(pricing) = self.prices.get("claude-opus-4-1") {
                return pricing;
            }
        }
        if model_lower.contains("sonnet") {
            if let Some(pricing) = self.prices.get("claude-sonnet-4-5") {
                return pricing;
            }
        }
        if model_lower.contains("haiku") {
            if let Some(pricing) = self.prices.get("claude-haiku-4-5") {
                return pricing;
            }
        }

        // Return a default (sonnet pricing as reasonable default)
        self.prices
            .get("claude-sonnet-4-5")
            .unwrap_or_else(|| self.prices.values().next().unwrap())
    }
}

impl Default for PricingTable {
    fn default() -> Self {
        let mut prices = HashMap::new();

        // Claude models (USD per million tokens)
        prices.insert(
            "claude-opus-4-1".to_string(),
            ModelPricing {
                input_per_million: 15.0,
                output_per_million: 75.0,
                cache_write_per_million: Some(18.75),
                cache_read_per_million: Some(1.50),
            },
        );
        prices.insert(
            "claude-sonnet-4-5".to_string(),
            ModelPricing {
                input_per_million: 3.0,
                output_per_million: 15.0,
                cache_write_per_million: Some(3.75),
                cache_read_per_million: Some(0.30),
            },
        );
        prices.insert(
            "claude-haiku-4-5".to_string(),
            ModelPricing {
                input_per_million: 0.80,
                output_per_million: 4.0,
                cache_write_per_million: Some(1.00),
                cache_read_per_million: Some(0.08),
            },
        );

        // Azure models (placeholder pricing)
        prices.insert(
            "gpt-5.1-codex-mini".to_string(),
            ModelPricing {
                input_per_million: 2.0,
                output_per_million: 6.0,
                cache_write_per_million: None,
                cache_read_per_million: None,
            },
        );

        // DeepSeek models
        prices.insert(
            "deepseek-chat".to_string(),
            ModelPricing {
                input_per_million: 0.27,
                output_per_million: 1.10,
                cache_write_per_million: None,
                cache_read_per_million: None,
            },
        );
        prices.insert(
            "deepseek-coder".to_string(),
            ModelPricing {
                input_per_million: 0.14,
                output_per_million: 0.28,
                cache_write_per_million: None,
                cache_read_per_million: None,
            },
        );

        // Ollama models (free/local)
        prices.insert(
            "qwen2.5-coder".to_string(),
            ModelPricing {
                input_per_million: 0.0,
                output_per_million: 0.0,
                cache_write_per_million: None,
                cache_read_per_million: None,
            },
        );

        Self { prices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_metrics() -> RequestMetrics {
        RequestMetrics {
            timestamp: Utc::now(),
            request_id: "test-123".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5".to_string(),
            agent_type: Some("code-reviewer".to_string()),
            project_id: Some("test-project".to_string()),
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            cost_usd: 0.0105, // 1000 input * 3/1M + 500 output * 15/1M
            latency_ms: 2500,
        }
    }

    #[test]
    fn test_cost_tracker_record() {
        let tracker = CostTracker::new();
        let metrics = test_metrics();

        tracker.record(&metrics);

        let totals = tracker.get_totals();
        assert_eq!(totals.total_requests, 1);
        assert_eq!(totals.total_input_tokens, 1000);
        assert_eq!(totals.total_output_tokens, 500);
    }

    #[test]
    fn test_cost_tracker_by_agent() {
        let tracker = CostTracker::new();
        let metrics = test_metrics();

        tracker.record(&metrics);

        let by_agent = tracker.get_by_agent();
        assert!(by_agent.contains_key("code-reviewer"));
        assert_eq!(by_agent["code-reviewer"].requests, 1);
    }

    #[test]
    fn test_pricing_calculation() {
        let pricing = PricingTable::default();
        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };

        // Sonnet: $3 input + $1.50 output = $4.50
        let cost = pricing.calculate_cost("claude-sonnet-4-5", &usage);
        assert!((cost - 4.5).abs() < 0.001);
    }

    #[test]
    fn test_pricing_with_cache() {
        let pricing = PricingTable::default();
        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 0,
            cache_creation_input_tokens: Some(500_000),
            cache_read_input_tokens: Some(500_000),
        };

        // Sonnet: $3 input + $1.875 cache_write + $0.15 cache_read = $5.025
        let cost = pricing.calculate_cost("claude-sonnet-4-5", &usage);
        assert!((cost - 5.025).abs() < 0.001);
    }

    #[test]
    fn test_recent_requests_limit() {
        let tracker = CostTracker::new();
        let metrics = test_metrics();

        // Add more than MAX_RECENT_REQUESTS
        for _ in 0..MAX_RECENT_REQUESTS + 100 {
            tracker.record(&metrics);
        }

        let recent = tracker.get_recent_requests(MAX_RECENT_REQUESTS + 50);
        assert_eq!(recent.len(), MAX_RECENT_REQUESTS);
    }

    #[test]
    fn test_pricing_fallback() {
        let pricing = PricingTable::default();

        // Should match claude-sonnet via prefix
        let cost = pricing.calculate_cost(
            "claude-sonnet-4-5-20250929",
            &Usage {
                input_tokens: 1_000_000,
                output_tokens: 0,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        );
        assert!((cost - 3.0).abs() < 0.001); // Sonnet input price
    }
}
