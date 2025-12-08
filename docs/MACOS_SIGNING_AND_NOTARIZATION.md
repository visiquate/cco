# macOS Code Signing and Notarization for CCO

## Overview

This document provides a comprehensive design for adding macOS code signing and Apple notarization to CCO's GitHub Actions release workflow. The solution leverages a self-hosted macOS ARM runner with pre-installed certificates in the system keychain.

## Architecture Decision

### Certificate Storage Approach: **Keychain-Based (Recommended)**

Given the self-hosted macOS runner, we use OS keychain storage instead of GitHub secrets:

**Why Keychain-Based:**
- ✅ Prevents certificate exposure in GitHub logs
- ✅ Self-hosted runner controls certificate lifecycle
- ✅ Better integration with `codesign` tool
- ✅ Simplifies multi-certificate management
- ✅ Runner admin has physical control

**Why NOT GitHub Secrets:**
- ❌ Base64-encoded certs visible in logs
- ❌ Complex export/import during workflow
- ❌ Difficult to manage multiple certificates
- ❌ Less secure for code signing

### Prerequisites for Self-Hosted Runner

The `mac-mini-arm64` self-hosted runner requires one-time setup:

```bash
# 1. Install Apple Developer certificate (from Apple Developer Program)
#    - Login to developer.apple.com
#    - Download Developer ID Application certificate (.cer)
#    - Download Developer ID Application key (.p8)
#    - Import to Keychain:
open ~/Downloads/DeveloperIDApplication.cer

# 2. Create a local user for the runner
# The runner service should run as this user to access their keychain

# 3. Verify certificate is importable in Xcode
security find-identity -v -p codesigning

# Output should show:
# 1) ABC123DEF... "Developer ID Application: Your Name (TEAM_ID)"

# 4. Generate App-Specific Password for notarization
# https://appleid.apple.com/account/manage/security
# Store in runner's keychain:
security add-generic-password -a "YOUR_APPLE_ID" \
  -s "notarytool-password" -w "GENERATED_PASSWORD" \
  -A
```

## Implementation Guide

### 1. Workflow Changes

**File:** `.github/workflows/release.yml`

Add these steps to the `build` job after the binary is created (after "Upload build artifact"):

```yaml
jobs:
  build:
    # ... existing steps ...

    - name: Sign binary (macOS ARM)
      if: matrix.target == 'aarch64-apple-darwin' && runner.os == 'macOS'
      env:
        CODESIGN_IDENTITY: ${{ secrets.MACOS_CODESIGN_IDENTITY }}
        APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
      run: |
        # Find the developer identity
        # Fallback to first available if not specified
        IDENTITY="${CODESIGN_IDENTITY:-Developer ID Application}"

        BINARY_PATH="artifacts/cco"

        echo "Signing binary with identity: $IDENTITY"

        # Sign the binary without entitlements (CLI tool doesn't need special entitlements)
        codesign --sign "$IDENTITY" \
          --timestamp \
          --options runtime \
          --force \
          "$BINARY_PATH"

        # Verify signature
        codesign --verify --verbose "$BINARY_PATH"
        echo "✓ Binary signed successfully"

        # Extract code signature info
        spctl -a -vvv "$BINARY_PATH"

    - name: Notarize binary (macOS ARM)
      if: matrix.target == 'aarch64-apple-darwin' && runner.os == 'macOS'
      env:
        APPLE_ID: ${{ secrets.APPLE_ID }}
        APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
      run: |
        BINARY_PATH="artifacts/cco"
        ARTIFACT_NAME="${{ matrix.artifact_name }}.tar.gz"
        ARTIFACT_PATH="artifacts/$ARTIFACT_NAME"

        # Extract binary for notarization (notarytool works best with executables, not archives)
        mkdir -p notary_temp
        cd notary_temp
        tar -xzf "../$ARTIFACT_PATH"

        echo "Notarizing binary..."

        # Submit for notarization
        # Password is stored in keychain: security add-generic-password -a "APPLE_ID" -s "notarytool-password"
        xcrun notarytool submit "cco" \
          --apple-id "$APPLE_ID" \
          --team-id "$APPLE_TEAM_ID" \
          --password-keychain "notarytool-password" \
          --wait \
          --timeout 600

        if [ $? -ne 0 ]; then
          echo "❌ Notarization failed"
          exit 1
        fi

        echo "✓ Notarization successful"

        # Re-create tarball with notarized binary
        cd ..
        rm "$ARTIFACT_PATH"
        cd notary_temp
        tar -czvf "../$ARTIFACT_PATH" cco
        cd ..

        # Verify staple status
        xcrun stapler validate "$ARTIFACT_PATH"

    - name: Verify signed and notarized binary
      if: matrix.target == 'aarch64-apple-darwin' && runner.os == 'macOS'
      run: |
        BINARY_PATH="artifacts/cco"

        echo "=== Signature Verification ==="
        codesign -dvv "$BINARY_PATH" || true

        echo "=== Security Policy Check ==="
        spctl -a -vvv "$BINARY_PATH" || true

        echo "=== Entitlements Check ==="
        codesign --display --entitlements - "$BINARY_PATH" || true

        # Final verification - this simulates Gatekeeper
        echo "=== Gatekeeper Check ==="
        xattr -l "$BINARY_PATH" | grep -i "quarantine" || echo "No quarantine attribute (good!)"

        # Check for valid signature
        if codesign --verify "$BINARY_PATH"; then
          echo "✓ Signature is valid"
        else
          echo "❌ Signature verification failed"
          exit 1
        fi
```

### 2. GitHub Secrets Configuration

Add these secrets to the repository (Settings → Secrets and variables → Actions):

| Secret Name | Value | Notes |
|-------------|-------|-------|
| `MACOS_CODESIGN_IDENTITY` | "Developer ID Application: Your Name (TEAM_ID)" | Full identity string |
| `APPLE_ID` | your.email@example.com | Apple ID for notarization |
| `APPLE_TEAM_ID` | ABC1234567 | 10-character team ID from Apple Developer |

**Note:** The actual certificate and notarization password are stored in the runner's keychain, NOT in GitHub secrets.

### 3. Self-Hosted Runner Setup Script

**File:** `scripts/setup-macos-signing.sh`

```bash
#!/bin/bash
set -euo pipefail

# Setup script for macOS code signing and notarization on self-hosted runner
# Run this once on the mac-mini-arm64 runner

echo "=== macOS Signing & Notarization Setup ==="

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script must run on macOS"
    exit 1
fi

# Check if running as runner user (not root)
if [[ $(id -u) -eq 0 ]]; then
    echo "⚠️  Running as root - certificates will be in root's keychain"
    echo "    Recommend running as the GitHub Actions runner user instead"
fi

RUNNER_USER=$(whoami)
echo "Setting up for user: $RUNNER_USER"

# 1. Import Developer ID Application certificate
echo ""
echo "Step 1: Import Developer ID Application certificate"
echo "-------"
echo "1. Download certificate from developer.apple.com"
echo "2. Save to ~/Downloads/DeveloperIDApplication.cer"
echo ""

if [ -f "$HOME/Downloads/DeveloperIDApplication.cer" ]; then
    echo "Found certificate, importing..."
    open "$HOME/Downloads/DeveloperIDApplication.cer"
    echo "Complete the import in Keychain Access and press Enter..."
    read -p "Press Enter when done: "
else
    echo "⚠️  Certificate not found at ~/Downloads/DeveloperIDApplication.cer"
    echo "    Please download it from developer.apple.com first"
fi

# 2. Verify certificate
echo ""
echo "Step 2: Verify certificate installation"
echo "-------"
IDENTITIES=$(security find-identity -v -p codesigning)
echo "$IDENTITIES"

# Ensure at least one Developer ID found
if echo "$IDENTITIES" | grep -q "Developer ID"; then
    echo "✓ Developer ID Application certificate found"
else
    echo "❌ Developer ID Application certificate NOT found"
    echo "   Please import the certificate in Keychain Access"
    exit 1
fi

# 3. Setup notarization password
echo ""
echo "Step 3: Setup notarization credentials"
echo "-------"
echo "1. Go to https://appleid.apple.com/account/manage/security"
echo "2. Generate an App-Specific Password"
echo "3. Enter your Apple ID email:"
read -p "Apple ID: " APPLE_ID

read -s -p "Paste the app-specific password (hidden): " APP_PASSWORD
echo ""

# Store in keychain
security add-generic-password \
    -a "$APPLE_ID" \
    -s "notarytool-password" \
    -w "$APP_PASSWORD" \
    -T /usr/bin/security \
    -T /usr/bin/xcrun \
    -A 2>/dev/null || echo "Password entry already exists, skipping"

echo "✓ Credentials stored in keychain"

# 4. Test notarytool
echo ""
echo "Step 4: Test notarytool configuration"
echo "-------"

# Create minimal test binary
TEST_BINARY=$(mktemp)
echo "#!/bin/bash" > "$TEST_BINARY"
chmod +x "$TEST_BINARY"

# Try to authenticate with notarytool
if xcrun notarytool validate-credentials \
    --apple-id "$APPLE_ID" \
    --team-id "test" \
    --password-keychain "notarytool-password" 2>&1 | grep -q "credentials"; then
    echo "✓ notarytool credentials work"
else
    echo "⚠️  Could not verify notarytool credentials"
    echo "    This will be tested during workflow execution"
fi

rm -f "$TEST_BINARY"

# 5. Final summary
echo ""
echo "=== Setup Complete ==="
echo ""
echo "Keychain contents:"
security find-identity -v -p codesigning | head -5
echo ""
echo "Next steps:"
echo "1. Add these secrets to GitHub repository (Settings → Secrets):"
echo "   - MACOS_CODESIGN_IDENTITY: (identity string from above)"
echo "   - APPLE_ID: $APPLE_ID"
echo "   - APPLE_TEAM_ID: (your 10-digit team ID)"
echo ""
echo "2. The notarytool password is stored in this user's keychain"
echo "3. Ensure GitHub Actions runner runs as: $RUNNER_USER"
echo ""
echo "For troubleshooting, see: docs/MACOS_SIGNING_TROUBLESHOOTING.md"
```

### 4. Error Handling and Verification

The workflow includes comprehensive error handling:

#### Codesign Failures
- **Cause:** Certificate not in keychain or identity mismatch
- **Solution:** Verify certificate exists: `security find-identity -v -p codesigning`

#### Notarization Failures
- **Cause:** Apple credentials invalid or network issues
- **Solution:** Verify credentials: `xcrun notarytool validate-credentials`

#### Gatekeeper Issues
- **Cause:** Unsigned or improperly signed binary
- **Solution:** Verify with `codesign --verify --verbose`

### 5. Homebrew Formula Updates

**File:** `Formula/cco.rb` (in homebrew-cco repository)

Update the formula to document the signing/notarization:

```ruby
class Cco < Formula
  desc "Claude Code Orchestrator - AI-powered development automation"
  homepage "https://github.com/visiquate/cco"
  version "2025.12.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-aarch64-apple-darwin.tar.gz"
      sha256 "XXXX..."
    end
    on_intel do
      # If/when x86_64 macOS support is added
      url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-x86_64-apple-darwin.tar.gz"
      sha256 "YYYY..."
    end
  end
  on_linux do
    url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "ZZZZ..."
  end

  def install
    bin.install "cco"
  end

  # Verify the binary is properly signed and notarized
  def post_install
    # On macOS, verify the signature
    if OS.mac?
      system "codesign", "--verify", "--verbose=2", "#{bin}/cco"
    end
  end

  test do
    assert_match "usage", shell_output("#{bin}/cco --help")
  end
end
```

## Verification Checklist

Before releasing:

- [ ] Certificate is importable in Keychain Access
- [ ] `security find-identity -v -p codesigning` shows Developer ID
- [ ] `xcrun notarytool validate-credentials` passes
- [ ] Workflow includes all signing/notarization steps
- [ ] GitHub secrets are configured (identity, Apple ID, team ID)
- [ ] Test run completes with successful notarization
- [ ] Binary passes `codesign --verify`
- [ ] Binary passes `spctl` Gatekeeper check
- [ ] Downloaded binary doesn't show Gatekeeper warning

## Troubleshooting

### Certificate Not Found
```bash
# Import certificate
open ~/Downloads/DeveloperIDApplication.cer

# Verify import
security find-identity -v -p codesigning
```

### Notarization Credentials Invalid
```bash
# Update keychain password
security delete-generic-password -a "APPLE_ID" -s "notarytool-password"
security add-generic-password -a "APPLE_ID" -s "notarytool-password" -w "NEW_PASSWORD"

# Test
xcrun notarytool validate-credentials \
  --apple-id "EMAIL" \
  --team-id "TEAM_ID" \
  --password-keychain "notarytool-password"
```

### Gatekeeper Still Blocking
```bash
# Verify signature validity
codesign -dvv /path/to/cco

# Check for quarantine attribute
xattr -l /path/to/cco

# Remove quarantine if present
xattr -d com.apple.quarantine /path/to/cco
```

## Security Considerations

1. **Certificate Lifecycle**
   - Certificates expire yearly - renew before expiration
   - Monitor Apple Developer account for renewal dates
   - Set calendar reminders 30 days before expiration

2. **Keychain Access**
   - Only the runner user can access the keychain
   - GitHub Actions runner runs with correct user permissions
   - Consider disk encryption for the runner machine

3. **Credentials Management**
   - App-specific passwords are safer than full Apple ID password
   - Rotate credentials yearly
   - Do not share credentials between runners

4. **Audit Trail**
   - All notarizations are logged in Apple Developer account
   - Review notarization history for anomalies
   - Gatekeeper validates signatures at runtime

## macOS Security Policy Summary

When a user downloads the signed/notarized binary from GitHub:

1. **Download**: `com.apple.quarantine` extended attribute is set
2. **First Run**: Gatekeeper checks:
   - Is the binary signed? ✓ (yes)
   - Is the signature valid? ✓ (yes)
   - Is it notarized by Apple? ✓ (yes - from notarization ticket)
   - Has malware been reported? ✓ (no - passes Apple scan)
3. **Result**: No warning, binary runs normally
4. **Subsequent Runs**: Uses cached notarization ticket (faster)

## References

- [Apple Code Signing Guide](https://developer.apple.com/documentation/security/code-signing-guide)
- [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [xcrun notarytool Documentation](https://developer.apple.com/documentation/technotes/tn3147-migrating-to-the-latest-notarization-experience)
- [Gatekeeper Overview](https://support.apple.com/en-us/HT202491)
