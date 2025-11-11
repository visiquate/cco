# Autonomous Operation Framework

## Overview

This framework enables the Claude Orchestra to operate autonomously for extended periods (4-8 hours) without user intervention, while maintaining quality, safety, and alignment with user intent.

---

## Core Components

### 1. Model Fallback System

**Purpose:** Ensure continuity when primary models hit token limits or rate limits.

**Configuration:**
```json
{
  "architect": {
    "model": "opus",
    "fallback": {
      "model": "sonnet-4.5",
      "triggers": ["token_limit", "rate_limit", "availability"],
      "tokenThreshold": 0.8,
      "automatic": true
    }
  }
}
```

**Fallback Triggers:**
1. **Token Limit**: Switch at 80% of token limit (proactive)
2. **Rate Limit**: Switch when rate limited (reactive)
3. **Availability**: Switch if primary model unavailable (reactive)

**Fallback Behavior:**
- Automatic and transparent to agents
- Previous context preserved via MCP memory
- Fallback logged for metrics
- Can fall back multiple times in one session
- Can restore to primary model when available

---

### 2. Compaction Management

**Purpose:** Preserve ALL critical state across conversation compactions.

**Pre-Compaction Protocol:**
```bash
# Run automatically before compaction
./scripts/pre-compaction.sh

# Exports to MCP memory:
# - Architect specification and decisions
# - Definition of Done
# - Credentials inventory and access methods
# - Integration configurations (Salesforce, Authentik, APIs)
# - Agent states (completed, in_progress, pending, blocked)
# - File structure and critical files
# - Test results and coverage
# - Completed milestones
# - Blocked tasks
```

**Post-Compaction Protocol:**
```bash
# Run immediately after compaction
./scripts/post-compaction.sh <SESSION_ID>

# Restores from MCP memory:
# - All state to /tmp/ files
# - Environment variables
# - Broadcasts restoration to all agents
```

**Memory Keys Structure:**
```
compaction/
  <SESSION_ID>/
    metadata (timestamp, session info)
    architect/
      specification
      decisions
      current_phase
    project/
      definition-of-done
    credentials/
      inventory
      access-methods
    integrations/
      salesforce/config
      authentik/config
      other-apis
    agents/
      <agent-name>/state
    files/
      structure
      critical
    testing/
      results
      coverage
    milestones/
      completed
    tasks/
      blocked
```

---

### 3. Error Recovery System

**Purpose:** Autonomously recover from common errors without user intervention.

**Error Classification:**

| Class | Examples | Recovery Strategy | Escalation |
|-------|----------|------------------|------------|
| **Recoverable** | API timeout, transient network error | Retry 3x with exponential backoff | After 3 failures |
| **Requires Fallback** | Rate limit, token exhaustion | Switch to fallback model/approach | After fallback fails |
| **Requires User** | Invalid credentials, unclear requirements | Document issue, request clarification | Immediate |

**Recovery Strategies:**

**1. Exponential Backoff Retry:**
```javascript
async function retryWithBackoff(operation, maxAttempts = 3) {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      if (attempt === maxAttempts) throw error;
      const delay = Math.pow(2, attempt) * 1000; // 2s, 4s, 8s
      await sleep(delay);
    }
  }
}
```

**2. Alternative Approach:**
```javascript
// If approach A fails, try approach B
const strategies = [strategyA, strategyB, strategyC];
for (const strategy of strategies) {
  try {
    return await strategy.execute();
  } catch (error) {
    logAttempt(strategy, error);
  }
}
// All strategies failed → escalate
```

**3. Graceful Degradation:**
```javascript
// If full feature fails, implement minimal viable version
try {
  return await fullFeatureImplementation();
} catch (error) {
  logDegradation(error);
  return await minimalViableImplementation();
}
```

---

### 4. Decision Authority Matrix

**Purpose:** Define what decisions agents can make autonomously vs. what requires user approval.

**Authority Levels:**

| Risk Level | Examples | Can Decide | Requires | Documentation |
|-----------|----------|------------|----------|---------------|
| **Low** | Code formatting, minor version updates, test strategies, file organization | Any agent | None | Optional |
| **Medium** | Technology within approved stack, API design, database schema, security approach | Architect + specialist | Architect approval | Required |
| **High** | New external service, major architecture change, breaking API change, production deploy | Architect | User checkpoint | Required |

**Decision Making Flow:**
```javascript
async function makeDecision(decision, agent) {
  const risk = assessRisk(decision);

  if (risk === 'low') {
    // Make decision immediately
    const result = await agent.decide(decision);
    await logDecision(decision, result, 'autonomous');
    return result;
  }

  if (risk === 'medium') {
    // Get architect approval
    const approval = await getArchitectApproval(decision);
    if (approval.approved) {
      const result = await agent.decide(decision);
      await documentDecision(decision, result, approval.rationale);
      return result;
    }
  }

  if (risk === 'high') {
    // Checkpoint with user
    return await requestUserApproval(decision);
  }
}
```

---

### 5. Progress Checkpointing

**Purpose:** Track progress continuously and enable resume from any point.

**Checkpoint Milestones:**
1. ✅ Discovery complete, specification generated
2. ✅ Architecture design complete
3. ✅ Each major component implemented
4. ✅ Tests passing for each component
5. ✅ Security audit complete
6. ✅ Documentation complete
7. ✅ Deployment scripts ready
8. ✅ Full integration tests passing

**Checkpoint Protocol:**
```javascript
async function createCheckpoint(milestone) {
  const state = {
    milestone: milestone,
    timestamp: new Date().toISOString(),
    agentStates: await captureAllAgentStates(),
    completedTasks: await getCompletedTasks(),
    pendingTasks: await getPendingTasks(),
    artifacts: await listProjectFiles(),
    testResults: await getLatestTestResults(),
    percentComplete: calculateProgress(milestone)
  };

  // Store in MCP memory
  await storeInMemory({
    key: `checkpoints/${milestone}/${Date.now()}`,
    value: state,
    ttl: 86400 // 24 hours
  });

  // Broadcast progress
  await broadcastProgress({
    milestone: milestone,
    percentComplete: state.percentComplete,
    estimatedTimeRemaining: estimateTimeRemaining(state)
  });
}
```

**Resume Protocol:**
```javascript
async function resumeFromCheckpoint(checkpointId) {
  const checkpoint = await retrieveFromMemory(`checkpoints/${checkpointId}`);

  // Restore agent states
  for (const [agentName, state] of Object.entries(checkpoint.agentStates)) {
    await restoreAgentState(agentName, state);
  }

  // Broadcast resumption
  await notifyAllAgents({
    type: 'resumption',
    checkpoint: checkpoint.milestone,
    nextSteps: determineNextSteps(checkpoint)
  });

  return checkpoint;
}
```

---

### 6. Heartbeat and Coordination

**Purpose:** Maintain coordination over long-running operations.

**Heartbeat Protocol:**
```javascript
// Each agent sends heartbeat every 10 minutes
setInterval(async () => {
  await sendHeartbeat({
    agent: agentName,
    timestamp: Date.now(),
    status: currentStatus,
    currentTask: currentTask,
    progressPercent: progressPercent
  });
}, 10 * 60 * 1000);
```

**Monitoring Protocol:**
```javascript
// Coordinator monitors heartbeats every 5 minutes
setInterval(async () => {
  const agents = await getAllAgents();

  for (const agent of agents) {
    const lastHeartbeat = await getHeartbeat(agent.name);
    const timeSinceHeartbeat = Date.now() - lastHeartbeat.timestamp;

    if (timeSinceHeartbeat > 15 * 60 * 1000) {
      // Agent hasn't checked in for 15 minutes
      await handleStaleAgent(agent);
    }
  }
}, 5 * 60 * 1000);
```

**Stale Agent Recovery:**
```javascript
async function handleStaleAgent(agent) {
  // Try to restore from last known state
  const lastState = await retrieveAgentState(agent.name);

  // Respawn agent with last known state
  await spawnAgent({
    ...agent,
    initialState: lastState,
    resumeMode: true
  });

  // Notify architect of recovery
  await notifyArchitect({
    type: 'agent_recovery',
    agent: agent.name,
    reason: 'stale_heartbeat'
  });
}
```

---

### 7. Autonomous Testing and Quality

**Purpose:** Automatically fix failing tests and improve code quality.

**Test Failure Recovery:**
```javascript
async function handleTestFailure(testResults) {
  // Analyze failure patterns
  const analysis = await analyzeFailures(testResults);

  // Common fixable patterns
  if (analysis.pattern === 'timing_issue') {
    return await addWaitStatements(analysis.tests);
  }

  if (analysis.pattern === 'flaky_test') {
    return await stabilizeTest(analysis.tests);
  }

  if (analysis.pattern === 'outdated_assertion') {
    return await updateAssertions(analysis.tests);
  }

  // Not automatically fixable
  // Escalate to coding agent with detailed analysis
  return await requestCodeFix({
    agent: analysis.responsibleAgent,
    testFailures: testResults.failures,
    analysis: analysis,
    suggestedFixes: analysis.suggestions
  });
}
```

**Quality Improvement Loop:**
```javascript
async function autonomousQualityImprovement() {
  // Run quality checks
  const metrics = await runQualityChecks();

  // Identify improvement opportunities
  const opportunities = identifyImprovements(metrics);

  for (const opportunity of opportunities) {
    if (opportunity.risk === 'low' && opportunity.impact === 'high') {
      // Safe, high-value improvement → do it autonomously
      await applyImprovement(opportunity);
      await documentImprovement(opportunity);
    }
  }
}
```

---

## Integration with Workflows

### Discovery Phase
```javascript
// Run comprehensive discovery interview
await runDiscoverySkill();

// Generate specification
const spec = await generateSpecification();

// Store in memory for compaction resilience
await storeInMemory({
  key: 'architect/specification',
  value: spec
});

// Create first checkpoint
await createCheckpoint('discovery_complete');
```

### Implementation Phase
```javascript
// Architecture design
await architect.designSystem(spec);
await createCheckpoint('architecture_complete');

// Parallel implementation
await Promise.all([
  codingAgents.implement(),
  qaAgent.createTests(),
  securityAgent.reviewDesign(),
  docsAgent.documentArchitecture()
]);

await createCheckpoint('implementation_complete');
```

### Quality Phase
```javascript
// Run tests
const testResults = await qaAgent.runAllTests();

// If tests fail, attempt autonomous recovery
if (!testResults.allPassed) {
  await handleTestFailure(testResults);
}

// Security audit
await securityAgent.fullAudit();

await createCheckpoint('quality_complete');
```

---

## Monitoring and Metrics

### Real-Time Metrics

**Track:**
- Agent heartbeats (alive/stale)
- Error recovery success rate
- Autonomous decision count by risk level
- Token usage by agent and model
- Test pass/fail rates
- Checkpoint frequency
- Time to milestone completion

**Dashboard View:**
```
Claude Orchestra Status Dashboard
============================
Uptime: 4h 23m | Autonomous: 97% | Interventions: 2

Agents (14):
  ✅ Architect (Sonnet 4.5 - fallback active)
  ✅ Python Specialist (Sonnet 4.5)
  ✅ Go Specialist (Sonnet 4.5)
  ...

Current Phase: Implementation (68% complete)
Next Milestone: Component 3 tests passing (est. 45m)

Recent Activity:
  [15:42] QA Engineer: Tests passing (87% coverage)
  [15:38] Python Specialist: API endpoints implemented
  [15:35] Security Auditor: No critical issues found
  [15:30] Checkpoint created: component_2_complete

Errors Recovered: 12
Decisions Made: 47 (42 low-risk, 5 medium-risk)
User Approvals Needed: 0
```

---

## Safety Mechanisms

### 1. Automatic Rollback
```javascript
if (qualityMetrics.criticalIssues > 0) {
  await rollbackToCheckpoint(lastGoodCheckpoint);
  await notifyUser('Critical issues detected. Rolled back to last good state.');
}
```

### 2. Budget Limits
```javascript
if (tokenUsage > tokenBudget * 0.9) {
  await notifyUser('Approaching token budget limit');
  await optimizeForEfficiency();
}
```

### 3. Security Gates
```javascript
// Security Auditor can block deployment
if (securityAudit.criticalVulnerabilities > 0) {
  blockDeployment();
  await notifyUser('Critical security issues must be resolved before deployment');
}
```

---

## Usage

### Starting Autonomous Operation

```bash
# 1. Run discovery (gather requirements)
# Use comprehensive questioning to understand project scope

# 2. Initialize army with Knowledge Manager
node src/orchestra-conductor.js "Build [your project]"

# 3. Army works autonomously with:
#    - Automatic model fallback (Opus → Sonnet 4.5)
#    - Knowledge Manager for persistent memory
#    - Error recovery via Knowledge Manager context
#    - Progress tracking in Knowledge Manager
```

### During Operation

**Army automatically:**
- ✅ Switches models when needed
- ✅ Recovers from errors
- ✅ Makes low/medium risk decisions
- ✅ Creates checkpoints at milestones
- ✅ Broadcasts progress
- ✅ Fixes failing tests
- ✅ Maintains coordination

**User intervention only needed for:**
- ❌ High-risk decisions
- ❌ New external services
- ❌ Production deployments
- ❌ Unrecoverable errors

### After Compaction

```bash
# Automatic restoration
./scripts/post-compaction.sh <SESSION_ID>

# Army resumes from last checkpoint with full context
```

---

## Future Enhancements

### Phase 2 (Month 2)
- Machine learning from past decisions
- Predictive error prevention
- Adaptive checkpoint intervals
- Performance optimization loops

### Phase 3 (Month 3)
- Multi-project learning
- Agent specialization refinement
- Automated refactoring
- Self-improvement from retrospectives

---

## Success Criteria

**Week 1:**
- ✅ 2 hours autonomous operation
- ✅ 70% error recovery rate
- ✅ Model fallback working
- ✅ Compaction resilience

**Month 1:**
- ✅ 4 hours autonomous operation
- ✅ 85% error recovery rate
- ✅ 80% autonomous decisions
- ✅ < 5 user interventions per session

**Month 3:**
- ✅ 8+ hours autonomous operation
- ✅ 95% error recovery rate
- ✅ 95% autonomous decisions
- ✅ Only high-risk decisions require approval

The framework provides a robust foundation for extended autonomous operation while maintaining quality, safety, and user control where it matters most.
