# CCO Installation Guide

## System Requirements

- **Operating Systems:** macOS, Linux, Windows (WSL2)
- **Architecture:** x86_64 (Intel/AMD), aarch64 (ARM64/Apple Silicon)
- **Disk Space:** ~50MB
- **Dependencies:** None (statically linked binary)

## Installation Methods

<<### Method 1: Install via GitHub Releases (Recommended)

#### One-liner (macOS/Linux)

```bash
# Installs to ~/.local/bin/cco by default
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/scripts/install_from_github.sh | bash
# Optional: export GITHUB_TOKEN to raise GitHub API limits
```

#### Manual download

Visit the [Releases page](https://github.com/visiquate/cco/releases/latest) and download the appropriate archive for your platform:

- **macOS (Apple Silicon):** `cco-aarch64-apple-darwin.tar.gz`
- **macOS (Intel):** `cco-x86_64-apple-darwin.tar.gz` (if published)
- **Linux (Intel/AMD):** `cco-x86_64-unknown-linux-gnu.tar.gz`
- **Linux (ARM64):** `cco-aarch64-unknown-linux-gnu.tar.gz` (if published)

Example:

```bash
# macOS Apple Silicon
curl -LO https://github.com/visiquate/cco/releases/latest/download/cco-aarch64-apple-darwin.tar.gz

# Linux x86_64
curl -LO https://github.com/visiquate/cco/releases/latest/download/cco-x86_64-unknown-linux-gnu.tar.gz
```

#### Step 2: Extract and install

```bash
tar -xzf cco-*.tar.gz
chmod +x cco
# System-wide installation (requires sudo)
# sudo mv cco /usr/local/bin/cco

# User-local installation (no sudo required)
mkdir -p ~/.local/bin
mv cco ~/.local/bin/cco
# Add ~/.local/bin to your PATH if not already added
export PATH="$HOME/.local/bin:$PATH"
```

#### Step 3: Verify Installation

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

### Method 3: Homebrew (macOS/Linux)

```bash
brew tap visiquate/cco
brew install visiquate/cco/cco
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
