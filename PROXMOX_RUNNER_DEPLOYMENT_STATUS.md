# Proxmox Runner Deployment Status

**Last Updated**: 2025-11-19
**Status**: ✅ READY FOR DEPLOYMENT
**Target Host**: root@192.168.9.220

---

## 📋 Pre-Deployment Checklist

- ✅ GitHub repository 'cco' created and configured (private)
- ✅ Initial release v2025.11.19.1 published
  - Binary: 18MB
  - SHA256: `9ba05b11a6f6e9cf00b993f0ba7573b4310ee6aa6b33fa68d0590fafff2dbe71`
  - Test Results: 366/366 passing ✓
- ✅ Deployment scripts ready in `/tmp/`
- ✅ GitHub CLI authenticated as `brentley`
- ✅ Security configuration reviewed

---

## 🚀 Deployment Steps

### Step 1: SSH to Proxmox Host

```bash
ssh root@192.168.9.220
```

### Step 2: Download and Execute Deployment Script

```bash
# Copy the script to Proxmox host
scp /tmp/proxmox-runner-setup.sh root@192.168.9.220:/tmp/

# Connect and execute
ssh root@192.168.9.220
cd /tmp
bash proxmox-runner-setup.sh
```

### Step 3: Interactive Configuration

When prompted, provide:
- GitHub Owner: `brentley`
- Repository Name: Leave empty (for organization-level runners)
- Or specify: `cco` (for repo-specific runners)

### Step 4: Verify Deployment

Once script completes, verify:

```bash
# Check runners on GitHub
gh run list -R brentley/cco

# Check runner status
gh api repos/brentley/cco/actions/runners
```

---

## 📊 Runner Configuration

| Component | Configuration |
|-----------|---------------|
| **Runner Count** | 4 initial |
| **Container Type** | LXC (unprivileged) |
| **Container IDs** | 200-203 |
| **CPU per Runner** | 4 cores |
| **RAM per Runner** | 8GB |
| **Storage per Runner** | 50GB SSD |
| **Label** | `self-hosted` (universal) |
| **Storage** | local-lvm |
| **Network** | Bridge vmbr0 (DHCP) |
| **Boot** | Automatic on Proxmox reboot |

---

## 🔒 Security Hardening Applied

1. **Unprivileged Containers**
   - Non-root user (`runner`)
   - No container escalation to host

2. **Network Isolation**
   - Firewall rules via iptables
   - Egress: DNS (53), HTTPS (443) only to GitHub
   - Ingress: None (outbound only)
   - Established connections allowed

3. **Resource Limits**
   - CPU limits enforced at container level
   - Memory limits: 8GB per container
   - Swap disabled (swap: 0)

4. **Audit Logging**
   - Auditd enabled in all containers
   - Runner activity tracked
   - Work directory monitoring

5. **SSH Security**
   - SSH server installed
   - Random root password (not used)
   - Key-based auth recommended for container access

---

## 📈 Scaling Strategy

### From 4 to 6 Runners (Add 2)

```bash
# Modify script: RUNNER_COUNT=6, START_CTID=200
# Run script again (existing containers skipped automatically)
bash proxmox-runner-setup.sh
```

### From 4 to 10 Runners (Add 6)

Similar process - script handles detection of existing containers.

---

## ⚠️ Important Notes

1. **PAT Rotation**: The GitHub PAT is embedded in the deployment script (for automation). Rotate immediately after successful deployment:
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Delete old PAT
   - Create new PAT with `repo` and `workflow` scopes
   - Update script before next deployment

2. **Container Storage**: Each container uses 50GB. For 10 runners = 500GB total storage required.

3. **Network**: Ensure Proxmox host and containers can reach github.com:443

4. **Resource Planning**:
   - 4 runners = 16 CPU cores, 32GB RAM
   - 6 runners = 24 CPU cores, 48GB RAM
   - 10 runners = 40 CPU cores, 80GB RAM

---

## 📝 Post-Deployment Verification

After script execution, verify all runners are online:

```bash
# List all runners
gh api repos/brentley/cco/actions/runners --jq '.runners[] | {id, name, status, labels}'

# Expected output:
# id: 1234567890
# name: runner-01
# status: online
# labels: [self-hosted, linux, x64]
```

---

## 🔧 Health Check Script

A companion script is available for ongoing verification:

```bash
bash /tmp/proxmox-runner-health-check.sh
```

This checks:
- ✓ All containers running
- ✓ Runner services active
- ✓ Network connectivity to GitHub
- ✓ Firewall rules in place
- ✓ Disk space available
- ✓ Container resource usage

---

## 📞 Troubleshooting

### Runners Not Appearing in GitHub

1. Check registration token generation:
   ```bash
   pct exec 200 -- journalctl -u actions.runner.* -n 50
   ```

2. Verify network connectivity:
   ```bash
   pct exec 200 -- curl -I https://github.com
   ```

### Container Creation Failed

1. Verify Ubuntu template exists:
   ```bash
   pveam list local | grep ubuntu
   ```

2. Check storage space:
   ```bash
   df -h /var/lib/vz
   ```

### Runner Service Not Starting

1. Check logs:
   ```bash
   pct exec 200 -- journalctl -xe -u actions.runner.* --since "10 minutes ago"
   ```

2. Verify runner directory permissions:
   ```bash
   pct exec 200 -- ls -la /home/runner/actions-runner/
   ```

---

## 🎯 Next Steps

1. **Execute on Proxmox Host**
   ```bash
   ssh root@192.168.9.220
   cd /tmp
   bash proxmox-runner-setup.sh
   ```

2. **Verify Runners Online**
   - GitHub: Settings → Actions → Runners
   - Expected: 4 runners with "Idle" status

3. **Test First Build**
   ```bash
   gh workflow run build.yml -R brentley/cco --ref main
   gh run watch -R brentley/cco
   ```

4. **Monitor Performance**
   - Check build times on self-hosted runners vs cloud runners
   - Monitor resource usage via `pct status` and `pct exec`
   - Adjust resources if needed

---

## 📊 Current Release Status

- **Version**: 2025.11.19.1
- **Release URL**: https://github.com/brentley/cco/releases/tag/v2025.11.19.1
- **Binary**: Available for download
- **Tests**: 366/366 passing
- **Performance**: Metrics cache 700x improvement (7s → 10ms for /api/stats)

---

**Deployment can proceed immediately when ready.**
