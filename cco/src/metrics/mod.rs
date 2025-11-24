//! Metrics module for tracking API call events and cost analysis
//!
//! This module provides the core data structures and aggregation engine for
//! monitoring Claude API usage in real-time.

pub mod engine;

pub use engine::{MetricsEngine, MetricsSummary};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Model tier for cost calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    Opus,
    Sonnet,
    Haiku,
}

impl ModelTier {
    /// Parse model tier from model name
    pub fn from_model_name(model: &str) -> Option<Self> {
        let lower = model.to_lowercase();
        if lower.contains("opus") {
            Some(ModelTier::Opus)
        } else if lower.contains("sonnet") {
            Some(ModelTier::Sonnet)
        } else if lower.contains("haiku") {
            Some(ModelTier::Haiku)
        } else {
            None
        }
    }

    /// Get pricing for this tier (per 1M tokens)
    pub fn pricing(&self) -> ModelPricing {
        match self {
            ModelTier::Opus => ModelPricing {
                input_cost: 15.0,
                output_cost: 75.0,
                cache_write_cost: 18.75,
                cache_read_cost: 1.5,
            },
            ModelTier::Sonnet => ModelPricing {
                input_cost: 3.0,
                output_cost: 15.0,
                cache_write_cost: 3.75,
                cache_read_cost: 0.3,
            },
            ModelTier::Haiku => ModelPricing {
                input_cost: 0.8,
                output_cost: 4.0,
                cache_write_cost: 1.0,
                cache_read_cost: 0.08,
            },
        }
    }
}

/// Model pricing structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost: f64,       // $ per 1M tokens
    pub output_cost: f64,      // $ per 1M tokens
    pub cache_write_cost: f64, // $ per 1M tokens
    pub cache_read_cost: f64,  // $ per 1M tokens
}

/// Token breakdown for an API call
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
}

impl TokenBreakdown {
    /// Calculate total cost for this token breakdown
    pub fn calculate_cost(&self, pricing: &ModelPricing) -> f64 {
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * pricing.input_cost;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
        let cache_write_cost = (self.cache_write_tokens as f64 / 1_000_000.0) * pricing.cache_write_cost;
        let cache_read_cost = (self.cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read_cost;

        input_cost + output_cost + cache_write_cost + cache_read_cost
    }

    /// Get total tokens (all types)
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_write_tokens + self.cache_read_tokens
    }
}

/// A single API call event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallEvent {
    /// When the call occurred
    pub timestamp: DateTime<Utc>,

    /// Model tier used
    pub model_tier: ModelTier,

    /// Full model name
    pub model_name: String,

    /// Token breakdown
    pub tokens: TokenBreakdown,

    /// Calculated cost in USD
    pub cost_usd: f64,

    /// Source file (if available from SSE stream)
    pub file_source: Option<String>,

    /// Agent name (if available)
    pub agent_name: Option<String>,
}

impl ApiCallEvent {
    /// Create a new API call event
    pub fn new(
        model_name: String,
        tokens: TokenBreakdown,
        file_source: Option<String>,
        agent_name: Option<String>,
    ) -> Option<Self> {
        let model_tier = ModelTier::from_model_name(&model_name)?;
        let pricing = model_tier.pricing();
        let cost_usd = tokens.calculate_cost(&pricing);

        Some(Self {
            timestamp: Utc::now(),
            model_tier,
            model_name,
            tokens,
            cost_usd,
            file_source,
            agent_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tier_from_name() {
        assert_eq!(
            ModelTier::from_model_name("claude-opus-4"),
            Some(ModelTier::Opus)
        );
        assert_eq!(
            ModelTier::from_model_name("claude-sonnet-3.5"),
            Some(ModelTier::Sonnet)
        );
        assert_eq!(
            ModelTier::from_model_name("claude-3-haiku-20240307"),
            Some(ModelTier::Haiku)
        );
        assert_eq!(ModelTier::from_model_name("gpt-4"), None);
    }

    #[test]
    fn test_token_breakdown_cost_calculation() {
        let tokens = TokenBreakdown {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let opus_pricing = ModelTier::Opus.pricing();
        let cost = tokens.calculate_cost(&opus_pricing);

        // $15 for 1M input + $37.5 for 500K output = $52.5
        assert!((cost - 52.5).abs() < 0.01);
    }

    #[test]
    fn test_token_breakdown_with_cache() {
        let tokens = TokenBreakdown {
            input_tokens: 0,
            output_tokens: 500_000,
            cache_write_tokens: 500_000,
            cache_read_tokens: 500_000,
        };

        let sonnet_pricing = ModelTier::Sonnet.pricing();
        let cost = tokens.calculate_cost(&sonnet_pricing);

        // $7.5 output + $1.875 cache write + $0.15 cache read = $9.525
        assert!((cost - 9.525).abs() < 0.01);
    }

    #[test]
    fn test_api_call_event_creation() {
        let tokens = TokenBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new(
            "claude-opus-4".to_string(),
            tokens,
            Some("/path/to/file.rs".to_string()),
            Some("rust-specialist".to_string()),
        );

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.model_tier, ModelTier::Opus);
        assert_eq!(event.model_name, "claude-opus-4");
        assert_eq!(event.agent_name, Some("rust-specialist".to_string()));
    }

    #[test]
    fn test_total_tokens() {
        let tokens = TokenBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 200,
            cache_read_tokens: 300,
        };

        assert_eq!(tokens.total_tokens(), 2000);
    }
}
