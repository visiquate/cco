# Deploy: Deployment Coordination

## Trigger
`/deploy <version or component>`

Orchestrates deployment with coordination, testing, rollback capability, and monitoring.

## Purpose
The Deploy skill provides:
- Deployment planning and coordination
- Environment promotion (dev → staging → prod)
- Zero-downtime deployment support
- Canary deployment strategies
- Rollback capability
- Health checking and monitoring

## Workflow

### Phase 1: Define Deployment Scope

1. **Identify what's deploying**
   - Code version/commit
   - Database migrations
   - Infrastructure changes
   - Configuration updates
   - Dependencies

2. **Query deployment decisions**
   ```bash
   cco knowledge search "deployment strategy"
   cco knowledge search "release process"
   cco knowledge search "deployment checklist"
   ```

3. **Assess readiness**
   - All code reviewed?
   - All tests passing?
   - Security approved?
   - Performance validated?
   - Database schema compatible?

### Phase 2: Spawn Deployment Team

**Parallel coordination:**
- **Deployment Engineer** (Sonnet) - Coordination & execution
- **DevOps Engineer** (Sonnet) - Infrastructure & automation
- **Test Engineer** (Sonnet) - Testing & validation
- **Monitoring Specialist** (Sonnet) - Health checks & alerts

### Phase 3: Pre-Deployment Validation

**Deployment Engineer checklist:**

```
PRE-DEPLOYMENT CHECKLIST
========================

Code Readiness:
☐ All features merged to main branch
☐ All code reviews completed and approved
☐ All tests passing (unit, integration, E2E)
☐ Code quality checks passing (lint, format)
☐ No security vulnerabilities found
☐ No performance regressions detected

Database Readiness:
☐ Schema migration tested on staging
☐ Rollback procedure documented
☐ Data backup created
☐ Migration duration estimated < 5 min
☐ Zero-downtime strategy planned if needed

Documentation:
☐ Release notes prepared
☐ Deployment guide documented
☐ Rollback procedure documented
☐ Team notified of deployment window
☐ Customer communication prepared

Environment:
☐ Staging environment identical to production
☐ All environment variables configured
☐ Secrets properly managed (not in code)
☐ Infrastructure ready
☐ Monitoring configured

Dependencies:
☐ All external services operational
☐ API integrations tested
☐ Third-party services accessible
☐ No known issues blocking deployment
```

### Phase 4: Test Deployment Plan

**Test Engineer validates on staging:**

1. **Smoke Tests**
   ```bash
   # Quick validation
   curl -H "Authorization: Bearer $TOKEN" \
     https://staging-api.example.com/health

   # Response: {"status": "healthy", "version": "1.2.3"}
   ```

2. **Functional Tests**
   ```bash
   # Run full test suite on staging
   pytest tests/ -v --tb=short

   # Expected: All tests passing
   # Test count: 450 tests
   # Duration: 8 minutes
   # Coverage: 87%
   ```

3. **Load Testing**
   ```bash
   # Simulate production load
   k6 run load-test.js

   # Results:
   # - 500 concurrent users
   # - p95 response: 145ms (target: 200ms) ✓
   # - Error rate: 0% ✓
   # - CPU usage: 45% ✓
   ```

4. **Security Validation**
   ```bash
   # OWASP checks
   # Dependency vulnerability scan
   # No critical issues found
   ```

### Phase 5: Create Deployment Plan

**Deployment strategy options:**

**Option A: Blue-Green Deployment (Zero-downtime)**
```
Blue Environment: Currently serving production traffic
Green Environment: New version ready to test

Deployment Steps:
1. Deploy new code to Green environment
2. Run full test suite on Green
3. Warm up Green with requests (10 min)
4. Switch router to send traffic to Green
5. Monitor Green for 30 minutes
6. If OK: Blue becomes backup, green is primary
7. If issue: Switch back to Blue (instant rollback)
```

**Option B: Canary Deployment (Gradual rollout)**
```
Canary Release (5% of traffic):
1. Deploy new version
2. Route 5% of traffic to new version
3. Monitor for errors (30 minutes)
4. If healthy: Increase to 25%
5. Continue monitoring (30 minutes)
6. If healthy: Increase to 50%
7. Continue monitoring (30 minutes)
8. If healthy: Route 100% to new version
9. Keep old version running 2 hours for rollback

Rollback: Instant switch back to old version
```

**Option C: Rolling Deployment (Sequential updates)**
```
4 Servers, update one at a time:

1. Remove server-1 from load balancer
2. Deploy new code to server-1
3. Run health checks
4. Add server-1 back to load balancer
5. Monitor traffic (15 min)
6. Repeat for servers 2, 3, 4

Advantage: Zero downtime, gradual rollout
Disadvantage: Mixed versions running during deployment
```

**Selected: Blue-Green (fastest rollback, zero downtime)**

### Phase 6: Coordinate Deployment

**DevOps Engineer executes:**

```
DEPLOYMENT EXECUTION
====================

14:00 - Pre-deployment

- Notify team: #deployments channel
  "Starting deployment of version 1.2.3"
  "Expected duration: 15 minutes"
  "ETA completion: 14:15"

- Enable detailed logging
- Start deployment metrics collection
- Verify backup systems operational

14:05 - Deploy to Green Environment

- Deploy code artifact to green servers
  aws deploy create-deployment \
    --application-name my-app \
    --deployment-group-name green \
    --s3-location s3://releases/v1.2.3.zip

- Deploy takes 3 minutes
- Server status: OK ✓

14:08 - Database Migrations (if needed)

- Run migration script
  psql prod_db -f migrations/schema.sql
- Migration takes 2 minutes
- All tables verified ✓

14:10 - Run Health Checks

- Send test requests to Green
- Check application endpoints
- Verify database connectivity
- Check external service integrations

14:11 - Warm Up Green Environment

- Send traffic to green for warm-up
- Run synthetic monitoring (10 min)
- Verify performance metrics
- p95 response: 145ms ✓

14:21 - Switch Traffic to Green

- Switch load balancer to Green
- 100% of traffic now goes to Green
- Blue remains running, can switch back instantly

14:22 - Monitor Green

- Watch metrics for 30 minutes
- Error rate: 0% ✓
- Response time: 148ms (target: 200ms) ✓
- CPU: 42% (capacity: 80%) ✓
- Database: 8/20 connections ✓
- No alerts triggered ✓

14:52 - Declare Success

- Green is healthy and stable
- Blue becomes backup
- Notify team: "Deployment successful, version 1.2.3 live"

Rollback Available:
- If issues detected: Switch back to Blue (instant)
- Keep Blue running for 2 hours
- Then decommission Blue
```

### Phase 7: Post-Deployment Validation

**Test Engineer validates:**

1. **Smoke Tests**
   ```bash
   # Quick health check
   curl https://api.example.com/health
   # Response: {"status": "healthy", "version": "1.2.3"}
   ```

2. **Functional Tests**
   ```bash
   # Verify key features work
   curl -H "Authorization: Bearer $TOKEN" \
     https://api.example.com/api/users
   # Response: 200 OK with data
   ```

3. **User Testing**
   ```
   - Try logging in (auth works)
   - Try posting comment (feature works)
   - Try API call (returns correct data)
   - Check UI loads correctly
   - Verify new features work
   ```

### Phase 8: Monitor Production

**Monitoring Specialist tracks:**

```
POST-DEPLOYMENT MONITORING (2 hours)
====================================

Metrics to Track:
- Error rate: 0% (no spike)
- Response time: 145ms avg (baseline: 142ms)
- CPU usage: 42% (baseline: 40%)
- Memory: 245MB (baseline: 240MB)
- Database connections: 8/20 (baseline: 7/20)
- Requests/sec: 450 (baseline: 440)

Alerts Configured:
- Error rate > 1% → CRITICAL
- Response time > 300ms → HIGH
- CPU > 80% → HIGH
- Database connections > 18/20 → MEDIUM
- Out of memory → CRITICAL

Monitoring Results:
- 30 min check: All green ✓
- 60 min check: All green ✓
- 90 min check: All green ✓
- 120 min check: All green ✓

Status: STABLE, ready to remove Blue environment
```

### Phase 9: Commit and Document

**Store deployment results:**

```bash
cco knowledge store "Deployment: v1.2.3 - 2024-01-15
Timestamp: 14:00 - 14:52 (52 min)
Strategy: Blue-Green
Duration: 15 minutes (active)
Status: SUCCESS ✓

Changes:
- JWT auth feature complete
- API endpoints secured
- Performance optimized
- Tests: 450/450 passing

Validation:
- Smoke tests: ✓
- Functional tests: ✓
- Load tests: ✓
- Security: ✓
- Performance: ✓

Monitoring (2 hours):
- Error rate: 0%
- Performance: Stable
- User impact: None
- Rollback: Available but not needed

Bluegreen:
- Old version (Blue): Running as backup
- New version (Green): Running as primary
- Switch time: < 1 second

Next: Remove Blue in 2 hours" \
  --type completion \
  --agent deployment-engineer
```

### Phase 10: Post-Deployment Review

**Team reviews deployment:**

```
DEPLOYMENT RETROSPECTIVE
=======================

What went well:
✓ Deployment completed on time
✓ Zero downtime achieved
✓ All tests passed
✓ Performance stable
✓ No user impact
✓ Quick rollback capability

What could improve:
- Pre-deployment checklist took 30 min (target 20)
- Staging environment slightly different from prod
- Documentation could be clearer

Action items:
- Improve staging parity
- Update deployment guide
- Streamline checklist
- Add automated pre-checks
```

## Deployment Strategies Comparison

| Strategy | Downtime | Rollback | Complexity | Risk |
|----------|----------|----------|-----------|------|
| **Blue-Green** | 0 min | Instant | Low | Low |
| **Canary** | 0 min | Fast (5 min) | Medium | Low |
| **Rolling** | 0 min | Gradual | High | Medium |
| **Maintenance Window** | 10-30 min | Instant | Low | Medium |

## Rollback Procedure

**If problems detected:**

```bash
# 1. Switch back to previous version
# Blue-Green: Switch load balancer back to Blue (instant)
# Canary: Revert traffic to stable version (1 second)

# 2. Notify team
# "Rolling back to v1.2.2 due to [issue]"

# 3. Investigate root cause
# Logs, metrics, error traces

# 4. Plan fix
# Code review, testing, re-deployment

# 5. Re-deploy fixed version
# Once root cause fixed and tested

# 6. Document lessons learned
# Add to knowledge base
```

## Best Practices

1. **Automate everything** - Reduce human error
2. **Test thoroughly** - Before production
3. **Zero-downtime** - Always preferred
4. **Quick rollback** - Must be instant
5. **Monitor closely** - First 2 hours critical
6. **Communicate** - Team and users informed
7. **Coordinate timing** - Off-peak hours if possible
8. **Have checklist** - Nothing forgotten
9. **Document process** - Repeatable
10. **Learn from issues** - Improve next time

## Output Format

Return comprehensive deployment plan with:
- Deployment scope and changes
- Pre-deployment validation checklist
- Selected deployment strategy with rationale
- Step-by-step deployment procedure
- Rollback procedure with estimated time
- Health checks and validation tests
- Monitoring plan (first 2 hours)
- Post-deployment checklist
- Team communication plan
- Knowledge base references
