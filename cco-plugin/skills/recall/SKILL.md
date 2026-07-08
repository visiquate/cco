# Recall: Retrieve Context

## Trigger
`/recall <query or topic>`

Searches knowledge base to retrieve stored decisions, context, and facts for the current task.

## Purpose
The Recall skill provides:
- Intelligent knowledge base queries with multi-hop retrieval
- Decision retrieval for consistency
- Context lookup across agent work
- Cross-referencing related entries
- Pattern discovery and dependency identification

## Workflow

### 1. Formulate Query
**Query types:**
- Decision Lookup: "What was decided about JWT tokens?"
- Requirement Check: "Performance targets for API?"
- Pattern Discovery: "Error handling pattern?"
- Context Understanding: "Why did we choose JWT over sessions?"
- Dependency Mapping: "What depends on JWT implementation?"

### 2. Execute Knowledge Search

**Primary tool: `knowledge_recall()`**
```
knowledge_recall(query="JWT token strategy", limit=5, max_hops=2)
```
Performs hybrid search with multi-hop retrieval across insights and chunks.

**Alternative searches:**
- `knowledge_search(query="decision JWT", limit=10)` — filtered search by type/term
- `chunk_search(query="JWT validation", limit=5)` — search conversation history
- `knowledge_graph_query(action="get_neighbors", node_id="jwt-decision")` — explore relationships

### 3. Analyze Retrieved Information
- Review primary decision/requirement
- Follow cross-references to related entries
- Extract constraints and dependencies
- Verify status and authority

### 4. Apply Context to Task
- Implement per architecture decision
- Follow established patterns
- Test per requirements
- Document per specification

## Example

**Query:** "JWT authentication strategy"

**Tools:**
1. `knowledge_recall(query="JWT authentication strategy", max_hops=2)`
2. Explore relationships: `knowledge_graph_query(action="get_neighbors", name="JWT")`
3. Filter for patterns: `knowledge_search(query="pattern JWT validation")`

**Result:** Consolidated view of:
- JWT decision (algorithm, expiration, rationale)
- Related decisions (refresh tokens, security policy)
- Patterns (validation approach, error handling)
- Requirements (performance, security constraints)

## Best Practices

1. Start with `knowledge_recall()` for comprehensive context
2. Use `knowledge_search()` for specific types or constraints
3. Check `knowledge_graph_query()` to explore dependencies
4. Verify decision authority and current status
5. Look for rationale — understanding why matters
6. Cross-reference related items for complete picture

## Output Format

Return comprehensive recall with:
- Matched knowledge entries ranked by relevance
- Primary decision or requirement
- Related cross-references
- Implementation guidance
- Constraints and dependencies
- Actionable next steps
