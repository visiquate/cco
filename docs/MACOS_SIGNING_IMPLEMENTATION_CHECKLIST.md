# macOS Code Signing & Notarization Implementation Checklist

Complete this checklist to implement code signing and notarization for CCO releases on macOS.

## Phase 1: Self-Hosted Runner Setup (One-Time)

**Timeline:** 30-45 minutes
**Who:** DevOps/Infrastructure team with runner access

### Prerequisites
- [ ] Access to mac-mini-arm64 self-hosted runner
- [ ] macOS with Xcode Command Line Tools installed
- [ ] Apple Developer Program account
- [ ] Developer ID Application certificate from Apple

### Steps

#### 1.1 Certificate Preparation
- [ ] Go to [developer.apple.com/account](https://developer.apple.com/account)
- [ ] Navigate to "Certificates, Identifiers & Profiles"
- [ ] Click "Certificates"
- [ ] Create or download Developer ID Application certificate
- [ ] Download the .cer file
- [ ] Save to runner machine at: `~/Downloads/DeveloperIDApplication.cer`

#### 1.2 Runner Setup Script Execution
- [ ] SSH into mac-mini-arm64 runner
- [ ] Clone the repository: `git clone https://github.com/visiquate/cc-orchestra.git`
- [ ] Make script executable: `chmod +x scripts/setup-macos-signing.sh`
- [ ] Run setup script: `./scripts/setup-macos-signing.sh`
- [ ] Follow all prompts in the setup script
- [ ] Note the generated values:
  - Developer ID: `_________________________`
  - Apple ID: `_________________________`
  - Team ID: `_________________________`

#### 1.3 Verify Setup Completed
- [ ] Run diagnostic: `./scripts/diagnose-signing.sh`
- [ ] All checks pass (except maybe some warnings)
- [ ] Code signing identities found: `security find-identity -v -p codesigning`
- [ ] Notarization credentials stored: `security find-generic-password -s notarytool-password`

## Phase 2: GitHub Configuration

**Timeline:** 15 minutes
**Who:** Repository maintainer with GitHub admin access

### Add Repository Secrets
- [ ] Go to repository Settings → Secrets and variables → Actions
- [ ] Create secret `MACOS_CODESIGN_IDENTITY`
  - Value: Full identity string from Phase 1
  - Example: `"Developer ID Application: Your Name (ABCD1E2F3G)"`
- [ ] Create secret `APPLE_ID`
  - Value: Email address from Phase 1
- [ ] Create secret `APPLE_TEAM_ID`
  - Value: Team ID from Phase 1 (10 characters)

**Note:** Do NOT add the app-specific password as a secret - it's stored in runner keychain

### Verify Secrets
- [ ] Run: `gh secret list` (verify 3 secrets appear)
- [ ] Each secret value is correct (no typos)
- [ ] Secrets are accessible to the release workflow

## Phase 3: Workflow Implementation

**Timeline:** Already done - see release.yml changes**
**What was changed:**

### Modified Files
- [x] `.github/workflows/release.yml`
  - Added: "Sign binary (macOS ARM)" step
  - Added: "Notarize binary (macOS ARM)" step
  - Added: "Verify signed and notarized binary" step

### Workflow Steps Overview
1. Build binary (existing)
2. Sign binary with Developer ID certificate (NEW)
3. Notarize binary with Apple (NEW)
4. Verify signatures and notarization (NEW)
5. Upload artifact (existing)
6. Create release (existing)
7. Update Homebrew (existing)

### Workflow Features
- [ ] Signing uses `--timestamp` flag (required for notarization)
- [ ] Signing uses `--options runtime` flag (hardening on macOS 10.15+)
- [ ] Notarization uses `--wait` to block until Apple completes review
- [ ] Notarization timeout set to 600 seconds (10 minutes)
- [ ] Verification steps check all aspects:
  - Code signature validity
  - Entitlements
  - Gatekeeper compatibility
  - Notarization metadata

## Phase 4: Documentation Review

**Timeline:** 10 minutes
**Who:** Anyone implementing or maintaining this

### Documentation Files
- [ ] Read: `docs/MACOS_SIGNING_AND_NOTARIZATION.md`
  - Understand architecture decisions
  - Review all workflow steps
  - Know the notarization process
  - Understand macOS security policy

- [ ] Read: `docs/MACOS_SIGNING_TROUBLESHOOTING.md`
  - Bookmark for reference
  - Know where to find solutions
  - Understand common issues

- [ ] Reference: `docs/HOMEBREW_FORMULA_TEMPLATE.rb`
  - How to update Homebrew formula
  - Post-install verification steps
  - Security considerations

### Supporting Scripts
- [ ] Understand: `scripts/setup-macos-signing.sh`
  - One-time runner setup
  - Interactive configuration
  - Credential storage

- [ ] Understand: `scripts/diagnose-signing.sh`
  - Troubleshooting tool
  - System verification
  - Quick status check

## Phase 5: Testing & Validation

**Timeline:** 30 minutes
**Who:** Release engineer or maintainer

### Pre-Release Test
- [ ] Create a test tag: `git tag v2025.12.1-test`
- [ ] Push the tag: `git push origin v2025.12.1-test`
- [ ] Monitor workflow:
  - Go to Actions tab
  - Watch the release workflow
  - All jobs complete successfully
  - Verify signing step completes
  - Verify notarization step completes

### Workflow Log Verification
- [ ] "Sign binary" step shows:
  - `codesign --verify` passes
  - Signature details displayed
  - No errors or timeouts

- [ ] "Notarize binary" step shows:
  - Binary submitted to Apple
  - Notarization ticket received
  - Status shows "Accepted"
  - No rejection messages

- [ ] "Verify" step shows:
  - Code signature is valid
  - Gatekeeper check passes
  - Notarization metadata present

### Binary Verification (Download Test)
- [ ] Download the test macOS binary from GitHub release
- [ ] Extract: `tar -xzf cco-aarch64-apple-darwin.tar.gz`
- [ ] Verify signature: `codesign --verify --verbose cco`
  - Should show: "valid on disk" and "satisfies its Designated Requirement"
- [ ] Check Gatekeeper: `spctl -a -vvv cco`
  - Should show: "accepted" or similar approval
- [ ] Verify executable: `./cco --version`
  - Should run without Gatekeeper prompt
- [ ] Cleanup test tag: `git tag -d v2025.12.1-test && git push origin :v2025.12.1-test`

### First-Run User Experience
- [ ] Copy binary to another macOS machine
- [ ] Try to run it
- [ ] Verify NO "unidentified developer" warning appears
- [ ] Verify it runs on first try
- [ ] Verify subsequent runs are fast (cached notarization)

## Phase 6: Deployment

**Timeline:** 5 minutes
**Who:** Release engineer

### Create Release
- [ ] Create git tag: `git tag v2025.12.1`
- [ ] Push tag: `git push origin v2025.12.1`
- [ ] Workflow runs automatically
- [ ] All jobs complete successfully
- [ ] Release appears on GitHub

### Update Homebrew
- [ ] Update `Formula/cco.rb` in homebrew-cco repo
  - Update version number
  - Update SHA256 checksums
  - Test locally: `brew install --build-from-source Formula/cco.rb`
  - Commit and push changes

### Verify Release
- [ ] Release is visible on GitHub
- [ ] Binaries are available for download
- [ ] SHA256 checksums are published
- [ ] Release notes mention code signing

### Post-Release
- [ ] Test Homebrew installation: `brew install visiquate/cco/cco`
- [ ] Verify binary is properly signed
- [ ] Announce release to users

## Phase 7: Ongoing Maintenance

**Timeline:** Every 12 months (certificate renewal)
**Who:** DevOps team

### Certificate Renewal Reminder
- [ ] Set calendar reminder: 30 days before certificate expiration
- [ ] Developer ID Application certificates expire yearly
- [ ] Monitor Apple Developer account for renewal dates

### Rotation Steps (When Needed)
1. [ ] Go to developer.apple.com
2. [ ] Download renewed Developer ID Application certificate
3. [ ] Import to runner: `open ~/Downloads/DeveloperIDApplication.cer`
4. [ ] Extract new identity: `security find-identity -v -p codesigning`
5. [ ] Update GitHub secret: `MACOS_CODESIGN_IDENTITY`
6. [ ] Verify: Run `scripts/diagnose-signing.sh`

### Credentials Audit
- [ ] Monthly: Check for any notarization failures
- [ ] Quarterly: Verify all credentials are still valid
- [ ] Yearly: Review Apple Developer account activity

## Quick Reference: Commands

### On Runner Machine
```bash
# Verify setup
./scripts/diagnose-signing.sh

# View code signing identities
security find-identity -v -p codesigning

# Check notarization credentials
security find-generic-password -s "notarytool-password"

# Manual signing test
codesign --sign "Developer ID Application: Name (TEAM)" \
  --timestamp --options runtime /path/to/binary
```

### On GitHub
```bash
# Create release tag
git tag v2025.12.1
git push origin v2025.12.1

# Check workflow
gh workflow view release
gh run list --workflow release.yml

# View secrets
gh secret list
```

### Download & Verify
```bash
# Verify code signature
codesign --verify --verbose ./cco

# Gatekeeper check
spctl -a -vvv ./cco

# Check certificate details
codesign -dvvv ./cco
```

## Troubleshooting Quick Links

| Issue | Solution |
|-------|----------|
| Certificate not found | [TROUBLESHOOTING.md - Issue #1](MACOS_SIGNING_TROUBLESHOOTING.md#1-certificate-not-found) |
| Notarization fails | [TROUBLESHOOTING.md - Issue #5](MACOS_SIGNING_TROUBLESHOOTING.md#5-notarization-rejected) |
| Gatekeeper warning | [TROUBLESHOOTING.md - Issue #6](MACOS_SIGNING_TROUBLESHOOTING.md#6-gatekeeper-still-blocking-binary) |
| Timeout | [TROUBLESHOOTING.md - Issue #9](MACOS_SIGNING_TROUBLESHOOTING.md#9-workflow-timeout-during-notarization) |

## Files Modified/Created

### Modified
- `.github/workflows/release.yml` - Added signing/notarization steps

### New Files Created
- `docs/MACOS_SIGNING_AND_NOTARIZATION.md` - Main implementation guide
- `docs/MACOS_SIGNING_TROUBLESHOOTING.md` - Troubleshooting reference
- `docs/MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md` - This file
- `docs/HOMEBREW_FORMULA_TEMPLATE.rb` - Updated Homebrew formula
- `scripts/setup-macos-signing.sh` - Interactive setup script
- `scripts/diagnose-signing.sh` - Diagnostic tool

## Sign-Off

- [ ] Phase 1 Complete - Runner Setup (Date: __________)
- [ ] Phase 2 Complete - GitHub Configuration (Date: __________)
- [ ] Phase 3 Complete - Workflow Implemented (Date: __________)
- [ ] Phase 4 Complete - Documentation Reviewed (Date: __________)
- [ ] Phase 5 Complete - Testing & Validation (Date: __________)
- [ ] Phase 6 Complete - First Release Deployed (Date: __________)
- [ ] Phase 7 Complete - Maintenance Plan In Place (Date: __________)

## Success Criteria

Mark when each criterion is met:

- [ ] Workflow runs without errors on macOS ARM build
- [ ] Binary is successfully code-signed with Developer ID
- [ ] Binary is successfully notarized by Apple
- [ ] Downloaded binary has valid code signature
- [ ] Downloaded binary passes Gatekeeper check
- [ ] Downloaded binary runs without warnings on first run
- [ ] Homebrew installation works correctly
- [ ] Certificate renewal process is documented
- [ ] Team understands the process and troubleshooting

## Next Steps After Completion

1. Train team members on the process
2. Document certificate owner and rotation schedule
3. Set up calendar reminders for certificate renewal
4. Create runbook for emergencies (e.g., certificate compromised)
5. Monitor first few releases for issues
6. Gather user feedback on installation experience

## Contact & Support

- **Technical Questions:** See MACOS_SIGNING_AND_NOTARIZATION.md
- **Troubleshooting:** See MACOS_SIGNING_TROUBLESHOOTING.md
- **Apple References:** See documentation links in main guide
