//! Optimized model download and caching system
//!
//! Provides robust model download with:
//! - Streaming chunk-based downloads (prevents OOM)
//! - Atomic writes with temporary files
//! - SHA256 checksum verification
//! - Concurrent access prevention via file locks
//! - Progress tracking with ETA
//! - Automatic fallback to quantized models
//! - Graceful error recovery

use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Default chunk size for streaming downloads (1MB)
/// This prevents loading entire model into memory
const DOWNLOAD_CHUNK_SIZE: usize = 1024 * 1024;

/// Lock file suffix to prevent concurrent downloads
const LOCK_FILE_SUFFIX: &str = ".download.lock";

/// Temporary file suffix during download
const TEMP_FILE_SUFFIX: &str = ".tmp";

/// Model download configuration
#[derive(Debug, Clone)]
pub struct ModelDownloadConfig {
    /// URL to download the model from
    pub url: String,

    /// Expected SHA256 checksum (hex-encoded)
    /// If None, checksum verification is skipped with a warning
    pub expected_checksum: Option<String>,

    /// Target path for the downloaded model
    pub target_path: PathBuf,

    /// Model size in bytes (for progress bar)
    pub expected_size_bytes: Option<u64>,

    /// Fallback URL if primary download fails
    pub fallback_url: Option<String>,

    /// Maximum download retries
    pub max_retries: u32,
}

/// Model cache manager with optimized download
pub struct ModelCache {
    /// HTTP client for downloads
    client: Client,

    /// Lock to prevent concurrent downloads of the same model
    download_lock: Arc<Mutex<()>>,
}

impl ModelCache {
    /// Create a new model cache manager
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout per request
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            download_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Ensure model is available, downloading if necessary
    ///
    /// This method is thread-safe and prevents concurrent downloads
    /// of the same model file.
    ///
    /// # Arguments
    ///
    /// * `config` - Download configuration including URL and target path
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if model was downloaded, Ok(false) if already exists
    ///
    /// # Errors
    ///
    /// Returns error if download fails after all retries
    pub async fn ensure_model_available(&self, config: &ModelDownloadConfig) -> Result<bool> {
        // Check if model already exists and is valid
        if self.verify_existing_model(&config.target_path, config.expected_checksum.as_deref())? {
            debug!("Model already exists and is valid at {:?}", config.target_path);
            return Ok(false);
        }

        // Acquire download lock to prevent concurrent downloads
        let _lock = self.download_lock.lock().await;

        // Double-check after acquiring lock (another thread may have downloaded)
        if self.verify_existing_model(&config.target_path, config.expected_checksum.as_deref())? {
            debug!("Model was downloaded by another thread");
            return Ok(false);
        }

        // Download the model
        self.download_model(config).await?;

        Ok(true)
    }

    /// Download model with streaming and atomic writes
    async fn download_model(&self, config: &ModelDownloadConfig) -> Result<()> {
        info!("Downloading model from: {}", config.url);

        let mut last_error = None;

        // Try primary URL with retries
        for attempt in 1..=config.max_retries {
            match self.download_with_progress(config, &config.url).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!("Download attempt {}/{} failed: {}", attempt, config.max_retries, e);
                    last_error = Some(e);

                    // Clean up partial download
                    let temp_path = self.get_temp_path(&config.target_path);
                    let _ = fs::remove_file(&temp_path);

                    // Wait before retry (exponential backoff)
                    if attempt < config.max_retries {
                        let delay = std::time::Duration::from_secs(2u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // Try fallback URL if available
        if let Some(fallback_url) = &config.fallback_url {
            info!("Trying fallback URL: {}", fallback_url);

            match self.download_with_progress(config, fallback_url).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    error!("Fallback download failed: {}", e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Download failed after all retries")))
    }

    /// Download with progress tracking and streaming
    async fn download_with_progress(&self, config: &ModelDownloadConfig, url: &str) -> Result<()> {
        // Create lock file to signal download in progress
        let lock_path = self.get_lock_path(&config.target_path);
        self.create_lock_file(&lock_path)?;

        // Ensure cleanup of lock file on error
        let _lock_guard = LockFileGuard::new(lock_path.clone());

        // Ensure parent directory exists
        if let Some(parent) = config.target_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create model directory")?;
        }

        // Initiate HTTP request
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to initiate download")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let total_size = response.content_length()
            .or(config.expected_size_bytes)
            .unwrap_or(0);

        // Create progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
                .context("Failed to create progress bar template")?
                .progress_chars("#>-"),
        );

        // Write to temporary file first (atomic write pattern)
        let temp_path = self.get_temp_path(&config.target_path);
        let temp_file = File::create(&temp_path)
            .context("Failed to create temporary file")?;

        let mut writer = BufWriter::new(temp_file);
        let mut hasher = Sha256::new();
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        // Stream download in chunks
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .context("Download stream error")?;

            // Write chunk to file
            writer.write_all(&chunk)
                .context("Failed to write to temporary file")?;

            // Update hash
            hasher.update(&chunk);

            // Update progress
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Flush all buffered data
        writer.flush()
            .context("Failed to flush temporary file")?;

        pb.finish_with_message("Download complete");

        // Verify checksum if expected hash is provided
        let computed_hash = hex::encode(hasher.finalize());

        if let Some(expected_hash) = &config.expected_checksum {
            if computed_hash != expected_hash.to_lowercase() {
                // Clean up invalid file
                let _ = fs::remove_file(&temp_path);

                anyhow::bail!(
                    "Checksum verification failed!\nExpected: {}\nGot: {}",
                    expected_hash,
                    computed_hash
                );
            }

            info!("Checksum verified: {}", computed_hash);
        } else {
            warn!("Checksum verification skipped - no expected hash provided");
            info!("Downloaded file SHA256: {}", computed_hash);
        }

        // Atomic rename: move temp file to final location
        fs::rename(&temp_path, &config.target_path)
            .context("Failed to move temporary file to target location")?;

        info!("Model downloaded successfully to {:?}", config.target_path);

        Ok(())
    }

    /// Verify existing model file
    ///
    /// Returns true if file exists and checksum matches (if provided)
    pub fn verify_existing_model(&self, path: &Path, expected_checksum: Option<&str>) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // If no checksum provided, just check existence
        let Some(expected_hash) = expected_checksum else {
            return Ok(true);
        };

        info!("Verifying existing model at {:?}", path);

        // Compute checksum of existing file
        let file = File::open(path)
            .context("Failed to open existing model file")?;

        let computed_hash = self.compute_file_checksum(&file)?;

        if computed_hash != expected_hash.to_lowercase() {
            warn!(
                "Existing model checksum mismatch. Expected: {}, Got: {}",
                expected_hash, computed_hash
            );
            return Ok(false);
        }

        info!("Existing model verified successfully");
        Ok(true)
    }

    /// Compute SHA256 checksum of a file
    ///
    /// Uses buffered reading to avoid loading entire file into memory
    pub fn compute_file_checksum(&self, file: &File) -> Result<String> {
        let mut reader = io::BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; DOWNLOAD_CHUNK_SIZE];

        loop {
            let count = io::Read::read(&mut reader, &mut buffer)
                .context("Failed to read file for checksum")?;

            if count == 0 {
                break;
            }

            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Get temporary file path for atomic writes
    pub fn get_temp_path(&self, target_path: &Path) -> PathBuf {
        // Simply append .tmp to the full path (preserving original extension)
        let mut path_str = target_path.as_os_str().to_string_lossy().to_string();
        path_str.push_str(TEMP_FILE_SUFFIX);
        PathBuf::from(path_str)
    }

    /// Get lock file path
    pub fn get_lock_path(&self, target_path: &Path) -> PathBuf {
        // Simply append .download.lock to the full path (preserving original extension)
        let mut path_str = target_path.as_os_str().to_string_lossy().to_string();
        path_str.push_str(LOCK_FILE_SUFFIX);
        PathBuf::from(path_str)
    }

    /// Create lock file
    fn create_lock_file(&self, lock_path: &Path) -> Result<()> {
        if lock_path.exists() {
            // Check if lock is stale (older than 1 hour)
            if let Ok(metadata) = fs::metadata(lock_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > std::time::Duration::from_secs(3600) {
                            warn!("Removing stale lock file: {:?}", lock_path);
                            let _ = fs::remove_file(lock_path);
                        } else {
                            anyhow::bail!("Another download is in progress (lock file: {:?})", lock_path);
                        }
                    }
                }
            }
        }

        // Create lock file
        File::create(lock_path)
            .context("Failed to create lock file")?;

        Ok(())
    }
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for lock file cleanup
pub struct LockFileGuard {
    lock_path: PathBuf,
}

impl LockFileGuard {
    pub fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for LockFileGuard {
    fn drop(&mut self) {
        if self.lock_path.exists() {
            if let Err(e) = fs::remove_file(&self.lock_path) {
                warn!("Failed to remove lock file {:?}: {}", self.lock_path, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_temp_path_generation() {
        let cache = ModelCache::new();
        let target = PathBuf::from("/tmp/model.gguf");
        let temp = cache.get_temp_path(&target);

        assert_eq!(temp, PathBuf::from("/tmp/model.gguf.tmp"));
    }

    #[test]
    fn test_lock_path_generation() {
        let cache = ModelCache::new();
        let target = PathBuf::from("/tmp/model.gguf");
        let lock = cache.get_lock_path(&target);

        assert_eq!(lock, PathBuf::from("/tmp/model.gguf.download.lock"));
    }

    #[test]
    fn test_checksum_computation() {
        let cache = ModelCache::new();

        // Create a temporary file with known content
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test.txt");

        let test_content = b"Hello, World!";
        fs::write(&test_file_path, test_content).unwrap();

        // Compute checksum
        let file = File::open(&test_file_path).unwrap();
        let checksum = cache.compute_file_checksum(&file).unwrap();

        // Expected SHA256 of "Hello, World!"
        let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        assert_eq!(checksum, expected);
    }

    #[tokio::test]
    async fn test_verify_existing_model_not_found() {
        let cache = ModelCache::new();
        let result = cache.verify_existing_model(
            &PathBuf::from("/nonexistent/model.gguf"),
            None
        ).unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_lock_file_guard() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // Create lock file
        File::create(&lock_path).unwrap();
        assert!(lock_path.exists());

        // Guard should remove on drop
        {
            let _guard = LockFileGuard::new(lock_path.clone());
        }

        assert!(!lock_path.exists());
    }

    #[tokio::test]
    async fn test_concurrent_download_prevention() {
        let cache = Arc::new(ModelCache::new());
        let temp_dir = TempDir::new().unwrap();

        let config = ModelDownloadConfig {
            url: "https://example.com/model.gguf".to_string(),
            expected_checksum: None,
            target_path: temp_dir.path().join("model.gguf"),
            expected_size_bytes: Some(1000),
            fallback_url: None,
            max_retries: 1,
        };

        // Acquire lock
        let _lock = cache.download_lock.lock().await;

        // Try to download concurrently (should wait for lock)
        let cache2 = cache.clone();
        let config2 = config.clone();

        let handle = tokio::spawn(async move {
            // This will wait for the lock
            cache2.ensure_model_available(&config2).await
        });

        // Give spawned task a moment to try acquiring lock
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Lock should still be held
        assert!(!handle.is_finished());

        // Release lock
        drop(_lock);

        // Now the spawned task should be able to proceed (will fail due to bad URL, but that's ok)
        let _ = handle.await;
    }
}
