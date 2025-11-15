# Changelog

All notable changes to CCO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- GPG signature verification for binary releases
- Delta updates for bandwidth efficiency
- Package manager integrations (Homebrew, apt, Chocolatey)
- Encrypted cache option
- Telemetry system (opt-in)

## [0.2.0] - 2025-11-15

### Added
- **Initial public release** of CCO (Claude Code Orchestra)
- HTTP/HTTPS proxy server for Claude API and compatible endpoints
- Moka in-memory cache for API response caching
  - Configurable cache size and TTL
  - Automatic cache eviction with LRU policy
  - 50-90% cost savings on typical workloads
- Multi-model routing system
  - Pattern-based routing to different providers
  - Support for Anthropic, OpenAI, Ollama, and custom endpoints
  - Automatic fallback chains
- Real-time web dashboard
  - Project-level cost analytics
  - Machine-wide statistics
  - Cache hit rate and savings visualization
  - Live request monitoring
- SQLite-based analytics database
  - Per-request cost tracking
  - Token usage statistics
  - Model performance metrics
  - Historical data retention
- Comprehensive test suite
  - 112 tests covering core functionality
  - Integration tests for API compatibility
  - Performance benchmarks
- Multi-platform support
  - macOS (Apple Silicon and Intel)
  - Linux (x86_64 and ARM64)
  - Windows (x86_64)
- Automatic update system
  - Background update checking
  - Configurable update intervals (daily/weekly/never)
  - Update channels (stable/beta/nightly)
  - Atomic updates with rollback support
  - SHA256 checksum verification
- Configuration management
  - TOML-based configuration
  - Environment variable support
  - Per-project configuration override
- Command-line interface
  - `cco proxy` - Start the proxy server
  - `cco stats` - View cost statistics
  - `cco cache` - Manage cache
  - `cco update` - Manage updates
  - `cco config` - Configure settings
- Installation scripts
  - Universal installer for Unix systems (install.sh)
  - PowerShell installer for Windows (install.ps1)
  - Automatic PATH configuration
  - Platform detection and binary selection
- Security features
  - API keys stored in memory only (not persisted)
  - Secure file permissions for config (600)
  - HTTPS support for production deployments
  - Certificate validation for upstream APIs
- Documentation
  - Comprehensive README
  - Installation guide
  - Configuration reference
  - Usage examples
  - Troubleshooting guide
  - Security policy

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- Initial security baseline established
- SHA256 checksum verification for downloads
- Secure configuration file handling
- API key protection (memory-only storage)

## [0.1.0] - Internal Development

### Added
- Initial prototype development
- Basic proxy functionality
- Core caching implementation
- SQLite analytics integration
- Test framework setup

---

## Version History

| Version | Release Date | Status | Notes |
|---------|--------------|--------|-------|
| 0.2.0 | 2025-11-15 | **Latest** | Initial public release |
| 0.1.0 | - | Internal | Development prototype |

## Upgrade Notes

### Upgrading to 0.2.0

This is the first public release. New installations only.

## Breaking Changes

### 0.2.0
- No breaking changes (initial release)

## Migration Guide

### From 0.1.x to 0.2.0

Not applicable - 0.1.x was internal development only.

## Support Policy

- **Latest stable**: Active development, security updates, bug fixes
- **Previous stable**: Security updates only for 6 months
- **Older versions**: No support

Current support matrix:
- **0.2.x**: Full support (latest stable)
- **< 0.2.0**: Not publicly released

## Release Process

CCO follows this release process:

1. **Development**: New features developed in feature branches
2. **Testing**: Comprehensive test suite + manual testing
3. **Beta Release**: `vX.Y.Z-beta.N` tagged for early adopters
4. **Release Candidate**: `vX.Y.Z-rc.N` for final testing
5. **Stable Release**: `vX.Y.Z` published to GitHub Releases
6. **Announcement**: Release notes and documentation updated

## Versioning Strategy

CCO uses [Semantic Versioning](https://semver.org/):

- **Major version** (X.0.0): Breaking changes to API or configuration
- **Minor version** (0.X.0): New features, backward compatible
- **Patch version** (0.0.X): Bug fixes, no new features

Pre-release identifiers:
- `alpha`: Early development, unstable
- `beta`: Feature complete, testing in progress
- `rc`: Release candidate, final testing

## Deprecation Policy

When features are deprecated:

1. **Announce**: Deprecation notice in changelog
2. **Maintain**: Feature continues to work with warnings
3. **Remove**: Feature removed in next major version (minimum 6 months notice)

Example:
```
0.3.0: Feature X deprecated (warning added)
0.4.0: Feature X still works (warning continues)
0.5.0: Feature X still works (warning continues)
1.0.0: Feature X removed (major version bump)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to contribute changes.

## Links

- [GitHub Repository](https://github.com/brentley/cco-releases)
- [Issue Tracker](https://github.com/brentley/cco-releases/issues)
- [Documentation](docs/)
- [Security Policy](SECURITY.md)

---

**Legend**:
- 🎉 New feature
- ✨ Enhancement
- 🐛 Bug fix
- 🔒 Security fix
- ⚡ Performance improvement
- 📝 Documentation
- 🗑️ Deprecation
- ❌ Removal
