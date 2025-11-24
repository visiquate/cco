# Distribution Repository Setup Guide

Step-by-step guide to setting up the `brentley/cco-releases` public distribution repository.

## Table of Contents

- [Creating the Repository](#creating-the-repository)
- [Repository Structure Setup](#repository-structure-setup)
- [Branch Protection](#branch-protection)
- [GitHub Pages Setup](#github-pages-setup-optional)
- [Repository Settings](#repository-settings)
- [Secrets Configuration](#secrets-configuration)
- [Initial Release](#initial-release)
- [Automation Setup](#automation-setup)
- [Testing the Setup](#testing-the-setup)

## Creating the Repository

### 1. Create Repository on GitHub

1. Go to: https://github.com/new
2. **Repository name**: `cco-releases`
3. **Description**: `Public distribution repository for CCO (Claude Code Orchestra) binaries and documentation`
4. **Visibility**: Public
5. **Initialize**:
   - ✅ Add README (we'll replace it)
   - ✅ Add .gitignore (select "None" - we'll add custom)
   - ✅ Choose license: Apache License 2.0
6. Click **Create repository**

### 2. Clone Repository Locally

```bash
git clone https://github.com/brentley/cco-releases.git
cd cco-releases
```

## Repository Structure Setup

### 1. Copy Distribution Files

```bash
# From your development machine, copy all distribution docs
cp -r /Users/brent/git/cc-orchestra/cco/distribution-docs/* .

# Verify structure
tree -L 2
```

Expected structure:
```
cco-releases/
├── README.md
├── SECURITY.md
├── CHANGELOG.md
├── LICENSE
├── DIRECTORY_STRUCTURE.md
├── .github/
│   ├── ISSUE_TEMPLATE/
│   └── PULL_REQUEST_TEMPLATE.md
├── docs/
│   ├── INSTALLATION.md
│   ├── CONFIGURATION.md
│   ├── USAGE.md
│   ├── UPDATING.md
│   └── TROUBLESHOOTING.md
└── (installer scripts and other files to be added)
```

### 2. Create .gitignore

```bash
cat > .gitignore << 'EOF'
# Build artifacts (binaries distributed via releases, not committed)
*.tar.gz
*.zip
cco
cco.exe

# Temporary files
*.tmp
*.log
*.swp
*~
.DS_Store

# IDE
.idea/
.vscode/
*.iml

# Local development
.env
.env.local
local/
EOF
```

### 3. Create version-manifest.json

```bash
cat > version-manifest.json << 'EOF'
{
  "latest": {
    "stable": "0.2.0",
    "beta": null,
    "nightly": null
  },
  "versions": {
    "0.2.0": {
      "date": "2025-11-15T10:00:00Z",
      "platforms": {
        "darwin-arm64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz",
          "sha256": "TBD",
          "size": 0
        },
        "darwin-x86_64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-x86_64.tar.gz",
          "sha256": "TBD",
          "size": 0
        },
        "linux-x86_64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-linux-x86_64.tar.gz",
          "sha256": "TBD",
          "size": 0
        },
        "linux-aarch64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-linux-aarch64.tar.gz",
          "sha256": "TBD",
          "size": 0
        },
        "windows-x86_64": {
          "url": "https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-windows-x86_64.zip",
          "sha256": "TBD",
          "size": 0
        }
      },
      "release_notes": "https://github.com/brentley/cco-releases/releases/tag/v0.2.0",
      "minimum_rust": "1.75.0",
      "breaking_changes": false
    }
  },
  "deprecation_notices": {}
}
EOF
```

### 4. Create checksums directory

```bash
mkdir -p checksums
touch checksums/.gitkeep
```

### 5. Commit Initial Structure

```bash
git add .
git commit -m "Initial distribution repository setup

- Add comprehensive documentation
- Add issue templates
- Add version manifest structure
- Add checksums directory"
git push origin main
```

## Branch Protection

### 1. Protect main Branch

1. Go to: Repository → Settings → Branches
2. Click **Add rule**
3. **Branch name pattern**: `main`
4. Enable:
   - ✅ Require a pull request before merging
   - ✅ Require approvals: 1
   - ✅ Dismiss stale pull request approvals when new commits are pushed
   - ✅ Require status checks to pass before merging
   - ✅ Require conversation resolution before merging
   - ✅ Include administrators (for safety)
5. Click **Create**

## GitHub Pages Setup (Optional)

If you want to host documentation on GitHub Pages:

### 1. Enable GitHub Pages

1. Go to: Repository → Settings → Pages
2. **Source**: Deploy from a branch
3. **Branch**: `main`
4. **Folder**: `/docs` (or `/` if using root)
5. Click **Save**

### 2. Create GitHub Pages Site (Optional)

Create `docs/index.html` for a landing page:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CCO - Claude Code Orchestra</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        .install-cmd {
            background: #f6f8fa;
            padding: 15px;
            border-radius: 6px;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <h1>CCO - Claude Code Orchestra</h1>
    <p>Multi-agent development system with intelligent caching, multi-model routing, and real-time analytics.</p>

    <h2>Quick Install</h2>
    <div class="install-cmd">
        curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
    </div>

    <h2>Documentation</h2>
    <ul>
        <li><a href="INSTALLATION.html">Installation Guide</a></li>
        <li><a href="CONFIGURATION.html">Configuration Reference</a></li>
        <li><a href="USAGE.html">Usage Guide</a></li>
        <li><a href="UPDATING.html">Update Guide</a></li>
        <li><a href="TROUBLESHOOTING.html">Troubleshooting</a></li>
    </ul>

    <h2>Links</h2>
    <ul>
        <li><a href="https://github.com/brentley/cco-releases">GitHub Repository</a></li>
        <li><a href="https://github.com/brentley/cco-releases/releases">Releases</a></li>
        <li><a href="https://github.com/brentley/cco-releases/issues">Issues</a></li>
    </ul>
</body>
</html>
```

## Repository Settings

### 1. General Settings

1. Go to: Repository → Settings → General
2. **Features**:
   - ✅ Issues
   - ✅ Projects (optional)
   - ✅ Wiki (optional)
   - ✅ Discussions (recommended)
3. **Pull Requests**:
   - ✅ Allow squash merging
   - ✅ Allow auto-merge
   - ✅ Automatically delete head branches

### 2. Topics

Add repository topics for discoverability:
1. Go to: Repository main page
2. Click gear icon next to "About"
3. Add topics:
   - `llm`
   - `claude`
   - `api-proxy`
   - `cost-optimization`
   - `caching`
   - `rust`
   - `cli`
   - `anthropic`

### 3. Social Preview

1. Go to: Repository → Settings → General
2. **Social preview**: Upload a banner image (1280x640px)
   - Should include CCO logo and tagline
   - Professional design

## Secrets Configuration

These secrets are needed for CI/CD automation (when implemented):

### 1. GitHub Token

Already available as `GITHUB_TOKEN` in GitHub Actions.

### 2. GPG Signing Key (Future)

For signing releases:

1. Generate GPG key:
   ```bash
   gpg --full-generate-key
   ```

2. Export private key:
   ```bash
   gpg --export-secret-keys --armor <KEY_ID> > private.key
   ```

3. Add to repository secrets:
   - Go to: Repository → Settings → Secrets and variables → Actions
   - Click **New repository secret**
   - Name: `GPG_PRIVATE_KEY`
   - Value: Contents of `private.key`

4. Add passphrase:
   - Name: `GPG_PASSPHRASE`
   - Value: Your GPG key passphrase

## Initial Release

### 1. Create First Release Manually

Until CI/CD is set up, create the first release manually:

1. Go to: Repository → Releases
2. Click **Create a new release**
3. **Tag**: `v0.2.0`
4. **Release title**: `CCO v0.2.0 - Initial Public Release`
5. **Description**: Copy from CHANGELOG.md
6. **Assets**: Upload binaries:
   - `cco-v0.2.0-darwin-arm64.tar.gz`
   - `cco-v0.2.0-darwin-x86_64.tar.gz`
   - `cco-v0.2.0-linux-x86_64.tar.gz`
   - `cco-v0.2.0-linux-aarch64.tar.gz`
   - `cco-v0.2.0-windows-x86_64.zip`
   - `checksums.sha256`
7. Click **Publish release**

### 2. Update version-manifest.json

After creating the release, update the manifest with actual checksums and sizes:

```bash
# Generate checksums
sha256sum cco-*.tar.gz cco-*.zip > checksums/v0.2.0.sha256

# Update version-manifest.json with actual values
# Get file sizes: ls -l cco-*.tar.gz
# Get checksums: cat checksums/v0.2.0.sha256

# Commit updated manifest
git add version-manifest.json checksums/
git commit -m "Update version manifest for v0.2.0"
git push origin main
```

## Automation Setup

### 1. Create Installer Scripts

Create `install.sh` for Unix systems:

```bash
#!/bin/bash
# See DISTRIBUTION_ARCHITECTURE.md for complete implementation
# This is a placeholder - full script to be implemented
set -e

echo "Installing CCO..."
# Platform detection
# Version selection
# Download
# Verification
# Installation
# PATH update
```

Create `install.ps1` for Windows:

```powershell
# Windows PowerShell installer
# Similar functionality to install.sh
```

### 2. Create GitHub Actions (Future)

Create `.github/workflows/update-manifest.yml`:

```yaml
name: Update Version Manifest
on:
  release:
    types: [published]
jobs:
  update-manifest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Update manifest
        run: |
          # Parse release data
          # Update version-manifest.json
          # Commit changes
      - name: Commit changes
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "Update version manifest for ${{ github.ref_name }}"
```

## Testing the Setup

### 1. Test Installation Script

```bash
# Test installer (in a clean environment)
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Verify
cco --version
```

### 2. Test Version Manifest

```bash
# Fetch manifest
curl https://raw.githubusercontent.com/brentley/cco-releases/main/version-manifest.json

# Verify structure
jq . version-manifest.json
```

### 3. Test GitHub Pages (if enabled)

Visit: https://brentley.github.io/cco-releases/

### 4. Test Issue Templates

1. Go to: Repository → Issues → New issue
2. Verify templates appear correctly
3. Test creating an issue with each template

### 5. Test Release Download

```bash
# Download from release
curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz

# Verify checksum
curl -L -o checksums.sha256 https://github.com/brentley/cco-releases/releases/download/v0.2.0/checksums.sha256
sha256sum -c checksums.sha256 --ignore-missing
```

## Post-Setup Checklist

- [ ] Repository created and public
- [ ] All documentation files committed
- [ ] Branch protection enabled
- [ ] Issue templates working
- [ ] Version manifest created
- [ ] First release published
- [ ] Installation script tested
- [ ] GitHub Pages enabled (optional)
- [ ] Repository topics added
- [ ] README renders correctly
- [ ] All links in docs working
- [ ] Social preview image uploaded

## Maintenance

### Regular Tasks

1. **Release new versions**:
   - Build binaries
   - Create GitHub Release
   - Update version-manifest.json
   - Update CHANGELOG.md

2. **Monitor issues**:
   - Respond to bug reports
   - Triage feature requests
   - Answer questions

3. **Update documentation**:
   - Keep docs current with new features
   - Fix broken links
   - Add new examples

4. **Security**:
   - Monitor for vulnerabilities
   - Keep dependencies updated
   - Respond to security reports

## Support

For questions about repository setup:
- Email: support@visiquate.com
- Reference: DISTRIBUTION_ARCHITECTURE.md

---

Last updated: 2025-11-15
