# CCO Distribution & Auto-Update Implementation Roadmap

## Overview

This document provides a detailed implementation plan for the CCO distribution and auto-update system, with specific tasks, timelines, and technical requirements.

## Phase 1: Foundation (Week 1-2)

### 1.1 Create Distribution Repository

**Task**: Set up `brentley/cco-releases` public repository

**Steps**:
1. Create new public GitHub repository
2. Configure repository settings:
   - Enable GitHub Pages (for installer hosting)
   - Set up branch protection for `main`
   - Configure release settings
3. Initial commit with:
   - README.md (from template)
   - LICENSE (MIT)
   - .gitignore
   - version-manifest.json (initial empty version)

**Files to create**:
```
cco-releases/
├── README.md
├── LICENSE
├── .gitignore
├── install.sh
├── install.ps1
├── version-manifest.json
└── .github/
    └── workflows/
        └── update-manifest.yml
```

### 1.2 Implement Rust Update Module

**Location**: `/cco/src/updater/`

**Core Components**:

```rust
// src/updater/mod.rs
pub mod checker;
pub mod downloader;
pub mod installer;
pub mod config;

pub use checker::UpdateChecker;
pub use config::UpdateConfig;
```

**Key Features**:
- Non-blocking update checks
- Manifest parsing
- Version comparison
- Platform detection

### 1.3 Add Update CLI Commands

**Modify**: `src/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Check for and install updates
    Update {
        #[arg(long)]
        check: bool,

        #[arg(long)]
        install: bool,

        #[arg(long)]
        channel: Option<String>,
    },

    /// Configure CCO settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}
```

## Phase 2: Core Update System (Week 3-4)

### 2.1 Manifest Fetcher

**File**: `src/updater/manifest.rs`

```rust
use serde::{Deserialize, Serialize};
use reqwest;
use semver::Version;

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: HashMap<String, String>,
    pub versions: HashMap<String, VersionInfo>,
}

impl VersionManifest {
    pub async fn fetch() -> Result<Self> {
        let url = "https://raw.githubusercontent.com/brentley/cco-releases/main/version-manifest.json";
        let response = reqwest::get(url).await?;
        let manifest = response.json::<Self>().await?;
        Ok(manifest)
    }

    pub fn get_latest(&self, channel: &str) -> Option<Version> {
        self.latest.get(channel)
            .and_then(|v| Version::parse(v).ok())
    }
}
```

### 2.2 Update Checker

**File**: `src/updater/checker.rs`

```rust
use crate::updater::manifest::VersionManifest;
use tokio::time::{Duration, interval};

pub struct UpdateChecker {
    config: UpdateConfig,
    last_check: Option<DateTime<Utc>>,
}

impl UpdateChecker {
    pub async fn start_background_check(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check hourly

            loop {
                interval.tick().await;

                if self.should_check() {
                    if let Ok(Some(update)) = self.check_for_updates().await {
                        self.handle_update(update).await;
                    }
                }
            }
        });
    }
}
```

### 2.3 Binary Downloader

**File**: `src/updater/downloader.rs`

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};

pub struct Downloader {
    http_client: reqwest::Client,
}

impl Downloader {
    pub async fn download_update(&self, url: &str, expected_hash: &str) -> Result<PathBuf> {
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().join("cco-update");

        // Download with progress
        let response = self.http_client.get(url).send().await?;
        let total_size = response.content_length();

        let mut file = File::create(&temp_path).await?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(total) = total_size {
                let progress = (downloaded as f64 / total as f64) * 100.0;
                // Emit progress event
            }
        }

        // Verify checksum
        self.verify_checksum(&temp_path, expected_hash).await?;

        Ok(temp_path)
    }
}
```

### 2.4 Atomic Installer

**File**: `src/updater/installer.rs`

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub struct Installer;

impl Installer {
    pub async fn install_update(&self, new_binary: PathBuf) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("backup");

        // Atomic replacement strategy
        // 1. Move current binary to backup
        fs::rename(&current_exe, &backup_path)?;

        // 2. Move new binary to current location
        match fs::rename(&new_binary, &current_exe) {
            Ok(_) => {
                // 3. Set executable permissions
                #[cfg(unix)]
                {
                    let mut perms = fs::metadata(&current_exe)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&current_exe, perms)?;
                }

                // 4. Remove backup
                fs::remove_file(backup_path).ok();

                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                fs::rename(&backup_path, &current_exe)?;
                Err(e.into())
            }
        }
    }
}
```

## Phase 3: CI/CD Pipeline (Week 5)

### 3.1 Update Existing Build Workflow

**Modify**: `.github/workflows/build.yml`

Add release job trigger:
```yaml
on:
  push:
    tags:
      - 'v*'
```

### 3.2 Create Release Workflow

**File**: `.github/workflows/release.yml`

Key features:
- Multi-platform builds
- Binary optimization and stripping
- Checksum generation
- GitHub Release creation
- Manifest update in distribution repo

### 3.3 Cross-Compilation Setup

**Cargo Configuration**: `.cargo/config.toml`

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Phase 4: Security & Polish (Week 6)

### 4.1 Implement Checksum Verification

```rust
pub async fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = format!("{:x}", hasher.finalize());
    Ok(result == expected)
}
```

### 4.2 Add Rollback Support

```rust
pub fn rollback() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let backup_path = current_exe.with_extension("backup");

    if backup_path.exists() {
        fs::rename(&backup_path, &current_exe)?;
        println!("Rolled back to previous version");
    } else {
        println!("No backup version available");
    }

    Ok(())
}
```

### 4.3 Configuration Management

**File**: `src/updater/config.rs`

```rust
use serde::{Deserialize, Serialize};
use dirs;

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateConfig {
    pub enabled: bool,
    pub auto_install: bool,
    pub check_interval: CheckInterval,
    pub channel: UpdateChannel,
    pub notify_on_update: bool,
    pub verify_signatures: bool,
}

impl UpdateConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            let default = Self::default();
            default.save()?;
            Ok(default)
        }
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("cco");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("config.toml"))
    }
}
```

## Phase 5: Testing & Documentation (Week 6+)

### 5.1 Test Matrix

**Unit Tests**:
- Version comparison logic
- Manifest parsing
- Configuration management
- Platform detection

**Integration Tests**:
- Download simulation
- Update installation (in temp directory)
- Rollback scenarios
- Checksum verification

**Manual Testing**:
- Fresh installation on each platform
- Update from various versions
- Network failure scenarios
- Permission edge cases

### 5.2 Documentation

Create comprehensive docs:
1. **User Guide**: Installation and update instructions
2. **Troubleshooting Guide**: Common issues and solutions
3. **Developer Guide**: Building from source
4. **API Documentation**: Update system internals

## Phase 6: Future Enhancements

### 6.1 Delta Updates (Future)

Implement binary diff updates to reduce bandwidth:
```rust
pub struct DeltaUpdate {
    base_version: Version,
    target_version: Version,
    patch_url: String,
    patch_size: u64,
}
```

### 6.2 GPG Signature Verification (Future)

Add cryptographic signatures:
```rust
pub async fn verify_signature(file: &Path, signature: &[u8]) -> Result<bool> {
    // GPG verification logic
}
```

### 6.3 Package Manager Integration (Future)

- **Homebrew Formula** (macOS)
- **Scoop Manifest** (Windows)
- **Snap Package** (Linux)
- **AUR Package** (Arch Linux)

## Implementation Checklist

### Week 1-2: Foundation
- [ ] Create `cco-releases` repository
- [ ] Set up basic installer scripts
- [ ] Implement update module structure
- [ ] Add CLI commands

### Week 3-4: Core System
- [ ] Implement manifest fetcher
- [ ] Create update checker
- [ ] Build downloader with progress
- [ ] Implement atomic installer

### Week 5: CI/CD
- [ ] Update build workflow
- [ ] Create release workflow
- [ ] Set up cross-compilation
- [ ] Test multi-platform builds

### Week 6: Polish
- [ ] Add checksum verification
- [ ] Implement rollback support
- [ ] Create configuration system
- [ ] Write comprehensive tests

### Post-Launch
- [ ] Monitor telemetry (if enabled)
- [ ] Gather user feedback
- [ ] Fix reported issues
- [ ] Plan future enhancements

## Success Metrics

**Technical Metrics**:
- Update success rate > 99%
- Download size < 3MB compressed
- Update check latency < 100ms
- Installation time < 10 seconds

**User Metrics**:
- Installation success rate > 95%
- Auto-update adoption > 50%
- User-reported issues < 5 per release
- Platform coverage > 90%

## Risk Mitigation

**Risk**: Binary corruption during update
**Mitigation**: Checksum verification, atomic replacement, automatic rollback

**Risk**: Network failures during download
**Mitigation**: Resume support, retry logic, fallback URLs

**Risk**: Permission issues on installation
**Mitigation**: User-space installation, clear error messages, fallback locations

**Risk**: Breaking changes in updates
**Mitigation**: Semantic versioning, compatibility checks, rollback support

## Conclusion

This implementation roadmap provides a clear path to building a robust distribution and auto-update system for CCO. The phased approach ensures each component is thoroughly tested before moving to the next, while the architecture supports future enhancements without breaking existing functionality.