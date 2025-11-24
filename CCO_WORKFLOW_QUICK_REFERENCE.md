# CCO Release Repository - Workflow Quick Reference

## Workflow Testing Commands

### 1. Test Self-Hosted Build (Manual)
```bash
# Trigger build workflow manually
gh workflow run build.yml -R brentley/cco --ref main

# Watch the run
gh run watch -R brentley/cco

# List recent runs
gh run list -R brentley/cco -w build.yml -L 5
```

### 2. Create a Release (Two Methods)

**Method A: Via Git Tag (Automatic)**
```bash
# In source repository
cd /Users/brent/git/cc-orchestra
git tag v2025.11.19.1 -m "First public release"
git push origin v2025.11.19.1

# Watch release workflow
gh run watch -R brentley/cco
```

**Method B: Manual Workflow Trigger**
```bash
# Trigger release workflow manually
gh workflow run release.yml -R brentley/cco --ref main \
  -f version=v2025.11.19.1

# Watch the run
gh run watch -R brentley/cco
```

### 3. Test Documentation Sync
```bash
# This triggers automatically on push to main with doc changes
# Or trigger manually:
gh workflow run docs-sync.yml -R brentley/cco --ref main

# Watch the run
gh run watch -R brentley/cco
```

## Monitoring Commands

### Check Workflow Status
```bash
# List all workflows
gh workflow list -R brentley/cco

# View specific workflow runs
gh run list -R brentley/cco -w build.yml
gh run list -R brentley/cco -w release.yml
gh run list -R brentley/cco -w docs-sync.yml
```

### View Run Details
```bash
# Get run ID from list, then:
gh run view <RUN_ID> -R brentley/cco

# View logs
gh run view <RUN_ID> -R brentley/cco --log
```

### Check Repository Status
```bash
# View repo info
gh repo view brentley/cco

# Check branch protection
gh api repos/brentley/cco/branches/main/protection

# List releases
gh release list -R brentley/cco
```

## First Release Workflow

### Prerequisites
1. Self-hosted runners active and registered
2. Source code in brentley/cc-orchestra is ready
3. Build tested successfully

### Steps
```bash
# 1. Test build on self-hosted runner
gh workflow run build.yml -R brentley/cco --ref main
# Wait for completion (check gh run list -R brentley/cco)

# 2. Verify artifacts generated
gh run view <RUN_ID> -R brentley/cco
# Check artifacts section for cco-binaries

# 3. Download and verify binary
gh run download <RUN_ID> -R brentley/cco -n cco-binaries
# Test the binary locally

# 4. Create release
gh workflow run release.yml -R brentley/cco --ref main \
  -f version=v2025.11.19.1

# 5. Verify release created
gh release view v2025.11.19.1 -R brentley/cco

# 6. Test installation
curl -LO https://github.com/brentley/cco/releases/download/v2025.11.19.1/cco-<platform>
chmod +x cco-<platform>
./cco-<platform> --version
```

## Troubleshooting

### Build Workflow Not Starting
```bash
# Check runner status
gh api repos/brentley/cco/actions/runners

# Check workflow file syntax
gh workflow view build.yml -R brentley/cco
```

### Release Failed
```bash
# View logs
gh run view <RUN_ID> -R brentley/cco --log

# Check permissions
gh api repos/brentley/cco --jq '.permissions'
```

### Documentation Sync Failed
```bash
# Check for missing files
gh workflow view docs-sync.yml -R brentley/cco

# Verify file structure
gh api repos/brentley/cco/contents/docs
```

## Repository URLs

- **Repository:** https://github.com/brentley/cco
- **Actions:** https://github.com/brentley/cco/actions
- **Workflows:** https://github.com/brentley/cco/actions/workflows
- **Releases:** https://github.com/brentley/cco/releases
- **Settings:** https://github.com/brentley/cco/settings

## Workflow IDs

- **build.yml:** 208544496
- **docs-sync.yml:** 208544497
- **release.yml:** 208544498
- **dependabot:** 208544551

Use these IDs with:
```bash
gh workflow view 208544496 -R brentley/cco
gh run list -R brentley/cco -w 208544496
```

---

**Quick Test (Once Runners Active):**
```bash
gh workflow run build.yml -R brentley/cco && gh run watch -R brentley/cco
```
