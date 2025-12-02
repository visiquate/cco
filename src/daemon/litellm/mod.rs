//! LiteLLM subprocess management
//!
//! Manages the LiteLLM proxy as a subprocess, providing:
//! - Automatic PEX extraction from embedded bytes
//! - Process lifecycle management (start/stop/restart)
//! - Health checking with retry logic
//! - Graceful shutdown handling

pub mod process;

pub use process::{LiteLLMProcess, LiteLLMConfig, ensure_litellm_pex};
