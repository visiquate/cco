# Codex Review: External Code Review via OpenAI Codex

## Trigger
`/codex-review` or auto-triggered after non-trivial code changes

External code review using OpenAI Codex CLI as an independent reviewer via MCP.

## Purpose
The Codex Review skill provides:
- Independent external code review from a second AI perspective
- Security-focused review pass separate from code quality
- Graceful degradation when Codex is unavailable
- Zero friction — auto-triggers when appropriate, silent when unavailable

## Availability Check

**Re-check before every use:**

1. Call the CCO MCP tool `codex_status` to check binary + auth status
2. If `available: false` — skip external review silently
3. Do NOT mention Codex to the user if unavailable
4. Do NOT suggest installing or configuring Codex
5. Fall back to self-review only (Claude's own code review)

## When to Trigger

**Auto-trigger when ALL of these are true:**
- The current task involved writing or modifying code
- Changes are non-trivial (more than config/docs, more than a one-line fix)
- Codex is available (per `codex_status` check)

**Skip when ANY of these are true:**
- Changes are docs-only or config-only
- Changes are a one-line fix or trivial typo
- User said "skip codex", "no external review", "quick fix", or "just do it"
- This is a planning or research task (no code changes)
- Codex is unavailable

## Workflow

### Step 1: Self-Review (Claude)
Before calling Codex, do a quick self-review:
- Fix any obvious issues (typos, missing imports, logic errors)
- Ensure code compiles and tests pass
- This prevents wasting Codex calls on easily-caught issues

### Step 2: External Code Review (Codex via MCP)
Call the Codex MCP server tool `codex` with a code review prompt:

**Focus areas:**
- Correctness and edge cases
- Maintainability and readability
- Design patterns and idiomatic usage
- Error handling completeness
- API consistency

### Step 3: External Security Review (Codex via MCP)
Separate Codex call focused on security:

**Focus areas:**
- Injection vulnerabilities (SQL, command, XSS)
- Authentication and authorization gaps
- Secret/credential exposure
- Data validation at boundaries
- PII/HIPAA compliance concerns

### Step 4: Synthesize and Fix
1. Collect findings from both Codex passes
2. Deduplicate overlapping findings
3. Categorize by severity: CRITICAL, HIGH, MEDIUM, LOW
4. Fix CRITICAL and HIGH findings immediately
5. Note MEDIUM findings with inline comments
6. Mention LOW findings in the summary only

## Error Handling

- Any MCP call failure — skip that step, do not retry, do not block
- Timeout (>5 minutes) — skip, note "external review timed out"
- Authentication expired mid-session — skip remaining Codex calls
- Brief one-line message only, never a stack trace or error dump

## User Control

| User says | Action |
|-----------|--------|
| "skip codex" / "no external review" | Skip for this task |
| "always skip codex" | Suggest removing Codex from MCP config |
| "run codex review" / "get a second opinion" | Force even on small changes |
| "codex security only" | Skip code review pass, do security only |
| "codex review only" | Skip security pass, do code review only |

## Output Format

When Codex review runs, append to the task summary:

```
EXTERNAL REVIEW (Codex)
=======================
Code Review: [N findings]
Security Review: [N findings]

[Findings listed by severity, with file:line references]

Actions Taken:
- Fixed: [list of auto-fixed items]
- Noted: [list of items noted but not auto-fixed]
```

When Codex is unavailable, say nothing about it. Just proceed with self-review.
