//! Context Injector Component
//!
//! Finds relevant files, analyzes project structure, identifies previous agent outputs,
//! and performs intelligent truncation for fast context gathering (<100ms target).

use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[allow(dead_code)]
const MAX_CONTEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_FILE_SIZE: usize = 1 * 1024 * 1024; // 1MB per file
const PERFORMANCE_TARGET_MS: u128 = 100;

/// Context injector for gathering relevant project context
pub struct ContextInjector {
    /// Cache of project structures by project root
    project_cache: Arc<DashMap<PathBuf, CachedProjectInfo>>,
}

#[derive(Debug, Clone)]
struct CachedProjectInfo {
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    last_updated: Instant,
}

impl ContextInjector {
    /// Create a new context injector
    pub fn new() -> Self {
        Self {
            project_cache: Arc::new(DashMap::new()),
        }
    }

    /// Gather context for an agent
    pub async fn gather_context(
        &self,
        agent_type: &str,
        issue_id: &str,
    ) -> Result<super::knowledge_broker::Context> {
        let start = Instant::now();

        let mut context = super::knowledge_broker::Context::default();

        // Get project root
        let project_root = self.detect_project_root()?;

        // Gather project structure (cached)
        context.project_structure = self.gather_project_structure(&project_root).await?;

        // Gather relevant files based on agent type
        context.relevant_files = self
            .gather_relevant_files(&project_root, agent_type)
            .await?;

        // Gather git context
        context.git_context = self.gather_git_context(&project_root).await?;

        // Gather previous agent outputs
        context.previous_agent_outputs = self
            .gather_previous_outputs(&project_root, issue_id)
            .await?;

        // Gather metadata
        context.metadata = self.gather_metadata(&project_root).await?;

        let elapsed = start.elapsed().as_millis();
        if elapsed > PERFORMANCE_TARGET_MS {
            tracing::warn!(
                "Context gathering took {}ms (target: {}ms)",
                elapsed,
                PERFORMANCE_TARGET_MS
            );
        } else {
            tracing::debug!("Context gathered in {}ms", elapsed);
        }

        Ok(context)
    }

    /// Detect project root directory
    fn detect_project_root(&self) -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;

        // Look for common project markers
        let mut dir = current_dir.clone();
        loop {
            if dir.join(".git").exists()
                || dir.join("Cargo.toml").exists()
                || dir.join("package.json").exists()
                || dir.join("pyproject.toml").exists()
            {
                return Ok(dir);
            }

            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return Ok(current_dir),
            }
        }
    }

    /// Gather project structure with caching
    async fn gather_project_structure(
        &self,
        root: &Path,
    ) -> Result<super::knowledge_broker::ProjectStructure> {
        // Check cache first
        if let Some(cached) = self.project_cache.get(root) {
            if cached.last_updated.elapsed().as_secs() < 60 {
                // Cache valid for 1 minute
                return Ok(super::knowledge_broker::ProjectStructure {
                    root: root.to_path_buf(),
                    files: cached
                        .files
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect(),
                    directories: cached
                        .directories
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect(),
                });
            }
        }

        // Scan project
        let (files, directories) = self.scan_directory(root)?;

        // Update cache
        self.project_cache.insert(
            root.to_path_buf(),
            CachedProjectInfo {
                files: files.clone(),
                directories: directories.clone(),
                last_updated: Instant::now(),
            },
        );

        Ok(super::knowledge_broker::ProjectStructure {
            root: root.to_path_buf(),
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

    /// Scan a directory recursively
    fn scan_directory(&self, root: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut files = Vec::new();
        let mut directories = Vec::new();

        let ignore_dirs: HashSet<&str> = [
            ".git",
            "node_modules",
            "target",
            "dist",
            "build",
            ".venv",
            "__pycache__",
        ]
        .iter()
        .copied()
        .collect();

        self.scan_recursive(root, root, &mut files, &mut directories, &ignore_dirs)?;

        Ok((files, directories))
    }

    /// Recursive directory scanning
    fn scan_recursive(
        &self,
        root: &Path,
        current: &Path,
        files: &mut Vec<PathBuf>,
        directories: &mut Vec<PathBuf>,
        ignore_dirs: &HashSet<&str>,
    ) -> Result<()> {
        if !current.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !ignore_dirs.contains(name) {
                        directories.push(path.strip_prefix(root)?.to_path_buf());
                        self.scan_recursive(root, &path, files, directories, ignore_dirs)?;
                    }
                }
            } else if path.is_file() {
                files.push(path.strip_prefix(root)?.to_path_buf());
            }
        }

        Ok(())
    }

    /// Gather relevant files based on agent type
    async fn gather_relevant_files(
        &self,
        root: &Path,
        agent_type: &str,
    ) -> Result<Vec<super::knowledge_broker::FileInfo>> {
        let extensions = self.get_relevant_extensions(agent_type);

        let mut relevant_files = Vec::new();

        let (all_files, _) = self.scan_directory(root)?;

        for file_path in all_files {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    let full_path = root.join(&file_path);

                    // Check file size
                    if let Ok(metadata) = fs::metadata(&full_path) {
                        let size = metadata.len() as usize;

                        if size <= MAX_FILE_SIZE {
                            if let Ok(content) = fs::read_to_string(&full_path) {
                                relevant_files.push(super::knowledge_broker::FileInfo {
                                    path: file_path.to_string_lossy().to_string(),
                                    content: self.truncate_content(&content, size),
                                    last_modified: metadata
                                        .modified()
                                        .ok()
                                        .and_then(|t| {
                                            <std::time::SystemTime as Into<
                                                chrono::DateTime<chrono::Utc>,
                                            >>::into(t)
                                            .to_rfc3339()
                                            .into()
                                        })
                                        .unwrap_or_default(),
                                    size,
                                });
                            }
                        }
                    }
                }
            }

            // Limit total number of files to prevent excessive context
            if relevant_files.len() >= 50 {
                break;
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

    /// Truncate content if too large
    fn truncate_content(&self, content: &str, size: usize) -> String {
        if size > MAX_FILE_SIZE {
            let chars_to_keep = (MAX_FILE_SIZE / 2).min(content.len());
            format!(
                "{}...\n\n[Content truncated - {} bytes omitted]\n",
                &content[..chars_to_keep],
                size - chars_to_keep
            )
        } else {
            content.to_string()
        }
    }

    /// Gather git context
    async fn gather_git_context(&self, root: &Path) -> Result<super::knowledge_broker::GitContext> {
        use std::process::Command;

        let git_dir = root.join(".git");
        if !git_dir.exists() {
            // Not a git repository
            return Ok(super::knowledge_broker::GitContext {
                branch: String::from("unknown"),
                recent_commits: vec![],
                uncommitted_changes: vec![],
            });
        }

        // Get current branch
        let branch = Command::new("git")
            .current_dir(root)
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
            .current_dir(root)
            .args(["log", "--format=%H%n%s%n%an%n%aI", "-5", "--no-merges"])
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
                        commits.push(super::knowledge_broker::CommitInfo {
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
            .current_dir(root)
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

        Ok(super::knowledge_broker::GitContext {
            branch,
            recent_commits,
            uncommitted_changes,
        })
    }

    /// Gather previous agent outputs
    async fn gather_previous_outputs(
        &self,
        root: &Path,
        issue_id: &str,
    ) -> Result<Vec<super::knowledge_broker::AgentOutput>> {
        // Try to query the knowledge store via daemon API
        // If daemon is not running, return empty results
        let port = match crate::daemon::lifecycle::read_daemon_port() {
            Ok(port) => port,
            Err(_) => return Ok(vec![]), // Daemon not running
        };

        // Get project ID from git repo or directory name
        let project_id = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Query knowledge store for agent outputs related to this issue
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?;

        // Search for agent outputs with the issue ID
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
            _ => return Ok(vec![]), // Failed to query, return empty
        };

        let results: Vec<serde_json::Value> = match response.json().await {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };

        // Convert search results to AgentOutput format
        let outputs = results
            .iter()
            .filter_map(|item| {
                Some(super::knowledge_broker::AgentOutput {
                    agent: item["agent"].as_str()?.to_string(),
                    timestamp: item["timestamp"].as_str()?.to_string(),
                    decision: item["text"].as_str()?.to_string(),
                })
            })
            .collect();

        Ok(outputs)
    }

    /// Gather project metadata
    async fn gather_metadata(
        &self,
        root: &Path,
    ) -> Result<super::knowledge_broker::ProjectMetadata> {
        let project_type = self.detect_project_type(root);
        let dependencies = self.extract_dependencies(root, &project_type);

        Ok(super::knowledge_broker::ProjectMetadata {
            project_type,
            dependencies,
            test_coverage: None,
            last_build_status: None,
        })
    }

    /// Extract dependencies from project files
    fn extract_dependencies(&self, root: &Path, project_type: &str) -> Vec<String> {
        match project_type {
            "rust" => self.extract_cargo_dependencies(root),
            "javascript" => self.extract_npm_dependencies(root),
            "python" => self.extract_python_dependencies(root),
            "go" => self.extract_go_dependencies(root),
            _ => vec![],
        }
    }

    /// Extract Cargo dependencies
    fn extract_cargo_dependencies(&self, root: &Path) -> Vec<String> {
        let cargo_toml = root.join("Cargo.toml");
        if !cargo_toml.exists() {
            return vec![];
        }

        match fs::read_to_string(&cargo_toml) {
            Ok(content) => {
                // Simple TOML parsing for dependencies
                let mut deps = Vec::new();
                let mut in_dependencies = false;

                for line in content.lines() {
                    if line.starts_with("[dependencies]") {
                        in_dependencies = true;
                        continue;
                    } else if line.starts_with('[') {
                        in_dependencies = false;
                    }

                    if in_dependencies && !line.trim().is_empty() && !line.trim().starts_with('#') {
                        if let Some(dep_name) = line.split('=').next() {
                            deps.push(dep_name.trim().to_string());
                        }
                    }
                }

                deps
            }
            Err(_) => vec![],
        }
    }

    /// Extract npm dependencies
    fn extract_npm_dependencies(&self, root: &Path) -> Vec<String> {
        let package_json = root.join("package.json");
        if !package_json.exists() {
            return vec![];
        }

        match fs::read_to_string(&package_json) {
            Ok(content) => {
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
            }
            Err(_) => vec![],
        }
    }

    /// Extract Python dependencies
    fn extract_python_dependencies(&self, root: &Path) -> Vec<String> {
        // Try requirements.txt first
        let requirements_txt = root.join("requirements.txt");
        if requirements_txt.exists() {
            if let Ok(content) = fs::read_to_string(&requirements_txt) {
                return content
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                    .filter_map(|line| {
                        // Extract package name (before ==, >=, etc.)
                        line.split(|c| c == '=' || c == '>' || c == '<')
                            .next()
                            .map(|s| s.trim().to_string())
                    })
                    .collect();
            }
        }

        // Try pyproject.toml
        let pyproject_toml = root.join("pyproject.toml");
        if pyproject_toml.exists() {
            if let Ok(content) = fs::read_to_string(&pyproject_toml) {
                let mut deps = Vec::new();
                let mut in_dependencies = false;

                for line in content.lines() {
                    if line.contains("[tool.poetry.dependencies]")
                        || line.contains("[project.dependencies]")
                    {
                        in_dependencies = true;
                        continue;
                    } else if line.starts_with('[') {
                        in_dependencies = false;
                    }

                    if in_dependencies && !line.trim().is_empty() {
                        if let Some(dep_name) = line.split('=').next() {
                            let dep = dep_name.trim().trim_matches('"');
                            if !dep.is_empty() && dep != "python" {
                                deps.push(dep.to_string());
                            }
                        }
                    }
                }

                return deps;
            }
        }

        vec![]
    }

    /// Extract Go dependencies
    fn extract_go_dependencies(&self, root: &Path) -> Vec<String> {
        let go_mod = root.join("go.mod");
        if !go_mod.exists() {
            return vec![];
        }

        match fs::read_to_string(&go_mod) {
            Ok(content) => {
                let mut deps = Vec::new();
                let mut in_require = false;

                for line in content.lines() {
                    let trimmed = line.trim();

                    if trimmed.starts_with("require (") {
                        in_require = true;
                        continue;
                    } else if trimmed == ")" {
                        in_require = false;
                        continue;
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
            }
            Err(_) => vec![],
        }
    }

    /// Detect project type from markers
    fn detect_project_type(&self, root: &Path) -> String {
        if root.join("Cargo.toml").exists() {
            "rust".to_string()
        } else if root.join("package.json").exists() {
            "javascript".to_string()
        } else if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            "python".to_string()
        } else if root.join("go.mod").exists() {
            "go".to_string()
        } else if root.join("pubspec.yaml").exists() {
            "flutter".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Clear cache for a specific issue
    pub async fn clear_cache(&self, _issue_id: &str) -> Result<usize> {
        // Clear all project caches for simplicity
        let count = self.project_cache.len();
        self.project_cache.clear();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_detect_project_root() {
        let injector = ContextInjector::new();
        let root = injector.detect_project_root().unwrap();
        assert!(root.exists());
    }

    #[tokio::test]
    async fn test_scan_directory() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();

        let injector = ContextInjector::new();
        let (files, dirs) = injector.scan_directory(root).unwrap();

        assert!(files.len() >= 2);
        assert!(dirs.len() >= 1);
    }

    #[tokio::test]
    async fn test_get_relevant_extensions() {
        let injector = ContextInjector::new();

        let rust_exts = injector.get_relevant_extensions("rust-specialist");
        assert!(rust_exts.contains(&"rs"));

        let python_exts = injector.get_relevant_extensions("python-specialist");
        assert!(python_exts.contains(&"py"));
    }
}
