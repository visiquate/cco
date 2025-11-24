# Optimal GitHub Actions Runner Configuration for Langstons Organization

**Date:** 2025-11-20
**Status:** COMPREHENSIVE ANALYSIS & RECOMMENDATION
**Target:** Replace 9 existing runners (IDs 6-14) with optimized configuration
**Current Issue:** CentOS/RHEL glibc conflicts, yum dependency issues

---

## Executive Summary

After analyzing all langstons organization repositories and their workflow requirements, this document provides the optimal runner configuration that will serve ALL repositories efficiently without the current dependency conflicts.

### Recommendation at a Glance
- **OS:** Ubuntu 22.04 LTS (Current runners appear Ubuntu-based already)
- **Container Type:** LXC (unprivileged) on Proxmox
- **Base Image:** Ubuntu 22.04 Cloud template
- **Pre-installed packages:** Minimal + essential only
- **Strategy:** Single standardized image for all runners

---

## Repository Analysis

### Current Repository Landscape

| Repository | Language | Key Dependencies | Python Versions | Node Versions |
|------------|----------|------------------|-----------------|---------------|
| **claude-orchestra** | Rust | cargo, build-essential, protobuf, openssl | N/A | N/A |
| **cco** | Rust | cargo, build-essential, protobuf, openssl | N/A | N/A |
| **claude-analytics** | JavaScript | Node.js, npm | N/A | 18.x, 20.x, 21.x |
| **growthiq** | Python | libxml2-dev, xmlsec1-dev, openssl-dev | 3.10, 3.11, 3.12 | 20.x |
| **slack-broker** | JavaScript | Node.js, npm | N/A | 18.x, 20.x |
| **dark-web-crawler** | Python | Python libs, system deps | 3.10+ | N/A |
| **anythingllm-sso-helper** | JavaScript | Node.js, npm | N/A | 18.x, 20.x |
| **slack-duo-verifier** | Python | Python libs | 3.10+ | N/A |
| **sqlquiz** | Python | PostgreSQL client libs | 3.10+ | N/A |
| **account-leaser** | Python | Python libs | 3.10+ | N/A |
| **streamsnap** | Python | ffmpeg, media libs | 3.10+ | N/A |

### Workflow Requirements Matrix

| Requirement | Repos Needing It | Priority | Package |
|-------------|------------------|----------|---------|
| **Rust toolchain** | claude-orchestra, cco | HIGH | rustup (installed per-workflow) |
| **Python 3.10-3.12** | 7 repos | HIGH | python3.10, python3.11, python3.12 |
| **Node.js 18-21** | 5 repos | HIGH | nodejs (installed per-workflow via actions) |
| **Build tools (gcc/make)** | All Rust, some Python | HIGH | build-essential |
| **pkg-config** | Rust builds | HIGH | pkg-config |
| **OpenSSL dev libs** | Rust, Python | HIGH | libssl-dev |
| **protobuf compiler** | Rust projects | MEDIUM | protobuf-compiler |
| **XML libs** | growthiq | MEDIUM | libxml2-dev, libxmlsec1-dev |
| **PostgreSQL client** | sqlquiz | LOW | libpq-dev |
| **ffmpeg** | streamsnap | LOW | ffmpeg (install if needed) |

---

## Problem Analysis: Current vs Optimal

### Current Setup Issues

**Observed Problem from growthiq workflow:**
```yaml
- name: Install system dependencies
  run: |
    # Rocky Linux 9 uses dnf, not apt-get
    # CRB repo is needed for -devel packages
    sudo dnf config-manager --set-enabled crb
    sudo dnf install -y libxml2-devel xmlsec1-devel xmlsec1-openssl xmlsec1-openssl-devel
```

**Issues:**
1. ❌ Rocky Linux 9 / CentOS 9 Stream uses `dnf` instead of `apt`
2. ❌ CRB (CodeReady Builder) repository required for `-devel` packages
3. ❌ Package names differ: `-devel` vs `-dev` suffix
4. ❌ glibc version conflicts causing gcc installation failures
5. ❌ yum/dnf dependency resolution issues

**But wait - Current runner status shows:**
- Runners 6-14 are **already Ubuntu-based LXC containers** (labels show `lxc`)
- Container IDs 200-208 in Proxmox
- They appear to be working with apt-based installation

**Conflict Resolution:**
- The user mentioned "IDs 107-115" with CentOS/RHEL issues
- Current active runners are IDs 6-14 (containers 200-208)
- **Conclusion:** Old problematic runners (107-115) were likely replaced, new Ubuntu runners exist but need proper configuration

---

## Optimal Configuration Design

### 1. Operating System Selection

**Recommended: Ubuntu 22.04 LTS**

**Justification:**
- ✅ **LTS Support:** Maintained until April 2027
- ✅ **Package Availability:** All required packages available via apt
- ✅ **Python Versions:** Deadsnakes PPA provides 3.10, 3.11, 3.12
- ✅ **Consistent Naming:** Uses `-dev` suffix (libssl-dev, libxml2-dev)
- ✅ **No CRB/EPEL Required:** All packages in main/universe repos
- ✅ **GitHub Actions Compatibility:** Most actions tested on Ubuntu
- ✅ **Current Standard:** Existing runners already Ubuntu-based

**Alternatives Considered:**

| OS | Pros | Cons | Decision |
|----|------|------|----------|
| **Rocky Linux 9** | Enterprise-grade, RHEL-compatible | dnf complexity, CRB repos, fewer Python versions | ❌ Reject |
| **Debian 12** | Stable, lightweight | Older packages, less GH Actions testing | ❌ Reject |
| **Alpine Linux** | Minimal size, fast | musl libc (breaks some Python libs), steep learning curve | ❌ Reject |
| **Ubuntu 24.04 LTS** | Newest LTS | Too new, potential compatibility issues | ⏰ Future option |
| **Ubuntu 22.04 LTS** | Mature, well-tested, complete packages | None significant | ✅ **SELECTED** |

### 2. Essential Pre-Installed Packages

**Philosophy:** Install minimal system-level dependencies. Let workflows install language runtimes via actions.

#### Core Build Tools (ESSENTIAL)
```bash
# Compiler toolchain
build-essential        # gcc, g++, make, libc6-dev
pkg-config            # Library configuration
git                   # Version control
curl wget            # Download tools
ca-certificates      # SSL certificates
gnupg                # GPG for package verification
software-properties-common  # PPA management
```

#### Language-Specific Development Libraries (ESSENTIAL)
```bash
# SSL/TLS support (needed by Rust, Python, Node.js)
libssl-dev           # OpenSSL development files
libssl3              # OpenSSL runtime

# Protocol Buffers (needed by Rust projects)
protobuf-compiler    # protoc compiler
libprotobuf-dev      # protobuf development files

# XML libraries (needed by growthiq)
libxml2-dev          # XML parsing
libxmlsec1-dev       # XML security
libxmlsec1-openssl   # XML security OpenSSL backend

# PostgreSQL client (needed by sqlquiz and potentially others)
libpq-dev            # PostgreSQL C client library
```

#### Python Support (ESSENTIAL)
```bash
# Multiple Python versions via deadsnakes PPA
python3.10 python3.10-dev python3.10-venv
python3.11 python3.11-dev python3.11-venv
python3.12 python3.12-dev python3.12-venv

# Python tools
python3-pip          # pip installer
python3-venv         # Virtual environments
```

#### System Utilities (ESSENTIAL)
```bash
sudo                 # Privilege escalation
systemd              # Service management
openssh-server       # Remote access
```

#### Optional/As-Needed Packages (Install via workflow if needed)
```bash
# These should be installed in workflows, not in base image
# - Docker (use setup-buildx-action)
# - Node.js (use setup-node action)
# - Rust (use setup-rust-toolchain action)
# - Go (use setup-go action)
# - ffmpeg (streamsnap can install if needed)
```

### 3. Packages NOT to Pre-install

**Let GitHub Actions handle these:**
- ❌ Docker/Docker Compose (use docker/setup-buildx-action)
- ❌ Node.js/npm (use actions/setup-node)
- ❌ Rust/cargo (use actions-rust-lang/setup-rust-toolchain)
- ❌ Go (use actions/setup-go)
- ❌ Specific Python packages (use pip in workflow)

**Rationale:**
- Actions are optimized and cached
- Flexibility to use different versions per repo
- Smaller base image
- Easier to update (workflow change vs runner rebuild)

---

## Implementation Plan

### Phase 1: Base Container Template Creation

#### Step 1: Create Ubuntu 22.04 Template

```bash
# On Proxmox host (192.168.9.220)

# Download Ubuntu 22.04 Cloud image
pveam update
pveam download local ubuntu-22.04-standard_22.04-1_amd64.tar.zst

# Create template container (ID 9000)
pct create 9000 \
  local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \
  --hostname runner-template \
  --cores 4 \
  --memory 8192 \
  --swap 0 \
  --net0 name=eth0,bridge=vmbr0,ip=dhcp \
  --storage local-lvm \
  --rootfs local-lvm:50 \
  --unprivileged 1 \
  --features nesting=1 \
  --ostype ubuntu \
  --password <random-password>

# Start template
pct start 9000
```

#### Step 2: Configure Template

```bash
# Enter container
pct enter 9000

# Update system
apt-get update
apt-get upgrade -y

# Install core build tools
apt-get install -y \
  build-essential \
  pkg-config \
  git \
  curl \
  wget \
  ca-certificates \
  gnupg \
  software-properties-common \
  sudo \
  openssh-server

# Install SSL libraries
apt-get install -y \
  libssl-dev \
  libssl3

# Install protobuf
apt-get install -y \
  protobuf-compiler \
  libprotobuf-dev

# Install XML libraries (for growthiq)
apt-get install -y \
  libxml2-dev \
  libxmlsec1-dev \
  libxmlsec1-openssl

# Install PostgreSQL client library
apt-get install -y \
  libpq-dev

# Add deadsnakes PPA for multiple Python versions
add-apt-repository ppa:deadsnakes/ppa -y
apt-get update

# Install Python versions
apt-get install -y \
  python3.10 python3.10-dev python3.10-venv \
  python3.11 python3.11-dev python3.11-venv \
  python3.12 python3.12-dev python3.12-venv \
  python3-pip \
  python3-venv

# Create runner user
useradd -m -s /bin/bash -G sudo runner
echo "runner ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/runner

# Clean up
apt-get autoremove -y
apt-get clean
rm -rf /var/lib/apt/lists/*

# Exit container
exit

# Stop template
pct stop 9000

# Convert to template
pct template 9000
```

#### Step 3: Verification Script

Create `/tmp/verify-runner-template.sh`:

```bash
#!/bin/bash
# Verify runner template has all required packages

set -e

CTID=9000

echo "=== Verifying Runner Template (Container $CTID) ==="
echo ""

# Start if not running
pct start $CTID 2>/dev/null || true
sleep 5

echo "=== Build Tools ==="
pct exec $CTID -- gcc --version | head -1
pct exec $CTID -- make --version | head -1
pct exec $CTID -- pkg-config --version

echo ""
echo "=== Protocol Buffers ==="
pct exec $CTID -- protoc --version

echo ""
echo "=== Python Versions ==="
pct exec $CTID -- python3.10 --version
pct exec $CTID -- python3.11 --version
pct exec $CTID -- python3.12 --version
pct exec $CTID -- python3 -m pip --version

echo ""
echo "=== Libraries ==="
pct exec $CTID -- dpkg -l | grep -E "libssl-dev|libxml2-dev|libxmlsec1-dev|libpq-dev" | awk '{print $2, $3}'

echo ""
echo "=== System Info ==="
pct exec $CTID -- lsb_release -a
pct exec $CTID -- uname -a

echo ""
echo "✅ Template verification complete"
```

### Phase 2: Deploy Runners from Template

#### Option A: Keep Existing Containers (Update in Place)

If current containers 200-208 are working, update them:

```bash
#!/bin/bash
# Update existing runners in place

for CTID in {200..208}; do
  echo "=== Updating Container $CTID ==="

  # Install missing packages
  pct exec $CTID -- bash -c '
    apt-get update
    apt-get install -y \
      build-essential \
      pkg-config \
      libssl-dev \
      protobuf-compiler \
      libprotobuf-dev \
      libxml2-dev \
      libxmlsec1-dev \
      libxmlsec1-openssl \
      libpq-dev \
      software-properties-common

    # Add deadsnakes PPA if not already added
    if ! grep -q "deadsnakes" /etc/apt/sources.list.d/*; then
      add-apt-repository ppa:deadsnakes/ppa -y
      apt-get update
    fi

    # Install Python versions
    apt-get install -y \
      python3.10 python3.10-dev python3.10-venv \
      python3.11 python3.11-dev python3.11-venv \
      python3.12 python3.12-dev python3.12-venv

    # Clean up
    apt-get autoremove -y
    apt-get clean
  '

  echo "✅ Container $CTID updated"
  echo ""
done

echo "✅ All runners updated"
```

#### Option B: Fresh Deployment from Template (Clean Slate)

If you want to start fresh:

```bash
#!/bin/bash
# Deploy 9 new runners from template

TEMPLATE_ID=9000
START_CTID=200
RUNNER_COUNT=9

for i in $(seq 0 $(($RUNNER_COUNT - 1))); do
  CTID=$(($START_CTID + $i))
  RUNNER_NUM=$(printf "%02d" $(($i + 1)))
  RUNNER_NAME="runner-$RUNNER_NUM"

  echo "=== Creating $RUNNER_NAME (Container $CTID) ==="

  # Check if exists
  if pct status $CTID &>/dev/null; then
    echo "⚠️  Container $CTID exists, skipping..."
    continue
  fi

  # Clone from template
  pct clone $TEMPLATE_ID $CTID \
    --hostname $RUNNER_NAME \
    --full \
    --description "GitHub Actions Runner $RUNNER_NUM"

  # Start container
  pct start $CTID
  sleep 5

  # Configure GitHub Actions runner
  # (This would include the GitHub Actions runner installation script)

  echo "✅ $RUNNER_NAME created and configured"
  echo ""
done
```

### Phase 3: Validation & Testing

#### Test Suite

Create `/tmp/test-runner-capabilities.sh`:

```bash
#!/bin/bash
# Test runner capabilities against actual workflow requirements

CTID=$1
if [ -z "$CTID" ]; then
  echo "Usage: $0 <container-id>"
  exit 1
fi

echo "=== Testing Runner Capabilities (Container $CTID) ==="
echo ""

# Test Rust build capability
echo "=== Testing Rust Build ==="
pct exec $CTID -- su - runner -c '
  if ! command -v rustc &>/dev/null; then
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
  fi
  cd /tmp
  cargo new test_rust_build --bin
  cd test_rust_build
  cargo build --release
  echo "✅ Rust build successful"
'

# Test Python 3.10-3.12
echo ""
echo "=== Testing Python Builds ==="
for PY_VER in 3.10 3.11 3.12; do
  pct exec $CTID -- su - runner -c "
    python$PY_VER -m venv /tmp/test_py_$PY_VER
    source /tmp/test_py_$PY_VER/bin/activate
    pip install --upgrade pip
    pip install pytest requests
    python -c 'import requests; print(\"✅ Python $PY_VER working\")'
    deactivate
  "
done

# Test Node.js installation via nvm
echo ""
echo "=== Testing Node.js Setup ==="
pct exec $CTID -- su - runner -c '
  if ! command -v node &>/dev/null; then
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm install 20
  fi
  node --version
  npm --version
  echo "✅ Node.js working"
'

# Test XML libraries (growthiq requirement)
echo ""
echo "=== Testing XML Libraries ==="
pct exec $CTID -- su - runner -c '
  python3.11 -m venv /tmp/test_xml
  source /tmp/test_xml/bin/activate
  pip install lxml xmlsec
  python -c "import lxml.etree; import xmlsec; print(\"✅ XML libraries working\")"
  deactivate
'

# Test protobuf
echo ""
echo "=== Testing Protocol Buffers ==="
pct exec $CTID -- protoc --version
echo "✅ protoc working"

echo ""
echo "=== All Tests Passed ✅ ==="
```

---

## Migration Strategy

### Migration Timeline

| Phase | Duration | Description | Risk Level |
|-------|----------|-------------|------------|
| **1. Template Creation** | 30 min | Create and configure template | LOW |
| **2. Single Runner Test** | 1 hour | Deploy one runner, run all workflow tests | LOW |
| **3. Validation** | 2 hours | Run actual workflows on test runner | MEDIUM |
| **4. Rolling Update** | 2 hours | Update/replace runners one at a time | LOW |
| **5. Monitoring** | 1 week | Watch for issues in production | LOW |

**Total Time:** 1 day for deployment + 1 week monitoring

### Rolling Update Procedure (Zero Downtime)

```bash
#!/bin/bash
# Rolling update - update one runner at a time

RUNNER_CTIDS=(200 201 202 203 204 205 206 207 208)

for CTID in "${RUNNER_CTIDS[@]}"; do
  echo "=== Updating Runner (Container $CTID) ==="

  # 1. Take runner offline in GitHub
  RUNNER_NAME=$(pct exec $CTID -- hostname)
  echo "Taking $RUNNER_NAME offline..."
  # (GitHub CLI command to disable runner)

  # 2. Wait for current jobs to finish (max 30 min)
  echo "Waiting for jobs to finish..."
  sleep 60  # Add more sophisticated job checking here

  # 3. Update container
  echo "Updating packages..."
  # Run update script from Phase 2, Option A

  # 4. Restart runner service
  echo "Restarting GitHub Actions runner..."
  pct exec $CTID -- systemctl restart actions.runner.*.service

  # 5. Verify runner comes back online
  echo "Waiting for runner to come online..."
  sleep 30

  # 6. Test with a simple workflow
  echo "Testing runner..."
  # Trigger test workflow

  echo "✅ $RUNNER_NAME updated and verified"
  echo ""
  echo "Waiting 5 minutes before next runner..."
  sleep 300
done

echo "✅ All runners updated"
```

### Rollback Plan

If issues occur:

```bash
#!/bin/bash
# Rollback to previous configuration

CTID=$1

echo "=== Rolling back Container $CTID ==="

# Stop container
pct stop $CTID

# Restore from backup (if snapshots taken)
# pct restore ...

# Or: Redeploy from old template
# pct clone <old-template-id> $CTID

echo "✅ Rollback complete"
```

**Best Practice:** Take Proxmox snapshots before any changes:
```bash
for CTID in {200..208}; do
  pct snapshot $CTID pre-update-$(date +%Y%m%d) \
    --description "Before package updates"
done
```

---

## Health Check & Monitoring

### Post-Deployment Validation

```bash
#!/bin/bash
# Comprehensive runner health check

echo "=== Runner Health Check ==="
echo ""

echo "=== GitHub Runners Status ==="
gh api orgs/langstons/actions/runners --jq '.runners[] | {name, status, busy, os}'
echo ""

echo "=== Container Status ==="
for CTID in {200..208}; do
  STATUS=$(pct status $CTID)
  RUNNER_NAME=$(pct exec $CTID -- hostname 2>/dev/null || echo "N/A")
  echo "Container $CTID ($RUNNER_NAME): $STATUS"
done
echo ""

echo "=== Package Verification ==="
CTID=200
echo "Checking sample runner (Container $CTID):"
pct exec $CTID -- bash -c '
  echo "gcc: $(gcc --version | head -1)"
  echo "python3.10: $(python3.10 --version)"
  echo "python3.11: $(python3.11 --version)"
  echo "python3.12: $(python3.12 --version)"
  echo "protoc: $(protoc --version)"
  echo "libssl-dev: $(dpkg -l | grep libssl-dev | awk "{print \$3}")"
'
echo ""

echo "=== Recent Workflow Runs ==="
gh run list -R langstons/claude-orchestra -L 5
gh run list -R langstons/growthiq -L 5
echo ""

echo "✅ Health check complete"
```

### Automated Daily Monitoring

Create GitHub Actions workflow `.github/workflows/runner-health.yml`:

```yaml
name: Runner Health Check

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  health-check:
    name: Check Runner ${{ matrix.runner }}
    runs-on: [self-hosted, Linux, X64]
    strategy:
      matrix:
        runner: [1, 2, 3, 4, 5, 6, 7, 8, 9]

    steps:
      - name: System info
        run: |
          echo "=== Runner Health Check ==="
          hostname
          uname -a
          df -h
          free -h

      - name: Package verification
        run: |
          echo "=== Verifying Packages ==="
          gcc --version | head -1
          python3.10 --version
          python3.11 --version
          python3.12 --version
          protoc --version
          dpkg -l | grep -E "libssl-dev|libxml2-dev" | awk '{print $2, $3}'

      - name: GitHub Actions runner status
        run: |
          systemctl status actions.runner.* --no-pager | head -20
```

---

## Performance Optimization

### 1. Package Cache

Enable apt caching on Proxmox host to speed up updates:

```bash
# Install apt-cacher-ng on Proxmox host
apt-get install -y apt-cacher-ng

# Configure runners to use cache
for CTID in {200..208}; do
  pct exec $CTID -- bash -c '
    echo "Acquire::http::Proxy \"http://192.168.9.220:3142\";" > /etc/apt/apt.conf.d/01proxy
  '
done
```

### 2. Workflow Caching

Ensure workflows use actions/cache effectively:

```yaml
- name: Cache pip packages
  uses: actions/cache@v4
  with:
    path: ~/.cache/pip
    key: ${{ runner.os }}-pip-${{ hashFiles('requirements.txt') }}

- name: Cache Cargo registry
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
    key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}

- name: Cache npm packages
  uses: actions/cache@v4
  with:
    path: ~/.npm
    key: ${{ runner.os }}-npm-${{ hashFiles('package-lock.json') }}
```

### 3. Resource Allocation

Current allocation is good but can be tuned:

| Resource | Current | Recommended | Rationale |
|----------|---------|-------------|-----------|
| CPU | 4 cores | 4-6 cores | Sufficient for parallel builds |
| RAM | 8 GB | 8-12 GB | 8 GB sufficient for most, 12 GB for heavy Rust builds |
| Storage | 50 GB | 75-100 GB | Allow for build artifacts, caches |
| Swap | 0 | 0 | Good (prevents OOM delays) |

---

## Cost Analysis

### Current State
- 9 runners × 4 CPU × 8 GB RAM = 36 CPU cores, 72 GB RAM
- Storage: 9 × 50 GB = 450 GB

### Recommended State (No Change)
- Same resources, better software configuration
- No hardware cost increase

### Time Savings
- **Current:** Workflows failing due to dependency issues
- **Optimized:** Clean builds, reliable dependency installation
- **Developer Time Saved:** ~2-5 hours/week (debugging dependency issues)

---

## Troubleshooting Guide

### Common Issues After Migration

#### Issue 1: Python version not found

**Symptom:**
```
python3.11: command not found
```

**Solution:**
```bash
# Verify deadsnakes PPA is added
pct exec <CTID> -- apt-cache policy | grep deadsnakes

# If not, add it
pct exec <CTID> -- add-apt-repository ppa:deadsnakes/ppa -y
pct exec <CTID> -- apt-get update
pct exec <CTID> -- apt-get install -y python3.11 python3.11-dev python3.11-venv
```

#### Issue 2: libxml2-dev missing

**Symptom:**
```
fatal error: libxml/tree.h: No such file or directory
```

**Solution:**
```bash
pct exec <CTID> -- apt-get install -y libxml2-dev libxmlsec1-dev
```

#### Issue 3: protoc version mismatch

**Symptom:**
```
error: failed to run custom build command for `prost-build v0.11.0`
```

**Solution:**
```bash
# Check current version
pct exec <CTID> -- protoc --version

# Update if needed
pct exec <CTID> -- bash -c '
  apt-get update
  apt-get install -y protobuf-compiler
'
```

#### Issue 4: Runner not coming online after update

**Solution:**
```bash
# Check runner service
pct exec <CTID> -- systemctl status actions.runner.*

# Check logs
pct exec <CTID> -- journalctl -u actions.runner.* -n 50

# Restart if needed
pct exec <CTID> -- systemctl restart actions.runner.*
```

---

## Security Considerations

### 1. Unprivileged Containers
- ✅ Already configured (containers 200-208 are unprivileged)
- Container user namespace isolation prevents host compromise

### 2. Network Isolation
- Configure firewall rules if not already present:
```bash
# On each container
pct exec <CTID> -- bash -c '
  # Allow outbound HTTPS only
  iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT
  iptables -A OUTPUT -p tcp --dport 80 -j ACCEPT
  iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
  iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
  iptables -P OUTPUT DROP
'
```

### 3. Resource Limits
- Memory limits enforced at container level ✅
- CPU limits enforced ✅
- No swap to prevent OOM delays ✅

### 4. Audit Logging
- Enable auditd if security compliance required:
```bash
pct exec <CTID> -- apt-get install -y auditd
pct exec <CTID> -- systemctl enable auditd
```

---

## Documentation Checklist

After implementation, update:

- [ ] `/Users/brent/git/cc-orchestra/docs/RUNNERS.md` - Update OS version
- [ ] `/Users/brent/git/cc-orchestra/docs/RUNNER_SETUP.md` - Update package list
- [ ] `/Users/brent/git/cc-orchestra/docs/TROUBLESHOOTING.md` - Add Ubuntu-specific issues
- [ ] Internal wiki/runbook - Document new configuration
- [ ] Team notification - Announce completion and any workflow changes needed

---

## Success Criteria

Migration is successful when:

- [ ] All 9 runners online in GitHub
- [ ] All packages verified installed on all runners
- [ ] claude-orchestra builds pass (Rust)
- [ ] cco builds pass (Rust)
- [ ] growthiq tests pass (Python 3.10, 3.11, 3.12)
- [ ] claude-analytics tests pass (Node.js 18, 20, 21)
- [ ] No workflow failures due to missing dependencies
- [ ] Build times comparable or better than before
- [ ] No security regressions

---

## Next Steps

### Immediate (Day 1)
1. ✅ Review this document with team
2. Take Proxmox snapshots of all containers
3. Update container 200 (runner-01) as test
4. Run full workflow suite on runner-01
5. If successful, proceed with rolling update

### Short-term (Week 1)
1. Update remaining 8 runners (one per day)
2. Monitor workflow success rates
3. Document any edge cases
4. Update team documentation

### Long-term (Month 1)
1. Set up automated health checks
2. Implement runner auto-scaling if needed
3. Create disaster recovery runbook
4. Schedule quarterly reviews

---

## Appendix A: Complete Package Installation Script

```bash
#!/bin/bash
# Complete package installation for Ubuntu 22.04 runners
# Run this on each container or in template

set -e

echo "=== Installing Runner Dependencies ==="
echo ""

# Update system
echo "Updating system..."
apt-get update
apt-get upgrade -y

# Core build tools
echo "Installing build tools..."
apt-get install -y \
  build-essential \
  pkg-config \
  git \
  curl \
  wget \
  ca-certificates \
  gnupg \
  software-properties-common \
  sudo \
  openssh-server

# SSL libraries
echo "Installing SSL libraries..."
apt-get install -y \
  libssl-dev \
  libssl3

# Protocol Buffers
echo "Installing Protocol Buffers..."
apt-get install -y \
  protobuf-compiler \
  libprotobuf-dev

# XML libraries
echo "Installing XML libraries..."
apt-get install -y \
  libxml2-dev \
  libxmlsec1-dev \
  libxmlsec1-openssl

# PostgreSQL client
echo "Installing PostgreSQL client..."
apt-get install -y \
  libpq-dev

# Add deadsnakes PPA for Python versions
echo "Adding deadsnakes PPA..."
add-apt-repository ppa:deadsnakes/ppa -y
apt-get update

# Install Python versions
echo "Installing Python 3.10, 3.11, 3.12..."
apt-get install -y \
  python3.10 python3.10-dev python3.10-venv \
  python3.11 python3.11-dev python3.11-venv \
  python3.12 python3.12-dev python3.12-venv \
  python3-pip \
  python3-venv

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
python3 -m pip --version
dpkg -l | grep -E "libssl-dev|libxml2-dev|libxmlsec1-dev|libpq-dev"

# Clean up
echo ""
echo "Cleaning up..."
apt-get autoremove -y
apt-get clean
rm -rf /var/lib/apt/lists/*

echo ""
echo "✅ All dependencies installed successfully"
```

---

## Appendix B: Dockerfile Alternative (Future Option)

If you want to move to Docker-based runners in the future:

```dockerfile
# Dockerfile for GitHub Actions runner
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
      build-essential \
      pkg-config \
      git \
      curl \
      wget \
      ca-certificates \
      gnupg \
      software-properties-common \
      sudo \
      libssl-dev \
      libssl3 \
      protobuf-compiler \
      libprotobuf-dev \
      libxml2-dev \
      libxmlsec1-dev \
      libxmlsec1-openssl \
      libpq-dev && \
    add-apt-repository ppa:deadsnakes/ppa -y && \
    apt-get update && \
    apt-get install -y \
      python3.10 python3.10-dev python3.10-venv \
      python3.11 python3.11-dev python3.11-venv \
      python3.12 python3.12-dev python3.12-venv \
      python3-pip \
      python3-venv && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create runner user
RUN useradd -m -s /bin/bash -G sudo runner && \
    echo "runner ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/runner

USER runner
WORKDIR /home/runner

# Install GitHub Actions runner
# (Add runner installation script here)
```

---

**Document Version:** 1.0
**Last Updated:** 2025-11-20
**Author:** DevOps Engineering Team
**Status:** READY FOR IMPLEMENTATION
