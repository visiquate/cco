# Security Quick Reference - CCO Auto-Update

**Quick checklist for developers implementing security fixes**

---

## 🔴 CRITICAL - Block Deployment

### ✅ Checksum Verification Must Be Mandatory
```rust
// ❌ NEVER DO THIS
if let Some(checksum_asset) = checksum_asset {
    verify_checksum()?;
} else {
    println!("⚠️ No checksum, skipping");  // DANGEROUS!
}

// ✅ ALWAYS DO THIS
let checksum_asset = release.assets.iter()
    .find(|a| a.name == "checksums.sha256")
    .ok_or_else(|| anyhow!("SECURITY: No checksums - refusing to install"))?;

verify_checksum(&temp_file, expected_checksum)?;
```

### ✅ Verify Repository Ownership
```rust
// ❌ NEVER trust hardcoded repo without verification
const GITHUB_REPO: &str = "brentley/cco-releases";

// ✅ ALWAYS verify ownership before download
verify_repository_ownership(GITHUB_REPO).await?;
let release = fetch_latest_release(channel).await?;
```

---

## 🟠 HIGH - Required for Release

### ✅ Secure Temporary Directories
```rust
// ❌ NEVER use predictable names with default permissions
let temp_dir = std::env::temp_dir().join("cco-update");
fs::create_dir_all(&temp_dir)?;

// ✅ ALWAYS use random names with restricted permissions
use uuid::Uuid;
let temp_dir = std::env::temp_dir().join(format!("cco-update-{}", Uuid::new_v4()));

#[cfg(unix)]
{
    let mut builder = DirBuilder::new();
    builder.mode(0o700);  // Owner only
    builder.create(&temp_dir)?;
}
```

### ✅ Validate All External Input
```rust
// ❌ NEVER trust GitHub API responses directly
download_file(&asset.browser_download_url, &temp_file).await?;

// ✅ ALWAYS validate before use
validate_asset_name(&asset.name, EXPECTED_PATTERN)?;
validate_download_url(&asset.browser_download_url)?;
download_file(&asset.browser_download_url, &temp_file).await?;
```

### ✅ Set and Verify File Permissions
```rust
// ❌ NEVER assume permissions were set correctly
perms.set_mode(0o755);
fs::set_permissions(&binary_path, perms)?;

// ✅ ALWAYS verify after setting
perms.set_mode(0o755);
fs::set_permissions(&binary_path, perms)?;

let verify_perms = fs::metadata(&binary_path)?.permissions();
if verify_perms.mode() & 0o777 != 0o755 {
    return Err(anyhow!("Permission verification failed"));
}
```

---

## 🟡 MEDIUM - Recommended

### ✅ Limit Download Sizes
```rust
// ❌ NEVER load entire file into memory
let bytes = response.bytes().await?;
fs::write(path, &bytes)?;

// ✅ ALWAYS stream with size limits
const MAX_SIZE: u64 = 100 * 1024 * 1024;
let mut downloaded: u64 = 0;
let mut stream = response.bytes_stream();

while let Some(chunk) = stream.next().await {
    downloaded += chunk.len() as u64;
    if downloaded > MAX_SIZE {
        return Err(anyhow!("Size limit exceeded"));
    }
    file.write_all(&chunk)?;
}
```

### ✅ Clean Up on Failure
```rust
// ❌ NEVER leave temp files on error
if !verify_checksum(&temp_file, checksum)? {
    return Err(anyhow!("Checksum failed"));
}

// ✅ ALWAYS clean up on all error paths
struct TempDirGuard(PathBuf);
impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

let _guard = TempDirGuard(temp_dir.clone());
// ... operations ...
_guard.persist();  // On success only
```

---

## Common Validation Patterns

### Asset Name Validation
```rust
fn validate_asset_name(name: &str, pattern: &str) -> Result<()> {
    // No path traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(anyhow!("Invalid characters in asset name"));
    }

    // Match expected format
    let re = regex::Regex::new(pattern)?;
    if !re.is_match(name) {
        return Err(anyhow!("Asset name format mismatch"));
    }

    Ok(())
}

// Usage
validate_asset_name(&asset.name, r"^cco-v[\d.]+-[a-z0-9_-]+\.tar\.gz$")?;
```

### URL Validation
```rust
fn validate_download_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;

    // HTTPS only
    if parsed.scheme() != "https" {
        return Err(anyhow!("URL must use HTTPS"));
    }

    // GitHub domains only
    if let Some(host) = parsed.host_str() {
        if !host.ends_with("github.com") && !host.ends_with("githubusercontent.com") {
            return Err(anyhow!("URL must be from GitHub"));
        }
    }

    Ok(())
}
```

### Version String Validation
```rust
fn validate_version(version: &str) -> Result<()> {
    // Date-based: YYYY.MM.N
    let re = regex::Regex::new(r"^\d{4}\.\d{1,2}\.\d+$")?;

    if !re.is_match(version) {
        return Err(anyhow!("Invalid version format"));
    }

    // No path traversal
    if version.contains("..") || version.contains('/') {
        return Err(anyhow!("Invalid characters in version"));
    }

    Ok(())
}
```

---

## File Permission Reference

### Unix Permissions
```rust
// Directories
0o700  // rwx------  (owner only - SECURE)
0o755  // rwxr-xr-x  (world readable - OK for some cases)
0o777  // rwxrwxrwx  (world writable - NEVER USE)

// Files
0o600  // rw-------  (owner only - for sensitive files)
0o644  // rw-r--r--  (world readable - OK for public files)
0o755  // rwxr-xr-x  (executable - for binaries)

// Setting permissions
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms)?;
}

// Verifying permissions
let perms = fs::metadata(&path)?.permissions();
let mode = perms.mode() & 0o777;  // Mask out file type bits
assert_eq!(mode, 0o755);
```

---

## Error Handling Patterns

### Always Clean Up on Error
```rust
// Pattern 1: RAII Guard
struct Guard(PathBuf);
impl Drop for Guard {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

// Pattern 2: Explicit Cleanup
fn download_and_verify() -> Result<()> {
    download_file(&url, &path).await.context("Download failed")?;

    if !verify_checksum(&path, checksum)? {
        let _ = fs::remove_file(&path);  // Clean up
        return Err(anyhow!("Checksum failed"));
    }

    Ok(())
}

// Pattern 3: Defer Pattern (using scopeguard crate)
use scopeguard::defer;
defer! {
    let _ = fs::remove_file(&temp_file);
}
```

### Safe Path Construction
```rust
// ❌ NEVER concatenate paths with strings
let path = format!("/tmp/{}", user_input);

// ✅ ALWAYS use Path::join and validate
let path = std::env::temp_dir().join(user_input);

// Then validate the result
if !path.starts_with(std::env::temp_dir()) {
    return Err(anyhow!("Path traversal detected"));
}
```

---

## Logging Security Events

```rust
// Always log security-relevant events
tracing::info!("Update check initiated for channel: {}", channel);
tracing::info!("Repository ownership verified: {}", owner);
tracing::info!("Checksum verified for: {}", asset_name);

// Log failures at appropriate levels
tracing::error!("SECURITY: Checksum verification FAILED for {}", path);
tracing::warn!("No GPG signature available for release {}", version);

// Never log sensitive data
// ❌ tracing::debug!("Auth token: {}", token);
// ✅ tracing::debug!("Auth token: [REDACTED]");
```

---

## Testing Checklist

### Unit Tests
```rust
#[test]
fn test_asset_name_validation() {
    // Valid
    assert!(validate_asset_name("cco-v2025.11.1-darwin-arm64.tar.gz", PATTERN).is_ok());

    // Path traversal
    assert!(validate_asset_name("../../../etc/passwd", PATTERN).is_err());

    // Invalid format
    assert!(validate_asset_name("malicious.exe", PATTERN).is_err());
}

#[test]
fn test_url_validation() {
    // Valid GitHub URL
    assert!(validate_download_url("https://github.com/user/repo/releases/file").is_ok());

    // Non-HTTPS
    assert!(validate_download_url("http://github.com/file").is_err());

    // Non-GitHub domain
    assert!(validate_download_url("https://evil.com/file").is_err());
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_failed_checksum_cleanup() {
    // Setup: Create temp dir with malicious binary
    let temp_dir = create_temp_dir()?;

    // Execute: Try to install with wrong checksum
    let result = install_update(&bad_release, true).await;

    // Verify: Installation failed AND temp dir cleaned up
    assert!(result.is_err());
    assert!(!temp_dir.exists());
}
```

---

## Pre-Commit Checklist

Before committing security-related code:

- [ ] All external input validated
- [ ] File permissions set and verified
- [ ] Cleanup implemented on ALL error paths
- [ ] Security events logged appropriately
- [ ] No sensitive data in logs
- [ ] Unit tests for validation functions
- [ ] Integration tests for error paths
- [ ] Code reviewed by another developer
- [ ] Security checklist reviewed

---

## Pre-Release Checklist

Before releasing to production:

- [ ] All CRITICAL issues fixed
- [ ] All HIGH issues fixed
- [ ] Penetration testing completed
- [ ] Security code review done
- [ ] SECURITY.md created
- [ ] Incident response plan documented
- [ ] Monitoring configured
- [ ] Rollback plan tested

---

## Common Mistakes to Avoid

### ❌ Don't Trust External Data
```rust
// Bad: Using API response directly
let url = release.assets[0].browser_download_url;
download_file(&url, &path).await?;

// Good: Validate everything
let asset = find_asset(&release.assets, expected_name)?;
validate_asset_name(&asset.name)?;
validate_download_url(&asset.browser_download_url)?;
download_file(&asset.browser_download_url, &path).await?;
```

### ❌ Don't Assume Permissions
```rust
// Bad: Assume directory has correct permissions
fs::create_dir_all(&temp_dir)?;

// Good: Set and verify permissions
#[cfg(unix)]
{
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder.create(&temp_dir)?;

    let perms = fs::metadata(&temp_dir)?.permissions();
    if perms.mode() & 0o077 != 0 {
        return Err(anyhow!("Insecure permissions"));
    }
}
```

### ❌ Don't Skip Cleanup
```rust
// Bad: Cleanup only on happy path
download_file(&url, &temp_file).await?;
verify_checksum(&temp_file)?;
fs::remove_file(&temp_file)?;  // Never reached if verify fails

// Good: Cleanup on ALL paths
let _guard = TempFileGuard::new(temp_file.clone());
download_file(&url, &temp_file).await?;
verify_checksum(&temp_file)?;
_guard.persist();  // Only prevent cleanup on success
```

---

## Resources

- **Full Audit Report:** `SECURITY_AUDIT_AUTO_UPDATE.md`
- **Remediation Checklist:** `SECURITY_REMEDIATION_CHECKLIST.md`
- **Executive Summary:** `SECURITY_EXECUTIVE_SUMMARY.md`
- **OWASP Top 10:** https://owasp.org/Top10/
- **CWE Database:** https://cwe.mitre.org/

---

## Getting Help

**Security Questions:**
- Review full audit report for detailed explanations
- Consult with security team before implementing fixes
- Test all fixes in isolated environment first

**Code Review:**
- All security fixes require peer review
- Use security checklist during review
- Test with malicious inputs and error cases

---

**Last Updated:** 2025-11-17
**Version:** 1.0
