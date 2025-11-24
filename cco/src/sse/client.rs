//! SSE client for streaming analytics events from CCO proxy
//!
//! The SseClient connects to the CCO proxy's SSE stream endpoint and processes
//! incoming events, recording them in the shared MetricsEngine.

use crate::analytics::ActivityEvent;
use crate::metrics::{ApiCallEvent, MetricsEngine, TokenBreakdown};
use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};

/// Maximum reconnection delay (30 seconds)
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Initial reconnection delay (1 second)
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);

/// SSE stream response structure from CCO proxy
#[derive(Debug, Deserialize)]
struct SseStreamResponse {
    #[allow(dead_code)]
    project: ProjectInfo,
    #[allow(dead_code)]
    machine: MachineInfo,
    #[serde(default)]
    activity: Vec<ActivityEvent>,
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    cost: f64,
    #[allow(dead_code)]
    tokens: u64,
    #[allow(dead_code)]
    calls: u64,
    #[allow(dead_code)]
    last_updated: String,
}

#[derive(Debug, Deserialize)]
struct MachineInfo {
    #[allow(dead_code)]
    cpu: String,
    #[allow(dead_code)]
    memory: String,
    #[allow(dead_code)]
    uptime: u64,
    #[allow(dead_code)]
    process_count: u64,
}

/// SSE client for streaming analytics from CCO proxy
pub struct SseClient {
    /// CCO proxy endpoint (e.g., "http://localhost:3000")
    endpoint: String,
    /// Shared metrics engine for recording events
    metrics: Arc<MetricsEngine>,
    /// Shutdown signal
    shutdown: Arc<Mutex<bool>>,
}

impl SseClient {
    /// Create a new SSE client
    ///
    /// # Arguments
    ///
    /// * `endpoint` - CCO proxy base URL (e.g., "http://localhost:3000")
    /// * `metrics` - Shared metrics engine for recording events
    pub fn new(endpoint: String, metrics: Arc<MetricsEngine>) -> Self {
        Self {
            endpoint,
            metrics,
            shutdown: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to SSE stream and start processing events
    ///
    /// This method runs indefinitely, automatically reconnecting with exponential backoff
    /// when the connection is lost. It will only return when shutdown is signaled.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial connection setup fails. Connection errors during
    /// streaming are handled internally with automatic reconnection.
    #[allow(unused_assignments)]
    pub async fn connect(&self) -> Result<()> {
        let stream_url = format!("{}/api/stream", self.endpoint);
        info!("SSE client connecting to: {}", stream_url);

        let mut backoff = INITIAL_BACKOFF;
        let mut retry_count = 0u32;

        loop {
            // Check shutdown signal
            if *self.shutdown.lock().await {
                info!("SSE client shutting down");
                break;
            }

            trace!("SSE client attempting connection (attempt {})", retry_count + 1);

            // Create event source
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(300)) // 5 minute read timeout
                .build()
                .context("Failed to build HTTP client")?;

            let mut event_source = EventSource::new(client.get(&stream_url))
                .context("Failed to create event source")?;

            info!("SSE connection established, listening for events");

            // Reset backoff on successful connection
            backoff = INITIAL_BACKOFF;
            retry_count = 0;

            // Process events from the stream
            while let Some(event) = event_source.next().await {
                // Check shutdown signal
                if *self.shutdown.lock().await {
                    info!("SSE client shutting down during stream processing");
                    event_source.close();
                    return Ok(());
                }

                match event {
                    Ok(Event::Open) => {
                        debug!("SSE connection opened");
                    }
                    Ok(Event::Message(message)) => {
                        trace!("SSE message received: event={}", message.event);

                        // Process analytics events
                        if message.event == "analytics" {
                            if let Err(e) = self.process_analytics_event(&message.data).await {
                                warn!("Failed to process analytics event: {}", e);
                            }
                        } else if message.event == "error" {
                            error!("SSE error event received: {}", message.data);
                        }
                    }
                    Err(e) => {
                        warn!("SSE stream error: {}", e);
                        break; // Exit inner loop to trigger reconnection
                    }
                }
            }

            // Connection closed, attempt reconnection with exponential backoff
            event_source.close();

            // Check shutdown signal before attempting reconnection
            if *self.shutdown.lock().await {
                info!("SSE client shutting down, skipping reconnection");
                break;
            }

            retry_count += 1;
            warn!(
                "SSE connection lost, reconnecting in {:?} (attempt {})",
                backoff, retry_count
            );

            sleep(backoff).await;

            // Exponential backoff: double delay each time, up to MAX_BACKOFF
            backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
        }

        Ok(())
    }

    /// Signal the client to shutdown
    pub async fn shutdown(&self) {
        info!("SSE client shutdown requested");
        *self.shutdown.lock().await = true;
    }

    /// Process an analytics event from the SSE stream
    async fn process_analytics_event(&self, data: &str) -> Result<()> {
        trace!("Processing analytics event: {}", data);

        // Parse the SSE response
        let response: SseStreamResponse = serde_json::from_str(data)
            .context("Failed to parse analytics event JSON")?;

        // Convert and record each activity event
        for event in response.activity {
            // Only process API call events (skip cache hits, errors, etc.)
            if event.event_type == "api_call" {
                if let Some(api_event) = self.convert_to_api_call_event(&event) {
                    trace!(
                        "Recording API call: model={}, tokens={:?}, cost=${:.4}",
                        api_event.model_name,
                        api_event.tokens.total_tokens(),
                        api_event.cost_usd
                    );
                    self.metrics.record_event(api_event).await;
                }
            }
        }

        Ok(())
    }

    /// Convert an ActivityEvent from the SSE stream to an ApiCallEvent for the MetricsEngine
    fn convert_to_api_call_event(&self, event: &ActivityEvent) -> Option<ApiCallEvent> {
        // Extract required fields
        let model_name = event.model.as_ref()?.clone();
        let tokens = event.tokens?;

        // Create token breakdown (assume no cache tokens from SSE stream for now)
        // In Phase 1b, we'll enhance this with actual cache token data
        let token_breakdown = TokenBreakdown {
            input_tokens: tokens / 2, // Approximate split
            output_tokens: tokens / 2,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        // Create API call event
        ApiCallEvent::new(
            model_name,
            token_breakdown,
            None, // file_source not available in current ActivityEvent
            event.agent_name.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn test_sse_client_creation() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics);

        assert_eq!(client.endpoint, "http://localhost:3000");
    }

    #[tokio::test]
    async fn test_process_analytics_event() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics.clone());

        // Create a mock SSE response
        let event_data = r#"{
            "project": {
                "name": "test-project",
                "cost": 1.23,
                "tokens": 1000,
                "calls": 5,
                "last_updated": "2025-11-17T12:00:00Z"
            },
            "machine": {
                "cpu": "Test CPU",
                "memory": "16GB",
                "uptime": 3600,
                "process_count": 10
            },
            "activity": [
                {
                    "timestamp": "2025-11-17T12:00:00Z",
                    "event_type": "api_call",
                    "agent_name": "test-agent",
                    "model": "claude-sonnet-4.5",
                    "tokens": 1000,
                    "latency_ms": 150,
                    "status": "success",
                    "cost": 0.05
                }
            ]
        }"#;

        // Process the event
        client.process_analytics_event(event_data).await.unwrap();

        // Verify event was recorded in metrics engine
        let summary = metrics.get_summary().await;
        assert_eq!(summary.call_count, 1);
        assert!(summary.total_cost_usd > 0.0); // Cost should be calculated
    }

    #[tokio::test]
    async fn test_shutdown_signal() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics);

        // Initially not shutdown
        assert!(!*client.shutdown.lock().await);

        // Signal shutdown
        client.shutdown().await;

        // Verify shutdown flag is set
        assert!(*client.shutdown.lock().await);
    }

    #[tokio::test]
    async fn test_parse_invalid_json() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics);

        let invalid_json = "{ invalid json }";
        let result = client.process_analytics_event(invalid_json).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_empty_activity() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics.clone());

        let event_data = r#"{
            "project": {
                "name": "test-project",
                "cost": 0.0,
                "tokens": 0,
                "calls": 0,
                "last_updated": "2025-11-17T12:00:00Z"
            },
            "machine": {
                "cpu": "Test CPU",
                "memory": "16GB",
                "uptime": 3600,
                "process_count": 10
            },
            "activity": []
        }"#;

        // Process the event
        client.process_analytics_event(event_data).await.unwrap();

        // Verify no events were recorded
        let summary = metrics.get_summary().await;
        assert_eq!(summary.call_count, 0);
    }

    #[tokio::test]
    async fn test_convert_activity_to_api_event() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics);

        // Create an ActivityEvent
        let activity = ActivityEvent {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "api_call".to_string(),
            agent_name: Some("rust-specialist".to_string()),
            model: Some("claude-opus-4".to_string()),
            tokens: Some(1000),
            latency_ms: Some(150),
            status: Some("success".to_string()),
            cost: Some(0.05),
        };

        // Convert to ApiCallEvent
        let api_event = client.convert_to_api_call_event(&activity);

        assert!(api_event.is_some());
        let event = api_event.unwrap();
        assert_eq!(event.model_name, "claude-opus-4");
        assert_eq!(event.agent_name, Some("rust-specialist".to_string()));
        assert_eq!(event.tokens.total_tokens(), 1000);
    }

    #[tokio::test]
    async fn test_skip_non_api_call_events() {
        let metrics = Arc::new(MetricsEngine::new());
        let client = SseClient::new("http://localhost:3000".to_string(), metrics.clone());

        // Create SSE response with cache_hit event (should be skipped)
        let event_data = r#"{
            "project": {
                "name": "test-project",
                "cost": 0.0,
                "tokens": 0,
                "calls": 0,
                "last_updated": "2025-11-17T12:00:00Z"
            },
            "machine": {
                "cpu": "Test CPU",
                "memory": "16GB",
                "uptime": 3600,
                "process_count": 10
            },
            "activity": [
                {
                    "timestamp": "2025-11-17T12:00:00Z",
                    "event_type": "cache_hit",
                    "agent_name": null,
                    "model": "claude-sonnet-4.5",
                    "tokens": 1000,
                    "latency_ms": 10,
                    "status": "success",
                    "cost": 0.0
                }
            ]
        }"#;

        client.process_analytics_event(event_data).await.unwrap();

        // Verify cache_hit event was NOT recorded (only api_call events are recorded)
        let summary = metrics.get_summary().await;
        assert_eq!(summary.call_count, 0);
    }
}
