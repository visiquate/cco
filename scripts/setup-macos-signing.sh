#!/bin/bash
set -euo pipefail

# Setup script for macOS code signing and notarization on self-hosted runner
# This script must be run once on the mac-mini-arm64 runner to configure code signing
#
# Usage:
#   ./scripts/setup-macos-signing.sh
#
# Requirements:
#   - macOS system with Xcode installed
#   - Apple Developer Program membership
#   - Developer ID Application certificate from Apple Developer Program

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  macOS Code Signing & Notarization Setup for GitHub Actions   ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Verify macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ Error: This script must run on macOS"
    exit 1
fi

# Verify Xcode
if ! xcode-select -p &>/dev/null; then
    echo "❌ Error: Xcode Command Line Tools not installed"
    echo ""
    echo "Install with:"
    echo "  xcode-select --install"
    exit 1
fi

RUNNER_USER=$(whoami)
echo "ℹ️  Running as user: $RUNNER_USER"

if [[ $(id -u) -eq 0 ]]; then
    echo "⚠️  Running as root - certificates will be in root's keychain"
    echo "   Recommend running as the GitHub Actions runner user instead"
    echo ""
    read -p "Continue as root? (y/n): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 1: Certificate import
echo ""
echo "Step 1: Developer ID Application Certificate"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This script expects you to have downloaded a Developer ID Application"
echo "certificate from the Apple Developer Program."
echo ""
echo "Steps to obtain the certificate:"
echo "  1. Go to https://developer.apple.com/account"
echo "  2. Navigate to Certificates, Identifiers & Profiles"
echo "  3. Click Certificates"
echo "  4. Create or download your Developer ID Application certificate"
echo "  5. Save the .cer file and the private key (.p8 if using new format)"
echo ""

CERT_PATH=""
while true; do
    read -p "Enter path to Developer ID certificate (.cer file): " -r CERT_PATH
    CERT_PATH="${CERT_PATH/#\~/$HOME}"  # Expand ~

    if [ -f "$CERT_PATH" ]; then
        break
    else
        echo "❌ File not found: $CERT_PATH"
    fi
done

echo ""
echo "Importing certificate into Keychain..."
open "$CERT_PATH"
echo ""
echo "📋 Keychain Access opened. Double-click the certificate to import it."
echo "   Make sure to import it to the 'login' keychain, not 'System'"
echo ""
read -p "Press Enter once the certificate has been imported: "

# Step 2: Verify certificate
echo ""
echo "Step 2: Verify Certificate Installation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

IDENTITIES=$(security find-identity -v -p codesigning 2>/dev/null || echo "")

if [ -z "$IDENTITIES" ]; then
    echo "❌ No code signing identities found!"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Verify the certificate was imported to the 'login' keychain"
    echo "  2. Check Keychain Access for 'Developer ID Application' entry"
    echo "  3. The certificate might need to be re-imported"
    exit 1
fi

echo "Found code signing identities:"
echo "$IDENTITIES"
echo ""

# Extract identity
IDENTITY=$(echo "$IDENTITIES" | grep "Developer ID Application" | head -1 | sed 's/^[[:space:]]*[0-9]*)[[:space:]]*//')

if [ -z "$IDENTITY" ]; then
    echo "❌ Developer ID Application certificate not found!"
    echo ""
    echo "The imported certificate doesn't appear to be a valid Developer ID"
    echo "Application certificate. Please verify the certificate type and try again."
    exit 1
fi

echo "✓ Found identity: $IDENTITY"
echo ""

# Step 3: Notarization credentials
echo ""
echo "Step 3: Notarization Credentials"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To notarize binaries, you need:"
echo "  1. Apple ID (with Developer Program access)"
echo "  2. Team ID (10-character code from Apple Developer Program)"
echo "  3. App-Specific Password (generated for security)"
echo ""
echo "For the App-Specific Password:"
echo "  1. Go to https://appleid.apple.com/account/manage/security"
echo "  2. Sign in with your Apple ID"
echo "  3. Under 'APP-SPECIFIC PASSWORDS', click 'Generate'"
echo "  4. Select 'macOS'  and 'Developer Tools'"
echo "  5. Copy the generated 16-character password"
echo ""

read -p "Enter your Apple ID email: " APPLE_ID
if [ -z "$APPLE_ID" ]; then
    echo "❌ Apple ID is required"
    exit 1
fi

read -p "Enter your Team ID (10 characters): " TEAM_ID
if [ -z "$TEAM_ID" ] || [ ${#TEAM_ID} -ne 10 ]; then
    echo "❌ Team ID must be exactly 10 characters"
    exit 1
fi

echo ""
echo "Now enter your app-specific password (it will be hidden)"
echo "This will be stored in the Keychain, not displayed anywhere"
echo ""
read -s -p "Paste app-specific password: " APP_PASSWORD
echo ""

if [ -z "$APP_PASSWORD" ]; then
    echo "❌ Password is required"
    exit 1
fi

# Store in keychain
echo "Storing credentials in Keychain..."
security add-generic-password \
    -a "$APPLE_ID" \
    -s "notarytool-password" \
    -w "$APP_PASSWORD" \
    -T /usr/bin/security \
    -T /usr/bin/xcrun \
    -A 2>/dev/null && echo "✓ Credentials stored" || echo "⚠️  Credentials already in keychain (updated)"

# Step 4: Test credentials
echo ""
echo "Step 4: Verify Notarization Credentials"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing notarytool access..."
if xcrun notarytool validate-credentials \
    --apple-id "$APPLE_ID" \
    --team-id "$TEAM_ID" \
    --password-keychain "notarytool-password" &>/dev/null; then
    echo "✓ notarytool credentials validated successfully"
else
    echo "⚠️  Could not validate credentials at this time"
    echo "    This might be normal - validation will be tested during workflow execution"
fi

# Step 5: Summary
echo ""
echo "Step 5: Setup Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Configuration complete! Here's what was set up:"
echo ""
echo "Signing Identity:"
echo "  $IDENTITY"
echo ""
echo "Notarization Credentials:"
echo "  Apple ID: $APPLE_ID"
echo "  Team ID: $TEAM_ID"
echo "  Password: Stored in Keychain (hidden)"
echo ""

# Step 6: GitHub secrets
echo ""
echo "Step 6: GitHub Repository Secrets"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Now add these secrets to your GitHub repository:"
echo "  Settings → Secrets and variables → Actions"
echo ""
echo "Required secrets:"
echo ""
echo "  1. MACOS_CODESIGN_IDENTITY"
echo "     Value: $IDENTITY"
echo ""
echo "  2. APPLE_ID"
echo "     Value: $APPLE_ID"
echo ""
echo "  3. APPLE_TEAM_ID"
echo "     Value: $TEAM_ID"
echo ""
echo "Note: The app-specific password is stored in this machine's Keychain"
echo "      and does NOT need to be added to GitHub secrets."
echo ""

# Step 7: Verify signing works
echo ""
echo "Step 7: Test Code Signing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create a simple test binary
TEST_DIR=$(mktemp -d)
trap "rm -rf '$TEST_DIR'" EXIT

echo "#!/bin/bash" > "$TEST_DIR/test_binary"
chmod +x "$TEST_DIR/test_binary"

echo "Attempting to sign test binary..."
if codesign --sign "$IDENTITY" \
    --timestamp \
    --options runtime \
    --force \
    "$TEST_DIR/test_binary" 2>/dev/null; then
    echo "✓ Successfully signed test binary"

    # Verify
    if codesign --verify "$TEST_DIR/test_binary" 2>/dev/null; then
        echo "✓ Signature verification successful"
    else
        echo "❌ Signature verification failed"
    fi
else
    echo "❌ Failed to sign test binary"
    echo "   The identity might be incorrect or missing private key"
fi

# Step 8: Final instructions
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    SETUP COMPLETE!                            ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo ""
echo "1. Add the GitHub secrets listed above to your repository"
echo ""
echo "2. Verify the runner configuration:"
echo "   - Ensure GitHub Actions runner runs as: $RUNNER_USER"
echo "   - Check: /Library/LaunchDaemons (or ~/Library/LaunchAgents for user)"
echo ""
echo "3. Test the workflow:"
echo "   - Create a git tag: git tag v2025.12.1-test"
echo "   - Push the tag: git push origin v2025.12.1-test"
echo "   - Monitor the release workflow"
echo ""
echo "4. Verify the release:"
echo "   - Download the macOS binary"
echo "   - Verify: codesign --verify ~/Downloads/cco"
echo "   - Verify: spctl -a -vvv ~/Downloads/cco"
echo ""
echo "For troubleshooting:"
echo "  - See: docs/MACOS_SIGNING_TROUBLESHOOTING.md"
echo "  - Or check: docs/MACOS_SIGNING_AND_NOTARIZATION.md"
echo ""
echo "Manual verification commands:"
echo ""
echo "  # List code signing identities"
echo "  security find-identity -v -p codesigning"
echo ""
echo "  # Check credentials"
echo "  xcrun notarytool validate-credentials \\"
echo "    --apple-id '$APPLE_ID' \\"
echo "    --team-id '$TEAM_ID' \\"
echo "    --password-keychain 'notarytool-password'"
echo ""
echo "  # Sign a test binary"
echo "  codesign --sign \"$IDENTITY\" \\\"
echo "    --timestamp --options runtime /path/to/binary"
echo ""
