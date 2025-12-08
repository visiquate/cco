#!/bin/bash
set -euo pipefail

# Diagnostic script for macOS code signing and notarization setup
# This helps identify missing configuration or issues

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║   macOS Code Signing & Notarization Diagnostic Report         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Run date: $(date)"
echo "System: $(sw_vers -productName) $(sw_vers -productVersion)"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_pass() { echo -e "${GREEN}✓${NC} $1"; }
check_fail() { echo -e "${RED}❌${NC} $1"; }
check_warn() { echo -e "${YELLOW}⚠️${NC} $1"; }

# Counter for issues
PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

# Section 1: System Prerequisites
echo ""
echo "Section 1: System Prerequisites"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    check_pass "Running on macOS"
    ((PASS_COUNT++))
else
    check_fail "Not running on macOS"
    ((FAIL_COUNT++))
    exit 1
fi

# Check Xcode
if xcode-select -p &>/dev/null; then
    XCODE_PATH=$(xcode-select -p)
    check_pass "Xcode Command Line Tools installed"
    echo "       Location: $XCODE_PATH"
    ((PASS_COUNT++))
else
    check_fail "Xcode Command Line Tools not installed"
    ((FAIL_COUNT++))
fi

# Check xcrun/notarytool
if xcrun --version &>/dev/null; then
    check_pass "xcrun is available"
    ((PASS_COUNT++))
else
    check_fail "xcrun not found"
    ((FAIL_COUNT++))
fi

# Section 2: Code Signing Certificates
echo ""
echo "Section 2: Code Signing Certificates"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

IDENTITIES=$(security find-identity -v -p codesigning 2>/dev/null || echo "")

if [ -z "$IDENTITIES" ]; then
    check_fail "No code signing identities found"
    ((FAIL_COUNT++))
else
    check_pass "Code signing identities found:"
    echo "$IDENTITIES" | while read -r line; do
        echo "       $line"
    done
    ((PASS_COUNT++))
fi

# Check for Developer ID
DEV_ID_COUNT=$(echo "$IDENTITIES" | grep -c "Developer ID Application" || echo "0")
if [ "$DEV_ID_COUNT" -gt 0 ]; then
    check_pass "Developer ID Application certificate found"
    ((PASS_COUNT++))
else
    check_warn "Developer ID Application certificate not found"
    ((WARN_COUNT++))
fi

# Section 3: Notarization Credentials
echo ""
echo "Section 3: Notarization Credentials"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for notarytool password in keychain
if security find-generic-password -s "notarytool-password" &>/dev/null 2>&1; then
    STORED_EMAIL=$(security find-generic-password -s "notarytool-password" -w 2>/dev/null || echo "")
    if [ -n "$STORED_EMAIL" ]; then
        check_pass "notarytool password stored in keychain"
        ((PASS_COUNT++))
    else
        check_warn "notarytool password found but may be inaccessible"
        ((WARN_COUNT++))
    fi
else
    check_fail "notarytool password not found in keychain"
    ((FAIL_COUNT++))
fi

# Section 4: GitHub Secrets
echo ""
echo "Section 4: GitHub Repository Secrets"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if we're in a git repo
if git rev-parse --git-dir > /dev/null 2>&1; then
    check_pass "Git repository detected"
    ((PASS_COUNT++))

    # Try to get GitHub secrets (limited info without auth)
    REPO_OWNER=$(git remote get-url origin | sed 's/.*github.com[:/]\([^/]*\).*/\1/')
    REPO_NAME=$(git remote get-url origin | sed 's/.*\/\([^/]*\)\.git.*/\1/')

    echo "       Owner: $REPO_OWNER"
    echo "       Repo: $REPO_NAME"
    echo ""
    echo "       To check secrets, run:"
    echo "       $ gh secret list"
else
    check_warn "Not in a git repository"
    ((WARN_COUNT++))
fi

# Section 5: Test Signing
echo ""
echo "Section 5: Test Code Signing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create temporary test binary
TEST_DIR=$(mktemp -d)
trap "rm -rf '$TEST_DIR'" EXIT

echo "#!/bin/bash" > "$TEST_DIR/test_binary"
chmod +x "$TEST_DIR/test_binary"

# Get the first Developer ID identity
DEV_ID=$(echo "$IDENTITIES" | grep "Developer ID Application" | head -1 | sed 's/^[[:space:]]*[0-9]*)[[:space:]]//' || echo "")

if [ -z "$DEV_ID" ]; then
    check_warn "Cannot test signing - no Developer ID certificate found"
    ((WARN_COUNT++))
else
    # Try to sign
    if codesign --sign "$DEV_ID" \
        --timestamp \
        --options runtime \
        --force \
        "$TEST_DIR/test_binary" 2>/dev/null; then
        check_pass "Test binary signed successfully"
        ((PASS_COUNT++))

        # Verify signature
        if codesign --verify "$TEST_DIR/test_binary" 2>/dev/null; then
            check_pass "Test signature verified successfully"
            ((PASS_COUNT++))
        else
            check_fail "Test signature verification failed"
            ((FAIL_COUNT++))
        fi
    else
        check_fail "Failed to sign test binary"
        ((FAIL_COUNT++))
    fi
fi

# Section 6: Keychain Access
echo ""
echo "Section 6: Keychain Access"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check keychain status
KEYCHAIN_STATUS=$(security default-keychain 2>/dev/null || echo "unknown")
check_pass "Default keychain: $KEYCHAIN_STATUS"
((PASS_COUNT++))

# Check if login keychain is unlocked
if security show-keychain-info ~/Library/Keychains/login.keychain-db &>/dev/null 2>&1; then
    check_pass "Login keychain is accessible"
    ((PASS_COUNT++))
else
    check_warn "Login keychain may be locked or inaccessible"
    ((WARN_COUNT++))
fi

# Section 7: Runner Configuration
echo ""
echo "Section 7: GitHub Actions Runner Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

CURRENT_USER=$(whoami)
check_pass "Current user: $CURRENT_USER"
((PASS_COUNT++))

# Check for GitHub Actions runner process
RUNNER_PID=$(pgrep -f "actions-runner" || echo "")
if [ -n "$RUNNER_PID" ]; then
    RUNNER_USER=$(ps -o user= -p "$RUNNER_PID" 2>/dev/null || echo "unknown")
    check_pass "GitHub Actions runner detected"
    echo "       PID: $RUNNER_PID"
    echo "       User: $RUNNER_USER"
    ((PASS_COUNT++))

    if [ "$RUNNER_USER" != "$CURRENT_USER" ]; then
        check_warn "Runner runs as '$RUNNER_USER' but current user is '$CURRENT_USER'"
        ((WARN_COUNT++))
    fi
else
    check_warn "GitHub Actions runner not currently running"
    ((WARN_COUNT++))
fi

# Section 8: Workflow File
echo ""
echo "Section 8: Workflow Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WORKFLOW_FILE=".github/workflows/release.yml"
if [ -f "$WORKFLOW_FILE" ]; then
    check_pass "Release workflow file exists"
    ((PASS_COUNT++))

    # Check for signing steps
    if grep -q "Sign binary" "$WORKFLOW_FILE"; then
        check_pass "Signing step found in workflow"
        ((PASS_COUNT++))
    else
        check_warn "Signing step not found in workflow"
        ((WARN_COUNT++))
    fi

    if grep -q "Notarize binary" "$WORKFLOW_FILE"; then
        check_pass "Notarization step found in workflow"
        ((PASS_COUNT++))
    else
        check_warn "Notarization step not found in workflow"
        ((WARN_COUNT++))
    fi
else
    check_fail "Release workflow file not found: $WORKFLOW_FILE"
    ((FAIL_COUNT++))
fi

# Section 9: Documentation
echo ""
echo "Section 9: Documentation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SETUP_DOC="docs/MACOS_SIGNING_AND_NOTARIZATION.md"
TROUBLE_DOC="docs/MACOS_SIGNING_TROUBLESHOOTING.md"

[ -f "$SETUP_DOC" ] && \
    check_pass "Setup documentation exists" && ((PASS_COUNT++)) || \
    check_warn "Setup documentation not found" && ((WARN_COUNT++))

[ -f "$TROUBLE_DOC" ] && \
    check_pass "Troubleshooting documentation exists" && ((PASS_COUNT++)) || \
    check_warn "Troubleshooting documentation not found" && ((WARN_COUNT++))

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary:"
echo ""
echo -e "${GREEN}Passed:${NC}   $PASS_COUNT"
echo -e "${YELLOW}Warnings:${NC} $WARN_COUNT"
echo -e "${RED}Failed:${NC}   $FAIL_COUNT"
echo ""

# Final status
if [ $FAIL_COUNT -eq 0 ]; then
    if [ $WARN_COUNT -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed! System is ready for code signing.${NC}"
        exit 0
    else
        echo -e "${YELLOW}✓ Setup mostly complete, but review warnings above.${NC}"
        exit 0
    fi
else
    echo -e "${RED}❌ Issues found. Please review failures above.${NC}"
    echo ""
    echo "Quick fixes:"
    echo ""

    if grep -q "No code signing identities found" <(echo "$IDENTITIES"); then
        echo "  1. Import Developer ID certificate:"
        echo "     $ open ~/Downloads/DeveloperIDApplication.cer"
        echo ""
    fi

    if grep -q "notarytool password not found" <(echo "$(security find-generic-password -s 'notarytool-password' 2>&1)"); then
        echo "  2. Store notarization password:"
        echo "     $ security add-generic-password -a 'EMAIL' -s 'notarytool-password' -w 'PASSWORD'"
        echo ""
    fi

    echo "  3. See docs/MACOS_SIGNING_TROUBLESHOOTING.md for detailed help"
    echo ""
    exit 1
fi
