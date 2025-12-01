//! In-memory metrics cache for fast /api/stats responses
//!
//! Provides a thread-safe in-memory cache with background aggregation task
//! to reduce /api/stats response time from 7s to <10ms.
//!
//! Performance optimizations:
//! - Write batching: Accumulate writes in memory, flush every 30s or when buffer full
//! - Reduces database I/O overhead from 50ms per write to 50ms per 100 writes
//!
//! ## Atomic Metrics Aggregation
//! - `AggregatedMetricsCache`: Atomic counters for thread-safe stats aggregation
//! - Integrates with `crate::metrics::MetricsEngine` for real-time API call tracking
//! - O(1) snapshot reads with <50ms target latency for /api/stats endpoint

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
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

/// Event to be written to database
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricEvent {
    pub timestamp: SystemTime,
    pub model_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost: f64,
}

/// Thread-safe in-memory cache for metrics snapshots with write batching
pub struct MetricsCache {
    cache: Arc<RwLock<Vec<StatsSnapshot>>>,
    max_entries: usize,
    /// Pending writes buffer for batching
    pending_writes: Arc<RwLock<Vec<MetricEvent>>>,
    /// Maximum buffer size before forcing flush
    max_buffer_size: usize,
}

impl MetricsCache {
    /// Create new metrics cache with specified capacity
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of snapshots to store (e.g., 3600 for 1 hour @ 1/sec)
    pub fn new(max_entries: usize) -> Self {
        Self::with_buffer_size(max_entries, 100)
    }

    /// Create new metrics cache with custom buffer size
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of snapshots to store
    /// * `max_buffer_size` - Maximum pending writes before forcing flush (default: 100)
    pub fn with_buffer_size(max_entries: usize, max_buffer_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            max_entries,
            pending_writes: Arc::new(RwLock::new(Vec::with_capacity(max_buffer_size))),
            max_buffer_size,
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
        cache
            .iter()
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

    /// Queue a metric event for batched database write
    ///
    /// Events are accumulated in memory and written to database
    /// in batches to reduce I/O overhead.
    ///
    /// # Arguments
    /// * `event` - Metric event to queue
    ///
    /// # Returns
    /// true if buffer is full and needs flushing
    pub fn queue_write(&self, event: MetricEvent) -> bool {
        let mut pending = self.pending_writes.write();
        pending.push(event);
        pending.len() >= self.max_buffer_size
    }

    /// Get and clear pending writes for database flush
    ///
    /// Returns all pending writes and clears the buffer.
    /// Should be called periodically (every 30s) or when buffer is full.
    ///
    /// # Returns
    /// Vec of pending metric events
    pub fn take_pending_writes(&self) -> Vec<MetricEvent> {
        let mut pending = self.pending_writes.write();
        std::mem::take(&mut *pending)
    }

    /// Get count of pending writes without draining
    pub fn pending_write_count(&self) -> usize {
        let pending = self.pending_writes.read();
        pending.len()
    }

    /// Check if buffer is full and needs flushing
    pub fn needs_flush(&self) -> bool {
        let pending = self.pending_writes.read();
        pending.len() >= self.max_buffer_size
    }
}

impl Clone for MetricsCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            max_entries: self.max_entries,
            pending_writes: self.pending_writes.clone(),
            max_buffer_size: self.max_buffer_size,
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

    #[test]
    fn test_write_batching() {
        let cache = MetricsCache::with_buffer_size(100, 3);

        assert_eq!(cache.pending_write_count(), 0);
        assert!(!cache.needs_flush());

        // Add events
        let event1 = MetricEvent {
            timestamp: SystemTime::now(),
            model_name: "claude-sonnet-4-5".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            total_cost: 0.01,
        };

        let should_flush = cache.queue_write(event1.clone());
        assert!(!should_flush);
        assert_eq!(cache.pending_write_count(), 1);

        cache.queue_write(event1.clone());
        assert_eq!(cache.pending_write_count(), 2);

        // Third event should trigger flush threshold
        let should_flush = cache.queue_write(event1);
        assert!(should_flush);
        assert!(cache.needs_flush());
        assert_eq!(cache.pending_write_count(), 3);
    }

    #[test]
    fn test_take_pending_writes() {
        let cache = MetricsCache::with_buffer_size(100, 10);

        // Queue 5 events
        for i in 0..5 {
            let event = MetricEvent {
                timestamp: SystemTime::now(),
                model_name: format!("model-{}", i),
                input_tokens: 100,
                output_tokens: 50,
                cache_write_tokens: 0,
                cache_read_tokens: 0,
                total_cost: 0.001,
            };
            cache.queue_write(event);
        }

        assert_eq!(cache.pending_write_count(), 5);

        // Take pending writes
        let pending = cache.take_pending_writes();
        assert_eq!(pending.len(), 5);
        assert_eq!(cache.pending_write_count(), 0);

        // Taking again should return empty vec
        let pending2 = cache.take_pending_writes();
        assert_eq!(pending2.len(), 0);
    }

    #[test]
    fn test_needs_flush_threshold() {
        let cache = MetricsCache::with_buffer_size(100, 5);

        // Add 4 events (below threshold)
        for _ in 0..4 {
            let event = MetricEvent {
                timestamp: SystemTime::now(),
                model_name: "test-model".to_string(),
                input_tokens: 100,
                output_tokens: 50,
                cache_write_tokens: 0,
                cache_read_tokens: 0,
                total_cost: 0.001,
            };
            cache.queue_write(event);
        }

        assert!(!cache.needs_flush());

        // Add 5th event (reaches threshold)
        let event = MetricEvent {
            timestamp: SystemTime::now(),
            model_name: "test-model".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            total_cost: 0.001,
        };
        cache.queue_write(event);

        assert!(cache.needs_flush());
    }
}

// ============================================================================
// Aggregated Metrics Cache with Atomic Counters
// ============================================================================

/// Token counts by type (for aggregated cache)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokensByType {
    pub input: u64,
    pub output: u64,
    pub cache_write: u64,
    pub cache_read: u64,
}

/// Statistics for a specific model tier (for aggregated cache)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    pub tokens: u64,
    pub cost_cents: u64,
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
}

/// Snapshot of aggregated metrics (for /api/stats endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSnapshot {
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub request_count: u64,
    pub error_count: u64,
    pub last_updated: DateTime<Utc>,
    pub first_request: Option<DateTime<Utc>>,
    pub tokens_by_type: TokensByType,
    pub model_stats: HashMap<String, ModelStats>,
}

/// Thread-safe aggregated metrics cache with atomic counters
///
/// This cache is designed to be updated by a background parser task
/// every 5 seconds and provide O(1) reads to the /api/stats endpoint.
///
/// Uses atomic counters for thread-safe numeric updates without locks,
/// and RwLock for complex types (timestamps, breakdowns, model stats).
pub struct AggregatedMetricsCache {
    // Atomic counters for thread-safe updates (no locks needed)
    total_tokens: AtomicU64,
    total_cost_cents: AtomicU64, // Store as cents to avoid floats in atomics
    request_count: AtomicU64,
    error_count: AtomicU64,

    // Timestamps (need RwLock for complex types)
    last_updated: RwLock<DateTime<Utc>>,
    first_request: RwLock<Option<DateTime<Utc>>>,

    // Token breakdown by type (need RwLock for struct)
    tokens_by_type: RwLock<TokensByType>,

    // Breakdown by model tier (need RwLock for HashMap)
    model_stats: RwLock<HashMap<String, ModelStats>>,
}

impl AggregatedMetricsCache {
    /// Create a new empty aggregated metrics cache
    pub fn new() -> Self {
        Self {
            total_tokens: AtomicU64::new(0),
            total_cost_cents: AtomicU64::new(0),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_updated: RwLock::new(Utc::now()),
            first_request: RwLock::new(None),
            tokens_by_type: RwLock::new(TokensByType::default()),
            model_stats: RwLock::new(HashMap::new()),
        }
    }

    /// Update cache with new metrics summary (incremental update)
    ///
    /// This method aggregates new metrics into the cache, adding to
    /// existing counts. Should be called by the background parser task
    /// every 5 seconds with new metrics from the MetricsEngine.
    ///
    /// # Arguments
    /// * `summary` - Metrics summary from `crate::metrics::MetricsEngine`
    pub fn update(&self, summary: &crate::metrics::MetricsSummary) {
        // Update atomic counters (using Relaxed ordering for performance)
        let new_tokens = summary.total_tokens;
        let new_cost_cents = (summary.total_cost_usd * 100.0).round() as u64;
        let new_requests = summary.call_count;

        self.total_tokens.fetch_add(new_tokens, Ordering::Relaxed);
        self.total_cost_cents
            .fetch_add(new_cost_cents, Ordering::Relaxed);
        self.request_count
            .fetch_add(new_requests, Ordering::Relaxed);

        // Update timestamps
        {
            let mut last_updated = self.last_updated.write();
            *last_updated = Utc::now();
        }

        // Set first_request timestamp if not already set
        {
            let mut first_request = self.first_request.write();
            if first_request.is_none() && new_requests > 0 {
                *first_request = Some(Utc::now());
            }
        }

        // Update token breakdown by type
        {
            let mut tokens_by_type = self.tokens_by_type.write();
            tokens_by_type.input += summary.tokens_by_type.input;
            tokens_by_type.output += summary.tokens_by_type.output;
            tokens_by_type.cache_write += summary.tokens_by_type.cache_write;
            tokens_by_type.cache_read += summary.tokens_by_type.cache_read;
        }

        // Update model breakdown
        {
            let mut model_stats = self.model_stats.write();
            for (tier_name, tier_metrics) in &summary.by_model_tier {
                let stats = model_stats.entry(tier_name.clone()).or_default();
                stats.tokens += tier_metrics.total_tokens;
                stats.cost_cents += (tier_metrics.total_cost_usd * 100.0).round() as u64;
                stats.request_count += tier_metrics.call_count;
                stats.input_tokens += tier_metrics.input_tokens;
                stats.output_tokens += tier_metrics.output_tokens;
                stats.cache_write_tokens += tier_metrics.cache_write_tokens;
                stats.cache_read_tokens += tier_metrics.cache_read_tokens;
            }
        }
    }

    /// Set the entire cache state (for initial load or reset)
    ///
    /// Unlike `update()`, this replaces the entire cache state
    /// rather than adding to existing values.
    ///
    /// # Arguments
    /// * `summary` - Metrics summary from `crate::metrics::MetricsEngine`
    pub fn set(&self, summary: &crate::metrics::MetricsSummary) {
        // Replace atomic counters
        let total_tokens = summary.total_tokens;
        let total_cost_cents = (summary.total_cost_usd * 100.0).round() as u64;
        let request_count = summary.call_count;

        self.total_tokens.store(total_tokens, Ordering::Relaxed);
        self.total_cost_cents
            .store(total_cost_cents, Ordering::Relaxed);
        self.request_count.store(request_count, Ordering::Relaxed);

        // Update timestamps
        {
            let mut last_updated = self.last_updated.write();
            *last_updated = Utc::now();
        }

        {
            let mut first_request = self.first_request.write();
            if first_request.is_none() && request_count > 0 {
                *first_request = Some(Utc::now());
            }
        }

        // Replace token breakdown
        {
            let mut tokens_by_type = self.tokens_by_type.write();
            *tokens_by_type = TokensByType {
                input: summary.tokens_by_type.input,
                output: summary.tokens_by_type.output,
                cache_write: summary.tokens_by_type.cache_write,
                cache_read: summary.tokens_by_type.cache_read,
            };
        }

        // Replace model stats
        {
            let mut model_stats = self.model_stats.write();
            model_stats.clear();
            for (tier_name, tier_metrics) in &summary.by_model_tier {
                model_stats.insert(
                    tier_name.clone(),
                    ModelStats {
                        tokens: tier_metrics.total_tokens,
                        cost_cents: (tier_metrics.total_cost_usd * 100.0).round() as u64,
                        request_count: tier_metrics.call_count,
                        input_tokens: tier_metrics.input_tokens,
                        output_tokens: tier_metrics.output_tokens,
                        cache_write_tokens: tier_metrics.cache_write_tokens,
                        cache_read_tokens: tier_metrics.cache_read_tokens,
                    },
                );
            }
        }
    }

    /// Get a snapshot of current aggregated metrics (fast O(1) read)
    ///
    /// This method is optimized for serving the /api/stats endpoint
    /// with minimal latency (< 50ms target).
    ///
    /// # Returns
    /// Snapshot of all aggregated metrics
    pub fn get_snapshot(&self) -> AggregatedSnapshot {
        // Read atomic counters (fast, no locks)
        let total_tokens = self.total_tokens.load(Ordering::Relaxed);
        let total_cost_cents = self.total_cost_cents.load(Ordering::Relaxed);
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);

        // Convert cents back to USD
        let total_cost_usd = total_cost_cents as f64 / 100.0;

        // Read complex types (requires locks, but should be fast)
        let last_updated = *self.last_updated.read();
        let first_request = *self.first_request.read();
        let tokens_by_type = self.tokens_by_type.read().clone();
        let model_stats = self.model_stats.read().clone();

        AggregatedSnapshot {
            total_tokens,
            total_cost_usd,
            request_count,
            error_count,
            last_updated,
            first_request,
            tokens_by_type,
            model_stats,
        }
    }

    /// Increment error count (for tracking failed requests)
    pub fn increment_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Clear all metrics (for testing or reset)
    pub fn clear(&self) {
        self.total_tokens.store(0, Ordering::Relaxed);
        self.total_cost_cents.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);

        {
            let mut last_updated = self.last_updated.write();
            *last_updated = Utc::now();
        }

        {
            let mut first_request = self.first_request.write();
            *first_request = None;
        }

        {
            let mut tokens_by_type = self.tokens_by_type.write();
            *tokens_by_type = TokensByType::default();
        }

        {
            let mut model_stats = self.model_stats.write();
            model_stats.clear();
        }
    }
}

impl Default for AggregatedMetricsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod aggregated_tests {
    use super::*;
    use crate::metrics::engine::{MetricsSummary, TierMetrics, TokensByType as EngineTokensByType};

    fn create_test_summary() -> MetricsSummary {
        let mut by_model_tier = HashMap::new();
        by_model_tier.insert(
            "Opus".to_string(),
            TierMetrics {
                call_count: 5,
                total_cost_usd: 2.50,
                total_tokens: 50000,
                input_tokens: 30000,
                output_tokens: 20000,
                cache_write_tokens: 0,
                cache_read_tokens: 0,
            },
        );
        by_model_tier.insert(
            "Sonnet".to_string(),
            TierMetrics {
                call_count: 10,
                total_cost_usd: 1.25,
                total_tokens: 75000,
                input_tokens: 50000,
                output_tokens: 25000,
                cache_write_tokens: 0,
                cache_read_tokens: 0,
            },
        );

        MetricsSummary {
            total_cost_usd: 3.75,
            call_count: 15,
            tokens_by_type: EngineTokensByType {
                input: 80000,
                output: 45000,
                cache_write: 0,
                cache_read: 0,
            },
            by_model_tier,
            total_tokens: 125000,
        }
    }

    #[test]
    fn test_aggregated_cache_creation() {
        let cache = AggregatedMetricsCache::new();
        let snapshot = cache.get_snapshot();

        assert_eq!(snapshot.total_tokens, 0);
        assert_eq!(snapshot.total_cost_usd, 0.0);
        assert_eq!(snapshot.request_count, 0);
        assert_eq!(snapshot.error_count, 0);
    }

    #[test]
    fn test_aggregated_cache_update() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();

        cache.update(&summary);

        let snapshot = cache.get_snapshot();
        assert_eq!(snapshot.total_tokens, 125000);
        assert_eq!(snapshot.request_count, 15);
        assert!((snapshot.total_cost_usd - 3.75).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_cache_incremental_update() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();

        // First update
        cache.update(&summary);
        let snapshot1 = cache.get_snapshot();
        assert_eq!(snapshot1.total_tokens, 125000);
        assert_eq!(snapshot1.request_count, 15);

        // Second update (should accumulate)
        cache.update(&summary);
        let snapshot2 = cache.get_snapshot();
        assert_eq!(snapshot2.total_tokens, 250000); // 125k * 2
        assert_eq!(snapshot2.request_count, 30); // 15 * 2
        assert!((snapshot2.total_cost_usd - 7.50).abs() < 0.01); // 3.75 * 2
    }

    #[test]
    fn test_aggregated_cache_set() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();

        // First set
        cache.set(&summary);
        let snapshot1 = cache.get_snapshot();
        assert_eq!(snapshot1.total_tokens, 125000);

        // Second set (should replace, not accumulate)
        cache.set(&summary);
        let snapshot2 = cache.get_snapshot();
        assert_eq!(snapshot2.total_tokens, 125000); // Same as first
        assert_eq!(snapshot2.request_count, 15);
    }

    #[test]
    fn test_aggregated_cache_snapshot_performance() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();
        cache.update(&summary);

        // Multiple fast reads (should be O(1))
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = cache.get_snapshot();
        }
        let elapsed = start.elapsed();

        // Should complete 1000 reads in < 10ms (< 10μs per read)
        assert!(
            elapsed.as_millis() < 10,
            "Snapshot reads too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_aggregated_cache_error_tracking() {
        let cache = AggregatedMetricsCache::new();

        assert_eq!(cache.get_snapshot().error_count, 0);

        cache.increment_errors();
        assert_eq!(cache.get_snapshot().error_count, 1);

        cache.increment_errors();
        cache.increment_errors();
        assert_eq!(cache.get_snapshot().error_count, 3);
    }

    #[test]
    fn test_aggregated_cache_clear() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();
        cache.update(&summary);

        let snapshot1 = cache.get_snapshot();
        assert_eq!(snapshot1.total_tokens, 125000);

        cache.clear();

        let snapshot2 = cache.get_snapshot();
        assert_eq!(snapshot2.total_tokens, 0);
        assert_eq!(snapshot2.request_count, 0);
        assert_eq!(snapshot2.total_cost_usd, 0.0);
    }

    #[test]
    fn test_aggregated_cache_model_breakdown() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();
        cache.update(&summary);

        let snapshot = cache.get_snapshot();

        assert_eq!(snapshot.model_stats.len(), 2);

        let opus_stats = snapshot.model_stats.get("Opus").unwrap();
        assert_eq!(opus_stats.tokens, 50000);
        assert_eq!(opus_stats.request_count, 5);
        assert!((opus_stats.cost_cents as f64 / 100.0 - 2.50).abs() < 0.01);

        let sonnet_stats = snapshot.model_stats.get("Sonnet").unwrap();
        assert_eq!(sonnet_stats.tokens, 75000);
        assert_eq!(sonnet_stats.request_count, 10);
        assert!((sonnet_stats.cost_cents as f64 / 100.0 - 1.25).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_cache_tokens_by_type() {
        let cache = AggregatedMetricsCache::new();
        let summary = create_test_summary();
        cache.update(&summary);

        let snapshot = cache.get_snapshot();

        assert_eq!(snapshot.tokens_by_type.input, 80000);
        assert_eq!(snapshot.tokens_by_type.output, 45000);
        assert_eq!(snapshot.tokens_by_type.cache_write, 0);
        assert_eq!(snapshot.tokens_by_type.cache_read, 0);
    }

    #[test]
    fn test_aggregated_cache_first_request_timestamp() {
        let cache = AggregatedMetricsCache::new();

        let snapshot1 = cache.get_snapshot();
        assert!(snapshot1.first_request.is_none());

        let summary = create_test_summary();
        cache.update(&summary);

        let snapshot2 = cache.get_snapshot();
        assert!(snapshot2.first_request.is_some());
    }
}
