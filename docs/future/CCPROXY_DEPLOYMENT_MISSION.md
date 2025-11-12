# ccproxy Deployment Mission for Claude Orchestra

**Mission**: Deploy ccproxy (LiteLLM) on Mac mini (coder.visiquate.com) to route Claude Code requests intelligently between Ollama (coding) and Claude API (architecture) with automatic fallback.

**Server**: Mac mini running macOS (accessible via coder.visiquate.com)
**Deployment Location**: `/Users/coder/ccproxy/` (or appropriate Mac user home directory)
**Integration**: Behind existing Traefik on port 8080

---

## 🎯 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Developer's Machine                          │
│                                                                 │
│  Claude Code → ANTHROPIC_BASE_URL=https://coder.visiquate.com   │
└────────────────────────────┬────────────────────────────────────┘
                                                                  │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  coder.visiquate.com:8080                       │
│                      (Traefik Proxy)                            │
│                                                                 │
│  ✅ TLS Termination                                              │
│  ✅ Bearer Token Authentication (already configured)             │
│  ✅ Route /v1/messages → ccproxy:8081                            │
└────────────────────────────┬────────────────────────────────────┘
                                                                  │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ccproxy:8081 (LiteLLM)                       │
│                      Internal Service                           │
│                                                                 │
│  📋 Routing Logic:                                               │
│  • Coding tasks → Ollama qwen-fast (7B)                         │
│  • Architecture tasks → Claude Sonnet 4.5                       │
│  • Fallback: Ollama down → Claude Haiku                         │
└──────────────┬────────────────────────────┬─────────────────────┘
               │                                                  │
               ▼                            ▼
    ┌──────────────────┐        ┌──────────────────────┐
    │ Ollama:11434     │        │ api.anthropic.com    │
    │ qwen-fast 7B     │        │ Claude Sonnet 4.5    │
    │ qwen-quality 32B │        │ Claude Haiku         │
    └──────────────────┘        └──────────────────────┘
```

---

## 🏗️ Infrastructure Components

### 1. Existing Infrastructure (Do Not Modify)
- **Traefik**: Running on port 8080, handles TLS and bearer token auth
- **Ollama**: Running on port 11434 with two models
  - `qwen-fast`: qwen2.5-coder:7b-instruct (fast, 32k context)
  - `qwen-quality`: qwen2.5-coder:32b-instruct-128k (quality, 128k context)
- **Docker Network**: `coder_network`
- **Bearer Token**: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`

### 2. New Infrastructure (To Deploy)
- **ccproxy**: LiteLLM proxy service on internal port 8081
- **Docker Compose**: Managed deployment with health checks
- **Configuration**: LiteLLM config.yaml with routing rules

---

## 📦 Implementation Details

### Docker Compose Configuration

Create `/Users/coder/ccproxy/docker-compose.yml`:

```yaml
version: '3.8'

services:
  ccproxy:
    image: ghcr.io/berriai/litellm:main-latest
    container_name: ccproxy
    restart: unless-stopped
    ports:
      - "8081:8081"  # Internal only
    volumes:
      - ./config.yaml:/app/config.yaml:ro
      - ./logs:/app/logs
    environment:
      - LITELLM_MODE=PRODUCTION
      - LITELLM_LOG=INFO
      - LITELLM_PORT=8081
      - LITELLM_CONFIG=/app/config.yaml
      # Claude API key (secure)
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    networks:
      - coder_network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.ccproxy.rule=PathPrefix(`/v1/messages`)"
      - "traefik.http.routers.ccproxy.entrypoints=web"
      - "traefik.http.services.ccproxy.loadbalancer.server.port=8081"
      - "traefik.docker.network=coder_network"

networks:
  coder_network:
    external: true
```

### LiteLLM Configuration

Create `/Users/coder/ccproxy/config.yaml`:

```yaml
model_list:
  # Ollama Models (for coding tasks)
  - model_name: ollama/qwen-fast
    litellm_params:
      model: ollama/qwen2.5-coder:7b-instruct
      api_base: http://host.docker.internal:11434
      api_key: none
      max_tokens: 32768
      temperature: 0.7
    model_info:
      mode: completion
      supports_function_calling: true

  - model_name: ollama/qwen-quality
    litellm_params:
      model: ollama/qwen2.5-coder:32b-instruct-128k
      api_base: http://host.docker.internal:11434
      api_key: none
      max_tokens: 131072
      temperature: 0.7
    model_info:
      mode: completion
      supports_function_calling: true

  # Claude Models (for architecture tasks and fallback)
  - model_name: claude-sonnet-4.5
    litellm_params:
      model: anthropic/claude-sonnet-4-5-20250929
      api_key: ${ANTHROPIC_API_KEY}
      max_tokens: 200000
      temperature: 1.0
    model_info:
      mode: chat
      supports_function_calling: true

  - model_name: claude-haiku
    litellm_params:
      model: anthropic/claude-3-5-haiku-20241022
      api_key: ${ANTHROPIC_API_KEY}
      max_tokens: 200000
      temperature: 1.0
    model_info:
      mode: chat
      supports_function_calling: true

# Routing rules
router_settings:
  routing_strategy: usage-based-routing
  fallbacks:
    - ollama/qwen-fast: ["claude-haiku"]
    - ollama/qwen-quality: ["claude-haiku"]

  # Automatic routing based on keywords
  model_group_alias:
    coding:
      - ollama/qwen-fast
      - ollama/qwen-quality
      - claude-haiku  # fallback
    architecture:
      - claude-sonnet-4.5

# General settings
general_settings:
  master_key: "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
  database_url: "sqlite:////app/logs/litellm.db"

  # Cost tracking
  max_budget: 1000
  budget_duration: 30d

  # Rate limiting
  tpm_limit: 1000000
  rpm_limit: 10000

  # Logging
  success_callback: ["langfuse"]
  failure_callback: ["langfuse"]

  # Health checks
  health_check_interval: 30

litellm_settings:
  set_verbose: true
  drop_params: true
  json_logs: true
```

### Traefik Configuration Update

Update existing Traefik configuration to route `/v1/messages` to ccproxy:

**File**: `/Users/coder/traefik/dynamic/ccproxy.yml` (create new file)

```yaml
http:
  routers:
    ccproxy-router:
      rule: "PathPrefix(`/v1/messages`)"
      service: ccproxy-service
      entryPoints:
        - web
      middlewares:
        - bearer-auth  # Use existing bearer token middleware

  services:
    ccproxy-service:
      loadBalancer:
        servers:
          - url: "http://ccproxy:8081"
        healthCheck:
          path: /health
          interval: 30s
          timeout: 10s
```

### Environment Variables

Create `/Users/coder/ccproxy/.env`:

```bash
# Claude API Key (from credential manager)
ANTHROPIC_API_KEY=<retrieve-from-credential-manager>

# LiteLLM Configuration
LITELLM_MODE=PRODUCTION
LITELLM_LOG=INFO
LITELLM_PORT=8081
```

---

## 🤖 Agent Assignments

### 1. Chief Architect (system-architect, opus)
**Responsibilities**:
- Review complete architecture
- Validate integration with existing Traefik
- Ensure security best practices
- Approve deployment plan
- Store architectural decisions in memory

**Deliverables**:
- Architecture approval document
- Security validation checklist
- Integration verification

---

### 2. DevOps Engineer (deployment-engineer, sonnet)
**Responsibilities**:
- Create Docker Compose configuration
- Set up ccproxy service with health checks
- Configure logging and monitoring
- Create deployment scripts
- Test container networking with Traefik

**Deliverables**:
- `/Users/coder/ccproxy/docker-compose.yml`
- `/Users/coder/ccproxy/deploy.sh`
- Health check validation
- Container logs access

**Commands**:
```bash
# Deploy ccproxy
cd /home/ec2-user/ccproxy
docker compose up -d

# View logs
docker compose logs -f ccproxy

# Restart service
docker compose restart ccproxy

# Check health
curl http://localhost:8081/health
```

---

### 3. Backend Developer (backend-dev, sonnet)
**Responsibilities**:
- Create LiteLLM configuration (config.yaml)
- Implement routing logic for coding vs architecture
- Configure fallback chain (Ollama → Haiku → Error)
- Set up model aliases and groups
- Test routing rules

**Deliverables**:
- `/Users/coder/ccproxy/config.yaml`
- Routing logic validation
- Model configuration testing

**Testing Script** (`test-routing.sh`):
```bash
#!/bin/bash

BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
BASE_URL="https://coder.visiquate.com"

# Test 1: Coding task → Ollama
echo "Test 1: Coding task (should use Ollama)"
curl -X POST "${BASE_URL}/v1/messages" \
  -H "Authorization: Bearer ${BEARER_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "coding",
    "messages": [{"role": "user", "content": "Write a Python function to reverse a string"}],
    "max_tokens": 1024
  }'

# Test 2: Architecture task → Claude
echo "Test 2: Architecture task (should use Claude)"
curl -X POST "${BASE_URL}/v1/messages" \
  -H "Authorization: Bearer ${BEARER_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "architecture",
    "messages": [{"role": "user", "content": "Design a microservices architecture"}],
    "max_tokens": 4096
  }'

# Test 3: Fallback when Ollama down
echo "Test 3: Fallback to Haiku"
docker compose stop ollama
curl -X POST "${BASE_URL}/v1/messages" \
  -H "Authorization: Bearer ${BEARER_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "coding",
    "messages": [{"role": "user", "content": "Quick test"}],
    "max_tokens": 512
  }'
docker compose start ollama
```

---

### 4. Security Auditor (security-auditor, sonnet)
**Responsibilities**:
- Review bearer token authentication flow
- Validate API key security (environment variables only)
- Check Docker network isolation
- Audit LiteLLM security settings
- Test authentication failures

**Deliverables**:
- Security audit report
- Authentication test results
- Network isolation verification
- Secrets management validation

**Security Checklist**:
```markdown
- [ ] Bearer token passed from Traefik to ccproxy
- [ ] ANTHROPIC_API_KEY stored in .env (not in config)
- [ ] ccproxy port 8081 not exposed externally
- [ ] All services on internal Docker network
- [ ] Health endpoints don't leak sensitive data
- [ ] Rate limiting configured (10k RPM)
- [ ] Budget limits set ($1000/month)
- [ ] Logs don't contain API keys
```

---

### 5. QA Engineer (test-automator, sonnet)
**Responsibilities**:
- Create comprehensive test suite
- Test routing for coding vs architecture tasks
- Verify fallback chain works (Ollama → Haiku)
- Load test with 100 requests
- Monitor response times and error rates

**Deliverables**:
- Test suite (`tests/test-ccproxy.sh`)
- Load testing results
- Performance benchmarks
- Error handling validation

**Test Suite** (`tests/test-ccproxy.sh`):
```bash
#!/bin/bash

BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
BASE_URL="https://coder.visiquate.com"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Test counter
PASS=0
FAIL=0

test_endpoint() {
  local name="$1"
  local model="$2"
  local prompt="$3"

  echo "Testing: $name"
  response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/v1/messages" \
    -H "Authorization: Bearer ${BEARER_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"${model}\", \"messages\": [{\"role\": \"user\", \"content\": \"${prompt}\"}], \"max_tokens\": 512}")

  http_code=$(echo "$response" | tail -n1)

  if [ "$http_code" -eq 200 ]; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASS++))
  else
    echo -e "${RED}✗ FAIL (HTTP $http_code)${NC}"
    ((FAIL++))
  fi
}

# Run tests
echo "=== ccproxy Test Suite ==="

test_endpoint "Coding Task" "coding" "Write a function"
test_endpoint "Architecture Task" "architecture" "Design a system"
test_endpoint "Direct Ollama Fast" "ollama/qwen-fast" "Quick test"
test_endpoint "Direct Ollama Quality" "ollama/qwen-quality" "Complex task"
test_endpoint "Direct Claude Sonnet" "claude-sonnet-4.5" "Architecture"

# Invalid token test
echo "Testing: Invalid Token"
response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/v1/messages" \
  -H "Authorization: Bearer invalid-token" \
  -H "Content-Type: application/json" \
  -d '{"model": "coding", "messages": [{"role": "user", "content": "test"}]}')
http_code=$(echo "$response" | tail -n1)

if [ "$http_code" -eq 401 ] || [ "$http_code" -eq 403 ]; then
  echo -e "${GREEN}✓ PASS (401/403 as expected)${NC}"
  ((PASS++))
else
  echo -e "${RED}✗ FAIL (Expected 401/403, got $http_code)${NC}"
  ((FAIL++))
fi

# Summary
echo ""
echo "=== Test Summary ==="
echo -e "Passed: ${GREEN}$PASS${NC}"
echo -e "Failed: ${RED}$FAIL${NC}"

if [ $FAIL -eq 0 ]; then
  echo -e "${GREEN}All tests passed!${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed!${NC}"
  exit 1
fi
```

**Load Test** (`tests/load-test.sh`):
```bash
#!/bin/bash

BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
BASE_URL="https://coder.visiquate.com"

# Run 100 concurrent requests
echo "Running load test: 100 requests"

for i in {1..100}; do
  (curl -s -X POST "${BASE_URL}/v1/messages" \
    -H "Authorization: Bearer ${BEARER_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"model": "coding", "messages": [{"role": "user", "content": "test"}], "max_tokens": 128}' \
    > /dev/null 2>&1) &
done

wait
echo "Load test complete"
```

---

### 6. Documentation Lead (coder, haiku)
**Responsibilities**:
- Document deployment procedures
- Create operational runbook
- Write troubleshooting guide
- Document API usage for developers
- Create monitoring dashboard guide

**Deliverables**:
- `/Users/coder/ccproxy/README.md`
- `/Users/coder/ccproxy/RUNBOOK.md`
- `/Users/coder/ccproxy/TROUBLESHOOTING.md`
- API usage examples

**README.md Structure**:
```markdown
# ccproxy - LiteLLM Routing Proxy

## Overview
- What ccproxy does
- Architecture diagram
- Integration with Traefik

## Quick Start
- Start: `docker compose up -d`
- Stop: `docker compose down`
- Logs: `docker compose logs -f`

## Configuration
- config.yaml explained
- Environment variables
- Model configuration

## Usage
- How to send requests
- Model selection
- Fallback behavior

## Monitoring
- Health checks
- Logs location
- Metrics dashboard

## Troubleshooting
- Common issues
- Debug procedures
- Contact info
```

---

### 7. Credential Manager (coder, haiku)
**Responsibilities**:
- Retrieve ANTHROPIC_API_KEY securely
- Create .env file with proper permissions
- Validate credential access from containers
- Document credential rotation procedures
- Set up credential monitoring

**Deliverables**:
- Secure .env file with proper permissions (600)
- Credential validation script
- Rotation procedure documentation

**Security Script** (`setup-credentials.sh`):
```bash
#!/bin/bash

# Retrieve Claude API key
echo "Retrieving ANTHROPIC_API_KEY from credential manager..."
ANTHROPIC_API_KEY=$(node /Users/brent/git/cc-orchestra/src/credential-manager.js retrieve ANTHROPIC_API_KEY)

if [ -z "$ANTHROPIC_API_KEY" ]; then
  echo "ERROR: Failed to retrieve ANTHROPIC_API_KEY"
  exit 1
fi

# Create .env file
cat > .env <<EOF
# Claude API Key (DO NOT COMMIT)
ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}

# LiteLLM Configuration
LITELLM_MODE=PRODUCTION
LITELLM_LOG=INFO
LITELLM_PORT=8081
EOF

# Set proper permissions
chmod 600 .env

echo "✓ Credentials configured securely"
echo "✓ .env file created with 600 permissions"
```

---

## 🚀 Deployment Steps

### Pre-Deployment Checklist
- [ ] Access to Mac mini (local or remote)
- [ ] Traefik running and accessible on port 8080
- [ ] Ollama running with qwen-fast and qwen-quality models
- [ ] Bearer token authentication working
- [ ] ANTHROPIC_API_KEY available from credential manager
- [ ] Docker network `coder_network` exists
- [ ] macOS Docker Desktop running

### Deployment Procedure

1. **Chief Architect**: Review and approve architecture
2. **Credential Manager**: Set up .env file with API keys
3. **Backend Developer**: Create config.yaml with routing rules
4. **DevOps Engineer**: Deploy Docker Compose stack
5. **Security Auditor**: Run security audit
6. **QA Engineer**: Execute test suite
7. **Documentation Lead**: Finalize documentation

### Step-by-Step Commands

```bash
# 1. Create project directory on Mac mini
mkdir -p /Users/coder/ccproxy/{logs,tests}
cd /Users/coder/ccproxy

# 2. Create configuration files
# (Agents will create: docker-compose.yml, config.yaml, .env)

# 3. Set proper permissions
chmod 600 .env

# 4. Deploy ccproxy
docker compose up -d

# 5. Verify deployment
docker compose ps
docker compose logs ccproxy
curl http://localhost:8081/health

# 6. Update Traefik configuration
# (Agent will create: /Users/coder/traefik/dynamic/ccproxy.yml)

# 7. Reload Traefik
docker compose -f /Users/coder/traefik/docker-compose.yml restart traefik

# 8. Run tests
bash tests/test-ccproxy.sh
bash tests/load-test.sh

# 9. Monitor logs
docker compose logs -f ccproxy
```

---

## 🧪 Testing & Validation

### Unit Tests
- Config parsing validation
- Routing logic verification
- Fallback chain testing

### Integration Tests
- Traefik → ccproxy routing
- Bearer token authentication
- Ollama communication
- Claude API communication

### End-to-End Tests
- Developer machine → coder.visiquate.com
- Coding task routed to Ollama
- Architecture task routed to Claude
- Fallback when Ollama unavailable

### Load Tests
- 100 concurrent requests
- Response time < 2s for coding tasks
- Response time < 5s for architecture tasks
- Error rate < 1%

---

## 📊 Success Criteria

### Functional Requirements
- ✅ ccproxy successfully deploys on port 8081
- ✅ Traefik routes `/v1/messages` to ccproxy
- ✅ Bearer token authentication works
- ✅ Coding tasks route to Ollama qwen-fast
- ✅ Architecture tasks route to Claude Sonnet
- ✅ Fallback to Haiku when Ollama down
- ✅ Health checks pass

### Performance Requirements
- ✅ Response time < 2s for Ollama requests
- ✅ Response time < 5s for Claude requests
- ✅ Handles 100 concurrent requests
- ✅ 99% uptime

### Security Requirements
- ✅ Bearer token required for all requests
- ✅ API keys stored in environment variables only
- ✅ Internal ports not exposed externally
- ✅ Docker network isolation enforced
- ✅ Logs don't contain sensitive data

### Documentation Requirements
- ✅ README.md with quick start
- ✅ RUNBOOK.md with operational procedures
- ✅ TROUBLESHOOTING.md with debug steps
- ✅ API usage examples for developers

---

## 🔧 Operational Procedures

### Monitoring
```bash
# View ccproxy logs
docker compose logs -f ccproxy

# Check health
curl http://localhost:8081/health

# Monitor resource usage
docker stats ccproxy

# View LiteLLM dashboard
# (Access via browser: https://coder.visiquate.com/dashboard)
```

### Maintenance
```bash
# Restart ccproxy
docker compose restart ccproxy

# Update configuration
vim config.yaml
docker compose restart ccproxy

# View database
sqlite3 logs/litellm.db "SELECT * FROM request_log LIMIT 10;"

# Rotate logs
docker compose logs ccproxy > logs/ccproxy-$(date +%Y%m%d).log
docker compose restart ccproxy
```

### Troubleshooting
```bash
# ccproxy not responding
docker compose ps
docker compose logs ccproxy
curl http://localhost:8081/health

# Routing not working
tail -f logs/litellm.log
# Check config.yaml routing rules

# Ollama fallback not working
docker ps | grep ollama
curl http://localhost:11434/api/tags

# High error rate
docker compose logs ccproxy | grep ERROR
# Check Claude API key validity
```

---

## 🎯 Next Steps After Deployment

1. **Developer Configuration**: Update ANTHROPIC_BASE_URL on all developer machines
   ```bash
   export ANTHROPIC_BASE_URL="https://coder.visiquate.com"
   ```

2. **Monitoring Setup**: Configure Grafana dashboard for request metrics

3. **Cost Tracking**: Review LiteLLM cost dashboard weekly

4. **Load Testing**: Run continuous load tests to optimize

5. **Documentation**: Update team wiki with usage instructions

---

## 📞 Support & Contact

- **Deployment Issues**: DevOps Engineer
- **Security Concerns**: Security Auditor
- **Configuration Help**: Backend Developer
- **Testing Questions**: QA Engineer

---

## 🔄 Continuous Improvement

### Metrics to Track
- Request count by model
- Average response time
- Error rate
- Cost per request
- Fallback frequency

### Optimization Opportunities
- Adjust routing rules based on usage
- Fine-tune model selection criteria
- Optimize fallback chain
- Scale Ollama instances if needed

---

**Deployment Mission Created**: 2025-11-04
**Mission Status**: Ready for execution
**Estimated Deployment Time**: 2-4 hours with full testing
