# CCO Installation Guide

## System Requirements

- **Operating Systems:** macOS, Linux, Windows (WSL2)
- **Architecture:** x86_64 (Intel/AMD), aarch64 (ARM64/Apple Silicon)
- **Disk Space:** ~50MB
- **Dependencies:** None (statically linked binary)

## Installation Methods

### Method 1: Download Pre-built Binary (Recommended)

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

#### Step 2: Make Executable

```bash
chmod +x cco-*
```

#### Step 3: Move to PATH

```bash
# System-wide installation (requires sudo)
sudo mv cco-* /usr/local/bin/cco

# User-local installation (no sudo required)
mkdir -p ~/.local/bin
mv cco-* ~/.local/bin/cco
# Add ~/.local/bin to your PATH if not already added
export PATH="$HOME/.local/bin:$PATH"
```

#### Step 4: Verify Installation

```bash
cco --version
```

### Method 2: Build from Source

If you have access to the source repository and Rust toolchain installed:

```bash
# Clone the source repository
git clone <source-repo-url>
cd cco

# Build release binary
cargo build --release

# Binary will be at target/release/cco
sudo cp target/release/cco /usr/local/bin/cco
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

To update to a new version:

```bash
# Download the new version
curl -LO https://github.com/brentley/cco/releases/latest/download/cco-<platform>

# Replace the old binary
chmod +x cco-<platform>
sudo mv cco-<platform> /usr/local/bin/cco

# Verify new version
cco --version
```

## Uninstalling

To remove CCO:

```bash
# Remove the binary
sudo rm /usr/local/bin/cco

# Remove configuration (optional)
rm -rf ~/.config/cco
rm -rf ~/.local/share/cco
```

## Troubleshooting

### Command Not Found

If you get "command not found" after installation:

1. Check that `/usr/local/bin` is in your PATH:
   ```bash
   echo $PATH
   ```

2. Add it if missing (add to `~/.bashrc` or `~/.zshrc`):
   ```bash
   export PATH="/usr/local/bin:$PATH"
   ```

### Permission Denied

If you get "permission denied" when running `cco`:

```bash
chmod +x /usr/local/bin/cco
```

### macOS Security Warning

On macOS, you may see a security warning for unsigned binaries:

1. Go to **System Preferences** > **Security & Privacy**
2. Click "Allow" next to the CCO warning
3. Or bypass Gatekeeper:
   ```bash
   sudo xattr -rd com.apple.quarantine /usr/local/bin/cco
   ```

For more issues, see [Troubleshooting Guide](./troubleshooting.md).

## Next Steps

- [Configure CCO](./configuration.md)
- [Learn available commands](./commands.md)
- [Explore features](./features.md)
