# CCO Authentication Quick Reference

## CLI Commands

### Login
```bash
cco login
```

**What it does:**
1. Initiates OIDC device flow with cco-api.visiquate.com
2. Displays verification URL and code
3. Waits for user to authenticate in browser
4. Stores tokens securely in ~/.config/cco/tokens.json

**Example Output:**
```
🔐 Initiating CCO login...

Please visit: https://auth.visiquate.com/device
And enter code: ABCD-1234

Waiting for authentication...

✅ Login successful!
   Tokens stored securely
```

### Logout
```bash
cco logout
```

**What it does:**
1. Removes stored tokens from ~/.config/cco/tokens.json
2. Clears authentication state

**Example Output:**
```
✅ Logout successful!
   Tokens cleared
```

## Token Storage

### Location
```
~/.config/cco/tokens.json
```

### Permissions (Unix)
```
-rw------- (0o600) - Owner read/write only
```

### Format
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "refresh_token_here",
  "expires_at": "2025-11-24T14:30:00Z",
  "token_type": "Bearer"
}
```

### Security Features
- File-based storage with secure permissions
- Keyring support (when available)
- Automatic cleanup on logout
- Token expiry tracking

## Auto-Update Flow

### With Authentication
```bash
# Update checks now require authentication
cco update --check
```

**Flow:**
1. Check if authenticated (`is_authenticated()`)
2. Get access token (`get_access_token()` - auto-refreshes if needed)
3. Call `/releases/latest` with Bearer token
4. Get presigned URL from `/download/{version}/{platform}`
5. Download from R2 presigned URL
6. Verify SHA256 checksum
7. Install binary atomically

### Without Authentication
```bash
$ cco update --check
⚠️  Update check requires authentication.
   Please run 'cco login' to access updates.
```

## API Integration

### Base URL
```
https://cco-api.visiquate.com
```

### Endpoints

#### 1. Latest Release
```
GET /releases/latest?channel=stable
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "version": "2025.11.2",
  "release_notes": "Bug fixes and improvements",
  "platforms": [
    {
      "platform": "darwin-arm64",
      "filename": "cco-v2025.11.2-darwin-arm64.tar.gz",
      "size": 12345678,
      "checksum": "sha256_hash_here"
    }
  ]
}
```

#### 2. Download URL
```
GET /download/{version}/{platform}
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "url": "https://account.r2.cloudflarestorage.com/bucket/file.tar.gz?X-Amz-Signature=...",
  "expires_in": 3600
}
```

## Error Handling

### Authentication Errors

**401 Unauthorized**
```
Authentication failed. Your session may have expired.
Please run 'cco login' again.
```

**403 Forbidden**
```
Access denied. Your account does not have permission to access releases.
Contact your administrator.
```

**Not Authenticated**
```
Not authenticated. Please run 'cco login' first to access releases.
```

### Token Refresh

Tokens automatically refresh when:
- Current token expires within 5 minutes
- API returns 401 on first attempt

**Refresh Flow:**
1. Detect token expiry
2. Call refresh endpoint with refresh_token
3. Store new tokens
4. Retry original request

## Code Locations

### CLI Commands
- **File**: `/Users/brent/git/cc-orchestra/cco/src/main.rs`
- **Lines**: 169-172 (commands), 830-842 (handlers)

### Auth Module
- **Login/Logout**: `/Users/brent/git/cc-orchestra/cco/src/auth/mod.rs`
- **Device Flow**: `/Users/brent/git/cc-orchestra/cco/src/auth/device_flow.rs`
- **Token Storage**: `/Users/brent/git/cc-orchestra/cco/src/auth/token_storage.rs`
- **Config**: `/Users/brent/git/cc-orchestra/cco/src/auth/config.rs`

### Auto-Update
- **Main Module**: `/Users/brent/git/cc-orchestra/cco/src/auto_update/mod.rs`
- **Releases API**: `/Users/brent/git/cc-orchestra/cco/src/auto_update/releases_api.rs`
- **Updater**: `/Users/brent/git/cc-orchestra/cco/src/auto_update/updater.rs`

## Security Features

### Authentication
- ✅ OIDC device flow (no passwords in CLI)
- ✅ Bearer token authentication
- ✅ Automatic token refresh
- ✅ Secure token storage (0o600 permissions)

### Downloads
- ✅ Presigned URLs (time-limited, signed)
- ✅ HTTPS only
- ✅ Domain whitelist (R2 only)
- ✅ SHA256 checksum verification (mandatory)
- ✅ Size limits (100MB max)

### Installation
- ✅ Secure temp directories (0o700)
- ✅ Atomic binary replacement
- ✅ Rollback on failure
- ✅ Binary verification before/after install

## Testing

### Verify Installation
```bash
cd /Users/brent/git/cc-orchestra/cco
./verify_auth_implementation.sh
```

### Manual Testing
```bash
# 1. Test logout (safe, doesn't require API)
cco logout

# 2. Test login (requires API server)
cco login

# 3. Check token storage
ls -la ~/.config/cco/tokens.json
cat ~/.config/cco/tokens.json | jq .

# 4. Test update check
cco update --check

# 5. Test actual update
cco update
```

## Troubleshooting

### "Not authenticated" Error
```bash
# Solution: Login first
cco login
```

### "Token file has insecure permissions"
```bash
# Solution: Fix permissions
chmod 600 ~/.config/cco/tokens.json
```

### "Authentication failed" Error
```bash
# Solution: Re-login (token may be expired or invalid)
cco logout
cco login
```

### Update Check Fails
```bash
# Check if authenticated
ls -la ~/.config/cco/tokens.json

# If file doesn't exist, login first
cco login

# Try update again
cco update --check
```

## Configuration

### Update Settings
Location: `~/.config/cco/config.toml`

```toml
[updates]
enabled = true           # Enable auto-updates
auto_install = true      # Auto-install without prompt
check_interval = "daily" # daily, weekly, never
channel = "stable"       # stable, beta
```

### Environment Variables
```bash
# Disable auto-updates
export CCO_AUTO_UPDATE=false

# Override channel
export CCO_AUTO_UPDATE_CHANNEL=beta

# Override check interval
export CCO_AUTO_UPDATE_INTERVAL=weekly
```

## Development

### Build
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
```

### Test
```bash
# Run verification script
./verify_auth_implementation.sh

# Run unit tests
cargo test --release

# Test specific module
cargo test --release auth::
cargo test --release auto_update::
```

### Debug
```bash
# Enable debug logging
export RUST_LOG=debug
cco login

# Check token file
cat ~/.config/cco/tokens.json | jq .

# Check API connectivity
curl https://cco-api.visiquate.com/health
```

## Support

### Documentation
- Full implementation: `AUTH_CLI_IMPLEMENTATION_SUMMARY.md`
- Verification script: `verify_auth_implementation.sh`

### Key Contacts
- API Endpoint: https://cco-api.visiquate.com
- Release Storage: Cloudflare R2
- Token Storage: ~/.config/cco/tokens.json

### Common Issues

1. **"Not authenticated"** → Run `cco login`
2. **"Token expired"** → Automatic refresh (or re-login)
3. **"Access denied"** → Contact administrator
4. **"Checksum failed"** → Corrupted download, retry
5. **"Download too large"** → Security limit (100MB max)
