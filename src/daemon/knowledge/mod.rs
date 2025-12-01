//! Knowledge Store module for CCO daemon
//!
//! Embedded LanceDB-based vector knowledge storage system that replicates
//! all functionality from the JavaScript knowledge-manager.js implementation.
//!
//! ## Features
//!
//! - **Vector similarity search** using SHA256-based embeddings (384 dimensions)
//! - **Per-repository isolation** via project_id filtering
//! - **Pre/post-compaction hooks** for conversation context preservation
//! - **HTTP API** for all knowledge operations
//! - **Async/await** throughout for non-blocking operations
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │          Knowledge Store API (HTTP)             │
//! ├─────────────────────────────────────────────────┤
//! │  POST /api/knowledge/store                      │
//! │  POST /api/knowledge/store-batch                │
//! │  POST /api/knowledge/search                     │
//! │  GET  /api/knowledge/project/:id                │
//! │  POST /api/knowledge/pre-compaction             │
//! │  POST /api/knowledge/post-compaction            │
//! │  GET  /api/knowledge/stats                      │
//! │  POST /api/knowledge/cleanup                    │
//! └─────────────────────────────────────────────────┘
//!            │
//!            ▼
//! ┌─────────────────────────────────────────────────┐
//! │          Knowledge Store (Rust)                 │
//! ├─────────────────────────────────────────────────┤
//! │  - store()           - Store single item        │
//! │  - store_batch()     - Batch store              │
//! │  - search()          - Vector search            │
//! │  - get_project_knowledge() - Filter by project  │
//! │  - pre_compaction()  - Extract critical info    │
//! │  - post_compaction() - Retrieve context         │
//! │  - cleanup()         - Remove old items         │
//! │  - get_stats()       - Database statistics      │
//! └─────────────────────────────────────────────────┘
//!            │
//!            ▼
//! ┌─────────────────────────────────────────────────┐
//! │          LanceDB Vector Database                │
//! ├─────────────────────────────────────────────────┤
//! │  Table: orchestra_knowledge                     │
//! │  - id: String                                   │
//! │  - vector: [f32; 384]                           │
//! │  - text: String                                 │
//! │  - type: String (decision, architecture, etc.)  │
//! │  - project_id: String (repository isolation)    │
//! │  - session_id: String                           │
//! │  - agent: String                                │
//! │  - timestamp: String (ISO8601)                  │
//! │  - metadata: String (JSON)                      │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Embedding Strategy
//!
//! The embedding generation uses SHA256 hashing for deterministic, consistent vectors:
//!
//! 1. Hash the text using SHA256 (32 bytes)
//! 2. Cycle through hash bytes to fill 384 dimensions
//! 3. Normalize each byte to [-1, 1] range: `(byte / 128.0) - 1.0`
//! 4. Same text always produces the same vector (deterministic)
//!
//! This approach prioritizes consistency over semantic similarity, ensuring
//! knowledge retrieval works reliably across compactions.
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE orchestra_knowledge (
//!   id TEXT PRIMARY KEY,              -- "type-timestamp-random"
//!   vector FLOAT32[384],               -- SHA256-based embedding
//!   text TEXT,                         -- Knowledge content
//!   type TEXT,                         -- decision|architecture|implementation|...
//!   project_id TEXT,                   -- Repository identifier
//!   session_id TEXT,                   -- Agent session
//!   agent TEXT,                        -- Agent name
//!   timestamp TEXT,                    -- ISO8601 creation time
//!   metadata TEXT                      -- JSON-serialized metadata
//! );
//! ```
//!
//! ## Usage Example
//!
//! ```no_run
//! use cco::daemon::knowledge::{KnowledgeStore, StoreKnowledgeRequest};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create and initialize store
//!     let mut store = KnowledgeStore::new(
//!         "/path/to/repo",
//!         None,  // Use default base directory
//!         None,  // Use default table name
//!     );
//!     store.initialize().await?;
//!
//!     // Store knowledge
//!     let request = StoreKnowledgeRequest {
//!         text: "We decided to use Rust for the knowledge store".to_string(),
//!         knowledge_type: Some("decision".to_string()),
//!         project_id: Some("cc-orchestra".to_string()),
//!         session_id: Some("session-1".to_string()),
//!         agent: Some("architect".to_string()),
//!         metadata: None,
//!     };
//!
//!     let response = store.store(request).await?;
//!     println!("Stored knowledge with ID: {}", response.id);
//!
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod embedding;
pub mod models;
pub mod store;

// Re-export key types
pub use api::{knowledge_router, knowledge_router_without_state, KnowledgeState};
pub use embedding::{generate_embedding, EMBEDDING_DIM};
pub use models::*;
pub use store::KnowledgeStore;
