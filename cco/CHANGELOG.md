# Changelog

All notable changes to CCO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) with date-based versions (YYYY.MM.N).

## [Unreleased]

### Added - Private Binary Distribution
- **Authentication System**: OIDC device flow authentication for secure binary access
  - `cco login` - Login via browser-based device flow
  - `cco logout` - Clear stored authentication tokens
  - Automatic token refresh (transparent to users)
  - Secure token storage in `~/.config/cco/tokens.json` with 0o600 permissions
  - Token lifetime: 1 hour (access), 30 days (refresh)

- **Private Releases API**: Authenticated binary distribution system
  - Migration from public GitHub Releases to private API
  - Integration with `cco-api.visiquate.com` backend
  - Cloudflare R2 storage for binary distribution
  - Presigned download URLs (5 minute expiration)
  - Group-based access control via Authentik
  - Release channel support (stable/beta)

- **Security Enhancements**:
  - Mandatory SHA256 checksum verification for all downloads
  - Presigned URL validation (HTTPS + approved domains only)
  - Size limits (100MB max) to prevent DoS attacks
  - Secure temp directories (0o700 permissions on Unix)
  - No downloads without authentication
  - Audit logging for all authentication and download events

- **Authentication Module** (`src/auth/`):
  - `auth::login()` - Full device flow implementation
  - `auth::logout()` - Token cleanup
  - `auth::is_authenticated()` - Quick auth status check
  - `auth::get_access_token()` - Auto-refreshing token retrieval
  - RFC 8628 compliant device flow client
  - Automatic token refresh with 5 minute buffer

- **Releases API Client** (`src/auto_update/releases_api.rs`):
  - `fetch_latest_release()` - Get latest release for channel
  - `fetch_release_by_version()` - Get specific version
  - Platform detection (macOS, Linux, Windows - arm64/x86_64)
  - Multi-platform support with automatic asset selection

### Changed
- **Auto-Update System**: Migrated from GitHub API to authenticated releases API
  - `cco update` now requires authentication
  - Users must run `cco login` before updates
  - Clear error messages guide users to authenticate
  - Graceful fallback during migration period (optional)

- **Binary Distribution**: Downloads now use presigned URLs from R2
  - Faster downloads via Cloudflare CDN
  - Better reliability and scalability
  - Regional optimization

### Security
- **Checksum Verification**: Now mandatory (was optional)
  - Prevents corrupted downloads
  - Protects against MITM attacks
  - Clear security warnings on failure

- **Access Control**: Authentication required for all releases
  - Prevents unauthorized access
  - Usage tracking and analytics
  - Compliance with enterprise security policies

### Documentation
- **User Guide**: [USER_GUIDE_AUTHENTICATION.md](../docs/USER_GUIDE_AUTHENTICATION.md)
  - Complete authentication walkthrough
  - Troubleshooting common issues
  - Security best practices
  - FAQ

- **Administrator Guide**: [ADMIN_GUIDE_ACCESS_CONTROL.md](../docs/ADMIN_GUIDE_ACCESS_CONTROL.md)
  - User and group management
  - Token policies and rotation
  - Audit logging and monitoring
  - Security operations

- **Deployment Guide**: [DEPLOYMENT_PRIVATE_DISTRIBUTION.md](../docs/DEPLOYMENT_PRIVATE_DISTRIBUTION.md)
  - Complete server deployment instructions
  - Infrastructure requirements
  - Configuration reference
  - Troubleshooting guide

- **Migration Guide**: [MIGRATION_FROM_GITHUB_RELEASES.md](../docs/MIGRATION_FROM_GITHUB_RELEASES.md)
  - Step-by-step migration process
  - Parallel operation strategy
  - Rollback procedures
  - Communication templates

- **Architecture**: [ARCHITECTURE_PRIVATE_DISTRIBUTION.md](../docs/ARCHITECTURE_PRIVATE_DISTRIBUTION.md)
  - System architecture overview
  - Component descriptions
  - Data flow diagrams
  - Security model
  - API reference

### Developer Notes
- **New Dependencies**:
  - `secrecy` (0.8) - Secret zeroization
  - Authentication leverages existing credential infrastructure
  - No breaking changes to existing functionality

- **Backward Compatibility**:
  - Old CCO versions continue working during migration
  - Optional GitHub fallback for transition period
  - Deprecation warnings for outdated versions

### Migration Notes

**For End Users**:
1. Update CCO: `cco update` (last unauthenticated update)
2. Login: `cco login` (one-time browser authentication)
3. Continue normally: `cco update` now works with authentication

**For Administrators**:
1. Deploy `cco-api.visiquate.com` backend
2. Configure Authentik OIDC provider
3. Set up Cloudflare R2 bucket
4. Create user accounts and groups
5. Upload releases to R2
6. Monitor migration progress

**Timeline**:
- **Week 1-2**: Infrastructure deployment and testing
- **Week 3-4**: Parallel operation (both systems active)
- **Week 5-6**: Full migration to authenticated system
- **Week 7+**: GitHub fallback removed

### Testing
- Manual testing required for authentication flow
- Integration tests for full download cycle
- Security validation for all endpoints
- Load testing for concurrent operations

See [TESTING_AUTH_FLOW.md](./TESTING_AUTH_FLOW.md) for detailed test procedures.

## [2025.11.1] - 2025-11-XX

### Initial Release
- CCO proxy daemon for LLM API routing
- Multi-model support (Claude, OpenAI, Ollama)
- Transparent caching with Moka
- Real-time cost tracking
- TUI dashboard for monitoring
- Auto-update system (GitHub-based)
- Cross-platform support (macOS, Linux, Windows)

---

## Version Format

CCO uses date-based versioning: **YYYY.MM.N**

- **YYYY**: Four-digit year
- **MM**: Month (1-12)
- **N**: Release counter for that month (resets to 1 each month)

**Examples**:
- `2025.11.1` - First release in November 2025
- `2025.11.2` - Second release in November 2025
- `2025.12.1` - First release in December 2025

## Links

- **Repository**: https://github.com/yourusername/cc-orchestra
- **Documentation**: https://docs.visiquate.com/cco
- **Support**: https://github.com/yourusername/cc-orchestra/issues
- **API Server**: https://cco-api.visiquate.com
- **Authentication**: https://auth.visiquate.com
