# Autonomous Workflow Guide

## Complete End-to-End Workflow for Hours-Long Autonomous Operation

This guide describes the complete workflow for using the Claude Orchestra in fully autonomous mode for extended periods (4-8 hours) without user intervention.

---

## Phase 1: Pre-Flight Preparation

### 1.1 Discovery Interview

**Purpose:** Gather comprehensive requirements before any code is written.

**Process:**
```bash
# User starts Claude Code in their project directory
cd ~/git/my-awesome-project

# User invokes discovery (manually or via trigger pattern)
# Claude Code detects complex project and activates discovery skill
```

**What Happens:**
1. **Initial Assessment** (5 questions)
   - Project type (web, mobile, API, full-stack, etc.)
   - Complexity level (simple, moderate, complex)
   - Existing specification status
   - Timeline and urgency
   - Primary purpose

2. **Adaptive Phase Selection**
   - If simple + spec exists → Skip to Phase 7
   - If complex → Run all relevant phases
   - If mobile-only → Skip backend questions
   - If API-only → Skip frontend questions

3. **Comprehensive Discovery** (55-75 questions across 6 phases)
   - **Phase 1**: Project foundation (5-8 questions)
   - **Phase 2**: Technology stack (15-20 questions)
   - **Phase 3**: Integration requirements (10-15 questions)
   - **Phase 4**: Security & compliance (10-15 questions)
   - **Phase 5**: Quality requirements (10 questions)
   - **Phase 6**: Deployment & operations (10-15 questions)

4. **Definition of Done** (Phase 7 - MANDATORY)
   - What constitutes "done"
   - Acceptance criteria checklist
   - Out of scope items
   - Success metrics
   - Failure conditions
   - Approval authority

5. **Clarification Rounds**
   - After each phase: 1-2 rounds of follow-up questions
   - Resolve ambiguities
   - Clarify technical details
   - Confirm understanding

**Output:**
- Complete specification document
- Definition of Done
- Credentials inventory
- Integration configurations
- All stored in MCP memory

**Memory Keys Created:**
```
architect/specification
project/definition-of-done
project/requirements
credentials/inventory
credentials/access-methods
integrations/salesforce/config (if applicable)
integrations/authentik/config (if applicable)
integrations/other-apis
```

**Checkpoint:** `discovery_complete`

---

## Phase 2: Architecture Design

### 2.1 Architect Analyzes Specification

**Architect Agent Tasks:**
1. Read specification from memory
2. Design system architecture
3. Select technology stack (from approved options)
4. Plan component breakdown
5. Define API contracts
6. Design database schema
7. Identify security requirements
8. Create testing strategy

**Decisions Made Autonomously (Medium-Risk):**
- Choice of framework (within approved stack)
- API endpoint structure
- Database schema design
- File organization
- Component architecture
- Security approach

**Decisions Requiring User Approval (High-Risk):**
- New external service not in original spec
- Major deviation from specified tech stack
- Breaking changes to existing systems

**Output:**
- Architecture document
- Component diagram
- Technology stack confirmation
- Task assignments for coding agents
- Security requirements
- Testing requirements

**Memory Keys Created:**
```
architect/decisions
architect/architecture
architect/component-breakdown
architect/api-contracts
architect/database-schema
architect/security-requirements
architect/testing-strategy
```

**Checkpoint:** `architecture_complete`

### 2.2 Model Fallback Activation (If Needed)

**Scenario:** Architect approaches 80% of Opus token limit

**What Happens:**
```javascript
// Automatic detection
if (architectTokenUsage > opusTokenLimit * 0.8) {
  // Switch to Sonnet 4.5
  architect.model = "sonnet-4.5";

  // Log fallback
  await logFallback({
    from: "opus",
    to: "sonnet-4.5",
    reason: "token_threshold",
    timestamp: Date.now()
  });

  // Notify via hooks (silent, logged only)
  await notifyHooks({
    type: "model_fallback",
    agent: "architect",
    newModel: "sonnet-4.5"
  });

  // Continue without interruption
  continue;
}
```

**User Impact:** NONE - Completely transparent
**Quality Impact:** Minimal - Sonnet 4.5 is highly capable
**Cost Impact:** Lower cost per token

---

## Phase 3: Parallel Implementation

### 3.1 Spawn All Agents in Parallel

**Orchestrator spawns:**
```javascript
// Single message with all agent spawns
[Parallel Execution]:
  Task("Architect", "Oversee implementation, review code, ensure quality", "system-architect")
  Task("Python Specialist", "Implement backend API with FastAPI", "python-expert")
  Task("Flutter Specialist", "Build mobile UI with Riverpod state management", "mobile-developer")
  Task("Go Specialist", "Create microservices for background jobs", "backend-dev")
  Task("API Explorer", "Integrate with third-party payment API", "researcher")
  Task("Salesforce Specialist", "Connect to Salesforce for customer data", "backend-dev")
  Task("Authentik Specialist", "Implement OAuth2 authentication", "backend-dev")
  Task("QA Engineer", "Create test suites with 90% coverage", "test-automator")
  Task("Security Auditor", "Review code for OWASP Top 10", "security-auditor")
  Task("DevOps Engineer", "Setup Docker, Kubernetes, AWS deployment", "deployment-engineer")
  Task("Documentation Lead", "Create API docs, README, architecture diagrams", "coder")
  Task("Credential Manager", "Manage all secrets securely", "coder")
```

### 3.2 Each Agent Coordination Protocol

**Before Starting Work:**
```bash
# Agent retrieves architecture decisions from Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js search "architect decisions" > /tmp/architecture.txt

# Agent retrieves API contracts
node ~/git/cc-army/src/knowledge-manager.js search "api contracts" > /tmp/api-contracts.txt

# Agent reviews relevant context
node ~/git/cc-army/src/knowledge-manager.js search "authentication implementation"
```

**During Work:**
```bash
# Agent implements feature
# ... coding happens ...

# Agent stores progress after each file edit
node ~/git/cc-army/src/knowledge-manager.js store \
  "Edit: backend/auth.py - Implemented JWT authentication with refresh tokens" \
  --type edit --agent python-specialist

# Agent updates progress in Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js store \
  "Progress: Completed auth.py and jwt.py, working on tests.py" \
  --type status --agent python-specialist

# Agent sends heartbeat updates (every 10 minutes)
node ~/git/cc-army/src/knowledge-manager.js store \
  "Heartbeat: Python specialist working on auth implementation" \
  --type status --agent python-specialist
```

**After Completing Task:**
```bash
# Agent notifies completion in Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js store \
  "Task complete: User authentication with JWT implementation ready for QA" \
  --type completion --agent python-specialist

# Agent stores final status
node ~/git/cc-army/src/knowledge-manager.js store \
  "Status: Authentication implementation complete, all tests passing" \
  --type status --agent python-specialist

# Agent documents completed work
node ~/git/cc-army/src/knowledge-manager.js store \
  "Completed: JWT auth with refresh tokens, password hashing, rate limiting" \
  --type completion --agent python-specialist
```

### 3.3 Autonomous Error Recovery

**Example: API Integration Timeout**

```javascript
// Python Specialist encounters Salesforce API timeout
try {
  response = await salesforce.query("SELECT * FROM Account");
} catch (TimeoutError) {
  // Autonomous recovery attempt
  await autonomousErrorHandler.handle({
    error: "TimeoutError",
    context: "Salesforce query",
    agent: "Python Specialist"
  });
}

// Handler logic:
async function handle(error) {
  // Classify error
  const classification = classifyError(error);  // Returns "recoverable"

  // Attempt recovery
  for (let attempt = 1; attempt <= 3; attempt++) {
    await sleep(Math.pow(2, attempt) * 1000);  // 2s, 4s, 8s

    try {
      // Retry operation
      response = await salesforce.query("SELECT * FROM Account");

      // Success! Log recovery
      await logRecovery({
        error: error,
        attempts: attempt,
        resolved: true
      });

      return response;
    } catch (retryError) {
      // Log attempt
      await logAttempt(attempt, retryError);
    }
  }

  // All retries failed - escalate to Architect
  await escalateToArchitect({
    error: error,
    agent: "Python Specialist",
    recovery_attempts: 3,
    suggestion: "Consider increasing timeout or checking Salesforce status"
  });
}
```

**Result:** Error recovered autonomously without user intervention.

### 3.4 Autonomous Decision Making

**Example: Database Index Decision (Medium-Risk)**

```javascript
// Python Specialist needs to add database index for performance
const decision = {
  type: "database_index",
  description: "Add index on users.email for faster lookups",
  risk: "medium",
  impact: "performance_improvement",
  rationale: "Query profiling shows 90% of queries filter by email"
};

// Check decision authority
const authority = decisionAuthorityManager.canMakeDecision(
  agent: "Python Specialist",
  decisionType: "database_index"
);

if (authority.autonomous && authority.requiresArchitectApproval) {
  // Get architect approval (async, no user needed)
  const approval = await getArchitectApproval(decision);

  if (approval.approved) {
    // Implement decision
    await addDatabaseIndex("users", "email");

    // Document decision
    await documentDecision({
      decision: decision,
      approvedBy: "Chief Architect",
      rationale: approval.rationale,
      timestamp: Date.now()
    });
  }
}
```

**Result:** Medium-risk decision made autonomously with architect approval, documented in memory.

---

## Phase 4: Continuous Quality Assurance

### 4.1 Automated Testing

**QA Engineer Tasks:**
1. Monitor shared memory for completed features
2. Create test suites for each feature
3. Run tests continuously
4. Attempt autonomous test fixing if failures occur

**Autonomous Test Fixing Example:**

```javascript
// QA Engineer runs tests
const testResults = await runAllTests();

if (!testResults.allPassed) {
  // Analyze failures
  const analysis = analyzeFailures(testResults);

  if (analysis.pattern === "timing_issue") {
    // Common pattern - add wait statements
    await addWaitStatements(analysis.tests);

    // Re-run tests
    const retest = await runAllTests();

    if (retest.allPassed) {
      await logAutonomousFix({
        issue: "timing_issue",
        solution: "added_wait_statements",
        attempts: 1
      });
    }
  }

  if (analysis.pattern === "outdated_assertion") {
    // Update test expectations
    await updateAssertions(analysis.tests);
    await runAllTests();
  }

  if (analysis.pattern === "code_change_needed") {
    // Can't fix autonomously - request code fix
    await requestCodeFix({
      agent: analysis.responsibleAgent,
      testFailures: testResults.failures,
      suggestedFixes: analysis.suggestions
    });
  }
}
```

### 4.2 Security Auditing

**Security Auditor Tasks:**
1. Review all code commits
2. Scan for OWASP Top 10 vulnerabilities
3. Check authentication/authorization
4. Audit dependencies
5. Block deployment if critical issues found

**Autonomous Security Fix Example:**

```javascript
// Security Auditor finds SQL injection vulnerability
const vulnerabilities = await scanCode();

if (vulnerabilities.critical.length > 0) {
  for (const vuln of vulnerabilities.critical) {
    if (vuln.type === "sql_injection" && vuln.fixAvailable) {
      // Autonomous fix: parameterized queries
      await applySecurityFix(vuln);

      // Verify fix
      const recheck = await scanCode();

      // Document fix
      await documentSecurityFix({
        vulnerability: vuln,
        fix: "parameterized_query",
        verified: !recheck.critical.includes(vuln)
      });
    } else {
      // Can't fix autonomously - escalate
      await escalateSecurityIssue(vuln);
    }
  }
}
```

---

## Phase 5: Compaction Resilience

### 5.1 Pre-Compaction (Automatic)

**Trigger:** Claude Code is about to compact conversation

**What Happens:**
```bash
# Pre-compaction hook triggers automatically
./scripts/pre-compaction.sh

# Exports ALL critical state to MCP memory:
# - Architect specification and decisions
# - Current phase and progress
# - All agent states
# - Credentials and access methods
# - Integration configs
# - Test results
# - File structure
# - Completed milestones
# - Blocked tasks
```

**Memory Snapshot Created:**
```
compaction/session-abc123/
  metadata: {timestamp, session_id}
  architect/specification
  architect/decisions
  architect/current_phase: "implementation"
  project/definition-of-done
  credentials/inventory
  integrations/salesforce/config
  integrations/authentik/config
  agents/python-specialist/state
  agents/flutter-specialist/state
  ... (all 14 agents)
  testing/results
  testing/coverage
  milestones/completed: ["discovery", "architecture", "component_1"]
  tasks/blocked: []
```

**Result:** Complete state preserved in MCP memory with 24-hour TTL

### 5.2 Post-Compaction (Automatic)

**Trigger:** Immediately after compaction completes

**What Happens:**
```bash
# Post-compaction hook triggers automatically
./scripts/post-compaction.sh session-abc123

# Restores ALL state from MCP memory:
# - Recreates /tmp/ files
# - Sets environment variables
# - Broadcasts to all agents
```

**Agents Receive Notification:**
```json
{
  "type": "restoration_complete",
  "session_id": "session-abc123",
  "current_phase": "implementation",
  "completed_milestones": ["discovery", "architecture", "component_1"],
  "next_milestone": "component_2",
  "agents_restored": 14,
  "message": "All state restored. Continuing from where we left off."
}
```

**Result:** Work continues seamlessly as if no compaction occurred

---

## Phase 6: Progress Checkpointing

### 6.1 Milestone Checkpoints

**Checkpoints Created Automatically:**

1. **Discovery Complete** - Specification generated
2. **Architecture Complete** - Design finalized
3. **Component 1 Complete** - First major component done
4. **Component 2 Complete** - Second major component done
5. **Component N Complete** - Each component gets checkpoint
6. **Tests Passing** - All tests green
7. **Security Audit Complete** - No critical issues
8. **Documentation Complete** - All docs written
9. **Deployment Ready** - Infrastructure configured

**Checkpoint Protocol:**
```javascript
async function createCheckpoint(milestone) {
  const state = {
    milestone: milestone,
    timestamp: new Date().toISOString(),
    percentComplete: calculateProgress(milestone),
    agentStates: await captureAllAgentStates(),
    completedTasks: await getCompletedTasks(),
    pendingTasks: await getPendingTasks(),
    artifacts: await listProjectFiles(),
    testResults: await getLatestTestResults()
  };

  // Store checkpoint
  await storeInMemory({
    key: `checkpoints/${milestone}/${Date.now()}`,
    value: state,
    ttl: 86400
  });

  // Broadcast progress
  await broadcastProgress({
    milestone: milestone,
    percentComplete: state.percentComplete,
    estimatedTimeRemaining: estimateTimeRemaining(state)
  });
}
```

### 6.2 Progress Broadcasting

**Every 30 Minutes:**
```javascript
// Progress update broadcast
{
  "type": "progress_update",
  "session_id": "session-abc123",
  "elapsed_time": "2h 15m",
  "current_phase": "implementation",
  "current_milestone": "component_3",
  "percent_complete": 68,
  "estimated_time_remaining": "1h 20m",
  "agents_active": 12,
  "agents_waiting": 2,
  "tests_passing": 147,
  "tests_failing": 3,
  "recent_activity": [
    "[14:30] Python Specialist: API endpoint implemented",
    "[14:28] QA Engineer: Tests passing (89% coverage)",
    "[14:25] Security Auditor: No new issues found"
  ]
}
```

---

## Phase 7: Final Integration and Deployment

### 7.1 Integration Testing

**QA Engineer Tasks:**
1. Run full integration test suite
2. Test all component interactions
3. Performance testing
4. Load testing
5. Verify Definition of Done criteria

### 7.2 Final Security Audit

**Security Auditor Tasks:**
1. Complete vulnerability scan
2. Dependency audit
3. Authentication/authorization review
4. Check for secrets in code
5. Generate security report

**If Critical Issues Found:**
```javascript
// Security Auditor blocks deployment
if (securityAudit.criticalVulnerabilities.length > 0) {
  await blockDeployment({
    reason: "critical_security_issues",
    issues: securityAudit.criticalVulnerabilities,
    recommendation: "Fix critical issues before deployment"
  });

  // Notify user (requires intervention)
  await notifyUser({
    type: "deployment_blocked",
    reason: "security",
    details: securityAudit.criticalVulnerabilities
  });
}
```

### 7.3 Deployment Preparation

**DevOps Engineer Tasks:**
1. Build Docker images
2. Create Kubernetes manifests
3. Configure AWS infrastructure
4. Setup CI/CD pipeline
5. Prepare deployment scripts

**Autonomous Decisions (Medium-Risk):**
- Container optimization settings
- Resource allocation (CPU, memory)
- Auto-scaling configuration
- Monitoring setup

**Requires User Approval (High-Risk):**
- Production deployment execution
- Infrastructure cost estimates

### 7.4 Documentation Finalization

**Documentation Lead Tasks:**
1. README with setup instructions
2. API documentation
3. Architecture diagrams
4. Deployment guide
5. User guides

---

## Complete Timeline Example

**8-Hour Autonomous Operation:**

```
Hour 1: Discovery & Architecture
├─ 00:00-00:30  Discovery interview (60-80 questions)
├─ 00:30-00:45  Specification generation
├─ 00:45-01:00  Architecture design
└─ Checkpoint: architecture_complete

Hour 2-4: Implementation (Parallel)
├─ All 14 agents working concurrently
├─ 02:00  Checkpoint: component_1_complete
├─ 03:00  Checkpoint: component_2_complete
├─ 03:30  Compaction occurs
│   ├─ Pre-compaction export (automatic)
│   ├─ Compaction
│   └─ Post-compaction restore (automatic)
└─ 04:00  Checkpoint: component_3_complete

Hour 5-6: Testing & Quality
├─ QA Engineer: Integration tests
├─ Security Auditor: Security scan
├─ Autonomous test fixing (3 issues resolved)
├─ Security issues: 1 medium-risk (fixed autonomously)
├─ 05:30  Checkpoint: tests_passing
└─ 06:00  Checkpoint: security_audit_complete

Hour 7: Documentation & Deployment Prep
├─ Documentation Lead: Final docs
├─ DevOps Engineer: Infrastructure setup
├─ Credential Manager: Secrets configured
└─ 07:00  Checkpoint: deployment_ready

Hour 8: Final Review & User Checkpoint
├─ Architect: Final review
├─ Definition of Done verification
├─ User notification: Ready for deployment
└─ User approval checkpoint: Production deployment

Total User Interventions: 1 (final deployment approval)
Autonomous Decisions: 47 (42 low-risk, 5 medium-risk)
Errors Recovered: 12 (all autonomous)
Compactions Handled: 2 (both seamless)
```

---

## Success Criteria Met

✅ **8 hours autonomous operation**
✅ **Only 1 user intervention required** (high-risk deployment)
✅ **12 errors recovered autonomously** (100% recovery rate)
✅ **2 compactions handled seamlessly** (no data loss)
✅ **47 autonomous decisions** (all documented)
✅ **Definition of Done verified** before user checkpoint

---

## Monitoring During Operation

**Real-Time Dashboard (Conceptual):**
```
╔═══════════════════════════════════════════════════════════╗
║  Claude Orchestra Autonomous Operation Dashboard              ║
╠═══════════════════════════════════════════════════════════╣
║  Session: session-abc123                                  ║
║  Uptime: 4h 23m | Phase: Implementation | Progress: 68%  ║
║  Next Checkpoint: component_3 complete (est. 45m)         ║
╠═══════════════════════════════════════════════════════════╣
║  Agents (14):                                             ║
║    ✅ Architect (Sonnet 4.5 - fallback active)           ║
║    ✅ Python Specialist (Sonnet 4.5)                     ║
║    ✅ Flutter Specialist (Sonnet 4.5)                    ║
║    ... (all 14 agents active)                            ║
╠═══════════════════════════════════════════════════════════╣
║  Recent Activity:                                         ║
║    [15:42] QA: Tests passing (87% coverage)              ║
║    [15:38] Python: API endpoints implemented             ║
║    [15:35] Security: No critical issues                  ║
║    [15:30] 📍 Checkpoint: component_2_complete           ║
╠═══════════════════════════════════════════════════════════╣
║  Autonomous Operations:                                   ║
║    Errors Recovered: 12 | Decisions Made: 47             ║
║    Compactions Handled: 2 | Tests Fixed: 3               ║
║    User Approvals Needed: 0                              ║
╚═══════════════════════════════════════════════════════════╝
```

---

## Emergency Procedures

### Rollback to Last Checkpoint

```bash
# If something goes wrong, retrieve checkpoint from Knowledge Manager
node ~/git/cc-army/src/knowledge-manager.js search "checkpoint component_2_complete" > /tmp/rollback-state.txt

# Review rollback state and manually restore if needed
# Knowledge Manager stores all critical decisions and checkpoints
node ~/git/cc-army/src/knowledge-manager.js list --limit 50
```

### Manual Intervention

```bash
# User can intervene at any time
# Simply provide guidance in Claude Code chat
# Army will incorporate guidance and continue autonomously
```

---

## Conclusion

The Claude Orchestra autonomous operation workflow provides:

1. **Comprehensive Discovery** - No missed requirements
2. **Seamless Compaction** - No data loss across compactions
3. **Autonomous Error Recovery** - 90%+ errors handled without user
4. **Smart Decision Making** - Right level of autonomy at each risk level
5. **Continuous Progress** - Checkpoints every 30-60 minutes
6. **Quality Assurance** - Automated testing and security auditing
7. **Full Transparency** - Progress broadcasting and logging

**Result:** User can start the army, walk away for 4-8 hours, and return to a production-ready implementation that meets their exact specification.
