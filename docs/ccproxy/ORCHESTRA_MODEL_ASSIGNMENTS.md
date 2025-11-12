# Claude Orchestra Model Assignments - TDD Pipeline

**STATUS: FUTURE-STATE ARCHITECTURE (NOT CURRENTLY DEPLOYED)**

**Date**: 2025-11-04
**Version**: 3.0 (TDD Edition - Planned)
**Planned Status**: Future deployment (pending hardware)
**Configuration**: `/Users/brent/git/cc-orchestra/config/orchestra-config.json`

**IMPORTANT**: This document describes the planned TDD pipeline using Ollama qwen models via ccproxy. The Claude Orchestra **currently uses direct Anthropic Claude API** (1 Opus 4.1, 37 Sonnet 4.5, 81 Haiku 4.5).

---

## Planned Architecture

The Claude Orchestra will use **3 Ollama qwen models** routed through **ccproxy** (LiteLLM proxy) with specific agent-to-model mappings for the TDD pipeline (once hardware is available).

**Planned Deployment**: https://coder.visiquate.com (Mac mini at 192.168.9.123)

---

## 📊 The Three Models: Deployed Configuration

| Model | API Alias | Agents | Context | Memory | Phase |
|-------|-----------|--------|---------|--------|-------|
| **qwen2.5-coder:32b-instruct** | `claude-3-5-sonnet` | 1-10 (TDD, coding, integration) | 32k | ~20GB | Phase 1 |
| **qwen-fast:latest** | `claude-3-haiku` | 11 (Credential Manager) | 32k | ~5GB | Phase 1 |
| **qwen-quality-128k:latest** | `gpt-4` | 13-15 (QA, Security, Docs) | 128k | ~35GB | Phase 2 |

### Memory Strategy

**Phase 1**: Two models loaded simultaneously
```
qwen2.5-coder (20GB) + qwen-fast (5GB) = 25GB total ✅
Both stay in memory for entire phase
```

**Phase 2**: Single model after swap
```
qwen2.5-coder unloads, qwen-quality-128k loads = 35GB ✅
On-demand loading via Ollama
```

**Health Checks**: DISABLED (prevents model thrashing)

---

## 🎯 Agent-to-Model Mapping (All 15 Agents)

### Phase 0: Independent (Claude API)
**Chief Architect**
- Model: `claude-opus-4-1` → `claude-sonnet-4-5` fallback
- NOT routed through ccproxy
- Direct Claude API access

### Phase 1: qwen2.5-coder:32b-instruct (10 Agents)
**Coding & Integration** - Routes via `claude-3-5-sonnet` API alias

1. **TDD Coding Agent** - Test-driven development specialist
2. **Python Expert** - FastAPI, Django, ML/AI integration
3. **Swift Expert** - SwiftUI, UIKit, iOS development
4. **Go Expert** - Microservices, cloud-native apps
5. **Rust Expert** - Systems programming, performance
6. **Flutter Expert** - Cross-platform mobile
7. **API Explorer** - Third-party API integration
8. **Salesforce API Expert** - Salesforce REST/SOAP/Bulk API
9. **Authentik API Expert** - OAuth2/OIDC, SAML integration
10. **DevOps Engineer** - Docker, Kubernetes, AWS, CI/CD

### Phase 1: qwen-fast:latest (1 Agent)
**Lightweight Operations** - Routes via `claude-3-haiku` API alias

11. **Credential Manager** - Secure secrets management

### Phase 2: qwen-quality-128k:latest (3 Agents)
**Quality & Reasoning** - Routes via `gpt-4` API alias

13. **QA Engineer** - Integration testing, E2E tests, autonomous fixing
14. **Security Auditor** - Vulnerability scanning, OWASP compliance
15. **Documentation Lead** - API reference, technical documentation

---

## 🔧 ccproxy Configuration

**Location**: `/Users/brent/ccproxy/config.yaml`
**Backup**: `/Users/brent/git/cc-orchestra/config/ccproxy/ccproxy-config-tdd-pipeline.yaml`

**Key Settings**:
```yaml
router_settings:
  timeout: 300
  num_retries: 0
  routing_strategy: "simple-shuffle"
  disable_cooldowns: true      # CRITICAL: Prevents health check thrashing
  allowed_fails: 1000
  cooldown_time: 0
```

**Model Aliases**:
```yaml
model_list:
  - model_name: claude-3-5-sonnet
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct

  - model_name: claude-3-haiku
    litellm_params:
      model: ollama/qwen-fast:latest

  - model_name: gpt-4
    litellm_params:
      model: ollama/qwen-quality-128k:latest
```

---

## 📈 Why These Assignments?

### qwen2.5-coder:32b-instruct for Phase 1 Coding (10 agents)
✅ **Specialized coding model** optimized for implementation
✅ **32k context** sufficient for most coding tasks
✅ **TDD test writing** and code generation
✅ **Can coexist** with qwen-fast in memory (~25GB total)

### qwen-fast:latest for Phase 1 Lightweight (1 agent)
✅ **Lightweight operations** (credential management)
✅ **Lower memory** usage (~5GB)
✅ **Parallel execution** with qwen2.5-coder
✅ **Fast responses** for simple tasks

### qwen-quality-128k:latest for Phase 2 Reasoning (3 agents)
✅ **Deep reasoning** capabilities for security analysis
✅ **128k context** for large codebases and comprehensive reviews
✅ **Complex scenarios** and edge case testing
✅ **Architectural documentation** with full context

---

## 🚀 Deployment Status

**Status**: ✅ Fully Operational

**Verified**:
- ✅ ccproxy running (PID confirmed)
- ✅ All 3 models accessible via API
- ✅ Traefik routing working
- ✅ Health checks disabled
- ✅ On-demand model loading functional
- ✅ Zero model thrashing

**Public Endpoint**: https://coder.visiquate.com/v1/chat/completions

**Test Commands**:
```bash
# Test Phase 1 coding model (qwen2.5-coder)
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-3-5-sonnet","messages":[{"role":"user","content":"Hi"}]}'

# Test Phase 1 lightweight model (qwen-fast)
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-3-haiku","messages":[{"role":"user","content":"Hi"}]}'

# Test Phase 2 reasoning model (qwen-quality-128k)
curl -s https://coder.visiquate.com/v1/chat/completions \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"Hi"}]}'
```

---

## 📚 Related Documentation

- [ORCHESTRA_ROSTER_TDD.md](../ORCHESTRA_ROSTER_TDD.md) - Complete 15-agent roster
- [TDD_AWARE_PIPELINE.md](../TDD_AWARE_PIPELINE.md) - TDD methodology
- [DEPLOYMENT_STATUS.md](../DEPLOYMENT_STATUS.md) - Deployment history
- [HEALTH_CHECK_THRASHING_FIX.md](HEALTH_CHECK_THRASHING_FIX.md) - Thrashing issue resolution
- [orchestra-config.json](../../config/orchestra-config.json) - Agent configuration

---

**Previous Versions (Deprecated)**:
- Version 1.0: Original model exploration
- Version 2.0: Three-model investigation with qwen-latest references
- Version 3.0: **Current** - Deployed TDD pipeline with correct assignments
