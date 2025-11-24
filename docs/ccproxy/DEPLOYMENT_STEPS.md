# LiteLLM Proxy (ccproxy) - Native macOS Deployment Steps

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

This guide describes the planned ccproxy deployment process. The Claude Orchestra **currently uses direct Anthropic Claude API**, not ccproxy.

---

## Prerequisites (For Future Deployment)

- macOS system (Mac mini)
- Python 3.8+ installed (Homebrew recommended)
- Ollama already installed and running on port 11434
- Traefik already configured with bearer token authentication
- Admin/sudo access for service installation

## Quick Start (15-20 minutes) - Future Deployment

### Step 1: Install LiteLLM (5 minutes)

```bash
# Install LiteLLM with proxy support
pip3 install "litellm[proxy]"

# Verify installation
litellm --version
# Expected output: litellm, version X.X.X

# Test help
litellm --help
```

### Step 2: Create Directory Structure (2 minutes)

```bash
# Create ccproxy directory
sudo mkdir -p /Users/coder/ccproxy/logs

# Set ownership (replace 'coder' with actual username if different)
sudo chown -R coder:staff /Users/coder/ccproxy

# Set permissions
chmod 755 /Users/coder/ccproxy
chmod 755 /Users/coder/ccproxy/logs
```

### Step 3: Deploy Configuration (3 minutes)

```bash
# Copy config.yaml to deployment directory
# (Assuming you have the config.yaml from this repository)
sudo cp config.yaml /Users/coder/ccproxy/config.yaml

# Set ownership
sudo chown coder:staff /Users/coder/ccproxy/config.yaml

# Set permissions (readable by owner only for security)
chmod 600 /Users/coder/ccproxy/config.yaml

# Verify configuration syntax
python3 -c "import yaml; print(yaml.safe_load(open('/Users/coder/ccproxy/config.yaml')))"
```

### Step 4: Test Manual Run (5 minutes)

```bash
# Test LiteLLM with configuration
cd /Users/coder/ccproxy
litellm --config config.yaml --port 8081 --host 127.0.0.1

# In another terminal, test the endpoint
curl http://localhost:8081/health

# Expected response:
# {"status":"healthy"}

# Test models endpoint
curl http://localhost:8081/v1/models

# Expected: List of configured models

# Stop the test run (Ctrl+C)
```

### Step 5: Install launchd Service (5 minutes)

```bash
# Copy plist to LaunchAgents directory
cp com.visiquate.ccproxy.plist ~/Library/LaunchAgents/

# Set permissions
chmod 644 ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Load the service
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Verify service started
launchctl list | grep ccproxy
# Expected output: PID    Status    Label
#                  12345  0         com.visiquate.ccproxy

# Check if port 8081 is listening
lsof -i :8081
# Expected: python3 process listening on 127.0.0.1:8081
```

### Step 6: Verify Service Logs (2 minutes)

```bash
# Check stdout log
tail -f /Users/coder/ccproxy/logs/stdout.log

# Check stderr log (in another terminal)
tail -f /Users/coder/ccproxy/logs/stderr.log

# Check application log
tail -f /Users/coder/ccproxy/logs/litellm.log

# Expected: Service startup messages, no errors
```

### Step 7: Integration Testing (5 minutes)

```bash
# Test direct access (should work)
curl http://localhost:8081/health
# Expected: {"status":"healthy"}

# Test via Traefik (requires bearer token)
# Replace YOUR_TOKEN with actual bearer token
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/v1/messages

# Expected: Response from LiteLLM (or error if not configured)

# Test model listing
curl http://localhost:8081/v1/models
# Expected: JSON list of models

# Test actual completion (optional)
curl http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama/llama2",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

## Verification Checklist

- [ ] LiteLLM installed and accessible (`litellm --version`)
- [ ] Directory structure created (`/Users/coder/ccproxy/`)
- [ ] Configuration file deployed and validated
- [ ] Manual test run successful
- [ ] Service installed in launchd
- [ ] Service running (`launchctl list | grep ccproxy`)
- [ ] Port 8081 listening (`lsof -i :8081`)
- [ ] Health endpoint responding (`curl http://localhost:8081/health`)
- [ ] Logs being written (check `/Users/coder/ccproxy/logs/`)
- [ ] Traefik routing working (if configured)
- [ ] Ollama integration working (model requests successful)

## Traefik Configuration

### Required Traefik Changes (If Needed)

If Traefik currently routes to a Docker container, update the configuration:

**Before (Docker):**
```yaml
http:
  routers:
    ccproxy:
      rule: "PathPrefix(`/v1/messages`)"
      service: ccproxy-service
      middlewares:
        - bearer-auth
  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://litellm:8081"  # Docker container name
```

**After (Native):**
```yaml
http:
  routers:
    ccproxy:
      rule: "PathPrefix(`/v1/messages`)"
      service: ccproxy-service
      middlewares:
        - bearer-auth
  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://127.0.0.1:8081"  # Localhost
```

### Traefik Health Check

Add health check to Traefik configuration:

```yaml
http:
  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://127.0.0.1:8081"
        healthCheck:
          path: "/health"
          interval: "10s"
          timeout: "3s"
```

## Troubleshooting

### Service Won't Start

**Symptoms:** `launchctl list | grep ccproxy` shows no process or negative PID

**Solution:**
```bash
# Check stderr log
cat /Users/coder/ccproxy/logs/stderr.log

# Common issues:
# 1. Python path incorrect - verify: which python3
# 2. Config file missing/invalid - verify: ls -la /Users/coder/ccproxy/config.yaml
# 3. Permissions issue - verify: ls -la /Users/coder/ccproxy
# 4. Port 8081 already in use - verify: lsof -i :8081

# Try manual start for better error messages
cd /Users/coder/ccproxy
/opt/homebrew/bin/python3 -m litellm --config config.yaml --port 8081
```

### Port Already in Use

**Symptoms:** Error message about port 8081 being in use

**Solution:**
```bash
# Find what's using port 8081
lsof -i :8081

# Kill the process if it's safe
kill -9 PID

# Or change port in config.yaml and plist file
```

### Ollama Connection Failed

**Symptoms:** Errors in logs about connecting to Ollama

**Solution:**
```bash
# Verify Ollama is running
curl http://localhost:11434/api/tags

# Check Ollama status
ollama list

# Restart Ollama if needed
# (depends on how Ollama is installed)

# Verify Ollama port in config.yaml
grep api_base /Users/coder/ccproxy/config.yaml
```

### Configuration Errors

**Symptoms:** Service starts but requests fail

**Solution:**
```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"

# Check configuration logic
litellm --config /Users/coder/ccproxy/config.yaml --test

# Enable debug logging
# Edit config.yaml: set_verbose: true
# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Traefik Not Routing

**Symptoms:** Direct curl works, but Traefik returns 502/503

**Solution:**
```bash
# Verify LiteLLM is accessible
curl http://localhost:8081/health

# Check Traefik configuration
# (command depends on Traefik installation)

# Verify Traefik service configuration points to localhost:8081
# Not a Docker container name

# Check Traefik logs for routing errors
```

## Service Management Commands

### Start Service
```bash
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Stop Service
```bash
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Restart Service
```bash
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Check Service Status
```bash
launchctl list | grep ccproxy
```

### View Logs
```bash
# Real-time logs
tail -f /Users/coder/ccproxy/logs/stdout.log
tail -f /Users/coder/ccproxy/logs/stderr.log
tail -f /Users/coder/ccproxy/logs/litellm.log

# Recent logs
tail -50 /Users/coder/ccproxy/logs/stdout.log
```

### Disable Auto-Start
```bash
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Enable Auto-Start
```bash
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

## Maintenance Tasks

### Update LiteLLM
```bash
# Upgrade to latest version
pip3 install --upgrade "litellm[proxy]"

# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Verify new version
litellm --version
```

### Update Configuration
```bash
# Edit configuration
nano /Users/coder/ccproxy/config.yaml

# Validate changes
python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"

# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Log Rotation
```bash
# Create log rotation script
cat > /Users/coder/ccproxy/rotate-logs.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d)
cd /Users/coder/ccproxy/logs
for log in *.log; do
    if [ -f "$log" ]; then
        cp "$log" "${log%.log}-${DATE}.log"
        > "$log"
        gzip "${log%.log}-${DATE}.log"
    fi
done
# Delete logs older than 30 days
find . -name "*.log.gz" -mtime +30 -delete
EOF

chmod +x /Users/coder/ccproxy/rotate-logs.sh

# Add to cron (weekly rotation)
crontab -e
# Add line: 0 0 * * 0 /Users/coder/ccproxy/rotate-logs.sh
```

### Backup Configuration
```bash
# Backup configuration and plist
cp /Users/coder/ccproxy/config.yaml ~/Desktop/ccproxy-config-backup.yaml
cp ~/Library/LaunchAgents/com.visiquate.ccproxy.plist ~/Desktop/ccproxy-plist-backup.plist
```

## Uninstall Instructions

```bash
# Stop and unload service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Remove plist
rm ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Remove application directory
sudo rm -rf /Users/coder/ccproxy

# Uninstall LiteLLM
pip3 uninstall litellm

# Remove dependencies (optional)
pip3 uninstall prisma psycopg2-binary
```

## Next Steps

1. **Monitor Performance**: Watch logs for first 24 hours
2. **Tune Configuration**: Adjust timeouts, logging based on usage
3. **Set Up Monitoring**: Consider external monitoring (Prometheus, etc.)
4. **Document Integration**: Update Traefik and Ollama documentation
5. **Plan Updates**: Schedule regular LiteLLM updates

## Support Resources

- LiteLLM Documentation: https://docs.litellm.ai/
- LiteLLM GitHub: https://github.com/BerriAI/litellm
- Issue Tracker: https://github.com/BerriAI/litellm/issues
- Ollama Documentation: https://ollama.ai/docs
- Traefik Documentation: https://doc.traefik.io/traefik/

## Feedback

This deployment guide is version 1.0. Please report issues or suggestions to improve the deployment process.
