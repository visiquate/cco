# macOS Code Signing & Notarization - Quick Reference

## One-Page Overview

### Problem → Solution → Result

```
┌─────────────────────┐
│  Problem            │
│  Gatekeeper blocks  │
│  unsigned binary    │
└──────────┬──────────┘
           │
    ┌──────▼──────┐
    │   Solution  │
    │  Sign with  │
    │ Dev ID cert │
    │ + Notarize  │
    └──────┬──────┘
           │
┌──────────▼──────────┐
│      Result         │
│ Binary runs on      │
│ first click         │
│ No warnings         │
└─────────────────────┘
```

## Implementation Path (2 Hours)

```
Start
  │
  ├─ Run: ./scripts/setup-macos-signing.sh (45 min)
  │  └─ Import cert, store credentials
  │
  ├─ Add GitHub secrets (15 min)
  │  └─ MACOS_CODESIGN_IDENTITY, APPLE_ID, APPLE_TEAM_ID
  │
  ├─ Test with v*.*.* -test tag (30 min)
  │  └─ Push tag, watch workflow, verify binary
  │
  └─ Release production version (5 min)
     └─ Push tag, workflow signs/notarizes, done!
```

## GitHub Actions Workflow Steps (Added)

### Build Job (macOS ARM)

**After binary is created:**

```
1️⃣  Sign binary
    ├─ codesign --sign "$IDENTITY"
    ├─ --timestamp (required for notarization)
    ├─ --options runtime (hardening)
    └─ Verify with codesign --verify

2️⃣  Notarize binary
    ├─ Extract from tarball
    ├─ xcrun notarytool submit
    ├─ --wait (block until Apple approves)
    └─ Re-create tarball

3️⃣  Verify signatures
    ├─ codesign --verify
    ├─ spctl (Gatekeeper check)
    ├─ codesign -dvv (show details)
    └─ xcrun stapler validate
```

## Certificate Storage

### NOT in GitHub Secrets

```
❌ GitHub Secrets
   - Shows in logs
   - Less secure
   - Complicated export/import
```

### YES in Runner Keychain

```
✅ Runner Keychain
   - OS managed encryption
   - Runner user only
   - Automatic with xcrun
   - Secure & clean
```

## GitHub Secrets Required

| Secret Name | Example | Source |
|------------|---------|--------|
| `MACOS_CODESIGN_IDENTITY` | `"Developer ID Application: Acme (AB12CD34EF)"` | `security find-identity` |
| `APPLE_ID` | `release@example.com` | Apple Developer account |
| `APPLE_TEAM_ID` | `AB12CD34EF` | Apple Developer account |

**Password:** Stored in runner keychain (not in GitHub)

## Keychain Commands

### Store Credentials
```bash
security add-generic-password \
  -a "your.email@example.com" \
  -s "notarytool-password" \
  -w "16-char-app-password"
```

### Verify Credentials
```bash
security find-generic-password -s "notarytool-password"
```

### List Code Signing Certs
```bash
security find-identity -v -p codesigning
```

## Workflow Failure Diagnosis

```
❌ Signing step failed?
   └─ security find-identity -v -p codesigning
   └─ Check certificate is imported

❌ Notarization failed?
   └─ Check Apple Developer account
   └─ Look for rejection reason
   └─ Retry submission

❌ Verify step failed?
   └─ Run diagnostic: ./scripts/diagnose-signing.sh
   └─ Check GitHub secrets
   └─ Check runner keychain
```

## User Experience

### Before (Without Signing)
```
1. Download binary
2. Try to run
3. ⚠️  "Unidentified Developer" warning
4. Open System Preferences
5. Allow app
6. Run app
7. Finally works... (annoying!)
```

### After (With Signing + Notarization)
```
1. Download binary
2. Try to run
3. ✓ Just works!
```

## Files Modified/Created

### Modified
- `.github/workflows/release.yml` (3 new steps added)

### New Documentation
- `README_MACOS_SIGNING.md` - Start here
- `MACOS_SIGNING_DESIGN_SUMMARY.md` - Architecture
- `MACOS_SIGNING_AND_NOTARIZATION.md` - Complete guide
- `MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md` - Process
- `MACOS_SIGNING_TROUBLESHOOTING.md` - Problems
- `HOMEBREW_FORMULA_TEMPLATE.rb` - Updated formula
- `SIGNING_QUICK_REFERENCE.md` - This file

### New Scripts
- `scripts/setup-macos-signing.sh` - One-time setup
- `scripts/diagnose-signing.sh` - Health check

## Maintenance Schedule

| Task | Frequency | Effort |
|------|-----------|--------|
| Certificate renewal | Yearly | 15 min |
| Credentials audit | Quarterly | 10 min |
| Keychain unlock | After reboot | Auto |
| Workflow test | Per release | ~15 min |

## Quick Testing

### After Download
```bash
# Verify signature
codesign --verify --verbose cco

# Gatekeeper check
spctl -a -vvv cco

# Run it
./cco --version
```

### System Health Check
```bash
./scripts/diagnose-signing.sh
```

## Common Commands

```bash
# Show code signing identities
security find-identity -v -p codesigning

# Check certificate details
codesign -dvvv /path/to/cco

# Verify signature
codesign --verify /path/to/cco

# Gatekeeper check
spctl -a -vvv /path/to/cco

# Check notarization status (from logs)
xcrun notarytool info UUID \
  --apple-id EMAIL \
  --team-id TEAM_ID \
  --password-keychain notarytool-password
```

## Success Indicators

✅ All checks pass:
```bash
✓ security find-identity shows Developer ID
✓ codesign --verify passes
✓ spctl check passes
✓ ./cco runs without warnings
✓ Gatekeeper doesn't prompt
```

## Troubleshooting Flowchart

```
Issue?
  │
  ├─ Run: ./scripts/diagnose-signing.sh
  │
  └─ See output → Look up in:
     MACOS_SIGNING_TROUBLESHOOTING.md
```

## First Release Checklist

- [ ] Run: `./scripts/setup-macos-signing.sh`
- [ ] Add GitHub secrets (MACOS_CODESIGN_IDENTITY, APPLE_ID, APPLE_TEAM_ID)
- [ ] Create test tag: `git tag v*.*.*-test`
- [ ] Push test tag: `git push origin v*.*.*-test`
- [ ] Verify workflow passes
- [ ] Download binary and test locally
- [ ] Clean up test tag: `git tag -d v*.*.*-test && git push origin :v*.*.*-test`
- [ ] Create production tag: `git tag v*.*.*`
- [ ] Push production tag: `git push origin v*.*.*`
- [ ] Update Homebrew formula
- [ ] Announce release

## When Things Go Wrong

1. **First:** Run diagnostic
   ```bash
   ./scripts/diagnose-signing.sh
   ```

2. **Then:** Check troubleshooting guide
   - See: `MACOS_SIGNING_TROUBLESHOOTING.md`
   - Find your issue number
   - Follow solution

3. **Finally:** Re-run diagnostic
   ```bash
   ./scripts/diagnose-signing.sh
   ```

## References

- [Code Signing (Apple)](https://developer.apple.com/documentation/security/code-signing-guide)
- [Notarization (Apple)](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Gatekeeper (Apple Support)](https://support.apple.com/en-us/HT202491)
- [Complete Guide](README_MACOS_SIGNING.md)

## Support

**Setup Issues?** → `MACOS_SIGNING_TROUBLESHOOTING.md`
**Architecture?** → `MACOS_SIGNING_DESIGN_SUMMARY.md`
**Process?** → `MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md`
**Technical Details?** → `MACOS_SIGNING_AND_NOTARIZATION.md`

---

**Status:** ✅ Ready to implement
**Time to First Release:** 2 hours
**Cost:** Free (uses existing runner + free Apple notarization)
**Benefit:** Professional binary distribution, user trust, Homebrew integration
