//! Event Bus Component
//!
//! Topic-based pub-sub messaging system for agent coordination.
//! Features circular buffer, 24-hour retention, timeout handling,
//! and dead letter queue for failed events.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

const EVENT_BUFFER_CAPACITY: usize = 10_000;
const EVENT_RETENTION_HOURS: i64 = 24;
#[allow(dead_code)]
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Event stored in the bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub publisher: String,
    pub topic: String,
    pub data: serde_json::Value,
    pub correlation_id: Option<String>,
    pub project_id: String,
    pub timestamp: DateTime<Utc>,
    pub ttl_seconds: u32,
}

/// Event bus for pub-sub messaging
#[derive(Clone)]
pub struct EventBus {
    /// Circular buffer of events
    events: Arc<RwLock<VecDeque<Event>>>,

    /// Topic-based subscribers (topic -> broadcast channel)
    subscribers: Arc<DashMap<String, broadcast::Sender<Event>>>,

    /// Dead letter queue for failed events
    dead_letter_queue: Arc<RwLock<VecDeque<Event>>>,

    /// Event statistics
    stats: Arc<RwLock<EventStats>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStats {
    pub total_published: usize,
    pub total_delivered: usize,
    pub total_failed: usize,
    pub active_subscriptions: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(EVENT_BUFFER_CAPACITY))),
            subscribers: Arc::new(DashMap::new()),
            dead_letter_queue: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Publish an event to a topic
    pub async fn publish(
        &self,
        event_type: &str,
        publisher: &str,
        topic: &str,
        data: serde_json::Value,
    ) -> Result<String> {
        let event = Event {
            id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            publisher: publisher.to_string(),
            topic: topic.to_string(),
            data,
            correlation_id: None,
            project_id: "default".to_string(), // TODO: get from context
            timestamp: Utc::now(),
            ttl_seconds: 86400, // 24 hours
        };

        let event_id = event.id.clone();

        // Add to circular buffer
        let mut events = self.events.write().await;
        if events.len() >= EVENT_BUFFER_CAPACITY {
            events.pop_front();
        }
        events.push_back(event.clone());
        drop(events);

        // Publish to topic subscribers
        if let Some(sender) = self.subscribers.get(topic) {
            match sender.send(event.clone()) {
                Ok(count) => {
                    let mut stats = self.stats.write().await;
                    stats.total_published += 1;
                    stats.total_delivered += count;
                }
                Err(_) => {
                    // No active subscribers, add to dead letter queue
                    let mut dlq = self.dead_letter_queue.write().await;
                    dlq.push_back(event);

                    let mut stats = self.stats.write().await;
                    stats.total_failed += 1;
                }
            }
        } else {
            // No subscribers for this topic, add to dead letter queue
            let mut dlq = self.dead_letter_queue.write().await;
            dlq.push_back(event);
        }

        // Cleanup old events
        tokio::spawn({
            let events = Arc::clone(&self.events);
            async move {
                let mut events = events.write().await;
                let cutoff = Utc::now() - Duration::hours(EVENT_RETENTION_HOURS);

                while let Some(event) = events.front() {
                    if event.timestamp < cutoff {
                        events.pop_front();
                    } else {
                        break;
                    }
                }
            }
        });

        Ok(event_id)
    }

    /// Subscribe to a topic
    pub async fn subscribe(&self, topic: &str) -> broadcast::Receiver<Event> {
        let sender = self
            .subscribers
            .entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone();

        let mut stats = self.stats.write().await;
        stats.active_subscriptions += 1;

        sender.subscribe()
    }

    /// Wait for an event of a specific type (long-polling)
    pub async fn wait_for_event(
        &self,
        event_type: &str,
        timeout_ms: u64,
    ) -> Result<Vec<super::EventData>> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let start = std::time::Instant::now();

        // Check existing events first
        let events = self.events.read().await;
        let matching: Vec<super::EventData> = events
            .iter()
            .filter(|e| e.event_type == event_type)
            .map(|e| super::EventData {
                event_id: e.id.clone(),
                event_type: e.event_type.clone(),
                publisher: e.publisher.clone(),
                data: e.data.clone(),
                timestamp: e.timestamp,
            })
            .collect();

        if !matching.is_empty() {
            return Ok(matching);
        }
        drop(events);

        // Subscribe and wait for new events
        let mut receiver = self.subscribe("all").await;

        loop {
            if start.elapsed() >= timeout {
                return Ok(vec![]);
            }

            let remaining = timeout - start.elapsed();
            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Ok(event)) if event.event_type == event_type => {
                    return Ok(vec![super::EventData {
                        event_id: event.id,
                        event_type: event.event_type,
                        publisher: event.publisher,
                        data: event.data,
                        timestamp: event.timestamp,
                    }]);
                }
                Ok(Ok(_)) => continue,           // Wrong event type
                Ok(Err(_)) => return Ok(vec![]), // Channel closed
                Err(_) => return Ok(vec![]),     // Timeout
            }
        }
    }

    /// Get event queue depth
    pub async fn queue_depth(&self) -> usize {
        self.events.read().await.len()
    }

    /// Get event statistics
    #[allow(dead_code)]
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Get dead letter queue
    pub async fn get_dead_letter_queue(&self) -> Vec<Event> {
        self.dead_letter_queue
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    /// Clear dead letter queue
    pub async fn clear_dead_letter_queue(&self) -> usize {
        let mut dlq = self.dead_letter_queue.write().await;
        let count = dlq.len();
        dlq.clear();
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_and_receive() {
        let bus = EventBus::new();

        let event_id = bus
            .publish(
                "test_event",
                "test_publisher",
                "test_topic",
                serde_json::json!({"key": "value"}),
            )
            .await
            .unwrap();

        assert!(!event_id.is_empty());
        assert_eq!(bus.queue_depth().await, 1);
    }

    #[tokio::test]
    async fn test_subscribe_and_receive() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe("test_topic").await;

        tokio::spawn({
            let bus = bus.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                bus.publish(
                    "test_event",
                    "test_publisher",
                    "test_topic",
                    serde_json::json!({"key": "value"}),
                )
                .await
                .unwrap();
            }
        });

        match tokio::time::timeout(tokio::time::Duration::from_secs(1), receiver.recv()).await {
            Ok(Ok(event)) => {
                assert_eq!(event.event_type, "test_event");
                assert_eq!(event.publisher, "test_publisher");
            }
            _ => panic!("Failed to receive event"),
        }
    }

    #[tokio::test]
    async fn test_circular_buffer() {
        let bus = EventBus::new();

        // Publish more than buffer capacity
        for i in 0..EVENT_BUFFER_CAPACITY + 100 {
            bus.publish(
                "test_event",
                "test_publisher",
                "test_topic",
                serde_json::json!({"index": i}),
            )
            .await
            .unwrap();
        }

        // Should not exceed capacity
        assert_eq!(bus.queue_depth().await, EVENT_BUFFER_CAPACITY);
    }
}
