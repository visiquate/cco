# Authentik OIDC Endpoints for CCO CLI

This document provides comprehensive information about all Authentik OAuth2/OIDC endpoints used by the CCO CLI for device flow authentication.

## Base Configuration

```bash
# Environment variables for CCO CLI
export AUTHENTIK_BASE_URL="https://auth.visiquate.com"
export AUTHENTIK_CLIENT_ID="cco-cli"
export AUTHENTIK_SCOPES="openid profile email"
```

## 1. OpenID Connect Discovery

### Well-Known Configuration Endpoint

Retrieve the complete OIDC configuration automatically.

**Endpoint:**
```
GET https://auth.visiquate.com/application/o/cco-cli/.well-known/openid-configuration
```

**cURL Example:**
```bash
curl https://auth.visiquate.com/application/o/cco-cli/.well-known/openid-configuration
```

**Response Example:**
```json
{
  "issuer": "https://auth.visiquate.com/application/o/cco-cli/",
  "authorization_endpoint": "https://auth.visiquate.com/application/o/authorize/",
  "token_endpoint": "https://auth.visiquate.com/application/o/token/",
  "userinfo_endpoint": "https://auth.visiquate.com/application/o/userinfo/",
  "end_session_endpoint": "https://auth.visiquate.com/application/o/cco-cli/end-session/",
  "introspection_endpoint": "https://auth.visiquate.com/application/o/introspect/",
  "revocation_endpoint": "https://auth.visiquate.com/application/o/revoke/",
  "device_authorization_endpoint": "https://auth.visiquate.com/application/o/device/",
  "jwks_uri": "https://auth.visiquate.com/application/o/cco-cli/jwks/",
  "response_types_supported": ["code", "id_token", "id_token token", "code token", "code id_token", "code id_token token"],
  "subject_types_supported": ["public"],
  "id_token_signing_alg_values_supported": ["RS256"],
  "scopes_supported": ["openid", "email", "profile", "groups"],
  "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
  "grant_types_supported": ["authorization_code", "refresh_token", "urn:ietf:params:oauth:grant-type:device_code"],
  "code_challenge_methods_supported": ["plain", "S256"]
}
```

## 2. Device Authorization Endpoint

Initiate the device flow authentication process.

**Endpoint:**
```
POST https://auth.visiquate.com/application/o/device/
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

**Request Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `client_id` | Yes | OAuth2 client identifier (`cco-cli`) |
| `scope` | No | Space-separated scopes (default: `openid profile email`) |

**cURL Example:**
```bash
curl -X POST https://auth.visiquate.com/application/o/device/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=cco-cli" \
  -d "scope=openid profile email"
```

**Response Example:**
```json
{
  "device_code": "GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS",
  "user_code": "WDJB-MJHT",
  "verification_uri": "https://auth.visiquate.com/activate",
  "verification_uri_complete": "https://auth.visiquate.com/activate?user_code=WDJB-MJHT",
  "expires_in": 600,
  "interval": 5
}
```

**Response Fields:**
| Field | Description |
|-------|-------------|
| `device_code` | Unique device verification code (CLI uses for polling) |
| `user_code` | Short code user enters in browser (8 chars, dash-separated) |
| `verification_uri` | URL where user activates device |
| `verification_uri_complete` | Pre-filled URL with user code |
| `expires_in` | Seconds until device code expires (default: 600 = 10 minutes) |
| `interval` | Minimum polling interval in seconds (default: 5) |

**Error Responses:**
```json
{
  "error": "invalid_client",
  "error_description": "Client authentication failed"
}
```

Common errors:
- `invalid_client` - Client ID not found or misconfigured
- `invalid_scope` - Requested scope not available

## 3. Token Endpoint

Exchange device code for access token or refresh existing tokens.

**Endpoint:**
```
POST https://auth.visiquate.com/application/o/token/
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

### 3.1 Device Code Grant

Poll this endpoint to check if user has authorized the device.

**Request Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `grant_type` | Yes | Must be `urn:ietf:params:oauth:grant-type:device_code` |
| `device_code` | Yes | Device code from authorization response |
| `client_id` | Yes | OAuth2 client identifier (`cco-cli`) |

**cURL Example:**
```bash
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=urn:ietf:params:oauth:grant-type:device_code" \
  -d "device_code=GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS" \
  -d "client_id=cco-cli"
```

**Pending Response (user hasn't authorized yet):**
```json
{
  "error": "authorization_pending",
  "error_description": "User has not yet authorized the device"
}
```

**Slow Down Response (polling too fast):**
```json
{
  "error": "slow_down",
  "error_description": "Polling too frequently. Wait 5 seconds before retry."
}
```

**Success Response (user authorized):**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "pbhT3mJDzQZm3L3psH0GJ2pO0ASbJg4RoAmZKnEp",
  "scope": "openid profile email",
  "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses:**
```json
{
  "error": "expired_token",
  "error_description": "Device code expired"
}
```

Common errors:
- `authorization_pending` - User hasn't completed authorization (keep polling)
- `slow_down` - Polling too fast (increase interval by 5 seconds)
- `access_denied` - User denied authorization
- `expired_token` - Device code expired (start over)

### 3.2 Refresh Token Grant

Obtain new access token using refresh token.

**Request Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `grant_type` | Yes | Must be `refresh_token` |
| `refresh_token` | Yes | Refresh token from previous token response |
| `client_id` | Yes | OAuth2 client identifier (`cco-cli`) |

**cURL Example:**
```bash
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=pbhT3mJDzQZm3L3psH0GJ2pO0ASbJg4RoAmZKnEp" \
  -d "client_id=cco-cli"
```

**Success Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "newRefreshToken123456789",
  "scope": "openid profile email"
}
```

**Note:** Refresh token may be rotated (new refresh token in response).

## 4. UserInfo Endpoint

Retrieve authenticated user information.

**Endpoint:**
```
GET https://auth.visiquate.com/application/o/userinfo/
```

**Headers:**
```
Authorization: Bearer ACCESS_TOKEN
```

**cURL Example:**
```bash
curl https://auth.visiquate.com/application/o/userinfo/ \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Response Example:**
```json
{
  "sub": "8a8e1c9b-5d3f-4e8a-9c2d-7f6e5d4c3b2a",
  "name": "John Doe",
  "given_name": "John",
  "family_name": "Doe",
  "preferred_username": "johndoe",
  "email": "john.doe@example.com",
  "email_verified": true,
  "groups": ["cco-users", "developers"]
}
```

**Response Fields:**
| Field | Description |
|-------|-------------|
| `sub` | Unique user identifier (UUID) |
| `name` | Full name |
| `given_name` | First name |
| `family_name` | Last name |
| `preferred_username` | Username |
| `email` | Email address |
| `email_verified` | Email verification status |
| `groups` | Array of group names |

**Error Responses:**
```json
{
  "error": "invalid_token",
  "error_description": "Token is expired or invalid"
}
```

## 5. Token Introspection Endpoint

Validate and inspect access token details.

**Endpoint:**
```
POST https://auth.visiquate.com/application/o/introspect/
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

**Request Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `token` | Yes | Access token to introspect |
| `client_id` | Yes | OAuth2 client identifier (`cco-cli`) |

**cURL Example:**
```bash
curl -X POST https://auth.visiquate.com/application/o/introspect/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d "client_id=cco-cli"
```

**Active Token Response:**
```json
{
  "active": true,
  "scope": "openid profile email",
  "client_id": "cco-cli",
  "username": "johndoe",
  "token_type": "Bearer",
  "exp": 1732468800,
  "iat": 1732465200,
  "sub": "8a8e1c9b-5d3f-4e8a-9c2d-7f6e5d4c3b2a",
  "iss": "https://auth.visiquate.com/application/o/cco-cli/"
}
```

**Inactive Token Response:**
```json
{
  "active": false
}
```

## 6. Token Revocation Endpoint

Revoke access or refresh tokens.

**Endpoint:**
```
POST https://auth.visiquate.com/application/o/revoke/
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

**Request Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `token` | Yes | Token to revoke (access or refresh) |
| `client_id` | Yes | OAuth2 client identifier (`cco-cli`) |
| `token_type_hint` | No | `access_token` or `refresh_token` |

**cURL Example:**
```bash
curl -X POST https://auth.visiquate.com/application/o/revoke/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=pbhT3mJDzQZm3L3psH0GJ2pO0ASbJg4RoAmZKnEp" \
  -d "client_id=cco-cli" \
  -d "token_type_hint=refresh_token"
```

**Success Response:**
```
HTTP/1.1 200 OK
```

**Note:** Revocation endpoint returns 200 even if token is invalid/expired (RFC 7009).

## 7. JWKS (JSON Web Key Set) Endpoint

Retrieve public keys for JWT token verification.

**Endpoint:**
```
GET https://auth.visiquate.com/application/o/cco-cli/jwks/
```

**cURL Example:**
```bash
curl https://auth.visiquate.com/application/o/cco-cli/jwks/
```

**Response Example:**
```json
{
  "keys": [
    {
      "kty": "RSA",
      "use": "sig",
      "kid": "authentik-self-signed",
      "alg": "RS256",
      "n": "xGOF_hZsJa...",
      "e": "AQAB"
    }
  ]
}
```

**Response Fields:**
| Field | Description |
|-------|-------------|
| `kty` | Key type (RSA) |
| `use` | Key usage (sig = signature) |
| `kid` | Key ID (used in JWT header) |
| `alg` | Algorithm (RS256) |
| `n` | RSA modulus (base64url encoded) |
| `e` | RSA exponent (base64url encoded) |

## 8. End Session Endpoint

Log out user and invalidate session.

**Endpoint:**
```
GET https://auth.visiquate.com/application/o/cco-cli/end-session/
```

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `id_token_hint` | No | ID token from authentication |
| `post_logout_redirect_uri` | No | URL to redirect after logout |

**cURL Example:**
```bash
curl "https://auth.visiquate.com/application/o/cco-cli/end-session/?id_token_hint=eyJhbGci..."
```

**Note:** For CLI apps, typically just revoke tokens instead of using this endpoint.

## Complete Device Flow Example

Here's a complete workflow for CCO CLI authentication:

```bash
#!/bin/bash

CLIENT_ID="cco-cli"
BASE_URL="https://auth.visiquate.com"

# Step 1: Initiate device authorization
echo "Step 1: Requesting device code..."
DEVICE_RESPONSE=$(curl -s -X POST "$BASE_URL/application/o/device/" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$CLIENT_ID" \
  -d "scope=openid profile email")

DEVICE_CODE=$(echo "$DEVICE_RESPONSE" | jq -r '.device_code')
USER_CODE=$(echo "$DEVICE_RESPONSE" | jq -r '.user_code')
VERIFICATION_URI=$(echo "$DEVICE_RESPONSE" | jq -r '.verification_uri_complete')
INTERVAL=$(echo "$DEVICE_RESPONSE" | jq -r '.interval')

# Step 2: Display instructions to user
echo ""
echo "═══════════════════════════════════════════"
echo "  CCO CLI Authentication Required"
echo "═══════════════════════════════════════════"
echo ""
echo "Visit: $VERIFICATION_URI"
echo ""
echo "Or go to: https://auth.visiquate.com/activate"
echo "And enter code: $USER_CODE"
echo ""
echo "Waiting for authorization..."

# Step 3: Poll token endpoint
MAX_ATTEMPTS=120  # 10 minutes (120 * 5 seconds)
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
  sleep "$INTERVAL"

  TOKEN_RESPONSE=$(curl -s -X POST "$BASE_URL/application/o/token/" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=urn:ietf:params:oauth:grant-type:device_code" \
    -d "device_code=$DEVICE_CODE" \
    -d "client_id=$CLIENT_ID")

  ERROR=$(echo "$TOKEN_RESPONSE" | jq -r '.error // empty')

  if [ -z "$ERROR" ]; then
    # Success! We have tokens
    ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
    REFRESH_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.refresh_token')

    echo ""
    echo "✓ Authentication successful!"
    echo ""

    # Step 4: Get user info
    USER_INFO=$(curl -s "$BASE_URL/application/o/userinfo/" \
      -H "Authorization: Bearer $ACCESS_TOKEN")

    USERNAME=$(echo "$USER_INFO" | jq -r '.preferred_username')
    EMAIL=$(echo "$USER_INFO" | jq -r '.email')

    echo "Logged in as: $USERNAME ($EMAIL)"
    echo ""

    # Store tokens securely (example - use proper encryption in production)
    echo "$ACCESS_TOKEN" > ~/.cco/access_token
    echo "$REFRESH_TOKEN" > ~/.cco/refresh_token
    chmod 600 ~/.cco/access_token ~/.cco/refresh_token

    exit 0
  elif [ "$ERROR" = "authorization_pending" ]; then
    # Still waiting for user
    echo -n "."
  elif [ "$ERROR" = "slow_down" ]; then
    # Polling too fast, increase interval
    INTERVAL=$((INTERVAL + 5))
    echo " (slowing down polling)"
  else
    # Error occurred
    ERROR_DESC=$(echo "$TOKEN_RESPONSE" | jq -r '.error_description')
    echo ""
    echo "✗ Authentication failed: $ERROR"
    echo "  $ERROR_DESC"
    exit 1
  fi

  ATTEMPT=$((ATTEMPT + 1))
done

echo ""
echo "✗ Authentication timed out"
exit 1
```

## Token Validation Example

```bash
#!/bin/bash

BASE_URL="https://auth.visiquate.com"
ACCESS_TOKEN="$1"

# Validate token via introspection
INTROSPECT_RESPONSE=$(curl -s -X POST "$BASE_URL/application/o/introspect/" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=$ACCESS_TOKEN" \
  -d "client_id=cco-cli")

ACTIVE=$(echo "$INTROSPECT_RESPONSE" | jq -r '.active')

if [ "$ACTIVE" = "true" ]; then
  USERNAME=$(echo "$INTROSPECT_RESPONSE" | jq -r '.username')
  EXP=$(echo "$INTROSPECT_RESPONSE" | jq -r '.exp')

  # Calculate time until expiration
  NOW=$(date +%s)
  REMAINING=$((EXP - NOW))

  echo "✓ Token is valid"
  echo "  Username: $USERNAME"
  echo "  Expires in: $((REMAINING / 60)) minutes"

  exit 0
else
  echo "✗ Token is invalid or expired"
  exit 1
fi
```

## Error Handling Best Practices

1. **Respect polling intervals**: Always wait at least `interval` seconds between polls
2. **Handle slow_down**: Increase polling interval by 5 seconds when receiving `slow_down`
3. **Token expiration**: Check `expires_in` and refresh before expiration
4. **Graceful degradation**: Fall back to re-authentication if refresh fails
5. **Secure storage**: Encrypt tokens at rest (use OS keychain/keyring)
6. **Network errors**: Implement exponential backoff for network failures

## Security Considerations

1. **HTTPS Only**: All endpoints must use TLS (https://)
2. **Token Storage**: Never log or display full tokens
3. **Token Transmission**: Only send tokens in Authorization header or POST body
4. **JWKS Caching**: Cache JWKS response (TTL: 1 hour) to reduce requests
5. **Token Validation**: Always validate JWT signature using JWKS
6. **Clock Skew**: Allow 5-minute clock skew for token expiration checks
7. **Revocation**: Revoke tokens on logout or error conditions

## Implementation Checklist for CCO CLI

- [ ] Store base URL and client ID in configuration
- [ ] Implement device flow initiation
- [ ] Display user-friendly activation instructions
- [ ] Implement token polling with proper intervals
- [ ] Handle all error responses gracefully
- [ ] Store tokens securely (encrypted)
- [ ] Implement token refresh before expiration
- [ ] Validate tokens locally using JWKS
- [ ] Include access token in API requests
- [ ] Implement logout (token revocation)
- [ ] Add user info display command
- [ ] Handle network errors and retries

## References

- [RFC 8628 - OAuth 2.0 Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [RFC 6749 - OAuth 2.0 Authorization Framework](https://datatracker.ietf.org/doc/html/rfc6749)
- [RFC 7009 - OAuth 2.0 Token Revocation](https://datatracker.ietf.org/doc/html/rfc7009)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [Authentik OAuth2 Provider Documentation](https://goauthentik.io/docs/providers/oauth2/)
