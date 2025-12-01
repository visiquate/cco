//! CCO - Claude Code Orchestra
//!
//! A multi-agent development system with intelligent caching and cost tracking

pub mod agents_config;
pub mod analytics;
pub mod api_client;
pub mod auth;
pub mod auto_update;
pub mod cache;
pub mod claude_history;
pub mod credentials;
pub mod daemon;
pub mod embedded_agents;
pub mod metrics;
pub mod monitor;
pub mod orchestra;
pub mod orchestration;
pub mod persistence;
pub mod proxy;
pub mod router;
pub mod security;
pub mod server;
pub mod sse;
pub mod terminal;
pub mod tui;
pub mod tui_app;
pub mod version;

pub use analytics::{AnalyticsEngine, ApiCallRecord, ModelMetrics};
pub use api_client::{Agent, ApiClient, HealthResponse, Stats};
pub use cache::{CachedResponse, MokaCache};
pub use claude_history::{load_claude_project_metrics, ClaudeMetrics, ModelBreakdown};
pub use daemon::{DaemonConfig, DaemonManager, DaemonStatus};
pub use metrics::{ApiCallEvent, MetricsEngine, MetricsSummary, ModelTier, TokenBreakdown};
pub use monitor::{MonitorConfig, MonitorService};
pub use proxy::{ChatRequest, ChatResponse, Message, ProxyServer};
pub use router::{ModelPricing, ModelRouter, ProviderType, RouteRule, RouterConfig};
pub use server::{run_server, ServerState};
pub use sse::SseClient;
pub use terminal::TerminalSession;
pub use tui_app::TuiApp;
pub use version::DateVersion;
