# LiteLLM Proxy (ccproxy) - Native macOS Deployment Architecture

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

## Executive Summary

This document outlines the planned deployment architecture for running LiteLLM Proxy (ccproxy) **natively on macOS without Docker**. The deployment would integrate with existing Traefik routing and Ollama installation.

**IMPORTANT**: This is a future-state design pending hardware availability. The Claude Orchestra **currently uses direct Anthropic Claude API** (1 Opus 4.1, 37 Sonnet 4.5, 81 Haiku 4.5).

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Mac mini (macOS)                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Traefik Reverse Proxy (Port 8080)                    ││
│  │ - Bearer token authentication                        ││
│  │ - Routes /v1/messages → localhost:8081               ││
│  └──────────────┬───────────────────────────────────────┘  │
│                 │                                            │
│                 ▼                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ LiteLLM Proxy (ccproxy) - Port 8081                  ││
│  │ - Native Python application (pip install)            ││
│  │ - Managed by launchd service                         ││
│  │ - Config: /Users/coder/ccproxy/config.yaml          ││
│  │ - Logs: /Users/coder/ccproxy/logs/                  ││
│  └──────────────┬───────────────────────────────────────┘  │
│                 │                                            │
│                 ▼                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Ollama (Port 11434)                                   ││
│  │ - Native macOS installation                          ││
│  │ - Local LLM models                                   ││
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Installation Method: Native Python via pip

### Recommended Approach

**Installation**: Use pip3 with proxy extras for full functionality:
```bash
pip3 install "litellm[proxy]"
```

**Why This Approach:**
- ✅ Simple, standard Python package installation
- ✅ No container overhead
- ✅ Direct filesystem access for configuration
- ✅ Easy integration with macOS launchd
- ✅ Native performance
- ✅ Standard Python debugging tools available

**Alternative Tools Considered:**
- pipx: Good for isolation but unnecessary complexity for a single-purpose server
- Homebrew: No official formula available
- Binary release: Not provided by LiteLLM project
- UV package manager: Modern but less standard than pip

## Service Management: launchd

### Why launchd?

launchd is the native macOS service manager that:
- Auto-starts on boot
- Restarts on crashes
- Manages environment variables
- Handles logging
- Integrates with macOS security model

### Service Configuration

**File Location**: `~/Library/LaunchAgents/com.visiquate.ccproxy.plist`

**Key Features:**
- Runs as user service (not system daemon)
- Auto-restart on failure
- Standard output/error logging
- Environment variable management
- Working directory control

## Directory Structure

```
/Users/coder/ccproxy/
├── config.yaml              # LiteLLM configuration
├── logs/
│   ├── litellm.log         # Application logs
│   ├── stdout.log          # Standard output
│   └── stderr.log          # Error output
└── venv/                    # Optional: Python virtual environment
```

## Configuration Details

### LiteLLM Configuration (config.yaml)

**Key Settings:**
- Port: 8081 (internal, not exposed externally)
- Host: 127.0.0.1 (localhost only, Traefik routes externally)
- Upstream: Ollama at http://localhost:11434
- Authentication: Handled by Traefik (bearer token)
- Logging: File-based for debugging and monitoring

### Authentication Strategy

**Current Setup:**
- Traefik handles ALL authentication via bearer token
- LiteLLM runs without authentication (internal only)
- No external access to port 8081 (localhost binding)

**Security Model:**
```
External Request → Traefik (Auth) → LiteLLM (No Auth) → Ollama
```

**Why This Works:**
- LiteLLM only accepts connections from localhost
- Traefik validates bearer token before forwarding
- No direct external access to LiteLLM or Ollama

## Traefik Integration

### Routing Configuration

**Assumption**: Traefik is configured to route:
- Path: `/v1/messages`
- Target: `http://localhost:8081`
- Middleware: Bearer token authentication

**No Changes Required If:**
- Traefik routes to `localhost:8081` (not a Docker container name)
- Bearer token middleware is already configured
- Health checks point to `/health` endpoint

**Changes Required If:**
- Traefik routes to a Docker network (e.g., `http://litellm:8081`)
- Change to: `http://127.0.0.1:8081` or `http://localhost:8081`

## Deployment Steps

### Phase 1: Installation (5 minutes)

```bash
# 1. Install LiteLLM
pip3 install "litellm[proxy]"

# 2. Verify installation
litellm --version

# 3. Create directory structure
mkdir -p /Users/coder/ccproxy/logs

# 4. Test basic functionality
litellm --version
```

### Phase 2: Configuration (10 minutes)

```bash
# 1. Create config.yaml (see separate file)
# 2. Test configuration
litellm --config /Users/coder/ccproxy/config.yaml --test

# 3. Manual test run
litellm --config /Users/coder/ccproxy/config.yaml --port 8081 --host 127.0.0.1
```

### Phase 3: Service Setup (15 minutes)

```bash
# 1. Create launchd plist (see separate file)
# 2. Load service
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# 3. Verify service started
launchctl list | grep ccproxy

# 4. Check logs
tail -f /Users/coder/ccproxy/logs/stdout.log
```

### Phase 4: Integration Testing (10 minutes)

```bash
# 1. Test direct access (should work)
curl http://localhost:8081/health

# 2. Test via Traefik (requires bearer token)
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/v1/messages

# 3. Monitor logs
tail -f /Users/coder/ccproxy/logs/litellm.log
```

## Health Checks

### LiteLLM Health Endpoint

**URL**: `http://localhost:8081/health`

**Expected Response:**
```json
{
  "status": "healthy",
  "uptime": 3600,
  "models": ["ollama/llama2"]
}
```

### Monitoring Strategy

1. **launchd**: Auto-restart on process exit
2. **Traefik**: Health check endpoint monitoring
3. **Manual**: Log file monitoring
4. **External**: Bearer token-protected health endpoint via Traefik

## Comparison: Docker vs Native

| Aspect | Docker | Native macOS |
|--------|--------|--------------|
| Installation | docker-compose up | pip3 install |
| Service Management | Docker daemon | launchd |
| Resource Usage | Container overhead | Direct process |
| Configuration | Volume mounts | Direct filesystem |
| Logs | docker logs | Log files |
| Networking | Docker networks | localhost |
| Updates | Pull new image | pip3 install --upgrade |
| Debugging | docker exec | Standard Python tools |
| Integration | Container names | localhost:8081 |
| Auto-start | restart: unless-stopped | launchd KeepAlive |

## Advantages of Native Deployment

1. **Simplicity**: No Docker daemon required
2. **Performance**: No container overhead
3. **Integration**: Direct filesystem and network access
4. **Debugging**: Standard Python debugging tools
5. **Updates**: Simple pip upgrade process
6. **macOS Native**: Uses built-in launchd service manager

## Disadvantages vs Docker

1. **Isolation**: No container isolation
2. **Portability**: macOS-specific configuration
3. **Dependencies**: Must manage Python dependencies
4. **Cleanup**: Manual removal of files and services

## Troubleshooting Guide

### Service Won't Start

```bash
# Check launchd status
launchctl list | grep ccproxy

# View error logs
cat /Users/coder/ccproxy/logs/stderr.log

# Test manual start
/opt/homebrew/bin/python3 -m litellm --config /Users/coder/ccproxy/config.yaml --port 8081
```

### Configuration Errors

```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"

# Test configuration
litellm --config /Users/coder/ccproxy/config.yaml --test
```

### Traefik Connection Issues

```bash
# Verify LiteLLM is listening
lsof -i :8081

# Test direct connection
curl http://localhost:8081/health

# Check Traefik logs
# (command depends on Traefik installation method)
```

### Ollama Connection Issues

```bash
# Verify Ollama is running
curl http://localhost:11434/api/tags

# Check Ollama models
ollama list

# Test Ollama via LiteLLM
curl http://localhost:8081/v1/models
```

## Maintenance

### Updates

```bash
# Upgrade LiteLLM
pip3 install --upgrade "litellm[proxy]"

# Restart service
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Log Rotation

```bash
# Add to cron or launchd schedule
# Rotate logs weekly
mv /Users/coder/ccproxy/logs/litellm.log /Users/coder/ccproxy/logs/litellm.log.$(date +%Y%m%d)
gzip /Users/coder/ccproxy/logs/litellm.log.$(date +%Y%m%d)
```

### Monitoring

```bash
# Check service status
launchctl list | grep ccproxy

# Monitor logs in real-time
tail -f /Users/coder/ccproxy/logs/litellm.log

# Check resource usage
ps aux | grep litellm
```

## Architectural Decisions

### ADR-001: Native Python Deployment

**Decision**: Deploy LiteLLM using native Python pip installation instead of Docker

**Rationale**:
- Mac mini is single-purpose server (no Docker needed for other services)
- Simpler installation and maintenance
- Better integration with macOS native tools
- No Docker daemon overhead
- Easier debugging with standard Python tools

**Consequences**:
- Must manage Python dependencies manually
- Configuration is macOS-specific
- Less portable than Docker approach

### ADR-002: launchd Service Management

**Decision**: Use macOS native launchd for service management

**Rationale**:
- Native macOS service manager
- Auto-start on boot
- Auto-restart on failure
- Standard logging integration
- No additional tools required

**Consequences**:
- plist XML configuration required
- macOS-specific (not portable to Linux)
- Learning curve for launchd syntax

### ADR-003: Localhost-Only Binding

**Decision**: Bind LiteLLM to 127.0.0.1:8081 (localhost only)

**Rationale**:
- Security: No direct external access
- Traefik handles authentication
- Simplifies LiteLLM configuration (no auth needed)
- Standard reverse proxy pattern

**Consequences**:
- All external access must go through Traefik
- Cannot access LiteLLM directly from network
- Traefik becomes single point of authentication

### ADR-004: File-Based Configuration

**Decision**: Use YAML configuration file instead of environment variables

**Rationale**:
- Complex model configurations easier in YAML
- Version control friendly
- Better documentation
- Standard LiteLLM practice

**Consequences**:
- Must manage config file separately
- Secrets in config file (use file permissions)
- Changes require service restart

## Security Considerations

1. **File Permissions**: Config and logs should be readable only by service user
2. **Localhost Binding**: LiteLLM only accessible from localhost
3. **Traefik Authentication**: Bearer token required for all external access
4. **Secrets Management**: No API keys stored (Ollama is local)
5. **Log Security**: Logs may contain sensitive data (restrict access)

## Performance Considerations

1. **Native Performance**: No container overhead
2. **Python GIL**: Single-threaded Python process (may bottleneck)
3. **Ollama Upstream**: Performance depends on Ollama and hardware
4. **Logging**: File I/O may impact performance (rotate regularly)

## Conclusion

Native macOS deployment of LiteLLM is the optimal choice for this use case:
- Simpler than Docker for single-service deployment
- Better integration with existing macOS tools
- Lower resource overhead
- Easier debugging and maintenance
- Standard Python package management

The deployment integrates seamlessly with existing Traefik and Ollama installations, requiring minimal configuration changes.
