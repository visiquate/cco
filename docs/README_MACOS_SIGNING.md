# macOS Code Signing & Notarization - Complete Implementation Guide

**Quick Start:** 2 hours to first signed/notarized release

This guide helps you implement code signing and Apple notarization for CCO's macOS releases, ensuring users get binaries that run without Gatekeeper warnings.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Documentation Guide](#documentation-guide)
4. [Implementation Phases](#implementation-phases)
5. [Troubleshooting](#troubleshooting)

## Overview

### What We're Solving

**Problem:** Users download CCO binary and Gatekeeper blocks it
```
"CCO" can't be opened because it's from an unidentified developer.
```

**Solution:** Code signing + Apple notarization

**Result:** Binary runs immediately with no warnings
```
✓ Binary runs on first double-click
✓ No security dialog
✓ Trusted by macOS Gatekeeper
✓ Available in Homebrew
```

### How It Works

1. **Sign:** Add Developer ID Application certificate signature to binary
2. **Notarize:** Submit to Apple, they scan for malware and approve
3. **Distribute:** Users download and it just works
4. **Verify:** Gatekeeper checks signature and notarization at runtime

```
┌─────────────┐    ┌──────────────┐    ┌──────────┐
│ GitHub Repo │───▶│ Apple Server │───▶│ Download │───▶ ✓ Just Works
└─────────────┘    │ (Notarize)   │    └──────────┘
                   └──────────────┘
```

## Quick Start

### For First-Time Setup (30-45 minutes)

**1. On mac-mini-arm64 runner:**
```bash
cd /Users/brent/git/cc-orchestra
chmod +x scripts/setup-macos-signing.sh
./scripts/setup-macos-signing.sh
```

This interactive script will:
- Import your Developer ID Application certificate
- Store notarization credentials in keychain
- Verify everything works

**2. Add GitHub Secrets:**
Go to repository Settings → Secrets → Add:
- `MACOS_CODESIGN_IDENTITY` (from setup script output)
- `APPLE_ID` (your Apple ID email)
- `APPLE_TEAM_ID` (from setup script output)

**3. Create test release:**
```bash
git tag v2025.12.1-test
git push origin v2025.12.1-test
```

Watch the workflow - all steps should complete successfully.

**4. Verify the binary:**
```bash
# Download from GitHub release
tar -xzf cco-aarch64-apple-darwin.tar.gz

# Verify signature
codesign --verify --verbose cco

# Check Gatekeeper
spctl -a -vvv cco

# Run it
./cco --version
```

**5. Clean up test tag:**
```bash
git tag -d v2025.12.1-test
git push origin :v2025.12.1-test
```

### For Release Day (5 minutes)

```bash
git tag v2025.12.1
git push origin v2025.12.1

# Workflow runs automatically
# Binaries are signed and notarized
# Users download and it works!
```

## Documentation Guide

### Choose Your Path

**👤 I'm a DevOps Engineer Setting This Up**
1. Start: [MACOS_SIGNING_DESIGN_SUMMARY.md](#design-summary)
2. Implement: [MACOS_SIGNING_AND_NOTARIZATION.md](#main-guide)
3. Execute: [MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md](#checklist)
4. Troubleshoot: [MACOS_SIGNING_TROUBLESHOOTING.md](#troubleshooting-guide)

**🚀 I'm a Release Engineer Cutting Releases**
1. Start: [MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md](#checklist) - Phase 6
2. Reference: [MACOS_SIGNING_AND_NOTARIZATION.md](#main-guide) - Verification section
3. Troubleshoot: [MACOS_SIGNING_TROUBLESHOOTING.md](#troubleshooting-guide) - If issues

**🔧 Something's Broken, Help!**
1. Run diagnostic: `./scripts/diagnose-signing.sh`
2. Find your issue: [MACOS_SIGNING_TROUBLESHOOTING.md](#troubleshooting-guide)
3. Execute fix
4. Re-run diagnostic

**📚 I Want to Understand Everything**
1. Start here (you are here!)
2. Read: [MACOS_SIGNING_DESIGN_SUMMARY.md](#design-summary)
3. Deep dive: [MACOS_SIGNING_AND_NOTARIZATION.md](#main-guide)
4. Reference: [MACOS_SIGNING_TROUBLESHOOTING.md](#troubleshooting-guide)

---

## Documentation Detailed Guide

### Design Summary
**File:** `docs/MACOS_SIGNING_DESIGN_SUMMARY.md`

**Contains:** Architecture decisions, security considerations, cost analysis

**Read this if you want to:**
- Understand why we chose this approach
- See the security model
- Know the infrastructure requirements
- Review cost/benefit analysis
- See what alternatives were considered

**Key Sections:**
- Architecture Overview
- Technical Decisions (Keychain vs GitHub Secrets)
- Error Handling Strategy
- Cost Analysis
- Success Criteria

---

### Main Implementation Guide
**File:** `docs/MACOS_SIGNING_AND_NOTARIZATION.md`

**Contains:** Step-by-step implementation with YAML code

**Read this if you want to:**
- Set up the runner
- Understand each workflow step
- Configure GitHub secrets
- Update Homebrew formula
- Verify the setup

**Key Sections:**
1. Certificate Storage Approach
2. Prerequisites for Self-Hosted Runner
3. Workflow Changes (exact YAML code)
4. GitHub Secrets Configuration
5. Self-Hosted Runner Setup Script
6. Error Handling and Verification
7. Homebrew Formula Updates
8. Verification Checklist
9. References & Resources

**Highlight:** Lines 134-275 contain exact YAML to add to release.yml

---

### Implementation Checklist
**File:** `docs/MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md`

**Contains:** Phase-by-phase checklist with timeline

**Read this if you want to:**
- Follow a structured implementation plan
- Know exactly what to do next
- Track progress
- Have clear sign-off points

**Phases:**
1. Self-Hosted Runner Setup (30-45 min) - DevOps
2. GitHub Configuration (15 min) - Maintainer
3. Workflow Implementation (Already done!)
4. Documentation Review (10 min) - Team
5. Testing & Validation (30 min) - Release Eng
6. Deployment (5 min) - Release Eng
7. Ongoing Maintenance (Yearly)

**Each phase:** ✅ Checkbox, timeline, responsible party

---

### Troubleshooting Guide
**File:** `docs/MACOS_SIGNING_TROUBLESHOOTING.md`

**Contains:** 10 common issues with solutions

**Read this if:**
- Setup failed at some step
- Workflow is failing
- Binary won't run
- You're stuck

**Issues Covered:**
1. Certificate Not Found
2. Workflow Uses Old Certificate
3. Codesign Permission Denied
4. Notarization Password Not Found
5. Notarization Rejected
6. Gatekeeper Still Blocking Binary
7. Signature Timestamp Verification Failed
8. "Invalid Input" from notarytool
9. Workflow Timeout
10. "No matching identities found"

**Format:** Quick diagnosis, solution, manual testing

---

### Homebrew Formula Template
**File:** `docs/HOMEBREW_FORMULA_TEMPLATE.rb`

**Contains:** Updated Homebrew formula with verification steps

**Read this if:**
- You maintain the homebrew-cco repository
- You need to update the formula for a new release
- You want to add post-install verification

**Key Updates:**
- Post-install hook verifies code signature
- SHA256 values need updating each release
- Developer ID Application verification

---

### Diagnostic Script
**File:** `scripts/diagnose-signing.sh`

**What it does:** Complete system health check

**Run it:** `./scripts/diagnose-signing.sh`

**Checks:**
- Xcode installation
- Code signing certificates
- Notarization credentials
- Keychain access
- Runner configuration
- Workflow files
- Documentation

**Output:** ✓ Pass / ❌ Fail / ⚠️ Warning with guidance

---

### Setup Script
**File:** `scripts/setup-macos-signing.sh`

**What it does:** Interactive one-time runner setup

**Run it:** `./scripts/setup-macos-signing.sh`

**Prompts for:**
- Certificate file path
- Apple ID
- Team ID
- App-specific password

**Does:**
1. Imports certificate to keychain
2. Stores credentials
3. Tests everything works
4. Provides GitHub secret values
5. Tests code signing

---

## Implementation Phases

### Phase 1: One-Time Runner Setup (45 minutes)

**Who:** DevOps/Infrastructure team with runner access

**What:** Get the self-hosted runner ready to sign/notarize

**Steps:**
1. Get Developer ID certificate from Apple Developer Program
2. Run: `./scripts/setup-macos-signing.sh`
3. Answer prompts (certificate path, Apple ID, etc.)
4. Save output values
5. Run: `./scripts/diagnose-signing.sh` to verify

**Result:** Runner can sign and notarize binaries

---

### Phase 2: GitHub Configuration (15 minutes)

**Who:** Repository maintainer

**What:** Add signing credentials to GitHub

**Steps:**
1. Copy values from Phase 1 setup script
2. Go to GitHub: Settings → Secrets and variables → Actions
3. Add 3 secrets:
   - MACOS_CODESIGN_IDENTITY
   - APPLE_ID
   - APPLE_TEAM_ID
4. Verify secrets appear in `gh secret list`

**Result:** Workflow can access signing credentials

---

### Phase 3: Workflow Implementation

**Status:** ✅ Already done!

**What was added:** 3 new steps to release.yml
- Sign binary
- Notarize binary
- Verify signatures

**File:** `.github/workflows/release.yml` (lines 134-282)

---

### Phase 4: Testing (30 minutes)

**Who:** Release engineer

**What:** Verify everything works end-to-end

**Steps:**
1. Create test tag: `git tag v2025.12.1-test`
2. Push: `git push origin v2025.12.1-test`
3. Watch workflow in GitHub Actions
4. Download binary from test release
5. Verify: `codesign --verify --verbose cco`
6. Clean up: `git tag -d v2025.12.1-test && git push origin :v2025.12.1-test`

**Result:** Confident in production releases

---

### Phase 5: Production Release (5 minutes)

**Who:** Release engineer

**What:** Create an actual release with signed binaries

**Steps:**
1. Create tag: `git tag v2025.12.1`
2. Push: `git push origin v2025.12.1`
3. Wait for workflow to complete
4. Update Homebrew formula
5. Announce release

**Result:** Signed/notarized binary available to users

---

### Phase 6: Ongoing Maintenance

**Timeline:** Minimal work, yearly certificate renewal

**Quarterly:** Verify credentials still work
```bash
./scripts/diagnose-signing.sh
```

**Yearly:** Renew certificate (30 days before expiration)
- Download new Developer ID cert from developer.apple.com
- Import to keychain
- Update GitHub secret

---

## Troubleshooting

### Quick Diagnosis

Run this command first:
```bash
./scripts/diagnose-signing.sh
```

This checks:
- ✓ System prerequisites
- ✓ Code signing certificates
- ✓ Notarization credentials
- ✓ Keychain access
- ✓ Workflow configuration

### Common Issues

**Certificate not found:**
```bash
security find-identity -v -p codesigning
# If empty, import certificate:
open ~/Downloads/DeveloperIDApplication.cer
```

**Notarization fails:**
Check Apple Developer account for rejection reason:
```bash
# From workflow logs, get REQUEST_UUID
xcrun notarytool info "$REQUEST_UUID" \
  --apple-id "YOUR_EMAIL" \
  --team-id "YOUR_TEAM_ID" \
  --password-keychain "notarytool-password"
```

**Gatekeeper still blocking:**
- Wait 10-30 minutes (notarization propagation)
- Or manually remove quarantine (dev only):
```bash
xattr -d com.apple.quarantine ~/Downloads/cco
```

**More issues?** See [MACOS_SIGNING_TROUBLESHOOTING.md](MACOS_SIGNING_TROUBLESHOOTING.md)

---

## Key Files Overview

### Modified Files
- `.github/workflows/release.yml` - Added signing/notarization steps (3 new jobs)

### New Documentation (2,000+ lines)
- `docs/MACOS_SIGNING_DESIGN_SUMMARY.md` - Architecture & decisions
- `docs/MACOS_SIGNING_AND_NOTARIZATION.md` - Complete guide with YAML
- `docs/MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md` - Phase-by-phase checklist
- `docs/MACOS_SIGNING_TROUBLESHOOTING.md` - 10 common issues + solutions
- `docs/HOMEBREW_FORMULA_TEMPLATE.rb` - Updated Homebrew formula
- `docs/README_MACOS_SIGNING.md` - This file

### New Scripts (730 lines)
- `scripts/setup-macos-signing.sh` - Interactive one-time setup
- `scripts/diagnose-signing.sh` - System health check

---

## Success Criteria

You've successfully implemented code signing when:

✅ Workflow runs without errors
✅ Binary is signed with Developer ID
✅ Binary passes Apple notarization
✅ Downloaded binary has valid signature: `codesign --verify` ✓
✅ Gatekeeper approves binary: `spctl -a -vvv` ✓
✅ Users can install via Homebrew: `brew install visiquate/cco/cco`
✅ Binary runs without warnings on first run
✅ No "unidentified developer" dialog

---

## Next Steps

**Right Now:**
1. Read [MACOS_SIGNING_DESIGN_SUMMARY.md](MACOS_SIGNING_DESIGN_SUMMARY.md)
2. Follow [MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md](MACOS_SIGNING_IMPLEMENTATION_CHECKLIST.md)

**When It's Done:**
1. Test with `v2025.12.1-test` tag
2. Verify binary downloads and runs
3. Release production version

**Questions?**
- Technical: [MACOS_SIGNING_AND_NOTARIZATION.md](MACOS_SIGNING_AND_NOTARIZATION.md)
- Problems: [MACOS_SIGNING_TROUBLESHOOTING.md](MACOS_SIGNING_TROUBLESHOOTING.md)
- Apple Docs: [Code Signing Guide](https://developer.apple.com/documentation/security/code-signing-guide)

---

## Contact & Support

**Setup Issues?** → Check [MACOS_SIGNING_TROUBLESHOOTING.md](MACOS_SIGNING_TROUBLESHOOTING.md)

**Workflow Issues?** → See [MACOS_SIGNING_AND_NOTARIZATION.md](MACOS_SIGNING_AND_NOTARIZATION.md) - Error Handling section

**Architecture Questions?** → Read [MACOS_SIGNING_DESIGN_SUMMARY.md](MACOS_SIGNING_DESIGN_SUMMARY.md)

**Emergency?** → Run `./scripts/diagnose-signing.sh` and share output

---

## Document Versions

| Document | Pages | Focus | For |
|----------|-------|-------|-----|
| This file | 1 | Navigation | Everyone |
| Design Summary | 4 | Why/How | Architects |
| Main Guide | 6 | YAML/Setup | DevOps |
| Checklist | 5 | Process | Project Mgmt |
| Troubleshooting | 6 | Problems | Operators |
| Homebrew | 1 | Formula | Package Mgmt |

**Total Documentation:** 23 pages of detailed guidance

---

**Ready to start?** → [MACOS_SIGNING_DESIGN_SUMMARY.md](MACOS_SIGNING_DESIGN_SUMMARY.md)

**Questions?** → [MACOS_SIGNING_TROUBLESHOOTING.md](MACOS_SIGNING_TROUBLESHOOTING.md)

**Let's go!** 🚀
