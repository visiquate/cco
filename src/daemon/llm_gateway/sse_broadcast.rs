//! SSE Broadcast System for TUI Visibility
//!
//! Provides real-time streaming event broadcasting for monitoring LLM requests
//! in the TUI. Events are broadcast via Tokio's broadcast channel to multiple
//! subscribers (TUI, monitoring tools, etc.).

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events broadcast to TUI and other subscribers for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TuiStreamEvent {
    /// Stream started - initial event with request metadata
    #[serde(rename = "started")]
    Started {
        request_id: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        agent_type: Option<String>,
    },

    /// Text delta - incremental text content from the stream
    #[serde(rename = "text_delta")]
    TextDelta { request_id: String, text: String },

    /// Stream completed successfully
    #[serde(rename = "completed")]
    Completed {
        request_id: String,
        input_tokens: u32,
        output_tokens: u32,
        cost_usd: f64,
    },

    /// Stream error
    #[serde(rename = "error")]
    Error { request_id: String, message: String },
}

/// Broadcaster for streaming events to multiple subscribers
///
/// Uses Tokio's broadcast channel which allows multiple receivers to receive
/// the same events. The channel has a fixed capacity - if a receiver is too
/// slow and the buffer fills up, older messages will be dropped.
pub struct StreamEventBroadcaster {
    sender: broadcast::Sender<TuiStreamEvent>,
}

impl StreamEventBroadcaster {
    /// Create a new broadcaster with the specified channel capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events to buffer per subscriber.
    ///   If a subscriber falls behind, older events are dropped.
    ///   Recommended: 100-1000 depending on expected load.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_orchestra::daemon::llm_gateway::sse_broadcast::StreamEventBroadcaster;
    ///
    /// let broadcaster = StreamEventBroadcaster::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to receive streaming events
    ///
    /// Returns a new receiver that will receive all events sent after subscription.
    /// Multiple subscribers can exist simultaneously.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_orchestra::daemon::llm_gateway::sse_broadcast::StreamEventBroadcaster;
    ///
    /// let broadcaster = StreamEventBroadcaster::new(100);
    /// let mut receiver = broadcaster.subscribe();
    ///
    /// tokio::spawn(async move {
    ///     while let Ok(event) = receiver.recv().await {
    ///         println!("Received event: {:?}", event);
    ///     }
    /// });
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<TuiStreamEvent> {
        self.sender.subscribe()
    }

    /// Send an event to all subscribers
    ///
    /// Events are broadcast to all active subscribers. If no subscribers exist,
    /// the event is silently dropped. Slow subscribers may miss events if their
    /// buffer fills up.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to broadcast
    ///
    /// # Example
    ///
    /// ```
    /// use cc_orchestra::daemon::llm_gateway::sse_broadcast::{StreamEventBroadcaster, TuiStreamEvent};
    ///
    /// let broadcaster = StreamEventBroadcaster::new(100);
    ///
    /// broadcaster.send(TuiStreamEvent::Started {
    ///     request_id: "req_123".to_string(),
    ///     model: "claude-sonnet-4-5-20250929".to_string(),
    ///     agent_type: Some("python-expert".to_string()),
    /// });
    /// ```
    pub fn send(&self, event: TuiStreamEvent) {
        // Ignore send errors - if no receivers exist, that's fine
        let _ = self.sender.send(event);
    }

    /// Get the number of active subscribers
    ///
    /// Returns the current number of receivers that are subscribed to this broadcaster.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for StreamEventBroadcaster {
    /// Create a broadcaster with default capacity of 1000 events
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let broadcaster = StreamEventBroadcaster::new(100);
        assert_eq!(broadcaster.receiver_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_default() {
        let broadcaster = StreamEventBroadcaster::default();
        assert_eq!(broadcaster.receiver_count(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_and_receive() {
        let broadcaster = StreamEventBroadcaster::new(100);
        let mut receiver = broadcaster.subscribe();

        assert_eq!(broadcaster.receiver_count(), 1);

        // Send an event
        broadcaster.send(TuiStreamEvent::Started {
            request_id: "req_123".to_string(),
            model: "claude-sonnet-4".to_string(),
            agent_type: Some("test-agent".to_string()),
        });

        // Receive the event
        let event = receiver.recv().await.unwrap();
        match event {
            TuiStreamEvent::Started {
                request_id,
                model,
                agent_type,
            } => {
                assert_eq!(request_id, "req_123");
                assert_eq!(model, "claude-sonnet-4");
                assert_eq!(agent_type, Some("test-agent".to_string()));
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = StreamEventBroadcaster::new(100);
        let mut receiver1 = broadcaster.subscribe();
        let mut receiver2 = broadcaster.subscribe();

        assert_eq!(broadcaster.receiver_count(), 2);

        // Send an event
        broadcaster.send(TuiStreamEvent::TextDelta {
            request_id: "req_456".to_string(),
            text: "Hello".to_string(),
        });

        // Both receivers should get the event
        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();

        match (&event1, &event2) {
            (
                TuiStreamEvent::TextDelta {
                    request_id: id1,
                    text: text1,
                },
                TuiStreamEvent::TextDelta {
                    request_id: id2,
                    text: text2,
                },
            ) => {
                assert_eq!(id1, "req_456");
                assert_eq!(id2, "req_456");
                assert_eq!(text1, "Hello");
                assert_eq!(text2, "Hello");
            }
            _ => panic!("Expected TextDelta events"),
        }
    }

    #[tokio::test]
    async fn test_send_without_subscribers() {
        let broadcaster = StreamEventBroadcaster::new(100);

        // Send events without any subscribers - should not panic
        broadcaster.send(TuiStreamEvent::Started {
            request_id: "req_789".to_string(),
            model: "claude-opus-4".to_string(),
            agent_type: None,
        });

        broadcaster.send(TuiStreamEvent::Completed {
            request_id: "req_789".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost_usd: 0.01,
        });

        assert_eq!(broadcaster.receiver_count(), 0);
    }

    #[test]
    fn test_event_serialization() {
        // Test Started event
        let started = TuiStreamEvent::Started {
            request_id: "req_123".to_string(),
            model: "claude-sonnet-4".to_string(),
            agent_type: Some("test-agent".to_string()),
        };
        let json = serde_json::to_string(&started).unwrap();
        assert!(json.contains("\"type\":\"started\""));
        assert!(json.contains("\"request_id\":\"req_123\""));

        // Test TextDelta event
        let delta = TuiStreamEvent::TextDelta {
            request_id: "req_456".to_string(),
            text: "Hello World".to_string(),
        };
        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("\"type\":\"text_delta\""));
        assert!(json.contains("\"text\":\"Hello World\""));

        // Test Completed event
        let completed = TuiStreamEvent::Completed {
            request_id: "req_789".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost_usd: 0.01,
        };
        let json = serde_json::to_string(&completed).unwrap();
        assert!(json.contains("\"type\":\"completed\""));
        assert!(json.contains("\"cost_usd\":0.01"));

        // Test Error event
        let error = TuiStreamEvent::Error {
            request_id: "req_error".to_string(),
            message: "Connection failed".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"Connection failed\""));
    }

    #[test]
    fn test_event_deserialization() {
        // Test Started event
        let json = r#"{"type":"started","request_id":"req_123","model":"claude-sonnet-4","agent_type":"test-agent"}"#;
        let event: TuiStreamEvent = serde_json::from_str(json).unwrap();
        match event {
            TuiStreamEvent::Started { request_id, .. } => {
                assert_eq!(request_id, "req_123");
            }
            _ => panic!("Expected Started event"),
        }

        // Test TextDelta event
        let json = r#"{"type":"text_delta","request_id":"req_456","text":"Hello"}"#;
        let event: TuiStreamEvent = serde_json::from_str(json).unwrap();
        match event {
            TuiStreamEvent::TextDelta { text, .. } => {
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected TextDelta event"),
        }
    }
}
