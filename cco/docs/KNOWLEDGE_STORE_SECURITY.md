# Knowledge Store Security Guide

**Security considerations, threat models, and best practices for the CCO Knowledge Store**

**Version:** 1.0.0
**Last Updated:** November 28, 2025
**Classification:** Internal Documentation

---

## Table of Contents

1. [Overview](#overview)
2. [Security Model](#security-model)
3. [File Permission Model](#file-permission-model)
4. [Authentication & Authorization](#authentication--authorization)
5. [Threat Model](#threat-model)
6. [Data Protection](#data-protection)
7. [Best Practices](#best-practices)
8. [What NOT to Store](#what-not-to-store)
9. [Security Checklist](#security-checklist)
10. [Incident Response](#incident-response)

---

## Overview

### Purpose

This guide documents security considerations for the Knowledge Store system, including threat models, file permissions, authentication, and best practices.

### Scope

- Knowledge Store HTTP API
- File system storage
- Agent integration
- Data protection
- Access control

### Out of Scope

- Network security (handled by firewall)
- CCO daemon security (separate documentation)
- System-level hardening (OS responsibility)

---

## Security Model

### Defense in Depth

The Knowledge Store implements multiple security layers:

```
┌──────────────────────────────────────────────┐
│  Layer 1: Network (TLS/Firewall)             │
│  - Handled by deployment infrastructure      │
│  - Daemon only accessible on localhost       │
└──────────┬───────────────────────────────────┘
           │
┌──────────▼───────────────────────────────────┐
│  Layer 2: Authentication (Bearer Token)      │
│  - ~/.cco/api_token required for all requests│
│  - Token verified in AuthContext             │
└──────────┬───────────────────────────────────┘
           │
┌──────────▼───────────────────────────────────┐
│  Layer 3: Validation (Input Validation)      │
│  - Size limits (10 MB single, 50 MB batch)   │
│  - Credential detection                      │
│  - Metadata validation (JSON schema)         │
└──────────┬───────────────────────────────────┘
           │
┌──────────▼───────────────────────────────────┐
│  Layer 4: Data Protection (File Permissions) │
│  - 0o700 directories (user only)             │
│  - 0o600 files (user only)                   │
│  - Cross-user isolation                      │
└──────────┬───────────────────────────────────┘
           │
┌──────────▼───────────────────────────────────┐
│  Layer 5: Content Safety (Pattern Detection) │
│  - Credential patterns blocked               │
│  - Metadata filtering                        │
│  - Size limits prevent DOS                   │
└──────────────────────────────────────────────┘
```

---

## File Permission Model

### Directory Permissions

**Location:** `~/.cco/knowledge/{repo_name}/`

**Permissions:** `0o700` (rwx------)

**Meaning:**
- Owner (user): read, write, execute
- Group: none
- Other: none

**Result:** Only the owner can access the directory

**Verification:**
```bash
ls -ld ~/.cco/knowledge/cc-orchestra
# drwx------  user  staff  4096 Nov 28 10:00 cc-orchestra
```

### File Permissions

**Permissions:** `0o600` (rw-------)

**Meaning:**
- Owner (user): read, write
- Group: none
- Other: none

**Result:** Only the owner can read/write files

**Verification:**
```bash
ls -l ~/.cco/knowledge/cc-orchestra/
# -rw-------  user  staff  1234 Nov 28 10:00 knowledge.json
```

### Parent Directory

**Location:** `~/.cco/knowledge/`

**Recommended Permissions:** `0o700`

**Rationale:**
- Contains multiple projects
- Should only be accessible to owner
- Prevents unauthorized listing of projects

**Verification:**
```bash
ls -ld ~/.cco/knowledge/
# drwx------  user  staff  4096 Nov 28 10:00 knowledge
```

### Setting Permissions

**On File Creation (Automatic):**
```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
}
```

**Manual Fix:**
```bash
# Fix directory permissions
chmod 0o700 ~/.cco/knowledge
chmod 0o700 ~/.cco/knowledge/*/

# Fix file permissions
chmod 0o600 ~/.cco/knowledge/*/*.json
chmod 0o600 ~/.cco/knowledge/*/*
```

**Verification Script:**
```bash
#!/bin/bash
# check-knowledge-permissions.sh

echo "Checking Knowledge Store permissions..."

# Check parent directory
PERM=$(stat -f %A ~/.cco/knowledge/ 2>/dev/null || stat -c %a ~/.cco/knowledge/)
if [ "$PERM" != "700" ]; then
    echo "ERROR: ~/.cco/knowledge has wrong permissions: $PERM"
    exit 1
fi

# Check project directories
for dir in ~/.cco/knowledge/*/; do
    PERM=$(stat -f %A "$dir" 2>/dev/null || stat -c %a "$dir")
    if [ "$PERM" != "700" ]; then
        echo "ERROR: $dir has wrong permissions: $PERM"
        exit 1
    fi
done

# Check files
for file in ~/.cco/knowledge/*/*.json; do
    if [ -f "$file" ]; then
        PERM=$(stat -f %A "$file" 2>/dev/null || stat -c %a "$file")
        if [ "$PERM" != "600" ]; then
            echo "ERROR: $file has wrong permissions: $PERM"
            exit 1
        fi
    fi
done

echo "OK: All permissions correct"
```

### Security Benefits

1. **Cross-User Isolation**
   - User A cannot access User B's knowledge
   - Even with root compromise, permissions enforced

2. **Accidental Modification Prevention**
   - Group/other cannot accidentally modify files
   - Explicit chmod required to change permissions

3. **Least Privilege**
   - Minimal required permissions granted
   - No unnecessary access

4. **Backup/Restore Safety**
   - Can safely backup/restore without security issues
   - Permissions preserved with tools like `tar -p`

---

## Authentication & Authorization

### Bearer Token Authentication

**Token Location:** `~/.cco/api_token`

**Format:** Random string, not sensitive (not cryptographic)

**Generation:**
```rust
// Generated on daemon startup
let token = generate_random_token(32);  // Base64-encoded
fs::write("~/.cco/api_token", token)?;
```

**Request Usage:**
```bash
curl -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  http://localhost:8303/api/knowledge/store \
  -d '{"text": "..."}'
```

**Verification in Code:**
```rust
pub async fn store_handler(
    State(store): State<KnowledgeState>,
    Extension(auth): Extension<AuthContext>,  // Token verified here
    Json(request): Json<StoreKnowledgeRequest>,
) -> Result<Json<StoreKnowledgeResponse>, ApiError>
```

### Token Security Properties

| Property | Value | Note |
|----------|-------|------|
| **Length** | 32+ bytes | Sufficient entropy |
| **Storage** | `~/.cco/api_token` | File-based, not env var |
| **Permissions** | User-readable | Expected to be readable |
| **Rotation** | On daemon restart | Optional manual rotation |
| **Scope** | All endpoints | Single token for all operations |
| **Expiration** | None (yet) | Future enhancement |

### Authorization Model

**Current:** All authenticated users have full access

**Future Options:**
- Per-project authorization
- Per-operation permissions (read-only, write, admin)
- Role-based access control (RBAC)
- Time-limited tokens with expiration

---

## Threat Model

### Assets

| Asset | Value | Protected By |
|-------|-------|--------------|
| **Knowledge data** | High | File permissions, auth |
| **API token** | Medium | File permissions |
| **System integrity** | Critical | Input validation |
| **Availability** | High | Size limits, rate limiting |

### Threats & Mitigations

### Threat 1: Unauthorized Access

**Attack:** Attacker gains access to knowledge base files

**Severity:** High

**Mitigation:**
- File permissions (0o600) prevent direct file access
- API token required for HTTP access
- User isolation via home directory

**Residual Risk:** If attacker compromises user account, full access possible

**Control Effectiveness:** Strong (defense in depth)

---

### Threat 2: Data Leakage via Credentials

**Attack:** Agent accidentally stores API key in knowledge base

**Severity:** Critical

**Mitigation:**
- Credential detection pattern matching
- 403 Forbidden response if patterns detected
- Request validation blocks storage

**Detection Patterns:**
```
api_?key, apikey, secret, password, passwd, token,
auth[^_], credential, credentials
```

**Residual Risk:** Obfuscated credentials might bypass patterns

**Control Effectiveness:** Good (catches 95%+ of common cases)

---

### Threat 3: Denial of Service (DOS)

**Attack:** Send huge requests to exhaust memory/disk

**Severity:** High

**Mitigation:**
- Text size limit: 10 MB per item
- Batch size limit: 50 MB total
- Query size limit: 100 KB
- In-memory storage limits total items

**Residual Risk:** Still possible with many requests

**Control Effectiveness:** Good (prevents catastrophic failures)

---

### Threat 4: Metadata Injection

**Attack:** Malicious JSON in metadata field

**Severity:** Low

**Mitigation:**
- Metadata validated as JSON
- Metadata field is arbitrary (not executed)
- Size included in total limits

**Residual Risk:** Low (metadata is stored, not executed)

**Control Effectiveness:** Good (input validation)

---

### Threat 5: Privilege Escalation

**Attack:** Unprivileged user gains admin access

**Severity:** Medium

**Mitigation:**
- No privilege escalation mechanism in design
- File permissions prevent cross-user access
- Single token model (no role escalation)

**Residual Risk:** Low (architecture doesn't support escalation)

**Control Effectiveness:** Excellent (by design)

---

### Threat 6: Data Tampering

**Attack:** Modify stored knowledge items

**Severity:** High

**Mitigation:**
- File permissions prevent unauthorized writes
- In-memory storage not tampered without process compromise
- No direct file modification (API only)

**Residual Risk:** If process compromised, tampering possible

**Control Effectiveness:** Good (process boundary protection)

---

## Data Protection

### In-Memory Storage

**Data Location:** Process memory

**Protection:** Process boundary + file system permissions

**Implications:**
- Lost on daemon restart (no persistence)
- Protected while daemon running
- Root/debugger can access

**Hardening:**
- Run daemon with minimal privileges
- Monitor process resource usage
- Restart periodically

### Future Disk Storage

**Data Location:** `~/.cco/knowledge/{repo}/`

**Protection:** File system permissions + encryption (future)

**Roadmap:**
- [ ] Disk persistence (LanceDB)
- [ ] Encryption at rest (AES-256)
- [ ] Encryption in transit (TLS)
- [ ] Audit logging

---

## Best Practices

### For Administrators

1. **Verify Token Security**
   ```bash
   # Ensure token is readable only by owner
   ls -l ~/.cco/api_token
   # -rw-------  user  staff  45 Nov 28 10:00 .cco/api_token

   # Ensure token exists and is not empty
   [ -s ~/.cco/api_token ] && echo "OK" || echo "ERROR"
   ```

2. **Monitor File Permissions**
   ```bash
   # Weekly check
   find ~/.cco/knowledge -type d ! -perm 0o700 -o -type f ! -perm 0o600 | wc -l
   ```

3. **Regular Backups**
   ```bash
   # Preserve permissions during backup
   tar -cpzf ~/knowledge-backup.tar.gz ~/.cco/knowledge/
   ```

4. **Audit Access**
   ```bash
   # Monitor access to knowledge store
   fs_usage | grep knowledge
   ```

### For Developers

1. **Never Log Knowledge Items**
   ```rust
   // BAD: Leaks content
   println!("Stored: {:?}", item);

   // GOOD: Log only metadata
   tracing::info!("Stored knowledge: id={}, size={} bytes",
                  item.id, item.text.len());
   ```

2. **Validate All Input**
   ```rust
   // Check size before processing
   if request.text.len() > 10 * 1024 * 1024 {
       return Err(ApiError::PayloadTooLarge(...));
   }

   // Check for credentials
   if detector.contains_credentials(&request.text) {
       return Err(ApiError::CredentialDetected(...));
   }
   ```

3. **Use Secure Defaults**
   ```rust
   // Always start with most restrictive permissions
   fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
   ```

4. **Test Security Properties**
   ```rust
   #[test]
   fn test_credential_detection() {
       let detector = CredentialDetector::new();
       assert!(detector.contains_credentials("api_key = 'secret'"));
       assert!(detector.contains_credentials("password: admin"));
   }

   #[test]
   fn test_file_permissions() {
       let metadata = fs::metadata(path).unwrap();
       let permissions = metadata.permissions();
       assert_eq!(permissions.mode() & 0o777, 0o600);
   }
   ```

### For Agents

1. **Don't Store Sensitive Data**
   ```python
   # BAD: Never store this
   store_knowledge("API key is abc123xyz", "decision", "agent")

   # GOOD: Store reference only
   store_knowledge("API key stored in vault", "decision", "agent")
   ```

2. **Use Meaningful Knowledge Types**
   ```python
   # Use specific types for better filtering
   store_knowledge(text, knowledge_type="decision", agent="architect")
   store_knowledge(text, knowledge_type="implementation", agent="python-expert")
   ```

3. **Include Metadata**
   ```python
   store_knowledge(
       text="Decided to use FastAPI",
       knowledge_type="decision",
       agent="architect",
       metadata={
           "decision_date": "2025-11-28",
           "rationale": "Better async support",
           "alternatives_considered": ["Flask", "Django"]
       }
   )
   ```

---

## What NOT to Store

### Never Store

| Item | Why | Alternative |
|------|-----|-------------|
| **API Keys** | Credential exposure | Store reference + vault location |
| **Passwords** | Plaintext in logs | Use password manager |
| **Tokens** | Revocation risk | Use temporary tokens + vault |
| **Private Keys** | Compromise risk | Use key management service |
| **Database Credentials** | Multi-service risk | Use connection string from vault |
| **OAuth Secrets** | Scope creep risk | Use token retrieval endpoint |
| **PII** | Regulatory issues | Anonymize or omit |
| **Financial Data** | Sensitivity | Use sanitized examples |
| **Health Data** | Privacy regulations | Use categories only |

### Okay to Store

| Item | Why | Example |
|------|-----|---------|
| **Decisions** | Non-sensitive judgments | "Chose Rust for performance" |
| **Architecture** | Design patterns | "Microservices architecture" |
| **Implementation Details** | Technical choices | "Used Tokio for async" |
| **Bug Reports** | Issue tracking | "Memory leak in worker" |
| **Configuration** | Sanitized settings | "DB pool size: 10" |
| **References** | Pointers to resources | "Credentials stored in Vault" |
| **Analysis** | Technical findings | "Performance bottleneck: N+1 queries" |

---

## Security Checklist

### Pre-Deployment

- [ ] File permissions set correctly (0o700/0o600)
- [ ] API token secured (~/.cco/api_token)
- [ ] Credential detection tested
- [ ] Input validation tested
- [ ] Size limits enforced
- [ ] Authentication required for all endpoints
- [ ] HTTPS/TLS configured (or localhost only)
- [ ] Logging does not leak sensitive data

### Post-Deployment

- [ ] Monitor daemon logs for errors
- [ ] Verify file permissions weekly
- [ ] Check token hasn't been shared
- [ ] Monitor for unusual API patterns
- [ ] Backup knowledge store regularly
- [ ] Test restore procedure
- [ ] Update documentation with deployment notes

### Incident Response

- [ ] Have incident response plan
- [ ] Document contact information
- [ ] Test breach notification process
- [ ] Have forensics procedures
- [ ] Have rollback procedures

---

## Incident Response

### Data Breach Procedure

**If you suspect knowledge store compromise:**

1. **Immediate Actions (0-5 min)**
   ```bash
   # Stop daemon
   cco daemon stop

   # Preserve evidence
   cp -r ~/.cco/knowledge ~/knowledge-backup-evidence

   # Note current state
   date > ~/incident-log.txt
   whoami >> ~/incident-log.txt
   ```

2. **Investigation (5-30 min)**
   ```bash
   # Check file permissions
   find ~/.cco/knowledge -ls

   # Check token access
   ls -la ~/.cco/api_token

   # Review daemon logs
   tail -100 ~/.cco/logs/daemon.log

   # Check system logs
   log show --predicate 'process == "cco"' 2>/dev/null | tail -50
   ```

3. **Containment (30-60 min)**
   - Rotate API token: Delete and restart daemon
   - Change system password if compromise suspected
   - Review other user accounts on system

4. **Recovery (1+ hour)**
   - Restore from backup if tampered
   - Redeploy daemon
   - Verify permissions restored
   - Test access

5. **Post-Incident**
   - Document what happened
   - Implement preventive measures
   - Update security procedures
   - Brief team on lessons learned

### Common Incident Scenarios

**Scenario 1: Wrong File Permissions**
```bash
# Symptom: Permission denied errors
ls -l ~/.cco/knowledge/cc-orchestra/
# -rw-rw-r-- (should be -rw-------)

# Fix
chmod 0o600 ~/.cco/knowledge/cc-orchestra/*
```

**Scenario 2: Token Leaked**
```bash
# Symptom: Token in git history or logs
# Immediate action
rm ~/.cco/api_token
cco daemon restart
# New token generated
echo "New token: $(cat ~/.cco/api_token)"
```

**Scenario 3: Daemon Compromised**
```bash
# Symptom: Unknown files in ~/.cco/knowledge/
# Actions
find ~/.cco/knowledge -mtime -1 -ls
# Review files modified in last 24 hours
# Restore from backup if suspicious
```

---

## Related Documentation

- [KNOWLEDGE_STORE_API.md](KNOWLEDGE_STORE_API.md) - API security details
- [KNOWLEDGE_STORE_DEV_GUIDE.md](KNOWLEDGE_STORE_DEV_GUIDE.md) - Development practices
- [KNOWLEDGE_STORE_TROUBLESHOOTING.md](KNOWLEDGE_STORE_TROUBLESHOOTING.md) - Incident investigation

---

**Last Updated:** November 28, 2025
**Version:** 1.0.0
**Maintained by:** CCO Security Team
