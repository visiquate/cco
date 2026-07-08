#!/usr/bin/env bash
# cco-plugin/hooks/session-start.sh
# Ensures the cco binary is present (downloading it if this is a fresh
# sandbox, e.g. Claude Code cloud/web) before delegating to the real
# `cco hook session-start` handler. Mirrors the platform-detection and
# download logic in visiquate/cco's install.sh, without the interactive
# shell-rc setup that script does (not meaningful in an ephemeral session).
set -euo pipefail

REPO="visiquate/cco"
INSTALL_DIR="$HOME/.local/bin"
MODE="${1:-startup}"

# Best-effort auto-install. Every risky step below returns 1 on failure
# instead of letting set -e kill the whole script, and the call site
# below wraps this in `|| true` so a flaky network never aborts the hook.
# On any failure we simply fall through to the end-of-script fallback.
try_install_cco() {
  local os arch platform version tmp_dir archive

  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64)
      arch="aarch64"
      [ "$os" = "darwin" ] && arch="arm64" || true
      ;;
  esac

  platform=""
  case "$os" in
    darwin)
      [ "$arch" = "arm64" ] && platform="aarch64-apple-darwin" || platform="x86_64-apple-darwin"
      ;;
    linux)
      [ "$arch" = "aarch64" ] && platform="aarch64-unknown-linux-gnu" || platform="x86_64-unknown-linux-gnu"
      ;;
  esac

  [ -n "$platform" ] || return 1

  version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
    | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/') || true
  [ -n "$version" ] || return 1

  tmp_dir="$(mktemp -d)" || return 1
  # Clear the trap as part of firing it, so it never lingers process-wide
  # and re-fires on the return of some later, unrelated function.
  trap 'rm -rf "$tmp_dir"; trap - RETURN' RETURN
  archive="cco-${platform}.tar.gz"

  curl -fsSL "https://github.com/$REPO/releases/download/v${version}/${archive}" \
    -o "$tmp_dir/$archive" 2>/dev/null || return 1

  tar xzf "$tmp_dir/$archive" -C "$tmp_dir" || return 1
  mkdir -p "$INSTALL_DIR" || return 1
  mv "$tmp_dir/cco" "$INSTALL_DIR/cco" || return 1
  chmod +x "$INSTALL_DIR/cco" || return 1
}

if ! command -v cco >/dev/null 2>&1; then
  try_install_cco || true
fi

export PATH="$HOME/.local/bin:$PATH"

if command -v cco >/dev/null 2>&1; then
  exec cco hook session-start "$MODE"
else
  echo '{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"cco could not be installed this session; orchestration is running in prompt-only mode with no specialized agents or memory."}}'
fi
