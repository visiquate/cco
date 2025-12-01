//! HTTP Proxy module for model routing
//!
//! Intercepts Claude Code traffic and routes requests based on agent type:
//! - Orchestrator/Architect agents → Claude Opus (pass-through)
//! - Reviewer/Tester agents → Azure GPT-5.1-codex-mini

pub mod router;
pub mod server;
pub mod translator;

pub use router::{should_route_to_azure, AZURE_ROUTED_AGENTS};
pub use server::start_proxy_server;
pub use translator::{translate_request_to_azure, translate_response_from_azure};
