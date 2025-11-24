# CCO Installation Guide

This guide provides detailed installation instructions for CCO (Claude Code Orchestra) on all supported platforms.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Installation](#quick-installation)
- [Platform-Specific Installation](#platform-specific-installation)
  - [macOS](#macos)
  - [Linux](#linux)
  - [Windows](#windows)
- [Manual Installation](#manual-installation)
- [Custom Installation Paths](#custom-installation-paths)
- [Verifying Installation](#verifying-installation)
- [Post-Installation Setup](#post-installation-setup)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

## Prerequisites

### System Requirements

- **Operating System**:
  - macOS 11.0 (Big Sur) or later
  - Linux with glibc 2.31 or later
  - Windows 10 or later
- **Architecture**:
  - x86_64 (Intel/AMD 64-bit)
  - ARM64 (Apple Silicon, ARM servers)
- **Disk Space**: 20 MB for binary, 100 MB recommended for cache
- **Memory**: 50 MB minimum, 200 MB recommended
- **Network**: Internet connection for installation and API calls

### Required Software

- **macOS/Linux**: curl or wget
- **Windows**: PowerShell 5.0 or later
- **API Keys** (at least one):
  - Anthropic API key (for Claude models)
  - OpenAI API key (optional, for GPT models)
  - Ollama installation (optional, for local models)

## Quick Installation

### One-Line Install (Recommended)

#### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

#### Windows PowerShell

```powershell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex
```

The installer will:
1. Detect your platform and architecture
2. Download the appropriate binary
3. Verify the SHA256 checksum
4. Install to `~/.local/bin/` (or `%USERPROFILE%\.local\bin\` on Windows)
5. Update your PATH if needed
6. Create initial configuration

## Platform-Specific Installation

### macOS

#### Using the Installer Script

```bash
# Download and run installer
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Restart your shell or source your profile
source ~/.zshrc   # for zsh (default on modern macOS)
# or
source ~/.bashrc  # for bash
```

#### Using Homebrew (Coming Soon)

```bash
# Future installation method
brew tap brentley/cco
brew install cco
```

#### Manual Installation

1. **Download the binary**:
   ```bash
   # For Apple Silicon (M1/M2/M3)
   curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz

   # For Intel Macs
   curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-x86_64.tar.gz
   ```

2. **Extract and install**:
   ```bash
   tar -xzf cco.tar.gz
   mkdir -p ~/.local/bin
   mv cco ~/.local/bin/
   chmod +x ~/.local/bin/cco
   ```

3. **Update PATH**:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

4. **Verify installation**:
   ```bash
   cco --version
   ```

#### macOS Security Notice

First time running CCO on macOS, you may see a security warning:

```
"cco" cannot be opened because it is from an unidentified developer.
```

**Solution**:
1. Go to **System Preferences** → **Security & Privacy**
2. Click **"Open Anyway"** next to the CCO message
3. Or run: `xattr -d com.apple.quarantine ~/.local/bin/cco`

### Linux

#### Using the Installer Script

```bash
# Download and run installer
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Restart your shell or source your profile
source ~/.bashrc  # for bash
# or
source ~/.zshrc   # for zsh
```

#### Using Package Managers (Coming Soon)

```bash
# Debian/Ubuntu (future)
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install-deb.sh | sudo bash

# Fedora/RHEL (future)
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install-rpm.sh | sudo bash

# Arch Linux (future)
yay -S cco
```

#### Manual Installation

1. **Download the binary**:
   ```bash
   # For x86_64 (Intel/AMD)
   curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-linux-x86_64.tar.gz

   # For ARM64 (ARM servers)
   curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-linux-aarch64.tar.gz
   ```

2. **Extract and install**:
   ```bash
   tar -xzf cco.tar.gz
   mkdir -p ~/.local/bin
   mv cco ~/.local/bin/
   chmod +x ~/.local/bin/cco
   ```

3. **Update PATH**:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

4. **Verify installation**:
   ```bash
   cco --version
   ```

#### System-Wide Installation (Optional)

For system-wide installation (requires root):

```bash
sudo mv cco /usr/local/bin/
sudo chmod 755 /usr/local/bin/cco
```

### Windows

#### Using PowerShell Installer

1. **Open PowerShell as Administrator** (optional, only needed for system-wide install)

2. **Run the installer**:
   ```powershell
   iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex
   ```

3. **Restart PowerShell** to reload PATH

4. **Verify installation**:
   ```powershell
   cco --version
   ```

#### Using Scoop (Coming Soon)

```powershell
scoop bucket add brentley https://github.com/brentley/scoop-bucket
scoop install cco
```

#### Using Chocolatey (Coming Soon)

```powershell
choco install cco
```

#### Manual Installation

1. **Download the binary**:
   - Visit: https://github.com/brentley/cco-releases/releases
   - Download: `cco-v0.2.0-windows-x86_64.zip`

2. **Extract the archive**:
   - Right-click → **Extract All**
   - Or use PowerShell: `Expand-Archive cco-v0.2.0-windows-x86_64.zip -DestinationPath .`

3. **Move to installation directory**:
   ```powershell
   # Create directory if it doesn't exist
   New-Item -Path "$env:USERPROFILE\.local\bin" -ItemType Directory -Force

   # Move the binary
   Move-Item -Path "cco.exe" -Destination "$env:USERPROFILE\.local\bin\cco.exe"
   ```

4. **Add to PATH**:
   ```powershell
   # Get current PATH
   $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")

   # Add CCO directory
   $newPath = "$env:USERPROFILE\.local\bin;$currentPath"
   [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
   ```

5. **Restart PowerShell** and verify:
   ```powershell
   cco --version
   ```

#### Windows Security Notice

Windows Defender SmartScreen may warn about the binary:

```
Windows protected your PC
Microsoft Defender SmartScreen prevented an unrecognized app from starting.
```

**Solution**:
1. Click **"More info"**
2. Click **"Run anyway"**
3. Or add CCO directory to Windows Defender exclusions

## Manual Installation

For any platform, you can manually download and install:

1. **Choose your platform and download**:
   - Visit: https://github.com/brentley/cco-releases/releases/latest
   - Download the appropriate archive for your platform

2. **Verify the download** (recommended):
   ```bash
   # Download checksum file
   curl -L -o checksums.sha256 https://github.com/brentley/cco-releases/releases/download/v0.2.0/checksums.sha256

   # Verify (macOS/Linux)
   sha256sum -c checksums.sha256 --ignore-missing

   # Verify (macOS alternative)
   shasum -a 256 -c checksums.sha256 --ignore-missing

   # Verify (Windows)
   Get-FileHash -Algorithm SHA256 cco-v0.2.0-windows-x86_64.zip
   ```

3. **Extract and install**:
   - Follow platform-specific instructions above

## Custom Installation Paths

### Installing to a Custom Directory

```bash
# Set custom installation directory
export CCO_INSTALL_DIR="$HOME/bin"

# Run installer with custom path
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash -s -- --install-dir "$CCO_INSTALL_DIR"

# Or manually
tar -xzf cco.tar.gz
mkdir -p "$CCO_INSTALL_DIR"
mv cco "$CCO_INSTALL_DIR/"
chmod +x "$CCO_INSTALL_DIR/cco"

# Add to PATH
echo "export PATH=\"$CCO_INSTALL_DIR:\$PATH\"" >> ~/.bashrc
source ~/.bashrc
```

### Installing for All Users (Linux/macOS)

```bash
# Install system-wide (requires root)
sudo tar -xzf cco.tar.gz -C /usr/local/bin/
sudo chmod 755 /usr/local/bin/cco

# Verify
which cco  # Should show /usr/local/bin/cco
```

## Verifying Installation

### Check Version

```bash
cco --version
```

Expected output:
```
cco 0.2.0
```

### Check Installation Path

```bash
which cco  # Unix
where cco  # Windows
```

Expected: `~/.local/bin/cco` or your custom installation path

### Run Health Check

```bash
cco --help
```

Should display help text with available commands.

### Test Proxy Startup

```bash
# Set API key
export ANTHROPIC_API_KEY="sk-ant-your-key-here"

# Start proxy (should start without errors)
cco proxy --port 8000
```

Expected output:
```
CCO Proxy v0.2.0
Listening on http://127.0.0.1:8000
Dashboard available at http://127.0.0.1:8000
```

Press Ctrl+C to stop.

## Post-Installation Setup

### 1. Set API Keys

```bash
# Required: Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-your-key-here"

# Optional: OpenAI API key
export OPENAI_API_KEY="sk-your-openai-key"

# Make permanent (add to shell profile)
echo 'export ANTHROPIC_API_KEY="sk-ant-your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### 2. Initialize Configuration

```bash
# Create default configuration
cco config init

# View configuration
cco config show
```

This creates `~/.config/cco/config.toml` with default settings.

### 3. Test the Proxy

```bash
# Start proxy
cco proxy --port 8000 &

# Test with curl
curl -X POST http://localhost:8000/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -d '{
    "model": "claude-sonnet-3.5",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'

# Stop proxy
pkill cco
```

### 4. Configure Claude Code (Optional)

To use CCO with Claude Code:

```bash
# Set environment variable to point to CCO
export LLM_ENDPOINT="http://localhost:8000"

# Start CCO proxy
cco proxy --port 8000
```

Now all Claude Code requests will flow through CCO.

## Troubleshooting

### Installation Issues

#### "Permission denied" Error

**Problem**: Installer can't write to installation directory.

**Solution**:
```bash
# Check directory permissions
ls -ld ~/.local/bin

# Create directory with correct permissions
mkdir -p ~/.local/bin
chmod 755 ~/.local/bin

# Re-run installer
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

#### "Command not found" After Installation

**Problem**: Installation succeeded but `cco` command not found.

**Solution**:
```bash
# Verify binary exists
ls -la ~/.local/bin/cco

# Check if directory is in PATH
echo $PATH | grep .local/bin

# If not in PATH, add it
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify
which cco
```

#### Checksum Verification Failed

**Problem**: Downloaded file doesn't match expected checksum.

**Solution**:
```bash
# Download might be corrupted, try again
rm cco.tar.gz
curl -L -o cco.tar.gz https://github.com/brentley/cco-releases/releases/download/v0.2.0/cco-v0.2.0-darwin-arm64.tar.gz

# Verify again
sha256sum -c checksums.sha256 --ignore-missing
```

#### macOS: "Damaged and can't be opened"

**Problem**: macOS Gatekeeper blocking the binary.

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ~/.local/bin/cco

# Verify
xattr ~/.local/bin/cco  # Should show no attributes
```

#### Windows: "Missing VCRUNTIME140.dll"

**Problem**: Missing Visual C++ Runtime.

**Solution**:
1. Download and install: [Microsoft Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)
2. Restart PowerShell
3. Try running CCO again

### Platform Detection Issues

If the installer detects the wrong platform:

```bash
# Override platform detection
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | \
  bash -s -- --platform linux --arch x86_64
```

Available platforms:
- `darwin` (macOS)
- `linux`
- `windows`

Available architectures:
- `arm64` (Apple Silicon, ARM)
- `x86_64` (Intel/AMD)
- `aarch64` (ARM 64-bit, Linux only)

## Uninstallation

### Using the Uninstall Script

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/uninstall.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/uninstall.ps1 | iex
```

### Manual Uninstallation

```bash
# Remove binary
rm ~/.local/bin/cco  # or /usr/local/bin/cco for system-wide

# Remove configuration (optional)
rm -rf ~/.config/cco

# Remove cache (optional)
rm -rf ~/.cache/cco

# Remove from PATH (optional)
# Edit ~/.bashrc or ~/.zshrc and remove the CCO PATH line
```

## Upgrade vs Fresh Install

### Upgrading from Previous Version

The installer automatically detects existing installations and upgrades:

```bash
# Upgrade to latest version
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Or use built-in update command (if already installed)
cco update --install
```

Configuration and cache are preserved during upgrades.

### Fresh Install (Clean State)

To start fresh:

```bash
# Uninstall completely
rm ~/.local/bin/cco
rm -rf ~/.config/cco
rm -rf ~/.cache/cco

# Install fresh
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash
```

## Next Steps

After installation:

1. **Configure API keys**: See [CONFIGURATION.md](CONFIGURATION.md)
2. **Start the proxy**: See [USAGE.md](USAGE.md)
3. **View the dashboard**: Open http://localhost:8000
4. **Enable auto-updates**: `cco config set updates.auto_install true`

## Support

If you encounter issues:

1. Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. Search [existing issues](https://github.com/brentley/cco-releases/issues)
3. Create a [new issue](https://github.com/brentley/cco-releases/issues/new) with:
   - Operating system and version
   - Architecture (x86_64, ARM64)
   - Installation method used
   - Full error messages
   - Output of `cco --version` (if installed)

---

Last updated: 2025-11-15
