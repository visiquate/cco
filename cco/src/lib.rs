//! CCO - Claude Code Orchestra
//!
//! A multi-agent development system with intelligent caching and cost tracking

pub mod agents_config;
pub mod analytics;
pub mod cache;
pub mod claude_history;
pub mod embedded_agents;
pub mod proxy;
pub mod router;
pub mod security;
pub mod server;
pub mod terminal;
pub mod version;

pub use analytics::{AnalyticsEngine, ApiCallRecord, ModelMetrics};
pub use cache::{CachedResponse, MokaCache};
pub use claude_history::{load_claude_project_metrics, ClaudeMetrics, ModelBreakdown};
pub use proxy::{ChatRequest, ChatResponse, Message, ProxyServer};
pub use router::{ModelPricing, ModelRouter, ProviderType, RouteRule, RouterConfig};
pub use server::{run_server, ServerState};
pub use terminal::TerminalSession;
pub use version::DateVersion;
