# Migration from GitHub Releases to Private Distribution

## Overview

This guide covers migrating CCO from public GitHub Releases to a private authenticated distribution system. The migration provides better access control, security, and usage analytics while maintaining a smooth user experience.

## What's Changing

### Old System (GitHub Releases)

```
User → GitHub API (unauthenticated) → GitHub CDN → Download binary
```

**Characteristics**:
- Public access (anyone can download)
- No authentication required
- No access control
- No download tracking
- Rate limited by GitHub
- Dependent on GitHub availability

### New System (Private Distribution)

```
User → Login (OIDC) → cco-api.visiquate.com → R2 presigned URL → Download binary
```

**Characteristics**:
- Private access (authentication required)
- OIDC device flow authentication
- Group-based access control
- Full download tracking and analytics
- Self-hosted (no GitHub dependency)
- Cloudflare R2 for scalable storage

## Migration Timeline

### Phase 1: Preparation (Week 1-2)

**Goals**: Set up infrastructure, test with admins

**Tasks**:
- [ ] Deploy cco-api.visiquate.com
- [ ] Configure Authentik OIDC provider
- [ ] Set up Cloudflare R2 bucket
- [ ] Upload latest releases to R2
- [ ] Test authentication flow
- [ ] Test download flow
- [ ] Document procedures

**Testing**: Internal team only

### Phase 2: Parallel Operation (Week 3-4)

**Goals**: Both systems running, gradual user migration

**Tasks**:
- [ ] Release CCO with auth support
- [ ] Keep GitHub releases active
- [ ] Monitor both systems
- [ ] Gather user feedback
- [ ] Fix issues quickly
- [ ] Update documentation

**CCO Behavior**:
```rust
// Tries new system first, falls back to GitHub
match fetch_from_releases_api().await {
    Ok(release) => use_authenticated_download(release),
    Err(_) => {
        warn!("Private API unavailable, falling back to GitHub");
        fetch_from_github_releases().await
    }
}
```

**User Impact**: Minimal - automatic fallback

### Phase 3: Full Migration (Week 5-6)

**Goals**: All users migrated, GitHub deprecated

**Tasks**:
- [ ] Verify all active users authenticated
- [ ] Remove GitHub fallback code
- [ ] Deprecate GitHub releases (keep archived)
- [ ] Monitor for issues
- [ ] Support users with problems
- [ ] Final documentation updates

**User Impact**: Must authenticate to continue updates

### Phase 4: Cleanup (Week 7+)

**Goals**: Remove old system completely

**Tasks**:
- [ ] Archive GitHub releases (read-only)
- [ ] Remove old code paths
- [ ] Update all documentation
- [ ] Final user communication
- [ ] Lessons learned review

## User Migration Steps

### For End Users

#### Pre-Migration (Old System)

```bash
# Users currently do this:
cco update

# Which uses GitHub API automatically
# No authentication needed
```

#### During Migration (Parallel)

```bash
# Users need to login once:
cco login
# Opens browser for authentication

# Then updates work normally:
cco update
# Uses new authenticated API

# If new system fails, falls back to GitHub automatically
```

#### Post-Migration (New System Only)

```bash
# Users must be authenticated:
cco update
# If not logged in: "Please run 'cco login' first"

# Login required once per 30 days (token lifetime)
```

### For Administrators

#### Step 1: Create User Accounts

**Option A: Bulk import from employee directory**

```python
# bulk_import_users.py
import requests
import csv

AUTHENTIK_URL = "https://auth.visiquate.com"
ADMIN_TOKEN = "your-admin-token"

# Read employee list
with open('employees.csv') as f:
    reader = csv.DictReader(f)
    for row in reader:
        # Create user in Authentik
        response = requests.post(
            f"{AUTHENTIK_URL}/api/v3/core/users/",
            headers={"Authorization": f"Bearer {ADMIN_TOKEN}"},
            json={
                "username": row['email'].split('@')[0],
                "name": row['name'],
                "email": row['email'],
                "groups": ["cco-users"],
                "is_active": True
            }
        )
        print(f"Created: {row['email']} - Status: {response.status_code}")
```

**Option B: Manual user creation**

See [ADMIN_GUIDE_ACCESS_CONTROL.md](./ADMIN_GUIDE_ACCESS_CONTROL.md#adding-a-user)

#### Step 2: Notify Users

**Email template**:

```
Subject: Action Required: CCO Authentication Setup

Hi team,

We're upgrading CCO to use secure authentication for better security
and access control.

ACTION REQUIRED:
1. Update CCO: Run `cco update`
2. Login: Run `cco login` and follow the prompts
3. Test: Run `cco update` again to verify

This one-time setup takes less than 2 minutes.

Your username is: [USERNAME]
Your initial password is: [TEMP_PASSWORD]
Please change it after first login.

Questions? Contact: it-support@company.com

Timeline:
- Nov 24: New system available
- Dec 1: Both systems running
- Dec 15: Old system deprecated
- Jan 1: Authentication required

Thanks!
IT Team
```

#### Step 3: Monitor Migration Progress

```bash
# Track authenticated users
./scripts/migration-stats.sh

# Output:
# Total users: 100
# Authenticated: 75 (75%)
# Not yet migrated: 25 (25%)
# Failed logins: 3
```

```python
# migration-stats.py
import requests
from datetime import datetime, timedelta

# Get all users
users_response = requests.get(
    f"{AUTHENTIK_URL}/api/v3/core/users/",
    headers={"Authorization": f"Bearer {ADMIN_TOKEN}"},
    params={"groups": "cco-users"}
)
total_users = users_response.json()["pagination"]["count"]

# Get logins in last 7 days
week_ago = (datetime.now() - timedelta(days=7)).isoformat()
logins_response = requests.get(
    f"{AUTHENTIK_URL}/api/v3/events/events/",
    headers={"Authorization": f"Bearer {ADMIN_TOKEN}"},
    params={"action": "login", "created__gte": week_ago}
)
authenticated_users = len(set([e["user"]["username"] for e in logins_response.json()["results"]]))

print(f"Migration Progress:")
print(f"  Total users: {total_users}")
print(f"  Authenticated (last 7 days): {authenticated_users} ({authenticated_users/total_users*100:.1f}%)")
print(f"  Not yet migrated: {total_users - authenticated_users}")
```

#### Step 4: Support Users

Common issues and solutions:

**User: "I can't find the email with my credentials"**
- Resend credentials via Authentik
- Or create temporary password and share securely

**User: "Login isn't working"**
- Verify user in cco-users group
- Check Authentik logs for errors
- Test with their credentials

**User: "I'm traveling and can't access email/MFA"**
- Provide backup codes
- Temporarily disable MFA
- Or wait until they return

## Backward Compatibility

### CCO Version Support

| CCO Version | GitHub API | Private API | Notes |
|-------------|------------|-------------|-------|
| < 2025.11.1 | ✅ Yes | ❌ No | Old versions, no auth support |
| 2025.11.1 - 2025.12.31 | ✅ Yes (fallback) | ✅ Yes | Parallel operation |
| >= 2026.1.1 | ❌ No | ✅ Yes | Private API only |

### Code Changes

The CCO binary contains both systems during transition:

```rust
// src/auto_update/mod.rs

pub async fn check_for_updates() -> Result<Option<ReleaseInfo>> {
    // Try new authenticated API first
    match check_private_api().await {
        Ok(release) => return Ok(Some(release)),
        Err(e) => {
            tracing::warn!("Private API failed: {}, trying GitHub fallback", e);

            // During migration: Fall back to GitHub
            #[cfg(feature = "github-fallback")]
            {
                return check_github_api().await;
            }

            // After migration: No fallback
            #[cfg(not(feature = "github-fallback"))]
            {
                return Err(e);
            }
        }
    }
}
```

Build flags control fallback:

```bash
# During migration (with fallback)
cargo build --release --features github-fallback

# After migration (no fallback)
cargo build --release
```

### Deprecation Warnings

Users on old CCO versions will see:

```bash
$ cco update
⚠️  DEPRECATION NOTICE:
   GitHub-based updates will be removed on January 1, 2026.
   Please update to CCO >= 2025.11.1 to continue receiving updates.

   To update now:
   1. Download: https://cco-api.visiquate.com/latest
   2. Run: cco login
   3. Run: cco update

   Questions? Contact: support@company.com
```

## Rollback Plan

If migration encounters critical issues:

### Step 1: Identify Issue

```bash
# Check error rates
journalctl -u cco-api | grep ERROR | wc -l

# Check authentication success rate
# If < 90%, consider rollback
```

### Step 2: Decision Point

**Rollback if**:
- Authentication success rate < 90%
- Critical bugs in auth flow
- Authentik/R2 availability < 99%
- User complaints exceed threshold

**Don't rollback if**:
- Minor issues (can be fixed quickly)
- Only affecting small % of users
- User education issue (not technical)

### Step 3: Execute Rollback

```bash
# 1. Revert CCO to GitHub-only version
cd /path/to/cco/releases
git checkout v2025.10.1  # Last GitHub-only version

# 2. Build and upload to GitHub
cargo build --release
gh release create v2025.10.1-rollback \
  --title "Rollback Release" \
  --notes "Temporary rollback to GitHub releases" \
  target/release/cco-*

# 3. Notify users to downgrade
# Email: "Please run: cco update --force v2025.10.1-rollback"

# 4. Keep cco-api running for users who already migrated
# They can continue using the new system

# 5. Fix issues and try again later
```

### Step 4: Root Cause Analysis

```markdown
# Rollback Report Template

## Issue Summary
[Brief description of the problem]

## Impact
- Users affected: [number/%]
- Duration: [start time] to [end time]
- Severity: [Critical/High/Medium/Low]

## Timeline
- [Time]: Issue first reported
- [Time]: Investigation started
- [Time]: Rollback decision made
- [Time]: Rollback completed
- [Time]: Service restored

## Root Cause
[Technical explanation of what went wrong]

## Resolution
[Immediate fix applied]

## Prevention
- [ ] Action 1
- [ ] Action 2
- [ ] Action 3

## Lessons Learned
[What we learned from this incident]
```

## Testing Strategy

### Pre-Deployment Testing

#### Unit Tests

```bash
# Test authentication flow
cargo test test_device_flow
cargo test test_token_refresh
cargo test test_token_storage

# Test releases API
cargo test test_fetch_latest_release
cargo test test_presigned_url_validation
cargo test test_checksum_verification
```

#### Integration Tests

```bash
# Test full flow end-to-end
./tests/integration/test_auth_flow.sh

# Expected:
# ✓ Device code generation
# ✓ Browser authorization
# ✓ Token polling
# ✓ Token storage
# ✓ Release fetching
# ✓ Download with presigned URL
# ✓ Checksum verification
# ✓ Binary installation
```

#### Load Tests

```bash
# Simulate 100 concurrent users
./tests/load/test_concurrent_logins.sh 100

# Expected:
# - All logins succeed
# - No rate limiting errors
# - Average response time < 2s
# - No server errors

# Simulate 1000 concurrent downloads
./tests/load/test_concurrent_downloads.sh 1000

# Expected:
# - All downloads succeed
# - R2 bandwidth sufficient
# - No timeout errors
```

### Beta Testing

#### Phase 1: Internal Team (Week 1)

```bash
# 5-10 technical users
# Test all scenarios:
# - Fresh login
# - Token refresh
# - Token expiration
# - Logout/re-login
# - Multiple devices
# - Network issues
# - Error cases
```

#### Phase 2: Pilot Group (Week 2-3)

```bash
# 20-50 diverse users
# Mix of:
# - Different platforms (macOS, Linux, Windows)
# - Different network environments
# - Different use patterns
# - Technical and non-technical users

# Collect feedback:
# - Survey: https://forms.company.com/cco-pilot
# - Slack channel: #cco-pilot-feedback
# - Office hours: Tue/Thu 2-3pm
```

### Post-Deployment Monitoring

```bash
# Monitor for 7 days after launch

# Key metrics:
# - Authentication success rate (target: >95%)
# - Download success rate (target: >99%)
# - Average login time (target: <60s)
# - Average download time (target: <5min)
# - Support tickets (target: <10/day)
# - Error rate (target: <1%)

# Alert thresholds:
# - Auth success rate drops below 90%
# - Error rate exceeds 5%
# - Support tickets exceed 50/day
# - Any critical errors
```

## Communication Plan

### Pre-Migration (2 weeks before)

**Announcement Email**:
```
Subject: Upcoming CCO Authentication Changes - Action Required

We're improving CCO security with authentication.

WHAT'S CHANGING:
- CCO will require login to download updates
- One-time setup (2 minutes)
- Better security and access control

WHEN:
- Available: December 1
- Required: January 1

WHAT YOU NEED TO DO:
1. Update CCO: `cco update` (after Dec 1)
2. Login: `cco login` (follow prompts)
3. Done! Updates work normally

More info: https://docs.company.com/cco-auth
Questions: #cco-support on Slack

Thanks!
DevOps Team
```

### During Migration (weekly updates)

**Status Email**:
```
Subject: CCO Migration Update - Week [N]

Status: [X]% of users migrated

COMPLETED:
✅ [milestone 1]
✅ [milestone 2]

IN PROGRESS:
🔄 [current work]

UPCOMING:
📅 [next milestones]

NEED HELP?
- Slack: #cco-support
- Docs: https://docs.company.com/cco
- Office hours: Tue/Thu 2-3pm

Migration progress: https://dashboard.company.com/cco-migration
```

### Post-Migration (1 week after)

**Completion Email**:
```
Subject: CCO Migration Complete - Thank You!

The CCO authentication migration is complete!

RESULTS:
✅ [N] users migrated successfully
✅ [N] downloads with new system
✅ Zero downtime
✅ [Positive feedback metric]

WHAT'S NEXT:
- Old system removed: January 1
- New features enabled:
  - Usage analytics
  - Faster updates
  - Better security

Thanks for your patience!

Questions? Contact: devops@company.com
```

## Success Metrics

### Technical Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Authentication success rate | >95% | [TBD] | 🟢 |
| Download success rate | >99% | [TBD] | 🟢 |
| Average login time | <60s | [TBD] | 🟢 |
| System uptime | >99.9% | [TBD] | 🟢 |
| Error rate | <1% | [TBD] | 🟢 |

### User Experience Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| User migration rate | >90% in 4 weeks | [TBD] | 🟢 |
| Support tickets | <10/day | [TBD] | 🟢 |
| User satisfaction | >4/5 | [TBD] | 🟢 |
| Time to first login | <5min | [TBD] | 🟢 |

### Business Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Unauthorized access | 0 | [TBD] | 🟢 |
| Security incidents | 0 | [TBD] | 🟢 |
| Cost per download | <$0.01 | [TBD] | 🟢 |
| Audit compliance | 100% | [TBD] | 🟢 |

## FAQs

### For Users

**Q: Why do I need to authenticate now?**
A: To improve security and ensure only authorized users can download CCO.

**Q: How often do I need to login?**
A: Once per month (tokens last 30 days).

**Q: What if I forget my password?**
A: Use password reset at https://auth.visiquate.com or contact your administrator.

**Q: Can I use CCO offline?**
A: Yes, but you need to be online to login and download updates.

**Q: What happens to my current CCO installation?**
A: It continues to work. You only need to authenticate for updates.

### For Administrators

**Q: Can we delay the migration?**
A: Yes, but security benefits are lost. Recommend proceeding on schedule.

**Q: What if a user refuses to authenticate?**
A: Their current CCO continues working but cannot receive updates.

**Q: How do we handle contractors/external users?**
A: Create guest accounts or temporary access in Authentik.

**Q: What's the blast radius if something goes wrong?**
A: Limited - users fall back to GitHub during parallel operation.

**Q: Can we revert the migration?**
A: Yes, see [Rollback Plan](#rollback-plan) above.

## Related Documentation

- [Deployment Guide](./DEPLOYMENT_PRIVATE_DISTRIBUTION.md) - Server setup
- [User Guide](./USER_GUIDE_AUTHENTICATION.md) - End user instructions
- [Admin Guide](./ADMIN_GUIDE_ACCESS_CONTROL.md) - Access management
- [Architecture](./ARCHITECTURE_PRIVATE_DISTRIBUTION.md) - Technical details

## Timeline Summary

```
Week 1-2:  Infrastructure setup and testing
Week 3-4:  Parallel operation begins
Week 5-6:  Full migration to new system
Week 7+:   Cleanup and optimization

Day 0:     Deploy cco-api.visiquate.com
Day 7:     Begin beta testing
Day 14:    Release CCO with auth support
Day 28:    80% of users migrated
Day 42:    Remove GitHub fallback
Day 60:    Archive old system
```

## Conclusion

This migration improves CCO security while maintaining excellent user experience. With careful planning, parallel operation, and thorough testing, users will experience minimal disruption.

Key success factors:
- Clear communication
- Gradual rollout
- Fallback mechanisms
- Strong support
- Comprehensive monitoring

For questions or concerns about the migration, contact the DevOps team at devops@company.com.
