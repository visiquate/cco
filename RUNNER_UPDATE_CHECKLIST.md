# Runner Update Execution Checklist

**Date:** 2025-11-20
**Target:** Update 9 runners (containers 200-208) with optimal configuration
**Estimated Time:** 4 hours
**Status:** READY TO EXECUTE

---

## Pre-Flight Checklist

### Before You Begin
- [ ] Read executive summary (`RUNNER_CONFIGURATION_EXEC_SUMMARY.md`)
- [ ] Read full design doc (optional but recommended)
- [ ] Have SSH access to Proxmox host (192.168.9.220)
- [ ] Have GitHub CLI authenticated
- [ ] Maintenance window scheduled (or confirm low usage period)
- [ ] Team notified of runner updates

### Verify Current State
```bash
# Check all runners are online
gh api orgs/langstons/actions/runners --jq '.runners[] | {name, status}'

# Check container status on Proxmox
ssh root@192.168.9.220 "for i in {200..208}; do echo -n 'Container \$i: '; pct status \$i; done"

# Check for running workflows
gh run list -R langstons/claude-orchestra --status in_progress
gh run list -R langstons/growthiq --status in_progress
```

---

## Step 1: Create Safety Snapshots (5 minutes)

```bash
# SSH to Proxmox
ssh root@192.168.9.220

# Create snapshots of all containers
for CTID in {200..208}; do
  echo "Creating snapshot for container $CTID..."
  pct snapshot $CTID pre-ubuntu-update-$(date +%Y%m%d-%H%M) \
    --description "Before Ubuntu package updates"
done

# Verify snapshots
for CTID in {200..208}; do
  echo "Snapshots for container $CTID:"
  pct listsnapshot $CTID
done
```

**Expected:** 9 snapshots created, ~500MB each

- [ ] Snapshots created successfully
- [ ] Snapshot names verified

---

## Step 2: Update Test Runner (runner-01, Container 200) (30 minutes)

```bash
# Still on Proxmox host
CTID=200

echo "=== Updating Test Runner (Container $CTID) ==="

# Update packages
pct exec $CTID -- bash -c '
  apt-get update
  apt-get upgrade -y

  # Install essential packages
  apt-get install -y \
    build-essential pkg-config git curl wget ca-certificates \
    gnupg software-properties-common sudo openssh-server \
    libssl-dev libssl3 \
    protobuf-compiler libprotobuf-dev \
    libxml2-dev libxmlsec1-dev libxmlsec1-openssl \
    libpq-dev

  # Add Python PPA if not present
  if ! grep -q "deadsnakes" /etc/apt/sources.list.d/*; then
    add-apt-repository ppa:deadsnakes/ppa -y
    apt-get update
  fi

  # Install Python versions
  apt-get install -y \
    python3.10 python3.10-dev python3.10-venv \
    python3.11 python3.11-dev python3.11-venv \
    python3.12 python3.12-dev python3.12-venv \
    python3-pip python3-venv

  # Cleanup
  apt-get autoremove -y
  apt-get clean

  # Verify installations
  echo ""
  echo "=== Verification ==="
  gcc --version | head -1
  make --version | head -1
  pkg-config --version
  protoc --version
  python3.10 --version
  python3.11 --version
  python3.12 --version
  dpkg -l | grep -E "libssl-dev|libxml2-dev|libxmlsec1-dev|libpq-dev"
  echo ""
  echo "✅ Container $CTID updated successfully"
'
```

**Expected Output:**
```
gcc (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0
GNU Make 4.3
pkg-config 0.29.2
libprotoc 3.12.4
Python 3.10.X
Python 3.11.X
Python 3.12.X
ii  libssl-dev  3.0.X
ii  libxml2-dev 2.9.X
ii  libxmlsec1-dev 1.2.X
ii  libpq-dev   14.X
✅ Container 200 updated successfully
```

- [ ] Update completed without errors
- [ ] All packages verified installed
- [ ] Versions match expected

---

## Step 3: Test runner-01 with Real Workflows (30-45 minutes)

### Test 1: Rust Build (claude-orchestra)
```bash
# From your local machine
gh workflow run test.yml -R langstons/claude-orchestra --ref main

# Wait and monitor
gh run watch -R langstons/claude-orchestra

# Check result
gh run list -R langstons/claude-orchestra -L 1
```

**Expected:** ✅ All tests pass, build succeeds

- [ ] Rust build successful
- [ ] No missing package errors
- [ ] Build time acceptable

### Test 2: Python Multi-Version (growthiq)
```bash
gh workflow run build-and-deploy.yml -R langstons/growthiq --ref main

# Wait and monitor
gh run watch -R langstons/growthiq

# Check result
gh run list -R langstons/growthiq -L 1
```

**Expected:** ✅ Tests pass on Python 3.10, 3.11, 3.12

- [ ] Python 3.10 tests pass
- [ ] Python 3.11 tests pass
- [ ] Python 3.12 tests pass
- [ ] No libxml2/xmlsec errors

### Test 3: Node.js Build (claude-analytics or slack-broker)
```bash
gh workflow run test.yml -R langstons/claude-analytics --ref main

# Wait and monitor
gh run watch -R langstons/claude-analytics
```

**Expected:** ✅ Node.js tests pass

- [ ] Node.js setup successful
- [ ] npm install works
- [ ] Tests pass

### Decision Point
**If ANY test fails:**
1. Investigate logs: `gh run view <run-id> -R <repo> --log`
2. Check for missing packages
3. Fix issue on runner-01
4. Re-run tests
5. DO NOT proceed to other runners until runner-01 is 100% working

**If ALL tests pass:**
✅ Proceed to Step 4

---

## Step 4: Rolling Update of Remaining Runners (2 hours)

Update runners one at a time to maintain capacity:

```bash
# SSH to Proxmox
ssh root@192.168.9.220

# Update runners 2-9 (containers 201-208)
for CTID in {201..208}; do
  RUNNER_NUM=$(printf "%02d" $((CTID - 199)))
  echo ""
  echo "=========================================="
  echo "=== Updating runner-$RUNNER_NUM (Container $CTID) ==="
  echo "=========================================="

  # Update packages
  pct exec $CTID -- bash -c '
    apt-get update
    apt-get upgrade -y

    # Install essential packages
    apt-get install -y \
      build-essential pkg-config git curl wget ca-certificates \
      gnupg software-properties-common sudo openssh-server \
      libssl-dev libssl3 \
      protobuf-compiler libprotobuf-dev \
      libxml2-dev libxmlsec1-dev libxmlsec1-openssl \
      libpq-dev

    # Add Python PPA if not present
    if ! grep -q "deadsnakes" /etc/apt/sources.list.d/*; then
      add-apt-repository ppa:deadsnakes/ppa -y
      apt-get update
    fi

    # Install Python versions
    apt-get install -y \
      python3.10 python3.10-dev python3.10-venv \
      python3.11 python3.11-dev python3.11-venv \
      python3.12 python3.12-dev python3.12-venv \
      python3-pip python3-venv

    # Cleanup
    apt-get autoremove -y
    apt-get clean

    # Quick verification
    echo ""
    echo "=== Verification ==="
    gcc --version | head -1
    python3.11 --version
    protoc --version
  '

  if [ $? -eq 0 ]; then
    echo "✅ Container $CTID updated successfully"
  else
    echo "❌ Container $CTID update FAILED"
    echo "Pausing for investigation. Press Enter to continue or Ctrl+C to abort."
    read
  fi

  # Wait between runners (allows workflows to redistribute)
  if [ $CTID -lt 208 ]; then
    echo "Waiting 60 seconds before next runner..."
    sleep 60
  fi
done

echo ""
echo "=========================================="
echo "✅ All runners updated successfully!"
echo "=========================================="
```

### Track Progress
- [ ] runner-02 (201) updated ✅
- [ ] runner-03 (202) updated ✅
- [ ] runner-04 (203) updated ✅
- [ ] runner-05 (204) updated ✅
- [ ] runner-06 (205) updated ✅
- [ ] runner-07 (206) updated ✅
- [ ] runner-08 (207) updated ✅
- [ ] runner-09 (208) updated ✅

---

## Step 5: Post-Update Verification (15 minutes)

### Check All Runners Online
```bash
# From your local machine
gh api orgs/langstons/actions/runners --jq '.runners[] | {name, status, busy}'
```

**Expected:** All 9 runners showing `"status": "online"`

- [ ] All 9 runners online
- [ ] None showing errors

### Check Container Health
```bash
# On Proxmox
ssh root@192.168.9.220

for CTID in {200..208}; do
  RUNNER_NUM=$(printf "%02d" $((CTID - 199)))
  STATUS=$(pct status $CTID)
  echo "runner-$RUNNER_NUM (Container $CTID): $STATUS"
done
```

**Expected:** All showing "status: running"

- [ ] All containers running
- [ ] No errors in container logs

### Verify Package Installation (Spot Check)
```bash
# Check a few runners randomly
for CTID in 200 204 208; do
  echo "=== Container $CTID ==="
  pct exec $CTID -- bash -c '
    gcc --version | head -1
    python3.10 --version
    python3.11 --version
    python3.12 --version
    protoc --version
  '
  echo ""
done
```

- [ ] All packages present on spot-checked runners

---

## Step 6: Run Full Test Suite (30 minutes)

Trigger workflows across multiple repos to ensure everything works:

```bash
# Rust projects
gh workflow run test.yml -R langstons/claude-orchestra --ref main
gh workflow run build.yml -R langstons/cco --ref main

# Python projects
gh workflow run build-and-deploy.yml -R langstons/growthiq --ref main

# JavaScript projects
gh workflow run test.yml -R langstons/claude-analytics --ref main

# Monitor all
echo "Waiting for workflows to complete (5-10 minutes)..."
sleep 300

# Check results
echo ""
echo "=== Workflow Results ==="
gh run list -R langstons/claude-orchestra -L 1
gh run list -R langstons/cco -L 1
gh run list -R langstons/growthiq -L 1
gh run list -R langstons/claude-analytics -L 1
```

**Expected:** All workflows show ✅ success

### Test Results
- [ ] claude-orchestra build: ✅ PASS
- [ ] cco build: ✅ PASS
- [ ] growthiq tests: ✅ PASS
- [ ] claude-analytics tests: ✅ PASS

**If any fail:**
1. Check logs: `gh run view <run-id> --log`
2. Identify missing package or issue
3. Fix on affected runner
4. Re-run workflow
5. Update this checklist with resolution

---

## Step 7: Monitoring & Documentation (1 week)

### Day 1 (Implementation Day)
- [ ] All runners updated ✅
- [ ] All test workflows passing ✅
- [ ] Team notified of completion
- [ ] Document any issues encountered

### Day 2-3 (Active Monitoring)
```bash
# Check runner health twice daily
gh api orgs/langstons/actions/runners --jq '.runners[] | {name, status}'

# Check for workflow failures
gh run list -R langstons/claude-orchestra --status failure -L 5
gh run list -R langstons/growthiq --status failure -L 5
gh run list -R langstons/claude-analytics --status failure -L 5
```

- [ ] Day 2 AM check: All runners healthy
- [ ] Day 2 PM check: All runners healthy
- [ ] Day 3 AM check: All runners healthy
- [ ] Day 3 PM check: All runners healthy

### Day 4-7 (Passive Monitoring)
- [ ] Check daily for any unusual failures
- [ ] Monitor team feedback
- [ ] Address any edge cases

### End of Week 1
- [ ] Zero package-related failures
- [ ] Team reports no issues
- [ ] Update runner documentation
- [ ] Delete Proxmox snapshots (if all good)
- [ ] Mark project complete ✅

---

## Rollback Procedure (If Needed)

**If critical issues occur and you need to rollback:**

```bash
# Rollback single runner
ssh root@192.168.9.220
CTID=<problem-container-id>

# Stop container
pct stop $CTID

# List snapshots
pct listsnapshot $CTID

# Rollback to pre-update snapshot
pct rollback $CTID pre-ubuntu-update-<timestamp>

# Start container
pct start $CTID

# Verify runner comes back online
gh api orgs/langstons/actions/runners --jq '.runners[] | select(.name=="runner-XX")'
```

**Rollback Decision Criteria:**
- Critical workflows failing consistently
- Missing packages cannot be easily fixed
- Team cannot work due to runner issues
- Security vulnerability discovered

**Rollback Timeline:**
- Identify issue: 5 minutes
- Rollback runner: 2 minutes
- Verify: 3 minutes
- Total: 10 minutes per runner

- [ ] Rollback plan understood
- [ ] Snapshots available for rollback

---

## Success Criteria

Project is successful when:

- ✅ All 9 runners online and healthy
- ✅ All critical workflows passing
- ✅ Zero package missing errors
- ✅ Rust builds compile successfully
- ✅ Python 3.10/3.11/3.12 tests pass
- ✅ Node.js workflows run without issues
- ✅ Build times comparable or better
- ✅ Team reports no degradation
- ✅ 1 week monitoring shows stability

---

## Notes & Issues

### Issues Encountered (Document here)

**Issue #1:** [Description]
- Container: [ID]
- Error: [Message]
- Solution: [What fixed it]
- Time: [How long to resolve]

**Issue #2:** [Description]
- Container: [ID]
- Error: [Message]
- Solution: [What fixed it]
- Time: [How long to resolve]

### Lessons Learned
- [Anything unexpected?]
- [What would you do differently next time?]
- [Any packages that should be added to base image?]

---

## Post-Implementation

### Documentation Updates Needed
- [ ] Update `/Users/brent/git/cc-orchestra/docs/RUNNERS.md`
- [ ] Update `/Users/brent/git/cc-orchestra/docs/RUNNER_SETUP.md`
- [ ] Update `/Users/brent/git/cc-orchestra/docs/TROUBLESHOOTING.md`
- [ ] Update internal wiki/runbook

### Team Communication
- [ ] Send completion email with:
  - What changed (Ubuntu 22.04 packages)
  - What stayed same (runner labels, capacity)
  - Known issues (if any)
  - Point of contact for questions

### Cleanup
- [ ] Delete Proxmox snapshots after 30 days (if stable)
- [ ] Archive this checklist with notes
- [ ] Update runner inventory

---

## Timeline Summary

| Phase | Duration | Status |
|-------|----------|--------|
| Pre-flight checks | 15 min | [ ] |
| Snapshots | 5 min | [ ] |
| Update runner-01 | 30 min | [ ] |
| Test runner-01 | 45 min | [ ] |
| Update runners 2-9 | 2 hours | [ ] |
| Post-update verification | 15 min | [ ] |
| Full test suite | 30 min | [ ] |
| **Total** | **~4 hours** | [ ] |
| Week 1 monitoring | 1 week | [ ] |

---

## Quick Reference Commands

```bash
# Check all runners
gh api orgs/langstons/actions/runners --jq '.runners[] | {name, status}'

# Check container status
ssh root@192.168.9.220 "for i in {200..208}; do pct status \$i; done"

# Trigger test workflow
gh workflow run test.yml -R langstons/<repo> --ref main

# Watch workflow
gh run watch -R langstons/<repo>

# Check recent runs
gh run list -R langstons/<repo> -L 5

# View workflow logs
gh run view <run-id> -R langstons/<repo> --log

# Rollback runner
pct rollback <CTID> pre-ubuntu-update-<timestamp>
```

---

**Checklist Version:** 1.0
**Last Updated:** 2025-11-20
**Status:** READY FOR EXECUTION

**Start Time:** ________________
**Completion Time:** ________________
**Executed By:** ________________
**Overall Status:** [ ] SUCCESS  [ ] PARTIAL  [ ] ROLLBACK REQUIRED
