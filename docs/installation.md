# CCO Installation Guide

## System Requirements

- **Operating Systems:** macOS, Linux, Windows (WSL2)
- **Architecture:** x86_64 (Intel/AMD), aarch64 (ARM64/Apple Silicon)
- **Disk Space:** ~50MB
- **Dependencies:** None (statically linked binary)

## Installation Methods

### Method 1: Quick Install Script (Recommended)

The fastest way to install CCO is using the install script, which automatically detects your platform and configures your PATH:

```bash
curl -sSL https://raw.githubusercontent.com/brentley/cco/main/install.sh | bash
```

This script will:
- Detect your platform automatically
- Download the correct binary
- Install to `~/.local/bin/cco`
- Automatically configure your shell's PATH
- No sudo required

After installation, restart your shell or run:
```bash
source ~/.zshrc    # macOS (zsh)
source ~/.bashrc   # Linux (bash)
```

### Method 2: Manual Installation

#### Step 1: Download

Visit the [Releases page](https://github.com/brentley/cco/releases/latest) and download the appropriate binary for your platform:

- **macOS (Intel):** `cco-Darwin-x86_64`
- **macOS (Apple Silicon):** `cco-Darwin-aarch64`
- **Linux (Intel/AMD):** `cco-Linux-x86_64`
- **Linux (ARM):** `cco-Linux-aarch64`

Or use `curl`:

```bash
# macOS Apple Silicon
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-Darwin-aarch64

# macOS Intel
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-Darwin-x86_64

# Linux x86_64
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-Linux-x86_64

# Linux ARM64
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-Linux-aarch64
```

#### Step 2: Install to User Directory

```bash
# Create local bin directory (if it doesn't exist)
mkdir -p ~/.local/bin

# Move binary and make executable
mv cco-* ~/.local/bin/cco
chmod +x ~/.local/bin/cco
```

#### Step 3: Configure PATH

Add `~/.local/bin` to your PATH if not already configured:

```bash
# zsh (macOS default)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# bash (Linux default)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# fish shell
echo 'set -gx PATH $HOME/.local/bin $PATH' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

#### Step 4: Verify Installation

```bash
cco --version
```

### Method 3: Homebrew (macOS/Linux)

CCO can be installed via Homebrew:

```bash
brew tap brentley/tap
brew install cco
```

**Note:** The Homebrew installation also installs to `~/.local/bin/cco` (not Homebrew's bin directory) and configures your PATH automatically.

### Method 4: Build from Source

If you have access to the source repository and Rust toolchain installed:

```bash
# Clone the source repository
git clone <source-repo-url>
cd cco

# Build release binary
cargo build --release

# Install to user directory (no sudo required)
mkdir -p ~/.local/bin
cp target/release/cco ~/.local/bin/cco
```

## Configuration

After installation, CCO needs to be configured with your credentials.

See [Configuration Guide](./configuration.md) for details.

## Verifying Installation

Run these commands to verify CCO is properly installed:

```bash
# Check version
cco --version

# Check available commands
cco --help

# Verify configuration (if already configured)
cco config show
```

## Updating CCO

CCO includes built-in auto-update functionality. When a new version is available, CCO will automatically download and install it to `~/.local/bin/cco`.

You can also manually update using one of these methods:

### Method 1: Built-in Update Command

```bash
cco update
```

### Method 2: Homebrew (if installed via Homebrew)

```bash
brew upgrade cco
```

### Method 3: Manual Update

```bash
# Download the new version
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-<platform>

# Replace the old binary
chmod +x cco-<platform>
mv cco-<platform> ~/.local/bin/cco

# Verify new version
cco --version
```

## Uninstalling

To remove CCO:

```bash
# Remove the binary
rm ~/.local/bin/cco

# Remove configuration (optional)
rm -rf ~/.config/cco
rm -rf ~/.local/share/cco

# If installed via Homebrew
brew uninstall cco
```

## Troubleshooting

### Command Not Found

If you get "command not found" after installation:

1. Check that `~/.local/bin` is in your PATH:
   ```bash
   echo $PATH | grep -o "$HOME/.local/bin"
   ```

2. If missing, add it to your shell configuration:
   ```bash
   # zsh (macOS default)
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc

   # bash (Linux default)
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

3. Restart your terminal or source your shell configuration

### Migrating from Old Installation

If you previously installed CCO to `/usr/local/bin`, you should migrate to the new location:

```bash
# Remove old installation (requires sudo)
sudo rm /usr/local/bin/cco

# Reinstall using quick install script
curl -sSL https://raw.githubusercontent.com/brentley/cco/main/install.sh | bash

# Or manually move to new location
sudo mv /usr/local/bin/cco ~/.local/bin/cco
chmod +x ~/.local/bin/cco
```

### Permission Denied

If you get "permission denied" when running `cco`:

```bash
chmod +x ~/.local/bin/cco
```

### macOS Security Warning

On macOS, you may see a security warning for unsigned binaries:

1. Go to **System Preferences** > **Security & Privacy**
2. Click "Allow" next to the CCO warning
3. Or bypass Gatekeeper:
   ```bash
   xattr -rd com.apple.quarantine ~/.local/bin/cco
   ```

### Binary Not Updating

If auto-updates aren't working or you see stale versions:

1. Check the current version:
   ```bash
   cco --version
   ```

2. Manually trigger an update:
   ```bash
   cco update
   ```

3. Verify the binary location:
   ```bash
   which cco
   # Should output: /Users/<username>/.local/bin/cco (macOS)
   # Should output: /home/<username>/.local/bin/cco (Linux)
   ```

For more issues, see [Troubleshooting Guide](./troubleshooting.md).

## Next Steps

- [Configure CCO](./configuration.md)
- [Learn available commands](./commands.md)
- [Explore features](./features.md)
