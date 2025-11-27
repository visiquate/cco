# CCO Authentication and Releases API Implementation

## Summary

This implementation adds OIDC device flow authentication and migrates the auto-update system from GitHub API to an authenticated releases API at `cco-api.visiquate.com`.

## Files Created

### 1. Authentication Module (`src/auth/`)

- **`src/auth/mod.rs`** - Main auth module with public API
  - `login()` - OIDC device flow login
  - `logout()` - Clear stored tokens
  - `is_authenticated()` - Check auth status
  - `get_access_token()` - Get/refresh access token

- **`src/auth/device_flow.rs`** - OIDC device flow client
  - `DeviceFlowClient` - HTTP client for device flow
  - `start_device_flow()` - Initiate device flow
  - `poll_for_tokens()` - Poll for completion
  - `refresh_token()` - Refresh expired tokens

- **`src/auth/token_storage.rs`** - Secure token storage
  - `TokenStorage` - Manages token persistence
  - Stores tokens in `~/.config/cco/tokens.json`
  - Sets secure permissions (0o600 on Unix)
  - Handles token expiration tracking

### 2. Releases API Module

- **`src/auto_update/releases_api.rs`** - Authenticated releases API client
  - Replaces GitHub API with authenticated API
  - `fetch_latest_release()` - Get latest release
  - `fetch_release_by_version()` - Get specific version
  - Validates presigned R2 URLs
  - Enforces authentication

## Files Modified

### 1. Main CLI (`src/main.rs`)

Added two new commands:
- `cco login` - Triggers OIDC device flow
- `cco logout` - Clears stored tokens

### 2. Auto-Update Module (`src/auto_update/mod.rs`)

- Added `releases_api` module
- Updated `check_for_updates()` to use authenticated API
- Handles authentication errors gracefully
- Prompts users to login when not authenticated

### 3. Updater (`src/auto_update/updater.rs`)

- Changed import from `github::` to `releases_api::`
- Updated checksum handling (now mandatory String, not Option)
- All other logic remains the same

### 4. Library (`src/lib.rs`)

- Added `pub mod auth;` to expose authentication module

## Configuration

### API Endpoints

**Authentication API**: `https://cco-api.visiquate.com`
- `/auth/device/code` - Initiate device flow
- `/auth/device/token` - Poll for tokens
- `/auth/token/refresh` - Refresh access token

**Releases API**: `https://cco-api.visiquate.com`
- `/releases/latest` - Get latest release
- `/releases/{version}` - Get specific version
- `/download/{version}/{platform}` - Get presigned download URL

### Token Storage

- **Path**: `~/.config/cco/tokens.json`
- **Permissions**: `0o600` (owner read/write only on Unix)
- **Format**: JSON with expiration tracking

```json
{
  "access_token": "...",
  "refresh_token": "...",
  "expires_at": "2025-11-24T15:30:00Z",
  "token_type": "Bearer"
}
```

## Security Features

### 1. Authentication Flow
- OIDC device flow (no passwords in CLI)
- Automatic token refresh (5 minute buffer)
- Secure token storage with file permissions
- Clear error messages for auth failures

### 2. Download Security
- Presigned URL validation (HTTPS + R2 domain check)
- Mandatory SHA256 checksum verification
- Size limits (100MB max)
- No downloads without authentication

### 3. Error Handling
- **401 Unauthorized**: "Please run 'cco login' again"
- **403 Forbidden**: "Contact your administrator"
- **Token expired**: Automatically refreshes, retries once

## Usage Examples

### Login Flow

```bash
$ cco login
🔐 Initiating CCO login...

Please visit: https://cco-api.visiquate.com/device
And enter code: ABCD-EFGH

Waiting for authentication...

✅ Login successful!
   Tokens stored securely
```

### Logout

```bash
$ cco logout
✅ Logout successful!
   Tokens cleared
```

### Auto-Update (with authentication)

```bash
$ cco update
→ Checking for updates...
→ Update available: 2025.11.2 (current: 2025.11.1)
→ Downloading CCO 2025.11.2...
→ Installing update...
✅ Successfully updated to 2025.11.2
```

### Auto-Update (without authentication)

```bash
$ cco update
⚠️  Update check requires authentication.
   Please run 'cco login' to access updates.
```

## Implementation Notes

### 1. Backwards Compatibility

- `github.rs` module kept for reference (marked as legacy)
- All update logic migrated to `releases_api.rs`
- No breaking changes to existing CLI commands

### 2. Token Management

- Tokens refresh automatically when expired (5 min buffer)
- Refresh happens transparently during `get_access_token()`
- One retry on refresh failure, then prompts for re-login

### 3. Platform Detection

- Supports: macOS (arm64/x86_64), Linux (x86_64/aarch64), Windows (x86_64)
- Asset naming: `cco-v{version}-{platform}.{tar.gz|zip}`
- Checksum files: `checksums.sha256` in release

### 4. Error Recovery

- Network errors: User-friendly messages
- Auth errors: Clear instructions to re-login
- Download failures: Cleanup temp files, rollback support

## Testing Checklist

- [ ] `cco login` - Device flow works
- [ ] `cco logout` - Tokens cleared
- [ ] `cco update` (authenticated) - Downloads and installs
- [ ] `cco update` (not authenticated) - Prompts for login
- [ ] Token refresh - Automatic when expired
- [ ] Presigned URL validation - Rejects invalid URLs
- [ ] Checksum verification - Mandatory and enforced
- [ ] Error handling - Clear messages for all error cases

## Future Enhancements

1. **Multi-account support** - Store tokens per account
2. **Token rotation** - Periodic token rotation policy
3. **Audit logging** - Log all authentication events
4. **MFA support** - Additional authentication factors
5. **Organization policies** - Enforce update policies via API

## Dependencies Added

None! All required dependencies were already present in Cargo.toml:
- `reqwest` - HTTP client
- `serde`/`serde_json` - Serialization
- `chrono` - Timestamp handling
- `anyhow` - Error handling
- `dirs` - Config directory location

## Compilation Status

✅ **All changes compile successfully**
- Binary: `cargo check --bin cco` - Success
- Library: `cargo check --lib` - Success (pre-existing credential module errors unrelated)
- No new warnings or errors from auth/releases_api code

## File Paths Summary

```
cco/
├── src/
│   ├── auth/
│   │   ├── mod.rs              # New: Main auth module
│   │   ├── device_flow.rs      # New: OIDC device flow
│   │   └── token_storage.rs    # New: Secure token storage
│   ├── auto_update/
│   │   ├── mod.rs              # Modified: Use releases_api
│   │   ├── releases_api.rs     # New: Authenticated API client
│   │   ├── updater.rs          # Modified: Import releases_api
│   │   └── github.rs           # Legacy: Kept for reference
│   ├── main.rs                 # Modified: Add login/logout commands
│   └── lib.rs                  # Modified: Expose auth module
└── Cargo.toml                  # No changes needed
```

## API Response Formats

### Device Flow Init Response
```json
{
  "device_code": "abc123...",
  "user_code": "ABCD-EFGH",
  "verification_uri": "https://cco-api.visiquate.com/device",
  "expires_in": 600,
  "interval": 5
}
```

### Token Response
```json
{
  "access_token": "eyJ...",
  "refresh_token": "def456...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### Latest Release Response
```json
{
  "version": "2025.11.2",
  "release_notes": "## What's New\n- Feature 1\n- Feature 2",
  "platforms": [
    {
      "platform": "darwin-arm64",
      "filename": "cco-v2025.11.2-darwin-arm64.tar.gz",
      "size": 45123456,
      "checksum": "sha256:abc123..."
    }
  ]
}
```

### Download URL Response
```json
{
  "url": "https://releases.visiquate.com/cco/v2025.11.2/darwin-arm64?X-Amz-...",
  "expires_in": 300
}
```
