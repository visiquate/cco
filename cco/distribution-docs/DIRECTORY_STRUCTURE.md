# CCO Distribution Repository Structure

This document outlines the complete directory structure for the public `brentley/cco-releases` repository.

## Repository Layout

```
cco-releases/
├── README.md                       # Main user-facing documentation
├── SECURITY.md                     # Security policy and vulnerability reporting
├── CHANGELOG.md                    # Version history and release notes
├── LICENSE                         # Apache 2.0 license
├── CONTRIBUTING.md                 # Contribution guidelines
├── CODE_OF_CONDUCT.md             # Community code of conduct
│
├── install.sh                      # Universal Unix installer script
├── install.ps1                     # Windows PowerShell installer script
├── uninstall.sh                    # Unix uninstall script
├── uninstall.ps1                   # Windows uninstall script
│
├── version-manifest.json           # Version metadata and download URLs
│
├── .github/                        # GitHub-specific configuration
│   ├── FUNDING.yml                # Sponsorship/funding information
│   ├── ISSUE_TEMPLATE/            # Issue templates
│   │   ├── bug_report.md          # Bug report template
│   │   ├── feature_request.md     # Feature request template
│   │   └── question.md            # Question template
│   ├── PULL_REQUEST_TEMPLATE.md   # PR template
│   └── workflows/                  # GitHub Actions workflows
│       ├── update-manifest.yml    # Auto-update manifest on release
│       └── verify-checksums.yml   # Verify release checksums
│
├── docs/                           # Detailed documentation
│   ├── INSTALLATION.md            # Comprehensive installation guide
│   ├── CONFIGURATION.md           # Complete configuration reference
│   ├── USAGE.md                   # Usage guide and examples
│   ├── UPDATING.md                # Update and version management
│   ├── TROUBLESHOOTING.md         # Common issues and solutions
│   ├── ARCHITECTURE.md            # System architecture overview
│   ├── API.md                     # API compatibility reference
│   └── FAQ.md                     # Frequently asked questions
│
├── scripts/                        # Helper scripts
│   ├── verify-installation.sh     # Verify CCO installation
│   ├── check-system.sh            # System requirements checker
│   └── generate-config.sh         # Config file generator
│
└── checksums/                      # SHA256 checksums per version
    ├── v0.2.0.sha256              # Checksums for v0.2.0
    ├── v0.3.0.sha256              # Checksums for v0.3.0
    └── ...                         # Future versions
```

## File Descriptions

### Root Level Files

#### README.md
Primary user-facing documentation including:
- Quick install commands
- Platform support matrix
- Feature overview
- Basic usage examples
- Links to detailed docs

#### SECURITY.md
Security information including:
- Supported versions
- Security features
- Vulnerability reporting process
- Best practices
- Compliance information

#### CHANGELOG.md
Complete version history with:
- Release dates
- New features
- Bug fixes
- Breaking changes
- Upgrade notes

#### LICENSE
Apache 2.0 license text (or MIT, depending on choice).

#### CONTRIBUTING.md
Guidelines for contributing:
- How to report issues
- How to submit PRs
- Development setup
- Code style guidelines

#### CODE_OF_CONDUCT.md
Community standards and expected behavior.

### Installation Scripts

#### install.sh
Universal Unix installer that:
- Detects platform and architecture
- Downloads appropriate binary
- Verifies SHA256 checksum
- Installs to `~/.local/bin/`
- Updates PATH in shell RC files
- Handles upgrades gracefully

#### install.ps1
Windows PowerShell installer with equivalent functionality for Windows systems.

#### uninstall.sh / uninstall.ps1
Uninstallation scripts that:
- Remove binary
- Remove configuration (with confirmation)
- Clean up PATH modifications
- Remove cache directories (optional)

### Version Manifest

#### version-manifest.json
Machine-readable version metadata:
```json
{
  "latest": {
    "stable": "0.2.0",
    "beta": "0.3.0-beta.1",
    "nightly": "0.3.0-nightly.20251115"
  },
  "versions": {
    "0.2.0": {
      "date": "2025-11-15T10:00:00Z",
      "platforms": {
        "darwin-arm64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz",
          "sha256": "abc123...",
          "size": 2912345
        }
      },
      "release_notes": "https://github.com/brentley/cco-releases/releases/tag/v0.2.0"
    }
  }
}
```

### GitHub Configuration

#### .github/FUNDING.yml
Optional sponsorship information for GitHub Sponsors.

#### .github/ISSUE_TEMPLATE/
Issue templates for bug reports, feature requests, and questions.

#### .github/PULL_REQUEST_TEMPLATE.md
PR template for documentation contributions.

#### .github/workflows/
Automation workflows:
- Update manifest on new releases
- Verify checksums match releases
- Build documentation site

### Documentation Directory

#### docs/INSTALLATION.md
Detailed installation guide covering:
- Prerequisites
- Platform-specific instructions
- Manual installation steps
- Custom installation paths
- Troubleshooting installation issues
- Verifying installation

#### docs/CONFIGURATION.md
Complete configuration reference:
- Config file location and format
- All configuration options
- Environment variables
- Configuration examples
- Advanced configurations

#### docs/USAGE.md
Usage guide with examples:
- Basic commands
- Running the proxy
- Using the dashboard
- Managing credentials
- Monitoring costs
- Multi-model routing
- Best practices

#### docs/UPDATING.md
Update guide covering:
- Automatic updates (background)
- Manual updates (`cco update`)
- Update configuration
- Rollback instructions
- Update channels
- Version pinning

#### docs/TROUBLESHOOTING.md
Common issues and solutions:
- Installation problems
- PATH not updated
- Permission errors
- Binary not found
- Update failures
- Network errors
- Platform-specific issues

#### docs/ARCHITECTURE.md
System architecture overview:
- Component diagram
- Data flow
- Caching strategy
- Routing logic
- Update mechanism

#### docs/API.md
API compatibility reference:
- Supported endpoints
- Request/response formats
- Authentication
- Rate limiting
- Error handling

#### docs/FAQ.md
Frequently asked questions organized by category.

### Scripts Directory

#### scripts/verify-installation.sh
Post-installation verification script that checks:
- Binary exists and is executable
- Binary is in PATH
- Config directory exists
- Version command works
- Network connectivity

#### scripts/check-system.sh
System requirements checker:
- Operating system version
- Architecture compatibility
- Available disk space
- Network connectivity
- Required ports availability

#### scripts/generate-config.sh
Interactive config file generator for first-time setup.

### Checksums Directory

Contains SHA256 checksum files for each release version. Each file lists all platform binaries with their checksums.

Example `checksums/v0.2.0.sha256`:
```
abc123... cco-v0.2.0-darwin-arm64.tar.gz
def456... cco-v0.2.0-darwin-x86_64.tar.gz
ghi789... cco-v0.2.0-linux-x86_64.tar.gz
jkl012... cco-v0.2.0-linux-aarch64.tar.gz
mno345... cco-v0.2.0-windows-x86_64.zip
```

## Release Assets

GitHub Releases contain binary packages (not stored in repo):

```
cco-v0.2.0-darwin-arm64.tar.gz
├── cco                             # Binary executable
├── README.md                       # Quick start
├── LICENSE                         # License file
├── CHANGELOG.md                    # Changes in this version
└── checksums.sha256               # Self-verification

cco-v0.2.0-darwin-x86_64.tar.gz
cco-v0.2.0-linux-x86_64.tar.gz
cco-v0.2.0-linux-aarch64.tar.gz
cco-v0.2.0-windows-x86_64.zip
```

## Platform-Specific Notes

### macOS
- Binaries for both Apple Silicon (arm64) and Intel (x86_64)
- Installer handles Gatekeeper approval instructions
- Optional Homebrew formula (future)

### Linux
- Binaries for x86_64 and aarch64
- Compatible with most distributions
- Optional .deb/.rpm packages (future)

### Windows
- ZIP archive (not tar.gz)
- PowerShell installer script
- Includes instructions for Windows Defender SmartScreen

## Version Numbering

Release assets follow this naming convention:

```
cco-v{VERSION}-{PLATFORM}-{ARCH}.{EXT}

Examples:
cco-v0.2.0-darwin-arm64.tar.gz
cco-v0.2.0-linux-x86_64.tar.gz
cco-v0.2.0-windows-x86_64.zip
```

Where:
- `{VERSION}`: Semantic version (e.g., 0.2.0)
- `{PLATFORM}`: darwin, linux, windows
- `{ARCH}`: arm64, x86_64, aarch64
- `{EXT}`: tar.gz for Unix, zip for Windows

## Repository Settings

### Branch Protection
- `main` branch protected
- Require PR reviews for documentation changes
- Require status checks to pass

### GitHub Pages (Optional)
If hosting documentation site:
```
docs-site/
├── index.html
├── install.html
├── docs.html
└── assets/
    ├── css/
    └── js/
```

### Repository Topics
Suggested topics for discoverability:
- `llm`
- `claude`
- `api-proxy`
- `cost-optimization`
- `caching`
- `rust`
- `cli`

## Maintenance

### Adding a New Release

1. Create release in source repo (triggers CI/CD)
2. CI builds binaries and uploads to distribution repo
3. Update `version-manifest.json`
4. Update `CHANGELOG.md`
5. Create GitHub Release with notes
6. Upload checksums to `checksums/` directory

### Updating Documentation

1. Fork repository
2. Make changes to docs
3. Submit PR
4. Maintainer reviews and merges
5. GitHub Pages auto-deploys (if configured)

## License

All documentation in this repository is licensed under Apache 2.0, matching the software license.

---

Last updated: 2025-11-15
