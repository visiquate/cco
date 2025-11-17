# Troubleshooting: Compile-Time Agent Embedding

Common issues and solutions for CCO's compile-time embedding system.

## Build Issues

### Build Fails: "Config file not found"

**Error:**
```
Warning: Config file not found at "/path/to/config/orchestra-config.json"
```

**Cause:**
- File doesn't exist
- Wrong relative path in build.rs
- File was deleted

**Solution:**

```bash
# Verify file exists
ls -la cco/config/orchestra-config.json

# If missing, restore from git
git checkout cco/config/orchestra-config.json

# Or copy from backup
cp cco/config/orchestra-config.json.backup cco/config/orchestra-config.json

# Rebuild
cargo clean
cargo build --release
```

### Build Fails: "Invalid JSON"

**Error:**
```
Invalid JSON in ../config/orchestra-config.json:
trailing comma at line 42 column 5
```

**Cause:**
- Syntax error in JSON file
- Missing or extra comma
- Unmatched brackets
- Invalid escape sequences

**Solution:**

```bash
# Validate JSON syntax
jq . cco/config/orchestra-config.json

# If jq not installed, use Python
python3 -m json.tool cco/config/orchestra-config.json

# Use online validator
# https://jsonlint.com/

# Fix error (example: remove trailing comma)
sed -i '' 's/,$//' cco/config/orchestra-config.json

# Rebuild
cargo build --release
```

### Build Fails: "Failed to read config"

**Error:**
```
Failed to read ../config/orchestra-config.json:
No such file or directory
```

**Cause:**
- File permissions issue
- Symbolic link broken
- Path wrong

**Solution:**

```bash
# Check file permissions
ls -la cco/config/orchestra-config.json

# Fix permissions (should be readable)
chmod 644 cco/config/orchestra-config.json

# Check if symlink exists
file cco/config/orchestra-config.json
# If symlink broken, remove and copy file

# Verify path from build.rs location
pwd  # Should be cco/
ls ../config/orchestra-config.json  # Relative path

# Rebuild
cargo build --release
```

### Build Takes Too Long

**Cause:**
- Full rebuild triggered unnecessarily
- Dependency compilation

**Solution:**

```bash
# Check what's being recompiled
cargo clean --release
cargo build --release -vv 2>&1 | grep "Compiling"

# If only changing agent files, touch config to trigger rebuild
touch cco/config/
cargo build --release  # Should be fast incremental build

# Use parallel compilation
cargo build --release -j 8  # Use 8 cores

# Check build cache
du -sh target/
# If huge, clean and rebuild
cargo clean
cargo build --release
```

## Agent Loading Issues

### No Agents Loaded

**Symptom:**
```bash
./cco run --port 3000
# Logs show: "✓ Loaded 0 agents"

curl http://localhost:3000/api/agents
# Returns: "agents": []
```

**Cause:**
- `~/.claude/agents/` directory doesn't exist
- No `.md` files in directory
- Invalid agent definitions

**Solution:**

```bash
# Check if agents directory exists
ls -la ~/.claude/agents/

# Create if missing
mkdir -p ~/.claude/agents/

# Copy sample agents from repo
cp cco/config/agents/*.md ~/.claude/agents/

# Check files are readable
ls -la ~/.claude/agents/ | head -5

# Restart server
pkill cco
./target/release/cco run --port 3000

# Verify agents loaded
sleep 2
curl http://localhost:3000/api/agents | jq '.agents | length'
# Should show 119
```

### Agent Not Found After Build

**Symptom:**
```bash
curl http://localhost:3000/api/agents/new-agent
# Returns 404: "Agent not found: new-agent"
```

**Cause:**
- Agent file not created
- Build didn't complete
- Name mismatch between file and frontmatter
- Agent file has syntax errors

**Solution:**

```bash
# Verify agent file exists
ls cco/config/agents/new-agent.md

# Check agent name in frontmatter
head -5 cco/config/agents/new-agent.md
# Should show: name: new-agent

# Verify YAML format
yq eval 'keys' cco/config/agents/new-agent.md
# Should list: name, model, description, tools

# Rebuild
cargo clean
cargo build --release

# Wait for build to complete, then test
sleep 10
./target/release/cco run --port 3000 &
sleep 2
curl http://localhost:3000/api/agents/new-agent
```

### Wrong Agent Information

**Symptom:**
```bash
curl http://localhost:3000/api/agents/chief-architect
# Returns outdated description or model
```

**Cause:**
- File modified but binary not rebuilt
- Old binary still running
- Cache not cleared

**Solution:**

```bash
# Kill old process
pkill cco

# Force rebuild
cargo clean
cargo build --release

# Verify changes in source
cat cco/config/agents/chief-architect.md | head -10

# Start new binary
./target/release/cco run --port 3000

# Test
curl http://localhost:3000/api/agents/chief-architect
```

## Runtime Issues

### Server Won't Start

**Error:**
```
Error: Failed to bind to 127.0.0.1:3000
```

**Cause:**
- Port already in use
- Permission denied
- Port out of range

**Solution:**

```bash
# Check what's using the port
lsof -i :3000
netstat -tlnp | grep 3000

# Kill process using port
kill -9 <PID>

# Or use different port
./target/release/cco run --port 3001

# Or bind to all interfaces
./target/release/cco run --host 0.0.0.0 --port 3000
```

### Server Crashes on Startup

**Error:**
```
thread 'main' panicked at 'Config validation failed'
```

**Cause:**
- Invalid configuration
- Missing required fields
- Database connection error

**Solution:**

```bash
# Check logs for detailed error
./target/release/cco run --port 3000 2>&1 | head -50

# Enable debug logging
RUST_LOG=debug ./target/release/cco run --port 3000

# Validate configs manually
jq . cco/config/orchestra-config.json

# Check database location
ls -la analytics.db
# Create if missing
touch analytics.db

# Try starting again
./target/release/cco run --port 3000
```

### API Returns Empty Agent List

**Symptom:**
```bash
curl http://localhost:3000/api/agents
# Returns: {"agents": []}
```

**Cause:**
- Agents not loaded into memory
- Server initialization incomplete

**Solution:**

```bash
# Wait for full startup
sleep 3
curl http://localhost:3000/api/agents

# Check server logs during startup
./target/release/cco run --port 3000 2>&1 | grep "Loaded"

# Verify binary includes agents
strings ./target/release/cco | grep "chief-architect" | head -1
# If no match, binary was built without agents

# Rebuild from scratch
cargo clean
cargo build --release
```

## Version Issues

### Wrong Version Displayed

**Symptom:**
```bash
./target/release/cco --version
# Shows: cco 2025.11.1 (but expected 2025.11.2)
```

**Cause:**
- Built without setting `CCO_VERSION` env var
- Using cached build
- Build script didn't run

**Solution:**

```bash
# Clear build cache
cargo clean

# Build with explicit version
CCO_VERSION=2025.11.2 cargo build --release

# Verify
./target/release/cco --version
# Should show: cco 2025.11.2

# Check health endpoint
curl http://localhost:3000/health | jq .version
```

### Version Not in Binary

**Problem:**
```bash
./target/release/cco --version
# No output or error
```

**Cause:**
- Binary corrupted
- Build incomplete
- Binary is for different project

**Solution:**

```bash
# Verify you have the right binary
file ./target/release/cco
# Should show: ELF 64-bit executable

# Check if executable
ls -la ./target/release/cco | grep -x

# Rebuild from scratch
cargo clean
cargo build --release

# Verify after build
./target/release/cco --version
```

## Performance Issues

### Slow Agent Loading

**Symptom:**
```bash
time curl http://localhost:3000/api/agents/
# Takes > 1 second
```

**Cause:**
- Server just started (agents being parsed)
- Too many files in agents directory
- Filesystem slow

**Solution:**

```bash
# Wait for server to fully initialize
sleep 5
time curl http://localhost:3000/api/agents/

# Subsequent requests should be fast (< 100ms)

# Check server startup logs
./target/release/cco run --port 3000 2>&1 | grep "✓"

# If slow, check filesystem
df -h
# Or check if NFS mounted
mount | grep agents
```

### High Memory Usage

**Symptom:**
```bash
ps aux | grep cco
# Shows high RSS/VSZ
```

**Cause:**
- Cache size too large
- Many agents with large descriptions
- Memory leak

**Solution:**

```bash
# Check cache size setting
./target/release/cco run --port 3000 --cache-size 524288000
# Default 1GB, reduce to 512MB for constrained environments

# Monitor memory over time
watch -n 1 'ps aux | grep cco | grep -v grep'

# Check what's in memory
valgrind --leak-check=full ./target/release/cco 2>&1 | head -100

# Or use top/htop
top -p $(pgrep cco)
```

## Path and Directory Issues

### Agent Files Not Found

**Symptom:**
```
Warning: Agents directory not found: /home/user/.claude/agents
```

**Cause:**
- Directory doesn't exist
- Different user running server
- Wrong home directory detected

**Solution:**

```bash
# Check current user
whoami

# Verify home directory
echo $HOME

# Check if agents directory exists
ls -la ~/.claude/agents/

# Create if missing
mkdir -p ~/.claude/agents/

# Copy agents from repo
cp cco/config/agents/*.md ~/.claude/agents/

# Verify permissions
chmod 755 ~/.claude/agents/
chmod 644 ~/.claude/agents/*.md

# Restart server
pkill cco
./target/release/cco run --port 3000
```

### Permission Denied on Agent Files

**Error:**
```
Failed to read agent file: Permission denied
```

**Cause:**
- File permissions too restrictive
- Owner mismatch
- SELinux policy

**Solution:**

```bash
# Check permissions
ls -la ~/.claude/agents/

# Fix permissions
chmod 644 ~/.claude/agents/*.md
chmod 755 ~/.claude/agents/

# If permission issue persists
sudo chown -R $(whoami):$(whoami) ~/.claude/agents/

# Restart server
pkill cco
./target/release/cco run --port 3000
```

## Diagnostic Commands

### Check Build Status

```bash
# View build script output
cargo build --release 2>&1 | tail -50

# Check if rerun-if-changed is working
touch cco/config/
cargo build --release -vv 2>&1 | grep "rerun"
```

### Verify Agent Definitions

```bash
# Count agents in source
ls cco/config/agents/*.md | wc -l

# Validate all YAML
for file in cco/config/agents/*.md; do
    echo "Checking $file..."
    yq eval 'keys' "$file" > /dev/null && echo "  OK" || echo "  ERROR"
done
```

### Check Runtime State

```bash
# Health check
curl http://localhost:3000/health

# Agent count
curl http://localhost:3000/api/agents | jq '.agents | length'

# Check specific agent
curl http://localhost:3000/api/agents/chief-architect | jq .

# View logs
tail -50 ~/.local/share/cco/logs/cco-3000.log
```

### Binary Analysis

```bash
# Check binary size
ls -lh ./target/release/cco

# List symbols (includes agent names)
nm ./target/release/cco | grep agent | head -10

# Find version string
strings ./target/release/cco | grep "2025"
```

## Getting Help

If you encounter an issue not listed here:

1. **Collect diagnostic information**:
   ```bash
   cargo --version
   rustc --version
   uname -a
   ./target/release/cco --version
   ```

2. **Enable debug logging**:
   ```bash
   RUST_LOG=debug ./target/release/cco run --port 3000 2>&1 | tee debug.log
   ```

3. **Check build.rs source**:
   - Review `/Users/brent/git/cc-orchestra/cco/build.rs`
   - Look for validation logic
   - Check file paths

4. **Test with minimal setup**:
   ```bash
   cargo clean
   cargo build --release
   ./target/release/cco run --port 3000
   curl http://localhost:3000/api/agents
   ```

5. **Open issue with**:
   - Error message and logs
   - Steps to reproduce
   - System information
   - Output of diagnostic commands

## See Also

- [EMBEDDING_ARCHITECTURE.md](EMBEDDING_ARCHITECTURE.md) - System design
- [BUILD_PROCESS.md](BUILD_PROCESS.md) - Build details
- [EMBEDDING_IMPLEMENTATION.md](EMBEDDING_IMPLEMENTATION.md) - Code details
- [DEPLOYMENT_EMBEDDING.md](DEPLOYMENT_EMBEDDING.md) - Deployment guide
- [config/agents/README.md](config/agents/README.md) - Agent development

## Summary

Most issues fall into these categories:

| Category | Common Fix |
|----------|-----------|
| Build failures | `cargo clean && cargo build --release` |
| Agent not found | Create `~/.claude/agents/` and copy files |
| Wrong version | `CCO_VERSION=X cargo build --release` |
| Port in use | Use different port: `--port 3001` |
| Permission denied | `chmod 644` agent files |
| Slow startup | Wait for full initialization: `sleep 5` |

When all else fails: clean rebuild usually fixes 90% of issues.
