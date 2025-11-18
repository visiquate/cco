# Auto-Update Security Documentation

## Overview

This document details all security features implemented in CCO's auto-update system. CCO implements 12 comprehensive security fixes to protect users from supply chain attacks, credential theft, and binary tampering.

## Security Design Principles

Before updates are installed, CCO verifies:

1. **Source authenticity** - Updates come from the official GitHub repository
2. **File integrity** - Downloaded files haven't been corrupted or modified
3. **Version validity** - Update is a legitimate newer version
4. **Binary safety** - New binary works before replacing the old one
5. **Atomic operations** - Updates either fully succeed or fully rollback

## The 12 Security Fixes

### 1. HTTPS-Only Connections (Critical)

**Vulnerability**: Unencrypted HTTP downloads could be intercepted and modified.

**Fix Implemented**:
- All connections to GitHub use HTTPS (TLS 1.2+)
- No HTTP fallback is supported
- Connection verification is mandatory

**How It Protects You**:
```
GitHub Servers (HTTPS)
    ↓ Encrypted tunnel
    ↓ Cannot be intercepted
Your Machine
```

**Testing**:
```bash
# Verify HTTPS enforcement
cco update --check  # All connections are encrypted
curl https://api.github.com/repos/yourusername/cco/releases  # Works
curl http://api.github.com/repos/yourusername/cco/releases   # Fails
```

---

### 2. SHA256 Checksum Verification (Critical)

**Vulnerability**: Tampered binaries could be installed if checksums aren't verified.

**Fix Implemented**:
- Every binary download is verified against SHA256 checksums
- Checksums are downloaded separately from GitHub
- Constant-time comparison prevents timing attacks

**How It Protects You**:
```
Downloaded Binary
    ↓ SHA256 Hash (compute)
    ↓ Compare with GitHub checksums
    ↓ Match = safe, Mismatch = abort
```

**Checksum Process**:
```bash
# 1. Download binary
wget https://github.com/cco/releases/download/v2025.11.3/cco-v2025.11.3-linux-x86_64.tar.gz

# 2. Download checksums
wget https://github.com/cco/releases/download/v2025.11.3/checksums.sha256

# 3. Verify (automatic in auto-update)
sha256sum -c checksums.sha256
# cco-v2025.11.3-linux-x86_64.tar.gz: OK
```

**Manual Verification**:
```bash
# If you download manually:
sha256sum cco-v2025.11.3-linux-x86_64.tar.gz
# Output: abc123def456...

# Compare with checksums.sha256:
cat checksums.sha256 | grep linux-x86_64
# abc123def456...  cco-v2025.11.3-linux-x86_64.tar.gz
```

---

### 3. Secure Temporary File Handling (Critical)

**Vulnerability**: Attackers could manipulate files during download and extraction.

**Fix Implemented**:
- Temporary files created in isolated system temp directory
- Files created with restrictive permissions (0o600)
- Automatic cleanup after use
- Verification before use

**How It Protects You**:
```
Creation: mktemp /tmp/cco-update-XXXXX
    ↓ Permissions: 0o600 (owner read/write only)
    ↓ Other users cannot access
    ↓ Download binary
    ↓ Verify checksum
    ↓ Extract binary
    ↓ Verify works
    ↓ Cleanup: rm -rf /tmp/cco-update-XXXXX
```

**Technical Details**:
```rust
// Rust's tempfile crate handles secure temp files
let temp_dir = tempfile::tempdir()?;  // Secure creation
temp_dir.path()  // Has 0o700 permissions
// Automatically cleaned up when dropped
```

---

### 4. Release Tag Validation (High)

**Vulnerability**: Attacker could create fake releases with malicious versions.

**Fix Implemented**:
- Only validates releases with proper semantic versioning
- Requires format: vYYYY.MM.N (e.g., v2025.11.3)
- Rejects invalid version formats
- Version comparison prevents downgrade attacks

**How It Protects You**:
```
Downloaded Version: v2025.11.3
    ↓ Validate format: vYYYY.MM.N
    ↓ Compare: 2025.11.3 > 2025.11.2 (current)
    ↓ Valid = continue, Invalid = abort
```

**Validation Process**:
```rust
// Check format
let version = release.tag_name.strip_prefix("v")?;
// v2025.11.3 → "2025.11.3" ✓
// v2025-11-3 → rejected ✗
// 2025.11.3 → rejected (needs 'v') ✗

// Check it's newer
let new_version = DateVersion::parse(version)?;
let current = env!("CARGO_PKG_VERSION");
if new_version <= current {
    return Err("Cannot downgrade");
}
```

---

### 5. Binary Verification Before Installation (Critical)

**Vulnerability**: Corrupted binaries could break CCO after installation.

**Fix Implemented**:
- New binary is tested before replacing current version
- `cco --version` is run on new binary to verify functionality
- Only proceeds if verification succeeds
- Current version remains untouched if verification fails

**How It Protects You**:
```
Downloaded Binary
    ↓ Extract from tar.gz/zip
    ↓ Make executable (chmod +x)
    ↓ Test: ./new_cco --version
    ↓ Success = safe to install
    ↓ Failure = abort, keep current version
```

**What Gets Verified**:
```bash
# The new binary must successfully:
cco --version        # Output version string
cco config show      # Read configuration
cco update --check   # Connect to GitHub

# If any command fails, installation is aborted
```

---

### 6. Atomic Binary Replacement (High)

**Vulnerability**: Partial replacements could leave CCO in broken state.

**Fix Implemented**:
- Binary replacement uses atomic file operations
- Either fully succeeds or fully rolls back
- No intermediate broken states possible

**How It Protects You**:

On Unix (macOS/Linux):
```bash
# Atomic rename operation
mv /tmp/cco-new ~/.local/bin/cco.new
mv ~/.local/bin/cco ~/.local/bin/cco.backup  # Old backup
mv ~/.local/bin/cco.new ~/.local/bin/cco     # Atomic!
# All happen together or not at all
```

On Windows:
```
Similar atomicity achieved through:
- Delete old binary
- Move new binary in place
- Handles file locking appropriately
```

---

### 7. Automatic Backup Creation (High)

**Vulnerability**: Update failures could leave no way to recover previous version.

**Fix Implemented**:
- Before installation, current binary is backed up
- Backup preserved as `~/.local/bin/cco.backup`
- Automatic rollback on any failure

**How It Protects You**:
```
Before Update:
    ~/.local/bin/cco → current working version

After Backup:
    ~/.local/bin/cco → current
    ~/.local/bin/cco.backup → same copy (backup)

After Update Success:
    ~/.local/bin/cco → new version
    ~/.local/bin/cco.backup → previous version

After Update Failure:
    ~/.local/bin/cco → restored from backup
    ~/.local/bin/cco.backup → cleared
```

**Manual Rollback**:
```bash
# If you ever need to roll back manually:
mv ~/.local/bin/cco.backup ~/.local/bin/cco
cco --version  # Verify previous version works
```

---

### 8. GitHub Repository Verification (High)

**Vulnerability**: Updates could be fetched from fake repositories.

**Fix Implemented**:
- Repository URL is hardcoded and cannot be changed
- Owner and repository name are verified
- API responses validated for expected fields

**How It Protects You**:
```rust
// Hardcoded repository information
const GITHUB_OWNER: &str = "visiquate";
const GITHUB_REPO: &str = "cco";

// Cannot be overridden via environment
// Always fetches from: https://api.github.com/repos/visiquate/cco
```

---

### 9. Update Channel Isolation (Medium)

**Vulnerability**: Unstable pre-releases could be installed without user knowledge.

**Fix Implemented**:
- Default channel is "stable" (releases only)
- Beta channel available for opt-in testing
- Pre-releases are excluded by default

**How It Protects You**:
```
Default Behavior (Stable):
    - Only installs official releases
    - Pre-releases, RCs excluded

Opt-In Behavior (Beta):
    - Includes pre-releases and RCs
    - User explicitly enables with:
      cco config set updates.channel beta
```

---

### 10. Checksum File Validation (High)

**Vulnerability**: Attacker could provide fake checksums file.

**Fix Implemented**:
- Checksums file is downloaded from same release
- No separate checksum verification (checksums themselves trusted from GitHub)
- Multiple checksums checked if available

**How It Protects You**:
```
Single Source of Truth: GitHub Release
    ↓ Contains binary
    ↓ Contains checksums.sha256
    ↓ Both from same release tag
    ↓ GitHub's HTTPS ensures integrity
```

---

### 11. Network Retry Logic with Exponential Backoff (Medium)

**Vulnerability**: Network failures could leave system in incomplete update state.

**Fix Implemented**:
- Failed downloads automatically retry
- Exponential backoff prevents overwhelming GitHub
- Eventual failure is logged and reported

**How It Protects You**:
```
Download Attempt 1 (fails)
    ↓ Wait 1 second
Download Attempt 2 (fails)
    ↓ Wait 2 seconds
Download Attempt 3 (fails)
    ↓ Wait 4 seconds
Download Attempt 4 (succeeds)
    ↓ Continue with installation

All Attempts Fail:
    ↓ Logged
    ↓ Will retry on next check
    ↓ Current version untouched
```

---

### 12. Secure Update Logging (Medium)

**Vulnerability**: Users might not know if updates have failed or been compromised.

**Fix Implemented**:
- All update operations logged to `~/.cco/logs/updates.log`
- Logs include timestamps, actions, and results
- Sensitive information excluded from logs
- Log file permissions restrict access (0o600)

**How It Protects You**:
```bash
# View all update history
cat ~/.cco/logs/updates.log

# Example log entries:
[2025-11-17 14:32:15] Check started for stable channel
[2025-11-17 14:32:18] Update available: 2025.11.3 (current: 2025.11.2)
[2025-11-17 14:32:45] Downloaded 12.5 MB from GitHub
[2025-11-17 14:32:46] Verified checksum: SHA256 match
[2025-11-17 14:32:47] Created backup: ~/.local/bin/cco.backup
[2025-11-17 14:32:48] Verified new binary works
[2025-11-17 14:32:49] Successfully installed 2025.11.3
[2025-11-17 14:32:50] Cleanup: Removed temporary files
```

**Log Security**:
```bash
# Log files have restricted permissions
ls -la ~/.cco/logs/updates.log
# -rw------- (0o600) - only owner can read
```

## Security Implementation Details

### Threat Model

The auto-update system protects against:

| Threat | Protection |
|--------|-----------|
| Man-in-the-middle attacks | HTTPS-only, checksum verification |
| Tampered binaries | SHA256 verification, binary testing |
| Supply chain attacks | GitHub repository verification, release validation |
| Downgrade attacks | Version comparison, semantic versioning |
| Broken installations | Binary testing, atomic operations, automatic backup |
| Corrupted downloads | Network retry, checksum verification |
| Unauthorized access to temp files | Secure temp directory, 0o600 permissions |
| Silent failures | Comprehensive logging |
| Resource exhaustion | Exponential backoff on retries |

### Update Flow Diagram

```
┌─────────────────────────────────────────────────────┐
│ 1. CHECK FOR UPDATES (daily)                        │
│    ├─ HTTPS connection to GitHub                    │
│    ├─ Fetch release information                     │
│    ├─ Validate version format                       │
│    └─ Compare with current version                  │
└──────────────────┬──────────────────────────────────┘
                   ↓ (if newer version available)
┌─────────────────────────────────────────────────────┐
│ 2. DOWNLOAD BINARY                                  │
│    ├─ Create secure temp directory                  │
│    ├─ HTTPS download from GitHub                    │
│    ├─ Download checksums.sha256                     │
│    └─ Verify SHA256 (critical fix #2)              │
└──────────────────┬──────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────┐
│ 3. EXTRACT & VERIFY                                 │
│    ├─ Extract from tar.gz or zip                    │
│    ├─ Make executable                               │
│    ├─ Test: cco --version                           │
│    └─ Confirm new binary works (critical fix #5)   │
└──────────────────┬──────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────┐
│ 4. BACKUP & REPLACE                                 │
│    ├─ Create backup: ~/.local/bin/cco.backup        │
│    ├─ Atomic rename: new → current                  │
│    └─ Verify final binary works                     │
└──────────────────┬──────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────┐
│ 5. CLEANUP & LOG                                    │
│    ├─ Remove temporary files                        │
│    ├─ Log success to ~/.cco/logs/updates.log        │
│    └─ Update last_update timestamp                  │
└─────────────────────────────────────────────────────┘
```

## Testing Security Features

### Test 1: HTTPS Enforcement

```bash
# Verify all connections use HTTPS
cco update --check

# Check network connections
netstat -an | grep github

# Should see HTTPS (port 443), no HTTP (port 80)
```

### Test 2: Checksum Verification

```bash
# Temporarily corrupt a binary (simulation)
# The auto-update would reject it

# Expected behavior:
# [ERROR] Checksum verification failed! Update aborted for security.
```

### Test 3: Binary Verification

```bash
# Create a broken binary
# If installed, auto-update would detect it and rollback

# Test logs would show:
# [ERROR] Verification of new binary failed
# [INFO] Rolling back to backup
```

### Test 4: Secure Temp Files

```bash
# Check temp directory during update
while cco update --yes; do
    ls -la /tmp/cco-update-* 2>/dev/null
    # Would show: drwx------ (0o700)
    # Others cannot access
done
```

### Test 5: Atomic Operations

```bash
# Kill update mid-way
cco update --yes &
kill $!

# Check state:
cco --version  # Should be either old or new, never broken
```

## Credential Protection

Auto-updates never:

- Read configuration files during download
- Access environment variables with credentials
- Store credentials in temporary files
- Log credentials or API keys
- Modify credential storage

Your API keys and credentials remain in `~/.config/cco/credentials` and are untouched by updates.

## GPG Signature Support (Future Enhancement)

In future releases, we plan to add optional GPG signature verification:

```
GitHub Release
├─ cco-v2025.11.3-linux-x86_64.tar.gz
├─ checksums.sha256
├─ checksums.sha256.sig  (GPG signature)
└─ public-key.asc        (for verification)
```

This would provide cryptographic proof of release authenticity.

## Security Incident Response

If you discover a security vulnerability in auto-updates:

1. **Do not publicly disclose** the vulnerability
2. **Report via GitHub Security**: https://github.com/yourusername/cco/security/advisories
3. **Include**: vulnerability description, impact, how to reproduce
4. **Wait for**: confirmation and timeline for fix
5. **Disclosure**: coordinated with developers

## Compliance and Standards

The auto-update system follows:

- **OWASP**: Secure Software Development
- **CWE**: Common Weakness Enumeration (avoids CWE-427, 434, 502, 506)
- **NIST**: Software Supply Chain Security (SP 800-53)
- **CISA**: Secure Software Development Framework

## Audit and Verification

### Manual Verification

```bash
# See what gets downloaded
cco update --dry-run

# See what would be verified
cco update --verbose --check

# See full logs
tail -100 ~/.cco/logs/updates.log
```

### Checksum Verification (manual)

```bash
# Download and verify manually
VERSION=2025.11.3
URL=https://github.com/yourusername/cco/releases/download/v${VERSION}

wget ${URL}/cco-v${VERSION}-linux-x86_64.tar.gz
wget ${URL}/checksums.sha256

# Verify
sha256sum -c checksums.sha256
```

## Summary

CCO's auto-update system implements comprehensive security:

- **12 security fixes** protect against supply chain attacks
- **HTTPS-only** prevents man-in-the-middle attacks
- **SHA256 verification** ensures integrity
- **Binary testing** prevents broken installations
- **Atomic operations** guarantee consistency
- **Automatic backup** enables rollback
- **Secure logging** enables auditing
- **Default-secure** configuration

Your CCO installation is protected every time it updates.

## Related Documentation

- [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md) - User-friendly guide
- [Auto-Update Architecture](AUTO_UPDATE_ARCHITECTURE.md) - Technical architecture
- [Auto-Update Command Reference](AUTO_UPDATE_COMMAND_REFERENCE.md) - All commands
- [Auto-Update Troubleshooting (Advanced)](AUTO_UPDATE_TROUBLESHOOTING_ADVANCED.md) - Advanced debugging
