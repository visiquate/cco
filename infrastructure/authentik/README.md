# Authentik OIDC Configuration for CCO CLI

This directory contains complete Authentik configuration for OAuth2 Device Code Flow authentication with the CCO CLI application.

## Quick Start

### Option 1: Automated Setup (Blueprints)

Apply both blueprints to configure Authentik automatically:

```bash
# 1. Create OAuth2 provider
curl -X POST https://auth.visiquate.com/api/v3/managed/blueprints/apply/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/yaml" \
  --data-binary @blueprints/cco-oauth2-provider.yaml

# 2. Create application and policies
curl -X POST https://auth.visiquate.com/api/v3/managed/blueprints/apply/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/yaml" \
  --data-binary @blueprints/cco-application.yaml

# 3. Add users to cco-users group (replace USERNAME)
USER_UUID=$(curl -s "https://auth.visiquate.com/api/v3/core/users/?username=USERNAME" \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

GROUP_UUID=$(curl -s https://auth.visiquate.com/api/v3/core/groups/?name=cco-users \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

curl -X POST "https://auth.visiquate.com/api/v3/core/groups/$GROUP_UUID/add_user/" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"pk\": \"$USER_UUID\"}"
```

### Option 2: Manual Setup (Web UI)

Follow the step-by-step instructions in **[setup-guide.md](setup-guide.md)**.

## Files in this Directory

| File | Purpose |
|------|---------|
| **README.md** | This file - quick start and overview |
| **[setup-guide.md](setup-guide.md)** | Complete step-by-step configuration guide |
| **[endpoints.md](endpoints.md)** | All OIDC endpoints with examples and usage |
| **blueprints/cco-oauth2-provider.yaml** | OAuth2 provider blueprint (automated setup) |
| **blueprints/cco-application.yaml** | Application and policy blueprint (automated setup) |

## What Gets Configured

After applying this configuration, you'll have:

1. **OAuth2/OIDC Provider** (`CCO CLI Provider`)
   - Client ID: `cco-cli`
   - Client Type: Public (no secret)
   - Device Code Flow enabled
   - Token validity: 1hr access, 30d refresh
   - Scopes: `openid`, `profile`, `email`, `groups`

2. **Application** (`CCO CLI`)
   - Slug: `cco-cli`
   - Linked to OAuth2 provider
   - Group-based access control

3. **User Group** (`cco-users`)
   - Controls who can use CCO CLI
   - Users must be manually added

4. **Access Policy** (`CCO CLI - Require cco-users Group`)
   - Ensures only group members can authenticate
   - Bound to CCO CLI application

## Testing the Configuration

### 1. Test Device Flow

```bash
# Initiate device authorization
curl -X POST https://auth.visiquate.com/application/o/device/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=cco-cli" \
  -d "scope=openid profile email"
```

**Expected Response:**
```json
{
  "device_code": "...",
  "user_code": "ABCD-EFGH",
  "verification_uri": "https://auth.visiquate.com/activate",
  "expires_in": 600,
  "interval": 5
}
```

### 2. User Activates Device

1. Visit: `https://auth.visiquate.com/activate`
2. Enter user code from above
3. Authenticate with Authentik
4. Approve CCO CLI application

### 3. Poll for Token

```bash
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=urn:ietf:params:oauth:grant-type:device_code" \
  -d "device_code=DEVICE_CODE_FROM_STEP_1" \
  -d "client_id=cco-cli"
```

**Success Response:**
```json
{
  "access_token": "...",
  "refresh_token": "...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "openid profile email"
}
```

### 4. Get User Info

```bash
curl https://auth.visiquate.com/application/o/userinfo/ \
  -H "Authorization: Bearer ACCESS_TOKEN"
```

**Expected Response:**
```json
{
  "sub": "user-uuid",
  "name": "John Doe",
  "preferred_username": "johndoe",
  "email": "john.doe@example.com",
  "groups": ["cco-users"]
}
```

## Key Endpoints

All endpoints are documented in **[endpoints.md](endpoints.md)**. Here are the most important:

| Endpoint | Purpose |
|----------|---------|
| `POST /application/o/device/` | Initiate device flow |
| `POST /application/o/token/` | Exchange code for tokens / refresh |
| `GET /application/o/userinfo/` | Get authenticated user info |
| `POST /application/o/introspect/` | Validate tokens |
| `POST /application/o/revoke/` | Revoke tokens |
| `GET /application/o/cco-cli/jwks/` | Get public keys for JWT validation |
| `GET /application/o/cco-cli/.well-known/openid-configuration` | OIDC discovery |

## User Management

### Add User to CCO Access Group

```bash
# Get user UUID
USER_UUID=$(curl -s "https://auth.visiquate.com/api/v3/core/users/?username=USERNAME" \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Get group UUID
GROUP_UUID=$(curl -s https://auth.visiquate.com/api/v3/core/groups/?name=cco-users \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Add user to group
curl -X POST "https://auth.visiquate.com/api/v3/core/groups/$GROUP_UUID/add_user/" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"pk\": \"$USER_UUID\"}"
```

### Remove User from CCO Access Group

```bash
curl -X POST "https://auth.visiquate.com/api/v3/core/groups/$GROUP_UUID/remove_user/" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"pk\": \"$USER_UUID\"}"
```

## Troubleshooting

### "Invalid client" error

**Cause:** Client ID mismatch or provider misconfigured.

**Solution:**
1. Verify client ID is exactly `cco-cli`
2. Check provider client type is "Public"
3. Ensure application is linked to provider

### "Access denied" error

**Cause:** User not in `cco-users` group.

**Solution:**
1. Add user to `cco-users` group (see User Management above)
2. Verify policy is bound to application
3. Check policy execution logs in Authentik

### "authorization_pending" never changes

**Cause:** User hasn't completed activation.

**Solution:**
1. Ensure user visits activation URL
2. Check device code hasn't expired (10 minutes)
3. Verify user has permission to access application

### Token validation fails

**Cause:** Token expired or JWKS verification failed.

**Solution:**
1. Check token hasn't expired
2. Verify JWKS endpoint is accessible
3. Ensure system clocks are synchronized

## Security Notes

1. **No Client Secret**: Device flow uses public clients without secrets
2. **Short-Lived Tokens**: Access tokens expire in 1 hour
3. **Group-Based Access**: Only users in `cco-users` group can authenticate
4. **Token Storage**: CCO CLI must store tokens securely (encrypted)
5. **HTTPS Required**: All communication must use TLS
6. **Refresh Token Rotation**: Consider enabling for production

## Integration with CCO CLI

The CCO CLI should:

1. **Store configuration**:
   ```rust
   pub struct AuthConfig {
       pub base_url: String,          // "https://auth.visiquate.com"
       pub client_id: String,          // "cco-cli"
       pub scopes: Vec<String>,        // ["openid", "profile", "email"]
   }
   ```

2. **Implement device flow**:
   - Call `/application/o/device/` to start flow
   - Display user code and verification URL
   - Poll `/application/o/token/` every 5 seconds
   - Handle `authorization_pending`, `slow_down`, and success

3. **Store tokens securely**:
   - Use OS keychain/keyring for token storage
   - Encrypt tokens at rest
   - Never log or display full tokens

4. **Refresh tokens proactively**:
   - Check `expires_in` from token response
   - Refresh 5 minutes before expiration
   - Fall back to re-authentication if refresh fails

5. **Include tokens in API calls**:
   ```rust
   headers.insert(
       "Authorization",
       format!("Bearer {}", access_token)
   );
   ```

6. **Validate tokens locally**:
   - Fetch JWKS from `/application/o/cco-cli/jwks/`
   - Cache JWKS (TTL: 1 hour)
   - Verify JWT signature using JWKS

## Next Steps

1. **Apply blueprints** or follow manual setup guide
2. **Add users** to `cco-users` group
3. **Test device flow** using curl examples
4. **Integrate with CCO CLI** using endpoints documentation
5. **Monitor usage** in Authentik events dashboard

## Support

For issues or questions:
- Check **[setup-guide.md](setup-guide.md)** troubleshooting section
- Review **[endpoints.md](endpoints.md)** for API details
- Consult [Authentik OAuth2 Provider Documentation](https://goauthentik.io/docs/providers/oauth2/)
- Check [RFC 8628 - Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
