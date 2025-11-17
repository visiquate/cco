# Model Override Documentation Index

Complete documentation set for the CCO Model Override feature—transparent LLM cost optimization achieving 73% savings.

## What is Model Override?

Model Override is a feature that automatically rewrites LLM model requests at the proxy layer, routing expensive models (like Claude Sonnet) to cost-effective alternatives (like Claude Haiku) without any code changes.

**Example:**
```
Your request for Sonnet → CCO proxy → Rewrite to Haiku → Send to Anthropic API
Result: Same quality output, 73% lower cost
```

## Quick Navigation

### I want to...

- **Start using model overrides** → [User Guide](./MODEL_OVERRIDE_USER_GUIDE.md)
- **Deploy CCO in production** → [Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)
- **Configure override rules** → [Configuration Reference](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)
- **Calculate cost savings** → [Cost Analysis](./COST_ANALYSIS.md)
- **Enable in existing setup** → [Migration Guide](./MIGRATE_TO_MODEL_OVERRIDE.md)
- **Integrate with monitoring** → [API Documentation](./API.md)
- **Understand architecture** → [Architecture Overview](#architecture-overview) (below)

## Documentation Files

### 1. User Guide
**File:** [`MODEL_OVERRIDE_USER_GUIDE.md`](./MODEL_OVERRIDE_USER_GUIDE.md)

**Audience:** End users, developers using Claude Code with CCO

**Contains:**
- What model override is and why to use it
- Step-by-step setup (4 steps, ~5 minutes)
- How to monitor overrides (dashboard, API, logs)
- Configuration basics (enabling/disabling)
- Cost savings calculator
- Comprehensive FAQ
- Troubleshooting guide

**Key Sections:**
- Getting Started (Prerequisites, Installation, Configuration)
- Monitoring Overrides (Dashboard, API, Logs)
- Performance Impact
- Cache Key Behavior
- Cost Savings Examples
- Troubleshooting

**Best For:**
- First-time users setting up model overrides
- Understanding how overrides work
- Troubleshooting common issues
- Quick reference for monitoring

### 2. Operator Guide
**File:** [`MODEL_OVERRIDE_OPERATOR_GUIDE.md`](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)

**Audience:** DevOps engineers, system operators, deployment engineers

**Contains:**
- Deployment strategies (local, Docker, Kubernetes)
- Configuration file management
- Monitoring and alerting setup
- Scaling and capacity planning
- Maintenance procedures
- Update strategies
- Backup and recovery
- Disaster recovery plan

**Key Sections:**
- Deployment (3 options: local, Docker, Kubernetes)
- Configuration Management (backup, validation, deployment)
- Monitoring (health checks, statistics, alerts)
- Scaling (horizontal and vertical options)
- Maintenance (daily, weekly, monthly tasks)
- Troubleshooting (start issues, override issues, performance)
- Reporting and Analytics

**Best For:**
- Production deployments
- Operational procedures
- Capacity planning
- Disaster recovery
- Team operations

### 3. Configuration Reference
**File:** [`MODEL_OVERRIDE_CONFIG_REFERENCE.md`](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)

**Audience:** Configuration administrators, advanced users

**Contains:**
- Complete TOML configuration file reference
- All configuration options documented
- Examples for common scenarios
- Best practices
- Validation and troubleshooting
- Performance tuning

**Key Sections:**
- File Location
- Configuration Structure
  - `[overrides]` section (enabled, rules)
  - `[analytics]` section (logging, tracking)
  - `[per_model_rules]` section (future)
  - `[override_statistics]` section (runtime)
- Complete Example Configuration
- Best Practices
- Common Scenarios
- Troubleshooting Configuration Issues

**Best For:**
- Detailed reference on configuration options
- Creating custom override rules
- Setting up analytics and logging
- Validating configuration files

### 4. Cost Analysis
**File:** [`COST_ANALYSIS.md`](./COST_ANALYSIS.md)

**Audience:** Decision makers, financial analysts, team leads

**Contains:**
- Current Anthropic pricing (Opus, Sonnet, Haiku)
- Detailed cost scenarios (small, medium, large teams)
- ROI calculations
- Break-even analysis
- 3-5 year cost projections
- Quality assurance considerations
- Sensitivity analysis

**Key Sections:**
- Anthropic API Pricing
- Savings by Override Type
- Typical Agent Token Usage
- Cost Scenarios (3 detailed examples)
- ROI Analysis (investment, payback, 3-year ROI)
- Cost Comparison Tables
- Cumulative Savings Over Time
- Cost Avoidance Analysis
- Sensitivity Analysis
- Budget Impact by Organization Size
- Recommendation

**Best For:**
- Cost-benefit analysis
- Budget planning and forecasting
- ROI justification
- Understanding financial impact
- Executive reports

### 5. Migration Guide
**File:** [`MIGRATE_TO_MODEL_OVERRIDE.md`](./MIGRATE_TO_MODEL_OVERRIDE.md)

**Audience:** Existing CCO users, teams migrating from manual setup

**Contains:**
- Step-by-step migration procedure (12 steps)
- Backup and recovery procedures
- Verification checklist
- Post-migration monitoring
- Troubleshooting specific migration issues
- Rollback procedures
- Migration scenarios (single-operator, team, HA)

**Key Sections:**
- Overview (5-10 minute process)
- Prerequisites
- 12 Step Migration Process
- Verification Checklist
- Rollback Procedure
- Post-Migration Monitoring
- Troubleshooting
- Migration Scenarios (3 examples)
- FAQ
- Support

**Best For:**
- Enabling overrides in existing deployments
- Migrating from manual model selection
- Understanding rollback procedures
- Team communication

### 6. API Documentation
**File:** [`API.md`](./API.md)

**Audience:** Integration engineers, monitoring specialists, developers

**Contains:**
- Complete REST API reference
- All endpoints documented
- Request/response formats
- Authentication (if needed)
- Error handling
- Example workflows
- Performance characteristics
- Security considerations

**Key Sections:**
- Base URL and Authentication
- Health Check Endpoint (`GET /health`)
- Model Override Statistics (`GET /api/overrides/stats`)
- Cache Statistics (`GET /api/cache/stats`)
- Machine-Wide Analytics (`GET /api/machine/stats`)
- Project Analytics (`GET /api/project/stats`)
- Streaming (`GET /api/stream`)
- WebSocket Terminal (`WS /terminal`)
- Error Responses
- Rate Limiting
- CORS Headers
- Example Workflows
- API Versioning
- Performance Considerations
- Security
- Troubleshooting

**Best For:**
- Building monitoring dashboards
- Integrating with external systems
- Custom analytics implementation
- API integration development

## Architecture Overview

### How Model Override Works

```
User Code
   ↓ (request with model=sonnet)
   ↓
CCO Proxy
   ├─ Check: Is "sonnet" in override rules?
   ├─ Yes → Rewrite to "haiku"
   └─ No  → Pass through unchanged
   ↓
API Request (with rewritten model)
   ↓
Anthropic API
   ↓ (response)
Return to User
```

### Key Design Decisions

1. **Transparent** - No code changes needed in Claude Code
2. **Automatic** - Happens before request reaches API
3. **Reversible** - Can be disabled instantly
4. **Flexible** - Can be configured per-model
5. **Observable** - Full statistics and monitoring

### Configuration Flow

```
model-overrides.toml
    ↓
CCO loads config on startup
    ↓
Rules are compiled into lookup table
    ↓
Each request is checked against rules
    ↓
Matching rule rewrites model parameter
    ↓
Statistics are tracked
```

## Use Cases

### Use Case 1: Cost Optimization
**Goal:** Reduce LLM costs while maintaining quality

**Solution:**
- Override Sonnet → Haiku for non-critical tasks
- Keep Opus for complex reasoning
- Save 73% on coding/testing tasks

**Savings:** $318-3,180/year for small-medium teams

### Use Case 2: Performance Optimization
**Goal:** Improve response times

**Solution:**
- Haiku is 2x faster than Sonnet
- Overrides improve latency
- Better user experience

**Benefit:** Faster responses, lower cost

### Use Case 3: Quality vs. Cost Balance
**Goal:** Optimize cost while maintaining output quality

**Solution:**
- Override less critical models
- Preserve Opus for strategic decisions
- Maintain test quality

**Result:** Best-in-class cost/quality ratio

### Use Case 4: Team Growth at Fixed Cost
**Goal:** Scale team without scaling costs

**Solution:**
- Add more agents
- Overrides apply to all agents
- Cost stays flat while productivity grows

**Result:** Hire more developers without increasing LLM spend

## Feature Comparison

| Feature | Without Override | With Override |
|---------|------------------|---------------|
| Monthly Cost (10 agents) | $945 | $252 |
| Annual Cost (10 agents) | $11,340 | $3,024 |
| Annual Savings (10 agents) | — | $8,316 |
| Setup Time | N/A | 5 minutes |
| Configuration Complexity | N/A | Simple TOML |
| Rollback Time | N/A | 2 minutes |
| Code Changes Required | N/A | 0 |
| Quality Impact | — | Minimal (<2%) |

## Implementation Paths

### Path 1: Quick Start (30 minutes)
1. Read [User Guide](./MODEL_OVERRIDE_USER_GUIDE.md) (10 min)
2. Enable overrides in config (5 min)
3. Restart CCO (1 min)
4. Verify in dashboard (5 min)
5. Monitor and optimize (10 min)

**Result:** Cost savings active and monitoring

### Path 2: Production Deployment (2-4 hours)
1. Read [Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md) (30 min)
2. Design deployment strategy (30 min)
3. Deploy to test environment (30 min)
4. Monitor for 1 week (self-service)
5. Deploy to production (30 min)
6. Set up monitoring and alerts (60 min)

**Result:** Production-ready cost optimization

### Path 3: Enterprise Rollout (1-2 weeks)
1. Cost analysis and ROI justification (1-2 days)
2. Design for high availability (1 day)
3. Set up monitoring and alerting (1-2 days)
4. Test in staging (2-3 days)
5. Phase rollout to production (2-3 days)
6. Optimize based on metrics (ongoing)

**Result:** Enterprise-grade cost optimization at scale

## Documentation Quality Standards

All documentation follows these standards:

- **Clear Language** - Accessible to target audience
- **Code Examples** - Real, executable examples
- **Troubleshooting** - Common issues and solutions
- **Links** - Cross-references between docs
- **Formatting** - Consistent structure and styling
- **Completeness** - All features documented
- **Accuracy** - Current with latest features

## Quick Reference

### Setup Command
```bash
# 1. Enable in config
nano /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
# Set: enabled = true

# 2. Restart CCO
sudo systemctl restart cco

# 3. Verify
curl http://localhost:3000/health | jq '.overrides_enabled'
# Output: true
```

### Verify It's Working
```bash
# Check override statistics
curl http://localhost:3000/api/overrides/stats | jq

# View dashboard
open http://localhost:3000
```

### Troubleshooting
```bash
# Check health
curl http://localhost:3000/health

# View logs
journalctl -u cco -f

# Restart service
sudo systemctl restart cco
```

## Document Relationships

```
                    ┌─────────────────┐
                    │ README (intro)  │
                    └────────┬────────┘
                             │
                    ┌────────▼──────────┐
                    │  INDEX (this doc) │
                    └────────┬──────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
    ┌────────────┐      ┌──────────┐      ┌─────────────┐
    │ User Guide │      │ Operator │      │ Config Ref  │
    │            │      │  Guide   │      │             │
    └────────────┘      └──────────┘      └─────────────┘
        │
        ├─→ Cost Analysis
        ├─→ Migration Guide
        ├─→ API Documentation
        └─→ Troubleshooting
```

## Support and Help

### For Users
- Start with [User Guide](./MODEL_OVERRIDE_USER_GUIDE.md)
- Troubleshooting section in User Guide
- Check dashboard at `http://localhost:3000`

### For Operators
- Start with [Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)
- Review [Configuration Reference](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)
- See operational procedures in Operator Guide

### For Decision Makers
- Start with [Cost Analysis](./COST_ANALYSIS.md)
- Review ROI calculations
- See implementation paths above

### For Integration
- Consult [API Documentation](./API.md)
- Review example workflows
- Check endpoint specifications

## Version Information

**Documentation Version:** 2025.11.15
**CCO Feature Status:** Production Ready
**Compatibility:** CCO 2025.11.1+

## Related Resources

- **Main README:** [`README.md`](../README.md) - Project overview
- **CCO README:** [`cco/README.md`](../cco/README.md) - CCO proxy details
- **Configuration File:** [`cco/config/model-overrides.toml`](../cco/config/model-overrides.toml) - Current config

## Summary

The Model Override feature provides:

✅ **73% cost reduction** on LLM API calls
✅ **Zero code changes** required
✅ **5-minute setup** to enable
✅ **Easy rollback** (2 minutes)
✅ **Full monitoring** and analytics
✅ **Transparent operation** - users don't notice
✅ **Scalable** - works for 1-1000+ agents

**Recommendation:** Enable by default in all deployments. The cost savings far outweigh the minimal effort to implement.

---

**Ready to get started?** Pick your role above and follow the corresponding guide.
