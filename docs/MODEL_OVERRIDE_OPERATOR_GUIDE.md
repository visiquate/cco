# CCO Model Override - Operator Guide

This guide covers deploying, managing, monitoring, and maintaining CCO with model overrides in production environments.

## Overview

As an operator, you're responsible for:
- Deploying CCO with model overrides enabled
- Monitoring override performance and cost savings
- Managing configuration changes
- Troubleshooting issues
- Planning capacity and scaling
- Reporting on cost savings

## Deployment

### Prerequisites

- CCO source code (`/Users/brent/git/cc-orchestra/cco`)
- Rust toolchain (1.70+)
- Anthropic API key
- Port 3000 (or alternate) available

### Build CCO with Model Overrides

```bash
cd /Users/brent/git/cc-orchestra/cco

# Build optimized binary
cargo build --release

# Binary will be at:
# ./target/release/cco
```

### Local Development Deployment

```bash
# Start CCO with model overrides enabled
./target/release/cco run --port 3000

# In another terminal, verify it's working:
curl http://localhost:3000/health
```

Expected output:
```json
{
  "status": "ok",
  "overrides_enabled": true,
  "override_rules": 3,
  "cache_hits": 0,
  "cache_misses": 0
}
```

### Production Deployment

#### Option 1: Systemd Service (Linux/macOS)

Create `/etc/systemd/system/cco.service`:

```ini
[Unit]
Description=Claude Code Orchestrator (CCO)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=cco
WorkingDirectory=/opt/cco
Environment="ANTHROPIC_API_KEY=sk-ant-..."
ExecStart=/opt/cco/cco run --port 3000 --host 0.0.0.0
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Start the service:
```bash
sudo systemctl start cco
sudo systemctl enable cco
sudo systemctl status cco
```

#### Option 2: Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/cco /usr/local/bin/
EXPOSE 3000
ENV ANTHROPIC_API_KEY=""
ENTRYPOINT ["cco", "run", "--host", "0.0.0.0", "--port", "3000"]
```

Build and run:
```bash
# Build image
docker build -t cco:latest .

# Run container
docker run -d \
  --name cco \
  --port 3000:3000 \
  -e ANTHROPIC_API_KEY="sk-ant-..." \
  cco:latest

# Verify running
docker logs cco
```

#### Option 3: Kubernetes Deployment

Create `cco-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco
  labels:
    app: cco
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cco
  template:
    metadata:
      labels:
        app: cco
    spec:
      containers:
      - name: cco
        image: cco:latest
        ports:
        - containerPort: 3000
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: api-keys
              key: anthropic-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: cco-service
spec:
  selector:
    app: cco
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

Deploy:
```bash
# Create secret with API key
kubectl create secret generic api-keys --from-literal=anthropic-key="sk-ant-..."

# Apply deployment
kubectl apply -f cco-deployment.yaml

# Check status
kubectl get pods -l app=cco
kubectl logs -l app=cco
```

## Configuration Management

### Configuration File Location

```
/Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
```

### Configuration Structure

```toml
[overrides]
enabled = true
rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
]

[analytics]
log_overrides = true
track_statistics = true
report_format = "json"

[per_model_rules]
# Future: Fine-grained per-model configuration
```

### Configuration Changes

**When changing configuration:**

1. Edit `model-overrides.toml`
2. Validate the TOML syntax
3. Restart CCO service
4. Verify changes took effect

```bash
# Edit configuration
nano /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# Validate syntax (Rust will catch errors on startup)
# Restart service
sudo systemctl restart cco

# Verify changes
curl http://localhost:3000/health | jq .overrides_enabled
```

### Configuration Backup

Before making changes, always backup the current configuration:

```bash
# Backup current config
cp /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml \
   /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup.$(date +%Y%m%d-%H%M%S)

# Restore if needed
cp /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml.backup.20251115-120000 \
   /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
```

## Monitoring

### Health Check Endpoint

```bash
# Check CCO health
curl http://localhost:3000/health

# Response:
{
  "status": "ok",
  "uptime": 3600,
  "overrides_enabled": true,
  "override_rules": 3,
  "cache_hits": 1234,
  "cache_misses": 567
}
```

### Override Statistics

```bash
# Get override statistics
curl http://localhost:3000/api/overrides/stats

# Response:
{
  "total_overrides": 47,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": {
      "overridden_to": "claude-haiku-4-5-20251001",
      "count": 47,
      "percentage": 100,
      "cost_saved": "$18.50"
    }
  }
}
```

### Continuous Monitoring

#### Prometheus Metrics (if available)

```bash
# Metrics endpoint
curl http://localhost:3000/metrics

# Key metrics:
# - cco_overrides_total
# - cco_cache_hits
# - cco_cache_misses
# - cco_api_latency
```

#### Dashboard Monitoring

Access the web dashboard at `http://localhost:3000`:

- **Project Tab**: Current project metrics (cost, tokens, calls)
- **Machine Tab**: Overall system analytics
- **Terminal Tab**: Commands and debugging

#### Log Monitoring

Monitor CCO logs for important events:

```bash
# Local development (watch console output)
./target/release/cco run --port 3000

# Systemd service
journalctl -u cco -f

# Docker container
docker logs -f cco

# Kubernetes pod
kubectl logs -f deployment/cco
```

Watch for:
- Override statistics messages
- Cache hit/miss ratios
- API errors or failures
- Performance metrics

### Setting Up Alerts

#### Alert Conditions

Set up alerts for:

1. **Overrides Not Working**
   - Alert if override count hasn't changed in 1 hour
   - Alert if overrides_enabled is false

2. **Cache Performance**
   - Alert if cache hit rate drops below 30%
   - Alert if memory usage exceeds 500MB

3. **Service Health**
   - Alert if health check fails
   - Alert if response latency > 5 seconds

#### Example Prometheus Alert Rules

```yaml
groups:
- name: cco
  rules:
  - alert: CCODown
    expr: up{job="cco"} == 0
    for: 5m
    annotations:
      summary: "CCO is down"

  - alert: OverridesNotWorking
    expr: increase(cco_overrides_total[1h]) == 0
    for: 1h
    annotations:
      summary: "No model overrides applied in the last hour"

  - alert: LowCacheHitRate
    expr: cco_cache_hit_rate < 0.3
    for: 10m
    annotations:
      summary: "Cache hit rate below 30%"
```

## Scaling and Capacity Planning

### Single Instance Performance

A single CCO instance can handle:

- **Throughput**: 1000+ requests/second
- **Concurrent connections**: 500+
- **Memory**: 256MB-512MB
- **Cache size**: 100MB-1GB

### Scaling Strategies

#### Horizontal Scaling (Recommended for Production)

Deploy multiple CCO instances behind a load balancer:

```
     Client
       ↓
   Load Balancer (nginx/haproxy)
       ↓
   ┌───┴───┐
   ↓       ↓
  CCO     CCO
  (1)     (2)
```

**Benefits:**
- High availability (failover)
- Better performance
- Can scale to millions of requests

**Load Balancer Config (nginx):**

```nginx
upstream cco {
    least_conn;
    server localhost:3000 max_fails=2 fail_timeout=10s;
    server localhost:3001 max_fails=2 fail_timeout=10s;
}

server {
    listen 80;
    server_name api.company.com;

    location / {
        proxy_pass http://cco;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

#### Vertical Scaling

For single instance scenarios, increase resources:

```bash
# Container limits
docker run --memory 2g --cpus 2 cco:latest

# Kubernetes resources
resources:
  limits:
    memory: "2Gi"
    cpu: "2000m"
```

### Capacity Planning

**For 100 concurrent agents:**

```
Requests/month: 100 × 50 = 5,000
Peak requests/second: 5,000 / 2,592,000 ≈ 2 req/sec

Resources needed:
- Instances: 1
- Memory: 512MB
- CPU: 500m
- Storage: 100MB (cache)
```

**For 1,000 concurrent agents:**

```
Requests/month: 1,000 × 50 = 50,000
Peak requests/second: 50,000 / 2,592,000 ≈ 20 req/sec

Resources needed:
- Instances: 3 (load balanced)
- Memory per instance: 512MB
- CPU per instance: 500m
- Storage: 500MB per instance (distributed cache)
```

## Maintenance

### Regular Tasks

#### Daily

- Check health endpoint: `curl http://localhost:3000/health`
- Review override statistics
- Monitor logs for errors

#### Weekly

- Review cost savings dashboard
- Check cache hit rate trends
- Verify configuration is still correct

#### Monthly

- Backup configuration
- Review and optimize override rules
- Generate cost analysis report
- Plan for capacity needs

### Updates

#### Updating CCO Version

```bash
# Build new version
cd /Users/brent/git/cc-orchestra/cco
git pull origin main
cargo build --release

# Backup current binary
cp ./target/release/cco ./target/release/cco.backup

# Stop old version
sudo systemctl stop cco

# Deploy new version
sudo cp ./target/release/cco /opt/cco/

# Start new version
sudo systemctl start cco

# Verify
curl http://localhost:3000/health
```

#### Rolling Update (Kubernetes)

```bash
# Update image
kubectl set image deployment/cco cco=cco:v2025.11.2

# Monitor rollout
kubectl rollout status deployment/cco

# Rollback if needed
kubectl rollout undo deployment/cco
```

### Backup and Recovery

#### Backup Configuration

```bash
# Regular backup
cp -r /Users/brent/git/cc-orchestra/cco/config \
   /backups/cco-config-$(date +%Y%m%d).tar.gz

# Cloud backup (example with S3)
tar czf - /Users/brent/git/cc-orchestra/cco/config | \
  aws s3 cp - s3://backups/cco-config-$(date +%Y%m%d).tar.gz
```

#### Recovery Procedure

```bash
# Restore from backup
tar xzf /backups/cco-config-20251115.tar.gz \
  -C /Users/brent/git/cc-orchestra/

# Restart CCO
sudo systemctl restart cco

# Verify
curl http://localhost:3000/health
```

## Troubleshooting

### CCO Won't Start

**Check error logs:**

```bash
# Local development
./target/release/cco run --port 3000
# Look for error messages

# Systemd service
journalctl -u cco -n 50

# Docker container
docker logs cco
```

**Common issues:**

1. **Port already in use**
   ```bash
   # Find what's using port 3000
   lsof -i :3000

   # Kill process or use different port
   ./target/release/cco run --port 3001
   ```

2. **API key not set**
   ```bash
   # Verify environment variable
   echo $ANTHROPIC_API_KEY

   # Set it
   export ANTHROPIC_API_KEY="sk-ant-..."
   ```

3. **Config file issues**
   ```bash
   # Check file exists
   ls -la /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

   # Validate TOML (Rust will report errors on startup)
   ```

### Overrides Not Working

See [Troubleshooting in User Guide](./MODEL_OVERRIDE_USER_GUIDE.md#troubleshooting)

**Additional operator checks:**

```bash
# Verify config is loaded
curl http://localhost:3000/health | grep overrides

# Check configuration file directly
cat /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml

# Look for config parsing errors in logs
journalctl -u cco | grep -i error
```

### High Latency

**Investigate causes:**

```bash
# Check metrics
curl http://localhost:3000/metrics | grep latency

# Monitor system resources
top -p $(pgrep cco)

# Check network connectivity
ping api.anthropic.com
```

**Solutions:**

1. Increase CCO instance count (horizontal scaling)
2. Increase memory/CPU allocation
3. Check network connectivity to Anthropic API
4. Reduce cache size if memory-bound

### Memory Issues

```bash
# Check memory usage
docker stats cco

# Restart to free memory
sudo systemctl restart cco

# Increase limits if needed (Kubernetes example)
kubectl set resources deployment/cco --limits=memory=2Gi --requests=memory=1Gi
```

## Reporting and Analytics

### Cost Savings Report

Generate monthly report:

```bash
# Get override statistics
curl http://localhost:3000/api/overrides/stats > /tmp/stats.json

# Calculate savings
# Current: sonnet pricing
# With overrides: haiku pricing
# Savings = Current - Override
```

### Performance Report

```bash
# Dashboard provides
# - Override count
# - Cache hit/miss ratio
# - Cost breakdown
# - Model usage distribution

# Automated reporting can query:
# GET /api/overrides/stats
# GET /api/cache/stats
# GET /api/machine/stats
```

## Security Considerations

### API Key Management

- Store API keys in environment variables, not configuration files
- Use secret management for production (Kubernetes Secrets, HashiCorp Vault)
- Rotate keys periodically

```bash
# Good: Environment variable
export ANTHROPIC_API_KEY="sk-ant-..."

# Bad: Hardcoded in config
# api_key = "sk-ant-..."  # NEVER do this
```

### Access Control

- Restrict CCO endpoint access to authorized clients only
- Use firewall rules to limit who can reach port 3000
- Consider authentication/authorization for dashboard

```bash
# Firewall rule example (allow from Claude Code only)
ufw allow from 127.0.0.1 to any port 3000
```

### Monitoring and Logging

- Log all override activities
- Monitor for suspicious patterns
- Keep logs for audit trail

## Disaster Recovery Plan

### Disaster Recovery Procedure

**If CCO goes down:**

1. **Immediate** (5 minutes)
   - Restart CCO service
   - Verify health endpoint
   - Confirm overrides are working

   ```bash
   sudo systemctl restart cco
   curl http://localhost:3000/health
   ```

2. **Short-term** (30 minutes)
   - If restart fails, check logs
   - Verify API key is still valid
   - Check configuration file integrity
   - Restore from backup if needed

3. **Medium-term** (1 hour)
   - Identify root cause
   - Deploy fix (if needed)
   - Verify stable operation
   - Resume normal monitoring

**Fallback if CCO is unavailable:**

If CCO cannot be restored quickly:

1. Update `ANTHROPIC_API_BASE_URL` to point directly to Anthropic API (no CCO)
2. Users get full-price API calls (no overrides)
3. Restore CCO in parallel
4. Switch back when CCO is ready

## Next Steps

1. **[Configuration Reference](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)** - Detailed config options
2. **[Cost Analysis](./COST_ANALYSIS.md)** - Detailed cost breakdown
3. **[User Guide](./MODEL_OVERRIDE_USER_GUIDE.md)** - For end-user support
4. **[API Documentation](./API.md)** - For monitoring integration

---

**Production Deployment Checklist:**

- [ ] CCO built and tested
- [ ] Configuration validated
- [ ] API key configured securely
- [ ] Load balancer configured (if applicable)
- [ ] Health checks configured
- [ ] Monitoring and alerts enabled
- [ ] Backup and recovery procedures tested
- [ ] Documentation reviewed with team
- [ ] Rollback procedure documented
- [ ] Go/No-go decision made
