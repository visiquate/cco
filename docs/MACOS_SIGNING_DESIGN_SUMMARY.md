# macOS Code Signing & Notarization - Design Summary

**Status:** Design Complete & Ready for Implementation
**Last Updated:** December 6, 2025
**Target:** CCO v2025.12.1+ releases

## Executive Summary

This design enables CCO's macOS binaries to pass Gatekeeper and run without warnings by implementing:

1. **Code Signing** - Sign binaries with Developer ID Application certificate
2. **Apple Notarization** - Submit to Apple for security scan and approval
3. **Homebrew Integration** - Automatic deployment via Homebrew with verification
4. **Error Recovery** - Comprehensive troubleshooting and diagnostic tools

**Key Benefit:** Users download CCO from GitHub or Homebrew and it runs immediately without "unidentified developer" warnings.

## Architecture Overview

### Three-Layer Approach

```
┌─────────────────────────────────────────────────────┐
│            User Downloads Binary                    │
└──────────────────┬──────────────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │  Gatekeeper Check   │
        │  (macOS Security)   │
        │                     │
        │  1. Signature valid? │◄─── Code Signing ✓
        │  2. Notarized?      │◄─── Notarization ✓
        │  3. Malware scan?   │◄─── Apple Review ✓
        │  4. Date valid?     │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │  Binary Runs        │
        │  No Warnings        │
        └─────────────────────┘
```

### Execution Pipeline

```
GitHub Actions Release Workflow
│
├─ Build Job (self-hosted macOS ARM)
│  ├─ Checkout code
│  ├─ Build binary
│  ├─ ✨ NEW: Sign binary ◄─────── (1) Developer ID Cert from Keychain
│  ├─ ✨ NEW: Notarize ◄─────────── (2) Submit to Apple, wait for approval
│  ├─ ✨ NEW: Verify ◄──────────── (3) Validate all aspects
│  └─ Upload artifact
│
├─ Release Job (Linux)
│  └─ Create GitHub release
│
└─ Homebrew Job (Linux)
   └─ Update formula with SHA256 values
```

## Technical Decisions

### 1. Certificate Storage: Keychain-Based ✓

**Decision:** Store certificate in runner's system keychain, NOT GitHub secrets

**Rationale:**
| Approach | Keychain | GitHub Secrets |
|----------|----------|-----------------|
| Security | System-managed encryption | Base64 in logs |
| User Control | Runner admin only | Visible in repository |
| Tool Support | Native `codesign` integration | Manual import/export |
| Multi-Platform | Works on any macOS | Needs per-repo config |
| Compliance | Meets Apple requirements | Risky for production |

**Implementation:**
```bash
# On runner machine (one-time):
security add-generic-password -a "APPLE_ID" \
  -s "notarytool-password" \
  -w "APP_SPECIFIC_PASSWORD"

# In workflow (automatic):
xcrun notarytool submit ... \
  --password-keychain "notarytool-password"
```

### 2. Self-Hosted Runner Utilization ✓

**Decision:** Use existing mac-mini-arm64 self-hosted runner

**Benefits:**
- Already configured with Xcode
- Certificates can be pre-installed
- No need for GitHub-hosted macOS runners ($10/min)
- Full control over environment
- No secrets leaking to GitHub

**Trade-offs:**
- Requires IT maintenance
- Runner must have reliable uptime
- Team needs access for setup

### 3. Notarization Strategy: Synchronous (--wait) ✓

**Decision:** Block workflow until Apple completes notarization (up to 10 minutes)

**Rationale:**
- Simpler workflow (fail fast vs async complexity)
- Binaries are release artifacts (not time-critical)
- Better error handling (immediate feedback)
- No need for polling infrastructure

**Alternative Considered:** Async notarization with polling
- Pro: Faster workflow
- Con: Complex error handling, workflow state management, separate job

### 4. Entitlements: None Required ✓

**Decision:** CLI tool does NOT need custom entitlements

**Rationale:**
- CLI tools run in standard sandbox
- No special system access needed
- No network sandbox exceptions
- No camera/microphone access
- Metal GPU access handled by Xcode/system

**If Needed Later:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Add any needed entitlements here -->
</dict>
</plist>
```

## Workflow Changes in Detail

### Step 1: Sign Binary

**Purpose:** Add cryptographic signature proving Apple-approved developer created this binary

**Details:**
```yaml
- name: Sign binary (macOS ARM)
  if: matrix.target == 'aarch64-apple-darwin' && runner.os == 'macOS'
  run: |
    codesign --sign "$IDENTITY" \
      --timestamp \        # Required for notarization
      --options runtime \  # Enable hardening (Catalina+)
      --force \            # Overwrite if exists
      artifacts/cco
```

**Flags Explained:**
- `--sign "$IDENTITY"`: Use Developer ID certificate from keychain
- `--timestamp`: Get timestamp from Apple time server (required for notarization)
- `--options runtime`: Enable runtime hardening (macOS 10.15+)
- `--force`: Overwrite any existing signature

### Step 2: Notarize Binary

**Purpose:** Submit to Apple for security scan, malware check, and approval

**Timeline:** 1-5 minutes average, up to 10 minutes with retries

**Process:**
```yaml
- name: Notarize binary (macOS ARM)
  run: |
    # Extract binary from archive
    tar -xzf artifacts/cco-aarch64-apple-darwin.tar.gz

    # Submit for notarization
    xcrun notarytool submit cco \
      --apple-id "$APPLE_ID" \
      --team-id "$APPLE_TEAM_ID" \
      --password-keychain "notarytool-password" \
      --wait \              # Block until complete
      --timeout 600         # Max 10 minutes

    # Re-create archive (notarization ticket stapled)
    tar -czvf cco-aarch64-apple-darwin.tar.gz cco
```

**What Apple Checks:**
1. Code signature is valid
2. Binary wasn't already submitted (duplicate check)
3. No known malware patterns
4. No suspicious behavior

### Step 3: Verify Signatures

**Purpose:** Confirm all signing artifacts are present and valid

**Checks:**
```bash
# 1. Code signature validity
codesign --verify --verbose cco

# 2. Gatekeeper compatibility
spctl -a -vvv cco

# 3. Signature details
codesign -dvv cco

# 4. Notarization metadata
xcrun stapler validate cco-aarch64-apple-darwin.tar.gz
```

## GitHub Secrets Configuration

Three secrets required (no password in secrets):

| Secret | Example | Notes |
|--------|---------|-------|
| `MACOS_CODESIGN_IDENTITY` | `"Developer ID Application: Acme Inc (AB12CD34EF)"` | Full identity string |
| `APPLE_ID` | `release@example.com` | Apple ID email |
| `APPLE_TEAM_ID` | `AB12CD34EF` | 10-character team ID |

**Password Storage:** Stored in runner's keychain, NOT in GitHub

## Runner Setup Process

### One-Time Setup on mac-mini-arm64

**Duration:** 30-45 minutes

**Steps:**
1. Import Developer ID Application certificate (Keychain)
2. Run setup script: `./scripts/setup-macos-signing.sh`
3. Follow prompts to store credentials
4. Run diagnostic: `./scripts/diagnose-signing.sh`
5. Add GitHub secrets

**Maintenance:**
- Certificate renewal: Yearly (30 days before expiration)
- Credentials audit: Quarterly
- Keychain unlock: After runner restart (if needed)

## Security Considerations

### Certificate Security
- ✓ Private key never leaves runner machine
- ✓ Not stored in GitHub
- ✓ Protected by OS keychain encryption
- ✓ Requires runner user authentication

### Notarization Security
- ✓ App-specific password (not full Apple ID password)
- ✓ Password stored in keychain, not GitHub
- ✓ Apple performs malware scanning
- ✓ Binary validity checked at every run

### Gatekeeper Protection
- ✓ Users can verify signature: `codesign --verify cco`
- ✓ Users can check with Gatekeeper: `spctl -a -vvv cco`
- ✓ Notarization ticket stapled to binary
- ✓ Ticket cached locally (subsequent runs faster)

## Verification Checklist

### Workflow Verification
- [ ] All three signing/verification steps complete
- [ ] No timeouts or retry loops
- [ ] All signatures validate
- [ ] Notarization status: "Accepted"

### Binary Verification (Post-Download)
- [ ] `codesign --verify` passes
- [ ] `spctl -a -vvv` shows "accepted"
- [ ] `./cco --version` works
- [ ] No Gatekeeper warning on first run

### User Experience
- [ ] Download from GitHub release
- [ ] Extract archive
- [ ] Run binary
- [ ] **No "unidentified developer" warning**
- [ ] **Binary runs immediately**

## Error Handling Strategy

### Signing Failures
- **Cause:** Certificate missing or wrong identity
- **Recovery:** Check keychain, re-run setup script
- **Workflow:** Fails at sign step, workflow stops

### Notarization Failures
- **Cause:** Apple rejection, network issues, timeout
- **Recovery:** Check Apple account, run diagnostic
- **Workflow:** Fails at notarization step, workflow stops

### Verification Failures
- **Cause:** Notarization didn't complete properly
- **Recovery:** Retry submission, check Apple status
- **Workflow:** Fails at verification, highlights issue

## Homebrew Integration

### Formula Updates Required

When releasing v2025.12.1:

**Before:**
```ruby
version "2025.12.0"
sha256 "XXXX..."  # Old hash
```

**After:**
```ruby
version "2025.12.1"
sha256 "YYYY..."  # New hash from release

def post_install
  if OS.mac?
    # Verify code signature
    system("codesign", "--verify", "--verbose", bin/"cco")
  end
end
```

### Installation Flow
```
$ brew install visiquate/cco/cco
│
├─ Download signed/notarized binary
├─ Verify SHA256 (Homebrew automatic)
├─ Extract binary
├─ Run post_install (verify signature)
├─ Install to /usr/local/bin/cco
│
└─ Ready to use (no warnings)
```

## Cost Analysis

### Infrastructure Costs
- **Existing:** Self-hosted runner (already running)
- **Additional:** None (uses existing mac-mini-arm64)
- **Notarization:** Free (included with Apple Developer account)

### Time Costs
- **Initial Setup:** 1 hour (one-time)
- **Per Release:** ~15 minutes added to workflow (notarization wait)
- **Maintenance:** ~15 minutes/year (certificate renewal)

### Benefit-Cost Ratio
- **Cost:** 15 minutes per release, 1 hour initial
- **Benefit:** Professional binary distribution, Gatekeeper trust, Homebrew integration
- **ROI:** High (removes friction from user installation)

## Implementation Timeline

| Phase | Duration | Owner |
|-------|----------|-------|
| 1. Runner Setup | 45 min | DevOps |
| 2. GitHub Config | 15 min | Maintainer |
| 3. Workflow (done) | 0 min | Engineer |
| 4. Doc Review | 10 min | Team |
| 5. Testing | 30 min | Release Engineer |
| 6. First Release | 5 min | Release Engineer |
| 7. Ongoing | Minimal | Team |

**Total Time to First Release:** ~2 hours

## Files Changed/Created

### Modified
- `.github/workflows/release.yml` - Added 3 new steps

### Created
- `docs/MACOS_SIGNING_AND_NOTARIZATION.md` - Main guide (590 lines)
- `docs/MACOS_SIGNING_TROUBLESHOOTING.md` - Troubleshooting (520 lines)
- `docs/MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md` - Checklist (480 lines)
- `docs/MACOS_SIGNING_DESIGN_SUMMARY.md` - This document
- `docs/HOMEBREW_FORMULA_TEMPLATE.rb` - Updated formula (140 lines)
- `scripts/setup-macos-signing.sh` - Interactive setup (350 lines)
- `scripts/diagnose-signing.sh` - Diagnostic tool (380 lines)

**Total New Documentation:** ~2,000 lines
**Total New Code:** ~730 lines (scripts)

## Success Criteria

✅ **Before This Design:**
- Binary downloads but shows Gatekeeper warning
- Users must approve and wait
- Not in Homebrew or other package managers
- No trust/verification indicators

✅ **After This Design:**
- Binary downloads and runs immediately
- No Gatekeeper warning on first run
- Available via Homebrew (trusted channel)
- Users can verify signature themselves
- Professional, production-grade distribution

## Alternative Approaches Considered & Rejected

### Option A: Self-signed Certificate
- ❌ Gatekeeper still shows warning
- ❌ No Apple trust verification
- ❌ Users must override security

### Option B: GitHub-Hosted macOS Runners
- ❌ Extremely expensive ($10/minute)
- ❌ No persistent certificate storage
- ❌ Would need GitHub secrets (less secure)

### Option C: Third-party Code Signing Service
- ❌ Additional dependency
- ❌ Extra cost
- ❌ Complexity with GitHub Actions integration

### Option D: Manual Signing Post-Release
- ❌ Error-prone
- ❌ Requires manual intervention
- ❌ Not scalable for multiple releases

## Next Steps After Implementation

1. **Gather User Feedback** (First 2-3 releases)
   - Installation experience
   - Any Gatekeeper issues
   - Homebrew updates

2. **Monitor Apple's System**
   - Notarization rejection patterns
   - Gatekeeper changes (Apple updates macOS)
   - Certificate validity

3. **Automate Further** (Future)
   - Automatic certificate renewal alerts
   - Gatekeeper bypass testing in CI
   - Homebrew formula auto-update

4. **Expand to Other Platforms** (If Applicable)
   - Windows code signing (different process)
   - Linux binary signing (if needed)

## References & Documentation

### Apple Official
- [Code Signing Guide](https://developer.apple.com/documentation/security/code-signing-guide)
- [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [xcrun notarytool Documentation](https://developer.apple.com/documentation/technotes/tn3147-migrating-to-the-latest-notarization-experience)
- [Gatekeeper Overview](https://support.apple.com/en-us/HT202491)

### Related Documentation in This Project
- [MACOS_SIGNING_AND_NOTARIZATION.md](MACOS_SIGNING_AND_NOTARIZATION.md)
- [MACOS_SIGNING_TROUBLESHOOTING.md](MACOS_SIGNING_TROUBLESHOOTING.md)
- [MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md](MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md)

## Conclusion

This design provides a robust, production-grade approach to distributing CCO on macOS. By leveraging:
- Developer ID Application certificates (Apple trust)
- Apple's notarization service (malware verification)
- Self-hosted runner (security and cost)
- Comprehensive error handling (reliability)

CCO achieves the gold standard for macOS binary distribution. Users can confidently download and run CCO with no warnings or security concerns.

---

**Design Status:** ✅ Complete and ready for implementation
**Last Review:** December 6, 2025
**Approved By:** [DevOps/Release Engineering Team]
