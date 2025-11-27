# Testing CCO Authentication and Releases API

## Quick Test Commands

### 1. Test Login Command
```bash
cco login
```

**Expected Output:**
```
🔐 Initiating CCO login...

Please visit: https://cco-api.visiquate.com/device
And enter code: XXXX-XXXX

Waiting for authentication...

✅ Login successful!
   Tokens stored securely
```

**Verify Token Storage:**
```bash
cat ~/.config/cco/tokens.json
# Should show JSON with access_token, refresh_token, expires_at
```

**Check Permissions (Unix):**
```bash
ls -la ~/.config/cco/tokens.json
# Should show: -rw------- (0o600)
```

### 2. Test Logout Command
```bash
cco logout
```

**Expected Output:**
```
✅ Logout successful!
   Tokens cleared
```

**Verify Token Removal:**
```bash
ls ~/.config/cco/tokens.json
# Should return: No such file or directory
```

### 3. Test Update Without Authentication
```bash
# Make sure you're logged out first
cco logout
cco update
```

**Expected Output:**
```
⚠️  Update check requires authentication.
   Please run 'cco login' to access updates.
```

### 4. Test Update With Authentication
```bash
# Login first
cco login

# Then update
cco update
```

**Expected Output (if update available):**
```
→ Checking for updates...
→ Update available: 2025.11.X (current: 2025.11.Y)
→ Downloading CCO 2025.11.X...
→ Installing update...
✅ Successfully updated to 2025.11.X
```

**Expected Output (if no update):**
```
→ Checking for updates...
✅ You have the latest version
```

### 5. Test Automatic Token Refresh
```bash
# Manually expire the token by editing ~/.config/cco/tokens.json
# Change expires_at to a past date

cco update
# Should automatically refresh token and continue
```

## Manual API Testing

### Test Device Flow Initiation
```bash
curl -X POST https://cco-api.visiquate.com/auth/device/code
```

**Expected Response:**
```json
{
  "device_code": "...",
  "user_code": "XXXX-XXXX",
  "verification_uri": "https://cco-api.visiquate.com/device",
  "expires_in": 600,
  "interval": 5
}
```

### Test Latest Release (Authenticated)
```bash
# Get token from ~/.config/cco/tokens.json
TOKEN="your_access_token_here"

curl -H "Authorization: Bearer $TOKEN" \
     https://cco-api.visiquate.com/releases/latest?channel=stable
```

**Expected Response:**
```json
{
  "version": "2025.11.2",
  "release_notes": "...",
  "platforms": [
    {
      "platform": "darwin-arm64",
      "filename": "cco-v2025.11.2-darwin-arm64.tar.gz",
      "size": 45123456,
      "checksum": "sha256:..."
    }
  ]
}
```

### Test Download URL
```bash
TOKEN="your_access_token_here"

curl -H "Authorization: Bearer $TOKEN" \
     https://cco-api.visiquate.com/download/2025.11.2/darwin-arm64
```

**Expected Response:**
```json
{
  "url": "https://releases.visiquate.com/cco/...",
  "expires_in": 300
}
```

## Error Cases to Test

### 1. Authentication Failure (401)
```bash
# Use invalid token
curl -H "Authorization: Bearer invalid_token" \
     https://cco-api.visiquate.com/releases/latest
```

**Expected Response:**
```json
HTTP/1.1 401 Unauthorized
{
  "error": "invalid_token",
  "error_description": "Token is invalid or expired"
}
```

**CCO should handle:**
```
Authentication failed. Please run 'cco login' again.
```

### 2. Access Denied (403)
```bash
# Use token without proper permissions
```

**Expected Response:**
```json
HTTP/1.1 403 Forbidden
{
  "error": "access_denied",
  "error_description": "Insufficient permissions"
}
```

**CCO should handle:**
```
Access denied. Contact your administrator.
```

### 3. Invalid Presigned URL
Test that CCO rejects non-R2 URLs:
```rust
// This should fail in validate_presigned_url()
"https://evil.com/malware.tar.gz"
```

**Expected:** Security error refusing to download

### 4. Checksum Mismatch
Modify downloaded file to cause checksum failure.

**Expected:**
```
SECURITY: Checksum verification FAILED!
This indicates a corrupted download or possible MITM attack.
Update aborted for your safety.
```

## Integration Test Scenarios

### Scenario 1: Fresh Install → Login → Update
```bash
# Remove any existing tokens
rm -f ~/.config/cco/tokens.json

# Try update (should fail)
cco update
# Expected: "Update check requires authentication"

# Login
cco login
# Expected: Device flow completes successfully

# Update (should work)
cco update
# Expected: Downloads and installs update
```

### Scenario 2: Token Expiration
```bash
# Login
cco login

# Wait for token to expire OR manually edit expires_at

# Update (should auto-refresh)
cco update
# Expected: Transparently refreshes token and continues
```

### Scenario 3: Multiple Logins
```bash
# Login
cco login

# Login again (should replace token)
cco login

# Update (should use new token)
cco update
```

## Debugging

### Check Logs
```bash
# Auto-update logs
cat ~/.cco/logs/updates.log

# Look for authentication events
grep "authentication" ~/.cco/logs/updates.log
```

### Enable Debug Logging
```bash
RUST_LOG=debug cco update
```

### Inspect Token
```bash
# View stored token
cat ~/.config/cco/tokens.json | jq

# Check expiration
cat ~/.config/cco/tokens.json | jq -r '.expires_at'
```

### Test Token Refresh Manually
```bash
# Create a small test script
cat > test_refresh.sh << 'EOF'
#!/bin/bash
TOKEN=$(cat ~/.config/cco/tokens.json | jq -r '.refresh_token')
curl -X POST https://cco-api.visiquate.com/auth/token/refresh \
     -H "Content-Type: application/json" \
     -d "{\"refresh_token\": \"$TOKEN\"}"
EOF

chmod +x test_refresh.sh
./test_refresh.sh
```

## Security Validation

### 1. File Permissions
```bash
# Token file should be 0o600
stat -f "%Sp" ~/.config/cco/tokens.json  # macOS
# Should show: -rw-------

stat -c "%a" ~/.config/cco/tokens.json  # Linux
# Should show: 600
```

### 2. No Secrets in Logs
```bash
grep -i "token\|secret\|password" ~/.cco/logs/updates.log
# Should NOT show actual token values, only "Token stored" messages
```

### 3. HTTPS Only
```bash
# All API calls should use HTTPS
grep "http://" ~/.cco/logs/updates.log
# Should return nothing (all should be https://)
```

## Performance Testing

### Measure Update Time
```bash
time cco update
```

**Expected:** < 2 minutes for typical download sizes

### Measure Login Time
```bash
time cco login
```

**Expected:** < 30 seconds (depends on user interaction)

## Cleanup After Testing

```bash
# Remove test tokens
rm -f ~/.config/cco/tokens.json

# Remove test logs
rm -f ~/.cco/logs/updates.log

# Remove downloaded binaries
rm -rf /tmp/cco-update-*
```

## Known Issues / Limitations

1. **Windows ACLs**: Token file permissions not yet enforced on Windows
2. **Network Timeouts**: 30 second timeout for API calls
3. **Token Storage**: Single user only (no multi-account support yet)
4. **Refresh Failures**: Only 1 retry on refresh failure before requiring re-login

## Success Criteria

- ✅ Login completes without errors
- ✅ Tokens stored with secure permissions
- ✅ Update works when authenticated
- ✅ Update fails gracefully when not authenticated
- ✅ Token refresh happens automatically
- ✅ Clear error messages for all failure cases
- ✅ Checksum verification enforced
- ✅ Presigned URL validation works
- ✅ No credentials leaked in logs
