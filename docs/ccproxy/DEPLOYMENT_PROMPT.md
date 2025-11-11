# CCPROXY DEPLOYMENT PROMPT

**Mission**: Deploy LiteLLM Proxy (ccproxy) natively on macOS to serve local Ollama models through a Traefik reverse proxy with bearer token authentication.

**Target**: Mac mini (native installation, no Docker)
**Timeline**: 15-20 minutes
**Environment**: macOS with Python 3.8+, Ollama running on port 11434, Traefik on port 8080

---

## ARCHITECTURE OVERVIEW

```
┌─────────────────────────────────────────────────────┐
│                    Mac mini (macOS)                  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────────────────────────────────┐  │
│  │ Traefik Reverse Proxy (Port 8080)            │  │
│  │ • Bearer token authentication                │  │
│  │ • Routes /v1/messages → localhost:8081       │  │
│  └──────────────┬───────────────────────────────┘  │
│                 │                                   │
│                 ▼                                   │
│  ┌──────────────────────────────────────────────┐  │
│  │ LiteLLM Proxy (ccproxy) - Port 8081          │  │
│  │ • Native Python application (pip install)    │  │
│  │ • Managed by launchd service                 │  │
│  │ • Config: /Users/coder/ccproxy/config.yaml   │  │
│  │ • Logs: /Users/coder/ccproxy/logs/          │  │
│  └──────────────┬───────────────────────────────┘  │
│                 │                                   │
│                 ▼                                   │
│  ┌──────────────────────────────────────────────┐  │
│  │ Ollama (Port 11434)                          │  │
│  │ • Native macOS installation                  │  │
│  │ • Local LLM models                           │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Security Model**: External Request → Traefik (Bearer Token Auth) → LiteLLM (No Auth, Localhost Only) → Ollama

---

## DEPLOYMENT PHASES

### PHASE 1: INSTALLATION (5 minutes)

**Step 1.1: Install LiteLLM with proxy support**
```bash
pip3 install "litellm[proxy]"
```

**Step 1.2: Verify installation**
```bash
litellm --version
# Expected output: litellm, version X.X.X
```

**Step 1.3: Confirm help system works**
```bash
litellm --help
```

---

### PHASE 2: DIRECTORY STRUCTURE (2 minutes)

**Step 2.1: Create deployment directories**
```bash
sudo mkdir -p /Users/coder/ccproxy/logs
```

**Step 2.2: Set ownership and permissions**
```bash
sudo chown -R coder:staff /Users/coder/ccproxy
chmod 755 /Users/coder/ccproxy
chmod 755 /Users/coder/ccproxy/logs
```

---

### PHASE 3: CONFIGURATION FILES (3 minutes)

**Step 3.1: Create config.yaml**

Create the file `/Users/coder/ccproxy/config.yaml` with the following content:

```yaml
# LiteLLM Proxy Configuration for ccproxy
# Native macOS Deployment
# Location: /Users/coder/ccproxy/config.yaml

# General Settings
general_settings:
  # Master key for LiteLLM admin access (optional, handled by Traefik)
  # master_key: "your-admin-key-here"

  # Completion fallback models (if primary fails)
  # fallback_models:
  #   - "ollama/mistral"

  # Set to true for detailed debugging
  litellm_settings:
    drop_params: true  # Drop unsupported params instead of failing
    set_verbose: false  # Set to true for debugging

# Server Configuration
server:
  host: "127.0.0.1"  # Localhost only (Traefik handles external access)
  port: 8081         # Internal port (not exposed externally)

  # Health check endpoint (used by Traefik)
  # Available at: http://localhost:8081/health

# Model Configuration
model_list:
  # Ollama models available locally
  - model_name: claude-3-5-sonnet  # User-facing alias (Anthropic-compatible)
    litellm_params:
      model: ollama/llama2  # Actual Ollama model
      api_base: http://localhost:11434  # Ollama endpoint
      stream: true

  - model_name: gpt-4  # OpenAI-compatible alias
    litellm_params:
      model: ollama/llama2
      api_base: http://localhost:11434
      stream: true

  - model_name: ollama/llama2  # Direct Ollama access
    litellm_params:
      model: ollama/llama2
      api_base: http://localhost:11434
      stream: true

  # Add more Ollama models as needed:
  # - model_name: mistral
  #   litellm_params:
  #     model: ollama/mistral
  #     api_base: http://localhost:11434
  #     stream: true

# Logging Configuration
logging:
  # Log to file for monitoring and debugging
  log_file: /Users/coder/ccproxy/logs/litellm.log
  log_level: INFO  # Options: DEBUG, INFO, WARNING, ERROR, CRITICAL

  # Request/response logging (use with caution in production)
  request_log: true
  response_log: false  # Set to true for debugging (may log sensitive data)

# Router Configuration
router_settings:
  # Timeout settings
  timeout: 300  # 5 minutes for long-running requests

  # Retry settings
  num_retries: 0  # Don't retry (Ollama is local)

  # Routing strategy
  routing_strategy: "simple-shuffle"  # Random selection if multiple models

# Cost Tracking (Optional)
# litellm_settings:
#   success_callback: ["supabase"]  # Track usage in Supabase
#   failure_callback: ["supabase"]

# Environment Variables (Optional)
# environment_variables:
#   OLLAMA_API_BASE: "http://localhost:11434"

# Rate Limiting (Optional - disabled for local use)
# rate_limit:
#   enabled: false

# Callbacks (Optional)
# litellm_settings:
#   callbacks: ["prometheus"]  # Export metrics to Prometheus

# Notes:
# 1. No authentication configured (Traefik handles bearer token auth)
# 2. Bound to localhost only (no external access without Traefik)
# 3. All Ollama models should be pulled before configuration:
#    ollama pull llama2
#    ollama pull mistral
# 4. Add more model aliases as needed for API compatibility
# 5. Adjust log_level to DEBUG for troubleshooting
```

**Step 3.2: Set config file permissions**
```bash
sudo cp /path/to/config.yaml /Users/coder/ccproxy/config.yaml
sudo chown coder:staff /Users/coder/ccproxy/config.yaml
chmod 600 /Users/coder/ccproxy/config.yaml
```

**Step 3.3: Validate configuration syntax**
```bash
python3 -c "import yaml; print(yaml.safe_load(open('/Users/coder/ccproxy/config.yaml')))"
# Expected: Should print the YAML configuration without errors
```

---

### PHASE 4: MANUAL TESTING (5 minutes)

**Step 4.1: Start LiteLLM manually**
```bash
cd /Users/coder/ccproxy
litellm --config config.yaml --port 8081 --host 127.0.0.1
```

**Step 4.2: In a new terminal, test the health endpoint**
```bash
curl http://localhost:8081/health
# Expected response: {"status":"healthy"}
```

**Step 4.3: Test models endpoint**
```bash
curl http://localhost:8081/v1/models
# Expected: JSON list of configured models
```

**Step 4.4: (Optional) Test a completion request**
```bash
curl http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama/llama2",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

**Step 4.5: Stop the manual test**
```bash
# Press Ctrl+C in the terminal where LiteLLM is running
```

---

### PHASE 5: LAUNCHD SERVICE SETUP (5 minutes)

**Step 5.1: Create the plist file**

Create the file `~/Library/LaunchAgents/com.visiquate.ccproxy.plist` with the following content:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Service Identification -->
    <key>Label</key>
    <string>com.visiquate.ccproxy</string>

    <!-- Program to Execute -->
    <key>ProgramArguments</key>
    <array>
        <!-- Use full path to Python3 -->
        <string>/opt/homebrew/bin/python3</string>
        <string>-m</string>
        <string>litellm</string>
        <string>--config</string>
        <string>/Users/coder/ccproxy/config.yaml</string>
        <string>--port</string>
        <string>8081</string>
        <string>--host</string>
        <string>127.0.0.1</string>
    </array>

    <!-- Working Directory -->
    <key>WorkingDirectory</key>
    <string>/Users/coder/ccproxy</string>

    <!-- Auto-Start Configuration -->
    <key>RunAtLoad</key>
    <true/>

    <!-- Keep Service Alive (Auto-Restart on Crash) -->
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>  <!-- Restart even on clean exit -->
        <key>Crashed</key>
        <true/>   <!-- Restart on crash -->
    </dict>

    <!-- Logging Configuration -->
    <key>StandardOutPath</key>
    <string>/Users/coder/ccproxy/logs/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/Users/coder/ccproxy/logs/stderr.log</string>

    <!-- Environment Variables -->
    <key>EnvironmentVariables</key>
    <dict>
        <!-- Python environment -->
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>

        <!-- Python settings -->
        <key>PYTHONUNBUFFERED</key>
        <string>1</string>

        <!-- LiteLLM settings (optional) -->
        <key>LITELLM_LOG</key>
        <string>INFO</string>

        <!-- Home directory for Python -->
        <key>HOME</key>
        <string>/Users/coder</string>
    </dict>

    <!-- Resource Limits (Optional) -->
    <key>SoftResourceLimits</key>
    <dict>
        <!-- Max open files -->
        <key>NumberOfFiles</key>
        <integer>1024</integer>
    </dict>

    <!-- Process Priority -->
    <key>Nice</key>
    <integer>0</integer>  <!-- Normal priority -->

    <!-- Throttling (Prevent Rapid Restart Loops) -->
    <key>ThrottleInterval</key>
    <integer>10</integer>  <!-- Wait 10 seconds between restarts -->

    <!-- Process Type -->
    <key>ProcessType</key>
    <string>Interactive</string>

    <!-- Notes:
         1. This is a LaunchAgent (runs as user, not system daemon)
         2. Place in: ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
         3. Load with: launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
         4. Unload with: launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
         5. View status: launchctl list | grep ccproxy
         6. Logs: tail -f /Users/coder/ccproxy/logs/stdout.log
    -->
</dict>
</plist>
```

**Step 5.2: Set plist file permissions**
```bash
chmod 644 ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

**Step 5.3: Load the service**
```bash
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

**Step 5.4: Verify service is running**
```bash
launchctl list | grep ccproxy
# Expected output: Shows PID and "com.visiquate.ccproxy"
```

**Step 5.5: Verify port is listening**
```bash
lsof -i :8081
# Expected: python3 process listening on 127.0.0.1:8081
```

---

### PHASE 6: SERVICE VERIFICATION (2 minutes)

**Step 6.1: Check stdout log**
```bash
tail -f /Users/coder/ccproxy/logs/stdout.log
# Should see startup messages
```

**Step 6.2: Check stderr log (new terminal)**
```bash
tail -f /Users/coder/ccproxy/logs/stderr.log
# Should be empty or minimal
```

**Step 6.3: Check application log**
```bash
tail -f /Users/coder/ccproxy/logs/litellm.log
# Should see INFO level messages
```

---

### PHASE 7: INTEGRATION TESTING (5 minutes)

**Step 7.1: Test direct localhost access**
```bash
curl http://localhost:8081/health
# Expected: {"status":"healthy"}
```

**Step 7.2: Test models endpoint**
```bash
curl http://localhost:8081/v1/models
# Expected: JSON list of configured models
```

**Step 7.3: Verify Traefik routing (requires bearer token)**

Replace `YOUR_TOKEN` with actual Traefik bearer token:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/v1/messages
# Expected: Response from LiteLLM (or appropriate error if not configured)
```

**Step 7.4: Optional - Test actual completion**
```bash
curl http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama/llama2",
    "messages": [{"role": "user", "content": "What is 2+2?"}]
  }'
# Expected: Streaming response from Ollama model
```

---

## TRAEFIK INTEGRATION

### Current Configuration Check

Verify that Traefik is routing to the correct endpoint. If Traefik was previously configured for Docker, update the service URL.

**Check current Traefik configuration** (command depends on your Traefik setup):
```bash
# This varies by installation method - check your Traefik config file
cat /path/to/traefik/config.yml | grep -A 5 "ccproxy"
```

**If Traefik is routing to Docker container** (update required):

Before:
```yaml
http:
  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://litellm:8081"  # Docker container name
```

After:
```yaml
http:
  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://127.0.0.1:8081"  # Localhost
        healthCheck:
          path: "/health"
          interval: "10s"
          timeout: "3s"
```

**If Traefik routing is already correct** (no changes needed):

Traefik should already be configured to:
- Route `/v1/messages` to `http://127.0.0.1:8081`
- Validate bearer token authentication
- Check `/health` endpoint periodically

---

## TROUBLESHOOTING

### Service Won't Start

**Symptom**: `launchctl list | grep ccproxy` shows no process or negative PID

**Solutions**:

1. Check error log:
```bash
cat /Users/coder/ccproxy/logs/stderr.log
```

2. Common issues and fixes:

   **Python path incorrect:**
   ```bash
   which python3
   # Update plist file if path differs from /opt/homebrew/bin/python3
   ```

   **Config file missing/invalid:**
   ```bash
   ls -la /Users/coder/ccproxy/config.yaml
   python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"
   ```

   **Permission issues:**
   ```bash
   ls -la /Users/coder/ccproxy
   sudo chown -R coder:staff /Users/coder/ccproxy
   ```

   **Port 8081 already in use:**
   ```bash
   lsof -i :8081
   # Kill the process if safe, or change port in config.yaml and plist
   ```

3. Try manual start for detailed error messages:
```bash
cd /Users/coder/ccproxy
/opt/homebrew/bin/python3 -m litellm --config config.yaml --port 8081
```

### Port Already in Use

**Symptom**: Error about port 8081 being in use

**Solution**:
```bash
lsof -i :8081
# Kill the process: kill -9 PID
# Or change port in config.yaml and plist file
```

### Ollama Connection Failed

**Symptom**: Errors in logs about connecting to Ollama

**Solutions**:

1. Verify Ollama is running:
```bash
curl http://localhost:11434/api/tags
# Expected: JSON list of available models
```

2. Check Ollama status:
```bash
ollama list
```

3. Verify Ollama port in config.yaml:
```bash
grep api_base /Users/coder/ccproxy/config.yaml
# Should show: api_base: http://localhost:11434
```

4. Restart Ollama if needed (depends on installation)

### Configuration Validation Errors

**Symptom**: Service starts but requests fail

**Solutions**:

1. Validate YAML syntax:
```bash
python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"
```

2. Test configuration:
```bash
litellm --config /Users/coder/ccproxy/config.yaml --test
```

3. Enable debug logging (for troubleshooting only):
```bash
# Edit config.yaml and set: set_verbose: true
# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Traefik Not Routing

**Symptom**: Direct curl works, but Traefik returns 502/503

**Solutions**:

1. Verify LiteLLM is accessible:
```bash
curl http://localhost:8081/health
```

2. Verify Traefik configuration points to localhost:8081:
```bash
cat /path/to/traefik/config | grep -A 3 "ccproxy-service"
```

3. Check Traefik logs (method depends on installation)

4. Ensure health checks pass:
```bash
curl -v http://localhost:8081/health
# Should return 200 status
```

---

## SERVICE MANAGEMENT COMMANDS

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

### View Logs (Real-time)
```bash
tail -f /Users/coder/ccproxy/logs/stdout.log
tail -f /Users/coder/ccproxy/logs/stderr.log
tail -f /Users/coder/ccproxy/logs/litellm.log
```

### View Recent Logs
```bash
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

---

## MAINTENANCE

### Update LiteLLM
```bash
pip3 install --upgrade "litellm[proxy]"
# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Update Configuration
```bash
nano /Users/coder/ccproxy/config.yaml
# After editing, validate
python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"
# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Log Rotation (Weekly)
```bash
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

# Add to crontab
crontab -e
# Add line: 0 0 * * 0 /Users/coder/ccproxy/rotate-logs.sh
```

### Backup Configuration
```bash
cp /Users/coder/ccproxy/config.yaml ~/Desktop/ccproxy-config-backup.yaml
cp ~/Library/LaunchAgents/com.visiquate.ccproxy.plist ~/Desktop/ccproxy-plist-backup.plist
```

---

## UNINSTALL (If Needed)

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

---

## SUCCESS CRITERIA CHECKLIST

Complete ALL of the following to confirm successful deployment:

- [ ] **Installation**: `litellm --version` returns version number
- [ ] **Directory**: `/Users/coder/ccproxy/` exists with proper permissions
- [ ] **Configuration**: config.yaml created and validates without errors
- [ ] **Manual Test**: Manual run with `litellm` command works and responds to health check
- [ ] **Service Install**: plist file placed in `~/Library/LaunchAgents/`
- [ ] **Service Running**: `launchctl list | grep ccproxy` shows running process with PID
- [ ] **Port Listening**: `lsof -i :8081` shows python3 listening on 127.0.0.1:8081
- [ ] **Health Check**: `curl http://localhost:8081/health` returns `{"status":"healthy"}`
- [ ] **Logs Generated**: `/Users/coder/ccproxy/logs/` contains stdout.log, stderr.log, litellm.log
- [ ] **Models Available**: `curl http://localhost:8081/v1/models` returns JSON list
- [ ] **Ollama Connected**: Model requests work (optional, test with completion request)
- [ ] **Traefik Routing**: Traefik configuration points to `http://127.0.0.1:8081`
- [ ] **Bearer Auth**: Bearer token-protected requests through Traefik work (with valid token)

---

## QUICK REFERENCE

| Task | Command |
|------|---------|
| Install LiteLLM | `pip3 install "litellm[proxy]"` |
| Create directories | `sudo mkdir -p /Users/coder/ccproxy/logs && sudo chown -R coder:staff /Users/coder/ccproxy` |
| Validate config | `python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"` |
| Test manual run | `cd /Users/coder/ccproxy && litellm --config config.yaml --port 8081 --host 127.0.0.1` |
| Load service | `launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist` |
| Check service status | `launchctl list \| grep ccproxy` |
| Check port listening | `lsof -i :8081` |
| Health check | `curl http://localhost:8081/health` |
| View logs | `tail -f /Users/coder/ccproxy/logs/stdout.log` |
| Restart service | `launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist && launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist` |
| Update LiteLLM | `pip3 install --upgrade "litellm[proxy]"` && restart service |

---

## ARCHITECTURE DECISIONS

### Why Native Python (Not Docker)
- ✅ Simpler installation and maintenance
- ✅ No container overhead or Docker daemon needed
- ✅ Better integration with macOS native tools
- ✅ Direct filesystem access for configuration
- ✅ Standard Python debugging tools available

### Why launchd (Not systemd/supervisor)
- ✅ Native macOS service manager
- ✅ Auto-start on boot
- ✅ Auto-restart on crashes
- ✅ Standard logging integration
- ✅ No additional tools required

### Why Localhost-Only Binding
- ✅ Security: No direct external access
- ✅ Traefik handles authentication
- ✅ Simplifies LiteLLM configuration
- ✅ Standard reverse proxy pattern

### Why File-Based Configuration
- ✅ Complex model configurations easier in YAML
- ✅ Version control friendly
- ✅ Better documentation
- ✅ Standard LiteLLM practice

---

## SUPPORT RESOURCES

- **LiteLLM Docs**: https://docs.litellm.ai/
- **LiteLLM GitHub**: https://github.com/BerriAI/litellm
- **Issue Tracker**: https://github.com/BerriAI/litellm/issues
- **Ollama Docs**: https://ollama.ai/docs
- **Traefik Docs**: https://doc.traefik.io/traefik/

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
**Platform**: macOS
**Status**: Production Ready
