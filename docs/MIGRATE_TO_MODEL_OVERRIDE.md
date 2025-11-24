# Migrating to Model Override

Step-by-step guide for existing users to enable model overrides in their CCO deployment.

## Overview

Model overrides are already built into CCO. Migration is simple:

1. Verify overrides configuration exists
2. Enable overrides in configuration
3. Restart CCO
4. Verify it's working
5. Monitor and optimize

**Time Required:** 5-10 minutes
**Downtime:** ~1-2 minutes (during restart)
**Risk Level:** Very Low (can be disabled instantly)

## Prerequisites

- CCO already deployed and running
- Access to `cco/config/model-overrides.toml`
- Ability to restart CCO service
- (Optional) Access to CCO logs/dashboard for verification

## Migration Steps

### Step 1: Backup Current Configuration

Before making any changes, always backup your current configuration:

```bash
# Create timestamped backup
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
cp /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml \
   /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-${TIMESTAMP}

# Verify backup was created
ls -lh /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-*

# Store backup location for reference
echo "Backup location: /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-${TIMESTAMP}"
```

### Step 2: Verify Configuration File Exists

```bash
# Check if the file exists
ls -la /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# If file doesn't exist, check the git repository
cd /Users/brent/git/cc-orchestra
git status cco/config/model-overrides.toml
```

If the file doesn't exist or is missing, see [Troubleshooting](#troubleshooting).

### Step 3: Verify Current Configuration

```bash
# View the current configuration
cat /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml | head -20

# Look for the [overrides] section and enabled setting
```

Expected output should include:
```toml
[overrides]
enabled = true  # or false if not yet enabled
rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    ...
]
```

### Step 4: Enable Overrides (If Not Already Enabled)

Edit the configuration file:

```bash
# Open in your preferred editor
nano /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# Or use sed to enable (careful, verify first):
# sed -i 's/enabled = false/enabled = true/' /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
```

Make sure the configuration looks like this:

```toml
[overrides]
enabled = true  # MUST be true

rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-4", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-3.5", "claude-haiku-4-5-20251001"],
]

[analytics]
log_overrides = true
track_statistics = true
report_format = "json"
```

Save the file (Ctrl+O in nano, then Enter).

### Step 5: Verify Configuration Syntax

Validate the TOML file before restarting:

```bash
# Try to build/start CCO - it will error if TOML is invalid
cd /Users/brent/git/cc-orchestra/cco
cargo build --release 2>&1 | grep -i "error\|toml" || echo "✓ No syntax errors"
```

Or use an online TOML validator:
- Visit: https://www.toml-lint.com/
- Copy contents of model-overrides.toml
- Check for errors

### Step 6: Stop Current CCO Instance

Stop the running CCO service:

```bash
# Option 1: If running as a service
sudo systemctl stop cco

# Option 2: If running in a container
docker stop cco

# Option 3: If running in the foreground
# Press Ctrl+C in the terminal

# Option 4: Manual kill (last resort)
pkill -f "cco run" || pkill cco

# Verify it stopped
sleep 2
curl http://localhost:3000/health 2>/dev/null || echo "✓ CCO stopped"
```

### Step 7: Start CCO with Model Overrides

Start the CCO service:

```bash
# Option 1: Local development (recommended for testing)
cd /Users/brent/git/cc-orchestra/cco
./target/release/cco run --port 3000

# Option 2: Systemd service
sudo systemctl start cco

# Option 3: Docker container
docker start cco

# Option 4: Kubernetes
kubectl rollout restart deployment/cco
```

Wait for startup to complete. You should see:
```
CCO started on http://localhost:3000
Model overrides: enabled
Override rules loaded: 3
```

### Step 8: Verify Model Overrides Are Working

In another terminal, verify the configuration:

```bash
# Check health endpoint
curl http://localhost:3000/health | jq '.overrides_enabled'

# Should output: true
```

If you get `false`, see [Troubleshooting](#troubleshooting).

### Step 9: Monitor Initial Activity

Watch the logs to confirm overrides are being applied:

```bash
# View logs (systemd)
journalctl -u cco -f | grep -i "override\|model"

# View logs (Docker)
docker logs -f cco | grep -i "override\|model"

# Or check the dashboard
# Open browser to http://localhost:3000
# Look for "Model Overrides" section
```

Wait for Claude Code to make some requests. You should see override messages like:
```
🔄 Model override: claude-sonnet-4.5-20250929 → claude-haiku-4-5-20251001
📊 Override count: 1
```

### Step 10: Check Override Statistics

Query the statistics endpoint to confirm overrides are being applied:

```bash
# Get override statistics
curl http://localhost:3000/api/overrides/stats | jq

# Response should show overrides applied:
{
  "total_overrides": 5,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": {
      "overridden_to": "claude-haiku-4-5-20251001",
      "count": 5,
      "percentage": 100
    }
  }
}
```

If override count is 0 after making requests, see [Troubleshooting](#troubleshooting).

### Step 11: Verify Cost Reduction

Check your cost metrics:

```bash
# Dashboard: http://localhost:3000
# Look for:
# - "Cost Savings" section
# - Model usage with price breakdown
# - Should show Haiku pricing instead of Sonnet

# Or via API:
curl http://localhost:3000/api/machine/stats | jq '.totalCost'
```

### Step 12: Document Your Migration

Record the migration details for your records:

```bash
# Create migration report
cat > /Users/brent/git/cc-orchestra/cco/MIGRATION_REPORT_$(date +%Y%m%d).md << EOF
# Model Override Migration Report

## Date
$(date)

## Configuration
- Enabled overrides: true
- Rules configured: 3
- Report format: json

## Verification
- Health check: PASS
- Override statistics: $(curl -s http://localhost:3000/api/overrides/stats | jq '.total_overrides')
- Dashboard access: http://localhost:3000

## Backup Location
/Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-$(date +%Y%m%d-%H%M%S)

## Notes
- Migration completed successfully
- Model overrides active and working
- Ready for production use
EOF

# View the report
cat /Users/brent/git/cc-orchestra/cco/MIGRATION_REPORT_$(date +%Y%m%d).md
```

## Verification Checklist

Complete this checklist to confirm successful migration:

- [ ] Configuration file backed up
- [ ] Configuration file syntax validated
- [ ] `enabled = true` in [overrides] section
- [ ] CCO service stopped cleanly
- [ ] CCO service started without errors
- [ ] Health check returns `overrides_enabled: true`
- [ ] Override statistics show non-zero counts
- [ ] Dashboard accessible at http://localhost:3000
- [ ] Cost metrics showing Haiku pricing
- [ ] No error messages in logs
- [ ] Migration report created
- [ ] Team notified of change

## Rollback Procedure

If you need to disable overrides, it's simple:

### Quick Disable (Without Restart)

```bash
# Edit configuration file
nano /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# Change:
# [overrides]
# enabled = true

# To:
# [overrides]
# enabled = false

# Save file
```

Then restart:

```bash
# Restart CCO
sudo systemctl restart cco

# Verify disabled
curl http://localhost:3000/health | jq '.overrides_enabled'
# Should return: false
```

### Restore from Backup

If something goes wrong, restore your backup:

```bash
# List available backups
ls -lh /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-*

# Restore specific backup
cp /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-20251115-120000 \
   /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# Restart
sudo systemctl restart cco
```

## Post-Migration Monitoring

After migration, monitor these metrics:

### Daily
- Override count (should be > 0)
- Cache hit rate (should be > 30%)
- No errors in logs

### Weekly
- Cost savings trending downward
- Cache performance stable
- Override rules still matching correctly

### Monthly
- Total overrides applied
- Cost reduction achieved
- Quality metrics (error rates, test failures)

## Troubleshooting

### CCO Won't Start

**Symptom:** CCO crashes on startup with error messages.

**Solutions:**

```bash
# 1. Check configuration syntax
nano /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
# Look for missing quotes, incorrect formatting

# 2. Restore from backup
cp /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup-TIMESTAMP \
   /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# 3. Try starting with verbose output
./target/release/cco run --port 3000 2>&1 | head -50

# 4. Check port conflict
lsof -i :3000 | grep -v PID
```

### Overrides Still Not Working

**Symptom:** Override count stays at 0, costs unchanged.

**Solutions:**

```bash
# 1. Verify overrides are enabled
grep "enabled = " /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
# Should output: enabled = true

# 2. Verify rules are configured
grep -A 5 "^rules = " /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
# Should show at least 3 rules

# 3. Check that Claude Code is using CCO
echo $ANTHROPIC_API_BASE_URL
# Should output: http://localhost:3000/v1

# 4. Verify health endpoint
curl http://localhost:3000/health | jq

# 5. Check logs for errors
journalctl -u cco -n 30 | grep -i "error"

# 6. Restart CCO
sudo systemctl restart cco

# 7. Make test request
curl http://localhost:3000/health

# 8. Check stats again
curl http://localhost:3000/api/overrides/stats
```

### Configuration Changes Not Taking Effect

**Symptom:** Changed config but no difference in behavior.

**Solutions:**

```bash
# 1. Verify file was saved
cat /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml | grep -A 3 "enabled"

# 2. Restart CCO (important!)
sudo systemctl restart cco

# 3. Wait for requests
sleep 5

# 4. Check statistics again
curl http://localhost:3000/api/overrides/stats
```

### Wrong Models Being Overridden

**Symptom:** Different models are being overridden than expected.

**Solutions:**

```bash
# 1. Check exact model names in requests
# Enable verbose logging and check what models are requested

# 2. Verify rule configuration
cat /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# 3. Compare requested model names with rules (case-sensitive!)
# Example:
#   Requested: "claude-sonnet-4.5-20250929"
#   Rule:      "claude-sonnet-4.5-20250929"
#   (must match exactly)

# 4. Check for typos in model names
# Use: curl to query what's actually being used
```

## Migration Scenarios

### Scenario 1: Single-Operator Installation

You're the only one managing CCO.

**Steps:**
1. Backup config
2. Enable overrides
3. Restart in dev terminal
4. Verify manually
5. Done!

**Time:** 5 minutes

### Scenario 2: Team Installation with Testing

You have a team and want to test before production.

**Steps:**
1. Backup production config
2. Enable overrides on test/staging instance
3. Let team test for 1 week
4. Monitor metrics and error rates
5. If good, enable on production
6. Roll back if needed

**Time:** 1 week test + 15 minutes production migration

### Scenario 3: High-Availability Deployment

You have multiple CCO instances behind a load balancer.

**Steps:**
1. Backup all configs
2. Update config on all instances
3. Restart instances one by one (rolling restart)
4. Verify each instance after restart
5. Monitor load balancer health checks
6. Confirm all instances have overrides enabled

**Time:** 30 minutes total (low downtime)

## FAQ

### Q: Is migration reversible?

A: Yes! You can disable overrides instantly by setting `enabled = false` and restarting. Or restore from backup.

### Q: What happens during restart?

A: Claude Code will be unable to reach CCO for ~30 seconds while it restarts. This is normal and expected.

### Q: Do I need to update Claude Code?

A: No! Claude Code doesn't need any changes. Just point it to CCO and overrides work automatically.

### Q: Can I test overrides without committing?

A: Yes! Keep overrides disabled in production while testing locally:
- Test: `enabled = true`
- Production: `enabled = false`

### Q: What if something breaks?

A: Restore from backup - takes 2 minutes:
```bash
cp config/model-overrides.toml.backup-TIMESTAMP config/model-overrides.toml
sudo systemctl restart cco
```

## Next Steps

1. **Monitor Performance** - Check dashboard weekly
2. **Optimize Rules** - Adjust overrides based on usage patterns
3. **Reduce Costs Further** - Consider additional overrides (Opus → Sonnet)
4. **Document Changes** - Keep team informed of cost savings

## Support

If you encounter issues:

1. Check [Troubleshooting](#troubleshooting) section
2. Review [USER_GUIDE.md](./MODEL_OVERRIDE_USER_GUIDE.md)
3. Check [OPERATOR_GUIDE.md](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)
4. Review logs: `journalctl -u cco -f`

---

**You're ready to migrate!** Start with [Step 1](#step-1-backup-current-configuration) and follow through all steps.
