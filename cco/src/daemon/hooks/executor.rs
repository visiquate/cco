//! Hook executor with timeout and retry support
//!
//! Executes registered hooks with configurable timeouts and retry logic.
//! Ensures hook failures don't block daemon operation.

use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::error::{HookError, HookResult};
use super::registry::HookRegistry;
use super::types::{HookPayload, HookType};

/// Default timeout for hook execution (5 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default maximum retry attempts
const DEFAULT_MAX_RETRIES: u32 = 2;

/// Hook executor with async execution and timeout handling
///
/// The executor manages the execution of registered hooks with:
/// - Configurable timeouts to prevent blocking
/// - Retry logic for transient failures
/// - Panic recovery to prevent daemon crashes
/// - Detailed error logging
///
/// # Thread Safety
///
/// The executor is clone-able and can be shared across async tasks.
/// All hook executions are independent and non-blocking.
///
/// # Example
///
/// ```rust
/// use cco::daemon::hooks::{HookRegistry, HookExecutor, HookType, HookPayload};
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// let registry = Arc::new(HookRegistry::new());
/// let executor = HookExecutor::new(registry);
///
/// let payload = HookPayload::new("git status");
/// executor.execute_hook(HookType::PreCommand, payload).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct HookExecutor {
    /// Registry containing all registered hooks
    registry: Arc<HookRegistry>,

    /// Timeout for hook execution
    timeout: Duration,

    /// Maximum number of retry attempts
    max_retries: u32,
}

impl HookExecutor {
    /// Create a new executor with the given registry
    ///
    /// Uses default timeout (5 seconds) and retry count (2).
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookExecutor};
    /// use std::sync::Arc;
    ///
    /// let registry = Arc::new(HookRegistry::new());
    /// let executor = HookExecutor::new(registry);
    /// ```
    pub fn new(registry: Arc<HookRegistry>) -> Self {
        Self {
            registry,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Create an executor with custom timeout
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookExecutor};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// let registry = Arc::new(HookRegistry::new());
    /// let executor = HookExecutor::with_timeout(registry, Duration::from_secs(10));
    /// ```
    pub fn with_timeout(registry: Arc<HookRegistry>, timeout: Duration) -> Self {
        Self {
            registry,
            timeout,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Create an executor with custom timeout and retry count
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookExecutor};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// let registry = Arc::new(HookRegistry::new());
    /// let executor = HookExecutor::with_config(
    ///     registry,
    ///     Duration::from_secs(10),
    ///     5
    /// );
    /// ```
    pub fn with_config(registry: Arc<HookRegistry>, timeout: Duration, max_retries: u32) -> Self {
        Self {
            registry,
            timeout,
            max_retries,
        }
    }

    /// Execute all registered hooks for the given type
    ///
    /// Executes hooks in registration order. If a hook fails:
    /// - The error is logged
    /// - Retries are attempted if configured
    /// - Remaining hooks are still executed
    /// - The first error is returned
    ///
    /// # Arguments
    ///
    /// * `hook_type` - The type of hooks to execute
    /// * `payload` - The payload to pass to hooks
    ///
    /// # Errors
    ///
    /// Returns the first error encountered, or Ok(()) if all hooks succeed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookExecutor, HookType, HookPayload};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = Arc::new(HookRegistry::new());
    /// let executor = HookExecutor::new(registry);
    ///
    /// let payload = HookPayload::new("git status");
    /// executor.execute_hook(HookType::PreCommand, payload).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_hook(&self, hook_type: HookType, payload: HookPayload) -> HookResult<()> {
        let hooks = self.registry.get_hooks(hook_type);

        if hooks.is_empty() {
            debug!(hook_type = %hook_type, "No hooks registered");
            return Ok(());
        }

        info!(
            hook_type = %hook_type,
            count = hooks.len(),
            command = %payload.command,
            "Executing hooks"
        );

        let mut first_error = None;

        for (idx, hook) in hooks.iter().enumerate() {
            match self.execute_single_hook(hook_type, hook.clone(), &payload).await {
                Ok(_) => {
                    debug!(
                        hook_type = %hook_type,
                        index = idx,
                        "Hook executed successfully"
                    );
                }
                Err(e) => {
                    error!(
                        hook_type = %hook_type,
                        index = idx,
                        error = %e,
                        "Hook execution failed"
                    );

                    // Store first error but continue executing remaining hooks
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }

        if let Some(err) = first_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Execute a single hook with timeout and retry logic
    async fn execute_single_hook(
        &self,
        hook_type: HookType,
        hook: Arc<dyn super::types::Hook>,
        payload: &HookPayload,
    ) -> HookResult<()> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.max_retries {
            attempts += 1;

            match self.execute_with_timeout(hook_type, hook.clone(), payload).await {
                Ok(_) => {
                    if attempts > 1 {
                        info!(
                            hook_type = %hook_type,
                            attempts = attempts,
                            "Hook succeeded after retry"
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);

                    // Only retry if error is retryable and we have retries left
                    if let Some(err) = &last_error {
                        if !err.is_retryable() || attempts > self.max_retries {
                            break;
                        }

                        warn!(
                            hook_type = %hook_type,
                            attempt = attempts,
                            max_retries = self.max_retries,
                            error = %err,
                            "Hook failed, retrying..."
                        );

                        // Brief delay before retry
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }

        // All retries exhausted
        if attempts > self.max_retries {
            Err(HookError::max_retries_exceeded(hook_type.to_string(), self.max_retries))
        } else {
            Err(last_error.unwrap())
        }
    }

    /// Execute hook with timeout protection
    async fn execute_with_timeout(
        &self,
        hook_type: HookType,
        hook: Arc<dyn super::types::Hook>,
        payload: &HookPayload,
    ) -> HookResult<()> {
        let hook_type_str = hook_type.to_string();
        let payload_clone = payload.clone();

        // Spawn blocking task for hook execution to prevent blocking async runtime
        let task = tokio::task::spawn_blocking(move || {
            // Catch panics to prevent daemon crash
            std::panic::catch_unwind(AssertUnwindSafe(|| {
                hook.execute(&payload_clone)
            }))
        });

        // Apply timeout
        match timeout(self.timeout, task).await {
            Ok(Ok(Ok(result))) => result,
            Ok(Ok(Err(panic_err))) => {
                let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                Err(HookError::panic_recovery(hook_type_str, panic_msg))
            }
            Ok(Err(join_err)) => {
                Err(HookError::execution_failed(hook_type_str, format!("Task join error: {}", join_err)))
            }
            Err(_) => {
                Err(HookError::timeout(hook_type_str, self.timeout))
            }
        }
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the configured max retries
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::error::HookResult;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn create_executor() -> (Arc<HookRegistry>, HookExecutor) {
        let registry = Arc::new(HookRegistry::new());
        let executor = HookExecutor::new(registry.clone());
        (registry, executor)
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let (_, executor) = create_executor();
        assert_eq!(executor.timeout(), DEFAULT_TIMEOUT);
        assert_eq!(executor.max_retries(), DEFAULT_MAX_RETRIES);
    }

    #[tokio::test]
    async fn test_execute_no_hooks() {
        let (_, executor) = create_executor();
        let payload = HookPayload::new("test");

        let result = executor.execute_hook(HookType::PreCommand, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_successful_hook() {
        let (registry, executor) = create_executor();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        registry.register(
            HookType::PreCommand,
            Box::new(move |_: &super::HookPayload| -> HookResult<()> {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_execute_multiple_hooks() {
        let (registry, executor) = create_executor();

        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let counter_clone = counter.clone();
            registry.register(
                HookType::PreCommand,
                Box::new(move |_: &super::HookPayload| -> HookResult<()> {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            ).unwrap();
        }

        let payload = HookPayload::new("test");
        executor.execute_hook(HookType::PreCommand, payload).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_hook_failure() {
        let (registry, executor) = create_executor();

        registry.register(
            HookType::PreCommand,
            Box::new(|_: &super::HookPayload| -> HookResult<()> {
                Err(HookError::execution_failed("test", "test error"))
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        let result = executor.execute_hook(HookType::PreCommand, payload).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hook_timeout() {
        let (registry, executor) = create_executor();

        // Register a hook that takes too long
        registry.register(
            HookType::PreCommand,
            Box::new(|_: &super::HookPayload| -> HookResult<()> {
                std::thread::sleep(Duration::from_secs(10));
                Ok(())
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        let result = executor.execute_hook(HookType::PreCommand, payload).await;

        assert!(result.is_err());
        // Should be either timeout or max retries (timeout triggers retries)
        let err = result.unwrap_err();
        assert!(err.is_timeout() || matches!(err, HookError::MaxRetriesExceeded { .. }));
    }

    #[tokio::test]
    async fn test_hook_panic_recovery() {
        let (registry, executor) = create_executor();

        // Register a hook that panics
        registry.register(
            HookType::PreCommand,
            Box::new(|_: &super::HookPayload| -> HookResult<()> {
                panic!("Test panic");
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        let result = executor.execute_hook(HookType::PreCommand, payload).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().is_panic());
    }

    #[tokio::test]
    async fn test_hook_retry_success() {
        let (registry, executor) = create_executor();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // Hook that fails once then succeeds
        registry.register(
            HookType::PreCommand,
            Box::new(move |_: &super::HookPayload| -> HookResult<()> {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    Err(HookError::execution_failed("test", "first attempt"))
                } else {
                    Ok(())
                }
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        let result = executor.execute_hook(HookType::PreCommand, payload).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Failed once, succeeded second time
    }

    #[tokio::test]
    async fn test_custom_timeout() {
        let registry = Arc::new(HookRegistry::new());
        let executor = HookExecutor::with_timeout(registry, Duration::from_millis(100));

        assert_eq!(executor.timeout(), Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_custom_config() {
        let registry = Arc::new(HookRegistry::new());
        let executor = HookExecutor::with_config(
            registry,
            Duration::from_secs(10),
            5
        );

        assert_eq!(executor.timeout(), Duration::from_secs(10));
        assert_eq!(executor.max_retries(), 5);
    }

    #[tokio::test]
    async fn test_continue_on_hook_failure() {
        let (registry, executor) = create_executor();

        let counter = Arc::new(AtomicUsize::new(0));

        // First hook fails
        registry.register(
            HookType::PreCommand,
            Box::new(|_: &super::HookPayload| -> HookResult<()> {
                Err(HookError::execution_failed("test", "error"))
            })
        ).unwrap();

        // Second hook succeeds
        let counter_clone = counter.clone();
        registry.register(
            HookType::PreCommand,
            Box::new(move |_: &super::HookPayload| -> HookResult<()> {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        ).unwrap();

        let payload = HookPayload::new("test");
        let result = executor.execute_hook(HookType::PreCommand, payload).await;

        // First error is returned
        assert!(result.is_err());

        // But second hook still executed
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
