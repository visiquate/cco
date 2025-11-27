# Authentik OIDC Setup Guide for CCO CLI

This guide provides step-by-step instructions for configuring Authentik to support OAuth2 Device Code Flow authentication for the CCO CLI application.

## Overview

The CCO CLI uses the OAuth2 Device Code Grant (RFC 8628) to authenticate users without requiring a web browser redirect. This is ideal for CLI applications where users may not have a browser on the same device.

## Prerequisites

- Authentik instance running at `https://auth.visiquate.com`
- Admin access to Authentik
- Understanding of OAuth2/OIDC concepts
- Access to create groups and policies

## Architecture

```
┌─────────────┐      1. Device Auth Request       ┌────────────┐
│             │───────────────────────────────────>│            │
│  CCO CLI    │                                    │ Authentik  │
│             │<───────────────────────────────────│            │
└─────────────┘      2. Device & User Codes       └────────────┘
      │                                                    │
      │ 3. User visits verification URL                   │
      │ (https://auth.visiquate.com/activate)             │
      │                                                    │
      │ 4. CLI polls token endpoint                       │
      │──────────────────────────────────────────────────>│
      │                                                    │
      │<──────────────────────────────────────────────────│
      │ 5. Access token + Refresh token                   │
```

## Step 1: Create OAuth2/OIDC Provider

### Via Authentik Web UI

1. Navigate to **Applications** → **Providers**
2. Click **Create** and select **OAuth2/OpenID Provider**
3. Configure the provider:

   **Basic Settings:**
   - Name: `CCO CLI Provider`
   - Authorization flow: Select your default authentication flow (e.g., `default-authentication-flow`)

   **Protocol Settings:**
   - Client type: **Public** (no client secret required)
   - Client ID: `cco-cli`
   - Redirect URIs/Origins: Leave empty (device flow doesn't use redirects)

   **Token Settings:**
   - Access token validity: `3600` (1 hour)
   - Refresh token validity: `2592000` (30 days)
   - Scopes: `openid profile email`

   **Advanced Settings:**
   - Include claims in ID Token: ✓ (checked)
   - Issuer mode: `Per Provider`

4. Click **Create**

### Via Authentik API

```bash
curl -X POST https://auth.visiquate.com/api/v3/providers/oauth2/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CCO CLI Provider",
    "authorization_flow": "YOUR_AUTH_FLOW_UUID",
    "client_type": "public",
    "client_id": "cco-cli",
    "redirect_uris": "",
    "access_token_validity": "hours=1",
    "refresh_token_validity": "days=30",
    "include_claims_in_id_token": true,
    "issuer_mode": "per_provider",
    "sub_mode": "hashed_user_id"
  }'
```

### Via Blueprint (Automated)

Apply the provider blueprint:

```bash
curl -X POST https://auth.visiquate.com/api/v3/managed/blueprints/apply/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d @blueprints/cco-oauth2-provider.yaml
```

## Step 2: Create Application

### Via Authentik Web UI

1. Navigate to **Applications** → **Applications**
2. Click **Create**
3. Configure the application:

   - Name: `CCO CLI`
   - Slug: `cco-cli`
   - Provider: Select `CCO CLI Provider`
   - Policy engine mode: `any`
   - UI settings (optional):
     - Launch URL: Leave empty
     - Open in new tab: Unchecked

4. Click **Create**

### Via Authentik API

```bash
# First, get the provider UUID
PROVIDER_UUID=$(curl -s https://auth.visiquate.com/api/v3/providers/oauth2/?name=CCO%20CLI%20Provider \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Create the application
curl -X POST https://auth.visiquate.com/api/v3/core/applications/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"CCO CLI\",
    \"slug\": \"cco-cli\",
    \"provider\": \"$PROVIDER_UUID\",
    \"policy_engine_mode\": \"any\"
  }"
```

### Via Blueprint (Automated)

Apply the application blueprint:

```bash
curl -X POST https://auth.visiquate.com/api/v3/managed/blueprints/apply/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d @blueprints/cco-application.yaml
```

## Step 3: Create User Group

### Via Authentik Web UI

1. Navigate to **Directory** → **Groups**
2. Click **Create**
3. Configure the group:

   - Name: `cco-users`
   - Superuser privileges: Unchecked
   - Parent: None
   - Attributes: Leave empty (or add custom attributes)

4. Click **Create**

### Via Authentik API

```bash
curl -X POST https://auth.visiquate.com/api/v3/core/groups/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "cco-users",
    "is_superuser": false,
    "parent": null,
    "attributes": {}
  }'
```

## Step 4: Create Access Policy

### Via Authentik Web UI

1. Navigate to **Customization** → **Policies**
2. Click **Create** and select **Group Membership Policy**
3. Configure the policy:

   - Name: `CCO CLI - Require cco-users Group`
   - Execution logging: Optional (for debugging)
   - Group: Select `cco-users`

4. Click **Create**

### Via Authentik API

```bash
# First, get the group UUID
GROUP_UUID=$(curl -s https://auth.visiquate.com/api/v3/core/groups/?name=cco-users \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Create the policy
curl -X POST https://auth.visiquate.com/api/v3/policies/group_membership/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"CCO CLI - Require cco-users Group\",
    \"execution_logging\": false,
    \"group\": \"$GROUP_UUID\"
  }"
```

## Step 5: Bind Policy to Application

### Via Authentik Web UI

1. Navigate to **Applications** → **Applications**
2. Click on **CCO CLI** application
3. Go to **Policy Bindings** tab
4. Click **Bind existing policy**
5. Configure the binding:

   - Policy: Select `CCO CLI - Require cco-users Group`
   - Enabled: ✓ (checked)
   - Order: `0`
   - Timeout: `30` seconds

6. Click **Create**

### Via Authentik API

```bash
# Get the application UUID
APP_UUID=$(curl -s https://auth.visiquate.com/api/v3/core/applications/?slug=cco-cli \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Get the policy UUID
POLICY_UUID=$(curl -s "https://auth.visiquate.com/api/v3/policies/all/?name=CCO%20CLI%20-%20Require%20cco-users%20Group" \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Create the binding
curl -X POST https://auth.visiquate.com/api/v3/policies/bindings/ \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy\": \"$POLICY_UUID\",
    \"target\": \"$APP_UUID\",
    \"enabled\": true,
    \"order\": 0,
    \"timeout\": 30
  }"
```

## Step 6: Add Users to Group

### Via Authentik Web UI

1. Navigate to **Directory** → **Users**
2. Click on a user you want to grant access
3. Go to **Groups** tab
4. Click **Add to existing group**
5. Select `cco-users`
6. Click **Add**

### Via Authentik API

```bash
# Get user UUID
USER_UUID=$(curl -s "https://auth.visiquate.com/api/v3/core/users/?username=YOUR_USERNAME" \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Get group UUID
GROUP_UUID=$(curl -s https://auth.visiquate.com/api/v3/core/groups/?name=cco-users \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.results[0].pk')

# Add user to group
curl -X POST "https://auth.visiquate.com/api/v3/core/groups/$GROUP_UUID/add_user/" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"pk\": \"$USER_UUID\"
  }"
```

## Step 7: Configure Forward Auth (Optional)

If you need to protect CCO APIs with Authentik forward authentication:

### Create Forward Auth Provider

1. Navigate to **Applications** → **Providers**
2. Click **Create** and select **Proxy Provider**
3. Configure the provider:

   - Name: `CCO API Forward Auth`
   - Type: `Forward auth (single application)`
   - External host: `https://api.cco.visiquate.com`
   - Internal host: `http://localhost:8080` (CCO API internal address)
   - Authorization flow: Select your default authorization flow

4. Click **Create**

### Create Outpost

1. Navigate to **Applications** → **Outposts**
2. Click **Create**
3. Configure the outpost:

   - Name: `CCO Forward Auth Outpost`
   - Type: `Proxy`
   - Providers: Select `CCO API Forward Auth`
   - Configuration: Use default or customize

4. Click **Create**

### Traefik Middleware Configuration

Add to your Traefik configuration:

```yaml
http:
  middlewares:
    authentik-forward-auth:
      forwardAuth:
        address: https://auth.visiquate.com/outpost.goauthentik.io/auth/traefik
        trustForwardHeader: true
        authResponseHeaders:
          - X-authentik-username
          - X-authentik-groups
          - X-authentik-email
          - X-authentik-uid

  routers:
    cco-api:
      rule: Host(`api.cco.visiquate.com`)
      middlewares:
        - authentik-forward-auth
      service: cco-api-service
```

## Testing the Configuration

### 1. Test Device Authorization Flow

```bash
# Initiate device authorization
curl -X POST https://auth.visiquate.com/application/o/device/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=cco-cli" \
  -d "scope=openid profile email"

# Response:
# {
#   "device_code": "...",
#   "user_code": "ABCD-EFGH",
#   "verification_uri": "https://auth.visiquate.com/activate",
#   "verification_uri_complete": "https://auth.visiquate.com/activate?user_code=ABCD-EFGH",
#   "expires_in": 600,
#   "interval": 5
# }
```

### 2. User Activates Device

1. User visits: `https://auth.visiquate.com/activate`
2. Enters user code: `ABCD-EFGH`
3. Authenticates with Authentik
4. Approves the CCO CLI application

### 3. CLI Polls for Token

```bash
# Poll token endpoint (repeat every 5 seconds)
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=cco-cli" \
  -d "device_code=DEVICE_CODE_FROM_STEP_1" \
  -d "grant_type=urn:ietf:params:oauth:grant-type:device_code"

# While pending:
# {
#   "error": "authorization_pending"
# }

# After approval:
# {
#   "access_token": "...",
#   "token_type": "Bearer",
#   "expires_in": 3600,
#   "refresh_token": "...",
#   "scope": "openid profile email",
#   "id_token": "..."
# }
```

### 4. Verify Token

```bash
# Get user information
curl https://auth.visiquate.com/application/o/userinfo/ \
  -H "Authorization: Bearer ACCESS_TOKEN"

# Response:
# {
#   "sub": "user-uuid",
#   "name": "John Doe",
#   "given_name": "John",
#   "family_name": "Doe",
#   "preferred_username": "johndoe",
#   "email": "john.doe@example.com",
#   "email_verified": true,
#   "groups": ["cco-users"]
# }
```

### 5. Test Token Refresh

```bash
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=cco-cli" \
  -d "refresh_token=REFRESH_TOKEN" \
  -d "grant_type=refresh_token"

# Response:
# {
#   "access_token": "...",
#   "token_type": "Bearer",
#   "expires_in": 3600,
#   "refresh_token": "...",
#   "scope": "openid profile email"
# }
```

## Troubleshooting

### Issue: "Invalid client" error

**Cause:** Client ID mismatch or provider not configured correctly.

**Solution:**
1. Verify client ID is exactly `cco-cli`
2. Check that provider client type is set to "Public"
3. Ensure application is bound to the correct provider

### Issue: "Access denied" error

**Cause:** User not in `cco-users` group or policy not bound.

**Solution:**
1. Verify user is member of `cco-users` group
2. Check policy binding is enabled on the application
3. Review policy execution logs in Authentik

### Issue: "authorization_pending" never changes

**Cause:** User hasn't completed activation or polling interval too short.

**Solution:**
1. Ensure user visits verification URL and completes flow
2. Respect the `interval` value from device authorization response
3. Check device code hasn't expired (default: 10 minutes)

### Issue: Token validation fails

**Cause:** Token expired or JWKS verification failed.

**Solution:**
1. Check token expiration time
2. Verify JWKS endpoint is accessible
3. Ensure system clocks are synchronized (NTP)

### Debugging Tips

1. **Enable execution logging** on policies to see why access is denied
2. **Check Authentik events** (System → System Tasks → Events) for detailed logs
3. **Use Authentik's API browser** to test endpoints directly
4. **Validate JWT tokens** at https://jwt.io for debugging claims

## Security Considerations

1. **No Client Secret**: Device flow uses public clients without secrets
2. **Short Access Tokens**: 1-hour validity reduces exposure if leaked
3. **Refresh Token Rotation**: Consider enabling rotation for added security
4. **Group-Based Access**: Only users in `cco-users` group can authenticate
5. **Token Storage**: CCO CLI must securely store tokens (encrypted)
6. **HTTPS Required**: All communication must use TLS

## Next Steps

1. **Configure CCO CLI**: Update CCO configuration with Authentik endpoints
2. **Test Authentication**: Run through full device flow
3. **Monitor Usage**: Track authentication events in Authentik
4. **User Onboarding**: Add users to `cco-users` group as needed
5. **Documentation**: Share user guide for CLI authentication

## References

- [RFC 8628 - OAuth 2.0 Device Authorization Grant](https://tools.ietf.org/html/rfc8628)
- [Authentik OAuth2 Provider Documentation](https://goauthentik.io/docs/providers/oauth2/)
- [Authentik API Documentation](https://goauthentik.io/api/v3/)
- [OpenID Connect Core Specification](https://openid.net/specs/openid-connect-core-1_0.html)
