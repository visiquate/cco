#!/usr/bin/env bash
#
# CCO Installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | bash
#
# This script downloads and installs the CCO binary to ~/.local/bin

set -e

# Configuration
REPO="visiquate/cco"
INSTALL_DIR="$HOME/.local/bin"
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

# Shell detection functions
detect_shell() {
    basename "$SHELL"
}

get_shell_rc_path() {
    local shell="$1"
    local home="$HOME"

    case "$shell" in
        zsh)
            echo "$home/.zshrc"
            ;;
        bash)
            if [ "$(uname -s)" = "Darwin" ]; then
                echo "$home/.bash_profile"
            else
                echo "$home/.bashrc"
            fi
            ;;
        fish)
            echo "$home/.config/fish/config.fish"
            ;;
        *)
            return 1
            ;;
    esac
}

is_in_path() {
    echo "$PATH" | grep -q "$HOME/.local/bin"
}

update_shell_rc() {
    local shell="$1"
    local rc_path

    rc_path=$(get_shell_rc_path "$shell")
    if [ $? -ne 0 ]; then
        return 1
    fi

    # Create parent directory if needed (for fish)
    mkdir -p "$(dirname "$rc_path")"

    # Check if PATH export already exists
    if [ -f "$rc_path" ]; then
        if grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$rc_path" || \
           grep -q 'export PATH="~/.local/bin:$PATH"' "$rc_path" || \
           grep -q 'set -gx PATH $HOME/.local/bin $PATH' "$rc_path"; then
            log_info "PATH already configured in $rc_path"
            return 0
        fi
    fi

    # Append PATH export to RC file
    if [ "$shell" = "fish" ]; then
        echo "" >> "$rc_path"
        echo "# Added by CCO installer" >> "$rc_path"
        echo "set -gx PATH \$HOME/.local/bin \$PATH" >> "$rc_path"
    else
        echo "" >> "$rc_path"
        echo "# Added by CCO installer" >> "$rc_path"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rc_path"
    fi

    log_success "Updated $rc_path with PATH configuration"
    return 0
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

# Determine platform-specific binary name (using Rust target triples)
case "$OS" in
    darwin)
        if [ "$ARCH" = "arm64" ]; then
            PLATFORM="aarch64-apple-darwin"
        else
            PLATFORM="x86_64-apple-darwin"
        fi
        ;;
    linux)
        if [ "$ARCH" = "aarch64" ]; then
            PLATFORM="aarch64-unknown-linux-gnu"
        else
            PLATFORM="x86_64-unknown-linux-gnu"
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

# Construct download URL (release assets use target triple naming without version)
ARCHIVE_NAME="cco-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARCHIVE_NAME}"
CHECKSUM_URL="https://github.com/$REPO/releases/download/v${VERSION}/checksums.txt"

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
if curl -fsSL "$CHECKSUM_URL" -o checksums.txt 2>/dev/null; then
    # Extract checksum for our specific archive from checksums.txt
    EXPECTED_CHECKSUM=$(grep "$ARCHIVE_NAME" checksums.txt | awk '{print $1}')

    if [ -z "$EXPECTED_CHECKSUM" ]; then
        log_warn "Checksum not found in checksums.txt for $ARCHIVE_NAME"
    else
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

VERSION_OUTPUT=$(./"$BINARY_NAME" version 2>&1 || true)
log_success "Binary test passed"

# Create installation directory
log_info "Creating $INSTALL_DIR/"
mkdir -p "$INSTALL_DIR"

# Install binary
log_info "Installing to $INSTALL_DIR/$BINARY_NAME..."
mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

log_success "CCO installed successfully!"

# macOS-specific: Remove quarantine attribute
if [ "$OS" = "darwin" ]; then
    log_info "Removing quarantine attribute (macOS Gatekeeper)..."
    xattr -d com.apple.quarantine "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || true
fi

# Configure PATH
log_info "Configuring shell PATH..."
DETECTED_SHELL=$(detect_shell)

if [ -n "$DETECTED_SHELL" ]; then
    log_success "Detected shell: $DETECTED_SHELL"

    if is_in_path; then
        log_success "~/.local/bin is already in PATH"
    else
        if update_shell_rc "$DETECTED_SHELL"; then
            log_success "Shell configuration updated"
        else
            log_warn "Could not automatically update shell configuration"
            log_info "Manually add this to your shell RC file:"
            echo '  export PATH="$HOME/.local/bin:$PATH"'
        fi
    fi
else
    log_warn "Could not detect shell"
    log_info "Manually add this to your shell RC file:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
fi

# Display version
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
log_success "Installation Complete!"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Installed version:"
"$INSTALL_DIR/$BINARY_NAME" version 2>&1 || echo "$VERSION_OUTPUT"
echo ""

# Automatic daemon setup
log_info "Setting up daemon..."
DAEMON_INSTALLED=false
DAEMON_STARTED=false

# Determine service file path based on platform
if [ "$OS" = "darwin" ]; then
    SERVICE_PATH="$HOME/Library/LaunchAgents/com.anthropic.cco.daemon.plist"
else
    SERVICE_PATH="$HOME/.config/systemd/user/cco-daemon.service"
fi

# Check if daemon is already installed
if [ -f "$SERVICE_PATH" ]; then
    log_success "Daemon already installed"
    DAEMON_INSTALLED=true
else
    # Try to install daemon
    DAEMON_OUTPUT=$("$INSTALL_DIR/$BINARY_NAME" daemon install 2>&1 || true)

    # Check if installation was successful or if service already exists
    if [ -f "$SERVICE_PATH" ] || echo "$DAEMON_OUTPUT" | grep -q "Service is already installed"; then
        log_success "Daemon installed"
        DAEMON_INSTALLED=true
    else
        log_warn "Could not install daemon automatically"
    fi
fi

# Try to start daemon if installed
if [ "$DAEMON_INSTALLED" = true ]; then
    if "$INSTALL_DIR/$BINARY_NAME" daemon status 2>/dev/null | grep -q "Running: true"; then
        log_success "Daemon already running"
        DAEMON_STARTED=true
    else
        if "$INSTALL_DIR/$BINARY_NAME" daemon start 2>/dev/null; then
            log_success "Daemon started"
            DAEMON_STARTED=true
        else
            log_warn "Could not start daemon automatically"
        fi
    fi
fi

echo ""

# Post-installation instructions
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo ""

# Detect if we're being piped (stdin is not a terminal)
IS_PIPED=false
if [ ! -t 0 ]; then
    IS_PIPED=true
fi

# Determine if PATH activation is needed
PATH_ACTIVATED=false
if is_in_path; then
    # PATH was already configured
    PATH_ACTIVATED=true
    log_info "PATH already active in current session"
elif [ -n "$DETECTED_SHELL" ]; then
    RC_PATH=$(get_shell_rc_path "$DETECTED_SHELL" 2>/dev/null)
    if [ -n "$RC_PATH" ] && [ -f "$RC_PATH" ]; then
        # Attempt to source the RC file in current context
        log_info "Attempting to activate PATH in current session..."

        # Try to source the RC file (will fail silently if piped)
        if [ "$IS_PIPED" = false ]; then
            # Not piped - sourcing might work
            if source "$RC_PATH" 2>/dev/null; then
                # Check if cco is now available
                if command -v cco >/dev/null 2>&1; then
                    PATH_ACTIVATED=true
                    log_success "PATH activated! 'cco' command is now available"
                fi
            fi
        fi
    fi
fi

# Smart instructions based on PATH activation status
STEP_NUM=1

if [ "$PATH_ACTIVATED" = false ]; then
    echo "${STEP_NUM}. Activate PATH (choose one):"
    if [ -n "$DETECTED_SHELL" ]; then
        RC_PATH=$(get_shell_rc_path "$DETECTED_SHELL" 2>/dev/null)
        if [ -n "$RC_PATH" ]; then
            echo -e "   ${GREEN}# Quick start (one-liner):${NC}"
            echo "   source $RC_PATH && cco version"
            echo ""
            echo -e "   ${GREEN}# OR restart your terminal${NC}"
        else
            echo "   Restart your terminal"
        fi
    else
        echo "   Restart your terminal"
    fi
    echo ""
    STEP_NUM=$((STEP_NUM + 1))
else
    echo "${GREEN}✓ PATH is active - 'cco' command ready to use${NC}"
    echo ""
fi

echo "${STEP_NUM}. Set your Anthropic API key:"
echo "   export ANTHROPIC_API_KEY='sk-ant-...'"
echo ""
STEP_NUM=$((STEP_NUM + 1))

# Only show daemon steps if automatic setup failed
if [ "$DAEMON_INSTALLED" = false ]; then
    echo "${STEP_NUM}. Install the daemon:"
    if [ "$OS" = "darwin" ]; then
        echo "   cco daemon install  # Install macOS launchd service"
    else
        echo "   cco daemon install  # Install systemd service"
    fi
    echo ""
    STEP_NUM=$((STEP_NUM + 1))
fi

if [ "$DAEMON_INSTALLED" = true ] && [ "$DAEMON_STARTED" = false ]; then
    echo "${STEP_NUM}. Start the daemon:"
    echo "   cco daemon start"
    echo ""
    STEP_NUM=$((STEP_NUM + 1))
fi

echo "${STEP_NUM}. View status with the TUI:"
echo "   cco tui"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "For help and documentation:"
echo "  cco --help"
echo "  https://github.com/$REPO"
echo ""
