//! Hooks system for daemon lifecycle and command execution
//!
//! Provides a flexible, extensible hooks system with three hook types:
//! - PreCommand: Execute before command classification
//! - PostCommand: Execute after command is classified
//! - PostExecution: Execute after command completes
//!
//! # Architecture
//!
//! The hooks system consists of:
//! - **HookRegistry**: Thread-safe registry for managing hook callbacks
//! - **HookExecutor**: Async executor with timeout and retry support
//! - **HookConfig**: Configuration for hook behavior and permissions
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use cco::daemon::hooks::{HookRegistry, HookExecutor, HookType, HookPayload};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create registry and executor
//! let registry = Arc::new(HookRegistry::new());
//! let executor = HookExecutor::new(registry.clone());
//!
//! // Register a pre-command hook
//! registry.register(HookType::PreCommand, Box::new(|payload| {
//!     println!("Command: {}", payload.command);
//!     Ok(())
//! }))?;
//!
//! // Execute hook
//! let payload = HookPayload::new("test command");
//! executor.execute_hook(HookType::PreCommand, payload).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Thread Safety
//!
//! All components are designed for concurrent use:
//! - HookRegistry uses `Arc<RwLock<>>` for thread-safe registration/lookup
//! - HookExecutor is clone-able and shareable across async tasks
//! - All hook executions are independent and non-blocking

pub mod audit;
pub mod config;
pub mod error;
pub mod executor;
pub mod lifecycle;
pub mod llm;
pub mod permissions;
pub mod registry;
pub mod types;

// Re-export core types for convenience
pub use audit::{Decision, DecisionDatabase, DecisionStats, SqliteAuditDatabase};
pub use config::{HookLlmConfig, HooksCallbacks, HooksConfig, HooksPermissions};
pub use error::{HookError, HookResult};
pub use executor::HookExecutor;
pub use lifecycle::execute_lifecycle_hook;
pub use llm::CrudClassifier;
pub use permissions::{
    PermissionConfig, PermissionDecision, PermissionHandler, PermissionRequest, PermissionResponse,
};
pub use registry::HookRegistry;
pub use types::{
    ClassificationResult, CrudClassification, Hook, HookContext, HookPayload, HookType,
};
