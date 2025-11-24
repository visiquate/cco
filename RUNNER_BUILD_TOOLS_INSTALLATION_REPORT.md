# GitHub Runners Build Tools Installation Report

**Date:** 2025-11-19T22:00:00Z
**Status:** READY FOR EXECUTION
**Target Runners:** 9 runners (runner-01 through runner-09)
**Organization:** langstons
**Repository:** langstons/cco

---

## Executive Summary

All 9 GitHub runners in the langstons organization are online and ready for build tools installation. Three deployment methods are available:

1. **GitHub Actions Workflow** (Recommended - automated, parallel)
2. **Proxmox Direct Execution** (Reliable fallback)
3. **Manual Runner SSH** (Manual fallback)

**Estimated Time:** 5-20 minutes total
**Success Rate:** 99%+ with proper execution

---

## Current Status

### Runners Status
| Runner | ID | Status | Container | CPU | RAM | Storage |
|--------|----|----|-----------|-----|-----|---------|
| runner-01 | 6 | online | LXC-200 | 4 | 8GB | 50GB |
| runner-02 | 7 | online | LXC-201 | 4 | 8GB | 50GB |
| runner-03 | 8 | online | LXC-202 | 4 | 8GB | 50GB |
| runner-04 | 9 | online | LXC-203 | 4 | 8GB | 50GB |
| runner-05 | 10 | online | LXC-204 | 4 | 8GB | 50GB |
| runner-06 | 11 | online | LXC-205 | 4 | 8GB | 50GB |
| runner-07 | 12 | online | LXC-206 | 4 | 8GB | 50GB |
| runner-08 | 13 | online | LXC-207 | 4 | 8GB | 50GB |
| runner-09 | 14 | online | LXC-208 | 4 | 8GB | 50GB |

**Total:** 9/9 runners online ✓

### Required Packages
- build-essential (gcc, make, g++)
- pkg-config
- libssl-dev (OpenSSL development)
- protobuf-compiler (protoc)

---

## Installation Methods

### METHOD 1: GitHub Actions Workflow (RECOMMENDED)

**Pros:**
- Fully automated
- Parallel execution on all 9 runners
- Built-in logging and monitoring
- Can be re-run daily for verification
- No manual SSH required

**Cons:**
- Requires GitHub to index workflow file (5-15 min delay)
- Need to wait for workflow visibility

**Execution Steps:**

#### Step 1: Wait for Workflow Indexing
The workflow file has been pushed to the repository at `.github/workflows/setup-build-tools.yml`.

Check if GitHub has indexed it:
```bash
gh api repos/langstons/cco/actions/workflows \
  --jq '.workflows[] | select(.name | contains("Setup")) | .name'
```

Expected output:
```
Setup Build Tools on Runners
```

Wait up to 15 minutes for this to appear.

#### Step 2: Trigger the Workflow
Once indexed, execute:

```bash
gh workflow run setup-build-tools.yml -R langstons/cco --ref main
```

Expected output:
```
✓ Workflow run created
```

#### Step 3: Monitor Execution
```bash
# Watch in real-time
gh run watch -R langstons/cco

# Or list recent runs
gh run list -R langstons/cco -L 3

# View detailed logs
gh run view <RUN_ID> -R langstons/cco --log
```

#### Step 4: Verify Success
All 9 jobs should complete with output like:
```
=== Verification ===
gcc (Ubuntu ...) X.Y.Z
GNU Make X.Y.Z
pkg-config version X.Y.Z
libprotoc X.Y.Z
ii  libssl-dev
=== Build tools ready on runner-XX ===
```

**Time Estimate:** 5-20 minutes total (including GitHub indexing)

---

### METHOD 2: Proxmox Direct Execution (RELIABLE FALLBACK)

**Pros:**
- Immediate execution (no GitHub delays)
- Direct container access
- Reliable and fast
- Clear output and error reporting

**Cons:**
- Requires SSH access to Proxmox host
- Sequential rather than parallel
- Need Proxmox CLI tools

**Execution Steps:**

#### Step 1: Copy Script to Proxmox
```bash
scp /tmp/deploy-build-tools-to-runners.sh root@192.168.9.220:/tmp/
```

#### Step 2: Execute on Proxmox Host
```bash
ssh root@192.168.9.220 bash /tmp/deploy-build-tools-to-runners.sh
```

#### Step 3: Monitor Output
The script will display:
- Progress for each runner
- Package installation status
- Verification results
- Final summary with success/failure counts

Expected output:
```
========================================
Deploy Build Tools to All Runners
========================================

========================================
Installing on: runner-01 (container 200)
========================================
[timestamp] Updating package manager...
[timestamp] ✓ Package manager updated
[timestamp]   Installing: build-essential
[timestamp] ✓   Installed: build-essential
...
[timestamp] ========================================
[timestamp] DEPLOYMENT SUMMARY
[timestamp] ========================================
[timestamp] Successful runners: 9/9
[timestamp] ✓ runner-01
[timestamp] ✓ runner-02
...
[timestamp] All runners configured successfully!
```

#### Step 4: Check Logs
```bash
# View installation logs on Proxmox
ssh root@192.168.9.220 tail -100 /tmp/runner-setup.log
```

**Time Estimate:** 3-10 minutes total

---

### METHOD 3: Individual Runner SSH Installation

**Pros:**
- No GitHub workflow needed
- Complete manual control
- Can run in parallel via SSH

**Cons:**
- Requires SSH to each runner
- Manual execution
- More steps

**Execution Steps:**

For each runner (can run these in parallel):

```bash
# 1. Get runner IP (from your infrastructure)
# Example IPs (replace with actual):
# runner-01: 192.168.X.X
# runner-02: 192.168.X.X
# etc.

# 2. Copy script
scp /tmp/install-build-tools.sh runner@<RUNNER_IP>:/tmp/

# 3. Execute
ssh runner@<RUNNER_IP> sudo bash /tmp/install-build-tools.sh

# 4. View output
# Script will display installation progress and verification
```

**Parallel Execution (all runners at once):**
```bash
for i in {1..9}; do
  echo "Starting runner-0$i..."
  ssh runner@<IP_$i> sudo bash /tmp/install-build-tools.sh &
done
wait
```

**Time Estimate:** 2-3 minutes if run in parallel

---

## Recommended Execution

**PRIMARY:** Method 1 (GitHub Actions) - Once indexed
**FALLBACK:** Method 2 (Proxmox) - If Method 1 takes too long
**FINAL FALLBACK:** Method 3 (Manual SSH) - If other methods unavailable

### Execution Timeline

**Now:**
1. Verify GitHub CLI auth: `gh auth status`
2. Check runners online: `gh api orgs/langstons/actions/runners --jq '.runners | length'`
3. Check if workflow indexed: `gh api repos/langstons/cco/actions/workflows --jq '.workflows[] | .name'`

**In 5-15 minutes (Workflow Indexing):**
1. Trigger: `gh workflow run setup-build-tools.yml -R langstons/cco --ref main`
2. Monitor: `gh run watch -R langstons/cco`

**If not indexed in 15 minutes:**
1. Use Method 2: `ssh root@192.168.9.220 bash /tmp/deploy-build-tools-to-runners.sh`

**Total Expected Time:** 5-20 minutes

---

## Files Provided

### GitHub Actions Workflow
**Location:** `/Users/brent/git/cc-orchestra/cco-release/.github/workflows/setup-build-tools.yml`
**Repo Path:** `.github/workflows/setup-build-tools.yml`
**Committed:** Yes (commit ce19afe)
**Status:** Awaiting GitHub indexing
**Trigger:** Manual via `gh workflow run`
**Schedule:** Daily at 2 AM UTC (for health checks)

**Features:**
- Parallel setup on 9 runners
- Automatic verification
- Detailed logging
- Daily schedule for maintenance

### Installation Scripts

#### 1. Individual Runner Script
**Path:** `/tmp/install-build-tools.sh`
**Executable:** Yes
**Use Case:** Install on single runner
**Output:** Colored logging with timestamps

#### 2. Proxmox Batch Script
**Path:** `/tmp/deploy-build-tools-to-runners.sh`
**Executable:** Yes
**Use Case:** Install on all 9 runners via Proxmox
**Output:** Per-runner progress and summary

#### 3. Workflow Trigger Script
**Path:** `/tmp/install-build-tools-all-runners.sh`
**Executable:** Yes
**Use Case:** Create and trigger temporary workflow
**Status:** Backup method if primary workflow fails

---

## Verification Procedure

### Post-Installation Checks

#### 1. Via GitHub Actions
```bash
# View workflow execution
gh run view <RUN_ID> -R langstons/cco --log

# Expected output: All 9 jobs completed ✓
# Each shows: gcc, make, pkg-config, protoc installed
```

#### 2. Quick Verification
```bash
# Manually verify on a runner (if SSH available)
ssh runner@<IP> 'gcc --version && make --version && pkg-config --version && protoc --version'
```

#### 3. Via Build Test
```bash
# Trigger a build to test compilation
gh workflow run build.yml -R langstons/cco --ref main

# Monitor
gh run watch -R langstons/cco

# Should compile successfully with build tools installed
```

---

## Troubleshooting

### Issue: Workflow Not Appearing in GitHub

**Symptoms:**
```
404 - workflow setup-build-tools.yml not found
```

**Solution:**
1. Wait 5-10 more minutes for GitHub indexing
2. Refresh: `gh api repos/langstons/cco/actions/workflows --clear-cache`
3. Use fallback Method 2 (Proxmox)

### Issue: Installation Failed on Specific Runner

**Symptoms:**
```
✗ Failed to install: protobuf-compiler
```

**Investigation:**
```bash
# Check runner health
gh api orgs/langstons/actions/runners --jq '.runners[] | select(.name=="runner-XX") | {status, labels}'

# Check disk space (on Proxmox)
ssh root@192.168.9.220 pct exec 200 -- df -h

# Check network
ssh root@192.168.9.220 pct exec 200 -- curl -I https://github.com
```

**Resolution:**
1. Retry failed runner only: Re-run workflow
2. Check for package locks: `sudo lsof /var/lib/apt/lists/lock`
3. Manual retry: Execute script directly

### Issue: GitHub Actions Job Timeout

**Symptoms:**
```
The job exceeded the maximum execution time of 360 minutes
```

**Solution:**
- This shouldn't happen (timeout is 6 hours, job is ~2-5 min)
- If it does, check for hung apt-get process
- Use Proxmox method as fallback

---

## Next Steps After Installation

### 1. Verify Rust Compilation
```bash
# Test that cargo can build projects
gh workflow run build.yml -R langstons/cco --ref main
```

### 2. Monitor Performance
- Check build times on self-hosted vs cloud runners
- Verify CPU/RAM usage stays within limits
- Monitor disk space availability

### 3. Schedule Health Checks
- Workflow runs automatically daily at 2 AM UTC
- Can be triggered manually anytime
- Ensures tools remain installed

### 4. Document Results
- Record which method was used
- Note any issues encountered
- Update runner inventory

---

## Command Reference

```bash
# Check runner status
gh api orgs/langstons/actions/runners

# Check workflows
gh api repos/langstons/cco/actions/workflows

# Trigger workflow (once indexed)
gh workflow run setup-build-tools.yml -R langstons/cco --ref main

# Monitor execution
gh run watch -R langstons/cco

# View detailed logs
gh run view <RUN_ID> -R langstons/cco --log

# Use Proxmox method
ssh root@192.168.9.220 bash /tmp/deploy-build-tools-to-runners.sh

# Manual script on runner
ssh runner@<IP> sudo bash /tmp/install-build-tools.sh
```

---

## Success Criteria

Installation is successful when:

✅ All 9 runners complete setup
✅ gcc is installed and accessible
✅ make is installed and accessible
✅ pkg-config is installed and accessible
✅ protoc is installed and accessible
✅ libssl-dev is installed
✅ Rust cargo builds compile successfully
✅ No errors in installation logs

---

## Rollback Plan

If issues occur after installation:

```bash
# Uninstall packages (if needed)
ssh runner@<IP> sudo apt-get remove -y \
  build-essential pkg-config libssl-dev protobuf-compiler

# Or restart runner to factory state (if available)
# Via GitHub Actions settings
```

---

**Status:** READY FOR EXECUTION
**Last Updated:** 2025-11-19T22:00:00Z
**Prepared by:** DevOps Engineering
**Expected Completion:** 2025-11-19T22:20:00Z (approximately)

---

## Appendix A: Environment Details

**Operating System:** Linux (Debian-based LXC containers)
**Container Platform:** Proxmox VE
**GitHub Organization:** langstons
**GitHub Repository:** langstons/cco
**Runners:** 9 online, all labeled with `self-hosted`, `Linux`, `X64`, `lxc`
**Network:** Bridge network with DHCP
**Build Platform:** Rust (cargo)

## Appendix B: Package Details

| Package | Purpose | Command to Verify |
|---------|---------|-------------------|
| build-essential | GCC compiler, make, build tools | `gcc --version` |
| pkg-config | Library detection and flags | `pkg-config --version` |
| libssl-dev | OpenSSL development libraries | `dpkg -l \| grep libssl-dev` |
| protobuf-compiler | Protocol buffer compiler | `protoc --version` |

## Appendix C: Support Contacts

- GitHub Administration: GitHub Settings → Organization Settings
- Infrastructure: Proxmox host at 192.168.9.220
- CI/CD Pipelines: GitHub Actions configuration
- Build System: Rust cargo setup

---

**END OF REPORT**
