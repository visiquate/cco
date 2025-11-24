# CRUD Classifier Fix Instructions

## Problem

The daemon binary is outdated (Nov 19) and returns "Read" for ALL commands.
The source code is correct but the running daemon doesn't have the latest implementation.

## Solution

Rebuild and restart the daemon with the current source code.

## Step-by-Step Fix

### 1. Stop the Running Daemon

```bash
cco daemon stop
```

Or kill the process directly:
```bash
ps aux | grep cco-daemon
kill <PID>
```

### 2. Rebuild the Binary

```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
```

This will create: `target/release/cco`

### 3. Install the New Binary

```bash
# Copy to ~/.local/bin/
cp target/release/cco ~/.local/bin/cco

# Verify permissions
chmod +x ~/.local/bin/cco

# Verify it's updated
ls -l ~/.local/bin/cco
# Should show today's date
```

### 4. Start the Daemon

```bash
cco daemon start
```

Or run in foreground for testing:
```bash
cco daemon run
```

### 5. Verify the Fix

```bash
# Check daemon is running
cco daemon status

# Test a few classifications
curl -X POST http://127.0.0.1:3000/api/classify \
  -H "Content-Type: application/json" \
  -d '{"command": "mkdir test"}'
# Should return: "Create"

curl -X POST http://127.0.0.1:3000/api/classify \
  -H "Content-Type: application/json" \
  -d '{"command": "rm file.txt"}'
# Should return: "Delete"

curl -X POST http://127.0.0.1:3000/api/classify \
  -H "Content-Type: application/json" \
  -d '{"command": "git commit -m test"}'
# Should return: "Update"
```

### 6. Run Full Integration Tests

```bash
cd /Users/brent/git/cc-orchestra/cco
./test-classifier-comprehensive.sh
```

Expected results:
- Overall accuracy: 88%+
- READ: 95%+
- CREATE: 90%+
- UPDATE: 85%+
- DELETE: 90%+

## Quick Verification Commands

```bash
# One-liner to test all CRUD types
echo "Testing READ:" && curl -s -X POST http://127.0.0.1:3000/api/classify -H "Content-Type: application/json" -d '{"command": "ls -la"}' | jq -r '.classification'

echo "Testing CREATE:" && curl -s -X POST http://127.0.0.1:3000/api/classify -H "Content-Type: application/json" -d '{"command": "mkdir test"}' | jq -r '.classification'

echo "Testing UPDATE:" && curl -s -X POST http://127.0.0.1:3000/api/classify -H "Content-Type: application/json" -d '{"command": "git commit -m test"}' | jq -r '.classification'

echo "Testing DELETE:" && curl -s -X POST http://127.0.0.1:3000/api/classify -H "Content-Type: application/json" -d '{"command": "rm file.txt"}' | jq -r '.classification'
```

Expected output:
```
Testing READ:
Read

Testing CREATE:
Create

Testing UPDATE:
Update

Testing DELETE:
Delete
```

## Troubleshooting

### Binary Not Updated

If the binary date is still old:
```bash
# Check which binary is being used
which cco

# Make sure ~/.local/bin is in PATH
echo $PATH

# Try absolute path
~/.local/bin/cco daemon start
```

### Daemon Won't Start

Check logs:
```bash
cat ~/.cco/daemon.log
```

Check if port is in use:
```bash
lsof -i :3000
```

### Model Not Loading

Check model exists:
```bash
ls -lh ~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
```

If missing, daemon will download it automatically on first start.

### Still Getting Wrong Classifications

1. Verify daemon is using new binary:
   ```bash
   ps aux | grep cco
   ls -l /proc/<PID>/exe  # On Linux
   ```

2. Check daemon health:
   ```bash
   curl http://127.0.0.1:3000/health | jq .
   ```

3. Check logs for errors:
   ```bash
   tail -100 ~/.cco/daemon.log
   ```

4. Try a daemon restart:
   ```bash
   cco daemon stop
   sleep 2
   cco daemon start
   ```

## Expected Timeline

- Build: 2-3 minutes (Rust compilation)
- Install: <1 second
- Start: 2-5 seconds
- Model load: <1 second (if already downloaded)
- First classification: <20ms
- Full test suite: ~30 seconds

## Success Criteria

After fix:
- ✅ All CRUD types classified correctly
- ✅ Overall accuracy ≥ 88%
- ✅ Latency <200ms (typically 8-20ms)
- ✅ Confidence scores reasonable (0.8-1.0)
- ✅ Ready for production release

## Post-Fix Actions

1. Document the fix in changelog
2. Add CI/CD checks to prevent this issue
3. Consider adding build version checks in daemon
4. Update deployment documentation
5. Notify team of successful fix

## Prevention

To prevent this issue in the future:

1. **Always rebuild after code changes**:
   ```bash
   cargo build --release && cp target/release/cco ~/.local/bin/
   ```

2. **Add version checking**:
   - Daemon could check if binary is older than source
   - Health endpoint could show build timestamp
   - CI/CD could verify binary version

3. **Use make/script for deployment**:
   ```bash
   make install  # Builds and installs in one step
   ```

4. **Add build timestamp to version**:
   ```bash
   cco --version
   # Should show: 2025.11.4+1b4dcc8 (built 2025-11-24 10:15:00)
   ```

---

**Last Updated**: November 24, 2025
**Status**: Ready to execute
**Estimated Time**: 5 minutes
