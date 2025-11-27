# CCO Private Distribution - Architecture Documentation

## System Overview

The CCO private distribution system provides secure, authenticated binary distribution with access control, usage tracking, and enterprise-grade security features.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        End User                                  │
│                                                                   │
│  ┌──────────────┐         ┌──────────────┐                      │
│  │  CCO Binary  │         │   Browser    │                      │
│  │  (Rust CLI)  │         │ (Auth Flow)  │                      │
│  └──────┬───────┘         └──────┬───────┘                      │
└─────────┼───────────────────────┼──────────────────────────────┘
          │                       │
          │ HTTPS/TLS             │ HTTPS/TLS
          │                       │
          ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                   cco-api.visiquate.com                          │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    FastAPI Application                    │  │
│  │                                                            │  │
│  │  ┌───────────────┐   ┌────────────────┐   ┌───────────┐ │  │
│  │  │ Auth Proxy    │   │ Releases API   │   │ Analytics │ │  │
│  │  │ - Device flow │   │ - Latest       │   │ - Metrics │ │  │
│  │  │ - Token mgmt  │   │ - By version   │   │ - Logs    │ │  │
│  │  │ - Refresh     │   │ - Download URL │   │ - Audit   │ │  │
│  │  └───────┬───────┘   └────────┬───────┘   └─────┬─────┘ │  │
│  └──────────┼──────────────────────┼─────────────────┼───────┘  │
│             │                      │                 │           │
│             │                      │                 │           │
└─────────────┼──────────────────────┼─────────────────┼───────────┘
              │                      │                 │
              │ OIDC                 │ S3 API          │ SQL
              ▼                      ▼                 ▼
       ┌─────────────┐      ┌──────────────┐   ┌──────────────┐
       │  Authentik  │      │  Cloudflare  │   │  PostgreSQL  │
       │    OIDC     │      │      R2      │   │   Database   │
       │  Provider   │      │   Storage    │   │              │
       └─────────────┘      └──────────────┘   └──────────────┘
```

## Components

### 1. CCO Client (Rust)

**Location**: `/Users/brent/git/cc-orchestra/cco/src/`

**Purpose**: Command-line interface for CCO operations

**Key Modules**:

```rust
cco/src/
├── auth/
│   ├── mod.rs              // Public API
│   ├── device_flow.rs      // OIDC device flow client
│   └── token_storage.rs    // Secure token persistence
├── auto_update/
│   ├── mod.rs              // Update orchestration
│   ├── releases_api.rs     // Authenticated API client
│   └── updater.rs          // Binary update logic
└── main.rs                 // CLI entry point
```

**Responsibilities**:
- User authentication (OIDC device flow)
- Token management (storage, refresh)
- Release checking
- Binary downloading
- Checksum verification
- Atomic binary replacement

**Key Technologies**:
- `reqwest` - HTTP client
- `serde/serde_json` - Serialization
- `chrono` - Timestamp handling
- `sha2` - Checksum verification
- `dirs` - Config directory location

### 2. API Server (cco-api.visiquate.com)

**Technology**: Python FastAPI

**Location**: `/home/cco-api/cco-api/app/`

**Purpose**: Authentication proxy and release management

**Endpoints**:

#### Authentication Endpoints

```python
POST /auth/device/code
# Request: (empty)
# Response: {
#   "device_code": "abc123...",
#   "user_code": "ABCD-EFGH",
#   "verification_uri": "https://auth.visiquate.com/device",
#   "expires_in": 600,
#   "interval": 5
# }
# Purpose: Initiate OIDC device flow

POST /auth/device/token
# Request: {"device_code": "abc123..."}
# Response: {
#   "access_token": "eyJ...",
#   "refresh_token": "def456...",
#   "expires_in": 3600,
#   "token_type": "Bearer"
# }
# Purpose: Poll for device flow completion

POST /auth/token/refresh
# Request: {"refresh_token": "def456..."}
# Response: {
#   "access_token": "eyJ...",
#   "refresh_token": "ghi789...",
#   "expires_in": 3600,
#   "token_type": "Bearer"
# }
# Purpose: Refresh expired access token
```

#### Releases Endpoints

```python
GET /releases/latest?channel={stable|beta}
# Headers: Authorization: Bearer {token}
# Response: {
#   "version": "2025.11.2",
#   "release_notes": "...",
#   "platforms": [
#     {
#       "platform": "darwin-arm64",
#       "filename": "cco-v2025.11.2-darwin-arm64.tar.gz",
#       "size": 45000000,
#       "checksum": "sha256:abc123..."
#     }
#   ]
# }
# Purpose: Get latest release for channel

GET /releases/{version}
# Headers: Authorization: Bearer {token}
# Response: (same as /releases/latest)
# Purpose: Get specific version

GET /download/{version}/{platform}
# Headers: Authorization: Bearer {token}
# Response: {
#   "url": "https://releases.visiquate.com/cco/...",
#   "expires_in": 300
# }
# Purpose: Get presigned download URL
```

#### Management Endpoints

```python
GET /health
# Response: {
#   "status": "healthy",
#   "service": "cco-api",
#   "version": "1.0.0"
# }
# Purpose: Health check for monitoring
```

**Middleware**:
- Token verification (all authenticated endpoints)
- Group membership check (`cco-users`)
- Rate limiting (per user)
- Request logging
- Error handling

**Dependencies**:
- `fastapi` - Web framework
- `uvicorn` - ASGI server
- `httpx` - HTTP client (for Authentik)
- `boto3` - R2 client
- `pydantic` - Data validation

### 3. Authentik OIDC Provider

**URL**: `https://auth.visiquate.com`

**Purpose**: Identity and access management

**Configuration**:

```yaml
Provider: OAuth2/OIDC
Name: CCO CLI
Client Type: Public
Client ID: <generated>
Client Secret: <generated>
Redirect URIs: urn:ietf:wg:oauth:2.0:oob
Scopes:
  - openid
  - profile
  - email
  - groups
Token Validity:
  - Access: 1 hour
  - Refresh: 30 days
  - Device code: 10 minutes
```

**Groups**:
- `cco-users` - Standard users with download access
- `cco-admins` - Administrators with full access

**Flows**:

1. **Device Authorization Flow** (RFC 8628)
   ```
   1. Client → Authentik: POST /application/o/device/
      Response: {device_code, user_code, verification_uri}

   2. User visits verification_uri in browser
      User enters user_code
      User authenticates (username/password + MFA)
      User authorizes CCO CLI

   3. Client → Authentik: POST /application/o/token/ (polling)
      Request: {grant_type: device_code, device_code}
      Response: {access_token, refresh_token} when authorized
   ```

2. **Token Refresh Flow**
   ```
   Client → Authentik: POST /application/o/token/
   Request: {grant_type: refresh_token, refresh_token}
   Response: {access_token, refresh_token}
   ```

3. **Token Introspection**
   ```
   API → Authentik: GET /application/o/userinfo/
   Headers: Authorization: Bearer {access_token}
   Response: {
     "sub": "user123",
     "email": "user@example.com",
     "groups": ["cco-users"],
     "name": "John Doe"
   }
   ```

### 4. Cloudflare R2 Storage

**Bucket**: `cco-releases`

**Purpose**: Binary storage and distribution

**Directory Structure**:

```
cco-releases/
├── v2025.11.1/
│   ├── cco-v2025.11.1-darwin-arm64.tar.gz
│   ├── cco-v2025.11.1-darwin-x86_64.tar.gz
│   ├── cco-v2025.11.1-linux-x86_64.tar.gz
│   ├── cco-v2025.11.1-linux-aarch64.tar.gz
│   ├── cco-v2025.11.1-windows-x86_64.zip
│   └── checksums.sha256
├── v2025.11.2/
│   ├── cco-v2025.11.2-darwin-arm64.tar.gz
│   ├── cco-v2025.11.2-darwin-x86_64.tar.gz
│   ├── cco-v2025.11.2-linux-x86_64.tar.gz
│   ├── cco-v2025.11.2-linux-aarch64.tar.gz
│   ├── cco-v2025.11.2-windows-x86_64.zip
│   └── checksums.sha256
└── latest/
    ├── stable -> ../v2025.11.2/
    └── beta -> ../v2025.12.1-beta.1/
```

**Access Control**:
- Private bucket (no public access)
- Presigned URLs only (5 minute expiration)
- Generated by cco-api with user authentication

**Presigned URL Format**:
```
https://releases.visiquate.com/cco/v2025.11.2/cco-v2025.11.2-darwin-arm64.tar.gz
  ?X-Amz-Algorithm=AWS4-HMAC-SHA256
  &X-Amz-Credential=...
  &X-Amz-Date=20251124T143000Z
  &X-Amz-Expires=300
  &X-Amz-SignedHeaders=host
  &X-Amz-Signature=...
```

**Security**:
- HTTPS only
- Domain validation (`releases.visiquate.com`)
- Short expiration (5 minutes)
- One-time use (not cached by client)
- Size limits (100MB max)

### 5. PostgreSQL Database

**Purpose**: Release metadata and analytics

**Schema**:

```sql
-- Releases table
CREATE TABLE releases (
    id SERIAL PRIMARY KEY,
    version VARCHAR(20) NOT NULL UNIQUE,
    channel VARCHAR(10) NOT NULL DEFAULT 'stable',
    release_notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    published_at TIMESTAMP,
    deprecated_at TIMESTAMP,
    INDEX idx_version (version),
    INDEX idx_channel_published (channel, published_at)
);

-- Platforms table
CREATE TABLE platforms (
    id SERIAL PRIMARY KEY,
    release_id INTEGER NOT NULL REFERENCES releases(id),
    platform VARCHAR(20) NOT NULL,
    filename VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    checksum VARCHAR(128) NOT NULL,
    r2_key VARCHAR(200) NOT NULL,
    UNIQUE(release_id, platform)
);

-- Downloads table (analytics)
CREATE TABLE downloads (
    id SERIAL PRIMARY KEY,
    release_id INTEGER NOT NULL REFERENCES releases(id),
    platform_id INTEGER NOT NULL REFERENCES platforms(id),
    user_id VARCHAR(100) NOT NULL,
    user_email VARCHAR(255),
    user_groups TEXT[],
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT,
    INDEX idx_timestamp (timestamp),
    INDEX idx_user (user_id),
    INDEX idx_release (release_id)
);

-- Authentication logs
CREATE TABLE auth_logs (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    user_id VARCHAR(100),
    user_email VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    INDEX idx_timestamp (timestamp),
    INDEX idx_user (user_id),
    INDEX idx_event (event_type)
);
```

## Data Flow

### Authentication Flow

```
┌──────┐
│ User │
└───┬──┘
    │
    │ 1. cco login
    ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 2. POST /auth/device/code
      ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 3. POST /application/o/device/
       ▼
┌──────────────┐
│  Authentik   │
└──────┬───────┘
       │
       │ 4. {device_code, user_code, verification_uri}
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 5. Return to client
       ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 6. Display: "Visit https://... and enter ABCD-EFGH"
      │
┌─────▼──────┐
│  Browser   │◄─────┐
└─────┬──────┘      │
      │             │ 7. User visits URL
      │             └─────────────────┐
      │ 8. Enter code: ABCD-EFGH      │
      │                               │
      │ 9. Login (username/password)  │
      │                               │
      │ 10. Authorize CCO             │
      ▼                               │
┌──────────────┐                      │
│  Authentik   │                      │
└──────┬───────┘                      │
       │                              │
       │ (User authorized)            │
       │                              │
┌──────▼───────┐                      │
│ CCO Client   │◄─────────────────────┘
└──────┬───────┘
       │
       │ 11. Poll: POST /auth/device/token
       │     (every 5 seconds)
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 12. Check with Authentik
       ▼
┌──────────────┐
│  Authentik   │
└──────┬───────┘
       │
       │ 13. {access_token, refresh_token}
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 14. Return tokens
       ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 15. Store tokens in ~/.config/cco/tokens.json
      │     (with 0o600 permissions)
      ▼
    Done!
```

### Download Flow

```
┌──────┐
│ User │
└───┬──┘
    │
    │ 1. cco update
    ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 2. Read tokens from ~/.config/cco/tokens.json
      │
      │ 3. Check expiration (< 5 min remaining?)
      │
      │ 4. If expired: POST /auth/token/refresh
      │    (get new access_token)
      │
      │ 5. GET /releases/latest?channel=stable
      │    Header: Authorization: Bearer {access_token}
      ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 6. Verify token with Authentik
       ▼
┌──────────────┐
│  Authentik   │
└──────┬───────┘
       │
       │ 7. {user_id, email, groups: ["cco-users"]}
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 8. Check groups (is "cco-users" present?)
       │    YES → continue
       │    NO  → 403 Forbidden
       │
       │ 9. Query database for latest stable release
       ▼
┌──────────────┐
│  PostgreSQL  │
└──────┬───────┘
       │
       │ 10. {version: "2025.11.2", platforms: [...]}
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 11. Return release info
       ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 12. Compare versions
      │     Current: 2025.11.1
      │     Latest:  2025.11.2
      │     Action:  Update available!
      │
      │ 13. GET /download/2025.11.2/darwin-arm64
      │     Header: Authorization: Bearer {access_token}
      ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 14. Generate presigned URL
       ▼
┌──────────────┐
│      R2      │
└──────┬───────┘
       │
       │ 15. {url: "https://releases.visiquate.com/...",
       │      expires_in: 300}
       ▼
┌──────────────┐
│   cco-api    │
└──────┬───────┘
       │
       │ 16. Log download to database
       │     Record: user, version, platform, timestamp
       │
       │ 17. Return presigned URL
       ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 18. Validate presigned URL
      │     - HTTPS only
      │     - Domain: releases.visiquate.com
      │
      │ 19. Download binary (streaming)
      │     GET https://releases.visiquate.com/cco/...
      ▼
┌──────────────┐
│      R2      │
└──────┬───────┘
       │
       │ 20. Binary data (45 MB)
       ▼
┌────────────┐
│ CCO Client │
└─────┬──────┘
      │
      │ 21. Verify SHA256 checksum
      │     Expected: abc123...
      │     Actual:   abc123...
      │     Match: ✅
      │
      │ 22. Extract to temp directory
      │     /tmp/cco-update-xyz/cco
      │
      │ 23. Set executable permissions
      │     chmod +x /tmp/cco-update-xyz/cco
      │
      │ 24. Atomic replace
      │     mv /tmp/cco-update-xyz/cco ~/.local/bin/cco
      │
      │ 25. Verify new version
      │     ~/.local/bin/cco --version
      │     Output: "cco 2025.11.2"
      ▼
    Done! ✅
```

## Security Architecture

### Threat Model

**Assets to Protect**:
1. CCO binary integrity
2. User credentials
3. Access tokens
4. Download access
5. User privacy

**Threats**:
1. Unauthorized binary access
2. Binary tampering (MITM)
3. Token theft/replay
4. Credential compromise
5. Denial of service

**Mitigations**:

| Threat | Mitigation | Implementation |
|--------|------------|----------------|
| Unauthorized access | Authentication required | OIDC device flow |
| Binary tampering | Mandatory checksum verification | SHA256 validation |
| Token theft | Short-lived tokens | 1 hour access, 30 day refresh |
| MITM attack | TLS everywhere | HTTPS-only, cert pinning |
| Token replay | Token expiration | Automatic refresh logic |
| Credential compromise | MFA, password policies | Authentik configuration |
| DoS | Rate limiting | Per-user limits in API |
| Data breach | Encryption at rest | TLS for transit, R2 encryption |

### Defense in Depth

**Layer 1: Network**
- TLS 1.2+ only
- Certificate validation
- DNS security (DNSSEC)

**Layer 2: Authentication**
- OIDC device flow (no password in CLI)
- MFA support
- Token rotation
- Group-based access control

**Layer 3: Authorization**
- Token verification on every request
- Group membership validation
- Presigned URL time limits

**Layer 4: Data**
- Mandatory checksum verification
- Size limits (100MB max)
- HTTPS-only downloads
- Secure token storage (0o600 permissions)

**Layer 5: Audit**
- All authentication events logged
- All download events logged
- Failed access attempts logged
- Log retention (1+ year)

### Security Best Practices

**For Developers**:
```rust
// ✅ Good: Always verify checksums
let calculated_hash = sha256_hash(&binary);
if calculated_hash != expected_hash {
    return Err("Checksum mismatch - possible MITM attack");
}

// ✅ Good: Validate URLs before download
if !url.starts_with("https://releases.visiquate.com/") {
    return Err("Invalid download URL");
}

// ✅ Good: Use secure token storage
#[cfg(unix)]
std::fs::set_permissions(&token_file, Permissions::from_mode(0o600))?;

// ❌ Bad: Skipping checksum
// download_binary(&url).await?; // NO!

// ❌ Bad: Accepting any HTTPS URL
// download_binary(&url).await?; // NO!

// ❌ Bad: World-readable tokens
// std::fs::write(&token_file, &tokens)?; // NO!
```

**For Administrators**:
- Enable MFA for all users
- Rotate client secrets quarterly
- Monitor failed authentication attempts
- Review access logs weekly
- Keep all systems updated
- Regular security audits

## Performance Characteristics

### Latency

| Operation | Target | Typical | Notes |
|-----------|--------|---------|-------|
| Device code generation | <1s | 200ms | One-time per login |
| Token polling | <2s | 500ms | Every 5s during login |
| Token refresh | <1s | 300ms | Every hour |
| Release check | <2s | 800ms | Includes auth verification |
| Presigned URL generation | <500ms | 200ms | Per download |
| Binary download | <5min | 2min | 45MB @ 500KB/s |

### Throughput

| Metric | Capacity | Notes |
|--------|----------|-------|
| Concurrent logins | 1000/min | Authentik limit |
| Concurrent downloads | 10,000/min | R2 limit |
| API requests | 10,000/sec | FastAPI + uvicorn |
| Database queries | 5,000/sec | PostgreSQL |

### Scalability

**Horizontal Scaling**:
```
        ┌─────────────┐
        │ Load Balancer│
        │   (nginx)    │
        └──────┬───────┘
               │
       ┌───────┼───────┐
       │       │       │
       ▼       ▼       ▼
    ┌────┐  ┌────┐  ┌────┐
    │API │  │API │  │API │
    │ 1  │  │ 2  │  │ 3  │
    └────┘  └────┘  └────┘
       │       │       │
       └───────┼───────┘
               │
               ▼
        ┌────────────┐
        │ PostgreSQL │
        │  Primary   │
        └─────┬──────┘
              │
      ┌───────┴───────┐
      ▼               ▼
   ┌────┐          ┌────┐
   │Rep1│          │Rep2│
   └────┘          └────┘
```

**Bottlenecks**:
1. Authentik token verification (can cache)
2. Database queries (use connection pooling)
3. R2 bandwidth (already ~100 Gbps)

**Optimization**:
- Cache token verifications (5 min TTL)
- Database read replicas for analytics
- CDN for static assets
- Connection pooling (50 connections)

## Monitoring and Observability

### Metrics

**Application Metrics**:
- Requests per second (by endpoint)
- Response time (p50, p95, p99)
- Error rate (by status code)
- Active sessions
- Token refresh rate

**Business Metrics**:
- Unique users per day
- Downloads per day (by version)
- Authentication success rate
- Failed login attempts
- User growth rate

**Infrastructure Metrics**:
- CPU usage
- Memory usage
- Network I/O
- Disk usage
- R2 bandwidth
- Database connections

### Logging

**Log Levels**:
- **DEBUG**: Detailed execution flow
- **INFO**: Normal operations (auth, downloads)
- **WARNING**: Recoverable errors (token expired)
- **ERROR**: Failures (403, 500, exceptions)
- **CRITICAL**: System failures (DB down, R2 unavailable)

**Log Format** (JSON):
```json
{
  "timestamp": "2025-11-24T14:30:00.123Z",
  "level": "INFO",
  "service": "cco-api",
  "endpoint": "/releases/latest",
  "method": "GET",
  "status": 200,
  "duration_ms": 234,
  "user_id": "john.doe",
  "ip": "192.168.1.100",
  "user_agent": "cco/2025.11.2",
  "request_id": "req_abc123"
}
```

### Alerting

**Critical Alerts** (page on-call):
- API down (health check fails)
- Authentik down
- R2 unavailable
- Database down
- Error rate > 5%

**Warning Alerts** (email):
- High latency (p95 > 5s)
- Failed logins > 100/hour
- Low disk space (< 10% free)
- SSL cert expiring (< 30 days)

### Dashboards

**Operations Dashboard**:
- System health (all green/red)
- Request rate (last hour)
- Error rate (last hour)
- Active users (last 24 hours)
- Recent alerts

**Analytics Dashboard**:
- Downloads by version (pie chart)
- Downloads over time (line graph)
- Top users (table)
- Geographic distribution (map)
- Platform distribution (bar chart)

## Disaster Recovery

### Backup Strategy

**Database Backups**:
```bash
# Daily automated backups
0 2 * * * pg_dump cco_db > /backup/cco-$(date +\%Y\%m\%d).sql

# Retention: 30 days
```

**R2 Backups**:
```bash
# Weekly sync to backup bucket
0 3 * * 0 aws s3 sync \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  s3://cco-releases/ s3://cco-releases-backup/
```

**Configuration Backups**:
```bash
# Daily backups
/home/cco-api/cco-api/.env
/etc/systemd/system/cco-api.service
/etc/nginx/sites-available/cco-api
```

### Recovery Procedures

**API Server Failure**:
1. Check health: `curl https://cco-api.visiquate.com/health`
2. Check logs: `sudo journalctl -u cco-api -n 100`
3. Restart service: `sudo systemctl restart cco-api`
4. If persistent, restore from backup
5. RTO: 5 minutes, RPO: 24 hours

**Database Failure**:
1. Check status: `sudo systemctl status postgresql`
2. Check logs: `sudo tail -100 /var/log/postgresql/postgresql.log`
3. Restore from latest backup
4. RTO: 30 minutes, RPO: 24 hours

**R2 Outage**:
1. Monitor Cloudflare status page
2. If > 1 hour, activate backup bucket
3. Update API configuration
4. Notify users of potential delays
5. RTO: 1 hour, RPO: 7 days

## API Versioning

**Current**: v1 (implicit in URL structure)

**Future Versioning**:
```
/v1/auth/device/code      (current)
/v1/releases/latest       (current)

/v2/auth/device/code      (future)
/v2/releases/latest       (future)
```

**Deprecation Policy**:
- New versions supported for 2+ years
- Deprecation announced 6 months in advance
- Old versions supported for 12 months after deprecation
- Critical security fixes backported

## Related Documentation

- [Deployment Guide](./DEPLOYMENT_PRIVATE_DISTRIBUTION.md)
- [User Guide](./USER_GUIDE_AUTHENTICATION.md)
- [Admin Guide](./ADMIN_GUIDE_ACCESS_CONTROL.md)
- [Migration Guide](./MIGRATION_FROM_GITHUB_RELEASES.md)
- [API Reference](../cco/AUTH_AND_RELEASES_API_IMPLEMENTATION.md)

## Glossary

- **OIDC**: OpenID Connect - Authentication protocol
- **Device Flow**: OAuth2 flow for devices without browsers
- **Presigned URL**: Time-limited URL with embedded credentials
- **R2**: Cloudflare's S3-compatible object storage
- **RTO**: Recovery Time Objective - Max downtime
- **RPO**: Recovery Point Objective - Max data loss
- **MFA**: Multi-Factor Authentication
- **TLS**: Transport Layer Security (HTTPS)

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-24 | Initial architecture documentation |

---

**Document Owner**: DevOps Team
**Last Updated**: 2025-11-24
**Next Review**: 2026-02-24
