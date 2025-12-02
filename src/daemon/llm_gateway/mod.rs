//! Unified LLM Gateway
//!
//! Routes ALL LLM traffic through a single gateway for:
//! - Full cost tracking (per request, per agent, per model)
//! - Audit logging with request/response bodies
//! - Multi-provider support (Anthropic, Azure, DeepSeek, Ollama)
//! - Intelligent routing based on agent type and model tier

pub mod api;
pub mod audit;
pub mod config;
pub mod metrics;
pub mod providers;
pub mod router;

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use self::audit::AuditLogger;
use self::config::GatewayConfig;
use self::metrics::CostTracker;
use self::providers::ProviderRegistry;
use self::router::RoutingEngine;

/// Main LLM Gateway struct
pub struct LlmGateway {
    pub config: GatewayConfig,
    pub router: RoutingEngine,
    pub providers: ProviderRegistry,
    pub cost_tracker: Arc<CostTracker>,
    pub audit_logger: Arc<RwLock<AuditLogger>>,
}

impl LlmGateway {
    /// Create a new LLM Gateway from configuration
    pub async fn new(config: GatewayConfig) -> Result<Self> {
        let router = RoutingEngine::new(config.routing.clone());
        let providers = ProviderRegistry::from_config(&config.providers).await?;
        let cost_tracker = Arc::new(CostTracker::new());
        let audit_logger = Arc::new(RwLock::new(
            AuditLogger::new(&config.audit).await?,
        ));

        Ok(Self {
            config,
            router,
            providers,
            cost_tracker,
            audit_logger,
        })
    }

    /// Process a completion request through the gateway
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        // Determine routing
        let route = self.router.route(&request);
        tracing::info!(
            request_id = %request_id,
            agent_type = ?request.agent_type,
            provider = %route.provider,
            reason = %route.reason,
            "Routing request"
        );

        // Get provider and execute
        let provider = self.providers.get(&route.provider)?;
        let result = provider.complete(request.clone()).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((mut response, metrics)) => {
                // Enrich response with gateway metadata
                response.provider = route.provider.clone();
                response.latency_ms = latency_ms;
                response.cost_usd = metrics.cost_usd;

                // Track costs
                self.cost_tracker.record(&metrics);

                // Audit log
                {
                    let logger = self.audit_logger.read().await;
                    logger.log_success(
                        &request_id,
                        &request,
                        &response,
                        &metrics,
                    ).await?;
                }

                Ok(response)
            }
            Err(e) => {
                // Audit log error
                {
                    let logger = self.audit_logger.read().await;
                    logger.log_error(
                        &request_id,
                        &request,
                        &route.provider,
                        &e.to_string(),
                        latency_ms,
                    ).await?;
                }

                Err(e)
            }
        }
    }
}

// ============================================================================
// Core Types - Anthropic Messages API Compatible
// ============================================================================

/// Completion request (Anthropic Messages API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier (e.g., "claude-sonnet-4-5-20250929")
    pub model: String,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Temperature (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,

    // === Gateway metadata (extracted from request for routing) ===

    /// Agent type for routing decisions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    /// Project ID for cost attribution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

/// Completion response (Anthropic Messages API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique response ID
    pub id: String,

    /// Response type (always "message")
    #[serde(rename = "type")]
    pub response_type: String,

    /// Role (always "assistant")
    pub role: String,

    /// Model used
    pub model: String,

    /// Response content blocks
    pub content: Vec<ContentBlock>,

    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Stop sequence that triggered stop (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    /// Token usage
    pub usage: Usage,

    // === Gateway-added metadata ===

    /// Provider that handled the request
    #[serde(default)]
    pub provider: String,

    /// Request latency in milliseconds
    #[serde(default)]
    pub latency_ms: u64,

    /// Calculated cost in USD
    #[serde(default)]
    pub cost_usd: f64,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "user" or "assistant"
    pub role: String,

    /// Message content (string or array of content blocks)
    pub content: MessageContent,
}

/// Message content - either a simple string or array of content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Array of content blocks (for multi-modal)
    Blocks(Vec<ContentBlock>),
}

/// Content block in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    /// Tool use (function call)
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,

    /// Cache creation tokens (prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    /// Cache read tokens (prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// Request metrics for cost tracking
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
    pub provider: String,
    pub model: String,
    pub agent_type: Option<String>,
    pub project_id: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_write_tokens: u32,
    pub cache_read_tokens: u32,
    pub cost_usd: f64,
    pub latency_ms: u64,
}

impl RequestMetrics {
    pub fn new(
        request_id: String,
        provider: String,
        model: String,
        usage: &Usage,
        cost_usd: f64,
        latency_ms: u64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            request_id,
            provider,
            model,
            agent_type: None,
            project_id: None,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_write_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
            cache_read_tokens: usage.cache_read_input_tokens.unwrap_or(0),
            cost_usd,
            latency_ms,
        }
    }

    pub fn with_agent_type(mut self, agent_type: Option<String>) -> Self {
        self.agent_type = agent_type;
        self
    }

    pub fn with_project_id(mut self, project_id: Option<String>) -> Self {
        self.project_id = project_id;
        self
    }
}

/// Gateway state for API handlers
pub type GatewayState = Arc<LlmGateway>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_content_text_serialization() {
        let content = MessageContent::Text("Hello".to_string());
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, "\"Hello\"");
    }

    #[test]
    fn test_message_content_blocks_serialization() {
        let content = MessageContent::Blocks(vec![
            ContentBlock::Text { text: "Hello".to_string() },
        ]);
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"text\""));
    }

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert!(usage.cache_creation_input_tokens.is_none());
    }
}
