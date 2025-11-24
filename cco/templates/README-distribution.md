# CCO Releases

Official distribution repository for **CCO (Claude Code Orchestra)** - A multi-agent development system with intelligent caching and cost tracking.

## Quick Install

### Unix/Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

### Windows PowerShell

```powershell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex
```

## Features

- **Multi-Model Support**: Route requests to different LLM models based on complexity
- **Intelligent Caching**: Reduce API costs with smart response caching
- **Auto-Updates**: Keep CCO up-to-date automatically
- **Cross-Platform**: Runs on macOS, Linux, and Windows
- **Lightweight**: Single binary, no dependencies

## Installation Options

### Automatic Installation (Recommended)

The installer script will:
1. Detect your platform and architecture
2. Download the appropriate binary
3. Verify checksums
4. Install to `~/.local/bin` (Unix) or `%LOCALAPPDATA%\cco\bin` (Windows)
5. Update your PATH if needed
6. Create default configuration

### Manual Installation

1. Download the appropriate binary from the [latest release](https://github.com/brentley/cco-releases/releases/latest)
2. Verify the checksum:
   ```bash
   sha256sum cco-v*.tar.gz
   # Compare with checksums.sha256
   ```
3. Extract and install:
   ```bash
   tar -xzf cco-v*-darwin-arm64.tar.gz
   mv cco ~/.local/bin/
   chmod +x ~/.local/bin/cco
   ```
4. Add to PATH if needed:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   ```

### Install Specific Version

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash -s -- --version 0.3.0

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex -Version 0.3.0
```

### Install Beta/Nightly Versions

```bash
# Beta channel
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash -s -- --channel beta

# Nightly channel
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash -s -- --channel nightly
```

## Platform Support

| Platform | Architecture | Binary Name | Status |
|----------|-------------|-------------|---------|
| macOS | Apple Silicon (M1/M2) | `darwin-arm64` | ✅ Stable |
| macOS | Intel | `darwin-x86_64` | ✅ Stable |
| Linux | x86_64 | `linux-x86_64` | ✅ Stable |
| Linux | ARM64 | `linux-aarch64` | ✅ Stable |
| Windows | x86_64 | `windows-x86_64` | ✅ Stable |

## Updates

### Automatic Updates

CCO can automatically check for and install updates:

```bash
# Enable auto-updates
cco config set updates.auto_install true

# Check for updates manually
cco update --check

# Install available updates
cco update --install
```

### Update Configuration

Configuration is stored in:
- Unix/Linux/macOS: `~/.config/cco/config.toml`
- Windows: `%APPDATA%\cco\config.toml`

```toml
[updates]
enabled = true              # Enable update checks
auto_install = false        # Auto-install updates (default: false)
check_interval = "daily"    # Check frequency: daily, weekly, never
channel = "stable"          # Release channel: stable, beta, nightly
```

## Usage

```bash
# Start the proxy server
cco run

# Run with custom configuration
cco run --port 8080 --host 0.0.0.0

# Check version
cco --version

# View help
cco --help

# Manage configuration
cco config show
cco config set updates.channel beta
cco config get updates.auto_install
```

## Security

### Checksum Verification

All releases include SHA256 checksums for verification:

```bash
# Download checksums
curl -LO https://github.com/brentley/cco-releases/releases/latest/download/checksums.sha256

# Verify your download
sha256sum -c checksums.sha256
```

### GPG Signatures (Coming Soon)

Future releases will include GPG signatures for enhanced security.

## Troubleshooting

### Installation Issues

**Permission denied**
```bash
# Make installer executable
chmod +x install.sh

# Or use sudo for system-wide installation
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | sudo bash
```

**PATH not updated**
```bash
# Manually add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**SSL/TLS errors**
```bash
# Use --insecure flag (not recommended)
curl -fsSL --insecure https://... | bash
```

### Update Issues

**Updates not working**
```bash
# Check update configuration
cco config show

# Force update check
cco update --check --force

# Reinstall if needed
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash -s -- --force
```

### Platform-Specific Issues

**macOS Gatekeeper**
```bash
# If you see "cannot be opened because the developer cannot be verified"
xattr -d com.apple.quarantine ~/.local/bin/cco
```

**Windows Defender**
- Add an exception for `cco.exe` in Windows Security settings
- Or download and scan manually before running

**Linux permissions**
```bash
# Ensure executable permissions
chmod +x ~/.local/bin/cco

# Check binary architecture
file ~/.local/bin/cco
```

## Uninstallation

### Automatic Uninstall

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/uninstall.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/uninstall.ps1 | iex
```

### Manual Uninstall

1. Remove the binary:
   ```bash
   rm ~/.local/bin/cco  # Unix/Linux/macOS
   # or
   del %LOCALAPPDATA%\cco\bin\cco.exe  # Windows
   ```

2. Remove configuration:
   ```bash
   rm -rf ~/.config/cco  # Unix/Linux/macOS
   # or
   rmdir /s %APPDATA%\cco  # Windows
   ```

3. Remove from PATH (edit your shell configuration file)

## Release Channels

### Stable
- Production-ready releases
- Thoroughly tested
- Updated monthly
- Recommended for most users

### Beta
- Preview features
- Generally stable
- Updated weekly
- For early adopters

### Nightly
- Latest development builds
- May contain bugs
- Updated daily
- For testing only

## Version History

See [CHANGELOG.md](CHANGELOG.md) for detailed version history.

Recent releases:
- **v0.3.0** - Auto-update mechanism, performance improvements
- **v0.2.0** - Initial public release, core proxy functionality
- **v0.1.0** - Private beta

## Contributing

CCO is developed at [github.com/brentley/claude-orchestra](https://github.com/brentley/claude-orchestra).

For bug reports and feature requests, please use the [issue tracker](https://github.com/brentley/cco-releases/issues).

## License

CCO is distributed under the MIT License. See [LICENSE](LICENSE) for details.

## Support

- **Documentation**: [Wiki](https://github.com/brentley/cco-releases/wiki)
- **Issues**: [Bug Reports](https://github.com/brentley/cco-releases/issues)
- **Discussions**: [Community Forum](https://github.com/brentley/cco-releases/discussions)

## Telemetry

CCO includes optional, anonymous telemetry to help improve the product. This is:
- **Disabled by default**
- **Completely anonymous** (no PII collected)
- **Transparent** (see what's collected in the config)
- **Deletable** (can be purged at any time)

To enable telemetry:
```bash
cco config set telemetry.enabled true
```

---

*CCO is not affiliated with Anthropic or OpenAI. It is an independent tool for optimizing LLM API usage.*