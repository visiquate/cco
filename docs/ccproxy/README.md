# LiteLLM Proxy (ccproxy) - Native macOS Deployment

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

This directory contains comprehensive documentation for deploying LiteLLM Proxy (ccproxy) **natively on macOS without Docker**. The deployment provides Claude API compatibility using local Ollama models.

**IMPORTANT**: This ccproxy infrastructure is planned for future deployment pending hardware availability. The Claude Orchestra currently uses **direct Anthropic Claude API** (1 Opus 4.1, 37 Sonnet 4.5, 81 Haiku 4.5 agents), NOT the local Ollama models described in this documentation.

## Quick Links

- **[Native macOS Deployment Architecture](NATIVE_MACOS_DEPLOYMENT.md)** - Complete architectural overview
- **[Deployment Steps](DEPLOYMENT_STEPS.md)** - Step-by-step installation guide (15-20 minutes)
- **[Architecture Decisions](ARCHITECTURE_DECISIONS.md)** - ADRs explaining design choices
- **[Configuration File](config.yaml)** - Production-ready LiteLLM config
- **[launchd Service](com.visiquate.ccproxy.plist)** - macOS service configuration

## Architecture at a Glance

```
External Request → Traefik (Bearer Auth) → LiteLLM (localhost:8081) → Ollama (localhost:11434)
```

## Key Design Decisions

1. **No Docker**: Native Python installation via pip
2. **launchd Service**: macOS native service management
3. **Localhost Binding**: Port 8081 accessible only from localhost
4. **Traefik Authentication**: All auth handled by Traefik reverse proxy
5. **Ollama Upstream**: Local LLM models via Ollama

## Installation Summary

```bash
# Install LiteLLM
pip3 install "litellm[proxy]"

# Create directory structure
sudo mkdir -p /Users/coder/ccproxy/logs
sudo chown -R coder:staff /Users/coder/ccproxy

# Deploy configuration files
cp config.yaml /Users/coder/ccproxy/
cp com.visiquate.ccproxy.plist ~/Library/LaunchAgents/

# Start service
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist

# Verify
curl http://localhost:8081/health
```

**Total Time**: 15-20 minutes

## Documentation Structure

### 1. NATIVE_MACOS_DEPLOYMENT.md
Comprehensive architectural documentation including:
- System architecture diagrams
- Installation methods analysis
- Service management strategy
- Directory structure
- Configuration details
- Authentication model
- Traefik integration
- Comparison with Docker approach
- Advantages and disadvantages
- Troubleshooting guide
- Security considerations
- Performance considerations

### 2. DEPLOYMENT_STEPS.md
Step-by-step deployment guide including:
- Prerequisites checklist
- Installation steps with commands
- Configuration deployment
- Service installation
- Verification checklist
- Traefik configuration updates
- Troubleshooting solutions
- Service management commands
- Maintenance tasks
- Uninstall instructions

### 3. ARCHITECTURE_DECISIONS.md
Architecture Decision Records (ADRs):
- ADR-001: Native Python Deployment
- ADR-002: launchd Service Management
- ADR-003: Localhost-Only Binding
- ADR-004: File-Based Configuration
- ADR-005: No Authentication in LiteLLM
- ADR-006: Ollama as Upstream Provider
- ADR-007: Logging Strategy

### 4. config.yaml
Production-ready LiteLLM configuration:
- Server settings (host, port)
- Model configurations (Ollama integration)
- Logging configuration
- Router settings
- Comments and examples

### 5. com.visiquate.ccproxy.plist
macOS launchd service configuration:
- Auto-start on boot
- Auto-restart on failure
- Logging configuration
- Environment variables
- Resource limits

## System Requirements

- macOS (tested on latest versions)
- Python 3.8+ (Homebrew installation recommended)
- Ollama installed and running on port 11434
- Traefik configured with bearer token authentication
- Admin/sudo access for service installation

## Integration Points

### Traefik (Reverse Proxy)
- Routes external `/v1/messages` requests to LiteLLM
- Handles bearer token authentication
- Provides SSL/TLS termination
- Health check monitoring

### Ollama (LLM Provider)
- Provides local LLM models (llama2, mistral, etc.)
- Runs on port 11434
- No API keys required
- Privacy-preserving (local execution)

### LiteLLM (API Gateway)
- Provides OpenAI/Anthropic-compatible API
- Routes requests to Ollama
- Handles model aliases
- Logs requests for monitoring

## Security Model

```
┌─────────────────────────────────────────────────────────┐
│ Security Layer 1: Traefik Bearer Token Authentication   │
├─────────────────────────────────────────────────────────┤
│ Security Layer 2: Localhost-Only Binding (127.0.0.1)    │
├─────────────────────────────────────────────────────────┤
│ Security Layer 3: File Permissions (config.yaml)        │
└─────────────────────────────────────────────────────────┘
```

## Advantages Over Docker

1. **Simplicity**: No Docker daemon, simpler installation
2. **Performance**: No container overhead
3. **Integration**: Native macOS tools and debugging
4. **Updates**: Simple `pip3 install --upgrade`
5. **Resource Usage**: Lower memory and CPU usage
6. **Debugging**: Standard Python debugging tools

## Monitoring

### Health Check
```bash
curl http://localhost:8081/health
```

### Service Status
```bash
launchctl list | grep ccproxy
```

### Logs
```bash
tail -f /Users/coder/ccproxy/logs/litellm.log
tail -f /Users/coder/ccproxy/logs/stdout.log
tail -f /Users/coder/ccproxy/logs/stderr.log
```

### Process Info
```bash
lsof -i :8081
ps aux | grep litellm
```

## Common Tasks

### Restart Service
```bash
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Update LiteLLM
```bash
pip3 install --upgrade "litellm[proxy]"
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### Update Configuration
```bash
nano /Users/coder/ccproxy/config.yaml
launchctl unload ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
launchctl load ~/Library/LaunchAgents/com.visiquate.ccproxy.plist
```

### View Logs
```bash
tail -f /Users/coder/ccproxy/logs/litellm.log
```

## Troubleshooting Quick Reference

| Issue | Command | Solution |
|-------|---------|----------|
| Service won't start | `cat /Users/coder/ccproxy/logs/stderr.log` | Check error logs |
| Port in use | `lsof -i :8081` | Kill conflicting process |
| Ollama connection failed | `curl http://localhost:11434/api/tags` | Verify Ollama running |
| Config error | `python3 -c "import yaml; yaml.safe_load(open('/Users/coder/ccproxy/config.yaml'))"` | Validate YAML |
| Traefik not routing | `curl http://localhost:8081/health` | Verify LiteLLM accessible |

## Performance Metrics

- **Startup Time**: < 5 seconds
- **Memory Usage**: ~100-200 MB (depends on models)
- **Response Time**: < 100ms overhead (plus Ollama inference time)
- **Throughput**: Limited by Ollama performance

## Future Enhancements

1. **Monitoring**: Prometheus metrics export
2. **Log Aggregation**: Centralized logging
3. **High Availability**: Multiple instances with load balancing
4. **Cloud Fallback**: Fallback to cloud APIs when local unavailable
5. **Model Management**: Automated Ollama model updates
6. **Secrets Management**: External secrets manager integration

## Support

For issues or questions:
1. Check troubleshooting guides in documentation
2. Review logs in `/Users/coder/ccproxy/logs/`
3. Consult LiteLLM documentation: https://docs.litellm.ai/
4. Verify Traefik and Ollama are functioning
5. Test direct localhost access to isolate issues

## Version History

- **v1.0** (2025-11-04): Initial native macOS deployment architecture
  - Native Python installation via pip
  - launchd service management
  - Localhost-only binding
  - Traefik integration
  - Ollama upstream configuration

## License

This documentation is part of the VisiQuate infrastructure project.

## Contributing

When updating this documentation:
1. Update the relevant section
2. Test deployment steps
3. Update version history
4. Maintain ADR format for decisions
5. Keep examples up to date
