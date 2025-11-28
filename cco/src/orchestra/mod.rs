//! Orchestra module for agent configuration parsing and conversion
//!
//! This module provides functionality to parse the orchestra-config.json file
//! and convert it to Claude Code's agent format.

pub mod config_parser;

pub use config_parser::{OrchestraConfig, Agent, ClaudeAgent};
