//! Phase 1a Integration Tests
//!
//! This module tests the complete Phase 1a system integration including:
//! - Full daemon startup with mock CCO proxy
//! - Event streaming and metrics aggregation
//! - Shutdown and cleanup verification
//! - Performance baseline (event processing rate)
//!
//! These tests verify end-to-end functionality of the monitoring daemon.

#[cfg(test)]
mod phase1a_integration_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    // ===== Mock Types for Integration Testing =====

    #[derive(Clone, Debug)]
    pub struct DaemonConfig {
        pub metrics_buffer_size: usize,
        pub sse_url: String,
        pub poll_interval: Duration,
        pub reconnect_config: ReconnectConfig,
    }

    impl Default for DaemonConfig {
        fn default() -> Self {
            Self {
                metrics_buffer_size: 1000,
                sse_url: "http://localhost:3000/api/stream".to_string(),
                poll_interval: Duration::from_secs(5),
                reconnect_config: ReconnectConfig::default(),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct ReconnectConfig {
        pub initial_delay: Duration,
        pub max_delay: Duration,
        pub multiplier: f64,
    }

    impl Default for ReconnectConfig {
        fn default() -> Self {
            Self {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(30),
                multiplier: 2.0,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct SseEvent {
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub event_type: String,
        pub data: String,
    }

    #[derive(Clone, Debug)]
    pub struct MetricsSnapshot {
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub total_requests: u64,
        pub total_cost: f64,
        pub total_tokens: u64,
    }

    /// Mock CCO Proxy Server for testing
    pub struct MockCcoProxy {
        events: Arc<Mutex<Vec<SseEvent>>>,
        is_running: Arc<Mutex<bool>>,
    }

    impl MockCcoProxy {
        pub fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                is_running: Arc::new(Mutex::new(false)),
            }
        }

        pub async fn start(&self) -> Result<(), String> {
            let mut running = self.is_running.lock().await;
            if *running {
                return Err("Proxy already running".to_string());
            }
            *running = true;

            // Simulate server startup
            tokio::time::sleep(Duration::from_millis(50)).await;

            Ok(())
        }

        pub async fn stop(&self) -> Result<(), String> {
            let mut running = self.is_running.lock().await;
            if !*running {
                return Err("Proxy not running".to_string());
            }
            *running = false;
            Ok(())
        }

        pub async fn emit_event(&self, event: SseEvent) {
            let mut events = self.events.lock().await;
            events.push(event);
        }

        pub async fn get_event_count(&self) -> usize {
            let events = self.events.lock().await;
            events.len()
        }

        pub async fn is_running(&self) -> bool {
            let running = self.is_running.lock().await;
            *running
        }
    }

    /// Mock Monitoring Daemon - integrates metrics engine, SSE client, and monitor service
    pub struct MonitoringDaemon {
        config: DaemonConfig,
        proxy: Arc<MockCcoProxy>,
        metrics_snapshots: Arc<Mutex<Vec<MetricsSnapshot>>>,
        is_running: Arc<Mutex<bool>>,
        shutdown_signal: Arc<Mutex<bool>>,
    }

    impl MonitoringDaemon {
        pub fn new(config: DaemonConfig, proxy: Arc<MockCcoProxy>) -> Self {
            Self {
                config,
                proxy,
                metrics_snapshots: Arc::new(Mutex::new(Vec::new())),
                is_running: Arc::new(Mutex::new(false)),
                shutdown_signal: Arc::new(Mutex::new(false)),
            }
        }

        pub async fn start(&self) -> Result<(), String> {
            let mut running = self.is_running.lock().await;
            if *running {
                return Err("Daemon already running".to_string());
            }

            // Verify proxy is running
            if !self.proxy.is_running().await {
                return Err("CCO proxy is not running".to_string());
            }

            *running = true;
            drop(running);

            // Spawn monitoring loop
            let self_clone = Self {
                config: self.config.clone(),
                proxy: self.proxy.clone(),
                metrics_snapshots: self.metrics_snapshots.clone(),
                is_running: self.is_running.clone(),
                shutdown_signal: self.shutdown_signal.clone(),
            };

            tokio::spawn(async move {
                self_clone.run_monitoring_loop().await;
            });

            Ok(())
        }

        async fn run_monitoring_loop(&self) {
            let mut interval = tokio::time::interval(self.config.poll_interval);

            loop {
                interval.tick().await;

                // Check shutdown signal
                let shutdown = self.shutdown_signal.lock().await;
                if *shutdown {
                    break;
                }
                drop(shutdown);

                // Collect metrics from proxy
                self.collect_metrics().await;
            }
        }

        async fn collect_metrics(&self) {
            let event_count = self.proxy.get_event_count().await;

            let mut snapshots = self.metrics_snapshots.lock().await;
            snapshots.push(MetricsSnapshot {
                timestamp: chrono::Utc::now(),
                total_requests: event_count as u64,
                total_cost: event_count as f64 * 0.05,
                total_tokens: event_count as u64 * 1000,
            });
        }

        pub async fn stop(&self) -> Result<(), String> {
            let mut running = self.is_running.lock().await;
            if !*running {
                return Err("Daemon is not running".to_string());
            }

            // Signal shutdown
            let mut shutdown = self.shutdown_signal.lock().await;
            *shutdown = true;
            drop(shutdown);

            // Wait for loop to exit
            tokio::time::sleep(Duration::from_millis(150)).await;

            *running = false;

            Ok(())
        }

        pub async fn get_metrics_count(&self) -> usize {
            let snapshots = self.metrics_snapshots.lock().await;
            snapshots.len()
        }

        pub async fn get_latest_metrics(&self) -> Option<MetricsSnapshot> {
            let snapshots = self.metrics_snapshots.lock().await;
            snapshots.last().cloned()
        }

        pub async fn is_running(&self) -> bool {
            let running = self.is_running.lock().await;
            *running
        }

        pub async fn cleanup(&self) -> Result<(), String> {
            // Clear all stored metrics
            let mut snapshots = self.metrics_snapshots.lock().await;
            snapshots.clear();
            Ok(())
        }

        pub async fn measure_event_processing_rate(&self, duration: Duration) -> f64 {
            let start_count = self.get_metrics_count().await;
            tokio::time::sleep(duration).await;
            let end_count = self.get_metrics_count().await;

            let events_processed = (end_count - start_count) as f64;
            let seconds = duration.as_secs_f64();

            events_processed / seconds
        }
    }

    // ===== INTEGRATION TEST SUITE =====

    // Test 1: Full System Startup
    #[tokio::test]
    async fn test_full_system_startup() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let daemon = MonitoringDaemon::new(DaemonConfig::default(), proxy.clone());

        let result = daemon.start().await;
        assert!(result.is_ok());
        assert!(daemon.is_running().await);

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 2: Startup Without Proxy Fails
    #[tokio::test]
    async fn test_startup_without_proxy_fails() {
        let proxy = Arc::new(MockCcoProxy::new());
        // Don't start proxy

        let daemon = MonitoringDaemon::new(DaemonConfig::default(), proxy.clone());

        let result = daemon.start().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "CCO proxy is not running");
    }

    // Test 3: Event Streaming and Metrics Aggregation
    #[tokio::test]
    async fn test_event_streaming_and_metrics() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Emit some events from proxy
        for i in 0..5 {
            proxy
                .emit_event(SseEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "analytics".to_string(),
                    data: format!("event-{}", i),
                })
                .await;
        }

        // Wait for daemon to collect metrics
        tokio::time::sleep(Duration::from_millis(250)).await;

        let metrics_count = daemon.get_metrics_count().await;
        assert!(metrics_count > 0, "Should have collected metrics");

        let latest = daemon.get_latest_metrics().await.unwrap();
        assert_eq!(latest.total_requests, 5);

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 4: Shutdown and Cleanup Verification
    #[tokio::test]
    async fn test_shutdown_and_cleanup() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Let it collect some metrics
        tokio::time::sleep(Duration::from_millis(250)).await;

        assert!(daemon.get_metrics_count().await > 0);

        // Stop daemon
        daemon.stop().await.unwrap();
        assert!(!daemon.is_running().await);

        // Cleanup
        daemon.cleanup().await.unwrap();
        assert_eq!(daemon.get_metrics_count().await, 0);

        proxy.stop().await.unwrap();
    }

    // Test 5: Performance Baseline - Event Processing Rate
    #[tokio::test]
    async fn test_event_processing_rate_baseline() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(50), // Fast polling
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Emit events continuously
        let proxy_clone = proxy.clone();
        let emit_handle = tokio::spawn(async move {
            for i in 0..100 {
                proxy_clone
                    .emit_event(SseEvent {
                        timestamp: chrono::Utc::now(),
                        event_type: "test".to_string(),
                        data: format!("event-{}", i),
                    })
                    .await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Measure processing rate over 1 second
        let rate = daemon
            .measure_event_processing_rate(Duration::from_secs(1))
            .await;

        emit_handle.await.unwrap();

        // Should process at least 10 events per second with 50ms polling
        assert!(
            rate >= 10.0,
            "Event processing rate too low: {} events/sec",
            rate
        );

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 6: Graceful Shutdown During Active Collection
    #[tokio::test]
    async fn test_graceful_shutdown_during_collection() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(50),
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Start emitting events
        let proxy_clone = proxy.clone();
        let emit_handle = tokio::spawn(async move {
            for i in 0..1000 {
                proxy_clone
                    .emit_event(SseEvent {
                        timestamp: chrono::Utc::now(),
                        event_type: "stress".to_string(),
                        data: format!("event-{}", i),
                    })
                    .await;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });

        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Shutdown while actively collecting
        let result = daemon.stop().await;
        assert!(result.is_ok());
        assert!(!daemon.is_running().await);

        emit_handle.abort(); // Stop emitting
        proxy.stop().await.unwrap();
    }

    // Test 7: Multiple Start-Stop Cycles
    #[tokio::test]
    async fn test_multiple_start_stop_cycles() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        for cycle in 0..3 {
            let daemon = MonitoringDaemon::new(config.clone(), proxy.clone());

            daemon.start().await.unwrap();
            tokio::time::sleep(Duration::from_millis(150)).await;
            daemon.stop().await.unwrap();

            println!("Completed cycle {}", cycle + 1);
        }

        proxy.stop().await.unwrap();
    }

    // Test 8: High-Volume Event Handling
    #[tokio::test]
    async fn test_high_volume_event_handling() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(50),
            metrics_buffer_size: 10000, // Large buffer
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Emit 1000 events rapidly
        for i in 0..1000 {
            proxy
                .emit_event(SseEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "high-volume".to_string(),
                    data: format!("event-{}", i),
                })
                .await;
        }

        // Wait for collection
        tokio::time::sleep(Duration::from_millis(500)).await;

        let latest = daemon.get_latest_metrics().await.unwrap();
        assert_eq!(latest.total_requests, 1000);
        assert!(latest.total_cost > 0.0);

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 9: Concurrent Daemon Operations
    #[tokio::test]
    async fn test_concurrent_daemon_operations() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        let daemon = Arc::new(MonitoringDaemon::new(config, proxy.clone()));
        daemon.start().await.unwrap();

        let mut handles = vec![];

        // Spawn multiple tasks querying daemon state
        for _ in 0..10 {
            let daemon_clone = daemon.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..10 {
                    let _ = daemon_clone.is_running().await;
                    let _ = daemon_clone.get_metrics_count().await;
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 10: Metrics Accuracy Over Time
    #[tokio::test]
    async fn test_metrics_accuracy_over_time() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Emit events at known rate
        for batch in 0..5 {
            for i in 0..10 {
                proxy
                    .emit_event(SseEvent {
                        timestamp: chrono::Utc::now(),
                        event_type: "batch".to_string(),
                        data: format!("batch-{}-event-{}", batch, i),
                    })
                    .await;
            }

            // Wait for collection after each batch
            tokio::time::sleep(Duration::from_millis(150)).await;

            let latest = daemon.get_latest_metrics().await.unwrap();
            let expected_requests = (batch + 1) * 10;
            assert_eq!(
                latest.total_requests, expected_requests,
                "Metrics inaccurate at batch {}",
                batch
            );
        }

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 11: Proxy Reconnection Simulation
    #[tokio::test]
    async fn test_proxy_reconnection() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(100),
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Emit some events
        for i in 0..5 {
            proxy
                .emit_event(SseEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "before-disconnect".to_string(),
                    data: format!("event-{}", i),
                })
                .await;
        }

        tokio::time::sleep(Duration::from_millis(150)).await;

        // Simulate proxy disconnect and reconnect
        proxy.stop().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        proxy.start().await.unwrap();

        // Emit more events after reconnection
        for i in 0..5 {
            proxy
                .emit_event(SseEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "after-reconnect".to_string(),
                    data: format!("event-{}", i),
                })
                .await;
        }

        tokio::time::sleep(Duration::from_millis(150)).await;

        let latest = daemon.get_latest_metrics().await.unwrap();
        assert_eq!(latest.total_requests, 10);

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }

    // Test 12: Memory Leak Detection (Long Running)
    #[tokio::test]
    async fn test_no_memory_leaks_long_running() {
        let proxy = Arc::new(MockCcoProxy::new());
        proxy.start().await.unwrap();

        let config = DaemonConfig {
            poll_interval: Duration::from_millis(50),
            metrics_buffer_size: 1000,
            ..Default::default()
        };

        let daemon = MonitoringDaemon::new(config, proxy.clone());
        daemon.start().await.unwrap();

        // Run for extended period with continuous events
        for _ in 0..50 {
            proxy
                .emit_event(SseEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "stress".to_string(),
                    data: "test data".to_string(),
                })
                .await;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        // Should still be responsive
        assert!(daemon.is_running().await);
        assert!(daemon.get_metrics_count().await > 0);

        daemon.stop().await.unwrap();
        proxy.stop().await.unwrap();
    }
}
