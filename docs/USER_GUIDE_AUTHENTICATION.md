# CCO Authentication - User Guide

## Overview

CCO uses secure OIDC device flow authentication to protect binary downloads and ensure only authorized users can access releases. This guide covers everything you need to know about logging in, managing your session, and troubleshooting authentication issues.

## Quick Start

```bash
# Login to CCO
cco login

# Check authentication status
cco whoami  # (if implemented)

# Use CCO normally - authentication is automatic
cco update

# Logout when done
cco logout
```

## Detailed Authentication Flow

### Step 1: Initial Login

When you run `cco login`, the CLI initiates a secure device flow:

```bash
$ cco login
🔐 Initiating CCO login...

Please visit: https://auth.visiquate.com/device
And enter code: ABCD-EFGH

Waiting for authentication...
```

### Step 2: Browser Authentication

1. **Open the URL** in your web browser
   - Go to: `https://auth.visiquate.com/device`
   - This opens Authentik's device authorization page

2. **Enter the code**
   - Type the code shown in your terminal: `ABCD-EFGH`
   - Code is case-insensitive
   - Code expires in 10 minutes

3. **Login to Authentik**
   - Enter your username and password
   - Complete MFA if enabled
   - Your organization's SSO may be used

4. **Authorize CCO**
   - Review requested permissions:
     - Read your profile
     - Access your groups
   - Click "Authorize"

### Step 3: Confirmation

Back in your terminal, you'll see:

```bash
✅ Login successful!
   Tokens stored securely
```

Your authentication tokens are now saved in:
- **Location**: `~/.config/cco/tokens.json`
- **Permissions**: `0o600` (owner read/write only on Unix)
- **Contents**: Access token, refresh token, expiration time

## Using Authenticated Commands

Once logged in, CCO automatically uses your stored tokens for all operations:

### Checking for Updates

```bash
$ cco update
→ Checking for updates...
→ Authenticated as: user@example.com
→ Update available: 2025.11.2 (current: 2025.11.1)
→ Downloading CCO 2025.11.2...
→ Installing update...
✅ Successfully updated to 2025.11.2
```

### Auto-Update (Daemon)

If you have auto-update enabled, CCO checks for updates in the background:

```bash
# Check auto-update status
cco daemon status

# Enable auto-update
cco config set auto_update.enabled true

# Background updates happen automatically
# You'll be notified when a new version is installed
```

## Token Management

### Token Lifecycle

1. **Creation**: When you login
2. **Storage**: Saved locally with secure permissions
3. **Usage**: Automatically sent with API requests
4. **Refresh**: Auto-renewed before expiration
5. **Expiration**: Access tokens last 1 hour, refresh tokens last 30 days
6. **Deletion**: When you logout or manually clear

### Automatic Token Refresh

CCO automatically refreshes your access token when:
- It has expired
- It will expire within 5 minutes
- An API request receives a 401 Unauthorized error

This happens transparently - you won't notice any delays.

### Manual Token Check

To verify your authentication status:

```bash
# Check if logged in
if cco whoami &>/dev/null; then
    echo "Logged in"
else
    echo "Not logged in"
    cco login
fi
```

### Token Location

Your tokens are stored in:

**macOS/Linux**: `~/.config/cco/tokens.json`
**Windows**: `%APPDATA%\cco\tokens.json`

**Token file format**:
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "def456...",
  "expires_at": "2025-11-24T15:30:00Z",
  "token_type": "Bearer"
}
```

**Security**: Never share this file or its contents. It provides full access to your CCO account.

## Logout

### When to Logout

Logout when:
- You're done using CCO for the day
- You're on a shared machine
- You want to switch accounts
- You suspect your token was compromised

### Logout Process

```bash
$ cco logout
✅ Logout successful!
   Tokens cleared
```

This permanently deletes your stored tokens. You'll need to login again to use authenticated features.

### Forced Logout

If logout fails or tokens are corrupted:

```bash
# Manually remove tokens
rm -f ~/.config/cco/tokens.json

# Verify removal
ls ~/.config/cco/tokens.json
# Should return: No such file or directory
```

## Multi-Device Usage

### Same Account, Multiple Devices

You can login on multiple devices simultaneously:

```bash
# On laptop
laptop$ cco login
✅ Login successful!

# On desktop (different token)
desktop$ cco login
✅ Login successful!
```

Each device stores its own tokens independently. Logging out on one device doesn't affect others.

### Switching Accounts

To use a different account on the same device:

```bash
# Logout current account
cco logout

# Login with new account
cco login
# Use different credentials in browser
```

**Note**: Only one account can be active at a time per device.

## Troubleshooting

### Problem: "Not authenticated" Error

**Symptom**:
```bash
$ cco update
⚠️  Update check requires authentication.
   Please run 'cco login' to access updates.
```

**Solution**:
```bash
cco login
```

**If login fails**, see "Login Won't Complete" below.

### Problem: "Authentication failed" or 401 Error

**Symptom**:
```bash
$ cco update
❌ Authentication failed. Your session may have expired.
   Please run 'cco login' again.
```

**Causes**:
- Access token expired (> 1 hour old)
- Refresh token expired (> 30 days old)
- Token revoked by administrator
- Network connectivity issues

**Solution**:
```bash
# Try refresh first (automatic)
cco update

# If that fails, re-login
cco logout
cco login
```

### Problem: Login Won't Complete

**Symptom**:
```bash
$ cco login
🔐 Initiating CCO login...
Please visit: https://auth.visiquate.com/device
And enter code: ABCD-EFGH
Waiting for authentication...
# Hangs here forever
```

**Troubleshooting Steps**:

1. **Check the code**
   - Verify you entered the exact code shown
   - Code is case-insensitive but must be exact
   - Code expires in 10 minutes

2. **Check browser authorization**
   - Ensure you clicked "Authorize" in browser
   - Check for error messages on the web page

3. **Check network**
   ```bash
   curl https://cco-api.visiquate.com/health
   # Should return: {"status":"healthy"}
   ```

4. **Try again**
   ```bash
   # Cancel current attempt (Ctrl+C)
   ^C

   # Start fresh login
   cco login
   ```

### Problem: "Access Denied" or 403 Error

**Symptom**:
```bash
$ cco update
❌ Access denied. Your account does not have permission to access releases.
   Contact your administrator.
```

**Causes**:
- Your account is not in the `cco-users` group
- Your account was removed from authorized users
- Group membership cached (takes ~5 minutes to update)

**Solution**:
1. Contact your CCO administrator
2. Request access to `cco-users` group
3. Wait 5 minutes for group sync
4. Re-login:
   ```bash
   cco logout
   cco login
   ```

### Problem: Device Code Expired

**Symptom**:
```bash
Error: Device code expired. Please try again.
```

**Cause**: Took too long to authorize (> 10 minutes)

**Solution**:
```bash
# Start fresh login
cco login

# Complete within 10 minutes this time
```

### Problem: Cannot Delete Tokens File

**Symptom** (macOS/Linux):
```bash
$ rm ~/.config/cco/tokens.json
rm: cannot remove '~/.config/cco/tokens.json': Permission denied
```

**Solution**:
```bash
# Check file permissions
ls -la ~/.config/cco/tokens.json

# Fix permissions if needed
chmod 600 ~/.config/cco/tokens.json

# Try delete again
rm ~/.config/cco/tokens.json
```

### Problem: Tokens File Corrupted

**Symptom**:
```bash
$ cco update
Error: Failed to read tokens: invalid JSON
```

**Solution**:
```bash
# Remove corrupted file
rm -f ~/.config/cco/tokens.json

# Login fresh
cco login
```

### Problem: MFA/2FA Issues

If your organization requires multi-factor authentication:

1. **Check MFA device**
   - Ensure device is available (phone, authenticator app)
   - Check battery/connectivity

2. **Backup codes**
   - Use backup code if device unavailable
   - Contact admin for account recovery

3. **Wrong MFA code**
   - Wait for next code cycle (30-60 seconds)
   - Verify time synchronization on device

## Security Best Practices

### Do's

✅ **Logout on shared machines**
```bash
cco logout
```

✅ **Use strong passwords**
- 12+ characters
- Mix of letters, numbers, symbols
- Unique password for CCO

✅ **Enable MFA if available**
- Adds extra security layer
- Protects against password theft

✅ **Keep CCO updated**
```bash
cco update
```

✅ **Monitor login activity**
- Check your Authentik account for unexpected logins
- Report suspicious activity immediately

### Don'ts

❌ **Never share your tokens file**
- Contains full account access
- Treat like a password

❌ **Don't commit tokens to git**
```bash
# Verify .gitignore includes
echo "tokens.json" >> ~/.config/cco/.gitignore
```

❌ **Don't use same password elsewhere**
- Use unique password for CCO/Authentik
- Use password manager

❌ **Don't leave sessions active**
- Logout when done
- Especially on public computers

❌ **Don't ignore security warnings**
- Update when prompted
- Report suspicious behavior

## FAQ

### How long do tokens last?

- **Access token**: 1 hour
- **Refresh token**: 30 days
- Tokens auto-refresh transparently

### Do I need to login every time I use CCO?

No. Once logged in, your session lasts until:
- You logout manually
- Refresh token expires (30 days)
- Admin revokes your access

### Can I use CCO offline?

Partially:
- **Offline**: Can use already-downloaded features
- **Online required**: Login, updates, downloading releases

### What data does CCO store?

Locally:
- Access token (encrypted in memory during use)
- Refresh token (stored in `tokens.json`)
- Token expiration time
- Token type (Bearer)

CCO does **not** store:
- Your password
- Your MFA codes
- Your personal information

### Can my administrator see what I download?

Administrators can see:
- When you logged in
- When you downloaded releases
- Which versions you downloaded

They **cannot** see:
- Your password
- Your local CCO usage
- Your project files

### What happens if I forget to logout?

- Your session remains active
- Tokens auto-refresh for 30 days
- After 30 days, you'll need to re-login
- Use `cco logout` to end session immediately

### How do I report a security issue?

1. **Immediate**: Logout and change password
   ```bash
   cco logout
   ```

2. **Contact administrator**
   - Report suspected compromise
   - Request token revocation

3. **Re-secure account**
   - Change password
   - Review account activity
   - Re-login with new credentials

## Command Reference

| Command | Description |
|---------|-------------|
| `cco login` | Login via OIDC device flow |
| `cco logout` | Logout and clear tokens |
| `cco whoami` | Show current user (if implemented) |
| `cco update` | Check/install updates (requires auth) |
| `cco daemon status` | Check daemon status |

## Environment Variables

### CI/CD Authentication

For automated environments (CI/CD pipelines), use token override:

```bash
# Set token from secrets manager
export CCO_ACCESS_TOKEN="eyJhbGc..."

# CCO will use this instead of stored tokens
cco update
```

**Warning**: Only use in secure environments. Never commit tokens to git.

## Getting Help

### Check Logs

```bash
# View CCO logs
cco daemon logs

# Enable debug mode
RUST_LOG=debug cco login
```

### Contact Support

If authentication issues persist:

1. **Check documentation**: [TROUBLESHOOTING.md](../cco/TROUBLESHOOTING.md)
2. **Contact your administrator**: For access/permission issues
3. **Report bugs**: GitHub issues (without sensitive data)

### Additional Resources

- [Deployment Guide](./DEPLOYMENT_PRIVATE_DISTRIBUTION.md) - For administrators
- [Admin Guide](./ADMIN_GUIDE_ACCESS_CONTROL.md) - Access control details
- [Architecture](./ARCHITECTURE_PRIVATE_DISTRIBUTION.md) - Technical details
- [Migration Guide](./MIGRATION_FROM_GITHUB_RELEASES.md) - Upgrading from old system

## Summary

**Remember**:
1. Login once: `cco login`
2. Use CCO normally - authentication is automatic
3. Tokens refresh automatically
4. Logout on shared machines: `cco logout`
5. Re-login if you see "Not authenticated" errors

Authentication keeps CCO secure while staying out of your way. Most users will login once per month and forget about it!
