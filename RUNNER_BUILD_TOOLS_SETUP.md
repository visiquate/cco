# GitHub Runner Build Tools Setup Guide

## Objective
Install build tools on 9 self-hosted GitHub runners (runner-01 through runner-09) in the langstons organization to enable Rust cargo compilation.

## Required Packages
- **build-essential** - Compiler toolchain (gcc, make, g++)
- **pkg-config** - Package configuration tool
- **libssl-dev** - OpenSSL development libraries
- **protobuf-compiler** - Protocol buffer compiler

## Runner Status

### Current Online Runners
All 9 runners are currently online and ready for setup:

| Runner | ID | Status | Labels | Notes |
|--------|-----|--------|--------|-------|
| runner-01 | 6 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-02 | 7 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-03 | 8 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-04 | 9 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-05 | 10 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-06 | 11 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-07 | 12 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-08 | 13 | online | self-hosted, Linux, X64, lxc | Ready |
| runner-09 | 14 | online | self-hosted, Linux, X64, lxc | Ready |

## Installation Methods

### Method 1: GitHub Actions Workflow (Recommended)
A dedicated workflow has been created to install build tools on all runners.

**Workflow File:** `.github/workflows/setup-build-tools.yml`

**To Trigger:**
```bash
gh workflow run setup-build-tools.yml -R langstons/cco --ref main
```

**Features:**
- Parallel installation on all 9 runners
- Automatic verification after installation
- Daily health check (2 AM UTC)
- Detailed logging on each runner

**Verification Steps:**
1. Check workflow execution: `gh run list -R langstons/cco -w setup-build-tools.yml`
2. View detailed logs: `gh run view <RUN_ID> -R langstons/cco --log`
3. Confirm all 9 jobs completed successfully

### Method 2: Manual Proxmox Deployment Script
If direct script execution on Proxmox host is preferred:

**Script:** `/tmp/deploy-build-tools-to-runners.sh`

**To Execute on Proxmox Host:**
```bash
ssh root@<PROXMOX_HOST>
bash /tmp/deploy-build-tools-to-runners.sh
```

**Container Mapping:**
- runner-01 → container 200
- runner-02 → container 201
- runner-03 → container 202
- runner-04 → container 203
- runner-05 → container 204
- runner-06 → container 205
- runner-07 → container 206
- runner-08 → container 207
- runner-09 → container 208

### Method 3: Individual Shell Script
For installing on individual runners:

**Script:** `/tmp/install-build-tools.sh`

**To Run on Each Runner:**
```bash
# Copy to runner
scp /tmp/install-build-tools.sh runner@<RUNNER_IP>:/tmp/

# Execute on runner
ssh runner@<RUNNER_IP> sudo bash /tmp/install-build-tools.sh
```

## Installation Steps (GitHub Actions)

### Step 1: Verify Workflow File
The setup-build-tools.yml workflow is located at:
- Repository: `langstons/cco`
- Path: `.github/workflows/setup-build-tools.yml`
- Branch: `main`

### Step 2: Trigger Workflow
```bash
# Trigger the workflow
gh workflow run setup-build-tools.yml -R langstons/cco --ref main

# Wait a few seconds for it to appear
sleep 5

# List recent runs
gh run list -R langstons/cco -L 1
```

### Step 3: Monitor Execution
```bash
# Watch the workflow in real-time
gh run watch -R langstons/cco

# Or view specific run details
gh run view <RUN_ID> -R langstons/cco

# View logs
gh run view <RUN_ID> -R langstons/cco --log
```

### Step 4: Verify Success
Each runner job should:
1. Update package manager
2. Install 4 packages (build-essential, pkg-config, libssl-dev, protobuf-compiler)
3. Verify each tool is accessible
4. Display version information

**Expected Output on Each Runner:**
```
=== Verification ===
gcc (Ubuntu ...) X.X.X
GNU Make X.X.X
pkg-config version X.X.X
libprotoc X.X.X
ii  libssl-dev
=== Build tools ready on runner-XX ===
```

## Verification Checklist

After installation, verify on all runners:

### Via GitHub Actions (Recommended)
```bash
# Trigger verification workflow
gh workflow run setup-build-tools.yml -R langstons/cco --ref main

# All 9 jobs should complete successfully with:
# ✓ gcc installed and working
# ✓ make installed and working
# ✓ pkg-config installed and working
# ✓ protoc installed and working
# ✓ libssl-dev installed
```

### Manual Verification on Individual Runners
```bash
# Check gcc
gcc --version

# Check make
make --version

# Check pkg-config
pkg-config --version

# Check protobuf compiler
protoc --version

# Check libssl-dev
dpkg -l | grep libssl-dev
```

## Testing Rust Compilation

Once build tools are installed, test Rust compilation:

### Create Test Job
```bash
# Create a test workflow that compiles a Rust project
cat > /tmp/test-rust-build.yml << 'EOF'
name: Test Rust Build

on:
  workflow_dispatch:

jobs:
  test-build:
    runs-on: [self-hosted, lxc]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build CCO
        run: |
          cd cco
          cargo build --release

      - name: Run tests
        run: |
          cd cco
          cargo test --release
EOF

# View test
cat /tmp/test-rust-build.yml
```

### Run Test
```bash
# After pushing the test workflow:
gh workflow run test-rust-build.yml -R langstons/cco

# Watch execution
gh run watch -R langstons/cco
```

## Troubleshooting

### Workflow Not Found
**Issue:** "workflow setup-build-tools.yml not found"

**Solution:**
- The workflow file is at `.github/workflows/setup-build-tools.yml`
- Give GitHub 5-10 minutes to index new workflow files
- Refresh: `gh workflow list -R langstons/cco`

### Installation Failed on Runner
**Issue:** Package installation failed on a specific runner

**Steps:**
1. Check runner logs: `gh run view <RUN_ID> -R langstons/cco --log | grep runner-XX`
2. SSH into Proxmox host
3. Execute: `pct exec <CONTAINER_ID> -- apt-get install -y <package>`
4. Check for network/disk space issues

### Package Manager Lock
**Issue:** "Unable to acquire the dpkg frontend lock"

**Solution:**
- Wait for ongoing updates to complete
- Check: `sudo lsof /var/lib/apt/lists/lock`
- If stuck: `sudo rm /var/lib/apt/lists/lock` and retry

## Files Provided

1. **Setup Workflow**
   - Path: `/Users/brent/git/cc-orchestra/cco-release/.github/workflows/setup-build-tools.yml`
   - Pushed to: `langstons/cco` main branch
   - Status: Triggered via `gh workflow run` command

2. **Installation Script**
   - Path: `/tmp/install-build-tools.sh`
   - Executable: Yes
   - Manual fallback option

3. **Proxmox Deployment Script**
   - Path: `/tmp/deploy-build-tools-to-runners.sh`
   - Executable: Yes
   - For direct Proxmox execution

## Next Steps

### Immediate (Now)
1. Trigger the workflow: `gh workflow run setup-build-tools.yml -R langstons/cco --ref main`
2. Monitor execution for ~2-5 minutes
3. Verify all 9 runners complete successfully

### Short-term (Today)
1. Test Rust compilation with a simple build
2. Verify Release workflow works with build tools
3. Document any issues encountered

### Long-term (This Week)
1. Run daily health checks to ensure tools remain installed
2. Update documentation if additional tools needed
3. Plan for scaling to more runners if needed

## Additional Resources

- **Workflow Documentation:** See workflow file for detailed comments
- **Installation Logs:** Available in runner logs after execution
- **Health Check:** Workflow runs daily at 2 AM UTC
- **Manual Control:** Can be re-triggered anytime via `gh workflow run`

## Support

If build tools installation fails or verification shows issues:

1. Check runner health: `gh api orgs/langstons/actions/runners`
2. View installation logs: `gh run view <RUN_ID> --log`
3. Retry installation: `gh workflow run setup-build-tools.yml -R langstons/cco`
4. Manual install as fallback: Execute `/tmp/install-build-tools.sh` on individual runners

---

**Status:** Ready for execution
**Created:** 2025-11-19
**Last Updated:** 2025-11-19T21:50:00Z
