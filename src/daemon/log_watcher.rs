//! Filesystem watcher for Claude Code conversation logs
//!
//! Automatically detects when new `.jsonl` conversation files are created or updated
//! in `~/.claude/history/` and triggers immediate re-parsing for the stats collection
//! pipeline (Phase 3).
//!
//! Key features:
//! - Recursive watching of ~/.claude/history/ directory
//! - Filters for *.jsonl files only
//! - 1-second debounce to prevent duplicate parsing
//! - Non-blocking async event processing
//! - Continues operation even if individual parse errors occur

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Debounce duration to prevent multiple parsing of the same file
const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

/// Maximum channel buffer size for file change events
const EVENT_CHANNEL_SIZE: usize = 100;

/// Tracks last parse time for debouncing
type DebounceMap = Arc<RwLock<HashMap<PathBuf, Instant>>>;

/// Filesystem watcher for Claude Code conversation logs
pub struct LogWatcher {
    watcher: RecommendedWatcher,
    watch_path: PathBuf,
    #[allow(dead_code)] // Used in tests for clear_debounce()
    debounce_map: DebounceMap,
}

impl LogWatcher {
    /// Create a new log watcher for the specified directory
    ///
    /// # Arguments
    /// * `watch_path` - Path to watch (typically ~/.claude/history/)
    ///
    /// # Returns
    /// * `Self` - The LogWatcher instance
    /// * `mpsc::Receiver<PathBuf>` - Channel receiver for file change events
    ///
    /// # Example
    /// ```no_run
    /// use std::path::PathBuf;
    /// use cco::daemon::log_watcher::LogWatcher;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let watch_path = PathBuf::from("/Users/user/.claude/history");
    ///     let (mut watcher, mut rx) = LogWatcher::new(watch_path).unwrap();
    ///     watcher.start().unwrap();
    ///
    ///     // Process events
    ///     while let Some(path) = rx.recv().await {
    ///         println!("File changed: {:?}", path);
    ///     }
    /// }
    /// ```
    pub fn new(watch_path: PathBuf) -> Result<(Self, mpsc::Receiver<PathBuf>)> {
        if !watch_path.exists() {
            warn!(
                "Watch path does not exist, will create: {}",
                watch_path.display()
            );
            std::fs::create_dir_all(&watch_path).context("Failed to create watch directory")?;
        }

        info!("Initializing log watcher for: {}", watch_path.display());

        // Create channel for file change events
        let (tx, rx) = mpsc::channel(EVENT_CHANNEL_SIZE);
        let debounce_map: DebounceMap = Arc::new(RwLock::new(HashMap::new()));
        let debounce_map_clone = Arc::clone(&debounce_map);

        // Create watcher that sends events to channel
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Filter for modify/create events only
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                // Process each path in the event
                                for path in event.paths {
                                    // Filter: only process .jsonl files
                                    if !Self::is_valid_log_file(&path) {
                                        continue;
                                    }

                                    debug!("Detected change in log file: {:?}", path);

                                    // Check debounce (non-blocking check)
                                    let debounce_map = Arc::clone(&debounce_map_clone);
                                    let tx = tx.clone();

                                    // Try to spawn async task to check debounce and send event
                                    // Use try_current to check if a runtime is available
                                    if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                        handle.spawn(async move {
                                            if Self::should_process(&debounce_map, &path).await {
                                                if let Err(e) = tx.send(path.clone()).await {
                                                    error!(
                                                        "Failed to send file change event: {}",
                                                        e
                                                    );
                                                } else {
                                                    debug!("Queued file for parsing: {:?}", path);
                                                }
                                            }
                                        });
                                    } else {
                                        // No runtime available (e.g., during tests without async context)
                                        // Send synchronously using blocking channel
                                        debug!("No async runtime, skipping debounce check");
                                        if let Err(e) = tx.blocking_send(path.clone()) {
                                            error!(
                                                "Failed to send file change event (blocking): {}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Ignore other event types (delete, rename, etc.)
                            }
                        }
                    }
                    Err(e) => {
                        error!("Filesystem watcher error: {}", e);
                    }
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_secs(2))
                .with_compare_contents(false), // Performance: don't compare file contents
        )
        .context("Failed to create filesystem watcher")?;

        Ok((
            Self {
                watcher,
                watch_path,
                debounce_map,
            },
            rx,
        ))
    }

    /// Start watching the directory
    pub fn start(&mut self) -> Result<()> {
        info!("Starting log watcher on: {}", self.watch_path.display());

        self.watcher
            .watch(&self.watch_path, RecursiveMode::Recursive)
            .context("Failed to start watching directory")?;

        info!("✅ Log watcher started successfully");
        Ok(())
    }

    /// Stop watching the directory
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping log watcher");

        self.watcher
            .unwatch(&self.watch_path)
            .context("Failed to stop watching directory")?;

        info!("✅ Log watcher stopped");
        Ok(())
    }

    /// Check if a file should be processed (filter criteria)
    fn is_valid_log_file(path: &Path) -> bool {
        // Must be a file (not directory)
        if !path.is_file() {
            return false;
        }

        // Get filename
        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return false,
        };

        // Ignore hidden files
        if filename.starts_with('.') {
            return false;
        }

        // Ignore temporary files
        if filename.ends_with(".tmp") || filename.ends_with(".swp") {
            return false;
        }

        // Only process .jsonl files
        if !filename.ends_with(".jsonl") {
            return false;
        }

        true
    }

    /// Check if a file should be processed (debounce logic)
    ///
    /// Returns true if the file hasn't been processed in the last DEBOUNCE_DURATION
    async fn should_process(debounce_map: &DebounceMap, path: &Path) -> bool {
        let now = Instant::now();
        let mut map = debounce_map.write().await;

        if let Some(last_processed) = map.get(path) {
            if now.duration_since(*last_processed) < DEBOUNCE_DURATION {
                debug!(
                    "Debouncing file (processed {:?} ago): {:?}",
                    now.duration_since(*last_processed),
                    path
                );
                return false;
            }
        }

        // Update last processed time
        map.insert(path.to_path_buf(), now);
        true
    }

    /// Get the watched path
    pub fn watch_path(&self) -> &Path {
        &self.watch_path
    }

    /// Clear the debounce map (useful for testing)
    pub async fn clear_debounce(&self) {
        let mut map = self.debounce_map.write().await;
        map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_valid_log_file() {
        // Create temp directory for test files
        let temp_dir = TempDir::new().unwrap();

        // Create valid files
        let valid_jsonl = temp_dir.path().join("conversation.jsonl");
        fs::write(&valid_jsonl, "test").unwrap();
        assert!(LogWatcher::is_valid_log_file(&valid_jsonl));

        let valid_jsonl2 = temp_dir.path().join("test-123.jsonl");
        fs::write(&valid_jsonl2, "test").unwrap();
        assert!(LogWatcher::is_valid_log_file(&valid_jsonl2));

        // Create invalid files
        let hidden = temp_dir.path().join(".hidden.jsonl");
        fs::write(&hidden, "test").unwrap();
        assert!(!LogWatcher::is_valid_log_file(&hidden));

        let tmp_file = temp_dir.path().join("file.tmp");
        fs::write(&tmp_file, "test").unwrap();
        assert!(!LogWatcher::is_valid_log_file(&tmp_file));

        let swp_file = temp_dir.path().join("file.swp");
        fs::write(&swp_file, "test").unwrap();
        assert!(!LogWatcher::is_valid_log_file(&swp_file));

        let txt_file = temp_dir.path().join("file.txt");
        fs::write(&txt_file, "test").unwrap();
        assert!(!LogWatcher::is_valid_log_file(&txt_file));

        // Non-existent file should return false
        let nonexistent = temp_dir.path().join("nonexistent.jsonl");
        assert!(!LogWatcher::is_valid_log_file(&nonexistent));
    }

    #[tokio::test]
    async fn test_debounce_logic() {
        let debounce_map: DebounceMap = Arc::new(RwLock::new(HashMap::new()));
        let path = PathBuf::from("/tmp/test.jsonl");

        // First call should return true
        assert!(LogWatcher::should_process(&debounce_map, &path).await);

        // Immediate second call should return false (debounced)
        assert!(!LogWatcher::should_process(&debounce_map, &path).await);

        // Wait for debounce duration
        tokio::time::sleep(DEBOUNCE_DURATION + Duration::from_millis(100)).await;

        // After debounce period, should return true again
        assert!(LogWatcher::should_process(&debounce_map, &path).await);
    }

    #[tokio::test]
    async fn test_log_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watch_path = temp_dir.path().to_path_buf();

        let result = LogWatcher::new(watch_path.clone());
        assert!(result.is_ok());

        let (mut watcher, _rx) = result.unwrap();
        assert_eq!(watcher.watch_path(), watch_path.as_path());

        // Clean up
        let _ = watcher.stop();
    }

    #[tokio::test]
    async fn test_log_watcher_start_stop() {
        let temp_dir = TempDir::new().unwrap();
        let watch_path = temp_dir.path().to_path_buf();

        let (mut watcher, _rx) = LogWatcher::new(watch_path).unwrap();

        // Start watching
        assert!(watcher.start().is_ok());

        // Stop watching
        assert!(watcher.stop().is_ok());
    }

    #[tokio::test]
    async fn test_log_watcher_detects_new_file() {
        let temp_dir = TempDir::new().unwrap();
        // Canonicalize to resolve /var -> /private/var on macOS
        let watch_path = temp_dir.path().canonicalize().unwrap();

        let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
        watcher.start().unwrap();

        // Create a new .jsonl file
        let test_file = watch_path.join("test.jsonl");
        fs::write(&test_file, "test content").unwrap();

        // Wait for event (with timeout)
        let result = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;

        // Clean up
        let _ = watcher.stop();
        let _ = fs::remove_file(&test_file);

        // Verify event was received
        assert!(result.is_ok());
        let event_path = result.unwrap();
        assert!(event_path.is_some());
        // Compare file names instead of full paths (avoids /var vs /private/var issues)
        assert_eq!(
            event_path.unwrap().file_name().unwrap(),
            test_file.file_name().unwrap()
        );
    }

    #[tokio::test]
    async fn test_log_watcher_ignores_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let watch_path = temp_dir.path().to_path_buf();

        let (mut watcher, mut rx) = LogWatcher::new(watch_path.clone()).unwrap();
        watcher.start().unwrap();

        // Create invalid files (should be ignored)
        let hidden_file = watch_path.join(".hidden.jsonl");
        let temp_file = watch_path.join("test.tmp");
        let txt_file = watch_path.join("test.txt");

        fs::write(&hidden_file, "test").unwrap();
        fs::write(&temp_file, "test").unwrap();
        fs::write(&txt_file, "test").unwrap();

        // Wait a bit to see if any events come through
        let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;

        // Clean up
        let _ = watcher.stop();
        let _ = fs::remove_file(&hidden_file);
        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_file(&txt_file);

        // Should timeout (no valid events)
        assert!(result.is_err());
    }
}
