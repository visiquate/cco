//! Permission confirmation system for CRUD operations
//!
//! Provides request handling and decision making for command permissions:
//! - Auto-approve READ operations
//! - Queue CREATE/UPDATE/DELETE for user confirmation
//! - Support for auto-approve flags
//! - Timeout enforcement

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{ClassificationResult, CrudClassification};

/// Permission decision for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionDecision {
    /// Command approved for execution
    Approved,

    /// Command denied
    Denied,

    /// Awaiting user confirmation (interactive mode)
    Pending,

    /// Permission check skipped (auto-approve mode)
    Skipped,
}

impl std::fmt::Display for PermissionDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionDecision::Approved => write!(f, "APPROVED"),
            PermissionDecision::Denied => write!(f, "DENIED"),
            PermissionDecision::Pending => write!(f, "PENDING"),
            PermissionDecision::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// Permission request for command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// The command being requested
    pub command: String,

    /// CRUD classification of the command
    pub classification: CrudClassification,

    /// User making the request (optional)
    pub user: Option<String>,

    /// Timeout for user confirmation in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Timestamp of the request
    #[serde(default = "default_timestamp")]
    pub timestamp: SystemTime,
}

fn default_timeout_ms() -> u64 {
    5000 // 5 seconds default
}

fn default_timestamp() -> SystemTime {
    SystemTime::now()
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(command: impl Into<String>, classification: CrudClassification) -> Self {
        Self {
            command: command.into(),
            classification,
            user: None,
            timeout_ms: default_timeout_ms(),
            timestamp: SystemTime::now(),
        }
    }

    /// Set the user for this request
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set the timeout for this request
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Check if this request is safe (READ operation)
    pub fn is_safe(&self) -> bool {
        matches!(self.classification, CrudClassification::Read)
    }
}

/// Permission response with decision and reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    /// The permission decision
    pub decision: PermissionDecision,

    /// Reasoning for the decision
    pub reasoning: String,

    /// Timestamp of the decision
    pub timestamp: String,

    /// Classification confidence score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

impl PermissionResponse {
    /// Create a new permission response
    pub fn new(
        decision: PermissionDecision,
        reasoning: impl Into<String>,
    ) -> Self {
        Self {
            decision,
            reasoning: reasoning.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            confidence: None,
        }
    }

    /// Add confidence score to the response
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

/// Configuration for permission handling
#[derive(Debug, Clone)]
pub struct PermissionConfig {
    /// Skip confirmations and auto-approve all commands (dangerous!)
    pub dangerously_skip_confirmations: bool,

    /// Default timeout for user confirmation in milliseconds
    pub default_timeout_ms: u64,

    /// Whether to allow READ operations by default
    pub auto_approve_read: bool,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            dangerously_skip_confirmations: false,
            default_timeout_ms: 5000,
            auto_approve_read: true,
        }
    }
}

/// Handler for permission requests
///
/// Processes permission requests based on CRUD classification:
/// - READ: Auto-approve (safe operations)
/// - CREATE/UPDATE/DELETE: Queue for user confirmation or auto-approve if configured
///
/// # Thread Safety
///
/// The handler is thread-safe and can be shared across tasks using `Arc`.
#[derive(Clone)]
pub struct PermissionHandler {
    /// Configuration for permission handling
    config: Arc<RwLock<PermissionConfig>>,
}

impl PermissionHandler {
    /// Create a new permission handler with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(PermissionConfig::default())),
        }
    }

    /// Create a new permission handler with custom configuration
    pub fn with_config(config: PermissionConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Process a permission request
    ///
    /// # Arguments
    ///
    /// * `request` - The permission request to process
    ///
    /// # Returns
    ///
    /// A `PermissionResponse` with the decision and reasoning
    ///
    /// # Behavior
    ///
    /// - READ operations: Immediately approved
    /// - CREATE/UPDATE/DELETE with dangerously_skip_confirmations: Auto-approved with warning
    /// - CREATE/UPDATE/DELETE in interactive mode: Returns PENDING (waiting for user)
    pub async fn process_request(&self, request: PermissionRequest) -> PermissionResponse {
        let config = self.config.read().await;

        debug!(
            "Processing permission request: {} (classification: {})",
            request.command, request.classification
        );

        // Auto-approve READ operations
        if request.is_safe() && config.auto_approve_read {
            info!("Auto-approving READ operation: {}", request.command);
            return PermissionResponse::new(
                PermissionDecision::Approved,
                "READ operation - safe to execute",
            );
        }

        // Check if confirmations are skipped
        if config.dangerously_skip_confirmations {
            warn!(
                "⚠️  Auto-approving {} operation (confirmations disabled): {}",
                request.classification, request.command
            );
            return PermissionResponse::new(
                PermissionDecision::Skipped,
                format!(
                    "{} operation - auto-approved (dangerously-skip-confirmations enabled)",
                    request.classification
                ),
            );
        }

        // For CREATE/UPDATE/DELETE in interactive mode, return PENDING
        info!(
            "Pending user confirmation for {} operation: {}",
            request.classification, request.command
        );
        PermissionResponse::new(
            PermissionDecision::Pending,
            format!(
                "{} operation requires user confirmation",
                request.classification
            ),
        )
    }

    /// Process a permission request from a classification result
    ///
    /// Convenience method that creates a `PermissionRequest` from a
    /// `ClassificationResult` and processes it.
    pub async fn process_classification(
        &self,
        command: impl Into<String>,
        classification: ClassificationResult,
    ) -> PermissionResponse {
        let request = PermissionRequest::new(command, classification.classification);

        let mut response = self.process_request(request).await;
        response.confidence = Some(classification.confidence);
        response
    }

    /// Update the configuration
    pub async fn update_config(&self, config: PermissionConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
        info!("Permission configuration updated");
    }

    /// Enable or disable confirmation skipping
    pub async fn set_skip_confirmations(&self, skip: bool) {
        let mut config = self.config.write().await;
        config.dangerously_skip_confirmations = skip;

        if skip {
            warn!("⚠️  Confirmation skipping ENABLED - all commands will be auto-approved");
        } else {
            info!("✅ Confirmation skipping DISABLED - unsafe commands require approval");
        }
    }

    /// Get current configuration snapshot
    pub async fn get_config(&self) -> PermissionConfig {
        self.config.read().await.clone()
    }
}

impl Default for PermissionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_request_creation() {
        let request = PermissionRequest::new("ls -la", CrudClassification::Read);
        assert_eq!(request.command, "ls -la");
        assert_eq!(request.classification, CrudClassification::Read);
        assert!(request.is_safe());
        assert_eq!(request.timeout_ms, 5000);
    }

    #[test]
    fn test_permission_request_with_user() {
        let request = PermissionRequest::new("rm file.txt", CrudClassification::Delete)
            .with_user("testuser")
            .with_timeout(3000);

        assert_eq!(request.user, Some("testuser".to_string()));
        assert_eq!(request.timeout_ms, 3000);
        assert!(!request.is_safe());
    }

    #[test]
    fn test_permission_response_creation() {
        let response = PermissionResponse::new(
            PermissionDecision::Approved,
            "Safe operation"
        ).with_confidence(0.95);

        assert_eq!(response.decision, PermissionDecision::Approved);
        assert_eq!(response.reasoning, "Safe operation");
        assert_eq!(response.confidence, Some(0.95));
    }

    #[test]
    fn test_permission_decision_display() {
        assert_eq!(PermissionDecision::Approved.to_string(), "APPROVED");
        assert_eq!(PermissionDecision::Denied.to_string(), "DENIED");
        assert_eq!(PermissionDecision::Pending.to_string(), "PENDING");
        assert_eq!(PermissionDecision::Skipped.to_string(), "SKIPPED");
    }

    #[tokio::test]
    async fn test_handler_auto_approve_read() {
        let handler = PermissionHandler::new();
        let request = PermissionRequest::new("ls -la", CrudClassification::Read);

        let response = handler.process_request(request).await;
        assert_eq!(response.decision, PermissionDecision::Approved);
    }

    #[tokio::test]
    async fn test_handler_pending_for_unsafe() {
        let handler = PermissionHandler::new();
        let request = PermissionRequest::new("rm file.txt", CrudClassification::Delete);

        let response = handler.process_request(request).await;
        assert_eq!(response.decision, PermissionDecision::Pending);
    }

    #[tokio::test]
    async fn test_handler_skip_confirmations() {
        let config = PermissionConfig {
            dangerously_skip_confirmations: true,
            ..Default::default()
        };
        let handler = PermissionHandler::with_config(config);

        let request = PermissionRequest::new("rm -rf /", CrudClassification::Delete);
        let response = handler.process_request(request).await;

        assert_eq!(response.decision, PermissionDecision::Skipped);
    }

    #[tokio::test]
    async fn test_handler_config_update() {
        let handler = PermissionHandler::new();

        // Initially confirmations are required
        assert!(!handler.get_config().await.dangerously_skip_confirmations);

        // Enable skip confirmations
        handler.set_skip_confirmations(true).await;
        assert!(handler.get_config().await.dangerously_skip_confirmations);

        // Disable again
        handler.set_skip_confirmations(false).await;
        assert!(!handler.get_config().await.dangerously_skip_confirmations);
    }

    #[tokio::test]
    async fn test_process_classification() {
        let handler = PermissionHandler::new();
        let classification = ClassificationResult::new(CrudClassification::Read, 0.95);

        let response = handler.process_classification("git status", classification).await;

        assert_eq!(response.decision, PermissionDecision::Approved);
        assert_eq!(response.confidence, Some(0.95));
    }
}
