#!/bin/bash
# CCO Installer Script
# One-line installation: curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

set -e

# Configuration
GITHUB_ORG="brentley"
GITHUB_REPO="cco-releases"
INSTALL_DIR="${CCO_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/cco"
MANIFEST_URL="https://raw.githubusercontent.com/$GITHUB_ORG/$GITHUB_REPO/main/version-manifest.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; exit 1; }

# Parse command line arguments
VERSION=""
CHANNEL="stable"
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --channel)
            CHANNEL="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help)
            cat << EOF
CCO Installer

Usage:
    curl -fsSL $MANIFEST_URL | bash
    curl -fsSL $MANIFEST_URL | bash -s -- [options]

Options:
    --version VERSION     Install specific version (default: latest)
    --channel CHANNEL     Release channel: stable, beta, nightly (default: stable)
    --install-dir DIR     Installation directory (default: ~/.local/bin)
    --force              Force reinstallation
    --help               Show this help message

Examples:
    # Install latest stable version
    curl -fsSL $MANIFEST_URL | bash

    # Install specific version
    curl -fsSL $MANIFEST_URL | bash -s -- --version 0.3.0

    # Install beta channel
    curl -fsSL $MANIFEST_URL | bash -s -- --channel beta

EOF
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Detect platform and architecture
detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"

    case "$os" in
        Darwin)
            PLATFORM="darwin"
            EXT="tar.gz"
            ;;
        Linux)
            PLATFORM="linux"
            EXT="tar.gz"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            PLATFORM="windows"
            EXT="zip"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="arm64"
            ;;
        armv7l)
            ARCH="armv7"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    PLATFORM_STRING="${PLATFORM}-${ARCH}"
    info "Detected platform: $PLATFORM_STRING"
}

# Check for required tools
check_dependencies() {
    local deps=("curl" "tar" "grep")

    for cmd in "${deps[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error "Required command '$cmd' not found. Please install it and try again."
        fi
    done

    # Check for jq (optional but helpful)
    if ! command -v jq &> /dev/null; then
        warning "jq is not installed. Using alternative JSON parsing."
        USE_JQ=false
    else
        USE_JQ=true
    fi
}

# Fetch version manifest
fetch_manifest() {
    info "Fetching version manifest..."

    MANIFEST_CONTENT=$(curl -fsSL "$MANIFEST_URL" 2>/dev/null) || {
        error "Failed to fetch version manifest from $MANIFEST_URL"
    }

    if [ -z "$VERSION" ]; then
        if [ "$USE_JQ" = true ]; then
            VERSION=$(echo "$MANIFEST_CONTENT" | jq -r ".latest.$CHANNEL")
        else
            # Fallback JSON parsing
            VERSION=$(echo "$MANIFEST_CONTENT" | grep -o "\"$CHANNEL\":\"[^\"]*\"" | cut -d'"' -f4)
        fi

        if [ -z "$VERSION" ]; then
            error "Failed to determine latest version for channel: $CHANNEL"
        fi

        info "Latest $CHANNEL version: $VERSION"
    else
        info "Installing specified version: $VERSION"
    fi
}

# Check if already installed
check_existing_installation() {
    if [ -f "$INSTALL_DIR/cco" ] && [ "$FORCE" = false ]; then
        CURRENT_VERSION=$("$INSTALL_DIR/cco" --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")

        if [ "$CURRENT_VERSION" = "$VERSION" ]; then
            success "CCO v$VERSION is already installed at $INSTALL_DIR/cco"
            exit 0
        else
            info "Updating CCO from v$CURRENT_VERSION to v$VERSION"
        fi
    fi
}

# Create installation directory
create_install_dir() {
    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR" || error "Failed to create installation directory"
    fi
}

# Download and extract binary
download_binary() {
    local download_url="https://github.com/$GITHUB_ORG/$GITHUB_REPO/releases/download/v$VERSION/cco-v$VERSION-$PLATFORM_STRING.$EXT"
    local checksum_url="https://github.com/$GITHUB_ORG/$GITHUB_REPO/releases/download/v$VERSION/checksums.sha256"
    local tmp_dir=$(mktemp -d)

    info "Downloading CCO v$VERSION for $PLATFORM_STRING..."

    # Download binary archive
    curl -fsSL "$download_url" -o "$tmp_dir/cco.$EXT" || {
        rm -rf "$tmp_dir"
        error "Failed to download CCO from $download_url"
    }

    # Download and verify checksum
    info "Verifying download integrity..."
    curl -fsSL "$checksum_url" -o "$tmp_dir/checksums.sha256" 2>/dev/null || {
        warning "Could not download checksums. Skipping verification."
    }

    if [ -f "$tmp_dir/checksums.sha256" ]; then
        cd "$tmp_dir"
        grep "cco-v$VERSION-$PLATFORM_STRING.$EXT" checksums.sha256 > expected.sha256 || true

        if [ -s expected.sha256 ]; then
            if command -v sha256sum &> /dev/null; then
                mv "cco.$EXT" "cco-v$VERSION-$PLATFORM_STRING.$EXT"
                sha256sum -c expected.sha256 || {
                    rm -rf "$tmp_dir"
                    error "Checksum verification failed!"
                }
                mv "cco-v$VERSION-$PLATFORM_STRING.$EXT" "cco.$EXT"
            elif command -v shasum &> /dev/null; then
                mv "cco.$EXT" "cco-v$VERSION-$PLATFORM_STRING.$EXT"
                shasum -a 256 -c expected.sha256 || {
                    rm -rf "$tmp_dir"
                    error "Checksum verification failed!"
                }
                mv "cco-v$VERSION-$PLATFORM_STRING.$EXT" "cco.$EXT"
            else
                warning "Neither sha256sum nor shasum found. Skipping checksum verification."
            fi
        fi
        cd - > /dev/null
    fi

    # Extract binary
    info "Extracting CCO binary..."
    case "$EXT" in
        tar.gz)
            tar -xzf "$tmp_dir/cco.$EXT" -C "$tmp_dir" || {
                rm -rf "$tmp_dir"
                error "Failed to extract archive"
            }
            ;;
        zip)
            unzip -q "$tmp_dir/cco.$EXT" -d "$tmp_dir" || {
                rm -rf "$tmp_dir"
                error "Failed to extract archive"
            }
            ;;
    esac

    # Backup existing binary if it exists
    if [ -f "$INSTALL_DIR/cco" ]; then
        mv "$INSTALL_DIR/cco" "$INSTALL_DIR/cco.backup"
        info "Backed up existing installation to $INSTALL_DIR/cco.backup"
    fi

    # Install binary
    mv "$tmp_dir/cco" "$INSTALL_DIR/cco" || {
        # Restore backup on failure
        if [ -f "$INSTALL_DIR/cco.backup" ]; then
            mv "$INSTALL_DIR/cco.backup" "$INSTALL_DIR/cco"
        fi
        rm -rf "$tmp_dir"
        error "Failed to install CCO binary"
    }

    # Set permissions
    chmod 755 "$INSTALL_DIR/cco"

    # Clean up
    rm -rf "$tmp_dir"
    rm -f "$INSTALL_DIR/cco.backup"

    success "CCO v$VERSION installed successfully!"
}

# Update shell configuration
update_shell_config() {
    local shell_name="$(basename "$SHELL")"
    local shell_rc=""

    case "$shell_name" in
        bash)
            shell_rc="$HOME/.bashrc"
            [ -f "$HOME/.bash_profile" ] && shell_rc="$HOME/.bash_profile"
            ;;
        zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            warning "Unknown shell: $shell_name. Please manually add $INSTALL_DIR to your PATH."
            return
            ;;
    esac

    # Check if PATH already contains install directory
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        info "Installation directory already in PATH"
        return
    fi

    # Check if shell config already has the PATH export
    if [ -f "$shell_rc" ] && grep -q "$INSTALL_DIR" "$shell_rc"; then
        info "PATH configuration already exists in $shell_rc"
        return
    fi

    # Add to PATH
    info "Adding $INSTALL_DIR to PATH in $shell_rc"

    if [ "$shell_name" = "fish" ]; then
        echo "set -gx PATH \"$INSTALL_DIR\" \$PATH" >> "$shell_rc"
    else
        echo "" >> "$shell_rc"
        echo "# Added by CCO installer" >> "$shell_rc"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
    fi

    warning "Please restart your shell or run: source $shell_rc"
}

# Create default configuration
create_default_config() {
    if [ ! -d "$CONFIG_DIR" ]; then
        info "Creating configuration directory: $CONFIG_DIR"
        mkdir -p "$CONFIG_DIR"
    fi

    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        info "Creating default configuration..."
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
# CCO Configuration

[updates]
enabled = true
auto_install = false
check_interval = "daily"
channel = "stable"
notify_on_update = true
verify_signatures = true

[updates.schedule]
# Automatically populated by CCO

[telemetry]
enabled = false
EOF
        success "Default configuration created at $CONFIG_DIR/config.toml"
    fi
}

# Verify installation
verify_installation() {
    if [ -f "$INSTALL_DIR/cco" ]; then
        info "Verifying installation..."

        # Test execution
        if "$INSTALL_DIR/cco" --version &> /dev/null; then
            local installed_version=$("$INSTALL_DIR/cco" --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
            success "CCO v$installed_version is ready to use!"

            echo ""
            echo "Getting started:"
            echo "  1. Restart your shell or run: source ~/.$(basename "$SHELL")rc"
            echo "  2. Run: cco --help"
            echo "  3. Configure: cco config show"
            echo ""
            echo "For documentation, visit: https://github.com/$GITHUB_ORG/$GITHUB_REPO"
        else
            error "Installation verification failed. CCO binary may be corrupted."
        fi
    else
        error "Installation failed. CCO binary not found at $INSTALL_DIR/cco"
    fi
}

# Main installation flow
main() {
    echo ""
    echo "==========================================="
    echo "       CCO (Claude Code Orchestra)        "
    echo "         Installation Script v1.0          "
    echo "==========================================="
    echo ""

    detect_platform
    check_dependencies
    fetch_manifest
    check_existing_installation
    create_install_dir
    download_binary
    update_shell_config
    create_default_config
    verify_installation

    echo ""
    success "Installation complete!"
    echo ""
}

# Run main function
main "$@"