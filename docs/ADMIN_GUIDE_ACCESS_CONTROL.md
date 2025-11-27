# CCO Access Control - Administrator Guide

## Overview

This guide covers managing user access to CCO private binary distribution. As an administrator, you'll control who can access CCO releases, monitor usage, and maintain security.

## Table of Contents

- [Access Control Model](#access-control-model)
- [Managing Users](#managing-users)
- [Managing Groups](#managing-groups)
- [Token Policies](#token-policies)
- [Audit Logging](#audit-logging)
- [Monitoring](#monitoring)
- [Security Operations](#security-operations)
- [Troubleshooting](#troubleshooting)

## Access Control Model

### Architecture

```
┌────────────┐
│    User    │
└─────┬──────┘
      │ Member of
      ▼
┌────────────┐
│ cco-users  │ (Authentik Group)
│  or        │
│ cco-admins │
└─────┬──────┘
      │ Has permission
      ▼
┌────────────────┐
│ CCO Access     │
│ - Download     │
│ - Updates      │
│ - Releases     │
└────────────────┘
```

### Permission Levels

| Group | Permissions | Use Case |
|-------|-------------|----------|
| `cco-users` | Download releases, check updates | Standard users |
| `cco-admins` | All of above + manage releases, view analytics | Administrators |

### Access Decision Flow

```
User requests download
  │
  ├─> Is authenticated? ──NO──> Deny (401)
  │       │
  │      YES
  │       │
  ├─> In cco-users group? ──NO──> Deny (403)
  │       │
  │      YES
  │       │
  └─> Grant access (200)
```

## Managing Users

### Adding a User

#### Step 1: Create User in Authentik

1. Log into Authentik admin panel: `https://auth.visiquate.com/if/admin`
2. Navigate to **Directory** → **Users**
3. Click **Create**
4. Fill in user details:
   - **Username**: `john.doe`
   - **Name**: `John Doe`
   - **Email**: `john.doe@company.com`
   - **Groups**: Add to `cco-users`
5. Click **Create**

#### Step 2: Send Invitation

```bash
# Option 1: Authentik sends email invitation
# (Configure in Authentik email settings)

# Option 2: Manually share credentials
# Send username and temporary password to user
# Instruct them to:
#   1. Login to Authentik
#   2. Change password
#   3. Run: cco login
```

#### Step 3: Verify Access

```bash
# Ask user to test
cco login
cco update --check

# Or test with their credentials
# (Not recommended - use test accounts instead)
```

### Modifying User Access

#### Granting Access

```bash
# Via Authentik UI:
# 1. Directory → Users → [Select User]
# 2. Groups tab
# 3. Add to "cco-users"
# 4. Save

# User must re-login for changes to take effect:
# cco logout
# cco login
```

#### Revoking Access

```bash
# Via Authentik UI:
# 1. Directory → Users → [Select User]
# 2. Groups tab
# 3. Remove from "cco-users"
# 4. Save

# Optional: Revoke all tokens
# 1. Directory → Tokens
# 2. Find user's tokens
# 3. Delete tokens

# User will see 403 Forbidden on next request
```

#### Promoting to Admin

```bash
# Via Authentik UI:
# 1. Directory → Users → [Select User]
# 2. Groups tab
# 3. Add to "cco-admins"
# 4. Keep in "cco-users" as well
# 5. Save
```

### Removing a User

#### Soft Delete (Recommended)

```bash
# Via Authentik UI:
# 1. Directory → Users → [Select User]
# 2. Is Active → Uncheck
# 3. Groups → Remove all
# 4. Save

# User cannot login but data preserved
```

#### Hard Delete

```bash
# Via Authentik UI:
# 1. Directory → Users → [Select User]
# 2. Delete button (top right)
# 3. Confirm deletion

# User and all data permanently removed
```

## Managing Groups

### Group Configuration

#### cco-users Group

**Purpose**: Standard CCO users with download access

**Configuration**:
```yaml
Name: cco-users
Parent: None
Attributes:
  cco_access: true
  max_downloads_per_day: 100
  allowed_channels: ["stable", "beta"]
```

**Members**: All authorized CCO users

#### cco-admins Group

**Purpose**: CCO administrators with full access

**Configuration**:
```yaml
Name: cco-admins
Parent: cco-users (inherits permissions)
Attributes:
  cco_admin: true
  can_manage_releases: true
  can_view_analytics: true
  can_revoke_tokens: true
```

**Members**: CCO administrators only

### Viewing Group Members

#### Via Authentik UI

```bash
# 1. Directory → Groups
# 2. Click "cco-users" or "cco-admins"
# 3. Members tab shows all users
```

#### Via API

```bash
# Get group members
curl -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  https://auth.visiquate.com/api/v3/core/groups/cco-users/users/

# Returns JSON array of users
```

### Bulk Operations

#### Bulk Add Users

```python
# Python script for bulk user management
import requests

AUTHENTIK_URL = "https://auth.visiquate.com"
ADMIN_TOKEN = "your-admin-token"

users_to_add = [
    "user1@company.com",
    "user2@company.com",
    "user3@company.com"
]

for email in users_to_add:
    # Create user
    response = requests.post(
        f"{AUTHENTIK_URL}/api/v3/core/users/",
        headers={"Authorization": f"Bearer {ADMIN_TOKEN}"},
        json={
            "username": email.split("@")[0],
            "name": email.split("@")[0].replace(".", " ").title(),
            "email": email,
            "groups": ["cco-users"],
            "is_active": True
        }
    )
    print(f"Added: {email} - {response.status_code}")
```

#### Bulk Remove Users

```python
# Remove multiple users from group
for username in users_to_remove:
    # Find user
    user_response = requests.get(
        f"{AUTHENTIK_URL}/api/v3/core/users/?username={username}",
        headers={"Authorization": f"Bearer {ADMIN_TOKEN}"}
    )
    user_id = user_response.json()["results"][0]["pk"]

    # Remove from group
    requests.post(
        f"{AUTHENTIK_URL}/api/v3/core/groups/cco-users/remove_user/",
        headers={"Authorization": f"Bearer {ADMIN_TOKEN}"},
        json={"pk": user_id}
    )
```

## Token Policies

### Token Lifetimes

Configure in Authentik OIDC provider settings:

| Token Type | Default | Recommended | Maximum |
|------------|---------|-------------|---------|
| Access Token | 1 hour | 1 hour | 4 hours |
| Refresh Token | 30 days | 30 days | 90 days |
| Device Code | 10 minutes | 10 minutes | 15 minutes |

### Configuring Token Lifetimes

```bash
# Via Authentik UI:
# 1. Applications → Providers
# 2. Select "CCO CLI" provider
# 3. Token validity settings:
#    - Access code validity: 1 hour
#    - Refresh code validity: 30 days
# 4. Save
```

### Token Rotation

#### Manual Rotation

```bash
# Force all users to re-authenticate:
# 1. Applications → Providers → CCO CLI
# 2. Regenerate client secret
# 3. Update cco-api environment:
#    AUTHENTIK_CLIENT_SECRET=new_secret
# 4. Restart cco-api service
# 5. Notify users to re-login

# All existing tokens invalidated
```

#### Automatic Rotation

```python
# Scheduled token rotation script
# Run monthly via cron

import requests
from datetime import datetime

def rotate_tokens():
    # Get old tokens (>30 days)
    old_tokens = get_tokens_older_than(days=30)

    for token in old_tokens:
        # Revoke token
        revoke_token(token["id"])

        # Notify user
        send_email(
            to=token["user"]["email"],
            subject="CCO Token Expired",
            body="Please run 'cco login' to re-authenticate"
        )

# Schedule: 0 0 1 * * (first of month)
```

### Revoking Tokens

#### Revoke Single User's Tokens

```bash
# Via Authentik UI:
# 1. Directory → Tokens
# 2. Filter by user
# 3. Select all tokens
# 4. Actions → Delete

# User must re-login
```

#### Revoke All Tokens (Emergency)

```bash
# Emergency revocation (security incident)

# Method 1: Rotate client secret (recommended)
# See "Manual Rotation" above

# Method 2: Revoke all refresh tokens via API
curl -X DELETE \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  https://auth.visiquate.com/api/v3/oauth2/refresh_tokens/

# Method 3: Disable provider temporarily
# 1. Applications → Providers
# 2. CCO CLI → Disable
# 3. Re-enable after incident resolved
```

## Audit Logging

### Log Types

#### Authentication Logs

```bash
# View in Authentik
# Events → Logs
# Filter: login, logout, token_refresh

# Example entry:
{
  "timestamp": "2025-11-24T14:30:00Z",
  "event": "login_success",
  "user": "john.doe",
  "ip": "192.168.1.100",
  "client": "CCO CLI"
}
```

#### Download Logs

```bash
# View in cco-api logs
sudo journalctl -u cco-api | grep download

# Example entry:
2025-11-24 14:35:22 INFO [download] user=john.doe version=2025.11.2 platform=darwin-arm64 ip=192.168.1.100
```

#### Access Denied Logs

```bash
# View failed access attempts
sudo journalctl -u cco-api | grep -E "401|403"

# Example entry:
2025-11-24 14:40:15 WARNING [auth] 403 Forbidden user=jane.smith reason="not in cco-users group"
```

### Exporting Logs

#### Export Authentication Logs

```bash
# Export last 30 days
curl -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  "https://auth.visiquate.com/api/v3/events/events/?action=login&ordering=-created&page_size=1000" \
  > auth-logs-$(date +%Y%m).json
```

#### Export Download Logs

```bash
# Export from cco-api database
sqlite3 /home/cco-api/cco-api/data/cco.db \
  "SELECT * FROM downloads WHERE timestamp > datetime('now', '-30 days')" \
  -csv -header > downloads-$(date +%Y%m).csv
```

### Log Retention

```bash
# Configure retention in Authentik
# Admin → System → Settings
# Event retention:
#   - Login events: 365 days
#   - Other events: 180 days

# Configure retention for cco-api
# Add to systemd service:
# /etc/systemd/system/cco-api.service
[Service]
LogsDirectory=cco-api
LogsDirectoryMode=0750

# Rotate logs with logrotate
# /etc/logrotate.d/cco-api
/home/cco-api/cco-api/logs/*.log {
    daily
    rotate 90
    compress
    delaycompress
    notifempty
    create 0640 cco-api cco-api
}
```

## Monitoring

### User Activity Monitoring

#### Active Users

```bash
# Count active users (last 24 hours)
curl -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  "https://auth.visiquate.com/api/v3/events/events/?action=login&created__gte=$(date -u -d '24 hours ago' +%Y-%m-%dT%H:%M:%S)" \
  | jq '.pagination.count'
```

#### Download Statistics

```sql
-- Query cco-api database
SELECT
  DATE(timestamp) as date,
  COUNT(*) as downloads,
  COUNT(DISTINCT user) as unique_users,
  version
FROM downloads
WHERE timestamp > datetime('now', '-7 days')
GROUP BY DATE(timestamp), version
ORDER BY date DESC;
```

#### Top Users by Downloads

```sql
SELECT
  user,
  COUNT(*) as download_count,
  MAX(timestamp) as last_download
FROM downloads
WHERE timestamp > datetime('now', '-30 days')
GROUP BY user
ORDER BY download_count DESC
LIMIT 10;
```

### System Health Monitoring

#### API Availability

```bash
# Monitor API health
curl -f https://cco-api.visiquate.com/health || alert "API DOWN"

# Expected: {"status":"healthy","service":"cco-api","version":"1.0.0"}
```

#### Authentik Availability

```bash
# Monitor Authentik
curl -f https://auth.visiquate.com/-/health/live/ || alert "Authentik DOWN"
```

#### R2 Availability

```bash
# Test R2 connectivity
aws s3 ls \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  s3://cco-releases/ || alert "R2 UNAVAILABLE"
```

### Alerting

#### Set Up Alerts

```yaml
# Example: Prometheus alerting rules
groups:
  - name: cco_alerts
    rules:
      - alert: HighFailedLogins
        expr: rate(auth_failed_total[5m]) > 10
        for: 5m
        annotations:
          summary: "High rate of failed logins detected"

      - alert: APIDown
        expr: up{job="cco-api"} == 0
        for: 1m
        annotations:
          summary: "CCO API is down"

      - alert: UnauthorizedAccess
        expr: rate(http_requests_total{code="403"}[5m]) > 5
        for: 5m
        annotations:
          summary: "High rate of unauthorized access attempts"
```

## Security Operations

### Security Incident Response

#### Suspected Compromise

1. **Immediate Actions**
   ```bash
   # Revoke all tokens
   # (See "Revoke All Tokens" above)

   # Disable CCO API temporarily
   sudo systemctl stop cco-api

   # Review access logs
   sudo journalctl -u cco-api --since "1 hour ago" | grep -E "401|403"
   ```

2. **Investigation**
   ```bash
   # Identify compromised users
   # Check for suspicious activity:
   # - Logins from unusual IPs
   # - Excessive download attempts
   # - Failed authentication attempts

   # Export logs for analysis
   sudo journalctl -u cco-api --since "24 hours ago" > incident-$(date +%Y%m%d).log
   ```

3. **Remediation**
   ```bash
   # Reset affected user passwords
   # Revoke tokens
   # Update client secrets
   # Notify users
   ```

4. **Recovery**
   ```bash
   # Re-enable API
   sudo systemctl start cco-api

   # Verify health
   curl https://cco-api.visiquate.com/health

   # Monitor for continued attacks
   ```

### Regular Security Tasks

#### Weekly Tasks

```bash
# Review failed login attempts
# Review 403 errors (unauthorized access)
# Check for unusual download patterns
# Verify SSL certificate validity

# Script:
#!/bin/bash
echo "=== Weekly Security Review ==="
echo "Failed logins:"
journalctl -u authentik --since "1 week ago" | grep "login_failed" | wc -l

echo "Unauthorized access attempts:"
journalctl -u cco-api --since "1 week ago" | grep "403" | wc -l

echo "SSL expires in:"
echo | openssl s_client -servername cco-api.visiquate.com -connect cco-api.visiquate.com:443 2>/dev/null | openssl x509 -noout -dates
```

#### Monthly Tasks

```bash
# Rotate logs
# Review user access (remove inactive users)
# Update dependencies
# Test backup restoration
# Review and update security policies
```

#### Quarterly Tasks

```bash
# Full security audit
# Penetration testing
# Review and update access controls
# Disaster recovery drill
# Update documentation
```

## Troubleshooting

### Common Issues

#### User Cannot Login

**Check**:
1. Is user in `cco-users` group?
2. Is user account active?
3. Is Authentik provider enabled?
4. Are network/DNS working?

**Fix**:
```bash
# Verify user group membership
# Authentik UI: Directory → Users → [User] → Groups

# Verify provider status
# Authentik UI: Applications → Providers → CCO CLI → Check "Enabled"

# Test authentication manually
curl -X POST https://cco-api.visiquate.com/auth/device/code
```

#### User Gets 403 Forbidden

**Check**:
1. Is user in `cco-users` group?
2. Did user re-login after group change?
3. Is group sync working?

**Fix**:
```bash
# Add user to group
# Authentik UI: Directory → Groups → cco-users → Members → Add

# Have user re-login
cco logout
cco login
```

#### Tokens Not Refreshing

**Check**:
1. Is refresh token expired? (>30 days)
2. Was client secret rotated?
3. Network connectivity?

**Fix**:
```bash
# User must re-login
cco logout
cco login

# Check API logs for refresh attempts
sudo journalctl -u cco-api | grep refresh
```

### Diagnostic Commands

```bash
# Check user permissions
curl -H "Authorization: Bearer ${USER_TOKEN}" \
  https://cco-api.visiquate.com/auth/userinfo

# Test token validation
curl -H "Authorization: Bearer ${USER_TOKEN}" \
  https://cco-api.visiquate.com/releases/latest

# View recent authentication events
curl -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  "https://auth.visiquate.com/api/v3/events/events/?action=login&ordering=-created&page_size=10"
```

## Best Practices

### Access Control

- Use principle of least privilege
- Review access regularly (monthly)
- Remove inactive users promptly
- Use groups instead of individual permissions
- Document all permission changes

### Token Management

- Keep token lifetimes short (1 hour for access)
- Rotate client secrets quarterly
- Monitor token usage for anomalies
- Revoke tokens on user departure

### Monitoring

- Set up alerts for suspicious activity
- Review logs weekly
- Track download patterns
- Monitor failed authentication attempts
- Keep audit logs for 1+ year

### Security

- Enable MFA for all users
- Use strong password policies
- Keep all systems updated
- Regular security audits
- Incident response plan ready

## Reference

### API Endpoints

| Endpoint | Method | Purpose | Auth Required |
|----------|--------|---------|---------------|
| `/auth/device/code` | POST | Start device flow | No |
| `/auth/device/token` | POST | Poll for tokens | No |
| `/auth/token/refresh` | POST | Refresh tokens | No |
| `/releases/latest` | GET | Get latest release | Yes |
| `/releases/{version}` | GET | Get specific release | Yes |
| `/download/{version}/{platform}` | GET | Get download URL | Yes |
| `/health` | GET | Health check | No |

### Environment Variables

See [DEPLOYMENT_PRIVATE_DISTRIBUTION.md](./DEPLOYMENT_PRIVATE_DISTRIBUTION.md#environment-variables-reference)

### Related Documentation

- [Deployment Guide](./DEPLOYMENT_PRIVATE_DISTRIBUTION.md)
- [User Guide](./USER_GUIDE_AUTHENTICATION.md)
- [Architecture](./ARCHITECTURE_PRIVATE_DISTRIBUTION.md)
- [Migration Guide](./MIGRATION_FROM_GITHUB_RELEASES.md)

## Support

For administrator support:
- Review logs: `/home/cco-api/cco-api/logs/`
- Check Authentik documentation
- Consult security team for incidents
- Escalate to infrastructure team if needed
