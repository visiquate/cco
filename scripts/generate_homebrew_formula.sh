#!/usr/bin/env bash
# Generate a Homebrew formula for CCO using the latest GitHub Release.
# Usage:
#   ./scripts/generate_homebrew_formula.sh            # prints to stdout
#   OUTPUT=Formula/cco.rb ./scripts/generate_homebrew_formula.sh > Formula/cco.rb
# Optional: set GITHUB_TOKEN to raise GitHub API limits.
# Stable-only: uses the latest non-draft/non-prerelease release.

set -euo pipefail

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "[ERROR] Missing required command: $1" >&2; exit 1; }
}

require_cmd curl
require_cmd python3

API_URL="https://api.github.com/repos/visiquate/cco/releases/latest"
AUTH=()
if [ -n "${GITHUB_TOKEN:-}" ]; then
  AUTH=(-H "Authorization: Bearer $GITHUB_TOKEN")
fi

RELEASE_JSON=$(curl -fsSL -H "Accept: application/vnd.github+json" "${AUTH[@]:+${AUTH[@]}}" "$API_URL")

FORMULA=$(RELEASE_JSON="$RELEASE_JSON" python3 - <<'PY'
import json, os, sys

def fail(msg):
    print(f"[ERROR] {msg}", file=sys.stderr)
    sys.exit(1)

try:
    data = json.loads(os.environ.get("RELEASE_JSON", ""))
except Exception:
    fail("Failed to parse release JSON")

version_tag = data.get("tag_name") or ""
if not version_tag.startswith("v"):
    fail("Unexpected tag format; expected leading 'v'")
version = version_tag[1:]
assets = {a.get("name"): a for a in data.get("assets", [])}

required_assets = [
    "cco-aarch64-apple-darwin.tar.gz",
    "cco-x86_64-unknown-linux-gnu.tar.gz",
    "checksums.txt",
]
for name in required_assets:
    if name not in assets:
        fail(f"Missing required asset: {name}")

checksums_url = assets["checksums.txt"].get("browser_download_url")
if not checksums_url:
    fail("checksums.txt missing download URL")

import urllib.request
try:
    with urllib.request.urlopen(checksums_url) as resp:
        content = resp.read().decode()
except Exception as e:
    fail(f"Failed to download checksums.txt: {e}")

checksums = {}
for line in content.splitlines():
    parts = line.split()
    if len(parts) >= 2:
        checksums[parts[1]] = parts[0]

for name in required_assets[:-1]:
    if name not in checksums:
        fail(f"Checksum missing for {name}")

sha_mac_arm = checksums["cco-aarch64-apple-darwin.tar.gz"]
sha_linux_x86 = checksums["cco-x86_64-unknown-linux-gnu.tar.gz"]

formula = f'''class Cco < Formula
  desc "Claude Code Orchestrator - AI-powered development automation platform"
  homepage "https://github.com/visiquate/cco"
  license "MIT"
  version "{version}"

  on_macos do
    on_arm do
      url "https://github.com/visiquate/cco/releases/download/v#{{version}}/cco-aarch64-apple-darwin.tar.gz"
      sha256 "{sha_mac_arm}"
    end
    on_intel do
      odie "x86_64 macOS artifact not published for v#{{version}}"
    end
  end

  on_linux do
    on_arm do
      odie "Linux aarch64 artifact not published for v#{{version}}"
    end
    on_intel do
      url "https://github.com/visiquate/cco/releases/download/v#{{version}}/cco-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "{sha_linux_x86}"
    end
  end

  def install
    bin.install "cco"
  end

  test do
    assert_match("cco", shell_output("#{{bin}}/cco --version"))
  end
end
'''

print(formula)
PY
)

echo "$FORMULA"
