//! In-memory metrics cache for fast /api/stats responses
//!
//! Provides a thread-safe in-memory cache with background aggregation task
//! to reduce /api/stats response time from 7s to <10ms.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Snapshot of metrics at a point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatsSnapshot {
    pub timestamp: SystemTime,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: f64,
    pub uptime: Duration,
    pub port: u16,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub messages_count: u64,
}

/// Thread-safe in-memory cache for metrics snapshots
pub struct MetricsCache {
    cache: Arc<RwLock<Vec<StatsSnapshot>>>,
    max_entries: usize,
}

impl MetricsCache {
    /// Create new metrics cache with specified capacity
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of snapshots to store (e.g., 3600 for 1 hour @ 1/sec)
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Update cache with new snapshot (called by background task)
    ///
    /// Automatically prunes old entries when exceeding max_entries
    pub fn update(&self, snapshot: StatsSnapshot) {
        let mut cache = self.cache.write();

        // Add new snapshot
        cache.push(snapshot);

        // Keep only last max_entries (ring buffer behavior)
        if cache.len() > self.max_entries {
            cache.remove(0);
        }
    }

    /// Get latest snapshot (called by /api/stats)
    ///
    /// Returns None if cache is empty
    pub fn get_latest(&self) -> Option<StatsSnapshot> {
        let cache = self.cache.read();
        cache.last().cloned()
    }

    /// Get snapshots in time range
    ///
    /// # Arguments
    /// * `start` - Start time (inclusive)
    /// * `end` - End time (inclusive)
    pub fn get_range(&self, start: SystemTime, end: SystemTime) -> Vec<StatsSnapshot> {
        let cache = self.cache.read();
        cache.iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get all snapshots (for detailed analysis)
    pub fn get_all(&self) -> Vec<StatsSnapshot> {
        let cache = self.cache.read();
        cache.clone()
    }

    /// Get number of cached snapshots
    pub fn len(&self) -> usize {
        let cache = self.cache.read();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read();
        cache.is_empty()
    }
}

impl Clone for MetricsCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            max_entries: self.max_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_cache_creation() {
        let cache = MetricsCache::new(100);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_metrics_cache_update() {
        let cache = MetricsCache::new(100);

        let snapshot = StatsSnapshot {
            timestamp: SystemTime::now(),
            total_requests: 10,
            successful_requests: 8,
            failed_requests: 2,
            avg_response_time: 50.0,
            uptime: Duration::from_secs(3600),
            port: 3000,
            total_cost: 0.5,
            total_tokens: 1000,
            messages_count: 5,
        };

        cache.update(snapshot.clone());
        assert_eq!(cache.len(), 1);

        let latest = cache.get_latest();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().total_requests, 10);
    }

    #[test]
    fn test_metrics_cache_ring_buffer() {
        let cache = MetricsCache::new(10);

        // Add 15 entries
        for i in 0..15 {
            let snapshot = StatsSnapshot {
                timestamp: SystemTime::now(),
                total_requests: i,
                successful_requests: i,
                failed_requests: 0,
                avg_response_time: 50.0,
                uptime: Duration::from_secs(i),
                port: 3000,
                total_cost: 0.0,
                total_tokens: 0,
                messages_count: 0,
            };
            cache.update(snapshot);
        }

        // Should keep only last 10
        assert_eq!(cache.len(), 10);

        // Latest should be entry 14 (0-indexed)
        let latest = cache.get_latest().unwrap();
        assert_eq!(latest.total_requests, 14);
    }

    #[test]
    fn test_metrics_cache_clone() {
        let cache1 = MetricsCache::new(100);

        let snapshot = StatsSnapshot {
            timestamp: SystemTime::now(),
            total_requests: 42,
            successful_requests: 40,
            failed_requests: 2,
            avg_response_time: 25.0,
            uptime: Duration::from_secs(1800),
            port: 3000,
            total_cost: 1.5,
            total_tokens: 5000,
            messages_count: 20,
        };

        cache1.update(snapshot);

        let cache2 = cache1.clone();
        assert_eq!(cache2.len(), 1);

        let latest = cache2.get_latest().unwrap();
        assert_eq!(latest.total_requests, 42);
    }

    #[test]
    fn test_metrics_cache_time_range() {
        let cache = MetricsCache::new(100);
        let now = SystemTime::now();

        // Add snapshots with different timestamps
        for i in 0..5 {
            let timestamp = now + Duration::from_secs(i * 10);
            let snapshot = StatsSnapshot {
                timestamp,
                total_requests: i,
                successful_requests: i,
                failed_requests: 0,
                avg_response_time: 50.0,
                uptime: Duration::from_secs(i * 10),
                port: 3000,
                total_cost: 0.0,
                total_tokens: 0,
                messages_count: 0,
            };
            cache.update(snapshot);
        }

        // Query range (should get middle 3 entries)
        let start = now + Duration::from_secs(10);
        let end = now + Duration::from_secs(30);
        let range = cache.get_range(start, end);

        assert_eq!(range.len(), 3); // entries 1, 2, 3
    }
}
