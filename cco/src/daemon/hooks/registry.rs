//! Hook registry for managing hook callbacks
//!
//! Provides thread-safe registration and lookup of hooks by type.
//! Multiple hooks can be registered for the same hook type and will
//! be executed in registration order.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

use super::error::{HookError, HookResult};
use super::types::{Hook, HookType};

/// Thread-safe registry for hook callbacks
///
/// The registry stores hooks by type and ensures thread-safe
/// concurrent access for registration and lookup operations.
///
/// # Thread Safety
///
/// - Uses `Arc<RwLock<>>` for interior mutability
/// - Read operations (lookup) acquire read lock
/// - Write operations (register) acquire write lock
/// - Multiple readers can access concurrently
///
/// # Example
///
/// ```rust
/// use cco::daemon::hooks::{HookRegistry, HookType, HookPayload};
/// use std::sync::Arc;
///
/// let registry = Arc::new(HookRegistry::new());
///
/// // Register a hook
/// registry.register(HookType::PreCommand, Box::new(|payload| {
///     println!("Command: {}", payload.command);
///     Ok(())
/// })).unwrap();
///
/// // Get hooks for execution
/// let hooks = registry.get_hooks(HookType::PreCommand);
/// assert_eq!(hooks.len(), 1);
/// ```
#[derive(Clone)]
pub struct HookRegistry {
    /// Map of hook type to list of hook callbacks
    hooks: Arc<RwLock<HashMap<HookType, Vec<Arc<dyn Hook>>>>>,
}

impl HookRegistry {
    /// Create a new empty hook registry
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::HookRegistry;
    ///
    /// let registry = HookRegistry::new();
    /// ```
    pub fn new() -> Self {
        debug!("Creating new hook registry");
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hook callback for a specific hook type
    ///
    /// Multiple hooks can be registered for the same type.
    /// Hooks are executed in registration order.
    ///
    /// # Arguments
    ///
    /// * `hook_type` - The type of hook to register
    /// * `callback` - The hook callback implementation
    ///
    /// # Errors
    ///
    /// Returns `HookError::RegistrationFailed` if the registry lock is poisoned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookType};
    ///
    /// let registry = HookRegistry::new();
    /// registry.register(HookType::PreCommand, Box::new(|payload| {
    ///     // Hook implementation
    ///     Ok(())
    /// })).unwrap();
    /// ```
    pub fn register(&self, hook_type: HookType, callback: Box<dyn Hook>) -> HookResult<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HookError::registration_failed(format!("Lock poisoned: {}", e))
        })?;

        let hook_list = hooks.entry(hook_type).or_insert_with(Vec::new);
        hook_list.push(Arc::from(callback));

        info!(
            hook_type = %hook_type,
            count = hook_list.len(),
            "Registered hook"
        );

        Ok(())
    }

    /// Get all registered hooks for a specific type
    ///
    /// Returns a vector of all hooks registered for the given type.
    /// If no hooks are registered, returns an empty vector.
    ///
    /// # Arguments
    ///
    /// * `hook_type` - The type of hooks to retrieve
    ///
    /// # Returns
    ///
    /// A vector of Arc-wrapped hook callbacks. The Arc ensures hooks
    /// can be safely shared across threads during execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookRegistry, HookType};
    ///
    /// let registry = HookRegistry::new();
    /// let hooks = registry.get_hooks(HookType::PreCommand);
    /// assert!(hooks.is_empty());
    /// ```
    pub fn get_hooks(&self, hook_type: HookType) -> Vec<Arc<dyn Hook>> {
        let hooks = match self.hooks.read() {
            Ok(hooks) => hooks,
            Err(e) => {
                warn!(
                    hook_type = %hook_type,
                    error = %e,
                    "Failed to acquire read lock for hook registry"
                );
                return Vec::new();
            }
        };

        hooks.get(&hook_type).cloned().unwrap_or_default()
    }

    /// Get the count of registered hooks for a specific type
    ///
    /// # Arguments
    ///
    /// * `hook_type` - The type of hooks to count
    ///
    /// # Returns
    ///
    /// The number of hooks registered for the given type.
    pub fn count(&self, hook_type: HookType) -> usize {
        let hooks = match self.hooks.read() {
            Ok(hooks) => hooks,
            Err(_) => return 0,
        };

        hooks.get(&hook_type).map(|v| v.len()).unwrap_or(0)
    }

    /// Clear all hooks for a specific type
    ///
    /// # Arguments
    ///
    /// * `hook_type` - The type of hooks to clear
    ///
    /// # Errors
    ///
    /// Returns `HookError::RegistrationFailed` if the registry lock is poisoned.
    pub fn clear(&self, hook_type: HookType) -> HookResult<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HookError::registration_failed(format!("Lock poisoned: {}", e))
        })?;

        if let Some(removed) = hooks.remove(&hook_type) {
            info!(
                hook_type = %hook_type,
                count = removed.len(),
                "Cleared hooks"
            );
        }

        Ok(())
    }

    /// Clear all hooks from the registry
    ///
    /// # Errors
    ///
    /// Returns `HookError::RegistrationFailed` if the registry lock is poisoned.
    pub fn clear_all(&self) -> HookResult<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HookError::registration_failed(format!("Lock poisoned: {}", e))
        })?;

        let total: usize = hooks.values().map(|v| v.len()).sum();
        hooks.clear();

        info!(total = total, "Cleared all hooks");
        Ok(())
    }

    /// Get the total number of registered hooks across all types
    pub fn total_count(&self) -> usize {
        let hooks = match self.hooks.read() {
            Ok(hooks) => hooks,
            Err(_) => return 0,
        };

        hooks.values().map(|v| v.len()).sum()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::hooks::HookPayload;

    #[test]
    fn test_registry_creation() {
        let registry = HookRegistry::new();
        assert_eq!(registry.total_count(), 0);
    }

    #[test]
    fn test_register_and_get_hooks() {
        let registry = HookRegistry::new();

        // Register a hook
        let hook: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
        registry.register(HookType::PreCommand, hook).unwrap();

        // Verify it's registered
        let hooks = registry.get_hooks(HookType::PreCommand);
        assert_eq!(hooks.len(), 1);
        assert_eq!(registry.count(HookType::PreCommand), 1);
    }

    #[test]
    fn test_multiple_hooks_same_type() {
        let registry = HookRegistry::new();

        // Register multiple hooks
        for _ in 0..3 {
            let hook: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
            registry.register(HookType::PreCommand, hook).unwrap();
        }

        assert_eq!(registry.count(HookType::PreCommand), 3);
    }

    #[test]
    fn test_different_hook_types() {
        let registry = HookRegistry::new();

        // Register hooks for different types
        let hook1: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
        let hook2: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });

        registry.register(HookType::PreCommand, hook1).unwrap();
        registry.register(HookType::PostCommand, hook2).unwrap();

        assert_eq!(registry.count(HookType::PreCommand), 1);
        assert_eq!(registry.count(HookType::PostCommand), 1);
        assert_eq!(registry.count(HookType::PostExecution), 0);
        assert_eq!(registry.total_count(), 2);
    }

    #[test]
    fn test_clear_hooks() {
        let registry = HookRegistry::new();

        // Register hooks
        for _ in 0..3 {
            let hook: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
            registry.register(HookType::PreCommand, hook).unwrap();
        }

        assert_eq!(registry.count(HookType::PreCommand), 3);

        // Clear hooks
        registry.clear(HookType::PreCommand).unwrap();
        assert_eq!(registry.count(HookType::PreCommand), 0);
    }

    #[test]
    fn test_clear_all_hooks() {
        let registry = HookRegistry::new();

        // Register hooks for multiple types
        for hook_type in [HookType::PreCommand, HookType::PostCommand, HookType::PostExecution] {
            for _ in 0..2 {
                let hook: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
                registry.register(hook_type, hook).unwrap();
            }
        }

        assert_eq!(registry.total_count(), 6);

        // Clear all
        registry.clear_all().unwrap();
        assert_eq!(registry.total_count(), 0);
    }

    #[test]
    fn test_hook_execution() {
        let registry = HookRegistry::new();

        // Register a hook that modifies a counter
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let hook: Box<dyn Hook> = Box::new(move |_payload: &HookPayload| -> HookResult<()> {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        registry.register(HookType::PreCommand, hook).unwrap();

        // Execute the hook
        let hooks = registry.get_hooks(HookType::PreCommand);
        let payload = HookPayload::new("test");

        for hook in hooks {
            hook.execute(&payload).unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_registry_clone() {
        let registry = HookRegistry::new();
        let hook: Box<dyn Hook> = Box::new(|_payload: &HookPayload| -> HookResult<()> { Ok(()) });
        registry.register(HookType::PreCommand, hook).unwrap();

        // Clone the registry
        let cloned = registry.clone();

        // Both should see the same hooks
        assert_eq!(registry.count(HookType::PreCommand), 1);
        assert_eq!(cloned.count(HookType::PreCommand), 1);
    }
}
