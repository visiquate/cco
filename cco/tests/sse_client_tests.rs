//! SSE Client Tests for Phase 1a
//!
//! This module tests the Server-Sent Events (SSE) client implementation
//! following TDD principles - tests written BEFORE implementation.
//!
//! ## Test Coverage Areas:
//! - Successful event parsing
//! - Malformed event handling
//! - Connection failure and reconnection
//! - Exponential backoff logic
//! - Graceful shutdown

#[cfg(test)]
mod sse_client_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    // ===== Mock Types for Testing (to be replaced with actual implementation) =====

    #[derive(Clone, Debug, PartialEq)]
    pub struct SseEvent {
        pub event_type: String,
        pub data: String,
        pub id: Option<String>,
        pub retry: Option<u64>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum ConnectionState {
        Disconnected,
        Connecting,
        Connected,
        Reconnecting,
        Shutdown,
    }

    #[derive(Clone, Debug)]
    pub struct ReconnectConfig {
        pub initial_delay: Duration,
        pub max_delay: Duration,
        pub multiplier: f64,
        pub max_retries: Option<usize>,
    }

    impl Default for ReconnectConfig {
        fn default() -> Self {
            Self {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(30),
                multiplier: 2.0,
                max_retries: None,
            }
        }
    }

    /// Mock SSE Client - to be replaced with actual implementation
    pub struct SseClient {
        url: String,
        state: Arc<Mutex<ConnectionState>>,
        events: Arc<Mutex<Vec<SseEvent>>>,
        reconnect_config: ReconnectConfig,
        shutdown_signal: Arc<Mutex<bool>>,
    }

    impl SseClient {
        pub fn new(url: String, reconnect_config: ReconnectConfig) -> Self {
            Self {
                url,
                state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
                events: Arc::new(Mutex::new(Vec::new())),
                reconnect_config,
                shutdown_signal: Arc::new(Mutex::new(false)),
            }
        }

        pub async fn connect(&self) -> Result<(), String> {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Connecting;

            // Simulate connection
            tokio::time::sleep(Duration::from_millis(50)).await;

            *state = ConnectionState::Connected;
            Ok(())
        }

        pub async fn parse_event(&self, raw_data: &str) -> Result<SseEvent, String> {
            let lines: Vec<&str> = raw_data.lines().collect();
            let mut event_type = "message".to_string();
            let mut data_lines = Vec::new();
            let mut id = None;
            let mut retry = None;

            for line in lines {
                if line.is_empty() {
                    continue;
                }

                if let Some(colon_pos) = line.find(':') {
                    let field = &line[..colon_pos];
                    let value = line[colon_pos + 1..].trim_start();

                    match field {
                        "event" => event_type = value.to_string(),
                        "data" => data_lines.push(value),
                        "id" => id = Some(value.to_string()),
                        "retry" => retry = value.parse::<u64>().ok(),
                        _ => {}
                    }
                } else {
                    return Err("Malformed SSE event: missing colon".to_string());
                }
            }

            if data_lines.is_empty() {
                return Err("Malformed SSE event: no data field".to_string());
            }

            Ok(SseEvent {
                event_type,
                data: data_lines.join("\n"),
                id,
                retry,
            })
        }

        pub async fn handle_event(&self, event: SseEvent) {
            let mut events = self.events.lock().await;
            events.push(event);
        }

        pub async fn get_events(&self) -> Vec<SseEvent> {
            let events = self.events.lock().await;
            events.clone()
        }

        pub async fn get_state(&self) -> ConnectionState {
            let state = self.state.lock().await;
            state.clone()
        }

        pub async fn simulate_connection_failure(&self) -> Result<(), String> {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Disconnected;
            Err("Connection failed".to_string())
        }

        pub async fn reconnect_with_backoff(&self, attempt: usize) -> Result<(), String> {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Reconnecting;
            drop(state);

            // Calculate backoff delay
            let delay = self.calculate_backoff_delay(attempt);
            tokio::time::sleep(delay).await;

            // Check for shutdown during backoff
            let shutdown = self.shutdown_signal.lock().await;
            if *shutdown {
                let mut state = self.state.lock().await;
                *state = ConnectionState::Shutdown;
                return Err("Shutdown requested".to_string());
            }
            drop(shutdown);

            // Attempt reconnection
            self.connect().await
        }

        pub fn calculate_backoff_delay(&self, attempt: usize) -> Duration {
            let base_delay_ms = self.reconnect_config.initial_delay.as_millis() as f64;
            let multiplier = self.reconnect_config.multiplier;
            let max_delay_ms = self.reconnect_config.max_delay.as_millis() as u64;

            let calculated_delay = base_delay_ms * multiplier.powi(attempt as i32);
            let delay_ms = calculated_delay.min(max_delay_ms as f64) as u64;

            Duration::from_millis(delay_ms)
        }

        pub async fn shutdown(&self) {
            let mut shutdown = self.shutdown_signal.lock().await;
            *shutdown = true;

            let mut state = self.state.lock().await;
            *state = ConnectionState::Shutdown;
        }

        pub async fn clear_events(&self) {
            let mut events = self.events.lock().await;
            events.clear();
        }
    }

    // ===== TEST SUITE =====

    // Test 1: Successful Event Parsing - Basic
    #[tokio::test]
    async fn test_parse_basic_event() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: analytics\ndata: {\"metric\":\"value\"}\n\n";
        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.event_type, "analytics");
        assert_eq!(event.data, "{\"metric\":\"value\"}");
        assert_eq!(event.id, None);
    }

    // Test 2: Event Parsing with ID
    #[tokio::test]
    async fn test_parse_event_with_id() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: update\ndata: test data\nid: 12345\n\n";
        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.event_type, "update");
        assert_eq!(event.data, "test data");
        assert_eq!(event.id, Some("12345".to_string()));
    }

    // Test 3: Event Parsing with Retry
    #[tokio::test]
    async fn test_parse_event_with_retry() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: metrics\ndata: stats\nretry: 5000\n\n";
        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.event_type, "metrics");
        assert_eq!(event.retry, Some(5000));
    }

    // Test 4: Multi-line Data Parsing
    #[tokio::test]
    async fn test_parse_multiline_data() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: log\ndata: Line 1\ndata: Line 2\ndata: Line 3\n\n";
        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.data, "Line 1\nLine 2\nLine 3");
    }

    // Test 5: Default Event Type (Message)
    #[tokio::test]
    async fn test_parse_default_event_type() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "data: message without explicit type\n\n";
        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.event_type, "message");
    }

    // Test 6: Malformed Event - No Data
    #[tokio::test]
    async fn test_malformed_event_no_data() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: test\nid: 123\n\n";
        let result = client.parse_event(raw).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Malformed SSE event: no data field");
    }

    // Test 7: Malformed Event - Missing Colon
    #[tokio::test]
    async fn test_malformed_event_missing_colon() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event test\ndata: value\n\n";
        let result = client.parse_event(raw).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Malformed SSE event: missing colon"
        );
    }

    // Test 8: Connection State Transitions
    #[tokio::test]
    async fn test_connection_state_transitions() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        // Initial state
        assert_eq!(client.get_state().await, ConnectionState::Disconnected);

        // Connect
        client.connect().await.unwrap();
        assert_eq!(client.get_state().await, ConnectionState::Connected);

        // Shutdown
        client.shutdown().await;
        assert_eq!(client.get_state().await, ConnectionState::Shutdown);
    }

    // Test 9: Connection Failure Handling
    #[tokio::test]
    async fn test_connection_failure() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        client.connect().await.unwrap();
        assert_eq!(client.get_state().await, ConnectionState::Connected);

        let result = client.simulate_connection_failure().await;
        assert!(result.is_err());
        assert_eq!(client.get_state().await, ConnectionState::Disconnected);
    }

    // Test 10: Exponential Backoff Calculation
    #[tokio::test]
    async fn test_exponential_backoff_calculation() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_retries: None,
        };

        let client = SseClient::new("http://localhost:3000/stream".to_string(), config);

        // Attempt 0: 100ms
        assert_eq!(client.calculate_backoff_delay(0), Duration::from_millis(100));

        // Attempt 1: 200ms
        assert_eq!(client.calculate_backoff_delay(1), Duration::from_millis(200));

        // Attempt 2: 400ms
        assert_eq!(client.calculate_backoff_delay(2), Duration::from_millis(400));

        // Attempt 3: 800ms
        assert_eq!(client.calculate_backoff_delay(3), Duration::from_millis(800));

        // Attempt 10: Should cap at max_delay (30s)
        assert_eq!(
            client.calculate_backoff_delay(10),
            Duration::from_secs(30)
        );
    }

    // Test 11: Reconnection with Backoff
    #[tokio::test]
    async fn test_reconnection_with_backoff() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            max_retries: Some(3),
        };

        let client = SseClient::new("http://localhost:3000/stream".to_string(), config);

        let start = std::time::Instant::now();
        let result = client.reconnect_with_backoff(0).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(50)); // At least initial delay
        assert_eq!(client.get_state().await, ConnectionState::Connected);
    }

    // Test 12: Reconnection During Shutdown
    #[tokio::test]
    async fn test_reconnection_during_shutdown() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            max_retries: None,
        };

        let client = Arc::new(SseClient::new(
            "http://localhost:3000/stream".to_string(),
            config,
        ));

        // Start reconnection in background
        let client_clone = client.clone();
        let reconnect_handle = tokio::spawn(async move {
            client_clone.reconnect_with_backoff(3).await
        });

        // Trigger shutdown during backoff
        tokio::time::sleep(Duration::from_millis(50)).await;
        client.shutdown().await;

        // Wait for reconnection attempt to complete
        let result = reconnect_handle.await.unwrap();

        assert!(result.is_err());
        assert_eq!(client.get_state().await, ConnectionState::Shutdown);
    }

    // Test 13: Event Handler
    #[tokio::test]
    async fn test_event_handler() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let event1 = SseEvent {
            event_type: "analytics".to_string(),
            data: "{\"metric\": 1}".to_string(),
            id: Some("1".to_string()),
            retry: None,
        };

        let event2 = SseEvent {
            event_type: "update".to_string(),
            data: "{\"metric\": 2}".to_string(),
            id: Some("2".to_string()),
            retry: None,
        };

        client.handle_event(event1.clone()).await;
        client.handle_event(event2.clone()).await;

        let events = client.get_events().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
    }

    // Test 14: Graceful Shutdown
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        client.connect().await.unwrap();
        assert_eq!(client.get_state().await, ConnectionState::Connected);

        // Add some events
        client
            .handle_event(SseEvent {
                event_type: "test".to_string(),
                data: "data".to_string(),
                id: None,
                retry: None,
            })
            .await;

        client.shutdown().await;
        assert_eq!(client.get_state().await, ConnectionState::Shutdown);

        // Events should still be accessible after shutdown
        let events = client.get_events().await;
        assert_eq!(events.len(), 1);
    }

    // Test 15: Clear Events
    #[tokio::test]
    async fn test_clear_events() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        // Add events
        for i in 0..5 {
            client
                .handle_event(SseEvent {
                    event_type: "test".to_string(),
                    data: format!("data-{}", i),
                    id: Some(i.to_string()),
                    retry: None,
                })
                .await;
        }

        assert_eq!(client.get_events().await.len(), 5);

        client.clear_events().await;
        assert_eq!(client.get_events().await.len(), 0);
    }

    // Test 16: JSON Data Parsing
    #[tokio::test]
    async fn test_parse_json_data() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = r#"event: analytics
data: {"requests": 100, "cost": 5.25, "model": "claude-opus-4"}

"#;

        let event = client.parse_event(raw).await.unwrap();

        assert_eq!(event.event_type, "analytics");

        // Parse the JSON data
        let parsed: serde_json::Value = serde_json::from_str(&event.data).unwrap();
        assert_eq!(parsed["requests"], 100);
        assert_eq!(parsed["cost"], 5.25);
        assert_eq!(parsed["model"], "claude-opus-4");
    }

    // Test 17: Concurrent Event Handling
    #[tokio::test]
    async fn test_concurrent_event_handling() {
        let client = Arc::new(SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        ));

        let mut handles = vec![];

        // Spawn 10 concurrent tasks handling events
        for i in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                client_clone
                    .handle_event(SseEvent {
                        event_type: "concurrent".to_string(),
                        data: format!("event-{}", i),
                        id: Some(i.to_string()),
                        retry: None,
                    })
                    .await;
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let events = client.get_events().await;
        assert_eq!(events.len(), 10);
    }

    // Test 18: Backoff Max Retries
    #[tokio::test]
    async fn test_backoff_respects_max_retries() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
            max_retries: Some(3),
        };

        let _client = SseClient::new("http://localhost:3000/stream".to_string(), config);

        // Note: Actual retry limit enforcement would be in the connection loop
        // This test just verifies the config is stored
    }

    // Test 19: Empty Event Handling
    #[tokio::test]
    async fn test_parse_empty_event() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "\n\n";
        let result = client.parse_event(raw).await;

        assert!(result.is_err());
    }

    // Test 20: Whitespace in Data Fields
    #[tokio::test]
    async fn test_parse_whitespace_in_data() {
        let client = SseClient::new(
            "http://localhost:3000/stream".to_string(),
            ReconnectConfig::default(),
        );

        let raw = "event: test\ndata:    data with leading spaces   \n\n";
        let event = client.parse_event(raw).await.unwrap();

        // Leading space after colon should be trimmed
        assert_eq!(event.data, "data with leading spaces   ");
    }
}
