#!/usr/bin/env bash
#!/usr/bin/env bash
# GitHub-based installer for CCO (macOS/Linux)
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/scripts/install_from_github.sh | bash

set -euo pipefail

REPO="visiquate/cco"
BINARY_NAME="cco"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
SYSTEM_INSTALL_DIR="/usr/local/bin"
TMP_DIR=""

log_info() { echo "[INFO] $1"; }
log_warn() { echo "[WARN] $1"; }
log_error() { echo "[ERROR] $1" >&2; }
fail() { log_error "$1"; exit 1; }

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        fail "Required command '$1' not found. Please install it and retry."
    fi
}

cleanup() {
    if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}
trap cleanup EXIT

require_cmd curl
require_cmd tar
require_cmd python3

# Detect platform
OS_RAW="$(uname -s)"
ARCH_RAW="$(uname -m)"

case "$OS_RAW" in
    Darwin) OS="darwin" ;;
    Linux) OS="linux" ;;
    MINGW*|MSYS*|CYGWIN*)
        fail "Windows installation is not supported by this script. Download the zip from GitHub Releases."
        ;;
    *)
        fail "Unsupported operating system: $OS_RAW"
        ;;
 esac

case "$ARCH_RAW" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64)
        if [ "$OS" = "darwin" ]; then
            ARCH="arm64"
        else
            ARCH="aarch64"
        fi
        ;;
    *)
        fail "Unsupported architecture: $ARCH_RAW"
        ;;
 esac

case "$OS" in
    darwin)
        if [ "$ARCH" = "arm64" ]; then
            TARGET_TRIPLE="aarch64-apple-darwin"
        else
            TARGET_TRIPLE="x86_64-apple-darwin"
        fi
        ;;
    linux)
        if [ "$ARCH" = "aarch64" ]; then
            TARGET_TRIPLE="aarch64-unknown-linux-gnu"
        else
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
        fi
        ;;
    *)
        fail "Unsupported platform: $OS $ARCH"
        ;;
esac

TARGET="$OS-$ARCH"
log_info "Detected platform: $TARGET ($TARGET_TRIPLE)"

# Fetch latest stable release metadata
API_URL="https://api.github.com/repos/$REPO/releases/latest"
AUTH_HEADER=()
if [ -n "${GITHUB_TOKEN:-}" ]; then
    AUTH_HEADER=(-H "Authorization: Bearer $GITHUB_TOKEN")
fi

log_info "Fetching latest release info from GitHub..."
RELEASE_JSON="$(curl -fsSL -H "Accept: application/vnd.github+json" ${AUTH_HEADER[@]:+"${AUTH_HEADER[@]}"} "$API_URL")" || fail "Failed to fetch release metadata"

parse_release() {
    TARGET_TRIPLE="$1" python3 - <<'PY'
import json, os, sys
try:
    data = json.loads(os.environ.get("RELEASE_JSON", ""))
except Exception:
    sys.exit(1)

assets = data.get("assets") or []
tag = data.get("tag_name") or ""
target = os.environ["TARGET_TRIPLE"]
asset_name = f"cco-{target}.tar.gz"
asset = next((a for a in assets if a.get("name") == asset_name), None)
checksum = next((a for a in assets if a.get("name") == "checksums.txt"), None)
if not tag or asset is None or not asset.get("browser_download_url") or checksum is None:
    sys.exit(1)
print(tag)
print(asset.get("name", ""))
print(asset.get("browser_download_url", ""))
print(checksum.get("browser_download_url", ""))
PY
}

PARSED_INFO="$(RELEASE_JSON="$RELEASE_JSON" parse_release "$TARGET_TRIPLE" || true)"
if [ -z "$PARSED_INFO" ]; then
    fail "Could not locate a release asset for $TARGET"
fi

tag_name="$(printf '%s' "$PARSED_INFO" | sed -n '1p')"
asset_name="$(printf '%s' "$PARSED_INFO" | sed -n '2p')"
asset_url="$(printf '%s' "$PARSED_INFO" | sed -n '3p')"
checksum_url="$(printf '%s' "$PARSED_INFO" | sed -n '4p')"

if [ -z "$asset_url" ] || [ -z "$asset_name" ]; then
    fail "Release asset URL not found for $TARGET"
fi
if [ -z "$checksum_url" ]; then
    fail "checksums.txt not found in the latest release"
fi

log_info "Latest release: $tag_name"
log_info "Selected asset: $asset_name"

# Prepare workspace
TMP_DIR="$(mktemp -d)"
cd "$TMP_DIR"

# Download files
log_info "Downloading checksums.txt..."
curl -fsSL "$checksum_url" -o checksums.txt || fail "Failed to download checksums.txt"

log_info "Downloading $asset_name..."
curl -fsSL "$asset_url" -o "$asset_name" || fail "Failed to download release asset"

# Verify checksum
expected_checksum="$(awk -v n="$asset_name" '$2==n {print $1}' checksums.txt | head -n 1)"
if [ -z "$expected_checksum" ]; then
    fail "Checksum for $asset_name not found in checksums.txt"
fi

if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "$asset_name" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
    actual_checksum="$(shasum -a 256 "$asset_name" | awk '{print $1}')"
else
    fail "Neither sha256sum nor shasum is available for checksum verification"
fi

if [ "$expected_checksum" != "$actual_checksum" ]; then
    fail "Checksum mismatch for $asset_name"
fi
log_info "Checksum verified"

# Extract and locate binary
tar -xzf "$asset_name" || fail "Failed to extract archive"
binary_path="$(find "$TMP_DIR" -type f -name "$BINARY_NAME" -print -quit)"
if [ -z "$binary_path" ]; then
    fail "Could not find '$BINARY_NAME' in the archive"
fi
chmod +x "$binary_path"

# Choose install directory
install_dir="${CCO_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
if [ -z "${CCO_INSTALL_DIR:-}" ] && [ "${CCO_INSTALL_SYSTEM:-0}" != "0" ] && [ -w "$SYSTEM_INSTALL_DIR" ]; then
    install_dir="$SYSTEM_INSTALL_DIR"
fi
mkdir -p "$install_dir"
if [ ! -w "$install_dir" ]; then
    fail "Install directory $install_dir is not writable; set CCO_INSTALL_DIR or adjust permissions"
fi

install_path="$install_dir/$BINARY_NAME"
log_info "Installing to $install_path"

mv "$binary_path" "$install_path" || fail "Installation failed; check permissions for $install_dir"
chmod +x "$install_path"

# macOS quarantine removal
if [ "$OS" = "darwin" ]; then
    xattr -d com.apple.quarantine "$install_path" 2>/dev/null || true
fi

log_info "Running version check..."
if ! "$install_path" --version; then
    fail "Installed binary failed to run"
fi

# PATH hint
case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) log_warn "Add $install_dir to your PATH if commands are not found" ;;
esac

log_info "Installation complete"
