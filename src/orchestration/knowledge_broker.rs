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
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let ignore_dirs: std::collections::HashSet<&str> = [
            ".git",
            "node_modules",
            "target",
            "dist",
            "build",
            ".venv",
            "__pycache__",
            ".swarm",
            ".next",
            "vendor",
        ]
        .iter()
        .copied()
        .collect();

        let (files, directories) = self.scan_directory_recursive(&root, &root, &ignore_dirs)?;

        Ok(ProjectStructure {
            root: root.clone(),
            files: files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            directories: directories
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        })
    }

    /// Recursively scan directory
    fn scan_directory_recursive(
        &self,
        root: &PathBuf,
        current: &PathBuf,
        ignore_dirs: &std::collections::HashSet<&str>,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        use std::fs;

        let mut files = Vec::new();
        let mut directories = Vec::new();

        if !current.is_dir() {
            return Ok((files, directories));
        }

        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !ignore_dirs.contains(name) {
                        if let Ok(relative) = path.strip_prefix(root) {
                            directories.push(relative.to_path_buf());
                        }
                        // Recurse into subdirectory
                        let (sub_files, sub_dirs) =
                            self.scan_directory_recursive(root, &path, ignore_dirs)?;
                        files.extend(sub_files);
                        directories.extend(sub_dirs);
                    }
                }
            } else if path.is_file() {
                if let Ok(relative) = path.strip_prefix(root) {
                    files.push(relative.to_path_buf());
                }
            }
        }

        Ok((files, directories))
    }

    /// Gather relevant files based on agent type
    async fn gather_relevant_files(&self, agent_type: &str) -> Result<Vec<FileInfo>> {
        use std::fs;

        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Get file extensions relevant to this agent type
        let extensions = self.get_relevant_extensions(agent_type);

        let mut relevant_files = Vec::new();
        let project_structure = self.gather_project_structure().await?;

        for file_path_str in &project_structure.files {
            let file_path = PathBuf::from(file_path_str);

            // Check if file has relevant extension
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    let full_path = root.join(&file_path);

                    // Check file size (max 1MB)
                    if let Ok(metadata) = fs::metadata(&full_path) {
                        let size = metadata.len() as usize;
                        if size > 1024 * 1024 {
                            continue; // Skip files larger than 1MB
                        }

                        // Read file content
                        if let Ok(content) = fs::read_to_string(&full_path) {
                            relevant_files.push(FileInfo {
                                path: file_path_str.clone(),
                                content,
                                last_modified: metadata
                                    .modified()
                                    .ok()
                                    .and_then(|t| {
                                        chrono::DateTime::<chrono::Utc>::from(t)
                                            .to_rfc3339()
                                            .into()
                                    })
                                    .unwrap_or_default(),
                                size,
                            });
                        }
                    }

                    // Limit to 50 files per agent to prevent excessive context
                    if relevant_files.len() >= 50 {
                        break;
                    }
                }
            }
        }

        Ok(relevant_files)
    }

    /// Get relevant file extensions for agent type
    fn get_relevant_extensions(&self, agent_type: &str) -> Vec<&str> {
        match agent_type {
            "python-specialist" | "python-expert" => vec!["py", "pyx", "pyi"],
            "rust-specialist" | "rust-expert" => vec!["rs", "toml"],
            "go-specialist" | "go-expert" => vec!["go", "mod", "sum"],
            "typescript-specialist" | "javascript-specialist" => {
                vec!["ts", "tsx", "js", "jsx", "json"]
            }
            "swift-specialist" => vec!["swift"],
            "flutter-specialist" => vec!["dart"],
            "chief-architect" => vec![
                "md", "txt", "toml", "yaml", "yml", "json", "rs", "py", "go", "ts", "js",
            ],
            _ => vec!["md", "txt"], // Default to documentation
        }
    }

    /// Gather git context
    async fn gather_git_context(&self) -> Result<GitContext> {
        use std::process::Command;

        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let git_dir = root.join(".git");

        if !git_dir.exists() {
            // Not a git repository
            return Ok(GitContext {
                branch: String::from("unknown"),
                recent_commits: vec![],
                uncommitted_changes: vec![],
            });
        }

        // Get current branch
        let branch = Command::new("git")
            .current_dir(&root)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| String::from("unknown"));

        // Get recent commits (last 5)
        let recent_commits = Command::new("git")
            .current_dir(&root)
            .args([
                "log",
                "--format=%H%n%s%n%an%n%aI",
                "-5",
                "--no-merges",
            ])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| {
                let mut commits = Vec::new();
                let lines: Vec<&str> = s.lines().collect();

                for chunk in lines.chunks(4) {
                    if chunk.len() == 4 {
                        commits.push(CommitInfo {
                            hash: chunk[0].to_string(),
                            message: chunk[1].to_string(),
                            author: chunk[2].to_string(),
                            timestamp: chunk[3].to_string(),
                        });
                    }
                }

                commits
            })
            .unwrap_or_default();

        // Get uncommitted changes
        let uncommitted_changes = Command::new("git")
            .current_dir(&root)
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.lines().map(|line| line.to_string()).collect())
            .unwrap_or_default();

        Ok(GitContext {
            branch,
            recent_commits,
            uncommitted_changes,
        })
    }

    /// Gather previous agent outputs for an issue
    async fn gather_previous_outputs(&self, issue_id: &str) -> Result<Vec<AgentOutput>> {
        // Try to connect to daemon API to retrieve previous outputs from knowledge store
        let port = match crate::daemon::lifecycle::read_daemon_port() {
            Ok(port) => port,
            Err(_) => return Ok(vec![]), // Daemon not running
        };

        // Get project ID
        let project_id = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| String::from("unknown"));

        // Query knowledge store for agent outputs
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return Ok(vec![]),
        };

        let search_query = format!("issue:{} agent output", issue_id);
        let payload = serde_json::json!({
            "query": search_query,
            "limit": 20,
            "project_id": project_id,
            "knowledge_type": "implementation",
        });

        let response = match client
            .post(format!("http://localhost:{}/api/knowledge/search", port))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(vec![]),
        };

        let results: Vec<serde_json::Value> = match response.json().await {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };

        // Convert to AgentOutput format
        let outputs = results
            .iter()
            .filter_map(|item| {
                Some(AgentOutput {
                    agent: item["agent"].as_str()?.to_string(),
                    timestamp: item["timestamp"].as_str()?.to_string(),
                    decision: item["text"].as_str()?.to_string(),
                })
            })
            .collect();

        Ok(outputs)
    }

    /// Gather project metadata
    async fn gather_metadata(&self) -> Result<ProjectMetadata> {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Detect project type
        let project_type = self.detect_project_type(&root);

        // Extract dependencies based on project type
        let dependencies = self.extract_dependencies(&root, &project_type);

        Ok(ProjectMetadata {
            project_type,
            dependencies,
            test_coverage: None,
            last_build_status: None,
        })
    }

    /// Detect project type from marker files
    fn detect_project_type(&self, root: &PathBuf) -> String {
        if root.join("Cargo.toml").exists() {
            String::from("rust")
        } else if root.join("package.json").exists() {
            String::from("javascript")
        } else if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            String::from("python")
        } else if root.join("go.mod").exists() {
            String::from("go")
        } else if root.join("pubspec.yaml").exists() {
            String::from("flutter")
        } else {
            String::from("unknown")
        }
    }

    /// Extract dependencies from project files
    fn extract_dependencies(&self, root: &PathBuf, project_type: &str) -> Vec<String> {
        use std::fs;

        match project_type {
            "rust" => {
                // Parse Cargo.toml
                let cargo_toml = root.join("Cargo.toml");
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    let mut deps = Vec::new();
                    let mut in_deps = false;

                    for line in content.lines() {
                        if line.starts_with("[dependencies]") {
                            in_deps = true;
                        } else if line.starts_with('[') {
                            in_deps = false;
                        }

                        if in_deps && !line.trim().is_empty() && !line.trim().starts_with('#') {
                            if let Some(dep) = line.split('=').next() {
                                deps.push(dep.trim().to_string());
                            }
                        }
                    }

                    deps
                } else {
                    vec![]
                }
            }
            "javascript" => {
                // Parse package.json
                let package_json = root.join("package.json");
                if let Ok(content) = fs::read_to_string(&package_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let mut deps = Vec::new();

                        if let Some(dependencies) = json["dependencies"].as_object() {
                            deps.extend(dependencies.keys().map(|k| k.to_string()));
                        }

                        if let Some(dev_dependencies) = json["devDependencies"].as_object() {
                            deps.extend(dev_dependencies.keys().map(|k| k.to_string()));
                        }

                        deps
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            "python" => {
                // Try requirements.txt
                let requirements = root.join("requirements.txt");
                if let Ok(content) = fs::read_to_string(&requirements) {
                    content
                        .lines()
                        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                        .filter_map(|line| {
                            line.split(|c| c == '=' || c == '>' || c == '<')
                                .next()
                                .map(|s| s.trim().to_string())
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            "go" => {
                // Parse go.mod
                let go_mod = root.join("go.mod");
                if let Ok(content) = fs::read_to_string(&go_mod) {
                    let mut deps = Vec::new();
                    let mut in_require = false;

                    for line in content.lines() {
                        let trimmed = line.trim();

                        if trimmed.starts_with("require (") {
                            in_require = true;
                        } else if trimmed == ")" {
                            in_require = false;
                        }

                        if in_require || trimmed.starts_with("require ") {
                            let parts: Vec<&str> = trimmed.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let dep = if parts[0] == "require" {
                                    parts[1]
                                } else {
                                    parts[0]
                                };
                                deps.push(dep.to_string());
                            }
                        }
                    }

                    deps
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
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
