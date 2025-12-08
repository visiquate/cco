# macOS Code Signing and Notarization Research

**Date**: 2025-12-06
**Context**: CCO binary (Rust CLI tool) killed by macOS Gatekeeper after Homebrew installation
**Goal**: Add code signing to GitHub Actions release workflow

---

## Executive Summary

macOS Gatekeeper is blocking CCO binaries because they lack valid code signatures from an Apple Developer ID certificate. To distribute CLI binaries outside the App Store and avoid Gatekeeper issues, you need:

1. **Code Signing**: Sign binaries with a Developer ID Application certificate
2. **Notarization**: Submit to Apple's notary service for malware scanning
3. **Stapling** (optional for CLI): Attach notarization ticket to the binary/package

**Key Finding**: While major Rust projects like ripgrep and bat do NOT implement code signing, they still work because Homebrew doesn't add quarantine flags. However, direct downloads or other distribution methods WILL trigger Gatekeeper.

**Recommended Solution**: Implement basic code signing with ad-hoc signature as short-term fix, full Developer ID signing as long-term solution.

---

## 1. macOS Code Signing Requirements

### Certificate Types

Apple offers two main certificate types for distribution outside the App Store:

| Certificate Type | Purpose | Cost |
|-----------------|---------|------|
| **Developer ID Application** | Sign standalone applications | $99/year Apple Developer Program |
| **Developer ID Installer** | Sign .pkg installer packages | $99/year Apple Developer Program |

For CLI binaries like CCO, you need a **Developer ID Application** certificate.

### Getting a Developer ID Certificate

**Method 1: Via Xcode (Easiest)**
1. Install Xcode and sign in with your Apple Developer account
2. Go to Preferences → Account → Apple ID → Manage Certificates
3. Click "+" → "Developer ID Application"
4. Export certificate as .p12 file for CI/CD use

**Method 2: Via Web (Apple Developer Portal)**
1. Log in to [developer.apple.com](https://developer.apple.com)
2. Navigate to Certificates, Identifiers & Profiles
3. Create a new "Developer ID Application" certificate
4. Download the .cer file and import to Keychain
5. Export as .p12 for CI/CD

**Source**: [Signing Mac Software with Developer ID](https://developer.apple.com/developer-id/), [How to get certificate, code-sign & notarize macOS binaries](https://dennisbabkin.com/blog/?t=how-to-get-certificate-code-sign-notarize-macos-binaries-outside-apple-app-store)

---

## 2. The Notarization Process

### What is Notarization?

Notarization is Apple's automated malware scanning service. It verifies:
- Valid code signature
- Hardened runtime enabled
- No known malicious content

**Important**: Starting November 1, 2023, Apple deprecated the `altool` utility. You MUST use `xcrun notarytool` for all new implementations.

**Source**: [Code Signing and Notarization on macOS](https://www.msweet.org/blog/2020-12-10-macos-notarization.html)

### Notarization Workflow

```bash
# 1. Code sign the binary
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
         --options runtime \
         --timestamp \
         /path/to/cco

# 2. Create a ZIP for notarization (required for CLI binaries)
ditto -c -k --keepParent /path/to/cco cco.zip

# 3. Submit for notarization
xcrun notarytool submit cco.zip \
    --apple-id "your@email.com" \
    --password "app-specific-password" \
    --team-id "TEAM_ID" \
    --wait

# 4. Staple the notarization ticket (optional for CLI, required for .app/.pkg/.dmg)
# Note: Stapling is NOT supported for standalone binaries
# xcrun stapler staple /path/to/cco  # This will fail for CLI binaries
```

### Stapling Limitation for CLI Binaries

**CRITICAL**: You CANNOT staple notarization tickets to standalone CLI binaries. According to Apple's documentation, stapling only works for:
- .app bundles
- .pkg installer packages
- .dmg disk images
- Kernel extensions

For CLI binaries distributed as raw executables, Apple will verify notarization online on first run. This requires an internet connection but is transparent to the user once notarized.

**Source**: [macOS distribution — code signing, notarization, quarantine](https://gist.github.com/rsms/929c9c2fec231f0cf843a1a746a416f5), [Code Signing and Notarization on macOS](https://www.msweet.org/blog/2020-12-10-macos-notarization.html)

---

## 3. How Other Rust Projects Handle This

### ripgrep

**Finding**: ripgrep does NOT implement code signing in their GitHub Actions workflow.

Their macOS build process:
- Compiles with LTO optimization
- Strips binaries with `strip`
- Creates tar.gz archives
- Generates SHA256 checksums

**No signing steps present**. Users who download directly from GitHub releases may encounter Gatekeeper warnings.

**Source**: [ripgrep release workflow](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml)

### bat

**Finding**: bat also does NOT implement code signing.

Their workflow:
- Uses `dtolnay/rust-toolchain@stable`
- Compiles with `cargo build --locked --release`
- Packages binaries into tarballs
- No signing or notarization steps

**Source**: [bat CICD workflow](https://github.com/sharkdp/bat/blob/master/.github/workflows/CICD.yml)

### Why Don't They Sign?

Both ripgrep and bat work fine when installed via Homebrew because:
1. **Homebrew doesn't set quarantine flags** on installed binaries
2. Gatekeeper primarily focuses on quarantined programs (downloads from browsers, email attachments)
3. The cost/complexity of maintaining Apple Developer accounts may not justify the benefit

However, users who download binaries directly from GitHub Releases WILL encounter Gatekeeper issues.

**Source**: [My struggles with Gatekeeper errors and Homebrew](https://thefridaydeploy.substack.com/p/my-struggles-with-gatekeeper-errors)

### Projects That DO Sign: taiki-e/upload-rust-binary-action

The `upload-rust-binary-action` GitHub Action provides built-in macOS code signing support:

**Code Signing Inputs**:
- `codesign`: Sign using `codesign` on macOS (e.g., `"-"` for ad-hoc, or a certificate identity)
- `codesign-prefix`: Prefix for the codesign identifier (e.g., `"com.visiquate."`)
- `codesign-options`: Additional flags like `"runtime"` for hardened runtime

**Example Usage**:
```yaml
- uses: taiki-e/upload-rust-binary-action@v1
  with:
    bin: cco
    target: aarch64-apple-darwin
    codesign: "7FP48PW9TN"  # Team ID or certificate identity
    codesign-prefix: "com.visiquate."
    codesign-options: "runtime"
    token: ${{ secrets.GITHUB_TOKEN }}
```

**Note**: This action handles code signing but does NOT handle notarization. You would need additional steps for full notarization.

**Source**: [upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action), [Random Errata: Notarizing CLI apps](https://www.randomerrata.com/articles/2024/notarize/)

---

## 4. Required GitHub Secrets

To implement code signing and notarization in GitHub Actions, you need:

| Secret Name | Description | How to Obtain |
|------------|-------------|---------------|
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded .p12 certificate | Export from Keychain, encode with `base64 -i certificate.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 certificate | Set when exporting from Keychain |
| `APPLE_TEAM_ID` | 10-character team identifier | Found in Apple Developer account |
| `APPLE_ID` | Apple ID email address | Your Apple Developer email |
| `APPLE_ID_PASSWORD` | App-specific password | Generate at appleid.apple.com |
| `KEYCHAIN_PASSWORD` | Temporary keychain password | Generate a random password for CI |

**Alternative Authentication**: Instead of Apple ID + password, you can use App Store Connect API keys for notarization:

| Secret Name | Description |
|------------|-------------|
| `APP_STORE_CONNECT_KEY_ID` | API Key ID |
| `APP_STORE_CONNECT_ISSUER_ID` | Issuer ID |
| `APP_STORE_CONNECT_KEY_BASE64` | Base64-encoded .p8 key file |

**Source**: [Automatic Code-signing and Notarization for macOS apps using GitHub Actions](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/)

---

## 5. GitHub Actions Workflow Implementation

### Approach 1: Full Signing + Notarization (Recommended for Production)

**Prerequisites**:
- Apple Developer Program membership ($99/year)
- Developer ID Application certificate
- App-specific password or API key

**Workflow Steps**:

```yaml
jobs:
  build-macos-signed:
    name: Build and Sign macOS Binary
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
          target: aarch64-apple-darwin

      - name: Install dependencies
        run: |
          brew install protobuf
          sudo xcode-select -s /Applications/Xcode.app/Contents/Developer

      - name: Build binary
        run: |
          cargo build --release --target aarch64-apple-darwin
          cp target/aarch64-apple-darwin/release/cco ./cco

      # Import certificate into temporary keychain
      - name: Import certificate
        env:
          CERTIFICATE_BASE64: ${{ secrets.APPLE_CERTIFICATE_BASE64 }}
          CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
        run: |
          # Create temporary keychain
          KEYCHAIN_PATH=$RUNNER_TEMP/app-signing.keychain-db
          security create-keychain -p "$KEYCHAIN_PASSWORD" $KEYCHAIN_PATH
          security set-keychain-settings -lut 21600 $KEYCHAIN_PATH
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" $KEYCHAIN_PATH

          # Import certificate
          echo -n "$CERTIFICATE_BASE64" | base64 --decode -o certificate.p12
          security import certificate.p12 \
            -P "$CERTIFICATE_PASSWORD" \
            -A -t cert -f pkcs12 \
            -k $KEYCHAIN_PATH
          security list-keychain -d user -s $KEYCHAIN_PATH

      # Sign the binary
      - name: Sign binary
        env:
          TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          codesign --sign "Developer ID Application: Your Name ($TEAM_ID)" \
            --options runtime \
            --timestamp \
            --identifier "com.visiquate.cco" \
            --verbose \
            ./cco

          # Verify signature
          codesign --verify --verbose ./cco
          codesign --display --verbose=4 ./cco

      # Package for notarization
      - name: Create ZIP for notarization
        run: |
          ditto -c -k --keepParent ./cco cco.zip

      # Submit for notarization
      - name: Notarize binary
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_ID_PASSWORD: ${{ secrets.APPLE_ID_PASSWORD }}
          TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          # Store credentials in keychain profile
          xcrun notarytool store-credentials "notarytool-profile" \
            --apple-id "$APPLE_ID" \
            --team-id "$TEAM_ID" \
            --password "$APPLE_ID_PASSWORD"

          # Submit for notarization
          xcrun notarytool submit cco.zip \
            --keychain-profile "notarytool-profile" \
            --wait

          # Check notarization status
          xcrun notarytool log \
            --keychain-profile "notarytool-profile" \
            $(xcrun notarytool history --keychain-profile "notarytool-profile" | head -1)

      # Create final tarball
      - name: Create release tarball
        run: |
          tar -czvf cco-aarch64-apple-darwin.tar.gz cco
          shasum -a 256 cco-aarch64-apple-darwin.tar.gz > cco-aarch64-apple-darwin.sha256

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: cco-aarch64-apple-darwin
          path: |
            cco-aarch64-apple-darwin.tar.gz
            cco-aarch64-apple-darwin.sha256
```

**Source**: [Automatic Code-signing and Notarization for macOS apps using GitHub Actions](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/), [How to sign and notarize a PKG within a Github Actions macos runner](https://stackoverflow.com/questions/70991268/how-to-sign-and-notarize-a-pkg-within-a-github-actions-macos-runner)

### Approach 2: Ad-hoc Signing (Quick Fix, No Apple Developer Account)

Ad-hoc signing creates a local signature without a trusted certificate. It won't prevent Gatekeeper warnings but may resolve some SIGKILL issues.

```yaml
- name: Sign with ad-hoc signature
  run: |
    codesign --sign - \
      --force \
      --deep \
      --options runtime \
      --timestamp \
      ./cco

    codesign --verify --verbose ./cco
```

**Pros**:
- No Apple Developer account required
- Free
- Quick to implement

**Cons**:
- Users still see Gatekeeper warnings
- Not notarized by Apple
- Less trustworthy than Developer ID

**Source**: [UV Python SIGKILL Issue on macOS](https://github.com/astral-sh/uv/issues/16726)

### Approach 3: Using taiki-e/upload-rust-binary-action

Simplify the workflow by using a pre-built action:

```yaml
- uses: taiki-e/upload-rust-binary-action@v1
  with:
    bin: cco
    target: aarch64-apple-darwin
    codesign: "${{ secrets.APPLE_TEAM_ID }}"
    codesign-prefix: "com.visiquate."
    codesign-options: "runtime"
    token: ${{ secrets.GITHUB_TOKEN }}
```

**Note**: This handles signing but NOT notarization. You'd still need to add notarization steps.

---

## 6. Alternative Solutions (If Code Signing Not Available)

### Option 1: Homebrew with --no-quarantine Flag

Homebrew can bypass quarantine for specific packages:

**User-side fix**:
```bash
brew install visiquate/cco/cco --no-quarantine
```

**Global setting**:
```bash
export HOMEBREW_CASK_OPTS="--no-quarantine"
```

**Pros**: Works immediately, no changes needed to your build process
**Cons**: Users must remember the flag, only works for Homebrew

**Source**: [My struggles with Gatekeeper errors and Homebrew](https://thefridaydeploy.substack.com/p/my-struggles-with-gatekeeper-errors)

### Option 2: Documentation for Manual Quarantine Removal

Provide clear instructions for users to remove quarantine manually:

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine $(which cco)

# Or re-sign with ad-hoc signature
sudo codesign --force --deep --sign - $(which cco)
```

**Pros**: No build changes needed, gives users control
**Cons**: Poor user experience, requires command-line knowledge

**Source**: [Dealing with macOS Gatekeeper](https://elroyjetson.org/notes/dealing-with-macos-gatekeeper)

### Option 3: Distribute as .pkg Installer

Package the CLI as a .pkg installer, which supports stapling:

**Pros**:
- Can staple notarization ticket
- Better offline experience
- Professional distribution method

**Cons**:
- More complex build process
- Requires Developer ID Installer certificate
- Larger download size

### Option 4: Pure Rust Signing Solution

Use the `apple-codesign` Rust crate for a cross-platform signing solution:

```bash
# Install rcodesign (Apple's code signing in pure Rust)
cargo install apple-codesign

# Sign on any platform (Linux, Windows, macOS)
rcodesign sign \
  --p12-file certificate.p12 \
  --p12-password-file password.txt \
  --code-signature-flags runtime \
  ./cco
```

**Pros**:
- Works on any platform (Linux runners, Windows, etc.)
- No macOS runner required
- Open source alternative to Apple tools

**Cons**:
- Less mature than official tools
- Still requires Apple Developer certificate

**Source**: [Achieving A Completely Open Source Implementation of Apple Code Signing](https://gregoryszorc.com/blog/2022/08/08/achieving-a-completely-open-source-implementation-of-apple-code-signing-and-notarization/)

---

## 7. Analysis of Current CCO Release Workflow

### Current Process (from release.yml)

Your current workflow:
1. Builds on self-hosted macOS ARM64 runner
2. Compiles with `cargo build --release`
3. Creates tarball: `tar -czvf cco-aarch64-apple-darwin.tar.gz cco`
4. Generates SHA256 checksum
5. Uploads to GitHub Releases via `visiquate/cco` repository

**Missing**:
- No code signing
- No notarization
- No quarantine handling

### Why Users Experience SIGKILL

When users install via Homebrew:
1. Homebrew downloads tarball from GitHub Releases
2. macOS marks the file with `com.apple.quarantine` attribute (sometimes)
3. When CCO executes, Gatekeeper checks for valid signature
4. No signature found → Gatekeeper sends SIGKILL
5. Process exits with code 137 (128 + 9)

**Source**: [UV Python SIGKILL Issue on macOS](https://github.com/astral-sh/uv/issues/16726)

---

## 8. Recommended Implementation Plan

### Phase 1: Quick Fix (No Apple Developer Account)

**Timeline**: 1-2 hours
**Cost**: Free

Add ad-hoc signing to your existing workflow:

```yaml
- name: Build binary
  run: |
    cargo build --release --target ${{ matrix.target }}
    cp target/${{ matrix.target }}/release/cco artifacts/cco
    chmod +x artifacts/cco

    # Add ad-hoc signature
    codesign --sign - \
      --force \
      --deep \
      --options runtime \
      --timestamp \
      artifacts/cco

    codesign --verify --verbose artifacts/cco
```

**Expected Outcome**: May resolve SIGKILL issues, but users will still see Gatekeeper warnings on first run.

### Phase 2: Full Solution (Requires Apple Developer Account)

**Timeline**: 1 day
**Cost**: $99/year (Apple Developer Program)

1. **Enroll in Apple Developer Program** ($99/year)
2. **Generate Developer ID Application certificate**
3. **Export certificate as .p12** with password
4. **Store GitHub Secrets**:
   - `APPLE_CERTIFICATE_BASE64`
   - `APPLE_CERTIFICATE_PASSWORD`
   - `APPLE_TEAM_ID`
   - `APPLE_ID`
   - `APPLE_ID_PASSWORD`
   - `KEYCHAIN_PASSWORD`
5. **Update release.yml** with signing + notarization steps (see Approach 1 above)
6. **Test with a release candidate**
7. **Update documentation** with "verified developer" status

**Expected Outcome**: No Gatekeeper warnings, seamless user experience, professional distribution.

### Phase 3: Enhanced Distribution (Optional)

**Timeline**: 2-3 days
**Cost**: Development time

Consider:
- **Package as .pkg installer** for better stapling support
- **Universal binary** (combine x86_64 + aarch64 into single fat binary)
- **Homebrew cask** for GUI installation experience
- **Auto-update mechanism** with signature verification

---

## 9. Cost-Benefit Analysis

### Option A: Do Nothing
- **Cost**: $0
- **Benefit**: None
- **Risk**: Users continue experiencing SIGKILL, poor user experience, support requests

### Option B: Ad-hoc Signing
- **Cost**: 1-2 hours development time
- **Benefit**: May resolve SIGKILL, minimal effort
- **Risk**: Users still see warnings, not a complete solution

### Option C: Full Developer ID Signing + Notarization
- **Cost**: $99/year + 1 day setup
- **Benefit**: Professional distribution, no warnings, trusted developer status
- **Risk**: Annual cost, certificate management overhead

**Recommendation**: Start with **Option B (ad-hoc signing)** as immediate fix, then implement **Option C (full signing)** when resources allow. The $99/year cost is justified for a professional tool like CCO.

---

## 10. Additional Resources

### Official Apple Documentation
- [Signing Mac Software with Developer ID](https://developer.apple.com/developer-id/)
- [Notarizing macOS Software Before Distribution](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Introduction/Introduction.html)

### Community Guides
- [Automatic Code-signing and Notarization for macOS apps using GitHub Actions](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/)
- [A very rough guide to notarizing CLI apps for macOS](https://www.randomerrata.com/articles/2024/notarize/)
- [How to automatically sign macOS apps using GitHub Actions](https://localazy.com/blog/how-to-automatically-sign-macos-apps-using-github-actions)

### Tools & Libraries
- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action) - GitHub Action with macOS signing support
- [mitchellh/gon](https://github.com/mitchellh/gon) - Sign, notarize, and package macOS CLI tools
- [apple-codesign](https://gregoryszorc.com/docs/apple-codesign/stable/) - Pure Rust implementation of Apple code signing
- [xcnotary](https://github.com/akeru-inc/xcnotary) - macOS notarization helper built with Rust

### Troubleshooting
- [My struggles with Gatekeeper errors and Homebrew](https://thefridaydeploy.substack.com/p/my-struggles-with-gatekeeper-errors)
- [UV Python SIGKILL Issue on macOS](https://github.com/astral-sh/uv/issues/16726)
- [Dealing with macOS Gatekeeper](https://elroyjetson.org/notes/dealing-with-macos-gatekeeper)

---

## Conclusion

The CCO Gatekeeper issue stems from distributing unsigned binaries on macOS. While major Rust projects like ripgrep and bat also distribute unsigned binaries, they work via Homebrew because Homebrew doesn't set quarantine flags. However, for a professional tool like CCO, implementing proper code signing and notarization is the recommended solution.

**Short-term**: Add ad-hoc signing to the existing workflow (1-2 hours, free)
**Long-term**: Implement full Developer ID signing and notarization ($99/year, 1 day setup)

The long-term solution provides the best user experience and establishes CCO as a trusted, professionally-distributed tool.

---

## Sources

- [Signing Mac Software with Developer ID - Apple Developer](https://developer.apple.com/developer-id/)
- [Code Signing and Notarization on macOS](https://www.msweet.org/blog/2020-12-10-macos-notarization.html)
- [Automatic Code-signing and Notarization for macOS apps using GitHub Actions](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/)
- [A very rough guide to notarizing CLI apps for macOS](https://www.randomerrata.com/articles/2024/notarize/)
- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action)
- [My struggles with Gatekeeper errors and Homebrew](https://thefridaydeploy.substack.com/p/my-struggles-with-gatekeeper-errors)
- [UV Python SIGKILL Issue on macOS](https://github.com/astral-sh/uv/issues/16726)
- [macOS distribution — code signing, notarization, quarantine](https://gist.github.com/rsms/929c9c2fec231f0cf843a1a746a416f5)
- [How to sign and notarize a PKG within a Github Actions macos runner](https://stackoverflow.com/questions/70991268/how-to-sign-and-notarize-a-pkg-within-a-github-actions-macos-runner)
- [Achieving A Completely Open Source Implementation of Apple Code Signing](https://gregoryszorc.com/blog/2022/08/08/achieving-a-completely-open-source-implementation-of-apple-code-signing-and-notarization/)
- [Dealing with macOS Gatekeeper](https://elroyjetson.org/notes/dealing-with-macos-gatekeeper)
- [How to automatically sign macOS apps using GitHub Actions](https://localazy.com/blog/how-to-automatically-sign-macos-apps-using-github-actions)
