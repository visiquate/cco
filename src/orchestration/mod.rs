//! Orchestration Module
//!
//! Provides context injection, event coordination, result storage, and
//! multi-round agent interactions for the 119 Claude Orchestra agents.
//!
//! Note: The HTTP server functionality has been merged into the main daemon.
//! Orchestration API endpoints are now available at /api/orchestration/* routes.

pub mod cli;
pub mod context_injector;
pub mod event_bus;
pub mod knowledge_broker;
pub mod result_storage;

// Re-export main types
pub use cli::OrchestrationCli;
pub use context_injector::ContextInjector;
pub use event_bus::EventBus;
pub use knowledge_broker::KnowledgeBroker;
pub use result_storage::ResultStorage;

/// Server configuration (used by daemon for orchestration initialization)
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub storage_path: String,
    pub context_cache_ttl_secs: u64,
    pub max_context_size_bytes: usize,
    pub event_retention_hours: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 0, // Not used - orchestration uses daemon's port
            host: "127.0.0.1".to_string(),
            storage_path: dirs::home_dir()
                .unwrap_or_default()
                .join(".cco/orchestration")
                .to_string_lossy()
                .to_string(),
            context_cache_ttl_secs: 3600,
            max_context_size_bytes: 1024 * 1024, // 1MB
            event_retention_hours: 24,
        }
    }
}

/// Shared state for the orchestration system
pub struct OrchestrationState {
    pub config: ServerConfig,
    pub knowledge_broker: KnowledgeBroker,
    pub event_bus: EventBus,
    pub result_storage: ResultStorage,
    pub context_injector: ContextInjector,
}

/// Event data returned from event bus queries
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventData {
    pub event_id: String,
    pub event_type: String,
    pub publisher: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
