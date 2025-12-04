#!/bin/bash
# Sign CCO binary with JIT entitlements for macOS
# Usage: ./sign-cco.sh [path-to-cco]
#
# This script is only needed if the binary crashes with "Code Signature Invalid"
# on a fresh Mac. It embeds the required entitlements for ML inference.

set -e

CCO_PATH="${1:-$(which cco 2>/dev/null || echo "$HOME/.local/bin/cco")}"

if [ ! -f "$CCO_PATH" ]; then
    echo "Error: CCO binary not found at $CCO_PATH"
    echo "Usage: $0 [path-to-cco]"
    exit 1
fi

echo "Signing CCO binary with JIT entitlements..."
echo "Binary: $CCO_PATH"

# Create entitlements in temp file
ENTITLEMENTS=$(mktemp)
cat > "$ENTITLEMENTS" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
EOF

# Sign the binary
codesign --force --sign - --entitlements "$ENTITLEMENTS" --options runtime "$CCO_PATH"

# Clean up
rm -f "$ENTITLEMENTS"

# Clear quarantine
xattr -c "$CCO_PATH" 2>/dev/null || true

echo "Done! Verify with: $CCO_PATH --version"
