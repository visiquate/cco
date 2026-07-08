# Review Code: Code Review Workflow

## Trigger
`/review-code <file or requirement>`

Orchestrates comprehensive code review combining architecture, security, and quality perspectives.

## Purpose
The Review Code skill provides:
- Multi-perspective code review
- Security vulnerability detection
- Architecture compliance checking
- Best practices validation
- Performance impact assessment
- Consolidated feedback

## Workflow

### Phase 1: Identify Review Scope

1. **Analyze code changes**
   - File count and size
   - Languages involved
   - Changed functionality
   - Risk assessment

2. **Determine review type**
   - Feature implementation
   - Bug fix
   - Refactoring
   - Infrastructure code
   - Security-related changes

### Phase 2: Query Knowledge Base

1. **Retrieve architecture decisions**
   ```bash
   cco knowledge search "architecture decisions"
   ```

2. **Retrieve security policies**
   ```bash
   cco knowledge search "security requirements"
   ```

3. **Retrieve performance constraints**
   ```bash
   cco knowledge search "performance targets"
   ```

### Phase 3: Spawn Review Team

**Core Review Team:**
- **Code Reviewer** (Sonnet) - General code quality
- **Security Auditor** (Sonnet) - Security review
- **Architect Review** (Sonnet) - Architecture compliance (if major change)
- **Performance Engineer** (Sonnet) - Performance impact (if applicable)

**Spawn all in parallel** - Different perspectives, no blocking

```
Review Code Skill
    │
    ├─→ Code Reviewer (Sonnet)
    │      Code quality
    │      Best practices
    │      Maintainability
    │      Naming/structure
    │
    ├─→ Security Auditor (Sonnet)
    │      Security vulnerabilities
    │      Authentication/authorization
    │      Data protection
    │      Input validation
    │
    ├─→ Architect Review (Sonnet) [if major]
    │      Architecture compliance
    │      Design pattern validation
    │      API consistency
    │
    └─→ Performance Engineer (Sonnet) [if applicable]
           Performance impact
           Algorithmic efficiency
           Resource usage
```

### Phase 4: Code Review by Code Reviewer (Sonnet)

**Focus Areas:**

1. **Code Quality**
   - Readability and clarity
   - Complexity (cyclomatic, cognitive)
   - Duplication (DRY principle)
   - Error handling

2. **Best Practices**
   - Language idioms
   - Framework conventions
   - Design patterns
   - Standard library usage

3. **Maintainability**
   - Function/method length
   - Parameter count
   - Type hints (if applicable)
   - Comments and documentation

4. **Testing**
   - Test coverage
   - Test quality
   - Mocking appropriateness
   - Edge case coverage

**Output Format:**
```
Finding: [Issue Name]
Severity: [Critical/High/Medium/Low]
Location: [File]:[Line]
Description: [What's wrong]
Reason: [Why it matters]
Suggestion: [How to fix]

Example:
---
Finding: Uncaught Exception in API Handler
Severity: High
Location: src/auth.py:45
Description: API endpoint doesn't handle JWT decode errors
Reason: Users see 500 errors instead of 401 when JWT is invalid
Suggestion: Add try-except for jwt.InvalidSignatureError
```

### Phase 5: Security Audit (Security Auditor)

**Focus Areas:**

1. **Input Validation**
   - SQL injection prevention
   - XSS prevention (for web)
   - Command injection prevention
   - Path traversal prevention

2. **Authentication & Authorization**
   - Credential handling
   - Session management
   - Permission checking
   - OAuth/JWT correctness

3. **Data Protection**
   - Encryption at rest
   - Encryption in transit
   - Sensitive data logging
   - PII handling

4. **Secret Management**
   - Hardcoded secrets
   - Environment variable usage
   - Secret rotation
   - Credential storage

5. **Dependency Security**
   - Known vulnerabilities in dependencies
   - Dependency version constraints
   - Transitive dependency issues

**Output Format:**
```
Security Finding: [Vulnerability Name]
Severity: [Critical/High/Medium/Low]
Location: [File]:[Line]
Vulnerability: [Technical details]
Impact: [What's at risk]
Remediation: [How to fix]
CWE: [Common Weakness Enumeration ID]

Example:
---
Security Finding: Hardcoded API Key
Severity: Critical
Location: src/config.py:12
Vulnerability: API key visible in source code
Impact: Anyone with code access can use API key
Remediation: Move to environment variable or secrets manager
CWE: CWE-798 (Use of Hard-coded Credentials)
```

### Phase 6: Architecture Review (Architect Review)

**Focus Areas** (if applicable):

1. **API Design**
   - Endpoint consistency
   - HTTP method appropriateness
   - Status code correctness
   - Error response format

2. **Data Flow**
   - Request validation
   - Data transformation
   - Response formatting
   - Async handling

3. **Integration Points**
   - Dependency injection
   - Service coupling
   - Event handling
   - Message queue usage

4. **Scalability Patterns**
   - Caching strategy
   - Database query efficiency
   - Connection pooling
   - Rate limiting

**Output Format:**
```
Architecture Finding: [Issue Name]
Severity: [Critical/High/Medium/Low]
Location: [File]
Current Design: [What's implemented]
Issue: [Why it's problematic]
Architecture Decision: [Related decision from KB]
Suggestion: [Aligned approach]

Example:
---
Architecture Finding: Inconsistent API Response Format
Severity: Medium
Location: src/api/endpoints.py
Current Design: Some endpoints return {data: X}, others return X directly
Issue: Inconsistency makes client implementation difficult
Architecture Decision: Standard response format from KB
Suggestion: All endpoints wrap data in {data: X, meta: {...}}
```

### Phase 7: Performance Review (Performance Engineer)

**Focus Areas** (if applicable):

1. **Algorithm Efficiency**
   - Big-O analysis
   - Unnecessary loops/recursion
   - Inefficient string operations
   - Database query N+1 problems

2. **Resource Usage**
   - Memory leaks
   - Large object allocations
   - Connection pooling
   - Caching opportunities

3. **Concurrent Execution**
   - Race conditions
   - Deadlock potential
   - Lock contention
   - Async/await patterns

**Output Format:**
```
Performance Finding: [Issue Name]
Severity: [Critical/High/Medium/Low]
Location: [File]:[Function]
Problem: [What's slow]
Current: [Current approach]
Measurement: [How bad it is]
Suggestion: [Optimization]

Example:
---
Performance Finding: N+1 Database Queries
Severity: High
Location: src/api/get_user_posts
Problem: Fetches user, then queries DB for each post comment
Current: 1 user query + N post queries + N*M comment queries
Measurement: 1000ms for page with 20 posts
Suggestion: Use eager loading/joins to fetch all data at once
```

### Phase 8: Aggregate Findings

1. **Collect all findings** from reviewers
2. **Store in knowledge base**
   ```bash
   cco knowledge store "Code Review: [file]
   - Code Quality: [findings]
   - Security: [findings]
   - Architecture: [findings]
   - Performance: [findings]" \
     --type status \
     --agent code-reviewer
   ```

3. **Prioritize by severity**
   ```
   Critical: Must fix before merge
   High: Should fix before merge
   Medium: Fix in next iteration
   Low: Nice to have improvements
   ```

### Phase 9: Generate Review Summary

**Structure:**
```
CODE REVIEW SUMMARY
==================

Files Reviewed: [N files, X lines changed]
Reviewers: [Code Reviewer, Security Auditor, Architect Review, Performance Engineer]

CRITICAL FINDINGS: [N]
- [Finding 1]
- [Finding 2]

HIGH FINDINGS: [N]
- [Finding 1]
- [Finding 2]

MEDIUM FINDINGS: [N]
- [Finding 1]

LOW FINDINGS: [N]

OVERALL ASSESSMENT: [Pass/Conditional/Fail]

NEXT STEPS:
1. Address critical findings
2. Address high findings
3. Re-review before merge

KNOWLEDGE BASE REFERENCES:
- Architecture decision: [link]
- Security policy: [link]
```

## Review Types

### Quick Review (Small Change)
**Spawn**: Code Reviewer + Security Auditor (if touching security)
**Duration**: 5-10 minutes
**When**: Bug fix, small feature, documentation

### Standard Review (Medium Change)
**Spawn**: Code Reviewer + Security Auditor + Architect Review
**Duration**: 15-25 minutes
**When**: Feature implementation, refactoring

### Comprehensive Review (Major Change)
**Spawn**: All four reviewers + Performance Engineer
**Duration**: 30-45 minutes
**When**: New service, major refactoring, security-critical feature

## Review Checklist

### Code Reviewer Checklist
- [ ] Code is readable and understandable
- [ ] Variable/function names are clear
- [ ] No unnecessary complexity
- [ ] Error handling is present
- [ ] No code duplication (DRY)
- [ ] Tests cover the changes
- [ ] Documentation is updated
- [ ] Follows language idioms
- [ ] No obvious bugs
- [ ] Performance is reasonable

### Security Auditor Checklist
- [ ] No hardcoded secrets
- [ ] Input is validated
- [ ] No SQL injection risks
- [ ] No XSS vulnerabilities
- [ ] Authentication is correct
- [ ] Authorization is checked
- [ ] Sensitive data is protected
- [ ] Dependencies are up-to-date
- [ ] No known vulnerabilities
- [ ] Follows security policy

### Architect Review Checklist
- [ ] Follows architecture pattern
- [ ] API design is consistent
- [ ] Data flow is clear
- [ ] Service boundaries respected
- [ ] No unwanted coupling
- [ ] Scalability considered
- [ ] Caching strategy used
- [ ] Async patterns appropriate
- [ ] Event handling correct
- [ ] Integration points clean

### Performance Engineer Checklist
- [ ] Algorithms are efficient
- [ ] No N+1 query problems
- [ ] Memory usage acceptable
- [ ] Concurrency handled safely
- [ ] Connection pooling used
- [ ] Caching leveraged
- [ ] No unnecessary allocations
- [ ] Resource cleanup present
- [ ] Async/await correct
- [ ] Latency targets met

## Knowledge Base Integration

### Store Review Results
```bash
cco knowledge store "Code Review: [requirement]
Status: [Complete/Conditional/Failed]
Critical: [count] - [list]
High: [count] - [list]
Medium: [count] - [list]
Approved By: [Architect]" \
  --type status \
  --agent code-reviewer
```

### Query Architecture for Context
```bash
cco knowledge search "architecture decision [component]"
cco knowledge search "security requirements"
cco knowledge search "performance targets"
```

## Best Practices

1. **Review early and often** - Don't wait for completion
2. **Be specific** - Point to exact lines
3. **Be constructive** - Explain why, not just what
4. **Consider context** - Understand the full change
5. **Trust specialists** - Different reviewers have different focuses
6. **Document decisions** - Store non-obvious choices
7. **Prioritize correctly** - Security > Architecture > Performance > Style
8. **Be timely** - Review within 24 hours
9. **Celebrate good code** - Acknowledge excellent work
10. **Track patterns** - Look for recurring issues

## Output Format

Return comprehensive review report with:
- File overview and change summary
- Findings organized by severity
- Findings organized by category (code quality, security, architecture, performance)
- Specific line references with suggestions
- Architecture/security/performance decisions from knowledge base
- Overall assessment and approval status
- Action items and next steps
- Knowledge base references
