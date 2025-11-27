# Security Audit Report: CCO Private Binary Distribution System

**Audit Date**: 2025-11-24
**Auditor**: Security Auditor (Claude Sonnet 4.5)
**Scope**: Authentication, API security, R2 storage, binary distribution
**System Version**: CCO with OIDC device flow authentication and releases API

---

## Executive Summary

This security audit evaluated the CCO private binary distribution system, which uses OIDC device flow authentication and Cloudflare R2 for secure binary distribution. The system demonstrates **strong security fundamentals** with multiple defense-in-depth layers.

### Overall Security Rating: **B+ (Good)**

**Key Strengths:**
- OIDC device flow authentication (no passwords in CLI)
- Mandatory SHA256 checksum verification
- Secure token storage with file permissions (Unix)
- Presigned URL validation and HTTPS enforcement
- Download size limits and streaming (DoS protection)
- Automatic token refresh with expiration buffer
- Atomic binary replacement with rollback capability

**Critical Findings:** 0
**High Findings:** 1
**Medium Findings:** 4
**Low Findings:** 3
**Informational:** 2

### Risk Summary

The system has **no critical vulnerabilities** and demonstrates good security practices. The one HIGH finding relates to Windows token storage security. Medium findings address edge cases and defense-in-depth improvements. All findings have clear mitigation paths.

---

## Security Findings

### HIGH SEVERITY

#### H1: Windows Token Storage Lacks ACL Protection

**File**: `/Users/brent/git/cc-orchestra/cco/src/auth/token_storage.rs:78-84`

**Description**:
Token storage implements secure file permissions (0o600) on Unix systems but has no equivalent protection on Windows. Windows ACLs are not enforced, leaving tokens potentially readable by other users on shared Windows systems.

```rust
#[cfg(unix)]
{
    let mut perms = fs::metadata(&self.token_file)?.permissions();
    perms.set_mode(0o600); // rw-------
    fs::set_permissions(&self.token_file, perms)
        .context("Failed to set secure permissions on token file")?;
}
// No Windows equivalent implemented
```

**Impact**:
On Windows systems, tokens stored in `~/.config/cco/tokens.json` may be readable by other local users, potentially allowing unauthorized access to CCO releases.

**OWASP Reference**: A05:2021 - Security Misconfiguration

**Recommendation**:
1. Implement Windows ACL protection using `windows-acl` crate or Windows API
2. Set file to only be readable/writable by current user
3. Verify ACLs are correctly applied
4. Add automated tests for Windows ACL enforcement

**Example Implementation**:
```rust
#[cfg(windows)]
{
    use std::os::windows::fs::OpenOptionsExt;
    use windows::Win32::Storage::FileSystem::*;

    // Set DACL to only allow current user access
    // Reference: https://learn.microsoft.com/en-us/windows/win32/secauthz/access-control-lists
}
```

**Priority**: HIGH - Affects Windows users on shared systems

---

### MEDIUM SEVERITY

#### M1: Token Storage Location Predictable

**File**: `/Users/brent/git/cc-orchestra/cco/src/auth/token_storage.rs:57-66`

**Description**:
Tokens are always stored at a fixed, predictable path: `~/.config/cco/tokens.json`. While file permissions protect against casual access, advanced attackers targeting this specific application know exactly where to look.

```rust
fn get_token_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let cco_config_dir = config_dir.join("cco");
    // Always: ~/.config/cco/tokens.json
    Ok(cco_config_dir.join("tokens.json"))
}
```

**Impact**:
Reduces security through obscurity. Attackers with local access (malware, privilege escalation) know exactly which file to target.

**OWASP Reference**: A01:2021 - Broken Access Control

**Recommendation**:
1. Consider using OS keychain/credential manager instead of flat file
   - macOS: Keychain Services
   - Linux: Secret Service API (libsecret)
   - Windows: Credential Manager API
2. If staying with file storage, add obfuscation (random directory name component)
3. Implement file integrity monitoring (detect tampering)

**Priority**: MEDIUM - Defense-in-depth improvement

---

#### M2: No Token Revocation Support

**Files**:
- `/Users/brent/git/cc-orchestra/cco/src/auth/device_flow.rs`
- `/Users/brent/git/cc-orchestra/cco/src/auth/mod.rs:45-60`

**Description**:
The logout function only clears local tokens but doesn't revoke them on the server. Tokens remain valid on the API until expiration, even after user "logs out".

```rust
pub async fn logout() -> Result<()> {
    let storage = TokenStorage::new()?;
    storage.clear_tokens()?;  // Only clears locally
    println!("✅ Logout successful!");
    Ok(())
}
```

**Impact**:
If tokens are stolen before logout, attacker retains access until natural expiration. User cannot force-invalidate compromised tokens.

**OWASP Reference**: A07:2021 - Identification and Authentication Failures

**Recommendation**:
1. Implement token revocation endpoint: `POST /auth/token/revoke`
2. Call revocation during logout (best effort - continue even if fails)
3. Add "Revoke All Tokens" command for emergency use
4. Log revocation attempts for audit trail

**Example**:
```rust
pub async fn logout() -> Result<()> {
    let storage = TokenStorage::new()?;

    // Best-effort token revocation
    if let Ok(tokens) = storage.get_tokens() {
        let client = DeviceFlowClient::new(AUTH_API_URL);
        let _ = client.revoke_token(&tokens.access_token).await;
    }

    storage.clear_tokens()?;
    Ok(())
}
```

**Priority**: MEDIUM - Important for incident response

---

#### M3: No Rate Limiting in Client

**Files**:
- `/Users/brent/git/cc-orchestra/cco/src/auth/device_flow.rs:98-165`
- `/Users/brent/git/cc-orchestra/cco/src/auto_update/releases_api.rs:116-209`

**Description**:
Client-side code has no rate limiting or backoff logic beyond the device flow polling. An attacker with local access or malware could abuse the client to spam API endpoints.

```rust
pub async fn fetch_latest_release(channel: &str) -> Result<ReleaseInfo> {
    // No rate limiting or backoff
    let response = client.get(&url).bearer_auth(&access_token).send().await?;
    // ...
}
```

**Impact**:
Compromised client could be used for DoS attacks against the API. While server-side rate limiting should exist, client-side enforcement provides defense-in-depth.

**OWASP Reference**: A05:2021 - Security Misconfiguration

**Recommendation**:
1. Implement client-side rate limiting (e.g., token bucket algorithm)
2. Add exponential backoff for failed requests
3. Respect `Retry-After` headers from server
4. Log excessive retry attempts for detection

**Example**:
```rust
use governor::{Quota, RateLimiter};

pub struct RateLimitedClient {
    client: Client,
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RateLimitedClient {
    pub fn new() -> Self {
        let limiter = RateLimiter::direct(Quota::per_minute(nonzero!(60u32)));
        // Max 60 requests per minute
    }
}
```

**Priority**: MEDIUM - Defense-in-depth against abuse

---

#### M4: Presigned URL Validation Could Be Stricter

**File**: `/Users/brent/git/cc-orchestra/cco/src/auto_update/releases_api.rs:264-297`

**Description**:
Presigned URL validation checks HTTPS and R2 domains, but doesn't validate URL structure, query parameters, or expiration time embedded in presigned URL.

```rust
fn validate_presigned_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;

    // Only checks scheme and host
    if parsed.scheme() != "https" { return Err(...); }

    let is_r2 = host.ends_with(".r2.cloudflarestorage.com")
        || host.ends_with(".r2.dev")
        || host == "releases.visiquate.com";
    // ...
}
```

**Impact**:
Malicious API could craft presigned URLs that pass validation but point to unexpected resources or have very long expiration times.

**OWASP Reference**: A03:2021 - Injection

**Recommendation**:
1. Parse and validate AWS signature query parameters (`X-Amz-*`)
2. Verify expiration time is reasonable (e.g., < 1 hour)
3. Check URL path matches expected pattern (`/cco/v{version}/{platform}`)
4. Validate no unexpected query parameters (could be used for tracking)

**Example**:
```rust
fn validate_presigned_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;

    // Existing checks...

    // Validate path structure
    let path = parsed.path();
    if !path.starts_with("/cco/v") {
        return Err(anyhow!("SECURITY: Invalid path structure"));
    }

    // Check for AWS signature parameters
    let query_pairs: HashMap<_, _> = parsed.query_pairs().collect();
    if !query_pairs.contains_key("X-Amz-Signature") {
        return Err(anyhow!("SECURITY: Missing AWS signature"));
    }

    // Validate expiration
    if let Some(expires) = query_pairs.get("X-Amz-Expires") {
        let expires_secs: u64 = expires.parse()?;
        if expires_secs > 3600 { // Max 1 hour
            return Err(anyhow!("SECURITY: Expiration too long"));
        }
    }

    Ok(())
}
```

**Priority**: MEDIUM - Strengthens download security

---

### LOW SEVERITY

#### L1: No Logging of Authentication Events

**Files**:
- `/Users/brent/git/cc-orchestra/cco/src/auth/mod.rs`
- `/Users/brent/git/cc-orchestra/cco/src/auth/token_storage.rs`

**Description**:
Authentication events (login, logout, token refresh) are not logged to persistent audit trail. Only `tracing::info!` calls exist, which may not be captured in production.

**Impact**:
Difficult to detect unauthorized access attempts or token theft. No audit trail for security incidents.

**OWASP Reference**: A09:2021 - Security Logging and Monitoring Failures

**Recommendation**:
1. Implement persistent audit log for auth events
2. Log: login attempts, token refresh, logout, auth failures
3. Include: timestamp, user identifier, IP (if available), outcome
4. Protect log file with appropriate permissions
5. Consider sending audit events to centralized logging system

**Priority**: LOW - Important for compliance and incident response

---

#### L2: Download Progress Not Rate-Limited for Display

**File**: `/Users/brent/git/cc-orchestra/cco/src/auto_update/updater.rs:162-246`

**Description**:
The download function streams to disk but doesn't implement progress display or rate limiting for terminal output. While not a security issue per se, excessive progress updates could be used for terminal DoS.

```rust
while let Some(chunk_result) = stream.next().await {
    let chunk = chunk_result.context("Download stream error")?;
    downloaded += chunk.len() as u64;
    // No progress display rate limiting
    file.write_all(&chunk)?;
}
```

**Impact**:
Minimal. Could cause terminal flooding with progress updates.

**OWASP Reference**: None (usability/performance issue)

**Recommendation**:
1. Implement progress bar with rate-limited updates (e.g., max 10 updates/sec)
2. Use `indicatif` crate for professional progress display
3. Suppress progress in non-interactive mode (CI/CD)

**Priority**: LOW - Usability improvement with minor DoS prevention

---

#### L3: Token Refresh Has Short Retry Window

**File**: `/Users/brent/git/cc-orchestra/cco/src/auth/mod.rs:68-84`

**Description**:
Token refresh logic has a 5-minute buffer before expiration and only checks once. If refresh fails due to transient network issue, user must re-login.

```rust
pub async fn get_access_token() -> Result<String> {
    let tokens = storage.get_tokens()?;

    if tokens.is_expired(300) {  // 5 min buffer
        let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
        // Only one attempt - no retry
        storage.store_tokens(&new_tokens)?;
        Ok(new_tokens.access_token)
    } else {
        Ok(tokens.access_token)
    }
}
```

**Impact**:
Transient network failures force user to re-authenticate, degrading user experience. Not a security issue but affects availability.

**OWASP Reference**: None (availability issue)

**Recommendation**:
1. Implement retry logic with exponential backoff (3 attempts)
2. Increase buffer window closer to expiration (e.g., 10 minutes)
3. Cache last successful token longer
4. Only prompt for re-login after all retries exhausted

**Priority**: LOW - User experience improvement

---

### INFORMATIONAL

#### I1: No Multi-Factor Authentication Support

**Description**:
Current OIDC device flow implementation doesn't support MFA. While device flow itself provides some device-based verification, additional factors would strengthen security.

**Recommendation**:
Plan for MFA support in Phase 2:
- TOTP (Time-based One-Time Password)
- WebAuthn/FIDO2 for hardware keys
- SMS/Email OTP as fallback

**Priority**: INFORMATIONAL - Future enhancement

---

#### I2: Consider Certificate Pinning for API Endpoints

**Description**:
Client uses standard TLS certificate validation but doesn't pin API certificates. In high-security environments, certificate pinning prevents MITM attacks even with compromised CAs.

**Recommendation**:
1. Implement certificate pinning for `cco-api.visiquate.com`
2. Pin both leaf certificate and intermediate CA
3. Implement pin rotation mechanism
4. Make pinning optional (disabled by default)

**Priority**: INFORMATIONAL - Enterprise security feature

---

## Compliance Checklist

### Authentication Security

| Control | Status | Notes |
|---------|--------|-------|
| Device flow uses HTTPS | ✅ PASS | Hardcoded `https://` URL |
| No passwords in CLI | ✅ PASS | Device flow eliminates password entry |
| Tokens stored securely | ⚠️ PARTIAL | Unix: ✅ (0o600), Windows: ❌ (H1) |
| Token refresh before expiry | ✅ PASS | 5-minute buffer implemented |
| No hardcoded secrets | ✅ PASS | No credentials in code |
| Token revocation support | ❌ FAIL | No server-side revocation (M2) |

### Token Storage Security

| Control | Status | Notes |
|---------|--------|-------|
| File permissions (Unix) | ✅ PASS | 0o600 enforced and verified |
| File permissions (Windows) | ❌ FAIL | No ACL protection (H1) |
| Secure directory creation | ✅ PASS | Parent directory protected |
| No tokens in logs | ✅ PASS | Verified no token leakage |
| Secure deletion on logout | ✅ PASS | File removed, no recovery |
| Permission verification | ✅ PASS | Unix: verified at 0o600 |

### API Security

| Control | Status | Notes |
|---------|--------|-------|
| Bearer token validation | ✅ PASS | Server validates (assumed) |
| HTTPS only | ✅ PASS | Hardcoded HTTPS URLs |
| Rate limiting (server) | ⚠️ ASSUMED | Not verified in audit |
| Rate limiting (client) | ❌ FAIL | No client-side limiting (M3) |
| Input validation | ✅ PASS | Platform, version validated |
| Error handling | ✅ PASS | No sensitive data in errors |

### R2 Security

| Control | Status | Notes |
|---------|--------|-------|
| Private bucket | ✅ ASSUMED | Presigned URLs imply private |
| Presigned URL HTTPS | ✅ PASS | Validated before download |
| Presigned URL domain check | ✅ PASS | R2 domains whitelisted |
| Presigned URL TTL | ⚠️ ASSUMED | Server sets to 15 min (documented) |
| TTL validation | ❌ FAIL | Client doesn't verify TTL (M4) |
| URL structure validation | ❌ FAIL | No path/param validation (M4) |

### Binary Download Security

| Control | Status | Notes |
|---------|--------|-------|
| Mandatory SHA256 checksum | ✅ PASS | Required, not optional |
| Download size limits | ✅ PASS | 100MB enforced during download |
| Streaming download | ✅ PASS | Not loaded into memory |
| Secure temp directories | ✅ PASS | 0o700 permissions on Unix |
| Checksum before execution | ✅ PASS | Verified before extraction |
| Size validation | ✅ PASS | Checked against expected size |

### Binary Installation Security

| Control | Status | Notes |
|---------|--------|-------|
| Executable permissions | ✅ PASS | 0o755 set and verified |
| Atomic replacement | ✅ PASS | Unix: atomic, Windows: best-effort |
| Backup before replace | ✅ PASS | Old binary backed up |
| Verification after install | ✅ PASS | `--version` check |
| Rollback on failure | ✅ PASS | Restores from backup |
| Temp file cleanup | ✅ PASS | RAII guard ensures cleanup |

### Infrastructure Security

| Control | Status | Notes |
|---------|--------|-------|
| TLS/HTTPS everywhere | ✅ PASS | All endpoints use HTTPS |
| No exposed credentials | ✅ PASS | No secrets in repository |
| Secure secrets management | ⚠️ ASSUMED | Server-side not audited |
| API authentication required | ✅ PASS | All endpoints require bearer token |
| Presigned URL generation | ⚠️ ASSUMED | Server-side not audited |

---

## Risk Assessment Matrix

| Finding | Severity | Likelihood | Impact | Risk Score |
|---------|----------|------------|--------|------------|
| H1: Windows token storage | HIGH | Medium | High | **8/10** |
| M1: Predictable token location | MEDIUM | Low | Medium | 5/10 |
| M2: No token revocation | MEDIUM | Low | High | 6/10 |
| M3: No client rate limiting | MEDIUM | Low | Medium | 4/10 |
| M4: Presigned URL validation | MEDIUM | Low | Medium | 5/10 |
| L1: No auth event logging | LOW | N/A | Low | 3/10 |
| L2: Progress display DoS | LOW | Low | Low | 2/10 |
| L3: Token refresh retry | LOW | Medium | Low | 3/10 |

---

## Recommendations Priority Matrix

### Immediate (1-2 weeks)
1. **[H1] Implement Windows ACL protection** - Critical for Windows users
2. **[M2] Add token revocation endpoint and client support** - Important for incident response

### Short-term (1-2 months)
3. **[M4] Enhance presigned URL validation** - Strengthen download security
4. **[M3] Implement client-side rate limiting** - Defense-in-depth against abuse
5. **[L1] Add authentication event logging** - Audit trail and compliance

### Medium-term (3-6 months)
6. **[M1] Migrate to OS keychain/credential manager** - Better secret storage
7. **[I1] Plan for MFA support** - Enhanced authentication security
8. **[L3] Improve token refresh retry logic** - Better availability

### Long-term (6-12 months)
9. **[I2] Evaluate certificate pinning** - Enterprise security feature
10. Consider security audit of server-side API (not in scope)

---

## Security Best Practices Compliance

### OWASP Top 10 2021 Coverage

| OWASP Risk | Relevant Findings | Status |
|------------|-------------------|--------|
| A01: Broken Access Control | M1, M2 | ⚠️ Partial |
| A02: Cryptographic Failures | None | ✅ Good |
| A03: Injection | M4 | ⚠️ Minor |
| A04: Insecure Design | None | ✅ Good |
| A05: Security Misconfiguration | H1, M3 | ⚠️ Partial |
| A06: Vulnerable Components | N/A | ✅ Good |
| A07: Authentication Failures | M2, L3 | ⚠️ Minor |
| A08: Software/Data Integrity | None | ✅ Excellent |
| A09: Logging/Monitoring | L1 | ⚠️ Minor |
| A10: Server-Side Request Forgery | None | ✅ Good |

### Defense in Depth Layers

✅ **Application Layer**
- OIDC authentication
- Bearer token authorization
- Input validation

✅ **Transport Layer**
- HTTPS everywhere
- TLS certificate validation
- (Future: Certificate pinning)

✅ **Data Layer**
- SHA256 checksums
- Presigned URLs
- Secure token storage

✅ **Infrastructure Layer**
- Private R2 bucket
- Rate limiting (server-side assumed)
- Access control (presigned URLs)

⚠️ **Monitoring Layer**
- Limited audit logging (L1)
- No centralized monitoring
- No alerting on suspicious activity

---

## Positive Security Highlights

The CCO binary distribution system demonstrates several **excellent security practices**:

1. **Mandatory Checksum Verification** - SHA256 checksums are required, not optional. This prevents all binary tampering attacks.

2. **Secure Token Storage on Unix** - File permissions (0o600) are enforced AND verified after setting, preventing local access by other users.

3. **Presigned URL Validation** - Downloads only from validated R2 domains over HTTPS, preventing unauthorized download sources.

4. **Streaming Downloads** - Large binaries streamed to disk rather than loaded into memory, preventing memory exhaustion attacks.

5. **Atomic Binary Replacement** - Update installation is atomic (on Unix) with automatic rollback on failure, preventing broken installations.

6. **OIDC Device Flow** - No passwords in CLI, reducing credential theft risk and improving user security.

7. **Automatic Token Refresh** - Transparent to user, reduces re-authentication friction while maintaining security.

8. **Download Size Limits** - 100MB enforced during download stream, preventing disk exhaustion DoS attacks.

9. **Secure Temporary Directories** - Created with 0o700 permissions and cleaned up via RAII guard, preventing temp file attacks.

10. **Comprehensive Error Handling** - Clear messages guide users without exposing sensitive details.

---

## Testing Recommendations

### Security Test Cases

1. **Token Storage Security**
   ```bash
   # Unix: Verify file permissions
   cco login
   ls -la ~/.config/cco/tokens.json  # Should show -rw-------

   # Verify other users cannot read
   sudo -u otheruser cat ~/.config/cco/tokens.json  # Should fail
   ```

2. **Checksum Verification**
   ```bash
   # Simulate corrupted download (requires test mode)
   # Verify update aborts with checksum error
   ```

3. **Authentication Flow**
   ```bash
   # Test token expiration and refresh
   # Test 401 error handling
   # Test logout clears tokens
   ```

4. **Download Security**
   ```bash
   # Test size limit enforcement
   # Test presigned URL validation rejects HTTP
   # Test presigned URL validation rejects wrong domains
   ```

5. **Binary Replacement**
   ```bash
   # Test rollback on verification failure
   # Test atomic replacement (Unix)
   # Test backup restore
   ```

### Penetration Testing Scenarios

1. **Token Theft Attempt**
   - Attempt to read token file as different user
   - Attempt to capture tokens from logs
   - Attempt to intercept tokens in transit

2. **MITM Attack**
   - Attempt to serve malicious binary via MITM
   - Should fail at checksum verification

3. **Binary Tampering**
   - Modify binary after download but before installation
   - Should fail at checksum verification

4. **DoS Attacks**
   - Attempt oversized download
   - Attempt disk exhaustion
   - Attempt memory exhaustion

---

## Conclusion

The CCO private binary distribution system demonstrates **strong security fundamentals** with a defense-in-depth approach. The use of OIDC device flow, mandatory checksum verification, and secure token storage (on Unix) provides a solid security foundation.

### Summary Assessment

**Strengths:**
- Excellent binary integrity verification (mandatory SHA256)
- Strong authentication model (OIDC device flow)
- Good Unix security practices (file permissions)
- Defense against common attacks (MITM, tampering, DoS)

**Areas for Improvement:**
- Windows token storage security (HIGH priority)
- Token revocation support (MEDIUM priority)
- Client-side rate limiting (MEDIUM priority)
- Audit logging (LOW priority)

### Final Recommendation

**The system is APPROVED for production use** with the following conditions:

1. **Immediate**: Implement Windows ACL protection for token storage (H1)
2. **Within 30 days**: Add token revocation support (M2)
3. **Within 60 days**: Implement enhanced presigned URL validation (M4)
4. **Ongoing**: Monitor for security updates and conduct regular audits

The system's strong foundation in binary integrity verification and authentication security makes it suitable for private distribution. The identified issues are edge cases and defense-in-depth improvements rather than fundamental security flaws.

---

**Audit Completed**: 2025-11-24
**Next Audit Recommended**: 2026-05-24 (6 months)

---

## Appendix A: File Inventory

### Files Audited

```
Authentication Module:
- /Users/brent/git/cc-orchestra/cco/src/auth/mod.rs (85 lines)
- /Users/brent/git/cc-orchestra/cco/src/auth/device_flow.rs (206 lines)
- /Users/brent/git/cc-orchestra/cco/src/auth/token_storage.rs (262 lines)

Auto-Update Module:
- /Users/brent/git/cc-orchestra/cco/src/auto_update/mod.rs (796 lines)
- /Users/brent/git/cc-orchestra/cco/src/auto_update/releases_api.rs (453 lines)
- /Users/brent/git/cc-orchestra/cco/src/auto_update/updater.rs (570 lines)

API Client:
- /Users/brent/git/cc-orchestra/cco/src/api_client.rs (300 lines)

Documentation:
- /Users/brent/git/cc-orchestra/cco/AUTH_AND_RELEASES_API_IMPLEMENTATION.md
- /Users/brent/git/cc-orchestra/cco/IMPLEMENTATION_COMPLETE.md

Total Lines of Code Audited: ~2,672 lines
```

### Security-Relevant Configuration

```
API Endpoints:
- Authentication: https://cco-api.visiquate.com
- Releases: https://cco-api.visiquate.com

Token Storage:
- Path: ~/.config/cco/tokens.json
- Permissions: 0o600 (Unix only)

Download Limits:
- Max size: 100MB
- Timeout: 300 seconds

R2 Domains (Whitelisted):
- *.r2.cloudflarestorage.com
- *.r2.dev
- releases.visiquate.com
```

## Appendix B: Security Control Mapping

### NIST Cybersecurity Framework Mapping

| CSF Category | Controls Implemented | Gaps |
|--------------|---------------------|------|
| Identify | Asset inventory, Risk assessment | None |
| Protect | Authentication, Access control, Data security | Windows ACLs (H1) |
| Detect | Error monitoring | Audit logging (L1) |
| Respond | Rollback capability, Error handling | Token revocation (M2) |
| Recover | Backup and restore, Rollback | None |

---

**Document Version**: 1.0
**Classification**: Internal Use
**Distribution**: Security team, Development team, Management
