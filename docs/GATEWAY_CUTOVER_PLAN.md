# LLM Gateway Cutover Plan

**Version:** 1.0
**Date:** 2025-12-02
**Status:** DRAFT

## Executive Summary

This document outlines the migration plan from the OLD HTTP proxy (`src/daemon/proxy/server.rs`) to the NEW LLM Gateway (`src/daemon/llm_gateway/`). The launcher already prefers the Gateway when available (lines 405-426 in `src/commands/launcher.rs`), making this a low-risk cutover.

### Key Differences

| Feature | OLD Proxy | NEW Gateway |
|---------|-----------|-------------|
| **Architecture** | TCP intercept | Axum HTTP server |
| **Streaming** | ❌ NO | ✅ YES (SSE) |
| **Environment** | `HTTP_PROXY` | `ANTHROPIC_BASE_URL` |
| **Port Discovery** | `read_proxy_port()` | `read_gateway_port()` |
| **Cost Tracking** | ❌ NO | ✅ YES (per-request) |
| **Audit Logging** | ❌ NO | ✅ YES (request/response) |
| **Multi-Provider** | Basic (Anthropic/Azure) | Full (Anthropic/Azure/DeepSeek/Ollama) |
| **Routing** | TCP-level agent detection | HTTP-level intelligent routing |

---

## Phase 1: Validation (Current State)

**Goal:** Verify Gateway is working correctly before removing Proxy fallback.

### 1.1 Verification Checklist

**Gateway Health:**
```bash
# Start daemon and check gateway initialized
cco daemon restart

# Check daemon logs for Gateway startup
tail -50 ~/.cco/daemon.log | grep -i gateway

# Expected output:
# ✅ LLM Gateway initialized successfully
# ✅ LLM Gateway started on port XXXXX
# ✅ PID file updated with gateway port: XXXXX
```

**Gateway Functionality:**
```bash
# Test gateway health endpoint
GATEWAY_PORT=$(cat ~/.cco/daemon.pid | jq -r .gateway_port)
curl http://127.0.0.1:$GATEWAY_PORT/gateway/health | jq

# Expected: {"status":"healthy","providers":{...}}

# Test metrics endpoint
curl http://127.0.0.1:$GATEWAY_PORT/gateway/metrics | jq

# Expected: {"summary":{"total_requests":...},...}
```

**Claude Code Integration:**
```bash
# Launch Claude Code and verify ANTHROPIC_BASE_URL is set
cco

# Expected in launcher output:
# Gateway: http://127.0.0.1:XXXXX (LLM routing enabled)

# Verify no fallback to proxy:
# Should NOT see: "Proxy: http://127.0.0.1:XXXXX (HTTP proxy mode)"
```

**Test Request Flow:**
```bash
# Use Claude Code to send a request
# Verify in daemon logs:
tail -f ~/.cco/daemon.log | grep -E "(Gateway|Routing request)"

# Expected:
# Routing request: agent_type=..., provider=anthropic, reason=...
# Forwarding to provider: anthropic
```

### 1.2 Success Criteria

- [ ] Gateway starts successfully on daemon launch
- [ ] Gateway port written to PID file
- [ ] Launcher discovers gateway port and sets `ANTHROPIC_BASE_URL`
- [ ] Health endpoint returns `"status":"healthy"`
- [ ] Metrics endpoint returns valid data
- [ ] Claude Code requests route through gateway (check audit logs)
- [ ] Streaming requests work (SSE format)
- [ ] Cost tracking records metrics

### 1.3 Monitoring Metrics

**Gateway Metrics to Monitor:**
- Total requests (`/gateway/metrics` - `summary.total_requests`)
- Success rate (ratio of successful vs failed requests)
- Average latency (`summary.avg_latency_ms`)
- Cost tracking (`summary.total_cost_usd`)
- Provider health (`/gateway/health` - `providers.*`)

**Baseline Period:** 2-3 days of normal usage

---

## Phase 2: Soft Cutover (Remove Proxy Fallback)

**Goal:** Make Gateway the ONLY option while keeping Proxy code for emergency rollback.

### 2.1 Code Changes

#### Change 1: Remove Proxy Fallback in Launcher
**File:** `src/commands/launcher.rs` (lines 403-426)

**Current Code:**
```rust
// Inject LLM Gateway configuration if available (preferred)
match cco::daemon::read_gateway_port() {
    Ok(gateway_port) => {
        let gateway_url = format!("http://127.0.0.1:{}", gateway_port);
        cmd.env("ANTHROPIC_BASE_URL", &gateway_url);
        println!("   Gateway: {} (LLM routing enabled)", gateway_url);
    }
    Err(e) => {
        tracing::debug!("Gateway port not found: {}", e);
        // Fall back to HTTP proxy if gateway not available
        match cco::daemon::read_proxy_port() {
            Ok(proxy_port) => {
                let proxy_url = format!("http://127.0.0.1:{}", proxy_port);
                cmd.env("HTTPS_PROXY", &proxy_url);
                cmd.env("HTTP_PROXY", &proxy_url);
                println!("   Proxy: {} (HTTP proxy mode)", proxy_url);
            }
            Err(e) => {
                tracing::warn!("No proxy or gateway available: {}", e);
            }
        }
    }
}
```

**New Code:**
```rust
// Inject LLM Gateway configuration (REQUIRED)
match cco::daemon::read_gateway_port() {
    Ok(gateway_port) => {
        let gateway_url = format!("http://127.0.0.1:{}", gateway_port);
        cmd.env("ANTHROPIC_BASE_URL", &gateway_url);
        println!("   Gateway: {} (LLM routing enabled)", gateway_url);
    }
    Err(e) => {
        // Gateway is REQUIRED - fail if not available
        eprintln!("❌ Error: LLM Gateway not available: {}", e);
        eprintln!("   The gateway must be running for Claude Code to function.");
        eprintln!("   Try: cco daemon restart");
        std::process::exit(1);
    }
}
```

#### Change 2: Add Gateway Startup Check
**File:** `src/daemon/server.rs` (lines 1208-1253)

**Add after Gateway startup (line 1253):**
```rust
// CRITICAL: Verify gateway started successfully
// Gateway is now REQUIRED for Claude Code operation
if state.llm_gateway.is_none() {
    warn!("❌ CRITICAL: LLM Gateway failed to initialize");
    warn!("   Claude Code will NOT function without the gateway.");
    warn!("   Check orchestra-config.json for llmGateway configuration.");
    warn!("   Daemon will continue, but Claude Code launches will fail.");
}
```

#### Change 3: Stop Starting Proxy Server
**File:** `src/daemon/server.rs` (lines 1183-1206)

**Comment out proxy startup (but keep code for rollback):**
```rust
// PROXY DISABLED - Gateway is now the only LLM routing mechanism
// Keeping this code commented for emergency rollback capability
// Proxy startup logic remains in src/daemon/proxy/server.rs

/*
// Start the HTTP proxy server for model routing (random port)
match super::proxy::start_proxy_server(proxy_addr).await {
    Ok(proxy_port) => {
        info!("✅ Proxy server started on port {}", proxy_port);
        // ... (keep all proxy code intact)
    }
    Err(e) => {
        warn!("Failed to start proxy server: {}", e);
    }
}
*/

info!("ℹ️  HTTP Proxy disabled - Gateway handles all LLM routing");
```

### 2.2 Feature Flag Option (Optional - Recommended)

**Add to `src/daemon/config.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    // ... existing fields ...

    /// Enable LLM Gateway (default: true)
    /// Set to false for emergency rollback to proxy
    #[serde(default = "default_gateway_enabled")]
    pub gateway_enabled: bool,

    /// Enable HTTP Proxy (default: false)
    /// Legacy proxy system, disabled in favor of gateway
    #[serde(default)]
    pub proxy_enabled: bool,
}

fn default_gateway_enabled() -> bool {
    true
}
```

**Add to `config/daemon-config.json`:**
```json
{
  "gatewayEnabled": true,
  "proxyEnabled": false
}
```

**Update launcher logic:**
```rust
// Read config to check gateway_enabled flag
let config = cco::daemon::load_config().unwrap_or_default();

if config.gateway_enabled {
    // Try gateway first
    match cco::daemon::read_gateway_port() {
        Ok(gateway_port) => { /* use gateway */ }
        Err(e) => {
            // Fail hard if gateway is required
            eprintln!("❌ Error: LLM Gateway not available: {}", e);
            std::process::exit(1);
        }
    }
} else if config.proxy_enabled {
    // Emergency fallback to proxy
    warn!("⚠️  Using legacy HTTP Proxy (gateway disabled)");
    match cco::daemon::read_proxy_port() {
        Ok(proxy_port) => { /* use proxy */ }
        Err(e) => { /* fail */ }
    }
} else {
    eprintln!("❌ Error: No LLM routing mechanism enabled");
    std::process::exit(1);
}
```

### 2.3 Testing Checklist

After implementing Phase 2 changes:

- [ ] Daemon starts without proxy server
- [ ] Gateway is the only routing mechanism
- [ ] Launcher fails with clear error if gateway unavailable
- [ ] All Claude Code requests route through gateway
- [ ] Streaming still works
- [ ] Cost tracking still works
- [ ] Audit logs still work
- [ ] Feature flag allows emergency rollback (if implemented)

### 2.4 Rollback Plan (Phase 2)

If issues arise, rollback is simple:

**Option 1: Git Revert**
```bash
# Identify the commit that made Phase 2 changes
git log --oneline -20

# Revert to previous version
git revert <commit-hash>

# Rebuild and restart
cargo build --release
cco daemon restart
```

**Option 2: Feature Flag (if implemented)**
```bash
# Edit config to re-enable proxy
vim ~/.cco/config/daemon-config.json

# Change:
# "gatewayEnabled": false,
# "proxyEnabled": true

# Restart daemon
cco daemon restart
```

**Option 3: Uncomment Proxy Code**
Edit `src/daemon/server.rs` and uncomment the proxy startup block (lines 1183-1206).

---

## Phase 3: Hard Cutover (Remove Proxy Code)

**Goal:** Completely remove legacy proxy system.

### 3.1 Files to Remove

**Primary Proxy Implementation:**
- `src/daemon/proxy/server.rs` - TCP proxy server
- `src/daemon/proxy/translator.rs` - Anthropic ↔ Azure translation
- `src/daemon/proxy/router.rs` - Agent type routing logic
- `src/daemon/proxy/mod.rs` - Module definition

**Proxy Support Code:**
- `src/daemon/lifecycle.rs` - Lines 226-246 (`read_proxy_port`)
- `src/daemon/lifecycle.rs` - Lines 277-303 (`update_proxy_port`)
- `src/daemon/lifecycle.rs` - Line 41 (PidFileContent.proxy_port field)
- `src/daemon/server.rs` - Lines 1183-1206 (proxy startup code)
- `src/daemon/server.rs` - Lines 1287-1295 (proxy status display)
- `src/daemon/server.rs` - Line 82 (DaemonState.proxy_port field)
- `src/daemon/server.rs` - Line 308 (proxy_port initialization)

### 3.2 Code Changes

#### Change 1: Remove proxy_port from PidFileContent
**File:** `src/daemon/lifecycle.rs` (line 41)

```rust
// BEFORE:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidFileContent {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub port: u16,
    #[serde(default)]
    pub proxy_port: Option<u16>,  // ← REMOVE THIS
    #[serde(default)]
    pub gateway_port: Option<u16>,
    pub version: String,
}

// AFTER:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidFileContent {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub port: u16,
    #[serde(default)]
    pub gateway_port: Option<u16>,
    pub version: String,
}
```

#### Change 2: Remove proxy_port from DaemonState
**File:** `src/daemon/server.rs` (line 82 and 308)

```rust
// Remove from struct definition:
pub struct DaemonState {
    // ... other fields ...
    pub llm_gateway: Option<super::llm_gateway::GatewayState>,
    pub gateway_port: Arc<Mutex<Option<u16>>>,
    // pub proxy_port: Arc<Mutex<Option<u16>>>,  // ← REMOVE THIS
}

// Remove from initialization:
Ok(Self {
    // ... other fields ...
    llm_gateway,
    gateway_port: Arc::new(Mutex::new(None)),
    // proxy_port: Arc::new(Mutex::new(None)),  // ← REMOVE THIS
})
```

#### Change 3: Remove Proxy Module
**File:** `src/daemon/mod.rs`

```rust
// Remove these lines:
pub mod proxy;
use proxy::start_proxy_server;
pub use lifecycle::{read_proxy_port, update_proxy_port};
```

#### Change 4: Delete Proxy Directory
```bash
# After all code changes are complete
rm -rf src/daemon/proxy/
```

### 3.3 Update Documentation

**Files to Update:**
- `README.md` - Remove proxy references
- `docs/ARCHITECTURE.md` - Update system diagram
- `docs/DEPLOYMENT.md` - Update port configuration
- `CLAUDE.md` - Update daemon description

**Search for Proxy References:**
```bash
# Find all proxy references to clean up
rg -i "proxy" --type rust --type md | grep -v "GATEWAY_CUTOVER_PLAN"
```

### 3.4 Testing Checklist

After Phase 3 changes:

- [ ] Code compiles without errors
- [ ] No references to `proxy_port` in codebase
- [ ] PID file format is valid
- [ ] Daemon starts successfully
- [ ] Gateway functions normally
- [ ] Documentation updated
- [ ] Tests pass (`cargo test`)
- [ ] No broken imports or dead code

### 3.5 Rollback Plan (Phase 3)

Phase 3 changes are **irreversible** without restoring from git. This is intentional.

**Rollback Strategy:**
```bash
# Identify the commit BEFORE Phase 3 changes
git log --oneline -20

# Create a rollback branch
git checkout -b rollback-gateway-cutover <commit-before-phase3>

# Build and test
cargo build --release

# If good, force push to main (DANGEROUS - coordinate with team)
git push --force origin rollback-gateway-cutover:main
```

**Recovery Time Objective:** < 10 minutes
**Recovery Point Objective:** Last commit before Phase 3

---

## Rollback Procedures

### Emergency Rollback (All Phases)

**Fastest Rollback (1 minute):**
```bash
# Stop daemon
cco daemon stop

# Checkout last known good commit
git checkout <last-good-commit>

# Rebuild
cargo build --release

# Start daemon
cco daemon start
```

**Feature Flag Rollback (30 seconds):**
```bash
# Edit config
echo '{"gatewayEnabled": false, "proxyEnabled": true}' > ~/.cco/config/daemon-config.json

# Restart daemon
cco daemon restart
```

### Rollback Decision Criteria

**Rollback if:**
- Gateway fails to start on >5% of systems
- Streaming requests fail >10% of the time
- Cost tracking shows incorrect data
- Critical bugs discovered (memory leaks, crashes, data loss)
- User complaints exceed 10% of active users
- Performance degradation >50% (latency, throughput)

**Do NOT rollback for:**
- Minor UI issues
- Non-critical feature missing
- Single user edge case
- Performance degradation <25%
- Issues resolved by daemon restart

---

## Migration Timeline

### Recommended Schedule

**Week 1: Phase 1 (Validation)**
- Day 1-2: Deploy to staging/dev environments
- Day 3-5: Monitor metrics, collect feedback
- Day 6-7: Fix any issues, prepare Phase 2

**Week 2: Phase 2 (Soft Cutover)**
- Day 1: Deploy Phase 2 changes
- Day 2-5: Monitor for 4 days, fix critical bugs
- Day 6-7: Prepare Phase 3 changes

**Week 3: Phase 3 (Hard Cutover)**
- Day 1: Deploy Phase 3 changes
- Day 2-7: Monitor for 6 days, ensure stability

**Total Duration:** 3 weeks (21 days)

### Go/No-Go Criteria

**Phase 1 → Phase 2:**
- [ ] Zero critical bugs discovered
- [ ] Gateway uptime >99.9%
- [ ] All success criteria met
- [ ] Team approval

**Phase 2 → Phase 3:**
- [ ] Zero critical bugs discovered
- [ ] Gateway uptime >99.9% for 4+ days
- [ ] No rollbacks required
- [ ] User feedback positive
- [ ] Team approval

---

## Risk Assessment

### High Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| Gateway fails to start | HIGH | Feature flag rollback, proxy fallback |
| Streaming breaks | HIGH | Test extensively in Phase 1 |
| PID file corruption | MEDIUM | Validation on read, backup before write |
| Port discovery fails | HIGH | Clear error messages, auto-retry |

### Low Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| Metrics slightly off | LOW | Monitor and fix in-place |
| Audit logs incomplete | LOW | Non-blocking, fix incrementally |
| Documentation gaps | LOW | Update post-deployment |

---

## Communication Plan

### Stakeholders

**Internal Team:**
- Engineering: Daily updates during cutover
- DevOps: Pre-deployment briefing
- QA: Test plan review

**External Users:**
- Notify of scheduled maintenance
- Highlight new features (streaming, cost tracking)
- Provide rollback instructions

### Communication Templates

**Pre-Deployment (48 hours before):**
```
Subject: LLM Gateway Migration - Scheduled for [DATE]

We're upgrading to a new LLM Gateway system that provides:
- ✅ Streaming support (real-time responses)
- ✅ Enhanced cost tracking
- ✅ Improved audit logging

Expected downtime: < 5 minutes
Rollback plan: Fully tested, < 1 minute recovery

Questions? Contact: [TEAM CONTACT]
```

**Post-Deployment (24 hours after):**
```
Subject: LLM Gateway Migration - Complete ✅

The LLM Gateway migration completed successfully at [TIME].

New features now available:
- Streaming responses
- Real-time cost tracking
- Detailed audit logs

Known issues: None

Questions? Contact: [TEAM CONTACT]
```

---

## Success Metrics

### Key Performance Indicators (KPIs)

**Availability:**
- Target: >99.9% uptime
- Measurement: Gateway health endpoint

**Performance:**
- Target: <500ms average latency
- Measurement: `/gateway/metrics` endpoint

**Reliability:**
- Target: <0.1% error rate
- Measurement: Success/failure ratio in audit logs

**Cost Tracking:**
- Target: 100% of requests logged
- Measurement: Compare request count vs audit log count

### Post-Migration Review

**Week 1 Metrics:**
- Total requests processed
- Average latency
- Error rate
- Rollback count
- User feedback score

**Week 4 Metrics:**
- Cost savings (if any)
- Performance improvements
- Feature adoption (streaming usage)
- Incident count

---

## Appendix A: File Reference

### Files Modified (Phase 2)

- `src/commands/launcher.rs` (lines 403-426)
- `src/daemon/server.rs` (lines 1183-1206, 1208-1253)
- `src/daemon/config.rs` (optional feature flag)
- `config/daemon-config.json` (optional feature flag)

### Files Removed (Phase 3)

- `src/daemon/proxy/server.rs` (388 lines)
- `src/daemon/proxy/translator.rs` (~300 lines)
- `src/daemon/proxy/router.rs` (~150 lines)
- `src/daemon/proxy/mod.rs` (~50 lines)

### Files Modified (Phase 3)

- `src/daemon/lifecycle.rs` (lines 41, 226-303)
- `src/daemon/server.rs` (lines 82, 308, 1183-1206, 1287-1295)
- `src/daemon/mod.rs` (proxy module imports)
- `README.md`, `docs/*.md` (documentation)

---

## Appendix B: Testing Scripts

### Gateway Validation Script
```bash
#!/bin/bash
# test-gateway.sh - Validate gateway functionality

set -e

echo "🔍 Testing LLM Gateway..."

# Start daemon
echo "Starting daemon..."
cco daemon restart
sleep 3

# Check gateway port
GATEWAY_PORT=$(cat ~/.cco/daemon.pid | jq -r .gateway_port)
if [ -z "$GATEWAY_PORT" ] || [ "$GATEWAY_PORT" = "null" ]; then
    echo "❌ Gateway port not found in PID file"
    exit 1
fi
echo "✅ Gateway port: $GATEWAY_PORT"

# Test health endpoint
echo "Testing health endpoint..."
HEALTH=$(curl -s http://127.0.0.1:$GATEWAY_PORT/gateway/health)
STATUS=$(echo $HEALTH | jq -r .status)
if [ "$STATUS" != "healthy" ]; then
    echo "❌ Gateway health check failed: $STATUS"
    exit 1
fi
echo "✅ Gateway healthy"

# Test metrics endpoint
echo "Testing metrics endpoint..."
METRICS=$(curl -s http://127.0.0.1:$GATEWAY_PORT/gateway/metrics)
TOTAL=$(echo $METRICS | jq -r .summary.total_requests)
echo "✅ Metrics endpoint working (total requests: $TOTAL)"

# Test providers endpoint
echo "Testing providers endpoint..."
PROVIDERS=$(curl -s http://127.0.0.1:$GATEWAY_PORT/gateway/providers)
echo "✅ Providers: $(echo $PROVIDERS | jq -r '.providers[].name' | tr '\n' ', ')"

echo "🎉 All gateway tests passed!"
```

### Rollback Validation Script
```bash
#!/bin/bash
# validate-rollback.sh - Ensure rollback capability works

set -e

echo "🔄 Testing rollback capability..."

# Save current git state
CURRENT_COMMIT=$(git rev-parse HEAD)
echo "Current commit: $CURRENT_COMMIT"

# Find last stable commit (before gateway changes)
LAST_STABLE=$(git log --oneline --grep="feat: add unified LLM Gateway" -1 --format=%H)
if [ -z "$LAST_STABLE" ]; then
    echo "❌ Could not find last stable commit"
    exit 1
fi
echo "Last stable commit: $LAST_STABLE"

# Test rollback
echo "Rolling back to last stable..."
git checkout $LAST_STABLE

echo "Building..."
cargo build --release

echo "Testing..."
cargo test

# Return to current state
echo "Returning to current commit..."
git checkout $CURRENT_COMMIT

echo "✅ Rollback capability validated"
```

---

## Appendix C: Configuration Examples

### Minimal Gateway Config
```json
{
  "llmGateway": {
    "providers": {
      "anthropic": {
        "enabled": true,
        "providerType": "anthropic",
        "baseUrl": "https://api.anthropic.com",
        "apiKeyRef": "ANTHROPIC_API_KEY",
        "timeoutSecs": 300,
        "maxRetries": 2
      }
    },
    "routing": {
      "defaultProvider": "anthropic",
      "fallbackChain": ["anthropic"]
    },
    "costTracking": {
      "enabled": true
    },
    "audit": {
      "enabled": true,
      "logRequestBodies": true,
      "logResponseBodies": true,
      "retentionDays": 30
    }
  }
}
```

### Multi-Provider Gateway Config
```json
{
  "llmGateway": {
    "providers": {
      "anthropic": {
        "enabled": true,
        "providerType": "anthropic",
        "baseUrl": "https://api.anthropic.com",
        "apiKeyRef": "ANTHROPIC_API_KEY"
      },
      "azure": {
        "enabled": true,
        "providerType": "azure",
        "baseUrl": "https://YOUR-RESOURCE.cognitiveservices.azure.com",
        "apiKeyRef": "AZURE_API_KEY",
        "deployment": "gpt-5.1-codex-mini",
        "apiVersion": "2024-05-01-preview"
      },
      "deepseek": {
        "enabled": true,
        "providerType": "deepseek",
        "baseUrl": "https://api.deepseek.com",
        "apiKeyRef": "DEEPSEEK_API_KEY"
      }
    },
    "routing": {
      "defaultProvider": "anthropic",
      "agentRules": {
        "code-reviewer": "azure",
        "test-engineer": "azure",
        "python-specialist": "deepseek"
      },
      "modelTierRules": {
        "opus": "anthropic",
        "sonnet": "anthropic",
        "haiku": "anthropic"
      },
      "fallbackChain": ["anthropic", "azure", "deepseek"]
    }
  }
}
```

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-02 | System | Initial cutover plan created |

---

**End of Document**
