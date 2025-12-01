//! Orchestration Sidecar Module
//!
//! Autonomous HTTP server for coordinating 119 Claude Orchestra agents.
//! Provides context injection, event coordination, result storage, and
//! multi-round agent interactions.

pub mod cli;
pub mod context_injector;
pub mod event_bus;
pub mod knowledge_broker;
pub mod result_storage;
pub mod server;

// Re-export main types
pub use cli::OrchestrationCli;
pub use context_injector::ContextInjector;
pub use event_bus::EventBus;
pub use knowledge_broker::KnowledgeBroker;
pub use result_storage::ResultStorage;
pub use server::{OrchestrationServer, ServerConfig};

use anyhow::Result;
use std::sync::Arc;

/// Initialize the orchestration sidecar system
pub async fn initialize(config: ServerConfig) -> Result<Arc<OrchestrationState>> {
    // Create shared state
    let state = Arc::new(OrchestrationState {
        config: config.clone(),
        knowledge_broker: KnowledgeBroker::new(),
        event_bus: EventBus::new(),
        result_storage: ResultStorage::new(config.storage_path.clone())?,
        context_injector: ContextInjector::new(),
    });

    Ok(state)
}

/// Shared state for the orchestration sidecar
pub struct OrchestrationState {
    pub config: ServerConfig,
    pub knowledge_broker: KnowledgeBroker,
    pub event_bus: EventBus,
    pub result_storage: ResultStorage,
    pub context_injector: ContextInjector,
}

/// Lifecycle management for the sidecar
pub struct SidecarLifecycle {
    state: Arc<OrchestrationState>,
    server_handle: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl SidecarLifecycle {
    /// Create a new sidecar lifecycle manager
    pub fn new(state: Arc<OrchestrationState>) -> Self {
        Self {
            state,
            server_handle: None,
        }
    }

    /// Start the sidecar server
    pub async fn start(&mut self) -> Result<()> {
        let state = Arc::clone(&self.state);
        let config = state.config.clone();

        let handle = tokio::spawn(async move {
            let server = OrchestrationServer::new(state, config);
            server.run().await
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    /// Stop the sidecar server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Check if the sidecar is running
    pub fn is_running(&self) -> bool {
        self.server_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }
}
