# Security Considerations for CCO Releases API

## Overview

This document outlines security features, best practices, and threat mitigation for the CCO Releases API.

## Authentication & Authorization

### Traefik Forward Auth

All authentication is handled by Traefik's forward auth middleware:

- **Where**: Applied before requests reach this service
- **How**: Traefik validates Authentik bearer tokens
- **This Service**: Assumes all requests are pre-authenticated

**Important**: This service DOES NOT validate tokens. Authentication is the responsibility of Traefik.

### Authentik Integration

- **Provider**: Authentik OIDC Provider (CCO CLI Provider)
- **App**: CCO CLI Application
- **Policy**: Bound to `cco-users` group only
- **Token Validity**: Access tokens expire after 1 hour

## Credential Management

### R2 API Credentials

**Rules**:
1. Never commit credentials to repository
2. Use environment variables only
3. Rotate credentials every 90 days
4. Use IAM tokens with minimal permissions

**Permissions Required**:
```
- s3:GetObject (read binaries)
- s3:ListBucket (verify files exist)
- s3:GetBucketLocation (health checks)
```

**Never Grant**:
- s3:DeleteObject (accidental data loss)
- s3:PutObject (credential injection)
- s3:ListAllBuckets (information disclosure)

### Credential Storage

**Development**:
- Store in `.env.local` (git-ignored)
- Never commit `.env` files
- Use `.env.example` for templates

**Production**:
- Use Kubernetes Secrets or Docker Secrets
- Never store in Docker images
- Never log credential values
- Rotate on deployment updates

### Credential Monitoring

```python
# ✅ SAFE: Environment variable (not logged)
logger.info(f"Connected to R2: {R2_ACCOUNT_ID}")

# ❌ UNSAFE: Logging credentials
logger.info(f"R2 Key: {R2_ACCESS_KEY}")  # NEVER!
logger.debug(f"URL: {presigned_url}")     # May expose tokens
```

## Data Security

### Presigned URLs

**TTL (Time To Live)**:
- Set to 15 minutes (900 seconds)
- Prevents long-term exposure
- Forces client re-authentication for new URLs

**URL Structure**:
- Includes signature verification
- Tied to specific S3 key
- Cannot be reused for other objects
- Includes timestamp

**Client Security**:
- URL is returned only to authenticated user
- User downloads directly from R2 (no proxy)
- R2 validates signature on each request

### Binary Integrity

**Checksums**:
- SHA256 provided in metadata
- Client must verify after download
- Protects against tampering

**Format**:
```json
{
  "platforms": {
    "darwin-arm64": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ...
  }
}
```

## Network Security

### HTTPS Only

- Traefik handles TLS termination
- Enforces HTTPS via Let's Encrypt
- Redirects HTTP → HTTPS

### API Endpoints

**Public (No Auth Required)**:
- None (all endpoints behind auth)

**Private (Traefik Auth Required)**:
- `/health` - Health check (minimal data)
- `/releases/latest` - Release metadata
- `/releases/{version}` - Specific version metadata
- `/download/{version}/{platform}` - Presigned URL generation

**Not Exposed**:
- Root `/` returns 404
- No OpenAPI schema (`docs_url=None`)
- No Swagger UI
- No ReDoc
- No public index

### R2 Bucket Access

- Bucket is private (no public access)
- Presigned URLs bypass bucket policy
- Client IP is not verified (portable across networks)
- All downloads flow through R2 signature validation

## Application Security

### Input Validation

**Version Parameter**:
```python
# Format: YYYY.MM.N (e.g., 2025.11.24)
# Validated by FastAPI path parameter
```

**Platform Parameter**:
```python
# Whitelist validation
VALID_PLATFORMS = [
    "darwin-arm64",
    "darwin-x86_64",
    "linux-x86_64",
    "linux-aarch64",
    "windows-x86_64",
]
if platform not in VALID_PLATFORMS:
    raise HTTPException(400, "Invalid platform")
```

**Channel Parameter**:
```python
# Strict validation
if channel not in ["stable", "beta"]:
    raise HTTPException(400, "Invalid channel")
```

### Error Handling

**Sensitive Information**:
- R2 error messages are sanitized
- Don't expose internal stack traces
- Return generic error messages

**Logging**:
- Log errors for debugging
- Never log credentials
- Never log full presigned URLs
- Rotate logs regularly

### Dependencies

All dependencies pinned to specific versions:

```
fastapi==0.104.1
uvicorn[standard]==0.24.0
boto3==1.28.85
pydantic==2.5.0
```

**Vulnerability Scanning**:
- Use `pip install -r requirements.txt --upgrade`
- Run `safety check` before deployment
- Subscribe to security advisories

## Deployment Security

### Docker Image

**Non-Root User**:
```dockerfile
RUN useradd -r releases-api
USER releases-api
```

**Multi-Stage Build**:
- Reduces image size
- Removes build dependencies
- Minimizes attack surface

**Health Check**:
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:8000/health || exit 1
```

### Environment Variables

**Required in Production**:
```bash
R2_ACCOUNT_ID=<value>
R2_ACCESS_KEY=<value>
R2_SECRET_KEY=<value>
R2_BUCKET=cco-releases-private
VERSION=<version>
ENVIRONMENT=production
```

**Never in Production**:
- Debug mode
- Verbose logging
- Development endpoints

### Network Isolation

- Service listens on 0.0.0.0:8000
- Traefik provides external access control
- Only accessible via cco-api.visiquate.com
- Behind CloudFlare CDN

## Compliance

### Data Privacy

- No user data stored
- No download logs (for privacy)
- Binary metadata only
- Presigned URL logs at R2

### Audit Trail

**What's Logged**:
- Service startup/shutdown
- Health check results
- Metadata retrieval requests
- Presigned URL generation (no tokens)

**What's NOT Logged**:
- Credentials (R2 keys)
- User information
- Presigned URLs (contain signatures)
- Full request bodies

## Incident Response

### Credential Compromise

1. **Immediately**:
   - Revoke compromised R2 API token
   - Generate new credentials
   - Update deployment

2. **Within 1 Hour**:
   - Review R2 access logs
   - Check for unauthorized downloads
   - Notify CCO users if needed

3. **Follow-up**:
   - Root cause analysis
   - Update credential rotation policy
   - Implement monitoring

### Service Compromise

1. **Detect**:
   - Health check failures
   - Unauthorized API access
   - Unusual R2 operations

2. **Respond**:
   - Disable service
   - Investigate logs
   - Restore from clean image

3. **Prevent**:
   - Update dependencies
   - Patch vulnerabilities
   - Redeploy service

## Security Checklist

### Pre-Deployment

- [ ] Environment variables set correctly
- [ ] R2 credentials have minimal permissions
- [ ] Traefik forward auth configured
- [ ] HTTPS/TLS enabled
- [ ] Health check passing
- [ ] Logs configured appropriately
- [ ] Dependencies vulnerability-free

### Post-Deployment

- [ ] Monitor health check endpoint
- [ ] Review access logs regularly
- [ ] Update dependencies monthly
- [ ] Rotate credentials quarterly
- [ ] Test incident response procedures

### Ongoing

- [ ] Monthly security audits
- [ ] Quarterly dependency updates
- [ ] Annual penetration testing
- [ ] Credential rotation tracking
- [ ] Security patch monitoring

## References

- [AWS S3 Security Best Practices](https://docs.aws.amazon.com/AmazonS3/latest/userguide/security.html)
- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [FastAPI Security](https://fastapi.tiangolo.com/tutorial/security/)
- [CloudFlare R2 Documentation](https://developers.cloudflare.com/r2/)

## Questions or Concerns?

Contact: Security team
Process: Submit security issues via private channel (not GitHub issues)
