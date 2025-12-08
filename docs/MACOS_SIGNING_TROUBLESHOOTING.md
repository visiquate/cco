# macOS Code Signing and Notarization Troubleshooting

This guide helps diagnose and resolve common issues with code signing and notarization.

## Quick Diagnostic Command

Run this command to get a complete status report:

```bash
./scripts/diagnose-signing.sh
```

Or manually check:

```bash
echo "=== Signing Identities ===" && \
security find-identity -v -p codesigning && \
echo "" && \
echo "=== Xcode Installation ===" && \
xcode-select -p && \
echo "" && \
echo "=== Keychain Password ===" && \
security find-generic-password -s "notarytool-password" -w >/dev/null 2>&1 && \
echo "✓ Found" || echo "❌ Not found"
```

## Common Issues

### 1. Certificate Not Found

**Symptoms:**
- Workflow fails at "Sign binary" step
- Error: `User interaction not allowed`
- Error: `no identity found`

**Diagnosis:**
```bash
security find-identity -v -p codesigning
```

Expected output:
```
1) ABC123DEF456... "Developer ID Application: Your Name (ABCD1E2F3G)"
```

If empty, the certificate is not installed.

**Solutions:**

#### Option A: Import existing certificate
```bash
# 1. Get the certificate file from developer.apple.com
open ~/Downloads/DeveloperIDApplication.cer

# 2. In Keychain Access, select the imported cert
# 3. Right-click → Export...
# 4. Save as: developer_id.p12
# 5. Copy to runner and import:
security import developer_id.p12 \
  -k ~/Library/Keychains/login.keychain-db \
  -P "password"

# 6. Verify
security find-identity -v -p codesigning
```

#### Option B: Create new certificate
1. Go to [developer.apple.com](https://developer.apple.com/account)
2. Certificates, Identifiers & Profiles → Certificates
3. Create → Developer ID Application
4. Download the certificate
5. Follow Option A steps above

### 2. Workflow Still Uses Old Certificate

**Symptoms:**
- Certificate was updated but workflow uses old one
- GitHub Actions job fails with wrong identity

**Solution:**
Update the `MACOS_CODESIGN_IDENTITY` GitHub secret:

1. Get the correct identity:
   ```bash
   security find-identity -v -p codesigning | grep "Developer ID"
   ```

2. Copy the full identity string: `"Developer ID Application: Your Name (TEAM_ID)"`

3. Update GitHub secret:
   - Go to: Settings → Secrets and variables → Actions
   - Click `MACOS_CODESIGN_IDENTITY`
   - Paste the new identity
   - Click "Update secret"

4. Re-run the workflow

### 3. Codesign Permission Denied

**Symptoms:**
```
codesign: error: The specified item could not be found in the keychain.
User interaction not allowed.
```

**Causes:**
- Runner is not the same user who imported the certificate
- Keychain is locked
- Certificate permissions are restrictive

**Solutions:**

#### Verify runner user:
```bash
# Check who the GitHub Actions runner runs as
ps aux | grep -i runner | grep -v grep

# Confirm current user
whoami

# They should match!
```

#### Unlock keychain:
```bash
# Unlock the login keychain
security unlock-keychain ~/Library/Keychains/login.keychain-db
# (may prompt for password)

# Or for root user
sudo security unlock-keychain /root/Library/Keychains/login.keychain-db
```

#### Fix certificate permissions:
```bash
# List certificates
security find-identity -v -p codesigning

# Get the SHA-1 hash from output (long hex string)
# Then modify permissions:
security set-identity-preference \
  -n \
  "Developer ID Application: Your Name (TEAM_ID)" \
  com.github.actions
```

### 4. Notarization Password Not Found

**Symptoms:**
```
xcrun notarytool submit ...
Error: Password could not be retrieved from the keychain
```

**Diagnosis:**
```bash
# Check if password exists
security find-generic-password -s "notarytool-password"

# If not found, you'll see: status -25299
```

**Solution:**

Store the password:
```bash
APPLE_ID="your.email@example.com"
read -s -p "App-specific password: " PASSWORD
security add-generic-password \
  -a "$APPLE_ID" \
  -s "notarytool-password" \
  -w "$PASSWORD" \
  -T /usr/bin/security \
  -T /usr/bin/xcrun \
  -A
```

Or if you need to update an existing password:
```bash
# Delete the old one
security delete-generic-password -a "APPLE_ID" -s "notarytool-password"

# Add the new one (same as above)
```

### 5. Notarization Rejected

**Symptoms:**
```
Notarization status: Rejected
```

**Diagnosis:**
Get detailed rejection reason:
```bash
# Find the request ID from the workflow output
REQUEST_UUID="<UUID from workflow>"
APPLE_ID="your.email@example.com"
TEAM_ID="ABCD1E2F3G"

xcrun notarytool info "$REQUEST_UUID" \
  --apple-id "$APPLE_ID" \
  --team-id "$TEAM_ID" \
  --password-keychain "notarytool-password"

# Look at "logFileURL" - download and check the notarization log
```

**Common Rejection Reasons:**

| Reason | Fix |
|--------|-----|
| Binary contains malware | Rescan with Xcode analyzer |
| Invalid code signature | Re-sign: `codesign --force --sign ...` |
| Timestamp missing | Add `--timestamp` to codesign |
| Wrong certificate | Verify with `security find-identity` |

### 6. Gatekeeper Still Blocking Binary

**Symptoms:**
- Binary downloaded but Gatekeeper shows warning
- "can't be opened because it's from an unidentified developer"

**Diagnosis:**
```bash
# Check signature
codesign -dvv ~/Downloads/cco

# Check Gatekeeper status
spctl -a -vvv ~/Downloads/cco

# Check for quarantine attribute
xattr -l ~/Downloads/cco | grep -i quarantine
```

**Solutions:**

#### Option A: Wait for notarization propagation
Apple's notarization servers take 10-30 minutes to propagate. Wait and try again.

#### Option B: Manually remove quarantine (for testing)
```bash
# WARNING: Only do this for binaries you trust!
xattr -d com.apple.quarantine ~/Downloads/cco
```

#### Option C: If certificate expired
```bash
# Check certificate validity
security find-certificate -c "Developer ID Application" -p | \
  openssl x509 -noout -dates

# If expired, renew from developer.apple.com and re-import
```

### 7. Signature Timestamp Verification Failed

**Symptoms:**
```
xcrun notarytool submit ...
Error: Timestamp verification failed
```

**Cause:**
- The `--timestamp` flag was not used when signing
- System clock is incorrect
- Network issues during signing

**Solution:**

Re-sign with timestamp:
```bash
# Get the binary
BINARY="/path/to/cco"

# Re-sign with timestamp
codesign --remove-signature "$BINARY"
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --timestamp \
  --options runtime \
  --force \
  "$BINARY"

# Verify
codesign -dvv "$BINARY"
```

### 8. "Invalid Input" from notarytool

**Symptoms:**
```
xcrun notarytool submit cco --apple-id ... --team-id ...
Invalid input, invalid archive (code 4)
```

**Cause:**
- Binary was not signed before notarization
- Binary is corrupt
- Wrong file format

**Solution:**

```bash
# Verify binary is properly signed
codesign --verify --verbose cco

# Verify it's a valid executable
file cco

# Extract from tarball if needed
tar -xzf cco-aarch64-apple-darwin.tar.gz

# Try notarizing extracted binary
xcrun notarytool submit cco ...
```

### 9. Workflow Timeout During Notarization

**Symptoms:**
```
xcrun notarytool submit ... --wait --timeout 600
Error: Timeout waiting for notarization
```

**Causes:**
- Apple's servers are slow
- Network connectivity issues
- Apple servers are under load

**Solution:**

Increase timeout in workflow:
```yaml
- name: Notarize binary
  run: |
    xcrun notarytool submit "cco" \
      --apple-id "$APPLE_ID" \
      --team-id "$APPLE_TEAM_ID" \
      --password-keychain "notarytool-password" \
      --wait \
      --timeout 1200  # Increased from 600 to 1200 seconds (20 minutes)
```

Or use async notarization (submit and check later):
```bash
# Submit without --wait
REQUEST_UUID=$(xcrun notarytool submit "cco" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password-keychain "notarytool-password" | grep id: | awk '{print $NF}')

# Wait a bit
sleep 30

# Check status in a loop (max 30 attempts)
for i in {1..30}; do
  STATUS=$(xcrun notarytool info "$REQUEST_UUID" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password-keychain "notarytool-password" \
    --output json | jq -r '.status')

  if [ "$STATUS" = "Accepted" ]; then
    echo "✓ Notarized successfully"
    break
  elif [ "$STATUS" = "Rejected" ]; then
    echo "❌ Notarization rejected"
    exit 1
  else
    echo "Status: $STATUS, waiting..."
    sleep 10
  fi
done
```

### 10. "No matching identities found" from codesign

**Symptoms:**
```
codesign: error: No matching identities found
```

**Cause:**
- Identity string in GitHub secret doesn't match installed certificate
- Typo in identity string
- Certificate not installed for codesigning

**Solution:**

```bash
# Get exact identity string
security find-identity -v -p codesigning | grep "Developer ID"

# Copy the ENTIRE line after the number
# Example output:
# 1) ABC123DEF456... "Developer ID Application: Acme Inc (AB12CD34EF)"
#
# GitHub secret should be:
# "Developer ID Application: Acme Inc (AB12CD34EF)"

# Update GitHub secret with exact value
```

## Verification Checklist

Before declaring setup complete, verify:

- [ ] `security find-identity -v -p codesigning` shows Developer ID certificate
- [ ] `codesign --sign "identity" --timestamp --force /path/to/binary` works
- [ ] `xcrun notarytool validate-credentials` succeeds
- [ ] Test workflow completes without errors
- [ ] Downloaded binary passes `codesign --verify`
- [ ] Downloaded binary passes `spctl -a -vvv` (Gatekeeper check)
- [ ] No "unidentified developer" warning on first run

## Advanced Debugging

### Enable detailed codesign output:
```bash
codesign -dvvvv /path/to/binary
```

### Check what entitlements would be needed:
```bash
codesign --display --entitlements - /path/to/binary
```

### Export certificate for backup:
```bash
security export-identity -p -k ~/Library/Keychains/login.keychain-db \
  -P "keychain-password" \
  /path/to/backup.p12
```

### Check certificate expiration:
```bash
security find-certificate -c "Developer ID Application" \
  -p ~/Library/Keychains/login.keychain-db | \
  openssl x509 -noout -dates
```

## Manual End-to-End Test

Complete manual test without GitHub Actions:

```bash
# 1. Create test binary
echo '#!/bin/bash' > /tmp/test_binary
chmod +x /tmp/test_binary

# 2. Sign it
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --timestamp \
  --options runtime \
  --force \
  /tmp/test_binary

# 3. Verify signature
codesign --verify --verbose /tmp/test_binary

# 4. Create tarball (like workflow does)
cd /tmp
tar -czvf test_binary.tar.gz test_binary

# 5. Extract and notarize
mkdir notary_test
cd notary_test
tar -xzf ../test_binary.tar.gz

# 6. Submit for notarization
xcrun notarytool submit test_binary \
  --apple-id "your.email@example.com" \
  --team-id "TEAM_ID" \
  --password-keychain "notarytool-password" \
  --wait

# 7. Check final status
codesign -dvv test_binary
spctl -a -vvv test_binary
```

## Support

If you're still having issues:

1. Check the workflow logs on GitHub (Actions tab)
2. Run the diagnostic command above
3. Review [Apple Code Signing Guide](https://developer.apple.com/documentation/security/code-signing-guide)
4. Check [Notarization Documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)

## Related Documentation

- [Main Setup Guide](MACOS_SIGNING_AND_NOTARIZATION.md)
- [Apple Code Signing](https://developer.apple.com/documentation/security/code-signing-guide)
- [Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Gatekeeper Overview](https://support.apple.com/en-us/HT202491)
