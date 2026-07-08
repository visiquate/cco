# Remember: Store Context

## Trigger
`/remember <context or decision>`

Stores decisions, context, and facts in the knowledge base for future reference and cross-agent communication.

## Purpose
The Remember skill provides:
- Persistent decision storage for consistency
- Context preservation across sessions
- Cross-agent information sharing
- Searchable knowledge base
- Audit trail of decisions and lessons

## Workflow

### 1. Identify What to Remember
**Categories of information:**
- Architectural Decisions: Technology choices, design patterns, API design
- Business Decisions: Requirements, prioritization, scope boundaries
- Implementation Patterns: Code patterns, testing strategies, optimizations
- Constraints & Requirements: Performance, security, scalability
- Lessons Learned: What worked, what didn't, pitfalls
- Critical Context: Why decisions were made, dependencies
- Facts: Important metrics, baselines, resource locations

### 2. Classify and Store

**Using `knowledge_store()` MCP tool:**
```
knowledge_store(
  text="DECISION: JWT Token Strategy - HS256 algorithm, 24h expiration, env-var secret key...",
  knowledge_type="decision",
  agent="chief-architect"
)
```

**Valid knowledge types:**
- `decision` — Architectural or strategic choice
- `edit` — Code or documentation change
- `status` — Current work state
- `completion` — Work completed milestone
- `architecture` — System design detail
- `test` — Testing approach
- `bug` — Issue encountered
- `optimization` — Performance improvement

### 3. Link Related Items

Include cross-references in stored content:
```
knowledge_store(
  text="REQUIREMENT: API Security Policy - Validate JWT tokens, check permissions, log access...",
  knowledge_type="requirement",
  agent="backend-architect"
)
```

## Example

**Store an architectural decision:**
```
knowledge_store(
  text="JWT Authentication Strategy:\n" +
       "- Algorithm: HS256\n" +
       "- Expiration: 24 hours\n" +
       "- Secret: environment variable\n" +
       "- Validation: signature + expiration\n" +
       "Rationale: Stateless, industry standard, simple\n" +
       "Decided by: Chief Architect + Security Auditor",
  knowledge_type="decision",
  agent="chief-architect"
)
```

## Best Practices

1. **Be Specific** — Include exact details, not vague statements
2. **Include Rationale** — Why was this decided?
3. **Link Related Items** — Cross-reference other entries
4. **Date Context** — When and who made the decision
5. **Make it Actionable** — How should someone use this info?

## Output Format

Return confirmation with:
- Entry stored successfully
- Knowledge type assigned
- Searchable keywords identified
- Related entries found
- Cross-references established
