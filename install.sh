#!/usr/bin/env bash
#
# CCO Installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/USER/cc-orchestra/main/install.sh | bash
#
# This script downloads and installs the CCO binary to /usr/local/bin

set -e

# Configuration
REPO="USER/cc-orchestra"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="cco"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Header
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}   CCO Installer - Claude Code Orchestra${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}✨ No system dependencies required!${NC}"
echo -e "${GREEN}   - No macFUSE installation needed${NC}"
echo -e "${GREEN}   - No kernel extensions${NC}"
echo -e "${GREEN}   - Works on macOS, Linux, and Windows${NC}"
echo ""

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

log_info "Detected platform: $OS-$ARCH"

# Map architecture names
case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        if [ "$OS" = "darwin" ]; then
            ARCH="arm64"
        fi
        ;;
    *)
        log_error "Unsupported architecture: $ARCH"
        log_info "Supported architectures: x86_64, aarch64"
        exit 1
        ;;
esac

# Determine platform-specific binary name
case "$OS" in
    darwin)
        if [ "$ARCH" = "arm64" ]; then
            PLATFORM="darwin-arm64"
        else
            PLATFORM="darwin-x86_64"
        fi
        ;;
    linux)
        if [ "$ARCH" = "aarch64" ]; then
            PLATFORM="linux-aarch64"
        else
            PLATFORM="linux-x86_64"
        fi
        ;;
    *)
        log_error "Unsupported operating system: $OS"
        log_info "Supported systems: darwin (macOS), linux"
        exit 1
        ;;
esac

log_success "Platform identified: $PLATFORM"

# Get latest version from GitHub API
log_info "Fetching latest release version..."
VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    log_error "Failed to fetch latest version"
    log_info "You can manually download from: https://github.com/$REPO/releases"
    exit 1
fi

log_success "Latest version: $VERSION"

# Construct download URL
ARCHIVE_NAME="cco-v${VERSION}-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARCHIVE_NAME}"
CHECKSUM_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARCHIVE_NAME%.tar.gz}.sha256"

log_info "Download URL: $DOWNLOAD_URL"

# Create temporary directory
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$TMP_DIR"

# Download binary archive
log_info "Downloading CCO v$VERSION..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_NAME"; then
    log_error "Download failed"
    log_info "Please check your internet connection and try again"
    exit 1
fi

log_success "Downloaded $ARCHIVE_NAME"

# Download and verify checksum
log_info "Verifying checksum..."
if curl -fsSL "$CHECKSUM_URL" -o checksum.txt 2>/dev/null; then
    EXPECTED_CHECKSUM=$(cat checksum.txt | awk '{print $1}')

    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL_CHECKSUM=$(sha256sum "$ARCHIVE_NAME" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL_CHECKSUM=$(shasum -a 256 "$ARCHIVE_NAME" | awk '{print $1}')
    else
        log_warn "sha256sum or shasum not found, skipping verification"
        ACTUAL_CHECKSUM="$EXPECTED_CHECKSUM"  # Skip verification
    fi

    if [ "$EXPECTED_CHECKSUM" = "$ACTUAL_CHECKSUM" ]; then
        log_success "Checksum verified"
    else
        log_error "Checksum mismatch!"
        log_error "Expected: $EXPECTED_CHECKSUM"
        log_error "Got: $ACTUAL_CHECKSUM"
        exit 1
    fi
else
    log_warn "Checksum verification skipped (not available)"
fi

# Extract archive
log_info "Extracting archive..."
tar xzf "$ARCHIVE_NAME"

if [ ! -f "$BINARY_NAME" ]; then
    log_error "Binary not found in archive"
    exit 1
fi

# Make binary executable
chmod +x "$BINARY_NAME"

# Test binary
log_info "Testing binary..."
if ! ./"$BINARY_NAME" version >/dev/null 2>&1; then
    log_error "Binary test failed"
    exit 1
fi

VERSION_OUTPUT=$(./$ BINARY_NAME version 2>&1 || true)
log_success "Binary test passed"

# Check if we need sudo for installation
if [ -w "$INSTALL_DIR" ]; then
    SUDO=""
else
    SUDO="sudo"
    log_info "Administrator privileges required for installation to $INSTALL_DIR"
fi

# Install binary
log_info "Installing to $INSTALL_DIR/$BINARY_NAME..."
if [ -n "$SUDO" ]; then
    $SUDO mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
    mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

log_success "CCO installed successfully!"

# macOS-specific: Remove quarantine attribute
if [ "$OS" = "darwin" ]; then
    log_info "Removing quarantine attribute (macOS Gatekeeper)..."
    $SUDO xattr -d com.apple.quarantine "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || true
fi

# Verify installation
if command -v cco >/dev/null 2>&1; then
    log_success "Installation verified"
else
    log_warn "Installation succeeded but 'cco' not found in PATH"
    log_info "You may need to restart your terminal or add $INSTALL_DIR to your PATH"
fi

# Display version
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
log_success "Installation Complete!"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Installed version:"
cco version 2>&1 || echo "$VERSION_OUTPUT"
echo ""

# Post-installation instructions
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo ""
echo "1. Set your Anthropic API key:"
echo "   export ANTHROPIC_API_KEY='sk-ant-...'"
echo ""
echo "2. Start CCO:"
echo "   cco run"
echo ""
echo "3. View dashboard:"
echo "   Open http://localhost:3000 in your browser"
echo ""
echo "4. (Optional) Install as daemon:"
if [ "$OS" = "darwin" ]; then
    echo "   cco install  # macOS launchd"
else
    echo "   sudo systemctl enable cco  # Linux systemd"
fi
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "For help and documentation:"
echo "  cco --help"
echo "  https://github.com/$REPO"
echo ""
