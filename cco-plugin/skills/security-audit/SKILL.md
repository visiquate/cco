# Security Audit: Security Review

## Trigger
`/security-audit <code, feature, or requirement>`

Orchestrates comprehensive security review and compliance audit.

## Purpose
The Security Audit skill provides:
- Full security vulnerability assessment
- Compliance checking
- Penetration testing coordination
- Data protection validation
- Authentication/authorization review
- Infrastructure security assessment

## Workflow

### Phase 1: Define Audit Scope

1. **Identify what to audit**
   - New feature implementation
   - API endpoints
   - Data flows
   - Authentication system
   - Infrastructure configuration
   - Dependency security

2. **Determine audit type**
   - Feature security review
   - API security audit
   - Infrastructure security assessment
   - Penetration testing
   - Compliance verification
   - Dependency vulnerability scan

3. **Query security requirements**
   ```bash
   cco knowledge search "security requirement"
   cco knowledge search "compliance"
   cco knowledge search "data protection"
   ```

### Phase 2: Spawn Security Team

**Parallel Security Review Team:**
- **Security Auditor** (Sonnet) - General security review
- **Security Engineer** (Sonnet) - Secure implementation
- **Penetration Tester** (Sonnet) - Security testing
- **Compliance Specialist** (Haiku) - Compliance checking

```
Security Audit
    │
    ├─→ Security Auditor (Sonnet)
    │      Vulnerability assessment
    │      Security policy compliance
    │      Risk scoring
    │
    ├─→ Security Engineer (Sonnet)
    │      Secure design review
    │      Implementation guidance
    │      Remediation strategy
    │
    ├─→ Penetration Tester (Sonnet)
    │      Security testing
    │      Exploit validation
    │      Attack surface mapping
    │
    └─→ Compliance Specialist (Haiku)
           Regulatory compliance
           Data protection (GDPR, CCPA)
           Standards alignment
```

### Phase 3: Security Auditor Assessment

**Focus areas:**

1. **Authentication & Authorization**
   - Credential handling
   - Token management
   - Session security
   - Permission checking
   - Multi-factor authentication

2. **Input Validation**
   - SQL injection prevention
   - XSS prevention
   - Command injection prevention
   - Path traversal prevention
   - Deserialization attacks

3. **Data Protection**
   - Encryption at rest
   - Encryption in transit (TLS)
   - Sensitive data handling
   - PII protection
   - Data retention policies

4. **Secret Management**
   - Hardcoded credentials
   - Environment variable security
   - Secret rotation
   - Access control
   - Audit logging

5. **Dependency Security**
   - Known vulnerabilities (CVE)
   - Version constraints
   - Transitive dependencies
   - Supply chain security

**Finding Template:**
```
Vulnerability: [Name]
Severity: [Critical/High/Medium/Low]
Type: [Category]
Location: [File:Line or Component]
Description: [What's vulnerable]
Impact: [Business/Technical impact]
Reproduction: [How to verify]
Remediation: [How to fix]
CWE: [Common Weakness ID]
CVSS Score: [If applicable]
Status: [Open/Remediated/Accepted]
```

### Phase 4: Security Engineer Assessment

**Complement auditor findings with:**

1. **Secure Design Review**
   - Follow security best practices
   - Compare to threat model
   - Identify security patterns
   - Validate assumption

2. **Remediation Strategy**
   - Fix severity mapping
   - Implementation guidance
   - Timeline for fixes
   - Fallback options

3. **Implementation Validation**
   - Security code review
   - Encryption validation
   - Access control validation
   - Logging completeness

### Phase 5: Penetration Testing

**Security testing by Penetration Tester:**

1. **Attack Surface Mapping**
   - Identify entry points
   - List all endpoints
   - Find hidden services
   - Map data flows

2. **Exploit Testing**
   - SQL injection attempts
   - XSS payloads
   - Authentication bypass
   - Authorization bypasses
   - Business logic attacks

3. **Load Testing (if applicable)**
   - DoS resistance
   - Rate limiting effectiveness
   - Resource exhaustion
   - Connection limits

### Phase 6: Compliance Verification

**Compliance Specialist checks:**

1. **Data Protection Regulations**
   - GDPR compliance (EU)
   - CCPA compliance (California)
   - HIPAA (health data)
   - PCI-DSS (payment cards)
   - Industry-specific requirements

2. **Security Standards**
   - OWASP Top 10
   - NIST Cybersecurity Framework
   - ISO 27001
   - CIS Benchmarks

3. **Internal Policies**
   - Company security policy
   - Data handling procedures
   - Access control policies
   - Audit logging requirements

### Phase 7: Aggregate Findings

1. **Collect all findings** from all reviewers
2. **Consolidate and deduplicate**
3. **Prioritize by severity**
4. **Calculate risk scores**

### Phase 8: Generate Security Report

**Comprehensive report structure:**

```
SECURITY AUDIT REPORT
====================

Subject: [Feature/Component Name]
Audit Date: [Date]
Auditors: [Names]
Overall Risk Level: [Critical/High/Medium/Low]
Status: [Pass/Conditional/Fail]

EXECUTIVE SUMMARY
=================

Findings Summary:
- Critical: [count]
- High: [count]
- Medium: [count]
- Low: [count]

Critical Findings:
[List of critical issues]

Recommendation:
[Pass/Conditional approval/Fail]

CRITICAL FINDINGS
=================

Finding 1: [Name]
Severity: CRITICAL
Type: [Category]
Location: [Specific location]
Risk: [Business impact]
Remediation: [How to fix]
Status: [Action required immediately]

[Additional critical findings...]

HIGH FINDINGS
=============

[List of high severity findings with details]

MEDIUM FINDINGS
===============

[List of medium findings]

LOW FINDINGS
============

[List of low findings]

COMPLIANCE STATUS
=================

GDPR: ✓ Compliant
CCPA: ✓ Compliant
HIPAA: N/A
PCI-DSS: ✓ Compliant
OWASP Top 10: ✓ Addressed

Non-compliant areas: [if any]

SECURITY TESTING RESULTS
========================

Penetration Testing Summary:
- SQL injection: Not vulnerable
- XSS: Protected by sanitization
- CSRF: Protected by token
- Authentication bypass: Not possible
- Authorization bypass: Not found

Exploit Tests Performed:
[List of tests and results]

REMEDIATION PLAN
================

Critical Issues (Fix Immediately):
1. [Issue 1] → Fix by [date]
2. [Issue 2] → Fix by [date]

High Issues (Fix within 1 week):
1. [Issue 1] → Fix by [date]
2. [Issue 2] → Fix by [date]

Medium Issues (Fix within 1 month):
1. [Issue 1] → Fix by [date]

Low Issues (Backlog):
1. [Issue 1] → Add to backlog

APPROVAL
========

Security Auditor: [Name] - [Approval status]
Penetration Tester: [Name] - [Approval status]
Compliance Specialist: [Name] - [Approval status]
Chief Security Officer: [Name] - [Final approval]

Overall Assessment: [PASS/CONDITIONAL/FAIL]
```

### Phase 9: Store in Knowledge Base

```bash
cco knowledge store "Security Audit: [feature]
Status: [PASS/CONDITIONAL/FAIL]
Critical: [count]
High: [count]
Remediation: [timeline]
Approved: [yes/no]

Auditors: [names]
Key Issues: [list]" \
  --type status \
  --agent security-auditor
```

## Security Finding Categories

### Vulnerability Types
- **CWE-79**: Cross-site Scripting (XSS)
- **CWE-89**: SQL Injection
- **CWE-798**: Hardcoded Credentials
- **CWE-863**: Incorrect Authorization
- **CWE-22**: Path Traversal
- **CWE-352**: Cross-Site Request Forgery (CSRF)
- **CWE-434**: Unrestricted File Upload
- **CWE-502**: Deserialization of Untrusted Data

### Risk Scoring

| Severity | Likelihood | Impact | CVSS | Action |
|----------|-----------|--------|------|--------|
| **Critical** | High | Severe | 9.0-10.0 | Fix immediately, block deployment |
| **High** | High/Medium | Major | 7.0-8.9 | Fix before release, escalate |
| **Medium** | Medium/Low | Moderate | 4.0-6.9 | Fix in next iteration |
| **Low** | Low | Minor | 0.1-3.9 | Backlog, fix if time |

## Best Practices

1. **Secure by design** - Security first, not afterthought
2. **Defense in depth** - Multiple layers of security
3. **Fail secure** - Fail closed, not open
4. **Least privilege** - Minimum necessary access
5. **Assume breach** - Design as if compromised
6. **Input validation** - Never trust user input
7. **Output encoding** - Always encode output
8. **Secrets management** - Never hardcode credentials
9. **Logging & monitoring** - Audit trails for everything
10. **Regular updates** - Keep dependencies current

## Output Format

Return comprehensive security audit report with:
- Executive summary and risk level
- Vulnerability findings by severity
- Compliance status against standards
- Penetration testing results
- Remediation plan with timelines
- Approval status from all auditors
- Critical blocking issues
- Recommendations for hardening
- Knowledge base references
