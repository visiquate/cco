# Security Remediation Checklist - CCO Auto-Update

**Priority Order:** CRITICAL → HIGH → MEDIUM → LOW
**Target Completion:** Before public release

---

## 🔴 CRITICAL PRIORITY (Do Not Deploy Without These)

### 1. Make Checksum Verification Mandatory
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:306-328`
**Severity:** CRITICAL
**CWE:** CWE-494

**Current Code (Lines 306-328):**
```rust
if let Some(checksum_asset) = checksum_asset {
    println!("→ Verifying checksum...");
    // ... verification ...
} else {
    println!("  ⚠️  No checksum available, skipping verification");  // ❌ DANGEROUS
}
```

**Required Fix:**
```rust
// MANDATORY: Fail hard if checksums missing
let checksum_asset = release.assets.iter()
    .find(|a| a.name == "checksums.sha256")
    .ok_or_else(|| anyhow!("SECURITY: No checksum file found - refusing to install unverified binary"))?;

println!("→ Verifying checksum...");
download_file(&checksum_asset.browser_download_url, &temp_checksum).await?;

let checksum_content = fs::read_to_string(&temp_checksum)?;
let expected_checksum = checksum_content
    .lines()
    .find(|line| line.contains(&asset_name))
    .and_then(|line| line.split_whitespace().next())
    .ok_or_else(|| anyhow!("Checksum for {} not found in checksums file", asset_name))?;

if !verify_checksum(&temp_file, expected_checksum)? {
    // Clean up compromised download
    let _ = fs::remove_file(&temp_file);
    return Err(anyhow!("SECURITY: Checksum verification FAILED - possible MITM attack or corrupted download"));
}

println!("  ✓ Checksum verified successfully");
```

**Testing:**
- [ ] Create release WITHOUT checksums.sha256 - verify install FAILS
- [ ] Modify downloaded binary - verify checksum catches it
- [ ] Use correct checksum - verify install succeeds

**Estimated Time:** 1 hour
**Dependencies:** None

---

### 2. Verify Repository Ownership
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:90-126`
**Severity:** CRITICAL
**CWE:** CWE-494

**Add New Function:**
```rust
/// Verify repository is owned by legitimate account/organization
async fn verify_repository_ownership(repo: &str) -> Result<()> {
    const EXPECTED_OWNER: &str = "brentley";  // Or organization name

    let client = reqwest::Client::builder()
        .user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    #[derive(Deserialize)]
    struct RepoInfo {
        owner: Owner,
    }

    #[derive(Deserialize)]
    struct Owner {
        login: String,
        #[serde(rename = "type")]
        owner_type: String,
    }

    let url = format!("https://api.github.com/repos/{}", repo);
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to verify repository ownership"));
    }

    let repo_info: RepoInfo = response.json().await?;

    if repo_info.owner.login != EXPECTED_OWNER {
        return Err(anyhow!(
            "SECURITY: Repository owner mismatch! Expected '{}', found '{}'. Possible account takeover.",
            EXPECTED_OWNER,
            repo_info.owner.login
        ));
    }

    tracing::info!("Repository ownership verified: {}", repo_info.owner.login);
    Ok(())
}
```

**Integration Point (in `fetch_latest_release`):**
```rust
async fn fetch_latest_release(channel: &str) -> Result<GitHubRelease> {
    // VERIFY REPOSITORY OWNERSHIP FIRST
    verify_repository_ownership(GITHUB_REPO).await?;

    let url = if channel == "stable" {
        format!("{}/{}/releases/latest", GITHUB_API_URL, GITHUB_REPO)
    } else {
        format!("{}/{}/releases", GITHUB_API_URL, GITHUB_REPO)
    };

    // ... rest of function ...
}
```

**Testing:**
- [ ] Test with correct owner - verify succeeds
- [ ] Mock API to return wrong owner - verify fails
- [ ] Test with network error - verify fails safely

**Estimated Time:** 2 hours
**Dependencies:** None

---

## 🟠 HIGH PRIORITY (Fix Before Public Release)

### 3. Secure Temporary Directory Creation
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:296-298`
**Severity:** HIGH
**CWE:** CWE-377

**Current Code:**
```rust
let temp_dir = std::env::temp_dir().join(format!("cco-update-{}", release.tag_name));
fs::create_dir_all(&temp_dir)?;
```

**Required Fix:**
```rust
use uuid::Uuid;
use std::fs::DirBuilder;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

// Create unpredictable temp directory with secure permissions
let temp_dir_name = format!("cco-update-{}-{}",
    release.tag_name,
    Uuid::new_v4()  // Add randomness
);
let temp_dir = std::env::temp_dir().join(temp_dir_name);

#[cfg(unix)]
{
    let mut builder = DirBuilder::new();
    builder.mode(0o700);  // rwx------ (owner only)
    builder.create(&temp_dir)?;

    // Verify permissions were set correctly
    let metadata = fs::metadata(&temp_dir)?;
    let perms = metadata.permissions().mode();
    if perms & 0o077 != 0 {  // Check group/other have no access
        return Err(anyhow!("SECURITY: Failed to set secure permissions on temp directory"));
    }
}

#[cfg(not(unix))]
{
    fs::create_dir_all(&temp_dir)?;
    // TODO: Set Windows ACLs for equivalent protection
}

tracing::debug!("Created secure temp directory: {}", temp_dir.display());
```

**Testing:**
- [ ] Verify temp dir has 0o700 permissions on Unix
- [ ] Verify random UUID prevents prediction
- [ ] Test cleanup on success and failure

**Estimated Time:** 2 hours
**Dependencies:** `uuid` crate (already in Cargo.toml)

---

### 4. Validate Downloaded File Permissions
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:330-365`
**Severity:** HIGH
**CWE:** CWE-732

**Add After Archive Extraction:**
```rust
// Set secure permissions on archive immediately after download
#[cfg(unix)]
{
    let mut archive_perms = fs::metadata(&temp_file)?.permissions();
    archive_perms.set_mode(0o600);  // rw------- (owner read/write only)
    fs::set_permissions(&temp_file, archive_perms)?;
}

// Extract archive
println!("→ Extracting archive...");
let tar_gz = fs::File::open(&temp_file)?;
let tar = flate2::read::GzDecoder::new(tar_gz);
let mut archive = tar::Archive::new(tar);
archive.unpack(&temp_dir)?;
let binary_path = temp_dir.join("cco");

if !binary_path.exists() {
    return Err(anyhow!("Binary not found in archive"));
}

// Set and VERIFY executable permissions
#[cfg(unix)]
{
    let mut perms = fs::metadata(&binary_path)?.permissions();
    perms.set_mode(0o755);  // rwxr-xr-x
    fs::set_permissions(&binary_path, perms)?;

    // VERIFY permissions were actually set
    let verify_perms = fs::metadata(&binary_path)?.permissions();
    let actual_mode = verify_perms.mode() & 0o777;
    if actual_mode != 0o755 {
        return Err(anyhow!(
            "SECURITY: Failed to set correct permissions on binary (expected 0o755, got 0o{:o})",
            actual_mode
        ));
    }

    tracing::debug!("Binary permissions verified: 0o{:o}", actual_mode);
}
```

**Testing:**
- [ ] Verify binary has 0o755 after extraction
- [ ] Test permission verification catches incorrect modes
- [ ] Test on different filesystems (ext4, APFS, etc.)

**Estimated Time:** 2 hours
**Dependencies:** None

---

### 5. Validate GitHub API Responses
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:244-258`
**Severity:** HIGH
**CWE:** CWE-20

**Add New Validation Functions:**
```rust
/// Validate asset name format to prevent path traversal
fn validate_asset_name(name: &str, expected_pattern: &str) -> Result<()> {
    // No path traversal characters
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(anyhow!("SECURITY: Asset name contains path traversal: {}", name));
    }

    // Must match expected pattern
    let re = regex::Regex::new(expected_pattern)
        .context("Invalid asset name pattern")?;

    if !re.is_match(name) {
        return Err(anyhow!("SECURITY: Asset name does not match expected format: {}", name));
    }

    Ok(())
}

/// Validate download URL to prevent SSRF
fn validate_download_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)
        .context("Invalid download URL")?;

    // MUST be HTTPS
    if parsed.scheme() != "https" {
        return Err(anyhow!("SECURITY: Download URL must use HTTPS, got: {}", parsed.scheme()));
    }

    // MUST be GitHub domain
    if let Some(host) = parsed.host_str() {
        let allowed_hosts = ["github.com", "githubusercontent.com", "github.io"];
        let is_github = allowed_hosts.iter().any(|&allowed| {
            host == allowed || host.ends_with(&format!(".{}", allowed))
        });

        if !is_github {
            return Err(anyhow!(
                "SECURITY: Download URL must be from GitHub domains, got: {}",
                host
            ));
        }
    } else {
        return Err(anyhow!("SECURITY: Download URL has no host"));
    }

    tracing::debug!("Download URL validated: {}", url);
    Ok(())
}
```

**Integration (in `install_update`):**
```rust
// Find asset
let asset = release
    .assets
    .iter()
    .find(|a| a.name == asset_name)
    .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform))?;

// VALIDATE BEFORE USE
validate_asset_name(&asset.name, r"^cco-v[\d.]+-[a-z0-9_-]+\.(tar\.gz|zip)$")?;
validate_download_url(&asset.browser_download_url)?;

// Also validate checksum asset
if let Some(checksum_asset) = release.assets.iter().find(|a| a.name == "checksums.sha256") {
    validate_asset_name(&checksum_asset.name, r"^checksums\.sha256$")?;
    validate_download_url(&checksum_asset.browser_download_url)?;
}

// Now safe to download
println!("→ Downloading CCO {}...", release.tag_name);
download_file(&asset.browser_download_url, &temp_file).await?;
```

**Testing:**
- [ ] Test with valid GitHub URLs - succeeds
- [ ] Test with non-GitHub URL - fails
- [ ] Test with HTTP (not HTTPS) - fails
- [ ] Test with path traversal in asset name - fails
- [ ] Test with malformed URL - fails

**Estimated Time:** 3 hours
**Dependencies:** `regex` (already in Cargo.toml)

---

### 6. Validate Release Tags
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:176-187`
**Severity:** HIGH
**CWE:** CWE-20

**Replace `extract_version` with:**
```rust
/// Extract and validate version from tag (e.g., "v2025.11.1" -> "2025.11.1")
fn extract_version(tag: &str) -> Result<String> {
    // Remove 'v' prefix
    let version_str = tag.trim_start_matches('v');

    // Strict validation - only allow date-based or semantic versions
    let date_version_pattern = r"^\d{4}\.\d{1,2}\.\d+$";
    let semver_pattern = r"^\d+\.\d+\.\d+(-[a-z0-9]+)?$";

    let date_re = regex::Regex::new(date_version_pattern)?;
    let semver_re = regex::Regex::new(semver_pattern)?;

    if !date_re.is_match(version_str) && !semver_re.is_match(version_str) {
        return Err(anyhow!(
            "SECURITY: Invalid version format '{}' (expected YYYY.MM.N or X.Y.Z)",
            version_str
        ));
    }

    // No path traversal or special characters
    if version_str.contains("..") || version_str.contains('/') || version_str.contains('\\') {
        return Err(anyhow!("SECURITY: Version string contains invalid characters: {}", version_str));
    }

    // Verify it parses as DateVersion (primary format)
    if let Ok(_) = DateVersion::parse(version_str) {
        return Ok(version_str.to_string());
    }

    // For backward compatibility, accept semantic versions
    // but log a warning
    if semver_re.is_match(version_str) {
        tracing::warn!("Using legacy semantic version: {}", version_str);
        return Ok(version_str.to_string());
    }

    Err(anyhow!("Version validation failed: {}", version_str))
}
```

**Testing:**
- [ ] Test valid date version: "v2025.11.1" → "2025.11.1"
- [ ] Test valid semver: "v1.2.3" → "1.2.3"
- [ ] Test invalid format: "v../../etc/passwd" → error
- [ ] Test invalid chars: "v2025;rm -rf /" → error

**Estimated Time:** 1 hour
**Dependencies:** None

---

## 🟡 MEDIUM PRIORITY (Fix Before Release)

### 7. Add Download Size Limits
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:129-148`
**Severity:** MEDIUM
**CWE:** CWE-400

**Replace `download_file` with:**
```rust
async fn download_file(url: &str, path: &Path) -> Result<()> {
    const MAX_BINARY_SIZE: u64 = 100 * 1024 * 1024; // 100 MB limit

    let client = reqwest::Client::builder()
        .user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed with status: {}", response.status()));
    }

    // Check Content-Length if available
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_BINARY_SIZE {
            return Err(anyhow!(
                "SECURITY: Download size {} bytes exceeds maximum {} bytes (possible DoS attack)",
                content_length,
                MAX_BINARY_SIZE
            ));
        }

        tracing::debug!("Download size: {} bytes", content_length);
    } else {
        tracing::warn!("No Content-Length header, will enforce size limit during download");
    }

    // Stream to disk instead of loading into memory
    let mut file = fs::File::create(path)
        .context("Failed to create download file")?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Download stream error")?;
        downloaded += chunk.len() as u64;

        // Enforce size limit even if Content-Length was wrong/missing
        if downloaded > MAX_BINARY_SIZE {
            let _ = fs::remove_file(path);  // Clean up partial download
            return Err(anyhow!(
                "SECURITY: Download exceeded maximum size ({} bytes) - aborting",
                MAX_BINARY_SIZE
            ));
        }

        std::io::Write::write_all(&mut file, &chunk)
            .context("Failed to write download chunk")?;
    }

    tracing::info!("Download complete: {} bytes", downloaded);
    Ok(())
}
```

**Add Disk Space Check (before download):**
```rust
/// Check if sufficient disk space is available
fn check_disk_space(required_bytes: u64) -> Result<()> {
    use sysinfo::{DiskExt, System, SystemExt};

    let mut sys = System::new_all();
    sys.refresh_all();

    let temp_dir = std::env::temp_dir();
    for disk in sys.disks() {
        if temp_dir.starts_with(disk.mount_point()) {
            let available = disk.available_space();

            // Require 2x space for safety (compressed archive + extracted files)
            let required_with_margin = required_bytes * 2;

            if available < required_with_margin {
                return Err(anyhow!(
                    "Insufficient disk space: {} MB required, {} MB available",
                    required_with_margin / 1024 / 1024,
                    available / 1024 / 1024
                ));
            }

            tracing::debug!("Disk space check passed: {} MB available", available / 1024 / 1024);
            return Ok(());
        }
    }

    // Couldn't determine disk space - proceed with caution
    tracing::warn!("Could not determine disk space, proceeding anyway");
    Ok(())
}
```

**Integration (in `install_update`, before download):**
```rust
// Check disk space before downloading
const ESTIMATED_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024;  // 50 MB estimate
check_disk_space(ESTIMATED_DOWNLOAD_SIZE)?;

// Download binary
println!("→ Downloading CCO {}...", release.tag_name);
download_file(&asset.browser_download_url, &temp_file).await?;
```

**Testing:**
- [ ] Test with normal-sized binary (< 100MB) - succeeds
- [ ] Mock response with 200MB Content-Length - fails immediately
- [ ] Stream 150MB of data - fails during download
- [ ] Test on filesystem with < 100MB free - fails disk space check

**Estimated Time:** 4 hours
**Dependencies:** `futures` (already in Cargo.toml)

---

### 8. Improve Cleanup on Failure
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:244-400`
**Severity:** MEDIUM
**CWE:** CWE-459

**Add RAII Cleanup Guard:**
```rust
/// RAII guard for automatic temporary directory cleanup
struct TempDirGuard(PathBuf);

impl TempDirGuard {
    fn new(path: PathBuf) -> Self {
        TempDirGuard(path)
    }

    /// Prevent cleanup on success
    fn persist(self) {
        std::mem::forget(self);
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        // Cleanup happens automatically on scope exit (error or panic)
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
            tracing::debug!("Cleaned up temp directory: {}", self.0.display());
        }
    }
}
```

**Usage in `install_update`:**
```rust
async fn install_update(release: &GitHubRelease, auto_confirm: bool) -> Result<()> {
    // ... platform detection, asset selection ...

    // Create temporary directory
    let temp_dir = /* ... secure creation ... */;

    // RAII guard will clean up on ANY error or panic
    let _temp_guard = TempDirGuard::new(temp_dir.clone());

    // Download and verify
    download_file(&asset.browser_download_url, &temp_file).await?;

    // Checksum verification
    let checksum_asset = /* ... find checksums.sha256 ... */;
    download_file(&checksum_asset.browser_download_url, &temp_checksum).await?;

    let checksum_content = fs::read_to_string(&temp_checksum)?;
    let expected_checksum = /* ... parse checksum ... */;

    if !verify_checksum(&temp_file, expected_checksum)? {
        // Guard will auto-cleanup temp_dir when function returns
        return Err(anyhow!("Checksum verification failed!"));
    }

    // ... extraction and installation ...

    // SUCCESS - prevent cleanup
    _temp_guard.persist();

    Ok(())
}
```

**Testing:**
- [ ] Test failed checksum - verify temp dir cleaned up
- [ ] Test failed download - verify temp dir cleaned up
- [ ] Test panic during extraction - verify temp dir cleaned up
- [ ] Test successful install - verify temp dir NOT cleaned up (kept for reference)

**Estimated Time:** 2 hours
**Dependencies:** None

---

### 9. Sanitize Version Strings
**Severity:** MEDIUM (Already addressed in item #6)

---

### 10. Implement GPG Signature Verification
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs` (new function)
**Severity:** MEDIUM
**CWE:** CWE-347

**Add GPG Verification Module:**
```rust
#[cfg(feature = "gpg-verify")]
mod gpg {
    use anyhow::{anyhow, Result};
    use std::path::Path;
    use std::process::Command;

    /// Embedded GPG public key for release signing
    const RELEASE_SIGNING_KEY: &str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----

[Your GPG public key will go here]

-----END PGP PUBLIC KEY BLOCK-----
"#;

    /// Verify GPG signature on a file
    pub fn verify_signature(file_path: &Path, sig_path: &Path) -> Result<()> {
        // Import trusted public key
        let keyring_path = std::env::temp_dir().join("cco-gpg-keyring.gpg");

        // Create temporary keyring
        let import_output = Command::new("gpg")
            .arg("--no-default-keyring")
            .arg("--keyring")
            .arg(&keyring_path)
            .arg("--import")
            .stdin(std::process::Stdio::piped())
            .output()?;

        if !import_output.status.success() {
            return Err(anyhow!("Failed to import GPG public key"));
        }

        // Write public key to stdin
        use std::io::Write;
        if let Some(mut stdin) = import_output.stdin {
            stdin.write_all(RELEASE_SIGNING_KEY.as_bytes())?;
        }

        // Verify signature
        let verify_output = Command::new("gpg")
            .arg("--no-default-keyring")
            .arg("--keyring")
            .arg(&keyring_path)
            .arg("--verify")
            .arg(sig_path)
            .arg(file_path)
            .output()?;

        // Clean up keyring
        let _ = std::fs::remove_file(keyring_path);

        if !verify_output.status.success() {
            return Err(anyhow!(
                "GPG signature verification FAILED:\n{}",
                String::from_utf8_lossy(&verify_output.stderr)
            ));
        }

        tracing::info!("GPG signature verified successfully");
        Ok(())
    }
}
```

**Integration (in `install_update`, after download but before checksum):**
```rust
#[cfg(feature = "gpg-verify")]
{
    // Download GPG signature
    let sig_asset_name = format!("{}.sig", asset_name);
    let sig_asset = release.assets.iter()
        .find(|a| a.name == sig_asset_name)
        .ok_or_else(|| anyhow!("GPG signature file not found"))?;

    let temp_sig = temp_dir.join(&sig_asset_name);
    download_file(&sig_asset.browser_download_url, &temp_sig).await?;

    // Verify GPG signature FIRST (most authoritative check)
    println!("→ Verifying GPG signature...");
    gpg::verify_signature(&temp_file, &temp_sig)?;
    println!("  ✓ GPG signature verified");
}

// Then verify checksum (as secondary check)
println!("→ Verifying checksum...");
```

**Testing:**
- [ ] Test with valid signature - succeeds
- [ ] Test with invalid signature - fails
- [ ] Test with modified binary - fails signature check
- [ ] Test without signature file - fails gracefully

**Estimated Time:** 6 hours (including key generation and CI/CD integration)
**Dependencies:** `gpg` binary (external), optional feature flag

---

## 🟢 LOW PRIORITY (Future Improvements)

### 11. Reduce User-Agent Verbosity
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:99-102`
**Severity:** LOW

**Change:**
```rust
// Before
.user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))

// After (privacy-conscious)
.user_agent("cco/client")  // Generic

// OR (moderate)
.user_agent(format!("cco/{}.x", major_version))  // Major version only
```

**Estimated Time:** 15 minutes

---

### 12. Add Certificate Pinning (Optional)
**File:** `/Users/brent/git/cc-orchestra/cco/src/update.rs:99-103`
**Severity:** LOW

**Enhancement:**
```rust
use reqwest::tls::Certificate;

// Embedded GitHub certificate (must be updated periodically)
const GITHUB_CERT_PEM: &[u8] = include_bytes!("github_com_cert.pem");

let cert = Certificate::from_pem(GITHUB_CERT_PEM)?;
let client = reqwest::Client::builder()
    .add_root_certificate(cert)
    .user_agent("cco/client")
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

**Note:** High operational burden - certificates expire/rotate

**Estimated Time:** 2 hours
**Dependencies:** Certificate management process

---

## Testing Matrix

### Unit Tests
- [ ] Checksum verification (valid, invalid, missing)
- [ ] Version validation (date format, semver, invalid)
- [ ] Asset name validation (valid, path traversal)
- [ ] URL validation (HTTPS, GitHub domains, other domains)
- [ ] Platform detection (all supported platforms)

### Integration Tests
- [ ] Full update flow (mock GitHub API)
- [ ] Rollback on verification failure
- [ ] Cleanup on error paths
- [ ] Permission validation

### Security Tests
- [ ] MITM simulation (modify API responses)
- [ ] Checksum bypass attempt (no checksums file)
- [ ] Path traversal attack (malicious asset names)
- [ ] Large download DoS (multi-GB binary)
- [ ] Temporary directory race condition
- [ ] Repository ownership verification

### Penetration Tests
- [ ] Local privilege escalation attempts
- [ ] Symlink attacks on RC files
- [ ] Race conditions in atomic replacement

---

## Deployment Checklist

Before releasing to production:

### Code Changes
- [ ] All CRITICAL fixes implemented and tested
- [ ] All HIGH fixes implemented and tested
- [ ] All MEDIUM fixes implemented and tested
- [ ] Code reviewed by security team
- [ ] Penetration testing completed

### Release Infrastructure
- [ ] GPG key pair generated for release signing
- [ ] GitHub Actions updated to sign releases
- [ ] Checksums.sha256 mandatory in all releases
- [ ] GPG signatures mandatory in all releases
- [ ] Repository ownership documented

### Documentation
- [ ] SECURITY.md created with disclosure policy
- [ ] Update documentation includes security warnings
- [ ] Admin guide for GPG key rotation
- [ ] Incident response plan documented

### Monitoring
- [ ] Logging enabled for security events
- [ ] Alerts configured for failed updates
- [ ] Metrics tracking update success rate
- [ ] Anomaly detection for unusual downloads

---

## Estimated Total Effort

| Priority | Items | Hours | Status |
|----------|-------|-------|--------|
| CRITICAL | 2 | 3h | ⏳ Pending |
| HIGH | 4 | 10h | ⏳ Pending |
| MEDIUM | 4 | 14h | ⏳ Pending |
| LOW | 2 | 2.25h | ⏳ Optional |
| **TOTAL** | **12** | **29.25h** | |

**Timeline:**
- **Week 1:** CRITICAL + HIGH (13 hours)
- **Week 2:** MEDIUM (14 hours)
- **Week 3:** Testing + Documentation (8 hours)
- **Total:** ~35 hours (1 week with 1 developer)

---

## Sign-Off

### Before Deployment
- [ ] Security Auditor approval
- [ ] Lead Developer approval
- [ ] QA/Testing sign-off
- [ ] Penetration testing report reviewed

### Post-Deployment Monitoring
- [ ] First 24 hours: Hourly update monitoring
- [ ] First week: Daily security log review
- [ ] First month: Weekly incident analysis

---

**Document Version:** 1.0
**Last Updated:** 2025-11-17
**Next Review:** After remediation completion
