//! Knowledge Broker Component
//!
//! Auto-discovers agent context from files, gathers relevant knowledge,
//! includes project structure and previous agent outputs.
//! Features in-memory LRU cache with 1GB limit.

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_CACHE_SIZE_BYTES: usize = 1024 * 1024 * 1024; // 1GB
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

/// Cached context entry
#[derive(Debug, Clone)]
struct CachedContext {
    context: Context,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u32,
    size_bytes: usize,
}

/// Knowledge broker for gathering and caching context
pub struct KnowledgeBroker {
    cache: Arc<DashMap<String, CachedContext>>,
    total_cache_size: Arc<std::sync::atomic::AtomicUsize>,
}

impl KnowledgeBroker {
    /// Create a new knowledge broker
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            total_cache_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Gather context for an agent
    pub async fn gather_context(&self, agent_type: &str, issue_id: &str) -> Result<Context> {
        let cache_key = format!("{}:{}", issue_id, agent_type);

        // Check cache first
        if let Some(mut cached) = self.cache.get_mut(&cache_key) {
            // Check if cache is still valid
            if cached.created_at.elapsed() < CACHE_TTL {
                cached.last_accessed = Instant::now();
                cached.access_count += 1;
                return Ok(cached.context.clone());
            }
        }

        // Cache miss or expired - gather new context
        let context = self.gather_fresh_context(agent_type, issue_id).await?;

        // Calculate size and store in cache
        let size = estimate_context_size(&context);
        self.cache_context(cache_key, context.clone(), size);

        Ok(context)
    }

    /// Gather fresh context (cache miss)
    async fn gather_fresh_context(&self, agent_type: &str, issue_id: &str) -> Result<Context> {
        let mut context = Context::default();

        // Add project structure
        context.project_structure = self.gather_project_structure().await?;

        // Add relevant files based on agent type
        context.relevant_files = self.gather_relevant_files(agent_type).await?;

        // Add git context
        context.git_context = self.gather_git_context().await?;

        // Add previous agent outputs
        context.previous_agent_outputs = self.gather_previous_outputs(issue_id).await?;

        // Add metadata
        context.metadata = self.gather_metadata().await?;

        Ok(context)
    }

    /// Gather project structure
    async fn gather_project_structure(&self) -> Result<ProjectStructure> {
        // TODO: Implement actual project scanning
        Ok(ProjectStructure {
            root: PathBuf::from("."),
            files: vec![],
            directories: vec![],
        })
    }

    /// Gather relevant files based on agent type
    async fn gather_relevant_files(&self, _agent_type: &str) -> Result<Vec<FileInfo>> {
        // TODO: Implement file gathering based on agent type
        Ok(vec![])
    }

    /// Gather git context
    async fn gather_git_context(&self) -> Result<GitContext> {
        // TODO: Implement git context gathering
        Ok(GitContext {
            branch: String::from("main"),
            recent_commits: vec![],
            uncommitted_changes: vec![],
        })
    }

    /// Gather previous agent outputs for an issue
    async fn gather_previous_outputs(&self, issue_id: &str) -> Result<Vec<AgentOutput>> {
        // TODO: Implement previous output gathering
        Ok(vec![])
    }

    /// Gather project metadata
    async fn gather_metadata(&self) -> Result<ProjectMetadata> {
        // TODO: Implement metadata gathering
        Ok(ProjectMetadata {
            project_type: String::from("unknown"),
            dependencies: vec![],
            test_coverage: None,
            last_build_status: None,
        })
    }

    /// Cache context entry
    fn cache_context(&self, key: String, context: Context, size: usize) {
        // Check if adding this would exceed cache limit
        let current_size = self
            .total_cache_size
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_size + size > MAX_CACHE_SIZE_BYTES {
            self.evict_lru_entries(size);
        }

        let cached = CachedContext {
            context,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            size_bytes: size,
        };

        self.cache.insert(key, cached);
        self.total_cache_size
            .fetch_add(size, std::sync::atomic::Ordering::Relaxed);
    }

    /// Evict least recently used entries to make room
    fn evict_lru_entries(&self, needed_space: usize) {
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().last_accessed,
                    entry.value().size_bytes,
                )
            })
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1));

        let mut freed_space = 0;
        for (key, _, _size) in entries {
            if freed_space >= needed_space {
                break;
            }

            if let Some((_, cached)) = self.cache.remove(&key) {
                freed_space += cached.size_bytes;
                self.total_cache_size
                    .fetch_sub(cached.size_bytes, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Clear cache for a specific issue
    pub async fn clear_cache(&self, issue_id: &str) -> Result<usize> {
        let mut removed = 0;
        let prefix = format!("{}:", issue_id);

        let keys_to_remove: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            if let Some((_, cached)) = self.cache.remove(&key) {
                self.total_cache_size
                    .fetch_sub(cached.size_bytes, std::sync::atomic::Ordering::Relaxed);
                removed += 1;
            }
        }

        Ok(removed)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            total_size_bytes: self
                .total_cache_size
                .load(std::sync::atomic::Ordering::Relaxed),
            max_size_bytes: MAX_CACHE_SIZE_BYTES,
        }
    }
}

/// Context gathered for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub project_structure: ProjectStructure,
    pub relevant_files: Vec<FileInfo>,
    pub git_context: GitContext,
    pub previous_agent_outputs: Vec<AgentOutput>,
    pub metadata: ProjectMetadata,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            project_structure: ProjectStructure {
                root: PathBuf::from("."),
                files: vec![],
                directories: vec![],
            },
            relevant_files: vec![],
            git_context: GitContext {
                branch: String::from("main"),
                recent_commits: vec![],
                uncommitted_changes: vec![],
            },
            previous_agent_outputs: vec![],
            metadata: ProjectMetadata {
                project_type: String::from("unknown"),
                dependencies: vec![],
                test_coverage: None,
                last_build_status: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root: PathBuf,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub content: String,
    pub last_modified: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub branch: String,
    pub recent_commits: Vec<CommitInfo>,
    pub uncommitted_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent: String,
    pub timestamp: String,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_type: String,
    pub dependencies: Vec<String>,
    pub test_coverage: Option<f64>,
    pub last_build_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size_bytes: usize,
    pub max_size_bytes: usize,
}

/// Estimate the size of a context in bytes
fn estimate_context_size(context: &Context) -> usize {
    // Rough estimation based on serialized JSON size
    serde_json::to_string(context)
        .map(|s| s.len())
        .unwrap_or(1024)
}
