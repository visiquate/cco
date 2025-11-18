//! LLM integration for CRUD classification
//!
//! Provides embedded TinyLLaMA-based command classification using GGML inference.
//! This module handles:
//! - Model download and caching
//! - Lazy model loading
//! - CRUD classification with timeout enforcement
//! - Memory pressure management

pub mod classifier;
pub mod model;
pub mod prompt;

pub use classifier::CrudClassifier;
pub use model::ModelManager;
pub use prompt::{build_crud_prompt, parse_classification};
