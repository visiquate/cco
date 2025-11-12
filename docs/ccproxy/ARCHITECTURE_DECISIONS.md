# Architecture Decision Records (ADR) - ccproxy Native macOS Deployment

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

These ADRs document planned architectural decisions for the future ccproxy deployment. The Claude Orchestra **currently uses direct Anthropic Claude API**, not ccproxy.

---

## ADR-001: Native Python Deployment Instead of Docker

**Status**: Planned (for future deployment)

**Context**:
- Mac mini needs to run LiteLLM proxy (ccproxy) for Claude API compatibility
- Initial plan used Docker containers and Docker Compose
- Mac mini is single-purpose server with Traefik and Ollama already installed
- User explicitly requested avoiding Docker for this deployment

**Decision**:
Deploy LiteLLM using native Python pip installation instead of Docker containers.

**Rationale**:
1. **Simplicity**: No Docker daemon required, reducing system complexity
2. **Resource Efficiency**: No container overhead (memory, CPU, storage)
3. **Native Integration**: Direct access to macOS filesystem and network
4. **Debugging**: Standard Python tools available (pdb, profiling, etc.)
5. **Updates**: Simple `pip3 install --upgrade` process
6. **macOS Native**: Uses built-in launchd service manager
7. **Single Service**: Docker orchestration unnecessary for single service
8. **Existing Infrastructure**: Traefik and Ollama already running natively

**Consequences**:
- **Positive**:
  - Faster startup (no container initialization)
  - Lower resource usage
  - Simpler troubleshooting
  - Direct log file access
  - Standard Python package management
  - Better macOS integration

- **Negative**:
  - Less isolation (shares Python environment)
  - Manual dependency management required
  - Configuration is macOS-specific (not portable)
  - No container-based rollback mechanism
  - Must manage service lifecycle manually

- **Mitigations**:
  - Use virtual environment for isolation (optional)
  - Document all dependencies clearly
  - Create deployment scripts for consistency
  - Use launchd for automatic restart/recovery

**Alternatives Considered**:
1. **Docker Compose**: Rejected - adds unnecessary complexity for single service
2. **Kubernetes**: Rejected - massive overkill for single Mac mini
3. **Virtual Environment Only**: Rejected - still need service management
4. **Binary Distribution**: Rejected - LiteLLM doesn't provide binaries

---

## ADR-002: launchd for Service Management

**Status**: Accepted

**Context**:
- Native macOS deployment requires service management
- Service must auto-start on boot
- Service must auto-restart on failure
- Need standard logging and monitoring

**Decision**:
Use macOS native launchd for service management instead of alternatives.

**Rationale**:
1. **Native Solution**: Built into macOS, no installation required
2. **Auto-Start**: Handles boot-time startup automatically
3. **Auto-Restart**: KeepAlive configuration restarts on failure
4. **Logging**: Built-in stdout/stderr redirection
5. **Standard Practice**: Industry standard for macOS services
6. **Resource Control**: Can set limits and priorities
7. **User Context**: Runs as user (not root), better security

**Consequences**:
- **Positive**:
  - No additional tools required
  - Standard macOS service behavior
  - Integrated with system monitoring
  - Automatic crash recovery
  - Environment variable management
  - Process supervision included

- **Negative**:
  - plist XML configuration (less readable than YAML)
  - macOS-specific (not portable to Linux)
  - Learning curve for launchd syntax
  - Debugging requires understanding launchd
  - Must use launchctl commands

- **Mitigations**:
  - Provide well-documented plist template
  - Include common troubleshooting commands
  - Document launchctl usage patterns

**Alternatives Considered**:
1. **systemd**: Rejected - not available on macOS
2. **supervisord**: Rejected - requires additional installation
3. **pm2**: Rejected - Node.js dependency unnecessary
4. **Manual screen/tmux**: Rejected - no auto-restart, not production-ready
5. **Custom startup script**: Rejected - reinvents launchd

---

## ADR-003: Localhost-Only Binding

**Status**: Accepted

**Context**:
- LiteLLM needs to be accessible via Traefik reverse proxy
- Traefik handles bearer token authentication
- Security is critical for LLM access
- Port 8081 chosen for internal communication

**Decision**:
Bind LiteLLM to 127.0.0.1:8081 (localhost only), not 0.0.0.0:8081.

**Rationale**:
1. **Security**: No direct external network access
2. **Single Entry Point**: All requests must go through Traefik
3. **Authentication Simplification**: Traefik handles all auth
4. **Defense in Depth**: Additional layer of security
5. **Standard Pattern**: Common reverse proxy architecture
6. **Reduced Attack Surface**: Can't be accessed directly from network

**Consequences**:
- **Positive**:
  - Enhanced security (localhost only)
  - Simplified LiteLLM config (no auth needed)
  - Traefik is single auth point
  - No accidental exposure
  - Can test locally without auth

- **Negative**:
  - Cannot access from other network devices directly
  - All traffic must route through Traefik
  - Traefik becomes single point of failure
  - Debugging requires access to Mac mini
  - Cannot load balance across machines

- **Mitigations**:
  - Document direct localhost access for testing
  - Ensure Traefik is highly available
  - Provide health check endpoint
  - Document Traefik configuration clearly

**Alternatives Considered**:
1. **Bind to 0.0.0.0**: Rejected - security risk, no benefit
2. **LiteLLM Authentication**: Rejected - duplicates Traefik auth
3. **VPN Only**: Rejected - still want localhost binding
4. **Firewall Rules**: Rejected - localhost binding is simpler

---

## ADR-004: File-Based Configuration

**Status**: Accepted

**Context**:
- LiteLLM supports YAML config files and environment variables
- Need to configure multiple models and settings
- Configuration should be version-controllable
- May need to update configuration without code changes

**Decision**:
Use YAML configuration file (`config.yaml`) instead of environment variables.

**Rationale**:
1. **Complex Configuration**: YAML better for model lists and nested settings
2. **Readability**: YAML more readable than env vars for complex config
3. **Version Control**: Can track changes in git
4. **Documentation**: Self-documenting configuration
5. **Standard Practice**: LiteLLM documentation recommends YAML
6. **Validation**: Can validate YAML syntax before deployment
7. **Comments**: Can include inline documentation

**Consequences**:
- **Positive**:
  - Clear, readable configuration
  - Easy to version control
  - Self-documenting
  - Can validate before deployment
  - Supports complex structures
  - Easy to backup/restore

- **Negative**:
  - File must be managed separately
  - Secrets visible in file (need file permissions)
  - Changes require service restart
  - File path must be correct
  - YAML syntax errors possible

- **Mitigations**:
  - Set strict file permissions (600)
  - Provide validation script
  - Document restart process
  - Include syntax examples
  - Use external secrets manager for sensitive data (future)

**Alternatives Considered**:
1. **Environment Variables**: Rejected - poor for complex config
2. **Command-Line Args**: Rejected - too many settings
3. **Python Config File**: Rejected - less standard than YAML
4. **Database Config**: Rejected - overkill for static config
5. **Remote Config Service**: Rejected - adds complexity

---

## ADR-005: No Authentication in LiteLLM

**Status**: Accepted

**Context**:
- LiteLLM supports master key authentication
- Traefik already handles bearer token authentication
- Service bound to localhost only
- Need to avoid duplicate authentication layers

**Decision**:
Disable LiteLLM authentication (no master key), rely on Traefik exclusively.

**Rationale**:
1. **Single Auth Point**: Traefik handles all authentication
2. **Localhost Binding**: No external access possible
3. **Simplification**: Fewer credentials to manage
4. **Defense in Depth**: Traefik + localhost binding is sufficient
5. **Testing**: Can test locally without auth complexity
6. **Performance**: No authentication overhead in LiteLLM

**Consequences**:
- **Positive**:
  - Simpler configuration
  - Faster requests (no auth processing)
  - One set of credentials (Traefik)
  - Easier testing and debugging
  - No auth token management in LiteLLM

- **Negative**:
  - If Traefik bypassed, no auth
  - Local processes can access freely
  - Cannot have different auth per model
  - Reliance on Traefik availability
  - Less granular access control

- **Mitigations**:
  - Strict localhost binding
  - Monitor Traefik availability
  - Document security model
  - Consider adding auth if requirements change
  - Restrict file permissions on config

**Alternatives Considered**:
1. **Master Key in LiteLLM**: Rejected - duplicate auth
2. **API Keys per Model**: Rejected - unnecessary complexity
3. **OAuth2**: Rejected - overkill for internal service
4. **mTLS**: Rejected - complexity outweighs benefits

---

## ADR-006: Ollama as Upstream Provider

**Status**: Accepted

**Context**:
- Need LLM provider for Claude API compatibility
- Ollama already running on Mac mini (port 11434)
- Ollama provides local LLM models
- No cloud API keys required

**Decision**:
Configure LiteLLM to use Ollama as upstream LLM provider.

**Rationale**:
1. **Already Installed**: Ollama running on same machine
2. **No API Keys**: Avoid managing cloud API credentials
3. **Low Latency**: Local communication, no internet dependency
4. **Cost**: No per-request costs
5. **Privacy**: Data stays on local machine
6. **Simple Config**: Just point to localhost:11434

**Consequences**:
- **Positive**:
  - Fast response times (local)
  - No API costs
  - No internet dependency
  - Complete data privacy
  - Simple integration
  - Can test offline

- **Negative**:
  - Limited to Ollama-supported models
  - Model quality depends on Ollama
  - Hardware limitations (Mac mini GPU/CPU)
  - No cloud model fallback
  - Must manage Ollama separately
  - Model updates manual

- **Mitigations**:
  - Document Ollama model management
  - Consider cloud fallback (future)
  - Monitor Ollama health
  - Document hardware requirements

**Alternatives Considered**:
1. **OpenAI API**: Rejected - costs, API keys, privacy concerns
2. **Anthropic API**: Rejected - same issues as OpenAI
3. **Self-Hosted LLM**: Rejected - Ollama already provides this
4. **Multiple Providers**: Rejected - start simple, expand later

---

## ADR-007: Logging Strategy

**Status**: Accepted

**Context**:
- Need visibility into LiteLLM operation
- Debugging requires detailed logs
- Production needs efficient logging
- Log rotation required for long-term operation

**Decision**:
Implement three-tier logging: stdout, stderr, and application logs.

**Rationale**:
1. **Separation of Concerns**: Different log types in different files
2. **Debugging**: Detailed logs available when needed
3. **Production**: Can adjust verbosity without code changes
4. **launchd Integration**: Uses standard launchd logging
5. **Monitoring**: Easy to tail logs for issues
6. **History**: Logs preserved for troubleshooting

**Consequences**:
- **Positive**:
  - Clear separation of log types
  - Easy debugging
  - Standard log locations
  - Can adjust verbosity
  - Log files persist after restart
  - Easy to monitor with tail

- **Negative**:
  - Multiple log files to check
  - Disk space usage
  - Need log rotation
  - Sensitive data may be logged
  - Must manage log file growth

- **Mitigations**:
  - Implement log rotation script
  - Document log locations
  - Set appropriate verbosity levels
  - Consider log aggregation (future)
  - Restrict log file permissions

**Alternatives Considered**:
1. **Single Log File**: Rejected - harder to separate concerns
2. **syslog**: Rejected - less control, harder debugging
3. **Remote Logging**: Rejected - adds complexity initially
4. **No Logging**: Rejected - debugging would be impossible
5. **Database Logging**: Rejected - overkill for this use case

---

## Summary

These architectural decisions prioritize:
1. **Simplicity**: Native tools, minimal dependencies
2. **Security**: Localhost binding, single auth point
3. **Maintainability**: Standard practices, clear documentation
4. **Efficiency**: No container overhead, direct execution
5. **Reliability**: launchd supervision, automatic restart

The decisions support the goal of a simple, secure, maintainable deployment of LiteLLM on macOS without Docker complexity.
