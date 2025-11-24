//! Core types for the hooks system
//!
//! Defines the fundamental types used throughout the hooks infrastructure:
//! - Hook types (PreCommand, PostCommand, PostExecution)
//! - Hook payloads (data passed to hooks)
//! - Hook callbacks (trait for hook implementations)
//! - CRUD classification types for command analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use super::error::HookResult;

/// CRUD operation classification
///
/// Classifies commands into four fundamental operation types for
/// permission gating and safety enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrudClassification {
    /// Read operations - retrieve/display data with no side effects
    ///
    /// Examples: ls, cat, git status, ps, grep
    Read,

    /// Create operations - make new resources, files, processes
    ///
    /// Examples: touch, mkdir, git init, docker run
    Create,

    /// Update operations - modify existing resources
    ///
    /// Examples: echo >>, sed -i, git commit, chmod
    Update,

    /// Delete operations - remove resources
    ///
    /// Examples: rm, rmdir, docker rm, git branch -d
    Delete,
}

impl fmt::Display for CrudClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrudClassification::Read => write!(f, "READ"),
            CrudClassification::Create => write!(f, "CREATE"),
            CrudClassification::Update => write!(f, "UPDATE"),
            CrudClassification::Delete => write!(f, "DELETE"),
        }
    }
}

impl std::str::FromStr for CrudClassification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "READ" => Ok(CrudClassification::Read),
            "CREATE" => Ok(CrudClassification::Create),
            "UPDATE" => Ok(CrudClassification::Update),
            "DELETE" => Ok(CrudClassification::Delete),
            _ => Err(format!("Invalid CRUD classification: {}", s)),
        }
    }
}

/// Classification result with confidence and reasoning
///
/// Returned by the LLM classifier with metadata about the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// The CRUD classification
    pub classification: CrudClassification,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Reasoning for the classification (optional, for debugging)
    pub reasoning: Option<String>,

    /// Timestamp when classification was performed
    pub timestamp: SystemTime,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(classification: CrudClassification, confidence: f32) -> Self {
        Self {
            classification,
            confidence,
            reasoning: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a result with reasoning
    pub fn with_reasoning(
        classification: CrudClassification,
        confidence: f32,
        reasoning: impl Into<String>,
    ) -> Self {
        Self {
            classification,
            confidence,
            reasoning: Some(reasoning.into()),
            timestamp: SystemTime::now(),
        }
    }

    /// Check if this is a safe (READ) operation
    pub fn is_safe(&self) -> bool {
        matches!(self.classification, CrudClassification::Read)
    }

    /// Check if this requires confirmation (CREATE/UPDATE/DELETE)
    pub fn requires_confirmation(&self) -> bool {
        !self.is_safe()
    }
}

/// Types of hooks supported by the system
///
/// Each hook type fires at a different point in the command lifecycle:
/// - **PreCommand**: Before command classification (can modify command)
/// - **PostCommand**: After classification, before execution (can validate/log)
/// - **PostExecution**: After command completes (can log results/cleanup)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    /// Executed before command classification
    ///
    /// Use cases:
    /// - Command preprocessing/normalization
    /// - Security checks
    /// - Rate limiting
    PreCommand,

    /// Executed after command is classified but before execution
    ///
    /// Use cases:
    /// - Validation based on classification
    /// - Permission checks
    /// - Logging/analytics
    PostCommand,

    /// Executed after command execution completes
    ///
    /// Use cases:
    /// - Result logging
    /// - Cleanup operations
    /// - Metrics collection
    PostExecution,
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookType::PreCommand => write!(f, "pre_command"),
            HookType::PostCommand => write!(f, "post_command"),
            HookType::PostExecution => write!(f, "post_execution"),
        }
    }
}

/// Context information for hook execution
///
/// Provides additional context about the environment in which
/// the hook is being executed. Currently minimal but designed
/// for future extensibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookContext {
    /// Standard daemon context with optional metadata
    Daemon {
        /// Additional key-value metadata
        metadata: HashMap<String, String>,
    },

    /// Test context for unit/integration testing
    Test {
        /// Test identifier
        test_id: String,
    },
}

impl Default for HookContext {
    fn default() -> Self {
        HookContext::Daemon {
            metadata: HashMap::new(),
        }
    }
}

/// Payload passed to hook callbacks
///
/// Contains all information needed by hooks to process
/// commands and make decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    /// The command being processed
    pub command: String,

    /// CRUD classification (available after classification)
    pub classification: Option<ClassificationResult>,

    /// Execution result (available for PostExecution hooks only)
    pub execution_result: Option<String>,

    /// Execution context
    pub context: HookContext,

    /// Timestamp when payload was created
    pub timestamp: SystemTime,

    /// Additional metadata (extensible)
    pub metadata: HashMap<String, String>,
}

impl HookPayload {
    /// Create a new payload with the given command
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::HookPayload;
    ///
    /// let payload = HookPayload::new("git status");
    /// assert_eq!(payload.command, "git status");
    /// ```
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            classification: None,
            execution_result: None,
            context: HookContext::default(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create a payload with CRUD classification
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookPayload, CrudClassification, ClassificationResult};
    ///
    /// let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    /// let payload = HookPayload::with_classification("git status", classification);
    /// assert!(payload.classification.is_some());
    /// ```
    pub fn with_classification(
        command: impl Into<String>,
        classification: ClassificationResult,
    ) -> Self {
        Self {
            command: command.into(),
            classification: Some(classification),
            execution_result: None,
            context: HookContext::default(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create a payload with execution result
    ///
    /// # Example
    ///
    /// ```rust
    /// use cco::daemon::hooks::{HookPayload, CrudClassification, ClassificationResult};
    ///
    /// let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
    /// let payload = HookPayload::with_execution("git status", classification, "success");
    /// assert!(payload.execution_result.is_some());
    /// ```
    pub fn with_execution(
        command: impl Into<String>,
        classification: ClassificationResult,
        execution: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            classification: Some(classification),
            execution_result: Some(execution.into()),
            context: HookContext::default(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set the execution context
    pub fn with_context(mut self, context: HookContext) -> Self {
        self.context = context;
        self
    }

    /// Add metadata to the payload
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Hook callback trait
///
/// Hooks are synchronous closures that process payloads.
/// For async operations, use spawn within the hook.
///
/// # Safety
///
/// Hooks must:
/// - Not panic (use Result for error handling)
/// - Complete within the configured timeout
/// - Be thread-safe (Send + Sync)
pub trait Hook: Send + Sync {
    /// Execute the hook with the given payload
    fn execute(&self, payload: &HookPayload) -> HookResult<()>;
}

/// Blanket implementation for closures
impl<F> Hook for F
where
    F: for<'a> Fn(&'a HookPayload) -> HookResult<()> + Send + Sync,
{
    fn execute(&self, payload: &HookPayload) -> HookResult<()> {
        self(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_crud_classification_display() {
        assert_eq!(CrudClassification::Read.to_string(), "READ");
        assert_eq!(CrudClassification::Create.to_string(), "CREATE");
        assert_eq!(CrudClassification::Update.to_string(), "UPDATE");
        assert_eq!(CrudClassification::Delete.to_string(), "DELETE");
    }

    #[test]
    fn test_crud_classification_from_str() {
        assert_eq!(
            CrudClassification::from_str("READ").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            CrudClassification::from_str("read").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            CrudClassification::from_str("  CREATE  ").unwrap(),
            CrudClassification::Create
        );
        assert!(CrudClassification::from_str("INVALID").is_err());
    }

    #[test]
    fn test_classification_result() {
        let result = ClassificationResult::new(CrudClassification::Read, 0.95);
        assert_eq!(result.classification, CrudClassification::Read);
        assert_eq!(result.confidence, 0.95);
        assert!(result.is_safe());
        assert!(!result.requires_confirmation());

        let result = ClassificationResult::new(CrudClassification::Delete, 0.90);
        assert!(!result.is_safe());
        assert!(result.requires_confirmation());
    }

    #[test]
    fn test_classification_result_with_reasoning() {
        let result = ClassificationResult::with_reasoning(
            CrudClassification::Read,
            0.95,
            "Command only reads files",
        );
        assert_eq!(result.reasoning, Some("Command only reads files".to_string()));
    }

    #[test]
    fn test_hook_type_display() {
        assert_eq!(HookType::PreCommand.to_string(), "pre_command");
        assert_eq!(HookType::PostCommand.to_string(), "post_command");
        assert_eq!(HookType::PostExecution.to_string(), "post_execution");
    }

    #[test]
    fn test_hook_payload_new() {
        let payload = HookPayload::new("test");
        assert_eq!(payload.command, "test");
        assert!(payload.classification.is_none());
        assert_eq!(payload.execution_result, None);
    }

    #[test]
    fn test_hook_payload_with_classification() {
        let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
        let payload = HookPayload::with_classification("test", classification.clone());
        assert_eq!(payload.command, "test");
        assert!(payload.classification.is_some());
        assert_eq!(
            payload.classification.unwrap().classification,
            CrudClassification::Read
        );
        assert_eq!(payload.execution_result, None);
    }

    #[test]
    fn test_hook_payload_with_execution() {
        let classification = ClassificationResult::new(CrudClassification::Read, 0.95);
        let payload = HookPayload::with_execution("test", classification, "success");
        assert_eq!(payload.command, "test");
        assert!(payload.classification.is_some());
        assert_eq!(payload.execution_result, Some("success".to_string()));
    }

    #[test]
    fn test_hook_payload_metadata() {
        let payload = HookPayload::new("test")
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(payload.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(payload.get_metadata("key2"), Some(&"value2".to_string()));
        assert_eq!(payload.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_hook_context_default() {
        let context = HookContext::default();
        match context {
            HookContext::Daemon { metadata } => assert!(metadata.is_empty()),
            _ => panic!("Expected Daemon context"),
        }
    }

    #[test]
    fn test_hook_closure_implementation() {
        let hook: Box<dyn Hook> = Box::new(|payload: &HookPayload| -> HookResult<()> {
            assert_eq!(payload.command, "test");
            Ok(())
        });

        let payload = HookPayload::new("test");
        assert!(hook.execute(&payload).is_ok());
    }
}
