# CCO Authentication Module

Complete Rust auth module for CCO with OIDC device flow support and secure token storage.

## Overview

This module provides production-ready authentication for the CCO CLI, featuring:

- **OIDC Device Flow** (RFC 8628) - User-friendly authentication for CLI tools
- **Secure Token Storage** - OS keyring with encrypted file fallback
- **Automatic Token Refresh** - Transparent token renewal 5 minutes before expiry
- **Cross-Platform Support** - macOS Keychain, Windows Credential Manager, Linux Secret Service

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Public API                        │
│  login() | logout() | is_authenticated() |          │
│            get_access_token()                        │
└──────────────────┬──────────────────────────────────┘
                   │
         ┌─────────┴─────────┐
         │                   │
         ▼                   ▼
┌────────────────┐  ┌────────────────────┐
│ Device Flow    │  │  Token Storage     │
│  Client        │  │                    │
│                │  │  ┌──────────────┐  │
│  - Start flow  │  │  │ OS Keyring   │  │
│  - Poll tokens │  │  │  (Primary)   │  │
│  - Refresh     │  │  └──────────────┘  │
└────────────────┘  │  ┌──────────────┐  │
                    │  │ File Storage │  │
                    │  │  (Fallback)  │  │
                    │  └──────────────┘  │
                    └────────────────────┘
```

## Module Structure

```
cco/src/auth/
├── mod.rs              - Public API exports
├── config.rs           - OIDC configuration
├── device_flow.rs      - RFC 8628 implementation
├── token_storage.rs    - Secure token persistence
└── README.md           - This file
```

## Usage

### Basic Authentication

```rust
use cco::auth;

// Login (interactive)
auth::login().await?;

// Check authentication status
if auth::is_authenticated()? {
    println!("Authenticated!");
}

// Get access token (auto-refreshes if needed)
let token = auth::get_access_token().await?;

// Logout
auth::logout().await?;
```

### Using Token Storage Directly

```rust
use cco::auth::{TokenStorage, TokenInfo};

let storage = TokenStorage::new()?;

// Check which backend is being used
println!("Storage: {}", storage.get_backend());
// Output: "OS Keyring" or "File Storage"

// Check if valid tokens exist
if storage.has_valid_tokens()? {
    let tokens = storage.get_tokens()?;
    println!("Expires at: {}", tokens.expires_at);
}
```

### Device Flow Client

```rust
use cco::auth::DeviceFlowClient;

let client = DeviceFlowClient::new("https://cco-api.visiquate.com");

// Start device flow
let flow = client.start_device_flow().await?;
println!("Visit: {}", flow.verification_uri);
println!("Code: {}", flow.user_code);

// Poll for tokens
let tokens = client.poll_for_tokens(&flow).await?;

// Refresh tokens
let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
```

## CLI Commands

The auth module integrates with CCO's CLI:

```bash
# Login via OIDC device flow
$ cco login
🔐 Authenticating with VisiQuate...

Visit: https://auth.visiquate.com/device
Code: ABCD-EFGH

Waiting for authentication... ⏳

✅ Successfully logged in!
   Storage: OS Keyring

# Logout (clear tokens)
$ cco logout
✅ Logout successful!
   Tokens cleared

# Check authentication in scripts
$ cco auth-status
Authenticated: true
Storage: OS Keyring
Expires: 2025-11-25 14:30:00 UTC
```

## API Endpoints

The module communicates with `cco-api.visiquate.com`:

### Device Flow Endpoints

- `POST /auth/device/code` - Start device flow
  - Response: `{ device_code, user_code, verification_uri, expires_in, interval }`

- `POST /auth/device/token` - Poll for tokens
  - Body: `{ device_code }`
  - Response: `{ access_token, refresh_token, expires_in, token_type }`
  - Errors: `authorization_pending`, `slow_down`, `access_denied`, `expired_token`

- `POST /auth/token/refresh` - Refresh access token
  - Body: `{ refresh_token }`
  - Response: `{ access_token, refresh_token, expires_in, token_type }`

## Token Storage

### OS Keyring (Primary)

The module automatically uses OS keyring when available:

- **macOS**: Keychain Access
  - Service: `cco-cli`
  - Account: `cco-tokens`

- **Windows**: Credential Manager
  - Target: `cco-cli:cco-tokens`

- **Linux**: Secret Service (libsecret)
  - Collection: Default keyring
  - Attributes: `service=cco-cli, username=cco-tokens`

### File Storage (Fallback)

When keyring is unavailable, tokens are stored in:

```
~/.config/cco/tokens.json
```

File permissions are automatically set to `0o600` (owner read/write only) on Unix systems.

### Token Format

```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_at": "2025-11-24T15:30:00Z",
  "token_type": "Bearer"
}
```

## Error Handling

All public functions return `Result<T>` with descriptive errors:

```rust
use anyhow::Context;

// Graceful error handling
match auth::get_access_token().await {
    Ok(token) => println!("Token: {}", token),
    Err(e) => {
        if e.to_string().contains("Not authenticated") {
            println!("Please run 'cco login' first");
        } else {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Security Features

### Token Protection

1. **OS Keyring** - Hardware-backed encryption on supported platforms
2. **File Permissions** - Restricted to owner (0o600) on Unix
3. **No Plaintext Logging** - Tokens never logged or printed
4. **Automatic Cleanup** - Tokens cleared on logout

### Token Refresh

- Tokens automatically refreshed 5 minutes before expiry
- Refresh token used only when necessary
- Failed refresh prompts re-authentication

### TLS/HTTPS

- All API communication uses HTTPS
- Certificate validation enabled
- User-agent header: `cco/client`

## Testing

### Unit Tests

```bash
# Run all auth module tests
cargo test --lib auth

# Run specific test
cargo test token_expiry

# Unix-only tests (permissions)
cargo test secure_permissions
```

### Integration Testing

```rust
#[tokio::test]
async fn test_full_auth_flow() -> Result<()> {
    // This requires actual OIDC server
    // Run manually with: cargo test test_full_auth_flow -- --ignored

    auth::login().await?;
    assert!(auth::is_authenticated()?);

    let token = auth::get_access_token().await?;
    assert!(!token.is_empty());

    auth::logout().await?;
    assert!(!auth::is_authenticated()?);

    Ok(())
}
```

## Configuration

### Default Configuration

```rust
const AUTH_API_URL: &str = "https://cco-api.visiquate.com";
const KEYRING_SERVICE: &str = "cco-cli";
const KEYRING_USER: &str = "cco-tokens";
```

### Custom Configuration

For testing or custom deployments:

```rust
use cco::auth::OidcConfig;

let config = OidcConfig::new(
    "custom-client-id".to_string(),
    "https://auth.example.com/".to_string(),
    "https://auth.example.com/device".to_string(),
    "https://auth.example.com/token".to_string(),
);

config.validate()?;
```

## Dependencies

The auth module requires:

```toml
[dependencies]
keyring = "2.3"          # OS keyring access
dirs = "5.0"             # Cross-platform paths
reqwest = "0.11"         # HTTP client
serde = "1.0"            # Serialization
serde_json = "1.0"       # JSON support
chrono = "0.4"           # Date/time handling
anyhow = "1.0"           # Error handling
tokio = "1.35"           # Async runtime
```

## Platform Support

| Platform | Keyring Backend | File Fallback |
|----------|----------------|---------------|
| macOS    | Keychain       | ✅            |
| Windows  | Credential Mgr | ✅            |
| Linux    | Secret Service | ✅            |
| BSD      | -              | ✅            |

## Migration

If upgrading from file-only storage:

1. Existing tokens in `~/.config/cco/tokens.json` are automatically detected
2. Next login will use OS keyring
3. Old file tokens are cleared on logout
4. No manual migration needed

## Troubleshooting

### Keyring Not Available

If keyring initialization fails, the module automatically falls back to file storage:

```
WARN: Keyring unavailable, using file storage: <error>
```

This is normal on:
- Headless servers
- CI/CD environments
- Platforms without keyring support

### Permission Denied

On Unix, if token file has wrong permissions:

```
WARN: Token file has insecure permissions: 0o644. Fixing...
INFO: Token file permissions fixed to 0o600
```

The module automatically fixes permissions.

### Expired Tokens

Tokens are automatically refreshed when:
- Expired (based on `expires_at`)
- Within 5 minutes of expiry

If refresh fails, user must re-authenticate:

```bash
$ cco login
```

## Examples

### Check Auth Status Script

```rust
#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! cco = { path = "." }
//! tokio = "1"
//! ```

use cco::auth;

#[tokio::main]
async fn main() {
    match auth::is_authenticated() {
        Ok(true) => println!("✅ Authenticated"),
        Ok(false) => println!("❌ Not authenticated - run 'cco login'"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Make Authenticated API Request

```rust
use cco::auth;
use reqwest;

async fn call_api() -> Result<()> {
    let token = auth::get_access_token().await?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://cco-api.visiquate.com/releases")
        .bearer_auth(token)
        .send()
        .await?;

    println!("Status: {}", response.status());
    Ok(())
}
```

## Future Enhancements

Potential improvements:

- [ ] Support for client credentials flow (machine-to-machine)
- [ ] Token revocation endpoint
- [ ] PKCE support for enhanced security
- [ ] Token encryption for file storage
- [ ] Configurable token refresh buffer
- [ ] Token introspection endpoint
- [ ] Support for additional OIDC scopes
- [ ] Session management (multiple profiles)

## References

- [RFC 8628: OAuth 2.0 Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [Authentik Documentation](https://goauthentik.io/docs/)
- [keyring-rs Documentation](https://docs.rs/keyring/latest/keyring/)

## License

Part of the CCO project. See main repository LICENSE.
