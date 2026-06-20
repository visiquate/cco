# Credential System Migration Guide

## Overview

The Claude Orchestra has transitioned from a JavaScript-based file credential manager to a superior Rust-based system accessed via the `cco credentials` CLI. This guide explains the migration, benefits, and how to adopt the new system.

**Status**: Complete migration - JavaScript credential manager deprecated in favor of `cco credentials`

## What Changed

### Previous System (Deprecated)
```bash
# Old JavaScript approach
npm run credentials store <key> <value> [type]
npm run credentials retrieve <key>
npm run credentials list
npm run credentials check-rotation

# Storage: /tmp/credentials.json (file-based)
# Encryption: AES-256-CBC
# Integration: File system only
```

### New System (Current)
```bash
# New Rust-based approach via daemon
cco credentials store <key> <value> [--credential-type type] [--service service] [--description desc]
cco credentials retrieve <key>
cco credentials list
cco credentials check-rotation
cco credentials delete <key>

# Storage: OS-native keyring (Keychain/Secret Service/DPAPI)
# Encryption: AES-256-GCM (FIPS 140-2)
# Integration: HTTP API via daemon, language-agnostic
```

## Security Improvements

| Feature | JavaScript | Rust (New) |
|---------|-----------|-----------|
| **Storage Backend** | File (/tmp) | OS Keyring (Keychain/Secret Service/DPAPI) |
| **Encryption Algorithm** | AES-256-CBC | AES-256-GCM (FIPS 140-2 Compliant) |
| **Key Derivation** | SHA256 | PBKDF2 with configurable iterations |
| **Memory Security** | None | SecretString with zeroization |
| **File Permissions** | 600 | N/A (not file-based) |
| **Audit Logging** | None | Full audit trail with timestamps |
| **Rate Limiting** | None | 10 attempts per 60 seconds |
| **Platform Support** | File only | macOS, Linux, Windows |
| **Rotation Support** | Manual only | Configurable per credential |
| **Metadata** | Limited | Full support (type, service, description) |
| **Temp File Risk** | High | Eliminated |

## Detailed Benefits

### 1. OS Keyring Integration
**What**: Credentials stored in platform-native secure storage
- **macOS**: Keychain (same as passwords, SSH keys, certificates)
- **Linux**: Secret Service (systemd-user-secrets or KDE Wallet)
- **Windows**: DPAPI (Data Protection API)

**Why**: Operating system manages encryption, key derivation, and access control

### 2. FIPS 140-2 Compliance
**What**: AES-256-GCM encryption (government-certified)
**Why**:
- Stronger than AES-256-CBC (CBC mode has vulnerabilities)
- Includes authenticated encryption (prevents tampering)
- Required for many compliance frameworks (HIPAA, PCI-DSS, SOC 2)

### 3. Secure Memory Management
**What**: SecretString zeroization
**Why**:
- Automatically clears sensitive data from memory
- Prevents cold-boot attacks
- Eliminates password exposure in core dumps

### 4. Comprehensive Audit Logging
**What**: Every credential operation is logged
**Why**:
- Trace who accessed which credential and when
- Security incident investigation
- Compliance requirements (SOC 2, HIPAA)

**Example log entry**:
```json
{
  "timestamp": "2025-11-28T14:32:15Z",
  "operation": "retrieve",
  "key": "db_password",
  "service": "production",
  "user": "deploy_agent",
  "status": "success"
}
```

### 5. Rate Limiting
**What**: 10 attempts per 60 seconds per credential
**Why**: Prevents brute-force attacks and resource exhaustion

### 6. Automatic Rotation Tracking
**What**: Configure rotation policies per credential
**Why**:
- Know which credentials are overdue for rotation
- Automated reminders
- Compliance frameworks require rotation tracking

### 7. Zero File-Based Vulnerabilities
**What**: No temporary files, no encrypted JSON storage
**Why**:
- Eliminates file permission misconfigurations
- No accidental credential exposure
- No recovery of deleted credentials from disk

## Migration Path

### Step 1: Ensure Daemon is Running
```bash
# Start the CCO daemon
cco daemon start

# Verify it's running
cco daemon status
```

### Step 2: Migrate Existing Credentials
```bash
# Automatic migration from old /tmp/credentials.json
cco credentials migrate --from /tmp/credentials.json

# Or manual migration
cco credentials store old_key_1 "value1" --description "Migrated from JS system"
cco credentials store old_key_2 "value2" --credential-type api-token --service github
```

### Step 3: Update Agent Scripts
Replace all references to the old system:

**Old** (in agent code):
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js ...
npm run credentials retrieve db_password
```

**New** (in agent code):
```bash
cco knowledge search "task"
cco credentials retrieve db_password
```

### Step 4: Verify Migration
```bash
# Check all credentials are accessible
cco credentials list

# Test retrieval
cco credentials retrieve any_key

# Verify logging is working
cco credentials check-rotation
```

### Step 5: Clean Up (Optional)
```bash
# After verifying all credentials work:
# 1. Delete old /tmp/credentials.json
# 2. Remove npm credentials script from package.json
# 3. Remove credential-manager.js from src/
```

## New CLI Commands

### Store a Credential
```bash
# Basic storage
cco credentials store api_key "sk-1234567890"

# With all metadata
cco credentials store api_key "sk-1234567890" \
  --credential-type api-token \
  --service github \
  --description "GitHub personal access token for CI/CD"
```

**Credential Types**:
- `api-token` - API keys, tokens, bearer tokens
- `database` - Database passwords, connection strings
- `generic` - Any other credential

### Retrieve a Credential
```bash
# Retrieve and display
cco credentials retrieve db_password

# Returns: `<actual_password_value>`
```

### List Credentials
```bash
# Show all credential keys
cco credentials list

# Returns:
# - api_key (api-token, github)
# - db_password (database, production)
# - slack_webhook (api-token, notifications)
```

### Check Rotation Status
```bash
# Find credentials needing rotation
cco credentials check-rotation

# Returns:
# Credentials due for rotation:
# - db_password (last rotated: 2025-10-28, 31 days ago)
# - legacy_api_key (last rotated: 2025-06-15, 166 days ago)
```

### Delete a Credential
```bash
# Remove a credential
cco credentials delete old_token

# Confirm: Credential 'old_token' deleted successfully
```

## Integration with Agents

### Knowledge Manager Coordination
```bash
# Agents use cco commands instead of npm/node
cco knowledge search "authentication flow"
cco knowledge store "Implemented OAuth2" --type decision --agent python-expert

# Credential management
cco credentials retrieve salesforce_token
cco credentials store new_api_key "..." --service integration
```

### Example Agent Workflow
```bash
# 1. Search knowledge base
ARCHITECTURE=$(cco knowledge search "auth architecture")

# 2. Retrieve credentials
API_TOKEN=$(cco credentials retrieve github_token)

# 3. Do work...

# 4. Store results in knowledge base
cco knowledge store "Implemented JWT auth" --type decision --agent tdd-coding-agent

# 5. If needed, store new credentials
cco credentials store jwt_secret "generated-secret" \
  --credential-type generic \
  --service auth \
  --description "JWT signing key"
```

## Troubleshooting

### Daemon Not Running
```bash
# Error: "Daemon not running. Start it with: cco daemon start"

# Solution:
cco daemon start
cco daemon status
```

### Permission Denied on Keyring
```bash
# Error: "Failed to access system keyring"

# Possible causes:
# 1. Keyring service not running
# 2. User doesn't have keyring access permissions

# Solutions:
# Linux: Ensure secret-service is running
systemctl --user status secrets-service

# macOS: Ensure Keychain is available
security show-keychain-info
```

### Credential Not Found
```bash
# Error: "Credential 'my_key' not found"

# Debug:
cco credentials list  # Check if key exists

# If missing, re-store it:
cco credentials store my_key "value"
```

### Rate Limit Exceeded
```bash
# Error: "Rate limit exceeded. Try again in 45 seconds"

# Explanation: 10 attempts per 60 seconds per credential
# Solution: Wait and retry

# For agents: Implement exponential backoff
```

## Compliance & Standards

### FIPS 140-2 Compliance
- AES-256-GCM encryption certified for government use
- Compliant with HIPAA, PCI-DSS, SOC 2 requirements

### Audit Requirements
- Full audit logging of all operations
- Timestamps with user context
- Immutable audit trail (daemon-managed)

### Rotation Policies
- Per-credential rotation tracking
- Automatic notifications for overdue rotations
- Configurable rotation intervals (30, 60, 90 days)

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Store credential | ~10ms | Includes OS keyring write |
| Retrieve credential | ~5ms | From system cache |
| List credentials | ~50ms | Indexes all keys |
| Check rotation | ~100ms | Full scan and comparison |
| Rate limit check | <1ms | In-memory tracking |

## Future Enhancements

1. **Cloud Backup**: Optional encrypted backup to S3/cloud storage
2. **Credential Sharing**: Cross-agent credential sharing via broker
3. **MFA Support**: Multi-factor authentication for sensitive credentials
4. **Templates**: Pre-configured templates for common services (AWS, GitHub, Salesforce)
5. **Automated Rotation**: Auto-rotate credentials based on policies
6. **Web UI Dashboard**: Visual credential management interface

## FAQ

### Q: What happens to my old /tmp/credentials.json?
A: Use `cco credentials migrate` to import it into the new system. After verification, you can safely delete the old file.

### Q: Can I still use environment variables?
A: Not recommended. Environment variables leak in process listings and logs. Use `cco credentials` instead.

### Q: Does this require a running daemon?
A: Yes, the daemon must be running (`cco daemon start`). It provides the HTTP API for all credential operations.

### Q: What if the daemon crashes?
A: Credentials are safely stored in OS keyring. Just restart the daemon with `cco daemon start`.

### Q: Can multiple machines share credentials?
A: Not directly - each machine has its own OS keyring. For shared credentials, use a secrets management service (AWS Secrets Manager, HashiCorp Vault, etc.) or configure credential sync mechanisms.

### Q: What about emergency access if I forget a credential?
A: Once stored in OS keyring, the only way to access is through `cco credentials retrieve`. There's no "master password" recovery - this is intentional for security.

## Migration Checklist

- [ ] Understand new CLI commands
- [ ] Start CCO daemon (`cco daemon start`)
- [ ] Run migration (`cco credentials migrate`)
- [ ] Test credential retrieval (`cco credentials list` and `retrieve`)
- [ ] Update all agent scripts to use `cco credentials`
- [ ] Update all documentation references
- [ ] Test with full agent workflow
- [ ] Delete old `/tmp/credentials.json` after verification
- [ ] Remove npm credentials script from package.json
- [ ] Remove credential-manager.js from src/
- [ ] Commit documentation updates
- [ ] Announce migration to team

## Support & Questions

For issues or questions about the new credential system:

1. Check this guide's troubleshooting section
2. Review `cco credentials --help`
3. Check CCO daemon logs: `cco daemon logs`
4. File an issue in the project repository

---

**Version**: 1.0
**Last Updated**: 2025-11-28
**Status**: Active - All projects should migrate to `cco credentials`
