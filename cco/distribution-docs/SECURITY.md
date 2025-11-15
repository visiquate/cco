# Security Policy

## Supported Versions

CCO follows semantic versioning. Security updates are provided for the following versions:

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 0.2.x   | :white_check_mark: | Latest stable |
| < 0.2.0 | :x:                | Not supported |

We recommend always using the latest stable release to ensure you have the latest security patches.

## Security Features

CCO implements several security measures to protect your API keys and data:

### 1. API Key Handling
- **Never logged**: API keys are never written to logs or console output
- **Memory-only**: Keys are stored in memory only during runtime
- **No persistence**: Keys are not persisted to disk (except in cache for performance)
- **Environment variables**: Recommended method for providing API keys

### 2. Transport Security
- **HTTPS support**: Use TLS/SSL in production deployments
- **Certificate validation**: Strict certificate validation for upstream API calls
- **Secure defaults**: Insecure protocols disabled by default

### 3. Cache Security
- **Local only**: All caching is local to your machine
- **No external transmission**: Cached data never leaves your system
- **Memory-based**: Cache stored in memory, cleared on restart
- **Access control**: Cache directory restricted to user only (permissions: 700)

### 4. Update Security
- **HTTPS only**: All update checks and downloads use HTTPS
- **SHA256 verification**: Every binary download is verified against published checksums
- **Atomic updates**: Update process uses atomic file operations to prevent corruption
- **Rollback support**: Previous version backed up during updates

### 5. Configuration Security
- **User-only access**: Config files restricted to user (permissions: 600)
- **Safe defaults**: Secure settings enabled by default
- **No credential storage**: API keys not stored in config files

## Verifying Downloads

All CCO releases include SHA256 checksums for verification.

### Verify Manually

1. Download the binary for your platform
2. Download the corresponding `.sha256` file
3. Verify the checksum:

```bash
# macOS / Linux
sha256sum -c cco-v0.2.0-darwin-arm64.tar.gz.sha256

# macOS (alternative)
shasum -a 256 -c cco-v0.2.0-darwin-arm64.tar.gz.sha256

# Windows PowerShell
$hash = Get-FileHash -Algorithm SHA256 cco-v0.2.0-windows-x86_64.zip
$expected = Get-Content cco-v0.2.0-windows-x86_64.zip.sha256
if ($hash.Hash -eq $expected) { Write-Host "Checksum verified" }
```

### Automatic Verification

The installation script automatically verifies checksums. If verification fails, installation is aborted.

## Reporting a Vulnerability

We take all security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**Email**: security@visiquate.com

**Do NOT**:
- Create a public GitHub issue
- Disclose the vulnerability publicly before it's fixed
- Test vulnerabilities on production systems you don't own

**Do**:
- Provide detailed description of the vulnerability
- Include steps to reproduce
- Suggest potential fixes if you have them
- Allow reasonable time for us to fix before public disclosure

### What to Expect

1. **Acknowledgment**: We'll acknowledge receipt within 48 hours
2. **Investigation**: We'll investigate and assess severity within 5 business days
3. **Fix**: We'll develop and test a fix
4. **Release**: We'll release a patched version
5. **Disclosure**: We'll coordinate disclosure with you (typically 30 days after fix)
6. **Credit**: We'll credit you in the security advisory (if desired)

### Severity Classification

We use CVSS 3.1 to assess vulnerability severity:

- **Critical** (9.0-10.0): Immediate attention, emergency patch within 24-48 hours
- **High** (7.0-8.9): Priority fix, patch within 1 week
- **Medium** (4.0-6.9): Standard fix, patch in next release
- **Low** (0.1-3.9): Minor fix, included in regular updates

## Security Best Practices

### For Development Use

```bash
# Use environment variables for API keys (recommended)
export ANTHROPIC_API_KEY="sk-ant-..."
cco proxy

# Never commit API keys to version control
echo "ANTHROPIC_API_KEY=sk-ant-..." >> .env
echo ".env" >> .gitignore
```

### For Production Use

```bash
# 1. Use HTTPS for external access
cco proxy --host 0.0.0.0 --port 443 --tls-cert cert.pem --tls-key key.pem

# 2. Restrict access with firewall
# Only allow connections from trusted IPs

# 3. Use secrets management
# Kubernetes: Use secrets
# Docker: Use Docker secrets
# AWS: Use Secrets Manager or Parameter Store

# 4. Enable access logging
cco proxy --log-level info --log-file /var/log/cco/access.log

# 5. Set resource limits
cco proxy --cache-size 500 --max-connections 100
```

### Docker Security

```dockerfile
# Use specific version tags (not :latest)
FROM cco-releases:0.2.0

# Run as non-root user
USER cco

# Use read-only filesystem where possible
VOLUME ["/config"]
```

```bash
# Use Docker secrets for API keys
docker secret create anthropic_key /path/to/key
docker service create \
  --secret anthropic_key \
  --name cco \
  cco-releases:0.2.0
```

### Kubernetes Security

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-keys
type: Opaque
stringData:
  anthropic: "sk-ant-..."
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco-proxy
spec:
  template:
    spec:
      # Run as non-root
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
      containers:
      - name: cco
        image: cco-releases:0.2.0
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: api-keys
              key: anthropic
        # Resource limits
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
        # Read-only filesystem
        securityContext:
          readOnlyRootFilesystem: true
```

## Network Security

### Firewall Rules

```bash
# Allow only local connections (default)
cco proxy --host 127.0.0.1

# Allow specific network
cco proxy --host 192.168.1.100

# Allow all (production with firewall)
cco proxy --host 0.0.0.0
# Then configure firewall to restrict access
```

### Rate Limiting

```toml
# ~/.config/cco/config.toml
[rate_limiting]
enabled = true
max_requests_per_minute = 60
burst_size = 10
```

## Data Privacy

### What CCO Stores

**Locally**:
- API request/response cache (in-memory and optional SQLite)
- Cost analytics (SQLite database)
- Configuration (TOML file)

**Not Stored**:
- API keys (memory only)
- Request bodies (unless explicitly cached)
- Personal information

### What CCO Transmits

**To API Providers**:
- Your API requests (as configured)
- Standard HTTP headers (User-Agent, Content-Type)

**To GitHub** (for updates):
- CCO version number
- Platform/architecture
- No personally identifiable information

### Disabling Telemetry

CCO does not collect telemetry by default. If future versions include opt-in telemetry:

```toml
# ~/.config/cco/config.toml
[telemetry]
enabled = false  # Default is false
```

## Compliance

### GDPR Compliance

CCO processes data on your behalf. As the data controller, you are responsible for:
- Obtaining user consent for API calls
- Ensuring data processing agreements with API providers
- Managing data retention and deletion

CCO helps by:
- Processing data locally only
- Not transmitting data to third parties (except configured API providers)
- Providing cache clear functionality

### SOC 2 / ISO 27001

For enterprise deployments:
- Enable audit logging
- Use encrypted storage for cache
- Implement access controls
- Regular security updates
- Incident response procedures

## Security Roadmap

Future security enhancements planned:

- **GPG signature verification**: Verify binary signatures with GPG
- **RBAC support**: Role-based access control for multi-user deployments
- **Audit logging**: Comprehensive audit trail for compliance
- **Encrypted cache**: Optional encryption for cached data
- **API key rotation**: Automatic rotation support
- **SAML/OAuth**: Enterprise authentication support

## Security Changelog

### Version 0.2.0 (2025-11-15)
- Initial release with baseline security features
- SHA256 checksum verification
- Secure configuration file permissions
- API key memory-only storage
- HTTPS support for updates

## Additional Resources

- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Rust Security Best Practices](https://anssi-fr.github.io/rust-guide/)

## Contact

For security inquiries: security@visiquate.com

For general support: support@visiquate.com

---

Last updated: 2025-11-15
