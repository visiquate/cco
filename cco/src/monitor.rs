//! CCO Monitoring Service
//!
//! Background daemon service for monitoring Claude API usage via SSE stream.
//! Phase 1a: Core daemon skeleton with signal handling and graceful shutdown.

use crate::metrics::{MetricsEngine, MetricsSummary};
use crate::sse::SseClient;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn, error};

/// Configuration for the monitoring service
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// CCO endpoint to connect to
    pub cco_endpoint: String,

    /// Port for the CCO instance
    pub port: u16,

    /// Maximum number of metrics events to buffer
    pub metrics_buffer_size: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            cco_endpoint: "http://127.0.0.1".to_string(),
            port: 3000,
            metrics_buffer_size: 1000,
        }
    }
}

/// Monitoring service for tracking CCO API usage
pub struct MonitorService {
    /// Configuration
    config: MonitorConfig,

    /// Metrics aggregation engine
    metrics: Arc<MetricsEngine>,

    /// Shutdown flag
    shutdown: Arc<AtomicBool>,

    /// Background task handles
    tasks: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl MonitorService {
    /// Create a new monitoring service
    pub fn new(config: MonitorConfig) -> Self {
        let metrics = Arc::new(MetricsEngine::with_capacity(config.metrics_buffer_size));

        Self {
            config,
            metrics,
            shutdown: Arc::new(AtomicBool::new(false)),
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the monitoring service
    ///
    /// This spawns background tasks for:
    /// - Signal handling (SIGINT/SIGTERM)
    /// - SSE client for streaming events from CCO proxy
    /// - Metrics aggregation
    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting CCO Monitor Service (endpoint: {}:{})",
            self.config.cco_endpoint, self.config.port
        );

        // Setup signal handlers
        self.setup_signal_handlers().await?;

        // Spawn SSE client task
        self.spawn_sse_task().await;

        // Spawn metrics aggregation task (heartbeat)
        self.spawn_metrics_task().await;

        info!("Monitor service started successfully");

        Ok(())
    }

    /// Setup signal handlers for graceful shutdown
    #[cfg(unix)]
    async fn setup_signal_handlers(&self) -> Result<()> {
        use tokio::signal::unix::{signal, SignalKind};

        let shutdown = self.shutdown.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut sigint = signal(SignalKind::interrupt())
                .expect("Failed to setup SIGINT handler");
            let mut sigterm = signal(SignalKind::terminate())
                .expect("Failed to setup SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating graceful shutdown");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating graceful shutdown");
                }
            }

            // Set shutdown flag
            shutdown.store(true, Ordering::SeqCst);

            // Log final metrics summary
            let summary = metrics.get_summary().await;
            info!(
                "Final metrics: {} calls, ${:.2} total cost",
                summary.call_count, summary.total_cost_usd
            );
        });

        debug!("Signal handlers configured (SIGINT, SIGTERM)");

        Ok(())
    }

    /// Setup signal handlers for Windows
    #[cfg(not(unix))]
    async fn setup_signal_handlers(&self) -> Result<()> {
        let shutdown = self.shutdown.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to setup Ctrl+C handler");

            info!("Received Ctrl+C, initiating graceful shutdown");

            // Set shutdown flag
            shutdown.store(true, Ordering::SeqCst);

            // Log final metrics summary
            let summary = metrics.get_summary().await;
            info!(
                "Final metrics: {} calls, ${:.2} total cost",
                summary.call_count, summary.total_cost_usd
            );
        });

        debug!("Signal handlers configured (Ctrl+C)");

        Ok(())
    }

    /// Spawn SSE client task for streaming events from CCO proxy
    async fn spawn_sse_task(&self) {
        let endpoint = format!("{}:{}", self.config.cco_endpoint, self.config.port);
        let metrics = self.metrics.clone();
        let _shutdown = self.shutdown.clone();

        let handle = tokio::spawn(async move {
            // Create SSE client
            let client = SseClient::new(endpoint.clone(), metrics);

            info!("SSE client connecting to {}", endpoint);

            // Run SSE client (will auto-reconnect on errors)
            match client.connect().await {
                Ok(_) => {
                    info!("SSE client shut down gracefully");
                }
                Err(e) => {
                    error!("SSE client encountered fatal error: {}", e);
                }
            }
        });

        let mut tasks = self.tasks.write().await;
        tasks.push(handle);

        debug!("SSE client task spawned");
    }

    /// Spawn background metrics aggregation task
    async fn spawn_metrics_task(&self) {
        let shutdown = self.shutdown.clone();
        let metrics = self.metrics.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Check shutdown flag
                if shutdown.load(Ordering::SeqCst) {
                    debug!("Metrics task shutting down");
                    break;
                }

                // Log current metrics summary
                let summary = metrics.get_summary().await;
                debug!(
                    "Metrics heartbeat: {} calls, ${:.2} total cost, {} events buffered",
                    summary.call_count,
                    summary.total_cost_usd,
                    metrics.get_buffer_size().await
                );
            }
        });

        let mut tasks = self.tasks.write().await;
        tasks.push(handle);

        debug!("Metrics aggregation task spawned");
    }

    /// Shutdown the monitoring service gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down monitor service");

        // Signal shutdown to all tasks
        self.shutdown.store(true, Ordering::SeqCst);

        // Wait for all tasks to complete
        let mut tasks = self.tasks.write().await;
        let task_count = tasks.len();

        for (i, handle) in tasks.drain(..).enumerate() {
            debug!("Waiting for task {}/{} to complete", i + 1, task_count);
            if let Err(e) = handle.await {
                warn!("Task {} failed during shutdown: {}", i + 1, e);
            }
        }

        // Get final summary
        let summary = self.get_summary().await;
        info!(
            "Monitor service shutdown complete. Final stats: {} calls, ${:.2} total cost",
            summary.call_count, summary.total_cost_usd
        );

        Ok(())
    }

    /// Get current metrics summary
    pub async fn get_summary(&self) -> MetricsSummary {
        self.metrics.get_summary().await
    }

    /// Get reference to metrics engine (for testing/external access)
    pub fn metrics(&self) -> &Arc<MetricsEngine> {
        &self.metrics
    }

    /// Check if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_monitor_service_creation() {
        let config = MonitorConfig::default();
        let service = MonitorService::new(config);

        assert!(!service.is_shutdown_requested());
        assert_eq!(service.metrics.get_buffer_capacity(), 1000);
    }

    #[tokio::test]
    async fn test_monitor_service_start_and_shutdown() {
        let config = MonitorConfig {
            cco_endpoint: "http://127.0.0.1".to_string(),
            port: 3000,
            metrics_buffer_size: 100,
        };

        let service = MonitorService::new(config);

        // Start the service
        service.start().await.expect("Failed to start service");

        // Verify it's running
        assert!(!service.is_shutdown_requested());

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Shutdown
        service.shutdown().await.expect("Failed to shutdown");

        // Verify shutdown flag
        assert!(service.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_custom_buffer_size() {
        let config = MonitorConfig {
            cco_endpoint: "http://127.0.0.1".to_string(),
            port: 3000,
            metrics_buffer_size: 500,
        };

        let service = MonitorService::new(config);

        assert_eq!(service.metrics.get_buffer_capacity(), 500);
    }

    #[tokio::test]
    async fn test_get_summary() {
        let config = MonitorConfig::default();
        let service = MonitorService::new(config);

        let summary = service.get_summary().await;

        assert_eq!(summary.call_count, 0);
        assert_eq!(summary.total_cost_usd, 0.0);
    }

    #[tokio::test]
    async fn test_metrics_access() {
        use crate::metrics::{ApiCallEvent, TokenBreakdown};

        let config = MonitorConfig::default();
        let service = MonitorService::new(config);

        // Record an event through the metrics engine
        let tokens = TokenBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        let event = ApiCallEvent::new(
            "claude-opus-4".to_string(),
            tokens,
            None,
            None,
        )
        .unwrap();

        service.metrics().record_event(event).await;

        // Verify we can retrieve it
        let summary = service.get_summary().await;
        assert_eq!(summary.call_count, 1);
        assert!(summary.total_cost_usd > 0.0);
    }
}
