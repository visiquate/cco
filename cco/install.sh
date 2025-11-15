#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
PROJECT_NAME="Claude Orchestra Build System (CCO)"
BINARY_NAME="cco"
MIN_RUST_VERSION="1.70"

# Functions
print_header() {
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    print_success "Rust/Cargo found"

    # Check Rust version
    local rust_version=$(cargo --version | awk '{print $2}')
    print_info "Rust version: $rust_version"

    # Check if git is available
    if ! command -v git &> /dev/null; then
        print_warning "Git not found - version info will be limited"
    else
        print_success "Git found"
    fi
}

# Build release binary
build_binary() {
    print_header "Building $PROJECT_NAME (Release)"

    if ! cargo build --release; then
        print_error "Build failed"
        exit 1
    fi
    print_success "Build complete"

    # Verify binary exists
    local binary_path="target/release/$BINARY_NAME"
    if [ ! -f "$binary_path" ]; then
        print_error "Binary not found at $binary_path"
        exit 1
    fi
    print_success "Binary found: $binary_path"
    print_info "Binary size: $(du -h "$binary_path" | cut -f1)"
}

# Strip binary symbols
strip_binary() {
    print_info "Stripping debug symbols..."
    local binary_path="target/release/$BINARY_NAME"

    if command -v strip &> /dev/null; then
        strip "$binary_path" || print_warning "Strip failed (non-critical)"
        print_success "Binary stripped"
        print_info "Stripped size: $(du -h "$binary_path" | cut -f1)"
    else
        print_warning "Strip command not found - skipping symbol stripping"
    fi
}

# Copy binary to install directory
install_binary() {
    print_header "Installing $BINARY_NAME"

    # Check if install directory is writable
    if [ ! -d "$INSTALL_DIR" ]; then
        print_info "Creating install directory: $INSTALL_DIR"
        if ! mkdir -p "$INSTALL_DIR"; then
            print_error "Failed to create $INSTALL_DIR"
            print_info "Try: sudo mkdir -p $INSTALL_DIR"
            exit 1
        fi
    fi

    if [ ! -w "$INSTALL_DIR" ]; then
        print_error "Install directory not writable: $INSTALL_DIR"
        print_info "Try: sudo chown $(whoami) $INSTALL_DIR"
        print_info "Or: sudo cp target/release/$BINARY_NAME $INSTALL_DIR/$BINARY_NAME"
        exit 1
    fi

    # Copy binary
    print_info "Copying $BINARY_NAME to $INSTALL_DIR..."
    if cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"; then
        print_success "Binary copied"
    else
        print_error "Failed to copy binary"
        print_info "Try: sudo cp target/release/$BINARY_NAME $INSTALL_DIR/$BINARY_NAME"
        exit 1
    fi

    # Make executable
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    print_success "Binary made executable"
}

# Verify installation
verify_installation() {
    print_header "Verifying Installation"

    # Check if binary exists in PATH
    if command -v "$BINARY_NAME" &> /dev/null; then
        print_success "$BINARY_NAME found in PATH"
        local installed_path=$(command -v "$BINARY_NAME")
        print_info "Location: $installed_path"
    else
        print_warning "$BINARY_NAME not in PATH"
        print_info "Binary installed at: $INSTALL_DIR/$BINARY_NAME"
        print_info "Add $INSTALL_DIR to your PATH or update shell config"
    fi

    # Try to run version command if available
    if "$INSTALL_DIR/$BINARY_NAME" --version &> /dev/null; then
        print_success "Binary version check passed"
        "$INSTALL_DIR/$BINARY_NAME" --version
    else
        print_info "Binary found but --version not available"
    fi
}

# Print summary
print_summary() {
    print_header "Installation Summary"
    echo ""
    echo "Installation completed successfully!"
    echo ""
    echo "Binary location: $INSTALL_DIR/$BINARY_NAME"
    echo ""

    if command -v "$BINARY_NAME" &> /dev/null; then
        echo "You can now run: $BINARY_NAME"
    else
        echo "Add to your PATH or use: $INSTALL_DIR/$BINARY_NAME"
    fi
    echo ""
    echo "Useful commands:"
    echo "  $BINARY_NAME --help       Show help"
    echo "  $BINARY_NAME --version    Show version"
    echo ""
}

# Main installation process
main() {
    print_header "$PROJECT_NAME Installation Script"
    echo ""

    # Parse command line arguments
    case "${1:-}" in
        --help|-h)
            cat << EOF
Usage: ./install.sh [OPTIONS]

Options:
  --install-dir <dir>    Installation directory (default: /usr/local/bin)
  --skip-verify         Skip verification checks
  --help, -h            Show this help message

Environment variables:
  INSTALL_DIR          Override installation directory

Examples:
  ./install.sh
  ./install.sh --install-dir ~/.local/bin
  INSTALL_DIR=/opt/bin ./install.sh
EOF
            exit 0
            ;;
        --install-dir)
            INSTALL_DIR="${2:-/usr/local/bin}"
            ;;
    esac

    # Run installation steps
    check_prerequisites
    build_binary
    strip_binary
    install_binary
    verify_installation
    print_summary

    print_success "Installation complete!"
}

# Run main function
main "$@"
