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
        context.relevant_files = self.gather_relevant_files(&project_root, agent_type).await?;

        // Gather git context
        context.git_context = self.gather_git_context(&project_root).await?;

        // Gather previous agent outputs
        context.previous_agent_outputs =
            self.gather_previous_outputs(&project_root, issue_id).await?;

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
    async fn gather_git_context(
        &self,
        root: &Path,
    ) -> Result<super::knowledge_broker::GitContext> {
        // TODO: Implement git integration
        Ok(super::knowledge_broker::GitContext {
            branch: String::from("main"),
            recent_commits: vec![],
            uncommitted_changes: vec![],
        })
    }

    /// Gather previous agent outputs
    async fn gather_previous_outputs(
        &self,
        _root: &Path,
        _issue_id: &str,
    ) -> Result<Vec<super::knowledge_broker::AgentOutput>> {
        // TODO: Implement previous output gathering
        Ok(vec![])
    }

    /// Gather project metadata
    async fn gather_metadata(
        &self,
        root: &Path,
    ) -> Result<super::knowledge_broker::ProjectMetadata> {
        let project_type = self.detect_project_type(root);

        Ok(super::knowledge_broker::ProjectMetadata {
            project_type,
            dependencies: vec![],
            test_coverage: None,
            last_build_status: None,
        })
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
